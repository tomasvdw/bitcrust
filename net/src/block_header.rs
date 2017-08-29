use byteorder::{LittleEndian, WriteBytesExt};

use message::var_int;

#[derive(Debug, PartialEq)]
/// 4	version	int32_t	Block version information (note, this is signed)
/// 32	prev_block	char[32]	The hash value of the previous block this particular block references
/// 32	merkle_root	char[32]	The reference to a Merkle tree collection which is a hash of all transactions related to this block
/// 4	timestamp	uint32_t	A timestamp recording when this block was created (Will overflow in 2106[2])
/// 4	bits	uint32_t	The calculated difficulty target being used for this block
/// 4	nonce	uint32_t	The nonce used to generate this block… to allow variations of the header and compute different hashes
/// 1	txn_count	var_int	Number of transaction entries, this value is always 0
pub struct BlockHeader {
    pub version: i32,
    pub prev_block: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
    /// txn_count is a var_int on the wire
    pub txn_count: u64,
}

impl BlockHeader {
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(81);
        let _ = v.write_i32::<LittleEndian>(self.version);
        v.extend(&self.prev_block);
        v.extend(&self.merkle_root);
        let _ = v.write_u32::<LittleEndian>(self.timestamp);
        let _ = v.write_u32::<LittleEndian>(self.bits);
        let _ = v.write_u32::<LittleEndian>(self.nonce);
        v.extend_from_slice(&mut var_int(self.txn_count));
        v
    }
}