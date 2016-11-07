
use std::fmt;
use std::io;
use decode;
use decode::Parse;

use script::context;
use store::Store;

use itertools::Itertools;

use hash;

const MAX_TRANSACTION_SIZE: usize = 1000000;

#[derive(Debug)]
pub enum TransactionError {
    UnexpectedEndOfData,
    TransactionTooLarge,
    NoInputs,
    NoOutputs,
    DuplicateInputs,

    InputNotFound,
}

pub enum TransactionOk {
    AlreadyExists,

    VerifiesAndStored,

}

type TransactionResult<T> = Result<T, TransactionError>;

impl From<decode::EndOfBufferError> for TransactionError {
    fn from(_: decode::EndOfBufferError) -> TransactionError {

        TransactionError::UnexpectedEndOfData

    }
}

/// A transaction is kept in memory as a byte slice
///
/// The most common operations on a transaction are reading and writing
/// for which (de-)serialization would be an unnecessary overhead
pub struct RawTx<'a> {
    raw: &'a[u8]
}

#[derive(Debug)]
pub struct ParsedTx<'a> {
    pub version:   i32,
    pub txs_in:    Vec<TxInput<'a>>,
    pub txs_out:   Vec<TxOutput<'a>>,
    pub lock_time: u32,

    raw:           decode::Buffer<'a>,


}



impl<'a> decode::Parse<'a> for ParsedTx<'a> {
    /// Parses the raw bytes into individual fields
    /// and perform basic syntax checks
    fn parse(buffer: &mut decode::Buffer<'a>) -> Result<ParsedTx<'a>, decode::EndOfBufferError> {

        let org_buffer = *buffer;
        Ok(ParsedTx {
            version:   try!(i32::parse(buffer)),
            txs_in:    try!(Vec::parse(buffer)),
            txs_out:   try!(Vec::parse(buffer)),
            lock_time: try!(u32::parse(buffer)),
            raw:       buffer.consumed_since(org_buffer)

        })
    }
}

impl<'a> decode::ToRaw<'a> for ParsedTx<'a> {
    fn to_raw(&self) -> &[u8] {
        self.raw.inner
    }
}

impl<'a> ParsedTx<'a> {

    /// Performs basic syntax checks on the transaction
    pub fn verify_syntax(&self) -> TransactionResult<()> {

        if self.raw.len() > MAX_TRANSACTION_SIZE {
            return Err(TransactionError::TransactionTooLarge);
        }

        if self.txs_in.is_empty() {
            return Err(TransactionError::NoInputs);
        }

        if self.txs_out.is_empty() {
            return Err(TransactionError::NoOutputs);
        }

        // No double inputs
        if self.txs_in.iter().combinations(2).any(|pair|
               pair[0].prev_tx_out_idx == pair[1].prev_tx_out_idx
            && pair[0].prev_tx_out     == pair[1].prev_tx_out)
        {
            return Err(TransactionError::DuplicateInputs);
        }

        Ok(())
    }

    pub fn is_coinbase(&self) -> bool {

        self.txs_in.len() == 1 && self.txs_in[0].prev_tx_out.is_null()
    }

    pub fn verify_and_store(&self, store: &mut Store) -> TransactionResult<TransactionOk> {

        self.verify_syntax()?;

        let hash = hash::double_sha256(self.raw.inner);

        // First see if it already exists
        if store.index.get(hash.as_ref()).is_some() {
            return Ok(TransactionOk::AlreadyExists)
        }

        if !self.is_coinbase() {

            self.verify_input_scripts(store)?;
        }


        Ok(TransactionOk::VerifiesAndStored)
    }

    pub fn verify_input_scripts(&self, store: &mut Store) -> TransactionResult<()> {

        for input in &self.txs_in {

            let output = store.index.get(input.prev_tx_out.0)
                .ok_or(TransactionError::InputNotFound)?;


            let mut org_tx = decode::Buffer::new(store.file_transactions.read(output));

            let parsed_tx = ParsedTx::parse(&mut org_tx)?;


        }

        Ok(())
    }


}





pub struct TxInput<'a> {
    prev_tx_out:     hash::Hash32<'a>,
    prev_tx_out_idx: u32,
    script:          &'a[u8],
    sequence:        u32,
}


impl<'a> decode::Parse<'a> for TxInput<'a> {
    fn parse(buffer: &mut decode::Buffer<'a>) -> Result<TxInput<'a>, decode::EndOfBufferError> {

        let result = TxInput {
            prev_tx_out:     try!(hash::Hash32::parse(buffer)),
            prev_tx_out_idx: try!(u32::parse(buffer)),
            script:          try!(buffer.parse_compact_size_bytes()),
            sequence:        try!(u32::parse(buffer))
        };

        Ok(result)
    }
}

pub struct TxOutput<'a> {
    value:     i64,
    pk_script: &'a[u8]
}

impl<'a> decode::Parse<'a> for TxOutput<'a> {

    fn parse(buffer: &mut decode::Buffer<'a>) -> Result<TxOutput<'a>, decode::EndOfBufferError> {

        Ok(TxOutput {
            value:      i64::parse(buffer)?,
            pk_script:  buffer.parse_compact_size_bytes()?

        })
    }
}




impl<'a> fmt::Debug for TxInput<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(fmt, "Prev-TX:{:?}, idx={:?}, seq={:?} script=",
            self.prev_tx_out,
            self.prev_tx_out_idx,
            self.sequence));
        
        let ctx = context::Context::new(&self.script);
        write!(fmt, "{:?}", ctx)

        
    }
}

impl<'a> fmt::Debug for TxOutput<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        try!(write!(fmt, "v:{:?} ", self.value));
        
        let ctx = context::Context::new(&self.pk_script);
        write!(fmt, "{:?}", ctx)

    }
}



#[cfg(test)]
mod tests {
    extern crate rustc_serialize;

    #[test]
    fn test_parse_tx() {}
}
