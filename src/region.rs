use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, Error, SeekFrom};
use std::path::{Path, PathBuf};
use std::result::Result;

use bitreader::BitReader;

use byteorder::{BigEndian, ReadBytesExt};

use flate2::read::ZlibDecoder;

use regex::Regex;

use super::blocktypes::BlockType;
use super::nbt;
use super::sizes::*;
use super::types::*;
use super::world::World;

pub struct Block {
    pub btype: u16,
    pub slight: usize,
    pub blight: usize,
}

pub struct ChunkData {
    pub blocks: [u16; BLOCKS_IN_CHUNK_3D],
    pub lights: [u8; BLOCKS_IN_CHUNK_3D],
    pub biomes: [u8; BLOCKS_IN_CHUNK_2D],
}

static EMPTY_CHUNK: ChunkData = ChunkData {
    blocks: [0u16; BLOCKS_IN_CHUNK_3D],
    lights: [0u8; BLOCKS_IN_CHUNK_3D],
    biomes: [0u8; BLOCKS_IN_CHUNK_2D],
};

pub struct Chunk<'a> {
    pub data: &'a ChunkData,
    pub ndata: Edges<&'a ChunkData>,
}

impl<'a> Chunk<'a> {
    pub fn get_t_block(&self, by: &usize, bo3: &usize, ymax: usize) -> Block {
        let btype = match *by {
            y if y == ymax => 0,
            _ => self.data.blocks[bo3 + BLOCKS_IN_CHUNK_2D],
        };
        let light = match *by {
            y if y == ymax => MAX_LIGHT_LEVEL,
            _ => self.data.lights[bo3 + BLOCKS_IN_CHUNK_2D],
        };

        Block {
            btype,
            slight: (light & 0x0f) as usize,
            blight: ((light & 0xf0) >> 4) as usize,
        }
    }

    pub fn get_n_block(&self, bz: &usize, bo3: &usize) -> Block {
        let btype = match *bz {
            0 => self.ndata.n.blocks[bo3 + MAX_BLOCK_IN_CHUNK * BLOCKS_IN_CHUNK],
            _ => self.data.blocks[bo3 - BLOCKS_IN_CHUNK],
        };
        let light = match *bz {
            0 => self.ndata.n.lights[bo3 + MAX_BLOCK_IN_CHUNK * BLOCKS_IN_CHUNK],
            _ => self.data.lights[bo3 - BLOCKS_IN_CHUNK],
        };

        Block {
            btype,
            slight: (light & 0x0f) as usize,
            blight: ((light & 0xf0) >> 4) as usize,
        }
    }

    pub fn get_e_block(&self, bx: &usize, bo3: &usize) -> Block {
        let btype = match *bx {
            MAX_BLOCK_IN_CHUNK => self.ndata.e.blocks[bo3 - MAX_BLOCK_IN_CHUNK],
            _ => self.data.blocks[*bo3 + 1],
        };
        let light = match *bx {
            MAX_BLOCK_IN_CHUNK => self.ndata.e.lights[bo3 - MAX_BLOCK_IN_CHUNK],
            _ => self.data.lights[*bo3 + 1],
        };

        Block {
            btype,
            slight: (light & 0x0f) as usize,
            blight: ((light & 0xf0) >> 4) as usize,
        }
    }

    pub fn get_s_block(&self, bz: &usize, bo3: &usize) -> Block {
        let btype = match *bz {
            MAX_BLOCK_IN_CHUNK => self.ndata.s.blocks[bo3 - MAX_BLOCK_IN_CHUNK * BLOCKS_IN_CHUNK],
            _ => self.data.blocks[bo3 + BLOCKS_IN_CHUNK],
        };
        let light = match *bz {
            MAX_BLOCK_IN_CHUNK => self.ndata.s.lights[bo3 - MAX_BLOCK_IN_CHUNK * BLOCKS_IN_CHUNK],
            _ => self.data.lights[bo3 + BLOCKS_IN_CHUNK],
        };

        Block {
            btype,
            slight: (light & 0x0f) as usize,
            blight: ((light & 0xf0) >> 4) as usize,
        }
    }

    pub fn get_w_block(&self, bx: &usize, bo3: &usize) -> Block {
        let btype = match *bx {
            0 => self.ndata.w.blocks[bo3 + MAX_BLOCK_IN_CHUNK],
            _ => self.data.blocks[*bo3 - 1],
        };
        let light = match *bx {
            0 => self.ndata.w.lights[bo3 + MAX_BLOCK_IN_CHUNK],
            _ => self.data.lights[*bo3 - 1],
        };

        Block {
            btype,
            slight: (light & 0x0f) as usize,
            blight: ((light & 0xf0) >> 4) as usize,
        }
    }
}

pub struct RegionData {
    pub chunks: HashMap<Pair<usize>, ChunkData>,
    pub nchunks: Edges<HashMap<Pair<usize>, ChunkData>>,
}

impl RegionData {
    pub fn get_chunk(&self, c: &Pair<usize>) -> Option<Chunk> {
        self.chunks.get(c).and_then(|data| Some(Chunk {
            data,
            ndata: Edges {
                n: match c.z {
                    0 => self.nchunks.n.get(&Pair { x: c.x, z: MAX_CHUNK_IN_REGION }),
                    _ => self.chunks.get(&Pair { x: c.x, z: c.z - 1 }),
                }.unwrap_or_else(|| &EMPTY_CHUNK),
                e: match c.x {
                    MAX_CHUNK_IN_REGION => self.nchunks.e.get(&Pair { x: 0, z: c.z }),
                    _ => self.chunks.get(&Pair { x: c.x + 1, z: c.z }),
                }.unwrap_or_else(|| &EMPTY_CHUNK),
                s: match c.z {
                    MAX_CHUNK_IN_REGION => self.nchunks.s.get(&Pair { x: c.x, z: 0 }),
                    _ => self.chunks.get(&Pair { x: c.x, z: c.z + 1 }),
                }.unwrap_or_else(|| &EMPTY_CHUNK),
                w: match c.x {
                    0 => self.nchunks.w.get(&Pair { x: MAX_CHUNK_IN_REGION, z: c.z }),
                    _ => self.chunks.get(&Pair { x: c.x - 1, z: c.z }),
                }.unwrap_or_else(|| &EMPTY_CHUNK),
            }
        }))
    }
}

fn get_path_from_coords<'a>(worldpath: &Path, r: &Pair<isize>) -> PathBuf {
    worldpath.join("region").join(format!("r.{}.{}.mca", r.x, r.z))
}

pub fn get_coords_from_path(path_str: &str) -> Option<Pair<isize>> {
    Regex::new(r"r\.([-\d]+)\.([-\d]+)\.mca$").unwrap()
        .captures(path_str)
        .map(|caps| Pair {
            x: caps.get(1).unwrap().as_str().parse::<isize>().unwrap(),
            z: caps.get(2).unwrap().as_str().parse::<isize>().unwrap(),
        })
}

pub fn read_region_chunk_coords(path: &Path, rclimits: &Option<Edges<usize>>)
-> Result<Vec<Pair<usize>>, Error> {
    let mut file = File::open(path)?;
    let mut chunks = vec![];

    let climits = rclimits.unwrap_or_else(|| Edges::<usize>::full(CHUNKS_IN_REGION));

    for cz in climits.n..(climits.s + 1) {
        for cx in climits.w..(climits.e + 1) {
            if let Some(mut reader) = get_region_chunk_reader(&mut file, cx, cz)? {
                if nbt::seek_compound_tag_name(&mut reader, "Level")?.is_some() &&
                    nbt::seek_compound_tag_name(&mut reader, "Sections")?.is_some() &&
                    nbt::read_list_length(&mut reader)? > 0 {
                    chunks.push(Pair { x: cx, z: cz });
                }
            }
        }
    }

    Ok(chunks)
}

fn get_region_chunk_reader(file: &mut File, cx: usize, cz: usize)
-> Result<Option<ZlibDecoder<&mut File>>, Error> {
    let co = (cz * CHUNKS_IN_REGION + cx) * 4;
    file.seek(SeekFrom::Start(co as u64))?;

    let offset = (file.read_u32::<BigEndian>()? >> 8) as usize * SECTOR_SIZE;
    Ok(if offset == 0 { None } else {
        file.seek(SeekFrom::Start(offset as u64))?;
        let size = file.read_u32::<BigEndian>()? as usize;
        file.seek(SeekFrom::Current(1))?;

        let mut reader = ZlibDecoder::new_with_buf(file, vec![0u8; size - 1]);
        nbt::read_tag_header(&mut reader)?;

        Some(reader)
    })
}

pub fn read_region_chunk<R>(reader: &mut R, blocktypes: &[BlockType])
-> Result<Option<ChunkData>, Error> where R: Read {
    if nbt::seek_compound_tag_name(reader, "Level")?.is_none() {
        return Ok(None);
    }

    let mut chunk = ChunkData {
        blocks: [0u16; BLOCKS_IN_CHUNK_3D],
        lights: [0x0fu8; BLOCKS_IN_CHUNK_3D], // Default to full skylight and zero blocklight.
        biomes: [0u8; BLOCKS_IN_CHUNK_2D],
    };
    // Default to zero skylight and blocklight for sections that exist but don't contain those tags.
    let light_bytes_default = vec![0u8; BLOCKS_IN_SECTION_3D / 2];

    while let Some((id, name)) = nbt::seek_compound_tag_names(reader, vec!["Sections", "Biomes"])? {
        if name == "Sections" {
            let slen = nbt::read_list_length(reader)?;
            if slen == 0 {
                return Ok(None);
            }

            for _ in 0..slen {
                let section = nbt::read_compound_tag_names(reader,
                    vec!["Y", "Palette", "BlockStates", "BlockLight", "SkyLight"])?;
                let y = *section["Y"].to_u8()? as usize;
                if y > MAX_SECTION_IN_CHUNK_Y {
                    continue;
                }
                let so = y * BLOCKS_IN_SECTION_3D;

                // Read blocks.
                if section.contains_key("BlockStates") {
                    let palette = section["Palette"].to_list()?;
                    let states = section["BlockStates"].to_long_array()?;

                    let mut pblocks = Vec::with_capacity(palette.len());
                    for ptag in palette {
                        let pblock = ptag.to_hashmap()?;
                        let name = pblock["Name"].to_str()?;
                        let mut props = HashMap::new();
                        if pblock.contains_key("Properties") {
                            for (k, v) in pblock["Properties"].to_hashmap()?.iter() {
                                props.insert(k, v.to_str()?);
                            }
                        }
                        pblocks.push(blocktypes.iter()
                            .position(|b| {
                                b.name == name &&
                                b.state.iter().all(|(k, v)| props[k] == v)
                            })
                            .unwrap() as u16);
                    }

                    // BlockStates is an array of i64 representing 4096 blocks,
                    // but we have to check the array length to determine the # of bits per block.
                    let len = states.len();
                    let mut bytes = vec![0u8; len * 8];
                    for i in 0..len {
                        let long = states[len - i - 1];
                        for b in 0..8 {
                            bytes[i * 8 + b] = (long >> ((7 - b) * 8)) as u8;
                        }
                    }
                    let bits = (len / 64) as u8;

                    let mut br = BitReader::new(&bytes);
                    for i in (0..BLOCKS_IN_SECTION_3D).rev() {
                        chunk.blocks[so + i] = pblocks[br.read_u16(bits).unwrap() as usize];
                    }
                }

                // Read lights.
                let sbytes = section.get("SkyLight")
                    .map_or(&light_bytes_default, |tag| tag.to_u8_array().unwrap());
                let bbytes = section.get("BlockLight")
                    .map_or(&light_bytes_default, |tag| tag.to_u8_array().unwrap());

                for i in 0..(BLOCKS_IN_SECTION_3D / 2) {
                    // The bottom half of each byte, moving blocklight to the top.
                    chunk.lights[so + i * 2] = ((bbytes[i] & 0x0f) << 4) | (sbytes[i] & 0x0f);
                    // The top half of each byte, moving skylight to the bottom.
                    chunk.lights[so + i * 2 + 1] = (bbytes[i] & 0xf0) | (sbytes[i] >> 4);
                }
            }
        } else if name == "Biomes" {
            // Read biomes and cast to an array of bytes.
            let cbiomes = match nbt::read_tag_payload(reader, &id)? {
                nbt::Tag::ByteArray(cbiomes_tag) => cbiomes_tag,
                nbt::Tag::IntArray(cbiomes_tag) => cbiomes_tag.iter().map(|v| *v as u8).collect(),
                _ => continue,
            };

            if cbiomes.len() == BLOCKS_IN_CHUNK_2D {
                chunk.biomes.copy_from_slice(&cbiomes);
            }
        }
    }

    Ok(Some(chunk))
}

fn read_region_chunk_data(path: &Path, rclimits: &Edges<usize>, blocktypes: &[BlockType])
-> Result<HashMap<Pair<usize>, ChunkData>, Box<Error>> {
    let mut chunks = HashMap::new();

    if path.exists() {
        let mut file = File::open(path)?;

        for cz in rclimits.n..(rclimits.s + 1) {
            for cx in rclimits.w..(rclimits.e + 1) {
                if let Some(mut reader) = get_region_chunk_reader(&mut file, cx, cz)? {
                    if let Some(chunk) = read_region_chunk(&mut reader, blocktypes)? {
                        chunks.insert(Pair { x: cx, z: cz }, chunk);
                    }
                }
            }
        }
    }

    Ok(chunks)
}

pub fn read_region_data(world: &World, r: &Pair<isize>, blocktypes: &[BlockType])
-> Result<Option<RegionData>, Box<Error>> {
    let regionpath = get_path_from_coords(world.path, &r);
    if !regionpath.exists() {
        return Ok(None);
    }

    let rclimits = Edges {
        n: chunk_pos_in_region(block_to_chunk(world.bedges.n), Some(r.z)),
        e: chunk_pos_in_region(block_to_chunk(world.bedges.e), Some(r.x)),
        s: chunk_pos_in_region(block_to_chunk(world.bedges.s), Some(r.z)),
        w: chunk_pos_in_region(block_to_chunk(world.bedges.w), Some(r.x)),
    };

    let chunks = read_region_chunk_data(&regionpath, &rclimits, blocktypes)?;
    if chunks.len() == 0 {
        return Ok(None);
    }

    let npaths = Edges {
        n: get_path_from_coords(world.path, &Pair { x: r.x, z: r.z - 1 }),
        e: get_path_from_coords(world.path, &Pair { x: r.x + 1, z: r.z }),
        s: get_path_from_coords(world.path, &Pair { x: r.x, z: r.z + 1 }),
        w: get_path_from_coords(world.path, &Pair { x: r.x - 1, z: r.z }),
    };
    let nrclimits = Edges {
        n: Edges { n: MAX_CHUNK_IN_REGION, e: rclimits.e, s: MAX_CHUNK_IN_REGION, w: rclimits.w },
        e: Edges { n: rclimits.n, e: 0, s: rclimits.s, w: 0 },
        s: Edges { n: 0, e: rclimits.e, s: 0, w: rclimits.w },
        w: Edges { n: rclimits.n, e: MAX_CHUNK_IN_REGION, s: rclimits.s, w: MAX_CHUNK_IN_REGION },
    };

    Ok(Some(RegionData {
        chunks,
        nchunks: Edges {
            n: read_region_chunk_data(&npaths.n, &nrclimits.n, blocktypes)?,
            e: read_region_chunk_data(&npaths.e, &nrclimits.e, blocktypes)?,
            s: read_region_chunk_data(&npaths.s, &nrclimits.s, blocktypes)?,
            w: read_region_chunk_data(&npaths.w, &nrclimits.w, blocktypes)?,
        },
    }))
}
