use std::io::Write;
use nbt::{Blob, Value};
use byteorder::{LittleEndian, WriteBytesExt};
use std::convert::TryInto;
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

    fn encode_storage(&mut self, storage: &BlockStorage) -> Result<()> {
        let bits_per_block = bits_per_block(storage.palette.len());

        // the least significant bit of the format indicates whether
        // we are serializing for the network. we are not, hence we
        // leave it at 0.
        let format = bits_per_block << 1;
        self.writer.write_u8(format)?;

        self.encode_blocks(&storage.blocks, bits_per_block)?;
        self.encode_palette(&storage.palette)?;

        Ok(())
    }

    fn encode_blocks(&mut self, blocks: &[u16], bits_per_block: u8) -> Result<()> {
        const CHUNK_SIZE: usize = 4096;
        assert_eq!(blocks.len(), CHUNK_SIZE);

        let blocks_per_word = 32 / bits_per_block;

        let chunked = blocks.chunks(usize::from(blocks_per_word));

        for c in chunked {
            let packed = pack_word(c, bits_per_block);
            self.writer.write_u32::<LittleEndian>(packed)?;
        }

        Ok(())
    }

    fn encode_palette(&mut self, palette: &[PaletteEntry]) -> Result<()> {
        let num_entries = palette.len();
        self.writer.write_u32::<LittleEndian>(num_entries as u32)?;

        for e in palette {
            self.encode_palette_entry(e)?;
        }

        Ok(())
    }

    fn encode_palette_entry(&mut self, entry: &PaletteEntry) -> Result<()> {
        let mut nbt = Blob::new();
        nbt.insert("name".to_owned(), Value::String(entry.name.clone())).unwrap();
        nbt.insert("val".to_owned(), Value::Short(entry.val.try_into().unwrap())).unwrap();

        nbt.to_writer(self.writer)?;

        Ok(())
    }
}

fn bits_per_block(num_palette_entries: usize) -> u8 {
    const OPTIONS: [u8; 8] = [1u8, 2, 3, 4, 5, 6, 8, 16];

    // find the smallest number of bits per block that would be big
    // enough to hold all possibilities
    for o in &OPTIONS {
        let max_entries = 1usize << o;

        if num_palette_entries <= max_entries {
            return *o;
        }
    }

    panic!("palette too big");
}

fn pack_word(blocks: &[u16], bits_per_block: u8) -> u32 {
    let mut result = !0u32;

    // check that the blocks will fit inside a word
    let blocks_per_word = 32 / bits_per_block;
    assert!(blocks.len() <= usize::from(blocks_per_word));

    let max_index = 1u16 << bits_per_block;

    for b in blocks.iter().rev() {
        // check that the index is not too high
        assert!(*b < max_index);

        // create a space for the new block
        result <<= bits_per_block;

        // mask the block into position
        result |= u32::from(*b);
    }

    result
}
