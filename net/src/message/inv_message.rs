use super::var_int;
use inventory_vector::InventoryVector;
// use byteorder::{LittleEndian, WriteBytesExt};

#[derive(Debug, PartialEq)]
pub struct InvMessage {
    pub count: u64,
    pub inventory: Vec<InventoryVector>,
}

impl InvMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(8);
        v.append(&mut var_int(self.count));
        for item in &self.inventory {
            v.append(&mut item.encode());
        }
        v
    }
}