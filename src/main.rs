#![allow(dead_code)]
#![allow(unused_imports)]
#![warn(clippy::all)]
use crate::rawworld::*;
use crate::table::NOT_PRESENT;
use crate::world::*;
use crate::pos::*;
use std::path::Path;

mod encode;
mod rawchunk;
mod rawworld;
mod table;
mod world;
mod pos;

mod error {
    pub use failure::Error;
    pub type Result<T> = ::std::result::Result<T, Error>;
}

fn main() {
    let path = Path::new("/home/daniel/mcpe/Ns4HXdObBQA=/db");
    println!("{:?}", path);
    let world = World::open(&path).unwrap();

    // let blk = world.get_block(&WorldPos {
    //     x: -80,
    //     y: 11,
    //     z: -1,
    //     dimension: Dimension::Overworld,
    // }).unwrap().0;

    // println!("{:?}", world.global_palette.borrow_mut().get_description(blk));

    // let chunk_positions = world.iter_chunks();

    // for pos in chunk_positions {
    let pos = ChunkPos {
        x: -21,
        z: 3,
        dimension: Dimension::Overworld,
    };

    // if pos.dimension != Dimension::Overworld {
    //     continue;
    // }

    for dy in 0..=255 {
        for dz in 0..16 {
            for dx in 0..16 {
                let world_pos = WorldPos {
                    x: 16 * pos.x + dx,
                    y: dy,
                    z: 16 * pos.z + dz,
                    dimension: Dimension::Overworld,
                };
                world.get_block(&world_pos).unwrap();
            }
        }
    }
    // }

    println!("Great success!");
}
