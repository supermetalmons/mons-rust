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

Current runtime behavior:

- `fast` is CPU-shaped around `depth=2/max_nodes=480`.
- `normal` is CPU-shaped around `depth=3/max_nodes=3800`.
- `fast` uses `RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF` (boolean drainer danger, `-400`/`-300`, plus `supermana_race_control: 30`).
- `normal` uses phase-adaptive boolean drainer weights (`RUNTIME_NORMAL_BOOLEAN_DRAINER_*_SPIRIT_BASE_SCORING_WEIGHTS` family), switching by game phase.
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
