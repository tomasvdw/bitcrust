//! Transaction parsing and verification
//!
//!



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
    AlreadyExists(FilePtr),

    VerifiedAndStored(FilePtr),

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



    /// Reverse script validation
    ///
    /// This checks the passed input-ptrs are valid against the corresponding output of self
    ///
    pub fn verify_backtracking_outputs(&self, store: &mut Store, inputs: &Vec<FilePtr>) {


        for input_ptr in inputs {
            let mut _m = store.metrics.start("block.tx.backtrack");

            // read tx from disk
            let mut tx_raw   = Buffer::new(store.block_content.read(*input_ptr));
            let tx           = Transaction::parse(&mut tx_raw).
                    expect("Invalid tx data in database");

            let input_index  = input_ptr.input_index() as usize;
            let ref input    = tx.txs_in[input_index];
            let output_index = input.prev_tx_out_idx as usize;

            let mut _m2 = store.metrics.start("block.tx.backtrack.verify");

            ffi::verify_script(self.txs_out[output_index].pk_script, tx.to_raw(), input_index as u32)
                .expect("TODO: Handle script error without panic");


            // TODO: verify_amount here
        }
    }

    pub fn get_output_fileptrs(&self) -> Vec<FilePtr> {
        Vec::new()
    }

    /// Verifies and stores
    pub fn verify_and_store(&self, store: &mut Store) -> TransactionResult<(TransactionOk)> {

        self.verify_syntax()?;

        let hash_buf = hash::Hash32Buf::double_sha256(self.to_raw());
        let _        = hash_buf.as_ref();


        loop {

            // First see if it already exists
            let existing_ptrs = store.hash_index.get_ptr(hash_buf.as_ref());

            if existing_ptrs
                .iter()
                .any(|p| p.is_transaction()) {

                assert_eq!(existing_ptrs.len(), 1);

                return Ok(TransactionOk::AlreadyExists(existing_ptrs[0]))
            }

            // existing_ptrs are now inputs that are waiting for this transactions
            // they need to be verified
            self.verify_backtracking_outputs(store, &existing_ptrs);


            let ptr      = store.block_content.write(self.to_raw());

            if !self.is_coinbase() {
                self.verify_input_scripts(store, ptr)?;
            }


            // Store self in the hash_index.
            // This may fail if since ^^ loop new dependent txs were added,
            // in which case we must try again.
            if store.hash_index.set_tx_ptr(hash_buf.as_ref(), ptr, existing_ptrs) {

                return Ok(TransactionOk::VerifiedAndStored(ptr))
            }
        }

    }


    /// Finds the outputs corresponding to the inputs and verify the scripts
    pub fn verify_input_scripts(&self, store: &mut Store, tx_ptr: FilePtr) -> TransactionResult<()> {

        let mut _m = store.metrics.start("verify_scoped_scripts");
        _m.set_ticker(self.txs_in.len());

        for (index, input) in self.txs_in.iter().enumerate() {


            let output = store.hash_index.get_tx_for_output(input.prev_tx_out,
                tx_ptr.as_input(index as u32 ));
            let output = match output {
                None => {

                    // We can't find the transaction this input is pointing to
                    // Oddly, this is perfectly fine; we just postpone script validation
                    // Until that transaction comes in
                    // ^^ get_tx_for_output has placed apropriate guards in the hash_index

                    continue;
                },
                Some(o) => o
            };


            let mut previous_tx_raw = Buffer::new(store.block_content.read(output));
            let previous_tx = Transaction::parse(&mut previous_tx_raw)?;

            let previous_tx_out = previous_tx.txs_out.get(input.prev_tx_out_idx as usize)
                .ok_or(TransactionError::OutputIndexNotFound)?;

            let mut _m2 = store.metrics.start("verify_scripts");
            ffi::verify_script(previous_tx_out.pk_script, self.to_raw(), index as u32)
                .expect("TODO: Handle script error without panic");

            // TODO: verify_amount here
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


/// tx-tests are external

#[cfg(test)]
mod tests {
    extern crate rustc_serialize;


    #[test]
    fn test_parse_tx() {}
}
