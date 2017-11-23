


//! db_block embeds which transactions are included in a block.
//! It is represents as a vector of 64-bit Records

use record::Record;

type DbBlock = Vec<Record>;
