use std::path::Path;

use csv::Reader;

// #[derive(Debug)]
pub struct BlockType {
    pub name: String,
    pub color: (u8, u8, u8, u8),
    pub foliage: bool,
    pub grass: bool,
}

pub fn get_block_types() -> Vec<BlockType> {
    let csvpath = Path::new("./resources/blocks.csv");
    let mut reader = Reader::from_path(csvpath).unwrap();
    let mut blocks = Vec::new();
    for result in reader.records() {
        let blocktype = result.unwrap();
        blocks.push(BlockType {
            name: blocktype[0].to_string(),
            color: (
                blocktype[1].parse().unwrap_or(0),
                blocktype[2].parse().unwrap_or(0),
                blocktype[3].parse().unwrap_or(0),
                blocktype[4].parse().unwrap_or(0),
            ),
            foliage: blocktype[5] == *"1",
            grass: blocktype[6] == *"1",
        });
    }
    blocks
}
