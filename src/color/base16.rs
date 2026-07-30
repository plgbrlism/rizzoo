use crate::color::types::{MaterialRoles, Rgb};

pub struct Base16Generator;

impl Base16Generator {
    pub fn from_roles(roles: &MaterialRoles) -> [Rgb; 16] {
        [
            roles.surface_container_lowest,
            roles.surface_container_low,
            roles.surface_container_high,
            roles.outline,
            roles.on_surface_variant,
            roles.on_surface,
            roles.inverse_on_surface,
            roles.surface_bright,
            roles.error,
            roles.error_container,
            roles.tertiary,
            roles.secondary,
            roles.tertiary_container,
            roles.primary,
            roles.primary_container,
            roles.outline_variant,
        ]
    }
}
