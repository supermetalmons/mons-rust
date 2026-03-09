# mons-rust
`cargo add mons-rust`

or

`npm install mons-rust`

## automove

The active automove workflow lives here:

- canonical workflow: `HOW_TO_ITERATE_ON_AUTOMOVE.md`
- ideas backlog: `AUTOMOVE_IDEAS.md`
- compatibility entrypoint: `docs/automove-experiments.md`
- durable lessons: `docs/automove-knowledge.md`
- retired profile archive: `docs/automove-archive.md`

Useful scripts:

- `./scripts/run-automove-experiment.sh guardrails <candidate>`
- `./scripts/run-automove-experiment.sh runtime-preflight <candidate>`
- `./scripts/run-automove-experiment.sh triage-calibrate [reply_risk|opponent_mana|supermana|all]`
- `./scripts/run-automove-experiment.sh preflight <candidate>`
- `SMART_TRIAGE_SURFACE=<surface> ./scripts/run-automove-experiment.sh triage <candidate>`
- `SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh audit-screen <candidate>`
- `SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh fast-screen <candidate>`
- `SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh progressive <candidate>`
- `SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh ladder <candidate>`
- `SMART_TRIAGE_SURFACE=<opening_reply|primary_pro> ./scripts/run-automove-experiment.sh pro-triage <candidate>`
- `./scripts/run-automove-experiment.sh pro-audit-screen <candidate>`
- `./scripts/run-automove-experiment.sh pro-fast-screen <candidate>`
- `./scripts/run-automove-experiment.sh pro-progressive <candidate>`
- `./scripts/run-automove-experiment.sh pro-ladder <candidate>`
- `./scripts/run-automove-experiment.sh pre-screen <candidate>`
- `./scripts/run-automove-experiment.sh pro-pre-screen <candidate>`
- `./scripts/clean-experiment-artifacts.sh`

Run `triage-calibrate` before candidate work on `reply_risk`, `opponent_mana`, or `supermana`. Then use `guardrails -> triage -> runtime-preflight` so weak ideas die before the expensive CPU gate. `preflight` still exists as the old all-in-one wrapper. Most candidates should die at `triage`, `pro-triage`, or the first earned duel stage. For mode-specific `fast` or `normal` ideas, run duel stages with `SMART_PROMOTION_TARGET_MODE=<fast|normal>` so the target mode is the improvement bar and the other client mode is only a non-regression check. Use `audit-screen` or `pro-audit-screen` only as occasional spot checks for clean triage rejects. `pre-screen` and `pro-pre-screen` remain available only as legacy noise diagnostics. Unless documented otherwise, new candidates should be deltas on `runtime_current`.

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
