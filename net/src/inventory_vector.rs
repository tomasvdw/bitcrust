#[derive(Debug, PartialEq)]
pub struct InventoryVector {
    flags: InvFlags,
    pub hash: Vec<u8>,
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
    pub fn encode(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::with_capacity(36);
        v
    }

    pub fn new(flags: u32, hash: &[u8]) -> InventoryVector {
        InventoryVector {
            flags: InvFlags { bits: flags },
            hash: hash.to_owned(),
        }
    }
}
