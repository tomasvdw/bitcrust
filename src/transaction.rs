//! Transaction parsing and verification
//!
//!

use std::fmt;
use itertools::Itertools;

use buffer::*;
use hash::*;
use script::context;
use store::Store;
use ffi;

use store::TxPtr;
use store::Record;
use store::HashIndexGuard;
use store::TxIndex;
use store::FlatFileSet;

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
    AlreadyExists(TxPtr),

    VerifiedAndStored(TxPtr),

}

type TransactionResult<T> = Result<T, TransactionError>;

impl From<EndOfBufferError> for TransactionError {
    fn from(_: EndOfBufferError) -> TransactionError {

        TransactionError::UnexpectedEndOfData

    }
}

/// A transaction represents a parsed transaction
///
/// It always contains a reference to the buffer it was read from
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
            version:   i32::parse(buffer)?,
            txs_in:    Vec::parse(buffer)?,
            txs_out:   Vec::parse(buffer)?,
            lock_time: u32::parse(buffer)?,
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
    pub fn verify_backtracking_outputs(&self, tx_store: &mut FlatFileSet<TxPtr>, inputs: &Vec<TxPtr>) {


        for input_ptr in inputs.into_iter() {

            debug_assert!(input_ptr.is_guard());

            // read tx from disk
            let mut tx_raw   = Buffer::new(tx_store.read(*input_ptr));

            let tx           = Transaction::parse(&mut tx_raw).
                    expect("Invalid tx data in database");

            // find indixes
            let input_index  = input_ptr.get_input_index() as usize;
            let ref input    = tx.txs_in[input_index];
            let output_index = input.prev_tx_out_idx as usize;

            ffi::verify_script(self.txs_out[output_index].pk_script, tx.to_raw(), input_index as u32)
                .expect("TODO: Handle script error without panic");


            // TODO: verify_amount here
        }
    }

    /// Gets the output records referenced by the inputs of this tx
    ///
    /// Uses Record placeholder for outputs not found
    pub fn get_output_records(&self, store: &mut Store) -> Vec<Record> {

        self.txs_in.iter()

            .filter(|tx_in| !tx_in.prev_tx_out.is_null())
            .map(|input| {

                store.tx_index
                    .get(input.prev_tx_out)
                    .iter()
                    .find(|ptr| !ptr.is_guard())
                    .map_or(Record::new_unmatched_input(), |ptr| Record::new_output(*ptr, input.prev_tx_out_idx))
            })
            .collect()
    }

    /// Verifies and stores the transaction in the transaction_store and index
    pub fn verify_and_store(&self,
                            tx_index: &mut TxIndex,
                            tx_store: &mut FlatFileSet<TxPtr>,
                            hash: Hash32) -> TransactionResult<TransactionOk> {

        self.verify_syntax()?;

        loop {

            // First see if it already exists
            let existing_ptrs = tx_index.get(hash);

            if existing_ptrs
                .iter()
                .any(|p| !p.is_guard()) {

                assert_eq!(existing_ptrs.len(), 1);

                return Ok(TransactionOk::AlreadyExists(existing_ptrs[0]))
            }

            // existing_ptrs (if any) are now inputs that are waiting for this transactions
            // they need to be verified
            self.verify_backtracking_outputs(tx_store, &existing_ptrs);

            // store & verify
            let ptr      = tx_store.write(self.to_raw());


            if !self.is_coinbase() {
                self.verify_input_scripts(tx_index, tx_store, ptr)?;
            }

            // Store reference in the hash_index.
            // This may fail if since ^^ get_ptr new dependent txs were added,
            // in which case we must try again.
            if tx_index.set(hash, ptr, &existing_ptrs) {

                return Ok(TransactionOk::VerifiedAndStored(ptr))
            }
        }

    }


    /// Finds the outputs corresponding to the inputs and verify the scripts
    pub fn verify_input_scripts(&self,
                                tx_index: &mut TxIndex,
                                tx_store: &mut FlatFileSet<TxPtr>,
                                tx_ptr: TxPtr) -> TransactionResult<()> {

        for (index, input) in self.txs_in.iter().enumerate() {


            let output = tx_index.get_or_set(input.prev_tx_out,
                                                     tx_ptr.to_input(index as u16 ));
            let output = match output {
                None => {

                    // We can't find the transaction this input is pointing to
                    // Oddly, this is perfectly fine; we just postpone script validation
                    // until that transaction comes in.
                    //
                    // ^^ get_or_set has placed appropriate guards in the hash_index

                    continue;
                },
                Some(o) => o
            };


            let mut previous_tx_raw = Buffer::new(tx_store.read(output));
            let previous_tx = Transaction::parse(&mut previous_tx_raw)?;

            let previous_tx_out = previous_tx.txs_out.get(input.prev_tx_out_idx as usize)
                .ok_or(TransactionError::OutputIndexNotFound)?;


            ffi::verify_script(previous_tx_out.pk_script, self.to_raw(), index as u32)
                .expect("TODO: Handle script error more gracefully");

            // TODO: verify_amount here
        }

        Ok(())
    }


}


/// Transaction input
pub struct TxInput<'a> {
    pub prev_tx_out:     Hash32<'a>,
    pub prev_tx_out_idx: u32,
    script:          &'a[u8],
    sequence:        u32,
}


impl<'a> Parse<'a> for TxInput<'a> {
    fn parse(buffer: &mut Buffer<'a>) -> Result<TxInput<'a>, EndOfBufferError> {

        Ok(TxInput {
            prev_tx_out:     try!(Hash32::parse(buffer)),
            prev_tx_out_idx: try!(u32::parse(buffer)),
            script:          try!(buffer.parse_compact_size_bytes()),
            sequence:        try!(u32::parse(buffer))
        })

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
    use util::*;
    use super::*;
    use buffer;
    use buffer::Parse;


    #[test]
    fn test_parse_tx() {
        let tx_hex = "010000000236b01007488776b78a1e6cf59b72e2236ba378d42761eba901\
                      5d8bc243c7d9f0000000008a47304402206018582ef1405fbf9f08b71a2a\
                      b61b6a93caf713d50879573d42f87463c645b3022030e274e52bd107f604\
                      894d75968a47be340d633d3c38e5310fddf700ade244d501410475645fe0\
                      50491f9593348ba511bba43f91e02719cb604fc1f73ef57a5d8507d22b58\
                      20c9bf3065b1ac3543fc212b50218f7a4bf32aa664f84f336efa79660111\
                      ffffffff36b01007488776b78a1e6cf59b72e2236ba378d42761eba9015d\
                      8bc243c7d9f0010000008b4830450221009dd6581d23a64173cd5fd04c99\
                      dfc9b3581708c361433dfd340e7f5ea07e0eb1022042d08810307a92af6e\
                      f8c9ed748547f48e05b549f7bc004395b7c12879f94b2b014104607e781f\
                      9d685959b2009a4e35b7d2f240d8b515d59d2ddaa51b82f21ef56372f892\
                      39b836446bec96f5b66dee75425a38af3185610410e20655a9d333503f3b\
                      ffffffff0280f0fa02000000001976a914bb42487be1aae97292b5ecda5e\
                      66ba59d004d83088ac80f0fa02000000001976a914c3813e88eeddeba7de\
                      fe159bf9df3f210652571c88ac00000000";

        let slice = &from_hex(tx_hex);
        let mut buf = buffer::Buffer::new(slice);

        let tx = Transaction::parse(&mut buf);

        let _ = format!("{:?}", tx);
    }
}
