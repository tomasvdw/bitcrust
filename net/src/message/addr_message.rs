use net_addr::NetAddr;
use Encode;
use VarInt;

#[cfg(test)]
mod tests {
    use super::*;
    use ::Services;

    #[test]
    fn it_encodes_an_addr_message() {
        let input = vec![
                     // Payload:
                     0x01, // 1 address in this message
                     // Address:
                     0xE2,
                     0x15,
                     0x10,
                     0x4D, // Mon Dec 20 21:50:10 EST 2010 (only when version is >= 31402)
                     0x01,
                     0x00,
                     0x00,
                     0x00,
                     0x00,
                     0x00,
                     0x00,
                     0x00, // 1 (NODE_NETWORK service - see version message)
                     0x00,
                     0x00,
                     0x00,
                     0x00,
                     0x00,
                     0x00,
                     0x00,
                     0x00,
                     0x00,
                     0x00,
                     0xFF,
                     0xFF,
                     0x0A,
                     0x00,
                     0x00,
                     0x01, // IPv4: 10.0.0.1, IPv6: ::ffff:10.0.0.1 (IPv4-mapped IPv6 address)
                     0x20,
                     0x8D];
        let addr = AddrMessage {
            addrs: vec![
                NetAddr {
                    time: Some(1292899810),
                    services: Services::from(1),
                    ip: "::ffff:10.0.0.1".parse().unwrap(),
                    port: 8333 }] };
        let mut encoded = vec![];
        addr.encode(&mut encoded).expect("Failed to encode Addr");
        assert_eq!(input, encoded);
    }

    #[test]
    fn it_implements_types_required_for_protocol() {
        let m =  AddrMessage::default();
        assert_eq!(m.name(), "addr");
        assert_eq!(m.len(), 8);
    }
}

/// addr message
#[derive(Debug, Default, Encode, PartialEq)]
pub struct AddrMessage {
    #[count]
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