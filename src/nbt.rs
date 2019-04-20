use std::io::{Error, ErrorKind, Read};

use byteorder::{BigEndian, ReadBytesExt};


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

pub fn scan_compound_tag<R>(reader: &mut R, tag_name: &str) -> Result<Option<()>, Error> where R: Read {
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
            // println!("Found!");
            return Ok(Some(()))
        }

        skip_tag_payload(reader, &id)?;
    }
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

pub fn read_long_array<R>(reader: &mut R) -> Result<Vec<u64>, Error> where R: Read {
    let len = reader.read_u32::<BigEndian>()? as usize;
    // println!("Reading {} longs", len);
    let mut array = vec![0u64; len];
    for i in 0..len {
        array[i] = reader.read_u64::<BigEndian>()?;
    }
    Ok(array)
}
