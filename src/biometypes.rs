use std::path::Path;

use csv::Reader;

use super::colors::RGBA;

// #[derive(Debug)]
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
    for result in reader.records() {
        let row = result.unwrap();

        biometypes.push(BiomeType {
            id: row[0].parse().unwrap(),
            // name: row[1].to_string(),
            foliage: RGBA {
                r: row[2].parse().unwrap_or(0),
                g: row[3].parse().unwrap_or(0),
                b: row[4].parse().unwrap_or(0),
                a: 255,
            },
            grass: RGBA {
                r: row[5].parse().unwrap_or(0),
                g: row[6].parse().unwrap_or(0),
                b: row[7].parse().unwrap_or(0),
                a: 255,
            },
            water: RGBA {
                r: row[8].parse().unwrap_or(255),
                g: row[8].parse().unwrap_or(255),
                b: row[8].parse().unwrap_or(255),
                a: 255,
            }
        });
    }
    biometypes
}
