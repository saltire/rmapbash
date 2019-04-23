pub const BLOCKS_IN_CHUNK: usize = 16;

pub const BLOCKS_IN_SECTION_Y: usize = 16;
pub const SECTIONS_IN_CHUNK_Y: usize = 16;
pub const BLOCKS_IN_CHUNK_Y: usize = BLOCKS_IN_SECTION_Y * SECTIONS_IN_CHUNK_Y; // 256

pub const BLOCKS_IN_CHUNK_2D: usize = BLOCKS_IN_CHUNK * BLOCKS_IN_CHUNK; // 256
pub const BLOCKS_IN_CHUNK_3D: usize = BLOCKS_IN_CHUNK_2D * BLOCKS_IN_CHUNK_Y; // 65536

pub const BLOCKS_IN_SECTION_3D: usize = BLOCKS_IN_CHUNK_2D * BLOCKS_IN_SECTION_Y; // 4096
// pub const BLOCKS_IN_CHUNK_3D: usize = BLOCKS_IN_SECTION_3D * SECTIONS_Y; // 65536

pub const CHUNKS_IN_REGION: usize = 32;
pub const CHUNKS_IN_REGION_2D: usize = CHUNKS_IN_REGION * CHUNKS_IN_REGION; // 1024
// pub const BLOCKS_IN_REGION: usize = CHUNKS_IN_REGION * BLOCKS_IN_CHUNK; // 512

pub const SECTOR_SIZE: usize = 4096;


// pixel dimensions for isometric rendering

pub const ISO_BLOCK_WIDTH: usize = 4;
pub const ISO_BLOCK_HEIGHT: usize = 4;
pub const ISO_BLOCK_TOP_HEIGHT: usize = 2;
pub const ISO_BLOCK_SIDE_HEIGHT: usize = 3;
pub const ISO_BLOCK_X_MARGIN: usize = ISO_BLOCK_WIDTH / 2; // 2
pub const ISO_BLOCK_Y_MARGIN: usize = ISO_BLOCK_TOP_HEIGHT / 2; // 1

pub const ISO_CHUNK_WIDTH: usize = ISO_BLOCK_WIDTH * BLOCKS_IN_CHUNK; // 64
pub const ISO_CHUNK_SIDE_HEIGHT: usize = ISO_BLOCK_SIDE_HEIGHT * BLOCKS_IN_CHUNK_Y; // 768
pub const ISO_CHUNK_X_MARGIN: usize = ISO_CHUNK_WIDTH / 2; // 32
pub const ISO_CHUNK_Y_MARGIN: usize = ISO_BLOCK_Y_MARGIN * BLOCKS_IN_CHUNK; // 16

// pub const ISO_REGION_WIDTH: usize = ISO_CHUNK_WIDTH * CHUNKS_IN_REGION; // 2048
// pub const ISO_REGION_SIDE_HEIGHT: usize = ISO_CHUNK_SIDE_HEIGHT; // 768
// pub const ISO_REGION_X_MARGIN: usize = ISO_REGION_WIDTH / 2; // 1024
// pub const ISO_REGION_Y_MARGIN: usize = ISO_CHUNK_Y_MARGIN * CHUNKS_IN_REGION; // 512


pub const LIGHT_LEVELS: usize = 16;
pub const BIOME_ARRAY_SIZE: usize = 170;
