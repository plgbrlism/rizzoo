use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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
    let mut hooks = Vec::new();
    for (name, entry) in templates {
        match render_and_write(name, entry, paths) {
            Ok(Some(hook)) => hooks.push((name.clone(), hook)),
            Ok(None) => {}
            Err(e) => log::warn!("{e}"),
        }
    }
    for (name, hook) in hooks {
        spawn_hook(&name, &hook);
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
    if let Some(hook) = render_and_write(name, entry, paths)? {
        spawn_hook(name, &hook);
    }
    Ok(())
}

fn render_and_write(
    name: &str,
    entry: &TemplateConfiguration,
    paths: &Paths,
) -> Result<Option<String>, LRatio> {
    if !entry.enabled {
        log::info!("Skipping disabled template: {name}");
        return Ok(None);
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
    Ok(entry.post_hook.clone())
}

/// Spawn a post_hook detached from the process lifetime.
///
/// Hooks run in the background so slow scripts (e.g. ones that compile
/// something) never gate rizzoo's exit. stdout is discarded to keep output
/// clean; stderr is inherited so failures remain visible without users having
/// to wire up their own logging.
fn spawn_hook(name: &str, hook: &str) {
    match Command::new("sh")
        .arg("-c")
        .arg(hook)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            log::info!("hook: spawned for {name}");
            // Reap in the background so watch mode does not accumulate zombies.
            // This never blocks the caller; on one-shot runs the process exits
            // first and the hook is simply orphaned.
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => log::warn!("hook: spawn failed for {name}: {e}"),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn paths_in(tmp: &std::path::Path) -> Paths {
        let cache = tmp.join("cache");
        Paths {
            cache_dir: cache.clone(),
            output: cache.join("colors.json"),
            schemes_dir: cache.join("schemes"),
            config_dir: tmp.join("config"),
            templates_dir: tmp.join("config/templates"),
            config_toml: tmp.join("config/config.toml"),
            downloads_dir: cache.join("url-images"),
        }
    }

    fn entry(
        template: &str,
        output: &std::path::Path,
        hook: Option<&str>,
    ) -> TemplateConfiguration {
        TemplateConfiguration {
            template: template.to_string(),
            output: output.to_string_lossy().to_string(),
            post_hook: hook.map(str::to_string),
            enabled: true,
        }
    }

    fn wait_for(path: &std::path::Path) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if path.exists() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    fn seed_template(paths: &Paths, name: &str) {
        std::fs::create_dir_all(&paths.cache_dir).unwrap();
        std::fs::write(paths.cache_dir.join(name), "rendered").unwrap();
    }

    #[test]
    fn render_and_write_collects_hook_without_running_it() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths_in(tmp.path());
        seed_template(&paths, "alacritty.toml");
        let sentinel = tmp.path().join("ran");
        let out = tmp.path().join("out.toml");
        let e = entry(
            "alacritty.toml",
            &out,
            Some(&format!("touch '{}'", sentinel.display())),
        );

        let hook = render_and_write("alacritty", &e, &paths).unwrap();

        assert!(hook.is_some());
        assert!(out.exists());
        assert!(!sentinel.exists(), "hook must not run during collection");
    }

    #[test]
    fn render_and_write_returns_none_for_disabled() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths_in(tmp.path());
        let mut e = entry("x.toml", &tmp.path().join("out"), Some("true"));
        e.enabled = false;

        assert!(render_and_write("x", &e, &paths).unwrap().is_none());
    }

    #[test]
    fn spawn_hook_runs_detached() {
        let tmp = tempfile::tempdir().unwrap();
        let sentinel = tmp.path().join("spawned");

        spawn_hook("test", &format!("touch '{}'", sentinel.display()));

        assert!(wait_for(&sentinel), "detached hook did not run");
    }

    #[test]
    fn render_all_spawns_hooks_after_all_outputs_written() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths_in(tmp.path());
        seed_template(&paths, "a.toml");
        seed_template(&paths, "b.toml");

        let out_a = tmp.path().join("a.out");
        let out_b = tmp.path().join("b.out");
        let sentinel = tmp.path().join("saw_both");
        let hook = format!(
            "[ -f '{}' ] && [ -f '{}' ] && touch '{}'",
            out_a.display(),
            out_b.display(),
            sentinel.display()
        );

        let mut templates = HashMap::new();
        templates.insert("a".to_string(), entry("a.toml", &out_a, Some(&hook)));
        templates.insert("b".to_string(), entry("b.toml", &out_b, None));

        render_all(&paths, &templates).unwrap();

        assert!(wait_for(&sentinel), "hook saw missing outputs");
    }
}
