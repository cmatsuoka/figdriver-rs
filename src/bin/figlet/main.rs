use std::ffi::OsString;
use std::fmt;
use std::io::{self, BufRead, Read};
use std::path::{Path, PathBuf, is_separator};
use figdriver::Control;

mod cli;
mod figlist;
mod showfigfonts;

const FONT_DIR     : &str = "/usr/share/figlet";
const DEFAULT_FONT : &str = "standard.flf";
const DEFAULT_WIDTH: usize = 80;

const LICENSE: &str = include_str!("../../../LICENSE");

const fn first_line(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut end = 0;
    while end < bytes.len() && bytes[end] != b'\n' {
        end += 1;
    }
    s.split_at(end).0
}

const COPYRIGHT_NOTICE: &str = first_line(LICENSE);

#[derive(Debug)]
enum Error {
    Cli(String),
    Library(figdriver::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Cli(msg)   => write!(f, "{}", msg),
            Error::Library(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Cli(_)     => None,
            Error::Library(e) => Some(e),
        }
    }
}

impl From<figdriver::Error> for Error {
    fn from(e: figdriver::Error) -> Self {
        Error::Library(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Cli(e.to_string())
    }
}

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    let name = args.first()
        .map(std::path::Path::new)
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("figlet");

    match name {
        n if n.contains("showfigfonts") => showfigfonts::run(args),
        n if n.contains("figlist")      => figlist::run(args),
        _                               => { if let Err(e) = run_figlet() { eprintln!("{}", e); std::process::exit(1); } }
    }
}

fn run_figlet() -> Result<(), Error> {
    let mut args = cli::Args::from_env();

    if args.contains(["-v", "--version"]) {
        println!("figlet {} (FIGdriver-rs)", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.contains(["-h", "--help"]) {
  println!("Usage: figlet [options] message
  -C, --control <file>     specify a control file (can be repeated)
  -c, --center             center the output horizontally
  -d, --fontdir <dir>      set the default font directory
  -f, --font <name>        specify the figfont to use
  -h, --help               display usage information and exit
  -I, --infocode <num>     print info for infocode (0-5) and exit
  -k, --kern               use kerning mode to display characters
  -l, --left               left-align the output
  -L, --left-to-right      force left-to-right print direction
  -m, --layout-mode <num>  override the font layout mode
  -n, --normal             use normal mode (each newline causes a line break)
  -o, --overlap            use character overlapping mode
  -p, --paragraph          ignore mid-paragraph line breaks
  -R, --right-to-left      enable right-to-left print direction
  -r, --right              right-align the output
  -s, --smush-default      smushing respecting font default layout mode
  -S, --smush              force smushing mode to display characters
  -t, --terminal-width     use terminal width for output width
  -v, --version            display version information and exit
  -W, --full-width         display characters in full width
  -w, --width <cols>       set the output width
  -X, --font-direction     use font file's default print direction
  -x, --default-align      default justification (left for LTR, right for RTL)");
        return Ok(());
    }

    let infocode: Option<i32> = args.opt_value_from_str::<i32>(["-I", "--infocode"])
        .map_err(|e| Error::Cli(e.to_string()))?;

    let font_dir = args.opt_value_from_str::<String>(["-d", "--fontdir"])
        .map_err(|e| Error::Cli(e.to_string()))?
        .or_else(|| std::env::var("FIGLET_FONTDIR").ok())
        .unwrap_or(FONT_DIR.to_string());

    let font_name = args.opt_value_from_str::<String>(["-f", "--font"])
        .map_err(|e| Error::Cli(e.to_string()))?;

    let control_files: Vec<String> = args.collect_values(["-C", "--control"]);
    let use_terminal_width = args.contains(["-t", "--terminal-width"]);
    let width: usize = match args.opt_value_from_str::<usize>(["-w", "--width"])
        .map_err(|e| Error::Cli(e.to_string()))?
    {
        Some(w) => w,
        None => {
            if use_terminal_width {
                if let Some((w, _)) = terminal_size::terminal_size() {
                    w.0 as usize
                } else {
                    DEFAULT_WIDTH
                }
            } else {
                DEFAULT_WIDTH
            }
        }
    };

    let layout_mode_value: Option<i32> = args.opt_value_from_str::<i32>(["-m", "--layout-mode"])
        .map_err(|e| Error::Cli(e.to_string()))?;

    let layout_mode_flag = args.last_of(&[
        (figdriver::LayoutMode::SmushForce,  &["-S", "--smush"]),
        (figdriver::LayoutMode::SmushDefault, &["-s", "--smush-default"]),
        (figdriver::LayoutMode::Overlap,     &["-o", "--overlap"]),
        (figdriver::LayoutMode::Kern,        &["-k", "--kern"]),
        (figdriver::LayoutMode::FullWidth,   &["-W", "--full-width"]),
    ]);

    let m_idx = args.last_index_of(&["-m", "--layout-mode"]);
    let flag_idx = args.last_index_of(&["-S", "--smush", "-s", "--smush-default", "-o", "--overlap", "-k", "--kern", "-W", "--full-width"]);
    let layout_mode: Option<figdriver::LayoutMode> = if let Some(m) = layout_mode_value {
        let resolved_m = match m {
            0 => figdriver::LayoutMode::Kern,
           -1 => figdriver::LayoutMode::FullWidth,
           -2 => figdriver::LayoutMode::SmushDefault,
            1.. => figdriver::LayoutMode::Custom(m as u32),
            _ => return Err(Error::Cli(format!("Invalid mode value: {}", m))),
        };
        if let (Some(mi), Some(fi)) = (m_idx, flag_idx) {
            if mi > fi {
                Some(resolved_m)
            } else {
                layout_mode_flag.or(Some(resolved_m))
            }
        } else {
            Some(resolved_m)
        }
    } else {
        layout_mode_flag
    };

    let paragraph = args.last_of(&[
        (true,  &["-p", "--paragraph"]),
        (false, &["-n", "--normal"]),
    ]).unwrap_or(false);

    let print_dir = args.last_of(&[
        (PrintDir::Ltr,         &["-L", "--left-to-right"]),
        (PrintDir::Rtl,         &["-R", "--right-to-left"]),
        (PrintDir::FontDefault, &["-X", "--font-direction"]),
    ]).unwrap_or(PrintDir::FontDefault);

    let justify = args.last_of(&[
        (Justify::Left,    &["-l", "--left"]),
        (Justify::Right,   &["-r", "--right"]),
        (Justify::Center,  &["-c", "--center"]),
        (Justify::Default, &["-x", "--default-align"]),
    ]).unwrap_or(Justify::Default);

    if let Some(code) = infocode {
        let display_font = font_name.as_deref().unwrap_or(DEFAULT_FONT);
        print_infocode(code, &font_dir, display_font, width);
        return Ok(());
    }

    let mut fontpath = PathBuf::from(&font_dir);
    if let Some(name) = font_name {
        fontpath = find_font(PathBuf::from(&font_dir), name);
    } else {
        fontpath.push(DEFAULT_FONT);
    }

    let control_paths: Vec<PathBuf> = control_files.iter().map(|f| find_control(&font_dir, f.clone())).collect();

    let remaining: Vec<OsString> = args.finish();
    let double_dash_pos = remaining.iter().position(|arg| arg.as_os_str() == "--");
    let before_len = double_dash_pos.unwrap_or(remaining.len());

    if let Some(invalid) = remaining[..before_len].iter().find(|arg| {
        let s = arg.to_string_lossy();
        s.starts_with('-') && s != "-"
    }) {
        return Err(Error::Cli(format!("Invalid flag: {}", invalid.to_string_lossy())));
    }

    let msg: String = remaining
        .into_iter()
        .enumerate()
        .filter(|&(i, _)| Some(i) != double_dash_pos)
        .map(|(_, s)| s)
        .filter_map(|s: OsString| s.into_string().ok())
        .collect::<Vec<_>>()
        .join(" ");

    let control = if control_files.is_empty() {
        None
    } else {
        Some(Control::from_paths(&control_paths)?)
    };

    let codes = if msg.is_empty() {
        let encoding = control.as_ref().map_or(figdriver::InputEncoding::Default, |c| c.encoding());
        let mut stdin_bytes = Vec::new();
        io::stdin().read_to_end(&mut stdin_bytes)?;
        if stdin_bytes.is_empty() {
            None
        } else {
            match &control {
                Some(ctrl) => Some(ctrl.decode_bytes(&stdin_bytes)),
                None => Some(encoding.decode_bytes(&stdin_bytes)),
            }
        }
    } else {
        None
    };

    run_figlet_render(&fontpath, &msg, &RunConfig {
            layout_mode,
            print_dir,
            width,
            paragraph,
            justify,
            codes,
        }, control)?;
    Ok(())
}

fn print_infocode(code: i32, font_dir: &str, font_name: &str, width: usize) {
    match code {
        0 => {
            println!("{}", COPYRIGHT_NOTICE);
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
        }
        1 => { print_version_int(); }
        2 => { println!("{}", font_dir); }
        3 => { println!("{}", strip_font_suffix(font_name)); }
        4 => { println!("{}", width); }
        5 => { println!("flf2 tlf2"); }
        // Reference figlet exits silently (code 0) for unsupported infocodes.
        _ => {}
    }
}

fn print_version_int() {
    let major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap_or(0);
    let minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap_or(0);
    let patch = env!("CARGO_PKG_VERSION_PATCH").chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().unwrap_or(0);
    println!("{}", major * 10000 + minor * 100 + patch);
}

pub fn strip_font_suffix(name: &str) -> &str {
    name.strip_suffix(".flf")
        .or_else(|| name.strip_suffix(".tlf"))
        .unwrap_or(name)
}

fn find_font(font_dir: PathBuf, name: String) -> PathBuf {
    let candidates = if name.ends_with(".flf") || name.ends_with(".tlf") {
        vec![name]
    } else {
        vec![format!("{}.flf", name), format!("{}.tlf", name)]
    };

    for candidate in &candidates {
        let path = if candidate.starts_with(is_separator) {
            PathBuf::from(candidate)
        } else {
            font_dir.join(candidate)
        };
        if path.exists() {
            return path;
        }
    }

    PathBuf::from(candidates.into_iter().next().unwrap())
}

fn find_control(font_dir: &str, mut name: String) -> PathBuf {
    if !name.ends_with(".flc") {
        name = format!("{}.flc", name);
    }

    if name.starts_with(is_separator) {
        return PathBuf::from(name);
    }

    let mut path = PathBuf::from(font_dir);
    path.push(&name);
    if path.exists() {
        return path;
    }

    PathBuf::from(name)
}

/// Text print direction (controlled by -L, -R, -X)
#[derive(Clone)]
enum PrintDir {
    /// Force left-to-right (-L)
    Ltr,
    /// Force right-to-left (-R)
    Rtl,
    /// Use font's default (-X, the default)
    FontDefault,
}

/// Output justification (controlled by -l, -r, -c, -x)
#[derive(Clone)]
enum Justify {
    /// Flush left (-l)
    Left,
    /// Flush right (-r)
    Right,
    /// Center (-c)
    Center,
    /// Follow print direction: left for LTR, right for RTL (-x, the default)
    Default,
}

struct RunConfig {
    layout_mode: Option<figdriver::LayoutMode>,
    print_dir: PrintDir,
    width: usize,
    paragraph: bool,
    justify: Justify,
    codes: Option<Vec<i32>>,
}

fn run_figlet_render(path: &Path, msg: &str, cfg: &RunConfig, control: Option<Control>) -> Result<(), figdriver::Error> {
    if !path.exists() {
        return Err(figdriver::Error::FontNotFound(path.to_path_buf()));
    }

    let font = figdriver::FIGfont::from_path(path)?;

    let layout_mode = cfg.layout_mode.unwrap_or(figdriver::LayoutMode::Default);

    let resolved_rtl = match cfg.print_dir {
        PrintDir::Ltr => false,
        PrintDir::Rtl => true,
        PrintDir::FontDefault => font.right_to_left,
    };

    let justify_align = match cfg.justify {
        Justify::Left => figdriver::Align::Left,
        Justify::Right => figdriver::Align::Right,
        Justify::Center => figdriver::Align::Center,
        Justify::Default => if resolved_rtl {
            figdriver::Align::Right
        } else {
            figdriver::Align::Left
        },
    };

    let sm = figdriver::Smusher::builder(&font)
        .control(control.as_ref())
        .layout_mode(layout_mode)
        .right_to_left(resolved_rtl)
        .build();

    // Subtract 1 from width to match figlet's quirk: figlet treats `-w N` as
    // "allow lines up to N-1 characters" rather than N characters.
    let mut wr = figdriver::Wrapper::new(sm, cfg.width - 1, justify_align);

    if let Some(ref codes) = cfg.codes {
        if cfg.paragraph {
            for segment in split_by_newline(codes) {
                write_paragraph_codes(&mut wr, segment);
            }
            if !wr.is_empty() {
                print_output(&wr.get());
            }
        } else {
            for segment in split_by_newline(codes) {
                write_line_codes(&mut wr, segment);
            }
        }
    } else if !msg.is_empty() {
        write_line(&mut wr, msg);
    } else {
        let mut input = io::BufReader::new(io::stdin());
        if cfg.paragraph {
            for line in input.lines() {
                let line = line?;
                write_paragraph(&mut wr, &line);
            }
            if !wr.is_empty() {
                print_output(&wr.get());
            }
        } else {
            let mut line = String::new();
            while input.read_line(&mut line)? > 0 {
                write_line(&mut wr, &line);
                line.clear();
            }
        }
    }

    Ok(())
}

fn split_by_newline(codes: &[i32]) -> Vec<&[i32]> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < codes.len() {
        let code = codes[i];
        if code == 10 || code == 13 {
            segments.push(&codes[start..i]);
            if code == 13 && i + 1 < codes.len() && codes[i + 1] == 10 {
                i += 2;
            } else {
                i += 1;
            }
            start = i;
        } else {
            i += 1;
        }
    }
    segments.push(&codes[start..]);
    segments
}

fn is_ws_code(code: i32) -> bool {
    code == 32 || code == 9 || code == 10 || code == 13
}

fn is_blank_codes(codes: &[i32]) -> bool {
    codes.iter().all(|&code| is_ws_code(code))
}

fn is_blank_str(s: &str) -> bool {
    s.chars().all(|c| c == ' ' || c == '\t' || c == '\n' || c == '\r')
}

fn write_tokens_codes(wr: &mut figdriver::Wrapper, codes: &[i32]) {
    let mut indices = codes.iter().enumerate().peekable();
    let mut start = 0;

    while let Some((_i, &code)) = indices.next() {
        let is_ws = is_ws_code(code);
        let is_space = code == 32;
        while let Some(&(_j, &next_code)) = indices.peek() {
            let next_ws = is_ws_code(next_code);
            let next_space = next_code == 32;
            if next_ws != is_ws || (is_ws && next_space != is_space) {
                break;
            }
            indices.next();
        }
        let end = indices.peek().map_or(codes.len(), |(idx, _)| *idx);
        wr.wrap_codes(&codes[start..end], &print_output);
        start = end;
    }
}

fn write_line_codes(wr: &mut figdriver::Wrapper, codes: &[i32]) {
    wr.clear();
    write_tokens_codes(wr, codes);
    if !wr.is_empty() {
        print_output(&wr.get());
    }
}

fn write_paragraph_codes(wr: &mut figdriver::Wrapper, codes: &[i32]) {
    if !wr.is_empty() {
        if is_blank_codes(codes) || (codes.first() == Some(&32)) {
            print_output(&wr.get());
            wr.clear();
        } else {
            wr.wrap_codes(&[32], &print_output);
        }
    }
    if !is_blank_codes(codes) {
        write_tokens_codes(wr, codes);
    }
}

fn write_line(wr: &mut figdriver::Wrapper, s: &str) {
    wr.clear();
    write_tokens(wr, s);
    if !wr.is_empty() {
        print_output(&wr.get());
    }
}

fn write_paragraph(wr: &mut figdriver::Wrapper, s: &str) {
    if !wr.is_empty() {
        if s.starts_with(' ') || is_blank_str(s) {
            print_output(&wr.get());
            wr.clear();
        } else {
            wr.wrap_str(" ", &print_output);
        }
    }
    if !is_blank_str(s) {
        write_tokens(wr, s);
    }
}

fn write_tokens(wr: &mut figdriver::Wrapper, s: &str) {
    let mut chars = s.char_indices().peekable();
    let mut start = 0;

    while let Some((_, c)) = chars.next() {
        let is_ws = c.is_whitespace();
        while let Some(&(_, next_c)) = chars.peek() {
            if next_c.is_whitespace() != is_ws || (is_ws && next_c != ' ') {
                break;
            }
            chars.next();
        }
        let end = chars.peek().map_or(s.len(), |&(idx, _)| idx);
        wr.wrap_str(&s[start..end], &print_output);
        start = end;
    }
}

fn print_output(v: &[String]) {
    for x in v {
        println!("{}", x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_by_newline_empty() {
        let result = split_by_newline(&[]);
        assert_eq!(result, [<&[i32]>::default()]);
    }

    #[test]
    fn split_by_newline_no_newlines() {
        let codes = [104, 101, 108, 108, 111];
        let result = split_by_newline(&codes);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &codes);
    }

    #[test]
    fn split_by_newline_single_lf() {
        let codes = [104, 101, 10, 119, 111];
        let result = split_by_newline(&codes);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], &[104, 101]);
        assert_eq!(result[1], &[119, 111]);
    }

    #[test]
    fn split_by_newline_single_cr() {
        let codes = [104, 101, 13, 119, 111];
        let result = split_by_newline(&codes);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], &[104, 101]);
        assert_eq!(result[1], &[119, 111]);
    }

    #[test]
    fn split_by_newline_crlf_as_single_boundary() {
        let codes = [104, 101, 13, 10, 119, 111];
        let result = split_by_newline(&codes);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], &[104, 101]);
        assert_eq!(result[1], &[119, 111]);
    }

    #[test]
    fn split_by_newline_crlf_no_extra_empty_segment() {
        let codes = [65, 13, 10, 66, 13, 10, 67];
        let result = split_by_newline(&codes);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], &[65]);
        assert_eq!(result[1], &[66]);
        assert_eq!(result[2], &[67]);
    }

    #[test]
    fn split_by_newline_consecutive_lf() {
        let codes = [65, 10, 10, 66];
        let result = split_by_newline(&codes);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], &[65]);
        assert!(result[1].is_empty());
        assert_eq!(result[2], &[66]);
    }

    #[test]
    fn split_by_newline_mixed_newlines() {
        let codes = [65, 13, 10, 66, 10, 67, 13, 68];
        let result = split_by_newline(&codes);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], &[65]);
        assert_eq!(result[1], &[66]);
        assert_eq!(result[2], &[67]);
        assert_eq!(result[3], &[68]);
    }

    #[test]
    fn split_by_newline_trailing_crlf() {
        let codes = [65, 13, 10];
        let result = split_by_newline(&codes);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], &[65]);
        assert!(result[1].is_empty());
    }

    #[test]
    fn split_by_newline_leading_crlf() {
        let codes = [13, 10, 65];
        let result = split_by_newline(&codes);
        assert_eq!(result.len(), 2);
        assert!(result[0].is_empty());
        assert_eq!(result[1], &[65]);
    }

    #[test]
    fn is_blank_codes_empty() {
        assert!(is_blank_codes(&[]));
    }

    #[test]
    fn is_blank_codes_spaces_only() {
        assert!(is_blank_codes(&[32, 32, 32]));
    }

    #[test]
    fn is_blank_codes_newlines_only() {
        assert!(is_blank_codes(&[10, 13]));
    }

    #[test]
    fn is_blank_codes_tabs_only() {
        assert!(is_blank_codes(&[9, 9]));
    }

    #[test]
    fn is_blank_codes_mixed_whitespace() {
        assert!(is_blank_codes(&[32, 9, 10, 13]));
    }

    #[test]
    fn is_blank_codes_has_text() {
        assert!(!is_blank_codes(&[65, 32]));
    }

    #[test]
    fn is_blank_codes_only_text() {
        assert!(!is_blank_codes(&[65, 66, 67]));
    }
}
