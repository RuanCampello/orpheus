use anyhow::Result;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader};
use ratatui::style::Color;
use std::io::Cursor;

pub(crate) struct Rgb(pub u8, pub u8, pub u8);

const ASCII_CHARS: &[u8] = b"#+=-|:. ";
const DEFAULT_PRIMARY: Rgb = Rgb(135, 75, 252);

pub(crate) async fn image_url_to_ascii<'a>(
    url: &'a str,
    window_height: &'a u16,
    window_width: &'a u16,
) -> Result<String> {
    let response = reqwest::get(url).await?;
    let image_data = response.bytes().await?;

    let mut image = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()?
        .decode()?;

    let window_width = *window_width as u32 / 4;
    let window_height = (window_height.saturating_sub(7) as u32) / 2;

    image = image.resize_exact(window_width, window_height, FilterType::Nearest);

    let ascii_string = image_to_ascii(&image);

    Ok(ascii_string)
}

pub(crate) async fn colour_from_image<'u>(url: &'u str) -> Result<Rgb> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    let image = image::load_from_memory(&bytes)?.to_rgb8();

    let mut most_vivid = (0, 0, 0);
    let mut max_vividness = 0.0;

    let calculate_vividness = |r: u8, g: u8, b: u8| -> f32 {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;

        let max_rgb = r.max(g).max(b);
        let min_rgb = r.min(g).min(b);
        let delta = max_rgb - min_rgb;

        let saturation = if max_rgb != 0.0 { delta / max_rgb } else { 0.0 };
        let brightness = (r + g + b) / 3.0;
        saturation * brightness
    };

    let (width, height) = (image.width(), image.height());
    let step = 10;

    for x in (0..width).step_by(step) {
        for y in (0..height).step_by(step) {
            let pixel = image.get_pixel(x, y);
            let (r, g, b) = (pixel[0], pixel[1], pixel[2]);

            let vividness = calculate_vividness(r, g, b);
            if vividness > max_vividness {
                max_vividness = vividness;
                most_vivid = (r, g, b);
            }
        }
    }

    Ok(Rgb(most_vivid.0, most_vivid.1, most_vivid.2))
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

impl From<Rgb> for Color {
    fn from(rgb: Rgb) -> Self {
        Color::Rgb(rgb.0, rgb.1, rgb.2)
    }
}

impl From<&Rgb> for Color {
    fn from(rgb: &Rgb) -> Self {
        Color::Rgb(rgb.0, rgb.1, rgb.2)
    }
}

impl Default for Rgb {
    fn default() -> Self {
        DEFAULT_PRIMARY
    }
}
