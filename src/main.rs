pub mod mempool;
pub mod oddsketch;

use std::collections::HashSet;

use bitcoin::{Transaction, TxIn, Script, util::psbt::serialize::Deserialize};
use rand::prelude::*;
use log::info;
use crate::mempool::Mempool;

const TX_PADDING: usize = 256;
const N_SHARED: usize = 40954;
const N_ALICE: usize = 34;
const N_BOB: usize = 6;


fn generate_random_tx() -> Transaction {
    // Scripts for padding
    let sig_script: [u8; TX_PADDING] = [0; TX_PADDING];

    // Create input
    let input = TxIn {
        previous_output: Default::default(),
        script_sig: Script::deserialize(&sig_script[..]).unwrap(),
        sequence: thread_rng().gen(),
        witness: vec![]
    };

    Transaction {
        version: 2,
        lock_time: 0,
        input: vec![input],
        output: vec![]
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "INFO");
    env_logger::init();

    info!("generating {} shared transactions...", N_SHARED);
    // Generate shared transactions
    let shared_txs: Vec<Transaction> = (0..N_SHARED).map(|_| generate_random_tx()).collect();

    // Generate private transactions
    info!("generating {} private transactions...", N_ALICE + N_BOB);
    let alice_txs: Vec<Transaction> = (0..N_ALICE).map(|_| generate_random_tx()).collect();
    let bob_txs: Vec<Transaction> = (0..N_BOB).map(|_| generate_random_tx()).collect();

    // Mempools
    let mut alice_mempool = Mempool::default();
    let mut bob_mempool = Mempool::default();

    // Populate
    info!("inserting into mempool...");
    alice_mempool.insert_batch(shared_txs.clone());
    bob_mempool.insert_batch(shared_txs);
    alice_mempool.insert_batch(alice_txs);
    bob_mempool.insert_batch(bob_txs);
    
    // Calculate estimated difference
    info!("calculating difference...");
    let estimate = (alice_mempool.oddsketch() ^ bob_mempool.oddsketch()).size();
    info!("estimated difference: {}", estimate);

    // Calculate real difference
    info!("calculating real difference...");
    let sym_diff: HashSet<Transaction> = alice_mempool.tx_set().symmetric_difference(&bob_mempool.tx_set()).cloned().collect();
    info!("real difference: {}", sym_diff.len());
}
