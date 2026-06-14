use std::process::Command;

#[test]
fn version_flag_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_rookforge"))
        .arg("--version")
        .output()
        .expect("run rookforge --version");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("utf8 stdout"),
        format!("rookforge {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn help_command_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_rookforge"))
        .arg("help")
        .output()
        .expect("run rookforge help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("USAGE:"));
    assert!(stdout.contains("COMMANDS:"));
}

#[test]
fn perft_help_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_rookforge"))
        .args(["perft", "--help"])
        .output()
        .expect("run rookforge perft --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("rookforge perft"));
    assert!(stdout.contains("not implemented"));
}

#[test]
fn board_startpos_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_rookforge"))
        .args(["board", "--fen", "startpos"])
        .output()
        .expect("run rookforge board --fen startpos");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("8 r n b q k b n r"));
    assert!(stdout.contains("1 R N B Q K B N R"));
}

#[test]
fn board_empty_fen_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_rookforge"))
        .args(["board", "--fen", "8/8/8/8/8/8/8/8 w - - 0 1"])
        .output()
        .expect("run rookforge board --fen empty board");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("8 . . . . . . . ."));
    assert!(stdout.contains("  a b c d e f g h"));
}
