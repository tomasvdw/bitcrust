
mod flatfile;
mod flatfileset;


use lmdb_rs;
use config;

use std::path::Path;

use self::flatfileset::FlatFileSet;



pub struct Store {

    // Indexes
    pub db_env:    lmdb_rs::Environment,
    pub db_handle: lmdb_rs::DbHandle,

    // Flat files
    pub file_transactions: FlatFileSet,
//    pub file_blockheaders: FlatFileSet,
//    pub file_spenttree:    FlatFileSet,

}


impl Store {

    pub fn new(cfg: &config::Config) -> Store {


        let path = &cfg.root;

        let env = lmdb_rs::EnvBuilder::new().open(&path, 0o777).unwrap();
        let handle = env.get_default_db(lmdb_rs::DbFlags::empty()).unwrap();

        Store {
            db_env: env,
            db_handle: handle,

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