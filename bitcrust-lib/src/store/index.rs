
use std::mem;

use lmdb_rs;
use config;

use store::flatfileset;

/// Implement conversion for FilePtr (64-bit pointers) to values for lmdb
impl lmdb_rs::ToMdbValue for flatfileset::FilePtr {
    fn to_mdb_value<'a>(&'a self) -> lmdb_rs::MdbValue<'a> {

        lmdb_rs::MdbValue::new_from_sized(self)
    }

}

/// Implement conversion for FilePtr (64-bit pointers) to values for lmdb
impl lmdb_rs::FromMdbValue for flatfileset::FilePtr {
    fn from_mdb_value(value: &lmdb_rs::MdbValue) -> flatfileset::FilePtr {
        let ptr: &flatfileset::FilePtr = unsafe { mem::transmute( value.get_ref() ) };
        *ptr
    }

}



pub struct Index {
    pub db_env: lmdb_rs::Environment,
    pub db_handle: lmdb_rs::DbHandle
}


impl Index {

    /// Opens or creates the index pointed to by the given config
    pub fn new(cfg: &config::Config) -> Index  {

        let path = &cfg.root;

        let env = lmdb_rs::EnvBuilder::new().open(&path, 0o777).unwrap();
        let handle = env.get_default_db(lmdb_rs::DbFlags::empty()).unwrap();

        Index  {
            db_env: env,
            db_handle: handle

        }

    }

    /// Sets a value in the index
    pub fn set(&self, hash: &[u8], ptr: flatfileset::FilePtr) {
        let txn = self.db_env.new_transaction().unwrap();
        {
            let db = txn.bind(&self.db_handle); // get a database bound to this transaction


            db.set(&hash, &ptr).unwrap();

        }
        txn.commit().unwrap();
    }

    pub fn get(&self, hash: &[u8]) -> Option<flatfileset::FilePtr> {

        let txn = self.db_env.get_reader().unwrap();
        let db = txn.bind(&self.db_handle); // get a database bound to this transaction

        match db.get(&hash) {
            Ok(v) => Some(v),

            Err(lmdb_rs::MdbError::NotFound) => None,
            Err(e) => panic!("Error in index {:?}", e)
        }
    }

}