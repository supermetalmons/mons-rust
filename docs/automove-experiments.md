# Smart Automove Experimentation Guide

This document is the entry point for iterating on automove strength safely and quickly.

Pro-player strategy interview notes used as iteration input are tracked in `docs/automove-pro-strategy-interview.md`.

## What Is Shipped Right Now

Public API:

- `smartAutomoveAsync("fast")`
- `smartAutomoveAsync("normal")`

Current runtime behavior:

- `fast` is CPU-shaped around `depth=2/max_nodes=480`.
- `normal` is CPU-shaped around `depth=3/max_nodes=3800`.
- `fast` uses `RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS`.
- `normal` uses phase-adaptive runtime normal weights (`RUNTIME_NORMAL_*_SPIRIT_BASE_SCORING_WEIGHTS` family).
- `fast` and `normal` both use root efficiency tie-breaks (progress-aware, with soft no-effect/low-impact penalties).
- `normal` keeps root-safety rerank/deep-floor and now also uses root reply-risk guard (`score_margin=140`, shortlist `5`, reply-limit `12`, node-share cap `10%`).
- `fast` keeps reply-risk guard disabled for lower overhead.
- Root/child tactical class coverage uses strict guarantees for critical tactical classes before truncation.
- Root anti-help filtering rejects near-best mana-handoff/roundtrip roots when non-losing clean alternatives exist.
- Selective tactical extension is normal-only, capped to one extension per path with a dedicated node-share budget (`12%`).
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
- `SMART_GATE_BASELINE_PROFILE` (strict gate baseline, default `runtime_current`)
- `SMART_GATE_SPEED_POSITIONS`
- `SMART_GATE_BUDGET_DUEL_GAMES`, `SMART_GATE_BUDGET_DUEL_REPEATS`, `SMART_GATE_BUDGET_DUEL_MAX_PLIES`
- `SMART_GATE_BUDGET_DUEL_SEED_TAG`
- `SMART_GATE_PRIMARY_GAMES`, `SMART_GATE_PRIMARY_REPEATS`, `SMART_GATE_PRIMARY_MAX_PLIES`
- `SMART_GATE_CONFIRM_GAMES`, `SMART_GATE_CONFIRM_REPEATS`, `SMART_GATE_CONFIRM_MAX_PLIES`
- `SMART_GATE_POOL_GAMES` (pool non-regression stage in gate/ladder)
- `SMART_GATE_ALLOW_SELF_BASELINE` (`true` only for baseline artifact capture; disables `candidate != baseline` check)
- `SMART_TUNE_TRAIN_POSITIONS_PER_SEED`, `SMART_TUNE_HOLDOUT_POSITIONS_PER_SEED`
- `SMART_TUNE_ROOT_LIMIT`, `SMART_TUNE_TOP_K`, `SMART_TUNE_MANIFEST_OUTPUT`
- `SMART_TUNE_LABEL_DEPTH_BOOST`, `SMART_TUNE_LABEL_NODE_MULTIPLIER`
- `SMART_TUNE_FULL_GRID` (`true` = full +/-35% grid search; default quick local sweep)

Why `SMART_USE_WHITE_OPENING_BOOK` defaults to `false`:

- Production applies opening routes.
- Promotion experiments usually should compare search/eval quality directly, not opening-book luck.
- Enable it only when explicitly validating production-like opening behavior.

## Fast Iteration Loop

Use this loop for most work:

1. Speed + quick strength screen:
   - `SMART_FAST_PROFILES=runtime_current,<candidate_profile> SMART_FAST_BASELINE=runtime_current SMART_FAST_USE_CLIENT_MODES=true cargo test --lib smart_automove_pool_fast_pipeline -- --ignored --nocapture`
2. Quick mirrored duel screen (opening-book off), first orientation:
   - `SMART_DUEL_A=<candidate_profile> SMART_DUEL_B=runtime_current SMART_DUEL_GAMES=2 SMART_DUEL_REPEATS=2 SMART_DUEL_MAX_PLIES=72 SMART_DUEL_SEED_TAG=quick_v1 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`
3. Quick mirrored duel screen, reverse orientation:
   - `SMART_DUEL_A=runtime_current SMART_DUEL_B=<candidate_profile> SMART_DUEL_GAMES=2 SMART_DUEL_REPEATS=2 SMART_DUEL_MAX_PLIES=72 SMART_DUEL_SEED_TAG=quick_v1 cargo test --lib smart_automove_pool_profile_duel -- --ignored --nocapture`
4. Keep-going bar for small screens: aggregate delta win-rate `>= +0.04` before spending larger compute.
5. Run strict gate in reduced mode for quick feedback (includes tactical guardrails and CPU gate):
   - `SMART_CANDIDATE_PROFILE=<candidate_profile> SMART_GATE_BASELINE_PROFILE=runtime_current SMART_GATE_PRIMARY_GAMES=2 SMART_GATE_PRIMARY_REPEATS=2 SMART_GATE_CONFIRM_GAMES=2 SMART_GATE_CONFIRM_REPEATS=2 cargo test --lib smart_automove_pool_promotion_gate_v2 -- --ignored --nocapture`
6. Run the staged ladder (artifacts + early-stop + budget-conversion diagnostic):
   - `SMART_CANDIDATE_PROFILE=<candidate_profile> SMART_GATE_BASELINE_PROFILE=runtime_current SMART_LADDER_ARTIFACT_PATH=target/smart_ladder_artifacts.jsonl cargo test --lib smart_automove_pool_promotion_ladder -- --ignored --nocapture`
7. Only if reduced gate is promising, run the full strict promotion gate:
   - `SMART_CANDIDATE_PROFILE=<candidate_profile> SMART_GATE_BASELINE_PROFILE=runtime_current cargo test --lib smart_automove_pool_promotion_gate_v2 -- --ignored --nocapture`

## Useful Test Commands

Profile sweep:

- `SMART_POOL_GAMES=4 SMART_SWEEP_PROFILES=runtime_current,weights_balanced cargo test --lib smart_automove_pool_profile_sweep -- --ignored --nocapture`

Speed probe:

- `SMART_CANDIDATE_PROFILE=runtime_current SMART_SPEED_POSITIONS=20 cargo test --lib smart_automove_pool_profile_speed_probe -- --ignored --nocapture`

Runtime diagnostics:

- `SMART_DIAG_GAMES=4 SMART_DIAG_MODE=normal cargo test --lib smart_automove_pool_runtime_diagnostics -- --ignored --nocapture`

Fast-vs-normal head-to-head (same profile, different budgets):

- `SMART_BUDGET_DUEL_A=runtime_current SMART_BUDGET_DUEL_B=runtime_current SMART_BUDGET_DUEL_A_MODE=fast SMART_BUDGET_DUEL_B_MODE=normal SMART_BUDGET_DUEL_GAMES=3 SMART_BUDGET_DUEL_REPEATS=4 SMART_BUDGET_DUEL_MAX_PLIES=56 SMART_BUDGET_DUEL_SEED_TAG=fast_normal_v1 cargo test --lib smart_automove_pool_budget_duel -- --ignored --nocapture`

Eval tuning dataset export (test-only helper):

- `SMART_TUNE_PROFILE=runtime_current SMART_TUNE_POSITIONS=64 SMART_TUNE_ROOT_LIMIT=8 SMART_TUNE_SEED_TAG=eval_tune_v1 SMART_TUNE_OUTPUT_PATH=target/smart_eval_tuning_samples.jsonl cargo test --lib smart_automove_pool_export_eval_tuning_dataset -- --ignored --nocapture`

Train/holdout dataset suite export (board-eval workflow):

- `SMART_TUNE_PROFILE=runtime_current SMART_TUNE_TRAIN_POSITIONS_PER_SEED=256 SMART_TUNE_HOLDOUT_POSITIONS_PER_SEED=128 SMART_TUNE_ROOT_LIMIT=12 cargo test --lib smart_automove_pool_export_eval_tuning_dataset_suite -- --ignored --nocapture`

Deterministic coordinate-descent board-eval tuner:

- `SMART_TUNE_PROFILE=runtime_current SMART_TUNE_TRAIN_POSITIONS_PER_SEED=256 SMART_TUNE_HOLDOUT_POSITIONS_PER_SEED=128 SMART_TUNE_ROOT_LIMIT=12 SMART_TUNE_TOP_K=8 SMART_TUNE_MANIFEST_OUTPUT=target/eval_tune_ranked_candidates.json cargo test --lib smart_automove_pool_tune_eval_weights_coordinate_descent -- --ignored --nocapture`

Quick tuning mode (fast iteration):

- `SMART_TUNE_PROFILE=runtime_current SMART_TUNE_TRAIN_POSITIONS_PER_SEED=48 SMART_TUNE_HOLDOUT_POSITIONS_PER_SEED=24 SMART_TUNE_ROOT_LIMIT=10 SMART_TUNE_TOP_K=4 SMART_TUNE_LABEL_DEPTH_BOOST=0 SMART_TUNE_LABEL_NODE_MULTIPLIER=1 cargo test --lib smart_automove_pool_tune_eval_weights_coordinate_descent -- --ignored --nocapture`

Quality tuning mode (long run):

- `SMART_TUNE_PROFILE=runtime_current SMART_TUNE_TRAIN_POSITIONS_PER_SEED=256 SMART_TUNE_HOLDOUT_POSITIONS_PER_SEED=128 SMART_TUNE_ROOT_LIMIT=12 SMART_TUNE_TOP_K=8 SMART_TUNE_LABEL_DEPTH_BOOST=1 SMART_TUNE_LABEL_NODE_MULTIPLIER=2 SMART_TUNE_FULL_GRID=true cargo test --lib smart_automove_pool_tune_eval_weights_coordinate_descent -- --ignored --nocapture`

## Promotion Criteria

Candidate is considered promotable only when all are true:

- Primary strength gate (opening-book off):
  - Mirrored two-way duels across seed tags `neutral_v1`, `neutral_v2`, `neutral_v3`.
  - Duel settings: `SMART_DUEL_GAMES=4`, `SMART_DUEL_REPEATS=6`, client modes (`fast`, `normal`).
  - Aggregate delta win-rate vs `runtime_current` is `>= +0.12`.
  - Per-mode delta win-rate is `>= +0.08` for both `fast` and `normal`.
  - Aggregate confidence is `>= 0.90`.
- Production-like confirmation gate (opening-book on):
  - Mirrored two-way duel seed tag `prod_open_v1`.
  - Aggregate delta win-rate is `>= +0.05`.
  - Aggregate confidence is `>= 0.75`.
- CPU gate:
  - fast ratio `<= 1.08x`
  - normal ratio `<= 1.15x`
- Budget-conversion regression guard:
  - Run fast-vs-normal diagnostic for baseline and candidate inside promotion gate/ladder.
  - Candidate normal-edge (normal advantage over fast) must not regress vs baseline by more than `0.04`.
- Pool non-regression guard:
  - Candidate and baseline are each evaluated against `POOL_MODELS` under the same budgets.
  - Candidate must not beat fewer pool opponents than baseline.
  - Candidate aggregate pool win-rate must not be more than `0.01` below baseline.

Official command:

- `SMART_CANDIDATE_PROFILE=<candidate_profile> SMART_GATE_BASELINE_PROFILE=runtime_current cargo test --lib smart_automove_pool_promotion_gate_v2 -- --ignored --nocapture`

## Candidate Profiles To Know

- `runtime_current`: currently shipped behavior.
- `runtime_pre_efficiency_logic`: same runtime budgets/scoring as `runtime_current`, but with fast root-efficiency tie-break disabled.
- `runtime_pre_fast_efficiency_cleanup`: legacy fast runtime for this cleanup iteration (root efficiency and backtrack penalty enabled in fast mode).
- `runtime_pre_root_reply_floor`: legacy alias; reply-floor logic was removed from runtime root selection.
- `runtime_pre_event_ordering`: baseline with event-aware root/child ordering bonus disabled.
- `runtime_pre_backtrack_penalty`: baseline with fast root roundtrip/backtrack penalty disabled.
- `runtime_pre_drainer_tactical_requirements`: baseline with forced drainer-attack root filtering and drainer-safety root prefilter disabled.
- `runtime_pre_root_upgrade_bundle`: baseline with all three root upgrades above disabled.
- `runtime_pre_move_efficiency`: snapshot before current node-budget/runtime-shape increase (`fast=420`, `normal=3450`).
- `runtime_pre_fast_drainer_priority`: snapshot before current fast drainer-context promotion (uses fast `RUNTIME_RUSH` baseline).
- `runtime_pre_winloss_weights`: snapshot before current rush-scoring promotion.
- `runtime_pre_tactical_runtime`: snapshot before current tactical-runtime scorer promotion.
- `runtime_pre_transposition`: snapshot before TT-enabled search path.
- `runtime_pre_normal_x15`: snapshot before normal 1.5x budget/runtime-shape update.
- `runtime_normal_efficiency_reply_floor`: current promoted normal root-efficiency/backtrack path.
- `runtime_pre_normal_efficiency_reply_floor`: snapshot before promoting normal root-efficiency/backtrack in runtime.
- `runtime_pre_drainer_context`: snapshot before current fast drainer-context promotion.
- `runtime_legacy_phase_adaptive`: older legacy reference.
- `runtime_drainer_context`: fast-only drainer-context candidate path.
- `runtime_d2_tuned`: older fixed-weight reference.
- `runtime_eval_board_v1`: board-eval candidate profile hook (for tuned board-weight promotion runs).
- `runtime_eval_board_v2`: board-eval candidate profile hook (second tuned board-weight slot).
- `runtime_fast_env_tune_normal_x15_tactical_lite`: test-only fast env-tuning hook (normal branch fixed to `runtime_normal_x15_tactical_lite`) for rapid fast toggle/shape sweeps.

Fast env-tuning example:

- `SMART_DUEL_A=runtime_fast_env_tune_normal_x15_tactical_lite SMART_DUEL_B=runtime_current SMART_DUEL_MODE=fast SMART_DUEL_GAMES=2 SMART_DUEL_REPEATS=2 SMART_DUEL_MAX_PLIES=72 SMART_FAST_TUNE_NODE_SCALE_BP=10500 SMART_FAST_TUNE_EVENT_ORDERING=false SMART_FAST_TUNE_BACKTRACK=true SMART_FAST_TUNE_MOVE_CLASS=true SMART_FAST_TUNE_CHILD_MOVE_CLASS=false SMART_FAST_TUNE_SPIRIT_PREF=true SMART_FAST_TUNE_FORCED_PREPASS=true SMART_FAST_TUNE_REPLY_GUARD=false cargo test --release --lib smart_automove_pool_profile_duel -- --ignored --nocapture`

## Board Eval Workflow (Recommended)

1. Freeze baseline evidence:
   - `cargo test --lib smart_automove_tactical_suite -- --ignored --nocapture`
   - `SMART_CANDIDATE_PROFILE=runtime_current SMART_SPEED_POSITIONS=20 cargo test --lib smart_automove_pool_profile_speed_probe -- --ignored --nocapture`
   - `SMART_BUDGET_DUEL_A=runtime_current SMART_BUDGET_DUEL_B=runtime_current SMART_BUDGET_DUEL_A_MODE=fast SMART_BUDGET_DUEL_B_MODE=normal SMART_BUDGET_DUEL_GAMES=3 SMART_BUDGET_DUEL_REPEATS=4 SMART_BUDGET_DUEL_MAX_PLIES=56 SMART_BUDGET_DUEL_SEED_TAG=fast_normal_v1 cargo test --lib smart_automove_pool_budget_duel -- --ignored --nocapture`
2. Export train/holdout datasets with root labels.
3. Run coordinate-descent tuning to produce `target/eval_tune_ranked_candidates.json`.
4. Map best ranked bundle into a named profile (`runtime_eval_board_v1` / `runtime_eval_board_v2`).
5. Run strict ladder and strict gate against `runtime_current`:
   - `SMART_CANDIDATE_PROFILE=runtime_eval_board_v1 SMART_GATE_BASELINE_PROFILE=runtime_current SMART_LADDER_ARTIFACT_PATH=target/smart_ladder_artifacts_board_v1.jsonl cargo test --lib smart_automove_pool_promotion_ladder -- --ignored --nocapture`
   - `SMART_CANDIDATE_PROFILE=runtime_eval_board_v1 SMART_GATE_BASELINE_PROFILE=runtime_current cargo test --lib smart_automove_pool_promotion_gate_v2 -- --ignored --nocapture`
6. Promote runtime constants only after strict pass, keeping all tuner/export helpers test-only.

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

### 9) Aggressive `RUNTIME_RUSH` mana-pressure overlays

Idea:

- Add stronger immediate carrier and drainer-path urgency on top of `RUNTIME_RUSH_SCORING_WEIGHTS` to force quicker scoring.

What happened:

- In repeated duels this regressed versus baseline runtime behavior.
- It overcommitted in tactical spots and did not improve aggregate strength.

Takeaway:

- Keep rush profile stable; large urgency overlays were too brittle.

### 10) Hard no-effect pruning / heavy no-effect penalties

Idea:

- Detect no-effect turn transitions and aggressively remove or heavily penalize them.

What happened:

- Full pruning and high penalties reduced stability and hurt aggregate duel results.
- Soft/no-op handling performed better than aggressive filtering.

Takeaway:

- Do not hard-prune no-effect transitions; if used, keep this signal very light.

### 11) Fast root efficiency tie-break (light, fast-only)

Idea:

- Keep the main search/scoring path intact.
- Add a small fast-mode root tie-break that prefers clearer progress (carrier/drainer path improvements) and softly penalizes no-effect/low-impact transitions.
- Keep normal mode on the pre-efficiency selector path for stability.

What happened:

- Versus `runtime_pre_efficiency_logic` in fast-only duels, results were positive in aggregate but noisy per seed.
- Fast short-seed set (`5` tags, `6` games/tag): `19W-11L`.
- Fast longer-seed set (`3` tags, `15` games/tag): `23W-22L`.
- Combined fast aggregate across those runs: `42W-33L` (win rate `0.560`).
- Additional larger-seed pair (`2` tags, `25` games/tag): `27W-23L` (win rate `0.540`).
- Normal-mode spot checks were near neutral/slightly positive (for example `2W-1L` in one `3`-game seed).

CPU impact:

- `SMART_SPEED_POSITIONS=20` showed near-parity runtime:
  - `runtime_current`: fast `~54.55ms`, normal `~364.50ms`.
  - `runtime_pre_efficiency_logic`: fast `~54.51ms`, normal `~368.65ms`.

Takeaway:

- Light root efficiency signals can improve practical fast-mode move quality without material CPU cost.
- Keep this as a small, conservative layer; avoid aggressive penalties or child-search rewrites.

### 12) Same-budget self-label board-eval tuning

Idea:

- Export move-regret labels from the same runtime search budget and tune eval weights to minimize those regrets.

What happened:

- Coordinate-descent repeatedly converged to near-baseline weights.
- Ranked candidates were often identical or effectively equivalent to runtime defaults.

Takeaway:

- Same-budget self-labeling is weak supervision; use deeper-label targets (`SMART_TUNE_LABEL_DEPTH_BOOST`, `SMART_TUNE_LABEL_NODE_MULTIPLIER`) for meaningful signal.

### 13) Full-grid board-eval tuning by default

Idea:

- Run full `+/-35%` grid search for every tuned field in each family on every iteration.

What happened:

- Iteration time became too high for practical screening loops.
- It was useful for final quality runs but too slow for daily candidate churn.

Takeaway:

- Keep quick mode as default local sweep and reserve `SMART_TUNE_FULL_GRID=true` for final quality passes.

### 14) `runtime_normal_x15_tactical_lite` as direct promotion candidate

Idea:

- Use the strongest quick-screen candidate (normal tactical-lite shape) directly against `runtime_current`.

What happened:

- CPU stayed within gate limits and quick-screen looked strong.
- Strict gate failed at primary fast mode (no required fast delta).

Takeaway:

- Normal-only gains are not enough for strict promotion.
- Candidate selection must include explicit fast-mode proof on primary seed tags.

### 15) Fast-side micro-patches (reply-risk lite, simplified root heuristics)

Idea:

- Improve fast by toggling small fast-only controls:
  - enable light reply-risk guard
  - disable selected root heuristics (event-ordering/backtrack/class coverage/spirit pref)

What happened:

- Reply-risk variants were either neutral on strength or over fast CPU mode-ratio cap in screens.
- Simplified-root variants were neutral-to-worse in mirrored checks.

Takeaway:

- Small heuristic toggles alone did not produce robust fast-mode lift.
- Keep these as experiment-only probes; do not promote by quick-screen signal.

### 16) Fast depth-3 lite probe under branch caps

Idea:

- Try bounded `depth=3` for fast with tight branch caps and fast-specific scoring.

What happened:

- Fast mode-ratio exploded in screening (`>2x` in mode ratio), violating CPU expectations.

Takeaway:

- Depth bump for fast is not viable under current CPU caps unless search shape is fundamentally redesigned.

### 17) Interview policy candidates (`runtime_interview_policy_v1..v5`)

Idea:

- Translate interview priorities into mixed hard/soft runtime policy:
  - hard drainer-attack priority
  - hard/conditional spirit-off-base preference
  - deterministic root tie-break ordering
  - optional soft supermana/opponent-mana progress priors
- Validate both direct interview variants (`v1..v3`) and mode-split variants (`v4`, `v5`).

What happened:

- Mixed-mode neutral-seed screens for `runtime_interview_policy_v1`, `runtime_interview_policy_v4`, and `runtime_interview_policy_v5` were noisy and stayed around neutral in aggregate.
- After keeping default runtime weights unchanged (no implicit promotion), rerunning `runtime_interview_policy_v5` on `neutral_v1..v3` with `SMART_DUEL_GAMES=1`, `SMART_DUEL_REPEATS=1`, `SMART_DUEL_MAX_PLIES=48` was negative:
  - `neutral_v1`: `1W-1L`
  - `neutral_v2`: `1W-1L`
  - `neutral_v3`: `0W-2L`
  - aggregate: `2W-4L`
- A safety cleanup was kept in runtime interview filtering: hard spirit-deploy now preserves explicit same-turn scoring exceptions and prefers safe deploy alternatives when the interview spirit rule is enabled.

Takeaway:

- Interview policy priors are directionally useful but not sufficient for strict promotion under current search shape/caps.
- Current evidence does not support promoting interview profiles over `runtime_current`.
- Keep these profiles as experiment hooks; require larger-seed strict gate evidence before runtime promotion.

## What Worked Best So Far

Current promoted direction:

- Keep modestly larger runtime node budgets (`fast=480`, `normal=3800`) versus prior runtime (`420`/`3450`).
- Keep phase-adaptive runtime scoring:
  - `fast`: `RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS`
  - `normal`: `RUNTIME_NORMAL_*_SPIRIT_BASE_SCORING_WEIGHTS` family
- Apply root efficiency tie-breaks in both client modes.
- Keep normal root-safety rerank/deep-floor and reply-risk guard.
- Keep TT enabled in runtime search, but validate with de-biased two-way duels.
- Keep opening-route policy in production, but disabled by default in promotion experiments.

Observed runtime-shape benchmark vs `runtime_pre_move_efficiency`:

- De-biased two-way duel, seed `nodes3800_verify`: `9W-7L` for `runtime_current`.
- De-biased two-way duel, seed `nodes3800_verify2`: `9W-7L` for `runtime_current`.
- De-biased two-way duel, seed `nodes3800_verify3`: `10W-6L` for `runtime_current`.
- Combined across these seed tags: `28W-20L` (win rate `0.583`).
- Speed probe (`SMART_SPEED_POSITIONS=30`) remained close:
  - `runtime_current`: fast `~53.8ms`, normal `~366.4ms`.
  - `runtime_pre_move_efficiency`: fast `~52.6ms`, normal `~367.5ms`.

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

Observed fast-efficiency benchmark vs `runtime_pre_efficiency_logic`:

- Fast-only short seeds (`SMART_DUEL_GAMES=3`, `SMART_DUEL_REPEATS=2`, `SMART_DUEL_MAX_PLIES=56`):
  - `eff_logic_fast_final_s1`: `4W-2L`
  - `eff_logic_fast_final_s2`: `4W-2L`
  - `eff_logic_fast_final_s3`: `2W-4L`
  - `eff_logic_fast_final_s4`: `5W-1L`
  - `eff_logic_fast_final_s5`: `4W-2L`
  - Combined short-seed aggregate: `19W-11L` (win rate `0.633`)
- Fast-only longer seeds (`SMART_DUEL_REPEATS=5`):
  - `eff_logic_fast_long`: `7W-8L`
  - `eff_logic_fast_long2`: `9W-6L`
  - `eff_logic_fast_long3`: `7W-8L`
  - Combined long-seed aggregate: `23W-22L` (win rate `0.511`)
- Combined fast aggregate across all above: `42W-33L` (win rate `0.560`)
- Fast-only larger-game seeds (`SMART_DUEL_GAMES=5`, `SMART_DUEL_REPEATS=5`):
  - `eff_logic_fast_bulk1`: `14W-11L`
  - `eff_logic_fast_bulk2`: `13W-12L`
  - Combined bulk aggregate: `27W-23L` (win rate `0.540`)

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
