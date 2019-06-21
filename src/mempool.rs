use arrayref::array_ref;
use bitcoin::Transaction;
use minisketch_rs::Minisketch;
use std::collections::{HashMap, HashSet};

use oddsketch::Oddsketch;

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

        // Insert into mempool
        self.txs.insert(short_id, tx);

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

    pub fn tx_set(&self) -> HashSet<Transaction> {
        self.txs.values().cloned().collect()
    }

    pub fn insert_batch(&mut self, txs: Vec<Transaction>) {
        for tx in txs {
            self.insert(tx);
        }
    }
}
