use std::env;
use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use rookforge_core::{
    apply_move, evaluate, generate_bishop_moves, generate_king_moves, generate_knight_moves,
    generate_legal_moves, generate_pawn_moves, generate_pseudo_legal_moves, generate_queen_moves,
    generate_rook_moves, is_square_attacked, perft, perft_divide, search_best_move_with_options,
    Color, Move, PieceKind, Position, SearchOptions, Square, ENGINE_NAME, STARTING_POSITION_FEN,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_CLI_DEPTH: u32 = 6;
const FEN_HINT: &str = "Use `startpos` or a full six-field FEN.";
const MOVE_HINT: &str = "Use UCI-style long algebraic notation such as `e2e4` or `e7e8q`.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
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
        ["attacks", "help"] | ["attacks", "--help"] | ["attacks", "-h"] => Ok(attacks_help_text()),
        ["attacks", "--fen", fen, "--square", square, "--by", color] => {
            attacks_from_fen(fen, square, color)
        }
        ["attacks", ..] => Err("invalid attacks command. Try `rookforge attacks --help`.".into()),
        ["board", "help"] | ["board", "--help"] | ["board", "-h"] => Ok(board_help_text()),
        ["board", "--fen", fen] => board_from_fen(fen),
        ["board", ..] => Err("invalid board command. Try `rookforge board --help`.".into()),
        ["eval", "help"] | ["eval", "--help"] | ["eval", "-h"] => Ok(eval_help_text()),
        ["eval", "--fen", fen] => eval_from_fen(fen),
        ["eval", ..] => Err("invalid eval command. Try `rookforge eval --help`.".into()),
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
        ["movegen", "legal", "--fen", fen] => legal_moves_from_fen(fen),
        ["movegen", ..] => Err("invalid movegen command. Try `rookforge movegen --help`.".into()),
        ["perft", "help"] | ["perft", "--help"] | ["perft", "-h"] => Ok(perft_help_text()),
        ["perft", "--fen", fen, "--depth", depth] => perft_from_fen(fen, depth),
        ["perft", "--fen", fen, "--depth", depth, "--divide"] => perft_divide_from_fen(fen, depth),
        ["perft", ..] => Err("invalid perft command. Try `rookforge perft --help`.".into()),
        ["search", "help"] | ["search", "--help"] | ["search", "-h"] => Ok(search_help_text()),
        ["search", "--fen", fen, "--depth", depth] => {
            search_from_fen(fen, depth, true, OutputFormat::Text)
        }
        ["search", "--fen", fen, "--depth", depth, "--no-quiescence"] => {
            search_from_fen(fen, depth, false, OutputFormat::Text)
        }
        ["search", "--fen", fen, "--depth", depth, "--json"] => {
            search_from_fen(fen, depth, true, OutputFormat::Json)
        }
        ["search", "--fen", fen, "--depth", depth, "--no-quiescence", "--json"]
        | ["search", "--fen", fen, "--depth", depth, "--json", "--no-quiescence"] => {
            search_from_fen(fen, depth, false, OutputFormat::Json)
        }
        ["search", ..] => Err("invalid search command. Try `rookforge search --help`.".into()),
        [unknown, ..] => Err(format!(
            "unknown command `{unknown}`. Try `rookforge help`."
        )),
    }
}

fn help_text() -> String {
    format!(
        "{ENGINE_NAME} {VERSION}\nFrom-scratch Rust chess engine.\n\nUSAGE:\n    rookforge <COMMAND>\n\nCOMMANDS:\n    apply       Apply a move to a FEN position\n    attacks     Check whether a square is attacked\n    board       Print a FEN position as a board\n    eval        Evaluate a FEN position\n    help        Show this help text\n    move        Parse a UCI-style move\n    movegen     Generate selected pseudo-legal or legal move sets\n    perft       Count legal move-tree nodes\n    search      Search a FEN position\n\nOPTIONS:\n    -h, --help      Show this help text\n    -V, --version   Show version information\n\nEXAMPLES:\n    rookforge board --fen startpos\n    rookforge movegen legal --fen startpos\n    rookforge perft --fen startpos --depth 2\n    rookforge search --fen startpos --depth 3\n"
    )
}

fn apply_help_text() -> String {
    "rookforge apply\n\nUSAGE:\n    rookforge apply --fen <FEN|startpos> --move <MOVE>\n\nOPTIONS:\n    --fen <FEN|startpos>   Position to update\n    --move <MOVE>          UCI-style move, such as e2e4 or e7e8q\n\nSTATUS:\n    Applies a structurally parsed move to a FEN position for local debugging.\n"
        .to_string()
}

fn attacks_help_text() -> String {
    "rookforge attacks\n\nUSAGE:\n    rookforge attacks --fen <FEN|startpos> --square <SQUARE> --by <white|black>\n\nOPTIONS:\n    --fen <FEN|startpos>   Position to inspect\n    --square <SQUARE>      Board square from a1 through h8\n    --by <white|black>     Attacking color to query\n\nSTATUS:\n    Reports whether a square is attacked by the selected color for local debugging.\n"
        .to_string()
}

fn board_help_text() -> String {
    "rookforge board\n\nUSAGE:\n    rookforge board --fen <FEN|startpos>\n\nOPTIONS:\n    --fen <FEN|startpos>   Position to render\n\nSTATUS:\n    Prints a parsed FEN position as a human-readable board for local debugging.\n"
        .to_string()
}

fn eval_help_text() -> String {
    "rookforge eval\n\nUSAGE:\n    rookforge eval --fen <FEN|startpos>\n\nOPTIONS:\n    --fen <FEN|startpos>   Position to evaluate\n\nSTATUS:\n    Evaluates a position with the current material-only static evaluator.\n"
        .to_string()
}

fn move_help_text() -> String {
    "rookforge move\n\nUSAGE:\n    rookforge move --parse <MOVE>\n\nOPTIONS:\n    --parse <MOVE>   UCI-style move, such as e2e4 or e7e8q\n\nSTATUS:\n    Parses UCI-style long algebraic moves for local debugging.\n"
        .to_string()
}

fn movegen_help_text() -> String {
    "rookforge movegen\n\nUSAGE:\n    rookforge movegen pawns --fen <FEN|startpos>\n    rookforge movegen knights --fen <FEN|startpos>\n    rookforge movegen kings --fen <FEN|startpos>\n    rookforge movegen bishops --fen <FEN|startpos>\n    rookforge movegen rooks --fen <FEN|startpos>\n    rookforge movegen queens --fen <FEN|startpos>\n    rookforge movegen all --fen <FEN|startpos>\n    rookforge movegen legal --fen <FEN|startpos>\n\nOPTIONS:\n    --fen <FEN|startpos>   Position to generate moves from\n\nSTATUS:\n    Generates selected pseudo-legal or legal moves for local debugging.\n"
        .to_string()
}

fn perft_help_text() -> String {
    "rookforge perft\n\nUSAGE:\n    rookforge perft --fen <FEN|startpos> --depth <DEPTH>\n    rookforge perft --fen <FEN|startpos> --depth <DEPTH> --divide\n\nOPTIONS:\n    --fen <FEN|startpos>   Position to count from\n    --depth <DEPTH>        Integer search depth from 0 through 6\n    --divide               Print per-root-move counts\n\nSTATUS:\n    Counts legal move-tree nodes for local move-generation validation.\n"
        .to_string()
}

fn search_help_text() -> String {
    "rookforge search\n\nUSAGE:\n    rookforge search --fen <FEN|startpos> --depth <DEPTH>\n    rookforge search --fen <FEN|startpos> --depth <DEPTH> --no-quiescence\n    rookforge search --fen <FEN|startpos> --depth <DEPTH> --json\n\nOPTIONS:\n    --fen <FEN|startpos>   Position to search\n    --depth <DEPTH>        Integer search depth from 0 through 6\n    --no-quiescence        Disable quiescence for comparison/debugging\n    --json                 Emit a stable JSON object for automation\n\nSTATUS:\n    Searches a position with fixed-depth alpha-beta and quiescence by default.\n"
        .to_string()
}

fn board_from_fen(fen: &str) -> Result<String, String> {
    position_from_fen(fen).map(|position| format!("{}\n", position.to_pretty_string()))
}

fn eval_from_fen(fen: &str) -> Result<String, String> {
    let position = position_from_fen(fen)?;
    let score = evaluate(&position);

    Ok(format!(
        "fen: {fen}\nscore_cp: {score}\nperspective: white-positive\n"
    ))
}

fn apply_move_from_fen(fen: &str, value: &str) -> Result<String, String> {
    let position = position_from_fen(fen)?;
    let mv = move_from_cli(value)?;
    let result = apply_move(&position, mv)
        .map_err(|error| format!("cannot apply move `{value}`: {error}"))?;

    Ok(format!(
        "fen: {}\nboard:\n{}\n",
        result.to_fen(),
        result.to_pretty_string()
    ))
}

fn attacks_from_fen(fen: &str, square: &str, color: &str) -> Result<String, String> {
    let position = position_from_fen(fen)?;
    let square = Square::from_algebraic(square)
        .ok_or_else(|| format!("invalid square `{square}`: expected a square from a1 to h8"))?;
    let color = color_from_cli(color)?;
    let attacked = is_square_attacked(&position, square, color);

    Ok(format!(
        "square: {}\nby: {}\nattacked: {}\n",
        square.to_algebraic(),
        color_name(color),
        attacked
    ))
}

fn move_from_uci(value: &str) -> Result<String, String> {
    let mv = move_from_cli(value)?;

    Ok(format!(
        "from: {}\nto: {}\npromotion: {}\nuci: {}\n",
        mv.from.to_algebraic(),
        mv.to.to_algebraic(),
        promotion_name(mv.promotion),
        mv.to_uci()
    ))
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

fn legal_moves_from_fen(fen: &str) -> Result<String, String> {
    movegen_moves_from_fen(fen, generate_legal_moves)
}

fn perft_from_fen(fen: &str, depth: &str) -> Result<String, String> {
    let position = position_from_fen(fen)?;
    let depth = depth_from_cli(depth)?;
    let started_at = Instant::now();
    let nodes = perft(&position, depth);
    let elapsed = started_at.elapsed();

    Ok(format!(
        "fen: {fen}\ndepth: {depth}\nnodes: {nodes}\nelapsed: {}\nnodes_per_second: {}\n",
        format_duration(elapsed),
        nodes_per_second(nodes, elapsed)
    ))
}

fn perft_divide_from_fen(fen: &str, depth: &str) -> Result<String, String> {
    let position = position_from_fen(fen)?;
    let depth = depth_from_cli(depth)?;
    let started_at = Instant::now();
    let rows = perft_divide(&position, depth);
    let elapsed = started_at.elapsed();
    let total = rows.iter().map(|(_, nodes)| *nodes).sum::<u64>();

    let mut output = format!("fen: {fen}\ndepth: {depth}\n");
    for (mv, nodes) in rows {
        output.push_str(&format!("{}: {nodes}\n", mv.to_uci()));
    }
    output.push_str(&format!(
        "total: {total}\nelapsed: {}\nnodes_per_second: {}\n",
        format_duration(elapsed),
        nodes_per_second(total, elapsed)
    ));

    Ok(output)
}

fn search_from_fen(
    fen: &str,
    depth: &str,
    quiescence: bool,
    format: OutputFormat,
) -> Result<String, String> {
    let position = position_from_fen(fen)?;
    let depth = depth_from_cli(depth)?;
    let result = search_best_move_with_options(&position, SearchOptions { depth, quiescence });
    let best_move = result.best_move.map(|mv| mv.to_uci());

    match format {
        OutputFormat::Text => Ok(format_search_text(fen, &result, best_move.as_deref())),
        OutputFormat::Json => Ok(format_search_json(fen, &result, best_move.as_deref())),
    }
}

fn format_search_text(
    fen: &str,
    result: &rookforge_core::SearchResult,
    best_move: Option<&str>,
) -> String {
    let best_move = best_move.unwrap_or("none");

    format!(
        "fen: {fen}\ndepth: {}\nbest_move: {best_move}\nscore_cp: {}\nnodes: {}\nqnodes: {}\nelapsed_ms: {}\nnodes_per_second: {}\noutcome: {}\nsearch: {}\n",
        result.depth,
        result.score_cp,
        result.nodes,
        result.qnodes,
        result.elapsed_ms,
        result.nodes_per_second,
        result.outcome.label(),
        result.search.label()
    )
}

fn format_search_json(
    fen: &str,
    result: &rookforge_core::SearchResult,
    best_move: Option<&str>,
) -> String {
    let best_move = best_move
        .map(json_string)
        .unwrap_or_else(|| "null".to_string());

    format!(
        "{{\"fen\":{},\"depth\":{},\"best_move\":{},\"score_cp\":{},\"nodes\":{},\"qnodes\":{},\"elapsed_ms\":{},\"nodes_per_second\":{},\"outcome\":{},\"search\":{}}}\n",
        json_string(fen),
        result.depth,
        best_move,
        result.score_cp,
        result.nodes,
        result.qnodes,
        result.elapsed_ms,
        result.nodes_per_second,
        json_string(result.outcome.label()),
        json_string(result.search.label())
    )
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

    Position::from_fen(fen).map_err(|error| format!("invalid FEN `{fen}`: {error}. {FEN_HINT}"))
}

fn depth_from_cli(depth: &str) -> Result<u32, String> {
    let depth = depth.parse::<u32>().map_err(|_| {
        format!("invalid depth `{depth}`: expected an integer from 0 through {MAX_CLI_DEPTH}")
    })?;

    if depth > MAX_CLI_DEPTH {
        return Err(format!(
            "invalid depth `{depth}`: maximum supported CLI depth is {MAX_CLI_DEPTH}"
        ));
    }

    Ok(depth)
}

fn move_from_cli(value: &str) -> Result<Move, String> {
    Move::from_uci(value).map_err(|error| format!("invalid move `{value}`: {error}. {MOVE_HINT}"))
}

fn json_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');

    for marker in value.chars() {
        match marker {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            value if value.is_control() => {
                let _ = write!(output, "\\u{:04x}", value as u32);
            }
            value => output.push(value),
        }
    }

    output.push('"');
    output
}

fn format_duration(duration: Duration) -> String {
    format!("{:.6}s", duration.as_secs_f64())
}

fn nodes_per_second(nodes: u64, elapsed: Duration) -> u64 {
    let seconds = elapsed.as_secs_f64();

    if seconds == 0.0 {
        nodes
    } else {
        (nodes as f64 / seconds).round() as u64
    }
}

fn color_from_cli(value: &str) -> Result<Color, String> {
    match value {
        "white" => Ok(Color::White),
        "black" => Ok(Color::Black),
        _ => Err(format!("invalid color: {value}. Use `white` or `black`.")),
    }
}

const fn color_name(color: Color) -> &'static str {
    match color {
        Color::White => "white",
        Color::Black => "black",
    }
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

        assert!(output.contains(&format!("{ENGINE_NAME} {VERSION}")));
        assert!(output.contains("COMMANDS:"));
        assert!(output.contains("attacks"));
        assert!(output.contains("eval"));
        assert!(output.contains("perft"));
        assert!(output.contains("search"));
        assert!(output.contains("EXAMPLES:"));
    }

    #[test]
    fn command_help_outputs_use_consistent_sections() {
        let commands = [
            "apply", "attacks", "board", "eval", "move", "movegen", "perft", "search",
        ];

        for command in commands {
            let output =
                run([command.to_string(), "--help".to_string()]).expect("command help output");

            assert!(
                output.contains("USAGE:"),
                "{command} help should include usage"
            );
            assert!(
                output.contains("STATUS:"),
                "{command} help should include status"
            );
        }
    }

    #[test]
    fn eval_help_reports_command_usage() {
        let output = run(["eval".to_string(), "--help".to_string()]).expect("eval help");

        assert!(output.contains("rookforge eval"));
        assert!(output.contains("--fen <FEN|startpos>"));
    }

    #[test]
    fn eval_command_reports_starting_position_score() {
        let output = run([
            "eval".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
        ])
        .expect("eval output");

        assert_eq!(
            output,
            "fen: startpos\nscore_cp: 0\nperspective: white-positive\n"
        );
    }

    #[test]
    fn eval_command_reports_material_advantage() {
        let output = run([
            "eval".to_string(),
            "--fen".to_string(),
            "8/8/8/8/8/8/4P3/4K2k w - - 0 1".to_string(),
        ])
        .expect("eval output");

        assert_eq!(
            output,
            "fen: 8/8/8/8/8/8/4P3/4K2k w - - 0 1\nscore_cp: 100\nperspective: white-positive\n"
        );
    }

    #[test]
    fn perft_help_reports_command_usage() {
        let output = run(["perft".to_string(), "--help".to_string()]).expect("perft help");

        assert!(output.contains("rookforge perft"));
        assert!(output.contains("--depth <DEPTH>"));
        assert!(output.contains("--divide"));
    }

    #[test]
    fn search_help_reports_command_usage() {
        let output = run(["search".to_string(), "--help".to_string()]).expect("search help");

        assert!(output.contains("rookforge search"));
        assert!(output.contains("--depth <DEPTH>"));
        assert!(output.contains("--no-quiescence"));
        assert!(output.contains("--json"));
    }

    #[test]
    fn search_command_reports_starting_position_result() {
        let output = run([
            "search".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--depth".to_string(),
            "1".to_string(),
        ])
        .expect("search output");

        assert!(output.contains("fen: startpos\n"));
        assert!(output.contains("depth: 1\n"));
        assert!(output.contains("best_move: "));
        assert!(!output.contains("best_move: none"));
        assert!(output.contains("score_cp: 0\n"));
        assert!(output.contains("nodes: "));
        assert!(output.contains("qnodes: "));
        assert!(output.contains("elapsed_ms: "));
        assert!(output.contains("nodes_per_second: "));
        assert!(output.contains("outcome: bestmove\n"));
        assert!(output.contains("search: alpha-beta+quiescence\n"));
    }

    #[test]
    fn search_command_reports_no_move_for_empty_position() {
        let output = run([
            "search".to_string(),
            "--fen".to_string(),
            "8/8/8/8/8/8/8/8 w - - 0 1".to_string(),
            "--depth".to_string(),
            "1".to_string(),
        ])
        .expect("search output");

        assert!(output.contains("fen: 8/8/8/8/8/8/8/8 w - - 0 1\n"));
        assert!(output.contains("depth: 1\n"));
        assert!(output.contains("best_move: none\n"));
        assert!(output.contains("score_cp: 0\n"));
        assert!(output.contains("nodes: 1\n"));
        assert!(output.contains("qnodes: 0\n"));
        assert!(output.contains("outcome: stalemate\n"));
        assert!(output.contains("search: alpha-beta+quiescence\n"));
    }

    #[test]
    fn search_command_supports_no_quiescence() {
        let output = run([
            "search".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--depth".to_string(),
            "1".to_string(),
            "--no-quiescence".to_string(),
        ])
        .expect("search output");

        assert!(output.contains("qnodes: 0\n"));
        assert!(output.contains("search: alpha-beta\n"));
    }

    #[test]
    fn search_command_supports_json_output() {
        let output = run([
            "search".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--depth".to_string(),
            "0".to_string(),
            "--json".to_string(),
        ])
        .expect("search json output");

        assert!(output.starts_with('{'));
        assert!(output.ends_with("}\n"));
        assert!(output.contains("\"fen\":\"startpos\""));
        assert!(output.contains("\"depth\":0"));
        assert!(output.contains("\"best_move\":null"));
        assert!(output.contains("\"score_cp\":0"));
        assert!(output.contains("\"nodes\":1"));
        assert!(output.contains("\"qnodes\":0"));
        assert!(output.contains("\"outcome\":\"bestmove\""));
        assert!(output.contains("\"search\":\"alpha-beta+quiescence\""));
    }

    #[test]
    fn search_command_supports_json_without_quiescence() {
        let output = run([
            "search".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--depth".to_string(),
            "0".to_string(),
            "--json".to_string(),
            "--no-quiescence".to_string(),
        ])
        .expect("search json output");

        assert!(output.contains("\"qnodes\":0"));
        assert!(output.contains("\"search\":\"alpha-beta\""));
    }

    #[test]
    fn json_string_escapes_special_characters() {
        assert_eq!(
            json_string("quote\" slash\\ newline\n tab\t"),
            "\"quote\\\" slash\\\\ newline\\n tab\\t\""
        );
    }

    #[test]
    fn board_command_rejects_invalid_fen_with_hint() {
        let error = run([
            "board".to_string(),
            "--fen".to_string(),
            "8/8/8/8/8/8/8 w - - 0 1".to_string(),
        ])
        .expect_err("invalid FEN should fail");

        assert!(error.contains("invalid FEN"));
        assert!(error.contains(FEN_HINT));
    }

    #[test]
    fn move_command_rejects_invalid_move_with_hint() {
        let error = run([
            "move".to_string(),
            "--parse".to_string(),
            "e2e9".to_string(),
        ])
        .expect_err("invalid move should fail");

        assert!(error.contains("invalid move `e2e9`"));
        assert!(error.contains(MOVE_HINT));
    }

    #[test]
    fn search_command_rejects_invalid_depth_with_hint() {
        let error = run([
            "search".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--depth".to_string(),
            "deep".to_string(),
        ])
        .expect_err("invalid depth should fail");

        assert!(error.contains("invalid depth `deep`"));
        assert!(error.contains("expected an integer"));
    }

    #[test]
    fn search_command_rejects_depth_above_cli_limit() {
        let error = run([
            "search".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--depth".to_string(),
            "7".to_string(),
        ])
        .expect_err("too-deep search should fail");

        assert!(error.contains("maximum supported CLI depth is 6"));
    }

    #[test]
    fn perft_command_reports_starting_position_nodes() {
        let output = run([
            "perft".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--depth".to_string(),
            "2".to_string(),
        ])
        .expect("perft output");

        assert!(output.contains("fen: startpos\n"));
        assert!(output.contains("depth: 2\n"));
        assert!(output.contains("nodes: 400\n"));
        assert!(output.contains("elapsed: "));
        assert!(output.contains("nodes_per_second: "));
    }

    #[test]
    fn perft_divide_command_reports_root_move_counts() {
        let output = run([
            "perft".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--depth".to_string(),
            "2".to_string(),
            "--divide".to_string(),
        ])
        .expect("perft divide output");

        assert!(output.contains("fen: startpos\n"));
        assert!(output.contains("depth: 2\n"));
        assert!(output.contains("a2a3: 20\n"));
        assert!(output.contains("e2e4: 20\n"));
        assert!(output.contains("total: 400\n"));
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
    fn attacks_help_reports_debug_status() {
        let output = run(["attacks".to_string(), "--help".to_string()]).expect("attacks help");

        assert!(output.contains("rookforge attacks"));
        assert!(output.contains("--square <SQUARE>"));
    }

    #[test]
    fn attacks_command_reports_attacked_square() {
        let output = run([
            "attacks".to_string(),
            "--fen".to_string(),
            "4r3/8/8/8/4K3/8/8/8 w - - 0 1".to_string(),
            "--square".to_string(),
            "e4".to_string(),
            "--by".to_string(),
            "black".to_string(),
        ])
        .expect("attacks output");

        assert_eq!(output, "square: e4\nby: black\nattacked: true\n");
    }

    #[test]
    fn attacks_command_reports_unattacked_square() {
        let output = run([
            "attacks".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--square".to_string(),
            "e4".to_string(),
            "--by".to_string(),
            "white".to_string(),
        ])
        .expect("attacks output");

        assert_eq!(output, "square: e4\nby: white\nattacked: false\n");
    }

    #[test]
    fn attacks_command_rejects_invalid_color() {
        let error = run([
            "attacks".to_string(),
            "--fen".to_string(),
            "startpos".to_string(),
            "--square".to_string(),
            "e4".to_string(),
            "--by".to_string(),
            "green".to_string(),
        ])
        .expect_err("invalid color should fail");

        assert!(error.contains("invalid color"));
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

    #[test]
    fn movegen_legal_command_prints_starting_position_moves() {
        let output = run([
            "movegen".to_string(),
            "legal".to_string(),
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
