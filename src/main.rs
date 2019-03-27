#![allow(dead_code)]
use crate::rawworld::Dimension;
use crate::world::{World, WorldPos};
use std::path::Path;

mod encode;
mod rawchunk;
mod rawworld;
mod table;
mod world;

mod error {
    pub use failure::Error;
    pub type Result<T> = ::std::result::Result<T, Error>;
}

fn main() {
    let path = Path::new("/home/daniel/L6yaXFjeAAA=/db");
    println!("{:?}", path);
    let mut world = World::open(&path).unwrap();
    // let _test_chunk = world
    //     .load_chunk(SubchunkPos {
    //         x: 0,
    //         z: 0,
    //         subchunk: 4,
    //         dimension: Dimension::Overworld,
    //     })
    //     .unwrap()
    //     .unwrap();

    // let chunk_list = world.iter_chunks();

    // for c in chunk_list {
    //     println!("{:?}", c.unwrap());
    // }

    for x in 47..=67 {
        for y in 60..=80 {
            for z in -78..=-58 {
                let (block1, block2) = world
                    .get_block(&WorldPos {
                        x: x,
                        y: y,
                        z: z,
                        dimension: Dimension::Overworld,
                    })
                    .unwrap();

                println!("{:?}", world.global_palette.get_description(block1));
            }
        }
    }

    println!("Great success!");
}
