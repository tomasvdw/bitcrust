use net_addr::NetAddr;
use super::var_int;

/// addr message
#[derive(Debug, PartialEq)]
pub struct AddrMessage {
    pub addrs: Vec<NetAddr>,
}

impl AddrMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(8);
        v.append(&mut var_int(self.addrs.len() as u64));
        for addr in &self.addrs {
            v.append(&mut addr.encode())
        }
        v
    }
}
