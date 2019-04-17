use std::error::Error;
use std::fs::File;
use std::path::Path;

use csv::{Reader, StringRecord};

use super::data;
use super::image;

fn get_blocks() -> Vec<StringRecord> {
    let csvpath = Path::new("./resources/blocks.csv");
    let mut reader = Reader::from_path(csvpath).unwrap();
    let mut blocks = Vec::new();
    for result in reader.records() {
        blocks.push(result.unwrap());
    }
    blocks
}

fn is_empty(block: u16) -> bool {
    block == 0 || block == 14 || block == 98 || block == 563
}

pub fn draw_world_block_map(worldpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating block map from world dir {}", worldpath.display());

    let blocktypes = get_blocks();
    let blocknames: Vec<String> = blocktypes.iter().map(|b| b[0].to_string()).collect();

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

    let mut pixels: Vec<u8> = vec![0; width * height * 4];
    for (rx, rz) in regions.iter() {
        println!("Reading block maps for region {}, {}", rx, rz);
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", rx, rz));
        let rblockmaps = data::read_region_chunk_block_maps(regionpath.as_path(), &blocknames)?;

        println!("Drawing block maps for region {}, {}", rx, rz);
        let arx = (rx - min_rx) as usize;
        let arz = (rz - min_rz) as usize;

        for ((cx, cz), cblocks) in rblockmaps.iter() {
            // println!("Drawing chunk {}, {}", cx, cz);
            let acx = arx * 32 + *cx as usize;
            let acz = arz * 32 + *cz as usize;
            let co = (acz - margins.0 as usize) * 16 * width + (acx - margins.3 as usize) * 16;

            for bz in 0..16 {
                for bx in 0..16 {
                    let mut topblock = 0;
                    for by in (0..256).rev() {
                        let bo = by * 256 + bz * 16 + bx;
                        if !is_empty(cblocks[bo]) {
                            topblock = cblocks[bo];
                            break;
                        }
                    }
                    let po = co + bz * width + bx;
                    let blocktype = &blocktypes[topblock as usize];
                    for c in 0..4 {
                        pixels[po * 4 + c] = blocktype[c + 1].parse().unwrap_or(0);
                    }
                }
            }
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, width, height, file)?;

    Ok(())
}

pub fn draw_region_block_map(regionpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating block map from region file {}", regionpath.display());

    let blocktypes = get_blocks();
    let blocknames: Vec<String> = blocktypes.iter().map(|b| b[0].to_string()).collect();

    let blockmaps = data::read_region_chunk_block_maps(regionpath, &blocknames)?;
    if blockmaps.keys().len() == 0 {
        println!("No chunks in region.");
        return Ok(());
    }

    let min_cx = blockmaps.keys().map(|(x, _)| x).min().unwrap();
    let max_cx = blockmaps.keys().map(|(x, _)| x).max().unwrap();
    let min_cz = blockmaps.keys().map(|(_, z)| z).min().unwrap();
    let max_cz = blockmaps.keys().map(|(_, z)| z).max().unwrap();
    let width = (max_cx - min_cx + 1) as usize * 16;
    let height = (max_cz - min_cz + 1) as usize * 16;

    let mut pixels: Vec<u8> = vec![0; width * height * 4];

    for ((cx, cz), cblocks) in blockmaps.iter() {
        // println!("Drawing chunk {}, {}", cx, cz);
        let acx = (cx - min_cx) as usize;
        let acz = (cz - min_cz) as usize;
        let co = acz * 16 * width + acx * 16;

        for bz in 0..16 {
            for bx in 0..16 {
                let mut topblock = 0;
                for by in (0..256).rev() {
                    let bo = by * 256 + bz * 16 + bx;
                    if !is_empty(cblocks[bo]) {
                        topblock = cblocks[bo];
                        break;
                    }
                }
                let po = co + bz * width + bx;
                let blocktype = &blocktypes[topblock as usize];
                for c in 0..4 {
                    pixels[po * 4 + c] = blocktype[c + 1].parse().unwrap_or(0);
                }
            }
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, width, height, file)?;

    Ok(())
}
