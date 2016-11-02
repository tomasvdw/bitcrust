

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

use std::sync;

mod ffi;

mod decode;
//mod encode;


pub use block::Block;

mod hash;

pub mod transaction;
pub mod block;
pub mod script;

mod store;

mod config;
use store::Store;

use decode::*;

pub fn init() -> Store {
    let config = config::Config::new_test();
    Store::new(&config)
}

pub fn add_block(store: &mut store::Store, buffer: &[u8]) {

    let block = Block::new(buffer).unwrap();
    //let store = sync::Mutex::new(store);

    println!("{:?}", block);

    block.process_transactions(|tx| {

        println!("{:?}", tx);

        store.file_transactions.write(tx.to_raw());

        Ok(())

    }).unwrap();


}

pub fn add_transaction(_: &[u8]) {

}

pub mod test {
    pub fn xx() {

    }
}