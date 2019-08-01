use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::blocktypes::BlockType;
use super::color;
use super::color::RGBA;
use super::image;
use super::region;
use super::sizes::*;
use super::types::*;
use super::world;

fn draw_chunk(pixels: &mut [u8], blocktypes: &[BlockType], chunk: &region::Chunk, co: &i32,
    cblimits: &Edges<usize>, width: &usize) {
    for bz in (cblimits.n..(cblimits.s + 1)).rev() {
        for bx in (cblimits.w..(cblimits.e + 1)).rev() {
            let bo2 = bz * BLOCKS_IN_CHUNK + bx;

            let bpx = (ISO_CHUNK_X_MARGIN as i16 +
                (bx as i16 - bz as i16 - 1) * ISO_BLOCK_X_MARGIN as i16) as usize;
            let bpy2 = (bx + bz) * ISO_BLOCK_Y_MARGIN;

            let biome = chunk.data.biomes[bo2] as usize;

            for by in (0..BLOCKS_IN_CHUNK_Y).rev() {
                let bo3 = by * BLOCKS_IN_CHUNK_2D + bo2;
                let btype = chunk.data.blocks[bo3];
                let blocktype = &blocktypes[btype as usize];
                if blocktype.empty {
                    continue;
                }

                // Get the base color of the block.
                let tblock = chunk.get_t_block(&by, &bo3);
                let tcolor = &blocktype.colors[biome][tblock.slight][tblock.blight][1];
                let tcolor2 = &blocktype.colors[biome][tblock.slight][tblock.blight][4];
                // Don't draw the top if the block above is the same as this one and solid.
                // This prevents stripes appearing in columns of translucent blocks.
                let skip_top = tblock.btype == btype && blocktype.solid;

                // Add a hilight if block to the left has skylight and is not solid.
                let lblock = chunk.get_s_block(&bz, &bo3);
                let lblocktype = &blocktypes[lblock.btype as usize];
                let lshade = if lblock.slight > 0 && !lblocktype.solid { 2 } else { 1 };
                let lcolor = &blocktype.colors[biome][lblock.slight][lblock.blight][lshade];
                let lcolor2 = &blocktype.colors[biome][lblock.slight][lblock.blight][lshade + 3];

                // Add a shadow if block to the right has skylight and is not solid.
                // let rblock = chunk.get_s_block(&bz, &bo3);
                let rblock = chunk.get_e_block(&bx, &bo3);
                let rblocktype = &blocktypes[rblock.btype as usize];
                let rshade = if rblock.slight > 0 && !rblocktype.solid { 3 } else { 1 };
                let rcolor = &blocktype.colors[biome][rblock.slight][rblock.blight][rshade];
                let rcolor2 = &blocktype.colors[biome][rblock.slight][rblock.blight][rshade + 3];

                // Create an index of colors corresponding to the digits in the block shape.
                let bcolors = [&RGBA::default(),
                    &tcolor, &lcolor, &rcolor, &tcolor2, &lcolor2, &rcolor2];

                let bpy = bpy2 + (MAX_BLOCK_IN_CHUNK_Y - by) * ISO_BLOCK_SIDE_HEIGHT;

                for y in (if skip_top { ISO_BLOCK_Y_MARGIN } else { 0 })..ISO_BLOCK_HEIGHT {
                    for x in 0..ISO_BLOCK_WIDTH {
                        let po = (co + ((bpy + y) * width + bpx + x) as i32) as usize * 4;
                        if pixels[po + 3] == MAX_CHANNEL_VALUE {
                            continue;
                        }

                        let pcolor = color::blend_alpha_color(&RGBA {
                            r: pixels[po],
                            g: pixels[po + 1],
                            b: pixels[po + 2],
                            a: pixels[po + 3],
                        }, bcolors[blocktype.shape[x][y]]);
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

pub fn draw_iso_map(worldpath: &Path, outpath: &Path, blocktypes: &[BlockType],
    blimits: &Option<Edges<i32>>)
-> Result<(), Box<Error>> {
    println!("Creating block map from world dir {}", worldpath.display());

    let world = world::get_world(worldpath, blimits)?;

    let bsize = Pair {
        x: (world.bedges.e - world.bedges.w + 1) as usize,
        z: (world.bedges.s - world.bedges.n + 1) as usize,
    };
    let size = Pair {
        x: (bsize.x + bsize.z) * ISO_BLOCK_X_MARGIN,
        z: (bsize.x + bsize.z) * ISO_BLOCK_Y_MARGIN + ISO_CHUNK_SIDE_HEIGHT,
    };
    let cbcrop = match blimits {
        Some(blimits) => Pair {
            x: block_pos_in_chunk(blimits.w, None),
            z: block_pos_in_chunk(blimits.n, None),
        },
        None => Pair { x: 0, z: 0 },
    };
    let crop = (cbcrop.x + cbcrop.z) * ISO_BLOCK_Y_MARGIN * size.x +
        (cbcrop.x + cbcrop.z) * ISO_BLOCK_X_MARGIN;
    let mut pixels = vec![0u8; size.x * size.z * 4];

    let mut i = 0;
    let len = world.regions.len();

    for rz in (world.redges.n..world.redges.s + 1).rev() {
        for rx in (world.redges.w..world.redges.e + 1).rev() {
            let r = &Pair { x: rx, z: rz };
            if !world.regions.contains_key(&r) {
                continue;
            }

            i += 1;
            println!("Reading block data for region {}, {} ({}/{})", r.x, r.z, i, len);
            if let Some(reg) = region::read_region_data(worldpath, r, blocktypes, blimits)? {
                let chunk_count = reg.chunks.len();
                println!("Drawing block map for region {}, {} ({} chunk{})", r.x, r.z,
                    chunk_count, if chunk_count == 1 { "" } else { "s" });

                let arc = Pair {
                    x: r.x * CHUNKS_IN_REGION as i32 - world.cedges.w,
                    z: r.z * CHUNKS_IN_REGION as i32 - world.cedges.n,
                };

                for cz in (0..CHUNKS_IN_REGION).rev() {
                    for cx in (0..CHUNKS_IN_REGION).rev() {
                        let c = &Pair { x: cx, z: cz };
                        if let Some(chunk) = reg.get_chunk(c) {
                            // println!("Drawing chunk {}, {}", c.x, c.z);
                            let wc = Pair {
                                x: r.x * CHUNKS_IN_REGION as i32 + c.x as i32,
                                z: r.z * CHUNKS_IN_REGION as i32 + c.z as i32,
                            };
                            let cblimits = match blimits {
                                Some(blimits) => Edges {
                                    n: block_pos_in_chunk(blimits.n, Some(wc.z)),
                                    e: block_pos_in_chunk(blimits.e, Some(wc.x)),
                                    s: block_pos_in_chunk(blimits.s, Some(wc.z)),
                                    w: block_pos_in_chunk(blimits.w, Some(wc.x)),
                                },
                                None => Edges::<usize>::full(BLOCKS_IN_CHUNK),
                            };

                            let ac = Pair {
                                x: (arc.x + c.x as i32) as usize,
                                z: (arc.z + c.z as i32) as usize,
                            };
                            let cp = Pair {
                                x: (ac.x + world.csize.z - ac.z - 1) * ISO_CHUNK_X_MARGIN,
                                z: (ac.x + ac.z) * ISO_CHUNK_Y_MARGIN,
                            };
                            let co = (cp.z * size.x + cp.x) as i32 - crop as i32;

                            draw_chunk(&mut pixels, blocktypes, &chunk, &co, &cblimits, &size.x);
                        }
                    }
                }
            } else {
                println!("No data in region.");
            }
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, size, file, true)?;

    Ok(())
}
