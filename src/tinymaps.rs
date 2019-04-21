use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::image;
use super::region;
use super::world;

#[allow(dead_code)]
pub fn draw_world_region_map(worldpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating map of regions from world dir {}", worldpath.display());

    let regions = world::read_world_regions(worldpath)?;
    let min_rx = regions.iter().map(|(x, _)| x).min().unwrap();
    let max_rx = regions.iter().map(|(x, _)| x).max().unwrap();
    let min_rz = regions.iter().map(|(_, z)| z).min().unwrap();
    let max_rz = regions.iter().map(|(_, z)| z).max().unwrap();
    let width = max_rx - min_rx + 1;
    let height = max_rz - min_rz + 1;

    let mut pixels: Vec<bool> = vec![false; (width * height) as usize];
    for (rx, rz) in regions.iter() {
        pixels[((rz - min_rz) * width + (rx - min_rx)) as usize] = true;
    }

    let file = File::create(outpath)?;
    image::draw_tiny_map(pixels.as_slice(), width as u32, height as u32, file)?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_world_chunk_map(worldpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating map of chunks from world dir {}", worldpath.display());

    let regions = world::read_world_regions(worldpath)?;
    let min_rx = regions.iter().map(|(x, _)| x).min().unwrap();
    let max_rx = regions.iter().map(|(x, _)| x).max().unwrap();
    let min_rz = regions.iter().map(|(_, z)| z).min().unwrap();
    let max_rz = regions.iter().map(|(_, z)| z).max().unwrap();
    let cwidth = (max_rx - min_rx + 1) * 32;
    let cheight = (max_rz - min_rz + 1) * 32;

    let mut pixels: Vec<bool> = vec![false; (cwidth * cheight) as usize];
    for (rx, rz) in regions.iter() {
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", rx, rz));
        let regionpixels = region::read_region_chunks(&regionpath)?;

        let ro = (rz - min_rz) * cwidth * 32 + (rx - min_rx) * 32;

        for cz in 0..32 {
            for cx in 0..32 {
                pixels[(ro + cz * cwidth + cx) as usize] = regionpixels[(cz * 32 + cx) as usize];
            }
        }
    }

    let file = File::create(outpath)?;
    image::draw_tiny_map(pixels.as_slice(), cwidth as u32, cheight as u32, file)?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_region_chunk_map(regionpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating map of chunks from region file {}", regionpath.display());

    let pixels = region::read_region_chunks(regionpath)?;

    let file = File::create(outpath)?;
    image::draw_tiny_map(&pixels, 32, 32, file)?;

    Ok(())
}
