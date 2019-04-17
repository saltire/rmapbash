use std::env;
use std::path::Path;

mod blockmaps;
mod data;
mod heightmaps;
mod image;
mod tinymaps;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.into_iter().skip(1).take(1).next() {
        let inpath = Path::new(&arg);

        let mode = match inpath.extension() {
            Some(ext) if ext == "dat" => "data",
            Some(ext) if ext == "mca" => "region",
            _ => "world",
        };

        match mode {
            "data" => match data::read_dat_file(inpath) {
                Ok(()) => println!("Done."),
                Err(err) => eprintln!("Error reading data: {}", err),
            },
            _ => {
                let outdir = Path::new("./results");
                std::fs::create_dir_all(outdir).unwrap();
                let outpath = outdir.join(format!("{}.png", mode));

                let result = match mode {
                    "region" => blockmaps::draw_region_block_map(inpath, outpath.as_path()),
                    // "region" => heightmaps::draw_region_heightmap(inpath, outpath.as_path()),
                    _ => heightmaps::draw_world_heightmap(inpath, outpath.as_path()),
                    // "region" => tinymaps::draw_region_chunk_map(inpath, outpath.as_path()),
                    // _ => tinymaps::draw_world_chunk_map(inpath, outpath.as_path()),
                };

                match result {
                    Ok(()) => println!("Saved map to {}", outpath.display()),
                    Err(err) => eprintln!("Error creating map: {}", err),
                }
            }
        }
    } else {
        eprintln!("Error: A path argument is required.");
    }
}
