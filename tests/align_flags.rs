mod common;
use common::run_no_trim;

#[test]
fn align_left() {
    for flag in ["-l", "--left"] {
        let output = run_no_trim(&["-f", "small", flag, "Hi"]);
        assert_eq!(output, [
            " _  _ _ ",
            "| || (_)",
            "| __ | |",
            "|_||_|_|",
        ]);
    }
}

#[test]
fn align_right() {
    for flag in ["-r", "--right"] {
        let output = run_no_trim(&["-f", "small", flag, "-w", "40", "Hi"]);
        assert_eq!(output, [
            "                                _  _ _ ",
            "                               | || (_)",
            "                               | __ | |",
            "                               |_||_|_|",
        ]);
    }
}

#[test]
fn align_center() {
    for flag in ["-c", "--center"] {
        let output = run_no_trim(&["-f", "small", flag, "-w", "40", "Hi"]);
        assert_eq!(output, [
            "                 _  _ _ ",
            "                | || (_)",
            "                | __ | |",
            "                |_||_|_|",
        ]);
    }
}

#[test]
fn align_left_matches_default() {
    let default_output = run_no_trim(&["-f", "small", "-w", "40", "Hi"]);
    let left_output = run_no_trim(&["-f", "small", "-l", "-w", "40", "Hi"]);
    assert_eq!(default_output, left_output);
}

#[test]
fn align_left_wins_over_center_when_last() {
    let output = run_no_trim(&["-f", "small", "-c", "-l", "-w", "40", "Hi"]);
    assert_eq!(output, [
        " _  _ _ ",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn align_center_wins_over_left_when_last() {
    let output = run_no_trim(&["-f", "small", "-l", "-c", "-w", "40", "Hi"]);
    assert_eq!(output, [
        "                 _  _ _ ",
        "                | || (_)",
        "                | __ | |",
        "                |_||_|_|",
    ]);
}

#[test]
fn align_right_wins_over_left_when_last() {
    let output = run_no_trim(&["-f", "small", "-l", "-r", "-w", "40", "Hi"]);
    assert_eq!(output, [
        "                                _  _ _ ",
        "                               | || (_)",
        "                               | __ | |",
        "                               |_||_|_|",
    ]);
}

#[test]
fn align_right_to_left_reverses_text() {
    // -R enables right-to-left rendering, which reverses character order
    let rtl_output = run_no_trim(&["-f", "small", "-R", "-w", "40", "Hi"]);
    assert_eq!(rtl_output, [
        "                                _ _  _ ",
        "                               (_) || |",
        "                               | | __ |",
        "                               |_|_||_|",
    ]);
}

#[test]
fn align_right_has_padding() {
    let left_output = run_no_trim(&["-f", "small", "-l", "-w", "40", "Hi"]);
    let right_output = run_no_trim(&["-f", "small", "-r", "-w", "40", "Hi"]);

    assert_eq!(
        left_output.len(),
        right_output.len(),
        "Left and right outputs must have the same number of lines"
    );

    for (left, right) in left_output.iter().zip(right_output.iter()) {
        assert!(
            left.len() < right.len(),
            "Right-aligned line ({:?}) should be wider than left-aligned ({:?})",
            right,
            left
        );
    }
}
