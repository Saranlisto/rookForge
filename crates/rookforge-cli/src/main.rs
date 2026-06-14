use std::env;
use std::process::ExitCode;

use rookforge_core::{Position, ENGINE_NAME, STARTING_POSITION_FEN};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    let args = args.into_iter().collect::<Vec<_>>();
    let parts = args.iter().map(String::as_str).collect::<Vec<_>>();

    match parts.as_slice() {
        [] | ["help"] | ["--help"] | ["-h"] => Ok(help_text()),
        ["--version"] | ["-V"] => Ok(format!("rookforge {VERSION}\n")),
        ["board", "help"] | ["board", "--help"] | ["board", "-h"] => Ok(board_help_text()),
        ["board", "--fen", fen] => board_from_fen(fen),
        ["board", ..] => Err("invalid board command. Try `rookforge board --help`.".into()),
        ["perft", "help"] | ["perft", "--help"] | ["perft", "-h"] => Ok(perft_help_text()),
        ["perft", ..] => Err("perft is not implemented yet. Try `rookforge perft --help`.".into()),
        [unknown, ..] => Err(format!(
            "unknown command `{unknown}`. Try `rookforge help`."
        )),
    }
}

fn help_text() -> String {
    format!(
        "{ENGINE_NAME} chess engine scaffold\n\nUSAGE:\n    rookforge <COMMAND>\n\nCOMMANDS:\n    board       Print a FEN position as a board\n    help        Show this help text\n    perft       Inspect perft command options\n\nOPTIONS:\n    -h, --help      Show this help text\n    -V, --version   Show version information\n"
    )
}

fn board_help_text() -> String {
    "rookforge board\n\nUSAGE:\n    rookforge board --fen <FEN|startpos>\n\nSTATUS:\n    Prints a parsed FEN position as a human-readable board for local debugging.\n"
        .to_string()
}

fn perft_help_text() -> String {
    "rookforge perft\n\nUSAGE:\n    rookforge perft --help\n\nSTATUS:\n    Perft execution is planned but not implemented in the scaffold.\n"
        .to_string()
}

fn board_from_fen(fen: &str) -> Result<String, String> {
    let fen = if fen == "startpos" {
        STARTING_POSITION_FEN
    } else {
        fen
    };

    Position::from_fen(fen)
        .map(|position| format!("{}\n", position.to_pretty_string()))
        .map_err(|error| format!("invalid FEN: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_flag_reports_binary_name() {
        let output = run(["--version".to_string()]).expect("version output");

        assert_eq!(output, format!("rookforge {VERSION}\n"));
    }

    #[test]
    fn help_command_reports_available_commands() {
        let output = run(["help".to_string()]).expect("help output");

        assert!(output.contains("COMMANDS:"));
        assert!(output.contains("perft"));
    }

    #[test]
    fn perft_help_reports_scaffold_status() {
        let output = run(["perft".to_string(), "--help".to_string()]).expect("perft help");

        assert!(output.contains("rookforge perft"));
        assert!(output.contains("not implemented"));
    }

    #[test]
    fn board_command_prints_starting_position() {
        let output = run([
            "board".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
        ])
        .expect("board output");

        assert!(output.contains("8 r n b q k b n r"));
        assert!(output.contains("1 R N B Q K B N R"));
    }

    #[test]
    fn board_command_prints_empty_position() {
        let output = run([
            "board".to_string(),
            "--fen".to_string(),
            "8/8/8/8/8/8/8/8 w - - 0 1".to_string(),
        ])
        .expect("board output");

        assert!(output.contains("8 . . . . . . . ."));
        assert!(output.contains("  a b c d e f g h"));
    }
}
