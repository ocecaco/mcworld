extern crate byteorder;
extern crate leveldb;

use std::path::Path;
use world::{ChunkPos, Dimension, World};

mod world;

mod error {
    use leveldb::error::Error as LevelDBError;

    #[derive(Debug)]
    pub struct Error;

    impl From<LevelDBError> for Error {
        fn from(_db_error: LevelDBError) -> Error {
            Error
        }
    }

    pub type Result<T> = ::std::result::Result<T, Error>;
}

fn main() {
    let path = Path::new("/home/daniel/mcpe_viz/4UIBAHQOAQA=/db");
    let world = World::open(&path).unwrap();
    let test_chunk = world
        .load_chunk(ChunkPos {
            x: 57,
            z: -9,
            subchunk: 4,
            dimension: Dimension::Overworld,
        })
        .unwrap()
        .unwrap();

    let mut counter = [0u32; 256];

    // for b in &test_chunk.blocks {
    //     counter[b.block_id as usize] += 1;
    // }

    // for (i, count) in counter.iter().enumerate() {
    //     println!("{}: {}", i, count);
    // }
    println!("Great success!");
}
