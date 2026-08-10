use mcu_material_color::hct::Hct;
use mcu_material_color::{DynamicScheme, DynamicSchemeOptions, Variant};
use std::path::PathBuf;

use crate::color::roles::RoleExtractor;
use crate::color::types::{ColorScheme, Rgb};
use crate::config::Style;

pub struct SchemeGenerator;

fn variant_for(style: Style) -> Variant {
    match style {
        Style::TonalSpot => Variant::TonalSpot,
        Style::Neutral => Variant::Neutral,
        Style::Vibrant => Variant::Vibrant,
        Style::Expressive => Variant::Expressive,
        Style::Rainbow => Variant::Rainbow,
        Style::FruitSalad => Variant::FruitSalad,
        Style::Monochrome => Variant::Monochrome,
        Style::Fidelity => Variant::Fidelity,
        Style::Content => Variant::Content,
    }
}

fn make_dynamic_scheme(seed_hct: Hct, style: Style, is_dark: bool, contrast: f64) -> DynamicScheme {
    DynamicScheme::new(DynamicSchemeOptions {
        spec_version: Some(mcu_material_color::SpecVersion::Spec2025),
        ..DynamicSchemeOptions::new(seed_hct, variant_for(style), contrast, is_dark)
    })
}

impl SchemeGenerator {
    pub fn generate(seed: &Rgb, style: Style, is_dark: bool, contrast: f64) -> ColorScheme {
        let seed_hct = Hct::from_int(seed.to_argb_u32());
        let dynamic_scheme = make_dynamic_scheme(seed_hct, style, is_dark, contrast);
        let roles = RoleExtractor::extract(&dynamic_scheme);
        ColorScheme {
            wallpaper: String::new(),
            roles,
            seed_colors: None,
            pick: None,
            scheme: Some(dynamic_scheme),
        }
    }

    pub fn generate_with_options(
        seed: &Rgb,
        style: Style,
        is_dark: bool,
        contrast: f64,
        wallpaper: PathBuf,
    ) -> ColorScheme {
        let mut cs = Self::generate(seed, style, is_dark, contrast);
        cs.wallpaper = wallpaper.display().to_string();
        cs
    }

    pub fn blend_styles(
        seed: &Rgb,
        primary_style: Style,
        secondary_style: Style,
        ratio: f64,
        is_dark: bool,
        contrast: f64,
    ) -> ColorScheme {
        let seed_hct = Hct::from_int(seed.to_argb_u32());

        let primary_scheme = make_dynamic_scheme(seed_hct, primary_style, is_dark, contrast);
        let secondary_scheme = make_dynamic_scheme(seed_hct, secondary_style, is_dark, contrast);

        let primary_roles = RoleExtractor::extract(&primary_scheme);
        let secondary_roles = RoleExtractor::extract(&secondary_scheme);

        let blended_roles =
            crate::color::blend::blend_roles(&primary_roles, &secondary_roles, ratio);
        ColorScheme {
            wallpaper: String::new(),
            roles: blended_roles,
            seed_colors: None,
            pick: None,
            scheme: Some(primary_scheme),
        }
    }
}
