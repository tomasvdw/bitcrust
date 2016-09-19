

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

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



extern crate serde;
extern crate serde_json;

extern crate lmdb_rs;


mod decode;
mod encode;

pub use decode::decode;
pub use encode::encode;

pub use block::Block;

mod hash;

pub mod transaction;
pub mod block;
pub mod script;

mod store;

mod config;

use lmdb_rs::{Environment};


