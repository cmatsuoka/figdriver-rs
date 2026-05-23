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
            right2left: false,
            output    : Vec::new(),
        };
        for _ in 0..sm.font.height {
            sm.output.push(String::new());
        }
        sm
    }

    /// Get the contents of the output buffer.
    pub fn get(&self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        for line in &self.output {
            res.push(line.replace(&self.font.hardblank.to_string(), " "));
        }
        res
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
    pub fn push_str(&mut self, s: &str) {
        for x in s.chars() {
            self.push(x);
        }
    }

    /// Add a character to the output buffer, applying the smushing rules specified in the font
    /// layout.
    pub fn push(&mut self, ch: char) {
        let fc = self.font.get(ch);
        self.output = smush(&self.output, fc, self.font.hardblank, self.full_width, self.mode);
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

fn amount(output: &[String], c: &FIGchar, hardblank: char, mode: u32) -> usize {
    let mut amt = 9999;
    for (line, cline) in output.iter().zip(c.get()) {
        amt = min(amt, strsmush::amount(line, cline, hardblank, mode));
    }
    amt
}

fn trim(output: &[String], width: usize) -> Vec<String> {
    output.iter().map(|line| {
        let s: &str = &line;
        let index = s.char_indices().nth(width).unwrap().0;
        s[..index].to_string()
    }).collect()
}

fn smush(output: &[String], c: &FIGchar, hardblank: char, full_width: bool, mode: u32) -> Vec<String> {

    let amt = match full_width {
        true  => 0,
        false => amount(&output, c, hardblank, mode),
    };

    let mut res = Vec::new();

    for (line, cline) in output.iter().zip(c.get()) {
        res.push(strsmush::smush(line, cline, amt, hardblank, mode));
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
        assert_eq!(amount(&output, &fc, '$', 0xbf), 0);

        let output = vec_of_strings![ "", "", "", "" ];
        let fc = FIGchar::from_lines(&vec![ "   ", "  x", " xx", "   " ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf), 1);

        let output = vec_of_strings![ "xxx ", "xx  ", "x   ", "    " ];
        let fc = FIGchar::from_lines(&vec![ "   y", "  yy", " yyy", "yyyy" ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf), 4);

        let output = vec_of_strings![  "xxxx ", "xxx  ", "xx   ", "x    " ];
        let fc = FIGchar::from_lines(&vec![ "   x", "  xx", " xxx", "xxxx" ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf), 5);
    }

    #[test]
    fn test_amount_utf8() {
        let output = vec_of_strings![ "", "", "", "" ];
        let fc = FIGchar::from_lines(&vec![ "   ", "  á", " áá", "   " ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf), 1);

        let output = vec_of_strings![ "ááá ", "áá  ", "á   ", "    " ];
        let fc = FIGchar::from_lines(&vec![ "   é", "  éé", " ééé", "éééé" ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf), 4);

        let output = vec_of_strings![  "áááá ", "ááá  ", "áá   ", "á    " ];
        let fc = FIGchar::from_lines(&vec![ "   á", "  áá", " ááá", "áááá" ]).unwrap();
        assert_eq!(amount(&output, &fc, '$', 0xbf), 5);
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
}
