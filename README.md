# mons-rust
`cargo add mons-rust`

or

`npm install mons-rust`

## automove

Canonical automove workflow:

- runbook: `HOW_TO_ITERATE_ON_AUTOMOVE.md`
- live board: `AUTOMOVE_IDEAS.md`
- durable lessons: `docs/automove-knowledge.md`
- archive: `docs/automove-archive.md`

Active experiment stages:

- `guardrails`
- `pro-triage`
- `runtime-preflight`
- `pro-reliability`
- `pro-reliability-confirm`

Canonical commands:

- `./scripts/run-automove-experiment.sh guardrails runtime_pro_turn_engine_v30`
- `SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage runtime_pro_turn_engine_v30`
- `./scripts/run-automove-experiment.sh runtime-preflight runtime_pro_turn_engine_v30`
- `./scripts/run-automove-experiment.sh pro-reliability runtime_pro_turn_engine_v30`
- `./scripts/run-automove-experiment.sh pro-reliability-confirm runtime_pro_turn_engine_v30`
- `./scripts/clean-experiment-artifacts.sh --dry-run`

Active retained profile surface:

- `runtime_current`
- `runtime_pro_turn_engine_v30`

Notes:

- Shipping runtime is `runtime_current`.
- `runtime_pro_turn_engine_v30` is the only retained Pro frontier for offline experiments.
- Archive profiles, including `runtime_pro_turn_engine_v1`, are not valid active experiment targets.
- Post-promotion maintenance runs may show `pro-triage` `0/0` for `runtime_pro_turn_engine_v30` vs `runtime_current`; that is the expected stable-equivalence result, not a failed challenger attempt.

Default artifact layout:

- logs: `target/experiment-runs/<candidate>/`
- workflow-only logs: `target/experiment-runs/misc/`
- runtime-preflight stamps: `target/experiment-stamps/`

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

## repo cleanup

`./repo-clean.sh`

- switches back to a kept branch (`main`, `master`, or `keep/*`) before deleting disposable branches
- removes non-primary worktrees, clears stashes, deletes non-kept local branches, and prunes non-kept remote branches
- use `./repo-clean.sh --local-only` to skip remote branch deletion
- use `keep/<name>` for any branch you want to protect from cleanup

## publishing to npm

`./publish.sh`

Release checklist:

- Review `git status` before publish and confirm only intentional committed changes are present.
- Confirm `runtime_current` is still the shipping automove path.
- Confirm `runtime_pro_turn_engine_v30` remains fenced off as an offline experiment frontier.
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

- the retained `runtime_pro_turn_engine_v30` frontier plus ignored probes
- experiment workflow/logging helpers
- compressed automove backlog / knowledge / archive docs
