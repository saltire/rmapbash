use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::data;
use super::image;

pub fn draw_world_heightmap(worldpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating heightmap from world dir {}", worldpath.display());

    let regions = data::read_world_regions(worldpath)?;
    let min_rx = regions.iter().map(|(x, _)| x).min().unwrap();
    let max_rx = regions.iter().map(|(x, _)| x).max().unwrap();
    let min_rz = regions.iter().map(|(_, z)| z).min().unwrap();
    let max_rz = regions.iter().map(|(_, z)| z).max().unwrap();
    let width = (max_rx - min_rx + 1) as u32 * 512;
    let height = (max_rz - min_rz + 1) as u32 * 512;

    let mut pixels: Vec<u8> = vec![0; (width * height) as usize];
    for (rx, rz) in regions.iter() {
        println!("Processing region {}, {}", rx, rz);

        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", rx, rz));
        let heightmaps = data::read_region_chunk_heightmaps(regionpath.as_path())?;
        let ro = (rz - min_rz) as u32 * width * 512 + (rx - min_rx) as u32 * 512;

        for ((cx, cz), cpixels) in heightmaps.iter() {
            let co = ro + *cz as u32 * width * 16 + *cx as u32 * 16;

            for bz in 0..16 {
                for bx in 0..16 {
                    pixels[(co + bz * width + bx) as usize] =
                        cpixels[(bz * 16 + bx) as usize] as u8;
                }
            }
        }
    }

    let file = File::create(outpath)?;
    image::draw_height_map(&pixels, width, height, file)?;

    Ok(())
}

pub fn draw_region_heightmap(regionpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating heightmap from region file {}", regionpath.display());

    let heightmaps = data::read_region_chunk_heightmaps(regionpath)?;
    let min_cx = heightmaps.keys().map(|(x, _)| x).min().unwrap();
    let max_cx = heightmaps.keys().map(|(x, _)| x).max().unwrap();
    let min_cz = heightmaps.keys().map(|(_, z)| z).min().unwrap();
    let max_cz = heightmaps.keys().map(|(_, z)| z).max().unwrap();
    let width = (max_cx - min_cx + 1) as u32 * 16;
    let height = (max_cz - min_cz + 1) as u32 * 16;

    let mut pixels: Vec<u8> = vec![0; (width * height) as usize];
    for ((cx, cz), cpixels) in heightmaps.iter() {
        let co = (cz - min_cz) as u32 * width * 16 + (cx - min_cx) as u32 * 16;

        for bz in 0..16 {
            for bx in 0..16 {
                pixels[(co + bz * width + bx) as usize] = cpixels[(bz * 16 + bx) as usize] as u8;
            }
        }
    }

    let file = File::create(outpath)?;
    image::draw_height_map(&pixels, width, height, file)?;

    Ok(())
}
