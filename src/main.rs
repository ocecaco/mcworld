#![allow(dead_code)]
#![allow(unused_imports)]
#![warn(clippy::all)]
use crate::rawworld::*;
use crate::table::{BlockId, NOT_PRESENT, AIR};
use crate::world::*;
use crate::pos::*;
use crate::neighbor::*;
use crate::error::*;
use std::path::Path;
use fnv::FnvHashMap;
use std::collections::VecDeque;
use std::collections::hash_map::Entry;
use std::io::{self, Write};

mod encode;
mod rawchunk;
mod rawworld;
mod table;
mod world;
mod pos;
mod neighbor;

mod error {
    pub use failure::Error;
    pub type Result<T> = ::std::result::Result<T, Error>;
}

type ParentMap = FnvHashMap<WorldPos, WorldPos>;

fn is_inside(world: &World, mut pos: WorldPos) -> Result<bool> {
    let (info1, _info2) = world.get_block(&pos)?;
    if info1.block_id != AIR {
        return Ok(false);
    }

    loop {
        let (info1, _info2) = world.get_block(&pos)?;
        if info1.block_id != AIR || pos.y == 255 {
            break;
        }
        pos.y += 1;
    }

    Ok(pos.y != 255)
}

fn bfs(world: &World, start_pos: WorldPos) -> Result<(Vec<WorldPos>, ParentMap)> {
    let mut deep = Vec::new();
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
                if is_inside(world, neighbor)? {
                    o.insert(source);
                    queue.push_back(neighbor);

                    deep.push(neighbor);
                }
            }
        }
    }

    Ok((deep, parents))
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

    // let blk = world.get_block(&WorldPos {
    //     x: -80,
    //     y: 12,
    //     z: -1,
    //     dimension: Dimension::Overworld,
    // }).unwrap().0;

    // println!("{:?}", world.global_palette.borrow_mut().get_description(blk));

    // let mut block_counts: FnvHashMap<BlockId, u32> = FnvHashMap::default();

    // let chunk_positions = world.iter_chunks();
    // for pos in chunk_positions {
    //     let pos = pos.unwrap();

    //     if pos.dimension != Dimension::Overworld {
    //         continue;
    //     }

    //     for dy in 0..=255 {
    //         for dz in 0..16 {
    //             for dx in 0..16 {
    //                 let world_pos = WorldPos {
    //                     x: 16 * pos.x + dx,
    //                     y: dy,
    //                     z: 16 * pos.z + dz,
    //                     dimension: Dimension::Overworld,
    //                 };

    //                 let (info1, _info2) = world.get_block(&world_pos).unwrap();
    //                 let id = info1.block_id;
    //                 let new_count = block_counts.get(&id).map(|id| *id).unwrap_or(0) + 1;
    //                 block_counts.insert(id, new_count);
    //             }
    //         }
    //     }
    // }

    // for (id, count) in block_counts {
    //     println!("{} {}", count, world.block_name(id));
    // }
    // let start_pos = WorldPos {
    //     x: -80,
    //     y: 80,
    //     z: -1,
    //     dimension: Dimension::Overworld,
    // };
    let start_pos = WorldPos {
        x: -34,
        y: 16,
        z: -19,
        dimension: Dimension::Overworld,
    };

    let (deep, _parents) = bfs(&world, start_pos).unwrap();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for p in deep {
        writeln!(handle, "{} {} {}", p.x, p.y, p.z).unwrap();
    }
}
