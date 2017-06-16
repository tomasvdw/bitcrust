use byteorder::{LittleEndian, WriteBytesExt};

use super::var_int;

#[derive(Debug, PartialEq)]
pub struct GetheadersMessage {
    pub version: u32,
    pub locator_hashes: Vec<Vec<u8>>,
    pub hash_stop: Vec<u8>,
}

impl GetheadersMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(128);
        let _ = v.write_u32::<LittleEndian>(self.version);
        v.append(&mut var_int(self.locator_hashes.len() as u64));
        // for hash in self.locator_hashes {
        //     v.append(&mut hash);
        // }
        // v.append(&mut self.hash_stop);
        v
    }
}
