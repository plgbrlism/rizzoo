use std::collections::HashMap;

use crate::color::types::{ColorScheme, Rgb};
use crate::error::LRatio;
use crate::paths::Paths;
use crate::template::interpreter::{TemplateContext, TemplateEvaluator};
use crate::template::parser::TemplateParser;

pub struct TemplateRenderer;

impl TemplateRenderer {
    pub fn render_all(
        paths: &Paths,
        scheme: &ColorScheme,
        custom_colors: Option<&HashMap<String, Rgb>>,
    ) -> Result<(), LRatio> {
        let templates = Self::collect_templates(paths)?;
        let context = Self::build_context(scheme, custom_colors);

        for (filename, source) in &templates {
            let nodes = TemplateParser::parse(source).map_err(|e| {
                LRatio::Custom(format!("failed to parse template '{filename}': {e}"))
            })?;

            let rendered = TemplateEvaluator::evaluate(&nodes, &context).map_err(|e| {
                LRatio::Custom(format!("failed to evaluate template '{filename}': {e}"))
            })?;

            let out_path = paths.cache_dir.join(filename);

            if out_path.exists() {
                if let Ok(existing) = std::fs::read_to_string(&out_path) {
                    if existing == rendered {
                        continue;
                    }
                }
            }

            let tmp_path = out_path.with_extension("tmp");
            if let Err(e) = std::fs::write(&tmp_path, &rendered) {
                log::warn!("{}", LRatio::TemplateWrite(out_path, e.to_string()));
                continue;
            }
            if let Err(e) = std::fs::rename(&tmp_path, &out_path) {
                log::warn!("{}", LRatio::TemplateWrite(out_path, e.to_string()));
            }
        }

        Ok(())
    }

    fn build_context(
        scheme: &ColorScheme,
        custom_colors: Option<&HashMap<String, Rgb>>,
    ) -> TemplateContext {
        let mut ctx = TemplateContext::new();
        if let Ok(roles_json) = serde_json::to_value(&scheme.roles) {
            if let Some(obj) = roles_json.as_object() {
                for (key, value) in obj {
                    if let Some(hex) = value.as_str() {
                        ctx.vars.insert(key.clone(), hex.to_string());
                    }
                }
            }
        }

        ctx.vars
            .insert("wallpaper".into(), scheme.wallpaper.clone());

        let base16_hex: Vec<String> = scheme.base16.iter().map(|c| c.to_hex()).collect();
        for (i, hex) in base16_hex.iter().enumerate() {
            ctx.vars.insert(format!("base{:02}", i), hex.clone());
        }
        ctx.arrays.insert("colors".into(), base16_hex);

        if let Some(custom) = custom_colors {
            for (name, color) in custom {
                ctx.vars.insert(format!("custom_{}", name), color.to_hex());
            }
        }

        ctx
    }

    fn collect_templates(paths: &Paths) -> Result<HashMap<String, String>, LRatio> {
        let mut map = HashMap::new();

        if paths.templates_dir.is_dir() {
            let entries = std::fs::read_dir(&paths.templates_dir).map_err(LRatio::Io)?;

            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let filename = match path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                match std::fs::read_to_string(&path) {
                    Ok(contents) => {
                        map.insert(filename, contents);
                    }
                    Err(e) => log::warn!("{}", LRatio::TemplateRead(path, e.to_string())),
                }
            }
        }

        Ok(map)
    }
}
