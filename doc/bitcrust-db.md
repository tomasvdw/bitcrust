
# Bitcrust-db

- [Introduction](#introduction)
- [Block content](#block-content)
- [Spend tree instead of a UTXO-set](#spend-tree)
- [Concurrent block validation](#concurrent-validation)

## Introduction

Bitcoin-core uses a linearized model of the block tree. On-disk, only a main chain 
is stored to ensure that there is always one authorative UTXO set.

Bitcrust uses a tree-structure and indexes spends instead of unspends. This has several key
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
 

## Spend tree

When transactions are stored, only their scripts are validated. When blocks come in,
we need to verify for each input of each transaction whether the referenced output exists and is unspend before
this one. Instead of a UTXO-set we use the spend-tree.

This is a table (stored in a flatfileset) consisting of four types of records: block-start, block-end, transactions and spends.

Here we see a spend-tree with two blocks, where the third transaction has one input referencing the output of the first transaction (purple).


![Spend tree example 1](https://cdn.rawgit.com/tomasvdw/bitcrust/master/doc/spend-tree1.svg "Spend-tree example")

If another block (2b) comes with the same block 1 as parent this can be simply appended with the proper pointer:  

![Spend tree example 2](https://cdn.rawgit.com/tomasvdw/bitcrust/master/doc/spend-tree2.svg "Spend-tree example 2")

The rule for verification is simple: A spend (purple) record can only be added if, when browsing back through the 
records, we will find the corresponding transaction before we find the same spend. This ensures both the existence 
of the referenced transaction, and it being unspend.
    
Obviously, with hundreds of millions transactions, simply scanning won't do. 
This is where the spent-index comes in to play. This is a very compact bit-index of spends that
lags behind the tips and serves as a broom wagon. When scanning the spend-tree reaches the broom-wagon,
the order can be verifies with two simple lookups.
 

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
  