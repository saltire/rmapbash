use std::ops::Range;

use super::blocktypes::BlockType;
use super::color;
use super::region;
use super::sizes::*;
use super::types::*;
use super::world::World;


pub fn get_size(world: &World) -> Pair<usize> {
    world.bsize.clone()
}

pub fn get_crop(world: &World, size: &Pair<usize>) -> usize {
    let cbcrop = Pair {
        x: block_pos_in_chunk(world.bedges.w, None),
        z: block_pos_in_chunk(world.bedges.n, None),
    };
    cbcrop.z * size.x + cbcrop.x
}

pub fn get_chunk_pixel(arc: &Pair<isize>, c: &Pair<usize>) -> Pair<usize> {
    Pair {
        x: (arc.x + c.x as isize) as usize * BLOCKS_IN_CHUNK,
        z: (arc.z + c.z as isize) as usize * BLOCKS_IN_CHUNK,
    }
}

pub fn draw_chunk(pixels: &mut [u8], blocktypes: &[BlockType], chunk: &region::Chunk, co: &isize,
    width: &usize, cblimits: &Edges<usize>, ylimits: &Range<usize>) {
    for bz in cblimits.n..(cblimits.s + 1) {
        for bx in cblimits.w..(cblimits.e + 1) {
            let po = (co + (bz * width + bx) as isize) as usize * 4;
            let color = get_block_color(bx, bz, blocktypes, chunk, ylimits);
            pixels[po] = color.r;
            pixels[po + 1] = color.g;
            pixels[po + 2] = color.b;
            pixels[po + 3] = color.a;
        }
    }
}

fn get_block_color(bx: usize, bz: usize, blocktypes: &[BlockType], chunk: &region::Chunk,
    ylimits: &Range<usize>)
-> color::RGBA {
    let mut color = color::RGBA { r: 0, g: 0, b: 0, a: 0 };

    let bo2 = bz * BLOCKS_IN_CHUNK + bx;
    let bio2 = bz / BLOCKS_IN_BIOME * BIOMES_IN_CHUNK + bx / BLOCKS_IN_BIOME;

    for by in ylimits.clone().rev() {
        let bo3 = by * BLOCKS_IN_CHUNK_2D + bo2;
        let btype = chunk.data.blocks[bo3];
        let blocktype = &blocktypes[btype as usize];
        if blocktype.empty {
            continue;
        }

        let bio3 = by / BLOCKS_IN_BIOME * BIOMES_IN_CHUNK_2D + bio2;
        let biome = chunk.data.biomes[bio3] as usize;

        let tblock = chunk.get_t_block(&by, &bo3, ylimits.end - 1);
        let nblocks = Edges {
            n: chunk.get_n_block(&bz, &bo3),
            e: chunk.get_e_block(&bx, &bo3),
            s: chunk.get_s_block(&bz, &bo3),
            w: chunk.get_w_block(&bx, &bo3),
        };
        let is_edge = Edges {
            n: nblocks.n.slight > 0 && !blocktypes[nblocks.n.btype as usize].solid,
            s: nblocks.s.slight > 0 && !blocktypes[nblocks.s.btype as usize].solid,
            e: nblocks.e.slight > 0 && !blocktypes[nblocks.e.btype as usize].solid,
            w: nblocks.w.slight > 0 && !blocktypes[nblocks.w.btype as usize].solid,
        };
        let shade = match (is_edge.n || is_edge.w, is_edge.e || is_edge.s) {
            (true, false) => 2,
            (false, true) => 3,
            _ => 1,
        };
        let blockcolor = &blocktype.colors[biome][tblock.slight][tblock.blight][shade];

        color = color::blend_alpha_color(&color, blockcolor);
        if color.a == MAX_CHANNEL_VALUE {
            break;
        }
    }

    color
}
