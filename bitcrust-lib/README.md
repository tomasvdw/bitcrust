
# Bitcrust-db

- [Introduction](#introduction)
- [Block content](#block_content)
- [Spent tree instead of a UTXO-set](#spent_tree)
- [Concurrent block validation](#spent_tree)

## Introduction

Bitcoin-core uses a linearized model of the block tree. On-disk, only a main chain 
is stored to ensure that there is always one authorative UTXO set.

Bitcrust uses a tree-structure to store the spent-information. This has several key
advantages as explained below.


## Block content

Transactions are stored on disk after they have been verified. 
Unconfirmed transactions are not necessarily kept in memory, as 
they to be bad canditdates to pollute precious RAM space. After the scripts
are checked they are no longer needed.

Blockheaders are stored in the same data-files, and both transactions
 and blocks are referenced by 48-bit file-pointers.
 
## Spent tree

When transactions are stored, only their scripts are validated. When blocks come in,
  we need to verify for each input of each transaction whether the referenced output exists and is unspent before
   this one. For this we use the spent-tree.
   
   This is a data-structure consisting of three types of records: blocks, transactions and spents.
   
   Here we see a spent-tree with two blocks, where the 3rd transaction has one input referencing the 1st transaction.
   
   
   ![Spent tree example](https://github.com/tomasvdw/bitcrust/blob/master/doc/spent-tree1.svg "Spent-tree example")