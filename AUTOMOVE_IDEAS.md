# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired branch history to `docs/automove-archive.md`.

## Current Gate Snapshot

- Shipping Pro stays `runtime_current`.
- The only live Pro challenger is `runtime_pro_turn_engine_v30`.
- Latest focused gate (`2026-04-05`):
  - `pro-reliability`
  - `12` games
  - `vs current Pro`: `win_rate=0.9167`, `confidence=0.9968`, `candidate_avg_ms=95.94`
  - `vs current Normal`: `win_rate=0.5000`, `confidence=0.0000`, `candidate_avg_ms=96.28`
- Latest cheap cross-check on the same line:
  - `pro_fast_screen vs normal`
  - `delta=-0.2500`
- Last retained larger confirmation result on the same line:
  - `pro-reliability-confirm`
  - `32` games
  - `win_rate=0.7812`
  - `confidence=0.9989`
  - `candidate_avg_ms=100.11`
- `pro-triage(primary_pro)` still moves on `10/52`, while `opening_reply` stays `0/3`.
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

- Keep the retained challenger ID and stay out of new wrapper-only branches.
- Major direction 1: selectively re-enable the shipping allowed-root planner path inside ProV2 for close non-tactical clusters. `runtime_current` keeps planner guidance while v30 disables it entirely; the first split should test whether planner guidance restores the safer current pickup/setup choice without reopening wrapper families.
- Major direction 2: import normal-style safety / deep-floor discipline into ProV2 reply-risk arbitration for non-tactical progress and spirit-setup competitions. The branch should let safe pickup-now, shorter secure progress, and safer same-opening setup roots beat marginally higher unsafe progress roots across the full shortlist, not only in the current late-white mana-only and late-black rescue special cases.
- Major direction 3: restore a cheap live setup-quality signal in root evaluation. `spirit_setup_gain` is effectively dead when static exact eval is off; populate an equivalent from exact-lite / tactical spirit summary so ProV2 can tell concrete setup from soft progress before override logic fires.
- Major direction 4: if the first three still stall, spend extra budget on shared exact shortlist comparison, not wrappers. Use exact secure progress, same-turn score window, deny gain, and drainer-threat deltas to break close progress ties in `pick_root_move_with_reply_risk_guard`.
- Proof target for the next branch: move the six diagnostic fixtures above in the same direction as `runtime_current`, then rerun the canonical Pro loop against `runtime_current`.
