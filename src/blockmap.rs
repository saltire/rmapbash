use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::blocktypes::BlockType;
use super::color;
use super::image;
use super::region;
use super::sizes::*;
use super::types::*;
use super::world;

fn draw_chunk(pixels: &mut [u8], blocktypes: &[BlockType], chunk: &region::Chunk,
    co: &usize, width: &usize) {
    for bz in 0..BLOCKS_IN_CHUNK {
        for bx in 0..BLOCKS_IN_CHUNK {
            let bo2 = bz * BLOCKS_IN_CHUNK + bx;
            let mut color = color::RGBA { r: 0, g: 0, b: 0, a: 0 };

            let biome = chunk.data.biomes[bo2] as usize;

            for by in (0..BLOCKS_IN_CHUNK_Y).rev() {
                let bo3 = by * BLOCKS_IN_CHUNK_2D + bo2;
                let btype = chunk.data.blocks[bo3];
                let blocktype = &blocktypes[btype as usize];
                if blocktype.empty {
                    continue;
                }

                let tblock = chunk.get_t_block(&by, &bo3);
                let nblocks = Edges {
                    n: chunk.get_n_block(&bz, &bo3),
                    e: chunk.get_e_block(&bx, &bo3),
                    s: chunk.get_s_block(&bz, &bo3),
                    w: chunk.get_w_block(&bx, &bo3),
                };
                let is_edge = Edges {
                    n: nblocks.n.slight > 0 && nblocks.n.btype != btype,
                    s: nblocks.s.slight > 0 && nblocks.s.btype != btype,
                    e: nblocks.e.slight > 0 && nblocks.e.btype != btype,
                    w: nblocks.w.slight > 0 && nblocks.w.btype != btype,
                };
                let shade = match (is_edge.n || is_edge.w, is_edge.e || is_edge.s) {
                    (true, false) => 2,
                    (false, true) => 3,
                    _ => 1,
                };
                let blockcolor = &blocktype.colors[biome][tblock.slight][tblock.blight][shade];

                color = color::blend_alpha_color(&color, blockcolor);
                if color.a == MAX_CHANNEL_VALUE {
                    break;
                }
            }

            let po = (co + bz * width + bx) * 4;
            pixels[po] = color.r;
            pixels[po + 1] = color.g;
            pixels[po + 2] = color.b;
            pixels[po + 3] = color.a;
        }
    }
}

#[allow(dead_code)]
pub fn draw_world_block_map(worldpath: &Path, outpath: &Path, blocktypes: &[BlockType])
-> Result<(), Box<Error>> {
    println!("Creating block map from world dir {}", worldpath.display());

    let world = world::get_world(worldpath)?;

    let size = world.get_ortho_size();
    let mut pixels = vec![0u8; size.x * size.z * 4];

    let blocknames: Vec<&str> = blocktypes.iter().map(|b| &b.name[..]).collect();

    let mut i = 0;
    let len = world.regions.len();

    for rz in (world.rlimits.n..world.rlimits.s + 1).rev() {
        for rx in (world.rlimits.w..world.rlimits.e + 1).rev() {
            let r = Pair { x: rx, z: rz };
            if !world.regions.contains_key(&r) {
                continue;
            }

            i += 1;
            println!("Reading block data for region {}, {} ({}/{})", r.x, r.z, i, len);
            if let Some(reg) = region::read_region_data(worldpath, &r, &blocknames)? {
                println!("Drawing block map for region {}, {}", r.x, r.z);
                let arx = (r.x - world.rlimits.w) as usize;
                let arz = (r.z - world.rlimits.n) as usize;

                for cz in (0..CHUNKS_IN_REGION).rev() {
                    for cx in (0..CHUNKS_IN_REGION).rev() {
                        let c = &Pair { x: cx, z: cz };
                        if !reg.chunks.contains_key(c) {
                            continue;
                        }

                        // println!("Drawing chunk {}, {}", c.x, c.z);
                        let acx = arx * CHUNKS_IN_REGION + c.x - world.margins.w;
                        let acz = arz * CHUNKS_IN_REGION + c.z - world.margins.n;
                        let co = (acz * size.x + acx) * BLOCKS_IN_CHUNK;

                        draw_chunk(&mut pixels, &blocktypes, &reg.get_chunk(c), &co, &size.x);
                    }
                }
            } else {
                println!("No data in region.");
            }
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, true)?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_region_block_map(worldpath: &Path, r: &Pair<i32>, outpath: &Path, blocktypes: &[BlockType])
-> Result<(), Box<Error>> {
    println!("Reading block data for region {}, {}", r.x, r.z);
    let blocknames: Vec<&str> = blocktypes.iter().map(|b| &b.name[..]).collect();
    if let Some(reg) = region::read_region_data(worldpath, &r, &blocknames)? {
        if reg.chunks.keys().len() > 0 {
            println!("Drawing block map");

            let climits = Edges {
                n: reg.chunks.keys().map(|c| c.z).min().unwrap(),
                e: reg.chunks.keys().map(|c| c.x).max().unwrap(),
                s: reg.chunks.keys().map(|c| c.z).max().unwrap(),
                w: reg.chunks.keys().map(|c| c.x).min().unwrap(),
            };
            let size = Pair {
                x: (climits.e - climits.w + 1) * BLOCKS_IN_CHUNK,
                z: (climits.s - climits.n + 1) * BLOCKS_IN_CHUNK,
            };

            let mut pixels = vec![0u8; size.x * size.z * 4];

            for c in reg.chunks.keys() {
                // println!("Drawing chunk {}, {}", c.x, c.z);
                let acx = c.x - climits.w;
                let acz = c.z - climits.n;
                let co = (acz * size.x + acx) * BLOCKS_IN_CHUNK;

                draw_chunk(&mut pixels, &blocktypes, &reg.get_chunk(c), &co, &size.x);
            }

            let file = File::create(outpath)?;
            image::draw_block_map(&pixels, size, file, true)?;

            return Ok(());
        }
    }

    println!("No data in region.");
    Ok(())
}
