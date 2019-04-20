use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::blocktypes;
use super::colors;
use super::data;
use super::image;
use super::world;

fn is_empty(block: u16) -> bool {
    block == 0 || block == 14 || block == 98 || block == 563
}

fn draw_chunk(pixels: &mut [u8], blocktypes: &Vec<blocktypes::BlockType>,
    cblocks: &[u16], cbiomes: &[u8], co: &usize, width: &usize) {
    for bz in 0..16 {
        for bx in 0..16 {
            let mut color = colors::RGBA { r: 0, g: 0, b: 0, a: 0 };

            for by in (0..256).rev() {
                let bo = by * 256 + bz * 16 + bx;
                if !is_empty(cblocks[bo]) {
                    let blocktype = &blocktypes[cblocks[bo] as usize];
                    let blockcolor = if blocktype.has_biome_colors {
                        &blocktype.biome_colors[&cbiomes[bz * 16 + bx]]
                    } else {
                        &blocktype.color
                    };

                    color = colors::blend_alpha_color(&color, &blockcolor);
                    if color.a == 255 {
                        break;
                    }
                }
            }

            let po = (co + bz * width + bx) * 4;
            pixels[po] = color.r;
            pixels[po + 1] = color.g;
            pixels[po + 2] = color.b;
            pixels[po + 3] = color.a;
        }
    }
}

pub fn draw_world_block_map(worldpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating block map from world dir {}", worldpath.display());

    let world = world::get_world(worldpath)?;

    let blocktypes = blocktypes::get_block_types();
    let blocknames: Vec<&str> = blocktypes.iter().map(|b| &b.name[..]).collect();

    let mut pixels = vec![0u8; world.width * world.height * 4];
    for (rx, rz) in world.regions.iter() {
        let regionpath = worldpath.join("region").join(format!("r.{}.{}.mca", rx, rz));

        println!("Reading blocks for region {}, {}", rx, rz);
        let rblocks = data::read_region_chunk_blocks(regionpath.as_path(), &blocknames)?;
        let rbiomes = data::read_region_chunk_biomes(regionpath.as_path())?;

        println!("Drawing block map for region {}, {}", rx, rz);
        let arx = (rx - world.rmin.x) as usize;
        let arz = (rz - world.rmin.z) as usize;

        for (c, cblocks) in rblocks.iter() {
            let (cx, cz) = c;
            // println!("Drawing chunk {}, {}", cx, cz);
            let acx = arx * 32 + *cx as usize;
            let acz = arz * 32 + *cz as usize;
            let co = (acz - world.margins.n as usize) * 16 * world.width +
                (acx - world.margins.w as usize) * 16;

            draw_chunk(&mut pixels, &blocktypes, cblocks, &rbiomes[c], &co, &world.width);
        }
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, world.width, world.height, file, true)?;

    Ok(())
}

pub fn draw_region_block_map(regionpath: &Path, outpath: &Path) -> Result<(), Box<Error>> {
    println!("Creating block map from region file {}", regionpath.display());

    println!("Getting block types");
    let blocktypes = blocktypes::get_block_types();
    let blocknames: Vec<&str> = blocktypes.iter().map(|b| &b.name[..]).collect();

    println!("Reading blocks");
    let rblocks = data::read_region_chunk_blocks(regionpath, &blocknames)?;
    if rblocks.keys().len() == 0 {
        println!("No chunks in region.");
        return Ok(());
    }

    println!("Reading biomes");
    let rbiomes = data::read_region_chunk_biomes(regionpath)?;

    println!("Drawing block map");
    let climits = world::Edges {
        n: rblocks.keys().map(|(_, z)| z).min().unwrap(),
        e: rblocks.keys().map(|(x, _)| x).max().unwrap(),
        s: rblocks.keys().map(|(_, z)| z).max().unwrap(),
        w: rblocks.keys().map(|(x, _)| x).min().unwrap(),
    };
    let width = (climits.e - climits.w + 1) as usize * 16;
    let height = (climits.s - climits.n + 1) as usize * 16;

    let mut pixels = vec![0u8; width * height * 4];

    for (c, cblocks) in rblocks.iter() {
        let (cx, cz) = c;
        // println!("Drawing chunk {}, {}", cx, cz);
        let acx = (cx - climits.w) as usize;
        let acz = (cz - climits.n) as usize;
        let co = acz * 16 * width + acx * 16;

        draw_chunk(&mut pixels, &blocktypes, cblocks, &rbiomes[c], &co, &width);
    }

    let file = File::create(outpath)?;
    image::draw_block_map(&pixels, width, height, file, true)?;

    Ok(())
}
