use std::collections::HashMap;
use std::path::Path;

use crate::rawchunk::RawBlockStorage;
use crate::rawworld::{Dimension, RawWorld, SubchunkPos};
use crate::table::{BlockId, BlockTable, AIR, NOT_PRESENT};

use crate::error::*;

const NUM_SUBCHUNKS: usize = 16;

pub struct World {
    pub raw_world: RawWorld,
    pub global_palette: BlockTable,
    chunk_cache: HashMap<ChunkPos, Option<Chunk>>,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
    pub dimension: Dimension,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorldPos {
    pub x: i32,
    pub y: u8,
    pub z: i32,
    pub dimension: Dimension,
}

impl WorldPos {
    fn to_chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: flooring_divide(self.x, 16),
            z: flooring_divide(self.z, 16),
            dimension: self.dimension,
        }
    }
}

impl ChunkPos {
    fn subchunk_pos(&self, subchunk: u8) -> SubchunkPos {
        SubchunkPos {
            x: self.x,
            z: self.z,
            subchunk,
            dimension: self.dimension,
        }
    }
}

// uses indices into table stored in the World instead of a separate palette for
// each subchunk
struct ConvertedSubchunk {
    data1: Vec<BlockId>,
    data2: Option<Vec<BlockId>>,
}

struct Chunk {
    // subchunks might be missing, which means they are filled with air. The
    // length of this vector should always be exactly 16, since that is the
    // number of subchunks per chunk.
    subchunks: Vec<Option<ConvertedSubchunk>>,
}

fn flooring_divide(n: i32, k: u32) -> i32 {
    let k = k as i32;
    let div = n / k;
    let rem = n - div * k;

    // no need for fancy rounding if the remainder is 0
    if rem == 0 {
        return div;
    }

    // otherwise fix up the negative numbers to make the rounding go to negative
    //  infinity instead of zero
    if n < 0 {
        div - 1
    } else {
        div
    }
}

impl Chunk {
    fn get_block(&self, w: &WorldPos) -> (BlockId, BlockId) {
        let subchunk_offset = w.y / 16;
        println!("subchunk offset: {:?}", subchunk_offset);
        // Not really sure why this formula is required... The heights don't
        // seem to be stored from y = 0 to y = 15 but in a different order.
        let inner_y = (7 - w.y % 16) % 16;
        let inner_x = w.x - flooring_divide(w.x, 16) * 16;
        println!("w.z: {}", w.z);
        let inner_z = w.z - flooring_divide(w.z, 16) * 16;

        assert!(inner_x >= 0 && inner_x < 16);
        assert!(inner_y < 16);
        assert!(inner_z >= 0 && inner_z < 16);
        assert!(subchunk_offset < 16);

        // TODO: Correct order?
        let final_offset = (16 * 16 * inner_x + 16 * inner_z + inner_y as i32) as usize;

        let maybe_subchunk = &self.subchunks[subchunk_offset as usize];
        match maybe_subchunk {
            Some(subchunk) => {
                let block1 = subchunk.data1[final_offset];
                let block2 = if let Some(data2) = &subchunk.data2 {
                    data2[final_offset]
                } else {
                    NOT_PRESENT
                };
                (block1, block2)
            }
            None => (AIR, NOT_PRESENT),
        }
    }
}

impl World {
    pub fn open(path: &Path) -> Result<World> {
        let raw_world = RawWorld::open(path)?;
        Ok(World {
            raw_world,
            global_palette: BlockTable::new(),
            chunk_cache: HashMap::new(),
        })
    }

    pub fn iter_chunks<'a>(&'a mut self) -> impl Iterator<Item = Result<ChunkPos>> + 'a {
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

    fn translate_block_storage(&mut self, storage: &RawBlockStorage) -> Vec<BlockId> {
        storage
            .blocks
            .iter()
            .map(|b| self.global_palette.get_id(&storage.palette[*b as usize]))
            .collect()
    }

    fn load_subchunk(&mut self, pos: &SubchunkPos) -> Result<Option<ConvertedSubchunk>> {
        let maybe_sc = self.raw_world.load_chunk(pos)?;

        println!("subchunk pos: {:?}", pos);

        match maybe_sc {
            Some(sc) => {
                let count = sc.block_storages.len();
                assert!(
                    count == 1 || count == 2,
                    "should have at one or two BlockStorages"
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

    fn load_chunk(&mut self, pos: &ChunkPos) -> Result<Option<Chunk>> {
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

    pub fn get_block(&mut self, pos: &WorldPos) -> Result<(BlockId, BlockId)> {
        let chunk_pos = pos.to_chunk_pos();
        println!("chunk pos: {:?}", chunk_pos);

        // try to load chunk from cache, and otherwise load from disk and put it
        // in the cache
        let maybe_chunk = if let Some(c) = self.chunk_cache.get(&chunk_pos) {
            c
        } else {
            let chunk = self.load_chunk(&chunk_pos)?;
            self.chunk_cache.insert(chunk_pos.clone(), chunk);
            &self.chunk_cache[&chunk_pos]
        };

        match maybe_chunk {
            Some(chunk) => Ok(chunk.get_block(pos)),
            None => Ok((NOT_PRESENT, NOT_PRESENT)),
        }
    }
}
