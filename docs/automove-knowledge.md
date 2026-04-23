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
- Tracing that shortlist-local fallback showed the retained Fast failure was not fresh collateral from `black_recovery_branch`; both Fast non-wins were the already-pinned late black head-accept seam `l1,8;l1,9` vs shipping `l1,8;l0,8`.
- That late black Fast seam was not a head-acceptance mismatch after all. On the live board, `pre_accept`, head, and final selection all stayed on the same vulnerable root; the actual fix point was the advisor's existing late-window mana safety override.
- The durable fix on that board is narrow: let `pro_v2_root_advisor_black_late_window_mana_safety_override` rescue a rank-zero vulnerable one-window `ManaTempo` root only when a same-lane safe sibling also restores the lane's progress surface. That keeps the retained `l1,8;l0,8` board aligned without disturbing the small loop.
- A second late black advisor miss was narrower than the earlier legacy-selector seams. On late black action+mana turn-start boards with no window or deny pressure, a quiet no-progress `ManaTempo` incumbent can still be the wrong retained answer when the reply-risk shortlist isolates a quiet `SpiritImpact` own-setup challenger that keeps the same progress path and gains at least `+64` setup. That rescue belongs in advisor family competition, not in search-only reranking.
- The remaining black board `l7,1;l9,3` vs shipping `l1,5;l2,7;l1,8` is not another legacy/search shortlist seam. Frontier already keeps the safe-progress incumbent on the full pool, while the candidate-only ProV1 replay prefers a different spirit root than shipping. Treat that as a progress-vs-setup/model-ranking mismatch rather than another reply-risk or legacy-alignment bug.
- On white turn-three action+mana mid-turn boards, a safe-progress incumbent can still be the wrong retained choice even when it wins raw search score. If a spirit-own-setup challenger preserves the same progress surface, gains at least `+64` setup, and leads by at least six root-rank slots, `pro_v2_root_advisor_white_early_safe_progress_setup_competition_override` should still be allowed to rescue that spirit line across a wider score gap.
- On that same black seam, a score-only mana fallback is unsafe: the first naive scan picked `l6,0;l7,0` instead of the actual shipping legacy root.
- `white_split_trace` can move independently of the other white walls; fixing mana-sibling competition there does not solve the late white head seams.
- A narrow package that fixes only `opening_reply_white`, `black_recovery_branch`, and `normal_white_head_acceptance` is still not enough; the retained duel can stay at `0.8333 / 0.7500 / 0.7500` and previously clean fast/normal packs can regress.
- Cheap turn-three white approval escapes can fix `white_split_trace` and `black_bridge_nonwin` together while keeping Fast clean, but retained duel coverage can rotate onto new white turn-three misses outside the original five-board live probe.
- On white turn-three no-action mana boards, the legacy-alignment override is only safe when the legacy root is at least two root-rank slots worse than the currently approved safe root; allowing the `+1` case reopens the rotated Pro/Normal sibling misses.
- The rotated white turn-three sibling misses were resolved by tightening that legacy-alignment rank-gap check, not by widening omitted-root reentry or adding another late preserved-root shim.
- Passing the small `pro-reliability` gate at `3x2` does not guarantee confirm readiness; the `4x4` confirm spend can still uncover Fast and Normal losses that never appear in the smaller sample.
- Recent confirm-only frontier losses clustered around two seams: early white head acceptance where `engine_post_search` accepts a frontier head that shipping rejects or never executes, and a separate late black search-vs-spirit approval seam.
- On white turn-three no-action weak-window boards, a safe `DrainerSafetyRecovery` root can be the correct `pre_accept` answer even when the reply-risk shortlist is dominated by vulnerable `ManaTempo` window roots. ProV2 has to both admit that recovery root at approval time and prevent `engine_post_search` from immediately reinstalling the vulnerable one-window head.
- Broadening that white turn-three no-action recovery rule from turn-start only to `mons_moves_count == 1`, even with a paired head-rejection guard, is too broad to keep. It does fix `l9,4;l8,3` vs shipping `l9,4;l8,5`, but it also drags older vulnerable white mana-only boards onto recovery roots and fails retained `pro-reliability` at `0.9167 / 0.7500 / 0.9167` by rotating Normal onto engine-disabled early-white seams such as `l8,5;l7,6` vs `l8,7;l7,8` and `l9,4;l8,5` vs `l9,4;l9,3`.
- The narrower direct fallback version of that white search-only seam is also too shallow to keep. Both a shipping-assisted runtime fallback and a frontier-local nearby `DrainerSafetyRecovery` fallback fix `l9,4;l8,3` vs shipping `l9,4;l8,5` and still fail retained `pro-reliability` at `0.9167 / 0.8333 / 0.9167`, because Normal still loses on engine-disabled early-white seams such as `l8,5;l7,6` vs `l9,5;l8,3;l7,4`.
- The retained fix for that remaining early-white Normal seam is not another advisor or head tweak. On the exact turn-five white action+mana weak-window board class where frontier keeps engine selection enabled and settles on a vulnerable quiet `ManaTempo` root, a narrow wrapper fallback can safely mirror shipping's engine-disabled `SpiritImpact` progress line instead. Keep that fix wrapper-local and exact-context; broader white recovery widening was already disproved.
- The analogous black turn-six residue board does not admit the same direct wrapper cure. Mirroring shipping on `l7,1;l9,3` vs `l1,5;l2,7;l1,8` fixes that local board and still fails `pro-reliability-confirm` at `0.9375 / 0.9062 / 0.8750`, because Fast rotates onto later black engine-disabled seams (`l0,0;l1,1` vs `l7,1;l8,0` and `l0,5;l1,5` vs `l2,5;l3,7;l2,8`) plus the old white search-only split. Do not keep that black wrapper fallback without a story for the downstream Fast pack.
- On the black progress-vs-setup residue board itself, the actual shipping move is still `l1,5;l2,7;l1,8`, not the candidate-only ProV1 replay `l1,5;l3,7;l2,8`. Shipping reaches it with `engine_disabled`, while frontier stays on `frontier_execute` with the safe-progress incumbent `l7,1;l9,3`.
- On late black action+mana weak-window boards (`window<=1`, `deny<=1`, no attack), a safe progress incumbent can still be the wrong retained answer even when it is already approved and locally safe. If the reply-risk shortlist already contains a competitive `SpiritImpact` own-setup progress root with the same progress flags, at least `+64` setup gain, and at most four root-rank slots of gap, the rescue belongs in advisor family competition rather than wrapper mirroring or head acceptance.
- Fixing one early white confirm recovery board can leave the aggregate confirm gate completely unchanged; when that happens, the right read is that the seam rotated, not that the local fix was fake.
- The same rotation rule applies on black late-fast seams. Fixing one retained black Fast wall can leave `pro-reliability-confirm` unchanged if the `4x4` pack simply rotates onto other white early head/order seams and separate black legacy/search seams.
- The same logic distinguishes the remaining early white Fast seams: `l9,7;l8,6` vs `l9,7;l7,6;l7,7` was an advisor-layer safe-progress-vs-setup competition miss, while `l9,4;l8,3` vs `l9,4;l8,5` is a search-only rerank split where shipping changes the same pre-accept root through the search-only allowance path. Do not treat those as the same bug class.
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
- `black_progress_vs_setup_residue_probe`
- `black_confirm_fast_lane_split_probe`
- `black_confirm_fast_setup_split_probe`

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
