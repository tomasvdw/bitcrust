[![Build Status](https://travis-ci.org/tomasvdw/bitcrust.svg?branch=master)](https://travis-ci.org/tomasvdw/bitcrust)
[![Coverage Status](https://coveralls.io/repos/github/tomasvdw/bitcrust/badge.svg)](https://coveralls.io/github/tomasvdw/bitcrust)

# Bitcrust-lib


**Bitcrust** is a full software suite for bitcoin in development. This repository 
contains the core library which provides a _shared_, _lock-free_ concurrent database various components.   


## Table of Contents

- [Install](#install)
- [Internals](#internals)
- [Contribute](#contribute)

## Install

The library is work-in-process and not yet published on cargo.

To run, you first need to build and install [libbitcoin-consensus](https://github.com/libbitcoin/libbitcoin-consensus) by following 
the instructions. This may require building [libsecp256k1](https://github.com/bitcoin-core/secp256k1) 

After that  you can build and test bitcrust-lib with

```
cargo test
```


## Internals

**bitcrust-lib** uses compare-and-swap semantics to provide lock-free
concurrency. Some information on the data-structures is available in 
[doc](doc/file_format.md)

## Contribute

Help is very much wanted; if you have a fix or improvement, please submit a pull request.
 
 Or contact the maintainer for suggestions on work to be done. 
