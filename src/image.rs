use std::fs::File;
use std::io::BufWriter;

use png::HasParameters;

pub fn draw_tiny_map(pixels: &[bool], width: u32, height: u32, file: File)
-> Result<(), png::EncodingError> {
    let size = (width * height * 4) as usize;
    println!("Map size {}x{} ({} bytes)", width, height, size);

    let mut data: Vec<u8> = vec![0; size];
    pixels.iter().enumerate().filter(|(_, v)| **v).for_each(|(i, _)| {
        for c in 0..4 {
            data[(i * 4 + c) as usize] = 255;
        }
    });

    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder
        .set(png::ColorType::RGBA)
        .set(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;

    Ok(())
}

pub fn draw_height_map(pixels: &[u8], width: u32, height: u32, file: File)
-> Result<(), png::EncodingError> {
    let size = (width * height * 4) as usize;
    println!("Map size {}x{} ({} bytes)", width, height, size);

    let mut data: Vec<u8> = vec![0; size];
    pixels.iter().enumerate().for_each(|(i, value)| {
        for c in 0..3 {
            data[(i * 4 + c) as usize] = *value;
        }
        data[(i * 4 + 3) as usize] = 255;
    });

    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder
        .set(png::ColorType::RGBA)
        .set(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;

    Ok(())
}
