// extern crate flate2;
// extern crate nbt;
// extern crate serde_json;

use std::env;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::process::exit;

// use flate2::read::GzDecoder;

// use nbt::Result;
// use nbt::Blob;

mod data;
mod image;

// fn read_file() -> Result<()> {
//     let args: Vec<String> = env::args().collect();
//     if let Some(arg) = args.into_iter().skip(1).take(1).next() {
//         let file = fs::File::open(&arg)?;
//         let mut level_reader = GzDecoder::new(file);
//         let blob = Blob::from_reader(&mut level_reader)?;
//         println!("================================= NBT Contents =================================");
//         println!("{}", blob);
//         println!("============================== JSON Representation =============================");
//         match serde_json::to_string_pretty(&blob) {
//             Ok(json) => println!("{}", json),
//             Err(e) => {
//                 eprintln!("error: {}", e);
//                 exit(1)
//             },
//         }
//         Ok(())
//     } else {
//         eprintln!("error: a filename is required.");
//         exit(1)
//     }
// }

// fn draw_world_region_map(worldpath: &Path) -> Result<&str, Box<Error>> {
//     println!("Drawing map of regions from world dir {}", worldpath.display());

//     let regions = data::read_regions(worldpath)?;

//     let min_x = regions.iter().map(|(x, _)| x).min().unwrap();
//     let max_x = regions.iter().map(|(x, _)| x).max().unwrap();
//     let min_z = regions.iter().map(|(_, z)| z).min().unwrap();
//     let max_z = regions.iter().map(|(_, z)| z).max().unwrap();
//     let width = max_x - min_x + 1;
//     let height = max_z - min_z + 1;

//     let mut pixels: Vec<bool> = vec![false; (width * height) as usize];
//     for (rx, rz) in regions.iter() {
//         pixels[((rz - min_z) * width + (rx - min_x)) as usize] = true;
//     }

//     let outpath = Path::new("./world.png");
//     let file = File::create(outpath)?;

//     image::draw_tiny_map(pixels.as_slice(), width as u32, height as u32, file)?;

//     Ok(outpath.to_str().unwrap())
// }

fn draw_world_chunk_map(worldpath: &Path) -> Result<&str, Box<Error>> {
    println!("Drawing map of chunks from world dir {}", worldpath.display());

    let regions = data::read_regions(worldpath)?;

    let min_rx = regions.iter().map(|(x, _)| x).min().unwrap();
    let max_rx = regions.iter().map(|(x, _)| x).max().unwrap();
    let min_rz = regions.iter().map(|(_, z)| z).min().unwrap();
    let max_rz = regions.iter().map(|(_, z)| z).max().unwrap();
    let cwidth = (max_rx - min_rx + 1) * 32;
    let cheight = (max_rz - min_rz + 1) * 32;

    let mut pixels: Vec<bool> = vec![false; (cwidth * cheight) as usize];
    for (rx, rz) in regions.iter() {
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", rx, rz));
        let regionpixels = data::read_region_chunks(&regionpath)?;

        let ro = (rz - min_rz) * cwidth * 32 + (rx - min_rx) * 32;

        for cx in 0..32 {
            for cz in 0..32 {
                pixels[(ro + cz * cwidth + cx) as usize] = regionpixels[(cz * 32 + cx) as usize];
            }
        }
    }

    let outpath = Path::new("./world.png");
    let file = File::create(outpath)?;

    image::draw_tiny_map(pixels.as_slice(), cwidth as u32, cheight as u32, file)?;

    Ok(outpath.to_str().unwrap())
}

fn draw_region_chunk_map(regionpath: &Path) -> Result<&str, Box<Error>> {
    println!("Drawing map of chunks from region file {}", regionpath.display());

    let pixels = data::read_region_chunks(regionpath)?;

    let outpath = Path::new("./region.png");
    let file = File::create(outpath)?;

    image::draw_tiny_map(&pixels, 32, 32, file)?;

    Ok(outpath.to_str().unwrap())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.into_iter().skip(1).take(1).next() {
        let path = Path::new(&arg);

        let result = match path.extension() {
            Some(ext) if ext == "mca" => draw_region_chunk_map(path),
            _ => draw_world_chunk_map(path),
        };

        match result {
            Ok(outpath) => println!("Saved map to {}", outpath),
            Err(err) => {
                eprintln!("error: {}", err);
                exit(1)
            }
        }
    } else {
        eprintln!("error: A path argument is required.");
        exit(1)
    }
}
