# Smart Automove Experiments

This file is a short compatibility entrypoint. The canonical workflow now lives in [`../HOW_TO_ITERATE_ON_AUTOMOVE.md`](../HOW_TO_ITERATE_ON_AUTOMOVE.md).

## Start Here

1. Read [`../HOW_TO_ITERATE_ON_AUTOMOVE.md`](../HOW_TO_ITERATE_ON_AUTOMOVE.md).
2. Pick or add the next item in [`../AUTOMOVE_IDEAS.md`](../AUTOMOVE_IDEAS.md).
3. Use `./scripts/run-automove-experiment.sh` for the earned loop. Most candidates should die at `triage`, `pro-triage`, or `fast-screen`.
4. Treat `pre-screen` and `pro-pre-screen` as legacy diagnostics only.
5. Promote durable conclusions into [`automove-knowledge.md`](automove-knowledge.md) or [`automove-archive.md`](automove-archive.md) before cleaning artifacts.

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
SMART_TRIAGE_SURFACE=<reply_risk|supermana|opponent_mana|spirit_setup|drainer_safety|cache_reuse> \
  ./scripts/run-automove-experiment.sh triage <candidate>
./scripts/run-automove-experiment.sh fast-screen <candidate>
./scripts/run-automove-experiment.sh progressive <candidate>
./scripts/run-automove-experiment.sh ladder <candidate>

# pro candidate
./scripts/run-automove-experiment.sh preflight <candidate>
SMART_TRIAGE_SURFACE=<opening_reply|primary_pro> \
  ./scripts/run-automove-experiment.sh pro-triage <candidate>
./scripts/run-automove-experiment.sh pro-fast-screen <candidate>
./scripts/run-automove-experiment.sh pro-progressive <candidate>
./scripts/run-automove-experiment.sh pro-ladder <candidate>

# optional legacy diagnostics
./scripts/run-automove-experiment.sh pre-screen <candidate>
./scripts/run-automove-experiment.sh pro-pre-screen <candidate>

./scripts/clean-experiment-artifacts.sh
```
