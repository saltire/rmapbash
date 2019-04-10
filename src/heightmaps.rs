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

    println!("Reading chunk boundaries");
    let mut margins = (32, 32, 32, 32);
    for (rx, rz) in regions.iter() {
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", rx, rz));
        if rx == min_rx || rx == max_rx || rz == min_rz || rz == max_rz {
            let chunks = data::read_region_chunk_coords(regionpath.as_path())?;
            if chunks.len() == 0 {
                continue;
            }

            if rz == min_rz {
                let min_cz = chunks.iter().map(|(_, z)| z).min().unwrap();
                margins.0 = std::cmp::min(margins.0, *min_cz);
            }
            if rx == max_rx {
                let max_cx = chunks.iter().map(|(x, _)| x).max().unwrap();
                margins.1 = std::cmp::min(margins.1, 31 - *max_cx);
            }
            if rz == max_rz {
                let max_cz = chunks.iter().map(|(_, z)| z).max().unwrap();
                margins.2 = std::cmp::min(margins.2, 31 - *max_cz);
            }
            if rx == min_rx {
                let min_cx = chunks.iter().map(|(x, _)| x).min().unwrap();
                margins.3 = std::cmp::min(margins.0, *min_cx);
            }
        }
    }
    let width = ((max_rx - min_rx + 1) as u32 * 32 - (margins.1 + margins.3) as u32) * 16;
    let height = ((max_rz - min_rz + 1) as u32 * 32 - (margins.0 + margins.2) as u32) * 16;

    let mut pixels: Vec<u8> = vec![0; (width * height) as usize];
    for (rx, rz) in regions.iter() {
        println!("Reading heightmap for region {}, {}", rx, rz);

        let arx = (rx - min_rx) as u32;
        let arz = (rz - min_rz) as u32;

        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", rx, rz));
        let heightmaps = data::read_region_chunk_heightmaps(regionpath.as_path())?;

        for ((cx, cz), cpixels) in heightmaps.iter() {
            let acx = arx * 32 + *cx as u32;
            let acz = arz * 32 + *cz as u32;
            let co = (acz - margins.0 as u32) * 16 * width + (acx - margins.3 as u32) * 16;

            for bz in 0..16 {
                for bx in 0..16 {
                    pixels[(co + bz * width + bx) as usize] = cpixels[(bz * 16 + bx) as usize];
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
    if heightmaps.keys().len() == 0 {
        println!("No chunks in region.");
        return Ok(());
    }

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
