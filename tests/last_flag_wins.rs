mod common;
use common::{run_no_trim, run_with_input};

#[test]
fn paragraph_normal_last_flag_wins_p_n() {
    let output = run_with_input(&["-f", "small", "-p", "-n"], "Hello\nWorld");
    assert_eq!(output.len(), 8, "normal mode should produce 2 separate lines (4 lines each)");
}

#[test]
fn paragraph_normal_last_flag_wins_n_p() {
    let output = run_with_input(&["-f", "small", "-n", "-p"], "Hello\nWorld");
    assert_eq!(output.len(), 4, "paragraph mode should merge into 1 line");
}

#[test]
fn flags_dont_leak_into_message() {
    let output = run_no_trim(&["-f", "small", "-l", "Hi"]);
    assert_eq!(output.len(), 4, "output should only render Hi, not the -l flag");
    assert_eq!(output, [
        " _  _ _ ",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn paragraph_flag_doesnt_leak_into_message() {
    let output = run_no_trim(&["-f", "small", "-p", "Hi"]);
    assert_eq!(output.len(), 4, "output should only render Hi");
    assert_eq!(output, [
        " _  _ _ ",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn alignment_flag_doesnt_leak_into_message() {
    let output = run_no_trim(&["-f", "small", "-c", "-w", "40", "Hi"]);
    assert_eq!(output.len(), 4, "output should only render Hi");
    assert!(output[0].contains(" _  _ _ "), "line should contain rendered Hi");
}
