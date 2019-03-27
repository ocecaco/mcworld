#![allow(dead_code)]
use crate::world::{Dimension, SubchunkPos, World};
use std::path::Path;

mod chunk;
mod encode;
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
    let _test_chunk = world
        .load_chunk(SubchunkPos {
            x: 0,
            z: 0,
            subchunk: 4,
            dimension: Dimension::Overworld,
        })
        .unwrap()
        .unwrap();

    let chunk_list = world.list_chunks().unwrap();

    for c in chunk_list {
        println!("{:?}", c);
    }

    println!("Great success!");
}
