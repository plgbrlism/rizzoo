use std::collections::HashMap;
use std::path::PathBuf;

use directories::BaseDirs;
use tabled::{
    builder::Builder,
    settings::{Modify, Padding, Style, object::Segment},
};
use yansi::{Color, Paint};

use crate::color::types::{MaterialRoles, Rgb};
use crate::config::TemplateConfiguration;
use crate::error::LRatio;
use crate::paths::Paths;

pub fn render_all(
    paths: &Paths,
    templates: &HashMap<String, TemplateConfiguration>,
) -> Result<(), LRatio> {
    for (name, entry) in templates {
        if let Err(e) = render_and_write(name, entry, paths) {
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
    render_and_write(name, entry, paths)
}

fn render_and_write(
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
            "Template '{}' not found in cache. Ensure it exists in ~/.config/rizzoora/templates/ and -r or --render flag was run.",
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
    if output_path.exists() {
        std::fs::remove_file(&output_path).map_err(|e| {
            LRatio::Custom(format!(
                "Failed to remove existing file at {}: {}",
                output_path.display(),
                e
            ))
        })?;
    }
    std::fs::copy(&source_cache_path, &output_path).map_err(|e| {
        LRatio::OutputWriteFailed(
            source_cache_path.clone(),
            output_path.clone(),
            e.to_string(),
        )
    })?;
    log::info!("output: {name} → {}", output_path.display());
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

fn color_table(records: Vec<[String; 4]>) -> String {
    let mut builder = Builder::default();
    for row in records {
        builder.push_record(row);
    }
    let mut table = builder.build();
    table
        .with(Style::blank())
        .with(Modify::new(Segment::all()).with(Padding::new(3, 3, 0, 0)));
    table.to_string()
}

pub fn preview(
    variants: &[(&str, &MaterialRoles)],
    seed_colors: Option<&[Rgb]>,
    pick: usize,
) {
    if let Some(seeds) = seed_colors
        && !seeds.is_empty()
    {
        println!("\n{}", "Source/s".bold());
        let records = seeds
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let marker = if i == pick {
                    "<< seed-color".bold().italic().to_string()
                } else {
                    String::new()
                };
                [swatch(c), format!("[{}]", i), c.to_hex(), marker]
            })
            .collect();
        println!("{}", color_table(records));
    }

    for (label, roles) in variants {
        let title = if variants.len() > 1 {
            format!("{} {}", "Material Palette".bold(), format!("({label})").dim())
        } else {
            format!("\n{}", "Material Palette".bold())
        };
        println!("\n{}", title);

        let records = match serde_json::to_value(roles) {
            Ok(value) => value
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(name, value)| {
                            let hex = value.as_str()?;
                            let color = Rgb::from_hex(hex)?;
                            Some([swatch(&color), name.to_string(), hex.to_string(), String::new()])
                        })
                        .collect()
                })
                .unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        println!("{}", color_table(records));
    }
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
