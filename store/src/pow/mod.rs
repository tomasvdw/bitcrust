

mod u256;

pub type Work = u256::U256;
pub use self::u256::U256;

// Converts a header "nbits" representation to a U256 difficulty target
// This doesn't check errors; the correctness of a compact target u32 (nbits) is
// is tested in header-validation
pub fn from_compact(compact_target: u32) -> U256 {

    let size = compact_target as usize & 0xFF00_0000;
    let mut word = U256::from((compact_target as u64) & 0x007f_ffff);
    if size <= 3 {
        word >> (8 * (3 - size))
    }
    else {
        word << (8 * (size - 3))
    }

}

// Converts the difficulty target (= maximum hash to find) to work,
// which is its reciprocal
// we multiply by constant 2^256 to keep ensure the results are integral
pub fn difficulty_target_to_work(target: U256) -> U256 {
    // We find:
    // (2^256) / (target+1)
    // = ((2^256 - (target+1))/ (target+1)) - 1
    // = (!target / (target+1)) + 1

    ((!target) / (target + U256::one())) + U256::one()
}
