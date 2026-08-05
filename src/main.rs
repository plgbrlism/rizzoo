pub mod cache;
pub mod cli;
pub mod color;
pub mod config;
pub mod engine;
pub mod error;
pub mod export;
pub mod image;
pub mod paths;
pub mod template;
pub mod wallpaper;
pub mod watch;

use clap::Parser;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let cli = cli::Cli::parse();

    if cli.init || cli.init_overwrite {
        let paths = match paths::Paths::resolve() {
            Ok(p) => p,
            Err(e) => {
                log::error!("{e}");
                std::process::exit(1);
            }
        };
        if let Err(e) = paths.check_config(cli.init_overwrite) {
            log::error!("{e}");
            std::process::exit(1);
        }
        return;
    }

    let paths = match paths::Paths::resolve() {
        Ok(p) => {
            if let Err(e) = p.check_directory() {
                log::error!("{e}");
                std::process::exit(1);
            }
            p
        }
        Err(e) => {
            log::error!("{e}");
            std::process::exit(1);
        }
    };

    let config = config::Configuration::load(&paths.config_toml);
    let cli = config.merge(&cli);

    if let Err(e) = cli.validate() {
        eprintln!("\x1b[31merror\x1b[0m: {e}");
        std::process::exit(1);
    }

    if let Err(e) = run(cli, config) {
        log::error!("{e}");
        std::process::exit(1);
    }
}

fn run(cli: cli::Cli, config: config::Configuration) -> Result<(), error::LRatio> {
    let state = engine::State::new(cli, config)?;

    state.set_wallpaper()?;
    state.render_templates()?;
    state.write_outputs()?;
    state.preview()?;
    state.start_watch()?;

    Ok(())
}
