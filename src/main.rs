use std::env;
use std::path::Path;
use std::process::exit;

mod data;
mod image;
mod tinymaps;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.into_iter().skip(1).take(1).next() {
        let inpath = Path::new(&arg);

        let mode = match inpath.extension() {
            Some(ext) if ext == "mca" => "region",
            _ => "world",
        };

        let outdir = Path::new("./results");
        std::fs::create_dir_all(outdir).unwrap();
        let outpath = outdir.join(format!("{}.png", mode));

        let result = match mode {
            "region" => tinymaps::draw_region_chunk_map(inpath, outpath.as_path()),
            _ => tinymaps::draw_world_chunk_map(inpath, outpath.as_path()),
        };

        match result {
            Ok(()) => println!("Saved map to {}", outpath.display()),
            Err(err) => {
                eprintln!("Error creating map: {}", err);
                exit(1)
            }
        }
    } else {
        eprintln!("Error: A path argument is required.");
        exit(1)
    }
}
