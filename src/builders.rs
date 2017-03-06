
//! Macros for used by tests to construct transactions and blocks


//! tx_builder must be invoked before tx and blk macros can be used
//! Usage:

//!
//!
//! ```
//! #[macro_use]
//! extern crate bitcrust_lib;
//!
//! fn main() {
//!
//!     tx_builder!(bld); // needed local "cache" to construct transactions
//!
//!     //let tx2 = tx!      (bld; a => b, c );
//! }
//!
//! ```
//!
//!


/// This macro setups a tx-builder variable
/// It initializes a hashmap that will be used to map output variables
#[macro_export]
macro_rules! tx_builder { ($b:ident ) =>
    (let mut $b: ::std::collections::HashMap<&str, Vec<u8>>    = ::std::collections::HashMap::new(); ) }



/// Creates a transaction
///
/// The letters used have no other meaning then to link outputs to inputs
///
/// Usage:
///
/// ```no_test
///
///    tx_builder!(bld);
///
///    // coinbase transaction:
///    let coinbase = tx!(bld, coinbase -> a);
///
///    // transaction with input a and output b,c
///    tx!(bld, a -> b,c);
///
/// ```
#[macro_export]
macro_rules! tx {

    ( $bld:ident;  $($input:ident),+     => $($output:ident $( ; $amount:expr )* ),+)
    =>
    ( {
        let b1 = $bld.clone();

        let txs_in : Vec<Vec<u8>>  = vec![ $(
            {
                if stringify!($input) == "coinbase" {

                    // create coinbase input
                    let mut txin_cb: Vec<u8> = Vec::new();
                    txin_cb.extend([0u8;32].iter());   // previous output = 0
                    txin_cb.extend([0xffu8,0xffu8,0xffu8,0xffu8].iter()); // index  = -1
                    txin_cb.extend([8u8].iter()); // script is 8 bytes

                    // 1 as "extra-nonce". This should actually be made unique
                    let bts = [1u8,0u8,0u8,0u8, 0u8,0u8,0u8,0u8];

                    txin_cb.extend(bts.iter());
                    txin_cb.extend([0u8;4].iter()); // sequence = 0

                    txin_cb
                }
                else {
                    b1.get(stringify ! ( $ input)).unwrap()
                        .into_iter().map(|&i| i).collect()
                }
            },
        )* ];

        let txs_out = vec![ $(
            {
                const DEFAULT_AMOUNT: u8 = 50_u8;
                let amount: u8 = * vec![$( $amount)*].get(0).unwrap_or(& DEFAULT_AMOUNT);
                let _ = stringify!($output);

                let outp = vec![
                    amount, 0u8,0u8,0u8,0u8,0u8,0u8,0u8,  /* the amount as little endian i64 */
                    1, 81 /* a one byte script consisting of OP_TRUE */
                ];

                outp
            }
            ,
        )* ];

        let mut tx = vec![
            0x01_u8, 0_u8, 0_u8, 0_u8, /* tx-version */
            txs_in.len() as u8 /* inputs */
        ];

        for ref tx_in in txs_in {
            tx.extend(tx_in.iter());
        }

        tx.push(txs_out.len() as u8);
        for ref tx_out in txs_out {
            tx.extend(tx_out.iter());
        }

        tx.extend([0u8;4].iter()); // locktime=0

        // now we have the tx we can create the input that references this
        let hash = ::hash::Hash32Buf::double_sha256(&tx);
        let mut _idx = 0_u8;
        $(
            {

                let mut txin: Vec<u8> = Vec::new();
                txin.extend(hash.as_ref().0.iter());
                txin.extend([_idx, 0u8, 0u8, 0u8].iter()); // index
                txin.extend([0u8;1].iter()); // script is empty
                txin.extend([0u8;4].iter()); // sequence = 0
                let _ = stringify!($output);
                $bld.insert(stringify!($output), txin);
            }
            _idx += 1;
        )*

        tx
    } );

}

macro_rules! genesis {
    () => (
    ::util::from_hex("0100000000000000000000000000000000000000000000000000000000000000\
                   000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa\
                   4b1e5e4a29ab5f49ffff001d1dac2b7c01010000000100000000000000000000\
                   00000000000000000000000000000000000000000000ffffffff4d04ffff001d\
                   0104455468652054696d65732030332f4a616e2f32303039204368616e63656c\
                   6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f75742066\
                   6f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe554827\
                   1967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4\
                   f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000")
    )
}


#[macro_export]
macro_rules! blk {
    ( prev = $prev:expr ; $( $txvec:expr ),* )
    =>
    (
    {
        let mut block: Vec<u8> = vec![1_u8,0_u8,0_u8,0_u8]; // block version = 1

        // hash of previous block
        let hash = ::hash::Hash32Buf::double_sha256(& $prev);
        block.extend(hash.as_ref().0.iter());

        // calculate merkle root
        let mut merkle = Vec::new();
        let mut count = 0_u8;
        $(
            merkle.push(::hash::Hash32Buf::double_sha256(& $txvec) );
            count += 1;
        )*

        block.extend(::merkle_tree::get_merkle_root(merkle).as_ref().0.iter());

        block.extend([0u8;4].iter()); // time = 0 for now
        block.extend([0u8;4].iter()); // bits = 0 for now
        block.extend([0u8;4].iter()); // nonce = 0 for now

        block.push(count);

        $(
            block.extend($txvec.iter());
        )*
        block
    }
    )
}

#[cfg(test)]
mod tests {

    use buffer::*;
    use transaction::Transaction;


    #[test]
    fn test_tx_builders() {

        tx_builder!(bld);

        let tx1 = tx!(bld; coinbase => a;12);
        println!("{:?}", tx1);

        let tx2 = tx!(bld; a     => b, c );
        let tx3 = tx!(bld; a,b   => c );

        println!("{:?}", tx2);

        let tx1p = Transaction::parse(&mut Buffer::new(&tx1)).unwrap();
        let tx2p = Transaction::parse(&mut Buffer::new(&tx2)).unwrap();
        let tx3p = Transaction::parse(&mut Buffer::new(&tx3)).unwrap();

        println!("{:?}", tx1p);
        println!("{:?}", tx2p);
        println!("{:?}", tx3p);

        assert_eq!(tx1p.txs_out.len(), 1);
        assert_eq!(tx1p.txs_in.len(), 1);

        assert_eq!(tx2p.txs_in.len(), 1);
        assert_eq!(tx2p.txs_out.len(), 2);
        assert_eq!(tx3p.txs_in.len(), 2);
        assert_eq!(tx3p.txs_out.len(), 1);

    }

    #[test]
    fn test_blk_builders() {
        tx_builder!(bld);

        let block0 = genesis![];

        let _ = blk![prev = block0;
            tx!(bld; coinbase => a;12),
            tx!(bld; a     => b, c ),
            tx!(bld; a,b   => c )
        ];

    }
}