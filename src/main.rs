#![allow(dead_code)]
use crate::rawworld::Dimension;
use crate::rawworld::SubchunkPos;
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

    let positions: Vec<_> = world.raw_world.iter_chunks().collect();

    for sc in positions {
        let sc = sc.unwrap();

        let subchunk = world.raw_world.load_chunk(&sc).unwrap().unwrap();

        if let Some(magma_id) = subchunk.block_storages[0]
            .palette
            .iter()
            .position(|d| d.name.contains("redstone_block"))
        {
            if sc.dimension == Dimension::Overworld {
                println!("sc: {:?}", sc);
                let block_positions = subchunk.block_storages[0]
                    .blocks
                    .iter()
                    .cloned()
                    .enumerate()
                    .filter(|(i, b)| *b == magma_id as u16);
                for (i, _) in block_positions {
                    println!("found at: {}", i);
                }
                println!(
                    "name: {}",
                    subchunk.block_storages[0].palette[magma_id].name
                );
                println!("val: {}", subchunk.block_storages[0].palette[magma_id].val);
            }
        }
    }

    // println!(
    //     "{:?}",
    //     test_chunk.block_storages[0]
    //         .blocks
    //         .iter()
    //         .position(|&b| b == 7)
    // );
    // println!("{:?}", test_chunk.block_storages[0].palette.len());

    // for x in 47..=67 {
    //     for y in 60..=80 {
    //         for z in -78..=-58 {
    //             let (block1, block2) = world
    //                 .get_block(&WorldPos {
    //                     x: x,
    //                     y: y,
    //                     z: z,
    //                     dimension: Dimension::Overworld,
    //                 })
    //                 .unwrap();

    //             println!("{:?}", world.global_palette.get_description(block1));
    //         }
    //     }
    // }

    println!("Great success!");
}
