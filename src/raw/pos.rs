use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;
use crate::raw::encode::Encode;
use crate::Dimension;
use crate::error::*;

pub(crate) const SUBCHUNK_KEY_LEN_OVERWORLD: usize = 10;
pub(crate) const SUBCHUNK_KEY_LEN_OTHER: usize = 14;
pub(crate) const SUBCHUNK_PREFIX: u8 = 47;

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
