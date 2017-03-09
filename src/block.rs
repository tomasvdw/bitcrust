//!
//! Bitcoin block
//!
//! Handles block-level validation and storage




use std::convert;

use buffer::*;
use hash::*;

use store::SpendingError;


use transaction::{Transaction, TransactionError};

const MAX_BLOCK_SIZE: usize =  1_000_000;


#[derive(Debug)]
pub enum BlockError {
    NoTransanctions,
    FirstNotCoinbase,
    DoubleCoinbase,

    BlockTooLarge,

    IncorrectMerkleRoot,

    UnexpectedEndOfBuffer,


    SpendingError(SpendingError),
    TransactionError(TransactionError)
}

impl convert::From<EndOfBufferError> for BlockError {

    fn from(_: EndOfBufferError) -> BlockError {

        BlockError::UnexpectedEndOfBuffer
    }

}

// wrap transaction errors as block errors
impl convert::From<TransactionError> for BlockError {

    fn from(inner: TransactionError) -> BlockError {

        BlockError::TransactionError(inner)
    }

}

impl convert::From<SpendingError> for BlockError {
    fn from(inner: SpendingError) -> BlockError {
        BlockError::SpendingError(inner)
    }
}


type BlockResult<T> = Result<T, BlockError>;




/// Parsed block
///
/// The transactions are not yet parsed and referenced as a slice
#[derive(Debug)]
pub struct Block<'a> {


    pub header: BlockHeader<'a>,
    pub txs:    Vec<Transaction<'a>>,

    /// the full block as slice
    raw:        &'a[u8],
}

/// BlockHeader represents the header of a block
#[derive(Debug)]
pub struct BlockHeader<'a> {

    version:     u32,
    pub prev_hash:   Hash32<'a>,    // TODO should not be pub
    merkle_root: Hash32<'a>,
    time:        u32,
    bits:        u32,
    nonce:       u32,

    raw:         &'a[u8],
}



impl<'a> Block<'a> {
    /// Parses the block from a raw blob
    ///
    /// The transactions will not be parsed yet, and simply stored as a slice
    pub fn new(raw: &'a [u8]) -> Result<Block<'a>, EndOfBufferError> {
        let mut buf = Buffer::new(raw);

        Ok(Block {
            raw: raw,
            header: BlockHeader::parse(&mut buf)?,
            txs: Vec::parse(&mut buf)?
        })
    }


    /// Compares the given merkle root against the headers merkle root
    pub fn verify_merkle_root(&self, calculated_merkle_root: Hash32<'a>) -> BlockResult<()> {
        if self.header.merkle_root != calculated_merkle_root {
            Err(BlockError::IncorrectMerkleRoot)
        } else {
            Ok(())
        }
    }

    /// Verifies the size of the block
    pub fn verify_block_size(&self) -> BlockResult<()> {

        if self.to_raw().len() > MAX_BLOCK_SIZE {
            Err(BlockError::BlockTooLarge)
        }
        else {
            Ok(())
        }
    }
}



impl<'a> Parse<'a> for BlockHeader<'a> {

    /// Parses the block-header
    fn parse(buffer: &mut Buffer<'a>) -> Result<BlockHeader<'a>, EndOfBufferError> {

        let org_buffer = *buffer;

        Ok(BlockHeader {
            version:     u32::parse(buffer)?,
            prev_hash:   Hash32::parse(buffer)?,
            merkle_root: Hash32::parse(buffer)?,
            time:        u32::parse(buffer)?,
            bits:        u32::parse(buffer)?,
            nonce:       u32::parse(buffer)?,

            raw:         buffer.consumed_since(org_buffer).inner
        })
    }
}

impl<'a> ToRaw<'a> for BlockHeader<'a> {
    fn to_raw(&self) -> &[u8] {
        self.raw
    }
}

impl<'a> ToRaw<'a> for Block<'a> {
    fn to_raw(&self) -> &[u8] {
        self.raw
    }
}




#[cfg(test)]
mod tests {


    use super::*;
    use util::*;
    use buffer::Parse;
    use buffer;
    use transaction;

    const BLOCK0: &'static str = "0100000000000000000000000000000000000000000000000000000000000000\
                   000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa\
                   4b1e5e4a29ab5f49ffff001d1dac2b7c01010000000100000000000000000000\
                   00000000000000000000000000000000000000000000ffffffff4d04ffff001d\
                   0104455468652054696d65732030332f4a616e2f32303039204368616e63656c\
                   6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f75742066\
                   6f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe554827\
                   1967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4\
                   f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";

    #[test]
    fn test_blockheader_read()  {



        let slice = &from_hex(BLOCK0);
        let mut buf = buffer::Buffer::new(slice);

        let hdr = BlockHeader::parse(&mut buf).unwrap();
        let txs: Vec<transaction::Transaction> = Vec::parse(&mut buf).unwrap();

        for tx in &txs {
            tx.verify_syntax().unwrap();
        }
        
        assert_eq!(hdr.version, 1);
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].txs_in.len(), 1);
        assert_eq!(txs[0].txs_out.len(), 1);

    }

}
