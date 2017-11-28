# hashstore
Key/Value store optimized for storing blockchain data

## Abstract

This key/value store can be used as write/purge only store (no updates) and is suitable to store transactions and blockheaders.

It has rather specific properties

* Keys are 32-byte hashes
* Values are arbitrary length byte sequences
* Append only. No updates of existsing data.
* Allows atoming receiving and storing of dependencies
* Very fast writes. Very fast MRU reads. Slow old reads.

## Architecture

The storage file starts with a header followed by a large hash-table of filepointers to values. After the hashtable, the values are appended.

The values form a LIFO linked list to resolve hash collisions.

## Usage 

    use hashstore::*;
    
    // open or create `myfile` with a 24-bits hashtable (= 8*(2^24) bytes).
    let hs = HashStore::new("myfile", 24).unwrap();
    
    // fast append (timestamp=10)
    hs.set(mykey1, myvalue1, 10).unwrap();

    assert_eq!(hs.get(mykey1, SearchDepth::FullSearch).unwrap(), Some(myvalue1));
    

## Timestamps

Values are added with a "timestamp", which is just an opaque increasing number. On retrieving values, the search can be limited to 
after specific timestamp. For instance, for blockchains, heights can be used as timestamps, and only the last X blocks can be searched
to allow deprecation of transactions.

    // May fail as we're searching after mykey1's timestamp
    assert_eq!(hs.get(mykey1, SearchDepth::After(20)).unwrap(), Some(myvalue1));
    
    
## Dependencies

Values can be retrieved as a required dependency of another. If A is retrieved as dependency of B, and A isn't found,
a dependency A->B is atomically inserted. This will block insertion of A until the dependency is resolved.

This can be used to process out-of-order (orphan) transactions and blocks.

    // failed lookup mykey2 doesn't exist
    assert!(hs.get_dependency(mykey2, mykey1, 20).unwrap().is_none());
    
    // failed set of mykey2
    assert!(hs.set(mykey2, myvalue2, vec![], SearchDepth::FullSearch, 30).unwrap().is_none());
    
    // ... verify mykey1->mykey2
    // now set and declare dependecy as met
    assert!(hs.set(mykey2, myvalue2, vec![mykey2], SearchDepth::FullSearch, 30).unwrap().is_none());
    
    
