use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::blocktypes::BlockType;
use super::color;
use super::image;
// use super::region;
use super::region2;
use super::sizes::*;
use super::types::*;
use super::world;

// struct Chunk<'a> {
//     blocks: &'a [u16; BLOCKS_IN_CHUNK_3D],
//     // nblocks: Edges<&'a [u16; BLOCKS_IN_CHUNK_3D]>,
//     lights: &'a [u8; BLOCKS_IN_CHUNK_3D],
//     // nlights: Edges<&'a [u8; BLOCKS_IN_CHUNK_3D]>,
//     biomes: &'a [u8; BLOCKS_IN_CHUNK_2D],
// }

fn draw_chunk(pixels: &mut [u8], blocktypes: &[BlockType], chunk: &region2::Chunk,
    co: &usize, width: &usize) {
    for bz in 0..BLOCKS_IN_CHUNK {
        for bx in 0..BLOCKS_IN_CHUNK {
            let bo2 = bz * BLOCKS_IN_CHUNK + bx;
            let mut color = color::RGBA { r: 0, g: 0, b: 0, a: 0 };

            let biome = chunk.data.biomes[bo2] as usize;

            for by in (0..BLOCKS_IN_CHUNK_Y).rev() {
                let bo3 = by * BLOCKS_IN_CHUNK_2D + bo2;
                let blocktype = &blocktypes[chunk.data.blocks[bo3] as usize];
                if blocktype.empty {
                    continue;
                }

                let tlight = match by {
                    MAX_BLOCK_IN_CHUNK_Y => MAX_LIGHT_LEVEL,
                    _ => chunk.data.lights[bo3 + BLOCKS_IN_CHUNK_2D],
                };
                let tslight = (tlight & 0x0f) as usize;
                let tblight = ((tlight & 0xf0) >> 4) as usize;
                let blockcolor = &blocktype.colors[biome][tslight][tblight][1];

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

    for r in world.regions.iter() {
        i += 1;
        println!("Reading block data for region {}, {} ({}/{})", r.x, r.z, i, len);
        if let Some(reg) = region2::read_region_data(worldpath, &r, &blocknames)? {
            println!("Drawing block map for region {}, {}", r.x, r.z);
            let arx = (r.x - world.rlimits.w) as usize;
            let arz = (r.z - world.rlimits.n) as usize;

            for c in reg.chunks.keys() {
                // println!("Drawing chunk {}, {}", c.x, c.z);
                let acx = arx * CHUNKS_IN_REGION + c.x - world.margins.w;
                let acz = arz * CHUNKS_IN_REGION + c.z - world.margins.n;
                let co = (acz * size.x + acx) * BLOCKS_IN_CHUNK;

                draw_chunk(&mut pixels, &blocktypes, &reg.get_chunk(c), &co, &size.x);
            }
        } else {
            println!("No data in region.");
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
    if let Some(reg) = region2::read_region_data(worldpath, &r, &blocknames)? {
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
