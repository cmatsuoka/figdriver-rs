use std::process::Command;

fn figlet_cmd() -> Command {
    let binary = env!("CARGO_BIN_EXE_figlet");
    let mut cmd = Command::new(binary);
    cmd.arg("-d").arg("fonts").arg("-f").arg("small");
    cmd
}

fn run(cmd: &mut Command) -> Vec<String> {
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.to_string())
        .filter(|line| !line.trim().is_empty())
        .collect()
}

fn run_with_input(cmd: &mut Command, input: &str) -> Vec<String> {
    let mut child = cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.to_string())
        .filter(|line| !line.trim().is_empty())
        .collect()
}

#[test]
fn paragraph_normal_last_flag_wins_p_n() {
    let mut cmd = figlet_cmd();
    cmd.arg("-p").arg("-n");
    let output = run_with_input(&mut cmd, "Hello\nWorld");
    assert_eq!(output.len(), 8, "normal mode should produce 2 separate lines (4 lines each)");
}

#[test]
fn paragraph_normal_last_flag_wins_n_p() {
    let mut cmd = figlet_cmd();
    cmd.arg("-n").arg("-p");
    let output = run_with_input(&mut cmd, "Hello\nWorld");
    assert_eq!(output.len(), 4, "paragraph mode should merge into 1 line");
}

#[test]
fn flags_dont_leak_into_message() {
    let mut cmd = figlet_cmd();
    cmd.arg("-l").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output.len(), 4, "output should only render Hi, not the -l flag");
    assert_eq!(output, [
        " _  _ _ ",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn paragraph_flag_doesnt_leak_into_message() {
    let mut cmd = figlet_cmd();
    cmd.arg("-p").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output.len(), 4, "output should only render Hi");
    assert_eq!(output, [
        " _  _ _ ",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn alignment_flag_doesnt_leak_into_message() {
    let mut cmd = figlet_cmd();
    cmd.arg("-c").arg("-w").arg("40").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output.len(), 4, "output should only render Hi");
    assert!(output[0].contains(" _  _ _ "), "line should contain rendered Hi");
}
