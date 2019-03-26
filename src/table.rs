use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BlockId(u32);

pub struct BlockTable {
    id_to_description: Vec<String>,
    description_to_id: HashMap<String, u32>,
}

impl BlockTable {
    pub fn new() -> Self {
        BlockTable {
            id_to_description: Vec::new(),
            description_to_id: HashMap::new(),
        }
    }

    pub fn get_id(&mut self, description: String) -> BlockId {
        match self.description_to_id.entry(description.clone()) {
            Entry::Occupied(o) => {
                BlockId(*o.get())
            }
            Entry::Vacant(v) => {
                let id = self.id_to_description.len() as u32;
                self.id_to_description.push(description.clone());
                v.insert(id);
                BlockId(id)
            }
        }
    }

    pub fn get_description(&self, id: BlockId) -> String {
        self.id_to_description[id.0 as usize].clone()
    }
}
