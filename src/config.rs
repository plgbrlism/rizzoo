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
            Self::Standard => "standard",
            Self::Medium => "medium",
            Self::High => "high",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_toml(s: &str) -> Configuration {
        toml::from_str(s).expect(s)
    }

    fn tonalspot_cli() -> crate::cli::Cli {
        use clap::Parser;
        crate::cli::Cli::parse_from(["rizzoo", "-p"])
    }

    #[test]
    fn empty_config_all_defaults() {
        let cfg = parse_toml("");
        assert!(cfg.style.is_none());
        assert!(cfg.light.is_none());
        assert!(cfg.contrast.is_none());
        assert!(cfg.pick.is_none());
        assert!(cfg.custom_colors.is_none());
        assert!(cfg.templates.is_empty());
    }

    #[test]
    fn full_config_preview() {
        let cfg = parse_toml(
            r##"
            style = "expressive"
            light = true
            contrast = "high"
            pick = 3
            [wallpaper]
            set = true
            command = "swaybg -i {{ image }} -m fill"
            [custom_colors]
            accent = { color = "#e06c75", blend = true }
            highlight = { color = "#61afef" }
            "##,
        );
        assert_eq!(cfg.style, Some(Style::Expressive));
        assert_eq!(cfg.light, Some(true));
        assert_eq!(cfg.contrast, Some(Contrast::High));
        assert_eq!(cfg.pick, Some(3));
        assert_eq!(cfg.wallpaper.as_ref().unwrap().set, Some(true));
        let cc = cfg.custom_colors.as_ref().unwrap();
        assert!(cc["accent"].blend);
        assert!(cc["highlight"].blend); // default-blend is true
    }

    #[test]
    fn template_sections_flatten_into_templates() {
        let cfg = parse_toml(
            r#"
            [alacritty]
            template = "alacritty.tmpl"
            output = "~/.config/alacritty/colors.toml"
            [kitty]
            template = "kitty.tmpl"
            output = "~/.config/kitty/colors.conf"
            post_hook = "kill -SIGUSR1 $(pidof kitty)"
            "#,
        );
        assert_eq!(cfg.templates.len(), 2);
        let alac = &cfg.templates["alacritty"];
        assert_eq!(alac.template, "alacritty.tmpl");
        assert_eq!(alac.output, "~/.config/alacritty/colors.toml");
        assert_eq!(alac.post_hook, None);
        assert!(alac.enabled); // default-enabled
        let kitty = &cfg.templates["kitty"];
        assert_eq!(kitty.post_hook.as_deref(), Some("kill -SIGUSR1 $(pidof kitty)"));
    }

    #[test]
    fn template_requires_template_and_output() {
        // Missing `template` field → parse fails (this is the unhelpful-error case)
        // we intentionally lock in so a regression surfaces loudly.
        assert!(toml::from_str::<Configuration>("[foo]\noutput = \"x\"").is_err());
        assert!(toml::from_str::<Configuration>("[foo]\ntemplate = \"x\"").is_err());
    }

    #[test]
    fn invalid_toml_gracefully_defaults_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is [[[ not valid toml").unwrap();
        let cfg = Configuration::load(&path);
        assert!(cfg.templates.is_empty());
        assert!(cfg.style.is_none());
    }

    #[test]
    fn missing_file_defaults_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = Configuration::load(&dir.path().join("nope.toml"));
        assert!(cfg.templates.is_empty());
    }

    #[test]
    fn style_variants_deserialize() {
        for (s, expect) in [
            ("tonal-spot", Style::TonalSpot),
            ("neutral", Style::Neutral),
            ("vibrant", Style::Vibrant),
            ("expressive", Style::Expressive),
            ("rainbow", Style::Rainbow),
            ("fruit-salad", Style::FruitSalad),
            ("monochrome", Style::Monochrome),
            ("fidelity", Style::Fidelity),
            ("content", Style::Content),
        ] {
            let cfg = parse_toml(&format!("style = {s:?}"));
            assert_eq!(cfg.style.unwrap(), expect, "{s}");
        }
    }

    #[test]
    fn contrast_multiplier() {
        assert_eq!(Contrast::Standard.multiplier(), 0.0);
        assert_eq!(Contrast::Medium.multiplier(), 0.5);
        assert_eq!(Contrast::High.multiplier(), 1.0);
    }

    #[test]
    fn merge_applies_config_when_cli_is_default() {
        let cfg = parse_toml("style = \"vibrant\"\nlight = true\ncontrast = \"high\"\npick = 2");
        let merged = cfg.merge(&tonalspot_cli());
        assert_eq!(merged.style, Style::Vibrant);
        assert!(merged.light);
        assert_eq!(merged.contrast, Contrast::High);
        assert_eq!(merged.pick, Some(2));
    }

    #[test]
    fn merge_keeps_explicit_cli_over_config() {
        let cfg = parse_toml("style = \"vibrant\"\ncontrast = \"high\"\npick = 2");
        use crate::cli::Cli;
        use clap::Parser;
        let cli = Cli::parse_from(["rizzoo", "-p", "--style", "neutral", "--contrast", "medium"]);
        let merged = cfg.merge(&cli);
        assert_eq!(merged.style, Style::Neutral);
        assert_eq!(merged.contrast, Contrast::Medium);
        assert_eq!(merged.pick, Some(2)); // pick from config still applies
    }

    #[test]
    fn harmonize_custom_colors_blend_flag() {
        let cfg = parse_toml(
            r##"
            [custom_colors]
            accent    = { color = "#e06c75", blend = true }
            highlight = { color = "#61afef", blend = false }
            "##,
        );
        let seed = Rgb::rgb(10, 20, 30);
        let out = cfg.harmonize_custom_colors(&seed);
        // highlight has blend=false → unchanged
        assert_eq!(out["highlight"], Rgb::from_hex("#61afef").unwrap());
        // accent has blend=true → shifted toward seed
        assert_ne!(out["accent"], Rgb::from_hex("#e06c75").unwrap());
    }

    #[test]
    fn harmonize_no_custom_colors() {
        let cfg = Configuration::default();
        assert!(cfg.harmonize_custom_colors(&Rgb::rgb(1, 2, 3)).is_empty());
    }
}
