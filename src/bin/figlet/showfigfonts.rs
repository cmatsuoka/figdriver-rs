use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;

use super::{cli, strip_font_suffix, FONT_DIR};

const USAGE: &str = "Usage: showfigfonts [options] [word]
  -d, --fontdir <dir>  set the default font directory
  -h, --help           display usage information and exit
  -v, --version        display version information and exit";

pub fn run(args: Vec<OsString>) {
    if let Err(e) = run_inner(&args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run_inner(args: &[OsString]) -> Result<(), String> {
    let mut cli_args = cli::Args::from_vec(args.to_vec());

    if cli_args.contains(["-v", "--version"]) {
        println!("showfigfonts {} (FIGdriver-rs)", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if cli_args.contains(["-h", "--help"]) {
        println!("{}", USAGE);
        return Ok(());
    }

    let font_dir = cli_args.opt_value_from_str::<String>(["-d", "--fontdir"])
        .map_err(|e| e.to_string())?
        .or_else(|| std::env::var("FIGLET_FONTDIR").ok())
        .unwrap_or(FONT_DIR.to_string());

    let remaining: Vec<OsString> = cli_args.finish();

    if remaining.len() > 1 {
        return Err(USAGE.to_string());
    }
    if let Some(arg) = remaining.first() {
        let s = arg.to_string_lossy();
        if s.starts_with('-') {
            return Err(USAGE.to_string());
        }
    }
    let word = remaining.first().map(|s| s.to_string_lossy().into_owned());

    let entries = std::fs::read_dir(&font_dir)
        .map_err(|e| format!("Unable to open directory '{}': {}", font_dir, e))?;

    let mut flf_names: Vec<String> = Vec::new();

    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name_str = match name.to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        if name_str.ends_with(".flf") || name_str.ends_with(".tlf") {
            flf_names.push(strip_font_suffix(&name_str).to_string());
        }
    }

    flf_names.sort();

    for font_name in flf_names {
        println!("{} :", font_name);

        let display_word = word.clone().unwrap_or_else(|| font_name.clone());
        render_font(&font_dir, &font_name, &display_word);
        println!();
        println!();
        let _ = io::stdout().flush();
    }

    Ok(())
}

fn render_font(font_dir: &str, font_name: &str, word: &str) {
    let candidates = if font_name.ends_with(".flf") || font_name.ends_with(".tlf") {
        vec![font_name.to_string()]
    } else {
        vec![
            format!("{}.flf", font_name),
            format!("{}.tlf", font_name),
        ]
    };

    let mut font_path = None;
    for candidate in &candidates {
        let path = PathBuf::from(font_dir).join(candidate);
        if path.exists() {
            font_path = Some(path);
            break;
        }
    }

    let font_path = match font_path {
        Some(p) => p,
        None => {
            eprintln!("Font not found: {}", font_name);
            return;
        }
    };

    let font = match figdriver::FIGfont::from_path(&font_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error reading font {}: {}", font_name, e);
            return;
        }
    };

    let sm = figdriver::Smusher::builder(font)
        .layout_mode(figdriver::LayoutMode::Default)
        .build();

    let mut wr = figdriver::Wrapper::new(sm, 80 - 1, figdriver::Align::Left);
    let print_fn = print_output;

    wr.write_line(word, &print_fn);
}

fn print_output(v: &[String]) {
    for line in v {
        println!("{}", line);
    }
}
