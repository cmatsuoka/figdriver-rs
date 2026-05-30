mod common;
use common::{cmd_figlet, run};

#[test]
fn smush_flag_parses() {
    let output = run(&["-S", "Hi"]);
    assert!(!output.is_empty());
}

#[test]
fn smush_default_flag_parses() {
    let output = run(&["-s", "Hi"]);
    assert!(!output.is_empty());
}

#[test]
fn smush_long_flag_parses() {
    let output = run(&["--smush", "Hi"]);
    assert!(!output.is_empty());
}

#[test]
fn smush_default_long_flag_parses() {
    let output = run(&["--smush-default", "Hi"]);
    assert!(!output.is_empty());
}

#[test]
fn smush_force_produces_smushed_output() {
    let output = run(&["-f", "small", "-S", "Hi"]);
    assert_eq!(output, [
        " _  _ _",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn smush_force_on_kerning_font_falls_back_to_overlap() {
    let output = run(&["-f", "banner", "-S", "Hi"]);
    assert_eq!(output, [
        "#     #",
        "#     ##",
        "#     ##",
        "########",
        "#     ##",
        "#     ##",
        "#     ##",
    ]);
}

#[test]
fn smush_respects_font_defaults() {
    let output = run(&["-f", "small", "-s", "Hi"]);
    assert_eq!(output, [
        " _  _ _",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn smush_default_skips_kerning_only_font() {
    let s_output = run(&["-f", "banner", "-s", "Hi"]);
    let kern_output = run(&["-f", "banner", "-k", "Hi"]);
    assert_eq!(s_output, kern_output);
}

#[test]
fn smush_force_overrides_smush() {
    let s_output = run(&["-f", "banner", "-s", "-S", "Hi"]);
    let upper_output = run(&["-f", "banner", "-S", "Hi"]);
    assert_eq!(s_output, upper_output);
}

#[test]
fn mode_0_equals_kerning() {
    let m0_output = run(&["-f", "small", "-m", "0", "Hi"]);
    let kern_output = run(&["-f", "small", "-k", "Hi"]);
    assert_eq!(m0_output, kern_output);
}

#[test]
fn mode_minus_1_equals_full_width() {
    let m1_output = run(&["-f", "small", "-m", "-1", "Hi"]);
    let full_output = run(&["-f", "small", "-W", "Hi"]);
    assert_eq!(m1_output, full_output);
}

#[test]
fn mode_minus_2_equals_smush_default() {
    let m2_output = run(&["-f", "small", "-m", "-2", "Hi"]);
    let s_output = run(&["-f", "small", "-s", "Hi"]);
    assert_eq!(m2_output, s_output);
}

#[test]
fn mode_minus_2_skips_kerning_only_font() {
    let m2_output = run(&["-f", "banner", "-m", "-2", "Hi"]);
    let kern_output = run(&["-f", "banner", "-k", "Hi"]);
    assert_eq!(m2_output, kern_output);
}

 #[test]
fn mode_1_enables_equal_smush() {
    let output = run(&["-f", "small", "-m", "1", "Hi"]);
    assert_eq!(output, [
        " _  _  _",
        "| || |(_)",
        "| __ || |",
        "|_||_||_|",
    ]);
}

#[test]
fn mode_63_enables_all_smush_rules() {
    let output = run(&["-f", "small", "-m", "63", "Hi"]);
    assert_eq!(output, [
        " _  _ _",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn mode_long_flag_works() {
    let output = run(&["-f", "small", "--layout-mode", "7", "Hi"]);
    assert_eq!(output, [
        " _  _ _",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn mode_invalid_value_fails() {
    let mut cmd = cmd_figlet(&["-f", "small", "-m", "-256", "Hi"]);
    let output = cmd.output().unwrap();
    assert!(!String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn mode_overrides_smush_force() {
    let mode_output = run(&["-f", "small", "-S", "-m", "0", "Hi"]);
    let kern_output = run(&["-f", "small", "-k", "Hi"]);
    assert_eq!(mode_output, kern_output);
}
