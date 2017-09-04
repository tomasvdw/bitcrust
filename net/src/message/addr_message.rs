use std::io;

use net_addr::NetAddr;
use Encode;
use VarInt;

/// addr message
#[derive(Debug, PartialEq)]
pub struct AddrMessage {
    pub addrs: Vec<NetAddr>,
}

impl AddrMessage {
    #[inline]
    pub fn len(&self) -> usize {
        8 + (30 * self.addrs.len())
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "addr"
    }
}

impl Encode for AddrMessage {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        let _ = VarInt::new(self.addrs.len() as u64).encode(&mut buff);
        let _ = self.addrs.encode(&mut buff);
        Ok(())
    }
}