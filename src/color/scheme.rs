use mcu_material_color::hct::Hct;
use mcu_material_color::{DynamicScheme, DynamicSchemeOptions, Variant};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::color::roles::RoleExtractor;
use crate::color::types::{ColorScheme, DualRoles, Rgb};
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

fn dual_roles(seed_hct: Hct, style: Style, contrast: f64) -> DualRoles {
    DualRoles {
        light: RoleExtractor::extract(&make_dynamic_scheme(seed_hct, style, false, contrast)),
        dark: RoleExtractor::extract(&make_dynamic_scheme(seed_hct, style, true, contrast)),
    }
}

impl SchemeGenerator {
    pub fn generate(seed: &Rgb, style: Style, contrast: f64) -> ColorScheme {
        let seed_hct = Hct::from_int(seed.to_argb_u32());
        ColorScheme {
            wallpaper: String::new(),
            roles: dual_roles(seed_hct, style, contrast),
            seed_colors: None,
            pick: None,
            custom_colors: HashMap::new(),
        }
    }

    pub fn generate_with_options(
        seed: &Rgb,
        style: Style,
        contrast: f64,
        wallpaper: PathBuf,
    ) -> ColorScheme {
        let mut cs = Self::generate(seed, style, contrast);
        cs.wallpaper = wallpaper.display().to_string();
        cs
    }

    pub fn blend_styles(
        seed: &Rgb,
        primary_style: Style,
        secondary_style: Style,
        ratio: f64,
        contrast: f64,
    ) -> ColorScheme {
        let seed_hct = Hct::from_int(seed.to_argb_u32());

        let blend = |is_dark: bool| {
            let primary_roles =
                RoleExtractor::extract(&make_dynamic_scheme(seed_hct, primary_style, is_dark, contrast));
            let secondary_roles = RoleExtractor::extract(&make_dynamic_scheme(
                seed_hct,
                secondary_style,
                is_dark,
                contrast,
            ));
            crate::color::blend::blend_roles(&primary_roles, &secondary_roles, ratio)
        };

        ColorScheme {
            wallpaper: String::new(),
            roles: DualRoles {
                light: blend(false),
                dark: blend(true),
            },
            seed_colors: None,
            pick: None,
            custom_colors: HashMap::new(),
        }
    }
}
