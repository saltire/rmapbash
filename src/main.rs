use std::path::Path;

use clap::{Arg, App};

mod biometypes;
mod blocktypes;
mod color;
mod data;
mod image;
mod isomap;
mod map;
mod nbt;
mod options;
mod orthomap;
mod region;
mod sizes;
mod types;
mod world;

fn main() {
    let matches = App::new("rmapbash")
        .about("Minecraft map renderer")
        .author("saltiresable@gmail.com")
        .version("0.1.0")
        .arg(Arg::with_name("INPATH")
            .help("Path to either a save directory or a .dat file")
            .required(true)
            .index(1))
        .arg(Arg::with_name("OUTPATH")
            .help("Path to an output .png file")
            .default_value("world.png")
            .validator(|v| match Path::new(&v).extension() {
                Some(ext) if ext == "png" => Ok(()),
                _ => Err("Output path must be a .png file".to_string()),
            })
            .index(2))
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
            .value_names(&["W", "N", "E", "S"])
            .allow_hyphen_values(true)
            .validator(|v| v.parse::<isize>().map(|_| ())
                .map_err(|_| "Horizontal block limits must be numbers".to_string()))
            .help("Horizontal block limits"))
        .arg(Arg::with_name("y")
            .short("y")
            .long("yblocks")
            .value_names(&["MIN", "MAX"])
            .validator(|v| v.parse::<usize>().map(|_| ())
                .map_err(|_| "Vertical block limits must be positive numbers".to_string()))
            .help("Vertical block limits"))
        .get_matches();

    let options = options::get_options(&matches);

    match options.inpath.extension() {
        Some(ext) if ext == "dat" => match data::read_dat_file(options.inpath) {
            Ok(()) => println!("Done."),
            Err(err) => eprintln!("Error reading data: {}", err),
        },
        _ => match map::create_map(&options) {
            Ok(()) => println!("Done."),
            Err(err) => eprintln!("Error creating map: {}", err),
        },
    };
}
