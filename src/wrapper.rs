use crate::Error;
use crate::Smusher;
use crate::control::Control;
use crate::smusher::display_width;

/// Horizontal alignment of rendered ASCII-art output within the wrapper width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    /// Left-align text; padding is added to the right.
    Left,
    /// Right-align text; padding is added to the left.
    Right,
    /// Center text; padding is distributed evenly on both sides.
    Center,
}

/// Render smushed ASCII-art characters with word wrapping.
///
/// Wrapper receives string or character input and renders the corresponding
/// FIGcharacters if the output text fits inside the maximum width specified on
/// creation. The wrapper will flush the output buffer earlier if the line is
/// too long, thus producing multiple "lines" of output text.
#[derive(Debug)]
pub struct Wrapper<'a> {
    sm                   : Smusher<'a>,      // the FIGcharacter smusher
    buffer               : Vec<i32>,         // buffer to keep our input codes
    pending_space        : Option<Vec<i32>>, // accumulated whitespace codes to commit with next word
    max_width            : usize,            // terminal width
    align                : Align,            // text alignment
    had_trailing_newline : bool,             // tracks if last write_paragraph had trailing newline
    control              : Control,          // control for decoding input bytes
}

impl<'a> Wrapper<'a> {
    /// Create a new wrapper using the specified Smusher, Control, terminal width, and alignment.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a smusher using the specified FIGfont
    /// let font = figdriver::FIGfont::from_path("small.flf")?;
    /// let sm = figdriver::Smusher::new(&font);
    ///
    /// // Create a line wrapper using our smusher and maximum width of 80 columns
    /// let mut wr = figdriver::Wrapper::new(sm, figdriver::Control::default(), 80, figdriver::Align::Left);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(sm: Smusher<'a>, control: Control, max_width: usize, align: Align) -> Self {
        Wrapper{
            sm,
            max_width,
            buffer                : Vec::new(),
            align,
            pending_space         : None,
            had_trailing_newline  : false,
            control,
        }
    }

    /// Return the current alignment.
    pub fn align(&self) -> Align {
        self.align
    }

    /// Return the maximum width limit.
    pub fn max_width(&self) -> usize {
        self.max_width
    }

    /// Clear the output buffer.
    pub fn clear(&mut self) {
        self.sm.clear();
        self.buffer.clear();
        self.pending_space = None;
        self.had_trailing_newline = false;
    }

    /// Retrieve the output buffer lines.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a new wrapper
    /// let mut font = figdriver::FIGfont::from_path("small.flf")?;
    /// let mut wr = figdriver::Wrapper::new(figdriver::Smusher::new(&font), figdriver::Control::default(), 80, figdriver::Align::Left);
    ///
    /// // Add a string to the output buffer
    /// wr.push_str("hello")?;
    ///
    /// // Get and print the current output buffer contents
    /// for line in &wr.get() {
    ///     println!("{}", line);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&mut self) -> Vec<String> {
        // Commit pending whitespace before returning output
        if let Some(sp) = self.pending_space.take() {
            self.push_codes(&sp).ok();
        }

        if self.width() > self.max_width {
            self.sm.trim(self.max_width);
        }

        let mut v = self.sm.get_raw();

        // Pad the block to its own widest line (figlet convention), then replace
        // hardblanks with spaces (figlet never trims output).
        let max_w = v.iter().map(|l| display_width(l)).max().unwrap_or(0);
        for line in &mut v {
            let len = display_width(line);
            if len < max_w {
                line.extend(std::iter::repeat_n(' ', max_w - len));
            }
        }
        self.sm.replace_hardblanks(&mut v);

        let w = self.max_width.saturating_sub(max_w);

        match self.align {
            Align::Left   => v.to_vec(),
            Align::Center => add_pad(&v, w / 2 + w % 2),
            Align::Right  => add_pad(&v, w),
        }
    }

    /// Get the width in screen columns of the current output buffer.
    pub fn width(&self) -> usize {
        self.sm.width()
    }

    /// Verify whether the output buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.sm.is_empty()
    }

    /// Add a character code to the output buffer.
    ///
    /// # Errors
    ///
    /// If adding the code results in a line wider than the maximum number of columns,
    /// the code is not added to the output buffer and a LineFull error is returned.
    fn push_code(&mut self, code: i32) -> Result<(), Error> {
        let rendered = self.sm.push_code(code);

        if self.sm.width() > self.max_width {
            self.sm.clear();
            for &c in &self.buffer {
                self.sm.push_code(c);
            }
            return Err(Error::LineFull)
        }

        if rendered {
            self.buffer.push(code);
        }
        Ok(())
    }

    /// Add character codes to the output buffer.
    ///
    /// # Errors
    ///
    /// If adding the codes results in a line wider than the maximum number of columns,
    /// the codes are not added to the output buffer and a LineFull error is returned.
    fn push_codes(&mut self, codes: &[i32]) -> Result<(), Error> {
        let buf_len = self.buffer.len();
        for &code in codes {
            let rendered = self.sm.push_code(code);
            if rendered {
                self.buffer.push(code);
            }
        }

        if self.sm.width() > self.max_width {
            self.sm.clear();
            self.buffer.truncate(buf_len);
            for &c in &self.buffer {
                self.sm.push_code(c);
            }
            return Err(Error::LineFull)
        }

        Ok(())
    }

    /// Add a string to the output buffer.
    ///
    /// # Errors
    ///
    /// If adding the string results in a line wider than the maximum number of columns,
    /// the string is not added to the output buffer and a LineFull error is returned.
    pub fn push_str(&mut self, s: &str) -> Result<(), Error> {
        self.push_codes(&codes_from_str(s))
    }

    /// Add a character to the output buffer.
    ///
    /// # Errors
    ///
    /// If adding the character results in a line wider than the maximum number of columns,
    /// the character is not added to the output buffer and a LineFull error is returned.
    pub fn push(&mut self, ch: char) -> Result<(), Error> {
        self.push_code(ch as i32)
    }

    /// Add character codes to the output buffer, wrapping them if necessary.
    ///
    /// If the new codes cause the output to be wider than the maximum width, the current
    /// buffer contents (if any) will be passed to the flush callback, the buffer will be
    /// cleared, and the new codes will be added to the buffer. If the codes are wider
    /// than the output buffer, they will be wrapped at code level.
    ///
    /// Explicit newline codes (10) in the input force a flush of the current
    /// buffer, starting a new output line (figfont.txt lines 1617-1621).
    /// Carriage return codes (13) are normalized to newline (10).
    fn wrap_codes(&mut self, codes: &[i32], flush: &dyn Fn(&[String])) {
        // Normalize CR to LF: convert \r\n -> \n and bare \r -> \n
        let normalized: Vec<i32> = normalize_newlines(codes);

        if normalized.contains(&10) {
            let mut parts = normalized.split(|&c| c == 10);
            let first = parts.next().unwrap();
            self.wrap_segment(first, flush);
            for segment in parts {
                flush(&self.get());
                self.clear();
                if segment.is_empty() {
                    continue;
                }
                self.wrap_segment(segment, flush);
            }
            return;
        }

        self.wrap_segment(&normalized, flush);
    }

    /// Add a string to the output buffer, wrapping it if necessary.
    ///
    /// If the new string causes the output to be wider than the maximum width, the current
    /// buffer contents (if any) will be passed to the flush callback, the buffer will be
    /// cleared, and the new string will be added to the buffer. If the string is wider
    /// than the output buffer, it will be wrapped at character level.
    ///
    /// Explicit newline characters ('\n') in the input force a flush of the current
    /// buffer, starting a new output line (figfont.txt lines 1617-1621).
    #[allow(dead_code)]
    fn wrap_str(&mut self, s: &str, flush: &dyn Fn(&[String])) {
        self.wrap_codes(&codes_from_str(s), flush);
    }

    /// Internal wrapper logic for a single segment (no newlines).
    fn wrap_segment(&mut self, codes: &[i32], flush: &dyn Fn(&[String])) {
        let all_space = codes.iter().all(|&c| is_whitespace_code(c));

        // Handle whitespace codes per spec (figfont.txt lines 1623-1633):
        // - At wrap points: discard all blanks until next non-blank character
        // - At input start or after linebreak: preserve blanks as FIGcharacters
        // - Between words: accumulate whitespace, commit when next word arrives
        if all_space {
            if self.buffer.is_empty() {
                self.push_codes(codes).ok();
                return;
            }
            self.pending_space.get_or_insert_with(Vec::new).extend_from_slice(codes);
            return;
        }

        let space = self.pending_space.take();
        let commit_space = space.is_some() && !self.buffer.is_empty();

        // Try to commit accumulated whitespace before adding the word.
        // Save buffer state before spaces so that, if the subsequent word push
        // fails, the flush outputs content without trailing whitespace (spec
        // says blanks at wrap points are discarded).
        if commit_space {
            let sp = space.unwrap();
            let pre_space_buffer = self.buffer.clone();
            if self.push_codes(&sp).is_err() {
                flush(&self.get());
                self.clear();
                if self.push_codes(codes).is_err() {
                    self.wrap_word(codes, flush);
                }
                return;
            } else if self.push_codes(codes).is_err() {
                // Word doesn't fit after space. Flush without trailing space.
                self.sm.clear();
                self.buffer = pre_space_buffer.clone();
                for &c in &self.buffer {
                    self.sm.push_code(c);
                }
                flush(&self.get());
                self.clear();
                if self.push_codes(codes).is_err() {
                    self.wrap_word(codes, flush);
                }
                return;
            } else {
                return;
            }
        }

        // Try codes on current buffer; wrap at code level if needed.
        if self.push_codes(codes).is_err() {
            if !self.buffer.is_empty() {
                flush(&self.get());
                self.clear();
            }
           if self.push_codes(codes).is_err() {
                    self.wrap_word(codes, flush);
            }
        }
    }

    /// Render a string in line mode.
    ///
    /// Decodes the string using the control, clears the buffer, tokenizes and wraps,
    /// and flushes if buffer non-empty. Returns the rendered output lines.
    pub fn write_line(&mut self, s: &str, print_fn: &dyn Fn(&[String])) -> Vec<String> {
        let codes = self.control.decode_bytes(s.as_bytes());
        self.clear();
        self.write_tokens(&codes, print_fn);
        if self.is_empty() {
            return Vec::new();
        }
        self.get()
    }

    /// Render a string in paragraph mode.
    ///
    /// Accepts a string that may include trailing newlines. Decodes using the
    /// control, strips trailing newlines internally, and tracks them for EOF
    /// flush. Blank line or leading space causes a hard break (flush). Otherwise
    /// joins with space. Flushed lines are passed to `print_fn`.
    pub fn write_paragraph(&mut self, s: &str, print_fn: &dyn Fn(&[String])) {
        self.had_trailing_newline = false;
        let codes = self.control.decode_bytes(s.as_bytes());
        let mut stripped = codes.as_slice();
        while let Some(&last) = stripped.last() {
            if last == 10 || last == 13 {
                stripped = &stripped[..stripped.len() - 1];
                self.had_trailing_newline = true;
            } else {
                break;
            }
        }
        if !self.is_empty() {
            if is_blank_codes(stripped) || (stripped.first() == Some(&32)) {
                print_fn(&self.get());
                self.clear();
            } else {
                self.wrap_codes(&[32], print_fn);
            }
        }
        if !is_blank_codes(stripped) {
            self.write_tokens(stripped, print_fn);
        }
    }

    /// Flush paragraph buffer at EOF.
    ///
    /// If buffer empty: no-op.
    /// If last line had a trailing newline: pushes space to preserve trailing whitespace, then flushes.
    /// Prints flushed lines via `print_fn`.
    pub fn flush_paragraph(&mut self, print_fn: &dyn Fn(&[String])) {
        if self.is_empty() {
            return;
        }
        if self.had_trailing_newline {
            self.push_str(" ").ok();
        }
        print_fn(&self.get());
    }

    /// Internal tokenization: groups consecutive codes by type and wraps each token.
    ///
    /// Whitespace codes and non-whitespace codes are separated. Within whitespace,
    /// spaces (code 32) are separated from other whitespace codes.
    fn write_tokens(&mut self, codes: &[i32], print_fn: &dyn Fn(&[String])) {
        let mut indices = codes.iter().enumerate().peekable();
        let mut start = 0;

        while let Some((_i, &code)) = indices.next() {
            let is_ws = is_whitespace_code(code);
            let is_space = code == 32;
            while let Some(&(_j, &next_code)) = indices.peek() {
                let next_ws = is_whitespace_code(next_code);
                let next_space = next_code == 32;
                if next_ws != is_ws || (is_ws && next_space != is_space) {
                    break;
                }
                indices.next();
            }
            let end = indices.peek().map_or(codes.len(), |(idx, _)| *idx);
            self.wrap_codes(&codes[start..end], print_fn);
            start = end;
        }
    }

    /// Add codes to the output buffer, breaking them if necessary.
    ///
    /// Add the codes one by one. If a new code causes the
    /// output to be wider than the maximum width, the current buffer contents (if any) will
    /// be passed to the flush callback, the buffer will be cleared, and the new code will be
    /// added to the buffer. If the code is wider than the maximum width, it
    /// will be added without any additional processing.
    fn wrap_word(&mut self, codes: &[i32], flush: &dyn Fn(&[String])) {
        for &code in codes {
            if self.push_code(code).is_err() {
                if !self.buffer.is_empty() {
                    flush(&self.get());
                    self.clear();
                }
                if self.sm.push_code(code) {
                    self.buffer.push(code);
                }
            }
        }
    }
}

/// Check whether a code represents a whitespace character
/// (space 32, tab 9, LF 10, VT 11, FF 12, CR 13).
pub fn is_whitespace_code(code: i32) -> bool {
    code == 32 || code == 9 || (10..=13).contains(&code)
}

/// Check whether all codes in a slice are whitespace.
fn is_blank_codes(codes: &[i32]) -> bool {
    codes.iter().all(|&code| is_whitespace_code(code))
}

/// Normalize carriage return codes to newline codes.
/// \r\n (13, 10) becomes \n (10), bare \r (13) becomes \n (10).
fn normalize_newlines(codes: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(codes.len());
    let mut i = 0;
    while i < codes.len() {
        if codes[i] == 13 && i + 1 < codes.len() && codes[i + 1] == 10 {
            result.push(10);
            i += 2;
        } else if codes[i] == 13 {
            result.push(10);
            i += 1;
        } else {
            result.push(codes[i]);
            i += 1;
        }
    }
    result
}

/// Convert a UTF-8 string to character codes.
fn codes_from_str(s: &str) -> Vec<i32> {
    s.chars().map(|c| c as i32).collect()
}

fn add_pad(v: &[String], pad_size: usize) -> Vec<String> {
    let p: String = (0..pad_size).map(|_| " ").collect();
    v.iter().map(|x| p.clone() + x).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FIGfont;
    use std::cell::RefCell;

    macro_rules! vec_string {
        ( $($x:expr),* ) => (vec![$($x.to_string()),*])
    }

    fn test_font() -> Result<FIGfont, Error> {
        FIGfont::from_path(env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/small.flf")
    }

    #[test]
    fn test_padding() {
        assert_eq!(add_pad(&vec_string!("x", "x"), 0), vec_string!("x", "x"));
        assert_eq!(add_pad(&vec_string!("x", "x"), 4), vec_string!("    x", "    x"));
    }

    #[test]
    fn test_padding_utf8() {
        assert_eq!(add_pad(&vec_string!("á", "á"), 0), vec_string!("á", "á"));
        assert_eq!(add_pad(&vec_string!("á", "á"), 4), vec_string!("    á", "    á"));
    }

    #[test]
    fn test_newline_splits_lines() {
        use std::cell::RefCell;
        let font = test_font().unwrap();
        let sm = Smusher::new(&font);
        let mut wr = Wrapper::new(sm, Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        wr.wrap_str("hello\nworld", &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()));

        let flushed = flushed.borrow();
        assert_eq!(flushed.len(), 1, "newline should flush the first line");
        assert_eq!(
            &flushed[0],
            &[
                r" _        _ _     ",
                r"| |_  ___| | |___ ",
                r"| ' \/ -_) | / _ \",
                r"|_||_\___|_|_\___/",
                r"                  ",
            ],
            "flushed content should match rendered hello"
        );

        let remaining = wr.get();
        assert_eq!(
            &remaining,
            &[
                r"                _    _ ",
                r"__ __ _____ _ _| |__| |",
                r"\ V  V / _ \ '_| / _` |",
                r" \_/\_/\___/_| |_\__,_|",
                r"                       ",
            ],
            "remaining buffer should match rendered world"
        );
    }

    #[test]
    fn test_consecutive_newlines_produce_blank_lines() {
        use std::cell::RefCell;
        let font = test_font().unwrap();
        let sm = Smusher::new(&font);
        let mut wr = Wrapper::new(sm, Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        wr.wrap_str("hello\n\nworld", &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()));

        let flushed = flushed.borrow();
        assert_eq!(flushed.len(), 2, "consecutive newlines should flush content and blank line");
        assert_eq!(
            &flushed[0],
            &[
                r" _        _ _     ",
                r"| |_  ___| | |___ ",
                r"| ' \/ -_) | / _ \",
                r"|_||_\___|_|_\___/",
                r"                  ",
            ],
            "first flush should contain rendered hello"
        );
        assert_eq!(&flushed[1], &[r"", r"", r"", r"", r""], "second flush should be blank lines");

        let remaining = wr.get();
        assert_eq!(
            &remaining,
            &[
                r"                _    _ ",
                r"__ __ _____ _ _| |__| |",
                r"\ V  V / _ \ '_| / _` |",
                r" \_/\_/\___/_| |_\__,_|",
                r"                       ",
            ],
            "remaining buffer should match rendered world"
        );
    }

    #[test]
    fn test_leading_whitespace_after_newline_preserved() {
        use std::cell::RefCell;
        let font = test_font().unwrap();
        let sm = Smusher::new(&font);
        let mut wr = Wrapper::new(sm, Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        wr.wrap_str("hello\n  world", &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()));

        let flushed = flushed.borrow();
        assert_eq!(flushed.len(), 1, "should flush the first line");
        assert_eq!(
            &flushed[0],
            &[
                r" _        _ _     ",
                r"| |_  ___| | |___ ",
                r"| ' \/ -_) | / _ \",
                r"|_||_\___|_|_\___/",
                r"                  ",
            ],
            "flushed content should match rendered hello"
        );

        let remaining = wr.get();
        assert_eq!(
            &remaining,
            &[
                r"                  _    _ ",
                r"  __ __ _____ _ _| |__| |",
                r"  \ V  V / _ \ '_| / _` |",
                r"   \_/\_/\___/_| |_\__,_|",
                r"                         ",
            ],
            "remaining should match rendered '  world'"
        );
    }

    #[test]
    fn test_crlf_and_bare_cr_handled_as_newlines() {
        use std::cell::RefCell;
        let font = test_font().unwrap();

        // CRLF
        let sm = Smusher::new(&font);
        let mut wr = Wrapper::new(sm, Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());
        wr.wrap_str("hello\r\nworld", &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()));
        let flushed = flushed.borrow();
        assert_eq!(flushed.len(), 1, "CRLF should flush the first line");
        assert_eq!(
            &flushed[0],
            &[
                r" _        _ _     ",
                r"| |_  ___| | |___ ",
                r"| ' \/ -_) | / _ \",
                r"|_||_\___|_|_\___/",
                r"                  ",
            ],
            "CRLF flush should contain rendered hello"
        );

        // Bare CR
        let sm = Smusher::new(&font);
        let mut wr = Wrapper::new(sm, Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());
        wr.wrap_str("hello\rworld", &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()));
        let flushed = flushed.borrow();
        assert_eq!(flushed.len(), 1, "bare CR should flush the first line");
        assert_eq!(
            &flushed[0],
            &[
                r" _        _ _     ",
                r"| |_  ___| | |___ ",
                r"| ' \/ -_) | / _ \",
                r"|_||_\___|_|_\___/",
                r"                  ",
            ],
            "bare CR flush should contain rendered hello"
        );
    }

    #[test]
    fn test_leading_newline_flushes_blank_line() {
        use std::cell::RefCell;
        let font = test_font().unwrap();
        let sm = Smusher::new(&font);
        let mut wr = Wrapper::new(sm, Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        wr.wrap_str("\nworld", &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()));

        let flushed = flushed.borrow();
        assert_eq!(flushed.len(), 1, "leading newline should flush blank line");
        assert_eq!(&flushed[0], &[r"", r"", r"", r"", r""], "flushed should be blank lines");

        let remaining = wr.get();
        assert_eq!(
            &remaining,
            &[
                r"                _    _ ",
                r"__ __ _____ _ _| |__| |",
                r"\ V  V / _ \ '_| / _` |",
                r" \_/\_/\___/_| |_\__,_|",
                r"                       ",
            ],
            "remaining buffer should match rendered world"
        );
    }

    #[test]
    fn test_trailing_newline_no_blank_line() {
        use std::cell::RefCell;
        let font = test_font().unwrap();
        let sm = Smusher::new(&font);
        let mut wr = Wrapper::new(sm, Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        wr.wrap_str("hello\n", &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()));

        let flushed = flushed.borrow();
        assert_eq!(flushed.len(), 1, "trailing newline should flush the content line once");
        assert_eq!(
            &flushed[0],
            &[
                r" _        _ _     ",
                r"| |_  ___| | |___ ",
                r"| ' \/ -_) | / _ \",
                r"|_||_\___|_|_\___/",
                r"                  ",
            ],
            "flushed content should match rendered hello"
        );

        assert!(wr.is_empty(), "buffer should be empty after trailing newline");
    }

    #[test]
    fn test_space_overflow_no_duplicate_flush() {
        // Regression test for commit 59006750. When input exceeds width and wrap_word
        // is called, characters are wrapped one by one. The fix ensures that after a
        // space-triggered flush, the subsequent word is processed correctly without
        // redundant re-flushing of the pre-space buffer.
        // Uses test font where each character renders to exactly 1 char width.
        let font = FIGfont::from_path(env!("CARGO_MANIFEST_DIR").to_owned() + "/tests/fixtures/test.flf").unwrap();
        let sm = Smusher::new(&font);
        let mut wr = Wrapper::new(sm, Control::default(), 5, Align::Left);
        let flushed = RefCell::new(Vec::new());
        let flush_count = RefCell::new(0usize);

        // "hi test" (7 chars) exceeds width 5, triggering wrap_word.
        // Characters 'h','i',' ','t','e' fit (5 chars). 's' overflows (6 > 5),
        // flushing "hi te". Then 's','t' remain in buffer as "st".
        wr.wrap_str("hi test", &|lines: &[String]| {
            *flush_count.borrow_mut() += 1;
            flushed.borrow_mut().push(lines[0].clone());
        });

        let fc = *flush_count.borrow();
        let flushed = flushed.borrow();
        assert_eq!(fc, 1, "should flush once during wrap_word");
        assert_eq!(&flushed[0], &"hi te", "flushed content is 'hi te' (5 chars)");
        assert_eq!(wr.get()[0], "st", "remaining buffer holds 'st'");
    }

    #[test]
    fn test_wrap_codes_basic() {
        let font = test_font().unwrap();
        let sm = Smusher::new(&font);
        let mut wr = Wrapper::new(sm, Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        // "Hi" as codes: 72, 105
        wr.wrap_codes(&[72, 105], &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()));

        let flushed = flushed.borrow();
        assert!(flushed.is_empty(), "short input should not flush");
        assert!(!wr.is_empty(), "buffer should contain rendered Hi");
    }

    #[test]
    fn test_wrap_codes_with_newline() {
        let font = test_font().unwrap();
        let sm = Smusher::new(&font);
        let mut wr = Wrapper::new(sm, Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        // "Hi\n" as codes: 72, 105, 10
        wr.wrap_codes(&[72, 105, 10], &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()));

        let flushed = flushed.borrow();
        assert_eq!(flushed.len(), 1, "newline code should flush");
        assert!(wr.is_empty(), "buffer should be empty after flush");
    }

    #[test]
    fn test_push_codes() {
        let font = test_font().unwrap();
        let mut wr = Wrapper::new(Smusher::new(&font), Control::default(), 80, Align::Left);

        wr.push_codes(&[72, 105]).unwrap(); // "Hi"
        assert!(!wr.is_empty());
    }

    #[test]
    fn test_write_line_basic() {
        let font = test_font().unwrap();
        let mut wr = Wrapper::new(Smusher::new(&font), Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        let output = wr.write_line(
            "Hi",
            &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()),
        );

        // No intermediate flushes expected
        assert!(flushed.borrow().is_empty());
        assert!(!output.is_empty());
    }

    #[test]
    fn test_write_line_empty() {
        let font = test_font().unwrap();
        let mut wr = Wrapper::new(Smusher::new(&font), Control::default(), 80, Align::Left);

        let output = wr.write_line("", &|_lines| {});
        assert!(output.is_empty());
    }

    #[test]
    fn test_write_paragraph_joins_with_space() {
        let font = test_font().unwrap();
        let mut wr = Wrapper::new(Smusher::new(&font), Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        let print_fn = |lines: &[String]| flushed.borrow_mut().push(lines.to_vec());
        wr.write_paragraph("Hi", &print_fn);
        wr.write_paragraph("World", &print_fn);

        // Nothing flushed yet
        assert!(flushed.borrow().is_empty());
        // Buffer has accumulated content
        assert!(!wr.is_empty());
    }

    #[test]
    fn test_write_paragraph_blank_line_flushes() {
        let font = test_font().unwrap();
        let mut wr = Wrapper::new(Smusher::new(&font), Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        let print_fn = |lines: &[String]| flushed.borrow_mut().push(lines.to_vec());
        wr.write_paragraph("Hi", &print_fn);
        wr.write_paragraph("", &print_fn); // blank line

        // Should have flushed once
        assert_eq!(flushed.borrow().len(), 1);
    }

    #[test]
    fn test_write_paragraph_leading_space_flushes() {
        let font = test_font().unwrap();
        let mut wr = Wrapper::new(Smusher::new(&font), Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        let print_fn = |lines: &[String]| flushed.borrow_mut().push(lines.to_vec());
        wr.write_paragraph("Hi", &print_fn);
        wr.write_paragraph(" World", &print_fn);

        // Should have flushed once
        assert_eq!(flushed.borrow().len(), 1);
    }

    #[test]
    fn test_flush_paragraph_empty() {
        let font = test_font().unwrap();
        let mut wr = Wrapper::new(Smusher::new(&font), Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        wr.flush_paragraph(
            &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec()),
        );
        assert!(flushed.borrow().is_empty());
    }

    #[test]
    fn test_flush_paragraph_with_content() {
        let font = test_font().unwrap();
        let mut wr = Wrapper::new(Smusher::new(&font), Control::default(), 80, Align::Left);
        let flushed = RefCell::new(Vec::new());

        let print_fn = |lines: &[String]| flushed.borrow_mut().push(lines.to_vec());
        wr.write_paragraph("Hi", &print_fn);
        wr.flush_paragraph(&print_fn);

        assert_eq!(flushed.borrow().len(), 1);
    }

    #[test]
    fn test_is_whitespace_code() {
        assert!(is_whitespace_code(32)); // space
        assert!(is_whitespace_code(9)); // tab
        assert!(is_whitespace_code(10)); // LF
        assert!(is_whitespace_code(11)); // VT
        assert!(is_whitespace_code(12)); // FF
        assert!(is_whitespace_code(13)); // CR
        assert!(!is_whitespace_code(65)); // 'A'
        assert!(!is_whitespace_code(0)); // NUL
    }

    #[test]
    fn test_is_blank_codes() {
        assert!(is_blank_codes(&[]));
        assert!(is_blank_codes(&[32, 9, 10, 13]));
        assert!(!is_blank_codes(&[65]));
        assert!(!is_blank_codes(&[32, 65]));
    }
}
