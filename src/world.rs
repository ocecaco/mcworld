use std::io::{Cursor, Write};
use byteorder::{LittleEndian, WriteBytesExt};
use leveldb::database::Database;
use leveldb::options::{Compression, Options, ReadOptions};
use std::path::Path;

use error::*;

mod nibbles {
    #[derive(Debug, Copy, Clone)]
    enum NibbleState {
        High,
        Low,
        Done,
    }

    pub struct Nibbles {
        high: u8,
        low: u8,
        state: NibbleState,
    }

    impl Iterator for Nibbles {
        type Item = u8; // Possible values: 0-16

        fn next(&mut self) -> Option<Self::Item> {
            let (result, next_state) = match self.state {
                NibbleState::High => (self.high, NibbleState::Low),
                NibbleState::Low => (self.low, NibbleState::Done),
                NibbleState::Done => return None,
            };

            self.state = next_state;

            Some(result)
        }
    }

    pub fn nibbles(byte: u8) -> Nibbles {
        Nibbles {
            high: byte >> 4,
            low: byte & 0b1111,
            state: NibbleState::High,
        }
    }
}

trait Encode {
    fn encode<T: Write>(&self, writer: &mut T);
}

const CHUNK_SIZE: usize = 4096;

#[derive(Debug, Copy, Clone)]
pub struct BlockInfo {
    pub block_id: u8,
    pub block_data: u8,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    blocks: Vec<BlockInfo>,
}

impl Chunk {
    fn deserialize(raw_data: &[u8]) -> Chunk {
        let without_version = &raw_data[1..];
        let (block_ids, block_data) = without_version.split_at(CHUNK_SIZE);
        let block_data_unpacked = block_data.iter().flat_map(|&x| nibbles::nibbles(x));

        let block_info = block_ids
            .iter()
            .cloned()
            .zip(block_data_unpacked)
            .map(|(id, data)| {
                BlockInfo {
                    block_id: id,
                    block_data: data,
                }
            })
            .collect::<Vec<_>>();

        assert_eq!(block_info.len(), CHUNK_SIZE);
        Chunk { blocks: block_info }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Dimension {
    Overworld = 0,
    Nether = 1,
    End = 2,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
    pub subchunk: u8,
    pub dimension: Dimension,
}

impl Encode for ChunkPos {
    fn encode<T: Write>(&self, buf: &mut T) {
        const SUBCHUNK_PREFIX: u8 = 47;
        buf.write_i32::<LittleEndian>(self.x).unwrap();
        buf.write_i32::<LittleEndian>(self.z).unwrap();
        if self.dimension != Dimension::Overworld {
            buf.write_u32::<LittleEndian>(self.dimension as u32)
                .unwrap();
        }
        buf.write_all(&[SUBCHUNK_PREFIX]).unwrap();
        buf.write_all(&[self.subchunk]).unwrap();
    }
}

fn encode_into_buffer<'a, 'b, T: Encode>(value: &'a T, buf: &'b mut [u8]) -> usize {
    let length = {
        let mut cursor = Cursor::new(buf);
        value.encode(&mut cursor);
        cursor.position() as usize
    };

    length
}

enum

pub struct ChunkItem<'a> {
    key_slice: &'a [u8],
    value_slice: &'a [u8],
}

impl<'a> ChunkItem<'a> {
    fn
}


pub struct ChunkIterate;

impl ChunkIterate {
    fn advance(&mut self) {
        unimplemented!();
    }

    fn get(&self) -> Option<ChunkItem> {
        unimplemented!();
    }

    fn next(&mut self) -> Option<ChunkItem> {
        self.advance();
        self.get()
    }
}

pub struct World {
    database: Database,
}

impl World {
    pub fn open(path: &Path) -> Result<World> {
        let mut options = Options::new();
        options.compression = Compression::ZlibRaw;

        let database = Database::open(path, options)?;

        Ok(World { database: database })
    }

    pub fn load_chunk(&self, pos: ChunkPos) -> Result<Option<Chunk>> {
        let mut key_buf = [0u8; 32];
        let key_length = encode_into_buffer(&pos, &mut key_buf[..]);
        let key_slice = &key_buf[..key_length];

        let read_options = ReadOptions::new();
        let maybe_data = self.database.get_bytes(&read_options, key_slice)?;

        Ok(maybe_data.map(|b| Chunk::deserialize(&b)))
    }

    pub fn iter_chunks(&self) -> ChunkIterate {}
}
