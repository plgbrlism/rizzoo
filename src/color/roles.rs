// delete ts shit bro

use mcu_material_color::dynamiccolor::{DynamicColor, DynamicScheme, MaterialDynamicColors};

use crate::color::types::{MaterialRoles, Rgb};

pub struct RoleExtractor;

impl RoleExtractor {
    pub fn extract(scheme: &DynamicScheme) -> MaterialRoles {
        MaterialRoles {
            primary: rgb(&MaterialDynamicColors::primary(), scheme),
            on_primary: rgb(&MaterialDynamicColors::on_primary(), scheme),
            primary_container: rgb(&MaterialDynamicColors::primary_container(), scheme),
            on_primary_container: rgb(&MaterialDynamicColors::on_primary_container(), scheme),
            primary_fixed: rgb(&MaterialDynamicColors::primary_fixed(), scheme),
            primary_fixed_dim: rgb(&MaterialDynamicColors::primary_fixed_dim(), scheme),
            on_primary_fixed: rgb(&MaterialDynamicColors::on_primary_fixed(), scheme),
            on_primary_fixed_variant: rgb(
                &MaterialDynamicColors::on_primary_fixed_variant(),
                scheme,
            ),

            secondary: rgb(&MaterialDynamicColors::secondary(), scheme),
            on_secondary: rgb(&MaterialDynamicColors::on_secondary(), scheme),
            secondary_container: rgb(&MaterialDynamicColors::secondary_container(), scheme),
            on_secondary_container: rgb(&MaterialDynamicColors::on_secondary_container(), scheme),
            secondary_fixed: rgb(&MaterialDynamicColors::secondary_fixed(), scheme),
            secondary_fixed_dim: rgb(&MaterialDynamicColors::secondary_fixed_dim(), scheme),
            on_secondary_fixed: rgb(&MaterialDynamicColors::on_secondary_fixed(), scheme),
            on_secondary_fixed_variant: rgb(
                &MaterialDynamicColors::on_secondary_fixed_variant(),
                scheme,
            ),

            tertiary: rgb(&MaterialDynamicColors::tertiary(), scheme),
            on_tertiary: rgb(&MaterialDynamicColors::on_tertiary(), scheme),
            tertiary_container: rgb(&MaterialDynamicColors::tertiary_container(), scheme),
            on_tertiary_container: rgb(&MaterialDynamicColors::on_tertiary_container(), scheme),
            tertiary_fixed: rgb(&MaterialDynamicColors::tertiary_fixed(), scheme),
            tertiary_fixed_dim: rgb(&MaterialDynamicColors::tertiary_fixed_dim(), scheme),
            on_tertiary_fixed: rgb(&MaterialDynamicColors::on_tertiary_fixed(), scheme),
            on_tertiary_fixed_variant: rgb(
                &MaterialDynamicColors::on_tertiary_fixed_variant(),
                scheme,
            ),

            error: rgb(&MaterialDynamicColors::error(), scheme),
            on_error: rgb(&MaterialDynamicColors::on_error(), scheme),
            error_container: rgb(&MaterialDynamicColors::error_container(), scheme),
            on_error_container: rgb(&MaterialDynamicColors::on_error_container(), scheme),
            error_fixed: rgb(&MaterialDynamicColors::error(), scheme),
            error_fixed_dim: rgb(&MaterialDynamicColors::error(), scheme),
            on_error_fixed: rgb(&MaterialDynamicColors::on_error(), scheme),
            on_error_fixed_variant: rgb(&MaterialDynamicColors::on_error(), scheme),

            surface: rgb(&MaterialDynamicColors::surface(), scheme),
            on_surface: rgb(&MaterialDynamicColors::on_surface(), scheme),
            surface_container: rgb(&MaterialDynamicColors::surface_container(), scheme),
            surface_container_low: rgb(&MaterialDynamicColors::surface_container_low(), scheme),
            surface_container_high: rgb(&MaterialDynamicColors::surface_container_high(), scheme),
            surface_container_highest: rgb(
                &MaterialDynamicColors::surface_container_highest(),
                scheme,
            ),
            surface_bright: rgb(&MaterialDynamicColors::surface_bright(), scheme),
            surface_dim: rgb(&MaterialDynamicColors::surface_dim(), scheme),
            on_surface_variant: rgb(&MaterialDynamicColors::on_surface_variant(), scheme),
            surface_tint: rgb(&MaterialDynamicColors::surface_tint(), scheme),
            surface_container_lowest: rgb(
                &MaterialDynamicColors::surface_container_lowest(),
                scheme,
            ),

            outline: rgb(&MaterialDynamicColors::outline(), scheme),
            outline_variant: rgb(&MaterialDynamicColors::outline_variant(), scheme),

            inverse_surface: rgb(&MaterialDynamicColors::inverse_surface(), scheme),
            inverse_on_surface: rgb(&MaterialDynamicColors::inverse_on_surface(), scheme),
            inverse_primary: rgb(&MaterialDynamicColors::inverse_primary(), scheme),

            background: rgb(&MaterialDynamicColors::background(), scheme),
            on_background: rgb(&MaterialDynamicColors::on_background(), scheme),

            shadow: rgb(&MaterialDynamicColors::shadow(), scheme),
            scrim: rgb(&MaterialDynamicColors::scrim(), scheme),
        }
    }
}

fn rgb(dc: &DynamicColor, scheme: &DynamicScheme) -> Rgb {
    Rgb::from_argb_u32(dc.get_argb(scheme))
}
