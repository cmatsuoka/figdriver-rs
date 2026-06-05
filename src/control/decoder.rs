use super::InputEncoding;

/// Iterator that decodes raw bytes into character codes according to the given encoding.
/// Yields one `i32` code per `next()` call.
pub struct EncodingDecoder<'a> {
    bytes: &'a [u8],
    encoding: InputEncoding,
    pos: usize,
    /// For HZ encoding: are we currently in two-byte mode?
    hz_two_byte: bool,
}

impl<'a> EncodingDecoder<'a> {
    pub fn new(bytes: &'a [u8], encoding: InputEncoding) -> Self {
        Self {
            bytes,
            encoding,
            pos: 0,
            hz_two_byte: false,
        }
    }
}

impl Iterator for EncodingDecoder<'_> {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        match self.encoding {
            InputEncoding::Default => self.next_latin1(),
            InputEncoding::UTF8 => self.next_utf8(),
            InputEncoding::Dbcs => self.next_dbcs(),
            InputEncoding::ShiftJIS => self.next_shiftjis(),
            InputEncoding::HZ => self.next_hz(),
        }
    }
}

impl EncodingDecoder<'_> {
    fn next_latin1(&mut self) -> Option<i32> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let b = self.bytes[self.pos];
        self.pos += 1;
        Some(b as i32)
    }

    fn next_utf8(&mut self) -> Option<i32> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let remaining = &self.bytes[self.pos..];
        let first = remaining[0];

        let (code, consumed) = if first < 0x80 {
            (first as i32, 1)
        } else if first < 0xC0 {
            (128, 1)
        } else if first < 0xE0 {
            if remaining.len() >= 2 && (remaining[1] & 0xC0) == 0x80 {
                let hi = (remaining[0] & 0x1F) as u32;
                let lo = (remaining[1] & 0x3F) as u32;
                let cp = (hi << 6) | lo;
                if cp >= 0x80 {
                    (cp as i32, 2)
                } else {
                    (128, 1)
                }
            } else {
                (128, 1)
            }
        } else if first < 0xF0 {
            if remaining.len() >= 3
                && (remaining[1] & 0xC0) == 0x80
                && (remaining[2] & 0xC0) == 0x80
            {
                let hi = (remaining[0] & 0x0F) as u32;
                let mid = (remaining[1] & 0x3F) as u32;
                let lo = (remaining[2] & 0x3F) as u32;
                let cp = (hi << 12) | (mid << 6) | lo;
                if cp >= 0x800 && (cp < 0xD800 || cp > 0xDFFF) {
                    (cp as i32, 3)
                } else {
                    (128, 1)
                }
            } else {
                (128, 1)
            }
        } else if first < 0xF8 {
            if remaining.len() >= 4
                && (remaining[1] & 0xC0) == 0x80
                && (remaining[2] & 0xC0) == 0x80
                && (remaining[3] & 0xC0) == 0x80
            {
                let hi = (remaining[0] & 0x07) as u32;
                let mid1 = (remaining[1] & 0x3F) as u32;
                let mid2 = (remaining[2] & 0x3F) as u32;
                let lo = (remaining[3] & 0x3F) as u32;
                let cp = (hi << 18) | (mid1 << 12) | (mid2 << 6) | lo;
                if cp >= 0x10000 && cp <= 0x10FFFF {
                    (cp as i32, 4)
                } else {
                    (128, 1)
                }
            } else {
                (128, 1)
            }
        } else {
            (128, 1)
        };

        self.pos += consumed;
        Some(code)
    }

    fn next_dbcs(&mut self) -> Option<i32> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let b = self.bytes[self.pos];

        if b < 0x80 {
            self.pos += 1;
            Some(b as i32)
        } else if self.pos + 1 < self.bytes.len() {
            let hi = b;
            let lo = self.bytes[self.pos + 1];
            self.pos += 2;
            Some(((hi as i32) << 8) | (lo as i32))
        } else {
            self.pos += 1;
            Some(b as i32)
        }
    }

    fn next_shiftjis(&mut self) -> Option<i32> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let b = self.bytes[self.pos];

        let is_high_byte = (b >= 0x80 && b <= 0x9F) || (b >= 0xE0 && b <= 0xEF);

        if is_high_byte && self.pos + 1 < self.bytes.len() {
            let hi = b;
            let lo = self.bytes[self.pos + 1];
            self.pos += 2;
            Some(((hi as i32) << 8) | (lo as i32))
        } else {
            self.pos += 1;
            Some(b as i32)
        }
    }

    fn next_hz(&mut self) -> Option<i32> {
        loop {
            if self.pos >= self.bytes.len() {
                return None;
            }

            if self.bytes[self.pos] == b'~' {
                self.pos += 1;
                if self.pos >= self.bytes.len() {
                    // Lone tilde at end of input: consume silently
                    continue;
                }

                match self.bytes[self.pos] {
                    b'{' => {
                        self.pos += 1;
                        self.hz_two_byte = true;
                        continue;
                    }
                    b'}' => {
                        self.pos += 1;
                        self.hz_two_byte = false;
                        continue;
                    }
                    b'~' => {
                        self.pos += 1;
                        return Some('~' as i32);
                    }
                    _ => {
                        // All other ~X sequences are removed from input
                        self.pos += 1;
                        self.hz_two_byte = false;
                        continue;
                    }
                }
            }

            if self.hz_two_byte {
                if self.pos + 1 < self.bytes.len() {
                    let hi = self.bytes[self.pos];
                    let lo = self.bytes[self.pos + 1];
                    self.pos += 2;
                    return Some(((hi as i32) << 8) | (lo as i32));
                } else {
                    let b = self.bytes[self.pos];
                    self.pos += 1;
                    self.hz_two_byte = false;
                    return Some(b as i32);
                }
            } else {
                let b = self.bytes[self.pos];
                self.pos += 1;
                return Some(b as i32);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(bytes: &[u8], encoding: InputEncoding) -> Vec<i32> {
        EncodingDecoder::new(bytes, encoding).collect()
    }

    fn collect_utf8(bytes: &[u8]) -> Vec<i32> {
        collect(bytes, InputEncoding::UTF8)
    }

    fn collect_dbcs(bytes: &[u8]) -> Vec<i32> {
        collect(bytes, InputEncoding::Dbcs)
    }

    fn collect_shiftjis(bytes: &[u8]) -> Vec<i32> {
        collect(bytes, InputEncoding::ShiftJIS)
    }

    fn collect_hz(bytes: &[u8]) -> Vec<i32> {
        collect(bytes, InputEncoding::HZ)
    }

    /* UTF-8 tests */

    #[test]
    fn utf8_empty_returns_none() {
        assert_eq!(collect_utf8(&[]), vec![]);
    }

    #[test]
    fn utf8_ascii_single_byte() {
        assert_eq!(collect_utf8(b"ABC"), vec![65, 66, 67]);
    }

    #[test]
    fn utf8_ascii_null() {
        assert_eq!(collect_utf8(&[0x00]), vec![0]);
    }

    #[test]
    fn utf8_2byte_sequence() {
        // U+00E9 = e-acute, encoded as 0xC3 0xA9
        assert_eq!(collect_utf8(&[0xC3, 0xA9]), vec![0xE9]);
    }

    #[test]
    fn utf8_3byte_sequence() {
        // U+0F60 = Tibetan vowel sign a, encoded as 0xE0 0xBD 0xA0
        assert_eq!(collect_utf8(&[0xE0, 0xBD, 0xA0]), vec![0xF60]);
    }

    #[test]
    fn utf8_4byte_sequence() {
        // U+1F600 = grinning face, encoded as 0xF0 0x9F 0x98 0x80
        assert_eq!(collect_utf8(&[0xF0, 0x9F, 0x98, 0x80]), vec![0x1F600]);
    }

    #[test]
    fn utf8_max_codepoint() {
        // U+10FFFF, encoded as 0xF4 0x8F 0xBF 0xBF
        assert_eq!(collect_utf8(&[0xF4, 0x8F, 0xBF, 0xBF]), vec![0x10FFFF]);
    }

    #[test]
    fn utf8_overlong_1byte_as_2byte() {
        // 0xC0 0xAF is overlong encoding of '/', lead byte rejected (128),
        // continuation byte 0xAF is bare and also yields 128
        assert_eq!(collect_utf8(&[0xC0, 0xAF]), vec![128, 128]);
    }

    #[test]
    fn utf8_overlong_2byte_as_3byte() {
        // 0xE0 0x80 0xAF is overlong encoding of '/', lead byte rejected (128),
        // remaining two continuation bytes each yield 128
        assert_eq!(collect_utf8(&[0xE0, 0x80, 0xAF]), vec![128, 128, 128]);
    }

    #[test]
    fn utf8_overlong_3byte_as_4byte() {
        // 0xF0 0x80 0x80 0xAF is overlong encoding of '/', lead byte rejected (128),
        // remaining three continuation bytes each yield 128
        assert_eq!(collect_utf8(&[0xF0, 0x80, 0x80, 0xAF]), vec![128, 128, 128, 128]);
    }

    #[test]
    fn utf8_surrogate_low_rejected() {
        // U+D800 is a surrogate, encoded as 0xED 0xA0 0x80,
        // lead byte rejected (128), two continuation bytes each yield 128
        assert_eq!(collect_utf8(&[0xED, 0xA0, 0x80]), vec![128, 128, 128]);
    }

    #[test]
    fn utf8_surrogate_high_rejected() {
        // U+DFFF is a surrogate, encoded as 0xED 0xBF 0xBF,
        // lead byte rejected (128), two continuation bytes each yield 128
        assert_eq!(collect_utf8(&[0xED, 0xBF, 0xBF]), vec![128, 128, 128]);
    }

    #[test]
    fn utf8_invalid_continuation_byte() {
        // 0xC2 followed by non-continuation byte 0x00
        assert_eq!(collect_utf8(&[0xC2, 0x00]), vec![128, 0]);
    }

    #[test]
    fn utf8_bare_continuation_byte() {
        // 0x80 is a bare continuation byte
        assert_eq!(collect_utf8(&[0x80]), vec![128]);
    }

    #[test]
    fn utf8_bare_continuation_byte_ff() {
        assert_eq!(collect_utf8(&[0xFF]), vec![128]);
    }

    #[test]
    fn utf8_truncated_2byte() {
        assert_eq!(collect_utf8(&[0xC3]), vec![128]);
    }

    #[test]
    fn utf8_truncated_3byte() {
        assert_eq!(collect_utf8(&[0xE0, 0xBD]), vec![128, 128]);
    }

    #[test]
    fn utf8_truncated_4byte() {
        assert_eq!(collect_utf8(&[0xF0, 0x9F, 0x98]), vec![128, 128, 128]);
    }

    #[test]
    fn utf8_codepoint_above_max() {
        // 0xF5 0x80 0x80 0x80 decodes to 0x110000, above U+10FFFF,
        // lead byte rejected (128), three continuation bytes each yield 128
        assert_eq!(collect_utf8(&[0xF5, 0x80, 0x80, 0x80]), vec![128, 128, 128, 128]);
    }

    #[test]
    fn utf8_mixed_valid_and_invalid() {
        // 'A' (0x41) + valid 2-byte (0xC3 0xA9 = U+00E9) + bare continuation (0x80) + 'B' (0x42)
        assert_eq!(collect_utf8(&[0x41, 0xC3, 0xA9, 0x80, 0x42]), vec![65, 0xE9, 128, 66]);
    }

    /* DBCS tests */

    #[test]
    fn dbcs_empty_returns_none() {
        assert_eq!(collect_dbcs(&[]), vec![]);
    }

    #[test]
    fn dbcs_ascii_bytes() {
        assert_eq!(collect_dbcs(b"AB"), vec![65, 66]);
    }

    #[test]
    fn dbcs_high_byte_pair() {
        assert_eq!(collect_dbcs(&[0xA1, 0xA1]), vec![0xA1A1]);
    }

    #[test]
    fn dbcs_mixed_ascii_and_high_bytes() {
        assert_eq!(collect_dbcs(&[0x41, 0xA1, 0xA1, 0x42]), vec![0x41, 0xA1A1, 0x42]);
    }

    #[test]
    fn dbcs_trailing_stray_high_byte() {
        // High byte at end of input with no low byte partner
        assert_eq!(collect_dbcs(&[0x41, 0xA1]), vec![0x41, 0xA1]);
    }

    #[test]
    fn dbcs_multiple_pairs() {
        assert_eq!(
            collect_dbcs(&[0x81, 0x40, 0x81, 0x41, 0x81, 0x42]),
            vec![0x8140, 0x8141, 0x8142]
        );
    }

    /* Shift-JIS tests */

    #[test]
    fn shiftjis_empty_returns_none() {
        assert_eq!(collect_shiftjis(&[]), vec![]);
    }

    #[test]
    fn shiftjis_ascii_bytes() {
        assert_eq!(collect_shiftjis(b"AB"), vec![65, 66]);
    }

    #[test]
    fn shiftjis_high_byte_range_1() {
        // 0x80-0x9F range
        assert_eq!(collect_shiftjis(&[0x81, 0x40]), vec![0x8140]);
    }

    #[test]
    fn shiftjis_high_byte_range_2() {
        // 0xE0-0xEF range
        assert_eq!(collect_shiftjis(&[0xEA, 0x40]), vec![0xEA40]);
    }

    #[test]
    fn shiftjis_non_high_byte_treated_as_single() {
        // 0xA0-0xDF and 0xF0-0xFF are NOT high bytes in Shift-JIS
        assert_eq!(collect_shiftjis(&[0xA0, 0xA1]), vec![0xA0, 0xA1]);
    }

    #[test]
    fn shiftjis_mixed_content() {
        // ASCII + high-byte pair + ASCII
        assert_eq!(collect_shiftjis(&[0x41, 0x81, 0x40, 0x42]), vec![0x41, 0x8140, 0x42]);
    }

    #[test]
    fn shiftjis_trailing_high_byte() {
        assert_eq!(collect_shiftjis(&[0x81]), vec![0x81]);
    }

    #[test]
    fn shiftjis_high_byte_boundaries() {
        // 0x9F is last byte of first high-byte range
        assert_eq!(collect_shiftjis(&[0x9F, 0x40]), vec![0x9F40]);
        // 0xEF is last byte of second high-byte range
        assert_eq!(collect_shiftjis(&[0xEF, 0x40]), vec![0xEF40]);
        // 0x7F is NOT a high byte
        assert_eq!(collect_shiftjis(&[0x7F, 0x40]), vec![0x7F, 0x40]);
    }

    /* HZ tests */

    #[test]
    fn hz_empty_returns_none() {
        assert_eq!(collect_hz(&[]), vec![]);
    }

    #[test]
    fn hz_ascii_default_mode() {
        assert_eq!(collect_hz(b"AB"), vec![65, 66]);
    }

    #[test]
    fn hz_enter_two_byte_mode() {
        // ~{ enters two-byte mode, then two bytes form a pair
        assert_eq!(collect_hz(&[b'~', b'{', 0xA1, 0xA1]), vec![0xA1A1]);
    }

    #[test]
    fn hz_exit_two_byte_mode() {
        // ~{ enters two-byte mode, ~} exits back to ASCII mode
        assert_eq!(
            collect_hz(&[b'~', b'{', 0xA1, 0xA1, b'~', b'}', b'A']),
            vec![0xA1A1, 65]
        );
    }

    #[test]
    fn hz_tilde_tilde_escape() {
        // ~~ produces a literal tilde
        assert_eq!(collect_hz(&[b'~', b'~']), vec![b'~' as i32]);
    }

    #[test]
    fn hz_stray_tilde_x() {
        // ~X (where X is not {, }, or ~) is removed from input per spec
        assert_eq!(collect_hz(&[b'~', b'X', b'A']), vec![65]);
    }

    #[test]
    fn hz_trailing_tilde_at_end() {
        // Lone ~ at end of input: consumed silently
        assert_eq!(collect_hz(&[b'~']), vec![]);
    }

    #[test]
    fn hz_two_byte_trailing_single_byte() {
        // In two-byte mode, odd byte at end yields single byte
        assert_eq!(
            collect_hz(&[b'~', b'{', 0xA1, 0xA2, 0xA3]),
            vec![0xA1A2, 0xA3]
        );
    }

    #[test]
    fn hz_multiple_mode_switches() {
        // Switch modes multiple times
        assert_eq!(
            collect_hz(&[
                b'A',                   // ASCII: 65
                b'~', b'{',             // Enter two-byte
                0xA1, 0xA2,             // Two-byte: 0xA1A2
                b'~', b'}',             // Exit two-byte
                b'B',                   // ASCII: 66
                b'~', b'{',             // Enter two-byte again
                0xB1, 0xB2,             // Two-byte: 0xB1B2
            ]),
            vec![65, 0xA1A2, 66, 0xB1B2]
        );
    }

    #[test]
    fn hz_stray_tilde_resets_mode() {
        // ~{ enters two-byte, ~X is removed from input (consumes both bytes silently)
        assert_eq!(collect_hz(&[b'~', b'{', b'~', b'X']), vec![]);
    }

    /* Latin1 tests */

    #[test]
    fn latin1_basic() {
        assert_eq!(collect(&[0x41, 0x42, 0x43], InputEncoding::Default), vec![0x41, 0x42, 0x43]);
    }

    #[test]
    fn latin1_high_bytes() {
        assert_eq!(collect(&[0xE9, 0xF1], InputEncoding::Default), vec![0xE9, 0xF1]);
    }

    #[test]
    fn latin1_empty() {
        assert_eq!(collect(&[], InputEncoding::Default), vec![]);
    }
}
