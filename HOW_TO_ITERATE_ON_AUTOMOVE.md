# How To Iterate On Automove

This is the canonical operator playbook for automove work.

Goal: produce one stronger, promotable change at a time. If a candidate is not promotable, compress the lesson, clean the artifacts, and move on.

## Quick Reference

1. Pick one target mode: `pro`, `normal`, or `fast`.
2. Pick exactly one active idea from `AUTOMOVE_IDEAS.md`.
3. Run the earned path in order.
4. Kill flat or noisy candidates early.
5. Record the outcome before cleaning artifacts.

## Earned Paths

### Fast / Normal

```sh
./scripts/run-automove-experiment.sh guardrails <candidate>
SMART_TRIAGE_SURFACE=<surface> ./scripts/run-automove-experiment.sh triage <candidate>
./scripts/run-automove-experiment.sh runtime-preflight <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh fast-screen <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh progressive <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh ladder <candidate>
```

### Pro

```sh
./scripts/run-automove-experiment.sh guardrails <candidate>
./scripts/run-automove-experiment.sh pro-opening-speed-probe <candidate>   # optional for opening_reply ideas
SMART_TRIAGE_SURFACE=<opening_reply|primary_pro> ./scripts/run-automove-experiment.sh pro-triage <candidate>
./scripts/run-automove-experiment.sh runtime-preflight <candidate>
./scripts/run-automove-experiment.sh pro-reliability <candidate>
./scripts/run-automove-experiment.sh pro-fast-screen <candidate>
./scripts/run-automove-experiment.sh pro-progressive <candidate>
./scripts/run-automove-experiment.sh pro-ladder <candidate>
```

### Promotion-Time Only

```sh
cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture
cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture
```

## Session Start Checklist

- Pick one target mode, one idea, and one candidate profile.
- Default the baseline to `runtime_current` for new work unless the idea explicitly needs another reference.
- For `reply_risk`, `opponent_mana`, and `supermana`, run `triage-calibrate` before candidate work if the surface is not already known-good.
- For Pro `opening_reply` ideas, decide whether the optional `pro-opening-speed-probe` should run before `pro-triage`.
- Confirm the first earned duel stage before budgeting progressive or ladder time.

## Artifact Layout

- Candidate logs default to `target/experiment-runs/<candidate>/`.
- Calibration and workflow-only runs default to `target/experiment-runs/misc/`.
- Runtime-preflight stamps live in `target/experiment-stamps/`.
- Use the cleaner intentionally:

```sh
./scripts/clean-experiment-artifacts.sh --dry-run
./scripts/clean-experiment-artifacts.sh --candidate <candidate> --logs-only --dry-run
./scripts/clean-experiment-artifacts.sh --candidate <candidate> --stamps-only --dry-run
./scripts/clean-experiment-artifacts.sh
```

- Logs and stamps are disposable evidence, not durable memory.

## Release Checklist

- Review `git status` before publish.
- Confirm `runtime_current` is still the shipping automove path.
- Confirm `runtime_pro_turn_engine_v30` is still the retained ProV2 frontier and not wired into production runtime.
- Confirm `runtime_pro_turn_engine_v1` remains reference-only retained history, not the live frontier.
- Run:

```sh
cargo test
cargo test --release --lib --no-run
cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture
cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture
```

- Commit valuable changes before version bump / publish.
- Clean disposable experiment artifacts after validation.

Production blockers:

- build/test failures
- release speed gate failures
- any regression that enables turn-engine in shipping `runtime_current`

Retained but non-blocking experiment state:

- `runtime_pro_turn_engine_v30` code, probes, and fixtures
- `runtime_pro_turn_engine_v1` reference-only code and shared regression coverage
- ignored experiment-only tests and scripts
- backlog, archive, and durable knowledge updates

## Stop Conditions

- `guardrails` fail: kill the candidate.
- `triage` or `pro-triage` fail: kill the candidate, unless you deliberately spend one audit spot check.
- `runtime-preflight` fail: kill the candidate.
- First earned duel stage is flat or negative: kill the candidate.
- Progressive fades, stalls, or hits a runtime cliff: kill the candidate.
- No clear story after one focused diagnostic split: kill the candidate and split the idea differently.

## Compatibility Surface

These still exist, but they are not the default workflow:

- `preflight`: legacy combined wrapper.
- `pre-screen` and `pro-pre-screen`: legacy reject-only diagnostics.
- `audit-screen` and `pro-audit-screen`: occasional spot checks for clean triage rejects.
- `docs/automove-experiments.md`: compatibility pointer only.

Do not treat any of the compatibility paths as promotion evidence unless the canonical earned path also passes.

## Session End Checklist

1. Record the result in `AUTOMOVE_IDEAS.md`.
2. Move durable lessons into `docs/automove-knowledge.md`.
3. Move wave history or retired branch context into `docs/automove-archive.md`.
4. Clean logs and stamps intentionally.
5. Start the next session with one fresh idea and one fresh candidate, not with raw artifact spelunking.

## Files That Matter

- `src/models/mons_game_model.rs`
- `src/models/automove_turn_engine.rs`
- `src/models/automove_turn_planner.rs`
- `src/models/automove_experiments/profiles.rs`
- `src/models/automove_experiments/tests.rs`
- `src/models/automove_experiments/harness.rs`
- `scripts/run-automove-experiment.sh`
- `scripts/run-experiment-logged.sh`
- `scripts/clean-experiment-artifacts.sh`
- `AUTOMOVE_IDEAS.md`
- `docs/automove-knowledge.md`
- `docs/automove-archive.md`
