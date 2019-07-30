use std::cmp::{min, max};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::path::Path;

use super::region;
use super::sizes::*;
use super::types::*;

pub struct Region {
    pub cedges: Edges<usize>,
}

pub struct World {
    pub regions: HashMap<Pair<i32>, Region>,
    pub redges: Edges<i32>,
    pub cedges: Edges<i32>,
    pub csize: Pair<usize>,
}

pub fn read_world_regions(path: &Path, blimits: &Option<Edges<i32>>)
-> Result<HashMap<Pair<i32>, Region>, Error> {
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "Directory not found."));
    }

    let region_path = path.join("region");
    if !region_path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "No region subdirectory found in path."));
    }

    // If block limits were passed, transform them into region limits.
    let rlimits = blimits.and_then(|blimits| Some(Edges {
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
                    // If block limits were passed, find the chunk limits within the region.
                    let rclimits = blimits.and_then(|blimits| Some(Edges {
                        n: chunk_pos_in_region(block_to_chunk(blimits.n), Some(r.z)),
                        e: chunk_pos_in_region(block_to_chunk(blimits.e), Some(r.x)),
                        s: chunk_pos_in_region(block_to_chunk(blimits.s), Some(r.z)),
                        w: chunk_pos_in_region(block_to_chunk(blimits.w), Some(r.x)),
                    }));
                    let chunklist = region::read_region_chunk_coords(
                        entry.path().as_path(), &rclimits)?;
                    if chunklist.len() > 0 {
                        regions.insert(r, Region {
                            cedges: Edges {
                                n: chunklist.iter().map(|c| c.z).min().unwrap(),
                                e: chunklist.iter().map(|c| c.x).max().unwrap(),
                                s: chunklist.iter().map(|c| c.z).max().unwrap(),
                                w: chunklist.iter().map(|c| c.x).min().unwrap(),
                            },
                        });
                    }
                }
            }
        }
    }

    Ok(regions)
}

pub fn get_world(worldpath: &Path, blimits: &Option<Edges<i32>>) -> Result<World, Error> {
    let regions = read_world_regions(worldpath, blimits)?;

    println!("Reading chunk boundaries");

    let redges = Edges {
        n: regions.keys().map(|r| r.z).min().unwrap(),
        e: regions.keys().map(|r| r.x).max().unwrap(),
        s: regions.keys().map(|r| r.z).max().unwrap(),
        w: regions.keys().map(|r| r.x).min().unwrap(),
    };

    let mut cedges = Edges {
        n: i32::max_value(),
        e: i32::min_value(),
        s: i32::min_value(),
        w: i32::max_value(),
    };
    for (r, region) in regions.iter() {
        if r.z == redges.n {
            cedges.n = min(cedges.n, r.z * CHUNKS_IN_REGION as i32 + region.cedges.n as i32);
        }
        if r.x == redges.e {
            cedges.e = max(cedges.e, r.x * CHUNKS_IN_REGION as i32 + region.cedges.e as i32);
        }
        if r.z == redges.s {
            cedges.s = max(cedges.s, r.z * CHUNKS_IN_REGION as i32 + region.cedges.s as i32);
        }
        if r.x == redges.w {
            cedges.w = min(cedges.w, r.x * CHUNKS_IN_REGION as i32 + region.cedges.w as i32);
        }
    }

    Ok(World {
        regions,
        redges,
        cedges,
        csize: Pair {
            x: (cedges.e - cedges.w + 1) as usize,
            z: (cedges.s - cedges.n + 1) as usize,
        },
    })
}
