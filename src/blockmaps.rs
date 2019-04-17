use std::error::Error;
use std::fs::File;
use std::path::Path;

use csv::{Reader, StringRecord};

use super::data;
use super::image;

fn get_blocks() -> Vec<StringRecord> {
    let csvpath = Path::new("./resources/colors.csv");
    let mut reader = Reader::from_path(csvpath).unwrap();
    let mut blocks = Vec::new();
    for result in reader.records() {
        blocks.push(result.unwrap());
    }
    blocks
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

    for ((cx, cz), blocks) in blockmaps.iter() {
        // println!("Drawing chunk {}, {}", cx, cz);
        let acx = (cx - min_cx) as usize;
        let acz = (cz - min_cz) as usize;
        let co = acz * 16 * width + acx * 16;

        for bz in 0..16 {
            for bx in 0..16 {
                let mut topblock = 0;
                for by in (0..256).rev() {
                    let bo = by * 256 + bz * 16 + bx;
                    // Find the first block from the top that isn't air.
                    if blocks[bo] != 0 && blocks[bo] != 14 {
                        topblock = blocks[bo];
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
