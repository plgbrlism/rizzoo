use mcu_material_color::blend::Blend;
use mcu_material_color::contrast::Contrast;

use crate::color::types::{MaterialRoles, Rgb};

macro_rules! blend_fields {
    ($a:expr, $b:expr, $fn:expr, $($field:ident),+ $(,)?) => {
        MaterialRoles {
            $($field: $fn(&$a.$field, &$b.$field),)+
        }
    };
}

// Interpolates between two RGB colors in CAM16-UCS, a perceptually uniform
// color space. Unlike sRGB lerp (which passes through muddy browns in the
// middle), CAM16-UCS preserves hue and saturation smoothly across the blend.
// This is the building block for all Material color blending operations.
pub fn lerp_rgb(a: &Rgb, b: &Rgb, ratio: f64) -> Rgb {
    Rgb::from_argb_u32(Blend::cam16_ucs(a.to_argb_u32(), b.to_argb_u32(), ratio))
}

// Blends every field of two MaterialRoles structs using CAM16-UCS interpolation.
// Delegates per-field work to the `blend_fields!` macro to avoid repeating
// all 48 field names at each blend call site.
pub fn blend_roles(a: &MaterialRoles, b: &MaterialRoles, ratio: f64) -> MaterialRoles {
    let lerp = |ca: &Rgb, cb: &Rgb| lerp_rgb(ca, cb, ratio);
    blend_fields!(
        a,
        b,
        lerp,
        primary,
        on_primary,
        primary_container,
        on_primary_container,
        primary_fixed,
        primary_fixed_dim,
        on_primary_fixed,
        on_primary_fixed_variant,
        secondary,
        on_secondary,
        secondary_container,
        on_secondary_container,
        secondary_fixed,
        secondary_fixed_dim,
        on_secondary_fixed,
        on_secondary_fixed_variant,
        tertiary,
        on_tertiary,
        tertiary_container,
        on_tertiary_container,
        tertiary_fixed,
        tertiary_fixed_dim,
        on_tertiary_fixed,
        on_tertiary_fixed_variant,
        error,
        on_error,
        error_container,
        on_error_container,
        error_fixed,
        error_fixed_dim,
        on_error_fixed,
        on_error_fixed_variant,
        surface,
        on_surface,
        surface_container,
        surface_container_low,
        surface_container_high,
        surface_container_highest,
        surface_bright,
        surface_dim,
        on_surface_variant,
        surface_tint,
        surface_container_lowest,
        outline,
        outline_variant,
        inverse_surface,
        inverse_on_surface,
        inverse_primary,
        background,
        on_background,
        shadow,
        scrim,
    )
}

// Shifts `color` toward `target` in CAM16-UCS hue while preserving its
// original tone and chroma. Used to reconcile custom colors with the
// extracted seed color so all colors in a scheme feel coherent.
pub fn harmonize_color(color: &Rgb, target: &Rgb) -> Rgb {
    Rgb::from_argb_u32(Blend::harmonize(color.to_argb_u32(), target.to_argb_u32()))
}

// Full CAM16-UCS blend between two colors with amount clamped to [0, 1].
// amount=0 → color, amount=1 → target. The union of `lerp_rgb` range.
pub fn blend_colors(color: &Rgb, target: &Rgb, amount: f64) -> Rgb {
    Rgb::from_argb_u32(Blend::cam16_ucs(
        color.to_argb_u32(),
        target.to_argb_u32(),
        amount.clamp(0.0, 1.0),
    ))
}

// WCAG-style contrast ratio using HCT tone instead of relative luminance.
// The Material Design 3 standard uses tone-based contrast because HCT's
// tone channel perceptually matches human lightness perception better
// than the CIE relative luminance formula.
pub fn contrast_ratio(fg: &Rgb, bg: &Rgb) -> f32 {
    Contrast::ratio_of_tones(fg.to_hct().tone(), bg.to_hct().tone()) as f32
}
