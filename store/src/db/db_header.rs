
use header::Header;
use serde_network;
use super::{HashStoreError, Db, DbError};
use hash::*;
use ::{ValuePtr};
use hashstore::SearchDepth;
use pow::U256;
use pow;


/// DbHeader is a header connected to genesis, with some meta-data
///
#[derive(Serialize, Deserialize)]
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

        // calculate accumulated work
        let target   = pow::from_compact(header.bits);
        let work     = pow::difficulty_target_to_work(target);
        let acc_work = parent.acc_work + work;

        //println!("target: {}, work:{}, parent_work:{}, acc_work:{}, h:{}", target, work, parent.acc_work, acc_work, parent.height+1);
        DbHeader {
            records_ptr: 0,
            records_ptr_connected: 0,
            previous_ptr: previous_ptr,
            height: parent.height + 1,
            acc_work: parent.acc_work + work,
            header: header
        }
    }

    pub fn has_transactions(&self) -> bool {
        self.records_ptr != 0
    }

    pub fn decode(buf: &[u8]) -> Result<DbHeader, serde_network::Error> {
        serde_network::deserialize(buf)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        serde_network::serialize(&mut buf, self);
        buf
    }

}

pub fn get_locator(_db: &mut Db, blockhash: &[u8;32]) -> Result<Vec<[u8; 32]>, HashStoreError> {
    let mut result = Vec::with_capacity(32);
    result.push(*blockhash);
    Ok(result)
}

pub fn get_best_header(db: &mut Db) -> Result<[u8;32], DbError> {

    db.hdr.get_extremum(::db::EXTREMUM_BEST_HEADER)?.ok_or(
        // can't really happen cause of hardcoded genesis
        DbError::HeaderFileCorrupted
    )
}

pub fn get(db: &mut Db, hash: &Hash) -> Result<Option<(ValuePtr, DbHeader)>, DbError> {

    if let Some((ptr,hdr)) = db.hdr.get(hash, SearchDepth::FullSearch)? {

        Ok(Some((ptr, serde_network::deserialize(&hdr)?)))
    }
    else {
        Ok(None)
    }
}


// Write a blockheader with a parent; no records
pub fn write_header(db: &mut Db, hash: &Hash, hdr: DbHeader) -> Result<ValuePtr, DbError> {

    let mut v = vec![];
    serde_network::serialize(&mut v, &hdr)?;

    let ptr = db.hdr.set(hash, &v, 0)?;

    db.hdr.update_extremum(ptr, ::db::EXTREMUM_BEST_HEADER,  |other| {

        if let Ok(other) = DbHeader::decode(&other) {
            other.acc_work < hdr.acc_work
        } else {
            false
        }

    })?;
    Ok(ptr)

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

    let db_hdr_raw = db_hdr.encode();

    let ptr = db.hdr.set(hash, &db_hdr_raw, 0)?;

    db.hdr.update_extremum(ptr, ::db::EXTREMUM_BEST_HEADER, |_| true);
    db.hdr.update_extremum(ptr, ::db::EXTREMUM_BEST_BLOCK, |_| true);
    Ok(ptr)
}




