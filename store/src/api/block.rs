
use db::*;
use Header;
pub enum BlockAddHeaderOk {
    Invalid,
    Orphan,

}

pub enum VerifyFlags {
    NoVerifySignatures,
    VerifyAll
}



pub enum HeaderAddResult {
    Ok,
    AlreadyExists,
    Invalid,
    Orphan([u8;32])
}
/// Adds a header
///
pub fn header_add(db: &mut Db, hash: &[u8;32], header: Header) -> Result<HeaderAddResult, DbError> {

    if let Some(_) = db_header::get(db, &hash)? {
        Ok(HeaderAddResult::AlreadyExists)

    } else if let Some((parent_ptr, parent)) = db_header::get(db, &header.prev_hash)? {

        let db_header = db_header::DbHeader::new(parent, parent_ptr, header);
        db_header::write_header(db, hash, db_header)?;
        Ok(HeaderAddResult::Ok)

    } else {

        Ok(HeaderAddResult::Orphan(header.prev_hash))
    }
}


pub enum BlockExistsOk {
    NotFound,
    FoundHeaderOrphan,
    FoundHeader,
    FoundHeaderAndData
}


pub fn block_add_transactions(_db: &mut Db, _block_data: &[u8], _validate: bool) -> Result<(), DbError>
{
    Ok(())
}



pub fn block_exists(_blockhash: &[u8;32]) -> Result<BlockExistsOk, DbError> {
    unimplemented!()
}

/// Returns the hash of the block header with the most accumulated work
pub fn header_get_best(db: &mut Db) -> Result<[u8;32], DbError> {

    Ok(db_header::get_best_header(db)?)
}

/// Returns a set of block hashes of which no records are known
pub fn block_needs_download(_db: &mut Db, _count: usize) -> Result<Vec<[u8;32]>, DbError> {

    /*let best_header = db_header::get_best_header(db)?;
    if best_header.has_transactions() {
        return Ok(vec![]);
    } else {

    }*/

    unimplemented!() //Ok(db_header::get_best_block(db)?)
}


pub fn header_get(db: &mut Db, hash: &[u8;32]) -> Result<Option<db_header::DbHeader>, DbError> {
    Ok(db_header::get(db, hash)?
           .map(|(_, db_hdr)| db_hdr))
}


/// Constructs a locator object for the given block hash
///
/// This consists of the blockhash and at most 32 hashes ancestor hashes,
/// ending in Genesis
pub fn block_get_locator(db: &mut Db, blockhash: &[u8;32]) -> Result<Vec<[u8; 32]>, DbError> {

    Ok(db_header::get_locator(db, blockhash)?)
}


