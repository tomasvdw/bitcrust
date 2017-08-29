use {Encode, VarInt};

#[derive(Debug, Encode, PartialEq)]
pub struct GetheadersMessage {
    pub version: u32,
    #[count]
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