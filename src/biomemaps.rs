use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::biometypes;
use super::data;
use super::image;

fn draw_chunk(pixels: &mut [u8], cbiomes: &[u8], biometypes: &Vec<biometypes::BiomeType>,
    co: &usize, width: &usize) {
    for bz in 0..16 {
        for bx in 0..16 {
            let bbiome = cbiomes[(bz * 16 + bx) as usize];
            if let Some(biome) = biometypes.iter().find(|b| b.id == bbiome) {
                let po = (co + bz * width + bx) * 4;
                pixels[po] = biome.grass.0;
                pixels[po + 1] = biome.grass.1;
                pixels[po + 2] = biome.grass.2;
                pixels[po + 3] = 255;
            }
        }
    }
}

#[allow(dead_code)]
pub fn draw_world_biome_map(worldpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating biome map from world dir {}", worldpath.display());

    let biometypes = biometypes::get_biome_types();

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
                margins.3 = std::cmp::min(margins.3, *min_cx);
            }
        }
    }
    let width = ((max_rx - min_rx + 1) as usize * 32 - (margins.1 + margins.3) as usize) * 16;
    let height = ((max_rz - min_rz + 1) as usize * 32 - (margins.0 + margins.2) as usize) * 16;

    let mut pixels: Vec<u8> = vec![0; (width * height * 4) as usize];
    for (rx, rz) in regions.iter() {
        println!("Reading biome map for region {}, {}", rx, rz);
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", rx, rz));
        let rheightmaps = data::read_region_chunk_biomes(regionpath.as_path())?;

        let arx = (rx - min_rx) as usize;
        let arz = (rz - min_rz) as usize;

        for ((cx, cz), cbiomes) in rheightmaps.iter() {
            let acx = arx * 32 + *cx as usize;
            let acz = arz * 32 + *cz as usize;
            let co = (acz - margins.0 as usize) * 16 * width + (acx - margins.3 as usize) * 16;

            draw_chunk(&mut pixels, cbiomes, &biometypes, &co, &width);
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, width, height, file, true)?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_region_biome_map(regionpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating biome map from region file {}", regionpath.display());

    let biometypes = biometypes::get_biome_types();

    let rbiomes = data::read_region_chunk_biomes(regionpath)?;
    if rbiomes.keys().len() == 0 {
        println!("No chunks in region.");
        return Ok(());
    }

    let min_cx = rbiomes.keys().map(|(x, _)| x).min().unwrap();
    let max_cx = rbiomes.keys().map(|(x, _)| x).max().unwrap();
    let min_cz = rbiomes.keys().map(|(_, z)| z).min().unwrap();
    let max_cz = rbiomes.keys().map(|(_, z)| z).max().unwrap();
    let width = (max_cx - min_cx + 1) as usize * 16;
    let height = (max_cz - min_cz + 1) as usize * 16;

    let mut pixels: Vec<u8> = vec![0; width * height * 4];
    for ((cx, cz), cbiomes) in rbiomes.iter() {
        let acx = (cx - min_cx) as usize;
        let acz = (cz - min_cz) as usize;
        let co = acz * 16 * width + acx * 16;

        draw_chunk(&mut pixels, cbiomes, &biometypes, &co, &width);
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, width, height, file, true)?;

    Ok(())
}
