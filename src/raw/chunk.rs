use crate::error::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use nbt::{Blob, Value};
use std::io::Read;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PaletteEntry {
    pub name: String,
    pub val: u32,
}

struct Decoder<'a, T: 'a> {
    reader: &'a mut T,
}

impl<'a, T> Decoder<'a, T>
where
    T: Read,
{
    fn decode_chunk(&mut self) -> Result<Subchunk> {
        let version = self.reader.read_u8()?;
        assert_eq!(version, 8);

        let num_storages = self.reader.read_u8()?;

        let mut storages = Vec::new();
        for _ in 0..num_storages {
            storages.push(self.decode_storage()?);
        }

        Ok(Subchunk {
            block_storages: storages,
        })
    }

    fn decode_storage(&mut self) -> Result<BlockStorage> {
        let format = self.reader.read_u8()?;
        let network = 0b0000_0001 & format;
        assert_eq!(network, 0);
        let bits_per_block = u32::from(0b1111_1110 & format) >> 1;

        let blocks = self.decode_blocks(bits_per_block)?;
        let palette = self.decode_palette()?;

        Ok(BlockStorage { blocks, palette })
    }

    fn decode_blocks(&mut self, bits_per_block: u32) -> Result<Vec<u16>> {
        const CHUNK_SIZE: usize = 4096;

        let mut blocks = Vec::new();
        while blocks.len() < CHUNK_SIZE {
            let w = self.reader.read_u32::<LittleEndian>()?;
            unpack_word(w, bits_per_block, &mut blocks);
        }
        blocks.truncate(CHUNK_SIZE);

        Ok(blocks)
    }

    fn decode_palette(&mut self) -> Result<Vec<PaletteEntry>> {
        let mut palette = Vec::new();
        let num_entries = self.reader.read_u32::<LittleEndian>()?;

        for _ in 0..num_entries {
            let entry = self.decode_palette_entry()?;
            palette.push(entry);
        }

        Ok(palette)
    }

    fn decode_palette_entry(&mut self) -> Result<PaletteEntry> {
        let blob = Blob::from_reader(self.reader)?;
        let name = match blob["name"] {
            Value::String(ref s) => s.clone(),
            _ => panic!("no name field"),
        };
        let val = match blob["val"] {
            Value::Short(i) => i,
            _ => panic!("no val field"),
        };
        Ok(PaletteEntry {
            name,
            val: val as u32,
        })
    }
}

fn unpack_word(mut w: u32, bits_per_block: u32, output: &mut Vec<u16>) {
    const WORD_SIZE: u32 = 32;

    let num_blocks = WORD_SIZE / bits_per_block;

    // all_ones = 11111111111111111111111111111111
    let all_ones = !0u32;
    // lower_zeros = 11111111111111111111111111100000 (for bits_per_block = 5)
    let lower_zeros = all_ones << bits_per_block;
    // lower_ones = 00000000000000000000000000011111
    let lower_ones = !lower_zeros;
    // lower_ones is a mask with the lower bits_per_block bits set to 1

    for _ in 0..num_blocks {
        let b = w & lower_ones;
        output.push(b as u16);

        // shift to next block
        w >>= bits_per_block;
    }
}

#[derive(Debug, Clone)]
pub struct Subchunk {
    pub block_storages: Vec<BlockStorage>,
}

impl Subchunk {
    pub fn deserialize<T: Read>(reader: &mut T) -> Result<Subchunk> {
        let mut decoder = Decoder { reader };
        decoder.decode_chunk()
    }
}

#[derive(Debug, Clone)]
pub struct BlockStorage {
    pub blocks: Vec<u16>,
    pub palette: Vec<PaletteEntry>,
}
