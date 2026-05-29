use std::ffi::OsString;
use std::io::{self, BufRead};
use std::path::{self, Path, PathBuf};
use pico_args::Arguments;
use regex::Regex;
use figdriver::Error;

const FONT_DIR     : &str = "/usr/share/figlet";
const DEFAULT_FONT : &str = "standard.flf";
const DEFAULT_WIDTH: usize = 80;


fn main() -> Result<(), Error> {
    let mut pargs = Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        println!("Usage: figlet-rs [options] message
  -c, --center          center the output horizontally
  -d, --dir <dir>       set the default font directory
  -f, --font <name>     specify the figfont to use
  -h, --help            display usage information and exit
  -k, --kern            use kerning mode to display characters
  -l, --left            left-align the output
  -m, --mode <num>      override the font layout mode
  -n, --normal          use normal mode (each newline causes a line break)
  -o, --overlap         use character overlapping mode
  -p, --paragraph       ignore mid-paragraph line breaks
  -R, --right-to-left   enable right-to-left print direction
  -r, --right           right-align the output
  -s, --smush-default   smushing respecting font default layout mode
  -S, --smush           force smushing mode to display characters
  -W, --full-width      display characters in full width
  -w, --width <cols>    set the output width");
        return Ok(());
    }

    let font_dir = pargs.opt_value_from_str::<_, String>(["-d", "--dir"])
        .map_err(|e| Error::Cli(e.to_string()))?
        .unwrap_or(FONT_DIR.to_string());

    let font_name = pargs.opt_value_from_str::<_, String>(["-f", "--font"])
        .map_err(|e| Error::Cli(e.to_string()))?;

    let use_kern = pargs.contains(["-k", "--kern"]);
    let use_overlap = pargs.contains(["-o", "--overlap"]);
    let use_paragraph = pargs.contains(["-p", "--paragraph"]);
    let use_normal = pargs.contains(["-n", "--normal"]);
    let use_full_width = pargs.contains(["-W", "--full-width"]);
    let use_center = pargs.contains(["-c", "--center"]);
    let use_right_to_left = pargs.contains(["-R", "--right-to-left"]);
    let use_left = pargs.contains(["-l", "--left"]);
    let use_right = pargs.contains(["-r", "--right"]);
    let use_smush = pargs.contains(["-s", "--smush-default"]);
    let use_smush_force = pargs.contains(["-S", "--smush"]);

    let width: usize = pargs.opt_value_from_str::<_, usize>(["-w", "--width"])
        .map_err(|e| Error::Cli(e.to_string()))?
        .unwrap_or(DEFAULT_WIDTH);

    let mut fontpath = PathBuf::from(font_dir);
    if let Some(name) = font_name {
        fontpath = find_font(fontpath, name);
    } else {
        fontpath.push(DEFAULT_FONT);
    }

    let msg: String = pargs.finish().into_iter()
        .filter_map(|s: OsString| s.into_string().ok())
        .collect::<Vec<_>>()
        .join(" ");

    // When both -p and -n are given, the last flag on the command line wins.
    let paragraph_mode = if use_paragraph && use_normal {
        let mut para = false;
        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "-p" | "--paragraph" => para = true,
                "-n" | "--normal" => para = false,
                _ => {}
            }
        }
        para
    } else {
        use_paragraph
    };

    run(&fontpath, &msg, &RunConfig {
            kern: use_kern,
            overlap: use_overlap,
            full_width: use_full_width,
            center: use_center,
            left: use_left,
            right: use_right,
            right_to_left: use_right_to_left,
            width,
            paragraph: paragraph_mode,
            smush: use_smush,
            smush_force: use_smush_force,
        })
}

fn find_font(mut fontpath: PathBuf, mut name: String) -> PathBuf {
    if !name.ends_with(".flf") && !name.ends_with(".tlf") {
        name = format!("{}.flf", name);
    }

    if name.starts_with(path::MAIN_SEPARATOR) {
        return PathBuf::from(name);
    }

    fontpath.push(&name);
    if fontpath.exists() {
        return fontpath;
    }

    PathBuf::from(name)
}

struct RunConfig {
    kern: bool,
    overlap: bool,
    full_width: bool,
    center: bool,
    left: bool,
    right: bool,
    right_to_left: bool,
    width: usize,
    paragraph: bool,
    smush: bool,
    smush_force: bool,
}

fn run(path: &Path, msg: &str, cfg: &RunConfig) -> Result<(), Error> {
    if !path.exists() {
        return Err(Error::FontNotFound(path.to_path_buf()));
    }

    let font = figdriver::FIGfont::from_path(path)?;
    let mut sm = figdriver::Smusher::new(&font);

    if cfg.smush_force {
        if (font.layout & figdriver::SMUSH_ENABLE) != 0 {
            sm.mode = font.layout;
        } else {
            sm.mode = 0;
        }
        sm.full_width = false;
    } else if cfg.smush && (font.layout & figdriver::SMUSH_ENABLE) != 0 {
        sm.mode = font.layout;
        sm.full_width = false;
    } else if cfg.overlap {
        sm.mode = 0;
    } else if cfg.kern {
        sm.mode = figdriver::SMUSH_KERN;
    }

    if cfg.full_width && !cfg.smush && !cfg.smush_force {
        sm.full_width = true;
    }

    if cfg.right_to_left {
        sm.right2left = true;
    }

    // Subtract 1 from width to match figlet's quirk: figlet treats `-w N` as
    // "allow lines up to N-1 characters" rather than N characters.
    let mut wr = figdriver::Wrapper::new(sm, cfg.width - 1);

    if cfg.center {
        wr.align = figdriver::Align::Center;
    } else if cfg.left {
        wr.align = figdriver::Align::Left;
    } else if cfg.right || cfg.right_to_left {
        wr.align = figdriver::Align::Right;
    }

    let re = Regex::new(r"(\S+|\s+)").unwrap();

    if !msg.is_empty() {
        write_line(&mut wr, msg, &re);
    } else {
        let input = io::BufReader::new(io::stdin());
        if cfg.paragraph {
            for line in input.lines() {
                let line = line?;
                write_paragraph(&mut wr, &line, &re);
            }
            print_output(&wr.get());
        } else {
            for line in input.lines() {
                let line = line?;
                write_line(&mut wr, &line, &re);
            }
        }
    }

    Ok(())
}

fn write_line(wr: &mut figdriver::Wrapper, s: &str, re: &Regex) {
    wr.clear();
    write_tokens(wr, s, re);
    print_output(&wr.get());
}

fn write_paragraph(wr: &mut figdriver::Wrapper, s: &str, re: &Regex) {
    if s.starts_with(char::is_whitespace) && !wr.is_empty() {
        print_output(&wr.get());
        wr.clear();
    }
    write_tokens(wr, s, re);
}

fn write_tokens(wr: &mut figdriver::Wrapper, s: &str, re: &Regex) {
    for caps in re.captures_iter(s) {
        if let Some(val) = caps.get(0) {
            wr.wrap_str(val.as_str(), &print_output);
        }
    }
}

fn print_output(v: &[String]) {
    for x in v {
        println!("{}", x);
    }
}
