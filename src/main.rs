pub mod mempool;
pub mod oddsketch;

use std::collections::HashSet;

use arrayref::array_ref;
use bitcoin::{util::psbt::serialize::Deserialize, Script, Transaction, TxIn};
use log::info;
use rand::prelude::*;

use crate::mempool::Mempool;

const TX_PADDING: usize = 256;
const N_SHARED: usize = 100_000;
const N_ALICE: usize = 33;
const N_BOB: usize = 3;

fn generate_random_tx() -> Transaction {
    // Scripts for padding to normal tx size
    let sig_script: [u8; TX_PADDING] = [0; TX_PADDING];

    // Create input
    let input = TxIn {
        previous_output: Default::default(),
        script_sig: Script::deserialize(&sig_script[..]).unwrap(),
        sequence: thread_rng().gen(),
        witness: vec![],
    };

    Transaction {
        version: 2,
        lock_time: 0,
        input: vec![input],
        output: vec![],
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "INFO");
    pretty_env_logger::init_timed();

    info!("generating {} shared transactions...", N_SHARED);
    // Generate shared transactions
    let alice_shared_txs: Vec<Transaction> = (0..N_SHARED).map(|_| generate_random_tx()).collect();
    let bob_shared_txs = alice_shared_txs.clone();

    // Generate private transactions
    info!("generating {} private transactions...", N_ALICE + N_BOB);
    let alice_txs: Vec<Transaction> = (0..N_ALICE).map(|_| generate_random_tx()).collect();
    let bob_txs: Vec<Transaction> = (0..N_BOB).map(|_| generate_random_tx()).collect();

    // Mempools
    let mut alice_mempool = Mempool::default();
    let mut bob_mempool = Mempool::default();

    // Populate
    info!("inserting into Alices mempool...");
    alice_mempool.insert_batch(alice_shared_txs);
    alice_mempool.insert_batch(alice_txs);

    info!("inserting into Bobs mempool...");
    bob_mempool.insert_batch(bob_txs);
    bob_mempool.insert_batch(bob_shared_txs);

    // Calculate estimated difference
    info!("calculating difference...");
    let estimate = (alice_mempool.oddsketch() ^ bob_mempool.oddsketch()).size();
    info!("estimated difference: {}", estimate);

    // Decode minisketches
    let padded_estimate = estimate + 4;
    let alice_sketch = alice_mempool.minisketch(); // Sliced
    let mut bob_sketch = bob_mempool.minisketch_slice(padded_estimate as usize);

    info!("merging sketch...");
    bob_sketch.merge(&alice_sketch).unwrap();

    let mut decoded_ids = [0u64; 512]; // Overestimation here
    info!("decoding sketch...");
    bob_sketch.decode(&mut decoded_ids).unwrap();

    // Calculate real difference
    info!("calculating actual symmetric difference...");
    let real_sym_diff: HashSet<u64> = alice_mempool
        .tx_set()
        .symmetric_difference(&bob_mempool.tx_set())
        .map(|tx| {
            let tx_id = tx.txid();
            u64::from_be_bytes(*array_ref![tx_id, 0, 8])
        })
        .collect();

    // Filter out overestimation
    let predicted_sym_diff: HashSet<u64> = decoded_ids
        .iter()
        .filter_map(|x| match x {
            0 => None,
            x => Some(*x),
        })
        .collect();

    // Difference between predicted and real
    info!("comparing actual symmetric difference with estimation...");
    let discrepency: HashSet<u64> = real_sym_diff
        .symmetric_difference(&predicted_sym_diff)
        .cloned()
        .collect();

    info!("discrepency: {:?}", discrepency);
}
