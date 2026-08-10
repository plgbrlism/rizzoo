use mcu_material_color::DynamicScheme;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl std::fmt::Display for Rgb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b) // todo: impl altered Rgb variants
    }
}

impl Rgb {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_raw_hex(&self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    pub fn to_css_rgb(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    pub fn to_css_rgba(&self, alpha: &str) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, alpha)
    }

    pub fn to_argb_u32(&self) -> u32 {
        (255u32 << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn from_argb_u32(argb: u32) -> Self {
        Self {
            r: ((argb >> 16) & 0xFF) as u8,
            g: ((argb >> 8) & 0xFF) as u8,
            b: (argb & 0xFF) as u8,
        }
    }

    pub fn to_hsl(&self) -> String {
        let (h, s, l) = self.to_tuple_hsl();
        format!("hsl({:.0}, {:.0}%, {:.0}%)", h, s, l)
    }

    pub fn to_hsla(&self, alpha: &str) -> String {
        let (h, s, l) = self.to_tuple_hsl();
        format!("hsla({:.0}, {:.0}%, {:.0}%, {})", h, s, l, alpha)
    }

    /// ~~ Color Space ~~

    pub fn to_hct(&self) -> mcu_material_color::hct::Hct {
        mcu_material_color::hct::Hct::from_int(self.to_argb_u32())
    }

    // sRGB → HSL conversion. Returns (hue 0-360, saturation 0-100, lightness 0-100).
    // Lightness uses the (max+min)/2 convention (not HSL's usual formula which
    // produces a different L for the same perceptual brightness). The saturation
    // formula accounts for the lightness axis to avoid dividing by zero at the
    // extremes (delta=0 → hue=0, saturation=0).
    pub fn to_tuple_hsl(&self) -> (f64, f64, f64) {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let l = (max + min) / 2.0;
        if delta == 0.0 {
            return (0.0, 0.0, l * 100.0);
        }
        let s = delta / (1.0 - (2.0 * l - 1.0).abs());
        let h = if max == r {
            60.0 * (((g - b) / delta).rem_euclid(6.0))
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };
        (
            h.clamp(0.0, 360.0),
            (s * 100.0).clamp(0.0, 100.0),
            (l * 100.0).clamp(0.0, 100.0),
        )
    }

    // HSL → sRGB conversion. Inverse of to_hsl_tuple.
    // Uses the standard HSL algorithm with hue_to_Rgb helper that maps hue
    // sextants to RGB channels. Achromatic case (s=0) is handled separately
    // to avoid division by zero in the saturation formula.
    pub fn from_hsl_tuple(h: f64, s: f64, l: f64) -> Self {
        let s = s / 100.0;
        let l = l / 100.0;
        if s == 0.0 {
            let v = (l * 255.0).round().clamp(0.0, 255.0) as u8;
            return Self { r: v, g: v, b: v };
        }
        let hue_to_rgb = |p: f64, q: f64, mut t: f64| -> f64 {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            if t < 1.0 / 6.0 {
                return p + (q - p) * 6.0 * t;
            }
            if t < 1.0 / 2.0 {
                return q;
            }
            if t < 2.0 / 3.0 {
                return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
            }
            p
        };
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        let h = h / 360.0;
        Self {
            r: (hue_to_rgb(p, q, h + 1.0 / 3.0) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8,
            g: (hue_to_rgb(p, q, h) * 255.0).round().clamp(0.0, 255.0) as u8,
            b: (hue_to_rgb(p, q, h - 1.0 / 3.0) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8,
        }
    }

    pub fn saturation(&self) -> f32 {
        self.to_tuple_hsl().1 as f32 / 100.0
    }

    // Relative luminance per WCAG: sRGB linearization + Rec.709 weights.
    // The piecewise function undoes the sRGB gamma curve to get linear
    // light values, then weights by human cone sensitivity (red 21%,
    // green 72%, blue 7%). The 0.03928 threshold is the sRGB spec's
    // breakpoint between linear and gamma-encoded segments.
    pub fn luminance_wcag(&self) -> f32 {
        fn adjust_wcag(c: u8) -> f32 {
            let sc = c as f32 / 255.0;
            if sc <= 0.03928 {
                sc / 12.92
            } else {
                ((sc + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * adjust_wcag(self.r) + 0.7152 * adjust_wcag(self.g) + 0.0722 * adjust_wcag(self.b)
    }
}

impl Serialize for Rgb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Rgb::from_hex(&s).ok_or_else(|| serde::de::Error::custom("invalid hex"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRoles {
    pub primary: Rgb,
    pub on_primary: Rgb,
    pub primary_container: Rgb,
    pub on_primary_container: Rgb,
    pub primary_fixed: Rgb,
    pub primary_fixed_dim: Rgb,
    pub on_primary_fixed: Rgb,
    pub on_primary_fixed_variant: Rgb,
    pub secondary: Rgb,
    pub on_secondary: Rgb,
    pub secondary_container: Rgb,
    pub on_secondary_container: Rgb,
    pub secondary_fixed: Rgb,
    pub secondary_fixed_dim: Rgb,
    pub on_secondary_fixed: Rgb,
    pub on_secondary_fixed_variant: Rgb,
    pub tertiary: Rgb,
    pub on_tertiary: Rgb,
    pub tertiary_container: Rgb,
    pub on_tertiary_container: Rgb,
    pub tertiary_fixed: Rgb,
    pub tertiary_fixed_dim: Rgb,
    pub on_tertiary_fixed: Rgb,
    pub on_tertiary_fixed_variant: Rgb,
    pub error: Rgb,
    pub on_error: Rgb,
    pub error_container: Rgb,
    pub on_error_container: Rgb,
    pub error_fixed: Rgb,
    pub error_fixed_dim: Rgb,
    pub on_error_fixed: Rgb,
    pub on_error_fixed_variant: Rgb,
    pub surface: Rgb,
    pub on_surface: Rgb,
    pub surface_container: Rgb,
    pub surface_container_low: Rgb,
    pub surface_container_high: Rgb,
    pub surface_container_highest: Rgb,
    pub surface_bright: Rgb,
    pub surface_dim: Rgb,
    pub on_surface_variant: Rgb,
    pub surface_tint: Rgb,
    pub surface_container_lowest: Rgb,
    pub outline: Rgb,
    pub outline_variant: Rgb,
    pub inverse_surface: Rgb,
    pub inverse_on_surface: Rgb,
    pub inverse_primary: Rgb,
    pub background: Rgb,
    pub on_background: Rgb,
    //pub shadow: Rgb,
    //pub scrim: Rgb,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub wallpaper: String,
    pub roles: MaterialRoles,
    pub base16: [Rgb; 16],
    pub seed_colors: Option<Vec<Rgb>>,
    #[serde(default)]
    pub pick: Option<usize>,
    #[serde(skip)]
    pub scheme: Option<DynamicScheme>,
}

impl std::fmt::Debug for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColorScheme")
            .field("wallpaper", &self.wallpaper)
            .field("roles", &self.roles)
            .field("base16", &self.base16)
            .field("seed_colors", &self.seed_colors)
            .finish()
    }
}
