#![allow(dead_code)]
#![allow(unused_imports)]
#![warn(clippy::all)]
use crate::table::{BlockId, AIR};
use crate::world::{World, BlockData, BlockInfo};
use crate::pos::{WorldPos, Dimension};
use crate::neighbor::NeighborIterator;
use crate::error::*;

use std::path::Path;
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use std::collections::VecDeque;
use std::collections::hash_map::Entry;
use std::io::{self, Write};

mod table;
mod world;
mod pos;
mod neighbor;
mod raw;

mod error {
    pub use failure::Error;
    pub type Result<T> = ::std::result::Result<T, Error>;
}

type ParentMap = FnvHashMap<WorldPos, WorldPos>;

fn is_inside(world: &World, mut pos: WorldPos) -> Result<Option<bool>> {
    let data = world.get_block(&pos)?;

    // stop if the starting block is not air
    if let Some(blk) = data {
        if blk.layer1.block_id != AIR {
            return Ok(Some(false));
        }
    } else {
        return Ok(None);
    }

    // go up in height until we find a non-air block or we hit the ceiling
    loop {
        pos.y += 1;
        let data = world.get_block(&pos)?;
        let data = data.expect("should never go out of world bounds when increasing y since y is a u8");
        if data.layer1.block_id != AIR || pos.y == 255 {
            break;
        }
    }

    // we are inside if we did not hit the world ceiling before stopping
    Ok(Some(pos.y != 255))
}

// fn is_air(world: &World, pos: WorldPos) -> Result<bool> {
//     if let Some(blk) = world.get_block(&pos)? {
//         Ok(blk.layer1.block_id == AIR)
//     } else {
//         Ok(false)
//     }
// }

fn bfs(world: &World, start_pos: WorldPos) -> Result<(FnvHashSet<WorldPos>, ParentMap)> {
    let mut parents = FnvHashMap::default();
    let mut queue = VecDeque::new();
    let mut clipped = FnvHashSet::default();

    // let diamond_id = world.block_id("minecraft:diamond_block");
    // let air_id = world.block_id("minecraft:air");
    // let diamond_block = BlockData {
    //     layer1: BlockInfo { block_id: diamond_id, block_val: 0 },
    //     layer2: BlockInfo { block_id: air_id, block_val: 0 },
    // };

    parents.insert(start_pos, start_pos);
    queue.push_back(start_pos);

    while let Some(source) = queue.pop_front() {
        let neighbors = NeighborIterator::new(source);

        for neighbor in neighbors {
            // only process those neighbor which we haven't already
            // seen, nodes which we have already seen will have a
            // parent node
            if let Entry::Vacant(o) = parents.entry(neighbor) {
                if let Some(inside) = is_inside(world, neighbor)? {
                    if inside {
                        o.insert(source);
                        queue.push_back(neighbor);
                    }
                } else {
                    clipped.insert(neighbor);
                }
            }
        }
    }

    Ok((clipped, parents))
}

fn get_path(parents: &ParentMap, mut current_pos: WorldPos) -> Vec<WorldPos> {
    let mut path = Vec::new();
    path.push(current_pos);
    while let Some(next_pos) = parents.get(&current_pos) {
        if &current_pos == next_pos {
            return path;
        }

        path.push(*next_pos);
        current_pos = *next_pos;
    }
    path
}

fn main() {
    let path = Path::new("/home/daniel/mcpe/Ns4HXdObBQA=/db");
    let world = World::open(&path).unwrap();

    let start_pos = WorldPos {
        x: -35,
        y: 13,
        z: -19,
        dimension: Dimension::Overworld,
    };

    let _parents = bfs(&world, start_pos).unwrap();

    world.save().unwrap();
}
