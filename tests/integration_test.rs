use std::path;

macro_rules! new_smusher {
    ( $a: ident, $b: expr ) => {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + &path::MAIN_SEPARATOR.to_string() + $b;
        let font = figdriver::FIGfont::from_path(path).unwrap();
        let mut $a = figdriver::Smusher::new(&font);
        $a.mode = font.layout;
    }
}

fn dummy(_: &[String]) {
}

#[test]
fn line_full() {
    new_smusher!(sm, "tests/test.flf");
    let mut wr = figdriver::Wrapper::new(sm, 8);
    assert!(!wr.push_str("this").is_err());
    assert!(!wr.push_str(" ").is_err());
    assert!(!wr.push_str("is").is_err());
    assert!(!wr.push_str(" ").is_err());
    assert!(wr.push_str("a").is_err());
    assert_eq!(wr.get(), vec!["this is "]);
}

#[test]
fn line_wrap() {
    new_smusher!(sm, "tests/test.flf");
    let mut wr = figdriver::Wrapper::new(sm, 8);
    [ "this", " ", "is", " ", "a", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["a test"]);
}

#[test]
fn wrap_align_left() {
    new_smusher!(sm, "tests/test.flf");
    let mut wr = figdriver::Wrapper::new(sm, 12);
    wr.align = figdriver::Align::Left;
    [ "this", " ", "is", " ", "a", " ", "new", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["new test"]);
}

#[test]
fn wrap_align_center() {
    new_smusher!(sm, "tests/test.flf");
    let mut wr = figdriver::Wrapper::new(sm, 12);
    wr.align = figdriver::Align::Center;
    [ "this", " ", "is", " ", "a", " ", "new", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["  new test"]);
}

#[test]
fn wrap_align_right() {
    new_smusher!(sm, "tests/test.flf");
    let mut wr = figdriver::Wrapper::new(sm, 12);
    wr.align = figdriver::Align::Right;
    [ "this", " ", "is", " ", "a", " ", "new", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["    new test"]);
}

#[test]
fn standard_font_char() {
    new_smusher!(sm, "fonts/standard.flf");
    let mut wr = figdriver::Wrapper::new(sm, 60);
    assert!(!wr.push('A').is_err());
    assert_eq!(wr.get(), vec![r"    _    ",
                              r"   / \   ",
                              r"  / _ \  ",
                              r" / ___ \ ",
                              r"/_/   \_\",
                              r"         "]);
}

#[test]
fn smushing() {
    new_smusher!(sm, "fonts/small.flf");
    let mut wr = figdriver::Wrapper::new(sm, 60);
    assert!(!wr.push_str("Smushy").is_err());
    assert_eq!(wr.get(), vec![r" ___              _        ",
                              r"/ __|_ __ _  _ __| |_ _  _ ",
                              r"\__ \ '  \ || (_-< ' \ || |",
                              r"|___/_|_|_\_,_/__/_||_\_, |",
                              r"                      |__/ "]);
}

#[test]
fn kerning() {
    new_smusher!(sm, "fonts/small.flf");
    sm.mode = figdriver::SMUSH_KERN;
    let mut wr = figdriver::Wrapper::new(sm, 60);
    assert!(!wr.push_str("Kerning").is_err());
    assert_eq!(wr.get(), vec![r" _  __                 _             ",
                              r"| |/ / ___  _ _  _ _  (_) _ _   __ _ ",
                              r"| ' < / -_)| '_|| ' \ | || ' \ / _` |",
                              r"|_|\_\\___||_|  |_||_||_||_||_|\__, |",
                              r"                               |___/ "]);
}

#[test]
fn overlap() {
    new_smusher!(sm, "fonts/standard.flf");
    sm.mode = 0;
    let mut wr = figdriver::Wrapper::new(sm, 60);
    assert!(!wr.push_str("Over Write").is_err());
    assert_eq!(wr.get(), vec![r"  ___                  __        __    _ _       ",
                              r" / _ \__   _____ _ __  \ \      / _ __(_| |_ ___ ",
                              r"| | | \ \ / / _ | '__|  \ \ /\ / | '__| | __/ _ \",
                              r"| |_| |\ V |  __| |      \ V  V /| |  | | ||  __/",
                              r" \___/  \_/ \___|_|       \_/\_/ |_|  |_|\__\___|",
                              r"                                                 "]);
}

#[test]
fn full_width() {
    new_smusher!(sm, "fonts/small.flf");
    sm.full_width = true;
    let mut wr = figdriver::Wrapper::new(sm, 60);
    assert!(!wr.push_str("Full width").is_err());
    assert_eq!(wr.get(), vec![r"  ___          _   _              _      _   _     _    ",
                              r" | __|  _  _  | | | |   __ __ __ (_)  __| | | |_  | |_  ",
                              r" | _|  | || | | | | |   \ V  V / | | / _` | |  _| | ' \ ",
                              r" |_|    \_,_| |_| |_|    \_/\_/  |_| \__,_|  \__| |_||_|",
                              r"                                                        "]);
}

#[test]
fn utf8_input() {
    new_smusher!(sm, "fonts/standard.flf");
    let mut wr = figdriver::Wrapper::new(sm, 60);
    assert!(!wr.push_str("Ação! ಠ_ಠ").is_err());
    assert_eq!(wr.get(), vec![r"    _        /\/|       _    _____)      _____)",
                              r"   / \   ___|/\/_  ___ | |  /_ ___/     /_ ___/",
                              r"  / _ \ / __/ _` |/ _ \| |  / _ \       / _ \  ",
                              r" / ___ \ (_| (_| | (_) |_| | (_) |     | (_) | ",
                              r"/_/   \_\___\__,_|\___/(_)  \___/ _____ \___/  ",
                              r"         )_)                     |_____|       "]);
}

#[test]
fn right_to_left() {
    new_smusher!(sm, "fonts/small.flf");
    sm.right2left = true;
    let mut wr = figdriver::Wrapper::new(sm, 60);
    wr.align = figdriver::Align::Right;
    assert!(!wr.push_str("ABC").is_err());
    let output = wr.get();
    // RTL + right-aligned: content is right-aligned with left padding
    assert_eq!(output[0].len(), 60);
    assert!(output[0].ends_with("___ ___   _   "));
}
