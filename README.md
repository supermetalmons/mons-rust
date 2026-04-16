# mons-rust
`cargo add mons-rust`

or

`npm install mons-rust`

## automove

Docs:

- runbook: `HOW_TO_ITERATE_ON_AUTOMOVE.md`
- live board: `AUTOMOVE_IDEAS.md`
- durable lessons: `docs/automove-knowledge.md`
- archive: `docs/automove-archive.md`

Public Pro runtime currently routes through `frontier_pro_v2_guarded`; `shipping_pro_search` remains the retained search-only baseline profile.

Quickstart:

- canonical Pro loop: `./scripts/run-automove-canonical-loop.sh frontier_pro_v2_guarded`
- larger confirmation pass: `./scripts/run-automove-canonical-loop.sh --confirm frontier_pro_v2_guarded`
- single-stage or diagnostic run: `./scripts/run-automove-experiment.sh <stage> frontier_pro_v2_guarded`
- cleanup preview: `./scripts/clean-experiment-artifacts.sh --dry-run`

Retained profile surface:

- `shipping_pro_search`
- `frontier_pro_v2_guarded`

Glossary:

- `shipping`: the deployed Pro path, currently `frontier_pro_v2_guarded`
- `baseline`: the retained search-only comparison profile, currently `shipping_pro_search`
- `frontier`: the guarded ProV2 selector/runtime line, currently `frontier_pro_v2_guarded`
- `probe`: forced turn-engine diagnostics that inspect search/acceptance behavior without changing shipping

Artifact layout:

- selected-profile logs: `target/experiment-runs/<profile>/`
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
- Confirm public Pro still routes through `frontier_pro_v2_guarded`.
- Confirm `shipping_pro_search` remains available as the retained search-only baseline profile.
- Run `cargo test`.
- Run `cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture`.
- Run `cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture`.
- Commit valuable changes before version bump / publish.
- Clean disposable experiment artifacts after validation with `./scripts/clean-experiment-artifacts.sh`.

Production blockers:

- build/test failures
- release speed gate failures
- any regression in the deployed `frontier_pro_v2_guarded` Pro path

Non-blocking retained experiment state:

- the retained `shipping_pro_search` baseline, the retained `frontier_pro_v2_guarded` frontier, and ignored probes
- experiment workflow/logging helpers
- compressed automove backlog / knowledge / archive docs
