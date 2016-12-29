//!
//! Bitcoin block
//!
//! Handles block-level validation and storage

/*

Block storing is a tricky part; blocks are stored in the spent-tree and referenced in the
 hash-index

 This can go out of order:  For instance consider 5 blocks added in the order A, B, D, E, C
 Each block has the previous letter as prev_block_hash

 We show the some pseudocode for actions on hashindex (hi) and spent-tree (st)

 A:
    st.store_block(A)
    fn do_connect(null,A) =
      (prev = null, no get_or_set)
      hi.set(A)

 B:
    st.store_block(B)
    hi.get_or_set(A, guard[B]) returns A
    fn do_connect(A,B) =
      st.connect_block(A,B)
      hi.set(B)


 D:
    st.store_block(D)
    hi.get_or_set(C, guard[D]) return none & adds guard[D] at hash C

 E:
   st.store_block(E)
   hi.get_or_set(D, guard[E]) return none & adds guard[E] at hash D

 C:
   st.store_block(C)
   hi.get_or_set(B, guard[C]) returns B
   fn do_connect(B,C) =
      st.connect_block(B,C)
      hi.set(C) -> fail due to guard([D])

        loop {
            hi.get(C) -> return guard[D]
            do_connect(C,D)
            hi.set(C, guard([D]) -> fail or break
        }

  Now let us consider what happens when C' is inserted *during* processing of B. Copyied from B above:
  B:
    st.store_block(B)
    hi.get_or_set(A, guard[B]) returns A
    fn do_connect(A,B) =
      st.connect_block(A,B)
        < here C' is inserted as B -> guard[C']
      hi.set(B) -> fail due to guard[C']
      loop {
            hi.get(B) -> return guard[C']
            do_connect(B,C')
            hi.set(B, guard([C']) -> success!
        }


*/



use std::convert;

use buffer::*;
use hash::*;
use util::*;

use store::SpendingError;

use merkle_tree::MerkleTree;

use transaction::{Transaction, TransactionError};
use store::Store;

use store::RecordPtr;
use store::fileptr::FilePtr;


#[derive(Debug)]
pub enum BlockError {
    NoTransanctions,
    FirstNotCoinbase,
    DoubleCoinbase,

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
    pub txcount: usize,
    pub txs:    &'a[u8],

    /// the full block as slice
    raw:        &'a[u8],
}

/// BlockHeader represents the header of a block
#[derive(Debug)]
pub struct BlockHeader<'a> {

    version:     u32,
    prev_hash:   Hash32<'a>,
    merkle_root: Hash32<'a>,
    time:        u32,
    bits:        u32,
    nonce:       u32,

    raw:         &'a[u8],
}


fn is_genesis_block(hash: Hash32) -> bool {
    const HASH_GENESIS: &'static str = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";
    let genesis = Hash32Buf::from_slice(&from_hex_rev(HASH_GENESIS));

    genesis.as_ref() == hash
}

// Connects two blocks (A,B) in the spent-tree and then stores the hash of B in the hash-index
// this may cause itself being called recursively for (B,C) if needed
fn connect_and_store_block(
         store:                 &mut Store,
         this_block_hash:       Hash32,
         previous_block_end:    Option<RecordPtr>,
         this_block_start:      RecordPtr)

        -> BlockResult<()>
{

    // connect the blocks
    let this_block_end = if let Some(p) = previous_block_end {
        store.spent_tree.connect_block(p, this_block_start)?
    }
    else {
        // .. unless this is a genesis block; we just find the end
        store.spent_tree.find_end(this_block_start)
    };



    // we can now store the reference in the hash-index unless there are guards that need to be solved
    let mut solved_guards: Vec<FilePtr> = vec![];
    while !store.hash_index.set(this_block_hash, this_block_end.ptr, &solved_guards) {

        let guards = store.hash_index.get(this_block_hash);

        if guards.iter().any(|ptr| ptr.is_blockheader()) {
            // this is not a guard, this is _this_ block. It seems the block
            // was added by a concurrent user; will do fine.
            return Ok(());
        }

        for ptr in guards.iter() {
            if solved_guards.contains(ptr) {
                continue;
            }

            let hash = store.get_block_hash(*ptr);

            // call self recursively; the guard block has this as previous
            connect_and_store_block(store, hash.as_ref(), Some(this_block_end), RecordPtr::new(*ptr))?;

            solved_guards.push(*ptr);
        }
    }

    Ok(())

    /*    st.connect_block(A,B)
        hi.set(B) -> fail due to guard[C']
        loop {
        hi.get(B) -> return guard[C']
        do_connect(B,C')
        hi.set(B, guard([C']) -> success!
        }
*/
}




impl<'a> Block<'a> {


    /// Parses the block from a raw blob
    ///
    /// The transactions will not be parsed yet, and simply stored as a slice
    pub fn new(raw: &'a[u8]) -> Result<Block<'a>, EndOfBufferError> {

        let mut buf = Buffer::new(raw);

        Ok(Block {
            raw:     raw,
            header:  BlockHeader::parse(&mut buf)?,
            txcount: buf.parse_compact_size()?,
            txs:     buf.inner

        })
    }



    /// Verifies the spending order and stores the block
    ///
    /// This assumes all transactions have already been (script) verified
    pub fn verify_and_store(&self, store: &mut Store, transactions: Vec<FilePtr>) -> BlockResult<()> {

        //let _m = store.metrics.start("block.spenttree.store");

        let block_hash = Hash32Buf::double_sha256(self.header.to_raw());

        info!(store.logger, "verify_and_store"; "status" => "start", "block" => format!("{:?}", block_hash));

        // see if it exists
        let ptr = store.hash_index.get(block_hash.as_ref());

        if ptr.iter().any(|ptr| ptr.is_blockheader()) {

            // this block is already in
            // TODO; distinct ok-result
            println!("Already exists");
            return Ok(())
        }


        // let's store the blockheader in block_content
        let blockheader_ptr = store.block_content.write_blockheader(&self.header);


        // now we store the block in the spent_tree
        let block_ptr = store.spent_tree.store_block(blockheader_ptr, transactions);

        if is_genesis_block(block_hash.as_ref()) {

            info!(store.logger, "verify_and_store - storing genesis block");

            connect_and_store_block(store, block_hash.as_ref(), None, block_ptr.start)?;
        }
        else {

            // we retrieve the pointer to the end of the previous block from the hash-index
            let previous_end = store.hash_index.get_or_set(self.header.prev_hash, block_ptr.start.ptr.to_guardblock());

            if let Some(previous_end) = previous_end {

                info!(store.logger, "verify_and_store - storing block");
                connect_and_store_block(store, block_hash.as_ref(), Some(RecordPtr::new(previous_end)), block_ptr.start)?;
            }

        }





        Ok(())
    }

    pub fn verify_merkle_root(&self, calculated_merkle_root: Hash32<'a>) -> BlockResult<()> {

        if self.header.merkle_root != calculated_merkle_root {
            Err(BlockError::IncorrectMerkleRoot)
        }
        else {
            Ok(())
        }

    }


    /// Parses each transaction in the block, and executes the callback for each
    ///
    /// This will also check whether only the first transaction is a coinbase
    /// and the rest is not, so that the transactions can be uniformly handled
    ///
    pub fn process_transactions<F>(&self, mut callback: F) -> BlockResult<()>
        where F : FnMut(Transaction<'a>) -> BlockResult<()> {

        if self.txs.is_empty() {
            return Err(BlockError::NoTransanctions);
        }


        let mut buffer = Buffer::new(self.txs);

        // check if the first is coinbase
        let first_tx = Transaction::parse(&mut buffer)?;
        if !first_tx.is_coinbase() {
            return Err(BlockError::FirstNotCoinbase);
        }

        callback(first_tx)?;

        for _ in 1..self.txcount {
            let tx = Transaction::parse(&mut buffer)?;

            // all but first may not be coinbase
            if tx.is_coinbase() {
                return Err(BlockError::DoubleCoinbase);
            }

            callback(tx)?;
        }

        if buffer.len() > 0  {

            // Buffer not fully consumed
            Err(BlockError::UnexpectedEndOfBuffer)
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
            prev_hash:   try!(Hash32::parse(buffer)),
            merkle_root: try!(Hash32::parse(buffer)),
            time:        try!(u32::parse(buffer)),
            bits:        try!(u32::parse(buffer)),
            nonce:       try!(u32::parse(buffer)),

            raw:         buffer.consumed_since(org_buffer).inner
        })
    }
}

impl<'a> ToRaw<'a> for BlockHeader<'a> {
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

    fn test_spenttree() {

    }



    /*
    #[test]
    fn test_blockheader_store()  {
        let hex = "0100000000000000000000000000000000000000000000000000000000000000\
                   000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa\
                   4b1e5e4a29ab5f49ffff001d1dac2b7c01010000000100000000000000000000\
                   00000000000000000000000000000000000000000000ffffffff4d04ffff001d\
                   0104455468652054696d65732030332f4a616e2f32303039204368616e63656c\
                   6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f75742066\
                   6f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe554827\
                   1967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4\
                   f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";
                   
        let blk_bytes = hex.from_hex().unwrap();     
        let blk       = super::Block::read(&mut Cursor::new(&blk_bytes)).unwrap();
              
        let path = Path::new("test-lmdb");
        let env = EnvBuilder::new().open(&path, 0o777).unwrap();
        
        let db_handle = env.get_default_db(DbFlags::empty()).unwrap();
        let txn = env.new_transaction().unwrap();
        {
            let xx        = unsafe { mem::transmute(&blk) };
        
            let db = txn.bind(&db_handle); // get a database bound to this transaction
            db.set(&"test", &xx);
            
        }
        txn.commit().unwrap();
        let reader = env.get_reader().unwrap();
        let db = reader.bind(&db_handle);
        //assert_eq!(1,0);
            
        let name = db.get::<&str>(&"test").unwrap();
        assert_eq!(&name, &"aa");
    }
    */
}
