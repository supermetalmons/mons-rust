# Smart Automove Experimentation Guide

This document is the entry point for iterating on automove strength safely and quickly.

## What Is Shipped Right Now

Public API:

- `smartAutomoveAsync("fast")`
- `smartAutomoveAsync("normal")`

Current runtime behavior:

- `fast` is CPU-shaped around old `depth=2/max_nodes=420` and uses drainer-context scoring.
- `normal` is CPU-shaped around old `depth=3/max_nodes=2300` and uses tactical-balanced scoring.
- On White turn 1, automove follows one random hardcoded opening route (one move per call). If the current position no longer matches any route, it falls back to normal smart search.

## Newcomer Map

Read these files in this order:

1. `src/models/mons_game.rs`
2. `src/models/scoring.rs`
3. `src/models/mons_game_model.rs`
4. `src/models/mons_game_model_automove_experiments.rs`

What each file is for:

- `mons_game.rs`: legal moves, event application, turn transitions, win conditions.
- `scoring.rs`: board preferability evaluation and weight presets.
- `mons_game_model.rs`: production automove API and runtime selector logic.
- `mons_game_model_automove_experiments.rs`: test-only tournament harness and candidate profiles.

## Release Safety

- Production automove logic is in `src/models/mons_game_model.rs`.
- Experiment harness is in `src/models/mons_game_model_automove_experiments.rs`.
- Harness is included only under `#[cfg(test)]` in `src/models/mons_game_model.rs`:
  - `#[path = "mons_game_model_automove_experiments.rs"]`
  - `mod smart_automove_pool_tests;`

Result: tournament harness code does not ship in release builds.

## First 10 Minutes

Run from workspace root:

1. `cargo test --lib smart_automove_pool_smoke_runs`
2. `cargo test --lib smart_automove_pool_keeps_ten_models`
3. `cargo test --lib opening_book`
4. `cargo check --release --target wasm32-unknown-unknown`

If these pass, your local setup is sane for experiments.

## Experiment Controls

Core knobs:

- `SMART_CANDIDATE_PROFILE`
- `SMART_POOL_GAMES`
- `SMART_POOL_OPPONENTS`
- `SMART_POOL_MAX_PLIES`
- `SMART_USE_WHITE_OPENING_BOOK` (`true/false`, default `false`)

Why `SMART_USE_WHITE_OPENING_BOOK` defaults to `false`:

- Production applies opening routes.
- Promotion experiments usually should compare search/eval quality directly, not opening-book luck.
- Enable it only when explicitly validating production-like opening behavior.

## Fast Iteration Loop

Use this loop for most work:

1. Speed + quick strength screen:
   - `SMART_FAST_PROFILES=runtime_current,<candidate_profile> SMART_FAST_BASELINE=runtime_current SMART_FAST_USE_CLIENT_MODES=true cargo test --lib smart_automove_pool_fast_pipeline -- --ignored --nocapture`
2. Direct duel vs shipped runtime:
   - `SMART_DUEL_A=<candidate_profile> SMART_DUEL_B=runtime_current SMART_DUEL_REPEATS=3 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`
3. If still positive, run larger duel:
   - `SMART_DUEL_A=<candidate_profile> SMART_DUEL_B=runtime_current SMART_DUEL_GAMES=4 SMART_DUEL_REPEATS=5 SMART_DUEL_MAX_PLIES=80 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`
4. Only then run full pool promotion:
   - `SMART_CANDIDATE_PROFILE=<candidate_profile> SMART_POOL_GAMES=100 cargo test --lib smart_automove_pool_candidate_promotion_with_client_budgets -- --ignored --nocapture`

## Useful Test Commands

Profile sweep:

- `SMART_POOL_GAMES=4 SMART_SWEEP_PROFILES=runtime_current,weights_balanced cargo test --lib smart_automove_pool_profile_sweep -- --ignored --nocapture`

Speed probe:

- `SMART_CANDIDATE_PROFILE=runtime_current SMART_SPEED_POSITIONS=20 cargo test --lib smart_automove_pool_profile_speed_probe -- --ignored --nocapture`

Runtime diagnostics:

- `SMART_DIAG_GAMES=4 SMART_DIAG_MODE=normal cargo test --lib smart_automove_pool_runtime_diagnostics -- --ignored --nocapture`

## Promotion Criteria

Candidate is considered promotable only when all are true:

- Beats at least `MIN_OPPONENTS_BEAT_TO_PROMOTE` opponents (`7` currently).
- Per-opponent confidence for beaten matchups >= `MIN_CONFIDENCE_TO_PROMOTE` (`0.75` currently).
- Per-mode aggregate confidence >= `0.75`.
- Combined aggregate confidence >= `0.75`.
- CPU is within acceptable ratio in fast pipeline gates.

## Candidate Profiles To Know

- `runtime_current`: currently shipped behavior.
- `runtime_pre_drainer_context`: snapshot before current fast drainer-context promotion.
- `runtime_legacy_phase_adaptive`: older legacy reference.
- `runtime_drainer_context`: fast-only drainer-context candidate path.
- `runtime_d2_tuned`: older fixed-weight reference.

## Failed Experiments Log

Use this section as an anti-pattern memory so future iterations skip known dead ends faster.

### 1) `runtime_drainer_priority` (weights-only drainer emphasis)

Idea:

- Increase drainer-centric weights globally to force mana-race pressure.

What happened:

- CPU stayed near baseline (`~0.99x` in quick fast pipeline).
- Strength did not beat baseline reliably (`0.500` quick win rate in fast pipeline checks).

Takeaway:

- Pure weight inflation without better tactical context was too blunt.

### 2) `runtime_drainer_priority_aggr` (more aggressive weights-only variant)

Idea:

- Push drainer and carrier urgency even harder than `runtime_drainer_priority`.

What happened:

- No robust strength lift vs `runtime_current`.
- Similar CPU, but no promotion-quality confidence.

Takeaway:

- More aggressive static weights amplified noise, not decision quality.

### 3) `runtime_drainer_tiebreak` (root-level drainer heuristic tie-break)

Idea:

- Keep search mostly unchanged, but re-rank near-top roots by a custom drainer delta metric.

What happened:

- Quick pipeline showed underperformance (`0.000` win rate in one screening run).
- No consistent improvement in repeated checks.

Takeaway:

- Late tie-break after search was weaker than integrating signals inside evaluation itself.

### 4) Full two-mode `runtime_drainer_context` (same idea applied to `fast` and `normal`)

Idea:

- Apply drainer-context scoring to both client modes.

What happened:

- Fast mode improved strongly.
- Normal mode regressed in larger samples.
- Combined large duel result: `22W-18L`, win rate `0.550`, confidence `0.682` (below confidence bar).

Takeaway:

- Fast and normal need separate strategies; one-size-fits-both-mode tuning was unstable.

### 5) `runtime_drainer_context` + wider-root in fast branch

Idea:

- Combine drainer-context scoring with wider-root search shape in fast mode.

What happened:

- Fast duel degraded vs non-wideroot variant:
  - Wideroot: `12W-8L`, confidence `0.748`
  - No wideroot: `13W-7L`, confidence `0.868`

Takeaway:

- For this scorer, widening fast root hurt move quality per node budget.

## What Worked Best So Far

Current promoted direction:

- Keep `normal` conservative (`TACTICAL_BALANCED_SCORING_WEIGHTS` path).
- Improve `fast` with drainer-context board signals.
- Keep opening-route policy in production, but disabled by default in promotion experiments.

Observed proof against pre-promotion snapshot (fast mode):

- `runtime_current` vs `runtime_pre_drainer_context`:
  - `13W-7L`
  - win rate `0.650`
  - confidence `0.868`

## How To Add A New Candidate

1. Add a candidate function in `src/models/mons_game_model_automove_experiments.rs`.
2. Register it in `candidate_model()` and `all_profile_variants()`.
3. Run fast loop and duel loop.
4. Promote only with strength + confidence + CPU evidence.
5. Move runtime changes into `src/models/mons_game_model.rs`.
6. Keep harness-only logic in experiment file.

## Final Validation Before Release

Run:

1. `cargo test --lib`
2. `cargo check --release`
3. `cargo check --release --target wasm32-unknown-unknown`

Then verify:

- No legacy API exposure (`smartAutomoveWithBudgetAsync` should not exist).
- Experiment harness remains test-only.

## Important

- Do not run/scan `rules-tests/` unless explicitly requested.
- Keep release checks focused on automove tests plus release `cargo check`.
