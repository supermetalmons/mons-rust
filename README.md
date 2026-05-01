# mons-rust

`cargo add mons-rust`

or

`npm install mons-rust`

## Automove

Docs:

- runbook: `HOW_TO_ITERATE_ON_AUTOMOVE.md`
- live board: `AUTOMOVE_IDEAS.md`
- structural review: `docs/automove-structural-review.md`
- durable rules: `docs/automove-knowledge.md`
- archive: `docs/automove-archive.md`

Live surface:

- retained profiles: `shipping_pro_search`, `frontier_pro_v2_guarded`
- canonical stages: `guardrails`, `pro-triage`, `runtime-preflight`, `pro-reliability`, `pro-reliability-confirm`

Quickstart:

- `./scripts/run-automove-canonical-loop.sh frontier_pro_v2_guarded`
- `./scripts/run-automove-canonical-loop.sh --confirm frontier_pro_v2_guarded`
- `./scripts/run-automove-experiment.sh <stage> frontier_pro_v2_guarded`
- `./scripts/run-automove-experiment.sh pro-profile-sweep frontier_pro_v2_raw`
- `./scripts/check-automove-hygiene.sh`
- `./scripts/clean-experiment-artifacts.sh --dry-run`
- `./scripts/clean-experiment-artifacts.sh --dry-run --all-target`

Artifacts:

- selected-profile logs: `target/experiment-runs/<profile>/`
- workflow-only logs: `target/experiment-runs/misc/`
- runtime-preflight stamps: `target/experiment-stamps/`
- full local build/artifact cache: `target/` via `--all-target`

## Rules Tests

Runner:

- `./scripts/run-rules-tests.sh`
- `./scripts/run-rules-tests.sh --limit 100`
- `./scripts/run-rules-tests.sh --log /tmp/rules-tests.log`

Generator:

- `./scripts/generate-rules-tests.sh --target-new 100`
- `./scripts/generate-rules-tests.sh --dir /tmp/rules-tests-work`
- `./scripts/pack-rules-tests.sh --dir /tmp/rules-tests-work --chunks-dir ./rules-tests-chunks --chunk-size 100000`

## Repo Cleanup

- `./repo-clean.sh`
- `./repo-clean.sh --local-only`

Use `keep/<name>` for any branch that should survive repo cleanup.

## Publishing

- `./publish.sh`
- Confirm public Pro still routes through `frontier_pro_v2_guarded`.
- Confirm `shipping_pro_search` remains available as the retained baseline.
- Run `cargo test`.
- Run `cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture`.
- Run `./scripts/assert-release-package-surface.sh pkg/web pkg/node` after the Wasm packages are built.
- Clean disposable experiment artifacts after validation with `./scripts/clean-experiment-artifacts.sh`.
