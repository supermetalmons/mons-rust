# Smart Automove Experimentation Guide

This repo keeps automove experimentation isolated from release code.

## Release Safety

- Production automove logic stays in `/Users/ivan/Developer/mons/rust/src/models/mons_game_model.rs`.
- Experiment harness lives in `/Users/ivan/Developer/mons/rust/src/models/mons_game_model_automove_experiments.rs`.
- The harness is included only under `#[cfg(test)]` via:
  - `#[path = "mons_game_model_automove_experiments.rs"]`
  - `mod smart_automove_pool_tests;`
- Scoring presets live in `/Users/ivan/Developer/mons/rust/src/models/scoring.rs`; some are shared by both runtime and experiments.

Result: no experiment tournament code is compiled into release builds.

## Public Automove API

Client-facing smart automove now takes one argument:

- `smartAutomoveAsync("fast")`
- `smartAutomoveAsync("normal")`

Internally (current mapping):

- `fast` -> approximately old `depth=2`, `max_nodes=420`
- `normal` -> approximately old `depth=3`, `max_nodes=2300`
- `fast` currently uses drainer-context scoring tuned for short-horizon pickup/score pressure and immediate drainer threat checks.
- `normal` keeps the runtime tactical-balanced scoring path.
- On White turn 1, `smartAutomoveAsync(...)` tries a random hardcoded 5-move opening route (one-move-per-call). If current position no longer matches any route, it falls back to regular smart search.

This keeps client API stable around CPU preference while allowing internal search implementation changes.

## Client Modes Used in Tournaments

Production-mode tournament runs in this harness are executed across both client modes:

- `fast`
- `normal`

Note: experiment harness profiles call internal selectors (`smart_search_best_inputs`) directly, so the production opening-route policy above is not applied unless explicitly modeled. Set `SMART_USE_WHITE_OPENING_BOOK=true` to model those openings in experiments; keep it `false` for pure algorithm-promotion runs.

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
- `SMART_USE_WHITE_OPENING_BOOK` (`true/false`, default `false`; applies production white-turn opening routes in simulated games)
- Opening generation is cached per `(seed, game_count)` inside one test process, so profile sweeps and fast pipelines avoid rebuilding identical opening sets.

## Candidate Profiles

Choose candidate logic with `SMART_CANDIDATE_PROFILE` (default: `base`).
`base` is aligned to the currently shipped runtime behavior (same mode mapping and runtime scoring policy).
Examples:

- `base`
- `runtime_current` (explicit alias of currently shipped runtime behavior)
- `runtime_pre_drainer_context` (runtime snapshot before fast drainer-context scoring promotion)
- `runtime_legacy_phase_adaptive` (pre-wideroot runtime reference for regression checks)
- `runtime_d2_tuned` (legacy fixed-weight runtime reference)
- `runtime_fast_wideroot_normal_current` (fast-mode wideroot candidate with normal-mode runtime baseline)
- `runtime_drainer_context` (fast-mode drainer-context scoring candidate; normal-mode runtime baseline)
- `weights_balanced`
- `focus_light_tactical_d2_only`
- `phase_adaptive_d2`
- `phase_adaptive_scoring_v2`
- `hybrid_deeper_fast`

Run example:

- `SMART_CANDIDATE_PROFILE=focus_light_tactical_d2_only SMART_POOL_GAMES=100 cargo test --lib smart_automove_pool_candidate_promotion_with_client_budgets -- --ignored --nocapture`

## Profile Sweep and Speed Probe

- Compare multiple profiles on same setup:
  - `SMART_POOL_GAMES=4 SMART_SWEEP_PROFILES=base,focus_light_tactical_d2_only cargo test --lib smart_automove_pool_profile_sweep -- --ignored --nocapture`
- Speed-only probe on fixed openings:
  - `SMART_CANDIDATE_PROFILE=focus_light_tactical_d2_only SMART_SPEED_POSITIONS=20 cargo test --lib smart_automove_pool_profile_speed_probe -- --ignored --nocapture`
- Runtime/ply diagnostics:
  - `SMART_DIAG_GAMES=4 SMART_DIAG_MODE=normal cargo test --lib smart_automove_pool_runtime_diagnostics -- --ignored --nocapture`

## Fast Iteration Pipeline

Use one command to filter by speed first, then rank strength among surviving profiles:

- `SMART_FAST_PROFILES=base,runtime_d2_tuned,runtime_d2_tuned_d3_phase_adaptive SMART_FAST_GAMES=4 SMART_FAST_OPPONENTS=3 SMART_FAST_MAX_PLIES=96 SMART_FAST_SPEED_POSITIONS=8 SMART_FAST_USE_CLIENT_MODES=true cargo test --lib smart_automove_pool_fast_pipeline -- --ignored --nocapture`

Fast-pipeline knobs:

- `SMART_FAST_PROFILES` (comma-separated profile names)
- `SMART_FAST_BASELINE` (speed baseline profile; default `base`)
- `SMART_FAST_GAMES` (default `2`)
- `SMART_FAST_OPPONENTS` (default `2`)
- `SMART_FAST_MAX_PLIES` (default `80`; temporarily mapped to `SMART_POOL_MAX_PLIES` during the run)
- `SMART_FAST_SPEED_POSITIONS` (default `6`)
- `SMART_FAST_SPEED_RATIO_MAX` (default `1.25`; max allowed average ratio vs baseline)
- `SMART_FAST_SPEED_RATIO_MODE_MAX` (default follows `SMART_FAST_SPEED_RATIO_MAX`; max allowed per-mode ratio)
- `SMART_FAST_USE_CLIENT_MODES` (`true/false`; default `true`; accepts legacy `SMART_FAST_USE_CLIENT_BUDGETS` too)

## Direct Duel (Profile vs Profile)

Run a direct head-to-head between two profiles on the same opening set:

- `SMART_DUEL_A=phase_adaptive_scoring_v2 SMART_DUEL_B=runtime_d2_tuned SMART_DUEL_GAMES=4 SMART_DUEL_MAX_PLIES=96 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`

Duel knobs:

- `SMART_DUEL_A` (profile name; default `base`)
- `SMART_DUEL_B` (profile name; default `runtime_d2_tuned`)
- `SMART_DUEL_GAMES` (games per mode; default `4`)
- `SMART_DUEL_REPEATS` (repeat count with different deterministic seeds; default `1`)
- `SMART_DUEL_MAX_PLIES` (default `96`)
- `SMART_DUEL_USE_CLIENT_MODES` (`true/false`; default `true`)
- `SMART_DUEL_MODE` (`fast` or `normal`; when set, duel runs only that mode)

## Recommended Iteration Flow

1. CPU/strength screen vs current runtime:
   - `SMART_FAST_PROFILES=base,<candidate_profile> SMART_FAST_USE_CLIENT_MODES=true cargo test --lib smart_automove_pool_fast_pipeline -- --ignored --nocapture`
2. Direct duel vs shipped runtime (`runtime_current`) with repeats:
   - `SMART_DUEL_A=<candidate_profile> SMART_DUEL_B=runtime_current SMART_DUEL_REPEATS=2 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`
3. For release-to-release checks, compare shipped runtime vs legacy snapshot:
   - `SMART_DUEL_A=runtime_current SMART_DUEL_B=runtime_pre_drainer_context SMART_DUEL_REPEATS=3 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`
4. Only then run heavier pool tournament:
   - `SMART_CANDIDATE_PROFILE=<candidate_profile> SMART_POOL_GAMES=100 cargo test --lib smart_automove_pool_candidate_promotion_with_client_budgets -- --ignored --nocapture`

This keeps iterations focused on meaningful non-regression checks: same client modes, same runtime baseline, and explicit CPU gating per mode.

## Promotion Rule (Current)

A candidate is marked promoted when all are true:

- Beats at least `MIN_OPPONENTS_BEAT_TO_PROMOTE` opponents (currently `7`).
- Per-opponent confidence for beaten matchups is at least `MIN_CONFIDENCE_TO_PROMOTE` (currently `0.75`).
- Per-mode aggregate confidence (for both client modes) is at least `0.75`.
- Combined aggregate confidence is at least `0.75`.

The report prints:

- Whether promoted
- Aggregate win rate/confidence
- Per-mode and per-opponent breakdown
- Which pool model would be removed

## Implementation Workflow

1. Add or modify a candidate implementation function in:
   - `/Users/ivan/Developer/mons/rust/src/models/mons_game_model_automove_experiments.rs`
2. Register it in `candidate_model()` and optionally in `smart_automove_pool_profile_sweep` variants.
3. Run smoke test and fast iteration pipeline first.
4. Run direct duel with repeats against `runtime_current`.
5. Run full promotion tournament only after passing speed + duel checks.
6. If promoted with acceptable speed, promote logic into runtime path in:
   - `/Users/ivan/Developer/mons/rust/src/models/mons_game_model.rs`
7. Optionally replace one of `pool_model_01..pool_model_10` with the newly promoted behavior to keep a diverse baseline pool.

## Important

- Do not run/scan `rules-tests/` during automove iteration unless explicitly requested.
- Keep runtime validation focused on automove-targeted tests and `cargo check`.
