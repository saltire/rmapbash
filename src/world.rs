use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::path::Path;

use super::region;
use super::sizes::*;
use super::types::{Edges, Pair};

pub struct World {
    pub regions: HashMap<Pair<i32>, Vec<Pair<usize>>>,
    pub rlimits: Edges<i32>,
    pub cmargins: Edges<usize>,
    pub csize: Pair<usize>,
}

pub fn read_world_regions(path: &Path, limits: &Option<Edges<i32>>)
-> Result<HashMap<Pair<i32>, Vec<Pair<usize>>>, Error> {
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "Directory not found."));
    }

    let region_path = path.join("region");
    if !region_path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "No region subdirectory found in path."));
    }

    // If block limits were passed, transform them into region limits.
    let rlimits = limits.and_then(|blimits| Some(Edges {
        n: block_to_region(blimits.n),
        e: block_to_region(blimits.e),
        s: block_to_region(blimits.s),
        w: block_to_region(blimits.w),
    }));

    let mut regions = HashMap::new();
    for dir_entry in std::fs::read_dir(region_path)? {
        let entry = dir_entry?;
        if let Some(filename) = entry.file_name().to_str() {
            if let Some(r) = region::get_coords_from_path(filename) {
                if rlimits.is_none() || rlimits.unwrap().contains(&r) {
                    let chunks = region::read_region_chunk_coords(entry.path().as_path())?;
                    if chunks.len() > 0 {
                        regions.insert(r, chunks);
                    }
                }
            }
        }
    }

    Ok(regions)
}

pub fn get_world(worldpath: &Path, limits: &Option<Edges<i32>>) -> Result<World, Error> {
    let regions = read_world_regions(worldpath, limits)?;

    println!("Reading chunk boundaries");

    let rlimits = Edges {
        n: regions.keys().map(|r| r.z).min().unwrap(),
        e: regions.keys().map(|r| r.x).max().unwrap(),
        s: regions.keys().map(|r| r.z).max().unwrap(),
        w: regions.keys().map(|r| r.x).min().unwrap(),
    };

    let mut cmargins = Edges {
        n: CHUNKS_IN_REGION,
        e: CHUNKS_IN_REGION,
        s: CHUNKS_IN_REGION,
        w: CHUNKS_IN_REGION,
    };
    for (r, chunks) in regions.iter() {
        if r.z == rlimits.n {
            let min_cz = chunks.iter().map(|c| c.z).min().unwrap();
            cmargins.n = std::cmp::min(cmargins.n, min_cz);
        }
        if r.x == rlimits.e {
            let max_cx = chunks.iter().map(|c| c.x).max().unwrap();
            cmargins.e = std::cmp::min(cmargins.e, CHUNKS_IN_REGION - max_cx - 1);
        }
        if r.z == rlimits.s {
            let max_cz = chunks.iter().map(|c| c.z).max().unwrap();
            cmargins.s = std::cmp::min(cmargins.s, CHUNKS_IN_REGION - max_cz - 1);
        }
        if r.x == rlimits.w {
            let min_cx = chunks.iter().map(|c| c.x).min().unwrap();
            cmargins.w = std::cmp::min(cmargins.w, min_cx);
        }
    }

    Ok(World {
        regions,
        rlimits,
        cmargins,
        csize: Pair {
            x: (rlimits.e - rlimits.w + 1) as usize * CHUNKS_IN_REGION - cmargins.e - cmargins.w,
            z: (rlimits.s - rlimits.n + 1) as usize * CHUNKS_IN_REGION - cmargins.n - cmargins.s,
        },
    })
}
