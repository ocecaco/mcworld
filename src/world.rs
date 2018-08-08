use std::io::{Cursor, Read, Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use leveldb::database::Database;
use leveldb::options::{Compression, Options, ReadOptions};
use std::path::Path;
use nbt::{Blob, Value};

use error::*;

pub fn decode_chunk<T: Read>(reader: &mut T) -> Result<Chunk> {
    let version = reader.read_u8()?;
    assert_eq!(version, 8);

    let num_storages = reader.read_u8()?;

    let mut storages = Vec::new();
    for _ in 0..num_storages {
        storages.push(decode_storage(reader)?);
    }

    Ok(Chunk { block_storages: storages })
}

fn decode_storage<T: Read>(reader: &mut T) -> Result<BlockStorage> {
    let format = reader.read_u8()?;
    let network = 0b0000_0001 & format;
    assert_eq!(network, 0);
    let bits_per_block = ((0b1111_1110 & format) as u32) >> 1;

    let mut blocks = Vec::new();
    const CHUNK_SIZE: usize = 4096;
    while blocks.len() < CHUNK_SIZE {
        let w = reader.read_u32::<LittleEndian>()?;
        unpack_word(w, bits_per_block, &mut blocks);
    }

    let mut palette = Vec::new();
    let num_entries = reader.read_u32::<LittleEndian>()?;
    for _ in 0..num_entries {
        let blob = Blob::from_reader(reader)?;
        let name = match blob["name"] {
            Value::String(ref s) => s.clone(),
            _ => panic!("no name field"),
        };
        let val = match blob["val"] {
            Value::Short(i) => i,
            _ => panic!("no val field"),
        };
        palette.push((name, val as u32));
    }

    Ok(BlockStorage { blocks: blocks, palette: palette })
}

fn unpack_word(mut w: u32, bits_per_block: u32, output: &mut Vec<u32>) {
    const WORD_SIZE: u32 = 32;

    let num_blocks = WORD_SIZE / bits_per_block;

    // mask with upper bits_per_block bits set to 1
    let mask = !((!0u32 << bits_per_block) >> bits_per_block);
    let shift_correction = WORD_SIZE - bits_per_block;

    for _ in 0..num_blocks {
        let b = (w & mask) >> shift_correction;
        output.push(b);

        // shift to next block
        w <<= bits_per_block;
    }
}

// fn decode_varint<T: Read>(reader: &mut T) -> Result<i32> {
//     let mut result = 0i32;
//     let mut num_read = 0;

//     loop {
//         let b = reader.read_u8()?;
//         let value = b & 0b0111_1111;
//         result |= (value as i32) << (7 * num_read);
//         num_read += 1;

//         if b & 0b1000_0000 == 0 {
//             break;
//         }
//     }

//     Ok(result)
// }

trait Encode {
    type Error;
    fn encode<T: Write>(&self, writer: &mut T) -> ::std::result::Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub struct Chunk {
    block_storages: Vec<BlockStorage>,
}

#[derive(Debug, Clone)]
pub struct BlockStorage {
    blocks: Vec<u32>,
    palette: Vec<(String, u32)>,
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
    type Error = Error;

    fn encode<T: Write>(&self, buf: &mut T) -> Result<()> {
        const SUBCHUNK_PREFIX: u8 = 47;
        buf.write_i32::<LittleEndian>(self.x)?;
        buf.write_i32::<LittleEndian>(self.z)?;
        if self.dimension != Dimension::Overworld {
            buf.write_u32::<LittleEndian>(self.dimension as u32)?;
        }
        buf.write_all(&[SUBCHUNK_PREFIX])?;
        buf.write_all(&[self.subchunk])?;
        Ok(())
    }
}

fn encode_into_buffer<'a, 'b, T>(value: &'a T, buf: &'b mut [u8]) -> ::std::result::Result<&'b mut [u8], T::Error>
where T: Encode {
    let (length, buf2) = {
        let mut cursor = Cursor::new(buf);
        value.encode(&mut cursor)?;
        (cursor.position() as usize, cursor.into_inner())
    };

    Ok(&mut buf2[..length])
}

pub struct ChunkIterate;

impl Iterator for ChunkIterate {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        None
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
        let key_slice = encode_into_buffer(&pos, &mut key_buf[..])?;

        let read_options = ReadOptions::new();
        let maybe_data = self.database.get_bytes(&read_options, key_slice)?;

        if let Some(b) = maybe_data {
            let mut cursor = Cursor::new(b);
            let chunk = decode_chunk(&mut cursor)?;
            Ok(Some(chunk))
        } else {
            Ok(None)
        }
    }

    // pub fn iter_chunks(&self) -> ChunkIterate {
    // }
}
