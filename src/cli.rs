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
        help = "image path as color source"
    )]
    pub image: Option<PathBuf>,

    #[arg(
        short = 'u', long = "image-url", value_name = "URL",
        conflicts_with_all = ["image", "restore"],
        help = "url link of an image as the color source"
    )]
    pub image_url: Option<String>,

    #[arg(
        short = 'c', long = "color", value_name = "HEX",
        conflicts_with_all = ["image", "image_url", "restore"],
        help = "a color of your choice in hex format"
    )]
    pub color: Option<String>,

    #[arg(
        short = 'R',
        long = "restore-wallpaper",
        default_value_t = false,
        conflicts_with_all = ["image", "image-url"],
        help = "restore last wallpaper"
    )]
    pub restore: bool,

    #[arg(
        short = 'p',
        long = "preview",
        default_value_t = false,
        conflicts_with = "silent",
        help = "print palette table"
    )]
    pub preview: bool,

    #[arg(
        short = 'r',
        long = "render",
        default_value_t = false,
        help = "fill template files with colors"
    )]
    pub render: bool,

    #[arg(
        short = 'l',
        long = "link",
        default_value_t = false,
        help = "link all processed template to output"
    )]
    pub symlink: bool,

    #[arg(
        long = "link-to",
        value_name = "APP",
        help = "link specific app to output"
    )]
    pub link_to: Option<String>,

    #[arg(
        short = 'w',
        long = "wallpaper",
        default_value_t = false,
        help = "set as desktop wallpaper"
    )]
    pub wallpaper: bool,

    #[arg(
        short = 'q', long = "silent", default_value_t = false,
        conflicts_with_all = ["preview"],
        help = "no process printed on screen"
    )]
    pub silent: bool,

    #[arg(
        short = 'n',
        long = "dry-run",
        default_value_t = false,
        help = "rendering and linking flags won't be applied"
    )]
    pub dry_run: bool,

    #[arg(
        short = 'S',
        long = "style",
        value_name = "STYLE",
        default_value = "tonal-spot"
    )]
    pub style: Style,

    #[arg(
        long = "light",
        default_value_t = false,
        help = "generate light variant colors"
    )]
    pub light: bool,

    #[arg(
        short = 'W',
        long = "watch",
        default_value_t = false,
        help = "reload when files change"
    )]
    pub watch: bool,

    #[arg(
        short = 't',
        long = "contrast",
        value_name = "LEVEL",
        default_value = "standard",
        help = "increase color contrast"
    )]
    pub contrast: Contrast,

    #[arg(
        short = 'P', long = "pick", value_name = "N",
        value_parser = clap::value_parser!(u8).range(0..5),
        conflicts_with = "prefer",
        help = "explicitly choose a source color"
    )]
    pub pick: Option<u8>,

    #[arg(
        short = 'e',
        long = "prefer",
        value_name = "MODE",
        conflicts_with = "pick",
        help = "auto-pick source color based on..."
    )]
    pub prefer: Option<PreferMode>,

    #[arg(
        long = "open-picker",
        default_value_t = false,
        conflicts_with = "pick",
        help = "explicitly open the interactive color picker"
    )]
    pub open_picker: bool,

    #[arg(
        short = 'b',
        long = "blend-style",
        value_name = "STYLE",
        help = "blend with another style (requires --blend-ratio)"
    )]
    pub blend_style: Option<Style>,

    #[arg(
        long = "blend-ratio",
        value_name = "RATIO",
        default_value_t = 0.5,
        help = "blend ratio 0.0-1.0 when using --blend-style"
    )]
    pub blend_ratio: f64,

    #[arg(
        long = "init",
        default_value_t = false,
        help = "generate configuration file"
    )]
    pub init: bool,
}

impl Cli {
    pub fn validate(&self) -> Result<(), String> {
        if self.init {
            return Ok(());
        }
        let has_image = self.image.is_some() || self.image_url.is_some();
        if !has_image
            && self.color.is_none()
            && !self.restore
            && !self.symlink
            && self.link_to.is_none()
            && !self.render
            && !self.preview
        {
            return Err(
                "what to do?: -i image, -u url, -c color, -R restore, -a terminals, -m fill templates, -r link apps, -p preview".into()
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
        if self.blend_ratio < 0.0 || self.blend_ratio > 1.0 {
            return Err("--blend-ratio must be between 0.0 and 1.0".into());
        }
        Ok(())
    }
}
