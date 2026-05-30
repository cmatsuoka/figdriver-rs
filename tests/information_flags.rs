mod common;
use common::cmd_figlet;

#[test]
fn version_flag() {
    for flag in ["-v", "--version"] {
        let mut cmd = cmd_figlet(&[flag]);
        let output = cmd.output().unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), format!("figlet {}", env!("CARGO_PKG_VERSION")));
    }
}
