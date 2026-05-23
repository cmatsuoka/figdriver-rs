use std::char;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::Error;

pub const SMUSH_EQUAL    : u32 = 1;
pub const SMUSH_UNDERLINE: u32 = 2;
pub const SMUSH_HIERARCHY: u32 = 4;
pub const SMUSH_PAIR     : u32 = 8;
pub const SMUSH_BIGX     : u32 = 16;
pub const SMUSH_HARDBLANK: u32 = 32;
pub const SMUSH_KERN     : u32 = 64;
pub const SMUSH_ENABLE   : u32 = 128;

/// A font made of large ASCII-art characters.
///
/// FIGfont defines a set of large ASCII-art (or UTF-8 Unicode art) characters,
/// called FIGcharacters, and a layout mode to control how FIGcharacters can be
/// fit together in a line. All FIGcharacters in a font must have the same number
/// of lines, and all lines in a FIGcharacter must have the same number of
/// characters.
#[derive(Debug, Default)]
pub struct FIGfont {
    version       : char,     // font standard version (currently 'a')
    pub hardblank : char,     // sub-character used to represent hardblanks
    pub height    : usize,
    baseline      : usize,    // number of lines from the baseline of a FIGcharacter
    max_length    : usize,    // maximum length of any line describing a FIGcharacter
    pub old_layout: i32,
    comment_lines : usize,    // number of comment lines at the start of the file
    right_to_left : bool,
    pub layout    : u32,
    count         : u32,      // number of code-tagged FIGcharacters in this FIGfont
    chars         : HashMap<char, FIGchar>, // actual FIGcharacter definitions for this font
}

impl FIGfont {

    fn new() -> Self {
        let mut font: FIGfont = Default::default();
        font.chars = HashMap::new();
        font
    }

    /// Create a new FIGfont from the specified .flf or .tlf file.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut font = Self::new();
        font.load(path)?;
        Ok(font)
    }

    /// Obtain the FIGchar in this font for the given char.
    pub fn get(&self, ch: char) -> &FIGchar {
        match self.chars.get(&ch) {
            Some(k) => k,
            None    => self.get( if ch == '\t' { ' ' } else { '\0' }),
        }
    } 

    /// Load a font from the given .flf or .tlf file.
    fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<&Self, Error> {
        let file = File::open(path)?;
        let mut f = BufReader::new(&file);

        let mut line = String::new();

        f.read_line(&mut line)?;
        self.parse_header(&line)?;

        // Skip comment lines
        for _ in 0..self.comment_lines {
            line.clear();
            f.read_line(&mut line)?;
        }

        // Define default 0-code character
        self.chars.insert('\0', FIGchar::with_lines(self.height));

        // Load required characters
        for i in (32..127).chain(vec![196, 215, 220, 228, 246, 252, 223]) {
            let mut c = FIGchar::new();
            c.load(&mut f, self.height)?;
            self.chars.insert(char_from_u32(i).unwrap(), c);
        }

        // Load code-tagged characters
        loop {
            line.clear();
            if f.read_line(&mut line)? == 0 {
                break
            }
            let code = match line.split_whitespace().next() {
                Some(val) => val,
                None      => break,
            };

            let mut c = FIGchar::new();
            c.load(&mut f, self.height)?;
            self.chars.insert(char_from_u32(u32_from_str(code)?)?, c);
        }

        Ok(self)
    }

    fn parse_header(&mut self, line: &String) -> Result<&Self, Error> {

        if !line.starts_with("flf2") && !line.starts_with("tlf2") {
            return Err(Error::FontFormat("unsupported font format"));
        }

        let parms = line.split_whitespace().collect::<Vec<&str>>();

        if parms[0].len() < 6 {
            return Err(Error::FontFormat("unsupported font format"));
        }

        self.version       = parms[0].chars().nth(4).unwrap();
        self.hardblank     = parms[0].chars().nth(5).unwrap();
        self.height        = parms[1].parse()?;
        self.baseline      = parms[2].parse()?;
        self.max_length    = parms[3].parse()?;
        self.old_layout    = parms[4].parse()?;
        self.comment_lines = parms[5].parse()?;
        self.right_to_left = parms[6] == "1";
        self.layout        = parms[7].parse()?;
        self.count         = parms[8].parse()?;

        Ok(self)
    }
}

fn char_from_u32(num: u32) -> Result<char, Error> {
    match char::from_u32(num) {
        Some(c) => Ok(c),
        None    => Err(Error::CodeTag(num)),
    }
}

// See https://github.com/rust-lang/rfcs/issues/1098
fn u32_from_str(s: &str) -> Result<u32, Error> {
    let mut s = s.trim();
    let mut radix = 10;

    // return an unused character for translation tables
    if s.starts_with("-") {
        return Ok(1);
    }

    if s.starts_with("0x") || s.starts_with("0X") {
        radix = 16;
        s = &s[2..];
    }

    Ok(u32::from_str_radix(s, radix)?)
}


#[derive(Debug)]
pub struct FIGchar {
    lines: Vec<String>,
}

impl FIGchar {
    fn new() -> Self {
        FIGchar{
            lines: Vec::new(),
        }
    }

    /// Create a new FIGchar using the given set of lines. All lines must be valid UTF-8 strings
    /// and have the same length in characters.
    ///
    /// # Example
    ///
    /// ```
    /// # fn foo() -> Result<(), figdriver::Error> {
    /// let c = figdriver::FIGchar::from_lines(&vec!["123", "456", "789"])?;
    /// let output = format!("{}", c);
    ///
    /// assert_eq!(output, "123\n456\n789\n".to_string());
    /// # Ok(())
    /// # }
    /// # foo();
    /// ```
    pub fn from_lines(lines: &Vec<&str>) -> Result<Self, Error> {
        let mut c = Self::new();
        if !lines.is_empty() {
            let width = lines[0].chars().count();
            for line in lines {
                if line.chars().count() != width {
                    return Err(Error::FontFormat("invalid character width"));
                }
                c.lines.push(line.to_string());
            }
        }
        Ok(c)
    }

    /// Retrieve the lines from this FIGchar.
    ///
    /// # Example
    ///
    /// ```
    /// # fn foo() -> Result<(), figdriver::Error> {
    /// let c = figdriver::FIGchar::from_lines(&vec!["123", "456", "789"])?;
    /// let lines = c.get();
    ///
    /// assert_eq!(lines, vec!["123".to_string(), "456".to_string(), "789".to_string()]);
    /// # Ok(())
    /// # }
    /// # foo();
    /// ```
    pub fn get(&self) -> Vec<String> {
        self.lines.to_owned()
    }

    fn with_lines(num: usize) -> Self {
        let mut c = Self::new();
        (0..num).for_each(|_| c.lines.push("".to_owned()));
        c
    }

    fn load<R: BufRead>(&mut self, f: &mut R, height: usize) -> Result<&Self, Error> {
        let mut line = String::new();
        for i in 0..height {
            line.clear();
            if f.read_line(&mut line).is_err() {
                // read rest of lines
                (i+1..height).for_each(|_| { let _ = f.read_line(&mut line); });
                // If one line fails to load, clear other lines as well
                self.lines.clear();
                (0..height).for_each(|_| self.lines.push("".to_owned()));
                return Ok(self)
            }
            line = line.trim_end().to_string();
            if line.len() < 1 {
                return Err(Error::FontFormat("invalid character width"));
            }
            let mark = line.pop().unwrap();
            self.lines.push(line.trim_end_matches(mark).to_string());
        }

        Ok(self)
    }
}

impl fmt::Display for FIGchar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for l in &self.lines {
            s += l;
            s += "\n";
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let f = FIGchar::from_lines(&vec![ "1  ", " 2 ", "  3" ]).unwrap();
        assert_eq!(format!("{}", f), "1  \n 2 \n  3\n");
    }

    #[test]
    fn test_char_from_u32() {
        assert_eq!(char_from_u32(0x0041).unwrap(), 'A');
        assert_eq!(char_from_u32(0x00C1).unwrap(), 'Á');
    }

    #[test]
    fn test_u32_from_str() {
        assert!(matches!(u32_from_str("0x0041"), Ok(0x41)));
        assert!(matches!(u32_from_str("0x00C1"), Ok(0xC1)));
        assert!(matches!(u32_from_str("  0x41"), Ok(0x41)));
        assert!(matches!(u32_from_str("0X0041"), Ok(0x41)));
        assert!(matches!(u32_from_str("-0x100"), Ok(1)));
        assert!(matches!(u32_from_str("-5"), Ok(1)));
        assert!(u32_from_str("foobar").is_err());
        assert!(u32_from_str("").is_err());
    }

    #[test]
    fn test_font_load() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
        let font = FIGfont::from_path(&path).unwrap();

        let ch = ' ';
        assert_eq!(font.get(ch).get(), vec![r" $",
                                            r" $",
                                            r" $",
                                            r" $",
                                            r" $",
                                            r" $"]);

        let ch = 'A';
        assert_eq!(font.get(ch).get(), vec![r"     _    ",
                                            r"    / \   ",
                                            r"   / _ \  ",
                                            r"  / ___ \ ",
                                            r" /_/   \_\",
                                            r"          "]);

        let ch = char::from_u32(223).unwrap();
        assert_eq!(font.get(ch).get(), vec![r"   ___ ",
                                            r"  / _ \",
                                            r" | |/ /",
                                            r" | |\ \",
                                            r" | ||_/",
                                            r" |_|   "]);

        let ch = char::from_u32(3232).unwrap();
        assert_eq!(font.get(ch).get(), vec![r"   _____)",
                                            r"  /_ ___/",
                                            r"  / _ \  ",
                                            r" | (_) | ",
                                            r" $\___/$ ",
                                            r"         "]);
    }

    #[test]
    fn test_get_tab() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
        let font = FIGfont::from_path(&path).unwrap();

        let ch = '\t';
        assert_eq!(font.get(ch).get(), vec![r" $",
                                            r" $",
                                            r" $",
                                            r" $",
                                            r" $",
                                            r" $"]);
    }

    #[test]
    fn test_char_line_width() {
        let c = FIGchar::from_lines(&vec!["123", "456", "789"]);
        assert!(c.is_ok());

        let c = FIGchar::from_lines(&vec!["123", "456", "7890"]);
        assert!(matches!(c, Err(Error::FontFormat(_))));
    }
}
