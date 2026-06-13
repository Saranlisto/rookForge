use std::env;
use std::process::ExitCode;

use rookforge_core::ENGINE_NAME;

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
        ["perft", "help"] | ["perft", "--help"] | ["perft", "-h"] => Ok(perft_help_text()),
        ["perft", ..] => Err("perft is not implemented yet. Try `rookforge perft --help`.".into()),
        [unknown, ..] => Err(format!(
            "unknown command `{unknown}`. Try `rookforge help`."
        )),
    }
}

fn help_text() -> String {
    format!(
        "{ENGINE_NAME} chess engine scaffold\n\nUSAGE:\n    rookforge <COMMAND>\n\nCOMMANDS:\n    help        Show this help text\n    perft       Inspect perft command options\n\nOPTIONS:\n    -h, --help      Show this help text\n    -V, --version   Show version information\n"
    )
}

fn perft_help_text() -> String {
    "rookforge perft\n\nUSAGE:\n    rookforge perft --help\n\nSTATUS:\n    Perft execution is planned but not implemented in the scaffold.\n"
        .to_string()
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
}
