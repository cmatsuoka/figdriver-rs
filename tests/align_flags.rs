use std::process::Command;

fn figlet_cmd() -> Command {
    let binary = env!("CARGO_BIN_EXE_figlet");
    let mut cmd = Command::new(binary);
    cmd.arg("-d").arg("fonts");
    cmd
}

fn run(cmd: &mut Command) -> Vec<String> {
    let output = cmd.output().unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn align_left_short_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("-l").arg("Hi");
    let output = run(&mut cmd);
    assert!(!output.is_empty());
}

#[test]
fn align_left_long_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("--align-left").arg("Hi");
    let output = run(&mut cmd);
    assert!(!output.is_empty());
}

#[test]
fn align_right_short_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("-r").arg("Hi");
    let output = run(&mut cmd);
    assert!(!output.is_empty());
}

#[test]
fn align_right_long_flag_parses() {
    let mut cmd = figlet_cmd();
    cmd.arg("--align-right").arg("Hi");
    let output = run(&mut cmd);
    assert!(!output.is_empty());
}

#[test]
fn align_left_has_no_padding() {
    let mut cmd_left = figlet_cmd();
    cmd_left.arg("-f").arg("small").arg("-l").arg("-w").arg("30").arg("Hi");
    let left_output = run(&mut cmd_left);

    let mut cmd_right = figlet_cmd();
    cmd_right.arg("-f").arg("small").arg("-r").arg("-w").arg("30").arg("Hi");
    let right_output = run(&mut cmd_right);

    for (left, right) in left_output.iter().zip(right_output.iter()) {
        assert!(left.len() < right.len(), "Left-aligned output should be shorter than right-aligned due to no padding");
    }
}

#[test]
fn align_right_produces_right_aligned_output() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-r").arg("-w").arg("30").arg("Hi");
    let output = run(&mut cmd);
    for line in &output {
        assert!(!line.ends_with(' '), "Right-aligned output should not have trailing spaces");
    }
}

#[test]
fn align_left_matches_default_output() {
    let mut cmd_default = figlet_cmd();
    cmd_default.arg("-f").arg("small").arg("-w").arg("30").arg("Hi");
    let default_output = run(&mut cmd_default);

    let mut cmd_left = figlet_cmd();
    cmd_left.arg("-f").arg("small").arg("-l").arg("-w").arg("30").arg("Hi");
    let left_output = run(&mut cmd_left);

    assert_eq!(default_output, left_output);
}

#[test]
fn align_right_differs_from_left() {
    let mut cmd_left = figlet_cmd();
    cmd_left.arg("-f").arg("small").arg("-l").arg("-w").arg("30").arg("Hi");
    let left_output = run(&mut cmd_left);

    let mut cmd_right = figlet_cmd();
    cmd_right.arg("-f").arg("small").arg("-r").arg("-w").arg("30").arg("Hi");
    let right_output = run(&mut cmd_right);

    assert_ne!(left_output, right_output);
}
