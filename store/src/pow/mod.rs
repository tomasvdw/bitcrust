

mod u256;

pub type Work = u256::U256;
pub use self::u256::U256;

// Converts a header "nbits" representation to a U256 difficulty
// This doesn't check errors; the correctness of nbits must already be tested
pub fn from_compact(compact_work: u32) -> Work {

    let size = compact_work as usize & 0xFF00_0000;
    let mut word = U256::from((compact_work as u64) & 0x007f_ffff);
    if size <= 3 {
        word >> (8 * (3 - size))
    }
    else {
        word << (8 * (size - 3))
    }

}

// converts the difficulty (= maximum hash to find) to work,
// by finding (x)/
pub fn difficulty_to_work() -> U256 {
    unimplemented!()
}
