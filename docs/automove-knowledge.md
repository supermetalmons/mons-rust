# Automove Knowledge

This file keeps only durable automove rules and reusable heuristics.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the workflow and `AUTOMOVE_IDEAS.md` for the current live state.

## Stable Runtime Truths

- Shipping Pro routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` is the retained search-only baseline.
- Probe paths are diagnostics only; they do not describe shipping behavior.
- Promotion evidence comes from direct frontier-vs-baseline duels, not fixture churn alone.
- `runtime-preflight` still matters after promotion: exact-lite is hard, stage-1 CPU is advisory for Pro.

## Durable Lessons

- Separate `pre_accept` search choice from final `engine_post_search` output before changing shared heuristics.
- Seam alignment can clear `pro-triage` and `runtime-preflight` while still failing retained duels; local seam coverage is not duel strength.
- White `opening_reply_white` is a post-search head-over-advisor seam.
- White `normal_white_head_acceptance` is a vulnerable-window recovery seam where safer recovery roots already exist but still lose at final head acceptance.
- Black `black_recovery_branch` is not a reachability problem alone; the preserved spirit path can keep winning approval even when the shipping mana root is already available.
- A blunt turn-six black spirit-reentry safety gate can align `black_recovery_branch` and still clear retained triage plus `runtime-preflight`, but it regresses Fast duel strength enough to fail retained `pro-reliability`.
- On `black_recovery_branch`, the direct black legacy-alignment override already prefers shipping `l6,0;l6,1`; if the live advisor path still stays on spirit, treat that as legacy-selector plumbing mismatch rather than a missing reply-risk threshold.
- That plumbing difference is now partially traced: the ProV1 legacy selector inside `pro_v2_root_advisor_select_root` inherits `shortlist_config`, so it runs with the reply-risk guard disabled. Re-enabling that guard by swapping the selector to full `config` does align `black_recovery_branch`, but it still fails retained `pro-reliability` at `0.8333 / 0.9167 / 0.8333`, so that global config swap is not a promotable fix.
- Narrowing the black fix to the local `reply_risk_shortlist` is still not enough. A shortlist-only vulnerable-mana fallback inside `pro_v2_root_advisor_black_legacy_alignment_override` also aligns `black_recovery_branch`, clears retained triage plus `runtime-preflight`, and still fails retained `pro-reliability` at `0.9167 / 0.9167 / 0.8333` because Fast stays below the floor.
- On that same black seam, a score-only mana fallback is unsafe: the first naive scan picked `l6,0;l7,0` instead of the actual shipping legacy root.
- `white_split_trace` can move independently of the other white walls; fixing mana-sibling competition there does not solve the late white head seams.
- A narrow package that fixes only `opening_reply_white`, `black_recovery_branch`, and `normal_white_head_acceptance` is still not enough; the retained duel can stay at `0.8333 / 0.7500 / 0.7500` and previously clean fast/normal packs can regress.
- Cheap turn-three white approval escapes can fix `white_split_trace` and `black_bridge_nonwin` together while keeping Fast clean, but retained duel coverage can rotate onto new white turn-three misses outside the original five-board live probe.
- On white turn-three no-action mana boards, the legacy-alignment override is only safe when the legacy root is at least two root-rank slots worse than the currently approved safe root; allowing the `+1` case reopens the rotated Pro/Normal sibling misses.
- The rotated white turn-three sibling misses were resolved by tightening that legacy-alignment rank-gap check, not by widening omitted-root reentry or adding another late preserved-root shim.
- Passing the small `pro-reliability` gate at `3x2` does not guarantee confirm readiness; the `4x4` confirm spend can still uncover Fast and Normal losses that never appear in the smaller sample.
- The confirm-only frontier losses are currently clustered around two seams: early white head acceptance where `engine_post_search` accepts a frontier head that shipping rejects or never executes, and a separate late black search-vs-spirit approval seam.
- On white turn-three no-action weak-window boards, a safe `DrainerSafetyRecovery` root can be the correct `pre_accept` answer even when the reply-risk shortlist is dominated by vulnerable `ManaTempo` window roots. ProV2 has to both admit that recovery root at approval time and prevent `engine_post_search` from immediately reinstalling the vulnerable one-window head.
- Fixing one early white confirm recovery board can leave the aggregate confirm gate completely unchanged; when that happens, the right read is that the seam rotated, not that the local fix was fake.
- If a rotated white confirm seam shows the incumbent quiet `ManaTempo` root ahead on both reply floor and selected override utility, it is not another shortlist-order or head-acceptance bug. Treat that as a root-scoring/model mismatch and do not paper over it with another advisor override.
- Runtime cost is a real gate. A candidate that fixes live walls but pushes stage-1 CPU into the `1.5x+` range against `shipping_pro_search` is still non-promotable.
- Wrapper-only reroutes, fallback widening, shortlist widening, and metadata-only advisor changes saturate quickly; the real frontier is shared approval and head logic.

## Live Retained Surface

- Retained seams worth watching in `primary_pro`: `human_win_pro_c`, `primary_white_safe_progress_rerank_ply27`, `primary_black_turn_four_action_mana_ply15`, `primary_black_mana_bridge_ply20`, `primary_black_spirit_bridge_ply19`, `primary_black_negative_deny_ply4`, `primary_live_nonwin_opening_reply_white`, `primary_live_nonwin_black_vulnerable_spirit_reentry`
- Diagnostic-only live non-win walls: `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, `vs_shipping_pro_white_split_trace`, `vs_shipping_normal_black_bridge_nonwin`, `vs_shipping_normal_white_head_acceptance`

## Diagnostic Toolbox

- `smart_automove_pro_reliability_duel_trace_probe`
- `smart_automove_pro_reliability_nonwin_trace_probe`
- `smart_automove_pro_reliability_hotspot_probe`
- `smart_automove_pro_triage_retained_churn_probe`
- `smart_automove_pro_forced_turn_engine_retained_churn_probe`
- `smart_automove_pro_root_advisor_trace_probe`
- `smart_automove_pro_white_turn_three_sibling_root_probe`
- `white_confirm_pro_ply11_reply_order_probe`
- `black_recovery_branch_legacy_alignment_probe`

## Kill Rules

- Kill any line that fails `guardrails` or pushes off-target retained churn above `1`.
- Kill any line that does not move direct duel evidence on the candidate-vs-baseline matchup.
- Kill any line that stays inert at `target_changed=0 off_target_changed=0` against active `frontier_pro_v2_guarded`.
- Kill any line that only changes advisor labels or `pre_accept` metadata while the final selected root stays unchanged.
- Kill any line that widens shortlist or injection coverage without moving the approved root on the live walls.
- Kill any line that fixes only `white_split_trace` while leaving the other white and black walls unchanged.
- Kill any line that regresses a duel pack that is currently clean, even if it fixes part of the retained live wall surface.
- Kill any line that aligns live walls but still fails retained duel strength or canonical cost.
- Do not reopen archived profiles, archived seams, or archived wave packages without a brand-new shared hypothesis.
