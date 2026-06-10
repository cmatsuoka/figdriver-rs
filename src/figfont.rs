use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek};
use std::path::Path;
use crate::Error;
use crate::zip::{is_zip, decompress_zip, is_zip_reader, decompress_zip_reader};

/// Horizontal smush rule 1: smushes identical sub-characters together.
pub const SMUSH_EQUAL    : u32 = 1;
/// Horizontal smush rule 2: replaces underscore with vertical, angled, or bracket characters.
pub const SMUSH_UNDERLINE: u32 = 2;
/// Horizontal smush rule 3: uses a hierarchy (|, /, [], {}, (), <>) with later classes winning.
pub const SMUSH_HIERARCHY: u32 = 4;
/// Horizontal smush rule 4: replaces opposing bracket/brace/paren pairs with a vertical bar.
pub const SMUSH_PAIR     : u32 = 8;
/// Horizontal smush rule 5: smushes / and \ into |, \ and / into Y, > and < into X.
pub const SMUSH_BIGX     : u32 = 16;
/// Horizontal smush rule 6: smushes two hardblanks into a single hardblank.
pub const SMUSH_HARDBLANK: u32 = 32;
/// Horizontal fitting (kerning) by default; characters are placed with minimal overlap.
pub const SMUSH_KERN     : u32 = 64;
/// Horizontal smushing by default; overrides SMUSH_KERN when both are set.
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
    /// The sub-character used to represent hardblanks in FIGcharacter data.
    /// Hardblanks are invisible placeholders that participate in smushing
    /// and kerning, displayed as a space in the output.
    pub hardblank     : char,
    /// Number of lines in each FIGcharacter.
    pub height        : usize,
    baseline      : usize,    // number of lines from the baseline of a FIGcharacter
    max_length    : usize,    // maximum length of any line describing a FIGcharacter
    /// Legacy layout parameter for backward compatibility.
    /// -1 means full-width layout, 0 means horizontal fitting (kerning),
    /// and positive values are bitmasks of horizontal smushing rules.
    pub old_layout: i32,
    comment_lines : usize,    // number of comment lines at the start of the file
    /// Default print direction: true for right-to-left, false for left-to-right.
    pub right_to_left : bool,
    /// Full layout parameter describing horizontal and vertical layout modes
    /// and smushing rules as a bitmask. See the FIGfont specification for
    /// the complete list of code values (0 to 32767).
    pub layout    : u32,
    count         : u32,      // number of code-tagged FIGcharacters in this FIGfont
    chars         : HashMap<i32, FIGchar>, // actual FIGcharacter definitions for this font
}

impl FIGfont {

    /// Create a new FIGfont from the specified .flf or .tlf file.
    /// Automatically detects and decompresses ZIP-compressed fonts.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut font = Self::default();
        font.load(path)?;
        Ok(font)
    }

    /// Create a new FIGfont from a reader.
    ///
    /// Automatically detects and decompresses ZIP-compressed fonts.
    /// The reader must implement [`Seek`] because ZIP archive parsing
    /// requires seeking. Suitable for reading from files, memory buffers
    /// (e.g., `std::io::Cursor<Vec<u8>>`), or WASM `ArrayBuffer` data.
    pub fn from_reader<R: Read + Seek>(mut reader: R) -> Result<Self, Error> {
        let mut font = Self::default();

        if is_zip_reader(&mut reader)? {
            let mut cursor = decompress_zip_reader(reader)
                .map_err(|_| Error::FontFormat("invalid ZIP archive"))?;
            font.load_from_reader(&mut cursor)?;
            return Ok(font);
        }

        font.load_from_reader(&mut BufReader::new(reader))?;
        Ok(font)
    }

    /// Obtain the FIGchar in this font for the given character code.
    ///
    /// Returns `None` if neither the code nor its fallback (code 0)
    /// is defined in the font, per spec: "If there is no FIGcharacter 0,
    /// nothing will be printed." Code 9 (tab) falls back to code 32 (space).
    pub fn get(&self, code: i32) -> Option<&FIGchar> {
        match self.chars.get(&code) {
            Some(k) => Some(k),
            None => {
                let fallback = if code == 9 { 32i32 } else { 0i32 };
                self.chars.get(&fallback)
            }
        }
    } 

    /// Load a font from the given .flf or .tlf file.
    /// Automatically detects and decompresses ZIP-compressed fonts.
    fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<&Self, Error> {
        let path = path.as_ref();

        if is_zip(path) {
            let mut reader = decompress_zip(path).map_err(|_| Error::FontFormat("invalid ZIP archive"))?;
            return self.load_from_reader(&mut reader);
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(&file);
        self.load_from_reader(&mut reader)
    }

    /// Load font data from a reader.
    fn load_from_reader<R: BufRead>(&mut self, f: &mut R) -> Result<&Self, Error> {
        let mut line = String::new();

        f.read_line(&mut line)?;
        self.parse_header(&line)?;

        // Skip comment lines
        for _ in 0..self.comment_lines {
            line.clear();
            f.read_line(&mut line)?;
        }

        // Load required characters
        for i in (32..127).chain(vec![196, 214, 220, 228, 246, 252, 223]) {
            let mut c = FIGchar::new();
            c.load(f, self.height)?;
            self.chars.insert(i, c);
        }

        // Load code-tagged characters
        loop {
            line.clear();
            if f.read_line(&mut line)? == 0 {
                break
            }
            let Some(code) = line.split_whitespace().next() else {
                break;
            };

            let mut c = FIGchar::new();
            c.load(f, self.height)?;
            self.chars.insert(i32_from_str(code)?, c);
        }

        Ok(self)
    }

    fn parse_header(&mut self, line: &str) -> Result<&Self, Error> {

        if !line.starts_with("flf2") && !line.starts_with("tlf2") {
            return Err(Error::FontFormat("unsupported font format"));
        }

        let parms = line.split_whitespace().collect::<Vec<&str>>();

        if parms[0].chars().count() < 6 {
            return Err(Error::FontFormat("unsupported font format"));
        }

        self.version       = parms[0].chars().nth(4).ok_or(Error::FontFormat("invalid font header"))?;
        self.hardblank     = parms[0].chars().nth(5).ok_or(Error::FontFormat("invalid font header"))?;
        self.height        = parms[1].parse()?;
        self.baseline      = parms[2].parse()?;
        self.max_length    = parms[3].parse()?;
        self.old_layout    = parms[4].parse()?;
        self.comment_lines = parms[5].parse()?;
        self.right_to_left = parms.get(6).map(|s| *s == "1").unwrap_or(false);
        self.layout        = parms.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
        self.count         = parms.get(8).and_then(|s| s.parse().ok()).unwrap_or(0);

        Ok(self)
    }
}

// See https://github.com/rust-lang/rfcs/issues/1098
fn i32_from_str(s: &str) -> Result<i32, Error> {
    let s = s.trim();
    let negative = s.starts_with("-");
    let s = s.strip_prefix('-').unwrap_or(s);

    let (radix, digits) = if s.starts_with("0x") || s.starts_with("0X") {
        (16, &s[2..])
    } else if s.starts_with('0') && s.len() > 1 {
        (8, s)
    } else {
        (10, s)
    };

    let value = u32::from_str_radix(digits, radix)? as i32;
    let value = if negative { -value } else { value };

    if value == -1 {
        Err(Error::CodeTag(-1))
    } else {
        Ok(value)
    }
}


/// A single FIGfont character, represented as a set of equally-wide text lines.
///
/// Each line is a row of the character glyph. The width in characters is
/// determined by the length of the first line; all subsequent lines must
/// have the same character width.
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
    /// let c = figdriver::FIGchar::from_lines(&["123", "456", "789"])?;
    /// let output = format!("{}", c);
    ///
    /// assert_eq!(output, "123\n456\n789\n".to_string());
    /// # Ok(())
    /// # }
    /// # foo();
    /// ```
    pub fn from_lines(lines: &[&str]) -> Result<Self, Error> {
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
    /// let c = figdriver::FIGchar::from_lines(&["123", "456", "789"])?;
    /// let lines = c.get();
    ///
    /// assert_eq!(lines, &["123".to_string(), "456".to_string(), "789".to_string()]);
    /// # Ok(())
    /// # }
    /// # foo();
    /// ```
    pub fn get(&self) -> &[String] {
        &self.lines
    }

    fn load<R: BufRead>(&mut self, f: &mut R, height: usize) -> Result<&Self, Error> {
        let mut line = String::new();
        for i in 0..height {
            line.clear();
            if f.read_line(&mut line).is_err() {
                // read rest of lines
                for _ in (i+1)..height {
                    let _ = f.read_line(&mut line);
                }
                // If one line fails to load, clear other lines as well
                self.lines.clear();
                for _ in 0..height {
                    self.lines.push(String::new());
                }
                return Ok(self)
            }
            line = line.trim_end().to_string();
            if line.is_empty() {
                return Err(Error::FontFormat("invalid character width"));
            }
            let mark = line.pop().expect("line is non-empty");
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
        let f = FIGchar::from_lines(&["1  ", " 2 ", "  3"]).unwrap();
        assert_eq!(format!("{}", f), "1  \n 2 \n  3\n");
    }

    #[test]
    fn test_i32_from_str() {
        assert!(matches!(i32_from_str("0x0041"), Ok(0x41)));
        assert!(matches!(i32_from_str("0x00C1"), Ok(0xC1)));
        assert!(matches!(i32_from_str("  0x41"), Ok(0x41)));
        assert!(matches!(i32_from_str("0X0041"), Ok(0x41)));
        assert!(matches!(i32_from_str("-0x100"), Ok(-0x100)));
        assert!(matches!(i32_from_str("-5"), Ok(-5)));
        assert!(i32_from_str("-1").is_err());
        assert!(matches!(i32_from_str("-0"), Ok(0)));
        assert!(matches!(i32_from_str("42"), Ok(42)));
        assert!(i32_from_str("foobar").is_err());
        assert!(i32_from_str("").is_err());
        assert!(matches!(i32_from_str("0755"), Ok(0o755)));
        assert!(matches!(i32_from_str("010"), Ok(0o10)));
        assert!(matches!(i32_from_str("-0755"), Ok(-0o755)));
        assert!(matches!(i32_from_str("-010"), Ok(-0o10)));
    }

    #[test]
    fn test_font_load() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
        let font = FIGfont::from_path(&path).unwrap();

        let ch = ' ' as i32;
        assert_eq!(font.get(ch).unwrap().get(), &[
            " $".to_string(),
            " $".to_string(),
            " $".to_string(),
            " $".to_string(),
            " $".to_string(),
            " $".to_string(),
        ]);

        let ch = 'A' as i32;
        assert_eq!(font.get(ch).unwrap().get(), &[
            r"     _    ".to_string(),
            r"    / \   ".to_string(),
            r"   / _ \  ".to_string(),
            r"  / ___ \ ".to_string(),
            r" /_/   \_\".to_string(),
            r"          ".to_string(),
        ]);

        assert_eq!(font.get(223).unwrap().get(), &[
            r"   ___ ".to_string(),
            r"  / _ \".to_string(),
            r" | |/ /".to_string(),
            r" | |\ \".to_string(),
            r" | ||_/".to_string(),
            r" |_|   ".to_string(),
        ]);

        assert_eq!(font.get(3232).unwrap().get(), &[
            r"   _____)".to_string(),
            r"  /_ ___/".to_string(),
            r"  / _ \  ".to_string(),
            r" | (_) | ".to_string(),
            r" $\___/$ ".to_string(),
            r"         ".to_string(),
        ]);
    }

    #[test]
    fn test_get_tab() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
        let font = FIGfont::from_path(&path).unwrap();

    let ch = '\t' as i32;
        assert_eq!(font.get(ch).unwrap().get(), &[
            " $".to_string(),
            " $".to_string(),
            " $".to_string(),
            " $".to_string(),
            " $".to_string(),
            " $".to_string(),
        ]);
    }

    #[test]
    fn test_char_line_width() {
        let c = FIGchar::from_lines(&["123", "456", "789"]);
        assert!(c.is_ok());

        let c = FIGchar::from_lines(&["123", "456", "7890"]);
        assert!(matches!(c, Err(Error::FontFormat(_))));
    }

    // FIGchar from empty slice produces an empty character
    #[test]
    fn test_figchar_from_lines_empty() {
        let c = FIGchar::from_lines(&[]).unwrap();
        assert!(c.get().is_empty());
    }

    // Display impl for empty FIGchar produces empty string
    #[test]
    fn test_figchar_display_empty() {
        let c = FIGchar::from_lines(&[]).unwrap();
        assert_eq!(format!("{}", c), "");
    }

    // get(0) returns None when font does not define FIGcharacter 0
    // Per spec: "If there is no FIGcharacter 0, nothing will be printed."
    #[test]
    fn test_font_get_missing_null() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
        let font = FIGfont::from_path(&path).unwrap();
        assert!(font.get(0i32).is_none());
    }

    // get() for unknown codepoint returns None when code 0 is not defined
    #[test]
    fn test_font_get_unknown_char_none() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
        let font = FIGfont::from_path(&path).unwrap();
        assert!(font.get(0x1F600).is_none());
    }

    // Tab (code 9) falls back to space (code 32)
    #[test]
    fn test_font_get_tab_fallback() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
        let font = FIGfont::from_path(&path).unwrap();
        let space = font.get(32i32);
        let tab = font.get(9i32);
        assert!(space.is_some());
        assert!(tab.is_some());
        assert_eq!(space.unwrap().get(), tab.unwrap().get());
    }

    // Loading a ZIP-compressed font produces the same result as the uncompressed file
    #[test]
    fn test_font_load_zip() {
        let path = format!(
            "{}/tests/fixtures/standard.flf.zip",
            env!("CARGO_MANIFEST_DIR")
        );
        let font = FIGfont::from_path(&path).unwrap();
        assert_eq!(font.height, 6);
        assert_eq!(font.hardblank, '$');
        assert!(font.get('A' as i32).is_some());
        assert!(font.get(0x00C4).is_some());
    }

    // Loaded font exposes correct height, hardblank, and layout values
    #[test]
    fn test_font_fields() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
        let font = FIGfont::from_path(&path).unwrap();
        assert_eq!(font.height, 6);
        assert_eq!(font.hardblank, '$');
        assert!(font.layout > 0);
    }

    // Loading a TLF font (tlf2 magic) with UTF-8 box-drawing sub-characters
    #[test]
    fn test_tlf_font_load() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/tests/fixtures/emboss.tlf";
        let font = FIGfont::from_path(&path).unwrap();
        assert_eq!(font.height, 3);
        assert_eq!(font.hardblank, '\u{7f}');
        assert_eq!(font.layout, 0);

        assert_eq!(font.get('A' as i32).unwrap().get(), &[
            "\u{250f}\u{2501}\u{2503}".to_string(),
            "\u{250f}\u{2501}\u{2503}".to_string(),
            "\u{251b} \u{251b}".to_string(),
        ]);

        assert_eq!(font.get('H' as i32).unwrap().get(), &[
            "\u{2503} \u{2503}".to_string(),
            "\u{250f}\u{2501}\u{2503}".to_string(),
            "\u{251b} \u{251b}".to_string(),
        ]);

        assert_eq!(font.get('I' as i32).unwrap().get(), &[
            "\u{251b}".to_string(),
            "\u{2503}".to_string(),
            "\u{251b}".to_string(),
        ]);
    }

    // Older font header with only 6 required params (no print_direction, full_layout, codetag_count)
    #[test]
    fn test_header_missing_optional_params() {
        let mut font = FIGfont::default();
        // 6 fields: identifier height baseline max_length old_layout comment_lines
        let short_header = "flf2a$ 6 2 30 -1 0";
        font.parse_header(short_header).unwrap();
        assert_eq!(font.height, 6);
        assert_eq!(font.baseline, 2);
        assert_eq!(font.max_length, 30);
        assert_eq!(font.old_layout, -1);
        assert_eq!(font.comment_lines, 0);
        assert!(!font.right_to_left);
        assert_eq!(font.layout, 0);
        assert_eq!(font.count, 0);
    }

    // Header with optional params set to non-default values
    #[test]
    fn test_header_with_optional_params() {
        let mut font = FIGfont::default();
        // 10 fields: identifier height baseline max_length old_layout comment_lines print_dir layout count
        let full_header = "flf2a$ 6 2 30 -1 0 1 255 42";
        font.parse_header(full_header).unwrap();
        assert_eq!(font.height, 6);
        assert!(font.right_to_left);
        assert_eq!(font.layout, 255);
        assert_eq!(font.count, 42);
    }

    // Verify required German character codes for umlauted "A", "O",
    // "U", "a", "o" and "u"; and also "ess-zed".
    #[test]
    fn test_required_deutsch_codes() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
        let font = FIGfont::from_path(&path).unwrap();

        let deutsch_chars = [
            ('\u{00C4}', 196),
            ('\u{00D6}', 214),
            ('\u{00DC}', 220),
            ('\u{00E4}', 228),
            ('\u{00F6}', 246),
            ('\u{00FC}', 252),
            ('\u{00DF}', 223),
        ];
        for (_ch, code) in &deutsch_chars {
            assert!(
                font.chars.contains_key(code),
                "Missing required Deutsch character U+{:04X} (code {})",
                *code, *code
            );
        }
    }
}
