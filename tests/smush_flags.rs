use std::process::Command;

fn figlet_cmd() -> Command {
    let binary = env!("CARGO_BIN_EXE_figlet");
    Command::new(binary)
}

fn run(figlet_cmd: &mut Command) -> Vec<String> {
    let output = figlet_cmd.output().unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn smush_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("-S").arg("Hi");
    let output = run(&mut cmd);
    assert!(!output.is_empty());
}

#[test]
fn smush_default_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("-s").arg("Hi");
    let output = run(&mut cmd);
    assert!(!output.is_empty());
}

#[test]
fn smush_long_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("--smush").arg("Hi");
    let output = run(&mut cmd);
    assert!(!output.is_empty());
}

#[test]
fn smush_default_long_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("--smush-default").arg("Hi");
    let output = run(&mut cmd);
    assert!(!output.is_empty());
}

#[test]
fn smush_force_produces_smushed_output() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-S").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output, [
        " _  _ _",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn smush_force_narrower_than_full_width() {
    let mut cmd_smush = figlet_cmd();
    cmd_smush.arg("-f").arg("small").arg("-S").arg("Hi");
    let smush_output = run(&mut cmd_smush);

    let mut cmd_full = figlet_cmd();
    cmd_full.arg("-f").arg("small").arg("-W").arg("Hi");
    let full_output = run(&mut cmd_full);

    assert!(smush_output[0].len() < full_output[0].len());
}

#[test]
fn smush_force_on_kerning_font_falls_back_to_overlap() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("banner").arg("-S").arg("Hi");
    let output = run(&mut cmd);
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
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-s").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output, [
        " _  _ _",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn smush_default_skips_kerning_only_font() {
    let mut cmd_s = figlet_cmd();
    cmd_s.arg("-f").arg("banner").arg("-s").arg("Hi");
    let s_output = run(&mut cmd_s);

    let mut cmd_kern = figlet_cmd();
    cmd_kern.arg("-f").arg("banner").arg("-k").arg("Hi");
    let kern_output = run(&mut cmd_kern);

    assert_eq!(s_output, kern_output);
}

#[test]
fn smush_force_overrides_smush() {
    let mut cmd_s = figlet_cmd();
    cmd_s.arg("-f").arg("banner").arg("-s").arg("-S").arg("Hi");
    let s_output = run(&mut cmd_s);

    let mut cmd_upper = figlet_cmd();
    cmd_upper.arg("-f").arg("banner").arg("-S").arg("Hi");
    let upper_output = run(&mut cmd_upper);

    assert_eq!(s_output, upper_output);
}
