extern crate byteorder;

use super::{Db, DbError, DbResult};
use hashstore::SearchDepth;
use ::{ValuePtr,Transaction};
use record::Record;
use serde_network::{Deserializer,Serializer,deserialize};
use transaction;
//use std::iter::Map;


/// An owned transaction that owns the data as extracted from the db
///
/// It can be parsed into an actual tx, using `as_tx()`
///
/// The structure is opaque and serialization and deserialization is done in
/// DbTransaction::write and DbTransaction::as_tx()
///
/// But for most accesses, only a part of the structure is deserialized
pub struct DbTransaction {
    buffer: Vec<u8>,
    sigs: Vec<u8>,
}

/*
pub fn store_transactions(db: &mut Db, txdata: &mut Buffer) -> Result<Vec<Record>, DbError> {

    let tx_count = txdata.decode_compact_size()?;
    //let result = Vec::with_capacity(3 * tx_count);
    for _ in 0..tx_count {
        let org_buffer = *txdata;

        let parsed = Transaction::decode(txdata)?;

        let txbytes = txdata.consumed_since(org_buffer);
        let hash = double_sha256(txbytes.inner);
        write_transaction(db, &hash, &parsed, vec![])?;

        //println!("tx: {}, {}", parsed.txs_in.len(), parsed.txs_out.len());

    }
    Ok(vec![])
}

*/

impl DbTransaction {

    /// Parses the db-format buffer as a transaction
    pub fn as_tx<'a>(&'a self) -> DbResult<Transaction<'a>> {

        let mut de_tx = Deserializer::new(&self.buffer);
        let mut de_sigs = Deserializer::new(&self.sigs);

        let input_count: u32  = de_tx.deserialize()?;
        let output_count: u32 = de_tx.deserialize()?;
     //   let sig_ptr: ValuePtr = de_tx.deserialize()?;

        let prev_outs: Vec<Record> = r#try!((0..input_count).map(|_|
            de_tx.deserialize()).collect());

        let txs_in: Result<Vec<_>,DbError> = prev_outs.iter().map(|rec|
            Ok(transaction::TxInput {
                script:          de_sigs.deserialize().map_err(DbError::from)?,
                sequence:        de_sigs.deserialize().map_err(DbError::from)?,
                prev_tx_out:     de_tx.deserialize().map_err(DbError::from)?,
                prev_tx_out_idx: rec.get_output_index()
            })
        ).collect();

        let txs_out: Result<Vec<_>,DbError>  = (0..output_count).map(|_|
            Ok(transaction::TxOutput {
                value:     de_tx.deserialize().map_err(DbError::from)?,
                pk_script: de_tx.deserialize().map_err(DbError::from)?,
            })
        ).collect();

        Ok(Transaction {
            version: de_tx.deserialize()?,
            txs_in:  txs_in?,
            txs_out: txs_out?,
            lock_time: de_tx.deserialize()?
        })

    }


}



/// Reads the full transaction, and returns as owned DbTransaction
///
/// This does not yet decode the content
pub fn read_transaction(db: &mut Db, tx_hash: &[u8; 32]) -> DbResult<Option<DbTransaction>> {

    if let Some((_, b)) = db.tx.get(tx_hash, SearchDepth::FullSearch)? {

        let sig_ptr = deserialize(&b[8..])?;

        let sigs = db.sig.get_value(sig_ptr)?;

        Ok(Some(DbTransaction {
            buffer: b,
            sigs: sigs
        }))
    }
    else {
        Ok(None)
    }
}



/// Write the transaction to tx and the signatures to sig
/// This procedure defines the on-disk format
pub fn write_transaction(db: &mut Db, tx_hash: &[u8;32], tx: &::Transaction, records: Vec<Record>)
                         -> DbResult<ValuePtr>
{
    let input_count = tx.txs_in.len();
    let output_count= tx.txs_out.len();

    // serialize signatures
    let mut buf_sigs = Vec::with_capacity(136 * input_count);
    {
        let mut ser_sigs = Serializer::new(&mut buf_sigs);
        for tx_in in tx.txs_in.iter() {
            ser_sigs.serialize(&tx_in.script)?;
            ser_sigs.serialize(&tx_in.sequence)?;
        }
    }

    // store and get pointer to result
    let sig_ptr = db.sig.set_value(&buf_sigs)?;


    // serialize the rest of the tx
    let mut buf_tx   = Vec::with_capacity(102 * output_count);
    {
        let mut ser_tx = Serializer::new(&mut buf_tx);

        ser_tx.serialize(&(input_count as u32))?;
        ser_tx.serialize(&(output_count as u32))?;
        ser_tx.serialize(& sig_ptr)?;
        for rec in records.into_iter() {
            ser_tx.serialize(&rec.0)?;
        }
        for tx_in in tx.txs_in.iter() {
            ser_tx.serialize(&tx_in.prev_tx_out)?;
        }
        for tx_out in tx.txs_out.iter() {
            ser_tx.serialize(&tx_out.value)?;
            ser_tx.serialize(&tx_out.pk_script)?;
        }

        ser_tx.serialize(&tx.version)?;
        ser_tx.serialize(&tx.lock_time)?;

    }
    // minimum size and align at qword
    let min_size = (output_count+1) as usize * 32;
    while buf_tx.len() < min_size || (buf_tx.len() % 7) > 0 {
        buf_tx.push(0);
    }
    let tx_ptr = db.tx.set(tx_hash, &buf_tx, 0)?;

    Ok(tx_ptr)

}

