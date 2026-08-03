use std::collections::HashMap;
use std::io::IsTerminal;
use yansi::Paint;

use crate::cache;
use crate::cli::Cli;
use crate::color;
use crate::color::types::{ColorScheme, Rgb};
use crate::config::Configuration;
use crate::error::LRatio;
use crate::export;
use crate::image;
use crate::paths::Paths;
use crate::template::renderer::TemplateRenderer;
use crate::wallpaper;

struct GeneratedScheme {
    scheme: ColorScheme,
    is_new: bool,
    seed_colors: Vec<Rgb>,
    custom_colors: HashMap<String, Rgb>,
    pick: usize,
}

pub struct State {
    pub cli: Cli,
    pub paths: Paths,
    pub config: Configuration,
    pub scheme: ColorScheme,
    pub seed_colors: Vec<Rgb>,
    pub custom_colors: HashMap<String, Rgb>,
    pub seed_color: Rgb,
    pub pick: usize,
}

impl State {
    pub fn new(cli: Cli, config: Configuration) -> Result<Self, LRatio> {
        let paths = Paths::resolve()?;

        let GeneratedScheme {
            scheme,
            is_new,
            seed_colors,
            custom_colors,
            pick,
        } = Self::load_or_generate(&cli, &paths, &config)?;

        let seed_color = seed_colors.first().copied().unwrap_or(scheme.roles.primary);

        if is_new && !cli.dry_run {
            export::json::write(&paths, &scheme)?;
            step(&cli, "updated: colors.json");
        }

        Ok(Self {
            cli,
            paths,
            config,
            scheme,
            seed_colors,
            custom_colors,
            seed_color,
            pick,
        })
    }

    fn resolve_image(cli: &Cli) -> Result<Option<std::path::PathBuf>, LRatio> {
        if let Some(url) = &cli.image_url {
            Ok(Some(crate::image::download::download(url)?))
        } else if let Some(path) = &cli.image {
            Ok(Some(crate::image::decode::resolve(path)?))
        } else {
            Ok(None)
        }
    }

    fn load_or_generate(
        cli: &Cli,
        paths: &Paths,
        config: &Configuration,
    ) -> Result<GeneratedScheme, LRatio> {
        if let Some(ref hex) = cli.color {
            Self::from_color(cli, paths, config, hex)
        } else if let Some(image_arg) = Self::resolve_image(cli)? {
            Self::from_image(cli, paths, config, &image_arg)
        } else if cli.restore {
            let scheme = export::json::read(paths)?;
            step(cli, "restore: loaded last scheme");
            Ok(GeneratedScheme {
                scheme,
                is_new: false,
                seed_colors: Vec::new(),
                custom_colors: HashMap::new(),
                pick: 0,
            })
        } else {
            if !paths.output.exists() {
                return Err(LRatio::CacheRead(
                    paths.output.clone(),
                    "colors.json missing — run with --image or --image-url first".to_string(),
                ));
            }
            let scheme = export::json::read(paths)?;
            Ok(GeneratedScheme {
                scheme,
                is_new: false,
                seed_colors: Vec::new(),
                custom_colors: HashMap::new(),
                pick: 0,
            })
        }
    }

    fn resolve_seed_color(cli: &Cli, seed_colors: &[Rgb]) -> Result<(Rgb, usize, String), LRatio> {
        if seed_colors.is_empty() {
            return Err(LRatio::NoColors);
        }
        let pick = if let Some(pref) = &cli.prefer {
            color::extract::ColorExtractor::select_by_preference(seed_colors, *pref)
        } else if let Some(p) = cli.pick {
            p.min(seed_colors.len().saturating_sub(1) as u8) as usize
        } else if cli.open_picker || (std::io::stdout().is_terminal() && seed_colors.len() > 1) {
            Self::show_interactive_picker(seed_colors)?
        } else {
            0
        };
        let c = &seed_colors[pick];
        let swatch = "            "
            .bg(yansi::Color::Rgb(c.r, c.g, c.b))
            .to_string();
        Ok((*c, pick, swatch))
    }

    fn show_interactive_picker(seed_colors: &[Rgb]) -> Result<usize, LRatio> {
        use dialoguer::Select;
        let swatches: Vec<String> = seed_colors
            .iter()
            .map(|c| {
                "            "
                    .bg(yansi::Color::Rgb(c.r, c.g, c.b))
                    .to_string()
            })
            .collect();
        let items: Vec<String> = seed_colors
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{}  {}  [{}]", swatches[i], c.to_hex(), i))
            .collect();
        let items_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();

        let selection = Select::new()
            //.with_prompt("\npick a source")
            .items(&items_refs)
            .default(0)
            .interact()
            .map_err(|e| LRatio::Custom(format!("interactive picker failed: {e}")))?;
        Ok(selection)
    }

    fn from_image(
        cli: &Cli,
        paths: &Paths,
        config: &Configuration,
        image_arg: &std::path::Path,
    ) -> Result<GeneratedScheme, LRatio> {
        let image_path = image::decode::resolve(image_arg)?;
        step(cli, &format!("image: {}", image_path.display()));

        if cli.light {
            step(cli, "mode: light");
        }

        let cache_key_str = format!("{}:{}", cli.style.as_str(), cli.contrast);
        let file_size = cache::file_size(&image_path);
        let key = cache::cache_key(&image_path, &cache_key_str, cli.light, file_size);

        let force_regenerate = cli.pick.is_some() || cli.open_picker;

        if !force_regenerate {
            if let Ok(Some(mut cached)) = cache::load_scheme(paths, &key) {
                step(cli, "colors: loaded from cache");
                cached.wallpaper = image_arg.to_string_lossy().to_string();
                let pick = cached.pick.unwrap_or(0);
                let seed_colors = if cli.preview && cached.seed_colors.is_none() {
                    color::extract::ColorExtractor::extract(&image_path).unwrap_or_default()
                } else {
                    cached.seed_colors.clone().unwrap_or_default()
                };
                cached.seed_colors = Some(seed_colors.clone());
                return Ok(GeneratedScheme {
                    scheme: cached,
                    is_new: false,
                    seed_colors,
                    custom_colors: HashMap::new(),
                    pick,
                });
            }
        }

        step(cli, &format!("scheme: {}", cli.style));
        let seed_colors = color::extract::ColorExtractor::extract(&image_path)?;
        let (raw_color, pick, swatch) = Self::resolve_seed_color(cli, &seed_colors)?;
        if let Some(pref) = &cli.prefer {
            step(cli, &format!("source: prefer: {pref}"));
        } else {
            step(cli, &format!("source: {}", swatch));
        }

        let is_dark = !cli.light;
        let contrast = cli.contrast.multiplier();

        let mut scheme = if let Some(blend_style) = cli.blend_style {
            color::scheme::SchemeGenerator::blend_styles(
                &raw_color,
                cli.style,
                blend_style,
                cli.blend_ratio,
                is_dark,
                contrast,
            )
        } else {
            color::scheme::SchemeGenerator::generate_with_options(
                &raw_color,
                cli.style,
                is_dark,
                contrast,
                image_path.clone(),
            )
        };

        scheme.seed_colors = Some(seed_colors.clone());
        scheme.pick = Some(pick);
        scheme.wallpaper = image_arg.to_string_lossy().to_string();
        step(cli, "colors: generated");

        let custom_colors = Self::generate_custom_colors(config, &raw_color)?;

        if !cli.dry_run {
            if let Err(e) = cache::save_scheme(paths, &key, &scheme) {
                log::warn!("{e}");
            }
        }

        // dead, transitioned into persistent caching of url images
        //if crate::image::loader::is_downloaded_temp_file(image_arg) {
        //    let _ = std::fs::remove_file(image_arg);
        //}

        Ok(GeneratedScheme {
            scheme,
            is_new: true,
            seed_colors,
            custom_colors,
            pick,
        })
    }

    fn from_color(
        cli: &Cli,
        _paths: &Paths,
        config: &Configuration,
        hex: &str,
    ) -> Result<GeneratedScheme, LRatio> {
        let raw_color =
            Rgb::from_hex(hex).ok_or_else(|| LRatio::Custom(format!("invalid color '{hex}'")))?;
        if cli.light {
            step(cli, "mode: light");
        }
        step(cli, &format!("color: {}", raw_color.to_hex()));
        step(cli, &format!("style: {}", cli.style));

        let is_dark = !cli.light;
        let contrast = cli.contrast.multiplier();

        let mut scheme = if let Some(blend_style) = cli.blend_style {
            color::scheme::SchemeGenerator::blend_styles(
                &raw_color,
                cli.style,
                blend_style,
                cli.blend_ratio,
                is_dark,
                contrast,
            )
        } else {
            color::scheme::SchemeGenerator::generate(&raw_color, cli.style, is_dark, contrast)
        };

        let seed_colors = vec![raw_color];
        scheme.seed_colors = Some(seed_colors.clone());
        step(cli, "colors: generated");

        let custom_colors = Self::generate_custom_colors(config, &raw_color)?;
        Ok(GeneratedScheme {
            scheme,
            is_new: true,
            seed_colors,
            custom_colors,
            pick: 0,
        })
    }

    fn generate_custom_colors(
        config: &Configuration,
        seed_color: &Rgb,
    ) -> Result<HashMap<String, Rgb>, LRatio> {
        Ok(config.harmonize_custom_colors(seed_color))
    }

    pub fn set_wallpaper(&self) -> Result<(), LRatio> {
        if self.cli.dry_run {
            return Ok(());
        }
        let wp_config = self.config.wallpaper.clone();
        let should_set = self.cli.wallpaper || wallpaper::is_enabled(wp_config.as_ref());
        if should_set && !self.scheme.wallpaper.is_empty() {
            let setter_used = wallpaper::set(
                &std::path::PathBuf::from(&self.scheme.wallpaper),
                wp_config.as_ref(),
            )?;
            step(
                &self.cli,
                &format!("wallpaper: {} via {}", self.scheme.wallpaper, setter_used),
            );
        }
        Ok(())
    }

    pub fn render_templates(&self) -> Result<(), LRatio> {
        if self.cli.dry_run {
            return Ok(());
        }
        if self.cli.render {
            match TemplateRenderer::render_all(&self.paths, &self.scheme, Some(&self.custom_colors))
            {
                Ok(()) => step(&self.cli, "templates: rendered to ~/.cache/rizzoo/"),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn symlink_configs(&self) -> Result<(), LRatio> {
        if self.cli.dry_run {
            return Ok(());
        }
        if let Some(app_name) = &self.cli.link_to {
            export::generate::render_one(&self.paths, &self.config.templates, app_name)?;
            step(&self.cli, &format!("output: symlinked {app_name}"));
        } else if self.cli.symlink {
            export::generate::render_all(&self.paths, &self.config.templates)?;
            step(&self.cli, "output: symlinked all rendered templates");
        }
        Ok(())
    }

    pub fn preview(&self) -> Result<(), LRatio> {
        if self.cli.preview {
            export::generate::preview(&self.scheme, Some(&self.seed_colors), self.pick);
        }
        Ok(())
    }

    pub fn start_watch(&self) -> Result<(), LRatio> {
        if self.cli.dry_run {
            return Ok(());
        }
        if self.cli.watch {
            crate::watch::start(&self.paths)?;
        }
        Ok(())
    }
}

fn step(cli: &Cli, msg: &str) {
    if !cli.silent {
        log::info!("{}", msg);
    }
}
