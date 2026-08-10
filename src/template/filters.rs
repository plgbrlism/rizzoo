use crate::color::types::Rgb;

pub struct FilterRegistry;

impl FilterRegistry {
    pub fn apply(name: &str, value: &str, args: &[String]) -> Result<String, String> {
        match name {
            "hex_raw" => Ok(Self::hex_raw(value)),
            "rgb" => Ok(Self::rgb(value)),
            "rgba" => Ok(Self::rgba(value, args.first())),
            "hsl" => Ok(Self::hsl(value)),
            "hsla" => Ok(Self::hsla(value, args.first())),
            "hue" => Ok(Self::hue(value)),
            "saturation" => Ok(Self::saturation(value)),
            "lightness" => Ok(Self::lightness(value)),
            "r" => Ok(Self::r(value)),
            "g" => Ok(Self::g(value)),
            "b" => Ok(Self::b(value)),

            "lighten" => Self::with_amount(value, args, Self::lighten_op),
            "darken" => Self::with_amount(value, args, Self::darken_op),
            "saturate" => Self::with_amount(value, args, Self::saturate_op),
            "desaturate" => Self::with_amount(value, args, Self::desaturate_op),

            "invert" => Self::parse_rgb(value).map(|c| Self::invert(&c).to_hex()),
            "grayscale" => Self::parse_rgb(value).map(|c| Self::grayscale(&c).to_hex()),
            "auto_lightness" => Self::with_amount(value, args, Self::auto_lightness_op),
            "harmonize" => Self::with_target(value, args, Self::harmonize_op),
            "blend" => Self::with_blend(value, args),
            "ensure_contrast" => Self::with_contrast(value, args),
            _ => Err(format!("unknown filter: {name}")),
        }
    }

    fn parse_rgb(value: &str) -> Result<Rgb, String> {
        Rgb::from_hex(value).ok_or_else(|| format!("invalid hex color: {value}"))
    }

    fn hex_raw(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| c.to_raw_hex())
            .unwrap_or_else(|_| value.to_string())
    }
    fn rgb(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| c.to_css_rgb())
            .unwrap_or_else(|_| value.to_string())
    }
    fn rgba(value: &str, alpha: Option<&String>) -> String {
        let a = alpha.map(|s| s.as_str()).unwrap_or("1.0");
        Self::parse_rgb(value)
            .map(|c| c.to_css_rgba(a))
            .unwrap_or_else(|_| value.to_string())
    }
    fn hsl(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| c.to_hsl())
            .unwrap_or_else(|_| value.to_string())
    }
    fn hsla(value: &str, alpha: Option<&String>) -> String {
        let a = alpha.map(|s| s.as_str()).unwrap_or("1.0");
        Self::parse_rgb(value)
            .map(|c| c.to_hsla(a))
            .unwrap_or_else(|_| value.to_string())
    }
    fn hue(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| format!("{:.0}", c.to_tuple_hsl().0))
            .unwrap_or_else(|_| value.to_string())
    }
    fn saturation(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| format!("{:.0}", c.to_tuple_hsl().1))
            .unwrap_or_else(|_| value.to_string())
    }
    fn lightness(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| format!("{:.0}", c.to_tuple_hsl().2))
            .unwrap_or_else(|_| value.to_string())
    }
    fn r(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| format!("{}", c.r))
            .unwrap_or_else(|_| value.to_string())
    }
    fn g(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| format!("{}", c.g))
            .unwrap_or_else(|_| value.to_string())
    }
    fn b(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| format!("{}", c.b))
            .unwrap_or_else(|_| value.to_string())
    }

    fn with_amount(
        value: &str,
        args: &[String],
        op: fn(&Rgb, f64) -> Rgb,
    ) -> Result<String, String> {
        let amount = args
            .first()
            .and_then(|a| a.parse::<f64>().ok())
            .unwrap_or(0.1);
        let color = Self::parse_rgb(value)?;
        Ok(op(&color, amount).to_hex())
    }

    fn with_target(
        value: &str,
        args: &[String],
        op: fn(&Rgb, &Rgb) -> Rgb,
    ) -> Result<String, String> {
        let target_hex = args
            .first()
            .ok_or_else(|| "expected target color argument".to_string())?;
        let target = Self::parse_rgb(target_hex)?;
        let color = Self::parse_rgb(value)?;
        Ok(op(&color, &target).to_hex())
    }

    fn lighten_op(color: &Rgb, amount: f64) -> Rgb {
        let (h, s, l) = color.to_tuple_hsl();
        Rgb::from_hsl_tuple(h, s, (l + amount * 100.0).clamp(0.0, 100.0))
    }

    fn darken_op(color: &Rgb, amount: f64) -> Rgb {
        let (h, s, l) = color.to_tuple_hsl();
        Rgb::from_hsl_tuple(h, s, (l - amount * 100.0).clamp(0.0, 100.0))
    }

    fn saturate_op(color: &Rgb, amount: f64) -> Rgb {
        let (h, s, l) = color.to_tuple_hsl();
        Rgb::from_hsl_tuple(h, (s + amount * 100.0).clamp(0.0, 100.0), l)
    }

    fn desaturate_op(color: &Rgb, amount: f64) -> Rgb {
        let (h, s, l) = color.to_tuple_hsl();
        Rgb::from_hsl_tuple(h, (s - amount * 100.0).clamp(0.0, 100.0), l)
    }

    fn invert(color: &Rgb) -> Rgb {
        Rgb::rgb(255 - color.r, 255 - color.g, 255 - color.b)
    }

    fn grayscale(color: &Rgb) -> Rgb {
        let avg = (color.r as u16 + color.g as u16 + color.b as u16) / 3;
        Rgb::rgb(avg as u8, avg as u8, avg as u8)
    }

    fn auto_lightness_op(color: &Rgb, amount: f64) -> Rgb {
        let (h, s, l) = color.to_tuple_hsl();
        let new_l = if l < 50.0 {
            (l + amount * 100.0).clamp(0.0, 100.0)
        } else {
            (l - amount * 100.0).clamp(0.0, 100.0)
        };
        Rgb::from_hsl_tuple(h, s, new_l)
    }

    fn harmonize_op(color: &Rgb, target: &Rgb) -> Rgb {
        crate::color::blend::harmonize_color(color, target)
    }

    fn with_blend(value: &str, args: &[String]) -> Result<String, String> {
        if args.is_empty() {
            return Err("blend requires target color".to_string());
        }
        let target = Self::parse_rgb(&args[0])?;
        let amount = if args.len() > 1 {
            args[1]
                .parse::<f64>()
                .map_err(|_| "invalid blend amount".to_string())?
                .clamp(0.1, 0.9)
        } else {
            0.5
        };
        let color = Self::parse_rgb(value)?;
        Ok(crate::color::blend::blend_colors(&color, &target, amount).to_hex())
    }

    fn with_contrast(value: &str, args: &[String]) -> Result<String, String> {
        if args.len() < 2 {
            return Err("ensure_contrast requires background color and target ratio".to_string());
        }
        let bg = Self::parse_rgb(&args[0])?;
        let target_ratio = args[1]
            .parse::<f32>()
            .map_err(|_| "invalid contrast ratio".to_string())?;
        let fg = Self::parse_rgb(value)?;
        Ok(Self::ensure_contrast_op(&fg, &bg, target_ratio).to_hex())
    }

    // Iteratively adjusts lightness in 5% steps (up to 20 iterations) until
    // the WCAG contrast ratio against the background meets the target.
    // Moves AWAY from the background's lightness — if fg is darker than bg,
    // it darkens further; if lighter, it lightens. If the target isn't reached
    // after 20 steps, returns the last candidate tried.
    //
    // brute-force 5% steps; a binary search would converge faster
    // for extreme ratios. Replace with binary search if contrast failures are
    // reported with reasonable input colors.
    // ponytail: 20-step linear walk caps the reachable shift (~100 lightness
    // pts); binary search + a peek at both directions would guarantee success.
    fn ensure_contrast_op(fg: &Rgb, bg: &Rgb, target_ratio: f32) -> Rgb {
        let current = crate::color::blend::contrast_ratio(fg, bg);
        if current >= target_ratio {
            return *fg;
        }
        let (h, s, mut l) = fg.to_tuple_hsl();
        let bg_l = bg.to_tuple_hsl().2;
        // move away from bg: if fg is darker than bg, darken it (and vice-versa)
        let trend = if l < bg_l { -5.0 } else { 5.0 };
        let mut last = *fg;
        for _ in 0..20 {
            l = (l + trend).clamp(0.0, 100.0);
            let candidate = Rgb::from_hsl_tuple(h, s, l);
            last = candidate;
            if crate::color::blend::contrast_ratio(&candidate, bg) >= target_ratio {
                return candidate;
            }
        }
        last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(name: &str, value: &str) -> String {
        FilterRegistry::apply(name, value, &[]).unwrap()
    }

    #[test]
    fn hex_raw() {
        assert_eq!(apply("hex_raw", "#ff8000"), "ff8000");
    }

    #[test]
    fn rgb() {
        assert_eq!(apply("rgb", "#ff8000"), "rgb(255, 128, 0)");
    }

    #[test]
    fn rgba_default_alpha() {
        assert_eq!(apply("rgba", "#ff8000"), "rgba(255, 128, 0, 1.0)");
    }

    #[test]
    fn rgba_custom_alpha() {
        assert_eq!(
            FilterRegistry::apply("rgba", "#ff8000", &["0.5".into()]).unwrap(),
            "rgba(255, 128, 0, 0.5)"
        );
    }

    #[test]
    fn hsl() {
        assert_eq!(apply("hsl", "#ff0000"), "hsl(0, 100%, 50%)");
    }

    #[test]
    fn hsla_default_alpha() {
        assert_eq!(apply("hsla", "#ff0000"), "hsla(0, 100%, 50%, 1.0)");
    }

    #[test]
    fn hsla_custom_alpha() {
        assert_eq!(
            FilterRegistry::apply("hsla", "#ff0000", &["0.5".into()]).unwrap(),
            "hsla(0, 100%, 50%, 0.5)"
        );
    }

    #[test]
    fn hue_saturation_lightness() {
        assert_eq!(apply("hue", "#ff0000"), "0");
        assert_eq!(apply("saturation", "#ff0000"), "100");
        assert_eq!(apply("lightness", "#ff0000"), "50");
    }

    #[test]
    fn channels() {
        assert_eq!(apply("r", "#ff8000"), "255");
        assert_eq!(apply("g", "#ff8000"), "128");
        assert_eq!(apply("b", "#ff8000"), "0");
    }

    #[test]
    fn invert() {
        assert_eq!(apply("invert", "#ff0000"), "#00ffff");
    }

    #[test]
    fn grayscale() {
        assert_eq!(apply("grayscale", "#ff8000"), "#7f7f7f");
    }

    #[test]
    fn lighten_black() {
        assert_eq!(apply("lighten", "#000000"), "#1a1a1a");
    }

    #[test]
    fn darken_white() {
        assert_eq!(apply("darken", "#ffffff"), "#e6e6e6");
    }

    #[test]
    fn lighten_with_explicit_amount() {
        let out = FilterRegistry::apply("lighten", "#000000", &["0.5".into()]).unwrap();
        assert_ne!(out, "#000000");
        assert!(Rgb::from_hex(&out).is_some());
    }

    #[test]
    fn saturate_desaturate_move() {
        let gray = "#808080";
        let sat = apply("saturate", gray);
        let desat = apply("desaturate", "#ff0000");
        assert_ne!(sat, gray);
        assert_ne!(desat, "#ff0000");
    }

    #[test]
    fn blend_endpoints() {
        let a = FilterRegistry::apply("blend", "#ff0000", &["#00ff00".into(), "0.0".into()]).unwrap();
        assert_ne!(a, "#ff0000", "0.0 clamps up to 0.10 minimum");
        let b = FilterRegistry::apply("blend", "#ff0000", &["#00ff00".into(), "1.0".into()]).unwrap();
        assert_ne!(b, "#00ff00", "1.0 is capped below target");
    }

    #[test]
    fn blend_defaults_to_half() {
        let d = FilterRegistry::apply("blend", "#ff0000", &["#00ff00".into()]).unwrap();
        let e = FilterRegistry::apply("blend", "#ff0000", &["#00ff00".into(), "0.5".into()]).unwrap();
        assert_eq!(d, e, "1-arg blend must default to 50/50");
    }

    #[test]
    fn ensure_contrast_already_met_unchanged() {
        let out =
            FilterRegistry::apply("ensure_contrast", "#ffffff", &["#000000".into(), "4.5".into()])
                .unwrap();
        assert_eq!(out, "#ffffff");
    }

    #[test]
    fn ensure_contrast_lifts_fg() {
        let out =
            FilterRegistry::apply("ensure_contrast", "#cccccc", &["#ffffff".into(), "7.0".into()])
                .unwrap();
        assert_ne!(out, "#cccccc");
        let fg = Rgb::from_hex(&out).unwrap();
        let bg = Rgb::from_hex("#ffffff").unwrap();
        assert!(crate::color::blend::contrast_ratio(&fg, &bg) >= 7.0);
    }

    #[test]
    fn harmonize_changes_hue_toward_target() {
        let out = FilterRegistry::apply("harmonize", "#ff0000", &["#00ff00".into()]).unwrap();
        assert!(Rgb::from_hex(&out).is_some());
        assert_ne!(out, "#ff0000");
    }

    #[test]
    fn auto_lightness_darkens_light_color() {
        let out = FilterRegistry::apply("auto_lightness", "#ffffff", &[]).unwrap();
        assert_ne!(out, "#ffffff");
    }

    #[test]
    fn invalid_hex_falls_back_to_input() {
        assert_eq!(apply("hex_raw", "not-a-color"), "not-a-color");
    }

    #[test]
    fn unknown_filter_errors() {
        assert!(FilterRegistry::apply("nope", "#ff0000", &[]).is_err());
    }
}
