

#![feature(link_args)]
#![feature(custom_derive, plugin)]
#![feature(question_mark)]
#![feature(integer_atomics)]

#![allow(dead_code)]



//!
//! Bitcrust main documentation
//! Provides access to the bitcrust database
/// let db = Db::new();
///
/// accessing info
/// db.peers.list();
/// db.blocks.find(312);
/// db.blocks.list();
/// db.addresses.list();
/// db.transactions.list();
/// 
/// insert/update a block
/// let block = db.blocks.get();
/// block.merkle_root = "hello";
/// db.blocks.insert(block);

/// Add a transaction to the mempool
/// let tx = transaction::new();
/// db.transactions.insert(tx);


extern crate memmap;

extern crate lmdb_rs;
extern crate itertools;
extern crate rand;

extern crate ring;

mod ffi;
mod metrics;

mod buffer;
use buffer::*;


pub use block::Block;

mod hash;

pub mod transaction;
pub mod block;
pub mod script;

mod store;

mod config;
use store::Store;



pub fn init() -> Store {

    let config = config::Config::new_test();
    Store::new(&config)
}

/// Validates and stores a block;
///
/// Currently used to collect what needs to be done;
/// TODO: distibute over different mods
pub fn add_block(store: &mut store::Store, buffer: &[u8]) {

    let block = Block::new(buffer).unwrap();

    let block_hash = hash::Hash32Buf::double_sha256(block.header.to_raw());

    let mut total_amount = 0_u64;


    block.process_transactions(|tx| {

        let hash = hash::Hash32Buf::double_sha256(tx.to_raw());

        total_amount += 1;

        let res = tx.verify_and_store(store);
        //if !tx.is_coinbase() || res.is_err() {
          //  println!("res={:?} cb={} TX={:?} ", res, tx.is_coinbase(), hash);
        //}

        Ok(())

    }).unwrap();



}

pub fn add_transaction(_: &[u8]) {

}

pub mod test {
    pub fn xx() {

    }
}
