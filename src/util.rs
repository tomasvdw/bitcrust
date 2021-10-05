//! General utility functions



/// Used mainly for tests; found somewhere (rustc_serialize I think)
pub fn from_hex(str: &str) -> Vec<u8> {

    // This may be an overestimate if there is any whitespace
    let mut b = Vec::with_capacity(str.len() / 2);
    let mut modulus = 0;
    let mut buf = 08;

    for byte in str.bytes() {
        buf <<= 4;

        match byte {
            b'A'..=b'F' => buf |= byte - b'A' + 10,
            b'a'..=b'f' => buf |= byte - b'a' + 10,
            b'0'..=b'9' => buf |= byte - b'0',
            b' '|b'\r'|b'\n'|b'\t' => {
                buf >>= 4;
                continue
           }
            _ => {
                panic!("Invalid hex char");
            }
        }

        modulus += 1;
        if modulus == 2 {
            modulus = 0;
            b.push(buf);
        }
    }

    b.into_iter().collect()
}

/// Useful to keep hashes in the same format as usually printed
pub fn from_hex_rev(str: &str) -> Vec<u8> {
    let mut v = from_hex(str);
    v.reverse();
    v
}
