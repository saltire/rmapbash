use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::path::Path;

use super::region;
use super::sizes::*;
use super::types::{Edges, Pair};

pub struct World {
    pub regions: HashMap<Pair<i32>, Vec<Pair<usize>>>,
    pub rlimits: Edges<i32>,
    pub margins: Edges<usize>,
}

impl World {
    pub fn get_chunk_size(&self) -> Pair<usize> {
        Pair {
            x: (self.rlimits.e - self.rlimits.w + 1) as usize * CHUNKS_IN_REGION
                - self.margins.e - self.margins.w,
            z: (self.rlimits.s - self.rlimits.n + 1) as usize * CHUNKS_IN_REGION
                - self.margins.n - self.margins.s,
        }
    }

    pub fn get_ortho_size(&self) -> Pair<usize> {
        let csize = self.get_chunk_size();
        Pair {
            x: csize.x * BLOCKS_IN_CHUNK,
            z: csize.z * BLOCKS_IN_CHUNK,
        }
    }
}

pub fn read_world_regions(path: &Path) -> Result<HashMap<Pair<i32>, Vec<Pair<usize>>>, Error> {
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "Directory not found."));
    }

    let region_path = path.join("region");
    if !region_path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "No region subdirectory found in path."));
    }

    let mut regions = HashMap::new();
    for dir_entry in std::fs::read_dir(region_path)? {
        let entry = dir_entry?;
        if let Some(filename) = entry.file_name().to_str() {
            if let Some(r) = region::get_coords_from_path(filename) {
                let chunks = region::read_region_chunk_coords(entry.path().as_path())?;
                if chunks.len() > 0 {
                    regions.insert(r, chunks);
                }
            }
        }
    }

    Ok(regions)
}

pub fn get_world(worldpath: &Path) -> Result<World, Error> {
    let regions = read_world_regions(worldpath)?;

    println!("Reading chunk boundaries");
    let rlimits = Edges {
        n: regions.keys().map(|r| r.z).min().unwrap(),
        e: regions.keys().map(|r| r.x).max().unwrap(),
        s: regions.keys().map(|r| r.z).max().unwrap(),
        w: regions.keys().map(|r| r.x).min().unwrap(),
    };
    let mut margins = Edges {
        n: CHUNKS_IN_REGION,
        e: CHUNKS_IN_REGION,
        s: CHUNKS_IN_REGION,
        w: CHUNKS_IN_REGION,
    };
    for (r, chunks) in regions.iter() {
        if r.z == rlimits.n {
            let min_cz = chunks.iter().map(|c| c.z).min().unwrap();
            margins.n = std::cmp::min(margins.n, min_cz);
        }
        if r.x == rlimits.e {
            let max_cx = chunks.iter().map(|c| c.x).max().unwrap();
            margins.e = std::cmp::min(margins.e, CHUNKS_IN_REGION - max_cx - 1);
        }
        if r.z == rlimits.s {
            let max_cz = chunks.iter().map(|c| c.z).max().unwrap();
            margins.s = std::cmp::min(margins.s, CHUNKS_IN_REGION - max_cz - 1);
        }
        if r.x == rlimits.w {
            let min_cx = chunks.iter().map(|c| c.x).min().unwrap();
            margins.w = std::cmp::min(margins.w, min_cx);
        }
    }

    Ok(World {
        regions,
        rlimits,
        margins,
    })
}
