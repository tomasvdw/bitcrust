
use hash::Hash256;

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    version:   i32,
    txs_in:    Vec<TxIn>,
    txs_out:   Vec<TxOut>,
    lock_time: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxIn {
    prev_tx_out:     Hash256,
    prev_tx_out_idx: u32,
    script:          Vec<u8>,
    sequence:        u32,        
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxOut {
    value:           i64,
    pk_script:       Vec<u8>,
}


