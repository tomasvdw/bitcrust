use std::io;

use inventory_vector::InventoryVector;
use {Encode, VarInt};

#[derive(Debug, PartialEq)]
pub struct InvMessage {
    pub count: VarInt,
    pub inventory: Vec<InventoryVector>,
}

impl InvMessage {
    #[inline]
    pub fn len(&self) -> usize {
        8 + (36 * self.inventory.len())
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "inv"
    }
}


impl Encode for InvMessage {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        let _ = self.count.encode(&mut buff);
        let _ = self.inventory.encode(&mut buff);
        Ok(())
    }
}