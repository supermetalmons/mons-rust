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
- Replaying that same shortlist-local black legacy fallback on the later promoted package still does not make it promotable. It aligns `black_recovery_branch` and collapses the five-board live nonwin probe back to the older white seams, but retained `pro-reliability` still fails at `0.9167 / 0.9167 / 0.8333`.
- The refreshed black recovery probe is now strong enough to avoid reopening that line blindly. It prints the full `reply_risk_shortlist` root details and confirms the best-ranked vulnerable mana candidate there is shipping `l6,0;l6,1`, not the earlier wrong score-leader `l6,0;l7,0`.
- On that replayed shortlist-local package, the new `pro` miss rotated to a later black lane split `l1,6;l1,7` vs shipping `l1,6;l1,5`, `normal` still lost on the old white `ply9` search-only split `l9,4;l8,3` vs `l9,4;l8,5`, and the retained Fast gate still failed on two no-diff games with `first_diff=none`. Treat that as a hard stop, not a live challenger.
- The later black lane split is not another shortlist omission. `black_pro_lane_split_probe` shows shipping `l1,6;l1,5` is already in the frontier `reply_risk_shortlist` beside frontier `l1,6;l1,7`, but shipping still loses under frontier's own reply-risk comparison (`shipping_vs_frontier=false`) because the reply floor is tied and frontier keeps better local utility (`drainer_safety=2` vs shipping `-1`).
- Treat that later black seam as a shipping-disabled lower-safety ordering mismatch, not a live advisor or head-acceptance bug. Shipping reaches `l1,6;l1,5` only because it disables the turn-engine selector on that board.
- The earlier Fast `first_diff=none` read was package-specific, not a durable property of the current promoted tree. `fast_hotspot_trace_probe` now shows the retained hotspot opening `0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/...` really does diverge on the current promoted package when frontier is white.
- The fresh part of that hotspot is late and white-owned: `ply=57` on `1 1 w 0 0 1 0 0 9 n04s1xn06/...`, where frontier chooses `l9,5;l8,6` from `engine_post_search` while shipping chooses `l8,5;l7,7;l8,8` from `engine_disabled`. Frontier's head is already rejected there, so treat it as a late white engine-disabled ordering mismatch, not another head-acceptance bug.
- The same hotspot opening with frontier as black is not a new seam. It simply rotates back onto the already-known `black_recovery_branch` split `l1,5;l3,3;l2,3` vs shipping `l6,0;l6,1`.
- `white_late_fast_hotspot_probe` shows that late white hotspot is not a safe wrapper-fallback candidate either. Frontier's approved `SafeSupermanaProgress` root `l9,5;l8,6` is the only reply-risk-shortlisted root and keeps the stronger reply floor (`921` vs shipping `651`).
- Shipping's move `l8,5;l7,7;l8,8` is outside the frontier shortlist, loses under frontier's own reply-risk comparator (`shipping_vs_frontier=false`), and is not even the strongest spirit-progress candidate in the full pool. Treat that as another shipping-only search ordering mismatch, not a live frontier omission.
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
- The improved residue probe also rules out the obvious full-pool advisor rescue. On that same board, the candidate pool already contains multiple safe `SpiritImpact` own-setup progress roots, but the strongest one under frontier's current utility, reply-floor, and followup metrics is `l1,5;l3,7;l2,8`, not shipping `l1,5;l2,7;l1,8`. That means the seam is not just “shipping root omitted from the shortlist”; it is an engine-disabled ordering mismatch, and another advisor family-competition override would be guessing at shipping semantics.
- On late black action+mana weak-window boards (`window<=1`, `deny<=1`, no attack), a safe progress incumbent can still be the wrong retained answer even when it is already approved and locally safe. If the reply-risk shortlist already contains a competitive `SpiritImpact` own-setup progress root with the same progress flags, at least `+64` setup gain, and at most four root-rank slots of gap, the rescue belongs in advisor family competition rather than wrapper mirroring or head acceptance.
- Fixing one early white confirm recovery board can leave the aggregate confirm gate completely unchanged; when that happens, the right read is that the seam rotated, not that the local fix was fake.
- The same rotation rule applies on black late-fast seams. Fixing one retained black Fast wall can leave `pro-reliability-confirm` unchanged if the `4x4` pack simply rotates onto other white early head/order seams and separate black legacy/search seams.
- The same logic distinguishes the remaining early white Fast seams: `l9,7;l8,6` vs `l9,7;l7,6;l7,7` was an advisor-layer safe-progress-vs-setup competition miss, while `l9,4;l8,3` vs `l9,4;l8,5` is a search-only rerank split where shipping changes the same pre-accept root through the search-only allowance path. Do not treat those as the same bug class.
- `white_fast_ply9_search_only_split_probe` now closes that remaining white `ply9` split as a live target too. Frontier's approved `l9,4;l8,3` is the only reply-risk-shortlisted root and keeps the stronger floor (`1191` vs shipping `730`).
- Shipping's `l9,4;l8,5` is a full-pool `DrainerSafetyRecovery` root outside the frontier shortlist, loses under frontier's own reply-risk comparator (`shipping_vs_frontier=false`), and is only reached through shipping's `search_only_engine_allowed_head` ordering. Treat that seam as another shipping-only search ordering mismatch, not a live frontier omission.
- `white_profile_config_ordering_probe` closes the last obvious hidden-config hypothesis for the remaining white seams. On both the `ply9` search-order board and the late Fast hotspot, shipping and frontier use the same depth, node budget, reply-risk shortlist budget, and scoring weights.
- The difference on those boards is purely structural and profile-level: shipping stays `selector=false`, `head_rerank=true`, `mode=ProV1`, while frontier stays `selector=true`, `head_rerank=false`, `mode=ProV2` with the extra ProV2 guards. Treat those white seams as search-profile semantics, not per-board config misses.
- `white_ordering_rerank_semantics_probe` sharpens that split. On `white_ply9_search_ordering`, shipping `l9,4;l8,5` is already rank `0` on the frontier root set, is `Accepted` by `classify_turn_engine_rerank_override`, passes `turn_engine_allowed_rerank_override_candidate`, and stays compatible with the ProV2 advisor.
- The late white Fast hotspot does not share that property. Shipping `l8,5;l7,7;l8,8` is rejected by `ProgressGate` and is not an allowed rerank candidate even on shipping's own root set, so it is not another hidden rerank omission.
- Even the narrower rerank-admissible read does not make `white_ply9_search_ordering` promotable. A frontier-local rerank-semantics fallback fixes `l9,4;l8,3` locally and still fails retained `pro-reliability` at `0.9167 / 0.8333 / 0.9167`, because Normal rotates onto other early-white engine-disabled seams such as `l8,5;l7,6` vs shipping `l8,7;l7,8` and `l8,5;l7,6` vs shipping `l8,5;l7,4`.
- Treat rotated residue from a discarded challenger as provisional until a clean-tree retained trace confirms it. The discarded white rerank fallback temporarily rotated Normal onto early-white engine-disabled boards, but the cleaned promoted package collapses back to the white `l9,4;l8,3` vs `l9,4;l8,5` search-order family.
- `white_search_order_allowed_head_probe` closes the remaining “maybe the frontier root set just lacks shipping's rerank head” read. On both white search-order siblings, shipping's rerank engine still picks `l9,4;l8,5` when it is fed the frontier root set, while frontier's own rerank config still prefers `l9,4;l8,3`.
- That means the live white residue is deeper than root-set reachability or rerank admissibility alone. The split is in rerank-engine profile semantics (`ProV1` shipping rerank vs `ProV2` frontier rerank), not in whether `l9,4;l8,5` is present or allowed on frontier.
- `white_search_order_rerank_mode_probe` narrows that profile split one step further. Forcing only the frontier rerank engine mode from `ProV2` to `ProV1` already flips both white sibling boards to shipping `l9,4;l8,5`, while shipping still chooses `l9,4;l8,5` even when its rerank mode is forced to `ProV2`.
- So the live seam is not “all ProV2 rerank semantics are bad” in general. It is specific to frontier's `ProV2` rerank behavior on those boards, which means a future runtime spend has to explain that profile-specific mismatch rather than just proving the shipping root is reachable.
- `white_search_order_rerank_budget_probe` narrows that mismatch to frontier's rerank own-search breadth. On both white search-order siblings, swapping only frontier's rerank own caps (`own_seed_cap`, `own_beam`, `per_node_family_cap`, `step_cap`) to the shipping `ProV2` values already flips the best allowed-head plan to `l9,4;l8,5`, while swapping only reply caps or only expansion cap leaves frontier on `l9,4;l8,3`.
- So the live white residue is not waiting on reply-search breadth or a larger expansion pool. The concrete frontier-side mismatch surface is rerank own-search breadth under `ProV2`.
- `white_search_order_rerank_own_cap_probe` narrows that again to the smallest active levers. `step_cap` alone flips both white siblings to shipping `l9,4;l8,5` and reproduces the shipping rerank utility on both boards; `own_seed_cap` alone also flips both boards but does not reproduce the same rerank utility on the Fast board.
- `own_beam` alone and `per_node_family_cap` alone do not move the live white boards.
- `white_search_order_seed_step_scope_probe` kills the simplest runtime read of that result. On the two white search-order siblings, both `own_seed_cap` and `step_cap` still flip frontier to shipping `l9,4;l8,5`. But on `white_late_fast_hotspot`, shrinking either knob drags frontier off its current `l9,5;l8,6` plan onto a different third spirit line, while shipping's rerank on the same frontier head set stays on `l9,5;l8,6`.
- So the concrete frontier-side mismatch is rerank own-search depth/seed, but neither broad `own_seed_cap` shrink nor broad `step_cap` shrink is a safe runtime lever for the remaining white residue.
- A narrower runtime attempt still failed before the canonical loop. Clamping frontier `ProV2` rerank `step_cap` to `1` only on the exact white `turn=3 / mons_moves=1 / no-action / mana-only / window=1 / deny=1 / drainer_safety<0` board class did not move the live runtime decision on either white sibling.
- On that local slice, frontier still returned through `engine_post_search`, the accepted head stayed `l9,4;l8,3`, and the approved shortlist stayed a singleton on the same root.
- So the remaining white residue is not waiting on `turn_engine_rerank_config` alone. Any future white spend has to move shortlist/approved-root behavior, not just board-local rerank caps.
- `white_search_order_shortlist_gate_probe` narrows that approval-path read again. On both white siblings, shipping `l9,4;l8,5` already survives frontier candidate focus and appears in `candidate_indices`.
- It still dies at shortlist construction. The approved shortlist stays a singleton on vulnerable `l9,4;l8,3` because the score gap to shipping is still about `809k`, far beyond the `165` shortlist margin, and the existing safe-progress sibling shortlist extension does not fire.
- So the remaining white residue is not waiting on candidate focus or the current safe-progress extension path either. Any future white spend has to explain a brand-new shortlist/approved-root reentry theory, or a deeper root-scoring normalization, rather than another rerank-cap or simple shortlist tweak.
- `white_search_order_selector_disable_probe` closes off the obvious wrapper-level config-mirroring read. On both white siblings and on `white_late_fast_hotspot`, forcing the incoming frontier runtime config to `selector=false`, `head_rerank=true`, shipping-like own caps, or even `TurnEngineMode::ProV1` still leaves the live decision on the same frontier `engine_post_search` result.
- That is a wrapper-plumbing fact, not evidence that selector-disabled semantics themselves are impossible. `select_frontier_pro_v2_guarded_inputs` re-enters through `apply_frontier_pro_v2_guarded_config`, so config-only selector-disable toggles do not survive wrapper entry.
- So do not spend another white wave on shallow runtime-config mirroring against the frontier wrapper. If the white search-order family is ever reopened, the spend has to change wrapper branching itself or move deeper shortlist/root scoring behavior.
- `white_search_order_wrapper_branch_probe` shows what happens when that wrapper reapply is truly bypassed. Raw `search-only + ProV2` still keeps frontier's current white outputs, raw `search-only + shipping own caps + ProV2` fixes the two `ply9/ply11` siblings but not the late Fast hotspot, and raw `search-only + ProV1` matches shipping on all three local white seams.
- That still is not a live runtime answer. `white_search_order_raw_prov1_scope_probe` shows the obvious broad gate for that line is too wide: the same raw `search-only + ProV1` reroute also flips the retained `frontier_pro_v2_guarded_profile_keeps_v30_white_turn_three_mana_only_vulnerable_root` from `l8,4;l7,3` to shipping `l8,4;l8,5`, even though it shares the same coarse `turn=3 / mons_moves=1 / no-action / mana-only / window=1 / deny=1 / drainer_safety<0` context as the unresolved white siblings.
- So the white wrapper frontier is now tighter: actual search-only `ProV1` semantics are a real local explanation, but no coarse white context gate found so far can separate the unresolved siblings from the retained vulnerable guard. Any future white wrapper spend has to distinguish those boards with a narrower root-scoring or shortlist theory, not just a broad branch into shipping-like search-only semantics.
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
- `black_pro_lane_split_probe`
- `black_confirm_fast_lane_split_probe`
- `black_confirm_fast_setup_split_probe`
- `fast_hotspot_trace_probe`
- `white_late_fast_hotspot_probe`
- `white_profile_config_ordering_probe`
- `white_ordering_rerank_semantics_probe`
- `white_normal_ply11_search_only_split_probe`
- `white_search_order_allowed_head_probe`
- `white_search_order_rerank_mode_probe`
- `white_search_order_rerank_budget_probe`
- `white_search_order_rerank_own_cap_probe`
- `white_search_order_seed_step_scope_probe`
- `white_search_order_shortlist_gate_probe`
- `white_search_order_selector_disable_probe`
- `white_search_order_wrapper_branch_probe`
- `white_search_order_raw_prov1_scope_probe`

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
