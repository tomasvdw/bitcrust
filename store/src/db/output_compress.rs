

/// Output compressor;
///
/// Uses the same implementation as ABC
/// Except; we do not optimize out the varint size
/// and we also prefix uncompressed scripts
//
// Currently unused

pub fn compress_output(script: &[u8]) -> Vec<u8> {

    // to key id
    if script.len() == 25
        && script[0] == 0x76 // op_dup
        && script[1] == 0xa9 // op_hash160
        && script[2] == 0x14
        && script[23] == 0x88 // op_equalverify
        && script[24] == 0xac // op_checksig
        {
            let mut v = Vec::with_capacity(21);
            v.push(0);
            v.extend_from_slice(&script[3..23]);
            v
        }
    else {
        // uncompressed
        let mut v = Vec::with_capacity(script.len()+1);
        v.push(0xff);
        v.extend_from_slice(&script[..]);
        v
    }

}

pub fn decompress_output(script: &[u8]) -> Vec<u8> {
    match script[0] {
        0 => {
            let mut v = Vec::with_capacity(25);
            v.extend_from_slice(&[0x76, 0xa9, 0x14]);
            v.extend_from_slice(&script[1..]);
            v.extend_from_slice(&[0x88, 0xac]);
            v
        }
        _ => {
            let mut v = Vec::with_capacity(script.len()-1);
            v.extend_from_slice(&script[1..]);
            v
        }
    }
}


#[cfg(test)]
mod tests {

    use util;
    use super::*;

    #[test]
    fn test_compress() {
        let not_shrunk = ["aabbcc00", "00000000"];
        let shrunk = ["76a914112233445566778899001122334455667788990088ac"];

        for s in not_shrunk.iter() {
            let base = util::from_hex(s);
            let compr = compress_output(&base);
            let decomp = decompress_output(&compr);

            assert_eq!(compr.len(), decomp.len()+1);
            assert_eq!(decomp, base);

        }

        for s in shrunk.iter() {
            let base = util::from_hex(s);
            let compr = compress_output(&base);
            let decomp = decompress_output(&compr);

            assert!(compr.len() < decomp.len());
            assert_eq!(decomp, base);

        }
    }
}