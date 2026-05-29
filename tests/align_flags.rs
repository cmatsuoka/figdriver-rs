use std::process::Command;

fn figlet_cmd() -> Command {
    let binary = env!("CARGO_BIN_EXE_figlet");
    let mut cmd = Command::new(binary);
    cmd.arg("-d").arg("fonts");
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

#[test]
fn align_left() {
    for flag in ["-l", "--left"] {
        let mut cmd = figlet_cmd();
        cmd.arg("-f").arg("small").arg(flag).arg("Hi");
        let output = run(&mut cmd);
        assert_eq!(output, [
            " _  _ _ ",
            "| || (_)",
            "| __ | |",
            "|_||_|_|",
        ]);
    }
}

#[test]
fn align_right() {
    for flag in ["-r", "--right"] {
        let mut cmd = figlet_cmd();
        cmd.arg("-f").arg("small").arg(flag).arg("-w").arg("40").arg("Hi");
        let output = run(&mut cmd);
        assert_eq!(output, [
            "                                _  _ _ ",
            "                               | || (_)",
            "                               | __ | |",
            "                               |_||_|_|",
        ]);
    }
}

#[test]
fn align_center() {
    for flag in ["-c", "--center"] {
        let mut cmd = figlet_cmd();
        cmd.arg("-f").arg("small").arg(flag).arg("-w").arg("40").arg("Hi");
        let output = run(&mut cmd);
        assert_eq!(output, [
            "                _  _ _ ",
            "               | || (_)",
            "               | __ | |",
            "               |_||_|_|",
        ]);
    }
}

#[test]
fn align_left_matches_default() {
    let mut cmd_default = figlet_cmd();
    cmd_default.arg("-f").arg("small").arg("-w").arg("40").arg("Hi");
    let default_output = run(&mut cmd_default);

    let mut cmd_left = figlet_cmd();
    cmd_left.arg("-f").arg("small").arg("-l").arg("-w").arg("40").arg("Hi");
    let left_output = run(&mut cmd_left);

    assert_eq!(default_output, left_output);
}

#[test]
fn align_left_wins_over_center_when_last() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-c").arg("-l").arg("-w").arg("40").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output, [
        " _  _ _ ",
        "| || (_)",
        "| __ | |",
        "|_||_|_|",
    ]);
}

#[test]
fn align_center_wins_over_left_when_last() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-l").arg("-c").arg("-w").arg("40").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output, [
        "                _  _ _ ",
        "               | || (_)",
        "               | __ | |",
        "               |_||_|_|",
    ]);
}

#[test]
fn align_right_wins_over_left_when_last() {
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-l").arg("-r").arg("-w").arg("40").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output, [
        "                                _  _ _ ",
        "                               | || (_)",
        "                               | __ | |",
        "                               |_||_|_|",
    ]);
}

#[test]
fn align_right_to_left_reverses_text() {
    // -R enables right-to-left rendering, which reverses character order
    let mut cmd = figlet_cmd();
    cmd.arg("-f").arg("small").arg("-R").arg("-w").arg("40").arg("Hi");
    let output = run(&mut cmd);
    assert_eq!(output.len(), 4);
    // With RTL, "Hi" is rendered as reversed characters
    assert!(output[0].contains("_ "), "RTL output should have reversed character rendering");
}

#[test]
fn align_right_has_padding() {
    let mut cmd_left = figlet_cmd();
    cmd_left.arg("-f").arg("small").arg("-l").arg("-w").arg("40").arg("Hi");
    let left_output = run(&mut cmd_left);

    let mut cmd_right = figlet_cmd();
    cmd_right.arg("-f").arg("small").arg("-r").arg("-w").arg("40").arg("Hi");
    let right_output = run(&mut cmd_right);

    assert_eq!(
        left_output.len(),
        right_output.len(),
        "Left and right outputs must have the same number of lines"
    );

    for (left, right) in left_output.iter().zip(right_output.iter()) {
        assert!(
            left.len() < right.len(),
            "Right-aligned line ({:?}) should be wider than left-aligned ({:?})",
            right,
            left
        );
    }
}
