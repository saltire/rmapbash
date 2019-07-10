use std::collections::HashMap;
use std::path::Path;

use csv::Reader;

use serde::Deserialize;

use super::biometypes;
use super::color;
use super::color::{RGBA, RGB};
use super::sizes::*;

#[derive(Deserialize)]
struct BlockRow {
    name: String,
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
    a: Option<u8>,
    r2: Option<u8>,
    g2: Option<u8>,
    b2: Option<u8>,
    a2: Option<u8>,
    biome: Option<u8>,
    state: String,
    shape: String,
}

#[derive(Deserialize)]
struct LightRow {
    sky: Option<usize>,
    block:Option<usize>,
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
}

pub struct BlockType {
    pub name: String,
    pub colors: [[[[RGBA; 7]; LIGHT_LEVELS]; LIGHT_LEVELS]; BIOME_ARRAY_SIZE],
    pub state: HashMap<String, String>,
    pub shape: [[usize; ISO_BLOCK_WIDTH]; ISO_BLOCK_HEIGHT],
    pub solid: bool,
    pub empty: bool,
}

impl PartialEq for BlockType {
    fn eq(&self, other: &BlockType) -> bool {
        self.name == other.name
    }
}

const HILIGHT_SHADOW_AMOUNT_DAY: f64 = 0.125;
const HILIGHT_SHADOW_AMOUNT_NIGHT: f64 = 0.05;

pub fn get_block_types(lighting: &str) -> Vec<BlockType> {
    let mut blocktypes = Vec::new();

    let biome_types = biometypes::get_biome_types();

    let lightfile = format!("./resources/light/{}.csv", lighting);
    let lightpath = Path::new(&lightfile);
    let mut lightreader = Reader::from_path(lightpath).unwrap();
    let lightrows: Vec<LightRow> = lightreader.deserialize().map(|res| res.unwrap()).collect();
    let mut light = [[RGB::default(); LIGHT_LEVELS]; LIGHT_LEVELS];
    for row in &lightrows {
        light[row.sky.unwrap()][row.block.unwrap()] = RGB {
            r: row.r.unwrap(),
            g: row.g.unwrap(),
            b: row.b.unwrap(),
        };
    }
    let hilight_shadow_amount = if lighting == "night" { HILIGHT_SHADOW_AMOUNT_NIGHT }
        else { HILIGHT_SHADOW_AMOUNT_DAY };

    let blockpath = Path::new("./resources/blocks.csv");
    let mut blockreader = Reader::from_path(blockpath).unwrap();
    let blockrows: Vec<BlockRow> = blockreader.deserialize().map(|res| res.unwrap()).collect();
    for row in &blockrows {
        let block_color = RGBA {
            r: row.r.unwrap_or(0),
            g: row.g.unwrap_or(0),
            b: row.b.unwrap_or(0),
            a: row.a.unwrap_or(0),
        };
        let block_color2 = RGBA {
            r: row.r2.unwrap_or(0),
            g: row.g2.unwrap_or(0),
            b: row.b2.unwrap_or(0),
            a: row.a2.unwrap_or(0),
        };
        let biome_color_type = row.biome.unwrap_or(0);

        let mut blockcolors = [[[[RGBA::default(); 7]; LIGHT_LEVELS]; LIGHT_LEVELS]; BIOME_ARRAY_SIZE];
        for biome in &biome_types {
            // Apply biome color to primary color only.
            let biome_id = biome.id as usize;
            let biome_color = match biome_color_type {
                1 => color::shade_biome_color(&block_color, &biome.foliage),
                2 => color::shade_biome_color(&block_color, &biome.grass),
                3 => color::multiply_color(&block_color, &biome.water),
                _ => block_color.clone(),
            };

            for sl in 0..LIGHT_LEVELS {
                for bl in 0..LIGHT_LEVELS {
                    let lit_block_color = color::set_light_color(&biome_color, &light[sl][bl]);
                    blockcolors[biome_id][sl][bl][1] = lit_block_color;
                    blockcolors[biome_id][sl][bl][2] =
                        color::adjust_brightness(&lit_block_color, &hilight_shadow_amount);
                    blockcolors[biome_id][sl][bl][3] =
                        color::adjust_brightness(&lit_block_color, &-hilight_shadow_amount);

                    if block_color2.a > 0 {
                        let lit_block_color2 = color::set_light_color(&block_color2, &light[sl][bl]);
                        blockcolors[biome_id][sl][bl][4] = lit_block_color2;
                        blockcolors[biome_id][sl][bl][5] =
                            color::adjust_brightness(&lit_block_color2, &hilight_shadow_amount);
                        blockcolors[biome_id][sl][bl][6] =
                            color::adjust_brightness(&lit_block_color2, &-hilight_shadow_amount);
                    }
                }
            }
        }

        // Convert state string into a key-value hashmap.
        let mut state = HashMap::new();
        for pair in row.state.split("&") {
            if pair != "" {
                let mut kv = pair.split("=");
                state.insert(kv.next().unwrap().to_string(), kv.next().unwrap().to_string());
            }
        }

        // Convert shape string into a nested X,Y array.
        let mut shape = [[0usize; ISO_BLOCK_HEIGHT]; ISO_BLOCK_WIDTH];
        let mut chars = row.shape.as_str().chars();
        for y in 0..ISO_BLOCK_HEIGHT {
            for x in 0..ISO_BLOCK_WIDTH {
                shape[x][y] = chars.next().unwrap_or('0').to_digit(10).unwrap() as usize;
            }
        }

        blocktypes.push(BlockType {
            name: format!("minecraft:{}", row.name),
            colors: blockcolors,
            shape: shape,
            state: state,
            solid: row.shape.find('0').is_none(),
            empty: row.shape == "" || row.shape == "0000000000000000",
        });
    }

    blocktypes
}
