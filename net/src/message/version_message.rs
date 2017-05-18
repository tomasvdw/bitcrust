use byteorder::{LittleEndian, WriteBytesExt};

use net_addr::NetAddr;

#[cfg(test)]
mod tests {
    use std::net::Ipv6Addr;
    use std::str::FromStr;
    use super::*;
    #[test]
    fn it_parses_a_version_message() {}

    #[test]
    fn it_encodes_a_version_message() {
        let v = VersionMessage {
            version: 60002,
            services: 1,
            timestamp: 1495102309,
            addr_recv: NetAddr {
                time: None,
                services: 1,
                ip: Ipv6Addr::from_str("::ffff:127.0.0.1").unwrap(),
                port: 8333,
            },
            addr_send: NetAddr {
                time: None,
                services: 1,
                ip: Ipv6Addr::from_str("::ffff:127.0.0.1").unwrap(),
                port: 8333,
            },
            nonce: 1,
            user_agent: "bitcrust".into(),
            start_height: 0,
            relay: false,
        };
        let encoded = v.encode();
        let expected: Vec<u8> =
            vec![98, 234, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 101, 115, 29, 89, 0, 0, 0, 0, 1, 0, 0, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 1, 32, 141, 1, 0,
                 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 1, 32, 141,
                 1, 0, 0, 0, 0, 0, 0, 0, 8, 98, 105, 116, 99, 114, 117, 115, 116, 0, 0, 0, 0];
        assert_eq!(expected, encoded);
    }
}

#[derive(Debug, PartialEq)]
pub struct VersionMessage {
    pub version: i32,
    pub services: u64,
    pub timestamp: i64,
    pub addr_recv: NetAddr,
    pub addr_send: NetAddr,
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: i32,
    pub relay: bool,
}

impl VersionMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(86 + self.user_agent.len());
        let _ = v.write_i32::<LittleEndian>(self.version);
        let _ = v.write_u64::<LittleEndian>(self.services);
        let _ = v.write_i64::<LittleEndian>(self.timestamp);
        v.append(&mut self.addr_recv.encode());
        if self.version >= 106 {
            v.append(&mut self.addr_send.encode());
            let _ = v.write_u64::<LittleEndian>(self.nonce);
            let _ = v.write_u8(self.user_agent.bytes().len() as u8);
            for byte in self.user_agent.bytes() {
                let _ = v.write_u8(byte);
            }
            let _ = v.write_i32::<LittleEndian>(self.start_height);
            if self.version >= 70001 {
                if self.relay {
                    v.push(1);
                } else {
                    v.push(0);
                }
            }
        }
        v
    }
}
