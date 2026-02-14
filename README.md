# mons-rust
`cargo add mons-rust`

or

`npm install mons-rust`

## rules-tests runner

Run all fixtures in `rules-tests`:

`./scripts/run-rules-tests.sh`

Useful options:

`./scripts/run-rules-tests.sh --limit 100`

`./scripts/run-rules-tests.sh --log /tmp/rules-tests.log`

## rules-tests generator

Generate new random unique fixtures in `rules-tests`:

`./scripts/generate-rules-tests.sh --target-new 100`

Run continuously until interrupted:

`./scripts/generate-rules-tests.sh`

## publishing to npm

`./publish.sh`
