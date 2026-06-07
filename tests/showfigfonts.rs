mod common;

#[test]
fn version_flag_short() {
    let mut cmd = common::cmd_showfigfonts(&["-v"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), format!("showfigfonts {} (FIGdriver-rs)", env!("CARGO_PKG_VERSION")));
}

#[test]
fn version_flag_long() {
    let mut cmd = common::cmd_showfigfonts(&["--version"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), format!("showfigfonts {} (FIGdriver-rs)", env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_flag_short() {
    let mut cmd = common::cmd_showfigfonts(&["-h"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: showfigfonts"));
}

#[test]
fn help_flag_long() {
    let mut cmd = common::cmd_showfigfonts(&["--help"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: showfigfonts"));
}

#[test]
fn renders_all_fonts() {
    let mut cmd = common::cmd_showfigfonts(&[]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("banner :"));
    assert!(stdout.contains("standard :"));
    assert!(stdout.contains("small :"));
}

#[test]
fn renders_fonts_sorted() {
    let mut cmd = common::cmd_showfigfonts(&[]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let headings: Vec<&str> = stdout.lines().filter(|l| l.ends_with(" :")).collect();
    for window in headings.windows(2) {
        let name1 = window[0].trim_end_matches(" :");
        let name2 = window[1].trim_end_matches(" :");
        assert!(
            name1 <= name2,
            "Fonts should be sorted: {} should come before {}",
            name1,
            name2
        );
    }
}

#[test]
fn uses_font_name_as_word() {
    let mut cmd = common::cmd_showfigfonts(&["-d", "fonts"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    let mut found_banner = false;
    let mut banner_lines = Vec::new();
    for line in &lines {
        if line.contains("banner :") {
            found_banner = true;
            continue;
        }
        if found_banner {
            if line.contains(" :") && !line.contains("banner :") {
                break;
            }
            banner_lines.push(*line);
        }
    }
    assert!(found_banner, "banner heading not found");
    let non_empty: Vec<&str> = banner_lines.iter().filter(|l| !l.trim().is_empty()).map(|l| *l).collect();
    assert!(
        !non_empty.is_empty(),
        "banner font produced no output"
    );
}

#[test]
fn accepts_word_argument() {
    let mut cmd = common::cmd_showfigfonts(&["Hi"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("banner :"));
}

#[test]
fn respects_fontdir_option() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("small.flf"), include_str!("../fonts/small.flf")).unwrap();
    let mut cmd = common::cmd_showfigfonts(&[]);
    cmd.arg("-d").arg(tmp.path());
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("small :"));
    assert!(!stdout.contains("standard :"));
}

#[test]
fn respects_figlet_fontdir_env() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("banner.flf"), include_str!("../fonts/banner.flf")).unwrap();
    let mut cmd = common::cmd_showfigfonts(&[]);
    cmd.env("FIGLET_FONTDIR", tmp.path());
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("banner :"));
}

#[test]
fn handles_missing_directory() {
    let mut cmd = common::cmd_showfigfonts(&[]);
    cmd.arg("-d").arg("/nonexistent/path");
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unable to open directory"));
}

#[test]
fn invalid_flag_rejected() {
    let mut cmd = common::cmd_showfigfonts(&["-z"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn too_many_word_arguments_rejected() {
    let mut cmd = common::cmd_showfigfonts(&["word1", "word2"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
}
