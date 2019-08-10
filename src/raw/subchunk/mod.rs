mod deserialize;
mod serialize;

use crate::error::Result;
use std::io::Read;
pub use deserialize::*;
pub use serialize::*;

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
    pub val: u32,
}

impl Subchunk {
    pub fn deserialize<T: Read>(reader: &mut T) -> Result<Subchunk> {
        let mut decoder = Decoder::new(reader);
        decoder.decode_chunk()
    }
}
