use std::ffi::OsString;
use std::path::PathBuf;

use super::{cli, strip_font_suffix, FONT_DIR};

const USAGE: &str = "Usage: figlist [options]
  -d, --fontdir <dir>  set the default font directory
  -h, --help           display usage information and exit
  -v, --version        display version information and exit";

pub fn run(args: Vec<OsString>) {
    let _ = run_inner(&args);
}

fn run_inner(args: &[OsString]) -> Result<(), String> {
    let mut cli_args = cli::Args::from_vec(args.to_vec());

    if cli_args.contains(["-v", "--version"]) {
        println!("figlist {} (FIGdriver-rs)", env!("CARGO_PKG_VERSION"));
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
    if !remaining.is_empty() {
        return Err(USAGE.to_string());
    }

    let default_font_path = PathBuf::from(&font_dir).join("standard.flf");
    let default_font_exists = default_font_path.exists();
    let default_font_name: String = if default_font_exists {
        "standard".to_string()
    } else {
        let mut found = None;
        if let Ok(entries) = std::fs::read_dir(&font_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".flf") || name.ends_with(".tlf") {
                    found.get_or_insert_with(|| strip_font_suffix(&name).to_string());
                    break;
                }
            }
        }
        found.unwrap_or("N/A".to_string())
    };

    println!("Default font: {}", default_font_name);
    println!("Font directory: {}", font_dir);

    let Ok(entries) = std::fs::read_dir(&font_dir) else {
        println!("Unable to open directory");
        return Ok(());
    };

    let mut flf_names: Vec<String> = Vec::new();
    let mut flc_names: Vec<String> = Vec::new();

    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name_str = match name.to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        if name_str.ends_with(".flf") || name_str.ends_with(".tlf") {
            flf_names.push(strip_font_suffix(&name_str).to_string());
        } else if name_str.ends_with(".flc") {
            flc_names.push(name_str.trim_end_matches(".flc").to_string());
        }
    }

    flf_names.sort();
    flc_names.sort();

    if flf_names.is_empty() {
        println!("No figlet fonts in this directory");
    } else {
        println!("Figlet fonts in this directory:");
        for name in flf_names {
            println!("{}", name);
        }
    }

    if flc_names.is_empty() {
        println!("No figlet control files in this directory");
    } else {
        println!("Figlet control files in this directory:");
        for name in flc_names {
            println!("{}", name);
        }
    }

    Ok(())
}
