# Bitcurst-db source

These sources provide the library that stores, verifies and retrieves blocks and 
transactions. It is intended to be used by bitcoin network node software, mining software and 
utility tools.

The interface itself is not yet complete; it only provides an [add_block](lib.rs#L71-74) method.
 
This procedure is handled in [add_block.rs](add_block.rs) and uses a [store](store/) to access the filesystem.

Concurrent access is allowed by using multiple Store instances both from different threads as well as from 
different processes; multiple processes can  concurrently read and write to the same datafiles.
 
Script validation is handled by libbitcoinconsensus via [ffi](ffi.rs)

The library uses a *deserialize-only* model supported by [buffer](buffer.rs); a reference to the binary block is kept 
such that serialization is not needed.