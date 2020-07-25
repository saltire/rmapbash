use std::error::Error;
use std::time::Instant;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use super::blocktypes;
use super::image;
use super::isomap;
use super::options::{Options, View};
use super::orthomap;
use super::region;
use super::sizes::*;
use super::types::*;
use super::world;

pub fn create_map(options: &Options) -> Result<(), Box<dyn Error>> {
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

    println!("Drawing block map");
    let result = draw_map(&world, &blocktypes, options);

    let elapsed = start.elapsed();
    let mins = elapsed.as_secs() / 60;
    let secs = elapsed.as_secs() % 60;
    let ms = elapsed.subsec_millis();
    println!("Time elapsed: {}:{:02}.{:03}", mins, secs, ms);

    result
}

pub fn draw_map(world: &world::World, blocktypes: &[blocktypes::BlockType], options: &Options)
-> Result<(), Box<dyn Error>> {
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

    let bar = ProgressBar::with_draw_target(world.regions.len() as u64,
        ProgressDrawTarget::stdout_nohz())
        .with_style(ProgressStyle::default_bar()
            .template("{wide_bar}\n{msg} ({pos}/{len})")
            .progress_chars("▪■ "));

    for rz in (world.redges.n..world.redges.s + 1).rev() {
        for rx in (world.redges.w..world.redges.e + 1).rev() {
            let r = &Pair { x: rx, z: rz };
            if !world.regions.contains_key(&r) {
                continue;
            }

            let msg = format!("Reading block data for region {}, {}", r.x, r.z);
            bar.set_message(&msg);
            bar.inc(1);

            if let Some(reg) = region::read_region_data(&world, r, blocktypes)? {
                let chunk_count = reg.chunks.len();
                let msg = format!("Drawing block map for region {}, {} ({} chunk{})", r.x, r.z,
                    chunk_count, if chunk_count == 1 { "" } else { "s" });
                bar.set_message(&msg);

                let arc = &Pair {
                    x: r.x * CHUNKS_IN_REGION as isize - world.cedges.w,
                    z: r.z * CHUNKS_IN_REGION as isize - world.cedges.n,
                };

                let cbar = ProgressBar::with_draw_target(CHUNKS_IN_REGION_2D as u64,
                    ProgressDrawTarget::stdout())
                    .with_style(ProgressStyle::default_bar().template("{wide_bar}")
                        .progress_chars("▪■ "));

                for cz in (0..CHUNKS_IN_REGION).rev() {
                    for cx in (0..CHUNKS_IN_REGION).rev() {
                        cbar.inc(1);

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

                cbar.finish_and_clear();
            }
        }
    }

    image::draw_block_map(&pixels, size, options.outpath, true)?;

    bar.finish_and_clear();

    Ok(())
}
