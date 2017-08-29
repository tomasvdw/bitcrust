use std::io;

use {Encode, VarInt};

#[derive(Debug, PartialEq)]
pub struct GetheadersMessage {
    pub version: u32,
    pub locator_hashes: Vec<[u8; 32]>,
    pub hash_stop: [u8; 32],
}

impl GetheadersMessage {
    #[inline]
    pub fn len(&self) -> usize {
        128
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "getheaders"
    }
}

impl Encode for GetheadersMessage {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        let _ = self.version.encode(&mut buff);
        let _ = VarInt::new(self.locator_hashes.len() as u64).encode(&mut buff);
        let _ = self.locator_hashes.encode(&mut buff);
        let _ = self.hash_stop.encode(&mut buff);
        Ok(())
    }
}