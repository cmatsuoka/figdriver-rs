use super::charsmush;

trait CharExt {
    fn char_len(&self) -> usize;
    fn char_index(&self, _: usize) -> usize;
}

impl<'a> CharExt for &'a str {
    fn char_len(&self) -> usize {
        self.chars().count()
    }

    fn char_index(&self, i: usize) -> usize {
        self.char_indices().nth(i).unwrap().0
    }
}

// Compute the number of characters a string can be smushed into another string.
pub fn amount(s1: &str, s2: &str, hardblank: char, mode: u32) -> usize {

    let mut v1 = s1.chars().rev();
    let mut v2 = s2.chars();
    let mut amt = 0;
       
    let mut l = ' ';
    if !s1.is_empty() {
        l = v1.next().unwrap();
        while l.is_whitespace() {
            amt += 1;
            l = match v1.next() {
                Some(val) => val,
                None      => break,
            };
        }
    }

    let mut r = ' ';
    if !s2.is_empty() {
        r = v2.next().unwrap();
        while r.is_whitespace() {
            amt += 1;
            r = match v2.next() {
                Some(val) => val,
                None      => break,
            };
        }
    }

    if l == ' ' || r == ' ' {
        return amt;
    }

    match charsmush::smush(l, r, hardblank, false, mode) {
        Some(_) => { amt + 1},
        None    => { amt },
    }
}

pub fn smush(s1: &str, s2x: &str, mut amt: usize, hardblank: char, mode: u32) -> String {

    if s2x.is_empty() {
        return s1.to_owned();
    }

    let mut s2 = s2x;
    let l1 = s1.char_len();

    if amt > l1 {
        s2 = &s2[s2.char_index(amt-l1)..];
        amt = l1;
    }

    let l2 = s2.char_len();
    let mut res = "".to_owned();
    let m1 = l1 - amt;

    // part 1: only characters from s1
    // don't use the index operator, we want characters not bytes
    let mut v1 = s1.chars();
    (0..m1).for_each(|_| res.push(v1.next().unwrap()));

    // part 2: s1 and s2 overlap
    let mut v2 = s2.chars();
    (0..l2).for_each(|_| {
        let l = match v1.next() {
            Some(v) => v,
            None    => ' ',
        };
        let r = v2.next().unwrap();
        if l != ' ' && r != ' ' {
            match charsmush::smush(l, r, hardblank, false, mode) {
                Some(c) => res.push(c),
                None    => res.push(r),
            }
        } else {
            res.push(if l == ' ' { r } else { l });
        }
    });

    // part 3: remainder of s1 after the end of s2
    // don't use the index operator, we want characters not bytes
    let m2 = m1 + l2;
    (m2..l1).for_each(|_| res.push(v1.next().unwrap()));

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_len() {
        assert_eq!("aeiou".char_len(), 5);
        assert_eq!("áéíóú".char_len(), 5);
    }

    #[test]
    fn test_amount() {
        assert_eq!(amount("", "", '$', 0xbf), 0);

        assert_eq!(amount("", "    ", '$', 0xbf), 4);
        assert_eq!(amount("", "   y", '$', 0xbf), 3);

        assert_eq!(amount("    ", "    ", '$', 0xbf), 8);
        assert_eq!(amount("x   ", "    ", '$', 0xbf), 7);
        assert_eq!(amount("xx  ", "    ", '$', 0xbf), 6);
        assert_eq!(amount("xxx ", "    ", '$', 0xbf), 5);
        assert_eq!(amount("xxxx", "    ", '$', 0xbf), 4);

        assert_eq!(amount("    ", "   y", '$', 0xbf), 7);
        assert_eq!(amount("x   ", "   y", '$', 0xbf), 6);
        assert_eq!(amount("xx  ", "   y", '$', 0xbf), 5);
        assert_eq!(amount("xxx ", "   y", '$', 0xbf), 4);
        assert_eq!(amount("xxxx", "   y", '$', 0xbf), 3);

        assert_eq!(amount("    ", "  yy", '$', 0xbf), 6);
        assert_eq!(amount("x   ", "  yy", '$', 0xbf), 5);
        assert_eq!(amount("xx  ", "  yy", '$', 0xbf), 4);
        assert_eq!(amount("xxx ", "  yy", '$', 0xbf), 3);
        assert_eq!(amount("xxxx", "  yy", '$', 0xbf), 2);

        assert_eq!(amount("    ", " yyy", '$', 0xbf), 5);
        assert_eq!(amount("x   ", " yyy", '$', 0xbf), 4);
        assert_eq!(amount("xx  ", " yyy", '$', 0xbf), 3);
        assert_eq!(amount("xxx ", " yyy", '$', 0xbf), 2);
        assert_eq!(amount("xxxx", " yyy", '$', 0xbf), 1);

        assert_eq!(amount("    ", "yyyy", '$', 0xbf), 4);
        assert_eq!(amount("x   ", "yyyy", '$', 0xbf), 3);
        assert_eq!(amount("xx  ", "yyyy", '$', 0xbf), 2);
        assert_eq!(amount("xxx ", "yyyy", '$', 0xbf), 1);
        assert_eq!(amount("xxxx", "yyyy", '$', 0xbf), 0);

        assert_eq!(amount("x", "y", '$', 0xbf), 0);
        assert_eq!(amount("x", "x", '$', 0xbf), 1);     // rule 1
        assert_eq!(amount("<", ">", '$', 0xbf), 0);
        assert_eq!(amount("_", "/", '$', 0xbf), 1);     // rule 2
        assert_eq!(amount("/", "_", '$', 0xbf), 1);     // rule 2
        assert_eq!(amount("[", "{", '$', 0xbf), 1);     // rule 3
        assert_eq!(amount("[", "]", '$', 0xbf), 1);     // rule 4
        assert_eq!(amount(">", "<", '$', 0xbf), 1);     // rule 5
        assert_eq!(amount("[ ", " {", '$', 0xbf), 3);   // rule 3 + spacing
    }

    #[test]
    fn test_amount_utf8() {
        assert_eq!(amount("", "   é", '$', 0xbf), 3);
        assert_eq!(amount("á   ", "    ", '$', 0xbf), 7);
        assert_eq!(amount("áá  ", "    ", '$', 0xbf), 6);
        assert_eq!(amount("á   ", "   é", '$', 0xbf), 6);
        assert_eq!(amount("áá  ", "   é", '$', 0xbf), 5);
        assert_eq!(amount("á   ", "  éé", '$', 0xbf), 5);
        assert_eq!(amount("áá  ", "  éé", '$', 0xbf), 4);
    }

    #[test]
    fn test_smush() {
        assert_eq!(smush("123! ", "xy", 1, '$', 0xbf), "123!xy".to_string());
        assert_eq!(smush("123> ", "<y", 2, '$', 0xbf), "123Xy".to_string());
        assert_eq!(smush("123! ", "   xy", 5, '$', 0xbf), "123xy".to_string());
        assert_eq!(smush("123/ ", "   /y", 5, '$', 0xbf), "123/y".to_string());
        assert_eq!(smush("", "   y", 3, '$', 0xbf), "y".to_string());
        assert_eq!(smush("", "      ", 1, '$', 0xbf), "     ".to_string());
    }

    #[test]
    fn test_smush_utf8() {
        assert_eq!(smush("áéí! ", "óú", 1, '$', 0xbf), "áéí!óú".to_string());
        assert_eq!(smush("", "   á", 3, '$', 0xbf), "á".to_string());
    }
}
