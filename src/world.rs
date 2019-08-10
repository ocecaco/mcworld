use std::cell::RefCell;
use std::path::Path;
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use std::collections::hash_map::Entry;

use crate::raw::RawWorld;
use crate::raw::subchunk::{Subchunk, BlockStorage, PaletteEntry};
use crate::pos::*;
use crate::table::{BlockId, BlockTable, AIR};
use crate::error::*;

const AIR_INFO: BlockInfo = BlockInfo { block_id: AIR, block_val: 0 };
const NUM_SUBCHUNKS: u8 = 16;
const CHUNK_SIZE: usize = 4096;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BlockInfo {
    pub block_id: BlockId,
    pub block_val: u16,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BlockData {
    pub layer1: BlockInfo,
    pub layer2: BlockInfo,
}

type ChunkCache = FnvHashMap<ChunkPos, Option<Chunk>>;

pub struct World {
    raw_world: RawWorld,
    global_palette: RefCell<BlockTable>,
    chunk_cache: RefCell<ChunkCache>,
}

// uses indices into table stored in the World instead of a separate palette for
// each subchunk
#[derive(Debug, Clone)]
struct WorldSubchunk {
    data1: Vec<BlockInfo>,
    data2: Vec<BlockInfo>,
}

#[derive(Debug, Clone)]
struct Chunk {
    // this vector should always hold 16 subchunks
    subchunks: Vec<WorldSubchunk>,
}

impl Chunk {
    fn get_block(&self, w: &WorldPos) -> BlockData {
        let sub_y = w.subchunk_y();
        let sub_offset = w.subchunk_offset();

        let subchunk = &self.subchunks[sub_y];
        let block1 = subchunk.data1[sub_offset];
        let block2 = subchunk.data2[sub_offset];
        BlockData { layer1: block1, layer2: block2 }
    }

    fn set_block(&mut self, w: &WorldPos, d: BlockData) {
        let sub_y = w.subchunk_y();
        let sub_offset = w.subchunk_offset();

        let subchunk = &mut self.subchunks[sub_y];

        subchunk.data1[sub_offset] = d.layer1;
        subchunk.data2[sub_offset] = d.layer2;
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


    fn load_subchunk(&self, pos: &SubchunkPos) -> Result<Option<WorldSubchunk>> {
        let maybe_sc = self.raw_world.load_subchunk(pos)?;

        Ok(maybe_sc.as_ref().map(|sc| self.convert_subchunk(sc)))
    }

    fn save_subchunk(&self, pos: &SubchunkPos, sc: &WorldSubchunk) -> Result<()> {
        let converted = self.convert_world_subchunk(sc);
        self.raw_world.save_subchunk(pos, &converted)?;

        Ok(())
    }

    fn load_subchunk_or_air(&self, pos: &SubchunkPos) -> Result<WorldSubchunk> {
        let maybe_sc = self.load_subchunk(pos)?;
        match maybe_sc {
            Some(sc) => Ok(sc),
            None => Ok(create_air_subchunk()),
        }
    }

    fn convert_subchunk(&self, sc: &Subchunk) -> WorldSubchunk {
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
            .map(|bs| self.translate_block_storage(&bs))
            .unwrap_or_else(create_air_layer);

        WorldSubchunk {
            data1: bs1,
            data2: bs2,
        }
    }

    fn create_palette(&self, layer: &[BlockInfo]) -> (FnvHashMap<BlockInfo, u16>, Vec<PaletteEntry>) {
        let unique_blocks: FnvHashSet<BlockInfo> = layer.iter().cloned().collect();
        let unique_blocks: Vec<BlockInfo> = unique_blocks.iter().cloned().collect();

        // mapping from BlockInfo to index in the palette
        let mapping = unique_blocks.iter().enumerate().map(|(i, bi)| (*bi, i as u16)).collect();

        // create a palette by looking up the names corresponding to
        // the block IDs
        let palette = unique_blocks.iter().map(|bi| PaletteEntry {
            name: self.block_name(bi.block_id),
            val: bi.block_val,
        }).collect();

        (mapping, palette)
    }

    fn convert_world_layer(&self, layer: &[BlockInfo]) -> BlockStorage {
        let (mapping, palette) = self.create_palette(layer);
        let paletted_blocks = layer.iter().map(|bi| mapping[bi]).collect();

        BlockStorage {
            blocks: paletted_blocks,
            palette,
        }
    }

    fn convert_world_subchunk(&self, sc: &WorldSubchunk) -> Subchunk {
        let mut layers = Vec::new();

        layers.push(self.convert_world_layer(&sc.data1));
        layers.push(self.convert_world_layer(&sc.data2));

        Subchunk {
            block_storages: layers,
        }
    }

    fn load_chunk(&self, pos: &ChunkPos) -> Result<Option<Chunk>> {
        // If the bottom-most subchunk is not there, then the chunk has not been
        // stored in the world. Hence the bottom-most subchunk must be present.
        let maybe_first = self.load_subchunk(&pos.subchunk_pos(0))?;

        if let Some(first) = maybe_first {
            let mut subchunks = Vec::with_capacity(usize::from(NUM_SUBCHUNKS));
            subchunks.push(first);

            // load the other subchunks as well
            for i in 1..NUM_SUBCHUNKS {
                subchunks.push(self.load_subchunk_or_air(&pos.subchunk_pos(i as u8))?);
            }

            Ok(Some(Chunk { subchunks }))
        } else {
            // chunk is not present
            Ok(None)
        }
    }

    fn do_save_chunk(&self, pos: &ChunkPos, chunk: &Chunk) -> Result<()> {
        // TODO: Optimize so subchunks filled with air at the top of
        // the world do not get saved.
        for i in 0..NUM_SUBCHUNKS {
            self.save_subchunk(&pos.subchunk_pos(i), &chunk.subchunks[usize::from(i)])?;
        }

        Ok(())
    }

    fn do_delete_chunk(&self, pos: &ChunkPos) -> Result<()> {
        for i in 0..NUM_SUBCHUNKS {
            self.raw_world.delete_subchunk(&pos.subchunk_pos(i))?;
        }

        Ok(())
    }

    fn cached_chunk<'a>(&self, cache: &'a mut ChunkCache, chunk_pos: ChunkPos) -> Result<&'a mut Option<Chunk>> {
        let entry = cache.entry(chunk_pos);

        // try to load chunk from cache, and otherwise load from disk and put it
        // in the cache
        if let Entry::Vacant(v) = entry {
            let chunk = self.load_chunk(&chunk_pos)?;
            v.insert(chunk);
        }

        Ok(cache.get_mut(&chunk_pos).unwrap())
    }

    pub fn get_block(&self, pos: &WorldPos) -> Result<Option<BlockData>> {
        let mut cache = self.chunk_cache.borrow_mut();
        let maybe_chunk = self.cached_chunk(&mut cache, pos.chunk_pos())?;

        match maybe_chunk {
            Some(chunk) => Ok(Some(chunk.get_block(pos))),
            None => Ok(None),
        }
    }

    pub fn set_block(&self, pos: &WorldPos, data: BlockData) -> Result<()> {
        let mut cache = self.chunk_cache.borrow_mut();
        let maybe_chunk = self.cached_chunk(&mut cache, pos.chunk_pos())?;

        match maybe_chunk {
            Some(chunk) => {
                chunk.set_block(pos, data);
                Ok(())
            },
            None => panic!("out of bounds"),
        }
    }

    pub fn delete_chunk(&self, pos: ChunkPos) -> Result<()> {
        let mut cache = self.chunk_cache.borrow_mut();
        cache.insert(pos, None);
        Ok(())
    }

    pub fn add_chunk(&self, pos: ChunkPos) -> Result<()> {
        let mut cache = self.chunk_cache.borrow_mut();
        cache.insert(pos, Some(create_air_chunk()));
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let cache = self.chunk_cache.borrow();

        for (pos, chunk) in cache.iter() {
            if let Some(c) = chunk {
                self.do_save_chunk(pos, c)?;
            } else {
                self.do_delete_chunk(pos)?;
            }
        }

        Ok(())
    }

    pub fn block_id(&self, name: &str) -> BlockId {
        self.global_palette.borrow_mut().get_id(name)
    }

    pub fn block_name(&self, id: BlockId) -> String {
        self.global_palette.borrow_mut().get_name(id).to_owned()
    }
}

fn create_air_layer() -> Vec<BlockInfo> {
    vec![AIR_INFO ;CHUNK_SIZE]
}

fn create_air_subchunk() -> WorldSubchunk {
    let blocks = create_air_layer();

    WorldSubchunk {
        data1: blocks.clone(),
        data2: blocks.clone(),
    }
}

fn create_air_chunk() -> Chunk {
    let sc = create_air_subchunk();
    let subchunks = vec![sc.clone(); usize::from(NUM_SUBCHUNKS)];
    Chunk {
        subchunks,
    }
}
