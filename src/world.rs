use std::io::{Error, ErrorKind};
use std::path::Path;

use regex::Regex;

use super::sizes::*;
use super::region;
use super::types::{Edges, Pair};

pub struct World {
    pub regions: Vec<Pair<i32>>,
    pub rlimits: Edges<i32>,
    pub margins: Edges<usize>,
}

impl World {
    pub fn get_ortho_size(&self) -> Pair<usize> {
        Pair {
            x: ((self.rlimits.e - self.rlimits.w + 1) as usize * CHUNKS_IN_REGION
                - (self.margins.e + self.margins.w)) * BLOCKS_IN_CHUNK,
            z: ((self.rlimits.s - self.rlimits.n + 1) as usize * CHUNKS_IN_REGION
                - (self.margins.n + self.margins.s)) * BLOCKS_IN_CHUNK,
        }
    }
}

pub fn read_world_regions(path: &Path) -> Result<Vec<Pair<i32>>, Error> {
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
                regions.push(Pair { x: rx, z: rz });
            }
        }
    }

    Ok(regions)
}

pub fn get_world(worldpath: &Path) -> Result<World, Error> {
    let regions = read_world_regions(worldpath)?;

    let rlimits = Edges {
        n: regions.iter().map(|c| c.z).min().unwrap(),
        e: regions.iter().map(|c| c.x).max().unwrap(),
        s: regions.iter().map(|c| c.z).max().unwrap(),
        w: regions.iter().map(|c| c.x).min().unwrap(),
    };

    println!("Reading chunk boundaries");
    let mut margins = Edges {
        n: CHUNKS_IN_REGION,
        e: CHUNKS_IN_REGION,
        s: CHUNKS_IN_REGION,
        w: CHUNKS_IN_REGION,
    };
    for r in regions.iter() {
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", r.x, r.z));
        if r.x == rlimits.w || r.x == rlimits.e || r.z == rlimits.n || r.z == rlimits.s {
            let chunks = region::read_region_chunk_coords(regionpath.as_path())?;
            if chunks.len() == 0 {
                continue;
            }

            if r.z == rlimits.n {
                let min_cz = chunks.iter().map(|c| c.z).min().unwrap() as usize;
                margins.n = std::cmp::min(margins.n, min_cz);
            }
            if r.x == rlimits.e {
                let max_cx = chunks.iter().map(|c| c.x).max().unwrap() as usize;
                margins.e = std::cmp::min(margins.e, CHUNKS_IN_REGION - max_cx - 1);
            }
            if r.z == rlimits.s {
                let max_cz = chunks.iter().map(|c| c.z).max().unwrap() as usize;
                margins.s = std::cmp::min(margins.s, CHUNKS_IN_REGION - max_cz - 1);
            }
            if r.x == rlimits.w {
                let min_cx = chunks.iter().map(|c| c.x).min().unwrap() as usize;
                margins.w = std::cmp::min(margins.w, min_cx);
            }
        }
    }

    Ok(World {
        regions,
        rlimits,
        margins,
    })
}
