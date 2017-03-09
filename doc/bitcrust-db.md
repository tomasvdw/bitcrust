
# Bitcrust-db

- [Introduction](#introduction)
- [Block content](#block-content)
- [Spent tree instead of a UTXO-set](#spent-tree)
- [Concurrent block validation](#concurrent-validation)

## Introduction

Bitcoin-core uses a linearized model of the block tree. On-disk, only a main chain 
is stored to ensure that there is always one authorative UTXO set.

Bitcrust uses a tree-structure and indexes spents instead of unspents. This has several key
advantages in terms of performance, minimal memory requirement, simplicity and most importantly, concurrency. 

The first results are very positive and show that this approach addresses Core's major bottlenecks in
block verification.

## Block content

Transactions are stored on disk after they have been verified even when not yet in a block.
 This means they are only written once. 
Unconfirmed transactions are not necessarily kept in memory, as 
they only pollute precious RAM space. After the scripts
are verified and transactions are relayed, the transaction contents is only rarely needed.

Blockheaders are stored in the same data-files (to ensure rough ordering), and both transactions
 and blocks are referenced by 48-bit file-pointers.

For more details, check the [store](../src/store/) documentation
 

## Spent tree

When transactions are stored, only their scripts are validated. When blocks come in,
we need to verify for each input of each transaction whether the referenced output exists and is unspent before
this one. Instead of a UTXO-set we use the spent-tree.

This is a table (stored in a flatfileset) consisting of three types of records: blocks, transactions and spents.

Here we see a spent-tree with two blocks, where the third transaction has one input referencing the output of the first transaction (purple).


![Spent tree example 1](https://cdn.rawgit.com/tomasvdw/bitcrust/master/doc/spent-tree1.svg "Spent-tree example")

If another block (2b) comes with the same block 1 as parent this can be simply appended with the proper pointer:  

![Spent tree example 2](https://cdn.rawgit.com/tomasvdw/bitcrust/master/doc/spent-tree2.svg "Spent-tree example 2")

The rule for verification is simple: A spent (purple) record can only be added if, when browsing back through the 
records, we will find the corresponding transaction before we find the same spent. This ensures both the existence 
of the referenced transaction, and it being unspent.
    
Obviously, with hundreds of millions transactions, simply scanning won't do. This is where we 
take advantage of the fact that these records are filepointers to the [block content](#block-content) fileset, and therefore *roughly* ordered. This allows us to create
a *loose skip tree*. Similarly to a skip list, each record contains a set of "highway" pointers that skip over records depending on the value searched for:
   
![Spent tree example 3](https://cdn.rawgit.com/tomasvdw/bitcrust/master/doc/spent-tree3.svg "Spent-tree example 3")
      
As the vast majority of spents refer to recent transactions, such skip tree can reduce the average number of nodes traversed per lookup to  about 100.

Developers with knowledge about B-Trees and hash-tables may start to giggle at such high number of nodes per lookup, but they would be forgetting the major gains, which makes this 
approach outperform other structures:

* Superior locality of reference. As the majority of lookups is in the end of the tree, the accessed memory usually fits in the CPU cache. 
This in sheer contrast with the UTXO set which is randomly scattered. 
* The data structure is append-only, absolving the need for transactional adding and removal of UTXO pointers. Adding to the tree 
is done concurrently using CAS-semantics.
* The tree structure is maintained on disk. This absolves the need for reorgs and for 
writing undo-information. A reorg in bitcrust is simply the pointing to a different tip.
* Parallel block validation. As there is no "main chain" at the storage level, concurrent blocks can
be verified in parallel.

## Concurrent validation

One major cause for sleepless nights for nodes and miners is the idea of a _toxic block_ or transaction. 
The flexibility of bitcoin allows one to create blocks that will cause a huge amount of time and effort to be processed and can thereby choke or even crash other
  nodes and miners, especially smaller ones. A simple example being a non-segwit transaction with a huge amount of inputs which abuses quadratic hashing.
  
  By its architecture, bitcrust is insensitive for such malice; blocks and transaction can be processed fully in parallel: 

![Parallel validation](https://cdn.rawgit.com/tomasvdw/bitcrust/master/doc/parallel-validation.svg "Parellel validation")

The long-lasting validation of block A does not at any point
 block the validation of block B, C and D.
 
 The actual orphaning and breaking of the connection (as well as deprioritizing) 
 can be implemented using the same per peer cost/benefit analysis as other DOS protection.
  