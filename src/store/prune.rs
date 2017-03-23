

//! Preliminairy pruning of tx-index
//! This is prbably less needed when using HAMT


use std::env;
use std::path;
use std::fs;

use hash::*;
use buffer::*;
use store::Store;
use config::Config;
use store::TxPtr;
use store::hash_index;
use transaction::Transaction;

use store::Record;

#[ignore]
#[test]
fn prune_tx_index() {

    // assume last arg is path
    let p =  &env::args().last().unwrap();
    if fs::metadata(p).is_err() {
        panic!("Path not found {}", p);
    }

    // open data store
    println!("Pruning {}", p);
    let cfg   = Config::new(p);
    let mut store = Store::new(&cfg);

    // add pruned txindex
    let mut path = path::PathBuf::from(p);
    path.push("tx-index-pruned");
    let _ =  fs::remove_dir_all(path);
    let mut new_tx_index: hash_index::HashIndex<TxPtr>
        = hash_index::HashIndex::new(&cfg, "tx-index-pruned");

    let mut tx_ptr = TxPtr::first();
    let mut count = 0;
    let mut count_purged = 0;
    loop {
        let tx_raw = store.transactions.read(tx_ptr);

        if tx_raw.len() == 0 {
            break;
        }

        // we parse it only for the output_count
        let tx = Transaction::parse(&mut Buffer::new(tx_raw)).unwrap();

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
            println!("Done: {}; purged {} %",count, count_purged as u64 * 100 / count as u64);
        }

        tx_ptr = tx_ptr.offset(tx_raw.len() as u32 + 4);
    }

    println!("Done");
    println!("  {} transactions", count);
    println!("  {} purged ({} %)", count_purged , count_purged as u64 * 100 / count as u64);
    println!("  {} remain", count - count_purged ,);

}