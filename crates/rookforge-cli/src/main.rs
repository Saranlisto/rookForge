use std::env;
use std::process::ExitCode;

use rookforge_core::{
    apply_move, generate_bishop_moves, generate_king_moves, generate_knight_moves,
    generate_pawn_moves, generate_pseudo_legal_moves, generate_queen_moves, generate_rook_moves,
    Move, PieceKind, Position, ENGINE_NAME, STARTING_POSITION_FEN,
};

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
        ["apply", "help"] | ["apply", "--help"] | ["apply", "-h"] => Ok(apply_help_text()),
        ["apply", "--fen", fen, "--move", value] => apply_move_from_fen(fen, value),
        ["apply", ..] => Err("invalid apply command. Try `rookforge apply --help`.".into()),
        ["board", "help"] | ["board", "--help"] | ["board", "-h"] => Ok(board_help_text()),
        ["board", "--fen", fen] => board_from_fen(fen),
        ["board", ..] => Err("invalid board command. Try `rookforge board --help`.".into()),
        ["move", "help"] | ["move", "--help"] | ["move", "-h"] => Ok(move_help_text()),
        ["move", "--parse", value] => move_from_uci(value),
        ["move", ..] => Err("invalid move command. Try `rookforge move --help`.".into()),
        ["movegen", "help"] | ["movegen", "--help"] | ["movegen", "-h"] => Ok(movegen_help_text()),
        ["movegen", "pawns", "--fen", fen] => pawn_moves_from_fen(fen),
        ["movegen", "knights", "--fen", fen] => knight_moves_from_fen(fen),
        ["movegen", "kings", "--fen", fen] => king_moves_from_fen(fen),
        ["movegen", "bishops", "--fen", fen] => bishop_moves_from_fen(fen),
        ["movegen", "rooks", "--fen", fen] => rook_moves_from_fen(fen),
        ["movegen", "queens", "--fen", fen] => queen_moves_from_fen(fen),
        ["movegen", "all", "--fen", fen] => pseudo_legal_moves_from_fen(fen),
        ["movegen", ..] => Err("invalid movegen command. Try `rookforge movegen --help`.".into()),
        ["perft", "help"] | ["perft", "--help"] | ["perft", "-h"] => Ok(perft_help_text()),
        ["perft", ..] => Err("perft is not implemented yet. Try `rookforge perft --help`.".into()),
        [unknown, ..] => Err(format!(
            "unknown command `{unknown}`. Try `rookforge help`."
        )),
    }
}

fn help_text() -> String {
    format!(
        "{ENGINE_NAME} chess engine scaffold\n\nUSAGE:\n    rookforge <COMMAND>\n\nCOMMANDS:\n    apply       Apply a move to a FEN position\n    board       Print a FEN position as a board\n    help        Show this help text\n    move        Parse a UCI-style move\n    movegen     Generate selected pseudo-legal moves\n    perft       Inspect perft command options\n\nOPTIONS:\n    -h, --help      Show this help text\n    -V, --version   Show version information\n"
    )
}

fn apply_help_text() -> String {
    "rookforge apply\n\nUSAGE:\n    rookforge apply --fen <FEN|startpos> --move <MOVE>\n\nSTATUS:\n    Applies a structurally parsed move to a FEN position for local debugging.\n"
        .to_string()
}

fn board_help_text() -> String {
    "rookforge board\n\nUSAGE:\n    rookforge board --fen <FEN|startpos>\n\nSTATUS:\n    Prints a parsed FEN position as a human-readable board for local debugging.\n"
        .to_string()
}

fn move_help_text() -> String {
    "rookforge move\n\nUSAGE:\n    rookforge move --parse <MOVE>\n\nSTATUS:\n    Parses UCI-style long algebraic moves for local debugging.\n"
        .to_string()
}

fn movegen_help_text() -> String {
    "rookforge movegen\n\nUSAGE:\n    rookforge movegen pawns --fen <FEN|startpos>\n    rookforge movegen knights --fen <FEN|startpos>\n    rookforge movegen kings --fen <FEN|startpos>\n    rookforge movegen bishops --fen <FEN|startpos>\n    rookforge movegen rooks --fen <FEN|startpos>\n    rookforge movegen queens --fen <FEN|startpos>\n    rookforge movegen all --fen <FEN|startpos>\n\nSTATUS:\n    Generates selected pseudo-legal moves for local debugging.\n"
        .to_string()
}

fn perft_help_text() -> String {
    "rookforge perft\n\nUSAGE:\n    rookforge perft --help\n\nSTATUS:\n    Perft execution is planned but not implemented in the scaffold.\n"
        .to_string()
}

fn board_from_fen(fen: &str) -> Result<String, String> {
    position_from_fen(fen).map(|position| format!("{}\n", position.to_pretty_string()))
}

fn apply_move_from_fen(fen: &str, value: &str) -> Result<String, String> {
    let position = position_from_fen(fen)?;
    let mv = Move::from_uci(value).map_err(|error| format!("invalid move: {error}"))?;
    let result =
        apply_move(&position, mv).map_err(|error| format!("cannot apply move: {error}"))?;

    Ok(format!(
        "fen: {}\nboard:\n{}\n",
        result.to_fen(),
        result.to_pretty_string()
    ))
}

fn move_from_uci(value: &str) -> Result<String, String> {
    Move::from_uci(value)
        .map(|mv| {
            format!(
                "from: {}\nto: {}\npromotion: {}\nuci: {}\n",
                mv.from.to_algebraic(),
                mv.to.to_algebraic(),
                promotion_name(mv.promotion),
                mv.to_uci()
            )
        })
        .map_err(|error| format!("invalid move: {error}"))
}

fn pawn_moves_from_fen(fen: &str) -> Result<String, String> {
    movegen_moves_from_fen(fen, generate_pawn_moves)
}

fn knight_moves_from_fen(fen: &str) -> Result<String, String> {
    movegen_moves_from_fen(fen, generate_knight_moves)
}

fn king_moves_from_fen(fen: &str) -> Result<String, String> {
    movegen_moves_from_fen(fen, generate_king_moves)
}

fn bishop_moves_from_fen(fen: &str) -> Result<String, String> {
    movegen_moves_from_fen(fen, generate_bishop_moves)
}

fn rook_moves_from_fen(fen: &str) -> Result<String, String> {
    movegen_moves_from_fen(fen, generate_rook_moves)
}

fn queen_moves_from_fen(fen: &str) -> Result<String, String> {
    movegen_moves_from_fen(fen, generate_queen_moves)
}

fn pseudo_legal_moves_from_fen(fen: &str) -> Result<String, String> {
    movegen_moves_from_fen(fen, generate_pseudo_legal_moves)
}

fn movegen_moves_from_fen(
    fen: &str,
    generator: fn(&Position) -> Vec<Move>,
) -> Result<String, String> {
    let position = position_from_fen(fen)?;
    let mut moves = generator(&position)
        .into_iter()
        .map(Move::to_uci)
        .collect::<Vec<_>>();
    moves.sort();

    let mut output = String::new();
    for mv in &moves {
        output.push_str(mv);
        output.push('\n');
    }
    output.push_str(&format!("total: {}\n", moves.len()));

    Ok(output)
}

fn position_from_fen(fen: &str) -> Result<Position, String> {
    let fen = if fen == "startpos" {
        STARTING_POSITION_FEN
    } else {
        fen
    };

    Position::from_fen(fen).map_err(|error| format!("invalid FEN: {error}"))
}

const fn promotion_name(promotion: Option<PieceKind>) -> &'static str {
    match promotion {
        None => "none",
        Some(PieceKind::Queen) => "queen",
        Some(PieceKind::Rook) => "rook",
        Some(PieceKind::Bishop) => "bishop",
        Some(PieceKind::Knight) => "knight",
        Some(PieceKind::King | PieceKind::Pawn) => "invalid",
    }
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

    #[test]
    fn apply_command_prints_resulting_fen_and_board() {
        let output = run([
            "apply".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--move".to_string(),
            "e2e4".to_string(),
        ])
        .expect("apply output");

        assert!(output.contains("fen: rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"));
        assert!(output.contains("4 . . . . P . . ."));
        assert!(output.contains("  a b c d e f g h"));
    }

    #[test]
    fn move_command_prints_normal_move() {
        let output = run([
            "move".to_string(),
            "--parse".to_string(),
            "e2e4".to_string(),
        ])
        .expect("move output");

        assert!(output.contains("from: e2"));
        assert!(output.contains("to: e4"));
        assert!(output.contains("promotion: none"));
        assert!(output.contains("uci: e2e4"));
    }

    #[test]
    fn move_command_prints_promotion_move() {
        let output = run([
            "move".to_string(),
            "--parse".to_string(),
            "e7e8q".to_string(),
        ])
        .expect("move output");

        assert!(output.contains("from: e7"));
        assert!(output.contains("to: e8"));
        assert!(output.contains("promotion: queen"));
        assert!(output.contains("uci: e7e8q"));
    }

    #[test]
    fn movegen_pawns_command_prints_starting_position_moves() {
        let output = run([
            "movegen".to_string(),
            "pawns".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
        ])
        .expect("movegen output");

        assert!(output.contains("a2a3"));
        assert!(output.contains("h2h4"));
        assert!(output.contains("total: 16"));
    }

    #[test]
    fn movegen_knights_command_prints_starting_position_moves() {
        let output = run([
            "movegen".to_string(),
            "knights".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
        ])
        .expect("movegen output");

        assert!(output.contains("b1a3"));
        assert!(output.contains("g1h3"));
        assert!(output.contains("total: 4"));
    }

    #[test]
    fn movegen_kings_command_prints_center_king_moves() {
        let output = run([
            "movegen".to_string(),
            "kings".to_string(),
            "--fen".to_string(),
            "8/8/8/8/4K3/8/8/8 w - - 0 1".to_string(),
        ])
        .expect("movegen output");

        assert!(output.contains("e4d3"));
        assert!(output.contains("e4f5"));
        assert!(output.contains("total: 8"));
    }

    #[test]
    fn movegen_bishops_command_prints_center_bishop_moves() {
        let output = run([
            "movegen".to_string(),
            "bishops".to_string(),
            "--fen".to_string(),
            "8/8/8/3B4/8/8/8/8 w - - 0 1".to_string(),
        ])
        .expect("movegen output");

        assert!(output.contains("d5a2"));
        assert!(output.contains("d5g8"));
        assert!(output.contains("total: 13"));
    }

    #[test]
    fn movegen_rooks_command_prints_center_rook_moves() {
        let output = run([
            "movegen".to_string(),
            "rooks".to_string(),
            "--fen".to_string(),
            "8/8/8/3R4/8/8/8/8 w - - 0 1".to_string(),
        ])
        .expect("movegen output");

        assert!(output.contains("d5a5"));
        assert!(output.contains("d5d8"));
        assert!(output.contains("total: 14"));
    }

    #[test]
    fn movegen_queens_command_prints_center_queen_moves() {
        let output = run([
            "movegen".to_string(),
            "queens".to_string(),
            "--fen".to_string(),
            "8/8/8/3Q4/8/8/8/8 w - - 0 1".to_string(),
        ])
        .expect("movegen output");

        assert!(output.contains("d5a2"));
        assert!(output.contains("d5d8"));
        assert!(output.contains("total: 27"));
    }

    #[test]
    fn movegen_all_command_prints_starting_position_moves() {
        let output = run([
            "movegen".to_string(),
            "all".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
        ])
        .expect("movegen output");

        assert!(output.contains("a2a3"));
        assert!(output.contains("b1c3"));
        assert!(output.contains("g1f3"));
        assert!(output.contains("h2h4"));
        assert!(output.contains("total: 20"));
    }
}
