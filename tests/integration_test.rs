use std::path::PathBuf;

fn load_font(path: &str) -> figdriver::FIGfont {
    figdriver::FIGfont::from_path(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
    ).unwrap()
}

macro_rules! new_smusher {
    ( $font:ident, $wr:ident, $path:expr, $width:expr, $align:expr ) => {
        let $font = load_font($path);
        let mut $wr = figdriver::Wrapper::new(
            figdriver::Smusher::new(&$font),
            $width,
            $align,
        );
    };
    ( $font:ident, $wr:ident, $path:expr, $width:expr, $align:expr, mode $mode:expr ) => {
        let $font = load_font($path);
        let mut $wr = figdriver::Wrapper::new(
            figdriver::Smusher::builder(&$font)
                .layout_mode($mode)
                .build(),
            $width,
            $align,
        );
    };
    ( $font:ident, $wr:ident, $path:expr, $width:expr, $align:expr, rtl ) => {
        let $font = load_font($path);
        let mut $wr = figdriver::Wrapper::new(
            figdriver::Smusher::builder(&$font)
                .right_to_left(true)
                .build(),
            $width,
            $align,
        );
    };
    ( $font:ident, $wr:ident, $path:expr, $width:expr, $align:expr, mode $mode:expr, rtl ) => {
        let $font = load_font($path);
        let mut $wr = figdriver::Wrapper::new(
            figdriver::Smusher::builder(&$font)
                .layout_mode($mode)
                .right_to_left(true)
                .build(),
            $width,
            $align,
        );
    };
}

fn dummy(_: &[String]) {
}

#[test]
fn line_full() {
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 8, figdriver::Align::Left);
    assert!(!wr.push_str("this").is_err());
    assert!(!wr.push_str(" ").is_err());
    assert!(!wr.push_str("is").is_err());
    assert!(!wr.push_str(" ").is_err());
    assert!(wr.push_str("a").is_err());
    assert_eq!(wr.get(), vec!["this is "]);
}

#[test]
fn line_wrap() {
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 8, figdriver::Align::Left);
    [ "this", " ", "is", " ", "a", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["a test"]);
}

#[test]
fn wrap_align_left() {
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 12, figdriver::Align::Left);
    [ "this", " ", "is", " ", "a", " ", "new", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["new test"]);
}

#[test]
fn wrap_align_center() {
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 12, figdriver::Align::Center);
    [ "this", " ", "is", " ", "a", " ", "new", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["  new test"]);
}

#[test]
fn wrap_align_right() {
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 12, figdriver::Align::Right);
    [ "this", " ", "is", " ", "a", " ", "new", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["    new test"]);
}

#[test]
fn standard_font_char() {
    new_smusher!(fnt, wr, "fonts/standard.flf", 60, figdriver::Align::Left);
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
    new_smusher!(fnt, wr, "fonts/small.flf", 60, figdriver::Align::Left);
    assert!(!wr.push_str("Smushy").is_err());
    assert_eq!(wr.get(), vec![r" ___              _        ",
                              r"/ __|_ __ _  _ __| |_ _  _ ",
                              r"\__ \ '  \ || (_-< ' \ || |",
                              r"|___/_|_|_\_,_/__/_||_\_, |",
                              r"                      |__/ "]);
}

#[test]
fn kerning() {
    new_smusher!(fnt, wr, "fonts/small.flf", 60, figdriver::Align::Left, mode figdriver::LayoutMode::Kern);
    assert!(!wr.push_str("Kerning").is_err());
    assert_eq!(wr.get(), vec![r" _  __                 _             ",
                              r"| |/ / ___  _ _  _ _  (_) _ _   __ _ ",
                              r"| ' < / -_)| '_|| ' \ | || ' \ / _` |",
                              r"|_|\_\\___||_|  |_||_||_||_||_|\__, |",
                              r"                               |___/ "]);
}

#[test]
fn overlap() {
    new_smusher!(fnt, wr, "fonts/standard.flf", 60, figdriver::Align::Left, mode figdriver::LayoutMode::Overlap);
    assert!(!wr.push_str("Over Write").is_err());
    assert_eq!(wr.get(), vec![r"  ___                __        __    _ _       ",
                              r" / _ \__   _____ _ __\ \      / _ __(_| |_ ___ ",
                              r"| | | \ \ / / _ | '__|\ \ /\ / | '__| | __/ _ \",
                              r"| |_| |\ V |  __| |    \ V  V /| |  | | ||  __/",
                              r" \___/  \_/ \___|_|     \_/\_/ |_|  |_|\__\___|",
                              r"                                               "]);
}

#[test]
fn full_width() {
    new_smusher!(fnt, wr, "fonts/small.flf", 60, figdriver::Align::Left, mode figdriver::LayoutMode::FullWidth);
    assert!(!wr.push_str("Full width").is_err());
    assert_eq!(wr.get(), vec![r"  ___          _   _              _      _   _     _    ",
                              r" | __|  _  _  | | | |   __ __ __ (_)  __| | | |_  | |_  ",
                              r" | _|  | || | | | | |   \ V  V / | | / _` | |  _| | ' \ ",
                              r" |_|    \_,_| |_| |_|    \_/\_/  |_| \__,_|  \__| |_||_|",
                              r"                                                        "]);
}

#[test]
fn utf8_input() {
    new_smusher!(fnt, wr, "fonts/standard.flf", 60, figdriver::Align::Left);
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
    new_smusher!(fnt, wr, "fonts/small.flf", 60, figdriver::Align::Right, rtl);
    assert!(!wr.push_str("ABC").is_err());
    let output = wr.get();
    // RTL + right-aligned: content is right-aligned with left padding
    assert_eq!(output[0].len(), 60);
    assert!(output[0].ends_with("___ ___   _   "));
}

#[test]
fn consecutive_blanks_collapsed_at_wrap() {
    // Multiple blanks at a wrap point should be discarded per spec.
    // With preserved whitespace, "this   is" exceeds width 8, wrapping earlier.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 8, figdriver::Align::Left);
    [ "this", "   ", "is", " ", "a", " ", "test" ].iter().for_each(|x| wr.wrap_str(x, &dummy));
    assert_eq!(wr.get(), vec!["test"]);
}

#[test]
fn leading_blanks_preserved() {
    // Blanks at the start of input should be rendered as FIGcharacters.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 30, figdriver::Align::Left);
    wr.wrap_str("   ", &dummy);
    wr.wrap_str("x", &dummy);
    assert_eq!(wr.get(), vec!["   x"]);
}

#[test]
fn rtl_consecutive_blanks_collapsed_at_wrap() {
    // Multiple blanks at a wrap point should be discarded in RTL mode.
    // In RTL mode with the test font, text is reversed (chars prepended left).
    // With preserved whitespace, "this   is" exceeds width 8, wrapping earlier.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 8, figdriver::Align::Left, rtl);
    [ "this", "   ", "is", " ", "a", " ", "test" ].iter().for_each(|x| wr.wrap_str(x, &dummy));
    assert_eq!(wr.get(), vec!["tset"]);
}

#[test]
fn rtl_leading_blanks_preserved() {
    // Blanks at the start of input should be rendered as FIGcharacters in RTL mode.
    // In RTL, leading blanks become trailing rendered spaces, preserved in output (figlet behavior).
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 30, figdriver::Align::Left, rtl);
    wr.wrap_str("   ", &dummy);
    wr.wrap_str("x", &dummy);
    // After RTL smushing: "x   " (x prepended left of 3 spaces)
    assert_eq!(wr.get(), vec!["x   "]);
}

#[test]
fn rtl_inter_word_blanks_preserved() {
    // Multiple blanks between words should be preserved in RTL mode.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 30, figdriver::Align::Left, rtl);
    [ "a", "   ", "b" ].iter().for_each(|x| wr.wrap_str(x, &dummy));
    assert_eq!(wr.get(), vec!["b   a"]);
}

#[test]
fn rtl_blank_after_wrap_discarded() {
    // Whitespace immediately after a flush should be discarded in RTL mode.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 4, figdriver::Align::Left, rtl);
    [ "this", "   ", "is", " ", "a", " ", "test" ].iter().for_each(|x| wr.wrap_str(x, &dummy));
    // "this" (4) wraps, "is a" (4) wraps, "test" (4) → RTL rendered as "tset"
    assert_eq!(wr.get(), vec!["tset"]);
}

#[test]
fn inter_word_blanks_preserved() {
    // Multiple blanks between words should be preserved.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 30, figdriver::Align::Left);
    [ "a", "   ", "b" ].iter().for_each(|x| wr.wrap_str(x, &dummy));
    assert_eq!(wr.get(), vec!["a   b"]);
}

#[test]
fn blank_after_wrap_discarded() {
    // Whitespace immediately after a flush should be discarded.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 4, figdriver::Align::Left);
    [ "this", "   ", "is", " ", "a", " ", "test" ].iter().for_each(|x| wr.wrap_str(x, &dummy));
    assert_eq!(wr.get(), vec!["test"]);
}

#[test]
fn smush_force_with_enable_uses_font_layout() {
    // -S on a font with SMUSH_ENABLE sets mode to font.layout
    let font = load_font("fonts/small.flf");
    assert!((font.layout & figdriver::SMUSH_ENABLE) != 0);
    new_smusher!(fnt, wr, "fonts/small.flf", 60, figdriver::Align::Left, mode figdriver::LayoutMode::SmushForce);
    assert!(!wr.push_str("Smushy").is_err());
    let output = wr.get();
    assert_eq!(output, vec![r" ___              _        ",
                            r"/ __|_ __ _  _ __| |_ _  _ ",
                            r"\__ \ '  \ || (_-< ' \ || |",
                            r"|___/_|_|_\_,_/__/_||_\_, |",
                            r"                      |__/ "]);
}

#[test]
fn smush_force_without_enable_falls_back_to_overlap() {
    // -S on a font without SMUSH_ENABLE (banner, layout=64) falls back to overlap mode 0
    let font = load_font("fonts/banner.flf");
    assert!((font.layout & figdriver::SMUSH_ENABLE) == 0);
    new_smusher!(fnt, wr, "fonts/banner.flf", 60, figdriver::Align::Left, mode figdriver::LayoutMode::SmushForce);
    assert!(!wr.push_str("Hi").is_err());
    let output = wr.get();
    assert!(!output[0].is_empty());
    let smushed_width = output[0].chars().count();
    new_smusher!(fnt2, wr_full, "fonts/banner.flf", 60, figdriver::Align::Left, mode figdriver::LayoutMode::FullWidth);
    assert!(!wr_full.push_str("Hi").is_err());
    let full_width = wr_full.get()[0].chars().count();
    assert!(smushed_width < full_width);
}

#[test]
fn smush_force_overrides_full_width() {
    // -S should disable full_width mode
    new_smusher!(fnt, wr, "fonts/small.flf", 60, figdriver::Align::Left, mode figdriver::LayoutMode::SmushForce);
    assert!(!wr.push_str("Hi").is_err());
    let output = wr.get();
    let smushed_width = output[0].chars().count();
    new_smusher!(fnt2, wr_full, "fonts/small.flf", 60, figdriver::Align::Left, mode figdriver::LayoutMode::FullWidth);
    assert!(!wr_full.push_str("Hi").is_err());
    let full_width = wr_full.get()[0].chars().count();
    assert!(smushed_width < full_width);
}

#[test]
fn smush_force_overrides_kern() {
    // -S should take precedence over kern mode
    new_smusher!(fnt, wr, "fonts/small.flf", 60, figdriver::Align::Left, mode figdriver::LayoutMode::SmushForce);
    assert!(!wr.push_str("Smushy").is_err());
    let smushed = wr.get();
    new_smusher!(fnt2, wr_kern, "fonts/small.flf", 60, figdriver::Align::Left, mode figdriver::LayoutMode::Kern);
    assert!(!wr_kern.push_str("Smushy").is_err());
    let kerned = wr_kern.get();
    let smushed_width = smushed[0].chars().count();
    let kerned_width = kerned[0].chars().count();
    assert!(smushed_width < kerned_width);
}

#[test]
fn flc_all_chars_skipped_produces_no_output() {
    // When an FLC control file maps all input characters to codes whose glyphs
    // are missing from the font, the wrapper should produce no output (matching
    // reference figlet behavior). See GitHub issue #36.
    let font = load_font("fonts/standard.flf");
    let pipeline = figdriver::FlcPipeline::from_paths(&[
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts/frango.flc"),
    ]).unwrap();
    let sm = figdriver::Smusher::builder(&font)
        .control(Some(&pipeline))
        .build();
    let mut wr = figdriver::Wrapper::new(sm, 80, figdriver::Align::Left);
    assert!(wr.push_str("Test").is_ok());
    assert!(wr.is_empty());
    let output = wr.get();
    assert!(output.iter().all(|line| line.is_empty()));
}

#[test]
fn paragraph_mode_newline_converts_to_space() {
    // In paragraph mode, newlines between lines are converted to spaces.
    // "Hello\nWorld" should render as "Hello World" with the space between
    // words, producing wider output than "HelloWorld" (no space).
    new_smusher!(fnt, wr, "fonts/small.flf", 80, figdriver::Align::Left);

    // Simulate paragraph mode: process two lines with space insertion
    wr.wrap_str("Hello", &dummy);
    wr.wrap_str(" ", &dummy);
    wr.wrap_str("World", &dummy);
    let with_space = wr.get();

    new_smusher!(fnt2, wr2, "fonts/small.flf", 80, figdriver::Align::Left);
    wr2.wrap_str("HelloWorld", &dummy);
    let without_space = wr2.get();

    assert!(
        with_space[0].chars().count() > without_space[0].chars().count(),
        "newline should insert a space (width {} > {})",
        with_space[0].chars().count(),
        without_space[0].chars().count()
    );
}
