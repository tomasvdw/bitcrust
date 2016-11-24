[![Build Status](https://travis-ci.org/tomasvdw/bitcrust.svg?branch=master)](https://travis-ci.org/tomasvdw/bitcrust)
[![Coverage Status](https://coveralls.io/repos/github/tomasvdw/bitcrust/badge.svg)](https://coveralls.io/github/tomasvdw/bitcrust)

# Bitcrust


**Bitcrust** is a full software suite for bitcoin in development. Currently the focus and progress is on 
 _[bitcrust-db](bitcrust-lib/README.md)_: a storage-engine which uses a novel approach to block-storage to 
provides a high performance, _lock-free_ concurrent R/W access, and 
fully parallel block verification.   


## Table of Contents

- [Install](#install)
- [Components](#components)
- [Contribute](#contribute)

## Install

Bitcrust depends on libbitcoinconsensus which can be created by building 
[bitcoin-core](https://github.com/bitcoin/bitcoin) from source per its instructions.


After that  you can build and test the bitcrust libraries with

```
cargo test
```


## Components

Bitcrust is planned to have the following components:

* _[bitcrust-db](bitcrust-lib/README.md)_ Bitcrust-db is the library component that verfies and stores blocks 
and transactions. 
* _bitcrust-net (planned)_ P2P bitcoin protocol implementation
* _bitcrust-mapreduce (planned)_ A scriptable indexing framework
* _bitcrust-monitor (planned)_ A scriptable indexing framework
* _bitcrust-node (planned)_  
* _bitcrust-wallet (planned)_


![Components of bitcrust](https://cdn.rawgit.com/tomasvdw/bitcrust/master/doc/spent-tree1.svg "Dependencies")


## Contribute

Help is very much wanted; if you have a fix or improvement, please submit a pull request.
 
 Or contact the maintainer for suggestions on work to be done. 
