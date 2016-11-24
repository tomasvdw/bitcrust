
# Bitcrust-db

- [Introduction](#introduction)
- [Block content](#block_content)
- [Spent tree instead of a UTXO-set](#spent_tree)
- [Concurrent block validation](#spent_tree)

## Introduction

Bitcoin-core uses a linearized model of the block tree. On-disk, only a main chain 
is stored to ensure that there is always one authorative UTXO set.

Bitcrust uses a tree-structure and stores spents instead of unspents. This has several key
advantages in terms of performance, simplicity and most importantly, concurrency. 

The first results are very positive and show that this approach addresses Core's major bottlenecks of
block verification.

## Block content

Transactions are stored on disk after they have been verified. 
Unconfirmed transactions are not necessarily kept in memory, as 
they to be bad canditdates to pollute precious RAM space. After the scripts
are verified, the transaction contents is only rarely needed.

Blockheaders are stored in the same data-files, and both transactions
 and blocks are referenced by 48-bit file-pointers.

For more details, check the [store](src/store/) documentation
 

## Spent tree

When transactions are stored, only their scripts are validated. When blocks come in,
we need to verify for each input of each transaction whether the referenced output exists and is unspent before
this one. For this we use the spent-tree.

This is a table (stored in a flatfileset) consisting of three types of records: blocks, transactions and spents.

Here we see a spent-tree with two blocks, where the 3rd transaction has one input referencing the 1st transaction (purple).


![Spent tree example 1](https://cdn.rawgit.com/tomasvdw/bitcrust/master/doc/spent-tree1.svg "Spent-tree example")

If another block (2b) comes with the same block 1 as parent this can be simply appended with the proper pointer:  

![Spent tree example 2](https://cdn.rawgit.com/tomasvdw/bitcrust/master/doc/spent-tree2.svg "Spent-tree example 2")

The rule for verification is simple. A spent (purple) record can only be added if, when browser back through the 
file, we will find the corresponding transaction before we find the same spent.
    
Obviously, with hundreds of millions transactions, simply scanning won't do. This is where we 
take advantage of the fact that these records are filepointers, and therefore *roughly* ordered. This allows us to create
a *loose skip tree*: every records contains a set of "highway" pointers that point skip over records depnding on the value searched for.
   
![Spent tree example 3](https://cdn.rawgit.com/tomasvdw/bitcrust/master/doc/spent-tree3.svg "Spent-tree example 3")
      