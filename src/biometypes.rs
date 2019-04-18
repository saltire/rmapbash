use std::path::Path;

use csv::Reader;

// #[derive(Debug)]
pub struct BiomeType {
    pub id: u8,
    // pub name: String,
    pub foliage: (u8, u8, u8, u8),
    pub grass: (u8, u8, u8, u8),
}

pub fn get_biome_types() -> Vec<BiomeType> {
    let csvpath = Path::new("./resources/biomes.csv");
    let mut reader = Reader::from_path(csvpath).unwrap();
    let mut biomes = Vec::new();
    for result in reader.records() {
        let row = result.unwrap();
        biomes.push(BiomeType {
            id: row[0].parse().unwrap(),
            // name: row[1].to_string(),
            foliage: (
                row[2].parse().unwrap(),
                row[3].parse().unwrap(),
                row[4].parse().unwrap(),
                255,
            ),
            grass: (
                row[5].parse().unwrap(),
                row[6].parse().unwrap(),
                row[7].parse().unwrap(),
                255,
            ),
        });
    }
    biomes
}
