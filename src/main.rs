use std::path::Path;
use std::time::Instant;

use clap::{Arg, App};

mod biometypes;
mod blocktypes;
mod color;
mod data;
mod image;
mod isomap;
mod nbt;
mod options;
mod orthomap;
mod region;
mod sizes;
mod types;
mod world;

use options::View;

fn main() {
    let matches = App::new("rmapbash")
        .about("Minecraft map renderer")
        .author("saltiresable@gmail.com")
        .version("0.1.0")
        .arg(Arg::with_name("PATH")
            .help("Path to either a save directory or a .dat file")
            .required(true)
            .index(1))
        .arg(Arg::with_name("i")
            .short("i")
            .long("isometric")
            .help("Isometric view"))
        .arg(Arg::with_name("n")
            .short("n")
            .long("night")
            .help("Night lighting"))
        .arg(Arg::with_name("b")
            .short("b")
            .long("blocks")
            .number_of_values(4)
            .allow_hyphen_values(true)
            .validator(|v| v.parse::<isize>().map(|_| ())
                .map_err(|_| "Block limits must be numbers".to_string()))
            .help("Block limits"))
        .get_matches();

    let options = options::get_options(&matches);

    match options.inpath.extension() {
        Some(ext) if ext == "dat" => match data::read_dat_file(options.inpath) {
            Ok(()) => println!("Done."),
            Err(err) => eprintln!("Error reading data: {}", err),
        },
        _ => {
            let outdir = Path::new("./results");
            std::fs::create_dir_all(outdir).unwrap();
            let outpathbuf = outdir.join("world.png");
            let outpath = outpathbuf.as_path();

            println!("View:     {:?}", options.view);
            println!("Lighting: {:?}", options.lighting);
            println!("Limits:   {}", if let Some(blimits) = options.blimits {
                format!("({}, {}) - ({}, {})", blimits.w, blimits.n, blimits.e, blimits.s)
            } else {
                "none".to_string()
            });

            let start = Instant::now();

            println!("Getting block types");
            let blocktypes = blocktypes::get_block_types(&options.lighting);

            let result = match options.view {
                View::Isometric =>
                    isomap::draw_iso_map(&options, outpath, &blocktypes),
                View::Orthographic =>
                    orthomap::draw_ortho_map(&options, outpath, &blocktypes),
            };

            let elapsed = start.elapsed();
            let mins = elapsed.as_secs() / 60;
            let secs = elapsed.as_secs() % 60;
            let ms = elapsed.subsec_millis();
            println!("Time elapsed: {}:{:02}.{:03}", mins, secs, ms);

            match result {
                Ok(()) => println!("Saved map to {}", outpath.display()),
                Err(err) => eprintln!("Error creating map: {}", err),
            }
        }
    }
}
