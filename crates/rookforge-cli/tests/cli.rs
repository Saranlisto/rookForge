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

#[test]
fn move_parse_normal_move_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_rookforge"))
        .args(["move", "--parse", "e2e4"])
        .output()
        .expect("run rookforge move --parse e2e4");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("from: e2"));
    assert!(stdout.contains("to: e4"));
    assert!(stdout.contains("promotion: none"));
    assert!(stdout.contains("uci: e2e4"));
}

#[test]
fn move_parse_promotion_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_rookforge"))
        .args(["move", "--parse", "e7e8q"])
        .output()
        .expect("run rookforge move --parse e7e8q");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("from: e7"));
    assert!(stdout.contains("to: e8"));
    assert!(stdout.contains("promotion: queen"));
    assert!(stdout.contains("uci: e7e8q"));
}

#[test]
fn movegen_pawns_startpos_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_rookforge"))
        .args(["movegen", "pawns", "--fen", "startpos"])
        .output()
        .expect("run rookforge movegen pawns --fen startpos");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("a2a3"));
    assert!(stdout.contains("h2h4"));
    assert!(stdout.contains("total: 16"));
}

#[test]
fn movegen_knights_startpos_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_rookforge"))
        .args(["movegen", "knights", "--fen", "startpos"])
        .output()
        .expect("run rookforge movegen knights --fen startpos");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("b1a3"));
    assert!(stdout.contains("g1h3"));
    assert!(stdout.contains("total: 4"));
}

#[test]
fn movegen_kings_center_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_rookforge"))
        .args(["movegen", "kings", "--fen", "8/8/8/8/4K3/8/8/8 w - - 0 1"])
        .output()
        .expect("run rookforge movegen kings --fen center king");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("e4d3"));
    assert!(stdout.contains("e4f5"));
    assert!(stdout.contains("total: 8"));
}
