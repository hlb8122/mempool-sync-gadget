// use minisketch_rs::{Minisketch, MinisketchError};
use std::{collections::{HashMap, HashSet}};
use bitcoin::Transaction;
use arrayref::array_ref;

use crate::oddsketch::Oddsketch;

#[derive(Default)]
pub struct Mempool {
    // minisketch: Minisketch,
    oddsketch: Oddsketch,
    txs: HashMap<u64, Transaction>
}

impl Mempool {
    pub fn insert(&mut self, tx: Transaction) {
        let tx_id = tx.txid();
        let short_id = u64::from_be_bytes(*array_ref![tx_id, 0, 8]);

        // Insert into mempool        
        self.txs.insert(short_id, tx);

        // Update Oddsketch
        self.oddsketch.insert(short_id);

        // Update Minisketch
        // TODO
    }

    pub fn oddsketch(&self) -> Oddsketch {
        self.oddsketch.clone()
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
