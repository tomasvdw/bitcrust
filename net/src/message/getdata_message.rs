use inventory_vector::InventoryVector;
use {Encode, VarInt};

#[derive(Debug, Encode, PartialEq)]
pub struct GetdataMessage {
    #[count]
    pub inventory: Vec<InventoryVector>,
}

impl GetdataMessage {
    #[inline]
    pub fn len(&self) -> usize {
        8 + (36 * self.inventory.len())
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "getdata"
    }
}
