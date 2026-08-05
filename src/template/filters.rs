use crate::color::types::Rgb;

pub struct FilterRegistry;

impl FilterRegistry {
    pub fn apply(name: &str, value: &str, args: &[String]) -> Result<String, String> {
        match name {
            "hex" => Ok(Self::pass(value)),
            "hex_raw" => Ok(Self::hex_raw(value)),
            "rgb" => Ok(Self::rgb(value)),
            "rgb_css" => Ok(Self::rgb_css(value)),
            "rgba" => Ok(Self::rgba(value, args.first())),
            "hsl" => Ok(Self::hsl(value)),
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

            "set_hue" => Self::with_val(value, args, Self::set_hue_op),
            "set_saturation" => Self::with_val(value, args, Self::set_saturation_op),
            "set_lightness" => Self::with_val(value, args, Self::set_lightness_op),
            "set_red" => Self::with_val(value, args, Self::set_red_op),
            "set_green" => Self::with_val(value, args, Self::set_green_op),
            "set_blue" => Self::with_val(value, args, Self::set_blue_op),
            _ => Err(format!("unknown filter: {name}")),
        }
    }

    fn parse_rgb(value: &str) -> Result<Rgb, String> {
        Rgb::from_hex(value).ok_or_else(|| format!("invalid hex color: {value}"))
    }

    fn pass(value: &str) -> String {
        value.to_string()
    }
    fn hex_raw(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| c.to_raw_hex())
            .unwrap_or_else(|_| value.to_string())
    }
    fn rgb(value: &str) -> String {
        Self::parse_rgb(value)
            .map(|c| c.to_raw_rgb())
            .unwrap_or_else(|_| value.to_string())
    }
    fn rgb_css(value: &str) -> String {
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
            .map(|c| c.to_raw_hsl())
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

    fn with_val(value: &str, args: &[String], op: fn(&Rgb, f64) -> Rgb) -> Result<String, String> {
        let val = args
            .first()
            .and_then(|a| a.parse::<f64>().ok())
            .ok_or_else(|| "expected numeric argument".to_string())?;
        let color = Self::parse_rgb(value)?;
        Ok(op(&color, val).to_hex())
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

    fn set_hue_op(color: &Rgb, val: f64) -> Rgb {
        let (_, s, l) = color.to_tuple_hsl();
        Rgb::from_hsl_tuple(val.clamp(0.0, 360.0), s, l)
    }

    fn set_saturation_op(color: &Rgb, val: f64) -> Rgb {
        let (h, _, l) = color.to_tuple_hsl();
        Rgb::from_hsl_tuple(h, val.clamp(0.0, 100.0), l)
    }

    fn set_lightness_op(color: &Rgb, val: f64) -> Rgb {
        let (h, s, _) = color.to_tuple_hsl();
        Rgb::from_hsl_tuple(h, s, val.clamp(0.0, 100.0))
    }

    fn set_red_op(color: &Rgb, val: f64) -> Rgb {
        Rgb::rgb(val.clamp(0.0, 255.0) as u8, color.g, color.b)
    }

    fn set_green_op(color: &Rgb, val: f64) -> Rgb {
        Rgb::rgb(color.r, val.clamp(0.0, 255.0) as u8, color.b)
    }

    fn set_blue_op(color: &Rgb, val: f64) -> Rgb {
        Rgb::rgb(color.r, color.g, val.clamp(0.0, 255.0) as u8)
    }

    fn with_blend(value: &str, args: &[String]) -> Result<String, String> {
        if args.len() < 2 {
            return Err("blend requires target color and amount".to_string());
        }
        let target = Self::parse_rgb(&args[0])?;
        let amount = args[1]
            .parse::<f64>()
            .map_err(|_| "invalid blend amount".to_string())?;
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
