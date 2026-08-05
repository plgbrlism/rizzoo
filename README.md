<div align="center">
  <h1>rizzoo</h1>
  <p>cross-platform Material 3 Expressive color generation</p>
  <sub>(pronounced: rihz-oo)</sub>
  <br><br>
  <img alt="license" src="https://img.shields.io/badge/license-MIT-2dd4bf?style=for-the-badge">
  <img alt="version" src="https://img.shields.io/crates/v/rizzoo?color=2dd4bf&style=for-the-badge">
  <img alt="stars" src="https://img.shields.io/github/stars/plgbrlism/rizzoo?color=2dd4bf&style=for-the-badge">
  <br>
  <a href="#showcase">Showcase</a>
  |
  <a href="#usage">Usage</a>
  |
  <a href="#configuration">Configuration</a>
  |
  <a href="#templates">Templates</a>
  |
  <a href="#installation">Installation</a>
  |
  <a href="#acknowledgements">Acknowledgements</a>
</div>

<!-- showcase gif: a `rizzoo -i image -r -o -p` terminal recording (try vhs by Charm, or asciinema + agg) -->

## Features

- **Material 3 Expressive** 
	- M3 + base16 palette from any image, URL, or hex color
- **Winnow-based Template Engine** 
	- minimal and simple color manipulation
- **Source Color Picking**
	- choose the source color from the top 4 optimal seed colors.
- **25+ Color Filters** 
	- `hex`, `rgb`, `hsl`, `lighten`, `darken`, `blend`, `harmonize`, `ensure_contrast`, and more...
- **Templating & Configuration Files** 
	- define templates, output paths, post hooks, and general configurations
- **Style Blending** 
	- mix two M3 styles at any ratio (e.g. 70% vibrant + 30% expressive)
- **Cross-platform Wallpaper Wrappers** 
	- feh (X11), swaybg (Wayland), osascript (macOS), Windows native
- **SHA256 Color Caching** 
	- identical images never regenerate
- **Live Reload** 
	- watch for changes and re-render automatically

## Usage

```sh
# from an image
rizzoo -i ~/Pictures/wall.jpg -r -o -p

# from a hex color
rizzoo -c "#7c3aed" -r -o -p

# from a URL
rizzoo -u "https://example.com/wall.jpg" -r -o

# first run? generate a default config
rizzoo --init
```

<details><summary>View Full Reference</summary>

```
Usage: rizzoo [OPTIONS]

Options:
  -i, --image <PATH>         image path as color source
  -u, --image-url <URL>      url link of an image as the color source
  -c, --color <HEX>          a color of your choice in hex format
  -R, --restore-wallpaper    restore last wallpaper
  -p, --preview              print palette table
  -r, --render               fill template files with colors
  -o, --output               write all processed templates to output
      --output-to <APP>      write specific processed template to output
  -w, --wallpaper            set as desktop wallpaper
  -q, --silent               no process printed on screen
  -n, --dry-run              rendering and linking flags won't be applied
  -S, --style <STYLE>        [default: tonal-spot] [possible values: tonal-spot, neutral, vibrant, expressive, rainbow, fruit-salad, monochrome, fidelity, content]
      --light                generate light variant colors
  -W, --watch                reload when files change
  -t, --contrast <LEVEL>     increase color contrast [default: standard] [possible values: standard, medium, high]
  -P, --pick <N>             explicitly choose a source color
  -e, --prefer <MODE>        auto-pick source color based on... [possible values: darkness, lightness, saturation]
      --open-picker          explicitly open the interactive color picker
  -b, --blend-style <STYLE>  blend with another style (requires --blend-ratio) [possible values: tonal-spot, neutral, vibrant, expressive, rainbow, fruit-salad, monochrome, fidelity, content]
      --blend-ratio <RATIO>  blend ratio 0.0-1.0 when using --blend-style [default: 0.5]
      --init                 generate configuration file
      --init-overwrite       overwrite configuration file with defaults
  -h, --help                 Print help
  -V, --version              Print version
```

</details>

## Configuration

`rizzoo` reads `~/.config/rizzoo/config.toml`. `rizzoo --init` generates a sane default with every option commented.

```toml
style = "tonal-spot"
light = false
contrast = "standard"

[wallpaper]
set = false
command = "swaybg -i {{ image }} -m fill"

[custom_colors]
accent = { color = "#e06c75", blend = true }

[alacritty]
template = "colors-alacritty.toml"
output   = "~/.config/alacritty/colors.toml"
post_hook = "pkill -SIGUSR1 alacritty"
```

`post_hook` runs after the template is written. Add `enabled = false` to skip an entry.

## Templates

Templates live in `~/.config/rizzoo/templates/`. `-r` renders them to cache, `-o` writes them to each entry's `output` path.


*Remember: Mr.`-o` or `--output` is dependent on Ms. `-r` or `--render`*

### Variables

Every Material role is a hex string: `{{ primary }}`, `{{ on_primary }}`, `{{ surface }}`, `{{ background }}`, `{{ outline }}`, `{{ error }}`… plus `{{ base00 }}`–`{{ base15 }}`, the `colors` array, `{{ custom_<name> }}`, and `{{ wallpaper }}`.

### Filters

<details>
<summary>View Table</summary>

| Filter | Syntax | Output |
|--------|---------|--------|
| `hex` | `{{ primary:hex }}` | `#a6ebc3` |
| `hex_raw` | `{{ primary:hex_raw }}` | `a6ebc3` |
| `rgb` | `{{ primary:rgb }}` | `166,235,195` |
| `rgb_css` | `{{ primary:rgb_css }}` | `Rgb166, 235, 195` |
| `rgba` | `{{ primary:rgba(0.5) }}` | `Rgba166, 235, 195, 0.5` |
| `hsl` | `{{ primary:hsl }}` | `150,59%,79%` |
| `hue` / `saturation` / `lightness` | `{{ primary:hue }}` | `150` |
| `r` / `g` / `b` | `{{ primary:r }}` | `166` |
| `lighten` / `darken` | `{{ primary:lighten(0.1) }}` | adjusted color |
| `saturate` / `desaturate` | `{{ primary:saturate(0.2) }}` | adjusted color |
| `invert` / `grayscale` | `{{ primary:invert }}` | transformed color |
| `blend` | `{{ primary:blend(%secondary, 0.5) }}` | CAM16-UCS blend |
| `harmonize` | `{{ primary:harmonize(%tertiary) }}` | hue-shifted toward target |
| `ensure_contrast` | `{{ on_surface:ensure_contrast(%surface, 4.5) }}` | WCAG-safe color |
| `set_hue` / `set_saturation` / `set_lightness` | `{{ primary:set_hue(180) }}` | forced channel |
| `set_red` / `set_green` / `set_blue` | `{{ primary:set_red(255) }}` | forced channel |

</details>

All chainable via colon `:` ➔ `{{ primary:darken(0.1):hex_raw }}`. Bare `%var` args resolve to other template variables.

### For loop

```
{{#for c in colors }}{{ c:hex_raw }}
{{/c }}
```

### Example

```css
/* material.css */
:root {
  --md-sys-color-primary: {{ primary }};
  --md-sys-color-on-primary: {{ on_primary }};
  --md-sys-color-surface: {{ surface }};
}
```

Check [`example/`](example/) for a sandboxed testing script.

## Installation

```sh
cargo install rizzoo
```

From source:

```sh
git clone https://github.com/plgbrlism/rizzoo && cd rizzoo
cargo build --release
```

## Acknowledgements

- [matugen](https://github.com/InioX/matugen) — material theming inspiration
- [pywal](https://github.com/dylanaraps/pywal) — popularized wallpaper-based color theming
- [mcu-material-color](https://crates.io/crates/mcu-material-color) — rust port of google material color utilities
- [wallust](https://codeberg.org/explosion-mental/wallust.git)
- [hellwal](https://github.com/danihek/hellwal) 
- [cwal](https://github.com/nitinbhat972/cwal)

## License

[MIT](LICENSE.md)
