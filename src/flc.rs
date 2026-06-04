use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::Error;
use crate::zip::{is_zip, decompress_zip};

/// A single transformation command within a stage.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FlcCommand {
    Single { input: i32, output: i32 },
    /// Range transformation: maps input_start..=input_end to output_start..=output_end.
    /// output_end is parsed from the FLC file for range-size validation but not needed
    /// for the actual transformation (both ranges must be the same size).
    Range { input_start: i32, input_end: i32, output_start: i32, output_end: i32 },
}

/// One transformation stage, consisting of a sequence of commands.
/// Within a stage, only the first matching command is applied.
#[derive(Debug, Clone)]
pub struct TransformationStage {
    commands: Vec<FlcCommand>,
}

/// Input encoding mode for multi-byte character processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEncoding {
    Default,
    HZ,
    ShiftJIS,
    Dbcs,
    UTF8,
}

/// Size of an ISO 2022 character set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Iso2022CharSetSize {
    Bits94,
    Bits96,
    Bits94x94,
}

/// An ISO 2022 character set assignment.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Iso2022CharSet {
    size: Iso2022CharSetSize,
    designating_byte: i32,
}

/// Accumulated ISO 2022 settings from "g" commands.
#[derive(Debug, Clone)]
pub struct Iso2022Settings {
    g_sets: [Option<Iso2022CharSet>; 4],
    left_half: i32,
    right_half: i32,
}

impl Default for Iso2022Settings {
    fn default() -> Self {
        Self {
            g_sets: [
                Some(Iso2022CharSet { size: Iso2022CharSetSize::Bits94, designating_byte: 66 }),
                Some(Iso2022CharSet { size: Iso2022CharSetSize::Bits96, designating_byte: 65 }),
                None,
                None,
            ],
            left_half: 0,
            right_half: 1,
        }
    }
}

/// A parsed FIGfont control file (.flc).
#[derive(Debug, Clone)]
pub struct Flc {
    stages: Vec<TransformationStage>,
    encoding: InputEncoding,
    iso2022: Iso2022Settings,
}

/// Pipeline for chaining multiple control files together.
#[derive(Debug, Clone)]
pub struct Control {
    files: Vec<Flc>,
    encoding: InputEncoding,
}

impl Flc {
    /// Parse a control file from the given path.
    /// Automatically detects and decompresses ZIP-compressed files.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();

        if is_zip(path) {
            let reader = decompress_zip(path).map_err(|_| Error::ControlFormat("failed to decompress ZIP"))?;
            Self::load_from_reader(reader.lines())
        } else {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            Self::load_from_reader(reader.lines())
        }
    }

    /// Parse control file data from a line iterator.
    fn load_from_reader<L: IntoIterator<Item = std::io::Result<String>>>(lines: L) -> Result<Self, Error> {
        let mut stages: Vec<TransformationStage> = vec![TransformationStage { commands: Vec::new() }];
        let mut encoding = InputEncoding::Default;
        let mut iso2022 = Iso2022Settings::default();
        let mut first_line = true;

        for line_result in lines {
            let line = line_result?;
            let line = line.trim_end();

            if first_line {
                first_line = false;
                if line == "flc2a" {
                    continue;
                }
            }

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let trimmed = line.trim_start();
            let cmd = trimmed.as_bytes().first().copied();

            match cmd {
                Some(b't') => {
                    if let Some(cmd) = parse_t_command(trimmed)? {
                        stages.last_mut().unwrap().commands.push(cmd);
                    }
                }
                Some(b'f') => {
                    stages.push(TransformationStage { commands: Vec::new() });
                }
                Some(b'h') => {
                    encoding = InputEncoding::HZ;
                }
                Some(b'j') => {
                    encoding = InputEncoding::ShiftJIS;
                }
                Some(b'b') => {
                    encoding = InputEncoding::Dbcs;
                }
                Some(b'u') => {
                    encoding = InputEncoding::UTF8;
                }
                Some(b'g') => {
                    parse_g_command(trimmed, &mut iso2022)?;
                }
                Some(b'0'..=b'9') | Some(b'-') | Some(b'\\') => {
                    if let Some(cmd) = parse_number_command(trimmed)? {
                        stages.last_mut().unwrap().commands.push(cmd);
                    }
                }
                _ => {}
            }
        }

        Ok(Flc { stages, encoding, iso2022 })
    }

    /// Apply all transformation stages to a character code.
    pub fn apply(&self, code: i32) -> i32 {
        let mut result = code;
        for stage in &self.stages {
            if let Some(mapped) = stage.apply(result) {
                result = mapped;
            }
        }
        result
    }

    /// Get the configured input encoding.
    pub fn encoding(&self) -> InputEncoding {
        self.encoding
    }

    /// Get the accumulated ISO 2022 settings.
    pub fn iso2022_settings(&self) -> &Iso2022Settings {
        &self.iso2022
    }
}

impl TransformationStage {
    fn apply(&self, code: i32) -> Option<i32> {
        for cmd in &self.commands {
            match cmd {
                FlcCommand::Single { input, output } => {
                    if code == *input {
                        return Some(*output);
                    }
                }
                FlcCommand::Range { input_start, input_end, output_start, output_end: _ } => {
                    if code >= *input_start && code <= *input_end {
                        let offset = code - input_start;
                        return Some(output_start + offset);
                    }
                }
            }
        }
        None
    }
}

impl Control {
    /// Create a pipeline from multiple control file paths.
    pub fn from_paths<P: AsRef<Path>, I: IntoIterator<Item = P>>(paths: I) -> Result<Self, Error> {
        let mut files = Vec::new();
        let mut encoding = InputEncoding::Default;

        for path in paths {
            let flc = Flc::from_path(path)?;
            encoding = flc.encoding();
            files.push(flc);
        }

        Ok(Control { files, encoding })
    }

    /// Apply all transformations across all files in order.
    pub fn apply(&self, code: i32) -> i32 {
        let mut result = code;
        for flc in &self.files {
            result = flc.apply(result);
        }
        result
    }

    /// Get the effective encoding (last encoding command wins).
    pub fn encoding(&self) -> InputEncoding {
        self.encoding
    }
}

/// Parse a "t" command line.
fn parse_t_command(line: &str) -> Result<Option<FlcCommand>, Error> {
    let tokens = tokenize_flc_line(line);
    if tokens.is_empty() || tokens[0] != "t" {
        return Ok(None);
    }
    if tokens.len() < 3 {
        return Err(Error::ControlFormat("invalid t command"));
    }

    let in_part: &str = &tokens[1];
    let out_part: &str = &tokens[2];

    // Check for range syntax
    if let Some(pos) = find_range_separator(in_part) {
        let (in_start, _) = in_part.split_at(pos);
        let in_end = &in_part[pos+1..];

        let out_pos = find_range_separator(out_part)
            .ok_or(Error::ControlFormat("range input requires range output"))?;
        let (out_start, _) = out_part.split_at(out_pos);
        let out_end = &out_part[out_pos+1..];

        let is = parse_char_code(in_start)?;
        let ie = parse_char_code(in_end)?;
        let os = parse_char_code(out_start)?;
        let oe = parse_char_code(out_end)?;

        let in_size = ie - is;
        let out_size = oe - os;
        if in_size != out_size {
            return Err(Error::ControlRangeMismatch);
        }

        Ok(Some(FlcCommand::Range {
            input_start: is,
            input_end: ie,
            output_start: os,
            output_end: oe,
        }))
    } else {
        let input = parse_char_code(in_part)?;
        let output = parse_char_code(out_part)?;
        Ok(Some(FlcCommand::Single { input, output }))
    }
}

/// Tokenize an FLC line, keeping consecutive \\0x escapes together.
fn tokenize_flc_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else if ch == '\\' {
            current.push(ch);
            if chars.peek() == Some(&'0') {
                current.push(chars.next().unwrap());
                if chars.peek() == Some(&'x') || chars.peek() == Some(&'X') {
                    current.push(chars.next().unwrap());
                    // Consume hex digits, but stop if we hit another \\0x
                    while let Some(&hex_ch) = chars.peek() {
                        if hex_ch.is_ascii_hexdigit() {
                            current.push(chars.next().unwrap());
                        } else if hex_ch == '\\' {
                            // Check if next is \\0x - if so, stop here to concatenate later
                            break;
                        } else {
                            break;
                        }
                    }
                }
            } else {
                // Other escape sequences - consume until next whitespace or end
                while let Some(&next) = chars.peek() {
                    if next.is_whitespace() {
                        break;
                    }
                    current.push(chars.next().unwrap());
                }
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Parse a "number number" command (no "t" prefix).
fn parse_number_command(line: &str) -> Result<Option<FlcCommand>, Error> {
    let tokens = tokenize_flc_line(line);
    if tokens.len() < 2 {
        return Ok(None);
    }

    let in_part = &tokens[0];
    let out_part = &tokens[1];

    // Check for range syntax (code1-code2). The - is a range separator if it's
    // not the first character (which would be a negative sign).
    if let Some(pos) = find_range_separator(in_part) {
        let (in_start, _) = in_part.split_at(pos);
        let in_end = &in_part[pos+1..];

        let out_pos = find_range_separator(out_part)
            .ok_or(Error::ControlFormat("range input requires range output"))?;
        let (out_start, _) = out_part.split_at(out_pos);
        let out_end = &out_part[out_pos+1..];

        let is = parse_char_code(in_start)?;
        let ie = parse_char_code(in_end)?;
        let os = parse_char_code(out_start)?;
        let oe = parse_char_code(out_end)?;

        let in_size = ie - is;
        let out_size = oe - os;
        if in_size != out_size {
            return Err(Error::ControlRangeMismatch);
        }

        Ok(Some(FlcCommand::Range {
            input_start: is,
            input_end: ie,
            output_start: os,
            output_end: oe,
        }))
    } else {
        let input = parse_char_code(in_part)?;
        let output = parse_char_code(out_part)?;
        Ok(Some(FlcCommand::Single { input, output }))
    }
}

/// Find the position of the range separator '-' in a character code expression.
/// Returns None if there's no range separator.
fn find_range_separator(s: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' {
            // Skip escape sequence
            i += 1;
            if i < chars.len() {
                match chars[i] {
                    '0' if i + 1 < chars.len() && (chars[i+1] == 'x' || chars[i+1] == 'X') => {
                        i += 2;
                        // Skip hex digits (at least 2)
                        while i < chars.len() && chars[i].is_ascii_hexdigit() {
                            i += 1;
                        }
                    }
                    '-' => {
                        // Negative escape, skip digits
                        i += 1;
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                    'a'..='z' | 'A'..='Z' => {
                        // Single char escape (like \n, \t, etc.), skip 1 char
                        i += 1;
                    }
                    '0'..='7' => {
                        // Octal/decimal numeric escape, skip digits
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                    _ => {
                        // Unknown escape, skip until next special char
                        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '-' {
                            i += 1;
                        }
                    }
                }
            }
        } else if chars[i] == '-' {
            // This is a range separator (not a negative sign since it's not the first char)
            if i > 0 {
                return Some(i);
            }
            i += 1;
        } else {
            i += 1;
        }
    }
    None
}

/// Parse a character code from "t" command format (with escape sequences).
fn parse_char_code(s: &str) -> Result<i32, Error> {
    if let Some(rest) = s.strip_prefix('\\') {
        parse_escape_sequence(rest)
    } else {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() == 1 {
            Ok(chars[0] as i32)
        } else {
            parse_numeric_code(s)
        }
    }
}

/// Parse an escape sequence after the backslash.
/// Per spec: \65 is decimal 65, \0x100 is hex 256.
/// Handles concatenated \\0xNN escapes (e.g., \0x01\0x01 = 0x0101).
fn parse_escape_sequence(s: &str) -> Result<i32, Error> {
    if s.is_empty() {
        return Ok(32);
    }

    let first = s.as_bytes()[0];

    match first as char {
    ' ' => Ok(32),
        'a' => Ok(7),
        'b' => Ok(8),
        'e' => Ok(27),
        'f' => Ok(12),
        'n' => Ok(10),
        'r' => Ok(13),
        't' => Ok(9),
        'v' => Ok(11),
        '\\' => Ok(92),
        '-' => {
            if s.len() > 1 {
                let digits = &s[1..];
                let val = if digits.starts_with("0x") || digits.starts_with("0X") {
                    i32::from_str_radix(&digits[2..], 16).map_err(|_| Error::ControlFormat("invalid hex number"))?
                } else if digits.starts_with('0') && digits.len() > 1 {
                    i32::from_str_radix(digits, 8).map_err(|_| Error::ControlFormat("invalid octal number"))?
                } else {
                    digits.parse().map_err(|_| Error::ControlFormat("invalid number"))?
                };
                Ok(-val)
            } else {
                Ok(-1)
            }
        }
        '0' => {
            if s.len() > 1 && (s.as_bytes()[1] == b'x' || s.as_bytes()[1] == b'X') {
                // Handle concatenated \\0xNN escapes
                parse_concatenated_hex(s)
            } else if s.len() > 1 {
                let val = i32::from_str_radix(s, 8).map_err(|_| Error::ControlFormat("invalid octal escape"))?;
                Ok(val)
            } else {
                Ok(0)
            }
        }
        _ => {
            if s.starts_with("0x") || s.starts_with("0X") {
                let hex_str = &s[2..];
                let val = i32::from_str_radix(hex_str, 16).map_err(|_| Error::ControlFormat("invalid hex escape"))?;
                Ok(val)
            } else {
                // Plain numbers in escape sequences are decimal (per spec: \65 = 65)
                let val: i32 = s.parse().map_err(|_| Error::ControlFormat("invalid number"))?;
                Ok(val)
            }
        }
    }
}

/// Parse concatenated hex escapes (e.g., "0x01\\0x01" -> 0x0101).
/// For single escape (e.g., "0x03b1"), parses as one value (0x03B1 = 945).
fn parse_concatenated_hex(s: &str) -> Result<i32, Error> {
    // Count how many \\0x prefixes are in the string
    let num_prefixes = s.matches("0x").count() + s.matches("0X").count();

    if num_prefixes == 1 {
        // Single escape, parse all hex digits as one value
        let hex_str = &s[2..];
        let val = i32::from_str_radix(hex_str, 16).map_err(|_| Error::ControlFormat("invalid hex escape"))?;
        Ok(val)
    } else {
        // Multiple escapes, concatenate 2 hex digits per \\0x prefix
        let mut result: i32 = 0;
        let mut remaining = s;

        while !remaining.is_empty() {
            if let Some(after_prefix) = remaining.strip_prefix("0x").or_else(|| remaining.strip_prefix("0X")) {
                if after_prefix.len() >= 2 {
                    let hex_bytes: Vec<u8> = after_prefix.bytes().take(2).collect();
                    let hex_str = String::from_utf8(hex_bytes).map_err(|_| Error::ControlFormat("invalid hex escape"))?;
                    let val = i32::from_str_radix(&hex_str, 16).map_err(|_| Error::ControlFormat("invalid hex escape"))?;
                    result = (result << 8) | val;
                    remaining = &after_prefix[2..];
                } else {
                    let val = i32::from_str_radix(after_prefix, 16).map_err(|_| Error::ControlFormat("invalid hex escape"))?;
                    result = (result << 8) | val;
                    break;
                }
            } else if let Some(rest) = remaining.strip_prefix('\\') {
                remaining = rest;
            } else {
                break;
            }
        }

        Ok(result)
    }
}

/// Parse a numeric character code (for "number number" format or escape sequences).
fn parse_numeric_code(s: &str) -> Result<i32, Error> {
    let s = s.trim();

    if let Some(hex_str) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        let val = i32::from_str_radix(hex_str, 16).map_err(|_| Error::ControlFormat("invalid hex number"))?;
        Ok(val)
    } else if let Some(digits) = s.strip_prefix('-') {
        let val = if let Some(h) = digits.strip_prefix("0x").or_else(|| digits.strip_prefix("0X")) {
            i32::from_str_radix(h, 16).map_err(|_| Error::ControlFormat("invalid hex number"))?
        } else if digits.starts_with('0') && digits.len() > 1 {
            i32::from_str_radix(digits, 8).map_err(|_| Error::ControlFormat("invalid octal number"))?
        } else {
            digits.parse().map_err(|_| Error::ControlFormat("invalid number"))?
        };
        Ok(-val)
    } else if s.starts_with('0') && s.len() > 1 {
        let val = i32::from_str_radix(s, 8).map_err(|_| Error::ControlFormat("invalid octal number"))?;
        Ok(val)
    } else {
        let val: i32 = s.parse().map_err(|_| Error::ControlFormat("invalid number"))?;
        Ok(val)
    }
}

/// Parse a "g" command for ISO 2022 settings.
fn parse_g_command(line: &str, iso2022: &mut Iso2022Settings) -> Result<(), Error> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(Error::ControlFormat("invalid g command"));
    }

    let g_keyword = parts[0];
    let rest = g_keyword.strip_prefix('g').unwrap_or("");

    if rest == "L" || rest == "R" {
        let g_reg: i32 = parts[1].parse().map_err(|_| Error::ControlFormat("invalid g register"))?;
        if !(0..=3).contains(&g_reg) {
            return Err(Error::ControlFormat("invalid g register"));
        }
        if rest == "L" {
            iso2022.left_half = g_reg;
        } else {
            iso2022.right_half = g_reg;
        }
    } else if rest == "0" || rest == "1" || rest == "2" || rest == "3" {
        let g_reg: usize = rest.parse().map_err(|_| Error::ControlFormat("invalid g register"))?;
        let size = parse_g_size(parts[1])?;
        let d_byte = if parts.len() >= 3 {
            parts[2].chars().next().unwrap_or('0') as i32
        } else {
            0
        };
        iso2022.g_sets[g_reg] = Some(Iso2022CharSet { size, designating_byte: d_byte });
    } else if g_keyword == "g" && parts.len() >= 3 {
        let first_arg = parts[1];
        if first_arg == "L" || first_arg == "R" {
            let g_reg: i32 = parts[2].parse().map_err(|_| Error::ControlFormat("invalid g register"))?;
            if !(0..=3).contains(&g_reg) {
                return Err(Error::ControlFormat("invalid g register"));
            }
            if first_arg == "L" {
                iso2022.left_half = g_reg;
            } else {
                iso2022.right_half = g_reg;
            }
        } else if first_arg == "0" || first_arg == "1" || first_arg == "2" || first_arg == "3" {
            let g_reg: usize = first_arg.parse().map_err(|_| Error::ControlFormat("invalid g register"))?;
            let size = parse_g_size(parts[2])?;
            let d_byte = if parts.len() >= 4 {
                parts[3].chars().next().unwrap_or('0') as i32
            } else {
                0
            };
            iso2022.g_sets[g_reg] = Some(Iso2022CharSet { size, designating_byte: d_byte });
        } else {
            return Err(Error::ControlFormat("invalid g command target"));
        }
    } else {
        return Err(Error::ControlFormat("invalid g command target"));
    }

    Ok(())
}

/// Parse the size part of a g command (94, 96, 94x94, or a charset designator letter).
fn parse_g_size(s: &str) -> Result<Iso2022CharSetSize, Error> {
    match s {
        "94" => Ok(Iso2022CharSetSize::Bits94),
        "96" => Ok(Iso2022CharSetSize::Bits96),
        "94x94" => Ok(Iso2022CharSetSize::Bits94x94),
        _ => {
            if s.len() == 1 && s.as_bytes()[0] >= b'A' && s.as_bytes()[0] <= b'~' {
                Ok(Iso2022CharSetSize::Bits94)
            } else {
                Err(Error::ControlFormat("invalid g charset size"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_flc(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    // Parse single character mapping with literal chars
    #[test]
    fn test_parse_single_mapping_literal() {
        let file = create_flc("t A B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(65), 66);
        assert_eq!(flc.apply(66), 66);
        assert_eq!(flc.apply(67), 67);
    }

    // Parse range mapping
    #[test]
    fn test_parse_range_mapping() {
        let file = create_flc("t a-z A-Z\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply('a' as i32), 'A' as i32);
        assert_eq!(flc.apply('z' as i32), 'Z' as i32);
        assert_eq!(flc.apply('m' as i32), 'M' as i32);
        assert_eq!(flc.apply('A' as i32), 'A' as i32);
    }

    // Parse number-number form (mapping table format)
    #[test]
    fn test_parse_number_command() {
        let file = create_flc("65 66\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(65), 66);
        assert_eq!(flc.apply(66), 66);
    }

    // Parse hex number-number form
    #[test]
    fn test_parse_hex_number_command() {
        let file = create_flc("0x41 0x42\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(0x41), 0x42);
    }

    // Parse escape sequences in "t" commands
    #[test]
    fn test_parse_escape_decimal() {
        let file = create_flc("t \\65 B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(65), 'B' as i32);
    }

    // Parse hex escape sequences
    #[test]
    fn test_parse_escape_hex() {
        let file = create_flc("t A \\0x42\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply('A' as i32), 0x42);
    }

    // Parse escape sequences: bell, backspace, newline, tab
    #[test]
    fn test_parse_escape_special() {
        let file = create_flc("t \\a B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(7), 'B' as i32);

        let file = create_flc("t \\b B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(8), 'B' as i32);

        let file = create_flc("t \\n B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(10), 'B' as i32);

        let file = create_flc("t \\t B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(9), 'B' as i32);
    }

    // Parse escape for backslash itself
    #[test]
    fn test_parse_escape_backslash() {
        let file = create_flc("t \\\\ B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(92), 'B' as i32);
    }

    // Parse escape for space
    #[test]
    fn test_parse_escape_space() {
        let file = create_flc("t \\ B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(32), 'B' as i32);
    }

    // Parse "f" command creates new transformation stage
    #[test]
    fn test_parse_freeze_command() {
        let file = create_flc("t a-z A-Z\nf\nt Q ~\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply('q' as i32), '~' as i32);
        assert_eq!(flc.apply('a' as i32), 'A' as i32);
        assert_eq!(flc.apply('Q' as i32), '~' as i32);
    }

    // Parse "h" command sets HZ encoding
    #[test]
    fn test_parse_hz_encoding() {
        let file = create_flc("h\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.encoding(), InputEncoding::HZ);
    }

    // Parse "j" command sets Shift-JIS encoding
    #[test]
    fn test_parse_shiftjis_encoding() {
        let file = create_flc("j\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.encoding(), InputEncoding::ShiftJIS);
    }

    // Parse "b" command sets Dbcs encoding
    #[test]
    fn test_parse_dbcs_encoding() {
        let file = create_flc("b\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.encoding(), InputEncoding::Dbcs);
    }

    // Parse "u" command sets UTF-8 encoding
    #[test]
    fn test_parse_utf8_encoding() {
        let file = create_flc("u\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.encoding(), InputEncoding::UTF8);
    }

    // Parse "g" commands for ISO 2022
    #[test]
    fn test_parse_iso2022_g_command() {
        let file = create_flc("g 0 94 B\ng 1 96 A\ng L 0\ng R 1\n");
        let flc = Flc::from_path(file.path()).unwrap();
        let settings = flc.iso2022_settings();
        assert_eq!(settings.left_half, 0);
        assert_eq!(settings.right_half, 1);
    }

    // Skip comment lines and blank lines
    #[test]
    fn test_skip_comments_and_blanks() {
        let file = create_flc("# This is a comment\n\nt A B\n\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply('A' as i32), 'B' as i32);
    }

    // Parse optional "flc2a" signature
    #[test]
    fn test_parse_signature() {
        let file = create_flc("flc2a\nt A B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply('A' as i32), 'B' as i32);
    }

    // Without signature still works
    #[test]
    fn test_no_signature() {
        let file = create_flc("t A B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply('A' as i32), 'B' as i32);
    }

    // First-match rule within a stage
    #[test]
    fn test_first_match_rule() {
        let file = create_flc("t A B\nt A C\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply('A' as i32), 'B' as i32);
    }

    // Swap pattern works correctly with two commands in same stage
    #[test]
    fn test_swap_pattern() {
        let file = create_flc("t A B\nt B A\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply('A' as i32), 'B' as i32);
        assert_eq!(flc.apply('B' as i32), 'A' as i32);
    }

    // Empty control file produces single empty stage
    #[test]
    fn test_empty_control_file() {
        let file = create_flc("");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(65), 65);
    }

    // Only "flc2a" signature
    #[test]
    fn test_signature_only() {
        let file = create_flc("flc2a\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(65), 65);
    }

    // Range size mismatch produces error
    #[test]
    fn test_range_mismatch_error() {
        let file = create_flc("t A-C a-z\n");
        let result = Flc::from_path(file.path());
        assert!(matches!(result, Err(Error::ControlRangeMismatch)));
    }

    // Encoding commands are mutually exclusive (last wins)
    #[test]
    fn test_encoding_last_wins() {
        let file = create_flc("h\nu\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.encoding(), InputEncoding::UTF8);
    }

    // Control chains multiple files
    #[test]
    fn test_pipeline_chains() {
        let file1 = create_flc("t a-z A-Z\n");
        let file2 = create_flc("t A B\n");
        let paths = vec![file1.path().to_path_buf(), file2.path().to_path_buf()];
        let pipeline = Control::from_paths(&paths).unwrap();
        assert_eq!(pipeline.apply('a' as i32), 'B' as i32);
        assert_eq!(pipeline.apply('b' as i32), 'B' as i32);
    }

    // Encoding is last file's encoding
    #[test]
    fn test_pipeline_encoding() {
        let file1 = create_flc("h\n");
        let file2 = create_flc("u\n");
        let paths = vec![file1.path().to_path_buf(), file2.path().to_path_buf()];
        let pipeline = Control::from_paths(&paths).unwrap();
        assert_eq!(pipeline.encoding(), InputEncoding::UTF8);
    }

    // Test negative number-number form
    #[test]
    fn test_negative_number_command() {
        let file = create_flc("-252 -255\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(-252), -255);
    }

    // Parse octal escape sequence (leading 0 indicates octal)
    #[test]
    fn test_parse_escape_octal() {
        let file = create_flc("t \\0101 B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(65), 'B' as i32);
    }

    // Escape sequence \e for ESC character
    #[test]
    fn test_parse_escape_esc() {
        let file = create_flc("t \\e B\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(27), 'B' as i32);
    }

    // Test Unicode character codes in number-number form
    #[test]
    fn test_unicode_codes() {
        let file = create_flc("0x3B1 0x391\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply(0x3B1), 0x391);
    }

    // Test "f" command with no surrounding "t" commands creates empty stage
    #[test]
    fn test_freeze_no_commands() {
        let file = create_flc("t A B\nf\nt C D\n");
        let flc = Flc::from_path(file.path()).unwrap();
        assert_eq!(flc.apply('A' as i32), 'B' as i32);
        assert_eq!(flc.apply('C' as i32), 'D' as i32);
        assert_eq!(flc.apply('B' as i32), 'B' as i32);
    }

    // Test from real upper.flc file
    #[test]
    fn test_upper_flc() {
        let path = format!("{}/fonts/upper.flc", env!("CARGO_MANIFEST_DIR"));
        let flc = Flc::from_path(&path).unwrap();
        assert_eq!(flc.apply('a' as i32), 'A' as i32);
        assert_eq!(flc.apply('z' as i32), 'Z' as i32);
    }

    // Test from real frango.flc file
    #[test]
    fn test_frango_flc() {
        let path = format!("{}/fonts/frango.flc", env!("CARGO_MANIFEST_DIR"));
        let flc = Flc::from_path(&path).unwrap();
        assert_eq!(flc.apply('a' as i32), 0x03B1);
        assert_eq!(flc.apply('A' as i32), 0x0391);
    }

    // Test from real utf8.flc file
    #[test]
    fn test_utf8_flc() {
        let path = format!("{}/fonts/utf8.flc", env!("CARGO_MANIFEST_DIR"));
        let flc = Flc::from_path(&path).unwrap();
        assert_eq!(flc.encoding(), InputEncoding::UTF8);
    }

    // Test from real hz.flc file
    #[test]
    fn test_hz_flc() {
        let path = format!("{}/fonts/hz.flc", env!("CARGO_MANIFEST_DIR"));
        let flc = Flc::from_path(&path).unwrap();
        assert_eq!(flc.encoding(), InputEncoding::HZ);
    }

    // Test from real 646-de.flc file (number-number format)
    #[test]
    fn test_646_de_flc() {
        let path = format!("{}/fonts/646-de.flc", env!("CARGO_MANIFEST_DIR"));
        let flc = Flc::from_path(&path).unwrap();
        assert_eq!(flc.apply(0x40), 0xA7);
        assert_eq!(flc.apply(0x7B), 0xE4);
    }

    // Test from real jis0201.flc file (with g commands)
    #[test]
    fn test_jis0201_flc() {
        let path = format!("{}/fonts/jis0201.flc", env!("CARGO_MANIFEST_DIR"));
        let flc = Flc::from_path(&path).unwrap();
        assert_eq!(flc.apply(0x4A005C), 0xA5);
    }

    // Loading a ZIP-compressed FLC produces the same result as the uncompressed file
    #[test]
    fn test_flc_load_zip() {
        let path = format!("{}/tests/fixtures/upper.flc.zip", env!("CARGO_MANIFEST_DIR"));
        let flc = Flc::from_path(&path).unwrap();
        assert_eq!(flc.apply('a' as i32), 'A' as i32);
        assert_eq!(flc.apply('z' as i32), 'Z' as i32);
    }
}
