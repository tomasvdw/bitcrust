use std::fmt::{self, Debug};
use std::io;

use Encode;

bitflags! {
  flags ServiceFlags: u64 {
      const NETWORK      = 0b00000001,
      const UTXO         = 0b00000010,
      const BLOOM        = 0b00000100,
  }
}

/// The following services are currently assigned:
///
/// Value Name  Description
/// 1 NODE_NETWORK  This node can be asked for full blocks instead of just headers.
/// 2 NODE_GETUTXO  See [BIP 0064](https://github.com/bitcoin/bips/blob/master/bip-0064.mediawiki)
/// 4 NODE_BLOOM  See [BIP 0111](https://github.com/bitcoin/bips/blob/master/bip-0111.mediawiki)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Services {
    flags: ServiceFlags,
}

impl Debug for Services {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        write!(f,
               r"Services {{
    network: {},
    utxo: {},
    bloom: {}}}",
               self.network(),
               self.utxo(),
               self.bloom())
    }
}

impl Services {
    pub fn as_i64(&self) -> i64 {
        self.flags.bits as i64
    }

    pub fn from(input: u64) -> Services {
        Services { flags: ServiceFlags { bits: input } }
    }

    pub fn network(&self) -> bool {
        self.flags.contains(NETWORK)
    }

    pub fn utxo(&self) -> bool {
        self.flags.contains(UTXO)
    }

    pub fn bloom(&self) -> bool {
        self.flags.contains(BLOOM)
    }
}

impl Encode for Services {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        self.flags.bits.encode(&mut buff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_identifies_a_network_node() {
        let svc = Services::from(0b00000001u64);
        println!("{:?}", svc);
        assert!(svc.network());
    }

    #[test]
    fn it_identifies_a_utxo_node() {
        let svc = Services::from(0b00000010u64);
        println!("{:?}", svc);
        assert!(svc.utxo());
    }

    #[test]
    fn it_identifies_a_bloom_node() {
        let svc = Services::from(0b00000100u64);
        println!("{:?}", svc);
        assert!(svc.bloom());
    }

    #[test]
    fn it_identifies_a_bloom_network_node() {
        let svc = Services::from(0b00000101u64);
        println!("{:?}", svc);
        assert!(svc.network());
        assert!(svc.bloom());
    }
}
