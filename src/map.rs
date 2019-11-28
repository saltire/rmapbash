use std::error::Error;
use std::time::Instant;

use super::blocktypes;
use super::image;
use super::isomap;
use super::options::{Options, View};
use super::orthomap;
use super::region;
use super::sizes::*;
use super::types::*;
use super::world;

pub fn create_map(options: &Options) -> Result<(), Box<Error>> {
    println!("View:              {}", options.view);
    println!("Lighting:          {}", options.lighting);
    println!("Horizontal limits: {}", match options.blimits {
        Some(blimits) => format!("({}, {}) - ({}, {})", blimits.w, blimits.n, blimits.e, blimits.s),
        _ => "none".to_string(),
    });
    println!("Vertical limits:   {} - {}", options.ylimits.start, options.ylimits.end - 1);

    let start = Instant::now();

    std::fs::create_dir_all(options.outpath.parent().unwrap())?;

    println!("Getting world info from world dir {}", options.inpath.display());
    let world = world::get_world(options.inpath, &options.blimits, &options.ylimits)?;

    println!("Getting block types");
    let blocktypes = blocktypes::get_block_types(&options.lighting);

    println!("Starting block map");
    let result = draw_map(&world, &blocktypes, options);

    let elapsed = start.elapsed();
    let mins = elapsed.as_secs() / 60;
    let secs = elapsed.as_secs() % 60;
    let ms = elapsed.subsec_millis();
    println!("Time elapsed: {}:{:02}.{:03}", mins, secs, ms);

    result
}

pub fn draw_map(world: &world::World, blocktypes: &[blocktypes::BlockType], options: &Options)
-> Result<(), Box<Error>> {
    let size = match options.view {
        View::Isometric => isomap::get_size(world),
        View::Orthographic => orthomap::get_size(world),
    };
    let crop = match options.view {
        View::Isometric => isomap::get_crop(world, &size),
        View::Orthographic => orthomap::get_crop(world, &size),
    };
    let mut pixels = vec![0u8; size.x * size.z * 4];

    let water_blocktype = blocktypes.iter().find(|b| b.name == "minecraft:water").unwrap();

    let mut i = 0;
    let len = world.regions.len();

    for rz in (world.redges.n..world.redges.s + 1).rev() {
        for rx in (world.redges.w..world.redges.e + 1).rev() {
            let r = &Pair { x: rx, z: rz };
            if !world.regions.contains_key(&r) {
                continue;
            }

            i += 1;
            println!("Reading block data for region {}, {} ({}/{})", r.x, r.z, i, len);
            if let Some(reg) = region::read_region_data(&world, r, blocktypes)? {
                let chunk_count = reg.chunks.len();
                println!("Drawing block map for region {}, {} ({} chunk{})", r.x, r.z,
                    chunk_count, if chunk_count == 1 { "" } else { "s" });

                let arc = &Pair {
                    x: r.x * CHUNKS_IN_REGION as isize - world.cedges.w,
                    z: r.z * CHUNKS_IN_REGION as isize - world.cedges.n,
                };

                for cz in (0..CHUNKS_IN_REGION).rev() {
                    for cx in (0..CHUNKS_IN_REGION).rev() {
                        let c = &Pair { x: cx, z: cz };
                        if let Some(chunk) = reg.get_chunk(c) {
                            // println!("Drawing chunk {}, {}", c.x, c.z);
                            let wc = Pair {
                                x: r.x * CHUNKS_IN_REGION as isize + c.x as isize,
                                z: r.z * CHUNKS_IN_REGION as isize + c.z as isize,
                            };
                            let cblimits = Edges {
                                n: block_pos_in_chunk(world.bedges.n, Some(wc.z)),
                                e: block_pos_in_chunk(world.bedges.e, Some(wc.x)),
                                s: block_pos_in_chunk(world.bedges.s, Some(wc.z)),
                                w: block_pos_in_chunk(world.bedges.w, Some(wc.x)),
                            };

                            let cp = match options.view {
                                View::Isometric => isomap::get_chunk_pixel(world, arc, c),
                                View::Orthographic => orthomap::get_chunk_pixel(arc, c),
                            };
                            let co = (cp.z * size.x + cp.x) as isize - crop as isize;

                            match options.view {
                                View::Isometric => isomap::draw_chunk(
                                    &mut pixels, blocktypes, water_blocktype, &chunk, &co, &size.x,
                                    &cblimits, world.ylimits),
                                View::Orthographic => orthomap::draw_chunk(
                                    &mut pixels, blocktypes, &chunk, &co, &size.x,
                                    &cblimits, world.ylimits),
                            };
                        }
                    }
                }
            } else {
                println!("No data in region.");
            }
        }
    }

    image::draw_block_map(&pixels, size, options.outpath, true)?;

    Ok(())
}
