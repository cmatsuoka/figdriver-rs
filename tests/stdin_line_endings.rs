use std::io::Write;
use std::process::Stdio;

mod common;

/// Run figlet with raw byte input via stdin, returning trimmed non-empty output lines.
fn run_with_raw_input(input: &[u8]) -> Vec<String> {
    let mut cmd = common::cmd_figlet(&[]);
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty())
        .collect()
}

#[test]
fn stdin_lf_line_endings() {
    let actual = run_with_raw_input(b"ab\ncd\n");
    let expected = run_with_raw_input(b"ab\ncd");
    assert_eq!(actual, expected, "LF line endings should work correctly");
}

#[test]
fn stdin_crlf_line_endings() {
    let actual = run_with_raw_input(b"ab\r\ncd\r\n");
    let expected = run_with_raw_input(b"ab\ncd");
    assert_eq!(actual, expected, "CRLF line endings should produce same output as LF");
}

#[test]
fn stdin_mixed_line_endings() {
    let actual = run_with_raw_input(b"ab\r\ncd\nef\r\n");
    let expected = run_with_raw_input(b"ab\ncd\nef");
    assert_eq!(actual, expected, "Mixed LF/CRLF endings should produce same output");
}

#[test]
fn stdin_no_trailing_newline() {
    let actual = run_with_raw_input(b"ab\ncd");
    let expected = run_with_raw_input(b"ab\ncd\n");
    assert_eq!(actual, expected, "Input without trailing newline should produce same output");
}

#[test]
fn stdin_trailing_crlf() {
    let actual = run_with_raw_input(b"ab\r\ncd\r\n");
    let expected = run_with_raw_input(b"ab\ncd\n");
    assert_eq!(actual, expected, "Trailing CRLF should be stripped");
}

#[test]
fn stdin_empty_lines_lf() {
    let actual = run_with_raw_input(b"ab\n\ncd\n");
    let expected = run_with_raw_input(b"ab\n\ncd");
    assert_eq!(actual, expected, "Empty lines with LF should be preserved");
}

#[test]
fn stdin_empty_lines_crlf() {
    let actual = run_with_raw_input(b"ab\r\n\r\ncd\r\n");
    let expected = run_with_raw_input(b"ab\n\ncd");
    assert_eq!(actual, expected, "Empty lines with CRLF should be preserved");
}

#[test]
fn stdin_single_line_no_newline() {
    let actual = run_with_raw_input(b"hello");
    let expected = run_with_raw_input(b"hello\n");
    assert_eq!(actual, expected, "Single line without newline should produce output");
}

#[test]
fn stdin_multiple_crlf_lines() {
    let actual = run_with_raw_input(b"a\r\nb\r\nc\r\n");
    let expected = run_with_raw_input(b"a\nb\nc\n");
    assert_eq!(actual, expected, "Multiple CRLF lines should match LF output");
}
