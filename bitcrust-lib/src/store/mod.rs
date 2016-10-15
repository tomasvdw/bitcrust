
use std::fs;

use lmdb_rs;
use config;

use std::path::Path;

mod txfile;

mod flatfile;

struct Store {

    // Indexes
    pub db_env: lmdb_rs::Environment,
    pub db_handle: lmdb_rs::DbHandle,

    // Flat files
    pub tx_file: txfile::TxFile


}


impl Store {

    pub fn new(cfg: &config::Config) -> Store {
        let path = Path::new("test-lmdb");

        let env = lmdb_rs::EnvBuilder::new().open(&path, 0o777).unwrap();

        let handle = env.get_default_db(lmdb_rs::DbFlags::empty()).unwrap();

        Store {
            db_env: env,
            db_handle: handle,

            tx_file: txfile::TxFile::new(path)

        }
    }
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