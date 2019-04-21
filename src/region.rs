use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, Error, SeekFrom};
use std::path::Path;
use std::result::Result;

use bitreader::BitReader;

use byteorder::{BigEndian, ReadBytesExt};

use flate2::read::ZlibDecoder;

use super::nbt;
use super::types::Pair;

pub fn read_region_chunks(path: &Path) -> Result<[bool; 1024], Error> {
    let mut file = File::open(path)?;
    let mut chunks = [false; 1024];

    for p in 0..1024 {
        if file.read_u32::<BigEndian>()? > 0 {
            chunks[p] = true;
        }
    }

    Ok(chunks)
}

pub fn read_region_chunk_coords(path: &Path) -> Result<Vec<Pair<u8>>, Error> {
    let mut file = File::open(path)?;
    let mut chunks = vec![];

    for cz in 0..32 {
        for cx in 0..32 {
            if file.read_u32::<BigEndian>()? > 0 {
                chunks.push(Pair { x: cx, z: cz });
            }
        }
    }

    Ok(chunks)
}

fn get_region_chunk_reader(file: &mut File, cx: u8, cz: u8)
-> Result<Option<ZlibDecoder<&mut File>>, Error> {
    let co = (cz as u64 * 32 + cx as u64) * 4;
    file.seek(SeekFrom::Start(co))?;

    let offset = (file.read_u32::<BigEndian>()? >> 8) * 4096;
    if offset > 0 {
        file.seek(SeekFrom::Start(offset as u64))?;
        let size = file.read_u32::<BigEndian>()? as usize;
        file.seek(SeekFrom::Current(1))?;

        let mut reader = ZlibDecoder::new_with_buf(file, vec![0u8; size - 1]);
        nbt::read_tag_header(&mut reader)?;
        Ok(Some(reader))
    }
    else {
        Ok(None)
    }
}

pub fn read_region_chunk_blocks(path: &Path, block_names: &[&str])
-> Result<HashMap<Pair<u8>, [u16; 65536]>, Error> {
    let mut file = File::open(path)?;
    let mut blockmaps = HashMap::new();

    for cz in 0..32 {
        for cx in 0..32 {
            if let Some(mut reader) = get_region_chunk_reader(&mut file, cx, cz)? {
                // println!("Reading chunk {}, {}", cx, cz);

                if nbt::seek_compound_tag_name(&mut reader, "Level")?.is_none() { continue; }
                if nbt::seek_compound_tag_name(&mut reader, "Sections")?.is_none() { continue; }
                let slen = nbt::read_list_length(&mut reader)?;

                let mut blocks = [0u16; 65536];

                for _ in 0..slen {
                    let section = nbt::read_compound_tag_names(&mut reader,
                        vec!["Y", "Palette", "BlockStates"])?;
                    let y = section["Y"].to_u8()?;
                    let palette = section["Palette"].to_list()?;
                    let states = section["BlockStates"].to_long_array()?;

                    let mut pblocks = Vec::with_capacity(palette.len());
                    for ptag in palette {
                        let pblock = ptag.to_hashmap()?;
                        let name = pblock["Name"].to_str()?;
                        pblocks.push(block_names.iter().position(|b| b == &name).unwrap() as u16);
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
                    let so = *y as usize * 4096;

                    let mut br = BitReader::new(&bytes);
                    for i in (0..4096).rev() {
                        blocks[so + i] = pblocks[br.read_u16(bits).unwrap() as usize];
                    }
                }

                blockmaps.insert(Pair { x: cx as u8, z: cz as u8 }, blocks);
            }
        }
    }

    Ok(blockmaps)
}

pub fn read_region_chunk_lightmaps(path: &Path) -> Result<HashMap<Pair<u8>, [u8; 65536]>, Error> {
    let mut file = File::open(path)?;
    let mut lightmaps = HashMap::new();

    for cz in 0..32 {
        for cx in 0..32 {
            if let Some(mut reader) = get_region_chunk_reader(&mut file, cx, cz)? {
                // println!("Reading chunk {}, {}", cx, cz);

                if nbt::seek_compound_tag_name(&mut reader, "Level")?.is_none() { continue; }
                if nbt::seek_compound_tag_name(&mut reader, "Sections")?.is_none() { continue; }
                let slen = nbt::read_list_length(&mut reader)?;

                let mut lights = [0u8; 65536];

                for _ in 0..slen {
                    let section = nbt::read_compound_tag_names(&mut reader,
                        vec!["Y", "BlockLight"])?;
                    let y = section["Y"].to_u8()?;
                    let bytes = section["BlockLight"].to_u8_array()?;
                    let so = *y as usize * 4096;

                    for i in 0..2048 {
                        lights[so + i * 2] = bytes[i] & 0x0f;
                        lights[so + i * 2 + 1] = bytes[i] >> 4;
                    }
                }

                lightmaps.insert(Pair { x: cx as u8, z: cz as u8 }, lights);
            }
        }
    }

    Ok(lightmaps)
}

pub fn read_region_chunk_biomes(path: &Path) -> Result<HashMap<Pair<u8>, [u8; 256]>, Error> {
    let mut file = File::open(path)?;
    let mut biomes = HashMap::new();

    for cz in 0..32 {
        for cx in 0..32 {
            if let Some(mut reader) = get_region_chunk_reader(&mut file, cx, cz)? {
                if nbt::seek_compound_tag_name(&mut reader, "Level")?.is_none() { continue; }
                if nbt::seek_compound_tag_name(&mut reader, "Biomes")?.is_none() { continue; }

                let mut cbiomes = [0u8; 256];
                let cbiomes_vector = nbt::read_u8_array(&mut reader)?;
                if cbiomes_vector.len() == 256 {
                    cbiomes.copy_from_slice(&cbiomes_vector);
                }
                biomes.insert(Pair { x: cx as u8, z: cz as u8 }, cbiomes);
            }
        }
    }

    Ok(biomes)
}

pub fn read_region_chunk_heightmaps(path: &Path) -> Result<HashMap<Pair<u8>, [u8; 256]>, Error> {
    let mut file = File::open(path)?;
    let mut heightmaps = HashMap::new();

    for cz in 0..32 {
        for cx in 0..32 {
            if let Some(mut reader) = get_region_chunk_reader(&mut file, cx, cz)? {
                let root = nbt::read_compound_tag_names(&mut reader, vec!["Level"])?;
                let level = root["Level"].to_hashmap()?;
                let maps = level["Heightmaps"].to_hashmap()?;
                let longs = maps["WORLD_SURFACE"].to_long_array()?;

                let mut bytes = [0u8; 288];
                for i in 0..36 {
                    let long = longs[35 - i];
                    for b in 0..8 {
                        bytes[i * 8 + b] = (long >> ((7 - b) * 8)) as u8;
                    }
                }

                let mut br = BitReader::new(&bytes);
                let mut heights = [0u8; 256];
                for i in (0..256).rev() {
                    heights[i] = br.read_u16(9).unwrap() as u8;
                }

                heightmaps.insert(Pair { x: cx as u8, z: cz as u8 }, heights);
            }
        }
    }

    Ok(heightmaps)
}
