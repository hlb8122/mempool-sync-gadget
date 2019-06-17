pub mod mempool;

use std::sync::{Arc, Mutex};

use mempool::Mempool;
use log::{info, error};
use futures_zmq::{prelude::*, Sub};
use futures::{Future, Stream, lazy, future::ok};
use bitcoin::{Transaction, util::psbt::serialize::Deserialize};

fn main() {
    // Logging
    std::env::set_var("RUST_LOG", "INFO");
    pretty_env_logger::init_timed();

    info!("starting...");

    // Mempool
    let mempool_shared = Arc::new(Mutex::new(Mempool::default()));

    // ZeroMQ
    // Transaction subscription
    let context = Arc::new(zmq::Context::new());
    let mempool_shared_inner = mempool_shared.clone();
    let tx_sub = Sub::builder(context.clone())
        .connect("tcp:///127.0.0.1:28332")
        .filter("rawtx".as_bytes())
        .build();
    let tx_runner = tx_sub.and_then(move |tx_sub| {
        tx_sub.stream()
            .for_each(move |multipart| {
                // Add new transactions to mempool
                let tx_raw: &[u8] = &multipart.get(0).unwrap();
                info!("new tx");
                let new_tx = Transaction::deserialize(tx_raw).unwrap();
                mempool_shared_inner.lock().unwrap().insert(new_tx);
                ok(())
            })
    }).map(|_| ()).map_err(|e| {
        error!("tx subscription error = {}", e);
    });

    // Block subscription
    let block_sub = Sub::builder(context.clone())
        .connect("tcp:///127.0.0.1:28332")
        .filter("hashblock".as_bytes())
        .build();
    let block_runner = block_sub.and_then(move |block_sub| {
        block_sub.stream()
            .for_each(move |multipart| {
                let block_hash: &[u8] = &multipart.get(0).unwrap();
                info!("new block = {:?}", block_hash);
                
                // Reset mempool
                *mempool_shared.lock().unwrap() = Mempool::default();
                
                // TODO: Repopulate

                ok(())
            })
    }).map(|_| ()).map_err(|e| {
        error!("block subscription error = {}", e);
    });

    tokio::run(lazy(|| {
        tokio::spawn(tx_runner);
        tokio::spawn(block_runner);
        ok(())
    }));
}