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

    #[cfg(feature = "verify_scripts")]
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
#[allow(dead_code)]
pub enum VerifyScriptError {
    UnknownError
}

/// Verifies whether the given `input` of the transaction spents the given `output`
/// using libbitcoin-consensus

#[cfg(feature = "verify_scripts")]
pub fn verify_script(previous_tx_out: &[u8], transaction: &[u8], input: u32) -> Result<(), VerifyScriptError> {
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

#[cfg(not(feature = "verify_scripts"))]
pub fn verify_script(_: &[u8], _: &[u8], _: u32) -> Result<(), VerifyScriptError> {
    Ok(())

}


#[cfg(test)]
mod tests {

}
