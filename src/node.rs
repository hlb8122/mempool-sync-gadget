// use minisketch_rs::{Minisketch, MinisketchError};
use std::{collections::HashMap, sync::mpsc::{Sender, Receiver}};
use bitcoin::Transaction;
use arrayref::array_ref;

use crate::oddsketch::Oddsketch;


struct Mempool {
    // minisketch: Minisketch,
    oddsketch: Oddsketch,
    txs: HashMap<u64, Transaction>
}

impl Mempool {
    fn insert(&mut self, tx: Transaction) {
        let tx_id = tx.txid();
        let short_id = u64::from_be_bytes(*array_ref![tx_id, 0, 8]);

        // Insert into mempool        
        self.txs.insert(short_id, tx);

        // Update Oddsketch
        self.oddsketch.insert(short_id);

        // Update Minisketch
        // TODO
    }

    fn insert_batch(&mut self, txs: Vec<Transaction>) {
        for tx in txs {
            self.insert(tx);
        }
    }
}





struct Node {
    mempool: Mempool,
}

