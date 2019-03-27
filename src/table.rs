use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BlockId(u32);

pub const NOT_PRESENT: BlockId = BlockId(0);
pub const AIR: BlockId = BlockId(1);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BlockDescription {
    pub name: String,
    pub val: u32,
}

pub struct BlockTable {
    id_to_description: Vec<BlockDescription>,
    description_to_id: HashMap<BlockDescription, u32>,
}

impl BlockTable {
    pub fn new() -> Self {
        let not_present_description = BlockDescription {
            name: "mcworld:missing".to_owned(),
            val: 0,
        };

        let air_description = BlockDescription {
            name: "minecraft:air".to_owned(),
            val: 0,
        };

        // this code should match the constants
        let id_to_description = vec![not_present_description.clone(), air_description.clone()];
        let mut description_to_id = HashMap::new();
        description_to_id.insert(not_present_description, 0);
        description_to_id.insert(air_description, 1);

        BlockTable {
            id_to_description,
            description_to_id,
        }
    }

    pub fn get_id(&mut self, description: &BlockDescription) -> BlockId {
        if let Some(id) = self.description_to_id.get(description) {
            BlockId(*id)
        } else {
            let id = self.id_to_description.len() as u32;
            self.id_to_description.push(description.clone());
            self.description_to_id.insert(description.clone(), id);
            BlockId(id)
        }
    }

    pub fn get_description(&self, id: BlockId) -> &BlockDescription {
        &self.id_to_description[id.0 as usize]
    }
}
