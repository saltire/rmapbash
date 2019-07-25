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


fn get_ortho_size(csize: &Pair<usize>) -> Pair<usize> {
    Pair {
        x: csize.x * BLOCKS_IN_CHUNK,
        z: csize.z * BLOCKS_IN_CHUNK,
    }
}

fn get_block_color(bx: usize, bz: usize, blocktypes: &[BlockType], chunk: &region::Chunk)
-> color::RGBA {
    let mut color = color::RGBA { r: 0, g: 0, b: 0, a: 0 };

    let bo2 = bz * BLOCKS_IN_CHUNK + bx;
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
            n: nblocks.n.slight > 0 && !blocktypes[nblocks.n.btype as usize].solid,
            s: nblocks.s.slight > 0 && !blocktypes[nblocks.s.btype as usize].solid,
            e: nblocks.e.slight > 0 && !blocktypes[nblocks.e.btype as usize].solid,
            w: nblocks.w.slight > 0 && !blocktypes[nblocks.w.btype as usize].solid,
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

    color
}

fn draw_chunk(pixels: &mut [u8], blocktypes: &[BlockType], chunk: &region::Chunk, co: &usize,
    width: &usize) {
    for bz in 0..BLOCKS_IN_CHUNK {
        for bx in 0..BLOCKS_IN_CHUNK {
            let po = (co + bz * width + bx) * 4;
            let color = get_block_color(bx, bz, blocktypes, chunk);
            pixels[po] = color.r;
            pixels[po + 1] = color.g;
            pixels[po + 2] = color.b;
            pixels[po + 3] = color.a;
        }
    }
}

pub fn draw_world_ortho_map(worldpath: &Path, outpath: &Path, blocktypes: &[BlockType],
    limits: &Option<Edges<i32>>)
-> Result<(), Box<Error>> {
    println!("Creating block map from world dir {}", worldpath.display());

    let world = world::get_world(worldpath, limits)?;
    let size = get_ortho_size(&world.csize);
    let mut pixels = vec![0u8; size.x * size.z * 4];

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
            if let Some(reg) = region::read_region_data(worldpath, r, blocktypes)? {
                println!("Drawing block map for region {}, {}", r.x, r.z);
                let ar = Pair {
                    x: (r.x - world.redges.w) as usize,
                    z: (r.z - world.redges.n) as usize,
                };

                for cz in (0..CHUNKS_IN_REGION).rev() {
                    for cx in (0..CHUNKS_IN_REGION).rev() {
                        let c = &Pair { x: cx, z: cz };
                        if !reg.chunks.contains_key(c) {
                            continue;
                        }

                        // println!("Drawing chunk {}, {}", c.x, c.z);
                        let ac = Pair {
                            x: ar.x * CHUNKS_IN_REGION + c.x - world.cmargins.w,
                            z: ar.z * CHUNKS_IN_REGION + c.z - world.cmargins.n,
                        };
                        let co = (ac.z * size.x + ac.x) * BLOCKS_IN_CHUNK;

                        draw_chunk(&mut pixels, blocktypes, &reg.get_chunk(c), &co, &size.x);
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

pub fn draw_region_ortho_map(worldpath: &Path, r: &Pair<i32>, outpath: &Path,
    blocktypes: &[BlockType], limits: &Option<Edges<i32>>)
-> Result<(), Box<Error>> {
    println!("Reading block data for region {}, {}", r.x, r.z);
    if let Some(reg) = region::read_region_data(worldpath, r, blocktypes)? {
        if reg.chunks.keys().len() > 0 {
            println!("Drawing block map");

            let climits = Edges {
                n: reg.chunks.keys().map(|c| c.z).min().unwrap(),
                e: reg.chunks.keys().map(|c| c.x).max().unwrap(),
                s: reg.chunks.keys().map(|c| c.z).max().unwrap(),
                w: reg.chunks.keys().map(|c| c.x).min().unwrap(),
            };
            let csize = Pair {
                x: climits.e - climits.w + 1,
                z: climits.s - climits.n + 1,
            };
            let size = get_ortho_size(&csize);
            let mut pixels = vec![0u8; size.x * size.z * 4];

            for cz in (0..CHUNKS_IN_REGION).rev() {
                for cx in (0..CHUNKS_IN_REGION).rev() {
                    let c = &Pair { x: cx, z: cz };
                    if !reg.chunks.contains_key(c) {
                        continue;
                    }

                    // println!("Drawing chunk {}, {}", c.x, c.z);
                    let ac = Pair {
                        x: c.x - climits.w,
                        z: c.z - climits.n,
                    };
                    let co = (ac.z * size.x + ac.x) * BLOCKS_IN_CHUNK;

                    draw_chunk(&mut pixels, blocktypes, &reg.get_chunk(c), &co, &size.x);
                }
            }

            let file = File::create(outpath)?;
            image::draw_block_map(&pixels, size, file, true)?;

            return Ok(());
        }
    }

    println!("No data in region.");
    Ok(())
}
