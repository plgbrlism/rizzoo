// ~~ Advanced Perceptual Color Algorithm ~~

fn soft_clamp(y: f32) -> f32 {
    if y < 0.022 {
        y + (0.022 - y).powf(1.414)
    } else {
        y
    }
}

pub fn contrast_apca(lum_fg: f32, lum_bg: f32) -> f32 {
    let y_bg = soft_clamp(lum_bg);
    let y_txt = soft_clamp(lum_fg);

    let lc = if y_bg > y_txt {
        (y_bg.powf(0.56) - y_txt.powf(0.57)) * 1.14
    } else {
        (y_bg.powf(0.65) - y_txt.powf(0.62)) * 1.14
    };

    (lc * 100.0).clamp(-100.0, 100.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApcaLevel {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

impl ApcaLevel {
    pub fn from_contrast(contrast: f32) -> Self {
        match contrast.abs() {
            x if x >= 45.0 => ApcaLevel::Diamond,
            x if x >= 35.0 => ApcaLevel::Platinum,
            x if x >= 25.0 => ApcaLevel::Gold,
            x if x >= 15.0 => ApcaLevel::Silver,
            _ => ApcaLevel::Bronze,
        }
    }
    pub fn is_compliant(&self) -> bool {
        matches!(self, ApcaLevel::Diamond | ApcaLevel::Platinum)
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Diamond => "Diamond",
            Self::Platinum => "Platinum",
            Self::Gold => "Gold",
            Self::Silver => "Silver",
            Self::Bronze => "Bronze",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApcaResult {
    pub role: String,
    pub score: f32,
    pub level: ApcaLevel,
    pub compliant: bool,
}

impl ApcaResult {
    pub fn new(role: &str, score: f32) -> Self {
        let level = ApcaLevel::from_contrast(score);
        Self {
            role: role.to_string(),
            score,
            level,
            compliant: level.is_compliant(),
        }
    }
}

impl std::fmt::Display for ApcaResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {:.1} ({})",
            self.role,
            self.score,
            self.level.label()
        )
    }
}

// Validate roles — caller provides Rgb pair closure
pub fn validate_roles<F>(mut get_pair: F) -> Vec<ApcaResult>
where
    F: FnMut(&str) -> Option<(f32, f32)>,
{
    let mut results = Vec::with_capacity(25);
    let roles = [
        "primary",
        "primary_container",
        "primary_fixed",
        "primary_fixed_dim",
        "secondary",
        "secondary_container",
        "secondary_fixed",
        "secondary_fixed_dim",
        "tertiary",
        "tertiary_container",
        "tertiary_fixed",
        "tertiary_fixed_dim",
        "error",
        "error_container",
        "error_fixed",
        "error_fixed_dim",
        "surface",
        "surface_container",
        "surface_container_low",
        "surface_container_high",
        "surface_container_highest",
        "surface_bright",
        "surface_dim",
        "surface_variant",
        "inverse_surface",
        "inverse_primary",
        "background",
    ];

    for role in roles {
        if let Some((lum_fg, lum_bg)) = get_pair(role) {
            let score = contrast_apca(lum_fg, lum_bg);
            results.push(ApcaResult::new(role, score));
        }
    }
    results
}

pub fn roles_compliant(results: &[ApcaResult]) -> bool {
    results.iter().all(|r| r.compliant)
}

pub fn role_issues(results: &[ApcaResult]) -> Vec<ApcaResult> {
    results.iter().filter(|r| !r.compliant).cloned().collect()
}
