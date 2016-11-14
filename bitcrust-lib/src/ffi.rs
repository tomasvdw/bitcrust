extern crate libc;

#[link(name = "secp256k1")]
#[link(name = "bitcoin-consensus")]
extern {
    pub fn bitcoin_verify_script(
        transaction:         *const u8,
        transaction_size:    libc::size_t,
        prevout_script:      *const u8,
        prevout_script_size: libc::size_t,
        tx_input_index:      u32,
        flags:               u32)

        -> i32;
}


#[cfg(test)]
mod tests {

}
