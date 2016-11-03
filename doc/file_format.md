
# Flatfiles

Various data is stored in *flatfiles*. A datatype is associate with a path and a prefix and the data is stored
as sequential records in files of the format path/prefix-XXXX.

Here XXXX is a hex signed file-sequence number and the maximum file size; the files are of fixed size to allow 
memory mapped access. The sequence number is signed such that files can be pruning or cleaned to lower numbers. 

Each file starts with a 16 byte header:

4 bytes magic number
4 bytes write-pointer; this is the location where the next transanction will be written in the file
(both in native endian)
8 bytes reserved.

In normal operation the files are append-only and writes and reads occur lock-free. Writes will first increase the 
write-pointer (using atomic CAS) and then write the record at the location of the previous write pointer.

Records in the files can be identified with 48-bit pointers (16-bit fileno and 32-bit filepos).

## Transactions

Transactions are stored in flatfiles `transactions/tx-XXXX` 

Transanctions are prefixed with a 4-byte length and written in network format.

## Blockheaders

Blcokheaders are stored  'headers/hdr-XXXX'. Without prefix as they are fixed lenth.

## Spenttree

Files with the name 'stree/st-XXXX' contain the spent-tree; Records are 16 byte long.

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


 
# Index

The index is used to map hashes to fileptrs. Currently LMDB is used.