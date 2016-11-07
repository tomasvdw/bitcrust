//! Hashing functions
//! For now: Hash32 acts as a reference wrapper
//! and HashBuf acts as an owned hash

use buffer;
use std::fmt::{Debug,Formatter,Error};
use ring;

type HashBuf = ring::digest::Digest;



/// Hashes the input twice with SHA256 and returns an owned buffer;
/// The actual hash-value can be extracted with as_ref()
pub fn double_sha256(input: &[u8]) -> HashBuf {
    // TODO: I think we want to return a [u8;32] here but that doesn't work this way
    let digest1 = ring::digest::digest(&ring::digest::SHA256, input);
    let digest2 = ring::digest::digest(&ring::digest::SHA256, digest1.as_ref());
    digest2
}

#[derive(PartialEq)]
pub struct Hash32<'a>(pub &'a[u8]);


impl<'a> buffer::Parse<'a> for Hash32<'a> {
    /// Parses the hash from a buffer; with 0-copy
    fn parse(buffer: &mut buffer::Buffer<'a>) -> Result<Hash32<'a>, buffer::EndOfBufferError> {
        Ok(
            Hash32(try!(buffer.parse_bytes(32)))
        )
    }
}

impl<'a> Hash32<'a> {

    /// Returns true if this hash consists only of zeros
    pub fn is_null(&self) -> bool {
        self.0.iter().all(|x| *x == 0)
    }


}




impl<'a> Debug for Hash32<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let x = self.0
            .iter()
            .rev()
            .map(|n| format!("{:02x}", n))
            .collect::<Vec<_>>()
            .concat();

        fmt.write_str(&x)
    }
}

#[cfg(test)]
mod test {
    
    use super::*;
    
    #[test]
    fn test_format_hash32() {
        
        println!("{:?}", Hash32(&[0;32]));
    }
}

