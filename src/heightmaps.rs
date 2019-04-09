use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::data;
use super::image;

pub fn draw_region_heightmap(regionpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Drawing heightmap from region file {}", regionpath.display());

    let heightmaps = data::read_region_chunk_heightmaps(regionpath)?;

    let min_cx = heightmaps.keys().map(|(x, _)| x).min().unwrap();
    let max_cx = heightmaps.keys().map(|(x, _)| x).max().unwrap();
    let min_cz = heightmaps.keys().map(|(_, z)| z).min().unwrap();
    let max_cz = heightmaps.keys().map(|(_, z)| z).max().unwrap();
    let cwidth = max_cx - min_cx + 1;
    let cheight = max_cz - min_cz + 1;
    let width: u32 = cwidth as u32 * 16;
    let height: u32 = cheight as u32 * 16;

    let mut pixels: Vec<u8> = vec![0; (width * height) as usize];
    for ((cx, cz), cpixels) in heightmaps.iter() {
        let co: u32 = (cz - min_cz) as u32 * width * 16 + (cx - min_cx) as u32 * 16;

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
