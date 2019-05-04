use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::blocktypes;
use super::color;
use super::color::RGBA;
use super::image;
use super::region;
use super::sizes::*;
use super::types::*;
use super::world;

fn get_iso_size(csize: &Pair<usize>) -> Pair<usize> {
    Pair {
        x: (csize.x + csize.z) * ISO_CHUNK_X_MARGIN,
        z: (csize.x + csize.z) * ISO_CHUNK_Y_MARGIN + ISO_CHUNK_SIDE_HEIGHT,
    }
}

fn draw_chunk(pixels: &mut [u8], blocktypes: &Vec<blocktypes::BlockType>,
    cblocks: &[u16], ncblocks: &Edges<&[u16; BLOCKS_IN_CHUNK_3D]>,
    clights: &[u8], nclights: &Edges<&[u8; BLOCKS_IN_CHUNK_3D]>,
    cbiomes: &[u8], co: &usize, width: &usize, night: &bool) {
    for bz in (0..BLOCKS_IN_CHUNK).rev() {
        for bx in (0..BLOCKS_IN_CHUNK).rev() {
            let bo2 = bz * BLOCKS_IN_CHUNK + bx;

            let bpx = (ISO_CHUNK_X_MARGIN as i16 +
                (bx as i16 - bz as i16 - 1) * ISO_BLOCK_X_MARGIN as i16) as usize;
            let bpy2 = (bx + bz) * ISO_BLOCK_Y_MARGIN;

            for by in (0..BLOCKS_IN_CHUNK_Y).rev() {
                let bo3 = by * BLOCKS_IN_CHUNK_2D + bo2;
                if cblocks[bo3] == 0 {
                    continue;
                }

                let blocktype = &blocktypes[cblocks[bo3] as usize];

                let tlight = if *night && by < MAX_BLOCK_IN_CHUNK_Y {
                    clights[bo3 + BLOCKS_IN_CHUNK_2D]
                } else {
                    MAX_LIGHT_LEVEL
                };
                let llight = if *night {
                    if bz == MAX_BLOCK_IN_CHUNK {
                        nclights.s[bo3 - MAX_BLOCK_IN_CHUNK * BLOCKS_IN_CHUNK]
                    } else {
                        clights[bo3 + BLOCKS_IN_CHUNK]
                    }
                } else {
                    MAX_LIGHT_LEVEL
                };
                let rlight = if *night {
                    if bx == MAX_BLOCK_IN_CHUNK {
                        nclights.e[bo3 - MAX_BLOCK_IN_CHUNK]
                    } else {
                        clights[bo3 + 1]
                    }
                } else {
                    MAX_LIGHT_LEVEL
                };

                let tcolor = &blocktype.colors[cbiomes[bo2] as usize][tlight as usize];
                if tcolor.a == 0 {
                    continue;
                }
                let lcolor = &blocktype.colors[cbiomes[bo2] as usize][llight as usize];
                let rcolor = &blocktype.colors[cbiomes[bo2] as usize][rlight as usize];

                let bpy = bpy2 + (MAX_BLOCK_IN_CHUNK_Y - by) * ISO_BLOCK_SIDE_HEIGHT;

                // Don't draw the top if the block above is the same as this one.
                // This prevents stripes appearing in columns of translucent blocks.
                let skip_top = by < MAX_BLOCK_IN_CHUNK_Y &&
                    cblocks[bo3] == cblocks[bo3 + BLOCKS_IN_CHUNK_2D];

                for y in (if skip_top { ISO_BLOCK_Y_MARGIN } else { 0 })..ISO_BLOCK_HEIGHT {
                    for x in 0..ISO_BLOCK_WIDTH {
                        let po = (co + (bpy + y) * width + bpx + x) * 4;
                        if pixels[po + 3] == MAX_CHANNEL_VALUE {
                            continue;
                        }

                        let bcolor = if y < ISO_BLOCK_Y_MARGIN {
                            &tcolor
                        } else if x < ISO_BLOCK_X_MARGIN {
                            &lcolor
                        } else {
                            &rcolor
                        };

                        let pcolor = color::blend_alpha_color(&RGBA {
                            r: pixels[po],
                            g: pixels[po + 1],
                            b: pixels[po + 2],
                            a: pixels[po + 3],
                        }, bcolor);
                        pixels[po] = pcolor.r;
                        pixels[po + 1] = pcolor.g;
                        pixels[po + 2] = pcolor.b;
                        pixels[po + 3] = pcolor.a;
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn draw_world_iso_map(worldpath: &Path, outpath: &Path, night: bool)
-> Result<(), Box<Error>> {
    println!("Creating block map from world dir {}", worldpath.display());

    let world = world::get_world(worldpath)?;

    let csize = world.get_chunk_size();
    let size = get_iso_size(&csize);
    let mut pixels = vec![0u8; size.x * size.z * 4];

    let blocktypes = blocktypes::get_block_types();
    let blocknames: Vec<&str> = blocktypes.iter().map(|b| &b.name[..]).collect();

    let mut i = 0;
    let len = world.regions.len();

    for rz in (world.rlimits.n..world.rlimits.s + 1).rev() {
        for rx in (world.rlimits.w..world.rlimits.e + 1).rev() {
            let r = Pair { x: rx, z: rz };
            if !world.regions.contains(&r) {
                continue;
            }

            i += 1;
            println!("Reading blocks for region {}, {} ({}/{})", r.x, r.z, i, len);
            let regionpath = region::get_path_from_coords(worldpath, &r);
            let npaths = Edges {
                n: region::get_path_from_coords(worldpath, &Pair { x: r.x, z: r.z - 1 }),
                s: region::get_path_from_coords(worldpath, &Pair { x: r.x, z: r.z + 1 }),
                w: region::get_path_from_coords(worldpath, &Pair { x: r.x - 1, z: r.z }),
                e: region::get_path_from_coords(worldpath, &Pair { x: r.x + 1, z: r.z }),
            };
            let nmargins = Edges {
                n: Edges { n: MAX_CHUNK_IN_REGION as u8, s: 0, w: 0, e: 0 },
                s: Edges { n: 0, s: MAX_CHUNK_IN_REGION as u8, w: 0, e: 0 },
                w: Edges { n: 0, s: 0, w: MAX_CHUNK_IN_REGION as u8, e: 0 },
                e: Edges { n: 0, s: 0, w: 0, e: MAX_CHUNK_IN_REGION as u8 },
            };
            let rblocks = region::read_region_chunk_blocks(regionpath.as_path(), &Edges::default(), &blocknames)?;
            let nrblocks = Edges {
                n: region::read_region_chunk_blocks(npaths.n.as_path(), &nmargins.n, &blocknames)?,
                s: region::read_region_chunk_blocks(npaths.s.as_path(), &nmargins.s, &blocknames)?,
                w: region::read_region_chunk_blocks(npaths.w.as_path(), &nmargins.w, &blocknames)?,
                e: region::read_region_chunk_blocks(npaths.e.as_path(), &nmargins.e, &blocknames)?,
            };
            let rlights = region::read_region_chunk_lightmaps(regionpath.as_path(), &Edges::default())?;
            let nrlights = Edges {
                n: region::read_region_chunk_lightmaps(npaths.n.as_path(), &nmargins.n)?,
                s: region::read_region_chunk_lightmaps(npaths.s.as_path(), &nmargins.s)?,
                w: region::read_region_chunk_lightmaps(npaths.w.as_path(), &nmargins.w)?,
                e: region::read_region_chunk_lightmaps(npaths.e.as_path(), &nmargins.e)?,
            };
            let rbiomes = region::read_region_chunk_biomes(regionpath.as_path())?;

            println!("Drawing block map for region {}, {}", r.x, r.z);
            let arx = (r.x - world.rlimits.w) as usize;
            let arz = (r.z - world.rlimits.n) as usize;

            for cz in (0..CHUNKS_IN_REGION as u8).rev() {
                for cx in (0..CHUNKS_IN_REGION as u8).rev() {
                    let c = &Pair { x: cx, z: cz };
                    if !rblocks.contains_key(c) {
                        continue;
                    }

                    // println!("Drawing chunk {}, {}", c.x, c.z);
                    let acx = arx * CHUNKS_IN_REGION + c.x as usize - world.margins.w;
                    let acz = arz * CHUNKS_IN_REGION + c.z as usize - world.margins.n;

                    let cpx = (acx + csize.z - acz - 1) * ISO_CHUNK_X_MARGIN;
                    let cpy = (acx + acz) * ISO_CHUNK_Y_MARGIN;
                    let co = cpy * size.x + cpx;

                    let ncblocks = Edges {
                        n: if cz == 0 {
                            nrblocks.n.get(&Pair { x: cx, z: MAX_CHUNK_IN_REGION as u8 }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                        } else {
                            rblocks.get(&Pair { x: cx, z: cz - 1 }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                        },
                        s: if cz == MAX_CHUNK_IN_REGION as u8 {
                            nrblocks.s.get(&Pair { x: cx, z: 0 }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                        } else {
                            rblocks.get(&Pair { x: cx, z: cz + 1 }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                        },
                        w: if cx == 0 {
                            nrblocks.w.get(&Pair { x: MAX_CHUNK_IN_REGION as u8, z: cz }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                        } else {
                            rblocks.get(&Pair { x: cx - 1, z: cz }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                        },
                        e: if cx == MAX_CHUNK_IN_REGION as u8 {
                            nrblocks.e.get(&Pair { x: 0, z: cz }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                        } else {
                            rblocks.get(&Pair { x: cx + 1, z: cz }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                        },
                    };
                    let nclights = Edges {
                        n: if cz == 0 {
                            nrlights.n.get(&Pair { x: cx, z: MAX_CHUNK_IN_REGION as u8 }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                        } else {
                            rlights.get(&Pair { x: cx, z: cz - 1 }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                        },
                        s: if cz == MAX_CHUNK_IN_REGION as u8 {
                            nrlights.s.get(&Pair { x: cx, z: 0 }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                        } else {
                            rlights.get(&Pair { x: cx, z: cz + 1 }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                        },
                        w: if cx == 0 {
                            nrlights.w.get(&Pair { x: MAX_CHUNK_IN_REGION as u8, z: cz }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                        } else {
                            rlights.get(&Pair { x: cx - 1, z: cz }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                        },
                        e: if cx == MAX_CHUNK_IN_REGION as u8 {
                            nrlights.e.get(&Pair { x: 0, z: cz }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                        } else {
                            rlights.get(&Pair { x: cx + 1, z: cz }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                        },
                    };

                    draw_chunk(&mut pixels, &blocktypes,
                        &rblocks[c], &ncblocks, &rlights[c], &nclights, &rbiomes[c],
                        &co, &size.x, &night);
                }
            }
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, true)?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_region_iso_map(worldpath: &Path, r: &Pair<i32>, outpath: &Path, night: bool)
-> Result<(), Box<Error>> {
    println!("Getting block types");
    let blocktypes = blocktypes::get_block_types();
    let blocknames: Vec<&str> = blocktypes.iter().map(|b| &b.name[..]).collect();

    println!("Creating block map for region {}, {}", r.x, r.z);
    let regionpath = region::get_path_from_coords(worldpath, r);
    let npaths = Edges {
        n: region::get_path_from_coords(worldpath, &Pair { x: r.x, z: r.z - 1 }),
        s: region::get_path_from_coords(worldpath, &Pair { x: r.x, z: r.z + 1 }),
        w: region::get_path_from_coords(worldpath, &Pair { x: r.x - 1, z: r.z }),
        e: region::get_path_from_coords(worldpath, &Pair { x: r.x + 1, z: r.z }),
    };
    let nmargins = Edges {
        n: Edges { n: MAX_CHUNK_IN_REGION as u8, s: 0, w: 0, e: 0 },
        s: Edges { n: 0, s: MAX_CHUNK_IN_REGION as u8, w: 0, e: 0 },
        w: Edges { n: 0, s: 0, w: MAX_CHUNK_IN_REGION as u8, e: 0 },
        e: Edges { n: 0, s: 0, w: 0, e: MAX_CHUNK_IN_REGION as u8 },
    };

    println!("Reading blocks");
    let rblocks = region::read_region_chunk_blocks(regionpath.as_path(), &Edges::default(), &blocknames)?;
    if rblocks.keys().len() == 0 {
        println!("No chunks in region.");
        return Ok(());
    }
    println!("Reading neighbouring blocks");
    let nrblocks = Edges {
        n: region::read_region_chunk_blocks(npaths.n.as_path(), &nmargins.n, &blocknames)?,
        s: region::read_region_chunk_blocks(npaths.s.as_path(), &nmargins.s, &blocknames)?,
        w: region::read_region_chunk_blocks(npaths.w.as_path(), &nmargins.w, &blocknames)?,
        e: region::read_region_chunk_blocks(npaths.e.as_path(), &nmargins.e, &blocknames)?,
    };

    println!("Reading light maps");
    let rlights = region::read_region_chunk_lightmaps(regionpath.as_path(), &Edges::default())?;
    println!("Reading neighbouring light maps");
    let nrlights = Edges {
        n: region::read_region_chunk_lightmaps(npaths.n.as_path(), &nmargins.n)?,
        s: region::read_region_chunk_lightmaps(npaths.s.as_path(), &nmargins.s)?,
        w: region::read_region_chunk_lightmaps(npaths.w.as_path(), &nmargins.w)?,
        e: region::read_region_chunk_lightmaps(npaths.e.as_path(), &nmargins.e)?,
    };

    println!("Reading biomes");
    let rbiomes = region::read_region_chunk_biomes(regionpath.as_path())?;

    println!("Drawing block map");
    let climits = Edges {
        n: rblocks.keys().map(|c| c.z).min().unwrap(),
        e: rblocks.keys().map(|c| c.x).max().unwrap(),
        s: rblocks.keys().map(|c| c.z).max().unwrap(),
        w: rblocks.keys().map(|c| c.x).min().unwrap(),
    };
    let csize = Pair {
        x: (climits.e - climits.w + 1) as usize,
        z: (climits.s - climits.n + 1) as usize,
    };
    let size = get_iso_size(&csize);

    let mut pixels = vec![0u8; size.x * size.z * 4];

    for cz in (0..CHUNKS_IN_REGION as u8).rev() {
        for cx in (0..CHUNKS_IN_REGION as u8).rev() {
            let c = &Pair { x: cx, z: cz };
            if !rblocks.contains_key(c) {
                continue;
            }

            // println!("Drawing chunk {}, {}", c.x, c.z);
            let acx = (c.x - climits.w) as usize;
            let acz = (c.z - climits.n) as usize;

            let cpx = (acx + csize.z - acz - 1) * ISO_CHUNK_X_MARGIN;
            let cpy = (acx + acz) * ISO_CHUNK_Y_MARGIN;
            let co = cpy * size.x + cpx;

            let ncblocks = Edges {
                n: if cz == 0 {
                    nrblocks.n.get(&Pair { x: cx, z: MAX_CHUNK_IN_REGION as u8 }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                } else {
                    rblocks.get(&Pair { x: cx, z: cz - 1 }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                },
                s: if cz == MAX_CHUNK_IN_REGION as u8 {
                    nrblocks.s.get(&Pair { x: cx, z: 0 }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                } else {
                    rblocks.get(&Pair { x: cx, z: cz + 1 }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                },
                w: if cx == 0 {
                    nrblocks.w.get(&Pair { x: MAX_CHUNK_IN_REGION as u8, z: cz }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                } else {
                    rblocks.get(&Pair { x: cx - 1, z: cz }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                },
                e: if cx == MAX_CHUNK_IN_REGION as u8 {
                    nrblocks.e.get(&Pair { x: 0, z: cz }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                } else {
                    rblocks.get(&Pair { x: cx + 1, z: cz }).unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
                },
            };
            let nclights = Edges {
                n: if cz == 0 {
                    nrlights.n.get(&Pair { x: cx, z: MAX_CHUNK_IN_REGION as u8 }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                } else {
                    rlights.get(&Pair { x: cx, z: cz - 1 }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                },
                s: if cz == MAX_CHUNK_IN_REGION as u8 {
                    nrlights.s.get(&Pair { x: cx, z: 0 }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                } else {
                    rlights.get(&Pair { x: cx, z: cz + 1 }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                },
                w: if cx == 0 {
                    nrlights.w.get(&Pair { x: MAX_CHUNK_IN_REGION as u8, z: cz }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                } else {
                    rlights.get(&Pair { x: cx - 1, z: cz }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                },
                e: if cx == MAX_CHUNK_IN_REGION as u8 {
                    nrlights.e.get(&Pair { x: 0, z: cz }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                } else {
                    rlights.get(&Pair { x: cx + 1, z: cz }).unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
                },
            };

            draw_chunk(&mut pixels, &blocktypes,
                &rblocks[c], &ncblocks, &rlights[c], &nclights, &rbiomes[c],
                &co, &size.x, &night);
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, true)?;

    Ok(())
}
