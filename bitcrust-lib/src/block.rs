//!
//! Bitcrust block main documentation

/// 2016 Tomas, no rights reserved, no warranties given



use hash::{Hash256};
use transaction::{Transaction};

use serde::{Serializer,Deserializer};


/// BlockHeader represents the header of a block
#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    
    version:     u32,
    prev_hash:   Hash256,
    merkle_root: Hash256,
    pub time:        u32,
    bits:        u32,
    nonce:       u32,    
}

pub enum BlockStatus {
    Verified,
    Unverified
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub txs:    Vec<Transaction>,
}

impl Block {



}

#[cfg(test)]
mod tests {
    extern crate rustc_serialize;
    use std::io::Cursor;
    use self::rustc_serialize::hex::FromHex;
    use std::mem;
    use lmdb_rs::{EnvBuilder, DbFlags};
    use decode;

    #[test]
    fn test_blockheader_read()  {
        let hex = "0100000000000000000000000000000000000000000000000000000000000000\
                   000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa\
                   4b1e5e4a29ab5f49ffff001d1dac2b7c01010000000100000000000000000000\
                   00000000000000000000000000000000000000000000ffffffff4d04ffff001d\
                   0104455468652054696d65732030332f4a616e2f32303039204368616e63656c\
                   6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f75742066\
                   6f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe554827\
                   1967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4\
                   f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";
                   
        let blk_bytes   = rustc_serialize::hex::FromHex::from_hex(hex).unwrap();     
        let blk1: super::Block = decode::decode(&blk_bytes).unwrap();
        
        assert_eq!(blk1.header.version, 1);
        assert_eq!(blk1.txs.len(), 1);
        //assert_eq!(blk.txs[0].txs_in.len(), 1);
        //assert_eq!(blk.txs[0].txs_out.len(), 1);
        
        let xx = [3u8; 16];
        let mut st: u8;
        
        //let y = &xx;
        let mut it = xx.iter();
        st = *it.next().unwrap();
        assert_eq!(it.next().unwrap(), &3);
                 
    }

    #[test]
    fn test_blockheader_store()  {
        let hex = "0100000000000000000000000000000000000000000000000000000000000000\
                   000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa\
                   4b1e5e4a29ab5f49ffff001d1dac2b7c01010000000100000000000000000000\
                   00000000000000000000000000000000000000000000ffffffff4d04ffff001d\
                   0104455468652054696d65732030332f4a616e2f32303039204368616e63656c\
                   6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f75742066\
                   6f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe554827\
                   1967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4\
                   f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";
                   
        let blk_bytes = hex.from_hex().unwrap();     
        let blk       = super::Block::read(&mut Cursor::new(&blk_bytes)).unwrap();
              
        let path = Path::new("test-lmdb");
        let env = EnvBuilder::new().open(&path, 0o777).unwrap();
        
        let db_handle = env.get_default_db(DbFlags::empty()).unwrap();
        let txn = env.new_transaction().unwrap();
        {
            let xx        = unsafe { mem::transmute(&blk) };
        
            let db = txn.bind(&db_handle); // get a database bound to this transaction
            db.set(&"test", &xx);
            
        }
        txn.commit().unwrap();
        let reader = env.get_reader().unwrap();
        let db = reader.bind(&db_handle);
        //assert_eq!(1,0);
            
        let name = db.get::<&str>(&"test").unwrap();
        assert_eq!(&name, &"aa");
    }
}
