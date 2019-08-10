use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use leveldb::database::iterator::DatabaseIterator;
use leveldb::database::Database;
use leveldb::options::{Compression, Options, ReadOptions, WriteOptions};
use std::io::{Cursor, Read, Write};
use std::path::Path;

use crate::error::*;
use crate::raw::encode::{encode_into_buffer, Encode};
use crate::raw::subchunk::Subchunk;
use crate::pos::*;

pub struct RawWorld {
    database: Database,
}

// fn test_roundtrip(chunk: &Subchunk) -> Result<()> {
//     let mut serialized = Vec::new();
//     chunk.serialize(&mut serialized)?;
//     let mut cursor = Cursor::new(&serialized);
//     let deserialized = Subchunk::deserialize(&mut cursor)?;

//     assert_eq!(chunk.block_storages.len(), deserialized.block_storages.len());

//     for (bs1, bs2) in chunk.block_storages.iter().zip(&deserialized.block_storages) {
//         for (b1, b2) in bs1.blocks.iter().zip(&bs2.blocks) {
//             assert_eq!(b1, b2);
//         }
//     }

//     Ok(())
// }

impl RawWorld {
    pub fn open(path: &Path) -> Result<RawWorld> {
        let mut options = Options::default();
        options.compression = Compression::ZlibRaw;

        let database = Database::open(path, options)?;

        Ok(RawWorld { database })
    }

    pub fn load_subchunk(&self, pos: &SubchunkPos) -> Result<Option<Subchunk>> {
        let mut key_buf = [0u8; 32];
        let key_slice = encode_into_buffer(pos, &mut key_buf[..])?;

        let read_options = ReadOptions::default();
        let maybe_data = self.database.get_bytes(&read_options, key_slice)?;

        if let Some(b) = maybe_data {
            let len = b.len();
            let mut cursor = Cursor::new(b);

            let chunk = Subchunk::deserialize(&mut cursor)?;
            // test_roundtrip(&chunk)?;

            // make sure we consume ALL of the data
            assert_eq!(cursor.position() as usize, len);

            Ok(Some(chunk))
        } else {
            Ok(None)
        }
    }

    pub fn save_subchunk(&self, pos: &SubchunkPos, sc: &Subchunk) -> Result<()> {
        let mut key_buf = [0u8; 32];
        let key_slice = encode_into_buffer(pos, &mut key_buf[..])?;

        let write_options = WriteOptions::default();

        let mut serialized = Vec::new();
        sc.serialize(&mut serialized)?;

        self.database.put(&write_options, key_slice, &serialized)?;

        Ok(())
    }

    pub fn iter_chunks(&self) -> SubchunkIterator {
        let read_options = ReadOptions::default();
        let dbiter = self.database.iter(&read_options);
        SubchunkIterator {
            iter: dbiter,
            state: SubchunkIteratorState::NotStarted,
        }
    }
}

enum SubchunkIteratorState {
    NotStarted,
    Started,
    Done,
}

pub struct SubchunkIterator<'a> {
    iter: DatabaseIterator<'a>,
    state: SubchunkIteratorState,
}

impl<'a> Iterator for SubchunkIterator<'a> {
    type Item = Result<SubchunkPos>;

    fn next(&mut self) -> Option<Result<SubchunkPos>> {
        match self.state {
            SubchunkIteratorState::Done => return None,
            SubchunkIteratorState::NotStarted => {
                self.iter.seek_to_first();
                self.state = SubchunkIteratorState::Started;
            }
            SubchunkIteratorState::Started => {}
        }

        // At this point, we now that the iterator is in the Started
        // state

        loop {
            if !self.iter.valid() {
                self.state = SubchunkIteratorState::Done;
                return None;
            }

            let key_slice = self.iter.key();
            if let Some(res) = try_decode_pos(key_slice) {
                // If an error occurs while trying to decode the
                // position key, then mark the iteration as done and
                // return the error
                if res.is_err() {
                    self.state = SubchunkIteratorState::Done;
                }

                self.iter.next();
                return Some(res);
            }

            // skip keys which do not represent subchunk block data
            self.iter.next();
        }
    }
}

fn try_decode_pos(key: &[u8]) -> Option<Result<SubchunkPos>> {
    // check if the one-to-last element of the key contains the subchunk
    // prefix, otherwise it does not contain block data
    if key[key.len() - 2] == SUBCHUNK_PREFIX {
        if key.len() == SUBCHUNK_KEY_LEN_OVERWORLD {
            Some(decode_pos(key, true))
        } else if key.len() == SUBCHUNK_KEY_LEN_OTHER {
            Some(decode_pos(key, false))
        } else {
            None
        }
    } else {
        None
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
