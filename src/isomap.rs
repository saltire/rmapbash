use std::ops::Range;

use super::blocktypes::BlockType;
use super::color;
use super::color::RGBA;
use super::region;
use super::sizes::*;
use super::types::*;
use super::world::World;

pub fn get_size(world: &World) -> Pair<usize> {
    Pair {
        x: (world.bsize.x + world.bsize.z) * ISO_BLOCK_X_MARGIN,
        z: (world.bsize.x + world.bsize.z) * ISO_BLOCK_Y_MARGIN + ISO_CHUNK_SIDE_HEIGHT,
    }
}

pub fn get_crop(world: &World, size: &Pair<usize>) -> usize {
    let cbcrop = Edges {
        n: block_pos_in_chunk(world.bedges.n, None),
        e: MAX_BLOCK_IN_CHUNK - block_pos_in_chunk(world.bedges.e, None),
        s: MAX_BLOCK_IN_CHUNK - block_pos_in_chunk(world.bedges.s, None),
        w: block_pos_in_chunk(world.bedges.w, None),
    };
    (cbcrop.w + cbcrop.n) * ISO_BLOCK_Y_MARGIN * size.x +
        (cbcrop.w + cbcrop.s) * ISO_BLOCK_X_MARGIN
}

pub fn get_chunk_pixel(world: &World, arc: &Pair<isize>, c: &Pair<usize>) -> Pair<usize> {
    let ac = Pair {
        x: (arc.x + c.x as isize) as usize,
        z: (arc.z + c.z as isize) as usize,
    };
    Pair {
        x: (ac.x + world.csize.z - ac.z - 1) * ISO_CHUNK_X_MARGIN,
        z: (ac.x + ac.z) * ISO_CHUNK_Y_MARGIN,
    }
}

pub fn draw_chunk(pixels: &mut [u8], blocktypes: &[BlockType], water_blocktype: &BlockType,
    chunk: &region::Chunk, co: &isize, width: &usize, cblimits: &Edges<usize>,
    ylimits: &Range<usize>) {
    let blank_color = RGBA::default();

    for bz in (cblimits.n..(cblimits.s + 1)).rev() {
        for bx in (cblimits.w..(cblimits.e + 1)).rev() {
            let bo2 = bz * BLOCKS_IN_CHUNK + bx;

            let bpx = (ISO_CHUNK_X_MARGIN as isize +
                (bx as isize - bz as isize - 1) * ISO_BLOCK_X_MARGIN as isize) as usize;
            let bpy2 = (bx + bz) * ISO_BLOCK_Y_MARGIN;

            let biome = chunk.data.biomes[bo2] as usize;

            for by in ylimits.clone().rev() {
                let bo3 = by * BLOCKS_IN_CHUNK_2D + bo2;
                let btype = chunk.data.blocks[bo3];
                let blocktype = &blocktypes[btype as usize];
                if blocktype.empty {
                    continue;
                }

                // Create an index of colors corresponding to the digits in the block shape.
                let mut bcolors = [&blank_color; 8];

                // Get the block above this one.
                let tblock = chunk.get_t_block(&by, &bo3, ylimits.end - 1);
                // Don't draw the top if the block above is the same as this one and solid.
                // This prevents stripes appearing in columns of translucent blocks.
                let skip_top = tblock.btype == btype && blocktype.solid;

                // Get the base color of the block, using light values from the block above.
                // TODO: are there cases where it's preferable to use the block's own light values?
                bcolors[1] = &blocktype.colors[biome][tblock.slight][tblock.blight][1];
                bcolors[4] = &blocktype.colors[biome][tblock.slight][tblock.blight][4];
                // Get the base color of the water block type, in case block is waterlogged.
                bcolors[7] = &water_blocktype.colors[biome][tblock.slight][tblock.blight][1];

                // If the block is solid, use light values from neighboring blocks for side colors.
                // Otherwise use the block's own light values.
                if blocktype.solid {
                    // Add a hilight if block to the left has skylight and is not solid.
                    let lblock = chunk.get_s_block(&bz, &bo3);
                    let lblocktype = &blocktypes[lblock.btype as usize];
                    let lshade = if lblock.slight > 0 && !lblocktype.solid { 2 } else { 1 };
                    bcolors[2] = &blocktype.colors[biome][lblock.slight][lblock.blight][lshade];
                    bcolors[5] = &blocktype.colors[biome][lblock.slight][lblock.blight][lshade + 3];

                    // Add a shadow if block to the right has skylight and is not solid.
                    let rblock = chunk.get_e_block(&bx, &bo3);
                    let rblocktype = &blocktypes[rblock.btype as usize];
                    let rshade = if rblock.slight > 0 && !rblocktype.solid { 3 } else { 1 };
                    bcolors[3] = &blocktype.colors[biome][rblock.slight][rblock.blight][rshade];
                    bcolors[6] = &blocktype.colors[biome][rblock.slight][rblock.blight][rshade + 3];
                } else {
                    let light = chunk.data.lights[bo3];
                    let slight = (light & 0x0f) as usize;
                    let blight = ((light & 0xf0) >> 4) as usize;

                    bcolors[2] = &blocktype.colors[biome][slight][blight][2];
                    bcolors[5] = &blocktype.colors[biome][slight][blight][5];

                    bcolors[3] = &blocktype.colors[biome][slight][blight][3];
                    bcolors[6] = &blocktype.colors[biome][slight][blight][6];
                }

                let bpy = bpy2 + (MAX_BLOCK_IN_CHUNK_Y - by) * ISO_BLOCK_SIDE_HEIGHT;

                for y in (if skip_top { ISO_BLOCK_Y_MARGIN } else { 0 })..ISO_BLOCK_HEIGHT {
                    for x in 0..ISO_BLOCK_WIDTH {
                        let po = (co + ((bpy + y) * width + bpx + x) as isize) as usize * 4;
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
