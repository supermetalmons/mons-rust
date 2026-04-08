# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired branch history to `docs/automove-archive.md`.

## Current Gate Snapshot

- Shipping Pro stays `runtime_current`.
- The only live Pro challenger is `runtime_pro_turn_engine_v30`.
- Latest diagnostic close (`2026-04-08`, latest):
  - rechecked the only remaining repeated live lead before code edits: the white `turn=3`, `mons_moves=1`, `action+mana` accepted-head seam `l9,4;l8,4` vs current `l8,7;l7,8`
  - reran `smart_automove_pro_runtime_faithful_retained_churn_probe` and every retained `primary_pro` fixture still had `accepted=false`; the retained surfaces remain: `primary_spirit_setup`, `primary_pvs_sensitive_search`, `primary_black_reliability_opening_3_ply4`, `primary_white_harvest_loss_c_ply24`, and `human_win_pro_c`
  - direct conclusion: kill the white turn-three `action+mana` accepted-head idea before code edits; the repeated fast-duel seam still has no retained `primary_pro` target-surface foothold
- Latest focused gate (`2026-04-08`, latest):
  - refreshed `smart_automove_pro_reliability_duel_trace_probe` on the retained challenger; the live wall moved into a broader white turn-three family plus the old fast black turn-four miss
  - duel summary:
    - `vs current Pro`: `1` regression, `5` improvements, `6` flat; repeated pair `l9,2;l8,3` vs current `l10,7;l9,7`
    - `vs current Normal`: `3` regressions, `2` improvements, `7` flat; one-off pairs `l0,5;l1,6` vs `l1,5;l3,6;l2,7`, `l10,4;l9,4` vs `l8,6;l7,7`, and `l2,4;l3,3` vs `l3,6;l2,7`
    - `vs current Fast`: `4` regressions, `5` improvements, `3` flat; repeated white turn-three pair `l9,4;l8,4` vs current `l8,7;l7,8` twice, plus `l1,6;l2,7` vs `l2,3;l3,2` once and `l9,5;l8,5` vs `l7,5;l6,4` once
  - a focused live-seam probe showed three of those white turn-three misses were the same wrapper family: `turn=3`, `mons_moves>0`, `action=false`, `mana=true` states that still routed through the broad fallback instead of the challenger’s own pre-accept/current root
  - tried a broader white turn-three mana-only reroute: route those `turn=3`, `mons_moves>0`, `!action`, `mana` boards back to the current Pro surface; focused regression tests on the live Pro/Normal/Fast boards all passed
  - cheap-gate result: `guardrails` passed, but `pro-triage(primary_pro)` stayed unchanged at the same stale `human_win_pro_c`-only `1/52`
  - direct conclusion: kill the broader white turn-three mana-only reroute before `runtime-preflight`; even a three-board duel-backed wrapper family is still too local if the cheap target surface does not move
- Latest diagnostic close (`2026-04-08`, latest):
  - compared the remaining `human_win_pro_c` drift against the sampled fast-duel black turn-four board with a shared projection probe before cutting more ProV2 code
  - `human_win_pro_c` is not the same seam: selected `l10,5;l9,6` is a safe progress root, but its post-root projection becomes `SpiritImpact -> ImmediateScore` and it beats the current spirit-own-setup root mostly on followup floor (`810871` vs `810407`), not on a distinct projected score delta
  - the sampled fast-duel board is different: injected `l1,6;l2,7` is a vulnerable `ManaTempo` root whose post-root projection becomes `SafeSupermanaProgress -> ImmediateScore`, while current `l2,3;l3,2` is another vulnerable `ManaTempo` root with a weaker projected followup
  - direct conclusion: kill the shared clamp idea before code edits; do not bundle `human_win_pro_c` and the sampled fast black turn-four divergence into one projection-family split without fresh duel evidence that they really share a code path
- Latest focused gate (`2026-04-08`, latest):
  - paired the duel-backed wrapper bundle with a narrow late-white omitted-root reply-risk rescue so `human_win_pro_c` would match current again without reopening `primary_black_reliability_opening_3_ply4`
  - cheap-gate result: `guardrails` passed, `opening_reply` stayed `0/3`, and `pro-triage(primary_pro)` collapsed to `0/52` because the challenger now matched `runtime_current` on every cheap Pro fixture, so the split still died before `runtime-preflight`
  - refreshed `smart_automove_pro_reliability_duel_trace_probe` on that line anyway; the traced wrapper misses were gone and `vs current Pro` improved to `0` regressions / `2` improvements, but the remaining wall was still real `engine_post_search` drift: `vs current Normal` stayed at `2` regressions / `1` improvement and `vs current Fast` stayed at `3` regressions / `3` improvements
  - direct conclusion: kill the late-white omitted-root rescue too; neutralizing the last cheap drift is still non-promotable if the target surface no longer changes, and the live wall is no longer wrapper-local
- Latest focused gate (`2026-04-08`, later):
  - refreshed `smart_automove_pro_reliability_duel_trace_probe` on the retained challenger and the live seam map stayed split across three families: one white `turn=3` wrapper miss in `vs current Pro`, three misses in `vs current Normal` (one black turn-two engine-enabled pre-accept miss, one black turn-four engine-enabled miss, one white turn-three wrapper miss), and four misses in `vs current Fast` (the repeated white `SafeSupermanaProgress` accepted-head seam twice, one black turn-four engine-enabled miss, and one white turn-three wrapper miss)
  - tried a duel-backed wrapper bundle: route all white `turn=3` mid-turn states plus black `turn=2`/`turn=4` one-move `action+mana` states back to the current Pro surface; focused regression tests on the traced boards all passed
  - then tried one extra cheap lever for the surviving `1/52` surface: a late white full-resource current-Pro guard aimed at `human_win_pro_c`, but it did not change the selected move at all
  - cheap-gate result: `guardrails` passed, but `pro-triage(primary_pro)` stayed unchanged at `1/52` with only `human_win_pro_c`, while `opening_reply` stayed `0/3`
  - direct conclusion: kill both wrapper-only lines before `runtime-preflight`; even a multi-board duel-backed guard bundle is too local if the cheap target surface does not move, and the late-white full-resource current-Pro guard is not a real lever on `human_win_pro_c`
- Latest focused gate (`2026-04-08`):
  - first tried a traced wrapper repair: route white `turn=3`, mana-only mid-turn states back to the current Pro surface instead of the broad fast fallback
  - that wrapper fix corrected three replay-traced white turn-three mana-only duel boards, but `pro-triage(primary_pro)` stayed unchanged at `1/52` with only `human_win_pro_c`, so the local guard split was killed before `runtime-preflight`
  - then tried a broader shared ProV2 selector split: keep high-setup-gain `spirit_own_mana_setup_now` roots alive against same-lane non-spirit progress challengers, plus a narrow own-setup reply-order allowance on traced normal-duel ties
  - cheap-gate result: `human_win_pro_c` collapsed, but `primary_black_reliability_opening_3_ply4` reopened, so `pro-triage(primary_pro)` moved to `2/52` with `opening_reply` still `0/3`
  - `pro-reliability`
  - `12` games
  - `vs current Pro`: `win_rate=0.7500`, `confidence=0.9270`, `candidate_avg_ms=88.82`
  - `vs current Normal`: `win_rate=0.4167`, `confidence=0.0000`, `candidate_avg_ms=84.35`
  - `vs current Fast`: `win_rate=0.4167`, `confidence=0.0000`, `candidate_avg_ms=92.73`
  - direct conclusion: kill both production splits; the white turn-three guard repair was too local to earn more spend by itself, and the broader same-lane own-setup override was too aggressive and cratered direct duel quality
- Latest diagnostic close (`2026-04-08`):
  - added `smart_automove_pro_reliability_duel_trace_probe` to replay the exact `pro-reliability` seed corpus and compare candidate turns against a shadow `runtime_current` Pro continuation
  - the probe exposed one repeated real fast-duel `engine_post_search` `SafeSupermanaProgress` override plus several later non-head selector drifts that the bounded hotspot corpus had missed
  - attempted a bounded large-search-deficit progress-head clamp on that traced fast-duel seam, but `pro-triage(primary_pro)` stayed unchanged at `1/52` with only `human_win_pro_c`
  - direct conclusion: keep the duel trace probe, but kill the production split before `runtime-preflight` and `pro-reliability`; do not retain another acceptance-only traced repair unless it moves the cheap target surface
- Latest diagnostic close (`2026-04-08`):
  - reran `smart_automove_pro_reliability_hotspot_probe` after the retained `primary_pvs_sensitive_search` repair
  - all real hotspot positions were still move-identical to current: `primary_spirit_setup`, `primary_black_loss_opening_a_ply19`, `human_win_pro_a`, `loss_opening_a`, and `loss_opening_b`
  - the only changed move was still the synthetic `quiet_positional` sample, so the retained PVS repair did not expose a new duel-linked production seam
  - `human_win_pro_c` remains only a triage drift and selector-probe story, not a hotspot-backed duel seam
  - direct conclusion: kill the selector split before code edits; do not reopen from `human_win_pro_c` alone without fresh direct duel or hotspot evidence
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
- Major direction 3a: the bounded reliability hotspot corpus still does not support a new duel seam right now, even after the retained PVS repair. Its compare probe is decision-identical to `runtime_current` on every real hotspot case, so use the duel-trace probe when the direct wall is unclear instead of spending another hotspot-first split.
- Major direction 4: do not spend another local seam-repair split unless it has a direct duel story. Cutting `primary_pro` churn from `2/52` to `1/52` still did not move `pro-reliability`.
- Immediate next split:
  - keep `primary_white_harvest_loss_c_ply24`, `primary_spirit_setup`, `primary_black_reliability_opening_3_ply4`, and `primary_pvs_sensitive_search` closed unless new duel evidence reopens them
  - refresh direct duel evidence first with `smart_automove_pro_reliability_duel_trace_probe` when the wall is unclear; the bounded hotspot corpus alone is no longer enough
  - if the line is revived, start from a duel-linked explanation that moves more than the current `1/52` `primary_pro` drift before touching more turn-engine head logic; `human_win_pro_c` alone is still not enough
  - do not spend another acceptance-only split unless the traced seam also changes the cheap target surface
- Do not reopen:
  - wrapper-only white `turn=3` plus black `turn=2`/`turn=4` one-move current-Pro guard bundles by themselves; on Apr 8 they fixed multiple traced duel boards but still left `pro-triage(primary_pro)` unchanged at `1/52`
  - isolated black `turn=4`, one-move `action+mana` current-Pro guards by themselves; on Apr 8 they fixed the sampled fast-duel `l1,6;l2,7` -> `l2,3;l3,2` divergence but still left `pro-triage(primary_pro)` unchanged at the same `human_win_pro_c`-only `1/52`
  - broader white `turn=3`, `mons_moves>0`, `action=false`, `mana=true` current-Pro reroutes by themselves; on Apr 8 they fixed three live duel boards across Pro/Normal/Fast but still left `pro-triage(primary_pro)` unchanged at the same `human_win_pro_c`-only `1/52`
  - white `turn=3`, `mons_moves=1`, `action+mana` accepted-head clamps by themselves when the retained churn probe still shows `accepted=false` on every retained `primary_pro` fixture; on Apr 8 the repeated fast-duel seam stayed outside the cheap target surface
  - shared `human_win_pro_c` plus sampled fast-duel black turn-four projection clamps by themselves; on Apr 8 the probe showed the human drift was a higher-followup-floor `SpiritImpact -> ImmediateScore` story while the fast board was an injected vulnerable `ManaTempo` `SafeSupermanaProgress -> ImmediateScore` seam
  - late white full-resource current-Pro guards as a `human_win_pro_c` lever; on Apr 8 the guard did not alter the selected move at all
  - white `turn=3` mana-only mid-turn wrapper reroutes by themselves; the traced guard repair fixed real duel boards but left `pro-triage(primary_pro)` unchanged at `1/52`
  - same-lane `spirit_own_mana_setup_now` progress overrides or larger own-setup reply-floor allowances by themselves; the Apr 8 combined split collapsed `human_win_pro_c` but reopened `primary_black_reliability_opening_3_ply4` and cratered direct duels
  - speculative immediate-score first-chunk non-regression clamps on `SpiritImpact` or `Safe*Progress` heads
  - multi-chunk `ImmediateScore` near-tie `ManaTempo` sibling clamps by themselves; on Apr 8 they cleared `human_win_pro_c`, fixed the bounded duel-accept seams, and passed `guardrails`, `pro-triage(primary_pro)=1/52`, and `runtime-preflight`, but `pro-reliability` still failed at `0.7500` vs current Pro, `0.6667` vs current Normal, and `0.5000` vs current Fast
  - setup-gain-only spirit-setup promotion against safe non-spirit roots
  - eval-only unsafe `Safe*Progress` late-head overrides on lower-scored non-progress roots; the retained PVS repair already closes that seam and did not move direct duels
  - traced fast-duel `Safe*Progress` acceptance clamps by themselves; the Apr 8 duel-replay split found a real repeated seed but still left `pro-triage(primary_pro)` unchanged
  - `human_win_pro_c` selector-only reranks without a new real-hotspot or direct-duel seam
- Proof target for the next retained branch: beat the current unchanged `pro-reliability` wall with a duel-linked fix, not just another local reduction from the present `1/52` `primary_pro` churn.
- Do not spend another production split until a fresh duel replay exposes a repeated seam that also has a retained `primary_pro` foothold; the current white turn-three `action+mana` accepted-head seam still does not.
