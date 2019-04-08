use std::fs::File;
use std::io::BufWriter;

use png::HasParameters;

pub fn draw_tiny_map(coords: Vec<(i32, i32)>, file: File) -> Result<(), png::EncodingError> {
    let min_x = coords.iter().map(|(x, _)| x).min().unwrap();
    let max_x = coords.iter().map(|(x, _)| x).max().unwrap();
    let min_z = coords.iter().map(|(_, z)| z).min().unwrap();
    let max_z = coords.iter().map(|(_, z)| z).max().unwrap();

    let width = (max_x - min_x + 1) as u32;
    let height = (max_z - min_z + 1) as u32;
    let size = (width * height * 4) as usize;

    println!("Map size {}x{} with {} blocks ({} bytes)", width, height, coords.len(), size);

    let mut data: Vec<u8> = vec![0; size];
    for (x, z) in coords.iter() {
        let p = (z - min_z) as u32 * width * 4 + (x - min_x) as u32 * 4;
        for c in 0..4 {
            data[(p + c) as usize] = 255;
        }
    }

    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder
        .set(png::ColorType::RGBA)
        .set(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;

    Ok(())
}
