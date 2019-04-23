use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::sizes::*;
use super::image;
use super::region;
use super::types::Pair;
use super::world;

#[allow(dead_code)]
pub fn draw_world_region_map(worldpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating map of regions from world dir {}", worldpath.display());

    let regions = world::read_world_regions(worldpath)?;
    let min_rx = regions.iter().map(|c| c.x).min().unwrap();
    let max_rx = regions.iter().map(|c| c.x).max().unwrap();
    let min_rz = regions.iter().map(|c| c.z).min().unwrap();
    let max_rz = regions.iter().map(|c| c.z).max().unwrap();
    let size = Pair {
        x: (max_rx - min_rx + 1) as usize,
        z: (max_rz - min_rz + 1) as usize,
    };

    let mut pixels: Vec<bool> = vec![false; size.x * size.z];
    for r in regions.iter() {
        let arx = (r.x - min_rx) as usize;
        let arz = (r.z - min_rz) as usize;
        pixels[arz * size.x + arx] = true;
    }

    let file = File::create(outpath)?;
    image::draw_tiny_map(pixels.as_slice(), size, file)?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_world_chunk_map(worldpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating map of chunks from world dir {}", worldpath.display());

    let regions = world::read_world_regions(worldpath)?;
    let min_rx = regions.iter().map(|c| c.x).min().unwrap();
    let max_rx = regions.iter().map(|c| c.x).max().unwrap();
    let min_rz = regions.iter().map(|c| c.z).min().unwrap();
    let max_rz = regions.iter().map(|c| c.z).max().unwrap();
    let size = Pair {
        x: (max_rx - min_rx + 1) as usize * CHUNKS_IN_REGION,
        z: (max_rz - min_rz + 1) as usize * CHUNKS_IN_REGION,
    };

    let mut pixels: Vec<bool> = vec![false; size.x * size.z];
    for r in regions.iter() {
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", r.x, r.z));
        let regionpixels = region::read_region_chunks(&regionpath)?;

        let arx = (r.x - min_rx) as usize;
        let arz = (r.z - min_rz) as usize;
        let ro = arz * size.x * CHUNKS_IN_REGION + arx * CHUNKS_IN_REGION;

        for cz in 0..CHUNKS_IN_REGION {
            for cx in 0..CHUNKS_IN_REGION {
                pixels[ro + cz * size.x + cx] = regionpixels[cz * CHUNKS_IN_REGION + cx];
            }
        }
    }

    let file = File::create(outpath)?;
    image::draw_tiny_map(pixels.as_slice(), size, file)?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_region_chunk_map(regionpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating map of chunks from region file {}", regionpath.display());

    let pixels = region::read_region_chunks(regionpath)?;

    let file = File::create(outpath)?;
    image::draw_tiny_map(&pixels, Pair { x: CHUNKS_IN_REGION, z: CHUNKS_IN_REGION }, file)?;

    Ok(())
}
