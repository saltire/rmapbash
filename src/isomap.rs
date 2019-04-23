use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::blocktypes;
use super::image;
use super::region;
use super::sizes::*;
use super::types::{Edges, Pair};
use super::world;

fn get_iso_size(csize: &Pair<usize>) -> Pair<usize> {
    Pair {
        x: (csize.x + csize.z) * ISO_CHUNK_X_MARGIN,
        z: (csize.x + csize.z) * ISO_CHUNK_Y_MARGIN + ISO_CHUNK_SIDE_HEIGHT,
    }
}

fn draw_chunk(pixels: &mut [u8], blocktypes: &Vec<blocktypes::BlockType>,
    cblocks: &[u16], clights: &[u8], cbiomes: &[u8], co: &usize, width: &usize, night: &bool) {
    for bz in 0..BLOCKS_IN_CHUNK {
        for bx in 0..BLOCKS_IN_CHUNK {
            let bo2 = bz * BLOCKS_IN_CHUNK + bx;

            let bpx = (ISO_CHUNK_X_MARGIN as i16 +
                (bx as i16 - bz as i16 - 1) * ISO_BLOCK_X_MARGIN as i16) as usize;
            let bpy2 = (bx + bz) * ISO_BLOCK_Y_MARGIN;

            for by in 0..BLOCKS_IN_CHUNK_Y {
                let bo3 = by * BLOCKS_IN_CHUNK_Y + bo2;
                if cblocks[bo3] == 0 {
                    continue;
                }

                let blocktype = &blocktypes[cblocks[bo3] as usize];

                let light = if *night && by < BLOCKS_IN_CHUNK_Y - 1 {
                    clights[bo3 + BLOCKS_IN_CHUNK_Y]
                } else {
                    LIGHT_LEVELS as u8 - 1
                };

                let blockcolor = &blocktype.colors[
                    cbiomes[bo2] as usize * LIGHT_LEVELS + light as usize];
                if blockcolor.a == 0 {
                    continue;
                }

                let bpy = bpy2 + (BLOCKS_IN_CHUNK_Y - by - 1) * ISO_BLOCK_SIDE_HEIGHT;

                for y in 0..ISO_BLOCK_HEIGHT {
                    for x in 0..ISO_BLOCK_WIDTH {
                        let po = (co + (bpy + y) * width + bpx + x) * 4;
                        pixels[po] = blockcolor.r;
                        pixels[po + 1] = blockcolor.g;
                        pixels[po + 2] = blockcolor.b;
                        pixels[po + 3] = blockcolor.a;
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn draw_world_iso_map(worldpath: &Path, outpath: &Path, night: bool)
-> Result<(), Box<Error>> {
    println!("Creating block map from world dir {}", worldpath.display());

    let world = world::get_world(worldpath)?;

    let csize = world.get_chunk_size();
    let size = get_iso_size(&csize);
    let mut pixels = vec![0u8; size.x * size.z * 4];

    let blocktypes = blocktypes::get_block_types();
    let blocknames: Vec<&str> = blocktypes.iter().map(|b| &b.name[..]).collect();

    for rz in world.rlimits.n..world.rlimits.s + 1 {
        for rx in world.rlimits.w..world.rlimits.e + 1 {
            let r = Pair { x: rx, z: rz };
            if !world.regions.contains(&r) {
                continue;
            }

            let regionpath_str = worldpath.join("region").join(format!("r.{}.{}.mca", r.x, r.z));
            let regionpath = regionpath_str.as_path();

            println!("Reading blocks for region {}, {}", r.x, r.z);
            let rblocks = region::read_region_chunk_blocks(regionpath, &blocknames)?;
            let rlights = region::read_region_chunk_lightmaps(regionpath)?;
            let rbiomes = region::read_region_chunk_biomes(regionpath)?;

            println!("Drawing block map for region {}, {}", r.x, r.z);
            let arx = (r.x - world.rlimits.w) as usize;
            let arz = (r.z - world.rlimits.n) as usize;

            for cz in 0..CHUNKS_IN_REGION as u8 {
                for cx in 0..CHUNKS_IN_REGION as u8 {
                    let c = &Pair { x: cx, z: cz };
                    if !rblocks.contains_key(c) {
                        continue;
                    }

                    // println!("Drawing chunk {}, {}", c.x, c.z);
                    let acx = arx * CHUNKS_IN_REGION + c.x as usize - world.margins.w;
                    let acz = arz * CHUNKS_IN_REGION + c.z as usize - world.margins.n;

                    let cpx = (size.x as i16 / 2 + (acx as i16 - acz as i16 - 1)
                        * ISO_CHUNK_X_MARGIN as i16) as usize;
                    let cpy = (acx + acz) * ISO_CHUNK_Y_MARGIN;
                    let co = cpy * size.x + cpx;

                    draw_chunk(&mut pixels, &blocktypes, &rblocks[c], &rlights[c], &rbiomes[c], &co,
                        &size.x, &night);
                }
            }
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, true)?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_region_iso_map(regionpath: &Path, outpath: &Path, night: bool)
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
    let csize = Pair {
        x: (climits.e - climits.w + 1) as usize,
        z: (climits.s - climits.n + 1) as usize,
    };
    let size = get_iso_size(&csize);

    let mut pixels = vec![0u8; size.x * size.z * 4];

    for cz in 0..CHUNKS_IN_REGION as u8 {
        for cx in 0..CHUNKS_IN_REGION as u8 {
            let c = &Pair { x: cx, z: cz };
            if !rblocks.contains_key(c) {
                continue;
            }

            // println!("Drawing chunk {}, {}", c.x, c.z);
            let acx = (c.x - climits.w) as usize;
            let acz = (c.z - climits.n) as usize;

            let cpx = (size.x as i16 / 2 + (acx as i16 - acz as i16 - 1)
                * ISO_CHUNK_X_MARGIN as i16) as usize;
            let cpy = (acx + acz) * ISO_CHUNK_Y_MARGIN;
            let co = cpy * size.x + cpx;

            draw_chunk(&mut pixels, &blocktypes, &rblocks[c], &rlights[c], &rbiomes[c], &co,
                &size.x, &night);
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, true)?;

    Ok(())
}
