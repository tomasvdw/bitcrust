/// Implements the block_add procedure


/*

Block storing is a tricky part; blocks are stored in the spent-tree and referenced in the
 hash-index

 This can go out of order:  For instance consider 5 blocks added in the order A, B, D, E, C
 (for this pseudocode, each block has the previous letter as prev_block_hash)

 We show the some pseudocode for actions on hashindex (hi) and spent-tree (st),
 for the insertion of this sequence

 insert A:
    st.store_block(A)
    fn do_connect(null,A) =
      (prev = null, no get_or_set)
      hi.set(A)

 insert B:
    st.store_block(B)
    hi.get_or_set(A, guard[B]) returns A
    fn do_connect(A,B) =
      st.connect_block(A,B)
      hi.set(B)


 insert D:
    st.store_block(D)
    hi.get_or_set(C, guard[D]) return none & adds guard[D] at hash C

 insert E:
   st.store_block(E)
   hi.get_or_set(D, guard[E]) return none & adds guard[E] at hash D

 insert C:
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

use hash::*;
use util::*;
use buffer::*;

#[macro_use]
use slog ;

use store::Store;
use transaction;
use merkle_tree;
use block::*;

use store::SpendingError;
use store::RecordPtr;
use store::fileptr::FilePtr;


type BlockResult<T> = Result<T, BlockError>;


/// Returns true if the given hash is the hash of a genesis block
fn is_genesis_block(hash: Hash32) -> bool {
    const HASH_GENESIS: &'static str =
    "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";

    let genesis = Hash32Buf::from_slice(&from_hex_rev(HASH_GENESIS));

    genesis.as_ref() == hash
}

// Connects two blocks (A,B) in the spent-tree and then stores the hash of B in the hash-index
// Connecting the blocks will verify double-spents
//
// It can be that another block C is also waiting for B; this will trigger their connection (B,C) too
// and maybe (C,D)... etcetera
//
// This would be neat to do recursively, but this could exhaust the stack , we use a loop with
// a to do for connections.
fn connect_block(
    store: &mut Store,
    this_block_hash: Hash32,
    previous_block_end: Option<RecordPtr>,
    this_block_start: RecordPtr)

    -> BlockResult<()>
{
    info!(store.logger, "Connect block";
        "this_hash"  => format!("{:?}",   this_block_hash),
        "prev_end"   =>  format!("{:?}", previous_block_end),
        "this_start" =>  format!("{:?}", this_block_start)
    );

    // we lay connections between the end of one block and the start of this_block
    // prev_end is None only for genesis
    #[derive(Debug)]
    struct Connection {
        this_end:      RecordPtr,
        this_hash:     Hash32Buf,
        solved_guards: Vec<FilePtr>
    }

    // connect first block ...
    let this_end = if let Some(prev_end) = previous_block_end {

        store.spent_tree.connect_block(&store.logger, prev_end, this_block_start)?
    } else {

        // .. unless this is a genesis block; we just find the end
        store.spent_tree.find_end(this_block_start)
    };


    let mut todo = vec![Connection {
        this_end:      this_end,
        this_hash:     this_block_hash.as_buf(),
        solved_guards: vec![]
    }];


    while let Some(conn) = todo.pop() {

        info!(store.logger, "Connect block - set-hash-loop";
            "conn"  => format!("{:?}",   conn));

        // if we can store this hash we can move to the next one
        if store.hash_index.set(conn.this_hash.as_ref(), conn.this_end.ptr.to_block(), &conn.solved_guards) {
            info!(store.logger, "Connect block - set-hash-loop - ok");
            continue;
        }


        let guards = store.hash_index.get(conn.this_hash.as_ref());

        if guards.clone().iter().any(|ptr| ptr.is_blockheader()) {
            // this is not a guard, this is _this_ block. It seems the block
            // was added by a concurrent user; will do fine.
            info!(store.logger, "Connect block - set-hash-loop - concurrent add");
            continue;
        }

        //let guards = store.hash_index.get(conn.this_hash);
        todo.push(Connection {
            this_end: conn.this_end,
            this_hash: conn.this_hash,
            solved_guards: guards.clone()
        });

        //let guards = store.hash_index.get(conn.this_hash.as_buf());

        for ptr in guards.iter() {
            if conn.solved_guards.contains(ptr) {
                continue;
            }

            let hash = store.get_block_hash(*ptr);

            info!(store.logger, "Connect block - set-hash-loop";
                "guard" => format!("{:?}", ptr),
                "hash" => format!("{:?}",   hash),
                "conn"  => format!("{:?}",   conn));



            store.spent_tree.revolve_orphan_pointers(
                &mut store.block_content,
                &mut store.hash_index,
              RecordPtr::new(*ptr));

            let guard_end = store.spent_tree.connect_block(&store.logger, conn.this_end, RecordPtr::new(*ptr))?;


            todo.push(Connection {
                this_end: guard_end,
                this_hash: hash,
                solved_guards: vec![]

            });

        }
    }


    Ok(())
}



/// Returns true if the block is already stored
fn block_exists(store: & mut Store, block_hash: Hash32) -> bool {
    let ptr = store.hash_index.get(block_hash);

    ptr.iter().any( | ptr | ptr.is_blockheader())

}


/// Verifies and stores the transactions in the block.
/// Also verifies the merkle_root & the amounts
///
/// Returns a list fileptrs to the transactions
///
fn verify_and_store_transactions(store: &mut Store, block: &Block) -> BlockResult<Vec<FilePtr>> {

    let mut total_amount = 0_u64;
    let mut result_ptrs  = Vec::new();
    let mut merkle       = merkle_tree::MerkleTree::new();

    block.process_transactions(|tx| {

        total_amount += 1;

        let res = tx.verify_and_store(store).unwrap();

        // AlreadyExists and VerifiedAndStored are both ok here
        let ptr = match res {
            transaction::TransactionOk::AlreadyExists(ptr) => ptr,
            transaction::TransactionOk::VerifiedAndStored(ptr) => ptr
        };

        result_ptrs.push(ptr);
        result_ptrs.append(&mut tx.get_output_fileptrs(store));

        merkle.add_hash(Hash32Buf::double_sha256(tx.to_raw()).as_ref());

        Ok(())

    })?;

    let calculated_merkle_root = merkle.get_merkle_root();
    block.verify_merkle_root(calculated_merkle_root.as_ref()).unwrap();

    Ok(result_ptrs)
}


/// Validates and stores a block;
///
pub fn add_block(store: &mut Store, buffer: &[u8]) {



    // parse
    let block      = Block::new(buffer).unwrap();
    let block_hash = Hash32Buf::double_sha256( block.header.to_raw());

    let block_logger = slog::Logger::new(&store.logger, o!("hash" => format!("{:?}", block_hash)));
    info!(block_logger, "start");

    // already done?
    if block_exists(store, block_hash.as_ref()) {
        info!(store.logger, "Block already exists");
        return;
    }

    // check and store the transactions in block_content and check the merkle_root
    let spent_tree_ptrs = verify_and_store_transactions(store, &block).unwrap();

    info!(block_logger, "transactions done");

    // store the blockheader in block_content
    let blockheader_ptr = store.block_content.write_blockheader( &block.header);

    // store the block in the spent_tree
    let block_ptr       = store.spent_tree.store_block(blockheader_ptr, spent_tree_ptrs);


    if is_genesis_block(block_hash.as_ref()) {

        info ! (block_logger, "verify_and_store - storing genesis block");

        // there is None previous block, but we call connect_block anyway as this will also
        // connect to next blocks if they are already in
        connect_block(store, block_hash.as_ref(), None, block_ptr.start).unwrap();
    }
    else {

        // we retrieve the pointer to the end of the previous block from the hash-index
        // if it is not yet in, this hash will be inserted as a guard-block
        let previous_end = store.hash_index.get_or_set( block.header.prev_hash, block_ptr.start.ptr.to_guardblock());

        info ! (block_logger, "verify_and_store";
            "previous" => format!("{:?}", block.header.prev_hash),
            "ptr" => format!("{:?}", previous_end));

        // if it is in, we will connect
        if let Some(previous_end) = previous_end {

            connect_block(store, block_hash.as_ref(), Some(RecordPtr::new(previous_end)), block_ptr.start).unwrap();
        }

    }

    // TODO verify amounts
    // TODO verify PoW
    // TODO verify header-syntax

    info!(block_logger, "done");
}



#[cfg(test)]
mod tests {



}

