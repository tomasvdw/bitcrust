//! Compile time parameters to the spent-tree engine.
//! Used by record::seek_and_set


// These are the thresholds to which tx-ptr are compared to see if a skip is made
// Essentially:
//
// loop
//   if seek_tx < this_tx + DELTA[n] then
//      this_tx = jump(this_tx[tx].skips[n])
pub const DELTA: [i64; SKIP_FIELDS] = [
    2 * 256 * 256,
    32 * 256 * 256,//32 * 256 * 256 ,
    16 * 256 * 256 * 256 //16 * 256 * 256 * 256
];
///
/// On every record we partition the space into 6 groups;
/// The total range of these groups slowly changes;
///
///  [] [] [] [] [] []
///
/*
[
12665503791,
   83184115,
 4461636966,
 5245763712,
14194978126,
   75962928*/
// Transactions don't need to be checked, but are searched anyway to set the skip values
// we stop after this amount of skip-values
pub const TX_NEEDED_SKIPS:usize = 0;


pub const SKIP_FIELDS: usize = 3;
