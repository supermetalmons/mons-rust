# Smart Automove Experimentation Guide

This repo keeps automove experimentation isolated from release code.

## Release Safety

- Production automove logic stays in `/Users/ivan/Developer/mons/rust/src/models/mons_game_model.rs`.
- Experiment harness lives in `/Users/ivan/Developer/mons/rust/src/models/mons_game_model_automove_experiments.rs`.
- The harness is included only under `#[cfg(test)]` via:
  - `#[path = "mons_game_model_automove_experiments.rs"]`
  - `mod smart_automove_pool_tests;`
- Experiment-only scoring presets are `#[cfg(test)]` in `/Users/ivan/Developer/mons/rust/src/models/scoring.rs`.

Result: no experiment tournament code is compiled into release builds.

## Client Budgets Used in Tournaments

These are hard-coded as `CLIENT_BUDGETS` in the harness and match production usage:

- `depth=2`, `max_nodes=420`
- `depth=3`, `max_nodes=2300`

## Core Tests

From `/Users/ivan/Developer/mons/rust`:

- Fast sanity:
  - `cargo test --lib smart_automove_pool_smoke_runs`
- Verify pool size:
  - `cargo test --lib smart_automove_pool_keeps_ten_models`
- Full candidate-vs-pool promotion tournament:
  - `SMART_POOL_GAMES=100 cargo test --lib smart_automove_pool_candidate_promotion_with_client_budgets -- --ignored --nocapture`

Useful knobs:

- `SMART_POOL_GAMES` (default `100` in promotion test)
- `SMART_POOL_OPPONENTS` (defaults to 10; clamps to 1..10)
- `SMART_POOL_MAX_PLIES` (default `320`)
- `SMART_CANDIDATE_PROFILE` (selects candidate implementation)

## Candidate Profiles

Choose candidate logic with `SMART_CANDIDATE_PROFILE` (default: `base`).
Examples:

- `base`
- `weights_balanced`
- `focus_light_tactical_d2_only`
- `runtime_d2_tuned`
- `hybrid_deeper_fast`

Run example:

- `SMART_CANDIDATE_PROFILE=focus_light_tactical_d2_only SMART_POOL_GAMES=100 cargo test --lib smart_automove_pool_candidate_promotion_with_client_budgets -- --ignored --nocapture`

## Profile Sweep and Speed Probe

- Compare multiple profiles on same setup:
  - `SMART_POOL_GAMES=4 SMART_SWEEP_PROFILES=base,focus_light_tactical_d2_only cargo test --lib smart_automove_pool_profile_sweep -- --ignored --nocapture`
- Speed-only probe on fixed openings:
  - `SMART_CANDIDATE_PROFILE=focus_light_tactical_d2_only SMART_SPEED_POSITIONS=20 cargo test --lib smart_automove_pool_profile_speed_probe -- --ignored --nocapture`
- Runtime/ply diagnostics:
  - `SMART_DIAG_GAMES=4 SMART_DIAG_DEPTH=3 SMART_DIAG_NODES=2300 cargo test --lib smart_automove_pool_runtime_diagnostics -- --ignored --nocapture`

## Promotion Rule (Current)

A candidate is marked promoted when all are true:

- Beats at least `MIN_OPPONENTS_BEAT_TO_PROMOTE` opponents (currently `7`).
- Per-opponent confidence for beaten matchups is at least `MIN_CONFIDENCE_TO_PROMOTE` (currently `0.75`).
- Per-budget aggregate confidence (for both client budgets) is at least `0.75`.
- Combined aggregate confidence is at least `0.75`.

The report prints:

- Whether promoted
- Aggregate win rate/confidence
- Per-budget and per-opponent breakdown
- Which pool model would be removed

## Iteration Workflow

1. Add or modify a candidate implementation function in:
   - `/Users/ivan/Developer/mons/rust/src/models/mons_game_model_automove_experiments.rs`
2. Register it in `candidate_model()` and optionally in `smart_automove_pool_profile_sweep` variants.
3. Run smoke test.
4. Run full promotion tournament with `SMART_POOL_GAMES=100`.
5. If promoted with acceptable speed, promote logic into runtime path in:
   - `/Users/ivan/Developer/mons/rust/src/models/mons_game_model.rs`
6. Optionally replace one of `pool_model_01..pool_model_10` with the newly promoted behavior to keep a diverse baseline pool.

## Important

- Do not run/scan `rules-tests/` during automove iteration unless explicitly requested.
- Keep runtime validation focused on automove-targeted tests and `cargo check`.
