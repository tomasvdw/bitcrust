use Encode;
use VarInt;

#[derive(Debug, Encode, PartialEq)]
pub struct TransactionInput {
    pub previous: Outpoint,
    #[count]
    pub script: Vec<u8>,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn len(&self) -> usize {
        36 + // previous Outpoint
        4 + // length of script
        self.script.len() +
        4 // sequence
    }
}

#[derive(Debug, Encode, PartialEq)]
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

#[derive(Debug, Encode, PartialEq)]
pub struct TransactionOutput{
    pub value: i64,
    #[count]
    pub pk_script: Vec<u8>,
}

impl TransactionOutput {
    pub fn len(&self) -> usize {
        8 + // Transaction Value
        4 + // length of script
        self.pk_script.len()
    }
}

#[derive(Debug, Encode, PartialEq)]
pub struct Witness {
    #[count]
    pub component: Vec<u8>
}

impl Witness {
    pub fn len(&self) -> usize {
        8 * self.component.len()
    }
}