
use std::fs;
use std::path::Path;
use hashstore::*;
use network_encoding::*;
use hash::*;
use record::Record;
use util;

pub mod db_transaction;
pub mod db_header;

mod db_block;

const ROOT_BITS_TX : u8 = 26;
const ROOT_BITS_SIG: u8 = 0;

const ROOT_BITS_HDR: u8 = 20;
const ROOT_BITS_BLK: u8 = 0;

pub const EXTREMUM_MOST_WORK: usize = 1;

/// All DbErrors are unrecoverable data corruption errors
#[derive(Debug)]
pub enum DbError {
    HashStoreError(HashStoreError),
    EndOfBufferError,
    ParentNotFound,
    HeaderFileCorrupted
}

impl From<HashStoreError> for DbError {
    fn from(err: HashStoreError) -> DbError {
        DbError::HashStoreError(err)
    }
}
impl From<EndOfBufferError> for DbError {
    fn from(_: EndOfBufferError) -> DbError {
        DbError::EndOfBufferError
    }
}


pub struct Db {
    tx: HashStore,
    sig: HashStore,

    hdr: HashStore,
    blk: HashStore,
}

// useful for testing
pub fn init_empty<P: AsRef<Path>>(db_path: P) -> Result<Db, HashStoreError> {
    let db_path = db_path.as_ref();
    let exists = db_path.exists();
    if exists {
        // temporary useful for testing
        fs::remove_dir_all(db_path).unwrap();
    }
    init(db_path)
}


pub fn init<P: AsRef<Path>>(db_path: P) -> Result<Db, HashStoreError> {
    let db_path = db_path.as_ref();
    let exists = db_path.exists();
    let mut db = Db {
        tx : HashStore::new(Path::join(db_path, "tx"),  ROOT_BITS_TX)?,
        sig: HashStore::new(Path::join(db_path, "sig"), ROOT_BITS_SIG)?,
        hdr: HashStore::new(Path::join(db_path, "hdr"), ROOT_BITS_HDR)?,
        blk: HashStore::new(Path::join(db_path, "blk"), ROOT_BITS_BLK)?,
    };

    if !exists {
        add_genesis(&mut db)?;
    }
    Ok(db)
}


const GENESIS_BLOCK: &'static str = "\
0100000000000000000000000000000000000000000000000000000000000000\
000000003BA3EDFD7A7B12B27AC72C3E67768F617FC81BC3888A51323A9FB8AA\
4B1E5E4A29AB5F49FFFF001D1DAC2B7C01010000000100000000000000000000\
00000000000000000000000000000000000000000000FFFFFFFF4D04FFFF001D\
0104455468652054696D65732030332F4A616E2F32303039204368616E63656C\
6C6F72206F6E206272696E6B206F66207365636F6E64206261696C6F75742066\
6F722062616E6B73FFFFFFFF0100F2052A01000000434104678AFDB0FE554827\
1967F1A67130B7105CD6A828E03909A67962E0EA1F61DEB649F6BC3F4CEF38C4\
F35504E51EC112DE5C384DF7BA0B8D578A4C702B6BF11D5FAC00000000";

/// Add genesis tx and block to the db
fn add_genesis(db: &mut Db) -> Result<(), HashStoreError> {

    let genesis = ::util::from_hex(GENESIS_BLOCK);
    let mut buf = Buffer::new(&genesis);

    let block_hash = double_sha256(&genesis[0..80]);
    let tx_hash =    double_sha256(&genesis[81..]);
    let hdr = ::Header::decode(&mut buf).
        expect("Hardcoded genesis is invalid");

    assert_eq!(buf.decode_compact_size().unwrap(), 1, "Hardcoded genesis is invalid");

    let tx= ::Transaction::decode(&mut buf).
        expect("Hardcoded genesis is invalid");

    let tx_ptr = db_transaction::write_transaction(db, &tx_hash, &tx, vec![Record::new_coinbase()])?;

    assert_eq!(&tx_hash[..],
               &::util::from_hex_rev("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b")[..]);

    // Create our blk-records
    let records = vec![
        Record::new_start_of_block(1), Record::new_transaction(tx_ptr)];

    let blk_ptr = db.blk.set_value(Record::to_bytes(&records))?;

    let hdr_ptr = db_header::write_genesis(db, &block_hash, hdr, blk_ptr)?;

    // we can update the most-work pointer to this without callback, as their is no extremum yet
    db.hdr.update_extremum(hdr_ptr, EXTREMUM_MOST_WORK, |_| true )?;
    Ok(())
}

