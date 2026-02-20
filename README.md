# mons-rust
`cargo add mons-rust`

or

`npm install mons-rust`

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
