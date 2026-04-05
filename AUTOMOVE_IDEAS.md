# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired branch history to `docs/automove-archive.md`.

## Current Gate Snapshot

- Shipping Pro stays `runtime_current`.
- The only live Pro challenger is `runtime_pro_turn_engine_v30`.
- Latest probe-led split (`2026-04-05`):
  - attempted retained shared split: followup-tolerant `spirit_own_mana_setup_now` competition plus a close `Safe*Progress` head normal-safety block
  - runtime-faithful retained-churn seams were unchanged, so the split was killed before the canonical Pro loop
  - direct conclusion: do not spend another loop on soft followup-tolerance or close quiet-root normal-safety guards unless a new exact seam proves they fire
- Latest focused gate (`2026-04-05`):
  - attempted retained shared split: speculative immediate-score non-regression clamp plus setup-gain-only spirit-setup promotion
  - `pro-triage(primary_pro)` stayed at `5/52`; the only movement was `primary_spirit_setup` shifting from candidate rank `8` to `5`, not collapsing back to current
  - `pro-reliability`
  - `12` games
  - `vs current Pro`: `win_rate=0.7500`, `confidence=0.9270`, `candidate_avg_ms=96.60`
  - `vs current Normal`: `win_rate=0.5000`, `confidence=0.0000`, `candidate_avg_ms=100.04`
  - `vs current Fast`: `win_rate=0.8333`, `confidence=0.9807`, `candidate_avg_ms=98.88`
  - direct conclusion: kill the split; it regressed direct Pro-vs-Pro, stayed flat on the `vs current Normal` wall, and did not reduce retained `primary_pro` churn
- Previous focused gate (`2026-04-05`):
  - `pro-reliability`
  - `12` games
  - `vs current Pro`: `win_rate=0.9167`, `confidence=0.9968`, `candidate_avg_ms=95.94`
  - `vs current Normal`: `win_rate=0.5000`, `confidence=0.0000`, `candidate_avg_ms=96.28`
- Latest cheap cross-check on the same line:
  - `pro_fast_screen vs normal`
  - `delta=-0.2500`
- Latest local selector-composition repair pass (`2026-04-05`, shared ProV2 code):
  - fixed deferred progress-head overrides on `primary_supermana_progress` and `primary_opponent_mana_progress`
  - fixed absent deferred progress-head injections on `human_win_pro_a`, `human_win_pro_c`, and `primary_black_gate_loss_b_ply31`
  - fixed non-concrete one-chunk progress-head override on `primary_ext_sensitive_no_ext_a`
  - retained challenger check after the shared fix is still flat on the tiny `1x1` mirrored sample:
    - `runtime_pro_turn_engine_v30` vs current Pro: `win_rate=0.5000`, `candidate_avg_ms=87.71`
    - `runtime_pro_turn_engine_v30` vs current Normal: `win_rate=0.5000`, `candidate_avg_ms=154.51`
    - `runtime_pro_turn_engine_v30` vs current Fast: `win_rate=0.5000`, `candidate_avg_ms=56.89`
  - scratch selective-allowed-root profile line stayed flat and was retired instead of being retained as a new profile ID
- Last retained larger confirmation result on the same line:
  - `pro-reliability-confirm`
  - `32` games
  - `win_rate=0.7812`
  - `confidence=0.9989`
  - `candidate_avg_ms=100.11`
- `pro-triage(primary_pro)` on the retained challenger still moves on `5/52`, while `opening_reply` stays `0/3`.
- Direct conclusion: speed is already acceptable. The live wall is broad `primary_pro` root-choice composition against current `Normal`, not the `700ms` move-time budget and not opening guards.

## Promotion Rule

- `pro-reliability` is the focused Pro gate.
- `pro-reliability-confirm` is the final promotion proof.
- Promote only after a completed confirmation run clears all three direct duels:
  - candidate Pro vs current Pro: `win_rate >= 0.90`, `confidence >= 0.99`, `candidate_avg_move_ms <= 700`
  - candidate Pro vs current Normal: `win_rate >= 0.90`, `confidence >= 0.99`, `candidate_avg_move_ms <= 700`
  - candidate Pro vs current Fast: `win_rate >= 0.90`, `confidence >= 0.99`, `candidate_avg_move_ms <= 700`
- `candidate_avg_move_ms` means candidate decision-selection time on candidate turns only. Do not count compile time, harness startup, or `game.process_input(...)`.
- A stalled or incomplete duel run is not promotable evidence.

## Live Code Surfaces

- Profile / guard composition:
  - `configure_runtime_pro_turn_engine_v30`
  - `runtime_pro_turn_engine_v30_guarded_inputs`
  - `turn_opportunity_planner_next_inputs_from_allowed`
- ProV2 root arbitration:
  - `pick_root_move_with_reply_risk_guard`
  - `is_better_reply_risk_candidate`
  - `pro_v2_safe_progress_sibling_order`
  - `pro_v2_white_spirit_followup_setup_reply_order`
  - `pro_v2_late_safe_mana_root_order`
- Shipping reference safety logic:
  - `pick_root_move_with_normal_safety`
  - `pick_normal_root_with_deep_floor`
- Shared exact / scoring signals:
  - `build_scored_root_move`
  - `build_exact_turn_summary`
  - `exact_tactical_spirit_summary`
  - `ScoringBoardSummary::from_board`
  - `evaluate_preferability_with_context`

## Diagnostic Fixtures

- `primary_supermana_progress`
- `primary_opponent_mana_progress`
- `primary_spirit_setup`
- `primary_black_gate_loss_b_ply31`
- `human_win_pro_a`
- `human_win_pro_c`

## Do Not Reopen

- Opening-only guard churn. `opening_reply` is already unchanged on the live v30 line.
- Wrapper-only current-Normal fallbacks or reroutes
- Exact replay repairs without a broader duel story
- Acceptance-only macro-head clamps
- Cache-size, memo-shape, reserve-heavy, or hasher experiments without a direct quality hypothesis
- Generic search-budget or search-knob retunes without evidence that the live wall is on that surface
- White-only or black-only local seam repairs that do not move the broader `vs current Normal` wall
- Branches that only shift counters, disagreement counts, or hotspot timing while duel quality stays flat

## Iteration Rules

- Accept a revived line only if direct-vs-`runtime_current` evidence improves while staying under `700ms`.
- Kill a line immediately if it is only a speed regression with no quality story.
- Kill a line immediately if it preserves behavior and only shifts counters.
- Prefer cheap quality-ranking probes first, then the canonical Pro loop, then confirmation only after the focused gate earns more spend.

## Next Live Split

- Keep the retained challenger ID and stay out of new unsupported scratch profiles.
- Major direction 1: continue deleting deferred `Safe*Progress -> ImmediateScore` composition mistakes in shared ProV2 code. The retained fix list is now: safe-pickup post-search blocks, absent deferred progress-head injection blocks, and non-concrete one-chunk progress-head rejection.
- Major direction 2: target the remaining retained-challenger deltas that still look like selector composition rather than root scoring, especially `primary_spirit_setup`, `primary_pvs_sensitive_search`, `primary_white_harvest_loss_c_ply24`, and `human_win_pro_c`.
- Major direction 3: when a remaining miss is not another deferred progress-head bug, import current/normal safety discipline into shortlist ordering for safe non-progress and spirit-impact ties before touching broader exact/search budgets.
- Major direction 4: only after the retained selector surface stops producing new regressions, spend more budget on shared exact shortlist comparison for close progress/spirit ties.
- Immediate next split:
  - treat `primary_pvs_sensitive_search` as a late `engine_post_search` acceptance seam
  - treat `primary_white_harvest_loss_c_ply24` as a forced/injected-root shortlist seam
  - treat `human_win_pro_c` as a pure `pre_accept` safe-progress bias
  - do not spend a shared-code split on `primary_black_reliability_opening_3_ply4` until the runtime-faithful probe says it is live again
- Do not reopen:
  - speculative immediate-score first-chunk non-regression clamps on `SpiritImpact` or `Safe*Progress` heads
  - setup-gain-only spirit-setup promotion against safe non-spirit roots
- Proof target for the next retained branch: reduce `runtime_pro_turn_engine_v30` vs `runtime_current` `primary_pro` churn below the current `5/52`, then rerun the canonical Pro loop against `runtime_current`.
