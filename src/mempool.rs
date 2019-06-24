use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use arrayref::array_ref;
use bitcoin::{util::psbt::serialize::Deserialize, Transaction};
use futures::{future::ok, stream, stream::Stream, Future};
use log::{error, info};
use minisketch_rs::Minisketch;
use oddsketch::Oddsketch;
use serde_json::json;

use crate::json_rpc_client::JsonClient;

pub struct Mempool {
    minisketch: Minisketch,
    oddsketch: Oddsketch,
    txs: HashMap<u64, Transaction>,
}

impl Default for Mempool {
    fn default() -> Self {
        let minisketch = Minisketch::try_new(64, 0, 256).unwrap();
        Mempool {
            minisketch,
            oddsketch: Default::default(),
            txs: HashMap::with_capacity(128),
        }
    }
}

impl Mempool {
    pub fn insert(&mut self, tx: Transaction) {
        let tx_id = tx.txid();
        let short_id = u64::from_be_bytes(*array_ref![tx_id, 0, 8]);

        if self.txs.contains_key(&short_id) {
            return;
        }

        // Insert into mempool
        self.txs.insert(short_id, tx);

        // Update Minisketch
        self.minisketch.add(short_id);

        // Update Oddsketch
        self.oddsketch.insert(short_id);
    }

    pub fn remove(&mut self, tx_id: Vec<u8>) {
        let short_id = u64::from_be_bytes(*array_ref![tx_id, 0, 8]);

        // Remove from mempool
        self.txs.remove(&short_id);

        // Update Minisketch
        self.minisketch.add(short_id);

        // Update Oddsketch
        self.oddsketch.insert(short_id);
    }

    pub fn txs(&self) -> &HashMap<u64, Transaction> {
        &self.txs
    }

    pub fn oddsketch(&self) -> Oddsketch {
        self.oddsketch.clone()
    }

    pub fn minisketch(&self) -> Minisketch {
        self.minisketch.clone()
    }

    pub fn minisketch_slice(&self, capacity: usize) -> Minisketch {
        let ssize = self.minisketch.serialized_size();
        let mut buf = vec![0u8; ssize];

        self.minisketch.serialize(&mut buf).unwrap();

        let mut new_minisketch = Minisketch::try_new(64, 0, capacity).unwrap();
        new_minisketch.deserialize(&buf[..capacity]);
        new_minisketch
    }

    pub fn insert_batch(&mut self, txs: Vec<Transaction>) {
        for tx in txs {
            self.insert(tx);
        }
    }
}

pub fn populate_via_rpc(
    json_client: Arc<JsonClient>,
    mempool: Arc<Mutex<Mempool>>,
) -> impl Future<Item = (), Error = ()> {
    let req = json_client.build_request("getrawmempool".to_string(), vec![json!(false)]);
    json_client
        .send_request(&req)
        .and_then(|resp| resp.result::<Vec<String>>())
        .and_then(move |tx_ids| {
            // Get txs from tx ids
            let txs_fut =
                stream::unfold(tx_ids.into_iter(), move |mut tx_ids| match tx_ids.next() {
                    Some(tx_id) => {
                        info!("fetching {} from rpc", tx_id);
                        let tx_req = json_client.build_request(
                            "getrawtransaction".to_string(),
                            vec![json!(tx_id), json!(false)],
                        );
                        let raw_tx_fut = json_client
                            .send_request(&tx_req)
                            .and_then(|resp| resp.result::<String>());
                        Some(raw_tx_fut.map(move |raw_tx| (raw_tx, tx_ids)))
                    }
                    None => None,
                })
                .collect();

            // Reset then add txs to gadget mempool
            let mempool_shared_inner = mempool.clone();
            txs_fut.and_then(move |raw_txs| {
                let mut mempool_guard = mempool_shared_inner.lock().unwrap();
                *mempool_guard = Mempool::default();

                raw_txs.iter().for_each(|raw_tx| {
                    mempool_guard
                        .insert(Transaction::deserialize(&hex::decode(raw_tx).unwrap()).unwrap());
                });
                drop(mempool_guard);
                ok(())
            })
        })
        .map_err(|e| error!("{:?}", e))
}
