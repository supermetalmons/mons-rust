# How To Iterate On Automove

This is the canonical long-term workflow for automove iteration.

The goal is not to collect experiments. The goal is to make automove stronger and get the stronger version promoted into the production runtime safely. A successful loop ends with either a promotable candidate or a compressed lesson that makes the next promotion attempt better.

## Quick Reference

1. Pick one target mode to improve first: usually `pro`, `normal`, or `fast`.
2. Keep one live candidate in the experiment registry.
3. Run the promotion loop:

```sh
./scripts/run-automove-experiment.sh preflight <candidate>
./scripts/run-automove-experiment.sh fast-screen <candidate>
./scripts/run-automove-experiment.sh progressive <candidate>
./scripts/run-automove-experiment.sh ladder <candidate>
```

4. Before any runtime promotion decision, run the release speed gates:

```sh
cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture
cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture
```

5. If the candidate is not promotable, archive the lesson and return to the next promotion attempt.
6. If the candidate is promotable, update the production runtime and commit the promotion immediately.

All experiment harness runs use ignored `#[cfg(test)]` tests:

```sh
cargo test --release --lib <test_name> -- --ignored --nocapture
```

## Files That Matter

- Active experiment registry: `src/models/automove_experiments/profiles.rs`
- Promotion gates and comparison tests: `src/models/automove_experiments/tests.rs`
- Experiment harness and artifacts: `src/models/automove_experiments/harness.rs`
- Production runtime that promotion must change: `src/models/mons_game_model.rs`
- Wrapper script for the main loop: `scripts/run-automove-experiment.sh`
- Artifact cleanup script: `scripts/clean-experiment-artifacts.sh`
- Working backlog for future loops: `AUTOMOVE_IDEAS.md`
- Durable lessons: `docs/automove-knowledge.md`
- Archived waves and retired context: `docs/automove-archive.md`
- Raw pro strategy notes: `docs/automove-pro-strategy-interview.md`

Promotion means changing the production runtime in `src/models/mons_game_model.rs`, not just leaving a stronger candidate in the experiment registry.

## Core Rules

- Optimize for promotion, not for experiment volume.
- Improve one mode at a time and verify the other modes do not regress.
- Prefer a focused sequence such as `pro` then `normal` then `fast`, or the reverse, instead of trying to improve all modes at once.
- Keep `main` as the last clean promotion checkpoint and do active exploration on a `codex/*` branch.
- Keep the active registry small. One live candidate at a time is easier to reason about than many overlapping waves.
- Do not use archived profiles for new promotion decisions unless there is a new hypothesis that cannot be tested on the active surface.
- It is fine to explore, diagnose, and run side experiments, but every loop should return to the next concrete promotion attempt.

## Safety And CPU Budget

The strongest candidate is only valuable if it is safe to ship.

- Stay within the release-safe CPU budget for the target mode.
- Do not promote anything that risks endless exploration, stalled turns, or getting stuck in unusual game states.
- Keep exact-lite and other expensive logic explicitly bounded.
- Treat opening black-reply latency as a hard release concern, not a cosmetic one.
- If strength depends on unsafe runtime behavior, it is not promotable yet.
- When in doubt, keep runtime behavior predictable and bounded first, then recover strength with cheaper signals or more selective exactness.

## Recommended Iteration Loop

1. Choose one target mode and one hypothesis.
2. Pull or add the next idea in `AUTOMOVE_IDEAS.md`.
3. Implement one live candidate in `src/models/automove_experiments/profiles.rs`.
4. Run `preflight` once.
5. Run `fast-screen`, then `progressive`, then `ladder`.
6. If needed, run focused diagnostics or mode-vs-mode checks to understand the result.
7. If the candidate is promising, run the release speed gates.
8. Promote if it is stronger and safe. Otherwise compress the lesson into docs, clean stale artifacts, and move to the next attempt.

A good loop often spreads findings across modes. For example: learn something in `pro`, then apply the safe subset to `normal`, then decide whether a cheaper version is worth trying in `fast`.

## Recording, Cleanup, And Commits

- Commit meaningful checkpoints once in a while during exploration.
- Do not let useful conclusions remain only in `target/experiment-runs` logs.
- Promote durable conclusions into `docs/automove-knowledge.md` or `docs/automove-archive.md` before cleaning artifacts.
- Clean obsolete local artifacts once in a while with:

```sh
./scripts/clean-experiment-artifacts.sh
./scripts/clean-experiment-artifacts.sh --dry-run
```

- Update `AUTOMOVE_IDEAS.md` as ideas are added, tried, promoted, or retired.
- When promotion is achieved, commit the runtime change, the supporting docs updates, and the cleaned experiment surface together.

## When The Backlog Runs Dry

If every current item in `AUTOMOVE_IDEAS.md` has already been tried, generate new ideas, add them to the backlog, and continue iterating. The workflow does not end when the current list is exhausted.
