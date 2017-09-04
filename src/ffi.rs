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

// typedef enum bitcoinconsensus_error_t
// {
//     bitcoinconsensus_ERR_OK = 0,
//     bitcoinconsensus_ERR_TX_INDEX,
//     bitcoinconsensus_ERR_TX_SIZE_MISMATCH,
//     bitcoinconsensus_ERR_TX_DESERIALIZE,
//     bitcoinconsensus_ERR_AMOUNT_REQUIRED,
//     bitcoinconsensus_ERR_INVALID_FLAGS,
// } bitcoinconsensus_error;

#[derive(Debug)]
pub enum VerifyScriptError {
    Index,
    SizeMismatch,
    Deserialize,
    AmountRequired,
    InvalidFlags,
}

/// Verifies whether the given `input` of the transaction spends the given `output`
/// using libbitcoin-consensus

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
        Err(match err {
            0 => VerifyScriptError::Index,
            1 => VerifyScriptError::SizeMismatch,
            2  => VerifyScriptError::Deserialize,
            3  => VerifyScriptError::AmountRequired,
            4  => VerifyScriptError::InvalidFlags,
            _ => unreachable!()
        })
    }
}



#[cfg(test)]
mod tests {

}
