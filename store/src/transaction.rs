
use std::fmt;

use network_encoding::*;
use hash;
use hash::*;
use itertools::*;

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


/// A transaction represents a decoded transaction
///
/// It always contains a reference to the buffer it was read from
#[derive(Debug)]
pub struct Transaction<'a> {
    pub version:   i32,
    pub txs_in:    Vec<TxInput<'a>>,
    pub txs_out:   Vec<TxOutput<'a>>,
    pub lock_time: u32,
}


impl<'a> NetworkEncoding<'a> for Transaction<'a> {

    /// Parses the raw bytes into individual fields
    /// and perform basic syntax checks
    fn decode(buffer: &mut Buffer<'a>) -> Result<Transaction<'a>, EndOfBufferError> {

        Ok(Transaction {
            version:   i32::decode(buffer)?,
            txs_in:    Vec::decode(buffer)?,
            txs_out:   Vec::decode(buffer)?,
            lock_time: u32::decode(buffer)?,
        })
    }

    fn encode(&self, buffer: &mut Vec<u8>) {
        self.version.encode(buffer);
        self.txs_in.encode(buffer);
        self.txs_out.encode(buffer);
        self.lock_time.encode(buffer);
    }
}


impl<'a> Transaction<'a> {

    /// Performs basic syntax checks on the transaction
    pub fn verify_syntax(&self) -> Result<(), TransactionError> {


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

        self.txs_in.len() == 1 && *self.txs_in[0].prev_tx_out == [0;32]
    }



}

/// Transaction input
pub struct TxInput<'a> {
    pub prev_tx_out: &'a Hash,
    pub prev_tx_out_idx: u32,
    pub script:          &'a[u8],
    pub sequence:        u32,
}


impl<'a> NetworkEncoding<'a> for TxInput<'a> {
    fn decode(buffer: &mut Buffer<'a>) -> Result<TxInput<'a>, EndOfBufferError> {
        type HashRef<'a> = &'a Hash;
        type Bytes<'a> = &'a [u8];

        Ok(TxInput {
            prev_tx_out:     HashRef::decode(buffer)?,
            prev_tx_out_idx: u32::decode(buffer)?,
            script:          Bytes::decode(buffer)?,
            sequence:        u32::decode(buffer)?
        })
    }

    fn encode(&self, buffer: &mut Vec<u8>) {
        self.prev_tx_out.encode(buffer);
        self.prev_tx_out_idx.encode(buffer);
        self.script.encode(buffer);
        self.sequence.encode(buffer);
    }
}

#[derive(PartialEq)]
pub struct TxOutput<'a> {
    pub value:     i64,
    pub pk_script: &'a[u8]
}

impl<'a> NetworkEncoding<'a> for TxOutput<'a> {
    fn decode(buffer: &mut Buffer<'a>) -> Result<TxOutput<'a>, EndOfBufferError> {
        Ok(TxOutput {
            value: i64::decode(buffer)?,
            pk_script: buffer.decode_compact_size_bytes()?
        })
    }

    fn encode(&self, buffer: &mut Vec<u8>) {
        self.value.encode(buffer);
        self.pk_script.encode(buffer);
    }
}


impl<'a> fmt::Debug for TxInput<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Prev-TX:{:?}, idx={:?}, seq={:?} script=",
                    self.prev_tx_out,
                    self.prev_tx_out_idx,
                    self.sequence)

    }
}

impl<'a> fmt::Debug for TxOutput<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        write!(fmt, "v:{:?} ", self.value)
    }
}


/// tx-tests are external

#[cfg(test)]
mod tests {
    use util::*;
    use super::*;
    use network_encoding;


    #[test]
    fn test_decode_tx() {
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
        let mut buf = network_encoding::Buffer::new(slice);

        let tx = Transaction::decode(&mut buf);

        let _ = format!("{:?}", tx);
    }
}
