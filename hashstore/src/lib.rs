#![feature(integer_atomics)]
//!
//! Key/value store with specific design properties
//! optimized for storing the transactions of a blockchain
//!
//! Apart from getting and setting values, it allows for
//! getting *dependent* values. If a dependency X of Y is requested
//! and X is missing, an anchor will be inserted to ensure that the
//! dependency is resolved before X is inserted.
//!
//! This allows a caller that wants to insert B with a dependency A to
//!
//! * Get dependency A
//! * If successful, verify the dependency
//! * If not, ignore and store B anyway.
//! * When A comes in later and is stored, set will fail and the dependency A->B
//! can still be verified
//!
//! Other design considerations
//!
//! * Keys are always 32-byte hashes
//! * Concurrent, lock-free R/W access across processes
//! * Append only
//! * Allows seeking only recent keys
//!
//! The implementation uses single root hash table, and MRU liked list of objects
//! to solve collisions.
//!
//! See [HashStore](struct.HashStore.html) for examples


#[macro_use]
extern crate serde_derive;
extern crate bincode;

mod header;
mod io;
mod values;
mod timer;
mod hashstore;

pub use hashstore::{HashStoreError, HashStore, SearchDepth};
pub use values::ValuePtr;


