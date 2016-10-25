

#![feature(custom_derive, plugin)]
#![feature(question_mark)]

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

mod decode;
//mod encode;


pub use block::Block;

mod hash;

pub mod transaction;
pub mod block;
pub mod script;

mod store;

mod config;


pub fn add_block(_: &[u8]) {

}

pub fn add_transaction(_: &[u8]) {

}

