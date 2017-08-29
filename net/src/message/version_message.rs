use std::io;

use Encode;
use net_addr::NetAddr;
use services::Services;

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
            services: Services::from(1),
            timestamp: 1495102309,
            addr_recv: NetAddr {
                time: None,
                services: Services::from(1),
                ip: Ipv6Addr::from_str("::ffff:127.0.0.1").unwrap(),
                port: 8333,
            },
            addr_send: NetAddr {
                time: None,
                services: Services::from(1),
                ip: Ipv6Addr::from_str("::ffff:127.0.0.1").unwrap(),
                port: 8333,
            },
            nonce: 1,
            user_agent: "bitcrust".into(),
            start_height: 0,
            relay: false,
        };
        let mut encoded = vec![];
        v.encode(&mut encoded).unwrap();
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
    pub services: Services,
    pub timestamp: i64,
    pub addr_recv: NetAddr,
    pub addr_send: NetAddr,
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: i32,
    pub relay: bool,
}

impl VersionMessage {
    #[inline]
    pub fn len(&self) -> usize {
        86 + self.user_agent.len()
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "version"
    }
}

impl Encode for VersionMessage {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        let _ = self.version.encode(&mut buff);
        let _ = self.services.encode(&mut buff);
        let _ = self.timestamp.encode(&mut buff);
        let _ = self.addr_recv.encode(&mut buff);
        if self.version >= 106 {
            let _ = self.addr_send.encode(&mut buff);
            let _ = self.nonce.encode(&mut buff);
            let _ = (self.user_agent.bytes().len() as u8).encode(&mut buff);
            let _ = self.user_agent.encode(&mut buff);
            let _ = self.start_height.encode(&mut buff);
            if self.version >= 70001 {
                if self.relay {
                    buff.push(1);
                } else {
                    buff.push(0);
                }
            }
        }
        Ok(())
    }
}