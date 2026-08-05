//! End-to-end template rendering tests using real-world template shapes
//! (Kotlin, CSS, TypeScript, Rasi, SCSS, base16 palettes) against a real
//! Material 3 scheme, exercising parse -> evaluate -> build_context -> render.

use std::collections::HashMap;

use rizzoo::color::scheme::SchemeGenerator;
use rizzoo::color::types::{ColorScheme, Rgb};
use rizzoo::config::Style;
use rizzoo::paths::Paths;
use rizzoo::template::renderer::TemplateRenderer;

fn scheme() -> ColorScheme {
    SchemeGenerator::generate(&Rgb::rgb(75, 32, 130), Style::TonalSpot, true, 0.0)
}

fn custom_colors() -> HashMap<String, Rgb> {
    HashMap::from([("accent".into(), Rgb::rgb(224, 108, 117))])
}

/// Writes the given templates into a tempdir, runs the real render_all
/// pipeline, and returns filename -> rendered content.
fn render(templates: &[(&str, &str)]) -> HashMap<String, String> {
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

    for (name, content) in templates {
        std::fs::write(paths.templates_dir.join(name), content).unwrap();
    }

    let s = scheme();
    TemplateRenderer::render_all(&paths, &s, Some(&custom_colors())).unwrap();

    templates
        .iter()
        .map(|(name, _)| {
            let rendered = std::fs::read_to_string(paths.cache_dir.join(name)).unwrap();
            (name.to_string(), rendered)
        })
        .collect()
}

#[test]
fn css_variables() {
    let out = render(&[(
        "theme.css",
        ":root {\n  --md-sys-color-primary: {{ primary }};\n  --md-sys-color-surface: {{ surface }};\n}",
    )]);
    let css = &out["theme.css"];
    let s = scheme();
    assert!(css.contains(&format!("--md-sys-color-primary: {};", s.roles.primary.to_hex())));
    assert!(css.contains(&format!("--md-sys-color-surface: {};", s.roles.surface.to_hex())));
}

#[test]
fn kotlin_color_scheme() {
    let out = render(&[(
        "MaterialColors.kt",
        "object MaterialColors {\n    val primary = Color(0xFF{{ primary:hex_raw }})\n    val onPrimary = Color(0xFF{{ on_primary:hex_raw }})\n    val surface = Color(0xFF{{ surface:hex_raw }})\n}",
    )]);
    let kt = &out["MaterialColors.kt"];
    let s = scheme();
    assert!(kt.contains(&format!("val primary = Color(0xFF{})", s.roles.primary.to_raw_hex())));
    assert!(kt.contains(&format!("val onPrimary = Color(0xFF{})", s.roles.on_primary.to_raw_hex())));
    assert!(kt.contains(&format!("val surface = Color(0xFF{})", s.roles.surface.to_raw_hex())));
}

#[test]
fn typescript_palette() {
    let out = render(&[(
        "colors.ts",
        "export const materialColors = {\n  primary: \"{{ primary:hex_raw }}\",\n  surface: \"{{ surface:hex_raw }}\",\n} as const;",
    )]);
    let ts = &out["colors.ts"];
    let s = scheme();
    assert!(ts.contains(&format!("primary: \"{}\",", s.roles.primary.to_raw_hex())));
    assert!(ts.contains(&format!("surface: \"{}\",", s.roles.surface.to_raw_hex())));
}

#[test]
fn rofi_rasi() {
    let out = render(&[(
        "colors.rasi",
        "* {\n    primary: {{ primary }};\n    background: {{ surface }};\n}",
    )]);
    let rasi = &out["colors.rasi"];
    let s = scheme();
    assert!(rasi.contains(&format!("primary: {};", s.roles.primary.to_hex())));
    assert!(rasi.contains(&format!("background: {};", s.roles.surface.to_hex())));
}

#[test]
fn scss_variables() {
    let out = render(&[(
        "_colors.scss",
        "$primary: {{ primary:hex_raw }};\n$on-primary: {{ on_primary:hex_raw }};",
    )]);
    let scss = &out["_colors.scss"];
    let s = scheme();
    assert!(scss.contains(&format!("$primary: {};", s.roles.primary.to_raw_hex())));
    assert!(scss.contains(&format!("$on-primary: {};", s.roles.on_primary.to_raw_hex())));
}

#[test]
fn base16_for_loop() {
    let out = render(&[(
        "terminal.conf",
        "#{{#for c in colors }}{{ c:hex_raw }}\n#{{/c }}",
    )]);
    let rendered = &out["terminal.conf"];
    let s = scheme();
    assert_eq!(rendered.matches("#").count(), 16);
    for (i, c) in s.base16.iter().enumerate() {
        assert!(rendered.contains(&format!("#{}", c.to_raw_hex())), "base16[{i}] missing");
    }
}

#[test]
fn kotlin_base16_array_for_loop() {
    let out = render(&[(
        "Base16.kt",
        "val base16 = listOf(\n{{#for c in colors }}    \"{{ c:hex_raw }}\",\n{{/c }})",
    )]);
    let kt = &out["Base16.kt"];
    let s = scheme();
    assert!(kt.starts_with("val base16 = listOf("));
    assert!(kt.trim_end().ends_with(")"));
    for (i, c) in s.base16.iter().enumerate() {
        assert!(kt.contains(&format!("\"{}\",", c.to_raw_hex())), "base16[{i}] missing");
    }
    assert_eq!(kt.matches("0x").count(), 0);
}

#[test]
fn rgb_filter_integration() {
    let out = render(&[(
        "rgb.txt",
        "{{ primary:rgb }} | {{ secondary:rgb_css }}",
    )]);
    let rendered = &out["rgb.txt"];
    let s = scheme();
    assert!(rendered.contains(&s.roles.primary.to_raw_rgb()));
    assert!(rendered.contains(&s.roles.secondary.to_css_rgb()));
}

#[test]
fn custom_color_variable() {
    let out = render(&[("custom.txt", "accent = {{ custom_accent }}")]);
    let rendered = &out["custom.txt"];
    assert!(rendered.contains(&format!("accent = {}", Rgb::rgb(224, 108, 117).to_hex())));
}

#[test]
fn filter_chain_end_to_end() {
    // darken primary by 20%, then strip the hash
    let out = render(&[("dark.txt", "{{ primary:darken 0.2:hex_raw }}")]);
    let rendered = &out["dark.txt"];
    let s = scheme();
    let darkened = rizzoo::color::types::Rgb::from_hex(&rendered).unwrap();
    let (_, _, orig_l) = s.roles.primary.to_tuple_hsl();
    let (_, _, new_l) = darkened.to_tuple_hsl();
    assert!(new_l < orig_l);
}

#[test]
fn undefined_variable_reports_template_name() {
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
    std::fs::write(paths.templates_dir.join("broken.tmpl"), "{{ does_not_exist }}").unwrap();

    let err = TemplateRenderer::render_all(&paths, &scheme(), Some(&custom_colors())).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("broken.tmpl"), "missing template name in error: {msg}");
    assert!(msg.contains("does_not_exist"), "missing var name in error: {msg}");
}

#[test]
fn whitespace_in_output_is_preserved() {
    let out = render(&[(
        "spaces.txt",
        "  {{ primary }}  \n\t{{ primary }}\n",
    )]);
    let rendered = &out["spaces.txt"];
    assert!(rendered.starts_with("  "));
    assert!(rendered.contains("  \n\t"));
}
