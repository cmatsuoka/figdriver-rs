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

#[test]
fn mode_0_equals_kerning() {
    let mut cmd_m0 = figlet_cmd();
    cmd_m0.arg("-f").arg("small").arg("-m").arg("0").arg("Hi");
    let m0_output = run(&mut cmd_m0);

    let mut cmd_k = figlet_cmd();
    cmd_k.arg("-f").arg("small").arg("-k").arg("Hi");
    let kern_output = run(&mut cmd_k);

    assert_eq!(m0_output, kern_output);
}

#[test]
fn mode_minus_1_equals_full_width() {
    let mut cmd_m1 = figlet_cmd();
    cmd_m1.arg("-f").arg("small").arg("-m").arg("-1").arg("Hi");
    let m1_output = run(&mut cmd_m1);

    let mut cmd_w = figlet_cmd();
    cmd_w.arg("-f").arg("small").arg("-W").arg("Hi");
    let full_output = run(&mut cmd_w);

    assert_eq!(m1_output, full_output);
}

#[test]
fn mode_minus_2_equals_smush_default() {
    let mut cmd_m2 = figlet_cmd();
    cmd_m2.arg("-f").arg("small").arg("-m").arg("-2").arg("Hi");
    let m2_output = run(&mut cmd_m2);

    let mut cmd_s = figlet_cmd();
    cmd_s.arg("-f").arg("small").arg("-s").arg("Hi");
    let s_output = run(&mut cmd_s);

    assert_eq!(m2_output, s_output);
}

#[test]
fn mode_minus_2_skips_kerning_only_font() {
    let mut cmd_m2 = figlet_cmd();
    cmd_m2.arg("-f").arg("banner").arg("-m").arg("-2").arg("Hi");
    let m2_output = run(&mut cmd_m2);

    let mut cmd_k = figlet_cmd();
    cmd_k.arg("-f").arg("banner").arg("-k").arg("Hi");
    let kern_output = run(&mut cmd_k);

    assert_eq!(m2_output, kern_output);
}

 #[test]
fn mode_1_enables_equal_smush() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-m").arg("1").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output, [
        " _  _  _",
        "| || |(_)",
        "| __ || |",
        "|_||_||_|",
    ]);
}

#[test]
fn mode_63_enables_all_smush_rules() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-m").arg("63").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output, [
        " _  _ _",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn mode_63_narrower_than_full_width() {
    let mut cmd_m63 = figlet_cmd();
    cmd_m63.arg("-f").arg("small").arg("-m").arg("63").arg("Hi");
    let m63_output = run(&mut cmd_m63);

    let mut cmd_full = figlet_cmd();
    cmd_full.arg("-f").arg("small").arg("-W").arg("Hi");
    let full_output = run(&mut cmd_full);

    assert!(m63_output[0].len() < full_output[0].len());
}

#[test]
fn mode_long_flag_works() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("--layout-mode").arg("7").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output, [
        " _  _ _",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn mode_invalid_value_fails() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-m").arg("-256").arg("Hi");
    let output = cmd.output().unwrap();
    assert!(!String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn mode_overrides_smush_force() {
    let mut cmd_mode = figlet_cmd();
    cmd_mode.arg("-f").arg("small").arg("-S").arg("-m").arg("0").arg("Hi");
    let mode_output = run(&mut cmd_mode);

    let mut cmd_kern = figlet_cmd();
    cmd_kern.arg("-f").arg("small").arg("-k").arg("Hi");
    let kern_output = run(&mut cmd_kern);

    assert_eq!(mode_output, kern_output);
}
