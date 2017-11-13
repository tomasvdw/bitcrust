extern crate byteorder;

use std::io::Cursor;
use network_encoding::*;
use super::{HashStoreError, Db};
use hashstore::SearchDepth;
use hash::*;
use ::*;
use record::Record;


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
 //   outputs: Cell<Vec<u8>> // space to decompress outputs
}


pub fn store_transactions(db: &mut Db, txdata: &mut Buffer) -> Result<Vec<Record>, DbError> {

    let tx_count = txdata.decode_compact_size()?;
    //let result = Vec::with_capacity(3 * tx_count);
    for _ in 0..tx_count {
        let org_buffer = *txdata;

        let parsed = Transaction::decode(txdata)?;

        let txbytes = txdata.consumed_since(org_buffer);
        let hash = double_sha256(txbytes.inner);
        write_transaction(db, &hash, &parsed, vec![]);

        //println!("tx: {}, {}", parsed.txs_in.len(), parsed.txs_out.len());

    }
    Ok(vec![])
}



impl DbTransaction {

    /// Parses the db-format buffer as a transaction
    pub fn as_tx<'a>(&'a self) -> Result<::Transaction<'a>, EndOfBufferError> {

        type HashRef<'a> = &'a Hash;
        type Bytes<'a> = &'a [u8];

        let mut buffer: Buffer<'a> = Buffer::new(&self.buffer);
        let mut buffer_sig: Buffer<'a> = Buffer::new(&self.sigs);

        println!("{:?}", buffer);

        let input_count = u32::decode(&mut buffer)?;
        let output_count = u32::decode(&mut buffer)?;
        let _ = u64::decode(&mut buffer);

        let prev_outs: Vec<Record> = try!((0..input_count).map(|_|
            Record::decode(&mut buffer)).collect());

        let txs_in = try!(prev_outs.iter().map(|rec|
            Ok(transaction::TxInput {
                script:          Bytes::decode(&mut buffer_sig)?,
                sequence:        buffer_sig.decode_compact_size()? as u32,
                prev_tx_out:     HashRef::decode(&mut buffer)?,
                prev_tx_out_idx: rec.get_output_index()
            })
        ).collect());

        let txs_out = try!((0..output_count).map(|_|
            Ok(transaction::TxOutput {
                value:     buffer.decode_compact_size()? as i64,
                pk_script: Bytes::decode(&mut buffer)?,
            })
        ).collect());

        Ok(Transaction {
            version: buffer.decode_compact_size()? as i32,
            txs_in:  txs_in,
            txs_out: txs_out,
            lock_time: buffer.decode_compact_size()? as u32,
        })
    }


}


/// Reads the full transaction, and returns as OwnedTransaction
///
/// This does not yet decode the content
pub fn read_transaction(db: &mut Db, tx_hash: &[u8; 32]) -> Result<Option<DbTransaction>, HashStoreError> {

    if let Some((_, b)) = db.tx.get(tx_hash, SearchDepth::FullSearch)? {

        let sig_ptr = {

            let mut sig_ptr = Buffer::new(&b[8..]);
            u64::decode(&mut sig_ptr).map_err(|_| HashStoreError::Other)? as ValuePtr
        };

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
                         -> Result<::ValuePtr, HashStoreError>
{

    let input_count = tx.txs_in.len() as u32;
    let output_count= tx.txs_out.len() as u32;

    // write the signatures
    let mut buffer_sig = Vec::with_capacity(136 * input_count as usize);
    for tx_in in tx.txs_in.iter() {
        tx_in.script.encode(&mut buffer_sig);
        encode_compact_size(&mut buffer_sig, tx_in.sequence as usize);
    }
    let sig_ptr = db.sig.set_value(&buffer_sig)?;


    let mut buffer = Vec::with_capacity(102 * output_count as usize);
    input_count.encode(&mut buffer);
    output_count.encode(&mut buffer);
    sig_ptr.encode(&mut buffer);
    for rec in records.into_iter() {
        rec.0.encode(&mut buffer);
    }
    for tx_in in tx.txs_in.iter() {
        tx_in.prev_tx_out.encode(&mut buffer);
    }
    for tx_out in tx.txs_out.iter() {
        encode_compact_size(&mut buffer, tx_out.value as usize);
        tx_out.pk_script.encode(&mut buffer);
    }

    encode_compact_size(&mut buffer, tx.version as usize);
    encode_compact_size(&mut buffer, tx.lock_time as usize);

    // minimum size and align at qword
    let min_size = output_count as usize * 32;
    while (buffer.len() < min_size || (buffer.len() % 7) > 0) {
        buffer.push(0);
    }

    let tx_ptr = db.tx.set(tx_hash, &buffer, 0)?;

    Ok(tx_ptr)
}

