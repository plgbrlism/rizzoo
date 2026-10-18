use clap::Parser;
use rizzoo::{cli, config, engine, error, paths};
use yansi::Paint;

fn main() {
    yansi::whenever(yansi::Condition::TTY_AND_COLOR);

    let cli = cli::Cli::parse();

    let level = if cli.silent {
        "off"
    } else if cli.debug {
        "debug"
    } else if cli.verbose {
        "info"
    } else {
        "warn"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level))
        .format(|buf, record| {
            use std::io::Write;
            match record.level() {
                log::Level::Error => writeln!(buf, "{}: {}", "error".red().bold(), record.args()),
                log::Level::Warn => writeln!(buf, "{}: {}", "warn".yellow().bold(), record.args()),
                log::Level::Info => writeln!(buf, "{}: {}", "info".cyan().bold(), record.args()),
                level => writeln!(
                    buf,
                    "{} {}: {}",
                    format!("{level}").to_lowercase().dim(),
                    record.target(),
                    record.args()
                ),
            }
        })
        .init();

    if cli.init || cli.init_overwrite {
        let paths = match paths::Paths::resolve() {
            Ok(p) => p,
            Err(e) => fail(&e.to_string()),
        };
        if let Err(e) = paths.check_config(cli.init_overwrite) {
            fail(&e.to_string());
        }
        return;
    }

    let paths = match paths::Paths::resolve() {
        Ok(p) => {
            if let Err(e) = p.check_directory() {
                fail(&e.to_string());
            }
            p
        }
        Err(e) => fail(&e.to_string()),
    };

    let config = config::Configuration::load(&paths.config_toml);
    let cli = config.merge(&cli);

    if let Err(e) = cli.validate() {
        fail(&e);
    }

    if let Err(e) = run(cli, config) {
        fail(&e.to_string());
    }
}

fn fail(msg: &str) -> ! {
    eprintln!("{}: {msg}", "error".red().bold());
    std::process::exit(1);
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
