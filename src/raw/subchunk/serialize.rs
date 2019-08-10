use std::io::Write;
use byteorder::{LittleEndian, WriteBytesExt};
use super::*;

pub struct Encoder<'a, T: 'a> {
    writer: &'a mut T,
}

impl<'a, T> Encoder<'a, T> {
    pub fn new(writer: &'a mut T) -> Self {
        Encoder { writer }
    }
}

impl<'a, T> Encoder<'a, T>
where
    T: Write,
{
    pub fn encode_chunk(&mut self, subchunk: &Subchunk) -> Result<()> {
        const VERSION: u8 = 8;
        self.writer.write_u8(VERSION)?;

        let num_storages = subchunk.block_storages.len();
        self.writer.write_u8(num_storages as u8)?;

        for s in &subchunk.block_storages {
            self.encode_storage(s)?;
        }

        Ok(())
    }

    pub fn encode_storage(&mut self, storage: &BlockStorage) -> Result<()> {
        unimplemented!();
    }
}

// fn msb_position(n: u32) -> u32 {
//     let pos = None;

//     while n != 0 {
//         n >>= 1;
//         pos += 1;
//     }


// }

// fn bits_per_block(num_palette_entries: usize) -> u8 {

// }
