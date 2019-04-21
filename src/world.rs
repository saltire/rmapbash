use std::io::{Error, ErrorKind};
use std::path::Path;

use regex::Regex;

use super::region;

pub struct Coords<T> {
    pub x: T,
    pub z: T,
}

pub struct Edges<T> {
    pub n: T,
    pub e: T,
    pub s: T,
    pub w: T,
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub regions: Vec<(i32, i32)>,
    pub rmin: Coords<i32>,
    pub margins: Edges<u8>,
}

pub fn read_world_regions(path: &Path) -> Result<Vec<(i32, i32)>, Error> {
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "Directory not found."));
    }

    let region_path = path.join("region");
    if !region_path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "No region subdirectory found in path."));
    }

    let mut regions = Vec::new();
    let re = Regex::new(r"^r\.([-\d]+)\.([-\d]+)\.mca$").unwrap();

    for entry in std::fs::read_dir(region_path)? {
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

pub fn get_world(worldpath: &Path) -> Result<World, Error> {
    let regions = read_world_regions(worldpath)?;

    let rlimits = Edges {
        n: *regions.iter().map(|(_, z)| z).min().unwrap(),
        e: *regions.iter().map(|(x, _)| x).max().unwrap(),
        s: *regions.iter().map(|(_, z)| z).max().unwrap(),
        w: *regions.iter().map(|(x, _)| x).min().unwrap(),
    };

    println!("Reading chunk boundaries");
    let mut margins = Edges { n: 32, e: 32, s: 32, w: 32 };
    for (rx, rz) in regions.iter() {
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", rx, rz));
        if rx == &rlimits.w || rx == &rlimits.e || rz == &rlimits.n || rz == &rlimits.s {
            let chunks = region::read_region_chunk_coords(regionpath.as_path())?;
            if chunks.len() == 0 {
                continue;
            }

            if rz == &rlimits.n {
                let min_cz = chunks.iter().map(|(_, z)| z).min().unwrap();
                margins.n = std::cmp::min(margins.n, *min_cz);
            }
            if rx == &rlimits.e {
                let max_cx = chunks.iter().map(|(x, _)| x).max().unwrap();
                margins.e = std::cmp::min(margins.e, 31 - *max_cx);
            }
            if rz == &rlimits.s {
                let max_cz = chunks.iter().map(|(_, z)| z).max().unwrap();
                margins.s = std::cmp::min(margins.s, 31 - *max_cz);
            }
            if rx == &rlimits.w {
                let min_cx = chunks.iter().map(|(x, _)| x).min().unwrap();
                margins.w = std::cmp::min(margins.w, *min_cx);
            }
        }
    }

    Ok(World {
        width: ((rlimits.e - rlimits.w + 1) as usize * 32 - (margins.e + margins.w) as usize) * 16,
        height: ((rlimits.s - rlimits.n + 1) as usize * 32 - (margins.n + margins.s) as usize) * 16,
        regions,
        rmin: Coords { x: rlimits.w, z: rlimits.n },
        margins,
    })
}
