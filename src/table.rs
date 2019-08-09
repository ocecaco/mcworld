use fnv::FnvHashMap;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BlockId(u32);

pub const NOT_PRESENT: BlockId = BlockId(0);
pub const AIR: BlockId = BlockId(1);


pub struct BlockTable {
    id_to_name: Vec<String>,
    name_to_id: FnvHashMap<String, u32>,
}

impl BlockTable {
    pub fn new() -> Self {
        let not_present = "mcworld:missing".to_owned();
        let air = "minecraft:air".to_owned();

        // this code should match the constants
        let id_to_name = vec![not_present.clone(), air.clone()];
        let mut name_to_id = FnvHashMap::default();
        name_to_id.insert(not_present, 0);
        name_to_id.insert(air, 1);

        BlockTable {
            id_to_name,
            name_to_id,
        }
    }

    pub fn get_id(&mut self, name: &str) -> BlockId {
        if let Some(id) = self.name_to_id.get(name) {
            BlockId(*id)
        } else {
            let id = self.id_to_name.len() as u32;
            self.id_to_name.push(name.to_owned());
            self.name_to_id.insert(name.to_owned(), id);
            BlockId(id)
        }
    }

    pub fn get_name(&self, id: BlockId) -> &str {
        &self.id_to_name[id.0 as usize]
    }
}
