// extern crate flate2;
// extern crate nbt;
extern crate regex;
// extern crate serde_json;

use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::exit;
use std::result::Result;

// use flate2::read::GzDecoder;

// use nbt::Result;
// use nbt::Blob;

use regex::Regex;

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

fn read_regions(path: &Path) -> Result<Vec<(i32, i32)>, Error> {
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "Directory not found."));
    }

    let region_path = path.join("region");
    if !region_path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "No region subdirectory found in path."));
    }

    let mut regions = Vec::new();
    let re = Regex::new(r"^r\.([-\d]+)\.([-\d]+)\.mca$").unwrap();

    for entry in fs::read_dir(region_path)? {
        if let Some(filename) = entry?.file_name().to_str() {
            if let Some(caps) = re.captures(filename) {
                let rx = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
                let rz = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();
                regions.push((rx, rz));
            }
        }
    }

    Ok(regions)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.into_iter().skip(1).take(1).next() {
        let path = Path::new(&arg);
        if let Some(path_str) = path.to_str() {
            println!("Path: {}", path_str);

            match read_regions(path) {
                Ok(regions) => println!("{:?}", regions),
                Err(err) => {
                    eprintln!("error: {}", err);
                    exit(1)
                }
            }
        } else {
            eprintln!("error: Path does not convert to string.");
            exit(1)
        }
    } else {
        eprintln!("error: A path argument is required.");
        exit(1)
    }
}
