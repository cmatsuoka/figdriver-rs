/// ISO 2022 character set state, accumulated from "g" commands in control files.
#[derive(Debug, Clone)]
pub struct Iso2022Settings {
    pub(super) g_sets: [Option<Iso2022GSet>; 4],
    pub(super) left_half: usize,
    pub(super) right_half: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Iso2022GSet {
    pub(super) base_code: i32,
    pub(super) double_byte: bool,
}

impl Default for Iso2022Settings {
    fn default() -> Self {
        Self {
            g_sets: [
                Some(Iso2022GSet { base_code: 0, double_byte: false }),
                Some(Iso2022GSet { base_code: 0x80, double_byte: false }),
                None,
                None,
            ],
            left_half: 0,
            right_half: 1,
        }
    }
}

impl Iso2022Settings {
    pub fn build_decoder(&self) -> Iso2022Decoder<'_> {
        let mut gn = [0i32; 4];
        let mut gndbl = [false; 4];

        for (i, gset) in self.g_sets.iter().enumerate() {
            if let Some(gs) = gset {
                gn[i] = gs.base_code;
                gndbl[i] = gs.double_byte;
            }
        }

        Iso2022Decoder {
            bytes: &[],
            pos: 0,
            gn,
            gndbl,
            gl: self.left_half,
            gr: self.right_half,
            single_shift: None,
        }
    }
}

/// ISO 2022 stream-level decoder.
///
/// Consumes raw bytes, intercepts ESC sequences to switch G-registers,
/// and yields one `i32` character code per call.
pub struct Iso2022Decoder<'a> {
    bytes: &'a [u8],
    pos: usize,
    gn: [i32; 4],
    gndbl: [bool; 4],
    gl: usize,
    gr: usize,
    single_shift: Option<usize>,
}

impl Default for Iso2022Decoder<'_> {
    fn default() -> Self {
        Self {
            bytes: &[],
            pos: 0,
            gn: [0, 0x80, 0, 0],
            gndbl: [false; 4],
            gl: 0,
            gr: 1,
            single_shift: None,
        }
    }
}

impl<'a> Iso2022Decoder<'a> {
    pub fn set_input(&mut self, bytes: &'a [u8]) {
        self.bytes = bytes;
        self.pos = 0;
    }

    /// Return the next character code from the input stream.
    pub fn next(&mut self) -> Option<i32> {
        if self.pos >= self.bytes.len() {
            return None;
        }

        let ch = self.read_byte()?;

        match ch {
            // SO: invoke G1 into GL
            0x0E => { self.single_shift = None; self.gl = 1; self.next() }
            // SI: invoke G0 into GL
            0x0F => { self.single_shift = None; self.gl = 0; self.next() }
            // SS2 (8-bit): invoke G2 into GL for next char only
            0x8E => { self.single_shift = Some(2); self.next() }
            // SS3 (8-bit): invoke G3 into GL for next char only
            0x8F => { self.single_shift = Some(3); self.next() }
            // ESC sequence
            27 => {
                self.single_shift = None;
                let second = self.read_byte().unwrap_or(0);
                self.handle_escape(second)
            }
            _ => self.decode_char(ch),
        }
    }

    fn read_byte(&mut self) -> Option<u8> {
        if self.pos >= self.bytes.len() {
            None
        } else {
            let b = self.bytes[self.pos];
            self.pos += 1;
            Some(b)
        }
    }

    /// Handle an ESC sequence. The `second` byte is the byte immediately after ESC.
    fn handle_escape(&mut self, second: u8) -> Option<i32> {
        let (base, third) = if second == b'$' {
            let t = self.read_byte().unwrap_or(0);
            (0x200, Some(t))
        } else {
            (0x100, None)
        };

        let ch = third.unwrap_or(second);

        // SI (14) and SO (15) come through as raw bytes, not ESC sequences
        // Handle ESC-letter sequences
        match (base, ch) {
            // SS2: invoke G2 into GL for next char only (ESC N)
            (0x100, b'N') => { self.single_shift = Some(2); return self.next(); }
            // SS3: invoke G3 into GL for next char only (ESC O)
            (0x100, b'O') => { self.single_shift = Some(3); return self.next(); }
            // LS2: invoke G2 into GL
            (0x100, b'n') => { self.gl = 2; return self.next(); }
            // LS3: invoke G3 into GL
            (0x100, b'o') => { self.gl = 3; return self.next(); }
            // GR switches
            (0x100, b'~') => { self.gr = 1; return self.next(); }
            (0x100, b'}') => { self.gr = 2; return self.next(); }
            (0x100, b'|') => { self.gr = 3; return self.next(); }
            // 94-char G0-G3
            (0x100, b'(') => { self.designate(0, 94); return self.next(); }
            (0x100, b')') => { self.designate(1, 94); return self.next(); }
            (0x100, b'*') => { self.designate(2, 94); return self.next(); }
            (0x100, b'+') => { self.designate(3, 94); return self.next(); }
            // 96-char G1-G3
            (0x100, b'-') => { self.designate(1, 96); return self.next(); }
            (0x100, b'.') => { self.designate(2, 96); return self.next(); }
            (0x100, b'/') => { self.designate(3, 96); return self.next(); }
            // 94x94 G0-G3
            (0x200, b'(') => { self.designate(0, 9999); return self.next(); }
            (0x200, b')') => { self.designate(1, 9999); return self.next(); }
            (0x200, b'*') => { self.designate(2, 9999); return self.next(); }
            (0x200, b'+') => { self.designate(3, 9999); return self.next(); }
            // Deprecated: ESC $ x (paren-less)
            _ if base == 0x200 => {
                self.gn[0] = (ch as i32) << 16;
                self.gndbl[0] = true;
                return self.next();
            }
            // Not implemented: ESC SP F (ACS - ECMA-35 section 15.2)
            // Not implemented: ESC ! F (CZD - C0 designate, ECMA-35 section 14.2)
            // Not implemented: ESC " F (C1D - C1 designate, ECMA-35 section 14.2)
            // Not needed for FIGfont; reference figlet-2.2.5 also omits these
            _ => {}
        }

        self.next()
    }

    fn designate(&mut self, reg: usize, kind: i32) {
        let mut d = self.read_byte().unwrap_or(0) as i32;
        if (kind == 94 && d == b'B' as i32)
            || ((kind == 96 || kind == 9999) && d == b'A' as i32)
        {
            d = 0;
        }
        if kind == 9999 {
            self.gn[reg] = d << 16;
            self.gndbl[reg] = true;
          } else if kind == 96 {
                // 0x80 flag marks 96-char sets in output codes (matches figlet-2.2.5)
                self.gn[reg] = (d << 16) | 0x80;
            self.gndbl[reg] = false;
        } else {
            self.gn[reg] = d << 16;
            self.gndbl[reg] = false;
        }
    }

    fn decode_char(&mut self, ch: u8) -> Option<i32> {
        if (0x21..=0x7E).contains(&ch) {
            let gl = self.single_shift.take().unwrap_or(self.gl);
            if self.gndbl[gl] {
                let ch2 = self.read_byte().unwrap_or(0) as i32;
                Some(self.gn[gl] | ((ch as i32) << 8) | ch2)
            } else {
                Some(self.gn[gl] | (ch as i32))
            }
        } else if (0xA0..=0xFF).contains(&ch) {
            if self.gndbl[self.gr] {
                let ch2 = self.read_byte().unwrap_or(0) as i32;
                Some(self.gn[self.gr] | ((ch as i32) << 8) | ch2)
            } else {
                Some(self.gn[self.gr] | (ch as i32 & !0x80))
            }
        } else {
            Some(ch as i32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(input: &[u8], settings: &Iso2022Settings) -> Vec<i32> {
        let mut decoder = settings.build_decoder();
        decoder.set_input(input);
        let mut result = Vec::new();
        while let Some(code) = decoder.next() {
            result.push(code);
        }
        result
    }

    #[test]
    fn default_ascii_passthrough() {
        let settings = Iso2022Settings::default();
        assert_eq!(decode(b"ABC", &settings), vec![65, 66, 67]);
    }

    #[test]
    fn right_half_gr() {
        let settings = Iso2022Settings::default();
        assert_eq!(decode(&[0xA0], &settings), vec![0xA0]);
        assert_eq!(decode(&[0xE9], &settings), vec![0xE9]);
    }

    #[test]
    fn esc_paren_designate_g0() {
        let settings = Iso2022Settings::default();
        let input = [0x1B, b'(', b'B', 0x41];
        assert_eq!(decode(&input, &settings), vec![0x41]);
    }

    #[test]
    fn esc_paren_designate_g0_custom() {
        let settings = Iso2022Settings::default();
        let input = [0x1B, b'(', b'X', 0x41];
        assert_eq!(decode(&input, &settings), vec![0x580041]);
    }

    #[test]
    fn esc_close_paren_designate_g1_94() {
        let settings = Iso2022Settings::default();
        let input = [0x1B, b')', b'X', 0x0E, 0x41];
        assert_eq!(decode(&input, &settings), vec![0x580041]);
    }

    #[test]
    fn esc_dash_designate_g1_96() {
        let settings = Iso2022Settings::default();
        // ESC - B designates G1 as 96-char set with designator 0x42
        // gn[1] = (0x42 << 16) | 0x80 = 0x420080
        // SI switches GL to G1, then 0x41 -> 0x420080 | 0x41 = 0x4200C1
        let input = [0x1B, b'-', b'B', 0x0E, 0x41];
        assert_eq!(decode(&input, &settings), vec![0x4200C1]);
    }

    #[test]
    fn esc_dollar_paren_94x94() {
        let settings = Iso2022Settings::default();
        let input = [0x1B, b'$', b'(', b'X', 0x41, 0x52];
        assert_eq!(decode(&input, &settings), vec![0x584152]);
    }

    #[test]
    fn esc_n_gl_switch() {
        let settings = Iso2022Settings::default();
        let input = [0x1B, b'n', 0x41, 0x42];
        assert_eq!(decode(&input, &settings), vec![0x41, 0x42]);
    }

    #[test]
    fn control_chars_passthrough() {
        let settings = Iso2022Settings::default();
        assert_eq!(decode(&[0x0A, 0x0D, 0x00], &settings), vec![10, 13, 0]);
    }

    #[test]
    fn empty_input() {
        let settings = Iso2022Settings::default();
        assert_eq!(decode(&[], &settings), vec![]);
    }

    #[test]
    fn ss2_esc_n_temporary_g2() {
        let settings = Iso2022Settings::default();
        // ESC * X designates G2 as 94-char set, then ESC N invokes G2 for one char
        let input = [0x1B, b'*', b'X', 0x1B, b'N', 0x41, 0x1B, b'n', 0x42];
        // ESC N A -> G2 decode: 0x580041
        // ESC n -> LS2 (GL=G2)
        // 0x42 -> G2 decode: 0x580042
        assert_eq!(decode(&input, &settings), vec![0x580041, 0x580042]);
    }

    #[test]
    fn ss3_esc_o_temporary_g3() {
        let settings = Iso2022Settings::default();
        // ESC + Y designates G3 as 94-char set, then ESC O invokes G3 for one char
        let input = [0x1B, b'+', b'Y', 0x1B, b'O', 0x41];
        // ESC O A -> G3 decode: 0x590041
        assert_eq!(decode(&input, &settings), vec![0x590041]);
    }

    #[test]
    fn ss2_restores_gl_after_one_char() {
        let settings = Iso2022Settings::default();
        // ESC ) X designates G1, SO switches to G1, ESC * Y designates G2
        // ESC N A -> G2 (one char, Y=0x59), then B -> G1 (restored, X=0x58)
        let input = [0x1B, b')', b'X', 0x0E, 0x1B, b'*', b'Y', 0x1B, b'N', 0x41, 0x42];
        assert_eq!(decode(&input, &settings), vec![0x590041, 0x580042]);
    }

    #[test]
    fn ss2_followed_by_locking_shift_preserves_locking_shift() {
        let settings = Iso2022Settings::default();
        // ESC ) X designates G1, ESC * Y designates G2
        // ESC N sets SS2 (single shift G2), then SO sets GL=G1 permanently
        // SO clears the single shift, so both A and B decode from G1
        // (with the old recursive code, SO's effect would be lost due to
        // gl restoration, causing B to decode from G0 instead of G1)
        let input = [0x1B, b')', b'X', 0x1B, b'*', b'Y', 0x1B, b'N', 0x0E, 0x41, 0x42];
        assert_eq!(decode(&input, &settings), vec![0x580041, 0x580042]);
    }
}
