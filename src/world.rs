use crate::chunk::Chunk;
use crate::encode::{encode_into_buffer, Encode};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use leveldb::database::Database;
use leveldb::options::{Compression, Options, ReadOptions};
use std::io::{Cursor, Read, Write};
use std::path::Path;

use crate::error::*;

const SUBCHUNK_KEY_LEN_OVERWORLD: usize = 10;
const SUBCHUNK_KEY_LEN_OTHER: usize = 14;
const SUBCHUNK_PREFIX: u8 = 47;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Dimension {
    Overworld = 0,
    Nether = 1,
    End = 2,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SubchunkPos {
    pub x: i32,
    pub z: i32,
    pub subchunk: u8,
    pub dimension: Dimension,
}

impl Encode for SubchunkPos {
    type Error = Error;

    fn encode<T: Write>(&self, buf: &mut T) -> Result<()> {
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

pub struct World {
    database: Database,
}

impl World {
    pub fn open(path: &Path) -> Result<World> {
        let mut options = Options::new();
        options.compression = Compression::ZlibRaw;

        let database = Database::open(path, options)?;

        Ok(World { database })
    }

    pub fn load_chunk(&mut self, pos: SubchunkPos) -> Result<Option<Chunk>> {
        let mut key_buf = [0u8; 32];
        let key_slice = encode_into_buffer(&pos, &mut key_buf[..])?;

        let read_options = ReadOptions::new();
        let maybe_data = self.database.get_bytes(&read_options, key_slice)?;

        if let Some(b) = maybe_data {
            let mut cursor = Cursor::new(b);
            let chunk = Chunk::deserialize(&mut cursor)?;
            Ok(Some(chunk))
        } else {
            Ok(None)
        }
    }

    pub fn list_chunks(&mut self) -> Result<Vec<SubchunkPos>> {
        let mut result = Vec::new();
        let read_options = ReadOptions::new();
        let mut iter = self.database.iter(&read_options);
        iter.seek_to_first();
        while iter.valid() {
            let key_slice = iter.key();

            // check if the one-to-last element of the key contains the subchunk
            // prefix, otherwise it does not contain the block data
            if key_slice[key_slice.len() - 2] == SUBCHUNK_PREFIX {
                if key_slice.len() == SUBCHUNK_KEY_LEN_OVERWORLD {
                    result.push(decode_pos(key_slice, true)?);
                } else if key_slice.len() == SUBCHUNK_KEY_LEN_OTHER {
                    result.push(decode_pos(key_slice, false)?);
                }
            }

            iter.next();
        }
        Ok(result)
    }
}

fn decode_pos(key: &[u8], overworld: bool) -> Result<SubchunkPos> {
    let mut cursor = Cursor::new(key);
    let x = cursor.read_i32::<LittleEndian>()?;
    let z = cursor.read_i32::<LittleEndian>()?;
    let dimension = if !overworld {
        let dim = cursor.read_u32::<LittleEndian>()?;
        match dim {
            1 => Dimension::Nether,
            2 => Dimension::End,
            _ => panic!("unexpected dimension {} in decode_pos", dim),
        }
    } else {
        Dimension::Overworld
    };

    // read subchunk prefix and subchunk height, the prefix is unused and we
    // only look at the subchunk y position
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf)?;

    assert_eq!(buf[0], SUBCHUNK_PREFIX, "invalid subchunk prefix");
    let subchunk = buf[1];

    Ok(SubchunkPos {
        x,
        z,
        subchunk,
        dimension,
    })
}
