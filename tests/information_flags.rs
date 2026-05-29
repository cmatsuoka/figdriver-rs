use std::process::Command;

fn figlet_cmd() -> Command {
    let binary = env!("CARGO_BIN_EXE_figlet");
    Command::new(binary)
}

#[test]
fn version_flag() {
    for flag in ["-v", "--version"] {
        let mut cmd = figlet_cmd();
        cmd.arg(flag);
        let output = cmd.output().unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), format!("figlet {}", env!("CARGO_PKG_VERSION")));
    }
}
