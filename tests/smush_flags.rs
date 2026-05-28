use std::process::Command;

fn figlet_cmd() -> Command {
    let binary = env!("CARGO_BIN_EXE_figlet");
    Command::new(binary)
}

#[test]
fn smush_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("-S").arg("Hi");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

#[test]
fn smush_default_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("-s").arg("Hi");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

#[test]
fn smush_long_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("--smush").arg("Hi");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

#[test]
fn smush_default_long_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("--smush-default").arg("Hi");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

#[test]
fn smush_force_produces_smushed_output() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-S").arg("Hi");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap();
    assert!(first_line.chars().count() < 30);
}

#[test]
fn smush_force_narrower_than_full_width() {
    let mut cmd_smush = figlet_cmd();
    cmd_smush.arg("-f").arg("small").arg("-S").arg("Hi");
    let output_smush = cmd_smush.output().unwrap();
    let stdout_smush = String::from_utf8_lossy(&output_smush.stdout);
    let smush_width = stdout_smush.lines().next().unwrap().chars().count();

    let mut cmd_full = figlet_cmd();
    cmd_full.arg("-f").arg("small").arg("-W").arg("Hi");
    let output_full = cmd_full.output().unwrap();
    let stdout_full = String::from_utf8_lossy(&output_full.stdout);
    let full_width = stdout_full.lines().next().unwrap().chars().count();

    assert!(smush_width < full_width);
}

#[test]
fn smush_force_on_kerning_font_falls_back_to_overlap() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("banner").arg("-S").arg("Hi");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
    let first_line = stdout.lines().next().unwrap();
    assert!(first_line.chars().count() < 30);
}

#[test]
fn smush_respects_font_defaults() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-s").arg("Hi");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

#[test]
fn smush_default_skips_kerning_only_font() {
    let mut cmd_s = figlet_cmd();
    cmd_s.arg("-f").arg("banner").arg("-s").arg("Hi");
    let output_s = cmd_s.output().unwrap();
    let stdout_s = String::from_utf8_lossy(&output_s.stdout);
    let s_width = stdout_s.lines().next().unwrap().chars().count();

    let mut cmd_kern = figlet_cmd();
    cmd_kern.arg("-f").arg("banner").arg("-k").arg("Hi");
    let output_kern = cmd_kern.output().unwrap();
    let stdout_kern = String::from_utf8_lossy(&output_kern.stdout);
    let kern_width = stdout_kern.lines().next().unwrap().chars().count();

    assert_eq!(s_width, kern_width);
}

#[test]
fn smush_force_overrides_smush() {
    let mut cmd_s = figlet_cmd();
    cmd_s.arg("-f").arg("banner").arg("-s").arg("-S").arg("Hi");
    let output_s = cmd_s.output().unwrap();
    let stdout_s = String::from_utf8_lossy(&output_s.stdout);
    let s_width = stdout_s.lines().next().unwrap().chars().count();

    let mut cmd_upper = figlet_cmd();
    cmd_upper.arg("-f").arg("banner").arg("-S").arg("Hi");
    let output_upper = cmd_upper.output().unwrap();
    let stdout_upper = String::from_utf8_lossy(&output_upper.stdout);
    let upper_width = stdout_upper.lines().next().unwrap().chars().count();

    assert_eq!(s_width, upper_width);
}
