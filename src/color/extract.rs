use crate::color::types::Rgb;
use crate::config::PreferMode;
use crate::error::LRatio;
use image::DynamicImage;
use mcu_material_color::quantize::QuantizerCelebi;
use mcu_material_color::score::Score;
use std::path::Path;

const SAMPLE_SIZE: u32 = 200;

pub struct ColorExtractor;

impl ColorExtractor {
    pub fn open(path: &Path) -> Result<DynamicImage, LRatio> {
        let file = std::fs::File::open(path).map_err(|e| LRatio::ImageDecode(e.to_string()))?;
        let reader = std::io::BufReader::new(file);
        image::ImageReader::new(reader)
            .with_guessed_format()
            .map_err(|e| LRatio::ImageDecode(e.to_string()))?
            .decode()
            .map_err(|e| LRatio::ImageDecode(e.to_string()))
    }

    pub fn sample_pixels(img: &DynamicImage) -> Vec<u32> {
        let resized = img.thumbnail_exact(SAMPLE_SIZE, SAMPLE_SIZE);
        resized
            .to_rgb8()
            .pixels()
            .map(|p| (255u32 << 24) | ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | (p[2] as u32))
            .collect()
    }

    pub fn extract(path: &Path) -> Result<Vec<Rgb>, LRatio> {
        let img = Self::open(path)?;
        let argb_pixels = Self::sample_pixels(&img);
        if argb_pixels.is_empty() {
            return Err(LRatio::NoColors);
        }
        let result = QuantizerCelebi::quantize(&argb_pixels, 128);
        let ranked = Score::score(&result);
        if ranked.is_empty() {
            return Err(LRatio::NoColors);
        }
        let colors: Vec<Rgb> = ranked.iter().map(|a| Rgb::from_argb_u32(*a)).collect();
        Ok(colors)
    }

    pub fn select_by_preference(colors: &[Rgb], prefer: PreferMode) -> usize {
        match prefer {
            PreferMode::Darkness => colors
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.luminance_wcag().total_cmp(&b.luminance_wcag()))
                .map(|(i, _)| i)
                .unwrap_or(0),
            PreferMode::Lightness => colors
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.luminance_wcag().total_cmp(&b.luminance_wcag()))
                .map(|(i, _)| i)
                .unwrap_or(0),
            PreferMode::Saturation => colors
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.saturation().total_cmp(&b.saturation()))
                .map(|(i, _)| i)
                .unwrap_or(0),
        }
    }
}
