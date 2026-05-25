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
  -o, --overlap         use character overlapping mode
  -p, --paragraph       ignore mid-paragraph line breaks
  -R, --right-to-left   enable right-to-left print direction
  -r, --right           right-align the output
  -S, --smush           use smushing mode to display characters
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
    let use_full_width = pargs.contains(["-W", "--full-width"]);
    let use_center = pargs.contains(["-c", "--center"]);
    let use_right_to_left = pargs.contains(["-R", "--right-to-left"]);
    let use_right = pargs.contains(["-r", "--right"]);

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

    run(&fontpath, &msg, use_kern, use_overlap, use_full_width, use_center, use_right, use_right_to_left, width, use_paragraph)
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

fn run(path: &Path, msg: &str, use_kern: bool, use_overlap: bool, use_full_width: bool,
       use_center: bool, use_right: bool, use_right_to_left: bool, width: usize, use_paragraph: bool) -> Result<(), Error> {
    if !path.exists() {
        return Err(Error::FontNotFound(path.to_path_buf()));
    }
    
    let font = figdriver::FIGfont::from_path(path)?;
    let mut sm = figdriver::Smusher::new(&font);

    if use_overlap {
        sm.mode = 0;
    } else if use_kern {
        sm.mode = figdriver::SMUSH_KERN;
    }

    if use_full_width {
        sm.full_width = true;
    }

    if use_right_to_left {
        sm.right2left = true;
    }

    let mut wr = figdriver::Wrapper::new(sm, width);

    if use_center {
        wr.align = figdriver::Align::Center;
    } else if use_right || use_right_to_left {
        wr.align = figdriver::Align::Right;
    }

    let re = Regex::new(r"(\S+|\s+)").unwrap();

    if !msg.is_empty() {
        write_line(&mut wr, &msg, &re);
    } else {
        let input = io::BufReader::new(io::stdin());
        if use_paragraph {
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
