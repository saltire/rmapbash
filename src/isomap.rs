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

struct Chunk<'a> {
    blocks: &'a [u16; BLOCKS_IN_CHUNK_3D],
    // nblocks: Edges<&'a [u16; BLOCKS_IN_CHUNK_3D]>,
    lights: &'a [u8; BLOCKS_IN_CHUNK_3D],
    nlights: Edges<&'a [u8; BLOCKS_IN_CHUNK_3D]>,
    biomes: &'a [u8; BLOCKS_IN_CHUNK_2D],
}

fn get_iso_size(csize: &Pair<usize>) -> Pair<usize> {
    Pair {
        x: (csize.x + csize.z) * ISO_CHUNK_X_MARGIN,
        z: (csize.x + csize.z) * ISO_CHUNK_Y_MARGIN + ISO_CHUNK_SIDE_HEIGHT,
    }
}

fn get_chunk_data<'a>(reg: &'a region::Region, c: &'a Pair<u8>) -> Chunk<'a> {
    Chunk {
        blocks: &reg.blocks[c],
        // nblocks: Edges {
        //     n: if c.z == 0 {
        //         reg.nblocks.n.get(&Pair { x: c.x, z: MAX_CHUNK_IN_REGION as u8 })
        //             .unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
        //     } else {
        //         reg.blocks.get(&Pair { x: c.x, z: c.z - 1 })
        //             .unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
        //     },
        //     s: if c.z == MAX_CHUNK_IN_REGION as u8 {
        //         reg.nblocks.s.get(&Pair { x: c.x, z: 0 })
        //             .unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
        //     } else {
        //         reg.blocks.get(&Pair { x: c.x, z: c.z + 1 })
        //             .unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
        //     },
        //     w: if c.x == 0 {
        //         reg.nblocks.w.get(&Pair { x: MAX_CHUNK_IN_REGION as u8, z: c.z })
        //             .unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
        //     } else {
        //         reg.blocks.get(&Pair { x: c.x - 1, z: c.z })
        //             .unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
        //     },
        //     e: if c.x == MAX_CHUNK_IN_REGION as u8 {
        //         reg.nblocks.e.get(&Pair { x: 0, z: c.z })
        //             .unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
        //     } else {
        //         reg.blocks.get(&Pair { x: c.x + 1, z: c.z })
        //             .unwrap_or(&[0u16; BLOCKS_IN_CHUNK_3D])
        //     },
        // },
        lights: &reg.lights[c],
        nlights: Edges {
            n: if c.z == 0 {
                reg.nlights.n.get(&Pair { x: c.x, z: MAX_CHUNK_IN_REGION as u8 })
                    .unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
            } else {
                reg.lights.get(&Pair { x: c.x, z: c.z - 1 })
                    .unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
            },
            s: if c.z == MAX_CHUNK_IN_REGION as u8 {
                reg.nlights.s.get(&Pair { x: c.x, z: 0 })
                    .unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
            } else {
                reg.lights.get(&Pair { x: c.x, z: c.z + 1 })
                    .unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
            },
            w: if c.x == 0 {
                reg.nlights.w.get(&Pair { x: MAX_CHUNK_IN_REGION as u8, z: c.z })
                    .unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
            } else {
                reg.lights.get(&Pair { x: c.x - 1, z: c.z })
                    .unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
            },
            e: if c.x == MAX_CHUNK_IN_REGION as u8 {
                reg.nlights.e.get(&Pair { x: 0, z: c.z })
                    .unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
            } else {
                reg.lights.get(&Pair { x: c.x + 1, z: c.z })
                    .unwrap_or(&[0u8; BLOCKS_IN_CHUNK_3D])
            },
        },
        biomes: &reg.biomes[c],
    }
}

fn draw_chunk(pixels: &mut [u8], blocktypes: &Vec<blocktypes::BlockType>,
    chunk: &Chunk, co: &usize, width: &usize, night: &bool) {
    for bz in (0..BLOCKS_IN_CHUNK).rev() {
        for bx in (0..BLOCKS_IN_CHUNK).rev() {
            let bo2 = bz * BLOCKS_IN_CHUNK + bx;

            let bpx = (ISO_CHUNK_X_MARGIN as i16 +
                (bx as i16 - bz as i16 - 1) * ISO_BLOCK_X_MARGIN as i16) as usize;
            let bpy2 = (bx + bz) * ISO_BLOCK_Y_MARGIN;

            for by in (0..BLOCKS_IN_CHUNK_Y).rev() {
                let bo3 = by * BLOCKS_IN_CHUNK_2D + bo2;
                if chunk.blocks[bo3] == 0 {
                    continue;
                }

                let blocktype = &blocktypes[chunk.blocks[bo3] as usize];

                let tlight = if *night && by < MAX_BLOCK_IN_CHUNK_Y {
                    chunk.lights[bo3 + BLOCKS_IN_CHUNK_2D]
                } else {
                    MAX_LIGHT_LEVEL
                };
                let llight = if *night {
                    if bz == MAX_BLOCK_IN_CHUNK {
                        chunk.nlights.s[bo3 - MAX_BLOCK_IN_CHUNK * BLOCKS_IN_CHUNK]
                    } else {
                        chunk.lights[bo3 + BLOCKS_IN_CHUNK]
                    }
                } else {
                    MAX_LIGHT_LEVEL
                };
                let rlight = if *night {
                    if bx == MAX_BLOCK_IN_CHUNK {
                        chunk.nlights.e[bo3 - MAX_BLOCK_IN_CHUNK]
                    } else {
                        chunk.lights[bo3 + 1]
                    }
                } else {
                    MAX_LIGHT_LEVEL
                };

                let tcolor = &blocktype.colors[chunk.biomes[bo2] as usize][tlight as usize];
                if tcolor.a == 0 {
                    continue;
                }
                let lcolor = &blocktype.colors[chunk.biomes[bo2] as usize][llight as usize];
                let rcolor = &blocktype.colors[chunk.biomes[bo2] as usize][rlight as usize];

                let bpy = bpy2 + (MAX_BLOCK_IN_CHUNK_Y - by) * ISO_BLOCK_SIDE_HEIGHT;

                // Don't draw the top if the block above is the same as this one.
                // This prevents stripes appearing in columns of translucent blocks.
                let skip_top = by < MAX_BLOCK_IN_CHUNK_Y &&
                    chunk.blocks[bo3] == chunk.blocks[bo3 + BLOCKS_IN_CHUNK_2D];

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
            println!("Reading block data for region {}, {} ({}/{})", r.x, r.z, i, len);
            let reg = region::read_region_data(worldpath, &r, &blocknames)?;

            println!("Drawing block map for region {}, {}", r.x, r.z);
            let arx = (r.x - world.rlimits.w) as usize;
            let arz = (r.z - world.rlimits.n) as usize;

            for cz in (0..CHUNKS_IN_REGION as u8).rev() {
                for cx in (0..CHUNKS_IN_REGION as u8).rev() {
                    let c = &Pair { x: cx, z: cz };
                    if !reg.blocks.contains_key(c) {
                        continue;
                    }

                    // println!("Drawing chunk {}, {}", c.x, c.z);
                    let acx = arx * CHUNKS_IN_REGION + c.x as usize - world.margins.w;
                    let acz = arz * CHUNKS_IN_REGION + c.z as usize - world.margins.n;

                    let cpx = (acx + csize.z - acz - 1) * ISO_CHUNK_X_MARGIN;
                    let cpy = (acx + acz) * ISO_CHUNK_Y_MARGIN;
                    let co = cpy * size.x + cpx;

                    let chunk = get_chunk_data(&reg, &c);
                    draw_chunk(&mut pixels, &blocktypes, &chunk, &co, &size.x, &night);
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

    println!("Reading block data for region {}, {}", r.x, r.z);
    let reg = region::read_region_data(worldpath, &r, &blocknames)?;

    println!("Drawing block map");
    let climits = Edges {
        n: reg.blocks.keys().map(|c| c.z).min().unwrap(),
        e: reg.blocks.keys().map(|c| c.x).max().unwrap(),
        s: reg.blocks.keys().map(|c| c.z).max().unwrap(),
        w: reg.blocks.keys().map(|c| c.x).min().unwrap(),
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
            if !reg.blocks.contains_key(c) {
                continue;
            }

            // println!("Drawing chunk {}, {}", c.x, c.z);
            let acx = (c.x - climits.w) as usize;
            let acz = (c.z - climits.n) as usize;

            let cpx = (acx + csize.z - acz - 1) * ISO_CHUNK_X_MARGIN;
            let cpy = (acx + acz) * ISO_CHUNK_Y_MARGIN;
            let co = cpy * size.x + cpx;

            let chunk = get_chunk_data(&reg, &c);
            draw_chunk(&mut pixels, &blocktypes, &chunk, &co, &size.x, &night);
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, true)?;

    Ok(())
}
