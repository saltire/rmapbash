use std::collections::HashMap;
use std::path::Path;

use csv::Reader;

use super::biometypes;
use super::colors;
use super::colors::RGBA;

// #[derive(Debug)]
pub struct BlockType {
    pub name: String,
    pub color: RGBA,
    pub has_biome_colors: bool,
    pub biome_colors: HashMap<u8, RGBA>,
}

pub fn get_block_types() -> Vec<BlockType> {
    let biome_types = biometypes::get_biome_types();

    let csvpath = Path::new("./resources/blocks.csv");
    let mut reader = Reader::from_path(csvpath).unwrap();
    let mut blocktypes = Vec::new();
    for result in reader.records() {
        let row = result.unwrap();

        let foliage = row[5] == *"1";
        let grass = row[5] == *"2";

        let mut blocktype = BlockType {
            name: row[0].to_string(),
            color: RGBA {
                r: row[1].parse().unwrap_or(0),
                g: row[2].parse().unwrap_or(0),
                b: row[3].parse().unwrap_or(0),
                a: row[4].parse().unwrap_or(0),
            },
            has_biome_colors: foliage || grass,
            biome_colors: HashMap::new(),
        };

        if blocktype.has_biome_colors {
            for biome in &biome_types {
                blocktype.biome_colors.insert(biome.id, colors::shade_biome_color(
                    if foliage { &biome.foliage } else { &biome.grass },
                    &blocktype.color));
            }
        }

        blocktypes.push(blocktype);
    }
    blocktypes
}
