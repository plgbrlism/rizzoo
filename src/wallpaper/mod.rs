use crate::config::WallpaperConfiguration;
use crate::error::LRatio;
use std::path::Path;

#[cfg(target_os = "linux")]
mod feh;
#[cfg(target_os = "linux")]
mod swaybg;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "windows")]
mod windows;

pub fn set(path: &Path, config: Option<&WallpaperConfiguration>) -> Result<(), LRatio> {
    if let Some(cfg) = config {
        if let Some(cmd) = &cfg.command {
            run_hook(cmd, path)?;
            return Ok(());
        }
    }

    // Platform-specific auto-detection
    auto_detect_set(path)
}

pub fn is_enabled(config: Option<&WallpaperConfiguration>) -> bool {
    config.and_then(|c| c.set).unwrap_or(true)
}

fn auto_detect_set(path: &Path) -> Result<(), LRatio> {
    #[cfg(target_os = "macos")]
    return macos::set(path);

    #[cfg(target_os = "windows")]
    return windows::set(path);

    #[cfg(target_os = "linux")]
    return linux_set(path);

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err(LRatio::NoCompositorDetected)
}

#[cfg(target_os = "linux")]
fn linux_set(path: &Path) -> Result<(), LRatio> {
    use std::env;
    use std::os::unix::fs::FileTypeExt;

    fn is_wayland_active() -> bool {
        if let Ok(display) = env::var("WAYLAND_DISPLAY") {
            if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
                let socket_path = Path::new(&runtime_dir).join(display);
                return std::fs::metadata(socket_path)
                    .map(|meta| meta.file_type().is_socket())
                    .unwrap_or(false);
            }
        }
        false
    }

    fn is_x11_active() -> bool {
        if Path::new("/tmp/.X11-unix").exists() {
            return true;
        }
        env::var("DISPLAY").is_ok()
    }

    fn is_bin_in_path(bin: &str) -> bool {
        std::process::Command::new("which")
            .arg(bin)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    struct Setter {
        name: &'static str,
        is_available: fn() -> bool,
        set: fn(&Path) -> Result<(), LRatio>,
    }

    let is_wayland = is_wayland_active();
    let is_x11 = is_x11_active();

    let wallpaper_setters: Vec<Setter> = if is_wayland {
        vec![Setter {
            name: "swaybg",
            is_available: || is_bin_in_path("swaybg"),
            set: swaybg::set,
        }]
    } else if is_x11 {
        vec![Setter {
            name: "feh",
            is_available: || is_bin_in_path("feh"),
            set: feh::set,
        }]
    } else {
        return Err(LRatio::NoCompositorDetected);
    };

    for setter in wallpaper_setters.iter().filter(|b| (b.is_available)()) {
        match (setter.set)(path) {
            Ok(()) => return Ok(()),
            Err(e) => log::warn!(
                "{}",
                LRatio::WallpaperSet(format!("{} failed: {}", setter.name, e))
            ),
        }
    }

    Err(LRatio::NoCompositorDetected)
}

fn run_hook(hook: &str, path: &Path) -> Result<(), LRatio> {
    let resolved = hook.replace("{{ image }}", &path.display().to_string());
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&resolved)
        .output()
        .map_err(|e| LRatio::WallpaperHook(format!("failed to execute hook: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            log::warn!(
                "{}",
                LRatio::WallpaperHook(format!("hook stderr: {stderr}"))
            );
        }
    }
    Ok(())
}
