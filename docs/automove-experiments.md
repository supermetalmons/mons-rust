# Smart Automove Experimentation Guide

This document is the entry point for iterating on automove strength safely and quickly.

## What Is Shipped Right Now

Public API:

- `smartAutomoveAsync("fast")`
- `smartAutomoveAsync("normal")`

Current runtime behavior:

- `fast` is CPU-shaped around old `depth=2/max_nodes=420`.
- `normal` is CPU-shaped around old `depth=3/max_nodes=3450` (about 1.5x historical `normal`).
- Both modes use `RUNTIME_RUSH_SCORING_WEIGHTS` (single runtime board-preferability profile promoted from experiments).
- Search uses alpha-beta plus a bounded transposition table (TT). TT writes are skipped for budget-cut partial nodes to avoid polluted cache reuse.
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
- `SMART_DUEL_SEED_TAG` (optional; when set, duel openings are seeded by this tag instead of profile names)

Why `SMART_USE_WHITE_OPENING_BOOK` defaults to `false`:

- Production applies opening routes.
- Promotion experiments usually should compare search/eval quality directly, not opening-book luck.
- Enable it only when explicitly validating production-like opening behavior.

## Fast Iteration Loop

Use this loop for most work:

1. Speed + quick strength screen:
   - `SMART_FAST_PROFILES=runtime_current,<candidate_profile> SMART_FAST_BASELINE=runtime_current SMART_FAST_USE_CLIENT_MODES=true cargo test --lib smart_automove_pool_fast_pipeline -- --ignored --nocapture`
2. Direct duel vs shipped runtime (first orientation):
   - `SMART_DUEL_A=<candidate_profile> SMART_DUEL_B=runtime_current SMART_DUEL_REPEATS=3 SMART_DUEL_SEED_TAG=neutral_v1 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`
3. Reverse orientation with the same seed tag:
   - `SMART_DUEL_A=runtime_current SMART_DUEL_B=<candidate_profile> SMART_DUEL_REPEATS=3 SMART_DUEL_SEED_TAG=neutral_v1 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`
4. Aggregate both orientations before deciding; this cancels opening-parity bias.
5. If still positive, run larger two-way duel:
   - `SMART_DUEL_A=<candidate_profile> SMART_DUEL_B=runtime_current SMART_DUEL_GAMES=4 SMART_DUEL_REPEATS=5 SMART_DUEL_MAX_PLIES=80 SMART_DUEL_SEED_TAG=neutral_v2 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`
   - `SMART_DUEL_A=runtime_current SMART_DUEL_B=<candidate_profile> SMART_DUEL_GAMES=4 SMART_DUEL_REPEATS=5 SMART_DUEL_MAX_PLIES=80 SMART_DUEL_SEED_TAG=neutral_v2 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`
6. Only then run full pool promotion:
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
- `runtime_pre_winloss_weights`: snapshot before current rush-scoring promotion.
- `runtime_pre_tactical_runtime`: snapshot before current tactical-runtime scorer promotion.
- `runtime_pre_transposition`: snapshot before TT-enabled search path.
- `runtime_pre_normal_x15`: snapshot before normal 1.5x budget/runtime-shape update.
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

### 6) `runtime_drainer_priority_fast_only`

Idea:

- Keep normal unchanged, but swap fast runtime scorer to drainer-priority weights.

What happened:

- Looked positive in small screens.
- Lost in larger de-biased two-way duel vs baseline:
  - aggregate `34W-38L`, win rate `0.472`.

Takeaway:

- Keep this as an experiment profile only; do not promote.

### 7) `runtime_d2_tuned_d3_winloss` (moderate normal win/loss blend)

Idea:

- Keep fast branch on tuned D2.
- Replace normal branch with a moderate win/loss-aware tactical blend.

What happened:

- Lost in de-biased normal-mode two-way runs vs `runtime_pre_winloss_weights`.
- Example aggregate from two neutral tags was negative (`3W-13L`).

Takeaway:

- Extra win/loss urgency in normal branch was unstable; avoid this shape as a direct runtime replacement.

### 8) `runtime_fast_winloss` variants (fast-only win/loss weighting)

Idea:

- Improve fast mode by adding stronger carrier/drainer urgency and threat penalties.

What happened:

- Multiple tuned variants underperformed tuned D2 baseline in fast-only two-way duels.
- Results were inconsistent across tags and trended negative overall.

Takeaway:

- Fast mode at current depth/node budget is sensitive to over-aggressive tactical weighting.

## What Worked Best So Far

Current promoted direction:

- Keep `normal` on ~1.5x budget vs historical baseline and compare with neutral duel seed tags to avoid profile-name seed coupling.
- Use a single runtime preferability profile for both modes: `RUNTIME_RUSH_SCORING_WEIGHTS`.
- Keep TT enabled in runtime search, but validate with de-biased two-way duels.
- Keep opening-route policy in production, but disabled by default in promotion experiments.

Observed rush-scoring benchmark vs `runtime_pre_winloss_weights`:

- De-biased two-way duel, seed `rush_1`: `10W-6L` for candidate `weights_rush`.
- De-biased two-way duel, seed `rush_2`: `13W-3L` for candidate `weights_rush`.
- De-biased two-way duel, seed `rush_3`: `11W-5L` for candidate `weights_rush`.
- Combined across these seed tags: `34W-14L` (win rate `0.708`).
- Post-promotion runtime check (seed `rush_4`, de-biased two-way):
  - `runtime_current` vs `runtime_pre_winloss_weights`: `13W-3L`.
- Speed probe (`SMART_SPEED_POSITIONS=30`) stayed effectively flat:
  - `runtime_current`: fast `~52.9ms`, normal `~364.7ms`.
  - `runtime_pre_winloss_weights`: fast `~52.8ms`, normal `~363.1ms`.

Observed proof against pre-promotion snapshot (fast mode):

- `runtime_current` vs `runtime_pre_drainer_context`:
  - `13W-7L`
  - win rate `0.650`
  - confidence `0.868`

Observed normal-mode benchmark after introducing `runtime_pre_normal_x15` snapshot:

- Use multiple neutral duel tags (`neutral_v1`, `neutral_v2`, `neutral_v3`, ...).
- Do not rely on a single profile-pairing seed, because profile-name-coupled seeding can flip conclusions.

Observed TT benchmark vs `runtime_pre_transposition`:

- Fast mode (de-biased two-way, larger run): `36W-36L` (neutral).
- Normal mode (de-biased two-way): `8W-4L`, win rate `0.667`.
- Speed probe (`SMART_SPEED_POSITIONS=30`):
  - `runtime_current`: fast `~52.1ms`, normal `~360.0ms`.
  - `runtime_pre_transposition`: fast `~52.9ms`, normal `~512.5ms`.

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
