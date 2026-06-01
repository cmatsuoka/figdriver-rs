mod common;

#[test]
fn invalid_short_flag_rejected() {
    let mut cmd = common::cmd_figlet(&["-f", "small", "-z"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid flag"));
}

#[test]
fn invalid_long_flag_rejected() {
    let mut cmd = common::cmd_figlet(&["-f", "small", "--trololo"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid flag"));
}

#[test]
fn combined_flag_with_invalid_char_rejected() {
    let mut cmd = common::cmd_figlet(&["-f", "small", "-Rz"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid flag"));
}

#[test]
fn double_dash_allows_dash_message() {
    let output = common::run(&["-f", "small", "--", "-Hello"]);
    assert!(!output.is_empty());
}

#[test]
fn invalid_flag_before_double_dash_rejected() {
    let mut cmd = common::cmd_figlet(&["-z", "--", "Hello"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid flag"));
}

#[test]
fn double_dash_does_not_leak_into_message() {
    let output = common::run_no_trim(&["-f", "small", "--", "Hello"]);
    assert_eq!(output, [
        r#" _  _     _ _     "#,
        r#"| || |___| | |___ "#,
        r#"| __ / -_) | / _ \"#,
        r#"|_||_\___|_|_\___/"#,
    ]);
}
