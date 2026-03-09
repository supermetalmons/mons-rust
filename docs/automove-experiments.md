# Smart Automove Experiments

This file is a short compatibility entrypoint. The canonical workflow now lives in [`../HOW_TO_ITERATE_ON_AUTOMOVE.md`](../HOW_TO_ITERATE_ON_AUTOMOVE.md).

## Start Here

1. Read [`../HOW_TO_ITERATE_ON_AUTOMOVE.md`](../HOW_TO_ITERATE_ON_AUTOMOVE.md).
2. Pick or add the next item in [`../AUTOMOVE_IDEAS.md`](../AUTOMOVE_IDEAS.md).
3. Use `./scripts/run-automove-experiment.sh` for the earned screen loop. Most candidates should die at `pre-screen`, `pro-pre-screen`, or `fast-screen`.
4. Promote durable conclusions into [`automove-knowledge.md`](automove-knowledge.md) or [`automove-archive.md`](automove-archive.md) before cleaning artifacts.

## Key References

- Active experiment registry: `src/models/automove_experiments/profiles.rs`
- Promotion gates and diagnostics: `src/models/automove_experiments/tests.rs`
- Harness and artifact handling: `src/models/automove_experiments/harness.rs`
- Production runtime promotion point: `src/models/mons_game_model.rs`
- Raw pro strategy notes: [`automove-pro-strategy-interview.md`](automove-pro-strategy-interview.md)

## Main Commands

```sh
# fast or normal candidate
./scripts/run-automove-experiment.sh preflight <candidate>
./scripts/run-automove-experiment.sh pre-screen <candidate>
./scripts/run-automove-experiment.sh fast-screen <candidate>
./scripts/run-automove-experiment.sh progressive <candidate>
./scripts/run-automove-experiment.sh ladder <candidate>

# pro candidate
./scripts/run-automove-experiment.sh preflight <candidate>
./scripts/run-automove-experiment.sh pro-pre-screen <candidate>
./scripts/run-automove-experiment.sh pro-fast-screen <candidate>
./scripts/run-automove-experiment.sh pro-progressive <candidate>
./scripts/run-automove-experiment.sh pro-ladder <candidate>

./scripts/clean-experiment-artifacts.sh
```
