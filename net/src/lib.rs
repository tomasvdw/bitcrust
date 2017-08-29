#![feature(tcpstream_connect_timeout)]

#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate circular;
#[macro_use]
extern crate log;
extern crate multiqueue;
#[macro_use]
extern crate nom;
extern crate rand;
extern crate ring;
extern crate regex;
extern crate rusqlite;
extern crate sha2;

pub mod bitcoin_network_connection;
mod block_header;
// pub mod client_message;
mod parser;
mod message;
mod inventory_vector;
mod net_addr;
mod services;

pub use message::*;
pub use net_addr::NetAddr;
// pub use client_message::ClientMessage;
pub use bitcoin_network_connection::{BitcoinNetworkConnection, BitcoinNetworkError};
pub use block_header::BlockHeader;
pub use services::Services;