use {Encode, VarInt};
use block_header::BlockHeader;

#[derive(Debug, Encode, PartialEq)]
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
