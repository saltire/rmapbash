use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::data;
use super::image;
use super::world;

fn draw_chunk(pixels: &mut [u8], cpixels: &[u8], co: &usize, width: &usize) {
    for bz in 0..16 {
        for bx in 0..16 {
            pixels[(co + bz * width + bx) as usize] = cpixels[(bz * 16 + bx) as usize];
        }
    }
}

#[allow(dead_code)]
pub fn draw_world_heightmap(worldpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating heightmap from world dir {}", worldpath.display());

    let world = world::get_world(worldpath)?;

    let mut pixels = vec![0u8; world.width * world.height];
    for (rx, rz) in world.regions.iter() {
        println!("Reading heightmap for region {}, {}", rx, rz);
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", rx, rz));
        let rheightmaps = data::read_region_chunk_heightmaps(regionpath.as_path())?;

        let arx = (rx - world.rmin.x) as usize;
        let arz = (rz - world.rmin.z) as usize;

        for ((cx, cz), cpixels) in rheightmaps.iter() {
            let acx = arx * 32 + *cx as usize;
            let acz = arz * 32 + *cz as usize;
            let co = (acz - world.margins.n as usize) * 16 * world.width +
                (acx - world.margins.w as usize) * 16;

            draw_chunk(&mut pixels, cpixels, &co, &world.width);
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, world.width, world.height, file, false)?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_region_heightmap(regionpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating heightmap from region file {}", regionpath.display());

    let heightmaps = data::read_region_chunk_heightmaps(regionpath)?;
    if heightmaps.keys().len() == 0 {
        println!("No chunks in region.");
        return Ok(());
    }

    let climits = world::Edges {
        n: heightmaps.keys().map(|(_, z)| z).min().unwrap(),
        e: heightmaps.keys().map(|(x, _)| x).max().unwrap(),
        s: heightmaps.keys().map(|(_, z)| z).max().unwrap(),
        w: heightmaps.keys().map(|(x, _)| x).min().unwrap(),
    };
    let width = (climits.e - climits.w + 1) as usize * 16;
    let height = (climits.s - climits.n + 1) as usize * 16;

    let mut pixels = vec![0u8; width * height];
    for ((cx, cz), cpixels) in heightmaps.iter() {
        let acx = (cx - climits.w) as usize;
        let acz = (cz - climits.n) as usize;
        let co = acz * 16 * width + acx * 16;

        draw_chunk(&mut pixels, cpixels, &co, &width);
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, width, height, file, false)?;

    Ok(())
}
