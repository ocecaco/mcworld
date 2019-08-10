use crate::error::*;
use crate::raw::encode::Encode;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
    pub dimension: Dimension,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorldPos {
    pub x: i32,
    pub y: u8,
    pub z: i32,
    pub dimension: Dimension,
}

impl WorldPos {
    pub fn chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: flooring_divide(self.x, 16),
            z: flooring_divide(self.z, 16),
            dimension: self.dimension,
        }
    }

    pub fn subchunk_y(&self) -> usize {
        let sub_y = self.y / 16;
        assert!(sub_y < 16);
        sub_y as usize
    }

    pub fn subchunk_offset(&self) -> usize {
        let inner_y = i32::from(self.y % 16);
        let inner_x = self.x - flooring_divide(self.x, 16) * 16;
        let inner_z = self.z - flooring_divide(self.z, 16) * 16;

        assert!(inner_x >= 0 && inner_x < 16);
        assert!(inner_y >= 0 && inner_y < 16);
        assert!(inner_z >= 0 && inner_z < 16);

        (16 * 16 * inner_x + 16 * inner_z + inner_y) as usize
    }
}

impl ChunkPos {
    pub fn subchunk_pos(&self, subchunk: u8) -> SubchunkPos {
        SubchunkPos {
            x: self.x,
            z: self.z,
            subchunk,
            dimension: self.dimension,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Dimension {
    Overworld = 0,
    Nether = 1,
    End = 2,
}

pub const SUBCHUNK_KEY_LEN_OVERWORLD: usize = 10;
pub const SUBCHUNK_KEY_LEN_OTHER: usize = 14;
pub const SUBCHUNK_PREFIX: u8 = 47;

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

fn flooring_divide(n: i32, k: u32) -> i32 {
    let k = k as i32;
    let div = n / k;
    let rem = n - div * k;

    // no need for fancy rounding if the remainder is 0
    if rem == 0 {
        return div;
    }

    // otherwise fix up the negative numbers to make the rounding go to negative
    //  infinity instead of zero
    if n < 0 {
        div - 1
    } else {
        div
    }
}
