# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired branch history to `docs/automove-archive.md`.

## Current Gate Snapshot

- Shipping Pro stays `runtime_current`.
- The only live Pro challenger is `runtime_pro_turn_engine_v30`.
- Latest focused gate (`2026-04-08`):
  - retained shared split: reject lower-scored unsafe `Safe*Progress` late heads on `primary_pvs_sensitive_search` unless they bring a material non-eval override win
  - retained-churn result: `primary_pvs_sensitive_search` now matches `runtime_current`; `pro-triage(primary_pro)` moved only `human_win_pro_c`, so the retained challenger is down to `1/52` changed primary-Pro fixtures with `opening_reply` still `0/3`
  - `pro-reliability`
  - `12` games
  - `vs current Pro`: `win_rate=0.8333`, `confidence=0.9807`, `candidate_avg_ms=97.99`
  - `vs current Normal`: `win_rate=0.5000`, `confidence=0.0000`, `candidate_avg_ms=91.00`
  - `vs current Fast`: `win_rate=0.6667`, `confidence=0.8062`, `candidate_avg_ms=110.13`
  - direct conclusion: kill the split; it closed the remaining late `engine_post_search` seam but did not move the direct duel wall, so do not spend another local acceptance-only repair loop
- Latest diagnostic close (`2026-04-05`):
  - extended `smart_automove_pro_reliability_hotspot_probe` to compare `runtime_pro_turn_engine_v30` against `runtime_current` on the bounded reliability hotspot corpus
  - all real hotspot positions were move-identical to current: `primary_spirit_setup`, `primary_black_loss_opening_a_ply19`, `human_win_pro_a`, `loss_opening_a`, and `loss_opening_b`
  - the only move difference was the synthetic `quiet_positional` sample, so there is still no new duel-linked production seam worth another canonical Pro loop
  - direct conclusion: kill the line here; do not reopen from hotspot counter deltas or synthetic-position drift alone
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
- `pro-triage(primary_pro)` on the retained challenger now moves only on `1/52`, while `opening_reply` stays `0/3`.
- Direct conclusion: speed is already acceptable. The live wall is broad `primary_pro` root-choice composition against current `Normal`, not the `700ms` move-time budget and not opening guards.
- Closing `primary_pvs_sensitive_search` reduced retained churn, but it did not change the duel wall. Remaining spend must come from a broader duel-linked selector story, not another local `engine_post_search` clamp.

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
- Major direction 1: continue deleting deferred `Safe*Progress -> ImmediateScore` composition mistakes in shared ProV2 code. The retained fix list now includes: safe-pickup post-search blocks, absent deferred progress-head injection blocks, non-concrete one-chunk progress-head rejection, weaker plain-spirit head rejection, and the black turn-two full-resource low-budget-clamp skip.
- Major direction 2: `primary_pvs_sensitive_search` is now closed as a retained late-head regression. Do not reopen it unless a fresh duel sample shows the seam alive again under a different runtime shape.
- Major direction 3: `human_win_pro_c` is the only remaining retained-challenger drift. The retained selector probe still says it is a pure `pre_accept` safe-progress bias where the chosen root has better followup floor than the baseline spirit-own-setup root.
- Major direction 3a: the bounded reliability hotspot corpus still does not support a new duel seam right now. Its compare probe is decision-identical to `runtime_current` on every real hotspot case, so do not spend another shared-code split unless a fresh duel sample or a broader compare probe exposes a real move difference.
- Major direction 4: do not spend another local seam-repair split unless it has a direct duel story. Cutting `primary_pro` churn from `2/52` to `1/52` still did not move `pro-reliability`.
- Immediate next split:
  - keep `primary_white_harvest_loss_c_ply24`, `primary_spirit_setup`, `primary_black_reliability_opening_3_ply4`, and `primary_pvs_sensitive_search` closed unless new duel evidence reopens them
  - if the line is revived, start from a duel-linked explanation for `human_win_pro_c` before touching more turn-engine head logic
  - refresh direct duel evidence first if the wall is unclear; otherwise do not spend another acceptance-only split
- Do not reopen:
  - speculative immediate-score first-chunk non-regression clamps on `SpiritImpact` or `Safe*Progress` heads
  - setup-gain-only spirit-setup promotion against safe non-spirit roots
  - eval-only unsafe `Safe*Progress` late-head overrides on lower-scored non-progress roots; the retained PVS repair already closes that seam and did not move direct duels
- Proof target for the next retained branch: beat the current unchanged `pro-reliability` wall with a duel-linked fix, not just another local reduction from the present `1/52` `primary_pro` churn.
