#![allow(dead_code)]
#![allow(unused_imports)]
#![warn(clippy::all)]
use crate::table::{BlockId, AIR};
use crate::world::{World, BlockData};
use crate::pos::{WorldPos, Dimension};
use crate::neighbor::NeighborIterator;
use crate::error::*;

use std::path::Path;
use fnv::FnvHashMap;
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

fn is_inside(world: &World, mut pos: WorldPos) -> Result<bool> {
    let data = world.get_block(&pos)?;

    // stop if the starting block is not air
    if let Some(blk) = data {
        if blk.layer1.block_id != AIR {
            return Ok(false);
        }
    } else {
        return Ok(false);
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
    Ok(pos.y != 255)
}

fn is_air(world: &World, pos: WorldPos) -> Result<bool> {
    if let Some(blk) = world.get_block(&pos)? {
        Ok(blk.layer1.block_id == AIR)
    } else {
        Ok(false)
    }
}

fn bfs(world: &World, start_pos: WorldPos) -> Result<ParentMap> {
    let mut parents = FnvHashMap::default();
    let mut queue = VecDeque::new();

    parents.insert(start_pos, start_pos);
    queue.push_back(start_pos);

    while let Some(source) = queue.pop_front() {
        let neighbors = NeighborIterator::new(source);

        for neighbor in neighbors {
            // only process those neighbor which we haven't already
            // seen, nodes which we have already seen will have a
            // parent node
            if let Entry::Vacant(o) = parents.entry(neighbor) {
                if is_air(world, neighbor)? {
                    o.insert(source);
                    queue.push_back(neighbor);
                }
            }
        }
    }

    Ok(parents)
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
        x: -34,
        y: 16,
        z: -19,
        dimension: Dimension::Overworld,
    };

    let parents = bfs(&world, start_pos).unwrap();
}
