use std::path::Path;

use csv::Reader;

use serde::Deserialize;

use super::colors::RGBA;

#[derive(Deserialize)]
struct Row {
    id: u8,
    // name: String,
    fr: Option<u8>,
    fg: Option<u8>,
    fb: Option<u8>,
    gr: Option<u8>,
    gg: Option<u8>,
    gb: Option<u8>,
    wr: Option<u8>,
    wg: Option<u8>,
    wb: Option<u8>,
}

pub struct BiomeType {
    pub id: u8,
    // pub name: String,
    pub foliage: RGBA,
    pub grass: RGBA,
    pub water: RGBA,
}

pub fn get_biome_types() -> Vec<BiomeType> {
    let csvpath = Path::new("./resources/biomes.csv");
    let mut reader = Reader::from_path(csvpath).unwrap();
    let mut biometypes = Vec::new();
    for result in reader.deserialize() {
        let row: Row = result.unwrap();

        biometypes.push(BiomeType {
            id: row.id,
            // name: row.name,
            foliage: RGBA {
                r: row.fr.unwrap_or(0),
                g: row.fg.unwrap_or(0),
                b: row.fb.unwrap_or(0),
                a: 255,
            },
            grass: RGBA {
                r: row.gr.unwrap_or(0),
                g: row.gg.unwrap_or(0),
                b: row.gb.unwrap_or(0),
                a: 255,
            },
            water: RGBA {
                r: row.wr.unwrap_or(255),
                g: row.wg.unwrap_or(255),
                b: row.wb.unwrap_or(255),
                a: 255,
            }
        });
    }
    biometypes
}
