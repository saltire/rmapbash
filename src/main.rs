use std::cmp::{min, max};
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
mod orthomap;
mod region;
mod sizes;
mod types;
mod world;

use types::*;

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

    if let Some(path_str) = matches.value_of("PATH") {
        let inpath = Path::new(&path_str);

        let mode = match inpath.extension() {
            Some(ext) if ext == "dat" => "data",
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
                let outpathbuf = outdir.join(format!("{}.png", mode));
                let outpath = outpathbuf.as_path();

                let iso = matches.is_present("i");

                let dim = match inpath.file_stem().unwrap().to_str() {
                    Some("DIM-1") => "nether",
                    Some("DIM1") => "end",
                    _ => "overworld",
                };
                let lighting = if dim != "overworld" { dim }
                    else if matches.is_present("n") { "night" }
                    else { "day" };

                let blimits = matches.values_of("b").and_then(|mut b| {
                    let x1 = b.next().unwrap().parse::<isize>().unwrap();
                    let z1 = b.next().unwrap().parse::<isize>().unwrap();
                    let x2 = b.next().unwrap().parse::<isize>().unwrap();
                    let z2 = b.next().unwrap().parse::<isize>().unwrap();
                    Some(Edges {
                        n: min(z1, z2),
                        e: max(x1, x2),
                        s: max(z1, z2),
                        w: min(x1, x2),
                    })
                });

                println!("View:     {}", if iso { "isometric" } else { "orthographic" });
                println!("Lighting: {}", lighting);
                println!("Limits:   {}", if let Some(lim) = &blimits {
                    format!("({}, {}) - ({}, {})", lim.w, lim.n, lim.e, lim.s)
                } else {
                    "none".to_string()
                });

                let start = Instant::now();

                println!("Getting block types");
                let blocktypes = blocktypes::get_block_types(lighting);

                let result = if iso {
                    isomap::draw_iso_map(inpath, outpath, &blocktypes, &blimits)
                } else {
                    orthomap::draw_ortho_map(inpath, outpath, &blocktypes, &blimits)
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
    } else {
        eprintln!("Error: A path argument is required.");
    }
}
