//! Transaction store
//! This is now by experiment split in two files
//! to see if this improves output-reading performance
//!
//!
//! This is a bit messy for now as we're not using typed access to these store
//! as they are still WIP

use std::mem;

use buffer::*;
use config;
use store::flatfileset::FlatFileSet;
use store::TxPtr;

use transaction::Transaction;
use transaction::TransactionError;
use store::flatfileset::FlatFilePtr;

const MB:                 u64 = 1024 * 1024;
const FILE_SIZE:          u64 = 2 * 1024 * MB ;
const MAX_CONTENT_SIZE:   u64 = FILE_SIZE - 10 * MB ;


/// Transaction store
pub struct Transactions {

    // part1 stores the header& inputs
    transactions1: FlatFileSet<TxPtr>,

    // part2 stores the outputs
    transactions2: FlatFileSet<TxPtr>,

}

impl Clone for Transactions {

    // Explicit cloning can be used to allow concurrent access.
    fn clone(&self) -> Transactions {

        Transactions {

            transactions1: self.transactions1.clone(),
            transactions2: self.transactions2.clone()
        }

    }
}


impl Transactions
{
    /// Opens the transaction store at the location given in the config
    ///
    /// Creates a new fileset if needed
    pub fn new(cfg: &config::Config) -> Transactions {
        let dir1 = &cfg.root.clone().join("transactions1");
        let dir2 = &cfg.root.clone().join("transactions2");

        Transactions {
            transactions1: FlatFileSet::new(dir1, "t1-", FILE_SIZE, MAX_CONTENT_SIZE),
            transactions2: FlatFileSet::new(dir2, "t2-", FILE_SIZE, MAX_CONTENT_SIZE)
        }
    }

    /// Writes the transaction to the store
    pub fn write(&mut self, tx: &Transaction) -> TxPtr {

        // We're doing "manual" serialization for now;
        // to test performance
        // TODO use typed structure

        // Split the transaction
        let start = tx.txs_out_idx[0];
        let raw_tx = tx.to_raw();
        let raw_part1 = &raw_tx[..start as usize];
        let raw_part2 = &raw_tx[start as usize..];

        // write first part
        let part1_ptr = self.transactions1.write(raw_part1);

        let header = vec![
        part1_ptr.get_file_number() as u32,
        part1_ptr.get_file_offset() as u32,
        tx.txs_out_idx.len() as u32];

        // gather the bytes
        let part2: Vec<u8> =

        header.into_iter()

            // add output indices, with corrected start
            .chain(tx.txs_out_idx.iter().map(|idx| idx - start))

            // convert to bytes
            .flat_map(|x| u32_to_bytes(x).into_iter())

            // add raw outputs data
            .chain(raw_part2.iter().map(|x| *x))

            .collect();

        self.transactions2.write(part2.as_slice())
    }


    /// Reads the full transaction from the given pointer
    pub fn read(&mut self, ptr: TxPtr) -> Vec<u8> {
        let (tx,_) = self.next(ptr).unwrap();
        tx
    }

    /// Reads the full transaction from the given pointer
    /// Returns the transaction and a pointer to the next one
    pub fn next(&mut self, ptr: TxPtr) -> Option<(Vec<u8>, TxPtr)> {

        let part2 = self.transactions2.read(ptr);
        let len = part2.len() as u32;
        if len == 0 {
            return None;
        }

        let part1_ptr = TxPtr::new(
            bytes_to_u32(&part2[0..4]) as i16,
            bytes_to_u32(&part2[4..8]) as u64
        );


        let part1 = self.transactions1.read(part1_ptr);

        // strip header of part2
        let output_count = bytes_to_u32(&part2[8..12]) as usize;
        let header_size = 4 + 4 + 4 + output_count * 4;
        let part2 = &part2[header_size..];

        // gather
        let mut tx: Vec<u8> = part1.into_iter().map(|&x| x).collect();
        tx.extend_from_slice(part2);

        Some((tx, ptr.offset(len + 4)))
    }


    /// Returns only an output from the given transaction
    /// The resulting Vec overflows until the end of the transaction
    pub fn read_output(&mut self, ptr: TxPtr, output_index: u32) -> Option<Vec<u8>> {

        // read all outputs
        let part2 = self.transactions2.read(ptr);
        let output_count = bytes_to_u32(&part2[8..12]);

        if output_count < output_index {
            return None;
        }

        let output_offset_pos = 12 + 4 * output_index as usize;
        let output_offset = bytes_to_u32(&part2[output_offset_pos..output_offset_pos+4]) as usize;

        let output_offset_from_part2 = output_offset + 12 + 4 * output_count as usize;

        
        Some(part2[output_offset_from_part2..].into_iter().map(|&x| x).collect())
    }
}

// helper
fn bytes_to_u32(x: &[u8]) -> u32 {
    ((x[0] as u32) << 24) |
        ((x[1] as u32) << 16) |
        ((x[2] as u32) << 8) |
        ((x[3] as u32) << 0)
}

// helper
fn u32_to_bytes(x: u32) -> Vec<u8> {
    vec![
    (x >> 24) as u8, (x >> 16) as u8, (x >> 8) as u8, (x >> 0) as u8,
    ]
}


#[cfg(test)]
mod tests {

    use super::*;
    use buffer::*;
    use buffer::ToRaw;

    #[macro_use]
    use builders::*;

    #[test]
    fn test_read_write() {
        tx_builder!(bld);

        let tx1 = tx!(bld; coinbase => a;12);

        let tx2 = tx!(bld; a     => b, c );
        let tx3 = tx!(bld; a,b   => c,d,e,f,g );


        let tx1p = Transaction::parse(&mut Buffer::new(&tx1)).unwrap();
        let tx2p = Transaction::parse(&mut Buffer::new(&tx2)).unwrap();
        let tx3p = Transaction::parse(&mut Buffer::new(&tx3)).unwrap();

        let mut store = ::store::Store::new(& test_cfg!());

        let ptr  = store.transactions.write(&tx1p);
        let read = store.transactions.read(ptr);
        assert_eq!(tx1, read.as_slice());

        let ptr = store.transactions.write(&tx2p);
        let read = store.transactions.read(ptr);
        assert_eq!(tx2, read.as_slice());

        let ptr = store.transactions.write(&tx3p);
        let read = store.transactions.read(ptr);
        assert_eq!(tx3, read.as_slice());

    }

    #[test]
    fn test_read_output() {
        tx_builder!(bld);

        let _tx1 = tx!(bld; coinbase => a;12);

        let _tx2 = tx!(bld; a     => b, c );
        let tx3 = tx!(bld; a,b   => c,d,e,f,g );

        println!("Transaction={:?}", tx3);

        let tx3p = Transaction::parse(&mut Buffer::new(&tx3)).unwrap();

        let mut store = ::store::Store::new(& test_cfg!());

        let ptr  = store.transactions.write(&tx3p);

        // read only one output
        let read_out_bytes = store.transactions.read_output(ptr, 3).unwrap();
        let read_out = ::transaction::TxOutput::parse(&mut Buffer::new(&read_out_bytes)).unwrap();

        assert_eq!(read_out, tx3p.txs_out[3]);

    }

}