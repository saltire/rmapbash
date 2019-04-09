extern crate regex;

use std::fs;
use std::io::{prelude::*, Error, ErrorKind};
use std::path::Path;
use std::result::Result;

use regex::Regex;

pub fn read_regions(path: &Path) -> Result<Vec<(i32, i32)>, Error> {
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

pub fn read_region_chunks(path: &Path) -> Result<[bool; 1024], Error> {
    let mut f = fs::File::open(path)?;
    let mut buf = [0; 4];
    let mut chunks = [false; 1024];

    for p in 0..1024 {
        f.read(&mut buf)?;
        let val = ((buf[0] as u32) << 24) | ((buf[1] as u32) << 16) |
            ((buf[2] as u32) << 8) | buf[3] as u32;
        if val > 0 {
            chunks[p] = true;
        }
    }

    Ok(chunks)
}
