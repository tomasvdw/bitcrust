use Encode;
use VarInt;
use super::TransactionMessage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_implements_types_required_for_protocol() {
        let m =  BlockMessage::default();
        assert_eq!(m.name(), "block");
        assert_eq!(m.len(), 84);
    }
}

#[derive(Debug, Default, Encode, PartialEq)]
pub struct BlockMessage {
    pub version: i32,
    pub previous_block: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
    #[count]
    pub transactions: Vec<TransactionMessage>
}

impl BlockMessage {
    #[inline]
    pub fn len(&self) -> usize {
        4usize + // version
        32usize + // previous block
        32usize + // merkle root
        4usize + // timestamp
        4usize + // bits
        4usize + // nonce
        4usize + // count of transactions
        self.transactions.iter().map(|i| i.len()).sum::<usize>() // transactions
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "block"
    }

    pub fn new(version: i32, prev_block: &[u8], merkle_root: &[u8], timestamp: u32,
               bits: u32, nonce: u32, transactions: Vec<TransactionMessage>) -> BlockMessage {
        debug_assert!(prev_block.len() == 32);
        let mut a: [u8; 32] = Default::default();
        a.copy_from_slice(&prev_block);
        debug_assert!(merkle_root.len() == 32);
        let mut b: [u8; 32] = Default::default();
        b.copy_from_slice(&merkle_root);
        BlockMessage {
            version: version,
            previous_block: a,
            merkle_root: b,
            timestamp: timestamp,
            bits: bits,
            nonce: nonce,
            transactions: transactions,
        }
    }
}