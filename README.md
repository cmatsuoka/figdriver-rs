<!--
      ___ __          __      __                                    
    .'  _|__.-----.--|  .----|__.--.--.-----.----.______.----.-----.
    |   _|  |  _  |  _  |   _|  |  |  |  -__|   _|______|   _|__ --|
    |__| |__|___  |_____|__| |__|\___/|_____|__|        |__| |_____|
            |_____|                                                 

-->

# FIGdriver-rs: A FIGfont renderer written in Rust

[![CI](https://github.com/cmatsuoka/figdriver-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/cmatsuoka/figdriver-rs/actions/workflows/ci.yaml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

FIGdrivers render text as large ASCII-art banners using FIGfont (`.flf`) files. This
project implements the FIGfont specification, available as a library or as a drop-in
replacement for the classic `figlet` command.

## Features

- FIGfont 2 (`.flf`) and TOIlet (`.tlf`) font support
- Character-mapping control files (`.flc`)
- Zip-compressed font and control file support
- Right-to-left rendering
- Word wrapping with configurable alignment
- Full Unicode support in font glyphs
- Drop-in `figlet` CLI replacement

## Installation

Install the binary from source:

```bash
cargo install --path . --features cli
```

Or build locally:

```bash
cargo build --release --features cli
```

The resulting binary is `target/release/figlet`.

## Using the library

Add to your `Cargo.toml`:

```toml
[dependencies]
figdriver = "0.2"
```

The library has minimal dependencies (`unicode-width`, `zip`). The `figlet` binary requires the `cli` feature (`pico-args`, `terminal_size`).

Load a font and render text with `Smusher`:

```rust
use figdriver::{FIGfont, Smusher, LayoutMode};

fn run() -> Result<Vec<String>, figdriver::Error> {
    let font = FIGfont::from_path("fonts/standard.flf")?;
    let mut sm = Smusher::builder(&font)
        .layout_mode(LayoutMode::Kern)
        .build();
    sm.push_str("Hello world");
    Ok(sm.get())
}
```

For word-wrapping and text alignment, use `Wrapper`:

```rust
use figdriver::{FIGfont, Smusher, Wrapper, Align};

let font = FIGfont::from_path("fonts/small.flf")?;
let mut wr = Wrapper::new(Smusher::new(&font), 80, Align::Center);
wr.wrap_str("Hello world", &|lines| {
    for line in lines {
        println!("{}", line);
    }
});
```

For character-mapping control files (`.flc`), use `Control`:

```rust
use figdriver::{FIGfont, Smusher, Control};

let font = FIGfont::from_path("fonts/standard.flf")?;
let ctrl = Control::from_paths(&["controls/slip.flc"])?;
let mut sm = Smusher::builder(&font)
    .control(Some(&ctrl))
    .build();
```

See the [examples](examples/) directory for more usage patterns, or the [API documentation](https://docs.rs/figdriver) for the full reference.

## CLI usage

```bash
figlet [options] [text...]
```

If no text is provided, input is read from stdin.

### Options

| Flag                        | Description                                        |
| ---                         | ---                                                |
| `-f, --font <name>`           | Font to use (default: standard.flf)                |
| `-d, --dir <dir>`             | Font directory (default: /usr/share/figlet)        |
| `-C, --control <file>`        | Control file to apply (can be repeated)            |
| `-w, --width <cols>`          | Output width for wrapping (default: 80)            |
| `-t, --terminal-width`        | Use terminal width for output width                |
| `-m, --layout-mode <num>`     | Override font layout mode                          |
| `-c, --center`                | Center output                                      |
| `-l, --left`                  | Left-align output                                  |
| `-r, --right`                 | Right-align output                                 |
| `-x, --default-align`         | Default alignment (left for LTR, right for RTL)    |
| `-o, --overlap`               | Overlap mode                                       |
| `-k, --kern`                  | Kerning mode                                       |
| `-W, --full-width`            | Full width mode (no smushing)                      |
| `-s, --smush-default`         | Smush respecting font defaults                     |
| `-S, --smush`                 | Force smush mode                                   |
| `-p, --paragraph`             | Paragraph mode (ignore mid-paragraph newlines)     |
| `-n, --normal`                | Normal mode (newlines are line breaks)             |
| `-L, --left-to-right`         | Force left-to-right print direction                |
| `-R, --right-to-left`         | Right-to-left print direction                      |
| `-X, --font-direction`        | Use font file's default print direction            |
| `-I, --infocode <num>`        | Print info for infocode (0-5) and exit             |
| `-v, --version`               | Print version and exit                             |
| `-h, --help`                  | Print usage and exit                               |

### Examples

```bash
figlet Hello

figlet -f small "Hello, World!"

echo "figdriver-rs" | figlet -c -f banner

figlet -C slip -f standard "Hello"
```

