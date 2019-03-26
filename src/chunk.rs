use crate::error::Result;
use crate::table::{BlockId, BlockTable};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use nbt::{Blob, Value};
use std::io::{Cursor, Read, Write};

struct Decoder<'a, 'b, T: 'a> {
    reader: &'a mut T,
    table: &'b mut BlockTable,
}

impl<'a, 'b, T> Decoder<'a, 'b, T>
where
    T: Read,
{
    fn decode_chunk(&mut self) -> Result<Chunk> {
        let version = self.reader.read_u8()?;
        assert_eq!(version, 8);

        let num_storages = self.reader.read_u8()?;

        let mut storages = Vec::new();
        for _ in 0..num_storages {
            storages.push(self.decode_storage()?);
        }

        Ok(Chunk {
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

        let translated = blocks.iter().map(|i| palette[*i as usize]).collect();

        Ok(BlockStorage { blocks: translated })
    }

    fn decode_blocks(&mut self, bits_per_block: u32) -> Result<Vec<u32>> {
        const CHUNK_SIZE: usize = 4096;

        let mut blocks = Vec::new();
        while blocks.len() < CHUNK_SIZE {
            let w = self.reader.read_u32::<LittleEndian>()?;
            unpack_word(w, bits_per_block, &mut blocks);
        }
        blocks.truncate(CHUNK_SIZE);

        Ok(blocks)
    }

    fn decode_palette(&mut self) -> Result<Vec<BlockInfo>> {
        let mut palette = Vec::new();
        let num_entries = self.reader.read_u32::<LittleEndian>()?;

        for _ in 0..num_entries {
            let (name, val) = self.decode_palette_entry()?;
            let id = self.table.get_id(name);
            palette.push(BlockInfo { id, val });
        }

        Ok(palette)
    }

    fn decode_palette_entry(&mut self) -> Result<(String, u32)> {
        let blob = Blob::from_reader(self.reader)?;
        let name = match blob["name"] {
            Value::String(ref s) => s.clone(),
            _ => panic!("no name field"),
        };
        let val = match blob["val"] {
            Value::Short(i) => i,
            _ => panic!("no val field"),
        };
        Ok((name, val as u32))
    }
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

// #[derive(Debug, Clone)]
// struct Encoder<'a, T: 'a> {
//     writer: &'a mut T,
// }

// impl<'a, T: Write>  for  {

// }

#[derive(Debug, Clone)]
pub struct Chunk {
    block_storages: Vec<BlockStorage>,
}

impl Chunk {
    pub fn deserialize<T: Read>(reader: &mut T, table: &mut BlockTable) -> Result<Chunk> {
        let mut decoder = Decoder { reader, table };
        decoder.decode_chunk()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BlockInfo {
    id: BlockId,
    val: u32,
}

#[derive(Debug, Clone)]
pub struct BlockStorage {
    blocks: Vec<BlockInfo>,
}
