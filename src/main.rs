#[macro_use]
extern crate clap;

pub mod json_rpc_client;
pub mod logging;
pub mod mempool;
pub mod messages;

use std::{
    io::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use bitcoin::{
    util::psbt::serialize::{Deserialize, Serialize},
    Transaction,
};
use clap::App;
use futures::{future, future::ok, lazy, sync::mpsc, Future, Stream};
use futures_zmq::{prelude::*, Sub};
use itertools::Itertools;
use log::{error, info, warn};
use mempool::Mempool;
use serde_json::json;
use tokio::{
    codec::Framed,
    net::{TcpListener, TcpStream},
    prelude::*,
    timer::Interval,
};

use crate::{
    json_rpc_client::JsonClient,
    messages::{Message, MessageCodec},
};

fn main() {
    // Load values from CLI
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let node_ip = matches.value_of("nodeip").unwrap_or("127.0.0.1");

    // Logging
    std::env::set_var("RUST_LOG", "mempool_sync_gadget=INFO");
    if matches.is_present("filelog") {
        logging::init_file_logging();
    } else {
        logging::init_console_logging();
    }

    info!("starting...");

    // New peer stream
    let (peer_send, peer_recv) = mpsc::unbounded::<TcpStream>();

    // Add peer
    let peer_opt = match matches.value_of("peerip") {
        Some(peer_ip) => {
            let port: u16 = matches
                .value_of("peerport")
                .map(|hb| hb.parse().unwrap_or(8885))
                .unwrap_or(8885);
            Some(
                TcpStream::connect(&format!("{}:{}", peer_ip, port).parse().unwrap())
                    .map_err(|e| error!("{}", e)),
            )
        }
        None => None,
    };
    let is_client = peer_opt.is_some();

    // Heartbeat duration
    let hb_duration = Duration::from_millis(
        matches
            .value_of("heartbeat")
            .map(|hb| hb.parse().unwrap_or(2000))
            .unwrap_or(2000),
    );

    // Overestimation of minisketch size
    let padding = matches
        .value_of("padding")
        .map(|padding| padding.parse().unwrap_or(3))
        .unwrap_or(3);

    // Bitcoin client
    let json_client = Arc::new(JsonClient::new(
        format!("http://{}:8332", node_ip),
        matches.value_of("rpcusername").unwrap_or("").to_string(),
        matches.value_of("rpcpassword").unwrap_or("").to_string(),
    ));

    // Mempool
    let mempool_shared = Arc::new(Mutex::new(Mempool::default()));

    // ZeroMQ
    let context = Arc::new(zmq::Context::new());

    // Transaction subscription
    let mempool_shared_inner = mempool_shared.clone();
    let json_client_inner = json_client.clone();
    let sub = Sub::builder(context.clone())
        .connect(&format!("tcp://{}:28332", node_ip))
        .filter(b"")
        .build();
    let runner = sub
        .map_err(|e| {
            error!("zmq subscriptions error = {}", e);
        })
        .and_then(move |sub| {
            // For each message received via ZMQ
            sub.stream()
                .map_err(|e| {
                    error!("zmq stream error = {}", e);
                })
                .for_each(move |multipart| {
                    match &**multipart.get(0).unwrap() {
                        b"rawtx" => {
                            // Add new transaction to gadget mempool
                            let tx_raw: &[u8] = &multipart.get(1).unwrap();
                            let new_tx = Transaction::deserialize(tx_raw).unwrap();
                            info!("new tx {} from zmq", new_tx.txid());
                            mempool_shared_inner.lock().unwrap().insert(new_tx);
                            future::Either::A(ok(()))
                        }
                        b"hashblock" => {
                            // Reset gadget mempool
                            info!("new block announcement from zmq");

                            // Get tx ids from node mempool
                            let mempool_shared_inner = mempool_shared_inner.clone();
                            let json_client_inner = json_client_inner.clone();

                            // TODO: Perhaps add a delay here while node mempool updating?
                            future::Either::B(mempool::populate_via_rpc(
                                json_client_inner,
                                mempool_shared_inner,
                            ))
                        }
                        _ => {
                            error!("unexpected zmq message");
                            unreachable!()
                        }
                    }
                })
        })
        .map(|_| ());

    // Server
    let mempool_shared_inner = mempool_shared.clone();
    let json_client_inner = json_client.clone();
    let server = TcpListener::bind(&"0.0.0.0:8885".parse().unwrap())
        .unwrap()
        .incoming()
        .map_err(|e| error!("{}", e))
        .select(peer_recv)
        .for_each(move |socket| {
            let peer_addr = socket.peer_addr().unwrap();
            info!("new peer {}", peer_addr);

            // Channel for excess txs
            let (excess_tx_send, excess_tx_recv) = mpsc::unbounded::<Vec<Transaction>>();

            // Frame socket
            let framed_sock = Framed::new(socket, MessageCodec);
            let (send_stream, received_stream) = framed_sock.split();

            // Inner variables
            let json_client_inner = json_client_inner.clone();
            let mempool_shared_inner = mempool_shared_inner.clone();
            let mempool_shared_outer = mempool_shared_inner.clone();

            // Response stream
            let responses = received_stream.filter_map(move |msg| {
                match msg {
                    Message::Minisketch(mut peer_minisketch) => {
                        info!("received minisketch from {}", peer_addr);
                        let mempool_guard = mempool_shared_inner.lock().unwrap();

                        // Merge minisketches
                        let minisketch = mempool_guard.minisketch();
                        peer_minisketch.merge(&minisketch).unwrap();

                        // Decode minisketch
                        let mut decoded_ids = [0u64; 512];
                        if peer_minisketch.decode(&mut decoded_ids).is_err() {
                            warn!("minisketch decoding failed");
                            return None;
                        }

                        // Find excess transaction and missing IDs
                        let (excess, missing): (Vec<Transaction>, Vec<u64>) = decoded_ids
                            .iter()
                            .filter(|id| **id != 0) // Remove excess
                            .partition_map(|id| match mempool_guard.txs().get(id) {
                                Some(tx) => itertools::Either::Left(tx.clone()),
                                None => itertools::Either::Right(*id),
                            });
                        drop(mempool_guard);

                        info!("{} excess txs, {} missing ids", excess.len(), missing.len());

                        if !excess.is_empty() {
                            tokio::spawn(
                                excess_tx_send
                                    .clone()
                                    .send(excess)
                                    .map_err(|_| ())
                                    .and_then(|_| ok(())),
                            );
                        }

                        if missing.is_empty() {
                            None
                        } else {
                            info!("sending gettxs to {}", peer_addr);
                            Some(Message::GetTxs(missing))
                        }
                    }
                    Message::Oddsketch(peer_oddsketch) => {
                        info!("received oddsketch from {}", peer_addr);

                        // Xor oddsketches
                        let mempool_guard = mempool_shared_inner.lock().unwrap();
                        let oddsketch = mempool_guard.oddsketch();

                        // Esimtate symmetric difference
                        let estimated_size = (oddsketch ^ peer_oddsketch).size();
                        info!("estimated difference {}", estimated_size);

                        if estimated_size == 0 {
                            // If est. diff. 0 then don't send
                            return None;
                        }

                        // Slice minisketch to that length
                        let out_minisketch =
                            mempool_guard.minisketch_slice(estimated_size as usize + padding);
                        drop(mempool_guard);

                        info!("sending minisketch to {}", peer_addr);
                        Some(Message::Minisketch(out_minisketch))
                    }
                    Message::GetTxs(vec_ids) => {
                        info!(
                            "received {} transaction requests from {}",
                            vec_ids.len(),
                            peer_addr
                        );

                        // Get txs from mempool
                        let mempool_guard = mempool_shared_inner.lock().unwrap();
                        let txs = vec_ids
                            .iter()
                            .filter_map(|id| mempool_guard.txs().get(id))
                            .cloned()
                            .collect();
                        drop(mempool_guard);

                        info!("sending txs to {}", peer_addr);
                        Some(Message::Txs(txs))
                    }
                    Message::Txs(vec_txs) => {
                        info!("received {} transactions {}", vec_txs.len(), peer_addr);

                        // Add txs to mempool (and node mempool)
                        let mut mempool_guard = mempool_shared_inner.lock().unwrap();

                        for tx in vec_txs {
                            let raw = hex::encode(tx.serialize());
                            mempool_guard.insert(tx);

                            let req = json_client_inner
                                .build_request("sendrawtransaction".to_string(), vec![json!(raw)]);
                            tokio::spawn(
                                json_client_inner
                                    .send_request(&req)
                                    .and_then(|resp| resp.result::<String>())
                                    .map(|tx_id| info!("added {} to mempool", tx_id))
                                    .map_err(|e| error!("{:?}", e)),
                            );
                        }
                        None
                    }
                }
            });

            // Heartbeat
            let mempool_shared_inner = mempool_shared_outer.clone();
            let interval = Interval::new_interval(hb_duration).map_err(|e| {
                error!("{}", e);
                Error::from_raw_os_error(0)
            });
            let heartbeat = interval.filter_map(move |_| {
                if is_client {
                    // If client then send oddsketch every heartbeat
                    info!("sending heartbeat oddsketch to {}", peer_addr);
                    let mempool_guard = mempool_shared_inner.lock().unwrap();
                    let oddsketch = mempool_guard.oddsketch();
                    drop(mempool_guard);

                    Some(Message::Oddsketch(oddsketch))
                } else {
                    None
                }
            });

            // Merge adjoin heartbeat and excess tx stream
            let excess_tx_recv = excess_tx_recv
                .map(Message::Txs)
                .map_err(|_| Error::from_raw_os_error(0));
            let out = responses.select(heartbeat).select(excess_tx_recv);

            // Send
            let send = send_stream.send_all(out).map(|_| ()).or_else(move |e| {
                error!("{}", e);
                Ok(())
            });
            tokio::spawn(send)
        });

    // Spawn event loop
    tokio::run(lazy(move || {
        // Populate mempool
        info!("populating gadget mempool via rpc...");
        mempool::populate_via_rpc(json_client, mempool_shared).and_then(|_| {
            // Spawn ZMQ runner
            tokio::spawn(runner);

            // Spawn gadget server
            tokio::spawn(server);
            tokio::spawn(match peer_opt {
                Some(peer) => future::Either::A(
                    peer.and_then(|socket| peer_send.send(socket).map_err(|e| error!("{}", e)))
                        .and_then(|_| ok(())),
                ),
                None => future::Either::B(ok(())),
            });
            ok(())
        })
    }));
}
