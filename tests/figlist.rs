mod common;

#[test]
fn version_flag_short() {
    let mut cmd = common::cmd_figlist(&["-v"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), format!("figlist {} (FIGdriver-rs)", env!("CARGO_PKG_VERSION")));
}

#[test]
fn version_flag_long() {
    let mut cmd = common::cmd_figlist(&["--version"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), format!("figlist {} (FIGdriver-rs)", env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_flag_short() {
    let mut cmd = common::cmd_figlist(&["-h"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: figlist"));
}

#[test]
fn help_flag_long() {
    let mut cmd = common::cmd_figlist(&["--help"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: figlist"));
}

#[test]
fn lists_fonts_and_control_files() {
    let mut cmd = common::cmd_figlist(&["-d", "fonts"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Default font: standard"));
    assert!(stdout.contains("Font directory: fonts"));
    assert!(stdout.contains("Figlet fonts in this directory:"));
    assert!(stdout.contains("standard"));
    assert!(stdout.contains("Figlet control files in this directory:"));
    assert!(stdout.contains("utf8"));
}

#[test]
fn lists_fonts_sorted() {
    let mut cmd = common::cmd_figlist(&["-d", "fonts"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    let mut in_fonts = false;
    let mut prev = String::new();
    for line in lines {
        if line.contains("Figlet fonts in this directory:") {
            in_fonts = true;
            continue;
        }
        if line.contains("Figlet control files") {
            break;
        }
        if in_fonts && !line.trim().is_empty() {
            assert!(
                line.trim() >= prev.as_str(),
                "Fonts should be sorted: {} should come before {}",
                line.trim(),
                prev
            );
            prev = line.trim().to_string();
        }
    }
}

#[test]
fn respects_fontdir_option() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("small.flf"), include_str!("../fonts/small.flf")).unwrap();
    let mut cmd = common::cmd_figlist(&[]);
    cmd.arg("-d").arg(tmp.path());
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("small"));
    assert!(!stdout.contains("standard"));
}

#[test]
fn respects_figlet_fontdir_env() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("banner.flf"), include_str!("../fonts/banner.flf")).unwrap();
    let mut cmd = common::cmd_figlist(&[]);
    cmd.env("FIGLET_FONTDIR", tmp.path());
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("banner"));
}

#[test]
fn handles_missing_directory() {
    let mut cmd = common::cmd_figlist(&["-d", "/nonexistent/path"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unable to open directory"));
}

#[test]
fn invalid_flag_rejected() {
    let mut cmd = common::cmd_figlist(&["-z"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn unknown_argument_rejected() {
    let mut cmd = common::cmd_figlist(&["garbage"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
}
