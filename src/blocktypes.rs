use std::collections::HashMap;
use std::path::Path;

use csv::Reader;

use serde::Deserialize;

use super::biometypes;
use super::colors;
use super::colors::RGBA;

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
    pub color: RGBA,
    pub has_biome_colors: bool,
    pub biome_colors: HashMap<u8, RGBA>,
}

pub fn get_block_types() -> Vec<BlockType> {
    let biome_types = biometypes::get_biome_types();

    let csvpath = Path::new("./resources/blocks.csv");
    let mut reader = Reader::from_path(csvpath).unwrap();
    let mut blocktypes = Vec::new();

    let rows: Vec<Row> = reader.deserialize().map(|res| res.unwrap()).collect();

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

    for row in &rows {
        let biome_color_type = row.biome.unwrap_or(0);

        let mut blocktype = BlockType {
            name: row.name.to_string(),
            color: row.copy.clone()
                .and_then(|c| colors.get(c.as_str()))
                .map_or_else(
                    || RGBA {
                        r: row.r.unwrap_or(0),
                        g: row.g.unwrap_or(0),
                        b: row.b.unwrap_or(0),
                        a: row.a.unwrap_or(0),
                    },
                    |c| c.clone()),
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
