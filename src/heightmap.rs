use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::sizes::*;
use super::image;
use super::region;
use super::types::{Edges, Pair};
use super::world;

fn draw_chunk(pixels: &mut [u8], cpixels: &[u8], co: &usize, width: &usize) {
    for bz in 0..BLOCKS_IN_CHUNK {
        for bx in 0..BLOCKS_IN_CHUNK {
            pixels[(co + bz * width + bx) as usize] = cpixels[(bz * BLOCKS_IN_CHUNK + bx) as usize];
        }
    }
}

#[allow(dead_code)]
pub fn draw_world_heightmap(worldpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating heightmap from world dir {}", worldpath.display());

    let world = world::get_world(worldpath)?;

    let size = world.get_ortho_size();
    let mut pixels = vec![0u8; size.x * size.z];

    for r in world.regions.iter() {
        println!("Reading heightmap for region {}, {}", r.x, r.z);
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", r.x, r.z));
        let rheightmaps = region::read_region_chunk_heightmaps(regionpath.as_path())?;

        let arx = (r.x - world.rlimits.n) as usize;
        let arz = (r.z - world.rlimits.w) as usize;

        for (c, cpixels) in rheightmaps.iter() {
            let acx = arx * CHUNKS_IN_REGION + c.x as usize;
            let acz = arz * CHUNKS_IN_REGION + c.z as usize;
            let co = ((acz - world.margins.n) * size.x + (acx - world.margins.w)) * BLOCKS_IN_CHUNK;

            draw_chunk(&mut pixels, cpixels, &co, &size.x);
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, false)?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_region_heightmap(regionpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating heightmap from region file {}", regionpath.display());

    let heightmaps = region::read_region_chunk_heightmaps(regionpath)?;
    if heightmaps.keys().len() == 0 {
        println!("No chunks in region.");
        return Ok(());
    }

    let climits = Edges {
        n: heightmaps.keys().map(|c| c.z).min().unwrap(),
        e: heightmaps.keys().map(|c| c.x).max().unwrap(),
        s: heightmaps.keys().map(|c| c.z).max().unwrap(),
        w: heightmaps.keys().map(|c| c.x).min().unwrap(),
    };
    let size = Pair {
        x: (climits.e - climits.w + 1) as usize * BLOCKS_IN_CHUNK,
        z: (climits.s - climits.n + 1) as usize * BLOCKS_IN_CHUNK,
    };

    let mut pixels = vec![0u8; size.x * size.z];
    for (c, cpixels) in heightmaps.iter() {
        let acx = (c.x - climits.w) as usize;
        let acz = (c.z - climits.n) as usize;
        let co = (acz * size.x + acx) * BLOCKS_IN_CHUNK;

        draw_chunk(&mut pixels, cpixels, &co, &size.x);
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, false)?;

    Ok(())
}
