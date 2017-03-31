

//! Preliminairy pruning of tx-index
//! This is probably less needed when using HAMT

// this is needed because of #[ignore]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::env;
use std::fs;
use std::path;

use hash::*;
use buffer::*;
use config::Config;
use store::hash_index;
use store::{Store,TxPtr,Record};
use transaction::Transaction;


/// Prunes the tx-index of the store to a separate tx-index-pruned folder
fn prune_to_new_index() {

    let cfg = test_cfg!();
    let mut store = Store::new(&cfg);


    let mut new_tx_index: hash_index::HashIndex<TxPtr>
        = hash_index::HashIndex::new(&cfg, "tx-index-pruned");

    let mut tx_ptr = TxPtr::first();
    let mut count: u64 = 0;
    let mut count_purged: u64 = 0;
    loop {
        let (tx_raw, next_ptr) = store.transactions.next(tx_ptr);

        if tx_raw.len() == 0 {
            break;
        }

        // we parse it only for the output_count
        let tx = Transaction::parse(&mut Buffer::new(tx_raw.as_slice())).unwrap();

        let input_count = tx.txs_out.len() as u32;

        let tx_ptr_copy = tx_ptr;
        let spend_outputs = (0..input_count)
            .map(|n|   Record::new_output(tx_ptr_copy, n))
            .map(|rec| rec.hash())
            .filter(|hash| store.spend_index.exists(*hash))
            .count() as u32;

        if spend_outputs  < input_count {

            // we still need this one
            let hash = Hash32Buf::double_sha256(tx.to_raw());

            assert_eq!(store.tx_index.get(hash.as_ref()).len(),1);

            new_tx_index.set(hash.as_ref(), tx_ptr, &[], true);
        }
            else {
                // all inputs are spend; don't add it to the new-index
                count_purged += 1;
            }


        count = count + 1;
        if count % 1000 == 0 {
            println!("Done: {}; purged {} %", count, count_purged as u64 * 100 / count as u64);
        }

        tx_ptr = next_ptr;
    }

    println!("Done");
    println!("  {} transactions", count);
    println!("  {} purged ({} %)", count_purged , count_purged as u64 * 100 / count as u64);
    println!("  {} remain", count - count_purged ,);

}

#[ignore]
#[test]
fn prune_tx_index() {


    let store_path = env::var(::config::ENV_BITCRUST_STORE).
        expect(&format!("Use {} env var to specify a store to prune", ::config::ENV_BITCRUST_STORE));

    // add pruned tx-index to store_path
    let pruned_path   = path::PathBuf::from(&store_path).join("tx-index-pruned");
    let tx_index_path = path::PathBuf::from(&store_path).join("tx-index");
    let _ =  fs::remove_dir_all(&pruned_path); // if exists

    prune_to_new_index();

    // move the new index into position
    fs::remove_dir_all(&tx_index_path).expect("Couldn't remove old tx-index");
    fs::rename(pruned_path, &tx_index_path).expect("Failed to move tx-index after pruning");

}