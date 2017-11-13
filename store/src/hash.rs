//! Hashing functions
//! For now: Hash32 acts as a reference wrapper
//! and HashBuf acts as an owned hash
//! TODO; move to AsRef trait instead of explicit conversions

extern crate ring;

use network_encoding::*;
use std::fmt::{Debug,Formatter,Error};



pub type Hash = [u8; 32];

/// Copies a slice into an owned buffer
pub fn hash_from_slice(slice: &[u8]) -> Hash {
    let mut result = [0;32];
    result.copy_from_slice(&slice[0..32]);
    result
}

/// Hashes the input twice with SHA256 and returns an owned buffer;
pub fn double_sha256(input: &[u8]) -> Hash {
    let digest1 = ring::digest::digest(&ring::digest::SHA256, input);
    let digest2 = ring::digest::digest(&ring::digest::SHA256, digest1.as_ref());

    hash_from_slice(digest2.as_ref())
}

/// Hashes the input twice with SHA256 and returns an owned buffer;
/// Can be extracted as an Hash32 using as_ref()
pub fn double_sha256_from_pair(first: &Hash, second: &Hash) -> Hash {
    let mut v: Vec<u8> = Vec::new();
    v.extend(first.iter());
    v.extend(second.iter());

    double_sha256(&v)
}

/// network encoding from reference
impl<'a> NetworkEncoding<'a> for &'a [u8;32] {

    fn decode(buffer: &mut Buffer<'a>) -> Result<&'a [u8;32], EndOfBufferError> {
        let h: &Hash = unsafe { &*(buffer.inner.as_ptr() as *const [_; 32]) };
        buffer.inner = &buffer.inner[32..];
        Ok(h)
    }

    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(*self);
    }
}

/// network encoding from owned
impl<'a> NetworkEncoding<'a> for [u8;32] {

    fn decode(buffer: &mut Buffer) -> Result<[u8;32], EndOfBufferError> {
        let h: &Hash = unsafe { &*(buffer.inner.as_ptr() as *const [_; 32]) };
        buffer.inner = &buffer.inner[32..];
        Ok(*h)
    }

    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self[..]);
    }
}


#[cfg(test)]
mod tests {
    use util::*;
    use super::*;


    #[test]
    fn test_double_hash() {


        const HASH1: &'static str = "212300e77d897f2f059366ed03c8bf2757bc2b1dd30df15d34f6f1ee521e58e8";
        const HASH2: &'static str = "4feec9316077e49b59bc23173303e13be9e9f5f9fa0660a58112a04a65a84ef1";
        const HASH3: &'static str = "03b750bf691caf40b7e33d8e15f64dd16becf944b39a82710d6d257159361b93";

        let hash1 = hash_from_slice(&from_hex_rev(HASH1));
        let hash2 = hash_from_slice(&from_hex_rev(HASH2));
        let hash3 = hash_from_slice(&from_hex_rev(HASH3));

        let paired = double_sha256_from_pair(&hash1, &hash2);

        assert_eq!(hash3, paired);

    }
}

