mod common;
use common::cmd_figlet;

#[test]
fn version_flag() {
    for flag in ["-v", "--version"] {
        let mut cmd = cmd_figlet(&[flag]);
        let output = cmd.output().unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), format!("figlet {} (FIGdriver-rs)", env!("CARGO_PKG_VERSION")));
    }
}

#[test]
fn infocode_zero_shows_copyright_and_version() {
    let mut cmd = cmd_figlet(&["-I", "0"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines[0].starts_with("FIGdriver-rs Copyright "));
    assert_eq!(lines[1], format!("Version: {}", env!("CARGO_PKG_VERSION")));
}


#[test]
fn infocode_one_version_integer() {
    let mut cmd = cmd_figlet(&["-I", "1"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version: u32 = stdout.parse().unwrap();
    assert_eq!(version, 501);
}

#[test]
fn infocode_two_font_directory() {
    let mut cmd = cmd_figlet(&["-I", "2"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "fonts");
}

#[test]
fn infocode_two_respects_dir_option() {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_figlet"));
    cmd.args(["-d", "/custom/dir", "-I", "2"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "/custom/dir");
}

#[test]
fn infocode_three_font_name() {
    let mut cmd = cmd_figlet(&["-I", "3"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "standard");
}

#[test]
fn infocode_three_font_name_with_suffix() {
    let mut cmd = cmd_figlet(&["-f", "doffee.flf", "-I", "3"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "doffee");
}

#[test]
fn infocode_three_font_name_full_path() {
    let mut cmd = cmd_figlet(&["-f", "/usr/share/figlet/doffee.flf", "-I", "3"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "/usr/share/figlet/doffee");
}

#[test]
fn infocode_four_output_width() {
    let mut cmd = cmd_figlet(&["-I", "4"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "80");
}

#[test]
fn infocode_four_respects_width_option() {
    let mut cmd = cmd_figlet(&["-w", "120", "-I", "4"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "120");
}

#[test]
fn infocode_five_supported_formats() {
    let mut cmd = cmd_figlet(&["-I", "5"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "flf2 tlf2");
}

#[test]
fn infocode_invalid_positive_exits_silently() {
    let mut cmd = cmd_figlet(&["-I", "6"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty());
}
