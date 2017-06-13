/// Implements the block_add procedure


/*

Block storing is a tricky part; blocks are stored in the spend-tree and referenced in the
 hash-index

 This can go out of order:  For instance consider 5 blocks added in the order A, B, D, E, C
 (for this pseudocode, each block has the previous letter as prev_block_hash)

 We show the some pseudocode for actions on hashindex (hi) and spend-tree (st),
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


    If this ^^ is only confusing; please ignore and check the code at connect_block below
*/

use std::time::Instant;
use hash::*;
use util::*;
use buffer::*;
use rayon::prelude::*;
use slog ;

use store::Store;
use transaction;
use transaction::TransactionStats;
use merkle_tree;
use block::*;
use store::Record;
use store::BlockPtr;
use store::HashIndexGuard;
use store::tips;

type BlockResult<T> = Result<T, BlockError>;

// minimum number of hashes to use parallel hashing
const PARALLEL_HASHING_THRESHOLD: usize = 10;


/// Returns true if the given hash is the hash of a genesis block
fn is_genesis_block(hash: Hash32) -> bool {
    const HASH_GENESIS: &'static str =
        "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";

    let genesis = Hash32Buf::from_slice(&from_hex_rev(HASH_GENESIS));

    genesis.as_ref() == hash
}

// Connects two blocks (A,B) in the spend-tree and then stores the hash of B in the hash-index
// Connecting the blocks will verify double-spends
//
// It can be that another block C is also waiting for B; this will trigger their connection (B,C) too
// and maybe (C,D)... etcetera
//
// This would be much cleaner to do recursively, but this can exhaust the stack, we use a loop with
// a to_do for connections.
fn connect_block(
    store:           &mut Store,
    this_block_hash: Hash32,
    previous_block:  Option<BlockPtr>,
    this_block:      BlockPtr)

    -> BlockResult<()>
{
    trace!(store.logger, "Connect block";
        "this_hash"  =>  format!("{:?}", this_block_hash),
        "prev_end"   =>  format!("{:?}", previous_block),
        "this_start" =>  format!("{:?}", this_block)
    );

    // we lay connections between the end of one block and the start of this_block
    // previous_block is None only for genesis

    // A to_do item
    #[derive(Debug)]
    struct Connection {
        block:         BlockPtr,
        block_hash:    Hash32Buf,
        solved_guards: Vec<BlockPtr>
    }

    // connect this block if not genesis...
    if let Some(previous_block) = previous_block {
        store.spend_tree.connect_block( &mut store.spend_index, & store.logger, previous_block, this_block) ?;
    }

    // The to_do list contains blocks that are connected to their previous but not yet added to the
    // block-index. Start with the one we just connected;
    let mut todo = vec![Connection {
        block:         this_block,
        block_hash:    this_block_hash.as_buf(),
        solved_guards: vec![]
    }];


    while let Some(conn) = todo.pop() {

        trace!(store.logger, "Connect block - set-hash-loop";
            "conn"  => format!("{:?}",   conn));

        // if we can store this hash we can move to the next one
        if store.block_index.set(conn.block_hash.as_ref(), conn.block.to_non_guard(), &conn.solved_guards, false) {
            trace!(store.logger, "Connect block - set-hash-loop - ok");
            continue;
        }

        // we couldn't add this block to the index, which means some block is awaiting connection
        let guards = store.block_index.get(conn.block_hash.as_ref());

        if guards.iter().any(|ptr| !ptr.is_guard()) {
            // this is not a guard, this is _this_ block. This means the block
            // was added by a concurrent user; will do just fine.
            trace!(store.logger, "Connect block - set-hash-loop - concurrent add");
            continue;
        }

        // we'll try this one again the next iteration
        todo.push(Connection {
            block: conn.block,
            block_hash: conn.block_hash,
            solved_guards: guards.clone()
        });

        // and connect pending blocks first
        for ptr in guards {
            if conn.solved_guards.contains(&ptr) {
                continue;
            }

            let hash = store.get_block_hash(ptr);

            trace!(store.logger, "Connect block - set-hash-loop";
                "guard" => format!("{:?}", ptr),
                "hash" => format!("{:?}",   hash),
                "conn"  => format!("{:?}",   conn));


            store.spend_tree.revolve_orphan_pointers(
                &mut store.transactions,
                &mut store.tx_index,
                ptr
            );

            store.spend_tree.connect_block(&mut store.spend_index, &store.logger, conn.block, ptr)?;


            todo.push(Connection {
                block: ptr,
                block_hash: hash,
                solved_guards: vec![]

            });

        }
    }


    Ok(())
}



/// Returns true if the block is already stored
fn block_exists(store: & mut Store, block_hash: Hash32) -> bool {
    let ptr = store.block_index.get(block_hash);

    ptr.iter().any( | ptr | !ptr.is_guard())

}


/// Verifies and stores the transactions in the block.
/// This does not yet check the order
/// Also verifies the merkle_root & the amounts
///
/// Returns a list fileptrs to the transactions
///
fn verify_and_store_transactions(store: &mut Store, block: &Block) -> BlockResult<Vec<Record>> {

    let timer = ::std::time::Instant::now();

    // We use chunked parallelization because otherwise we need to clone() the stores on each
    // iteration
    // The main procedure here is to hash and call verify_and_store for each transaction
    let chunks: Vec<_> =
        block.txs.par_chunks(PARALLEL_HASHING_THRESHOLD).map(|chunk_tx| {

        let len = chunk_tx.len();
        let p0  = Instant::now();

        let mut hashes:  Vec<Hash32Buf> = Vec::with_capacity(len); // accurate
        let mut records: Vec<Record>    = Vec::with_capacity(len * 3); // estimate (guessing 2 in per tx)

        let ref mut tx_index = &mut store.tx_index.clone();
        let ref mut tx_store = &mut store.transactions.clone();

        let cloning = Instant::now() - p0;
        let mut chunk_stats =   TransactionStats { cloning: cloning, ..Default::default() };

        for tx in chunk_tx {

            let p1  = Instant::now();

            let hash = Hash32Buf::double_sha256(tx.to_raw());
            hashes.push(hash);

            let p2  = Instant::now();

            let res = tx.verify_and_store(tx_index, tx_store, store.initial_sync, hash.as_ref()).unwrap();

            // AlreadyExists and VerifiedAndStored are both ok here;
            // Extract the TxPtr and the stats
            let (ptr,stats) = match res {
                transaction::TransactionOk::VerifiedAndStored {ptr, stats}  => (ptr, stats),
                transaction::TransactionOk::AlreadyExists     {ptr } => (ptr, Default::default())
            };

            records.push(Record::new_transaction(ptr));
            for rec in tx.get_output_records(tx_index) {
                records.push(rec);
            }

            chunk_stats = chunk_stats + stats;
            chunk_stats.hashing = chunk_stats.hashing + (p2 - p1);
        }
        (chunk_stats, (hashes, records))
    }).collect();


    let p3 = Instant::now();

    // split
    let (stats, hashes_records): (Vec<_>, Vec<_>) = chunks.into_iter().unzip();
    let (hashes, records):       (Vec<_>, Vec<_>) = hashes_records.into_iter().unzip();

    // flatten
    let hashes:  Vec<Hash32Buf> = hashes.into_iter().flat_map(|x| x).collect();
    let records: Vec<Record>    = records.into_iter().flat_map(|x| x).collect();
    let mut stats: TransactionStats = stats.into_iter().sum();

    let rec_count: usize = records.len();
    let tx_count: usize  = hashes.len();


    // check merkle roots
    let calculated_merkle_root = merkle_tree::get_merkle_root(hashes);
    block.verify_merkle_root(calculated_merkle_root.as_ref()).unwrap();
    stats.merkle = Instant::now() - p3;

    let elapsed : usize = timer.elapsed().as_secs() as usize * 1000 +
        timer.elapsed().subsec_nanos() as usize / 1_000_000 as usize;

    if rec_count > 0 && rec_count-tx_count > 0 {
        info!(store.logger, "add_block - transactions done";
            "input_avg_ms" => elapsed as f64 / (rec_count - tx_count) as f64,
            "tx_avg_ms"    => elapsed as f64 / tx_count as f64,
            "tx_stats"     => format!("{:?}", stats));
    }
    Ok(records)
}


/// Validates and stores a block;
///
/// For now; this panics on invalids; but all underlying functions
/// propagate problems to jere
pub fn add_block(store: &mut Store, buffer: &[u8]) {


    let block_logger = slog::Logger::new(&store.logger, o!());
    info!(block_logger, "add_block - start");

    // parse & hash block
    let block      = Block::new(buffer) .unwrap();
    let block_hash = Hash32Buf::double_sha256( block.header.to_raw());

    info!(block_logger, "add_block - hashed"; "hash" => format!("{:?}", block_hash));

    // already done?
    if block_exists(store, block_hash.as_ref()) {
        info!(store.logger, "add_block - Block already exists");
        return;
    }

    block.verify_block_size().unwrap();

    // check and store the transactions in block_content and check the merkle_root
    let spend_tree_ptrs = verify_and_store_transactions(store, &block).unwrap();

    // store the blockheader in block_content
    let block_header_ptr = store.block_headers.write( &block.header.to_raw());

    // we also store the txcount, although we only use it for a reindex benchmark
    let _ = store.block_headers.write_fixed( &block.txs.len());

    // store the block in the spend_tree

    let block_ptr       = store.spend_tree.store_block(block_header_ptr, spend_tree_ptrs);


    if is_genesis_block(block_hash.as_ref()) {

        info ! (block_logger, "add_block - storing genesis block");

        // there is None previous block, but we call connect_block anyway as this will also
        // connect to next blocks if they are already in
        connect_block(store, block_hash.as_ref(), None, block_ptr).unwrap();
    }
    else {

        // we retrieve the pointer to the end of the previous block from the hash-index
        // if it is not yet in, this hash will be inserted as a guard-block
        let previous_block = store.block_index.get_or_set( block.header.prev_hash, block_ptr.to_guard());

        info! (block_logger, "add_block - block-index done";
            "previous" => format!("{:?}", block.header.prev_hash),
            "ptr" => format!("{:?}", previous_block));

        // if it is in, we will connect
        if let Some(previous_block) = previous_block {

            connect_block(store, block_hash.as_ref(), Some(previous_block), block_ptr).unwrap();
        }

    }

    tips::add_tip(&store.tips, block_hash, 0, 0);


    // TODO verify amounts
    // TODO verify PoW
    // TODO verify header-syntax

    info!(block_logger, "add_block - done");
}



#[cfg(test)]
mod tests {

    use store;
    use super::*;


    #[test]
    fn test_clone_into_thread() {

        // some rust experimenting; ignore

        struct X { data: Vec<u32> };
        impl Clone for X {
            fn clone(&self) -> X { X { data: self.data.clone() } }
        }
        let x = X { data: vec![1,2,3,4,5,6,7,8,9,10]};

        let y = vec![10,11,12,13,14,15,16,17,18,19,20];

        y.into_par_iter().map(|z| {
            let u = &mut x.clone(); // is this possible?
            println!("{:?} {:?} ", z, u.data); u.data.push(2);
        });
    }


    #[test]
    fn test_block_simple() {

        let mut store = store::Store::new(& test_cfg!());

        tx_builder!(bld);

        let block0 = genesis!();

        let block1 = blk!(prev = block0;
            tx!(bld; coinbase => b;11 ),
            tx!(bld; b => c,e )
        );

        let block2 = blk!(prev = block1;
            tx!(bld; coinbase => f;12 ),
            tx!(bld; c => g )
        );

        add_block(&mut store, &block0);
        add_block(&mut store, &block1);
        add_block(&mut store, &block2);

    }

    #[test]
    fn test_blocks_reorder() {

        let mut store = store::Store::new(& test_cfg!());

        tx_builder!(bld);

        let block0 = genesis!();
        let block1 = blk!(prev = block0;
            tx!(bld; coinbase => b;11 ),
            tx!(bld; b => c,e )
        );

        let block2 = blk!(prev = block1;
            tx!(bld; coinbase => f;12 ),
            tx!(bld; c => g )
        );

        //        add_block(&mut store, &block0);

        println!("block1 = {:?}", block1);
        //println!("tx1 = {:?}", ::hash::Hash32Buf::double_sha256(&tx1));
        add_block(&mut store, &block0);
        add_block(&mut store, &block2);
        add_block(&mut store, &block1);

    }

}

