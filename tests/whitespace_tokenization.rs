use std::io::Write;
use std::process::Command;

mod common;

fn get_expected(input: &str) -> Vec<String> {
    let output = Command::new("figlet")
        .arg(input)
        .output()
        .expect("failed to execute reference figlet");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.to_string())
        .filter(|line| !line.trim().is_empty())
        .collect()
}

fn get_expected_stdin(input: &str) -> Vec<String> {
    let mut child = Command::new("figlet")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to execute reference figlet");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.to_string())
        .filter(|line| !line.trim().is_empty())
        .collect()
}

#[test]
fn multiple_spaces_argv() {
    let input = "a   b";
    let expected = get_expected(input);
    let actual = common::run_no_trim(&[input]);
    assert_eq!(actual, expected, "multiple spaces between words (argv)");
}

#[test]
fn multiple_spaces_stdin() {
    let input = "a   b\n";
    let expected = get_expected_stdin(input);
    let actual = common::run_with_input_no_trim(&[], input);
    assert_eq!(actual, expected, "multiple spaces between words (stdin)");
}

#[test]
fn leading_spaces_argv() {
    let input = "   hello";
    let expected = get_expected(input);
    let actual = common::run_no_trim(&[input]);
    assert_eq!(actual, expected, "leading spaces (argv)");
}

#[test]
fn trailing_spaces_argv() {
    let input = "hello   ";
    let expected = get_expected(input);
    let actual = common::run_no_trim(&[input]);
    assert_eq!(actual, expected, "trailing spaces (argv)");
}

#[test]
fn tab_characters_argv() {
    let input = "a\tb";
    let expected = get_expected(input);
    let actual = common::run_no_trim(&[input]);
    assert_eq!(actual, expected, "tab characters (argv)");
}

#[test]
fn tab_characters_stdin() {
    let input = "a\tb\n";
    let expected = get_expected_stdin(input);
    let actual = common::run_with_input_no_trim(&[], input);
    assert_eq!(actual, expected, "tab characters (stdin)");
}

#[test]
fn mixed_whitespace_argv() {
    let input = "a \t b";
    let expected = get_expected(input);
    let actual = common::run_no_trim(&[input]);
    assert_eq!(actual, expected, "mixed whitespace (argv)");
}

#[test]
fn mixed_whitespace_stdin() {
    let input = "a \t b\n";
    let expected = get_expected_stdin(input);
    let actual = common::run_with_input_no_trim(&[], input);
    assert_eq!(actual, expected, "mixed whitespace (stdin)");
}
