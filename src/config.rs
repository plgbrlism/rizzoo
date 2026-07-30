use crate::color::types::Rgb;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Style {
    #[default]
    TonalSpot,
    Neutral,
    Vibrant,
    Expressive,
    Rainbow,
    FruitSalad,
    Monochrome,
    Fidelity,
    Content,
}

impl Style {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TonalSpot => "tonal-spot",
            Self::Neutral => "neutral",
            Self::Vibrant => "vibrant",
            Self::Expressive => "expressive",
            Self::Rainbow => "rainbow",
            Self::FruitSalad => "fruit-salad",
            Self::Monochrome => "monochrome",
            Self::Fidelity => "fidelity",
            Self::Content => "content",
        }
    }
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Contrast {
    #[default]
    Standard,
    Medium,
    High,
}

impl Contrast {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "standard",
            Self::Medium => "medium",
            Self::Standard => "high",
        }
    }

    pub fn multiplier(&self) -> f64 {
        match self {
            Self::High => 1.0,
            Self::Medium => 0.5,
            Self::Standard => 0.0,
        }
    }
}

impl std::fmt::Display for Contrast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum PreferMode {
    Darkness,
    Lightness,
    Saturation,
}

impl std::fmt::Display for PreferMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Darkness => write!(f, "darkness"),
            Self::Lightness => write!(f, "lightness"),
            Self::Saturation => write!(f, "saturation"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomColor {
    pub color: String,
    #[serde(default = "is_blend")]
    pub blend: bool,
}

fn is_blend() -> bool {
    true
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct WallpaperConfiguration {
    #[serde(default)]
    pub set: Option<bool>,
    #[serde(default)]
    pub command: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TemplateConfiguration {
    pub template: String,
    pub output: String,
    pub post_hook: Option<String>,
    #[serde(default = "is_template_configuration")]
    pub enabled: bool,
}

fn is_template_configuration() -> bool {
    true
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Configuration {
    #[serde(default)]
    pub style: Option<Style>,
    #[serde(default)]
    pub light: Option<bool>,
    #[serde(default)]
    pub contrast: Option<Contrast>,
    #[serde(default)]
    pub pick: Option<u8>,
    #[serde(default)]
    pub prefer: Option<PreferMode>,
    #[serde(default)]
    pub custom_colors: Option<HashMap<String, CustomColor>>,
    #[serde(default)]
    pub wallpaper: Option<WallpaperConfiguration>,
    #[serde(default, flatten)]
    pub templates: HashMap<String, TemplateConfiguration>,
}

impl Configuration {
    pub fn harmonize_custom_colors(&self, seed_color: &Rgb) -> HashMap<String, Rgb> {
        let Some(ref custom) = self.custom_colors else {
            return HashMap::new();
        };
        let mut out = HashMap::new();
        for (name, cfg) in custom {
            let Some(mut custom) = Rgb::from_hex(&cfg.color) else {
                continue;
            };
            if cfg.blend {
                custom = crate::color::blend::harmonize_color(&custom, seed_color);
            }
            out.insert(name.clone(), custom);
        }
        out
    }

    pub fn load(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("failed to read {}: {e}", path.display());
                return Self::default();
            }
        };
        match toml::from_str::<Configuration>(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                log::warn!("failed to parse {}: {e}", path.display());
                Self::default()
            }
        }
    }

    pub fn merge(&self, cli: &crate::cli::Cli) -> crate::cli::Cli {
        let mut c = cli.clone();
        if c.style == Style::TonalSpot {
            if let Some(style) = self.style {
                c.style = style;
            }
        }
        if !c.light {
            if let Some(l) = self.light {
                c.light = l;
            }
        }
        if c.contrast == Contrast::Standard {
            if let Some(ct) = self.contrast {
                c.contrast = ct;
            }
        }
        if c.pick.is_none() {
            if let Some(p) = self.pick {
                c.pick = Some(p);
            }
        }
        if c.prefer.is_none() {
            if let Some(pr) = self.prefer {
                if c.image.is_some() || c.image_url.is_some() {
                    c.prefer = Some(pr);
                }
            }
        }
        c
    }
}
