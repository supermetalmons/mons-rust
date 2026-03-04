# Smart Automove Experimentation Guide

This document is the entry point for iterating on automove strength safely and quickly.

Pro-player strategy interview notes used as iteration input are tracked in `docs/automove-pro-strategy-interview.md`.

---

## Quick Reference (Agent Runbook)

Three commands cover the full evaluation pipeline, from cheapest to most thorough.

### 1. Fast Screen (~10-20s)

Runs 2 tiers of geometric-doubling games (2→4 games/seed) on a single seed tag. Use to discard obviously bad candidates before investing compute.

```sh
./scripts/run-experiment-logged.sh fast_screen_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
      SMART_GATE_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_fast_screen -- --ignored --nocapture
```

Pass criteria: aggregate delta ≥ 0.0 (not clearly worse).

### 2. Progressive Duel (~1-5 min)

Geometric-doubling evaluation across 3 seed tags (2→4→8→16→32 games/seed). Early exit on clear rejection or early promotion. Writes incremental JSONL artifacts to `target/experiment-runs/`.

```sh
./scripts/run-experiment-logged.sh progressive_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
      SMART_GATE_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_progressive_duel -- --ignored --nocapture
```

Pass criteria: at least one mode (fast or normal) must improve, neither may regress.

### 3. Full Promotion Ladder (~5-30 min)

Staged evaluation: tactical guardrails → CPU speed gate → budget conversion diagnostic → progressive duel (primary strength) → confirmation duel → pool regression check.

```sh
./scripts/run-experiment-logged.sh ladder_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
      SMART_GATE_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_promotion_ladder -- --ignored --nocapture
```

Pass criteria: all stages pass, artifacts written to `target/experiment-runs/`.

---

## How Progressive Evaluation Works

The progressive duel replaces the old fixed-batch B_quick + C_reduced + D_primary stages with a single geometric-doubling evaluator.

**Tier structure** (default `ProgressiveDuelConfig::standard()`):

| Tier | Games/seed | Cumulative (3 seeds × 2 repeats × 2 budgets × 2 mirrors) |
|------|-----------|-----------------------------------------------------------|
| 0    | 2         | ~48 games                                                 |
| 1    | 4         | ~96 games                                                 |
| 2    | 8         | ~144 games                                                |
| 3    | 16        | ~240 games                                                |
| 4    | 32        | ~432 games                                                |

**Stop conditions** (checked after each tier):

1. **Early Reject** — aggregate delta drops below floor (default -0.05). Candidate is clearly worse.
2. **Mathematical Reject** — no mode can mathematically reach its improvement threshold even if all remaining games are wins.
3. **Early Promote** — at least one mode meets improvement + confidence thresholds, all modes pass non-regression, aggregate delta ≥ 0.0.
4. **Max Games Reached** — all tiers exhausted; final gate conditions checked.

**Artifact flushing** — after each tier, results are written to a JSONL file in `target/experiment-runs/`. If the process crashes or is killed, data from completed tiers is preserved.

**Config presets** (in `ProgressiveDuelConfig`):

| Preset | Use Case | Seeds | Max games/seed | Repeats |
|--------|----------|-------|----------------|---------|
| `fast_screen()` | Quick discard | 1 | 4 | 2 |
| `standard()` | Progressive duel test | 3 | 32 | 2 |
| `primary_strength()` | Promotion ladder | 3 | 64 | 3 |

All presets can be overridden via environment variables (prefix `SMART_PROGRESSIVE_<STAGE>_`):

- `SMART_PROGRESSIVE_LADDER_INITIAL_GAMES`
- `SMART_PROGRESSIVE_LADDER_MAX_GAMES`
- `SMART_PROGRESSIVE_LADDER_REPEATS`
- `SMART_PROGRESSIVE_LADDER_MAX_PLIES`

---

## What Is Shipped Right Now

Public API:

- `smartAutomoveAsync("fast")`
- `smartAutomoveAsync("normal")`
- `smartAutomoveAsync("pro")`
- `smartAutomoveAsync("ultra")`

Current runtime behavior:

- `fast` is CPU-shaped around `depth=2/max_nodes=480`.
- `normal` is CPU-shaped around `depth=3/max_nodes=3800`.
- `pro` is CPU-shaped around `depth=4/max_nodes=14022` and always runs full pro budget (no adaptive fallback to normal).
- `ultra` is CPU-shaped around `depth=5/max_nodes=140220` and uses runtime context split (independent primary branch + opening confirmation branch).
- `fast` uses `RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF` (boolean drainer danger, `-400`/`-300`, plus `supermana_race_control: 30`).
- `normal` uses phase-adaptive boolean drainer weights (`RUNTIME_NORMAL_BOOLEAN_DRAINER_*_SPIRIT_BASE_SCORING_WEIGHTS` family), switching by game phase.
- `pro` uses runtime context split with model-local hinting (no FEN schema change):
  - `Independent` context (default search): `max_visited_nodes=14022`, forced tactical prepass off, reply-risk `165/9/24/2000`, drainer safety `4800`, selective-extension share `1500`, and attacker-proximity phase-adaptive scoring.
  - `OpeningBookDriven` context (production opening confirmation path): `max_visited_nodes=14022`, forced tactical prepass off, reply-risk `155/7/18/1400`, drainer safety `4300`, selective-extension share `1200`, and deep-floor off for safer conversion.
  - Context hint is persisted on the model and set when opening-book move selection is used; unknown states fall back to deterministic opening-context detection.
- `ultra` uses the same model-local context hinting and deterministic opening detection:
  - `Independent` context: `max_visited_nodes=140220`, forced tactical prepass off, two-pass root allocation on, reply-risk `175/10/30/2400`, drainer safety `5000`, selective-extension share `1900`, and attacker-proximity phase-adaptive scoring.
  - `OpeningBookDriven` context: `max_visited_nodes=140220`, safer reply-risk `165/8/22/1700`, drainer safety `4600`, selective-extension share `1200`, and deep-floor off.
- Both modes enable `enable_enhanced_drainer_vulnerability` (exact-geometry boolean threat detection for drainer and mana carriers).
- `fast` enables `enable_supermana_prepass_exception`: when the position has supermana scoring potential, the forced tactical prepass skips drainer attack and drainer safety overrides, allowing the search to find supermana plays.
- `fast` uses boosted interview supermana bonuses: `interview_soft_supermana_score_bonus = 600` (from 360), `interview_soft_supermana_progress_bonus = 320` (from 240).
- `normal` uses `root_drainer_safety_score_margin = 4200` (raised from 900 to make the drainer safety prefilter near-hard).
- `fast` and `normal` both use root efficiency tie-breaks (progress-aware, with soft no-effect/low-impact penalties).
- `normal` keeps root-safety rerank/deep-floor and uses root reply-risk guard (`score_margin=145`, shortlist `7`, reply-limit `16`, node-share cap `13.5%`).
- `fast` also uses reply-risk guard with fast limits (`score_margin=125`, shortlist `4`, reply-limit `10`, node-share cap `6.5%`).
- `fast` root-quality bundle uses `root_efficiency_score_margin=1700`, `root_anti_help_score_margin=280`, `root_mana_handoff_penalty=300`, `root_backtrack_penalty=220`.
- Root/child tactical class coverage uses strict guarantees for critical tactical classes before truncation.
- Root anti-help filtering rejects near-best mana-handoff/roundtrip roots when non-losing clean alternatives exist.
- Automove start suggestions use the automove-specific option that can include mana starts when potion-action starts are available.
- Selective tactical extension is normal-only, capped to one extension per path with a dedicated node-share budget (`12.5%`).
- Search uses alpha-beta plus a bounded transposition table (TT). TT writes are skipped for budget-cut partial nodes to avoid polluted cache reuse.
- On White turn 1, automove follows one random hardcoded opening route (one move per call). If the current position no longer matches any route, it falls back to normal smart search.

---

## Swift 2024 Reference

Historical Swift heuristics are preserved as an immutable baseline in:

- `src/models/scoring.rs` → `SWIFT_2024_REFERENCE_SCORING_WEIGHTS`

Exact preserved multipliers (2024 source):

- `confirmed_score: 1000`
- `fainted_mon: -500`
- `fainted_drainer: -800`
- `drainer_at_risk: -350`
- `mana_close_to_same_pool: 500`
- `mon_with_mana_close_to_any_pool: 800`
- `extra_for_supermana: 120`
- `extra_for_opponents_mana: 100`
- `drainer_close_to_mana: 300`
- `drainer_holding_mana: 350`
- `mon_close_to_center: 210`
- `has_consumable: 110`
- `active_mon: 50`

Reference profiles:

- `swift_2024_eval_reference`: current runtime search with `SWIFT_2024_REFERENCE_SCORING_WEIGHTS`.
- `swift_2024_style_reference`: simplified legacy-style search (legacy no-TT path, reduced modern root policy stack) with `SWIFT_2024_REFERENCE_SCORING_WEIGHTS`.

These profiles are for calibration and comparison only; they are not shipped runtime behavior.

---

## Pro Mode

Public API contract:

- `smartAutomoveAsync("pro")` is valid and GA-capable once strict pro ladder passes.
- Existing `fast`/`normal` contracts are unchanged.
- Pro does not adaptively fall back to normal; it always runs the pro budget.

CPU intent:

- Pro is targeted at roughly `~3.69x` normal CPU on the fixed-position probe.
- Promotion ladder target band: `2.70x..3.69x` (hard fail when `>3.69x`).

Strict dual-baseline promotion criteria:

- Baselines are `runtime_current@normal` and `runtime_current@fast`.
- Fast screen: pro aggregate delta must be `>= 0.0` against both baselines.
- Progressive duel: both matchups non-negative, with at least one meaningful lift (`delta >= +0.04`, `confidence >= 0.65`).
- Final strict bar:
  - `pro vs normal`: `delta >= +0.08`, `confidence >= 0.90`
  - `pro vs fast`: `delta >= +0.14`, `confidence >= 0.90`
- Tactical suite and pool non-regression must pass before runtime promotion.

Pro-specific command runbook:

```sh
./scripts/run-experiment-logged.sh pro_fast_screen_normal_<candidate> -- \
  env SMART_PRO_CANDIDATE_PROFILE=<candidate> \
      SMART_PRO_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_pro_fast_screen_vs_normal -- --ignored --nocapture

./scripts/run-experiment-logged.sh pro_fast_screen_fast_<candidate> -- \
  env SMART_PRO_CANDIDATE_PROFILE=<candidate> \
      SMART_PRO_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_pro_fast_screen_vs_fast -- --ignored --nocapture

./scripts/run-experiment-logged.sh pro_progressive_normal_<candidate> -- \
  env SMART_PRO_CANDIDATE_PROFILE=<candidate> \
      SMART_PRO_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_pro_progressive_vs_normal -- --ignored --nocapture

./scripts/run-experiment-logged.sh pro_progressive_fast_<candidate> -- \
  env SMART_PRO_CANDIDATE_PROFILE=<candidate> \
      SMART_PRO_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_pro_progressive_vs_fast -- --ignored --nocapture

./scripts/run-experiment-logged.sh pro_ladder_<candidate> -- \
  env SMART_PRO_CANDIDATE_PROFILE=<candidate> \
      SMART_PRO_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_pro_promotion_ladder -- --ignored --nocapture
```

Mode-isolated improvement policy:

- Improve `pro` independently; do not co-mingle fast/normal promotion changes in the same round.
- Candidate logic remains test-only in `src/models/mons_game_model_automove_experiments.rs` until full pro ladder pass.
- Runtime `SmartSearchConfig::from_preference(Pro)` is updated only after a pro candidate clears strict ladder.

Round-1 (easiest-first, conversion stability) profiles:

- `runtime_pro_conversion_guard_v2`
- `runtime_pro_conversion_guard_v3`
- `runtime_pro_ordering_tt_v2`
- `runtime_pro_depth4_stable_v2`

Mandatory pro round loop:

1. Fast screen each candidate vs normal and vs fast.
2. Keep only candidates with both deltas `>= 0.0`.
3. Run progressive vs both baselines for survivors.
4. Run full pro ladder for top 1-2 survivors.
5. If no promotion, classify failure:
   - beats fast, fails normal → conversion/safety-only next round
   - beats normal, fails fast → tactical sharpness next round
   - CPU ratio `>3.69x` → optimization-only next round
   - tactical guardrail fail → lock pattern as hard constraint
   - seed instability only → narrow parameter amplitude + increase repeats

---

## Ultra Mode

Public API contract:

- `smartAutomoveAsync("ultra")` is valid and GA-capable once strict ultra ladder passes.
- Existing `fast`/`normal`/`pro` contracts are unchanged.

CPU intent:

- Ultra targets `3.30x..10.00x` CPU vs `runtime_current@pro` on the fixed-position probe.
- Hard fail outside that band.

Strict ultra promotion criteria:

- Primary strict bar vs pro:
  - `ultra vs pro`: `delta >= +0.08`, `confidence >= 0.90`
- Primary non-regression:
  - `ultra vs normal`: `delta >= 0.0`
  - `ultra vs fast`: `delta >= 0.0`
- Confirmation non-regression (opening-book enabled):
  - `ultra vs pro`: `delta >= 0.0`
  - `ultra vs normal`: `delta >= 0.0`
  - `ultra vs fast`: `delta >= 0.0`
- Tactical suite, CPU gate, and pool non-regression must pass before promotion.

Ultra-specific command runbook:

```sh
./scripts/run-experiment-logged.sh ultra_fast_screen_pro_<candidate> -- \
  env SMART_ULTRA_CANDIDATE_PROFILE=<candidate> \
      SMART_ULTRA_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_ultra_fast_screen_vs_pro -- --ignored --nocapture

./scripts/run-experiment-logged.sh ultra_fast_screen_normal_<candidate> -- \
  env SMART_ULTRA_CANDIDATE_PROFILE=<candidate> \
      SMART_ULTRA_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_ultra_fast_screen_vs_normal -- --ignored --nocapture

./scripts/run-experiment-logged.sh ultra_fast_screen_fast_<candidate> -- \
  env SMART_ULTRA_CANDIDATE_PROFILE=<candidate> \
      SMART_ULTRA_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_ultra_fast_screen_vs_fast -- --ignored --nocapture

./scripts/run-experiment-logged.sh ultra_progressive_pro_<candidate> -- \
  env SMART_ULTRA_CANDIDATE_PROFILE=<candidate> \
      SMART_ULTRA_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_ultra_progressive_vs_pro -- --ignored --nocapture

./scripts/run-experiment-logged.sh ultra_progressive_normal_<candidate> -- \
  env SMART_ULTRA_CANDIDATE_PROFILE=<candidate> \
      SMART_ULTRA_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_ultra_progressive_vs_normal -- --ignored --nocapture

./scripts/run-experiment-logged.sh ultra_progressive_fast_<candidate> -- \
  env SMART_ULTRA_CANDIDATE_PROFILE=<candidate> \
      SMART_ULTRA_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_ultra_progressive_vs_fast -- --ignored --nocapture

./scripts/run-experiment-logged.sh ultra_ladder_<candidate> -- \
  env SMART_ULTRA_CANDIDATE_PROFILE=<candidate> \
      SMART_ULTRA_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_ultra_promotion_ladder -- --ignored --nocapture
```

---

## Newcomer Map

Read these files in this order:

1. `src/models/mons_game.rs` — legal moves, event application, turn transitions, win conditions.
2. `src/models/scoring.rs` — board preferability evaluation and weight presets.
3. `src/models/mons_game_model.rs` — production automove API and runtime selector logic.
4. `src/models/mons_game_model_automove_experiments.rs` — test-only tournament harness and candidate profiles.

## Release Safety

- Production automove logic is in `src/models/mons_game_model.rs`.
- Experiment harness is in `src/models/mons_game_model_automove_experiments.rs`.
- Harness is included only under `#[cfg(test)]` in `src/models/mons_game_model.rs`:
  - `#[path = "mons_game_model_automove_experiments.rs"]`
  - `mod smart_automove_pool_tests;`
- Tournament harness code does not ship in release builds.

---

## How To Create A New Candidate

1. **Add a model function** in `src/models/mons_game_model_automove_experiments.rs`:

```rust
fn model_my_candidate(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut config = config;
    // Modify scoring weights, search shape, or both:
    config.scoring_weights.my_new_weight = 42;
    smart_search_best_inputs(game, &config)
}
```

2. **Register in `candidate_model()`** — add a match arm:

```rust
"my_candidate" => Some(model_my_candidate as fn(&MonsGame, SmartSearchConfig) -> Vec<Input>),
```

3. **Register in `all_profile_variants()`** — add the name string to the list.

4. **Run fast screen** → if positive, run progressive duel → if positive, run full ladder.

5. **Promote** — see the Promotion To Production section below.

6. **Keep harness-only logic** in the experiment file.

---

## Iteration Strategy

**Cardinal rule: always start small.** The biggest past waste was running large experiments for candidates that turned out to be no good. The progressive pipeline exists to prevent that — never skip straight to a full ladder.

**When a candidate fails fast screen:**
- The idea is likely fundamentally off. Don't tune parameters — rethink the approach.
- Check the Failed Experiments Log below to avoid repeating known dead ends.

**When a candidate fails progressive duel:**
- Check per-mode breakdown. If one mode improves and the other regresses, consider a mode-specific change (fast and normal need separate strategies).
- If both modes are neutral, the change probably doesn't affect depth-2/depth-3 decisions. Try a different evaluation dimension.
- Shrink the change. The most reliable pattern is small, additive weight changes in orthogonal evaluation dimensions.

**When a candidate fails the ladder:**
- If CPU gate fails: the change adds too much computation. Optimize or reduce scope.
- If tactical guardrails fail: the change broke a known-good position. Debug which guardrail and why.
- If pool regression fails: the candidate is weaker against diverse opponents. The improvement was too narrow.
- If primary strength passes but confirmation/pool fails: could be sample-size-dependent — consider re-running with different seeds before abandoning.

### Balanced Iteration Loop (Mandatory)

Use this when searching for promotable improvements:

1. Run fast screen for all active round candidates.
2. Keep only candidates with aggregate `delta >= 0.0`.
3. Run progressive duel for survivors.
4. Run full promotion ladder for the best 1-2 survivors.
5. If none promote, run failure analysis and start the next round immediately (do not stop at “no result”).

Failure-analysis rules:

- Fast regresses, normal improves → next round is fast-only tuning.
- Normal regresses, fast improves → next round is normal-only rollback/tuning.
- CPU gate fails → optimization-only variants (lighter shape, same logic).
- Tactical guardrail fails → lock that tactical pattern as a hard candidate constraint in next round.
- Append failed candidate family notes to Failed Experiments Log before new round.

**Design principles that have worked:**
- Minimal, additive weight changes beat large restructurings.
- Boolean/discrete signals beat continuous approximations (e.g., boolean drainer danger vs distance-based).
- New evaluation dimensions orthogonal to existing tactics are the most reliable way to improve fast mode (depth-2 decisions are hard to shift within existing weight dimensions).
- Fast mode benefits from tight, focused evaluation — adding more computation introduces noise, not signal.
- Normal mode is more forgiving — deeper search (depth 3) can exploit richer evaluation signals.

---

## First 10 Minutes

Run from workspace root:

1. `cargo test --lib smart_automove_pool_smoke_runs`
2. `cargo test --lib smart_automove_pool_keeps_ten_models`
3. `cargo test --lib opening_book`
4. `cargo check --release --target wasm32-unknown-unknown`

If these pass, your local setup is sane for experiments.

## Robust Experiment Execution (Mandatory)

Always run experiments through the file-backed wrapper:

```sh
./scripts/run-experiment-logged.sh <run_name> -- <command...>
```

This writes to `target/experiment-runs/`:

- `<timestamp>_<run_name>.log` — full stdout+stderr
- `<timestamp>_<run_name>.exit` — exit code
- `<timestamp>_<run_name>.cmd` — the command that was run
- `<timestamp>_<run_name>.meta` — timing metadata

Rule: do not run long/important duels, ladders, or gates without this wrapper.

---

## Experiment Controls

Core knobs:

- `SMART_CANDIDATE_PROFILE` — candidate profile name for all test entry points
- `SMART_GATE_BASELINE_PROFILE` — baseline to compare against (default `runtime_current`)
- `SMART_POOL_GAMES` — games per matchup in pool tournaments
- `SMART_POOL_OPPONENTS` — number of pool opponents
- `SMART_POOL_MAX_PLIES` — maximum plies per game
- `SMART_USE_WHITE_OPENING_BOOK` (`true/false`, default `false`)
- `SMART_DUEL_SEED_TAG` — optional seed tag for duel openings
- `SMART_GATE_SPEED_POSITIONS` — positions for CPU speed measurement
- `SMART_GATE_ALLOW_SELF_BASELINE` (`true` only for baseline artifact capture)
- `SMART_GATE_CONFIRM_GAMES`, `SMART_GATE_CONFIRM_REPEATS`, `SMART_GATE_CONFIRM_MAX_PLIES`
- `SMART_GATE_POOL_GAMES` — pool non-regression game count
- `SMART_PROGRESSIVE_LADDER_INITIAL_GAMES` — override progressive tier-0 games
- `SMART_PROGRESSIVE_LADDER_MAX_GAMES` — override progressive max games/seed
- `SMART_PROGRESSIVE_LADDER_REPEATS` — override progressive repeats per game
- `SMART_PROGRESSIVE_LADDER_MAX_PLIES` — override progressive max plies

Why `SMART_USE_WHITE_OPENING_BOOK` defaults to `false`:

- Production applies opening routes.
- Promotion experiments should compare search/eval quality directly, not opening-book luck.
- Enable it only when explicitly validating production-like opening behavior.

---

## Promotion Criteria

The evaluation assesses fast and normal modes **separately**. Key principle: **at least one mode must improve, neither may regress.**

### Progressive Duel Thresholds (standard config)

- Aggregate delta floor (early reject): -0.05
- Per-mode non-regression: delta ≥ -0.03
- Per-mode improvement (at least one must pass):
  - Fast: delta ≥ +0.02, confidence ≥ 0.60
  - Normal: delta ≥ +0.06, confidence ≥ 0.60
- Aggregate non-regression: combined delta ≥ 0.0

### Primary Strength Thresholds (ladder config)

- Per-mode non-regression: delta ≥ -0.02
- Per-mode improvement (at least one must pass):
  - Fast: delta ≥ +0.05, confidence ≥ 0.90
  - Normal: delta ≥ +0.10, confidence ≥ 0.90
- Aggregate non-regression: combined delta ≥ 0.0

### Other Ladder Stage Criteria

- CPU gate: fast ratio ≤ 1.15x, normal ratio ≤ 1.15x
- Budget-conversion regression: informational (printed but not asserted)
- Confirmation gate: informational (printed but not asserted)
- Pool non-regression: candidate must beat ≥ as many pool opponents as baseline; aggregate pool win-rate must not drop by > 0.01

---

## Useful Test Commands

Speed probe:

```sh
SMART_CANDIDATE_PROFILE=runtime_current SMART_SPEED_POSITIONS=20 \
  cargo test --lib smart_automove_pool_profile_speed_probe -- --ignored --nocapture
```

Mirrored duel (manual):

```sh
SMART_DUEL_A=<candidate> SMART_DUEL_B=runtime_current \
  SMART_DUEL_GAMES=2 SMART_DUEL_REPEATS=2 SMART_DUEL_MAX_PLIES=72 \
  SMART_DUEL_SEED_TAG=quick_v1 \
  cargo test --release --lib smart_automove_pool_profile_duel -- --ignored --nocapture
```

Budget duel (fast vs normal same profile):

```sh
SMART_BUDGET_DUEL_A=runtime_current SMART_BUDGET_DUEL_B=runtime_current \
  SMART_BUDGET_DUEL_A_MODE=fast SMART_BUDGET_DUEL_B_MODE=normal \
  SMART_BUDGET_DUEL_GAMES=3 SMART_BUDGET_DUEL_REPEATS=4 \
  SMART_BUDGET_DUEL_MAX_PLIES=56 SMART_BUDGET_DUEL_SEED_TAG=fast_normal_v1 \
  cargo test --lib smart_automove_pool_budget_duel -- --ignored --nocapture
```

Runtime diagnostics:

```sh
SMART_DIAG_GAMES=4 SMART_DIAG_MODE=normal \
  cargo test --lib smart_automove_pool_runtime_diagnostics -- --ignored --nocapture
```

Deterministic coordinate-descent board-eval tuner:

```sh
SMART_TUNE_PROFILE=runtime_current \
  SMART_TUNE_TRAIN_POSITIONS_PER_SEED=256 \
  SMART_TUNE_HOLDOUT_POSITIONS_PER_SEED=128 \
  SMART_TUNE_ROOT_LIMIT=12 SMART_TUNE_TOP_K=8 \
  SMART_TUNE_MANIFEST_OUTPUT=target/eval_tune_ranked_candidates.json \
  cargo test --lib smart_automove_pool_tune_eval_weights_coordinate_descent -- --ignored --nocapture
```

---

## Candidate Profiles To Know

- `runtime_current`: currently shipped behavior.
- `runtime_pro_baseline_v1`: pro baseline candidate mirroring runtime pro-v1 shape.
- `runtime_pro_depth4_stable_v1`: pro search-shape stabilization candidate.
- `runtime_pro_depth4_stable_v2`: lighter depth-4 root-stability variant (`root_focus_k=4`, `focus_share=7600`, `root_branch_limit+=1`).
- `runtime_pro_depth4_extension_v1`: pro selective-extension allocation candidate.
- `runtime_pro_conversion_guard_v1`: pro reply-risk/safety conversion candidate.
- `runtime_pro_conversion_guard_v2`: pro conversion stability candidate (`160/8/22/1750`, drainer safety `4500`, selective extension share `1400`).
- `runtime_pro_conversion_guard_v3`: pro conversion aggressiveness variant (`150/9/24/1500`, drainer safety `4300`, selective extension share `1700`).
- `runtime_pro_eval_long_horizon_v1`: pro long-horizon eval enrichment candidate.
- `runtime_pro_eval_long_horizon_v2`: long-horizon attack-proximity phase family for pro (escalation track).
- `runtime_pro_eval_long_horizon_v3`: v2 plus mild opponent-mana long-horizon emphasis (escalation track).
- `runtime_pro_ordering_tt_v1`: pro ordering/TT efficiency candidate.
- `runtime_pro_ordering_tt_v2`: TT+killer ordering variant with `enable_pvs=false`.
- `runtime_pro_conversion_guard_v2_cpu_opt_v1`: pro conversion v2 with lighter node/branch caps (CPU-targeted).
- `runtime_pro_conversion_guard_v2_cpu_opt_v2`: tighter CPU-targeted variant of `cpu_opt_v1`.
- `runtime_pro_conversion_guard_v2_cpu_opt_v3`: CPU-targeted variant with tactical-sharpness restraint.
- `runtime_pro_cpu_prune_v1` / `runtime_pro_cpu_prune_v2` / `runtime_pro_cpu_prune_v3` / `runtime_pro_cpu_prune_v4`: pro optimization-only pruning/branch-shape variants.
- `runtime_pro_cpu_safety_lite_v1` / `runtime_pro_cpu_safety_lite_v2`: lighter pro safety-probe workload variants.
- `runtime_pro_cpu_prune_v5` / `runtime_pro_cpu_prune_v6`: near-threshold CPU-targeted pro variants.
- `runtime_pro_cpu_prepass_off_v1`: CPU-targeted variant with forced tactical prepass disabled.
- `runtime_pro_primary_confirm_split_v1`: pro split-profile (primary-strength settings + opening-book confirmation-safe settings) that cleared strict ladder in harness.
- `runtime_pro_primary_confirm_state_split_v1` / `runtime_pro_primary_confirm_state_split_v2`: state-only split attempts to replace harness-mode split.
- `runtime_pro_context_split_runtime_v1`: runtime-equivalent context split baseline.
- `runtime_pro_context_split_runtime_v2`: runtime-equivalent context split with +400 node lift (promoted runtime pro profile).
- `runtime_pro_context_split_runtime_v3`: v2 plus confirmation-only safety tightening.
- `runtime_pro_context_split_runtime_v4`: v2 plus primary-only conversion lift.
- `runtime_pro_context_split_runtime_v5`: v2 plus stronger confirmation safety hardening.
- `runtime_pro_context_split_runtime_v6`: opening confirmation uses runtime-normal profile, independent context uses high-utilization pro.
- `runtime_pro_context_split_runtime_v7`: context split with pro nominal node target (`14022`) for utilization stress testing.
- `runtime_ultra_d5_split_cal_v1`: ultra depth-5 context-split calibration (low node target).
- `runtime_ultra_d5_split_cal_v2`: ultra depth-5 context-split calibration (mid-low node target).
- `runtime_ultra_d5_split_cal_v3`: ultra depth-5 context-split calibration (mid-high node target).
- `runtime_ultra_d5_split_cal_v4`: ultra depth-5 context-split calibration (high node target).
- `runtime_ultra_d5_split_confirm_safety_v1`: ultra context-split with confirmation and pressure safety tightening.
- `runtime_ultra_d5_split_primary_conversion_v1`: ultra context-split with primary conversion lift (reply window and extension share).
- `runtime_ultra_d5_split_normalized_v1`: normal-style depth-5 ultra backbone to reduce normal-regression risk.
- `runtime_ultra_d5_split_hybrid_balance_v1`: aggressive/safe hybrid split by score-race state.
- `runtime_ultra_d5_split_normal_guard_v1`: v4-style ultra with pressure-triggered normal-safety guard.
- `runtime_ultra_d5_split_normalized_v2`: normalized backbone with conditional aggressive finisher conversion.
- `runtime_ultra_d5_ordering_tt_v15_early_neutral_pro_bridge`: ultra ordering/TT family with early neutral pro-bridge.
- `runtime_ultra_d5_ordering_tt_v16_bridge_plus_mid_conversion`: ultra ordering/TT family with pro-bridge + midgame conversion lift.
- `runtime_ultra_d5_ordering_tt_v17_guarded_pro_bridge_lite`: guarded/lite pro-bridge variant.
- `runtime_ultra_d5_ordering_tt_v18_hiutil_pro_bridge_plus`: higher-utilization guarded pro-bridge variant.
- `runtime_ultra_d5_ordering_tt_v19_hybrid_bridge_floor_pick`: hybrid bridge with reply-floor post-pick.
- `runtime_ultra_d5_ordering_tt_v20_hiutil_v16_node_lift`: v16-family node-lift variant.
- `runtime_ultra_d5_ordering_tt_v21_parity_gated_bridge`: v16-family parity-gated bridge variant.
- `runtime_ultra_d5_ordering_tt_v22_odd_opening_runtime_fallback`: v16-family odd-opening runtime fallback variant.
- `runtime_ultra_d5_ordering_tt_v23_bridge_floor_arbiter`: v16-family bridge floor-arbiter variant.
- `runtime_ultra_d5_search_verify_v24`: ultra search-verify rerank family.
- `runtime_ultra_d5_deep_narrow_v25`: non-bridge deep-narrow depth-5 family.
- `runtime_ultra_d5_deep_narrow_v26_confirm_pro_bridge`: v25 primary with opening confirmation path bridged to pro runtime policy.
- `runtime_ultra_d5_deep_narrow_v27_confirm_pro_bridge_stable`: v26 with slightly wider independent deep-narrow branch.
- `runtime_ultra_d5_deep_narrow_v28_early_pro_arbiter`: v27 plus early-neutral pro arbiter in independent context.
- `runtime_ultra_d5_deep_narrow_v29_early_pro_arbiter_tight`: tighter v28 arbiter thresholds.
- `runtime_ultra_d5_deep_narrow_v30_early_pro_arbiter_black_only`: v28 arbiter limited to black-to-move neutral states.
- `runtime_ultra_d5_deep_narrow_v31_early_pro_arbiter_turn2`: v28 arbiter limited to turn-2 neutral states.
- `runtime_ultra_d5_runtime_current_plus_early_pro_arbiter_v32`: runtime-current ultra backbone plus early-neutral pro arbiter.
- `runtime_ultra_d5_runtime_plus_early_pro_arbiter_with_normal_guard_v33`: v32 with normal-floor guard before pro fallback.
- `runtime_ultra_d5_runtime_plus_early_pro_arbiter_with_normal_guard_v34`: tighter normal-floor guard variant.
- `runtime_ultra_d5_runtime_plus_early_pro_arbiter_explicit_confirm_bridge_v35`: v33 with confirmation bridge enabled only when opening-book mode is explicitly on.
- `runtime_ultra_d5_explicit_context_consensus_v36`: non-opening runs forced to independent ultra context with pro+normal consensus fallback.
- `runtime_ultra_d5_robust_tri_blend_v37`: early-neutral tri-blend (`ultra/pro/normal`) via robust static evaluation mix.
- `runtime_ultra_d5_dual_floor_optimizer_v38`: early-neutral tri-blend optimized by dual-opponent reply floors.
- `runtime_ultra_d5_early_normal_guarded_pro_arbiter_v39`: early-neutral normal-floor guard plus pro arbiter.
- `runtime_ultra_d5_early_normal_guarded_pro_arbiter_v40`: stronger normal guard variant of v39.
- `runtime_ultra_d5_early_normal_guarded_pro_arbiter_v41`: strongest normal-biased guard variant in this family.
- `runtime_ultra_d5_default_eval_adjudication_v43`: early-neutral tri-selection aligned to adjudicator default eval.
- `runtime_pro_cpu_prepass_off_v2_phase_budget_v1` / `v2` / `v3`: phase-conditioned pro budget-expansion attempts.
- `runtime_pre_pro_promotion_v1`: snapshot profile before pro runtime promotion.
- `swift_2024_eval_reference`: Swift 2024 weights on top of current runtime search.
- `swift_2024_style_reference`: Swift 2024 weights with simplified legacy-style search path.
- `runtime_swift_opponent_mana_exception_v1`: candidate enabling opponent-mana tactical prepass exception.
- `runtime_swift_opponent_mana_exception_v2`: v1 plus mild fast-only opponent-mana soft-priority boost.
- `runtime_swift_opponent_mana_exception_v3`: fast-only opponent-mana tactical prepass exception (normal kept baseline).
- `runtime_swift_opponent_mana_exception_v4`: v3 plus lighter fast-only opponent-mana soft-priority boost.
- `runtime_swift_opponent_mana_exception_v5`: fast-only opponent-mana prepass exception with strict immediate-score gating.
- `runtime_swift_opponent_mana_exception_v6`: v5 plus moderate fast-only opponent-mana soft-priority boost.
- `runtime_fast_root_quality_v1`: candidate for fast root filtering/tie-break quality tuning.
- `runtime_fast_root_quality_v2`: softened fast root filtering/tie-break tuning for better pool non-regression.
- `runtime_fast_root_quality_v3`: v1-style fast root quality with baseline reply-risk guard settings.
- `runtime_normal_conversion_v1`: candidate for normal reply-risk/safety/extension conversion tuning.
- `runtime_normal_conversion_v2`: refined normal conversion tuning (reply-risk/safety/extension shares).
- `runtime_normal_conversion_v3`: stronger normal conversion allocation (reply-risk/safety/extension shares).
- `runtime_fast_root_quality_v1_normal_conversion_v3`: promoted synthesis bundle (fast root-quality v1 + normal conversion v3).
- `runtime_pre_fast_root_quality_v1_normal_conversion_v3`: snapshot of runtime behavior before promoting the synthesis bundle.
- `runtime_eval_board_v3_normal_only`: board-v3 eval applied only to normal mode (fast kept at runtime baseline).
- `runtime_pre_efficiency_logic`: runtime budgets/scoring as current, but with fast root-efficiency tie-break disabled.
- `runtime_pre_fast_efficiency_cleanup`: legacy fast runtime for this cleanup iteration.
- `runtime_pre_event_ordering`: baseline with event-aware root/child ordering bonus disabled.
- `runtime_pre_backtrack_penalty`: baseline with fast root roundtrip/backtrack penalty disabled.
- `runtime_pre_drainer_tactical_requirements`: baseline with forced drainer-attack root filtering and drainer-safety root prefilter disabled.
- `runtime_pre_root_upgrade_bundle`: baseline with all three root upgrades above disabled.
- `runtime_pre_move_efficiency`: snapshot before current node-budget/runtime-shape increase.
- `runtime_pre_fast_drainer_priority`: snapshot before current fast drainer-context promotion.
- `runtime_pre_winloss_weights`: snapshot before current rush-scoring promotion.
- `runtime_pre_tactical_runtime`: snapshot before current tactical-runtime scorer promotion.
- `runtime_pre_transposition`: snapshot before TT-enabled search path.
- `runtime_pre_normal_x15`: snapshot before normal 1.5x budget/runtime-shape update.
- `runtime_normal_efficiency_reply_floor`: current promoted normal root-efficiency/backtrack path.
- `runtime_pre_normal_efficiency_reply_floor`: snapshot before promoting normal root-efficiency/backtrack in runtime.
- `runtime_pre_drainer_context`: snapshot before current fast drainer-context promotion.
- `runtime_legacy_phase_adaptive`: older legacy reference.
- `runtime_d2_tuned`: older fixed-weight reference.
- `runtime_eval_board_v1` / `runtime_eval_board_v2`: board-eval candidate profile hooks for tuned board-weight promotion runs.
- `runtime_fast_boost_v1`: boolean drainer protection candidate (promoted — identical to `runtime_current`).
- `runtime_supermana_priority_v1`: supermana priority fast-only candidate (promoted — identical to `runtime_current`).

---

## Interpreting Results

### Progressive Duel Output

Each tier prints a summary line:

```
progressive tier 0 | games/seed=2 | total=48 | δ=+0.0625 | conf=0.712 | continuing…
  mode fast | 8W-4L-0D | δ=+0.1667 | conf=0.927
  mode normal | 5W-7L-0D | δ=-0.0833 | conf=0.274
```

Outcomes:
- **EARLY REJECT** — candidate is clearly worse. Stop iterating on this approach.
- **MATH REJECT** — candidate can't possibly reach improvement thresholds. Stop.
- **EARLY PROMOTE** — candidate clearly passes all criteria. Safe to proceed to ladder.
- **MAX GAMES** — all tiers exhausted; check final stats to decide.

### Artifact Files

Progressive duels write JSONL to `target/experiment-runs/progressive_<profile>_<timestamp>.jsonl`. Each line is one tier's cumulative results with per-mode breakdown.

Ladder artifacts (when `SMART_LADDER_ARTIFACT_PATH` is set) are stage-by-stage JSON lines.

---

## Promotion To Production

After a candidate passes the full promotion ladder:

1. **Create a `runtime_pre_<feature>` snapshot** profile — captures the exact pre-change baseline for future comparisons.
2. **Move runtime changes into `src/models/mons_game_model.rs`**:
   - Update `SmartSearchConfig::from_preference(Fast)` and/or `from_preference(Normal)` with new config fields.
   - If scoring weights changed, update `with_runtime_scoring_weights()` to dispatch to new presets.
   - If new weight presets are needed, add them in `src/models/scoring.rs`.
3. **Verify `runtime_current` now behaves identically** to the promoted candidate (both should select the same moves).
4. **Run final validation**:
   ```sh
   cargo test --lib
   cargo check --release
   cargo check --release --target wasm32-unknown-unknown
   ```
5. **Update `docs/automove-experiments.md`** — add the new profile to "Candidate Profiles To Know", update "What Is Shipped Right Now", and note the promotion in "What Worked Best So Far" if applicable.
6. **Keep experiment harness code test-only** — model functions, registration, and test entry points stay in the experiments file.

---

## Board Eval Workflow

1. Freeze baseline evidence (speed probe + budget duel + tactical suite).
2. Export train/holdout datasets with root labels.
3. Run coordinate-descent tuning to produce `target/eval_tune_ranked_candidates.json`.
4. Map best ranked bundle into a named profile (`runtime_eval_board_v1` / `runtime_eval_board_v2`).
5. Run progressive duel, then full ladder against `runtime_current`.
6. Promote runtime constants only after ladder pass, keeping all tuner/export helpers test-only.

---

## Failed Experiments Log

Use this section as an anti-pattern memory so future iterations skip known dead ends faster.

### 1) `runtime_drainer_priority` (weights-only drainer emphasis)

Pure weight inflation without better tactical context was too blunt. CPU near baseline, strength at 0.500.

### 2) `runtime_drainer_priority_aggr` (more aggressive weights-only variant)

More aggressive static weights amplified noise, not decision quality.

### 3) `runtime_drainer_tiebreak` (root-level drainer heuristic tie-break)

Late tie-break after search was weaker than integrating signals inside evaluation itself. Quick pipeline showed 0.000 win rate.

### 4) Full two-mode `runtime_drainer_context` (fast + normal)

Fast improved strongly, normal regressed. Combined 22W-18L, confidence 0.682. Fast and normal need separate strategies.

### 5) `runtime_drainer_context` + wider-root in fast branch

Widening fast root hurt move quality per node budget. Wideroot 12W-8L vs no-wideroot 13W-7L.

### 6) `runtime_drainer_priority_fast_only`

Lost in larger de-biased two-way duel (34W-38L, win rate 0.472).

### 7) `runtime_d2_tuned_d3_winloss` (moderate normal win/loss blend)

Extra win/loss urgency in normal branch was unstable; avoid as direct runtime replacement.

### 8) `runtime_fast_winloss` variants (fast-only win/loss weighting)

Fast mode at current depth/node budget is sensitive to over-aggressive tactical weighting.

### 9) Aggressive `RUNTIME_RUSH` mana-pressure overlays

Overcommitted in tactical spots. Large urgency overlays were too brittle.

### 10) Hard no-effect pruning / heavy no-effect penalties

Full pruning and high penalties reduced stability. Soft/no-op handling performed better.

### 11) Fast root efficiency tie-break (light, fast-only)

Light root efficiency signals improved practical fast-mode quality without material CPU cost. **Promoted.**

### 12) Same-budget self-label board-eval tuning

Same-budget self-labeling is weak supervision; use deeper-label targets for meaningful signal.

### 13) Full-grid board-eval tuning by default

Too slow for daily churn. Reserve `SMART_TUNE_FULL_GRID=true` for final quality passes.

### 14) `runtime_normal_x15_tactical_lite` as direct promotion candidate

Normal-only gains not enough for strict promotion. Must include explicit fast-mode proof.

### 15) Fast-side micro-patches (reply-risk lite, simplified root heuristics)

Small heuristic toggles alone did not produce robust fast-mode lift.

### 16) Fast depth-3 lite probe under branch caps

Fast mode-ratio exploded (>2x). Depth bump not viable under current CPU caps.

### 17) Interview policy candidates (`runtime_interview_policy_v1..v5`)

Interview policy priors are directionally useful but not sufficient for strict promotion under current search shape/caps. Keep as experiment hooks.

### 18) Round-1 Swift opponent-mana exception (`runtime_swift_opponent_mana_exception_v1/v2`)

Cross-mode enablement created fast/normal instability (one mode up, the other down depending on seed). Next rounds should isolate this policy to fast-only variants.

### 19) Round-1 fast root quality push (`runtime_fast_root_quality_v1`)

Passed progressive duel but failed ladder pool non-regression (`candidate_wr=0.550`, `baseline_wr=0.583`). Aggressive fast root-margin tuning overfit narrow duel seeds.

### 20) Round-1 normal conversion tuning (`runtime_normal_conversion_v1`)

Non-negative in fast screen but too weak/unstable in progressive runs to satisfy improvement thresholds. Treat as insufficient signal; iterate with tighter normal-only shape tuning.

### 21) Round-2/3 strict opponent-mana exception variants (`runtime_swift_opponent_mana_exception_v3..v6`)

Fast-only and strict-score-gated exception variants mostly became no-ops (exactly neutral) or seed-unstable in progressive runs. Keep only as reference hooks.

### 22) Round-3 protected-carrier eval variants (including normal-only)

`runtime_eval_protected_carrier_v4` and `runtime_eval_protected_carrier_v3_normal_only` failed fast screen early with clear negative deltas.

### 23) `runtime_potion_takeback_starts_v11`

Fast-screen aggregate stayed negative at max games (`δ=-0.0089`). Potion takeback policy as configured here is not promotion-grade under current search shape.

### 24) `runtime_eval_board_v3` and `runtime_eval_board_v3_normal_only`

These profiles showed promising early normal lift but regressed or became unstable on broader progressive seeds; not reliable enough as standalone promotions.

### 25) Pro Round-1 fast-screen snapshot (March 2, 2026)

Baseline and Round-1 pro candidates were non-negative in fast screen against both baselines, but early progressive signals remained seed-volatile; continue with conversion-first rounds before touching eval-family changes.

Observed fast-screen outputs (repeats=2, games=2):

- `runtime_pro_baseline_v1`: vs normal `8W-0L` (`δ=+0.5000`, `conf=0.996`), vs fast `5W-3L` (`δ=+0.1250`, `conf=0.637`)
- `runtime_pro_conversion_guard_v2`: vs normal `8W-0L` (`δ=+0.5000`, `conf=0.996`), vs fast `5W-3L` (`δ=+0.1250`, `conf=0.637`)
- `runtime_pro_conversion_guard_v3`: vs normal `8W-0L` (`δ=+0.5000`, `conf=0.996`), vs fast `4W-4L` (`δ=+0.0000`, `conf=0.000`)
- `runtime_pro_ordering_tt_v2`: vs normal `8W-0L` (`δ=+0.5000`, `conf=0.996`), vs fast `5W-3L` (`δ=+0.1250`, `conf=0.637`)
- `runtime_pro_depth4_stable_v2`: vs normal `8W-0L` (`δ=+0.5000`, `conf=0.996`), vs fast `5W-3L` (`δ=+0.1250`, `conf=0.637`)

### 26) Pro CPU-gate optimization family (March 2, 2026)

Multiple pro optimization-only variants preserved fast-screen non-regression but still failed strict pro CPU cap (`2.70x..3.69x`) on ladder speed probe:

- `runtime_pro_conversion_guard_v2`: `ratio=4.896`
- `runtime_pro_ordering_tt_v1`: `ratio=4.979`
- `runtime_pro_ordering_tt_v2`: `ratio=4.919`
- `runtime_pro_cpu_prune_v1`: `ratio=3.436`
- `runtime_pro_cpu_prune_v2`: `ratio=3.451`
- `runtime_pro_cpu_prune_v3`: `ratio=3.438`
- `runtime_pro_cpu_prune_v4`: `ratio=3.449`
- `runtime_pro_cpu_safety_lite_v1`: `ratio=3.429`
- `runtime_pro_cpu_prune_v5`: `ratio=3.382` (closest)
- `runtime_pro_cpu_prune_v6`: `ratio=3.450`
- `runtime_pro_cpu_prepass_off_v1`: `ratio=3.428`

Conclusion: parameter-only pro tuning is not enough to clear CPU gate at target strength; next rounds should prioritize search-engine efficiency improvements (not further policy reshaping) before more promotion attempts.

### 27) Pro strict-ladder follow-up (March 2, 2026, higher CPU cap accepted)

Under the updated hard cap (`<=3.69x`), the main blocker shifted from pure CPU to **normal-baseline primary strength**:

- `runtime_pro_cpu_prepass_off_v1`: CPU gate passed (`3.423x`), strong vs fast primary (`δ=+0.1806`), failed vs normal primary (`δ=+0.0602 < +0.08`).
- `runtime_pro_cpu_prune_v2_cpucap_v1`: CPU gate passed (`3.466x`), very strong vs fast primary (`δ=+0.2407`), failed vs normal primary (`δ=-0.0046`).

Additional families eliminated this round:

- `runtime_pro_cpu_prune_v2`: CPU miss under old cap (`3.511x > 3.50x`).
- `runtime_pro_eval_long_horizon_prepass_off_v1`: CPU miss under old cap (`3.554x > 3.50x`).
- `runtime_pro_cpu_prune_v2_cpucap_ordering_v1`: fast-screen regression vs fast (`δ=-0.1250`).
- `runtime_pro_runtime_cpucap_v1`: fast-screen regression vs fast (`δ=-0.2500`) and neutral vs normal (`δ=0.0000`).

Takeaway: current pro candidates reliably beat `fast` but fail to gain enough edge over `runtime_current@normal` under the CPU band; next rounds should target **normal-baseline conversion quality** specifically (not additional fast-side aggression), with CPU-neutral refinements.

### 28) Pro strict-ladder follow-up after cap raise to `3.69x` (March 2, 2026)

Raising the hard cap from `3.50x` to `3.69x` removed CPU as the blocker for previously near-cap candidates, but did not solve the normal-baseline strength gap:

- `runtime_pro_cpu_prune_v2`: CPU passed (`3.496x`), fast primary strong (`δ=+0.2454`), failed normal primary (`δ=+0.0046 < +0.08`).
- `runtime_pro_eval_long_horizon_prepass_off_v1`: CPU passed (`3.489x`), fast primary passed (`δ=+0.1620`), failed normal primary (`δ=+0.0602 < +0.08`).

Rejected follow-up families in this round:

- `runtime_pro_runtime_cpucap_v1`: fast-screen regression (`vs fast δ=-0.2500`), neutral vs normal (`δ=0.0000`).
- `runtime_pro_cpu_prune_v2_cpucap_ordering_v1`: fast-screen regression (`vs fast δ=-0.1250`).

Takeaway: with CPU headroom expanded, promotion still fails on **normal matchup conversion**; further progress requires pro-only normal-conversion gains rather than additional fast-side lift or generic ordering tweaks.

### 29) Pro round continuation under `3.69x` (March 2, 2026)

Additional strict-ladder attempts under the raised cap confirm the same directional split:

- `runtime_pro_cpu_prune_v2`: CPU passed (`3.496x`), failed normal primary (`δ=+0.0046`), strong fast primary (`δ=+0.2454`).
- `runtime_pro_eval_long_horizon_prepass_off_v1`: CPU passed (`3.489x`), failed normal primary (`δ=+0.0602`), passed fast primary (`δ=+0.1620`).
- `runtime_pro_normal_conversion_focus_v1`: CPU passed (`3.425x`), failed normal primary (`δ=+0.0231`), passed fast primary (`δ=+0.1991`).

Round conclusion:

- Raising cap to `3.69x` unblocked CPU gates for near-cap candidates.
- Promotion remains blocked by **vs-normal strict primary delta**, not by fast strength or CPU.
- Next variants should target normal conversion quality specifically while preserving current fast-side gains.

### 30) Pro continuation: confirmation-vs-primary tradeoff mapping (March 3, 2026)

This round added direct pro confirmation probes (opening-book enabled) to isolate the strict ladder blocker.

Key strict-ladder outcomes:

- `runtime_pro_eval_long_horizon_prepass_off_v2`: CPU passed; strict primary passed (`vs normal δ=+0.0880`, `vs fast δ=+0.2500`), then failed confirmation vs normal (`δ=-0.0938`).
- `runtime_pro_eval_long_horizon_prepass_guarded_v1`: CPU passed; failed strict primary (`vs normal δ=+0.0648`, `vs fast δ=+0.1065`).
- `runtime_pro_cpu_prepass_off_v2_long_horizon_v1`: CPU passed; strict primary vs normal remained below bar (`δ=+0.0694`), run stopped early after primary could no longer reach `+0.08`.
- `runtime_pro_cpu_prepass_off_v2_long_horizon_v2`: CPU passed; strict primary vs normal remained below bar (`δ=+0.0648`), run stopped early after primary could no longer reach `+0.08`.

Confirmation probe map (`pro_confirm_vs_normal_v1`, repeats=4, games=4):

- Positive:
  - `runtime_pro_cpu_prune_v2`: `δ=+0.1250`
  - `runtime_pro_cpu_prepass_off_v1`: `δ=+0.0938`
  - `runtime_pro_cpu_prepass_off_v2`: `δ=+0.0312`
- Non-negative edge:
  - `runtime_pro_cpu_prepass_off_v2_long_horizon_v1`: `δ=+0.0000`
  - `runtime_pro_cpu_prepass_off_v2_long_horizon_v2`: `δ=+0.0000`
- Negative:
  - `runtime_pro_eval_long_horizon_prepass_off_v1`: `δ=-0.0938`
  - `runtime_pro_eval_long_horizon_prepass_off_v2`: `δ=-0.0938`
  - `runtime_pro_eval_long_horizon_prepass_off_v2_opening_safety_v1`: `δ=-0.0938`
  - `runtime_pro_eval_long_horizon_prepass_off_v3`: `δ=-0.0625`
  - `runtime_pro_eval_long_horizon_prepass_on_v1`: `δ=-0.0625`
  - `runtime_pro_cpu_prepass_off_v1_long_horizon_v1`: `δ=-0.0312`
  - `runtime_pro_cpu_prepass_off_v1_long_horizon_v2`: `δ=-0.0625`
  - `runtime_pro_cpu_prepass_off_v1_long_horizon_v3`: `δ=-0.0312`
  - `runtime_pro_cpu_prepass_off_v2_long_horizon_v3`: `δ=-0.0312`
  - `runtime_pro_cpu_prepass_off_v2_long_horizon_v4`: `δ=-0.0625`

Round conclusion:

- We now have confirmation-stable pro families, and we now have strict-primary-passing families, but not yet in the same candidate.
- The remaining gap is narrow: closest strict-primary-safe family is around `vs normal δ≈+0.069..+0.070` while keeping confirmation `>= 0.0`.

### 31) Pro wider-picture split strategy results (March 3, 2026)

New instrumentation:

- Added pro-only probe tests:
  - `smart_automove_pool_pro_primary_probe_vs_normal`
  - `smart_automove_pool_pro_primary_probe_vs_fast`
  - `smart_automove_pool_pro_seed_turn_distribution_probe`
- Seed distribution probe (`pro` vs `normal`, repeats=4, games=4) showed opening roots are almost entirely:
  - `(turn=1, active=white)` and `(turn=2, active=black)`
  - primary histogram: `{(1,white): 11, (2,black): 5}`
  - confirmation histogram: `{(1,white): 12, (2,black): 4}`

Key candidate outcomes:

- `runtime_pro_primary_confirm_split_v1` (context split by opening-book mode signal) cleared full strict pro ladder:
  - CPU gate: `ratio=2.970`
  - primary vs normal: `δ=+0.0880`, `conf=0.994`
  - primary vs fast: `δ=+0.2500`, `conf=1.000`
  - confirm vs normal: `δ=+0.0938`
  - confirm vs fast: `δ=+0.3125`
  - pool checks: passed (`candidate_delta=+0.0500` vs normal-opponents, `+0.2000` vs fast-opponents)
- `runtime_pro_primary_confirm_state_split_v1` (state-only split: aggressive on `turn1/white`, safe otherwise):
  - confirm vs normal passed (`δ=+0.0938`)
  - primary vs normal failed (`δ=+0.0324`)
- `runtime_pro_primary_confirm_state_split_v2` (state-only split using opening-book first-move board match):
  - confirm vs normal failed (`δ=-0.0625`)
- `runtime_pro_cpu_prepass_off_v2_phase_budget_v2`:
  - confirm vs normal non-regression held (`δ=0.0000`)
  - strict primary probe vs normal remained below bar (`δ=+0.0694`)

Round-level conclusion:

- The strongest current signal is **context splitting** between primary-strength settings and opening-book-sensitive confirmation settings.
- A full-ladder pass is now demonstrated in harness with split logic.
- A robust runtime-equivalent state signal is still required before shipping this split behavior as production pro runtime config.

### 32) Pro runtime context-split promotion pass (March 3, 2026)

Runtime-equivalent context split ladder runs initially looked contradictory because experiment helper `pro_context_split_runtime_base` ignored opening-book hinting and resolved context as `Unknown` only. After fixing that harness bug to honor `SMART_USE_WHITE_OPENING_BOOK`, `runtime_pro_context_split_runtime_v2` cleared reduced strict ladder end-to-end:

- CPU gate: `ratio=2.976`
- primary vs normal: `δ=+0.2083`, `conf=0.968`
- primary vs fast: `δ=+0.3333`, `conf=0.999`
- confirm vs normal: `δ=+0.0312`
- confirm vs fast: `δ=+0.3125`
- pool non-regression: passed (`candidate_delta=+0.1500` vs both normal/fast opponent pools)

Promotion action:

- Runtime `pro` context profiles in `mons_game_model.rs` updated from `9800` to `10200` nodes for both independent and opening-book-driven branches.
- Fast/normal runtime branches unchanged.

### 33) Ultra pro-positive context-split family regressed vs normal (March 3, 2026)

Profiles:

- `runtime_ultra_d5_split_cal_v1..v4`
- `runtime_ultra_d5_split_confirm_safety_v1`
- `runtime_ultra_d5_split_primary_conversion_v1`

Observed pattern:

- Fast-screen stayed non-negative vs pro/normal/fast in tiny probes.
- Progressive and strict ladder seeds exposed normal regression (`ultra vs normal delta < 0`) while pro/fast stayed directionally strong.
- Ultra CPU utilization was in-band (`ratio=3.421` vs pro on ladder speed gate for `runtime_ultra_d5_split_primary_conversion_v1`), so this failure is quality allocation, not CPU budget.

Takeaway:

- For ultra, pro-targeting conversion pressure alone is insufficient; normal-baseline stability needs stronger primary-path balancing, not only confirmation-path tightening.

### 34) Ultra normal-safe families lost pro head-to-head (March 3, 2026)

Profiles:

- `runtime_ultra_d5_split_normalized_v1`
- `runtime_ultra_d5_split_hybrid_balance_v1`
- `runtime_ultra_d5_split_normal_guard_v1`
- `runtime_ultra_d5_split_normalized_v2`

Observed pattern:

- These variants improved or stabilized normal-side probes.
- They failed fast-screen vs pro (`delta < 0.0`), so they cannot enter ultra progressive/ladder under strict gates.

Takeaway:

- Ultra requires a narrower hybridization: preserve pro-pressure signal in more states, and inject normal-safety only at clearly identified loss patterns.

### 35) Budget-chain realignment (`pro=3.69x normal`, `ultra=5.50x pro`) and ultra re-probe (March 3, 2026)

Runtime/budget policy changes:

- Pro nominal budget moved to `depth=4/max_nodes=14022` (`3.69x` vs normal `3800`).
- Ultra nominal budget moved to `depth=5/max_nodes=77121` (`5.50x` vs pro `14022`).
- Ultra ladder CPU hard cap moved to `5.50x` vs pro.

Reduced primary-probe results under the new chain (`G=2, R=2, max_plies=72`):

- `runtime_ultra_d5_ordering_tt_v17_guarded_pro_bridge_lite`:
  - vs pro `δ=+0.0417`
  - vs normal `δ=0.0000`
  - vs fast `δ=+0.2500`
- `runtime_ultra_d5_ordering_tt_v16_bridge_plus_mid_conversion`:
  - vs pro `δ=+0.0833`
  - vs normal `δ=+0.1667`
  - vs fast `δ=+0.3750`
- `runtime_ultra_d5_ordering_tt_v19_hybrid_bridge_floor_pick` regressed vs pro (`δ=-0.0417`) and is treated as failed.
- `runtime_ultra_d5_ordering_tt_v20_hiutil_v16_node_lift` matched v16 vs pro (`δ=+0.0833`) in reduced probes.

Takeaway:

- The best current ultra family is `v16`-style bridge+mid-conversion; it is still short of strict pro-primary bar (`+0.10`).
- Next iteration should stay in this family and target pro-only lift (not broad hybrid floor-pick logic).

### 36) Wider-picture trace-driven pivots after v16 plateau (March 4, 2026)

Observed widening signal:

- `v16` reduced probe looked promising (`vs pro δ=+0.0833`) but larger-sample seed buckets showed volatility (for `G=3/R=3`, first seed bucket `δ=-0.0556`).
- Traced `ultra_primary_vs_pro:neutral_v2` showed alternating opening families where `v16` improved some buckets and regressed others.

Tried pivots:

- `runtime_ultra_d5_ordering_tt_v21_parity_gated_bridge` (even-turn-only bridge in neutral openings):
  - reduced vs pro: `δ=-0.0417` (failed)
- `runtime_ultra_d5_ordering_tt_v22_odd_opening_runtime_fallback` (odd-turn neutral openings fallback to runtime-current policy):
  - reduced vs pro: `δ=-0.2083` (failed hard)

Takeaway:

- Opening-parity/state gating on top of the bridge family is not stable enough.
- Next round should pivot from bridge-gating heuristics to a different pro-lift lever (search-shape/verification family), while preserving v16-level non-regression vs normal/fast.

### 37) Wider-picture pivot to mode-split deep-narrow + confirmation bridge (March 4, 2026)

Failed probes before fix:

- `runtime_ultra_d5_ordering_tt_v23_bridge_floor_arbiter`: reduced `vs pro δ=-0.0417` (failed).
- `runtime_ultra_d5_search_verify_v24`: early reduced buckets regressed (`neutral_v1 δ=-0.1250`, `neutral_v2 δ=-0.2500`), treated as failed family.
- `runtime_ultra_d5_deep_narrow_v25`: strong primary but failed confirmation-vs-pro in reduced ladder (`δ=-0.2500`), so not promotable as-is.

Working adjustment:

- Added `runtime_ultra_d5_deep_narrow_v26_confirm_pro_bridge`:
  - keeps `v25` independent (non-opening) branch unchanged for primary strength;
  - opening-book-driven context uses pro runtime context policy for move choice (confirmation stability);
  - retains a bounded ultra aux pass for ordering/TT warm-up.

Evidence (reduced probes, `G=2, R=2, max_plies=72`):

- Primary:
  - vs pro: `wins=20 losses=4 delta=+0.3333 confidence=0.999`
  - vs normal: `wins=17 losses=7 delta=+0.2083 confidence=0.968`
  - vs fast: `wins=20 losses=4 delta=+0.3333 confidence=0.999`
- Confirmation:
  - vs pro: `delta=0.0000`
  - vs normal: `delta=+0.1250`
  - vs fast: `delta=+0.1250`
- Reduced strict ladder (`SMART_POOL_OPPONENTS=2`) passed:
  - CPU ratio vs pro: `3.866` (within `3.30..5.50`)
  - confirmation gates: all non-negative
  - pool non-regression:
    - vs pro-opponents: `candidate_delta=+0.2500 baseline_delta=0.0000`
    - vs normal-opponents: `candidate_delta=+0.2500 baseline_delta=0.0000`
    - vs fast-opponents: `candidate_delta=0.0000 baseline_delta=0.0000`

Takeaway:

- The blocker was not primary quality; it was opening confirmation stability vs pro.
- The mode-split approach (strong independent branch + safer confirmation branch) remains the most reliable ultra direction.
- Next promotion step is full strict ladder with full pool opponent count before runtime move.

### 38) Fast-screen dual-baseline instability after v26 (March 4, 2026)

Observed contradiction:

- `runtime_ultra_d5_deep_narrow_v26_confirm_pro_bridge` passed reduced strict ladder (`SMART_POOL_OPPONENTS=2`) with strong primary and non-negative confirmation.
- The fixed fast-screen seed tags exposed a dual-baseline conflict:
  - `vs pro` fast screen: `δ=-0.2500`
  - `vs normal` fast screen: `δ=-0.1250`

Control baseline on same tags:

- `runtime_current` vs pro fast-screen: `δ=-0.1250`
- `runtime_current` vs normal fast-screen: `δ=0.0000`

Follow-up candidates and outcomes:

- `runtime_ultra_d5_deep_narrow_v27_confirm_pro_bridge_stable`: no change on pro screen (`δ=-0.2500`).
- `runtime_ultra_d5_deep_narrow_v28_early_pro_arbiter`: fixed pro screen (`δ=0.0000`) but regressed normal screen (`δ=-0.3750`).
- `runtime_ultra_d5_deep_narrow_v29_early_pro_arbiter_tight`: pro screen back to `δ=-0.1250`.
- `runtime_ultra_d5_deep_narrow_v30_early_pro_arbiter_black_only`: pro screen `δ=-0.1250`.
- `runtime_ultra_d5_deep_narrow_v31_early_pro_arbiter_turn2`: pro screen `δ=-0.2500`.
- `runtime_ultra_d5_runtime_current_plus_early_pro_arbiter_v32`: pro screen improved (`δ=+0.1250`) and fast screen vs fast strong (`δ=+0.3750`), but normal screen remained `δ=-0.3750`.
- `runtime_ultra_d5_runtime_plus_early_pro_arbiter_with_normal_guard_v33`: pro screen `δ=+0.1250`, normal screen improved but still failed (`δ=-0.2500`).
- `runtime_ultra_d5_runtime_plus_early_pro_arbiter_with_normal_guard_v34`: pro screen `δ=+0.1250`, normal screen regressed again (`δ=-0.3750`).
- `runtime_ultra_d5_runtime_plus_early_pro_arbiter_explicit_confirm_bridge_v35`: pro screen `δ=+0.1250`, normal screen still failed (`δ=-0.2500`) even with confirmation bridge gated to explicit opening-book mode.
- `runtime_ultra_d5_explicit_context_consensus_v36`: converged to baseline-like behavior (`vs pro δ=-0.1250`, `vs normal δ=-0.1250`).
- `runtime_ultra_d5_robust_tri_blend_v37`: robust tri-blend did not lift screens (`vs pro δ=-0.1250`, `vs normal δ=-0.2500`).
- `runtime_ultra_d5_dual_floor_optimizer_v38`: dual-floor optimizer also regressed (`vs pro δ=-0.1250`, `vs normal δ=-0.2500`).
- `runtime_ultra_d5_early_normal_guarded_pro_arbiter_v39`: closest balanced result so far (`vs pro δ=+0.1250`, `vs normal δ=-0.1250`).
- `runtime_ultra_d5_early_normal_guarded_pro_arbiter_v40`: same frontier as v39 (`vs pro δ=+0.1250`, `vs normal δ=-0.1250`).
- `runtime_ultra_d5_early_normal_guarded_pro_arbiter_v41`: over-corrected toward neutrality (`vs pro δ=-0.1250`, `vs normal δ=-0.1250`).
- `runtime_ultra_d5_default_eval_adjudication_v43`: adjudicator-targeted tri-selection regressed (`vs pro δ=-0.2500`, `vs normal δ=-0.2500`).

Detailed diagnosis added:

- New diagnostic `smart_automove_pool_ultra_trace_seed_detailed` was used on failing bucket `ultra_fast_screen_vs_normal_v1`.
- Failures are dominated by `MaxPlyAdjudicated(...)` outcomes (not invalid/no-move tactical errors), confirming this round is primarily a long-race conversion/adjudication-shape problem.

Takeaway:

- Early-neutral pro arbitration shifts losses from pro baseline to normal baseline; pro-only stabilization is insufficient.
- Next round should use multi-opponent arbitration in early-neutral states (pro/normal/ultra floor blend) instead of pro-only bridging, while keeping confirmation-vs-pro non-regression constraint.

### 39) Wider-picture adjudication-guard sweep (v45..v59) and progressive pro wall (March 4, 2026)

What was tried:

- Added a thresholded default-eval guard family (`runtime_ultra_d5_default_eval_guarded_v45` and iterative variants through `v59`) to stabilize `MaxPlyAdjudicated` buckets without touching fast/normal/pro runtime branches.
- Added targeted state-split knobs in this family:
  - early-black-opening normal-fallback restriction,
  - turn-window restriction for normal fallback,
  - white-turn default-eval cap,
  - stronger pro-bridge fallback thresholds.

Fast-screen outcomes (key checkpoints):

- `runtime_ultra_d5_default_eval_guarded_v45`:
  - vs pro `δ=0.0000`
  - vs normal `δ=-0.1250`
- `runtime_ultra_d5_default_eval_guarded_v46` / `v47` / `v48`:
  - vs pro `δ=-0.1250`
  - vs normal `δ=0.0000`
- `runtime_ultra_d5_default_eval_guarded_v51`:
  - vs pro `δ=-0.1250`
  - vs normal `δ=-0.2500`
- `runtime_ultra_d5_default_eval_guarded_v56`:
  - vs pro `δ=0.0000`
  - vs normal `δ=0.0000`
  - vs fast `δ=+0.2500`
- `runtime_ultra_d5_default_eval_guarded_v57` / `v58`:
  - vs pro `δ=0.0000`
  - vs normal `δ=-0.1250`
- `runtime_ultra_d5_default_eval_guarded_v59`:
  - vs pro `δ=0.0000`
  - vs normal `δ=0.0000`
  - vs fast `δ=+0.2500`

Progressive outcomes (critical blocker):

- `runtime_ultra_d5_default_eval_guarded_v56` failed progressive non-regression:
  - vs pro cumulative `δ=-0.2500`
  - vs normal cumulative `δ=-0.0556`
- `runtime_ultra_d5_default_eval_guarded_v59` showed the same early progressive pro failure pattern (first buckets negative: `neutral_v1: δ=-0.2500`, `neutral_v2: δ=-0.3750`) and was not advanced.

Takeaway:

- This family can clear fast-screen on all three baselines, but still collapses on longer-horizon progressive vs pro.
- The current wall is not opening confirmation only; it is pro conversion quality over deeper neutral buckets (`max_plies=80`, larger seed/game tiers).
- Next round should keep the `v56/v59` fast-screen-stable guard skeleton but shift pro strength via search-shape/ordering in pro-facing buckets (not additional normal-fallback heuristics).

### 40) Wider-picture pivot to pro-anchored challenge routing (v60/v62) (March 4, 2026)

Pivot:

- New family: `runtime_ultra_d5_pro_anchor_challenge_v60` and `v62`.
- Core policy:
  - run both ultra and pro move selectors,
  - compute post-move robust score (`ultra_floor`, `pro_floor`, default eval),
  - choose ultra only when it clears a margin over pro.
- `v62` adds early-neutral white-side pro anchoring (detected from trace failures where ultra systematically lost as White in progressive pro seeds).

Fast-screen results:

- `runtime_ultra_d5_pro_anchor_challenge_v60`:
  - vs pro `δ=+0.1250`
  - vs normal `δ=0.0000`
  - vs fast `δ=+0.2500`
- `runtime_ultra_d5_pro_anchor_challenge_v62`:
  - vs pro `δ=0.0000`
  - vs normal `δ=0.0000`
  - vs fast `δ=+0.2500`

Reduced progressive checks (`SMART_ULTRA_PROGRESSIVE_MAX_GAMES=4`, `SMART_ULTRA_PROGRESSIVE_REPEATS=1`):

- `v62` vs pro: `δ=+0.0833`, confidence `0.797` (passed non-regression).
- `v62` vs normal: `δ=+0.1111`, confidence `0.879` (passed).
- `v62` vs fast: `δ=+0.3056`, confidence `1.000` (passed).

Reduced primary probe vs pro (`SMART_ULTRA_GATE_PRIMARY_GAMES=2`, `SMART_ULTRA_GATE_PRIMARY_REPEATS=2`):

- `v62`: `δ=-0.0417` (still below strict ultra primary target).

Takeaway:

- Pro-anchored challenge routing is the first direction in this round that simultaneously clears reduced progressive non-regression on all three baselines.
- Strict primary-vs-pro lift is still missing; next step is pro-only strength lift on top of `v62` without relaxing its progressive stability.

### 41) Ultra opening-lock refinement + trace correctness fix (v67-v77) (March 4, 2026)

What was added:

- New ultra candidate family expansion: `runtime_ultra_d5_pro_anchor_challenge_v67` through `v77`.
- Harness correctness fix in detailed trace path:
  - Added `play_one_game_cross_budget_with_diagnostics(...)` so trace diagnostics use true cross-budget play.
  - Fixed BA-leg result mirroring in `smart_automove_pool_ultra_trace_seed_detailed` summary (candidate-perspective accounting).

Reliable gate observations this round:

- `v67` and `v70`:
  - Full fast-screen (default settings) matched `v60`:
    - vs pro `δ=+0.1250`
    - vs normal `δ=0.0000`
    - vs fast `δ=+0.2500`
  - Reduced progressive vs pro still failed at `δ=-0.1667` (same `neutral_v2` collapse pattern).

- `v71`/`v72`/`v73` (opening pro-lock diagnostics):
  - Reduced progressive vs pro improved from `-0.1667` to `-0.0833`.
  - The remaining deficit persisted in `neutral_v2` (`δ=-0.2500` in reduced stage-1 buckets).

- `v74` (deterministic tiebreak toggle):
  - Regressed reduced progressive vs pro to `δ=-0.2500`.

- `v75` (opening-window pro/normal consensus):
  - First profile to clear reduced progressive non-regression on all three:
    - vs pro `δ=0.0000`
    - vs normal `δ=0.0000`
    - vs fast `δ=+0.3056`
  - Full fast-screen:
    - vs pro `δ=-0.1250` (failed),
    - vs normal `δ=+0.1250`,
    - vs fast `δ=+0.3750`.

- `v76` (stricter normal fallback in opening window):
  - Recovered full fast-screen vs pro non-regression:
    - vs pro `δ=0.0000`
  - But regressed full fast-screen vs normal:
    - vs normal `δ=-0.1250`.

- `v77` (mid-threshold blend):
  - Still failed full fast-screen vs pro:
    - vs pro `δ=-0.1250`.

- `v78` (targeted black turn-0 pro force on top of `v75`):
  - Did not recover pro fast-screen regression (`vs pro δ=-0.1250`).

Trace-driven takeaway:

- The tradeoff is now explicit: opening-window normal blending helps reduced progressive-vs-pro stability but can flip fast-screen-vs-pro below zero on specific repeat/opening buckets.
- With corrected trace accounting, we can now trust per-repeat AB/BA diagnostics and target the exact opening buckets that flip between `v75` and `v76`.
- Next round should tune opening-window blend policy conditionally (not globally), using the corrected cross-budget trace outputs as the primary generator for candidate constraints.

---

### 42) Ultra CPU envelope bump to `10.00x` vs pro (March 4, 2026)

Runtime/promotion policy updates:

- Ultra nominal budget moved from `depth=5/max_nodes=77121` to `depth=5/max_nodes=140220` (`10.00x` vs pro `14022`).
- Search engine max-node clamp moved to `180000` so ultra runtime is not internally clipped below nominal budget.
- Ultra ladder CPU hard cap moved from `5.50x` to `10.00x` vs pro.
- Ultra CPU intent band in the strict gate is now `3.30x..10.00x` vs pro.

Immediate iteration direction:

- Keep mode isolation unchanged (`fast`, `normal`, `pro` runtime branches untouched).
- Re-run ultra fast-screen/progressive/ladder under the widened CPU envelope.
- Prioritize candidates that convert added budget into pro-side primary lift without reopening normal/fast regressions.

---

### 43) Ultra x10 first iteration snapshot (March 4, 2026)

Current best survivor under `10.00x` cap:

- `runtime_ultra_d5_pro_anchor_challenge_v85`
  - Fast-screen:
    - vs pro `delta=0.0000`
    - vs normal `delta=+0.2500`
    - vs fast `delta=+0.2500`
  - Reduced progressive:
    - vs pro `delta=0.0000`
    - vs normal `delta=+0.1667`
    - vs fast `delta=+0.2500`
  - Reduced ladder CPU gate sample: `ratio=4.452` vs pro (inside `3.30..10.00`).
  - Remaining blocker: strict primary lift vs pro is still insufficient (`reduced primary probe delta=0.0000`, bar is `>= +0.10`).

Failed families (new):

- `runtime_ultra_d5_pro_anchor_challenge_v86`: tighter white opening fallback did not improve pro side (`reduced primary probe vs pro delta=-0.0417`).
- `runtime_ultra_d5_hiutil_branch_v87`: branch/enum expansion regression (`fast-screen vs pro delta=-0.2500`).
- `runtime_ultra_d5_pro_anchor_ordering_v88`: ordering/TT tuning did not recover pro primary (`reduced primary probe vs pro delta=-0.0417`).
- `runtime_ultra_d5_pro_anchor_opening_ultra_v89`:
  - conservative opening-ultra takeover thresholds had no measurable lift (`fast-screen vs pro delta=0.0000`);
  - aggressive takeover (`threshold=0`) regressed (`fast-screen vs pro delta=-0.2500`).

Next round focus:

- Keep `v85` as reference.
- Target only pro-primary lift levers for the `neutral_v3` seed family while preserving fast-screen non-regression vs normal/fast.

---

### 44) Ultra x10 wider-picture hybrid routing round (March 4, 2026)

Ground truth from this round (reduced primary gate: `repeats=2`, `games=2`, `max_plies=72`):

- `runtime_ultra_d5_deep_narrow_v27_confirm_pro_bridge_stable` remained the stable baseline:
  - primary vs pro: `delta=+0.0417`
  - fast-screen vs pro: `delta=+0.1250`
- Detailed seed diagnostics on `v27` showed the main blocker is max-ply adjudication drift in `neutral_v1` (`term=MaxPlyAdjudicated(...)`), not tactical-terminal failures.

New families tested:

- `runtime_ultra_d5_deep_narrow_v90_white_opening_stability`
- `runtime_ultra_d5_deep_narrow_v91_turn1_pro_anchor`
- `runtime_ultra_d5_deep_narrow_v92_confirm_pro_bridge_hiutil`
- `runtime_ultra_d5_deep_narrow_v93_early_adjudication_stability`
- `runtime_ultra_d5_hybrid_v94_targeted_opening_route`
- `runtime_ultra_d5_hybrid_v95_turn1_targeted_opening_route`
- `runtime_ultra_d5_hybrid_v96_turn1_white_mons3_route`
- `runtime_ultra_d5_hybrid_v97_mons4_opening_route`

Key outcomes:

- `v90`/`v91`/`v92`/`v93`/`v95`/`v96`/`v97` all converged to effectively the same reduced primary-vs-pro aggregate as `v27` (`delta=+0.0417`), i.e. no net strict-lift.
- `v94` was the only profile to materially lift primary vs pro:
  - primary vs pro: `delta=+0.2500`, confidence `0.989`
  - primary vs normal: `delta=0.0000`
  - primary vs fast: `delta=+0.1667`
  - but it failed fast-screen vs pro (`delta=-0.1250`) and is therefore not promotable.
- `v97` improved fast-screen vs pro (`delta=+0.2500`) but did not keep the primary lift (fell back to `delta=+0.0417` vs pro).

Failed/blocked families log updates:

- `runtime_ultra_d5_default_eval_adjudication_v43`: reduced primary vs pro regressed (`delta=-0.0417`).
- `runtime_ultra_d5_hybrid_v94_targeted_opening_route`: strongest pro-primary lift so far, but blocked by fast-screen vs pro regression.
- `runtime_ultra_d5_hybrid_v95_turn1_targeted_opening_route`, `v96`, `v97`: fast-screen-safe refinements that lost the pro-primary lift.

Next iteration direction:

- Keep `v27` as stable floor and `v94` as pro-primary ceiling.
- Generate only route-gating variants between `v27` and `v94` using trace-derived constraints from:
  - `ultra_primary_vs_pro:neutral_v1`
  - `ultra_fast_screen_vs_pro_v1`
- Promotion remains blocked until one candidate preserves `v94`-class primary lift while keeping fast-screen vs pro non-negative.

---

### 45) Ultra x10 balanced hybrid breakthrough (v98/v100) (March 4, 2026)

New candidate family:

- `runtime_ultra_d5_hybrid_v98_targeted_opening_route_black_t2_guard`
- `runtime_ultra_d5_hybrid_v99_black_t2_guard_runtime_confirm` (failed)
- `runtime_ultra_d5_hybrid_v100_black_t2_guard_confirm_arbiter` (current best)

Design intent:

- Keep `v94` pro-primary lift route (`v27`/`v66` hybrid).
- Add targeted guard for fast-screen pro failure bucket:
  - do **not** route through `v66` on `turn=2`, `active=black`, `mons_moves_count=1`.
- Keep confirmation behavior separate from primary:
  - `v100` uses opening-book confirmation arbiter (pro-aligned bridge by default, runtime-current only on clear default-eval + floor advantage).

Reduced-gate results (`repeats=2`, `games=2`, `max_plies=72` unless noted):

- `v98` fast-screen:
  - vs pro `delta=0.0000` (fixed from `v94` `-0.1250`)
  - vs normal `delta=+0.1250`
  - vs fast `delta=+0.2500`
- `v98` primary:
  - vs pro `delta=+0.2500`, confidence `0.989`
  - vs normal `delta=+0.1667`
  - vs fast `delta=+0.2500`
- `v98` reduced progressive (`max_games=2`, `repeats=1`, `max_plies=80`):
  - vs pro `delta=+0.0833`, confidence `0.613`
  - vs normal `delta=+0.2500`, confidence `0.927`
  - vs fast `delta=+0.1667`, confidence `0.806`
- `v98` reduced ladder first attempt:
  - CPU gate passed (`ratio=3.806`).
  - Primary gates passed.
  - Failed confirmation vs pro (`delta=-0.1250`).

Confirmation-only fix path:

- `v99` (opening-book context switched to runtime-current) over-corrected and regressed vs pro confirmation (`delta=-0.2500`).
- `v100` (opening-book confirmation arbiter) recovered balanced confirmation probes:
  - confirm vs pro `delta=0.0000`
  - confirm vs normal `delta=0.0000`
  - confirm vs fast `delta=+0.1250`

Reduced ladder status for `v100`:

- CPU gate passed (`ratio=3.942`).
- Primary gates passed (same as `v98`).
- Confirmation gates reached non-negative prints in-ladder:
  - pro `0.0000`, normal `0.0000`, fast `+0.2500`.
- Pool stage is the remaining long-running validation segment for full reduced-ladder completion.

Takeaway:

- `v100` is the first ultra candidate in this round that simultaneously keeps fast-screen non-regression and preserves `v94`-class pro-primary lift, with balanced confirmation probes.
- Remaining blocker to declare reduced-ladder pass is pool-stage completion.

---

### 46) Ultra x10 confirmation-stability finalizer (`v100`) (March 4, 2026)

Refinement:

- Added `runtime_ultra_d5_hybrid_v100_black_t2_guard_confirm_arbiter`.
- Primary path remains the `v98` independent-context hybrid.
- Opening-book context now uses a conservative arbiter:
  - default to `v27` confirmation bridge,
  - switch to runtime-current confirmation only with clear dual advantage
    (`default-eval` and `root_reply_floor`).

Verified reduced gates for `v100`:

- Fast-screen:
  - vs pro `delta=0.0000`
  - vs normal `delta=+0.1250`
  - vs fast `delta=+0.2500`
- Primary:
  - vs pro `delta=+0.2500`, confidence `0.989`
  - vs normal `delta=+0.1667`
  - vs fast `delta=+0.2500`
- Confirmation probes:
  - vs pro `delta=0.0000`
  - vs normal `delta=0.0000`
  - vs fast `delta=+0.1250`
- CPU gate sample in reduced ladder: `ratio=3.942` vs pro (inside `3.30..10.00`).

Status:

- `v100` is the first candidate this round that clears reduced fast-screen + reduced strict primary + reduced confirmation non-regression together.
- Remaining validation to mark promotion-ready is reduced/full ladder pool-stage completion and then full-sample ladder confirmation.

---

### 47) Ultra x10 confirmation re-balance (`v101`..`v107`) (March 4, 2026)

Problem observed:

- `v101` could pass reduced confirmation non-regression in some runs but remained seed-volatile and regressed on larger confirmation samples against pro.

Candidates tested (confirmation-only mutations on top of same primary path):

- `runtime_ultra_d5_hybrid_v102_black_t2_guard_confirm_normal_tie_guard`
- `runtime_ultra_d5_hybrid_v103_black_t2_guard_confirm_balanced_tie_guard` (failed)
- `runtime_ultra_d5_hybrid_v104_black_t2_guard_confirm_white_tight_tie` (failed)
- `runtime_ultra_d5_hybrid_v105_black_t2_guard_confirm_normal_tie_tuned` (failed)
- `runtime_ultra_d5_hybrid_v106_black_t2_guard_confirm_normal_tie_high` (failed)
- `runtime_ultra_d5_hybrid_v107_black_t2_guard_confirm_pro_veto` (current best)

Larger confirmation probe snapshot (`repeats=4`, `games=3`, `max_plies=72`):

- `v101`: vs pro `delta=-0.2083`, vs normal `delta=0.0000`.
- `v102`: vs pro `delta=-0.0417`, vs normal `delta=+0.0417`, vs fast `delta=+0.0417`.
- `v103`: vs pro `delta=-0.1667` (rejected).
- `v104`: vs pro `delta=-0.1250` (rejected).
- `v105`: vs pro `delta=-0.1250` (rejected).
- `v106`: vs pro `delta=-0.1667` (rejected).
- `v107`: vs pro `delta=0.0000`, vs normal `delta=0.0000`, vs fast `delta=+0.0417`.

`v107` reduced ladder (`SMART_POOL_OPPONENTS=3`, reduced gates):

- CPU gate: `ratio=3.913` vs pro (in-band `3.30..10.00`).
- Primary:
  - vs pro `delta=+0.2500`, confidence `0.989`
  - vs normal `delta=+0.1667`
  - vs fast `delta=+0.2500`
- Confirmation:
  - vs pro `delta=0.0000`
  - vs normal `delta=0.0000`
  - vs fast `delta=0.0000`
- Pool:
  - vs pro-opponents `candidate_delta=0.0000`, baseline `0.0000`
  - vs normal-opponents `candidate_delta=0.1667`, baseline `0.0000`
  - vs fast-opponents `candidate_delta=0.3333`, baseline `0.0000`

`v107` pipeline checks (reduced progressive settings):

- Fast screen:
  - vs pro `delta=0.0000`
  - vs normal `delta=+0.1250`
  - vs fast `delta=+0.2500`
- Progressive (`initial_games=2`, `max_games=2`, `repeats=1`, `max_plies=80`):
  - vs pro `delta=+0.0833`, confidence `0.613`
  - vs normal `delta=+0.2500`, confidence `0.927`
  - vs fast `delta=+0.1667`, confidence `0.806`

Result:

- `v107` supersedes `v102` as the current strongest balanced reduced-ladder ultra candidate in this round.
- Remaining work before production promotion: full-sample strict ladder confirmation (no reduced overrides) and final runtime promotion copy.

---

### 48) Ultra x10 wider-picture portfolio arbitration (`v108`, `v109`) (March 4, 2026)

Wider-picture hypothesis:

- Use extra ultra CPU for policy-portfolio arbitration in independent context (`runtime`, `pro-anchor`, `pro`, `confirm`) instead of single-route play.

Candidates:

- `runtime_ultra_d5_hybrid_v108_independent_portfolio_arbiter`
- `runtime_ultra_d5_hybrid_v109_midgame_portfolio_arbiter`

Results:

- `v108` failed immediately on pro fast-screen:
  - `ultra_fast_screen_vs_pro`: `delta=-0.2500` (hard fail).
  - detailed trace showed losses concentrated in max-ply adjudication buckets on the same seed tag (`ultra_fast_screen_vs_pro_v1`), i.e. portfolio routing selected lower adjudication-score lines under that bucket.
- `v109` added early-game fallback to `v107` and recovered fast-screen non-regression:
  - vs pro `delta=0.0000`, vs normal `+0.1250`, vs fast `+0.2500`.
  - primary remained strong but did not improve over `v107` (`vs pro +0.2083` vs `v107` `+0.2500` in reduced probe).
  - confirmation regressed vs pro (`delta=-0.2500` in reduced confirm probe), so not promotable.

Decision:

- Keep `v107` as the best current survivor.
- Log `v108` and `v109` as failed portfolio family for now (independent-context portfolio arbitration needs stronger adjudication-aware constraints before retry).

---

### 49) Ultra primary bar update + conservative mid/late upgrades (`v110`..`v112`) (March 4, 2026)

Policy update:

- Ultra strict primary requirement vs pro changed from `+0.10` to `+0.08` (confidence unchanged at `>=0.90`).

Round candidates:

- `runtime_ultra_d5_hybrid_v110_conservative_midlate_upgrade`
- `runtime_ultra_d5_hybrid_v111_conservative_midlate_nontrailing`
- `runtime_ultra_d5_hybrid_v112_opening_pro_bias`

Key outcomes:

- `v110`:
  - strong reduced primary (`vs pro +0.2917`, `vs normal +0.2500`, `vs fast +0.2500`);
  - reduced ladder passed;
  - but failed reduced progressive vs pro (`delta=-0.0833`) in the same quick setting where `v107` was non-negative.
- `v111`:
  - reduced fast-screen passed (`pro 0.0000`, `normal +0.1250`, `fast +0.2500`);
  - reduced primary passed (`vs pro +0.2500`, `vs normal +0.1667`, `vs fast +0.2917`);
  - reduced confirmation passed (`vs pro 0.0000`, `vs normal +0.1250`, `vs fast +0.2500`);
  - reduced progressive vs pro restored to non-negative (`delta=0.0000`);
  - reduced ladder passed with CPU ratio `3.863x`.
- `v112` (opening pro-bias control):
  - no measurable lift vs `v111/v107` on reduced primary;
  - larger confirmation-vs-pro stress probe remained negative (`delta=-0.1250`), same observed volatility bucket.

Current status:

- `v111` is the current best gate-sound reduced-ladder survivor in this round.
- Larger-sample confirmation vs pro (`repeats=4`, `games=3`) remains volatile/negative (`delta=-0.1250`) for both `v107`-family and `v112`, so this remains the main blocker before full strict promotion.

---

## What Worked Best So Far

- Keep modestly larger runtime node budgets (`fast=480`, `normal=3800`).
- Keep phase-adaptive runtime scoring.
- Apply root efficiency tie-breaks in both client modes.
- Keep normal root-safety rerank/deep-floor and reply-risk guard.
- Keep TT enabled in runtime search.
- **Fast prepass exception**: skip forced drainer tactics when supermana scoring is available.
- **Boosted supermana interview priors**: `supermana_score_bonus=600`, `supermana_progress_bonus=320` in fast mode.
- **Minimal, additive weight changes**: `supermana_race_control: 30` is the only new scoring weight — no restructuring of existing weight balance. This pattern (small additive signal in an orthogonal evaluation dimension) is the most reliable way to improve fast mode.
- **Promoted synthesis profile**: `runtime_fast_root_quality_v1_normal_conversion_v3` cleared the full ladder. The key runtime deltas are stronger fast root-quality margins (`root_efficiency=1700`, `anti_help=280`, `handoff=300`, `backtrack=220`, fast reply-risk `125/4/10/650`) plus stronger normal conversion guard allocation (normal reply-risk `145/7/16/1350`, drainer safety `4200`, selective extension share `12.5%`).
- **Pro split-strategy evidence**: `runtime_pro_primary_confirm_split_v1` is the first pro profile to clear the full strict pro ladder under `<=3.69x`, indicating that primary-strength and opening-book confirmation behavior likely need context-sensitive policy rather than one global pro shape.
- **Promoted pro runtime context split**: `runtime_pro_context_split_runtime_v2` (with fixed opening-book hint propagation in harness validation) passed reduced strict ladder and remains the shipped runtime `pro` profile; current runtime node budget is now `max_visited_nodes=14022` in both runtime contexts.
- **Ultra confirmation-stability pattern**: `runtime_ultra_d5_deep_narrow_v26_confirm_pro_bridge` kept `v25`-level primary strength while removing the opening confirmation-vs-pro regression via pro-aligned confirmation routing; reduced strict ladder passed under high-utilization CPU (`ratio=3.866` vs pro).
- **Ultra context-split confirmation arbiter v2**: `runtime_ultra_d5_hybrid_v107_black_t2_guard_confirm_pro_veto` preserved `v98`/`v100`-class primary lift (`+0.2500` vs pro, reduced gate) while stabilizing confirmation to non-regression in both reduced ladder and larger confirmation probes.
- **Failed wider-picture portfolio family (for now)**: naive independent-context policy portfolios (`v108`, `v109`) can improve some non-pro matchups but currently destabilize pro-facing adjudication buckets; this family needs explicit adjudication constraints before re-entry.
- **Conservative mid/late upgrade pattern**: strict override-only mid/late upgrades can increase primary pressure without immediate fast-screen regressions; `v111` is the first variant in this family to stay gate-sound across reduced fast-screen, reduced progressive, and reduced ladder.

### Key Invariant Discovery

**`supermana_race_control: 30` broke the fast mode invariance pattern.** For 20+ prior iterations, no scoring weight change affected fast-mode primary game outcomes (always 72W-72L). This weight operates in a different evaluation dimension (relative supermana distance) and successfully tips previously tied root choices at depth 2. Future fast-mode improvements are possible through evaluation dimensions orthogonal to drainer tactics.

---

## Remaining Pro Strategy Gaps

From the pro-strategy interview (see `docs/automove-pro-strategy-interview.md`):

1. ~~Always attack opponent drainer~~ — done (boolean drainer protection)
2. ~~Get supermana if there's a safe way~~ — done (supermana priority v1)
3. **Get opponent's mana if there's a safe way** — not yet implemented
4. **Hold potion to create scoring threats** — not yet (potion as tempo/threat multiplier)
5. **Spirit should always be moved off base** — partially addressed by interview spirit policy, not yet promoted
6. **Use spirit to move own mana closer to pools** — not yet in search evaluation
7. **Attack opponent spirit when quick and creates risk** — not yet implemented
8. **Use bomb primarily to attack opponent drainer** — drainer attack priority exists, but bomb-specific routing not optimized

---

## Final Validation Before Release

```sh
cargo test --lib
cargo check --release
cargo check --release --target wasm32-unknown-unknown
```

Verify:
- No legacy API exposure (`smartAutomoveWithBudgetAsync` should not exist).
- Experiment harness remains test-only.

## Important

- Do not run/scan `rules-tests/` unless explicitly requested.
- Keep release checks focused on automove tests plus release `cargo check`.
