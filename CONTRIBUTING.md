# contributing to rizzoo

hey, thanks for checking this out. This is hobby project of mine i've taken interest with, 
maybe due to falling on the linux theming rabbithole. 
Although it aims to be cross-platform like the same mature project im basing this off, i still 
didn't got to compile this on other platforms. 
So if ur someone interested with this, ur help will be deeply appreciated. 
I hope to learn with everyone working with a rust project.

## getting started

```sh
git clone https://github.com/plgbrlism/rizzoo && cd rizzoo
cargo build
cargo test
```

to test the example suite (never touches your real config):

```sh
./example/run.sh
```

## roadmap/todos

- [ ] documentation site (move config/template docs out of the readme)
- [ ] `--continue-on-error` for processing templates
- [ ] shell sequence generation for tty color application
- [ ] color templates for applications

## what we need help with

- **templates** — write color templates for common applications
- **testing** — more coverage on the template engine and config parsing
- **documentation site**
- **shell sequences** — tty color application via shell sequences
- **continue-on-error** — right now one bad template aborts the whole render unless its enabled=false.

## findings

- **template parser** — no space before `:` in `{{ var:filter }}`. `{{ primary:hex }}` works, `{{ primary :hex }}` breaks
- **for loops** — no nesting yet. single level only: `{{#for c in colors }}...{{/c }}` (might not be necessary)
- **config errors** — missing `template` field in a TOML section gives a vague "missing field `template` at line 1, column 1" error. this is a known unhelpful message from serde

## license

its on MIT, do whatever you want with it.
