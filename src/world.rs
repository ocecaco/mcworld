use std::cell::RefCell;
use std::path::Path;
use fnv::FnvHashMap;

use crate::raw::RawWorld;
use crate::raw::subchunk::BlockStorage;
use crate::pos::*;
use crate::table::{BlockId, BlockTable, AIR};
use crate::error::*;

const AIR_INFO: BlockInfo = BlockInfo { block_id: AIR, block_val: 0 };

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BlockInfo {
    pub block_id: BlockId,
    pub block_val: u16,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BlockData {
    pub layer1: BlockInfo,
    pub layer2: Option<BlockInfo>,
}

pub struct World {
    raw_world: RawWorld,
    global_palette: RefCell<BlockTable>,
    chunk_cache: RefCell<FnvHashMap<ChunkPos, Option<Chunk>>>,
}

// uses indices into table stored in the World instead of a separate palette for
// each subchunk
struct ConvertedSubchunk {
    data1: Vec<BlockInfo>,
    data2: Option<Vec<BlockInfo>>,
}

struct Chunk {
    // subchunks might be missing, which means they are filled with air. The
    // length of this vector should always be exactly 16, since that is the
    // number of subchunks per chunk.
    subchunks: Vec<Option<ConvertedSubchunk>>,
}

impl Chunk {
    fn get_block(&self, w: &WorldPos) -> Option<BlockData> {
        let sub_y = w.subchunk_y();
        let sub_offset = w.subchunk_offset();

        let maybe_subchunk = &self.subchunks[sub_y];
        match maybe_subchunk {
            Some(subchunk) => {
                let block1 = subchunk.data1[sub_offset];
                let block2 = subchunk.data2.as_ref().map(|data2| data2[sub_offset]);
                Some(BlockData { layer1: block1, layer2: block2 })
            }
            None => Some(BlockData { layer1: AIR_INFO, layer2: None }),
        }
    }
}

impl World {
    pub fn open(path: &Path) -> Result<World> {
        let raw_world = RawWorld::open(path)?;
        Ok(World {
            raw_world,
            global_palette: RefCell::new(BlockTable::new()),
            chunk_cache: RefCell::new(FnvHashMap::default()),
        })
    }

    pub fn iter_chunks<'a>(&'a self) -> impl Iterator<Item = Result<ChunkPos>> + 'a {
        // only include chunks instead of subchunk granularity, and keep errors
        self.raw_world.iter_chunks().filter_map(|c| match c {
            Ok(pos) => {
                if pos.subchunk == 0 {
                    Some(Ok(ChunkPos {
                        x: pos.x,
                        z: pos.z,
                        dimension: pos.dimension,
                    }))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        })
    }

    fn translate_block_storage(&self, storage: &BlockStorage) -> Vec<BlockInfo> {
        storage
            .blocks
            .iter()
            .map(|b| {
                let description = &storage.palette[*b as usize];
                let block_id = self.global_palette
                    .borrow_mut()
                    .get_id(&description.name);
                BlockInfo {
                    block_id,
                    block_val: description.val,
                }
            })
            .collect()
    }

    fn load_subchunk(&self, pos: &SubchunkPos) -> Result<Option<ConvertedSubchunk>> {
        let maybe_sc = self.raw_world.load_chunk(pos)?;

        match maybe_sc {
            Some(sc) => {
                let count = sc.block_storages.len();
                assert!(
                    count == 1 || count == 2,
                    "should have one or two BlockStorages"
                );

                let bs1 = self.translate_block_storage(&sc.block_storages[0]);

                // second blockstorage might be missing
                let bs2 = sc
                    .block_storages
                    .get(1)
                    .map(|bs| self.translate_block_storage(&bs));

                Ok(Some(ConvertedSubchunk {
                    data1: bs1,
                    data2: bs2,
                }))
            }
            None => Ok(None),
        }
    }

    fn load_chunk(&self, pos: &ChunkPos) -> Result<Option<Chunk>> {
        const NUM_SUBCHUNKS: usize = 16;

        // If the bottom-most subchunk is not there, then the chunk has not been
        // stored in the world. Hence the bottom-most subchunk must be present.
        let bottom_subchunk = self.load_subchunk(&pos.subchunk_pos(0))?;

        if bottom_subchunk.is_some() {
            let mut subchunks = Vec::with_capacity(NUM_SUBCHUNKS);
            subchunks.push(bottom_subchunk);

            // load the other subchunks as well
            for i in 1..NUM_SUBCHUNKS {
                subchunks.push(self.load_subchunk(&pos.subchunk_pos(i as u8))?);
            }

            Ok(Some(Chunk { subchunks }))
        } else {
            // chunk is not present
            Ok(None)
        }
    }

    pub fn get_block(&self, pos: &WorldPos) -> Result<Option<BlockData>> {
        let chunk_pos = pos.chunk_pos();

        let mut cache = self.chunk_cache.borrow_mut();
        // try to load chunk from cache, and otherwise load from disk and put it
        // in the cache
        let maybe_chunk = if let Some(c) = cache.get(&chunk_pos) {
            c
        } else {
            let chunk = self.load_chunk(&chunk_pos)?;
            cache.insert(chunk_pos, chunk);
            &cache[&chunk_pos]
        };

        match maybe_chunk {
            Some(chunk) => Ok(chunk.get_block(pos)),
            None => Ok(None),
        }
    }

    pub fn block_id(&self, name: &str) -> BlockId {
        self.global_palette.borrow_mut().get_id(name)
    }

    pub fn block_name(&self, id: BlockId) -> String {
        self.global_palette.borrow_mut().get_name(id).to_owned()
    }
}
