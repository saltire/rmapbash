use std::ops::Range;

use super::blocktypes::BlockType;
use super::color;
use super::color::{RGBA, BLANK_RGBA};
use super::region::{Block, Chunk};
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
    chunk: &Chunk, co: &isize, width: &usize, cblimits: &Edges<usize>,
    ylimits: &Range<usize>) {
    for bz in (cblimits.n..(cblimits.s + 1)).rev() {
        let biz = bz / BLOCKS_IN_BIOME;

        for bx in (cblimits.w..(cblimits.e + 1)).rev() {
            let bo2 = bz * BLOCKS_IN_CHUNK + bx;
            let bio2 = biz * BIOMES_IN_CHUNK + bx / BLOCKS_IN_BIOME;

            let bpx = (ISO_CHUNK_X_MARGIN as isize +
                (bx as isize - bz as isize - 1) * ISO_BLOCK_X_MARGIN as isize) as usize;
            let bpy2 = (bx + bz) * ISO_BLOCK_Y_MARGIN;

            for by in ylimits.clone().rev() {
                let bo3 = by * BLOCKS_IN_CHUNK_2D + bo2;
                let block = chunk.get_block(&bo3);
                let blocktype = &blocktypes[block.btype as usize];
                if blocktype.empty {
                    continue;
                }

                let bio3 = by / BLOCKS_IN_BIOME * BIOMES_IN_CHUNK_2D + bio2;
                let biome = chunk.data.biomes[bio3] as usize;

                let nblocks = [
                    Some(chunk.get_t_block(&by, &bo3, ylimits.end - 1)),
                    if blocktype.solid || blocktype.waterlogged
                        { Some(chunk.get_s_block(&bz, &bo3)) } else { None },
                    if blocktype.solid || blocktype.waterlogged
                        { Some(chunk.get_e_block(&bx, &bo3)) } else { None },
                ];

                let bcolors = get_block_colors(&blocktypes, &blocktype, &block, &nblocks, biome);
                let wcolors = if blocktype.waterlogged {
                    Some(get_block_colors(&blocktypes, &water_blocktype, &block, &nblocks, biome))
                } else { None };

                let skip_top = nblocks[0].unwrap().btype == block.btype && blocktype.solid;
                let bpy = bpy2 + (MAX_BLOCK_IN_CHUNK_Y - by) * ISO_BLOCK_SIDE_HEIGHT;

                for y in (if skip_top { ISO_BLOCK_Y_MARGIN } else { 0 })..ISO_BLOCK_HEIGHT {
                    for x in 0..ISO_BLOCK_WIDTH {
                        let po = (co + ((bpy + y) * width + bpx + x) as isize) as usize * 4;
                        if pixels[po + 3] == MAX_CHANNEL_VALUE {
                            continue;
                        }
                        let color = if blocktype.waterlogged && blocktype.shape[x][y] == 0 {
                            wcolors.unwrap()[water_blocktype.shape[x][y]]
                        } else {
                            bcolors[blocktype.shape[x][y]]
                        };

                        let pcolor = color::blend_alpha_color(&RGBA {
                            r: pixels[po],
                            g: pixels[po + 1],
                            b: pixels[po + 2],
                            a: pixels[po + 3],
                        }, color);
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

fn get_block_colors<'a>(blocktypes: &'a [BlockType], blocktype: &'a BlockType, block: &Block,
    nblocks: &[Option<Block>], biome: usize)
-> [&'a RGBA; 7] {
    // Create an index of colors corresponding to the digits in the block shape.
    let mut bcolors = [&BLANK_RGBA; 7];

    // Get the block above this one.
    let tblock = nblocks[0].unwrap();

    // Get the base color of the block, using light values from the block above.
    // TODO: are there cases where it's preferable to use the block's own light values?
    bcolors[1] = &blocktype.colors[biome][tblock.slight][tblock.blight][1];
    bcolors[4] = &blocktype.colors[biome][tblock.slight][tblock.blight][4];

    // If the block is solid, use light values from neighboring blocks for side colors.
    // Otherwise use the block's own light values.
    if blocktype.solid {
        // Add a hilight if block to the left has skylight and is not solid or waterlogged.
        let lblock = nblocks[1].unwrap();
        let lblocktype = &blocktypes[lblock.btype as usize];
        let lshade = if lblock.slight > 0 && !lblocktype.solid && !lblocktype.waterlogged
            { 2 } else { 1 };
        bcolors[2] = &blocktype.colors[biome][lblock.slight][lblock.blight][lshade];
        bcolors[5] = &blocktype.colors[biome][lblock.slight][lblock.blight][lshade + 3];

        // Add a shadow if block to the right has skylight and is not solid or waterlogged.
        let rblock = nblocks[2].unwrap();
        let rblocktype = &blocktypes[rblock.btype as usize];
        let rshade = if rblock.slight > 0 && !rblocktype.solid && !rblocktype.waterlogged
            { 3 } else { 1 };
        bcolors[3] = &blocktype.colors[biome][rblock.slight][rblock.blight][rshade];
        bcolors[6] = &blocktype.colors[biome][rblock.slight][rblock.blight][rshade + 3];
    } else {
        bcolors[2] = &blocktype.colors[biome][block.slight][block.blight][2];
        bcolors[5] = &blocktype.colors[biome][block.slight][block.blight][5];

        bcolors[3] = &blocktype.colors[biome][block.slight][block.blight][3];
        bcolors[6] = &blocktype.colors[biome][block.slight][block.blight][6];
    }

    bcolors
}
