
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

This provides a file number (max 16-bit) and a file offset (max 32 bit).


## Block Content


Transactions and blockheaders are stored in flatfiles `transactions/tx-XXXX` 
and `headers/bh-0000`. 

Both are prefixed with a 4-byte length and written in network format. 
Blockheaders are not length-prefixed, and also stored in network format.


## Hash Index

Hashes of blocks and transactions are looked in two hash-indexes [(src)](hash_index.rs). 
It is stored in flat_files `hash_index/ht-XXXX` . The first 64mb of the flatfileset is 
the root node; it is a hash-table to resolve the first 24-bits of a hash. This points to a append-only unbalanced 
binary tree.
 
This set-up ensures a nice temporal locality of reference, as only the 64mb root node and recent tree-branches are 
needed in RAM.

## Spenttree

Files with the name 'spent_tree/st-XXXX' (src in progress) contain the spent-tree; Records are 16 byte long.

Three types of records exist:

* blockheader:   (with a 48-bit pointer to a blockheader record)
* transanction:  (with a 48-bit pointer to a transaction record)
* output-spent:  (with a 48-bit pointer to transaction record and the index to the output within the tx.)

The remaining space of the record is used for pointers to other spenttree records. Each record points at least to a 
parent: either implicitely the previous record, or an explicit pointer.

A block is added to the stree by first adding a blockheader record, then for each transanction a transaction record
and for each input of the transaction an output-spent record.
   
Each output-spent is verified by scanning backwards using the parent pointers, to ensure that the same output-spent is 
not found before the spent transaction record is found. 
  
This ensures that 
  
A. the transaction that is being spent exists on this branch of the tree and 
B. the output of the transaction was not yet spent.
