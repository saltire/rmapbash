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

        let biome_color_type = row[5].parse().unwrap_or(0);

        let mut blocktype = BlockType {
            name: row[0].to_string(),
            color: RGBA {
                r: row[1].parse().unwrap_or(0),
                g: row[2].parse().unwrap_or(0),
                b: row[3].parse().unwrap_or(0),
                a: row[4].parse().unwrap_or(0),
            },
            has_biome_colors: biome_color_type > 0,
            biome_colors: HashMap::new(),
        };

        if biome_color_type > 0 {
            for biome in &biome_types {
                blocktype.biome_colors.insert(biome.id,
                    if biome_color_type == 1 {
                        colors::shade_biome_color(&blocktype.color, &biome.foliage)
                    } else if biome_color_type == 2 {
                        colors::shade_biome_color(&blocktype.color, &biome.grass)
                    } else {
                        colors::multiply_color(&blocktype.color, &biome.water)
                    });
            }
        }

        blocktypes.push(blocktype);
    }
    blocktypes
}
