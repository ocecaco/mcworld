#![allow(dead_code)]
#![allow(unused_imports)]
extern crate byteorder;
extern crate leveldb;
extern crate nbt;
extern crate failure;

use std::path::Path;
use world::{SubchunkPos, Dimension, World};

mod world;
mod chunk;
mod encode;
mod table;

mod error {
    pub use failure::Error;
    pub type Result<T> = ::std::result::Result<T, Error>;
}

fn main() {
    let path = Path::new("/home/daniel/Shared/iTunes/games/com.mojang/minecraftWorlds/niABAF6qAgA=/db");
    let mut world = World::open(&path).unwrap();
    let _test_chunk = world
        .load_chunk(SubchunkPos {
            x: 0,
            z: 0,
            subchunk: 4,
            dimension: Dimension::Overworld,
        })
        .unwrap()
        .unwrap();

    println!("Great success!");
}
