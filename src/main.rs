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

fn main() {
    let matches = App::new("rmapbash")
        .about("Minecraft map renderer")
        .arg(Arg::with_name("PATH")
            .help("Path to either a save directory, a region file (.mca), or a data file (.dat)")
            .required(true)
            .index(1))
        // .arg(Arg::with_name("r")
        //     .short("r")
        //     .long("region")
        //     .value_names(&["RX", "RZ"])
        //     .help("Region coordinates"))
        .arg(Arg::with_name("i")
            .short("i")
            .long("isometric")
            .help("Isometric view"))
        .arg(Arg::with_name("n")
            .short("n")
            .long("night")
            .help("Night lighting"))
        .get_matches();

    if let Some(path_str) = matches.value_of("PATH") {
        let inpath = Path::new(&path_str);

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
                let worldpath = if mode == "world" { inpath }
                    else { inpath.parent().unwrap().parent().unwrap() };

                let outdir = Path::new("./results");
                std::fs::create_dir_all(outdir).unwrap();
                let outpathbuf = outdir.join(format!("{}.png", mode));
                let outpath = outpathbuf.as_path();

                let iso = matches.is_present("i");

                let dim = match worldpath.file_stem().unwrap().to_str() {
                    Some("DIM-1") => "nether",
                    Some("DIM1") => "end",
                    _ => "overworld",
                };
                let lighting = if dim != "overworld" { dim }
                    else if matches.is_present("n") { "night" }
                    else { "day" };

                println!("View:     {}", if iso { "isometric" } else { "orthographic" });
                println!("Lighting: {}", lighting);

                let start = Instant::now();

                println!("Getting block types");
                let blocktypes = blocktypes::get_block_types(lighting);

                let result = match mode {
                    "region" => {
                        let r = region::get_coords_from_path(inpath.to_str().unwrap()).unwrap();

                        if iso {
                            isomap::draw_region_iso_map(worldpath, &r, outpath, &blocktypes)
                        } else {
                            orthomap::draw_region_ortho_map(worldpath, &r, outpath, &blocktypes)
                        }
                    },
                    _ => if iso {
                        isomap::draw_world_iso_map(worldpath, outpath, &blocktypes)
                    } else {
                        orthomap::draw_world_ortho_map(worldpath, outpath, &blocktypes)
                    },
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
