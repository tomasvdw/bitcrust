//! Hashing functions
//! For now: Hash32 acts as a reference wrapper
//! and HashBuf acts as an owned hash
//! TODO; move to AsRef trait instead of explicit conversions

use buffer;
use std::mem;
use std::convert::AsMut;
use std::fmt::{Debug,Formatter,Error};
use ring;


/// Owned, 32-byte hash value
#[derive(Copy,Clone,PartialEq)]
pub struct Hash32Buf([u8;32]);

impl Hash32Buf {

    /// Copies a slice into an owned buffer
    pub fn from_slice(slice: &[u8]) -> Hash32Buf {
        let mut result: Hash32Buf = Hash32Buf([0;32]);
        result.0.as_mut().copy_from_slice(&slice[0..32]);
        result
    }

    pub fn as_ref(&self) -> Hash32 {

        Hash32(&self.0)
    }


    /// Hashes the input twice with SHA256 and returns an owned buffer;
    /// Can be extracted as an Hash32 using as_ref()
    pub fn double_sha256(input: &[u8]) -> Hash32Buf {
        let digest1 = ring::digest::digest(&ring::digest::SHA256, input);
        let digest2 = ring::digest::digest(&ring::digest::SHA256, digest1.as_ref());

        // convert to HashBuf
        Hash32Buf::from_slice(digest2.as_ref())
    }

    /// Hashes the input twice with SHA256 and returns an owned buffer;
 /// Can be extracted as an Hash32 using as_ref()
    pub fn double_sha256_from_pair(first: Hash32, second: Hash32) -> Hash32Buf {
        let mut v: Vec<u8> = Vec::new();
        v.extend(first.0.iter());
        v.extend(second.0.iter());

        Hash32Buf::double_sha256(&v)
    }
}


/// Reference to a 32-byte hash value
#[derive(Copy,Clone,PartialEq)]
pub struct Hash32<'a>(pub &'a[u8;32]);




impl<'a> buffer::Parse<'a> for Hash32<'a> {
    /// Parses the hash from a buffer; with 0-copy
    fn parse(buffer: &mut buffer::Buffer<'a>) -> Result<Hash32<'a>, buffer::EndOfBufferError> {

        Ok(Hash32(

            // we must transmute as rustc doesn't trust the slice is exactly 32 bytes
            // (transmuting &[u8] -> &[u8;32])
            unsafe { mem::transmute(
                try!(buffer.parse_bytes(32)).as_ptr()) }
        ))
    }
}

impl<'a> Hash32<'a> {

    /// Returns true if this hash consists only of zeros
    pub fn is_null(&self) -> bool {
        self.0.iter().all(|x| *x == 0)
    }

    pub fn as_buf(&self) -> Hash32Buf {
        Hash32Buf(*self.0)
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


impl Debug for Hash32Buf {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let x = self.0.as_ref()
            .iter()
            .rev()
            .map(|n| format!("{:02x}", n))
            .collect::<Vec<_>>()
            .concat();

        fmt.write_str(&x)
    }
}

#[cfg(test)]
mod tests {
    use util::*;
    use super::{Hash32Buf,Hash32};


    #[test]
    fn test_format_hash32() {
        
        println!("{:?}", Hash32(&[0;32]));
    }

    #[test]
    fn test_hash0() {
        assert!(Hash32(&[0;32]).is_null())

    }


    #[test]
    fn test_double_hash() {


        const HASH1: &'static str = "212300e77d897f2f059366ed03c8bf2757bc2b1dd30df15d34f6f1ee521e58e8";
        const HASH2: &'static str = "4feec9316077e49b59bc23173303e13be9e9f5f9fa0660a58112a04a65a84ef1";
        const HASH3: &'static str = "03b750bf691caf40b7e33d8e15f64dd16becf944b39a82710d6d257159361b93";

        let hash1 = Hash32Buf::from_slice(&from_hex_rev(HASH1));
        let hash2 = Hash32Buf::from_slice(&from_hex_rev(HASH2));
        let hash3 = Hash32Buf::from_slice(&from_hex_rev(HASH3));

        let paired = Hash32Buf::double_sha256_from_pair(hash1.as_ref(), hash2.as_ref());

        assert_eq!(hash3, paired);

    }
}

