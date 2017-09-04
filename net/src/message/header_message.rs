use std::io;

use {Encode, VarInt};
use block_header::BlockHeader;

#[derive(Debug, PartialEq)]
pub struct HeaderMessage {
    pub count: VarInt,
    pub headers: Vec<BlockHeader>,
}

impl HeaderMessage {
    #[inline]
    pub fn len(&self) -> usize {
        8 + (81 * self.headers.len())
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "headers"
    }


}

impl Encode for HeaderMessage {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        let _ = self.count.encode(&mut buff);
        let _ = self.headers.encode(&mut buff);
        Ok(())
    }
}