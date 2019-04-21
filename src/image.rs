use std::fs::File;
use std::io::BufWriter;

use png::HasParameters;

use super::types::Pair;

pub fn draw_tiny_map(pixels: &[bool], size: Pair<usize>, file: File)
-> Result<(), png::EncodingError> {
    let len = size.x * size.z;
    println!("Saving map of size {}x{} ({} bytes)", size.x, size.z, len);

    let mut data: Vec<u8> = vec![0; len];
    pixels.iter().enumerate().filter(|(_, v)| **v).for_each(|(i, _)| {
        data[i as usize] = 255;
    });

    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, size.x as u32, size.z as u32);
    encoder
        .set(png::ColorType::Grayscale)
        .set(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;

    Ok(())
}

pub fn draw_block_map(pixels: &[u8], size: Pair<usize>, file: File, color: bool)
-> Result<(), png::EncodingError> {
    let len = size.x * size.z;
    println!("Saving map of size {}x{} ({} bytes)", size.x, size.z, len);

    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, size.x as u32, size.z as u32);
    encoder
        .set(if color { png::ColorType::RGBA } else { png::ColorType::Grayscale })
        .set(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;

    Ok(())
}
