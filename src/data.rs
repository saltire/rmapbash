use std::fs::File;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::result::Result;

use flate2::read::GzDecoder;

use nbt::Blob;

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
