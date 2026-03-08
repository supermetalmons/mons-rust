# mons-rust
`cargo add mons-rust`

or

`npm install mons-rust`

## automove

The active automove workflow lives here:

- runbook: `docs/automove-experiments.md`
- durable lessons: `docs/automove-knowledge.md`
- retired profile archive: `docs/automove-archive.md`

Useful scripts:

- `./scripts/run-automove-experiment.sh preflight <candidate>`
- `./scripts/run-automove-experiment.sh fast-screen <candidate>`
- `./scripts/run-automove-experiment.sh progressive <candidate>`
- `./scripts/run-automove-experiment.sh ladder <candidate>`
- `./scripts/clean-experiment-artifacts.sh`

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
