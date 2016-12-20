

#![feature(link_args)]
#![feature(custom_derive, plugin)]
#![feature(question_mark)]
#![feature(integer_atomics)]




//!
//! Bitcrust main documentation
//! Provides access to the bitcrust database
/// let db = Db::new();
///


extern crate memmap;
extern crate lmdb_rs;
extern crate itertools;
extern crate rand;
extern crate ring;

#[macro_use]
pub extern crate slog ;
extern crate slog_term ;

use slog::DrainExt;



mod ffi;
pub mod metrics;

mod buffer;
use buffer::*;

mod util;

pub use block::Block;

mod hash;

pub mod transaction;
pub mod block;
pub mod script;

mod store;

mod config;

mod merkle_tree;
use store::Store;




pub fn init() -> Store {

    let config = config::Config::new_test();
    Store::new(&config)
}

/// Validates and stores a block;
///
/// Currently the fn here is used to collect what needs to be done;
/// TODO: distibute over different mods
pub fn add_block(store: &mut store::Store, buffer: &[u8]) {


    let block = Block::new(buffer).unwrap();
    let block_hash = hash::Hash32Buf::double_sha256(block.header.to_raw());

    let mut total_amount = 0_u64;
    let mut stree_pointers = Vec::new();

    block.process_transactions(|tx| {

        total_amount += 1;


        let res = tx.verify_and_store(store).unwrap();

        let ptr = match res {
            transaction::TransactionOk::AlreadyExists(ptr) => ptr,
            transaction::TransactionOk::VerifiedAndStored(ptr) => ptr
        };

        stree_pointers.push(ptr);
        stree_pointers.append(&mut tx.get_output_fileptrs(store));

        Ok(())

    }).unwrap();

    block.verify_and_store(store, stree_pointers);

    // TODO verify amounts
}

pub fn add_transaction(_: &[u8]) {

}

#[cfg(test)]
mod tests {

    use util::*;
    use super::*;

    #[test]
    pub fn test_add_block() {
        let hex = "0100000000000000000000000000000000000000000000000000000000000000\
           000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa\
           4b1e5e4a29ab5f49ffff001d1dac2b7c01010000000100000000000000000000\
           00000000000000000000000000000000000000000000ffffffff4d04ffff001d\
           0104455468652054696d65732030332f4a616e2f32303039204368616e63656c\
           6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f75742066\
           6f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe554827\
           1967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4\
           f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";


        let slice = &from_hex(hex);
        let mut store = init();

        add_block(&mut store, slice);

    }
}
