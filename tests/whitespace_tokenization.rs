mod common;

/// Hardcoded expected outputs from our figlet implementation.
/// Captured locally to avoid dependency on reference figlet in CI.

const MULTI_SPACE: &[&str] = &[
    "           _     ",
    "  __ _    | |__  ",
    " / _` |   | '_ \\ ",
    "| (_| |   | |_) |",
    " \\__,_|   |_.__/ ",
];

const LEADING_SPACE: &[&str] = &[
    "    _          _ _       ",
    "   | |__   ___| | | ___  ",
    "   | '_ \\ / _ \\ | |/ _ \\ ",
    "   | | | |  __/ | | (_) |",
    "   |_| |_|\\___|_|_|\\___/ ",
];

const TRAILING_SPACE: &[&str] = &[
    " _          _ _          ",
    "| |__   ___| | | ___     ",
    "| '_ \\ / _ \\ | |/ _ \\    ",
    "| | | |  __/ | | (_) |   ",
    "|_| |_|\\___|_|_|\\___/    ",
];

const TAB: &[&str] = &[
    "         _     ",
    "  __ _  | |__  ",
    " / _` | | '_ \\ ",
    "| (_| | | |_) |",
    " \\__,_| |_.__/ ",
];

const MIXED_WS: &[&str] = &[
    "           _     ",
    "  __ _    | |__  ",
    " / _` |   | '_ \\ ",
    "| (_| |   | |_) |",
    " \\__,_|   |_.__/ ",
];

fn multi_space() -> Vec<String> {
    MULTI_SPACE.iter().map(|s| s.to_string()).collect()
}

fn leading_space() -> Vec<String> {
    LEADING_SPACE.iter().map(|s| s.to_string()).collect()
}

fn trailing_space() -> Vec<String> {
    TRAILING_SPACE.iter().map(|s| s.to_string()).collect()
}

fn tab() -> Vec<String> {
    TAB.iter().map(|s| s.to_string()).collect()
}

fn mixed_ws() -> Vec<String> {
    MIXED_WS.iter().map(|s| s.to_string()).collect()
}

#[test]
fn multiple_spaces_argv() {
    let actual = common::run_no_trim(&["a   b"]);
    assert_eq!(actual, multi_space(), "multiple spaces between words (argv)");
}

#[test]
fn multiple_spaces_stdin() {
    let actual = common::run_with_input_no_trim(&[], "a   b\n");
    assert_eq!(actual, multi_space(), "multiple spaces between words (stdin)");
}

#[test]
fn leading_spaces_argv() {
    let actual = common::run_no_trim(&["   hello"]);
    assert_eq!(actual, leading_space(), "leading spaces (argv)");
}

#[test]
fn trailing_spaces_argv() {
    let actual = common::run_no_trim(&["hello   "]);
    assert_eq!(actual, trailing_space(), "trailing spaces (argv)");
}

#[test]
fn tab_characters_argv() {
    let actual = common::run_no_trim(&["a\tb"]);
    assert_eq!(actual, tab(), "tab characters (argv)");
}

#[test]
fn tab_characters_stdin() {
    let actual = common::run_with_input_no_trim(&[], "a\tb\n");
    assert_eq!(actual, tab(), "tab characters (stdin)");
}

#[test]
fn mixed_whitespace_argv() {
    let actual = common::run_no_trim(&["a \t b"]);
    assert_eq!(actual, mixed_ws(), "mixed whitespace (argv)");
}

#[test]
fn mixed_whitespace_stdin() {
    let actual = common::run_with_input_no_trim(&[], "a \t b\n");
    assert_eq!(actual, mixed_ws(), "mixed whitespace (stdin)");
}
