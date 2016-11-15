
use std::fmt;
use buffer::*;

use script::context;
use store::Store;

use itertools::Itertools;

use hash;

use ffi;
use store::fileptr::FilePtr;

const MAX_TRANSACTION_SIZE: usize = 1_000_000;

#[derive(Debug)]
pub enum TransactionError {
    UnexpectedEndOfData,
    TransactionTooLarge,
    NoInputs,
    NoOutputs,
    DuplicateInputs,

    OutputTransactionNotFound,
    OutputIndexNotFound,

    ScriptError(i32)

}

#[derive(Debug)]
pub enum TransactionOk {
    AlreadyExists,

    VerifiedAndStored,

}

type TransactionResult<T> = Result<T, TransactionError>;

impl From<EndOfBufferError> for TransactionError {
    fn from(_: EndOfBufferError) -> TransactionError {

        TransactionError::UnexpectedEndOfData

    }
}

/// A transaction represents a parsed transaction;
///
#[derive(Debug)]
pub struct Transaction<'a> {
    pub version:   i32,
    pub txs_in:    Vec<TxInput<'a>>,
    pub txs_out:   Vec<TxOutput<'a>>,
    pub lock_time: u32,

    raw:           Buffer<'a>,


}



impl<'a> Parse<'a> for Transaction<'a> {
    /// Parses the raw bytes into individual fields
    /// and perform basic syntax checks
    fn parse(buffer: &mut Buffer<'a>) -> Result<Transaction<'a>, EndOfBufferError> {

        let org_buffer = *buffer;
        Ok(Transaction {
            version:   try!(i32::parse(buffer)),
            txs_in:    try!(Vec::parse(buffer)),
            txs_out:   try!(Vec::parse(buffer)),
            lock_time: try!(u32::parse(buffer)),
            raw:       buffer.consumed_since(org_buffer)

        })
    }
}

impl<'a> ToRaw<'a> for Transaction<'a> {
    fn to_raw(&self) -> &[u8] {
        self.raw.inner
    }
}

impl<'a> Transaction<'a> {

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

    pub fn verify_pending_outputs(&self, inputs: &Vec<FilePtr>) {
        if !inputs.is_empty() {
            //println!("Back-tracking {} inputs", inputs.len());
        }
    }

    pub fn verify_and_store(&self, store: &mut Store) -> TransactionResult<TransactionOk> {

        self.verify_syntax()?;


        let hash_buf = hash::Hash32Buf::double_sha256(self.to_raw());
        let _        = hash_buf.as_ref();

        // First see if it already exists
        let existing_ptrs = store.hash_index.get_ptr(hash_buf.as_ref());

        if existing_ptrs
            .iter()
            .any(|p| p.is_transaction()) {

            assert_eq!(existing_ptrs.len(), 1);
            return Ok(TransactionOk::AlreadyExists)
        }

        // existing_ptrs are now inputs that are waiting for this transactions
        // they need to be verified
        self.verify_pending_outputs(&existing_ptrs);


        let ptr      = store.block_content.write(self.to_raw());

        if !self.is_coinbase() {
            self.verify_input_scripts(store, ptr)?;
        }


        store.hash_index.set_tx_ptr(hash_buf.as_ref(), ptr, existing_ptrs);


        Ok(TransactionOk::VerifiedAndStored)
    }


    pub fn verify_input_scripts(&self, store: &mut Store, tx_ptr: FilePtr) -> TransactionResult<()> {

        for (index, input) in self.txs_in.iter().enumerate() {

            let output = store.hash_index.get_tx_for_output(input.prev_tx_out,
                tx_ptr.as_input(index as u32 ));
            let output = match output {
                None => {

                    //println!("Ignoring output not found for {:?}", input);
                    return Ok(())
                },
                Some(o) => o
            };


            let mut previous_tx_raw = Buffer::new(store.block_content.read(output));
            let previous_tx = Transaction::parse(&mut previous_tx_raw)?;

            let previous_tx_out = previous_tx.txs_out.get(input.prev_tx_out_idx as usize)
                .ok_or(TransactionError::OutputIndexNotFound)?;

            ffi::verify_script(previous_tx_out.pk_script, self.to_raw(), index as u32)
                .expect("We can't have script errors at this stage!");


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


impl<'a> Parse<'a> for TxInput<'a> {
    fn parse(buffer: &mut Buffer<'a>) -> Result<TxInput<'a>, EndOfBufferError> {

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

impl<'a> Parse<'a> for TxOutput<'a> {

    fn parse(buffer: &mut Buffer<'a>) -> Result<TxOutput<'a>, EndOfBufferError> {

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
