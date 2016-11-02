use std::mem;

mod flatfile;
mod flatfileset;
mod index;

extern crate libc;

use lmdb_rs;
use config;

use std::path::Path;

use self::flatfileset::FlatFileSet;

pub struct Store {

    pub index: index::Index,

    // Flat files
    pub file_transactions: FlatFileSet,
//    pub file_blockheaders: FlatFileSet,
//    pub file_spenttree:    FlatFileSet,

}


impl Store {

    pub fn new(cfg: &config::Config) -> Store {


        Store {
            index: index::Index::new(cfg),

            file_transactions: FlatFileSet::new(
                &cfg.root.clone().join("transactions"),
                "tx-",
                1024 * 1024 * 1024,
                1024 * 1024 * 1024 - 10 * 1024 * 1024
            )

        }
    }

}


#[cfg(test)]
pub fn test_store() {

}


#[cfg(test)]
mod tests {
    use super::Store;
    use config;

    #[test]
    fn test_store_new() {
        let store = Store::new(&config::Config::new_test());
    }
}