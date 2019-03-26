use crate::chunk::Chunk;
use crate::encode::{encode_into_buffer, Encode};
use crate::table::BlockTable;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use leveldb::database::Database;
use leveldb::options::{Compression, Options, ReadOptions};
use std::io::{Cursor, Read, Write};
use std::path::Path;

use crate::error::*;

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

pub struct ChunkIterate;

impl Iterator for ChunkIterate {
    type Item = SubchunkPos;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub struct World {
    database: Database,
    table: BlockTable,
}

impl World {
    pub fn open(path: &Path) -> Result<World> {
        let mut options = Options::new();
        options.compression = Compression::ZlibRaw;

        let database = Database::open(path, options)?;

        Ok(World {
            database,
            table: BlockTable::new(),
        })
    }

    pub fn load_chunk(&mut self, pos: SubchunkPos) -> Result<Option<Chunk>> {
        let mut key_buf = [0u8; 32];
        let key_slice = encode_into_buffer(&pos, &mut key_buf[..])?;

        let read_options = ReadOptions::new();
        let maybe_data = self.database.get_bytes(&read_options, key_slice)?;

        if let Some(b) = maybe_data {
            let mut cursor = Cursor::new(b);
            let chunk = Chunk::deserialize(&mut cursor, &mut self.table)?;
            Ok(Some(chunk))
        } else {
            Ok(None)
        }
    }
}
