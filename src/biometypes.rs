use std::path::Path;

use csv::Reader;

use super::colors::RGBA;

// #[derive(Debug)]
pub struct BiomeType {
    pub id: u8,
    // pub name: String,
    pub foliage: RGBA,
    pub grass: RGBA,
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
                r: row[2].parse().unwrap(),
                g: row[3].parse().unwrap(),
                b: row[4].parse().unwrap(),
                a: 255,
            },
            grass: RGBA {
                r: row[5].parse().unwrap(),
                g: row[6].parse().unwrap(),
                b: row[7].parse().unwrap(),
                a: 255,
            },
        });
    }
    biometypes
}
