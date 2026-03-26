# Automove Archive

This document keeps compressed history for retired automove profiles and experiment waves. These IDs are archived context only. They are not part of the active experiment registry and should not be used for new promotion decisions.

## Wave 6: Pro Intent Planner V2 Stabilization (Mar 19-20, 2026)

- Candidate: `runtime_pro_intent_planner_v2`.
- What changed:
  - hybrid intent-first planner work in `automove_turn_planner`
  - candidate-only root injection hooks
  - emergency-only injected-root acceptance and diagnostics
  - additional Pro pool and reliability probes
- Stable shape that survived the loop:
  - `turn_planner_intent_root_injection_limit=1`
  - `turn_planner_intent_root_max_heuristic_gap=200`
  - `turn_planner_intent_root_emergency_only=true`
- What passed:
  - `guardrails`
  - `pro-triage primary_pro`
  - `runtime-preflight`
  - bounded first-duel and bounded ladder speed checks
- Why it left the live frontier:
  - direct reliability samples against the baseline stayed flat
  - sampled losses showed `losses_with_disagreement=0`
  - the branch consumed a lot of operator attention without proving direct selector value
- Compressed lesson:
  - crisis-gated injection is safer than global injection
  - opening-book fallback ordering must stay ahead of Pro-specific branching
  - do not keep a long-form branch diary live once the branch stops being the main frontier

## Wave 7: Pro Turn-Engine Compression (Mar 20-24, 2026)

- Scope: `runtime_pro_turn_engine_v2` through `runtime_pro_turn_engine_v30`.
- Why this wave happened:
  - the original `runtime_pro_turn_engine_v1` frontier proved the turn-engine direction but stayed too weak
  - the next wave tried to recover strength with a ProV2 engine, richer caching, selector diagnostics, and many narrow cross-budget wrappers
- Durable shared work retained from the wave:
  - opportunity-context extraction
  - best-plan / no-plan / continuation caching
  - config-fingerprinted cache reuse
  - selector utility and followup-floor caching
  - low-budget / eligibility / resume logic
  - Pro-aware `runtime-preflight`
  - `pro-reliability` as a canonical workflow stage
  - duel-progress logging and reusable selector / forced-root probes
- Compressed branch story:
  - `runtime_pro_turn_engine_v2`
    - first branch that showed the new ProV2 line could be materially stronger
    - main lesson: strength alone was not enough; the branch was too CPU-heavy to retain as the live selectable frontier
  - `runtime_pro_turn_engine_v25`
    - stable wrapper base that first cleared the fast-screen blocker family cleanly
    - main lesson: a small number of wrapper exceptions can recover real cross-budget lanes, but the family saturates quickly
  - `runtime_pro_turn_engine_v28` / `runtime_pro_turn_engine_v29` and similar neighbors
    - mostly narrow wrapper splits around traced openings
    - main lesson: keep the branch-local proof, archive the IDs, and do not leave these live in the active registry
  - `runtime_pro_turn_engine_v30`
    - retained frontier after the compression
    - cleared the earlier direct and fast-screen gates in the wave
    - did not finish the full earned path because progressive/larger proof was still incomplete
- Why the wave was compressed instead of promoted:
  - too many branch IDs accumulated
  - useful lessons had already moved into shared engine/workflow code
  - the next iteration needed one clear frontier, not another wrapper split family
- Retained decision:
  - archive `v2`..`v29`
  - keep `runtime_pro_turn_engine_v30` as the sole active ProV2 frontier
  - keep `runtime_pro_turn_engine_v1` only as reference history

## Wave 8: Stronger Pro Roadmap Follow-Through (Mar 25-26, 2026)

- Scope: `runtime_pro_turn_engine_v31` through `runtime_pro_turn_engine_v52`.
- Why this wave happened:
  - the stronger-Pro roadmap was used as a bounded follow-through program after the `v30` compression
  - the work stayed focused on finishing the retained frontier rather than opening a new permanent branch family
- What landed across the wave:
  - local child bundles and extra hotspot diagnostics
  - two-stage child ordering / shortlist safety
  - reach and payload cleanup in hot exact helpers
  - planner and oracle projection-profile narrowing
  - search-side and exact-oracle follow-up splits through `v52`
- Compressed lesson:
  - these IDs were useful follow-through evidence, not separate long-lived frontiers
  - `runtime_pro_turn_engine_v52_spirit_preview_no_board_summary_v1` became the strongest late-roadmap technical base
  - none of `v31`..`v52` finished `pro-reliability` in a practical promotion window, so the retained frontier stayed `runtime_pro_turn_engine_v30`
  - active work should stay in `AUTOMOVE_IDEAS.md`, with archived context here

## Wave 9: Post-Roadmap Exact Window Follow-Ups (Mar 26, 2026)

- Scope: `runtime_pro_turn_engine_v53` through `runtime_pro_turn_engine_v54`.
- Why this wave happened:
  - the late-roadmap blocker had snapped back to `exact_tactical_spirit_summary -> exact_best_immediate_tactical_window_on_board_with_hash`
  - the goal was to cut preview-local exact window work without reopening broader search-side churn immediately
- What landed across the wave:
  - `v53` added the spirit-preview saturation fast path
  - `v54` added a candidate-only cache for exact immediate tactical windows
- Compressed lesson:
  - `v54` was useful evidence: it improved bounded hotspot and stage-1 CPU numbers while keeping `guardrails`, `pro-triage`, and `runtime-preflight` green
  - neither `v53` nor `v54` finished `pro-reliability` in a practical window
  - the returned direct wall remained deeper in secure drainer recursion, payload churn, and remaining uncached exact-window work, so the retained frontier stayed `runtime_pro_turn_engine_v30`

## Wave 10: Exact Secure-Recursion Cache Follow-Ups (Mar 26, 2026)

- Scope: `runtime_pro_turn_engine_v55` through `runtime_pro_turn_engine_v56`.
- Why this wave happened:
  - `v54` proved the immediate tactical-window cache was real but not decisive
  - the next bounded question was whether the remaining wall was still mostly wrapper/setup work around secure drainer recursion
- What was tried:
  - `v55` added a secure-mana precheck against the existing exact secure-mana cache before synthetic game setup
  - `v56` added a secure drainer-walk metadata fast path to reuse cached transition results before reapplying the move
- Compressed lesson:
  - `v55` showed the wrapper was not the bottleneck: it generated many precheck hits, but hotspot wall time stayed flat or regressed (`primary_spirit_setup 322.20ms -> 324.37ms`, `human_win_pro_a 1273.92ms -> 1327.95ms`)
  - `v56` showed cached drainer-walk metadata was still the wrong cut for the returned boards (`primary_spirit_setup 407.55ms`, `human_win_pro_a 1525.18ms`, `spirit_development 612.71ms`)
  - both candidates were killed at the bounded hotspot stage and the code was discarded
  - next work should either go deeper into secure drainer recursion itself or shift back to search-side reuse; do not reopen the same exact cache-layer family without a materially different cut

## Wave 11: Secure-Recursion, Search-Summary, And Pickup-Window Follow-Ups (Mar 26, 2026)

- Scope: `runtime_pro_turn_engine_v59` through `runtime_pro_turn_engine_v62`.
- Why this wave happened:
  - `v55` and `v56` showed wrapper-local secure-recursion caches were the wrong cut
  - the next bounded question was whether one search-side reuse layer plus a deeper exact pickup-window cache could move the direct gate without reopening broader churn
- What was tried:
  - `v59` added a candidate-only secure-mana dead-end skip inside the exact secure recursion
  - `v60` re-enabled scoring board-summary reuse on top of the stronger exact base
  - `v61` added a candidate-only cache for exact drainer pickup windows inside the immediate tactical-window path
  - `v62` tried a deeper secure-mana specific-pickup prune on top of `v61`
- Compressed lesson:
  - `v59` kept `guardrails`, `pro-triage`, and `runtime-preflight` green, but it still did not finish `pro-reliability` in a practical window
  - `v60` kept `guardrails`, `pro-triage`, and `runtime-preflight` green but still did not finish `pro-reliability` in a practical window
  - `v61` was the strongest useful follow-up of the wave: it kept the front gates green and materially reduced bounded exact work, especially `payload_after_move`, by introducing `pickup_window_hits` on the returned wall
  - even with `v61`, the direct gate still ran past the practical window and was manually stopped after roughly two minutes, so the retained frontier stayed `runtime_pro_turn_engine_v30`
  - `v62` cut counted secure recursion but regressed the hotspot badly, so the code was discarded
  - the next exact split should target the remaining `payload_after_move` / uncached immediate-window / secure-recursion surface directly; do not reopen the discarded `v62` prune family

## Parked Campaign: Fast Tactical Uplift Against Current Normal (Mar 18-20, 2026)

- Goal: recover the `normal` vs `fast` gap without violating Fast latency.
- What was tried:
  - reply-risk envelope and tiebreak retunes
  - spirit-setup and drainer-safety top-offs
  - attacker-proximity and scoring-only lanes
  - exact-lite and tactical quiescence top-offs
  - no-reply-risk-guard safety variants
- Repeated pattern:
  - many splits failed triage outright
  - the ones that moved triage often stayed flat at the first earned duel
  - the strongest first-duel lanes hit progressive runtime cliffs and were still not promotable
- Decision:
  - no live Fast split remains from this wave
  - reopen Fast only with a genuinely new code path, not another micro-retune in the exhausted families

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

## Wave 4: Pro Confirmation Floor (Mar 14–15, 2026)

This wave focused on recovering Pro strength after the Normal promotion. Two idea families were exhausted and then retired.

### Closed — Pro Opening-Reply Fast-Policy Transplants

- Candidate family: `runtime_pro_opening_reply_fast_policy_v1..v8`.
- Typical pattern: deterministic `opening_reply` triage movement, clean fast-screen signal, then failure at ladder primary-vs-normal.
- Best directional outcomes still failed promote gates:
  - `runtime_pro_opening_reply_fast_policy_v3` passed minimized ladder CPU checks but failed primary gate (`delta 0.0000 < 0.0800`).
  - `runtime_pro_opening_reply_fast_policy_v2` and `v8` failed CPU ratio on minimized ladder.
- Decision: close opening-reply-only transplant line for now.

### Closed — Pro Primary Quiescence Shaping

- Retained anchor: `runtime_pro_quiescence_v2`.
- Split family tested: `runtime_pro_quiescence_v3..v14`.
- Repeated pattern:
  - `primary_pro` triage moved (`target_changed=3`, `off_target_changed=0`).
  - Fast-screen and bounded progressive remained positive.
  - Bounded ladder (`speed_positions=12`, `primary/confirm=3x3`, `max_plies=64`) failed in confirmation, usually vs fast.
- Hard blocker reproduced on multiple lines:
  - `runtime_pro_quiescence_v2` bounded ladder: `vs_normal -0.0556`, `vs_fast -0.1111` (tolerance `-0.10`).
  - `runtime_pro_quiescence_v11` bounded ladder: same floor (`vs_fast -0.1111`).
  - `runtime_pro_quiescence_v14` passed minimized ladder but failed bounded ladder (`vs_normal -0.1667`, `vs_fast -0.1111`).
- Decision: close quiescence-budget shaping line; minimized ladder is directional-only and not a keep gate.

### Workflow Outcome Preserved

- Added candidate-aware Pro opening-reply speed probe:
  - test: `smart_automove_pool_opening_reply_speed_probe`
  - stage: `./scripts/run-automove-experiment.sh pro-opening-speed-probe <candidate> [baseline]`
- This is retained as workflow infrastructure for future `opening_reply` ideas.

## Wave 5: Pro Turn-Opportunity Planner (Mar 17–18, 2026)

This wave replaced Pro’s per-input tactical behavior with a turn-opportunity planner that evaluates abstract full-turn routes first and compiles them back into legal inputs using existing game rules.

### Promoted — Pro Turn Planner

- Candidate: `runtime_pro_turn_planner_v1`.
- Core shipped behavior:
  - New planner module with route families (`drainer_score`, `drainer_kill`, `spirit_impact`, `drainer_safety`, `mana_move`) and bounded beam assembly.
  - 2-turn evaluation (`our full turn` + `opponent full response`) with tactical lexicographic utility.
  - Hash-guarded continuation cache keyed by search-state hash + planner mode.
  - Planner activation and acceptance are constrained by tactical signals and rooted against legal root candidates.
- Promotion proof (bounded Pro ladder):
  - speed ratio `5.889` (cap `10.0`)
  - primary: `vs_normal +0.1667`, `vs_fast +0.4444`
  - confirmation: `vs_normal 0.0000`, `vs_fast +0.2500` (tolerance `-0.10`)
  - pool summary improved over baseline.
- Production transplant:
  - Pro runtime in `runtime_current` now enables turn planner in primary Pro context with bounded budget lift and deterministic tie-break/event ordering enabled.
  - Release speed gates remained green after transplant.

### Closed — Fast/Normal Turn-Planner Port Attempts

- Multiple direct planner transplants into Fast/Normal failed mixed-ladder non-regression.
- Repeated pattern: Normal lane regressed (typical bounded ladder `delta=-0.0556`) despite local tactical improvements.
- Decision: keep planner rollout Pro-only for now; treat Fast/Normal as separate future abstractions instead of direct Pro transplant.

## Mistakes Not To Repeat

- Do not keep historical profiles live in the active registry after their lesson is absorbed.
- Do not treat `target/experiment-runs` as durable memory.
- Do not treat `target/experiment-stamps` as durable memory.
- Do not revive archived candidates unless there is a specific new hypothesis that cannot be tested with the active profile surface.
- Do not widen the active experiment surface faster than you can maintain the runbook and guardrail tests.
