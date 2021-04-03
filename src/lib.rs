

#![feature(link_args)]
#![feature(plugin)]
#![feature(integer_atomics)]
#![feature(rustc_private)]



extern crate memmap;
extern crate itertools;
extern crate rand;
extern crate ring;
extern crate rayon;

#[macro_use]
pub extern crate slog ;
extern crate slog_term ;


/// Macro to create and empty a storage folder; used by tests
macro_rules! test_cfg {
    () => (::config::Config::new_empty(format!("{}-{}", file!(), line!())))
}



mod hash;

#[macro_use]
mod builders;

pub mod metrics;
pub mod transaction;
pub mod block;
pub mod script;

mod ffi;
mod buffer;
mod util;
mod store;
mod config;
mod merkle_tree;
mod block_add;
mod api;


pub use store::Store;


pub use api::*;

