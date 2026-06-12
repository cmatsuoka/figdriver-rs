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
            figdriver::Smusher::new($font),
            $width,
            $align,
        );
    };
    ( $font:ident, $wr:ident, $path:expr, $width:expr, $align:expr, mode $mode:expr ) => {
        let $font = load_font($path);
        let mut $wr = figdriver::Wrapper::new(
            figdriver::Smusher::builder($font)
                .layout_mode($mode)
                .build(),
            $width,
            $align,
        );
    };
    ( $font:ident, $wr:ident, $path:expr, $width:expr, $align:expr, rtl ) => {
        let $font = load_font($path);
        let mut $wr = figdriver::Wrapper::new(
            figdriver::Smusher::builder($font)
                .right_to_left(true)
                .build(),
            $width,
            $align,
        );
    };
    ( $font:ident, $wr:ident, $path:expr, $width:expr, $align:expr, mode $mode:expr, rtl ) => {
        let $font = load_font($path);
        let mut $wr = figdriver::Wrapper::new(
            figdriver::Smusher::builder($font)
                .layout_mode($mode)
                .right_to_left(true)
                .build(),
            $width,
            $align,
        );
    };
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
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("this is a test", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 2);
    // Trailing space is discarded at wrap point per spec
    assert_eq!(lines[0][0], "this is");
    assert_eq!(lines[1][0], "a test");
}

#[test]
fn wrap_align_left() {
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 12, figdriver::Align::Left);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("this is a new test", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1][0], "new test");
}

#[test]
fn wrap_align_center() {
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 12, figdriver::Align::Center);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("this is a new test", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1][0], "  new test");
}

#[test]
fn wrap_align_right() {
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 12, figdriver::Align::Right);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("this is a new test", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1][0], "    new test");
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
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 8, figdriver::Align::Left);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("this   is a test", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[2][0], "test");
}

#[test]
fn leading_blanks_preserved() {
    // Blanks at the start of input should be rendered as FIGcharacters.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 30, figdriver::Align::Left);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("   x", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0][0], "   x");
}

#[test]
fn rtl_consecutive_blanks_collapsed_at_wrap() {
    // Multiple blanks at a wrap point should be discarded in RTL mode.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 8, figdriver::Align::Left, rtl);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("this   is a test", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[2][0], "tset");
}

#[test]
fn rtl_leading_blanks_preserved() {
    // Blanks at the start of input should be rendered as FIGcharacters in RTL mode.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 30, figdriver::Align::Left, rtl);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("   x", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0][0], "x   ");
}

#[test]
fn rtl_inter_word_blanks_preserved() {
    // Multiple blanks between words should be preserved in RTL mode.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 30, figdriver::Align::Left, rtl);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("a   b", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0][0], "b   a");
}

#[test]
fn rtl_blank_after_wrap_discarded() {
    // Whitespace immediately after a flush should be discarded in RTL mode.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 4, figdriver::Align::Left, rtl);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("this   is a test", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[2][0], "tset");
}

#[test]
fn inter_word_blanks_preserved() {
    // Multiple blanks between words should be preserved.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 30, figdriver::Align::Left);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("a   b", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0][0], "a   b");
}

#[test]
fn blank_after_wrap_discarded() {
    // Whitespace immediately after a flush should be discarded.
    new_smusher!(fnt, wr, "tests/fixtures/test.flf", 4, figdriver::Align::Left);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());
    wr.write_line("this   is a test", &cb);
    let lines = captured.borrow();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[2][0], "test");
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
    let pipeline = figdriver::Control::from_paths(&[
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts/frango.flc"),
    ]).unwrap();
    let sm = figdriver::Smusher::builder(font)
        .control(pipeline)
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
    let cap1 = std::cell::RefCell::new(Vec::new());
    let cb1 = |lines: &[String]| cap1.borrow_mut().push(lines.to_vec());
    wr.write_line("Hello World", &cb1);
    let with_space = cap1.borrow();

    new_smusher!(fnt2, wr2, "fonts/small.flf", 80, figdriver::Align::Left);
    let cap2 = std::cell::RefCell::new(Vec::new());
    let cb2 = |lines: &[String]| cap2.borrow_mut().push(lines.to_vec());
    wr2.write_line("HelloWorld", &cb2);
    let without_space = cap2.borrow();

    assert!(
        with_space[0][0].chars().count() > without_space[0][0].chars().count(),
        "newline should insert a space (width {} > {})",
        with_space[0][0].chars().count(),
        without_space[0][0].chars().count()
    );
}

#[test]
fn crlf_treated_as_single_newline() {
    new_smusher!(fnt, wr, "fonts/small.flf", 80, figdriver::Align::Left);
    let flushed = std::cell::RefCell::new(Vec::new());
    let closure = &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec());

    wr.write_line("Hello\r\nWorld", closure);

    let flushed = flushed.borrow();
    // CRLF produces one intermediate flush (for the newline) + one final flush
    assert_eq!(flushed.len(), 2, "CRLF should produce one intermediate flush plus final flush");
}

#[test]
fn bare_cr_treated_as_newline() {
    new_smusher!(fnt, wr, "fonts/small.flf", 80, figdriver::Align::Left);
    let flushed = std::cell::RefCell::new(Vec::new());
    let closure = &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec());

    wr.write_line("Hello\rWorld", closure);

    let flushed = flushed.borrow();
    // Bare CR produces one intermediate flush (for the newline) + one final flush
    assert_eq!(flushed.len(), 2, "bare CR should produce one intermediate flush plus final flush");
}

#[test]
fn tab_code_treated_as_whitespace() {
    new_smusher!(fnt, wr, "fonts/small.flf", 80, figdriver::Align::Left);
    let flushed = std::cell::RefCell::new(Vec::new());
    let closure = &|lines: &[String]| flushed.borrow_mut().push(lines.to_vec());

    wr.write_line("Hello\t\tWorld", closure);

    let flushed = flushed.borrow();
    // No intermediate flushes (line fits), only final flush
    assert_eq!(flushed.len(), 1, "tab-separated words should only produce final flush");
}

#[test]
fn consecutive_tabs_grouped_as_whitespace() {
    new_smusher!(fnt, wr, "fonts/small.flf", 80, figdriver::Align::Left);
    let captured = std::cell::RefCell::new(Vec::new());
    let cb = |lines: &[String]| captured.borrow_mut().push(lines.to_vec());

    wr.write_line("\t\t\t", &cb);
    let lines = captured.borrow();
    assert!(lines.iter().any(|batch| batch.iter().any(|s| !s.is_empty())),
        "tabs at start should be preserved as leading whitespace");
}
