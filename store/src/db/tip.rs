
use std::collections;

use record::Record;


/// Tip is an in-memory cache of information about the most worked header;
/// it contains:
/// * copies of the last connected-blocks
/// * pointers to blocks awaiting connection
///
/// When the most-worked chain moves; we move the tip along
///
struct Tip {

    block_hash: [u8; 32],
    header_ptr: ValuePtr,

    last_connected_blocks: Vec<Vec<Record>>,

    unconnected_blocks: HashMap
}


