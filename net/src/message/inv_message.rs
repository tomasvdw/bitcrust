use inventory_vector::InventoryVector;
use {Encode, VarInt};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_implements_types_required_for_protocol() {
        let m =  InvMessage::default();
        assert_eq!(m.name(), "inv");
        assert_eq!(m.len(), 8);
    }
}
#[derive(Debug, Default, Encode, PartialEq)]
pub struct InvMessage {
    #[count]
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
