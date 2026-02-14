use mons_rust::{FenRepresentable, Input, MonsGame};
use std::cmp::min;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process;

const DEFAULT_RULES_DIR: &str = "rules-tests";
const PROGRESS_INTERVAL: usize = 500;
const MAX_FAILURE_DETAILS: usize = 50;
const FAIL_SEPARATOR: &str =
    "================================================================================";

#[derive(Debug)]
struct CliOptions {
    rules_dir: PathBuf,
    limit: Option<usize>,
    log_path: Option<PathBuf>,
    verbose: bool,
}

#[derive(Debug)]
struct RuleTestCase {
    fen_before: String,
    fen_after: String,
    input_fen: String,
    output_fen: String,
}

#[derive(Debug)]
struct CaseResult {
    id: String,
    passed: bool,
    summary: String,
    details: Vec<String>,
}

impl CaseResult {
    fn pass(id: String) -> Self {
        Self {
            id,
            passed: true,
            summary: "ok".to_string(),
            details: vec![],
        }
    }

    fn fail(id: String, summary: String, details: Vec<String>) -> Self {
        Self {
            id,
            passed: false,
            summary,
            details,
        }
    }
}

fn main() {
    if let Err(err) = run() {
        if !err.ends_with("rules test(s) failed") {
            eprintln!("{err}");
        }
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let Some(options) = parse_cli()? else {
        return Ok(());
    };

    let mut logger = Logger::new(options.log_path.as_deref())
        .map_err(|err| format!("log setup failed: {err}"))?;
    let mut fixture_paths = collect_fixture_paths(options.rules_dir.as_path())?;

    let total = options
        .limit
        .map_or(fixture_paths.len(), |limit| min(limit, fixture_paths.len()));
    if total == 0 {
        return Err("no rules test fixtures to run".to_string());
    }

    logger
        .line(format!(
            "🧪 Running {total} rules tests from {}",
            options.rules_dir.display()
        ))
        .map_err(|err| format!("failed to write log: {err}"))?;

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut logged_failures = 0usize;
    let mut hidden_failures = 0usize;

    for (index, path) in fixture_paths.drain(..total).enumerate() {
        let case_result = run_case(path.as_path());

        if case_result.passed {
            passed += 1;
            if options.verbose {
                logger
                    .line(format!("✅ [PASS] {}", case_result.id))
                    .map_err(|err| format!("failed to write log: {err}"))?;
            }
        } else {
            failed += 1;
            if logged_failures < MAX_FAILURE_DETAILS {
                log_failure(&mut logger, &case_result)
                    .map_err(|err| format!("failed to write log: {err}"))?;
                logged_failures += 1;
            } else {
                hidden_failures += 1;
            }
        }

        let completed = index + 1;
        if completed % PROGRESS_INTERVAL == 0 || completed == total {
            logger
                .line(format!(
                    "📊 Progress: {completed}/{total} (pass: {passed}, fail: {failed})"
                ))
                .map_err(|err| format!("failed to write log: {err}"))?;
        }
    }

    if hidden_failures > 0 {
        logger
            .line(format!(
                "📝 Suppressed failure details for {hidden_failures} additional case(s)."
            ))
            .map_err(|err| format!("failed to write log: {err}"))?;
    }

    let finish_emoji = if failed > 0 { "❌" } else { "✅" };
    logger
        .line(format!(
            "{finish_emoji} Finished. Total: {total}, Passed: {passed}, Failed: {failed}"
        ))
        .map_err(|err| format!("failed to write log: {err}"))?;
    logger
        .flush()
        .map_err(|err| format!("failed to flush log output: {err}"))?;

    if failed > 0 {
        return Err(format!("{failed} rules test(s) failed"));
    }

    Ok(())
}

fn parse_cli() -> Result<Option<CliOptions>, String> {
    let mut options = CliOptions {
        rules_dir: PathBuf::from(DEFAULT_RULES_DIR),
        limit: None,
        log_path: None,
        verbose: false,
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
            "--limit" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--limit requires a value".to_string())?;
                let parsed = value
                    .parse::<usize>()
                    .map_err(|err| format!("invalid --limit value `{value}`: {err}"))?;
                options.limit = Some(parsed);
            }
            "--log" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--log requires a file path".to_string())?;
                options.log_path = Some(PathBuf::from(value));
            }
            "--verbose" => {
                options.verbose = true;
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
    println!("Run rules-tests fixtures against Mons game logic.");
    println!();
    println!("Usage:");
    println!("  cargo run --bin rules_tests -- [options]");
    println!("  ./scripts/run-rules-tests.sh [options]");
    println!();
    println!("Options:");
    println!("  --dir <path>    Fixture directory (default: {DEFAULT_RULES_DIR})");
    println!("  --limit <n>     Run only the first n fixtures");
    println!("  --log <path>    Also write output to a log file");
    println!("  --verbose       Print each passing case ID");
    println!("  --help, -h      Show this help message");
}

fn collect_fixture_paths(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    let entries = fs::read_dir(dir).map_err(|err| {
        format!(
            "failed to read fixture directory `{}`: {err}",
            dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read fixture entry: {err}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        // Ignore filesystem metadata files like `.DS_Store`.
        if name.starts_with('.') {
            continue;
        }

        paths.push(path);
    }

    paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(paths)
}

fn run_case(path: &Path) -> CaseResult {
    let id = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("<unknown>")
        .to_string();

    let raw = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(err) => {
            return CaseResult::fail(
                id,
                format!("failed to read fixture `{}`: {err}", path.display()),
                vec![],
            )
        }
    };

    let case = match RuleTestCase::from_json(raw.as_str()) {
        Ok(case) => case,
        Err(err) => {
            return CaseResult::fail(
                id,
                format!("invalid fixture JSON: {err}"),
                vec![format!("raw fixture: {raw}")],
            )
        }
    };
    let snapshot = snapshot_url(case.fen_before.as_str());

    let mut game = match MonsGame::from_fen(case.fen_before.as_str(), false) {
        Some(game) => game,
        None => {
            return CaseResult::fail(
                id,
                "invalid fenBefore".to_string(),
                vec![
                    format!("snapshot: {snapshot}"),
                    format!("inputFen: {}", case.input_fen),
                ],
            )
        }
    };

    let output = game.process_input(Input::array_from_fen(case.input_fen.as_str()), false, false);
    let actual_output_fen = output.fen();
    let actual_fen_after = game.fen();
    if actual_output_fen != case.output_fen {
        let mut details = vec![
            format!("snapshot: {snapshot}"),
            format!("inputFen: {}", case.input_fen),
            format!("expected outputFen: {}", case.output_fen),
            format!("actual outputFen:   {}", actual_output_fen),
        ];

        if actual_fen_after != case.fen_after {
            details.push(format!("expected fenAfter:  {}", case.fen_after));
            details.push(format!("actual fenAfter:    {}", actual_fen_after));
        }

        return CaseResult::fail(id, "outputFen mismatch".to_string(), details);
    }

    if actual_fen_after != case.fen_after {
        return CaseResult::fail(
            id,
            "fenAfter mismatch".to_string(),
            vec![
                format!("snapshot: {snapshot}"),
                format!("inputFen: {}", case.input_fen),
                format!("expected outputFen: {}", case.output_fen),
                format!("actual outputFen:   {}", actual_output_fen),
                format!("expected fenAfter:  {}", case.fen_after),
                format!("actual fenAfter:    {}", actual_fen_after),
            ],
        );
    }

    CaseResult::pass(id)
}

impl RuleTestCase {
    fn from_json(raw: &str) -> Result<Self, String> {
        Ok(Self {
            fen_before: extract_json_string_field(raw, "fenBefore")?,
            fen_after: extract_json_string_field(raw, "fenAfter")?,
            input_fen: extract_json_string_field(raw, "inputFen")?,
            output_fen: extract_json_string_field(raw, "outputFen")?,
        })
    }
}

fn extract_json_string_field(raw: &str, field: &str) -> Result<String, String> {
    let marker = format!("\"{field}\"");
    let marker_index = raw
        .find(marker.as_str())
        .ok_or_else(|| format!("missing `{field}` field"))?;

    let mut rest = &raw[marker_index + marker.len()..];
    rest = rest.trim_start();

    if !rest.starts_with(':') {
        return Err(format!("missing ':' after `{field}`"));
    }
    rest = rest[1..].trim_start();

    if !rest.starts_with('"') {
        return Err(format!("`{field}` must be a JSON string"));
    }

    parse_json_string(&rest[1..]).map_err(|err| format!("`{field}` parse error: {err}"))
}

fn parse_json_string(data: &str) -> Result<String, String> {
    let mut output = String::new();
    let chars: Vec<char> = data.chars().collect();
    let mut index = 0usize;

    while index < chars.len() {
        let ch = chars[index];
        if ch == '"' {
            return Ok(output);
        }

        if ch != '\\' {
            output.push(ch);
            index += 1;
            continue;
        }

        index += 1;
        if index >= chars.len() {
            return Err("incomplete escape sequence".to_string());
        }

        match chars[index] {
            '"' => output.push('"'),
            '\\' => output.push('\\'),
            '/' => output.push('/'),
            'b' => output.push('\u{0008}'),
            'f' => output.push('\u{000C}'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            'u' => {
                if index + 4 >= chars.len() {
                    return Err("incomplete unicode escape".to_string());
                }

                let codepoint_hex: String = chars[index + 1..=index + 4].iter().collect();
                let codepoint = u32::from_str_radix(codepoint_hex.as_str(), 16)
                    .map_err(|_| format!("invalid unicode escape `{codepoint_hex}`"))?;
                let decoded = char::from_u32(codepoint)
                    .ok_or_else(|| "invalid unicode scalar".to_string())?;
                output.push(decoded);
                index += 4;
            }
            value => {
                return Err(format!("unsupported escape sequence `\\{value}`"));
            }
        }
        index += 1;
    }

    Err("unterminated JSON string".to_string())
}

fn snapshot_url(fen: &str) -> String {
    let mut encoded = String::with_capacity(fen.len() * 3);

    for byte in fen.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(hex_digit(byte >> 4));
            encoded.push(hex_digit(byte & 0x0f));
        }
    }

    format!("https://mons.link/snapshot/{encoded}/")
}

fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'A' + nibble - 10) as char,
    }
}

fn log_failure(logger: &mut Logger, case_result: &CaseResult) -> io::Result<()> {
    logger.line(FAIL_SEPARATOR.to_string())?;
    logger.line(format!(
        "❌ [FAIL] {}: {}",
        case_result.id, case_result.summary
    ))?;
    for detail in &case_result.details {
        logger.line(detail.clone())?;
    }
    logger.line(FAIL_SEPARATOR.to_string())?;
    Ok(())
}

struct Logger {
    file: Option<BufWriter<File>>,
}

impl Logger {
    fn new(path: Option<&Path>) -> io::Result<Self> {
        let file = match path {
            Some(path) => Some(BufWriter::new(File::create(path)?)),
            None => None,
        };

        Ok(Self { file })
    }

    fn line(&mut self, line: String) -> io::Result<()> {
        println!("{line}");
        if let Some(file) = self.file.as_mut() {
            writeln!(file, "{line}")?;
        }
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(file) = self.file.as_mut() {
            file.flush()?;
        }
        Ok(())
    }
}
