
use db::*;


pub fn transaction_get(db: &mut Db, hash: &[u8;32]) -> Result<Option<::DbTransaction>, DbError> {
    let r = db_transaction::read_transaction(db, hash)?;
    Ok(r)
}

pub enum TransactionPutFlags {

}

pub enum TransactionPutOk {
    ValidationError,
    OkOrphan(Vec<[u8;32]>),
    Ok
}

pub fn transaction_put(_tx: ::Transaction ) -> Result<(), DbError> {
    unimplemented!()
}