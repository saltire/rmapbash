use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, Error, SeekFrom};
use std::path::{Path, PathBuf};
use std::result::Result;

use bitreader::BitReader;

use byteorder::{BigEndian, ReadBytesExt};

use flate2::read::ZlibDecoder;

use super::nbt;
use super::sizes::*;
use super::types::*;

fn get_path_from_coords<'a>(worldpath: &Path, r: &Pair<i32>) -> PathBuf {
    worldpath.join("region").join(format!("r.{}.{}.mca", r.x, r.z))
}

fn get_region_chunk_reader(file: &mut File, cx: usize, cz: usize)
-> Result<Option<ZlibDecoder<&mut File>>, Error> {
    let co = (cz * CHUNKS_IN_REGION + cx) * 4;
    file.seek(SeekFrom::Start(co as u64))?;

    let offset = (file.read_u32::<BigEndian>()? >> 8) as usize * SECTOR_SIZE;
    Ok(if offset > 0 {
        file.seek(SeekFrom::Start(offset as u64))?;
        let size = file.read_u32::<BigEndian>()? as usize;
        file.seek(SeekFrom::Current(1))?;

        let mut reader = ZlibDecoder::new_with_buf(file, vec![0u8; size - 1]);
        nbt::read_tag_header(&mut reader)?;
        Some(reader)
    } else {
        None
    })
}

pub struct RegionChunk {
    pub blocks: [u16; BLOCKS_IN_CHUNK_3D],
    pub lights: [u8; BLOCKS_IN_CHUNK_3D],
    pub biomes: [u8; BLOCKS_IN_CHUNK_2D],
}

pub struct Region {
    pub chunks: HashMap<Pair<usize>, RegionChunk>,
}

pub fn read_region_chunk<R>(reader: &mut R, blocknames: &[&str])
-> Result<Option<RegionChunk>, Error> where R: Read {
    // println!("Reading chunk {}, {}", cx, cz);

    if nbt::seek_compound_tag_name(reader, "Level")?.is_none() {
        return Ok(None);
    }

    let mut chunk = RegionChunk {
        blocks: [0u16; BLOCKS_IN_CHUNK_3D],
        lights: [0x0fu8; BLOCKS_IN_CHUNK_3D],
        biomes: [0u8; BLOCKS_IN_CHUNK_2D],
    };

    while let Some(tag_name) = nbt::seek_compound_tag_names(reader, vec!["Sections", "Biomes"])? {
        if tag_name == "Sections" {
            let slen = nbt::read_list_length(reader)?;

            let light_bytes_default = vec![0u8; BLOCKS_IN_SECTION_3D / 2];

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
                        pblocks.push(blocknames.iter().position(|b| b == &name).unwrap() as u16);
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
                if section.contains_key("BlockLight") || section.contains_key("SkyLight") {
                    let bbytes = section.get("BlockLight")
                        .map_or(&light_bytes_default, |tag| tag.to_u8_array().unwrap());
                    let sbytes = section.get("SkyLight")
                        .map_or(&light_bytes_default, |tag| tag.to_u8_array().unwrap());

                    for i in 0..(BLOCKS_IN_SECTION_3D / 2) {
                        // The bottom half of each byte, moving blocklight to the top.
                        chunk.lights[so + i * 2] = ((bbytes[i] & 0x0f) << 4) | (sbytes[i] & 0x0f);
                        // The top half of each byte, moving skylight to the bottom.
                        chunk.lights[so + i * 2 + 1] = (bbytes[i] & 0xf0) | (sbytes[i] >> 4);
                    }
                }
            }
        } else if tag_name == "Biomes" {
            // Read biomes.
            let cbiomes = nbt::read_u8_array(reader)?;
            if cbiomes.len() == BLOCKS_IN_CHUNK_2D {
                chunk.biomes.copy_from_slice(&cbiomes);
            }
        }
    }

    Ok(Some(chunk))
}

#[allow(dead_code)]
pub fn read_region_chunk_data(worldpath: &Path, r: &Pair<i32>, blocknames: &[&str])
-> Result<Region, Box<Error>> {
    let regionpath = get_path_from_coords(worldpath, &r);

    let mut region = Region {
        chunks: HashMap::new(),
    };
    if !regionpath.exists() {
        return Ok(region);
    }
    let mut file = File::open(regionpath)?;

    let margins = Edges::default();

    for cz in margins.n..(CHUNKS_IN_REGION - margins.s) {
        for cx in margins.w..(CHUNKS_IN_REGION - margins.e) {
            if let Some(mut reader) = get_region_chunk_reader(&mut file, cx, cz)? {
                if let Some(chunk) = read_region_chunk(&mut reader, blocknames)? {
                    region.chunks.insert(Pair { x: cx, z: cz }, chunk);
                }
            }
        }
    }

    Ok(region)
}
