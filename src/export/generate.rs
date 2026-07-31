use std::collections::HashMap;
use std::path::{Path, PathBuf};

use directories::BaseDirs;
use tabled::{
    builder::Builder,
    settings::{Modify, Padding, Style, object::Segment},
};
use yansi::{Color, Paint};

use crate::color::types::{ColorScheme, Rgb};
use crate::config::TemplateConfiguration;
use crate::error::LRatio;
use crate::paths::Paths;

fn symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dst)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(src, dst)
    }
    #[cfg(not(any(unix, windows)))]
    {
        std::fs::copy(src, dst).map(|_| ())
    }
}

pub fn render_all(
    paths: &Paths,
    templates: &HashMap<String, TemplateConfiguration>,
) -> Result<(), LRatio> {
    for (name, entry) in templates {
        if let Err(e) = render_and_symlink(name, entry, paths) {
            log::warn!("{e}");
        }
    }
    Ok(())
}

pub fn render_one(
    paths: &Paths,
    templates: &HashMap<String, TemplateConfiguration>,
    name: &str,
) -> Result<(), LRatio> {
    let entry = templates
        .get(name)
        .ok_or_else(|| LRatio::Custom(format!("entry '{name}' not found in config")))?;
    render_and_symlink(name, entry, paths)
}

fn render_and_symlink(
    name: &str,
    entry: &TemplateConfiguration,
    paths: &Paths,
) -> Result<(), LRatio> {
    if !entry.enabled {
        log::info!("Skipping disabled template: {name}");
        return Ok(());
    }
    let source_cache_path = paths.cache_dir.join(&entry.template);
    if !source_cache_path.exists() {
        return Err(LRatio::Custom(format!(
            "Template '{}' not found in cache. Ensure it exists in ~/.config/rizzoora/templates/ and the --map flag was run.",
            entry.template
        )));
    }

    let output_path = resolve_path(&entry.output);
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LRatio::DirectoryCreationFailed(parent.to_path_buf(), e.to_string())
            })?;
        }
    }
    if output_path.exists() || output_path.is_symlink() {
        std::fs::remove_file(&output_path).map_err(|e| {
            LRatio::Custom(format!(
                "Failed to remove existing file at {}: {}",
                output_path.display(),
                e
            ))
        })?;
    }
    symlink(&source_cache_path, &output_path).map_err(|e| {
        LRatio::SymlinkFailed(
            source_cache_path.clone(),
            output_path.clone(),
            e.to_string(),
        )
    })?;
    if let Some(hook) = &entry.post_hook {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(hook)
            .output()
            .map_err(|e| {
                LRatio::Custom(format!(
                    "failed to run post_hook for template '{}': {e}",
                    entry.template
                ))
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!(
                "post_hook for template '{}' failed: {stderr}",
                entry.template
            );
        }
    }
    Ok(())
}

fn swatch(color: &Rgb) -> String {
    "          "
        .bg(Color::Rgb(color.r, color.g, color.b))
        .to_string()
}

pub fn preview(scheme: &ColorScheme, seed_colors: Option<&Vec<Rgb>>, pick: usize) {
    let mut builder = Builder::default();

    if let Some(seeds) = seed_colors {
        if !seeds.is_empty() {
            builder.push_record(vec![
                "Sources".bold().to_string(),
                "".into(),
                "".into(),
                "".into(),
                "".into(),
            ]);
            for (i, c) in seeds.iter().enumerate() {
                let marker = if i == pick {
                    "<< seed-color"
                        .rgb(c.r, c.g, c.b)
                        .bold()
                        .italic()
                        .to_string()
                } else {
                    "".to_string()
                };
                builder.push_record(vec![swatch(c), format!("[{}]", i), c.to_hex(), marker]);
            }
        }
    }

    builder.push_record(vec![
        "\n\nPalette".bold().to_string(),
        "".into(),
        "".into(),
        "".into(),
    ]);

    let roles = [
        ("primary", &scheme.roles.primary),
        ("on_primary", &scheme.roles.on_primary),
        ("primary_container", &scheme.roles.primary_container),
        ("on_primary_container", &scheme.roles.on_primary_container),
        ("secondary", &scheme.roles.secondary),
        ("on_secondary", &scheme.roles.on_secondary),
        ("secondary_container", &scheme.roles.secondary_container),
        (
            "on_secondary_container",
            &scheme.roles.on_secondary_container,
        ),
        ("tertiary", &scheme.roles.tertiary),
        ("on_tertiary", &scheme.roles.on_tertiary),
        ("error", &scheme.roles.error),
        ("on_error", &scheme.roles.on_error),
        ("surface", &scheme.roles.surface),
        ("on_surface", &scheme.roles.on_surface),
        ("surface_container", &scheme.roles.surface_container),
        ("outline", &scheme.roles.outline),
        ("background", &scheme.roles.background),
        ("on_background", &scheme.roles.on_background),
        ("surface_bright", &scheme.roles.surface_bright),
        ("surface_dim", &scheme.roles.surface_dim),
    ];

    for (name, c) in &roles {
        builder.push_record(vec![swatch(c), name.to_string(), c.to_hex(), "".into()]);
    }

    builder.push_record(vec![
        "\n\nBase16".bold().to_string(),
        "".into(),
        "".into(),
        "".into(),
    ]);
    for (i, c) in scheme.base16.iter().enumerate() {
        builder.push_record(vec![
            swatch(c),
            format!("base{:02}", i),
            c.to_hex(),
            "".into(),
        ]);
    }

    let mut table = builder.build();
    table
        .with(Style::blank())
        .with(Modify::new(Segment::all()).with(Padding::new(3, 1, 0, 0)));

    println!("\n{}\n", table);
}

fn resolve_path(path: &str) -> PathBuf {
    let rest = path
        .strip_prefix("~/")
        .or_else(|| path.strip_prefix("$HOME/"))
        .map(|s| s.to_string());
    if let Some(rest) = rest {
        if let Some(base_dirs) = BaseDirs::new() {
            return base_dirs.home_dir().join(rest);
        }
    }
    PathBuf::from(path)
}
