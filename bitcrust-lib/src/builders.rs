
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
//!     //let tx1 = coinbase!(bld;   => a);
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


#[macro_export]
macro_rules! coinbase {
    ($bld:ident ; => $output:ident $( ; $amount:expr )* )
    =>
    (
    {
        const default_amount: u8 = 50_u8;
        let amount: u8 = * vec![$( $amount)*].get(0).unwrap_or(& default_amount);

        // first output is block reward
        let outp1 = vec![
            amount, 0u8,0u8,0u8,0u8,0u8,0u8,0u8,  /* the amount as little endian i64 */
            1, 81 /* a one byte script consisting of OP_TRUE */
        ];


        // we count the number of items in bld, and at that as OP_RETURN to
        // output 2 to make this unique
        // TODO support blockheight in this output
        let bytes_bld_len: [u8;8] = unsafe { ::std::mem::transmute::<_,[u8;8]>( $bld.len()) };
        let mut outp2 = vec![
            0u8, 0u8,0u8,0u8,0u8,0u8,0u8,0u8,  /* the amount as little endian i64 */
            10, 106, 8 /* 10-bytes script, OP_RETURN, OP_PUSHDATA(8) */

        ];

        outp2.extend(bytes_bld_len.iter());

        let mut tx = vec![
            0x01_u8, 0_u8, 0_u8, 0_u8, /* tx-version */
            0x00, /* 0-inputs */
            0x02 /* 1-output */

        ];

        tx.extend(outp1.iter());
        tx.extend(outp2.iter());
        tx.extend([0u8;4].iter()); // locktime=0

        // now we have the tx we can create the input that references this
        let hash = ::hash::Hash32Buf::double_sha256(&tx);

        let mut txin: Vec<u8> = Vec::new();
        txin.extend(hash.as_ref().0.iter());
        txin.extend([0u8;4].iter()); // index=0
        txin.extend([0u8;1].iter()); // script is empty
        txin.extend([0u8;4].iter()); // sequence = 0

        $bld.insert(stringify!($output), txin);

        tx
    }
    )
}

/// Creates a transaction
#[macro_export]
macro_rules! tx {

    ( $bld:ident;  $($input:ident),* => $($output:ident $( ; $amount:expr )* ),+)
    =>
    ( {
        let b1 = $bld.clone();

        let txs_in  = vec![ $(
            b1.get(stringify!($input)).unwrap(),
        )* ];
        let txs_out = vec![ $(
            {
                const default_amount: u8 = 50_u8;
                let amount: u8 = * vec![$( $amount)*].get(0).unwrap_or(& default_amount);

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
        let mut idx = 0_u8;
        $(
            {

                let mut txin: Vec<u8> = Vec::new();
                txin.extend(hash.as_ref().0.iter());
                txin.extend([idx, 0u8, 0u8, 0u8].iter()); // index
                txin.extend([0u8;1].iter()); // script is empty
                txin.extend([0u8;4].iter()); // sequence = 0
                let x = stringify!($output);
                $bld.insert(stringify!($output), txin);
            }
            idx += 1;
        )*

        tx
    } );

}

#[macro_export]
macro_rules! blk {
    ( $($txvec:ident),* )
=>
    (
    {

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

        let tx1 = coinbase!(bld; => a);
        let tx2 = tx!(bld; a     => b, c );
        let tx3 = tx!(bld; a,b   => c );

        println!("{:?}", tx2);

        let tx1p = Transaction::parse(&mut Buffer::new(&tx1)).unwrap();
        let tx2p = Transaction::parse(&mut Buffer::new(&tx2)).unwrap();
        let tx3p = Transaction::parse(&mut Buffer::new(&tx3)).unwrap();

        assert_eq!(tx1p.txs_out.len(), 2);
        assert_eq!(tx1p.txs_in.len(), 0);

        assert_eq!(tx2p.txs_in.len(), 1);
        assert_eq!(tx2p.txs_out.len(), 2);
        assert_eq!(tx3p.txs_in.len(), 2);
        assert_eq!(tx3p.txs_out.len(), 1);

    }

    #[test]
    fn test_blk_builders() {

    }
}