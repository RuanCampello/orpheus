use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader};
use std::io::Cursor;

const ASCII_CHARS: &[u8] = b"#+=-|:. ";

pub(crate) fn image_url_to_ascii<'a>(
    url: &'a str,
    window_height: &'a u16,
    window_width: &'a u16,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let image_data = response.bytes()?;

    let mut image = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()?
        .decode()?;

    let window_width = *window_width as u32 / 4;
    let window_height = (window_height.saturating_sub(7) as u32) / 2;

    image = image.resize_exact(window_width, window_height, FilterType::Nearest);

    let ascii_string = image_to_ascii(&image);

    Ok(ascii_string)
}

fn image_to_ascii(image: &DynamicImage) -> String {
    let (width, height) = image.dimensions();
    let mut ascii = String::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            // normalize pixel value 0-255 to 0.0-1.0
            let luma = pixel[0] as f32 / 255.0;
            let ascii_char = brightness_to_ascii(luma);
            ascii.push(ascii_char);
        }
        ascii.push('\n');
    }

    ascii
}

fn brightness_to_ascii(luma: f32) -> char {
    let idx = (luma * (ASCII_CHARS.len() as f32 - 1.0)).round() as usize;
    ASCII_CHARS[idx] as char
}
