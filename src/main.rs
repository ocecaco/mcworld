#![allow(dead_code)]
#![allow(unused_imports)]
use crate::rawworld::Dimension;
use crate::rawworld::SubchunkPos;
use crate::table::NOT_PRESENT;
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
    let world = World::open(&path).unwrap();

    let chunk_positions = world.iter_chunks();

    for pos in chunk_positions {
        let pos = pos.unwrap();

        if pos.dimension != Dimension::Overworld {
            continue;
        }

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
    }

    println!("Great success!");
}
