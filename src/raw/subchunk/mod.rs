mod deserialize;
mod serialize;

use crate::error::Result;
pub use deserialize::*;
pub use serialize::*;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct Subchunk {
    pub block_storages: Vec<BlockStorage>,
}

#[derive(Debug, Clone)]
pub struct BlockStorage {
    pub blocks: Vec<u16>,
    pub palette: Vec<PaletteEntry>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PaletteEntry {
    pub name: String,
    pub val: u16,
}

impl Subchunk {
    pub fn deserialize<T: Read>(reader: &mut T) -> Result<Subchunk> {
        let mut decoder = Decoder::new(reader);
        decoder.decode_chunk()
    }

    pub fn serialize<T: Write>(&self, writer: &mut T) -> Result<()> {
        let mut encoder = Encoder::new(writer);
        encoder.encode_chunk(self)
    }
}
