
use header::Header;
use network_encoding::*;
use super::{HashStoreError, Db, DbError};
use hash::*;
use record::Record;
use ::{ValuePtr};
use hashstore::SearchDepth;
use pow::U256;

// two things:

// add the records of header x


/// DbHeader is a header connected to genesis, with some meta-data
///
/// The meta-data is used
pub struct DbHeader {
    // previous_ptr point to the header at height
    // 0 => h-1
    // 1 => (h-1) % 16
    // 2 => (h-1) % 256
    // 3 => (h-1) % 4096
    pub previous_ptr: [ValuePtr; 4],


    // Pointer to the tx records in the `blk` file
    // or 0 if the records are not available
    pub records_ptr: ValuePtr,

    pub records_ptr_connected: ValuePtr,

    pub height: u64,
    pub acc_work: U256,
    pub header: Header
}


impl DbHeader {
    pub fn new(parent: DbHeader, parent_ptr: ValuePtr, header: Header) -> DbHeader {
        let previous_ptr = [
            parent_ptr,
            if (parent.height % 16) == 0 { parent_ptr } else { parent.previous_ptr[1] },
            if (parent.height % 256) == 0 { parent_ptr } else { parent.previous_ptr[2] },
            if (parent.height % 4096) == 0 { parent_ptr } else { parent.previous_ptr[3] }
        ];

        DbHeader {
            records_ptr: 0,
            records_ptr_connected: 0,
            previous_ptr: previous_ptr,
            height: parent.height + 1,
            acc_work: U256::zero(),
            header: header
        }
    }

}

pub fn get_locator(db: &mut Db, blockhash: &[u8;32]) -> Result<Vec<[u8; 32]>, HashStoreError> {
    let mut result = Vec::with_capacity(32);
    result.push(*blockhash);
    Ok(result)
}

pub fn get_best(db: &mut Db) -> Result<[u8;32], HashStoreError> {

    if let Some(key) = db.hdr.get_extremum(::db::EXTREMUM_MOST_WORK)? {
        Ok(key)
    }
    else {
        Ok([0;32]) // can't really happen cause of hardcoded genesis
    }
}

pub fn get(db: &mut Db, hash: &Hash) -> Result<Option<(ValuePtr, DbHeader)>, DbError> {

    if let Some((ptr,hdr)) = db.hdr.get(hash, SearchDepth::FullSearch)? {

        Ok(Some((ptr, DbHeader::decode(&mut Buffer::new(&hdr))?)))
    }
    else {
        Ok(None)
    }
}


fn update_extrema(db: &mut Db) -> Result<(), HashStoreError> {
    Ok(())
}

// Write a blockheader with a parent; no records
pub fn write_header(db: &mut Db, hash: &Hash, hdr: DbHeader) -> Result<ValuePtr, HashStoreError> {

    let mut v = vec![];
    let hdr = hdr.encode(&mut v);

    db.hdr.set(hash, &v, 0)
}


// Write a blockheader which is in-chain yet doesn't have a parent (aka genesis)
pub fn write_genesis(db: &mut Db, hash: &Hash, hdr: Header, records_ptr: ValuePtr) -> Result<ValuePtr, HashStoreError> {

    let db_hdr = DbHeader {
        records_ptr: records_ptr,
        records_ptr_connected: records_ptr,
        previous_ptr: [0; 4],
        height: 0,
        acc_work: U256::zero(),
        header: hdr };

    let mut v = vec![];
    let db_hdr = db_hdr.encode(&mut v);

    db.hdr.set(hash, &v, 0)
}



impl<'a> NetworkEncoding<'a> for DbHeader {
    fn decode(buffer: &mut Buffer) -> Result<DbHeader, EndOfBufferError> {

        Ok(DbHeader {
            previous_ptr: [u64::decode(buffer)?, u64::decode(buffer)?,
            u64::decode(buffer)?, u64::decode(buffer)?,],
            records_ptr: u64::decode(buffer)?,
            records_ptr_connected: u64::decode(buffer)?,
            height:      u64::decode(buffer)?,
            acc_work:    U256::decode(buffer)?,
            header:      Header::decode(buffer)?
        })
    }

    fn encode(&self, buffer: &mut Vec<u8>) {

        for ptr in self.previous_ptr.iter() {
            ptr.encode(buffer);
        }
        self.records_ptr.encode(buffer);
        self.records_ptr_connected.encode(buffer);
        self.height.encode(buffer);

        self.acc_work.low_u32().encode(buffer);
        self.acc_work.low_u32().encode(buffer);
        self.header.encode(buffer);

    }

}

