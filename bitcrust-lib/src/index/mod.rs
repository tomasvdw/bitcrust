
use std::fs;

use lmdb_rs;
use config;

use std::path::Path;


struct Index {
    pub db_env: lmdb_rs::Environment,
    pub db_handle: lmdb_rs::DbHandle
}


impl Index {

    pub fn new(cfg: &config::Config) -> Index  {
        let path = Path::new("test-lmdb");

        let env = lmdb_rs::EnvBuilder::new().open(&path, 0o777).unwrap();

        let handle = env.get_default_db(lmdb_rs::DbFlags::empty()).unwrap();

        Index  {
            db_env: env,
            db_handle: handle

        }
    }
}