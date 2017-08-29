use std::io;
use Encode;

#[derive(Debug, PartialEq)]
pub struct InventoryVector {
    flags: InvFlags,
    pub hash: [u8; 32],
}

bitflags! {
  flags InvFlags: u32 {
      const ERROR               = 0b0,
      const MSG_TX              = 0b00000001,
      const MSG_BLOCK           = 0b00000010,
      const MSG_FILTERED_BLOCK  = 0b00000100,
      const MSG_CMPCT_BLOCK     = 0b00001000
  }
}

impl InventoryVector {
    pub fn new(flags: u32, hash: &[u8]) -> InventoryVector {
      debug_assert!(hash.len() == 32);
      let mut a: [u8; 32] = Default::default();
      a.copy_from_slice(&hash);
        InventoryVector {
            flags: InvFlags { bits: flags },
            hash: a,
        }
    }
}

impl Encode for InventoryVector {
    fn encode(&self, mut v: &mut Vec<u8>) -> Result<(), io::Error> {
        // let mut v: Vec<u8> = Vec::with_capacity(36);
        self.flags.bits.encode(&mut v)?;
        self.hash.encode(&mut v)?;
        Ok(())
    }
}