use mons_rust::{FenRepresentable, Input, MonsGame, Output};
use rand::Rng;
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

const DEFAULT_RULES_DIR: &str = "rules-tests";
const MAX_TRANSITIONS_PER_GAME: usize = 50_000;

#[derive(Debug)]
struct CliOptions {
    rules_dir: PathBuf,
    target_new_cases: Option<usize>,
}

#[derive(Debug, Clone)]
struct TestCase {
    fen_before: String,
    fen_after: String,
    input_fen: String,
    output_fen: String,
}

#[derive(Debug, Clone)]
struct SaveCandidate {
    test_case: TestCase,
    turn_number: i32,
    white_score: i32,
    black_score: i32,
}

impl TestCase {
    fn canonical_json_bytes(&self) -> Vec<u8> {
        // Keep stable key ordering so hash-based naming remains deterministic.
        format!(
            "{{\"fenAfter\":\"{}\",\"fenBefore\":\"{}\",\"inputFen\":\"{}\",\"outputFen\":\"{}\"}}",
            escape_json_string(self.fen_after.as_str()),
            escape_json_string(self.fen_before.as_str()),
            escape_json_string(self.input_fen.as_str()),
            escape_json_string(self.output_fen.as_str())
        )
        .into_bytes()
    }
}

struct CaseSaver {
    rules_dir: PathBuf,
    known_ids: HashSet<String>,
}

impl CaseSaver {
    fn new(rules_dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(rules_dir)
            .map_err(|err| format!("failed to create `{}`: {err}", rules_dir.display()))?;

        let mut known_ids = HashSet::new();
        let entries = fs::read_dir(rules_dir)
            .map_err(|err| format!("failed to read `{}`: {err}", rules_dir.display()))?;

        for entry in entries {
            let entry = entry.map_err(|err| format!("failed to read fixture entry: {err}"))?;
            if entry.path().is_file() {
                if let Some(id) = entry.file_name().to_str() {
                    known_ids.insert(id.to_string());
                }
            }
        }

        Ok(Self {
            rules_dir: rules_dir.to_path_buf(),
            known_ids,
        })
    }

    fn write_if_unique(&mut self, test_case: &TestCase) -> Result<Option<String>, String> {
        let data = test_case.canonical_json_bytes();
        let id = fnv1a_hash(data.as_slice()).to_string();

        if self.known_ids.contains(id.as_str()) {
            return Ok(None);
        }

        let path = self.rules_dir.join(id.as_str());
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                self.known_ids.insert(id);
                return Ok(None);
            }
            Err(err) => {
                return Err(format!("failed to create `{}`: {err}", path.display()));
            }
        };

        file.write_all(&data)
            .map_err(|err| format!("failed to write `{}`: {err}", path.display()))?;
        self.known_ids.insert(id.clone());
        Ok(Some(id))
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let Some(options) = parse_cli()? else {
        return Ok(());
    };

    let mut saver = CaseSaver::new(options.rules_dir.as_path())?;
    let mut rng = rand::thread_rng();
    let mut new_cases_written = 0usize;

    loop {
        let mut game = MonsGame::new(false);
        let mut pending_inputs: Vec<Input> = vec![];
        let mut transitions = 0usize;
        let mut saved_first_options_for_turn = false;
        let mut last_board_modified_case_for_turn: Option<SaveCandidate> = None;

        while game.winner_color().is_none() && transitions < MAX_TRANSITIONS_PER_GAME {
            let turn_before = game.turn_number;
            let fen_before = game.fen();
            let output = game.process_input(pending_inputs.clone(), false, false);
            let fen_after = game.fen();

            let candidate = SaveCandidate {
                test_case: TestCase {
                    fen_before,
                    fen_after,
                    input_fen: Input::fen_from_array(&pending_inputs),
                    output_fen: output.fen(),
                },
                turn_number: turn_before,
                white_score: game.white_score,
                black_score: game.black_score,
            };

            if !saved_first_options_for_turn
                && matches!(
                    output,
                    Output::LocationsToStartFrom(_) | Output::NextInputOptions(_)
                )
            {
                if persist_candidate(
                    &mut saver,
                    &candidate,
                    &mut new_cases_written,
                    options.target_new_cases,
                )? {
                    return Ok(());
                }
                saved_first_options_for_turn = true;
            }

            if board_fen(candidate.test_case.fen_before.as_str())
                != board_fen(candidate.test_case.fen_after.as_str())
            {
                last_board_modified_case_for_turn = Some(candidate.clone());
            }

            let turn_advanced = game.turn_number != turn_before;
            let game_over = game.winner_color().is_some();
            if turn_advanced || game_over {
                if let Some(board_modified_candidate) = last_board_modified_case_for_turn.take() {
                    if persist_candidate(
                        &mut saver,
                        &board_modified_candidate,
                        &mut new_cases_written,
                        options.target_new_cases,
                    )? {
                        return Ok(());
                    }
                }
            }

            if turn_advanced {
                saved_first_options_for_turn = false;
            }

            transitions += 1;
            match output {
                Output::LocationsToStartFrom(locations) => {
                    if locations.is_empty() {
                        pending_inputs.clear();
                        continue;
                    }
                    let random_index = rng.gen_range(0..locations.len());
                    pending_inputs.push(Input::Location(locations[random_index]));
                }
                Output::NextInputOptions(next_inputs) => {
                    if next_inputs.is_empty() {
                        pending_inputs.clear();
                        continue;
                    }
                    let random_index = rng.gen_range(0..next_inputs.len());
                    pending_inputs.push(next_inputs[random_index].input);
                }
                Output::Events(_) | Output::InvalidInput => {
                    // Once a move resolves or chain is invalid, ask with empty input again.
                    pending_inputs.clear();
                }
            }
        }
    }
}

fn persist_candidate(
    saver: &mut CaseSaver,
    candidate: &SaveCandidate,
    new_cases_written: &mut usize,
    target_new_cases: Option<usize>,
) -> Result<bool, String> {
    if let Some(id) = saver.write_if_unique(&candidate.test_case)? {
        *new_cases_written += 1;
        println!(
            "✅ {} score {}:{} turn {}",
            id, candidate.white_score, candidate.black_score, candidate.turn_number
        );

        if let Some(limit) = target_new_cases {
            if *new_cases_written >= limit {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn board_fen(game_fen: &str) -> &str {
    game_fen.split_whitespace().nth(9).unwrap_or("")
}

fn parse_cli() -> Result<Option<CliOptions>, String> {
    let mut options = CliOptions {
        rules_dir: PathBuf::from(DEFAULT_RULES_DIR),
        target_new_cases: None,
    };

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--dir requires a value".to_string())?;
                options.rules_dir = PathBuf::from(value);
            }
            "--target-new" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--target-new requires a value".to_string())?;
                let parsed = value
                    .parse::<usize>()
                    .map_err(|err| format!("invalid --target-new value `{value}`: {err}"))?;
                options.target_new_cases = Some(parsed);
            }
            "--help" | "-h" => {
                print_help();
                return Ok(None);
            }
            _ => {
                return Err(format!("unknown argument `{arg}`. Use --help for usage."));
            }
        }
    }

    Ok(Some(options))
}

fn print_help() {
    println!("Generate random unique rules-tests fixtures.");
    println!();
    println!("Usage:");
    println!("  cargo run --quiet --bin generate_rules_tests -- [options]");
    println!("  ./scripts/generate-rules-tests.sh [options]");
    println!();
    println!("Options:");
    println!("  --dir <path>         Fixture directory (default: {DEFAULT_RULES_DIR})");
    println!("  --target-new <n>     Stop after writing n new unique fixtures");
    println!("                       If omitted, keeps generating until interrupted.");
    println!("  --help, -h           Show this help message");
}

fn escape_json_string(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            c if c.is_control() => {
                let _ = write!(&mut escaped, "\\u{:04X}", c as u32);
            }
            c => escaped.push(c),
        }
    }
    escaped
}

fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let prime: u64 = 1099511628211;
    let mut hash: u64 = 14695981039346656037;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(prime);
    }
    hash
}
