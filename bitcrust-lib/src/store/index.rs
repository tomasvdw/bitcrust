//! The index uses 32-bit hashes
use std::mem;

use lmdb_rs;
use config;
use hash;

use store::fileptr;

/// Implement conversion for FilePtr (64-bit pointers) to values for lmdb
impl lmdb_rs::ToMdbValue for fileptr::FilePtr {
    fn to_mdb_value<'a>(&'a self) -> lmdb_rs::MdbValue<'a> {

        lmdb_rs::MdbValue::new_from_sized(self)
    }

}

/// Implement conversion for FilePtr (64-bit pointers) to values for lmdb
impl lmdb_rs::FromMdbValue for fileptr::FilePtr {
    fn from_mdb_value(value: &lmdb_rs::MdbValue) -> fileptr::FilePtr {
        let ptr: &fileptr::FilePtr = unsafe { mem::transmute( value.get_ref() ) };
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

        let env = lmdb_rs::EnvBuilder::new()
            .map_size(2_000_000_000)
            .open(&path, 0o777)
            .unwrap();

        let handle = env.get_default_db(lmdb_rs::DbFlags::empty()).unwrap();

        Index  {
            db_env: env,
            db_handle: handle

        }

    }



    /// Sets a value in the index
    pub fn set(&self, hash: hash::Hash32, ptr: fileptr::FilePtr) {

        let u: &[u8] = &hash.0[..];
        let txn = self.db_env.new_transaction().unwrap();
        {
            let db = txn.bind(&self.db_handle);
            db.set(&u, &ptr).unwrap();

        }
        txn.commit().unwrap();
    }



    /// Retrieves the FilePtr to transaction given by `hash`
    /// If the key is not found, the given FilePtr is stored at that location
    ///
    /// This operations occurs atomically
    pub fn get_transaction_or_set_input(&self, hash: hash::Hash32, set_on_fail: fileptr::FilePtr) -> Option<fileptr::FilePtr> {

        unimplemented!();
    }


    pub fn get(&self, hash: hash::Hash32) -> Option<fileptr::FilePtr> {

        let txn = self.db_env.get_reader().unwrap();
        let db = txn.bind(&self.db_handle); // get a database bound to this transaction
        let u: &[u8] = &hash.0[..];
        match db.get(&u) {
            Ok(v) => Some(v),

            Err(lmdb_rs::MdbError::NotFound) => None,
            Err(e) => panic!("Error in index {:?}", e)
        }
    }

}