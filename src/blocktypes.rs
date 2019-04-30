use std::collections::HashMap;
use std::path::Path;

use csv::Reader;

use serde::Deserialize;

use super::biometypes;
use super::color;
use super::color::RGBA;
use super::sizes::*;

#[derive(Deserialize)]
struct Row {
    name: String,
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
    a: Option<u8>,
    copy: Option<String>,
    biome: Option<u8>,
}

pub struct BlockType {
    pub name: String,
    pub colors: [[RGBA; LIGHT_LEVELS]; BIOME_ARRAY_SIZE],
    // pub alpha: u8,
}

impl PartialEq for BlockType {
    fn eq(&self, other: &BlockType) -> bool {
        self.name == other.name
    }
}

pub fn get_block_types() -> Vec<BlockType> {
    let biome_types = biometypes::get_biome_types();

    let csvpath = Path::new("./resources/blocks.csv");
    let mut reader = Reader::from_path(csvpath).unwrap();
    let mut blocktypes = Vec::new();

    let rows: Vec<Row> = reader.deserialize().map(|res| res.unwrap()).collect();

    // Build a map of explicitly defined colours, indexed by the defining block type,
    // as a lookup for blocks that reference other blocks' colours.
    let mut colors: HashMap<&str, RGBA> = HashMap::new();
    for row in &rows {
        if row.copy.is_none() {
            colors.insert(&row.name, RGBA {
                r: row.r.unwrap_or(0),
                g: row.g.unwrap_or(0),
                b: row.b.unwrap_or(0),
                a: row.a.unwrap_or(0),
            });
        }
    }

    // Now iterate through all the block types and build their colours.
    for row in &rows {
        let block_color = row.copy.clone()
            .and_then(|c| colors.get(c.as_str()))
            .map_or_else(
                || RGBA {
                    r: row.r.unwrap_or(0),
                    g: row.g.unwrap_or(0),
                    b: row.b.unwrap_or(0),
                    a: row.a.unwrap_or(0),
                },
                |c| c.clone());

        let biome_color_type = row.biome.unwrap_or(0);

        let mut block_colors = [[RGBA::default(); LIGHT_LEVELS]; BIOME_ARRAY_SIZE];
        for biome in &biome_types {
            let biome_color = match biome_color_type {
                1 => color::shade_biome_color(&block_color, &biome.foliage),
                2 => color::shade_biome_color(&block_color, &biome.grass),
                3 => color::multiply_color(&block_color, &biome.water),
                _ => block_color.clone(),
            };

            for ll in 0..LIGHT_LEVELS {
                block_colors[biome.id as usize][ll] =
                    color::set_light_level(&biome_color, &(ll as u8));
            }

        }

        blocktypes.push(BlockType {
            name: format!("minecraft:{}", row.name),
            colors: block_colors,
            // alpha: block_color.a,
        });
    }

    blocktypes
}
