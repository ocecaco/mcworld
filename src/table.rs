use fnv::FnvHashMap;
use std::num::NonZeroU16;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BlockId(NonZeroU16);

pub const AIR: BlockId = BlockId(unsafe { NonZeroU16::new_unchecked(1) });

pub struct BlockTable {
    id_to_name: Vec<String>,
    name_to_id: FnvHashMap<String, BlockId>,
}

impl BlockTable {
    pub fn new() -> Self {
        let air = "minecraft:air".to_owned();

        // this code should match the constants
        let id_to_name = vec![air.clone()];
        let mut name_to_id = FnvHashMap::default();
        name_to_id.insert(air, AIR);

        BlockTable {
            id_to_name,
            name_to_id,
        }
    }

    pub fn get_id(&mut self, name: &str) -> BlockId {
        if let Some(id) = self.name_to_id.get(name) {
            *id
        } else {
            let id = self.id_to_name.len() as u16 + 1;
            let id = BlockId(NonZeroU16::new(id).unwrap());

            self.id_to_name.push(name.to_owned());
            self.name_to_id.insert(name.to_owned(), id);

            id
        }
    }

    pub fn get_name(&self, id: BlockId) -> &str {
        &self.id_to_name[id.0.get() as usize - 1]
    }
}
