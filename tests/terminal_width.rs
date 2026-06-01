mod common;
use common::{cmd_figlet, run_no_trim};

#[test]
fn t_flag_uses_terminal_width_fallback_when_no_tty() {
    // When not connected to a TTY, terminal_size returns None, falling back to default width
    let mut cmd = cmd_figlet(&["-t", "-I", "4"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "80");
}

#[test]
fn t_flag_w_takes_precedence_over_t() {
    // -w should always win over -t
    let mut cmd = cmd_figlet(&["-w", "120", "-t", "-I", "4"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "120");
}

#[test]
fn t_flag_reversed_order_w_takes_precedence() {
    // -t -w: -w should still win regardless of argument order
    let mut cmd = cmd_figlet(&["-t", "-w", "120", "-I", "4"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "120");
}

#[test]
fn t_flag_doesnt_leak_with_w_option() {
    // -t should be consumed from args even when -w is also provided
    let output = run_no_trim(&["-f", "small", "-w", "40", "-t", "Hi"]);
    assert_eq!(output.len(), 4, "output should only render Hi, not -t");
    assert_eq!(output, [
        " _  _ _ ",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn t_flag_doesnt_leak_reversed_order() {
    // Reversed order: -t before -w should also not leak
    let output = run_no_trim(&["-f", "small", "-t", "-w", "40", "Hi"]);
    assert_eq!(output.len(), 4, "output should only render Hi");
    assert_eq!(output, [
        " _  _ _ ",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn t_flag_alone_doesnt_leak_into_message() {
    // -t without -w should also be consumed properly
    let output = run_no_trim(&["-f", "small", "-t", "Hi"]);
    assert_eq!(output.len(), 4, "output should only render Hi");
    assert_eq!(output, [
        " _  _ _ ",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}
