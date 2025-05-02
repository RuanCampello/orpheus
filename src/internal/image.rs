use anyhow::Result;
use dashmap::DashMap;
use image::imageops::FilterType;
use image::{DynamicImage, EncodableLayout, GenericImageView, ImageReader};
use once_cell::sync::Lazy;
use ratatui::style::Color;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub(crate) struct Rgb(pub u8, pub u8, pub u8);

type Cache = Lazy<Arc<DashMap<String, (Vec<u8>, Rgb)>>>;
static CACHE: Cache = Lazy::new(|| Arc::new(DashMap::new()));

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

    let (window_width, window_height) = window_width_height(*window_width, *window_height);
    image = image.resize_exact(window_width, window_height, FilterType::Nearest);

    let ascii_string = image_to_ascii(&image);

    Ok(ascii_string)
}

pub(crate) async fn colour_from_image<'u>(url: &'u str) -> Result<Rgb> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    let hash = hash_bytes(bytes.as_bytes());

    if let Some(entry) = CACHE.get(url) {
        let (cached_hash, cached_colour) = entry.value();
        if *cached_hash == hash {
            return Ok(*cached_colour);
        }
    }

    let image = image::load_from_memory(&bytes)?.to_rgb8();

    let mut colour_count: HashMap<(u8, u8, u8), usize> = HashMap::new();
    let mut colour_list = vec![];

    let calculate_saturation = |r: u8, g: u8, b: u8| -> f32 {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;

        let max_rgb = r.max(g).max(b);
        let min_rgb = r.min(g).min(b);
        match max_rgb != 0.0 {
            true => (max_rgb - min_rgb) / max_rgb,
            false => 0.0,
        }
    };

    let calculate_brightness =
        |r: u8, g: u8, b: u8| -> f32 { (r as f32 + g as f32 + b as f32) / (3.0 * 255.0) };

    let is_near_black = |r: u8, g: u8, b: u8| -> bool {
        let brightness = calculate_brightness(r, g, b);
        brightness < 0.15
    };

    let is_near_white = |r: u8, g: u8, b: u8| -> bool {
        let brightness = calculate_brightness(r, g, b);
        brightness > 0.6
    };

    let (width, height) = (image.width(), image.height());
    let step = 10;

    for x in (0..width).step_by(step) {
        for y in (0..height).step_by(step) {
            let pixel = image.get_pixel(x, y);
            let colour = (pixel[0], pixel[1], pixel[2]);

            if is_near_black(colour.0, colour.1, colour.2)
                || is_near_white(colour.0, colour.1, colour.2)
            {
                continue;
            }

            *colour_count.entry(colour).or_insert(0) += 1;
        }
    }

    colour_list.extend(colour_count.keys().copied());

    // colour density for each colour
    let mut density_map: HashMap<(u8, u8, u8), f32> = HashMap::new();

    for &colour in &colour_list {
        let mut total_distance = 0.0;
        for &other_colour in &colour_list {
            if colour != other_colour {
                let dist = ((colour.0 as f32 - other_colour.0 as f32).powi(2)
                    + (colour.1 as f32 - other_colour.1 as f32).powi(2)
                    + (colour.2 as f32 - other_colour.2 as f32).powi(2))
                .sqrt();
                total_distance += dist;
            }
        }
        density_map.insert(colour, 1.0 / (total_distance + 1.0));
    }

    // the best colour is based on (density × saturation) + (brightness × λ) - (near-white penalty)
    let mut best_colour = (0, 0, 0);
    let mut best_score = 0.0;
    let brightness_weight = 0.2;
    let near_white_penalty = 0.3;

    for &colour in &colour_list {
        let saturation = calculate_saturation(colour.0, colour.1, colour.2);
        let brightness = calculate_brightness(colour.0, colour.1, colour.2);
        let density = density_map[&colour];

        let mut score = (density * saturation) + (brightness * brightness_weight);
        if brightness > 0.85 {
            score -= near_white_penalty;
        }

        if score > best_score {
            best_score = score;
            best_colour = colour;
        }
    }

    let colour = Rgb(best_colour.0, best_colour.1, best_colour.2);
    CACHE.insert(url.to_string(), (hash, colour));

    Ok(colour)
}

pub(crate) fn window_width_height(width: u16, height: u16) -> (u32, u32) {
    let window_width = width as u32 / 4;
    let window_height = (height.saturating_sub(7) as u32) / 2;

    (window_width, window_height)
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

fn hash_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
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
