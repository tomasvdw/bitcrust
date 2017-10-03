use {Encode, VarInt};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_implements_types_required_for_protocol() {
        let m =  GetblocksMessage::default();
        assert_eq!(m.name(), "getblocks");
        assert_eq!(m.len(), 45);
    }
}

#[derive(Debug, Default, Encode, PartialEq)]
pub struct GetblocksMessage {
    pub version: u32,
    #[count]
    pub locator_hashes: Vec<[u8; 32]>,
    pub hash_stop: [u8; 32],
}

impl GetblocksMessage {
    #[inline]
    pub fn len(&self) -> usize {
        4 + 9 + ( self.locator_hashes.len() * 32 ) + 32
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "getblocks"
    }
}