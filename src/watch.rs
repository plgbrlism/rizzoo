use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::time::Duration;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use crate::color::types::{ColorScheme, Rgb};
use crate::config::{self, TemplateConfiguration};
use crate::error::LRatio;
use crate::export;
use crate::paths::Paths;
use crate::template::renderer::TemplateRenderer;

type LoadedState = (
    ColorScheme,
    Option<HashMap<String, Rgb>>,
    HashMap<String, TemplateConfiguration>,
);

pub fn start(paths: &Paths) -> Result<(), LRatio> {
    let paths = paths.clone();
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| LRatio::Custom(format!("failed to start watcher: {e}")))?;

    if paths.templates_dir.exists() {
        watcher
            .watch(&paths.templates_dir, RecursiveMode::Recursive)
            .map_err(|e| LRatio::Custom(format!("failed to watch templates dir: {e}")))?;
    }

    if paths.config_toml.exists() {
        watcher
            .watch(&paths.config_toml, RecursiveMode::NonRecursive)
            .map_err(|e| LRatio::Custom(format!("failed to watch config toml: {e}")))?;
    }

    log::info!(
        "Watching for changes in {} (Ctrl+C to quit)",
        paths.config_dir.display()
    );

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| LRatio::Custom(format!("failed to set ctrl-c handler: {e}")))?;

    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(Ok(event)) => {
                if event.kind.is_access() {
                    continue;
                }

                std::thread::sleep(Duration::from_millis(150));
                while rx.try_recv().is_ok() {}

                log::info!("Change detected! Reloading...");

                let Some((scheme, custom_colors, templates)) = load_state(&paths) else {
                    continue;
                };

                if let Err(e) =
                    TemplateRenderer::render_all(&paths, &scheme, custom_colors.as_ref())
                {
                    log::warn!("{e}");
                    continue;
                }

                if let Err(e) = export::generate::render_all(&paths, &templates) {
                    log::warn!("{e}");
                } else {
                    log::info!("Reload complete.");
                }
            }
            Ok(Err(e)) => log::warn!("watch error: {e}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    log::info!("Watch mode terminated.");
    Ok(())
}

fn load_state(paths: &Paths) -> Option<LoadedState> {
    let scheme = export::json::read(paths).ok()?;

    let seed_color = scheme
        .seed_colors
        .as_ref()
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(scheme.roles.primary);

    let config = config::Configuration::load(&paths.config_toml);
    let custom_map = config.harmonize_custom_colors(&seed_color);
    let custom_colors = if custom_map.is_empty() {
        None
    } else {
        Some(custom_map)
    };
    let templates = config.templates;

    Some((scheme, custom_colors, templates))
}
