use {Encode, VarInt};
use block_header::BlockHeader;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_implements_types_required_for_protocol() {
        let m =  HeaderMessage::default();
        assert_eq!(m.name(), "headers");
        assert_eq!(m.len(), 8);
    }
}

#[derive(Debug, Default, Encode, PartialEq)]
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
