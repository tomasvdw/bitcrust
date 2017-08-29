use super::var_int;
use block_header::BlockHeader;

#[derive(Debug, PartialEq)]
pub struct HeaderMessage {
    pub count: u64,
    pub headers: Vec<BlockHeader>,
}

impl HeaderMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(8);
        v.append(&mut var_int(self.count));
        for item in &self.headers {
            v.append(&mut item.encode());
        }
        v
    }
}
