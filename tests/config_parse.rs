//! End-to-end config file tests: a realistic user config written to disk,
//! loaded through the real Configuration::load, then merged with a CLI.

use rizzoo::config::{Configuration, Style};
use rizzoo::paths::Paths;

fn load_in(tmp: &std::path::Path, content: &str) -> Configuration {
    let dir = tmp.join("config");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.toml");
    std::fs::write(&path, content).unwrap();
    Configuration::load(&path)
}

#[test]
fn realistic_full_config() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = load_in(
        tmp.path(),
        r##"
style = "expressive"
light = true
contrast = "medium"
pick = 1

[wallpaper]
set = true
command = "swaybg -i {{ image }} -m fill"

[custom_colors]
accent    = { color = "#e06c75", blend = true }
highlight = { color = "#61afef", blend = false }

[alacritty]
template = "colors-alacritty.toml"
output   = "~/.config/alacritty/colors.toml"
post_hook = "pkill -SIGUSR1 alacritty"

[kitty]
template = "colors-kitty.conf"
output   = "~/.config/kitty/colors.conf"

[sway]
template = "colors-sway"
output   = "~/.config/sway/colors"
"##,
    );

    assert_eq!(cfg.style, Some(Style::Expressive));
    assert_eq!(cfg.light, Some(true));
    assert!(cfg.wallpaper.as_ref().unwrap().set.unwrap_or(false));
    assert_eq!(cfg.custom_colors.as_ref().unwrap().len(), 2);
    assert_eq!(cfg.templates.len(), 3);

    let alacritty = &cfg.templates["alacritty"];
    assert_eq!(alacritty.post_hook.as_deref(), Some("pkill -SIGUSR1 alacritty"));
    assert!(alacritty.enabled);
    assert_eq!(cfg.templates["kitty"].post_hook, None);
}

#[test]
fn disabled_template_section() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = load_in(
        tmp.path(),
        r##"
[oldapp]
template = "old.tmpl"
output = "~/x"
enabled = false
"##,
    );
    let entry = &cfg.templates["oldapp"];
    assert!(!entry.enabled);
}

#[test]
fn commented_template_sections_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = load_in(
        tmp.path(),
        r#"
# [alacritty]
# template = "alacritty.tmpl"
# output   = "~/.config/alacritty/colors.toml"

[active]
template = "active.tmpl"
output = "~/active.out"
"#,
    );
    assert_eq!(cfg.templates.len(), 1);
    assert!(cfg.templates.contains_key("active"));
    assert!(!cfg.templates.contains_key("alacritty"));
}

#[test]
fn merge_config_with_cli() {
    use clap::Parser;
    use rizzoo::cli::Cli;

    let tmp = tempfile::tempdir().unwrap();
    let cfg = load_in(
        tmp.path(),
        "style = \"vibrant\"\nlight = true\ncontrast = \"high\"\npick = 2",
    );
    let cli = Cli::parse_from(["rizzoo", "-p", "-c", "#ff0000"]);
    let merged = cfg.merge(&cli);
    assert_eq!(merged.style, Style::Vibrant);
    assert!(merged.light);
    assert_eq!(merged.contrast, rizzoo::config::Contrast::High);
    assert_eq!(merged.pick, Some(2));
    assert_eq!(merged.color.as_deref(), Some("#ff0000"));
}

#[test]
fn render_one_missing_entry_errors() {
    use rizzoo::export;
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths {
        cache_dir: tmp.path().join("cache"),
        output: tmp.path().join("cache/colors.json"),
        schemes_dir: tmp.path().join("cache/schemes"),
        config_dir: tmp.path().join("config"),
        templates_dir: tmp.path().join("config/templates"),
        config_toml: tmp.path().join("config/config.toml"),
        downloads_dir: tmp.path().join("cache/url-images"),
    };
    paths.check_directory().unwrap();
    let cfg = Configuration::default();
    let err = export::generate::render_one(&paths, &cfg.templates, "nope").unwrap_err();
    assert!(err.to_string().contains("nope"));
}

#[test]
fn init_then_config_is_loadable() {
    // --init writes the default config; load() must parse it cleanly.
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths {
        cache_dir: tmp.path().join("cache"),
        output: tmp.path().join("cache/colors.json"),
        schemes_dir: tmp.path().join("cache/schemes"),
        config_dir: tmp.path().join("config"),
        templates_dir: tmp.path().join("config/templates"),
        config_toml: tmp.path().join("config/config.toml"),
        downloads_dir: tmp.path().join("cache/url-images"),
    };
    paths.check_config(false).unwrap();
    assert!(paths.config_toml.exists());
    let cfg = Configuration::load(&paths.config_toml);
    assert!(cfg.templates.is_empty()); // all commented out
}
