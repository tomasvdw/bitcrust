//! Merkle tree implementation
//!

// minimum number of hashes to use parallel hashing
const PARALLEL_HASHING_THRESHOLD: usize = 60;

use rayon::prelude::*;
use hash::*;

/// This halves the merkle tree leaves, taking it one level up
///
/// Calls itself recursively until one is left
fn shrink_merkle_tree(hashes: Vec<Hash32Buf>) -> Vec<Hash32Buf> {

    if hashes.len() == 1 {
        return hashes;
    }

    // the result is half the size rounded up
    let count = (hashes.len() + 1 ) / 2;

    // closure to hash n*2 and n*2+1
    let reduce = |n| {
        let ref first: Hash32Buf = hashes[n * 2];

        // we double the first one if we have an odd number of hashes
        // in this layer
        let ref second = hashes.get(n * 2 + 1).unwrap_or(first);

        Hash32Buf::double_sha256_from_pair(
            first.as_ref(),
            second.as_ref()
        )
    };

    let result = if count > PARALLEL_HASHING_THRESHOLD
        { (0..count).into_par_iter().map(reduce).collect() }
    else
        { (0..count).into_iter().map(reduce).collect() };

    shrink_merkle_tree(result)
}

/// Calculates the merkle root for the given set of hashes
pub fn get_merkle_root(hashes: Vec<Hash32Buf>) -> Hash32Buf {

    shrink_merkle_tree(hashes)[0]
}




#[cfg(test)]
mod tests {

    use super::*;
    use util::*;



    #[test]
    fn test_merkle1() {

        const HASH1: &'static str = "212300e77d897f2f059366ed03c8bf2757bc2b1dd30df15d34f6f1ee521e58e8";
        const HASH2: &'static str = "4feec9316077e49b59bc23173303e13be9e9f5f9fa0660a58112a04a65a84ef1";
        const EXP_MERKLE: &'static str = "03b750bf691caf40b7e33d8e15f64dd16becf944b39a82710d6d257159361b93";

        let hash1 = Hash32Buf::from_slice(&from_hex_rev(HASH1));
        let hash2 = Hash32Buf::from_slice(&from_hex_rev(HASH2));
        let exp_merkle = Hash32Buf::from_slice(&from_hex_rev(EXP_MERKLE));

        let mut merkle = Vec::new();
        merkle.push(hash1);
        merkle.push(hash2);


        let merkle_root = get_merkle_root(merkle);

        assert_eq!(exp_merkle, merkle_root);


    }

    #[test]
    fn test_merkle2() {

        const HASH_SET : [&'static str;12] = [
            "30803bc3fefa999bf187cda4fff3647a78db6b957fcf5a579270c0535ec1601e",
            "101d83a3e4739640fbb6279883478bb6a2814e6fcd58322f0b1d3bf03983268a",
            "d646c47be5581891fc8d098e0db6e288efa22962d175f6d8c77913c2f898c0aa",
            "61419189bed89e85689442ee144e0aafc8b1ae6ed813e15f241b0750b97886ec",
            "09e5c37fe017605a15ae1fb139b2c186c76013c8cb78e6524261b29aed0b0424",
            "8226f6778f0f900ed38d9ca7314cfc55b28cc9c809e99570e3499ffeaa57759f",
            "0568ba35848086f4b3352fb468338ff727411fbf0e49fbe46eaf926929d205fe",
            "4e9dc6455ee50181a12e43b46de1bc72b201e627294dea7abe2f928b3e86bdb3",
            "6780b0e7801554d92d409f30b19328948ee3bcfd53764be30db7f1146fc12890",
            "dbcf135133afb7c3a256f87d09ae15994c5358bffd3e3993b56efb43742dce1e",
            "29454d128096bf66ebefe3f80a38edba4befac78c0eebfd351b7099f01933e2f",
            "c8cb5078d0faae9dc7bfcd150207c15d331e12161ec0e7c66c744d1201e08f3b"];

        const HASH_SET_MERKLE: &'static str =
            "1ba9abf54ae4cb022f53e767669931bacdd42c783fe063c5738ca49d29f1fbe3";


        let exp_merkle = Hash32Buf::from_slice(&from_hex_rev(HASH_SET_MERKLE));

        let mut merkle = Vec::new();
        for tx in HASH_SET.iter() {
            let txh = Hash32Buf::from_slice(&from_hex_rev(tx));
            merkle.push(txh);
        }

        let merkle_root = get_merkle_root(merkle);

        assert_eq!(exp_merkle, merkle_root);


    }
}