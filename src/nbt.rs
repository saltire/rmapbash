use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read};

use byteorder::{BigEndian, ReadBytesExt};


pub enum Tag {
    Byte(u8),
    Short(u16),
    Int(u32),
    Long(u64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<Tag>),
    Compound(HashMap<String, Tag>),
    IntArray(Vec<u32>),
    LongArray(Vec<u64>),
}

fn read_string<R>(reader: &mut R) -> Result<String, Error> where R: Read {
    let len = reader.read_u16::<BigEndian>()? as usize;
    Ok(if len == 0 {
        "".to_string()
    } else {
        let mut bytes = vec![0u8; len];
        reader.read_exact(&mut bytes)?;
        String::from_utf8(bytes).map_err(|_| Error::new(ErrorKind::InvalidData, "Bad string."))?
    })
}

pub fn read_tag_header<R>(reader: &mut R) -> Result<(u8, String), Error> where R: Read {
    let id = reader.read_u8()?;
    let name = match id {
        0 => "".to_string(),
        _ => read_string(reader)?,
    };
    Ok((id, name))
}

fn skip_tag_payload<R>(reader: &mut R, id: &u8) -> Result<(), Error> where R: Read {
    match id {
        1 => { reader.read_u8()?; },
        2 => { reader.read_u16::<BigEndian>()?; },
        3 => { reader.read_u32::<BigEndian>()?; },
        4 => { reader.read_u64::<BigEndian>()?; },
        5 => { reader.read_u32::<BigEndian>()?; },
        6 => { reader.read_u64::<BigEndian>()?; },
        7 => {
            let len = reader.read_u32::<BigEndian>()? as usize;
            reader.read_exact(&mut vec![0u8; len])?;
        },
        8 => {
            let len = reader.read_u16::<BigEndian>()? as usize;
            reader.read_exact(&mut vec![0u8; len])?;
        },
        9 => {
            let list_id = reader.read_u8()?;
            let len = reader.read_u32::<BigEndian>()? as usize;
            for _ in 0..len {
                skip_tag_payload(reader, &list_id)?;
            }
        },
        10 => {
            loop {
                let (sub_id, _) = read_tag_header(reader)?;
                if sub_id == 0 {
                    break;
                }
                skip_tag_payload(reader, &sub_id)?;
            }
        },
        11 => {
            let len = reader.read_u32::<BigEndian>()?;
            for _ in 0..len {
                reader.read_u32::<BigEndian>()?;
            }
        },
        12 => {
            let len = reader.read_u32::<BigEndian>()?;
            for _ in 0..len {
                reader.read_u64::<BigEndian>()?;
            }
        },
        _ => {},
    };
    Ok(())
}

fn read_tag_payload<R>(reader: &mut R, id: &u8) -> Result<Tag, Error> where R: Read {
    Ok(match id {
        1 => Tag::Byte(reader.read_u8()?),
        2 => Tag::Short(reader.read_u16::<BigEndian>()?),
        3 => Tag::Int(reader.read_u32::<BigEndian>()?),
        4 => Tag::Long(reader.read_u64::<BigEndian>()?),
        5 => Tag::Float(reader.read_f32::<BigEndian>()?),
        6 => Tag::Double(reader.read_f64::<BigEndian>()?),
        7 => {
            let len = reader.read_u32::<BigEndian>()? as usize;
            let mut array = vec![0u8; len];
            reader.read_exact(&mut array)?;
            Tag::ByteArray(array)
        },
        8 => {
            let len = reader.read_u16::<BigEndian>()? as usize;
            if len == 0 {
                Tag::String("".to_string())
            } else {
                let mut bytes = vec![0u8; len];
                reader.read_exact(&mut bytes)?;
                Tag::String(String::from_utf8(bytes)
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Bad string."))?)
            }
        },
        9 => {
            let sub_id = reader.read_u8()?;
            let len = reader.read_u32::<BigEndian>()? as usize;
            let mut list = Vec::new();
            for _ in 0..len {
                list.push(read_tag_payload(reader, &sub_id)?);
            }
            Tag::List(list)
        },
        10 => Tag::Compound(read_compound_tag(reader)?),
        11 => {
            let len = reader.read_u32::<BigEndian>()? as usize;
            let mut array = vec![0u32; len];
            for i in 0..len {
                array[i] = reader.read_u32::<BigEndian>()?;
            }
            Tag::IntArray(array)
        },
        12 => {
            let len = reader.read_u32::<BigEndian>()? as usize;
            let mut array = vec![0u64; len];
            for i in 0..len {
                array[i] = reader.read_u64::<BigEndian>()?;
            }
            Tag::LongArray(array)
        },
        _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid tag id.")),
    })
}

pub fn seek_compound_tag_name<R>(reader: &mut R, tag_name: &str) -> Result<Option<()>, Error> where R: Read {
    loop {
        let (id, name) = read_tag_header(reader)?;
        // println!("Found subtag: {} {}", id, name);

        if id == 0 {
            return Ok(None);
        }
        if id > 12 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid tag id."));
        }
        if name == tag_name {
            return Ok(Some(()))
        }

        skip_tag_payload(reader, &id)?;
    }
}

pub fn read_compound_tag<R>(reader: &mut R) -> Result<HashMap<String, Tag>, Error> where R: Read {
    let mut values = HashMap::new();

    loop {
        let (id, name) = read_tag_header(reader)?;
        // println!("Found subtag: {} {}", id, name);

        if id == 0 {
            break;
        }
        values.insert(name, read_tag_payload(reader, &id)?);
    }

    Ok(values)
}

pub fn read_compound_tag_names<R>(reader: &mut R, names: Vec<&str>)
-> Result<HashMap<String, Tag>, Error> where R: Read {
    let mut values = HashMap::new();

    loop {
        let (id, name) = read_tag_header(reader)?;
        // println!("Found subtag: {} {}", id, name);

        if id == 0 {
            break;
        }
        if names.contains(&name.as_str()) {
            values.insert(name, read_tag_payload(reader, &id)?);
        } else {
            skip_tag_payload(reader, &id)?;
        }
    }

    Ok(values)
}

pub fn read_u8_array<R>(reader: &mut R) -> Result<Vec<u8>, Error> where R: Read {
    let len = reader.read_u32::<BigEndian>()? as usize;
    // println!("Reading {} ints", len);
    let mut array = vec![0u8; len];
    for i in 0..len {
        array[i] = reader.read_u32::<BigEndian>()? as u8;
    }
    Ok(array)
}

pub fn read_list_length<R>(reader: &mut R) -> Result<usize, Error> where R: Read {
    reader.read_u8()?;
    Ok(reader.read_u32::<BigEndian>()? as usize)
}
