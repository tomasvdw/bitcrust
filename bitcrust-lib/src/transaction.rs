
use std::fmt;
use script::context;

use hash::Hash256;

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    version:   i32,
    txs_in:    Vec<TxIn>,
    txs_out:   Vec<TxOut>,
    lock_time: u32,
}

#[derive(Serialize, Deserialize)]
pub struct TxIn {
    prev_tx_out:     Hash256,
    prev_tx_out_idx: u32,
    script:          Vec<u8>,
    sequence:        u32,        
}

#[derive(Serialize, Deserialize)]
pub struct TxOut {
    value:           i64,
    pk_script:       Vec<u8>,
}

impl fmt::Debug for TxIn {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(fmt, "Prev-TX:{:?}, idx={:?}, sq={:?} ", 
            self.prev_tx_out,
            self.prev_tx_out_idx,
            self.sequence));
        
        let ctx = context::Context::new(&self.script);
        write!(fmt, "{:?}", ctx)

        
    }
}



impl fmt::Debug for TxOut {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(fmt, "v:{:?} ", self.value));
        
        let ctx = context::Context::new(&self.pk_script);
        write!(fmt, "{:?}", ctx)

        
    }
}



