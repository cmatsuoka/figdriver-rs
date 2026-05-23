use crate::figfont;

macro_rules! cmp_return_other {
    ( $a:expr, $b:expr, $c:expr ) => {
        if $a == $b {
            return Some($c)
        }
        if $a == $c {
            return Some($b)
        }
    }
}

macro_rules! find_return_latter {
    ( $p1:expr, $p2:expr, $a:expr, $b:expr ) => {
        if $p1.find($a) != None && $p2.find($b) != None {
            return Some($b)
        }
        if $p1.find($b) != None && $p2.find($a) != None {
            return Some($a)
        }
    }
}

macro_rules! cmp_any_return {
    ( $c1:expr, $c2:expr, $a:expr, $b:expr, $r:expr ) => {
        if ($a == $c1 && $b == $c2) || ($a == $c2 && $b == $c1) {
            return Some($r)
        }
    }
}

macro_rules! cmp_return {
    ( $c1:expr, $c2:expr, $a:expr, $b:expr, $r:expr ) => {
        if $a == $c1 && $b == $c2 {
            return Some($r)
        }
    }
}

macro_rules! apply_rule {
    ( $cmd:expr ) => {
        if let Some(val) = $cmd {
            return Some(val);
        }
    }
}

pub fn smush(l: char, r: char, hardblank: char, right2left: bool, mode: u32) -> Option<char> {

    cmp_return_other!(' ', l, r);

    // Kerning mode
    if mode & figfont::SMUSH_KERN != 0 {
        return None;
    }

    // Universal smushing simply overrides the sub-character from the earlier
    // FIGcharacter with the sub-character from the later FIGcharacter. This
    // produces an "overlapping" effect with some FIGfonts, wherin the latter
    // FIGcharacter may appear to be "in front".
    if mode == 0 {
        // Ensure overlapping preference to visible characters
        //cmp_return_other!(hardblank, l, r);

        // Use this instead of the line above to conserve spacing between words.
        if l == hardblank || r == hardblank {
            return None
        }

        // Ensures that the dominant (foreground) fig-character for overlapping is
        // the latter in the user's text, not necessarily the rightmost character
        if right2left {
            return Some(l)
        }

        return Some(r)
    }

    // Rule 6: HARDBLANK SMUSHING (code value 32)
    // Smushes two hardblanks together, replacing them with a single hardblank.
    if l == hardblank && r == hardblank {
        if mode & figfont::SMUSH_HARDBLANK != 0 {
            return Some(l);
        } else {
            return None;
        }
    }

    apply_rule!(rule_1(l, r, mode));
    apply_rule!(rule_2(l, r, mode));
    apply_rule!(rule_3(l, r, mode));
    apply_rule!(rule_4(l, r, mode));
    apply_rule!(rule_5(l, r, mode));

    None
}


// Rule 1: EQUAL CHARACTER SMUSHING (code value 1)
// Two sub-characters are smushed into a single sub-character if they are the same (except
// hardblanks). 
fn rule_1(l: char, r: char, mode: u32) -> Option<char> {
    if mode & figfont::SMUSH_EQUAL != 0 {
        if l == r {
            return Some(l)
        }
    }

    None
}

// Rule 2: UNDERSCORE SMUSHING (code value 2)
// An underscore ("_") will be replaced by any of: "|", "/", "\", "[", "]", "{", "}", "(",
// ")", "<" or ">".
fn rule_2(l: char, r: char, mode: u32) -> Option<char> {
    if mode & figfont::SMUSH_UNDERLINE != 0 {
        find_return_latter!("_", r"|/\[]{}()<>", l, r);
    }

    None
}

// Rule 3: HIERARCHY SMUSHING (code value 4)
// A hierarchy of six classes is used: "|", "/\", "[]", "{}", "()", and "<>". When two
// smushing sub-characters are from different classes, the one from the latter class will
// be used.
fn rule_3(l: char, r: char, mode: u32) -> Option<char> {
    if mode & figfont::SMUSH_HIERARCHY != 0 {
        find_return_latter!("|", r"/\[]{}()<>", l, r);
        find_return_latter!(r"/\", "[]{}()<>", l, r);
        find_return_latter!("[]", "{}()<>", l, r);
        find_return_latter!("{}", "()<>", l, r);
        find_return_latter!("()", "<>", l, r);
    }

    None
}


// Rule 4: OPPOSITE PAIR SMUSHING (code value 8)
// Smushes opposing brackets ("[]" or "]["), braces ("{}" or "}{") and parentheses ("()"
// or ")(") together, replacing any such pair with a vertical bar ("|").
fn rule_4(l: char, r: char, mode: u32) -> Option<char> {
    if mode & figfont::SMUSH_PAIR != 0 {
        cmp_any_return!('[', ']', l, r, '|');
        cmp_any_return!('{', '}', l, r, '|');
        cmp_any_return!('(', ')', l, r, '|');
    }

    None
}

// Rule 5: BIG X SMUSHING (code value 16)
// Smushes "/\" into "|", "\/" into "Y", and "><" into "X". Note that "<>" is not smushed
// in any way by this rule. The name "BIG X" is historical; originally all three pairs
// were smushed into "X".
fn rule_5(l: char, r: char, mode: u32) -> Option<char> {
    if mode & figfont::SMUSH_BIGX != 0 {
        cmp_return!('/', '\\', l, r, '|');
        cmp_return!('\\', '/', l, r, 'Y');
        cmp_return!('>', '<', l, r, 'X');
    }

    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_1() {
        assert_eq!(rule_1('-', '-', 0), None);
        assert_eq!(rule_1('-', 'x', 0), None);
        assert_eq!(rule_1('-', '-', figfont::SMUSH_EQUAL), Some('-'));
        assert_eq!(rule_1('-', 'x', figfont::SMUSH_EQUAL), None);
    }

    #[test]
    fn test_rule_2() {
        for x in [ '|', '/', '\\', '[', ']', '{', '}', '(', ')', '<', '>' ].iter() {
            assert_eq!(rule_2('_', *x, 0), None);
            assert_eq!(rule_2(*x, '_', 0), None);
            assert_eq!(rule_2('_', *x, figfont::SMUSH_UNDERLINE), Some(*x));
            assert_eq!(rule_2(*x, '_', figfont::SMUSH_UNDERLINE), Some(*x));
        }
        assert_eq!(rule_2('_', 'x', 0), None);
        assert_eq!(rule_2('x', '_', 0), None);
        assert_eq!(rule_2('_', 'x', figfont::SMUSH_UNDERLINE), None);
        assert_eq!(rule_2('x', '_', figfont::SMUSH_UNDERLINE), None);
    }

    #[test]
    fn test_rule_3() {
        let list = [ '|', '/', '\\', '[', ']', '{', '}', '(', ')', '<', '>' ];

        for x in list.iter() {
            for y in list.iter() {
                assert_eq!(rule_3(*x, *y, 0), None);
            }
        }

        assert_eq!(rule_3('|', '|', figfont::SMUSH_HIERARCHY), None);
        for x in list[1..].iter() {
            assert_eq!(rule_3('|', *x, figfont::SMUSH_HIERARCHY), Some(*x));
        }
        for i in 0..4 {
            let idx = 3 + i*2;
            for x in list[idx..idx+1].iter() {
                for y in list[idx+2..].iter() {
                    assert_eq!(rule_3(*x, *y, figfont::SMUSH_HIERARCHY), Some(*y));
                    assert_eq!(rule_3(*y, *x, figfont::SMUSH_HIERARCHY), Some(*y));
                }
            }
        }
    }

    #[test]
    fn test_rule_4() {
        assert_eq!(rule_4('[', ']', 0), None);
        assert_eq!(rule_4(']', '[', 0), None);
        assert_eq!(rule_4('{', '}', 0), None);
        assert_eq!(rule_4('}', '{', 0), None);
        assert_eq!(rule_4('(', ')', 0), None);
        assert_eq!(rule_4(')', '(', 0), None);
        assert_eq!(rule_4('[', ']', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(rule_4(']', '[', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(rule_4('{', '}', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(rule_4('}', '{', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(rule_4('(', ')', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(rule_4(')', '(', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(rule_4('(', 'x', figfont::SMUSH_PAIR), None);
        assert_eq!(rule_4('(', '}', figfont::SMUSH_PAIR), None);
        assert_eq!(rule_4('(', ']', figfont::SMUSH_PAIR), None);
        assert_eq!(rule_4('(', '(', figfont::SMUSH_PAIR), None);
    }

    #[test]
    fn test_rule_5() {
        assert_eq!(rule_5('/', '\\', 0), None);
        assert_eq!(rule_5('\\', '/', 0), None);
        assert_eq!(rule_5('>', '<', 0), None);
        assert_eq!(rule_5('<', '>', 0), None);
        assert_eq!(rule_5('/', '\\', figfont::SMUSH_BIGX), Some('|'));
        assert_eq!(rule_5('\\', '/', figfont::SMUSH_BIGX), Some('Y'));
        assert_eq!(rule_5('>', '<', figfont::SMUSH_BIGX), Some('X'));
        assert_eq!(rule_5('<', '>', figfont::SMUSH_BIGX), None);
    }

    // Mode 0 (universal) prefers right char, drops hardblank-only collisions
    #[test]
    fn test_smush_mode_0() {
        assert_eq!(smush('x', 'y', '$', false, 0), Some('y'));
        assert_eq!(smush('x', '$', '$', false, 0), None);
        assert_eq!(smush('$', 'x', '$', false, 0), None);
        assert_eq!(smush('$', '$', '$', false, 0), None);
        assert_eq!(smush('a', 'a', '$', false, 0), Some('a'));
        assert_eq!(smush(' ', 'x', '$', false, 0), Some('x'));
        assert_eq!(smush('x', ' ', '$', false, 0), Some('x'));
    }

    // Right-to-left mode prefers the left character over the right
    #[test]
    fn test_smush_right2left() {
        assert_eq!(smush('x', 'y', '$', true, 0), Some('x'));
        assert_eq!(smush('x', 'y', '$', false, 0), Some('y'));
    }

    // Kern mode drops the left character entirely, returning None
    #[test]
    fn test_smush_kern() {
        assert_eq!(smush('x', 'x', '$', false, figfont::SMUSH_KERN), None);
        assert_eq!(smush('x', 'y', '$', false, figfont::SMUSH_KERN), None);
        assert_eq!(smush('/', '\\', '$', false, figfont::SMUSH_KERN), None);
    }

    // Hardblank mode only matches two hardblanks against each other
    #[test]
    fn test_smush_hardblank() {
        assert_eq!(smush('$', '$', '$', false, figfont::SMUSH_HARDBLANK), Some('$'));
        assert_eq!(smush('$', '$', '$', false, 0), None);
        assert_eq!(smush('x', '$', '$', false, figfont::SMUSH_HARDBLANK), None);
    }

    // Spaces are replaced by non-space characters in any mode
    #[test]
    fn test_smush_space() {
        assert_eq!(smush(' ', 'x', '$', false, 0xbf), Some('x'));
        assert_eq!(smush('x', ' ', '$', false, 0xbf), Some('x'));
        assert_eq!(smush(' ', ' ', '$', false, 0xbf), Some(' '));
    }

    // Multiple mode bits combined: equal and bigx rules both apply
    #[test]
    fn test_smush_combined_modes() {
        let mode = figfont::SMUSH_EQUAL | figfont::SMUSH_BIGX;
        assert_eq!(smush('x', 'x', '$', false, mode), Some('x'));
        assert_eq!(smush('/', '\\', '$', false, mode), Some('|'));
        assert_eq!(smush('>', '<', '$', false, mode), Some('X'));
        assert_eq!(smush('[', ']', '$', false, mode), None);
    }

    // Multi-byte unicode characters smush correctly
    #[test]
    fn test_smush_utf8() {
        assert_eq!(smush('á', 'é', '$', false, 0), Some('é'));
        assert_eq!(smush('á', 'á', '$', false, figfont::SMUSH_EQUAL), Some('á'));
    }
}
