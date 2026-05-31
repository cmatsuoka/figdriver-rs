use std::cmp::min;
pub use crate::figfont::{FIGchar, FIGfont};

mod charsmush;
pub mod strsmush;

/// Creates a message written with ASCII-art characters.
///
/// The Smusher adds FIGcharacters to an output buffer and controls how they fit
/// together in a line. Details of how exactly this fitting happens is given by
/// the FIGfont layout mode. Possible layout modes include full-width, kerning (when
/// FIGcharacters are moved closer to each other but without overlapping borders),
/// or smushing (where borders overlap).
#[derive(Debug)]
pub struct Smusher<'a> {
    pub mode      : u32,          // the layout mode
    pub full_width: bool,
    pub right2left: bool,
    font          : &'a FIGfont,
    output        : Vec<String>,
}

impl<'a> Smusher<'a> {
    /// Replace hardblanks in the given lines with spaces.
    pub fn replace_hardblanks(&self, lines: &mut [String]) {
        let hb = self.font.hardblank;
        for line in lines {
            *line = line.replace(&hb.to_string(), " ");
        }
    }

    /// Create a new smusher using the specified FIGfont.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// // Load a FIGfont
    /// let mut font = figdriver::FIGfont::from_path("small.flf")?;
    ///
    /// // Create a smusher using the FIGfont
    /// let mut sm = figdriver::Smusher::new(&font);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(font: &'a FIGfont) -> Self {
        let mut sm = Smusher{
            font,
            mode      : font.layout,
            full_width: font.old_layout == -1,
            right2left: font.right_to_left,
            output    : Vec::new(),
        };
        for _ in 0..sm.font.height {
            sm.output.push(String::new());
        }
        sm
    }

    /// Get the contents of the output buffer, replacing hardblanks with spaces.
    pub fn get(&self) -> Vec<String> {
        self.output.iter().map(|s| s.replace(&self.font.hardblank.to_string(), " ")).collect()
    }

    /// Get the contents of the output buffer, preserving hardblanks for further processing.
    pub fn get_raw(&self) -> Vec<String> {
        self.output.to_vec()
    }

    /// Verify whether output buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.output.len() == 0 || self.output[0].is_empty()
    }

    /// Clear the output buffer.
    pub fn clear(&mut self) {
        for x in &mut self.output {
            x.clear();
        }
    }

    /// Add a string to the output buffer, applying the smushing rules specified in the font
    /// layout.
    ///
    /// Returns the subset of characters that were actually rendered (characters missing
    /// from the font are omitted).
    pub fn push_str(&mut self, s: &str) -> String {
        let mut rendered = String::new();
        for x in s.chars() {
            if self.push(x) {
                rendered.push(x);
            }
        }
        rendered
    }

    /// Add a character to the output buffer, applying the smushing rules specified in the font
    /// layout.
    ///
    /// Returns `true` if the character was rendered, `false` if it was missing from the font
    /// and skipped.
    pub fn push(&mut self, ch: char) -> bool {
        if let Some(fc) = self.font.get(ch) {
            self.output = smush(&self.output, fc, self.font.hardblank, self.full_width, self.mode, self.right2left);
            true
        } else {
            false
        }
    }

    /// Obtain the size, in sub-characters, of any line of the output buffer.
    pub fn len(&self) -> usize {
        let s: &str = &self.output[0];
        s.chars().count()
    }

    /// Limit the size, in sub-characters, of the output buffer. If the buffer is longer than
    /// the specified size, the rightmost sub-characters will be removed.
    pub fn trim(&mut self, width: usize) {
        self.output = trim(&self.output, width);
    }
}

fn amount(output: &[String], c: &FIGchar, hardblank: char, mode: u32, right2left: bool) -> usize {
    let mut amt = 9999;
    for (line, cline) in output.iter().zip(c.get()) {
        amt = min(amt, strsmush::amount(line, cline, hardblank, mode, right2left));
    }
    amt
}

fn trim(output: &[String], width: usize) -> Vec<String> {
    output.iter().map(|line| {
        let s: &str = line;
        let len = s.chars().count();
        let index = s.char_indices().nth(width.min(len)).map(|(i, _)| i).unwrap_or(s.len());
        s[..index].to_string()
    }).collect()
}

fn smush(output: &[String], c: &FIGchar, hardblank: char, full_width: bool, mode: u32, right2left: bool) -> Vec<String> {

    let amt = match full_width {
        true  => 0,
        false => amount(output, c, hardblank, mode, right2left),
    };

    let mut res = Vec::new();

    for (line, cline) in output.iter().zip(c.get()) {
        if right2left {
            res.push(strsmush::smush_rtl(line, cline, amt, hardblank, mode));
        } else {
            res.push(strsmush::smush(line, cline, amt, hardblank, mode));
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! vec_of_strings {
        ( $($x:expr),* ) => (vec![$($x.to_string()),*])
    }

    #[test]
    fn test_amount() {
        let output = vec_of_strings![ "", "", "", "" ];
        let fc = FIGchar::from_lines(&vec![ "   ", "  x", " xx", "xx " ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf, false), 0);

        let output = vec_of_strings![ "", "", "", "" ];
        let fc = FIGchar::from_lines(&vec![ "   ", "  x", " xx", "   " ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf, false), 1);

        let output = vec_of_strings![ "xxx ", "xx  ", "x   ", "    " ];
        let fc = FIGchar::from_lines(&vec![ "   y", "  yy", " yyy", "yyyy" ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf, false), 4);

        let output = vec_of_strings![  "xxxx ", "xxx  ", "xx   ", "x    " ];
        let fc = FIGchar::from_lines(&vec![ "   x", "  xx", " xxx", "xxxx" ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf, false), 5);
    }

    #[test]
    fn test_amount_utf8() {
        let output = vec_of_strings![ "", "", "", "" ];
        let fc = FIGchar::from_lines(&vec![ "   ", "  á", " áá", "   " ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf, false), 1);

        let output = vec_of_strings![ "ááá ", "áá  ", "á   ", "    " ];
        let fc = FIGchar::from_lines(&vec![ "   é", "  éé", " ééé", "éééé" ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf, false), 4);

        let output = vec_of_strings![  "áááá ", "ááá  ", "áá   ", "á    " ];
        let fc = FIGchar::from_lines(&vec![ "   á", "  áá", " ááá", "áááá" ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf, false), 5);
    }

    #[test]
    fn test_trim() {
        let output = vec_of_strings![ "12345", "abcde" ];
        assert_eq!(trim(&output, 3), vec_of_strings![ "123", "abc" ]);
    }

    #[test]
    fn test_trim_utf8() {
        let output = vec_of_strings![ "12345", "áéíóú" ];
        assert_eq!(trim(&output, 3), vec_of_strings![ "123", "áéí" ]);
    }

    // get() replaces hardblank characters with spaces in the output
    #[test]
    fn test_smusher_get_hardblank_replacement() {
        let font = FIGfont::from_path(env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf").unwrap();
        let sm = Smusher::new(&font);
        let output = sm.get();
        for line in &output {
            assert!(!line.contains(font.hardblank));
        }
    }

    // is_empty() reports correct state before and after push
    #[test]
    fn test_smusher_is_empty() {
        let font = FIGfont::from_path(env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf").unwrap();
        let mut sm = Smusher::new(&font);
        assert!(sm.is_empty());
        sm.push('A');
        assert!(!sm.is_empty());
    }

    // clear() resets the smusher to empty, allowing reuse
    #[test]
    fn test_smusher_clear() {
        let font = FIGfont::from_path(env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf").unwrap();
        let mut sm = Smusher::new(&font);
        sm.push('A');
        assert!(!sm.is_empty());
        sm.clear();
        assert!(sm.is_empty());
        sm.push('B');
        assert!(!sm.is_empty());
    }

    // len() tracks character count through push and clear
    #[test]
    fn test_smusher_len() {
        let font = FIGfont::from_path(env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf").unwrap();
        let mut sm = Smusher::new(&font);
        assert_eq!(sm.len(), 0);
        sm.push('A');
        assert!(sm.len() > 0);
        sm.clear();
        assert_eq!(sm.len(), 0);
    }

    // push_str() appends multiple characters at once
    #[test]
    fn test_smusher_push_str() {
        let font = FIGfont::from_path(env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf").unwrap();
        let mut sm = Smusher::new(&font);
        sm.push_str("AB");
        let output = sm.get();
        assert!(!output[0].is_empty());
    }

    // trim() truncates the rendered output to the given width
    #[test]
    fn test_smusher_trim() {
        let font = FIGfont::from_path(env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf").unwrap();
        let mut sm = Smusher::new(&font);
        sm.push_str("ABC");
        let len_before = sm.len();
        sm.trim(2);
        assert_eq!(sm.len(), 2);
        assert!(sm.len() < len_before);
    }

    // full_width mode inserts space between characters
    #[test]
    fn test_smusher_full_width() {
        let font = FIGfont::from_path(env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf").unwrap();
        let mut sm = Smusher::new(&font);
        sm.full_width = true;
        sm.push('A');
        sm.push('B');
        let output = sm.get();
        assert!(output[0].chars().count() > 1);
    }

    // Smusher picks up right_to_left from font header
    #[test]
    fn test_smusher_right2left_from_font() {
        let mut font = FIGfont::default();
        font.height = 3;
        font.right_to_left = false;
        let sm = Smusher::new(&font);
        assert!(!sm.right2left);

        font.right_to_left = true;
        let sm = Smusher::new(&font);
        assert!(sm.right2left);
    }

    // Rendering a string with a missing character produces the same output as
    // rendering the string with that character removed.
    #[test]
    fn test_smusher_skips_missing_char() {
        let font = FIGfont::from_path(env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf").unwrap();

        let mut sm_with_unknown = Smusher::new(&font);
        sm_with_unknown.push_str("A\u{1F600}B");
        let output_with_unknown = sm_with_unknown.get();

        let mut sm_without_unknown = Smusher::new(&font);
        sm_without_unknown.push_str("AB");
        let output_without_unknown = sm_without_unknown.get();

        assert_eq!(output_with_unknown, output_without_unknown);
    }
}
