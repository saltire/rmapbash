use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, Error, ErrorKind, SeekFrom};
use std::path::Path;
use std::result::Result;

use bitreader::BitReader;

use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;

use nbt::Blob;

use regex::Regex;

fn read_u32(file: &mut File) -> Result<u32, Error> {
    let mut buf = [0; 4];
    file.read(&mut buf)?;
    Ok(((buf[0] as u32) << 24) | ((buf[1] as u32) << 16) | ((buf[2] as u32) << 8) | buf[3] as u32)
}

pub fn read_world_regions(path: &Path) -> Result<Vec<(i32, i32)>, Error> {
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "Directory not found."));
    }

    let region_path = path.join("region");
    if !region_path.is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "No region subdirectory found in path."));
    }

    let mut regions = Vec::new();
    let re = Regex::new(r"^r\.([-\d]+)\.([-\d]+)\.mca$").unwrap();

    for entry in std::fs::read_dir(region_path)? {
        if let Some(filename) = entry?.file_name().to_str() {
            if let Some(caps) = re.captures(filename) {
                let rx = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
                let rz = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();
                regions.push((rx, rz));
            }
        }
    }

    Ok(regions)
}

pub fn read_region_chunks(path: &Path) -> Result<[bool; 1024], Error> {
    let mut file = File::open(path)?;
    let mut chunks = [false; 1024];

    for p in 0..1024 {
        if read_u32(&mut file)? > 0 {
            chunks[p] = true;
        }
    }

    Ok(chunks)
}

pub fn read_region_chunk_coords(path: &Path) -> Result<Vec<(u8, u8)>, Error> {
    let mut file = File::open(path)?;
    let mut chunks: Vec<(u8, u8)> = vec![];

    for cz in 0..32 {
        for cx in 0..32 {
            if read_u32(&mut file)? > 0 {
                chunks.push((cx, cz));
            }
        }
    }

    Ok(chunks)
}

fn read_region_chunk(file: &mut File, cx: u8, cz: u8) -> Result<Option<Blob>, Error> {
    let co = (cz as u64 * 32 + cx as u64) * 4;
    file.seek(SeekFrom::Start(co))?;

    let offset = (read_u32(file)? >> 8) * 4096;
    if offset > 0 {
        file.seek(SeekFrom::Start(offset as u64))?;
        let size = read_u32(file)? as usize;
        let data = vec![0u8; size - 1];
        file.seek(SeekFrom::Current(1))?;

        let mut reader = ZlibDecoder::new_with_buf(file, data);
        Ok(Some(Blob::from_reader(&mut reader)?))
    }
    else {
        Ok(None)
    }
}

pub fn read_region_chunk_heightmaps(path: &Path) -> Result<HashMap<(u8, u8), [u8; 256]>, Error> {
    let mut file = File::open(path)?;
    let mut heightmaps = HashMap::new();

    for cz in 0..32 {
        for cx in 0..32 {
            if let Some(chunk) = read_region_chunk(&mut file, cx, cz)? {
                let value: serde_json::Value = serde_json::to_value(&chunk)?;
                let longs = &value["Level"]["Heightmaps"]["WORLD_SURFACE"];

                let mut bytes = [0u8; 288];
                for i in 0..36 {
                    if let Some(long) = longs[35 - i].as_i64() {
                        for b in 0..8 {
                            bytes[i * 8 + b] = (long >> ((7 - b) * 8)) as u8;
                        }
                    }
                }

                let mut br = BitReader::new(&bytes);
                let mut heights = [0u8; 256];
                for i in (0..256).rev() {
                    heights[i] = br.read_u16(9).unwrap() as u8;
                }

                heightmaps.insert((cx as u8, cz as u8), heights);
            }
        }
    }

    Ok(heightmaps)
}

pub fn read_dat_file(path: &Path) -> Result<(), Error> {
    let file = File::open(path)?;
    let mut reader = GzDecoder::new(file);

    println!("================================= NBT Contents =================================");
    let blob = match Blob::from_reader(&mut reader) {
        Ok(blob) => blob,
        Err(err) => return Err(Error::new(ErrorKind::InvalidData,
            format!("Error reading NBT: {}", err))),
    };
    println!("{}", blob);

    println!("============================== JSON Representation =============================");
    let json = match serde_json::to_string_pretty(&blob) {
        Ok(json) => json,
        Err(err) => return Err(Error::new(ErrorKind::InvalidData,
            format!("Error formatting NBT as JSON: {}", err))),
    };
    println!("{}", json);

    Ok(())
}
