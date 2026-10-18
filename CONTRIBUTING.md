# Contributing to rizzoo

Thanks for checking out **rizzoo**. It's a hobby project that fell out of the Linux theming rabbit hole, cross-platform by intent. Bug fixes, application templates, and Rust learning are all welcome.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (`cargo`, `rustc` 1.86+)
- `git`

## Build & test

```sh
git clone https://github.com/plgbrlism/rizzoo.git
cd rizzoo

cargo test             # run the suite
cargo build --release  # binary at target/release/rizzoo
```

Run your build directly:

```sh
./target/release/rizzoo -i path/to/image.jpg -r -o -p
```

## Contributing

- Work on the `dev` branch and open pull requests against it.
- Keep changes focused; match the existing style.
- Run `cargo test` before pushing.
- Template contributions live in the [Templates Collection](https://github.com/plgbrlism/rizzoo/wiki/Collections) - a template plus its config entry is enough.

## Reporting issues

Open an issue with your platform, the command you ran, and the output. Known issues are listed in [Troubleshooting](https://github.com/plgbrlism/rizzoo/wiki/Troubleshooting).
