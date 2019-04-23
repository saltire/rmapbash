use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::blocktypes;
use super::color;
use super::sizes::*;
use super::image;
use super::region;
use super::types::{Edges, Pair};
use super::world;

fn is_empty(block: u16) -> bool {
    block == 0 || block == 14 || block == 98 || block == 563
}

fn draw_chunk(pixels: &mut [u8], blocktypes: &Vec<blocktypes::BlockType>,
    cblocks: &[u16], clights: &[u8], cbiomes: &[u8], co: &usize, width: &usize, night: &bool) {
    for bz in 0..BLOCKS_IN_CHUNK {
        for bx in 0..BLOCKS_IN_CHUNK {
            let bo2 = bz * BLOCKS_IN_CHUNK + bx;
            let mut color = color::RGBA { r: 0, g: 0, b: 0, a: 0 };

            for by in (0..BLOCKS_IN_CHUNK_Y).rev() {
                let bo3 = by * BLOCKS_IN_CHUNK_Y + bo2;
                if !is_empty(cblocks[bo3]) {
                    let blocktype = &blocktypes[cblocks[bo3] as usize];
                    let blockcolor = if blocktype.has_biome_colors {
                        &blocktype.biome_colors[&cbiomes[bo2]]
                    } else {
                        &blocktype.color
                    };

                    if blockcolor.a == 0 {
                        continue;
                    }

                    color = if *night && by < BLOCKS_IN_CHUNK_Y - 1 {
                        color::blend_alpha_color(&color,
                            &color::set_light_level(&blockcolor, &clights[bo3 + BLOCKS_IN_CHUNK_Y]))
                    } else {
                        color::blend_alpha_color(&color, &blockcolor)
                    };

                    if color.a == 255 {
                        break;
                    }
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

pub fn draw_world_block_map(worldpath: &Path, outpath: &Path, night: bool)
-> Result<(), Box<Error>> {
    println!("Creating block map from world dir {}", worldpath.display());

    let world = world::get_world(worldpath)?;

    let size = world.get_ortho_size();
    let mut pixels = vec![0u8; size.x * size.z * 4];

    let blocktypes = blocktypes::get_block_types();
    let blocknames: Vec<&str> = blocktypes.iter().map(|b| &b.name[..]).collect();

    for r in world.regions.iter() {
        let regionpath_str = worldpath.join("region").join(format!("r.{}.{}.mca", r.x, r.z));
        let regionpath = regionpath_str.as_path();

        println!("Reading blocks for region {}, {}", r.x, r.z);
        let rblocks = region::read_region_chunk_blocks(regionpath, &blocknames)?;
        let rlights = region::read_region_chunk_lightmaps(regionpath)?;
        let rbiomes = region::read_region_chunk_biomes(regionpath)?;

        println!("Drawing block map for region {}, {}", r.x, r.z);
        let arx = (r.x - world.rlimits.w) as usize;
        let arz = (r.z - world.rlimits.n) as usize;

        for (c, cblocks) in rblocks.iter() {
            // println!("Drawing chunk {}, {}", c.x, c.z);
            let acx = arx * CHUNKS_IN_REGION + c.x as usize;
            let acz = arz * CHUNKS_IN_REGION + c.z as usize;
            let co = ((acz - world.margins.n) * size.x + (acx - world.margins.w)) * BLOCKS_IN_CHUNK;

            draw_chunk(&mut pixels, &blocktypes, cblocks, &rlights[c], &rbiomes[c], &co, &size.x,
                &night);
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, true)?;

    Ok(())
}

pub fn draw_region_block_map(regionpath: &Path, outpath: &Path, night: bool)
-> Result<(), Box<Error>> {
    println!("Creating block map from region file {}", regionpath.display());

    println!("Getting block types");
    let blocktypes = blocktypes::get_block_types();
    let blocknames: Vec<&str> = blocktypes.iter().map(|b| &b.name[..]).collect();

    println!("Reading blocks");
    let rblocks = region::read_region_chunk_blocks(regionpath, &blocknames)?;
    if rblocks.keys().len() == 0 {
        println!("No chunks in region.");
        return Ok(());
    }

    println!("Reading light maps");
    let rlights = region::read_region_chunk_lightmaps(regionpath)?;

    println!("Reading biomes");
    let rbiomes = region::read_region_chunk_biomes(regionpath)?;

    println!("Drawing block map");
    let climits = Edges {
        n: rblocks.keys().map(|c| c.z).min().unwrap(),
        e: rblocks.keys().map(|c| c.x).max().unwrap(),
        s: rblocks.keys().map(|c| c.z).max().unwrap(),
        w: rblocks.keys().map(|c| c.x).min().unwrap(),
    };
    let size = Pair {
        x: (climits.e - climits.w + 1) as usize * BLOCKS_IN_CHUNK,
        z: (climits.s - climits.n + 1) as usize * BLOCKS_IN_CHUNK,
    };

    let mut pixels = vec![0u8; size.x * size.z * 4];

    for (c, cblocks) in rblocks.iter() {
        // println!("Drawing chunk {}, {}", c.x, c.z);
        let acx = (c.x - climits.w) as usize;
        let acz = (c.z - climits.n) as usize;
        let co = (acz * size.x + acx) * BLOCKS_IN_CHUNK;

        draw_chunk(&mut pixels, &blocktypes, cblocks, &rlights[c], &rbiomes[c], &co, &size.x,
            &night);
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, true)?;

    Ok(())
}
