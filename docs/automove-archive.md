# Automove Archive

This document keeps compressed history for retired automove profiles and experiment waves. These IDs are archived context only. They are not part of the active experiment registry and should not be used for new promotion decisions.

## Retired Runtime Snapshots

- `runtime_historical_0_1_109`
  Fast and normal settings proved that reply-risk guardrails, tactical prepass, and stronger drainer safety mattered more than wider search.
- `runtime_historical_0_1_110`
  Helped validate the opening-reply latency problem and the need for stricter pro-context tuning instead of one shared pro shape.
- `runtime_historical_post_0_1_110_6c3d5cb`
  Showed that post-0.1.110 pro tuning could recover strength, but still needed cleaner promotion discipline.
- `runtime_historical_post_0_1_110_a70b842`
  Confirmed the stronger pro budget direction, but remained a historical checkpoint rather than a stable baseline.
- `runtime_historical_pre_exact_e9a05ce`
  Was the strongest archived pre-exact pro wave, but it is now historical context rather than an active baseline.

## Retired Candidate Line

- `runtime_eff_non_exact_v3`
  This hybrid line mixed current fast/normal with archived pre-exact pro behavior. It answered a one-time comparison question and is now retired because keeping it selectable in the active registry only adds noise.
- `runtime_root_signal_v1`
  This March 9, 2026 low-CPU signal wave improved cheap safe-progress detection, but it failed `fast-screen` against `runtime_release_safe_pre_exact` at `24` games with aggregate delta `-0.0417`. Keep the lesson that cheap safety-aware setup signals are useful, but do not promote a signal-only wave without stronger reply selection.
- `runtime_root_reply_v1`
  This March 9, 2026 low-CPU reply-selection wave cleared `preflight` and a constrained `fast-screen`, but follow-up diagnostics did not show promotable normal-side strength. Focused mode checks showed candidate `fast` at `+0.0208` vs baseline `fast` and `0.0000` vs baseline `normal` over `48` games each, while candidate `normal` ran `-0.0417` vs baseline `fast` over `48` games and `-0.0938` vs baseline `normal` after `32` games before the report was stopped. Keep the lesson that cheap reply-risk cleanup can be acceptable in `fast`, but do not carry the aggressive normal shortlist and penalty tuning forward.
- `drainer_exposure_v1`
  This March 9, 2026 heuristic penalty wave added a root `drainer_exposure_penalty` (200 fast, 300 normal) for uncompensated own-drainer-vulnerable moves plus wider `root_drainer_safety_score_margin` (2800 fast, 5200 normal). Cleared `preflight` with zero CPU cost and passed `fast-screen` at δ=+0.0089 (112 games, conf=0.538). Progressive duel showed δ=-0.0069 at 144 games with fast dead even (36-36) and normal slightly behind (35-37). The penalty alone at these magnitudes does not shift move selection often enough to yield measurable strength. Keep the lesson that the existing late-stage drainer-safety filter already handles most exposure cases; a heuristic penalty on top adds noise without signal. Future drainer work should focus on harder filtering or combining exposure awareness with other signals rather than standalone penalties.

## Removed Registry Aliases

- `runtime_efficient_v1`
  Removed from the active registry on March 9, 2026 because it was only a compatibility alias for `runtime_eff_non_exact_v1`. Use the canonical ID instead.
- `runtime_pre_pro_promotion_v1`
  Removed from the active registry on March 9, 2026 because it was a no-op duplicate of `runtime_current`. Use `runtime_current` for that behavior.

## Wave 3: Config Knob Exhaustion (Mar 9–16, 2026)

This wave tested every remaining SmartSearchConfig structural feature across all three modes, plus scoring weight tuning, new evaluation code (history heuristic, quiescence search), and null-move pruning. One promotion resulted (Normal no-extensions, +19.4%). The remaining config knob space is completely exhausted.

### Promoted

- `runtime_normal_no_extensions_v1`
  Disabled `enable_selective_extensions` for Normal depth≥3. +19.4% Normal mode strength (50W-22L, δ=+0.1944, conf=0.999 in progressive; 144 games). Extensions at depth-3 wasted budget on deep tactical paths that rarely changed the root decision. Breadth-over-depth evaluation is stronger. Production change: `enable_selective_extensions = false` in Normal branch.

### Killed — Pro Structural

- `runtime_pro_deeper_extensions_v1` (no extensions)
  Strongest candidate ever found: +13% across 1,488 games. Failed CPU ratio gate (0.607–1.346 vs 1.60 minimum). After Normal no-ext promotion, re-tested and failed confirmation (vs_normal δ=-0.1875).
- `runtime_pro_more_extensions_v1` (double-depth extensions)
  Pro-fast-screen vs_normal δ=+0.375 (strongest Pro vs_normal ever) but vs_fast δ=-0.250. Instability from starving other roots.
- `runtime_pro_no_quiet_reductions_v1` (flat search: no extensions + no quiet reductions)
  Progressive vs_normal δ=+0.1237 (744 games, conf=1.000) but vs_fast δ=-0.0556. All Pro extension×quiet-reduction combinations exhausted.
- `runtime_pro_no_futility_v1` — pro-triage 0/12.
- `runtime_pro_killer_ordering_v1` — pro-triage 0/12.
- `runtime_pro_search_combo_v1` (killer + no-futility + PVS) — pro-triage 0/12, audit noise.
- `runtime_pro_wider_roots_v1` (root_focus_k 3→4) — pro-triage 0/5.
- `runtime_pro_tight_ext_budget_v1` (ext budget 1500→800) — pro-triage 0/12.

### Killed — Normal Structural

- `runtime_normal_no_safety_rerank_v1` — triage 0/14.
- `runtime_normal_no_two_pass_v1` — triage 0/14.
- `runtime_normal_no_prepass_v1` — triage 0/11 (prepass never fires in fixtures).
- `runtime_normal_iterative_deepening_v1` — triage 0/18.
- `runtime_normal_wider_reply_v1` — fast-screen δ=0.000.
- Normal aspiration windows — triage 0/18+0/3.
- Normal disable move class coverage — triage 0/18.

### Killed — Normal Scoring Weight Tuning

All scoring weight changes pass triage by inflating heuristic scores but are noise at duel scale:
- `runtime_normal_spirit_race_v1` — progressive FadingSignal (±30 weight tweaks).
- `runtime_normal_scoring_upgrade_v1` (multi-path + mana-race activation) — fast-screen δ=0.000.
- `runtime_normal_tactical_finish_v1` (drainer path + score window boost) — fast-screen δ=0.000.
- `runtime_normal_node_boost_v1` (scoring + 25% nodes) — fast-screen δ=-0.0625.
- `runtime_normal_supermana_boost_v1` (interview_soft boost) — fast-screen δ=0.000.
- `runtime_attacker_proximity_v1` (scoring-only) — fast-screen δ=-0.0625 (three attempts).
- `runtime_attacker_futility_v1` (scoring + futility) — same outcomes as scoring-only.

### Killed — Normal Search/Structural

- `runtime_normal_tactical_focus_v1` (PVS + quiet reductions + event ordering + boosted interview-soft) — fast-screen δ=0.000.
- `runtime_normal_futility_pruning_v1` — triage 0/4.
- Normal exact tactics (static exact, root exact, child exact, exact-lite) — CPU 1.4–5.3x, triage or audit negative.
- Normal dormant feature sweep (12 single-variable tests) — all 0/N or audit δ=0.000.

### Killed — Fast Mode

- `runtime_fast_clean_reply_risk_v1` — fast-screen noisy then clearly weaker at 120 games.
- `runtime_fast_root_alloc_v1` — triage 0/3.
- `runtime_fast_boolean_drainer_v1` — triage+audit 0/N, δ=0.000.
- `runtime_fast_clean_reply_pref_v1` — triage 0/3.
- `runtime_fast_no_quiet_reductions_v1` — triage 0/14 (depth too shallow).
- `runtime_fast_two_pass_v1` — triage 0/14+0/3+0/3+0/2 (all surfaces invisible).
- `runtime_fast_spirit_deploy_v1` — fast-screen δ=0.000.

### Killed — New Code (History Heuristic, Quiescence, Null Move, LMR)

- `runtime_normal_history_heuristic_v1` — triage 0/18. Depth-3 tree too shallow for ordering-only improvements.
- `runtime_pro_history_heuristic_v1` — pro-triage 0/3+0/12. TT+killer bonuses already dominate ordering.
- `runtime_normal_quiescence_v1` — progressive pass (46W-26L, δ=0.139) but ladder pool fail. CPU tension: budget≥200 for signal but gate limits to ~30.
- `runtime_pro_quiescence_v1/v2` — in-progress (last tested idea).
- `runtime_normal_frontier_lmp_v1` (late move pruning) — fast-screen δ=0.000.
- `runtime_normal_full_pool_v1` (move pool reuse) — triage 0/N on all surfaces.
- `runtime_normal_medium_eval_v1` — fast-screen δ=0.000.
- `runtime_normal_null_move_v1` — triage 0/18.
- `runtime_pro_null_move_v1` — pro-progressive pass but pro-ladder pool non-regression fail.

### Infrastructure Completed

- Pro triage fixture expansion: 9→12 fixtures (4 close-decision + 3 human-win-game positions).
- Normal fixture expansion: 4→18 opponent_mana fixtures (depth-disagreement positions, close-decision positions).
- Fast fixture expansion: 0→4 Fast-mode opponent_mana fixtures.
- Normal sensitivity probe: 300 positions × 10 perturbations, found 25 sensitive positions.
- Pro sensitivity probe: 50 positions × multiple perturbations, found extension-sensitive positions.
- Fast sensitivity probe: 500 positions × 12 perturbations, only 2% sensitivity.
- Audit-driven fixture creation from random game positions and human-win games.
- Depth-disagreement probe for Normal (21/100 positions where Normal≠Pro).

## Mistakes Not To Repeat

- Do not keep historical profiles live in the active registry after their lesson is absorbed.
- Do not treat `target/experiment-runs` as durable memory.
- Do not revive archived candidates unless there is a specific new hypothesis that cannot be tested with the active profile surface.
- Do not widen the active experiment surface faster than you can maintain the runbook and guardrail tests.
