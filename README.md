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

- `./scripts/run-automove-experiment.sh preflight <candidate>`
- `SMART_TRIAGE_SURFACE=<surface> ./scripts/run-automove-experiment.sh triage <candidate>`
- `./scripts/run-automove-experiment.sh fast-screen <candidate>`
- `./scripts/run-automove-experiment.sh progressive <candidate>`
- `./scripts/run-automove-experiment.sh ladder <candidate>`
- `SMART_TRIAGE_SURFACE=<opening_reply|primary_pro> ./scripts/run-automove-experiment.sh pro-triage <candidate>`
- `./scripts/run-automove-experiment.sh pro-fast-screen <candidate>`
- `./scripts/run-automove-experiment.sh pro-progressive <candidate>`
- `./scripts/run-automove-experiment.sh pro-ladder <candidate>`
- `./scripts/run-automove-experiment.sh pre-screen <candidate>`
- `./scripts/run-automove-experiment.sh pro-pre-screen <candidate>`
- `./scripts/clean-experiment-artifacts.sh`

Most candidates should die at `triage`, `pro-triage`, or `fast-screen`. `pre-screen` and `pro-pre-screen` remain available only as legacy noise diagnostics.

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
