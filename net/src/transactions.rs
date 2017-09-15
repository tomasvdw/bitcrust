pub struct TransactionInput{
    pub previous: Outpoint,
    pub script: String,
    pub sequence: u32,
}

pub struct Outpoint {
    pub hash: [u8; 32],
    pub index: u32,
}

impl Outpoint {
    pub fn new(index: u32, hash: &[u8]) -> Outpoint {
      debug_assert!(hash.len() == 32);
      let mut a: [u8; 32] = Default::default();
      a.copy_from_slice(&hash);
        Outpoint {
            index: index,
            hash: a,
        }
    }
}
pub struct TransactionOutput{
    pub value: i64,
    pub pk_script: String,
}