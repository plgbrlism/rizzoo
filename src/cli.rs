use clap::Parser;
use std::path::PathBuf;

use crate::color::types::Rgb;
use crate::config::{Contrast, PreferMode, Style};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "rizzoo",
    version = env!("CARGO_PKG_VERSION"),
    about = "a cross platform color generation tool implementing Material 3 Expressive",
)]
pub struct Cli {
    #[arg(
        short = 'i',
        long = "image",
        value_name = "PATH",
        conflicts_with = "image_url",
        help = "image file / path as the color source"
    )]
    pub image: Option<PathBuf>,

    #[arg(
        short = 'u', long = "image-url", value_name = "URL",
        conflicts_with_all = ["image", "reload_scheme"],
        help = "url of an image to use as the color source"
    )]
    pub image_url: Option<String>,

    #[arg(
        short = 'c', long = "color", value_name = "HEX",
        conflicts_with_all = ["image", "image_url", "reload_scheme"],
        help = "hex color to use as the color source"
    )]
    pub color: Option<String>,

    #[arg(
        short = 'P', long = "pick", value_name = "N",
        value_parser = clap::value_parser!(u8).range(0..5),
        conflicts_with = "prefer",
        help = "explicitly choose the most optimal source color 0-3"
    )]
    pub pick: Option<u8>,

    #[arg(
        short = 'e',
        long = "prefer",
        value_name = "MODE",
        conflicts_with = "pick",
        help = "auto-pick the source color based on..."
    )]
    pub prefer: Option<PreferMode>,

    #[arg(
        long = "open-picker",
        default_value_t = false,
        conflicts_with = "pick",
        help = "open the interactive color picker"
    )]
    pub open_picker: bool,

    #[arg(
        short = 'r',
        long = "render",
        default_value_t = false,
        help = "fill template files with colors"
    )]
    pub render: bool,

    #[arg(
        short = 'o',
        long = "output",
        default_value_t = false,
        help = "write all processed templates to output"
    )]
    pub output: bool,

    #[arg(
        long = "output-only",
        value_name = "APP",
        help = "write specific processed template to output"
    )]
    pub output_to: Option<String>,

    #[arg(
        short = 'p',
        long = "preview",
        default_value_t = false,
        conflicts_with = "silent",
        help = "print palette table"
    )]
    pub preview: bool,

    #[arg(
        short = 'n',
        long = "dry-run",
        default_value_t = false,
        help = "render but skip writing or updating files anywhere"
    )]
    pub dry_run: bool,

    #[arg(
        short = 'q', long = "silent",
        default_value_t = false,
        conflicts_with_all = ["preview"],
        help = "suppress process outputs"
    )]
    pub silent: bool,

    #[arg(
        short = 'S',
        long = "style",
        value_name = "STYLE",
        default_value = "tonal-spot",
        help = "choose a Material 3 style"
    )]
    pub style: Style,

    #[arg(
        long = "light",
        default_value_t = false,
        help = "generate light variant colors"
    )]
    pub light: bool,

    #[arg(
        short = 't',
        long = "contrast",
        value_name = "LEVEL",
        default_value = "standard",
        help = "choose color contrast levels"
    )]
    pub contrast: Contrast,

    #[arg(
        short = 'b',
        long = "blend-style",
        value_name = "STYLE",
        help = "blend with a second Material 3 style"
    )]
    pub blend_style: Option<Style>,

    #[arg(
        long = "blend-ratio",
        value_name = "RATIO",
        default_value_t = 0.5,
        help = "blend ratio 0.1 – 0.9 ( requires --blend-style )"
    )]
    pub blend_ratio: f64,

    #[arg(
        short = 'w',
        long = "wallpaper",
        default_value_t = false,
        help = "set as desktop wallpaper"
    )]
    pub wallpaper: bool,

    #[arg(
        short = 'R',
        long = "reload-scheme",
        default_value_t = false,
        conflicts_with_all = ["image", "image_url"],
        help = "reload last generated color scheme"
    )]
    pub reload_scheme: bool,

    #[arg(
        short = 'W',
        long = "watch",
        default_value_t = false,
        help = "watch relevant files and re-render on change"
    )]
    pub watch: bool,

    #[arg(
        long = "init",
        default_value_t = false,
        help = "generate a default configuration file ( warns if already existing )"
    )]
    pub init: bool,

    #[arg(
        long = "init-overwrite",
        default_value_t = false,
        conflicts_with = "init",
        help = "overwrite configuration file with defaults"
    )]
    pub init_overwrite: bool,
}

impl Cli {
    pub fn validate(&self) -> Result<(), String> {
        if self.init || self.init_overwrite {
            return Ok(());
        }
        let has_image = self.image.is_some() || self.image_url.is_some();
        if !has_image
            && self.color.is_none()
            && !self.reload_scheme
            && !self.output
            && self.output_to.is_none()
            && !self.render
            && !self.preview
        {
            return Err(
                "what to do?\n [-i image, -u url, -c color, -R reload-scheme, -m fill templates, -l output apps, -p preview]".into()
            );
        }
        if self.prefer.is_some() && !has_image {
            return Err("--prefer needs an image (-i or -u)".into());
        }
        if let Some(ref hex) = self.color {
            if Rgb::from_hex(hex).is_none() {
                return Err(format!("invalid format --color '#{hex}'"));
            }
        }
        if self.blend_ratio < 0.1 || self.blend_ratio > 0.9 {
            return Err("--blend-ratio must be between 0.1 and 0.9".into());
        }
        Ok(())
    }
}
