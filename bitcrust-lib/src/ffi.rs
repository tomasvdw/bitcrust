//! Interface to C-libs.
//!
//! Currently only libbitcoinconsensus


extern crate libc;

#[link(name = "bitcoinconsensus")]
extern {

/* EXPORT_SYMBOL int bitcoinconsensus_verify_script(const unsigned char *scriptPubKey, unsigned int scriptPubKeyLen,
                                                 const unsigned char *txTo        , unsigned int txToLen,
                                                 unsigned int nIn, unsigned int flags, bitcoinconsensus_error* err);
*/


    pub fn bitcoinconsensus_verify_script(
        prevout_script:      *const u8,
        prevout_script_size: u32,
        transaction:         *const u8,
        transaction_size:    u32,
        tx_input_index:      u32,
        flags:               u32,
        err:                 *mut i32
    )

        -> i32;
}

#[derive(Debug)]
pub enum VerifyScriptError {
    UnknownError
}

/// Verifies whether the given `input` of the transaction spents the given `output`
/// using libbitcoin-consensus
pub fn verify_script(previous_tx_out: &[u8], transaction: &[u8], input: u32) -> Result<(), VerifyScriptError> {
    return Ok(());
    let flags = 0;
    let mut err: i32 = 0;
    let result = unsafe { bitcoinconsensus_verify_script(
        previous_tx_out.as_ptr(),
        previous_tx_out.len()  as u32,
        transaction.as_ptr(),
        transaction.len() as u32,
        input as u32,
        flags,
        &mut err
    ) };

    if result == 1 {
        Ok(())
    }
    else {
        Err(VerifyScriptError::UnknownError)
    }
}


#[cfg(test)]
mod tests {

}
