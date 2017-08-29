use inventory_vector::InventoryVector;
use {Encode, VarInt};

#[derive(Debug, Encode, PartialEq)]
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
