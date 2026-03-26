# mons-rust
`cargo add mons-rust`

or

`npm install mons-rust`

## automove

Canonical automove workflow:

- runbook: `HOW_TO_ITERATE_ON_AUTOMOVE.md`
- active backlog: `AUTOMOVE_IDEAS.md`
- durable lessons: `docs/automove-knowledge.md`
- archive: `docs/automove-archive.md`
- compatibility pointer: `docs/automove-experiments.md`

Session-start commands:

- `./scripts/run-automove-experiment.sh guardrails <candidate>`
- `SMART_TRIAGE_SURFACE=<surface> ./scripts/run-automove-experiment.sh triage <candidate>`
- `SMART_TRIAGE_SURFACE=<opening_reply|primary_pro> ./scripts/run-automove-experiment.sh pro-triage <candidate>`
- `./scripts/run-automove-experiment.sh runtime-preflight <candidate>`
- `SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh fast-screen <candidate>`
- `./scripts/run-automove-experiment.sh pro-fast-screen <candidate>`
- `./scripts/clean-experiment-artifacts.sh --dry-run`

Default artifact layout:

- logs: `target/experiment-runs/<candidate>/`
- workflow-only logs: `target/experiment-runs/misc/`
- runtime-preflight stamps: `target/experiment-stamps/`

Compatibility notes:

- `preflight`, `pre-screen`, and `pro-pre-screen` still exist, but they are legacy diagnostics.
- `audit-screen` and `pro-audit-screen` are spot checks for clean triage rejects, not promotion evidence.
- Unless a note says otherwise, new candidates should branch from `runtime_current`.
- Shipping runtime note: the package release ships `runtime_current`; `runtime_pro_turn_engine_v30` is the retained Pro frontier for offline experiments, and `runtime_pro_turn_engine_v1` remains reference-only history.

## rules-tests runner

Fixtures are stored as chunk archives in `rules-tests-chunks/` (default: `100000` fixtures per chunk).

Run all fixtures:

`./scripts/run-rules-tests.sh`

Useful options:

`./scripts/run-rules-tests.sh --limit 100`

`./scripts/run-rules-tests.sh --log /tmp/rules-tests.log`

`./scripts/run-rules-tests.sh --chunks-dir ./rules-tests-chunks --verbose`

## rules-tests generator

Generate new random unique fixtures and repack chunks:

`./scripts/generate-rules-tests.sh --target-new 100`

Generate directly into a directory (continuous mode):

`./scripts/generate-rules-tests.sh --dir /tmp/rules-tests-work`

Pack a directory back into chunks:

`./scripts/pack-rules-tests.sh --dir /tmp/rules-tests-work --chunks-dir ./rules-tests-chunks --chunk-size 100000`

## publishing to npm

`./publish.sh`

Release checklist:

- Review `git status` before publish and confirm only intentional committed changes are present.
- Confirm `runtime_current` is still the shipping automove path.
- Confirm retained Pro experiment frontiers (`runtime_pro_turn_engine_v30` and newer candidate-only follow-ups) remain fenced off from production.
- Run `cargo test`.
- Run `cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture`.
- Run `cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture`.
- Commit valuable changes before version bump / publish.
- Clean disposable experiment artifacts after validation with `./scripts/clean-experiment-artifacts.sh`.

Production blockers:

- build/test failures
- release speed gate failures
- any regression that enables turn-engine in shipping `runtime_current`

Non-blocking retained experiment state:

- retained Pro frontier profiles (`runtime_pro_turn_engine_v30`, candidate-only follow-ups, and `runtime_pro_turn_engine_v1` as reference history) plus ignored probes
- experiment workflow/logging helpers
- compressed automove backlog / knowledge / archive docs
