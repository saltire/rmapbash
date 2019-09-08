use std::cmp::{min, max};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::ops::Range;
use std::path::Path;

use super::region;
use super::sizes::*;
use super::types::*;

pub struct Region {
    pub cedges: Edges<usize>,
}

pub struct World<'a> {
    pub path: &'a Path,
    pub regions: HashMap<Pair<isize>, Region>,
    pub redges: Edges<isize>,
    pub cedges: Edges<isize>,
    pub bedges: Edges<isize>,
    pub rsize: Pair<usize>,
    pub csize: Pair<usize>,
    pub bsize: Pair<usize>,
    pub ylimits: &'a Range<usize>,
}

pub fn read_world_regions(path: &Path, blimits: &Option<Edges<isize>>)
-> Result<HashMap<Pair<isize>, Region>, Error> {
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

pub fn get_world<'a>(worldpath: &'a Path, blimits: &Option<Edges<isize>>, ylimits: &'a Range<usize>)
-> Result<World<'a>, Error> {
    let regions = read_world_regions(worldpath, blimits)?;
    if regions.len() == 0 {
        return Err(Error::new(ErrorKind::NotFound, "No data in world."));
    }

    let redges = Edges {
        n: regions.keys().map(|r| r.z).min().unwrap(),
        e: regions.keys().map(|r| r.x).max().unwrap(),
        s: regions.keys().map(|r| r.z).max().unwrap(),
        w: regions.keys().map(|r| r.x).min().unwrap(),
    };

    let mut cedges = Edges {
        n: isize::max_value(),
        e: isize::min_value(),
        s: isize::min_value(),
        w: isize::max_value(),
    };
    for (r, region) in regions.iter() {
        if r.z == redges.n {
            cedges.n = min(cedges.n, r.z * CHUNKS_IN_REGION as isize + region.cedges.n as isize);
        }
        if r.x == redges.e {
            cedges.e = max(cedges.e, r.x * CHUNKS_IN_REGION as isize + region.cedges.e as isize);
        }
        if r.z == redges.s {
            cedges.s = max(cedges.s, r.z * CHUNKS_IN_REGION as isize + region.cedges.s as isize);
        }
        if r.x == redges.w {
            cedges.w = min(cedges.w, r.x * CHUNKS_IN_REGION as isize + region.cedges.w as isize);
        }
    }

    let cbedges = Edges {
        n: cedges.n * BLOCKS_IN_CHUNK as isize,
        e: cedges.e * BLOCKS_IN_CHUNK as isize + MAX_BLOCK_IN_CHUNK as isize,
        s: cedges.s * BLOCKS_IN_CHUNK as isize + MAX_BLOCK_IN_CHUNK as isize,
        w: cedges.w * BLOCKS_IN_CHUNK as isize,
    };
    let bedges = match blimits {
        Some(blimits) => Edges {
            n: max(cbedges.n, blimits.n),
            e: min(cbedges.e, blimits.e),
            s: min(cbedges.s, blimits.s),
            w: max(cbedges.w, blimits.w),
        },
        None => cbedges,
    };

    Ok(World {
        path: worldpath,
        regions,
        redges,
        cedges,
        bedges,
        rsize: redges.size(),
        csize: cedges.size(),
        bsize: bedges.size(),
        ylimits,
    })
}
