
# Flatfiles

Various data is stored in *flatfilesets* [(src)](flatfileset.rs). 
A datatype is associate with a path and a prefix and the data is stored
as sequential records in files of the format path/prefix-XXXX.

Here XXXX is a hex signed file-sequence number and the maximum file size; the files are of fixed size to allow 
memory mapped access. The sequence number is signed such that files can be pruning or cleaned to lower numbers. 

Each file starts with a 16 byte header:

4 bytes magic number
4 bytes write-pointer; this is the location where the next transanction will be written in the file
(both in native endian)
8 bytes reserved.

In most operation the files are append-only and writes and reads occur lock-free. Writes will first increase the 
write-pointer (using atomic compare-and-swap) and then write the record at the location of the previous write pointer.

Data in the files can be identified with pointer that implements the *flatfileptr* Trait. 

This provides a file number (max 16-bit) and a file offset (max 64 bit),
but the different filesets use different semantics as seen in [TxPtr](txptr.rs) 
and [BlockHeaderPtr](blockheaderptr.rs).


## Block Content


Transactions and blockheaders are stored in flatfiles `transactions/tx-XXXX` 
and `headers/bh-0000`. 

Both are prefixed with a 4-byte length and written in network format. 
Blockheaders are not length-prefixed, and also stored in network format.


## Hash Index

Hashes of blocks and transactions are looked in two hash-indexes [(src)](hash_index.rs). 
They are stored in flat_files `tx-index/ti-XXXX` and `block-index/bi-XXXX`. The first 64mb of the flatfileset is 
the root node; it is a hash-table to resolve the first 24-bits of a hash. This points to a append-only unbalanced 
binary tree.
 
This set-up ensures a nice temporal locality of reference, as only the 64mb root node and recent tree-branches are 
needed in RAM.

## Spend-tree

Files with the name `spend-tree/st-XXXX` [(src)](spend_tree/mod.rs) contain the spend-tree; Records are 16 byte long.


A block is added to the spend_tree by first adding a start-of-block record, then for each transanction a transaction record
and for each input of the transaction an output-spend record. At the end an end-of-block record is added.
   
Each output-spend is verified by scanning backwards using the parent pointers, to ensure that the same output-spend is 
not found before the spend transaction record is found. 
  
This ensures that 
  
* the transaction that is being spent exists on this branch of the tree and
* the output of the transaction was not yet spent.

## Spend-index

The spend index `spend-index/si-XXXX`  [(src)](spend_index.rs) catches seeks earlier in the chain
and uses a simple concurrent bit-index to look them up.
