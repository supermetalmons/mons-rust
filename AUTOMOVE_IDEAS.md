# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the operator flow, `docs/automove-knowledge.md` for durable rules, and `docs/automove-archive.md` for retired wave detail.

## Current State

- Public Pro routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` remains the retained search-only baseline.
- The live experiment surface is Pro-only: 2 retained profiles and 5 canonical stages.
- The default operator entrypoint is `./scripts/run-automove-canonical-loop.sh`.
- There is no second live challenger today.

## Latest Gate Snapshot

- Date: `2026-04-23`
- Shipping decision: public Pro remains on `frontier_pro_v2_guarded`.
- The current package is the promoted retained package:
  - It adds a white turn-three no-action recovery override so ProV2 can keep the safe `DrainerSafetyRecovery` root on the confirm board `l9,7;l8,7` vs shipping `l9,7;l10,8`.
  - The retained regression for that board now lives in `frontier_pro_v2_guarded_profile_prefers_shipping_white_confirm_pro_ply9_root`.
- This iteration kept a second narrow black late-fast repair inside the same live challenger:
  - It extends `pro_v2_root_advisor_black_late_window_mana_safety_override` so a rank-zero vulnerable one-window black `ManaTempo` root can yield to a same-lane safe sibling when that sibling restores the lane's progress surface.
  - The retained regression for that board now lives in `frontier_pro_v2_guarded_profile_prefers_shipping_black_late_fast_trace_root`.
- This iteration kept a third narrow early-white repair in the same package:
  - It extends `pro_v2_root_advisor_white_early_safe_progress_setup_competition_override` so a turn-three action+mana safe-progress incumbent can still yield to a spirit-own-setup challenger across a larger score gap when the challenger keeps the same progress surface, gains at least `+64` setup, and is at least six root-rank slots better.
  - The retained regression for that board now lives in `frontier_pro_v2_guarded_profile_prefers_shipping_white_fast_ply10_root`.
- This iteration kept and promoted a fourth narrow black late repair in the same package:
  - It adds `pro_v2_root_advisor_black_late_reply_risk_setup_override`, allowing a late black action+mana quiet no-progress `ManaTempo` incumbent to yield to a reply-risk-shortlisted quiet `SpiritImpact` own-setup challenger when that challenger preserves the same progress path, gains at least `+64` setup, and is within four root-rank slots.
  - The retained regression for that board now lives in `frontier_pro_v2_guarded_profile_prefers_shipping_black_late_setup_reply_risk_root`.
- This iteration kept and promoted a fifth narrow early-white runtime repair in the same package:
  - It adds `select_white_early_engine_disabled_fallback_inputs`, a wrapper-level reroute that mirrors shipping only on the exact weak-window turn-five white action+mana board class where frontier keeps engine selection enabled and gets trapped on a vulnerable `ManaTempo` root.
  - The retained regression for that board now lives in `frontier_pro_v2_guarded_uses_white_early_engine_disabled_fallback_on_normal_root`.
- This iteration kept and promoted a sixth narrow black late repair in the same package:
  - It adds `pro_v2_root_advisor_black_late_weak_window_safe_progress_setup_override`, letting a late black action+mana weak-window safe-progress incumbent yield to a reply-risk-shortlisted `SpiritImpact` own-setup progress root when the exact context is only `window<=1 / deny<=1`, utility stays competitive, setup gain is at least `+64`, and the root-rank gap stays within four.
  - The retained regression for that board now lives in `frontier_pro_v2_guarded_profile_prefers_shipping_black_confirm_fast_setup_root`.
- Board-local confirm diagnostics collapsed the earlier two-board black Fast residue into one real live seam:
  - On `l0,0;l1,1` vs shipping `l7,1;l8,0`, frontier already matches shipping on the current retained package.
  - The only live confirm Fast approval miss was `l0,5;l1,5` vs shipping `l2,5;l3,7;l2,8`, where legacy and shipping already agree on the spirit own-setup progress root.
- This iteration spent the remaining turn-six black residue board directly and killed that line:
  - It added a wrapper-level `select_black_progress_setup_engine_disabled_fallback_inputs` that mirrored shipping on the exact black turn-six action+mana weak-window board where frontier keeps the safe-progress root `l7,1;l9,3` and shipping disables engine selection to choose the spirit-own-setup line `l1,5;l2,7;l1,8`.
  - Locally, that fixed the residue board and kept the nearby retained white engine-disabled, late black Fast, and late black reply-risk setup walls clean.
  - The package still cleared `guardrails`, retained `pro-triage` at `target_changed=5 / off_target_changed=0`, exact-lite, advisory stage-1 CPU at `1.555 / 1.526 / 1.369`, and retained `pro-reliability` at `0.9167 / 0.9167 / 1.0000`.
  - `pro-reliability-confirm` failed at `0.9375 / 0.9062 / 0.8750`, so the code was discarded.
  - The confirm-sized Fast non-win trace showed why the wrapper is too shallow: alongside the old white search-only split `l9,4;l8,3` vs shipping `l9,4;l8,5`, the pack rotated onto two later black engine-disabled seams, `l0,0;l1,1` vs shipping `l7,1;l8,0` and `l0,5;l1,5` vs shipping `l2,5;l3,7;l2,8`.
- The package still clears the small retained loop: `guardrails` passed, `pro-triage` stayed at `target_changed=5 / off_target_changed=0`, exact-lite passed, and retained `pro-reliability` passed at `0.9167 / 0.9167 / 1.0000`.
- The local retained slice for the new black fix stayed clean:
  - `frontier_pro_v2_guarded_profile_prefers_shipping_black_late_fast_trace_root` now aligns to shipping `l1,8;l0,8`.
  - `frontier_pro_v2_guarded_keeps_recovery_on_black_late_fast_trace_root` stayed aligned to shipping `l2,5;l0,5;l1,6`.
  - `frontier_pro_v2_guarded_profile_prefers_shipping_black_late_fast_second_lane_nonwin_root` stayed aligned to shipping `l0,8;l1,9`.
  - `frontier_pro_v2_guarded_rejects_late_black_plain_spirit_progress_head_without_concrete_gain` still passes.
- The local retained slice for the new white fix stayed clean:
  - `frontier_pro_v2_guarded_uses_white_early_engine_disabled_fallback_on_normal_root` now aligns to shipping `l9,5;l8,3;l7,4` while preserving the frontier `pre_accept` root `l8,5;l7,6`.
  - `frontier_pro_v2_guarded_profile_prefers_shipping_white_late_mana_sibling_duel_normal_root` stayed aligned to shipping `l7,7;l6,5;l6,6`.
  - `frontier_pro_v2_guarded_profile_prefers_shipping_black_late_fast_trace_root` stayed aligned to shipping `l1,8;l0,8`.
- The package now clears confirm as well: `pro-reliability-confirm` passed at `0.9375 / 0.9062 / 0.9375`.
- The refreshed canonical loop for the new black weak-window setup rescue stayed clean:
  - `guardrails` passed.
  - `pro-triage` stayed at `target_changed=5 / off_target_changed=0`.
  - exact-lite passed.
  - advisory stage-1 CPU stayed in the same band at `1.563 / 1.522 / 1.365`.
  - retained `pro-reliability` passed at `0.9167 / 0.9167 / 1.0000`.
  - `pro-reliability-confirm` passed at `0.9375 / 0.9062 / 0.9375`.
- The earlier engine-disabled early-white Normal seam `l8,5;l7,6` vs shipping `l9,5;l8,3;l7,4` is now gone from the retained and confirm surfaces.
- The repaired white and black Fast walls are both gone from the `4x4` Fast non-win trace:
  - `l1,8;l1,9` vs shipping `l1,8;l0,8` no longer appears.
  - `l9,7;l8,6` vs shipping `l9,7;l7,6;l7,7` no longer appears.
- The repaired black confirm Fast setup wall is also gone from confirm:
  - `l0,5;l1,5` vs shipping `l2,5;l3,7;l2,8` no longer appears.
- The earlier black confirm seam `l6,2;l5,3` vs shipping `l1,5;l3,7;l2,8` is now gone from the retained surface.
- The remaining unresolved black diagnostic board `l7,1;l9,3` vs `l1,5;l2,7;l1,8` is a different class from that fixed seam:
  - Frontier already keeps the safe-progress incumbent on the full pool.
  - The candidate-only ProV1 replay prefers a different spirit root than shipping.
  - Treat it as a progress-vs-setup/model-ranking mismatch, not another late black reply-risk shortlist miss.
- The ignored `black_progress_vs_setup_residue_probe` is now more useful than before:
  - It confirms the actual shipping move on that board is still `l1,5;l2,7;l1,8`, not the candidate-only ProV1 replay `l1,5;l3,7;l2,8`.
  - Shipping reaches that move with `engine_disabled`, while frontier stays on `frontier_execute` with `l7,1;l9,3`.
- This iteration spent a broader white turn-three no-action recovery cut and killed it:
  - It widened `pro_v2_root_advisor_white_turn_three_no_action_recovery_override` from `mons_moves_count == 0` to `<= 1` and paired that with a post-search head reject for same-lane vulnerable `ManaTempo -> DrainerSafetyRecovery` recovery pairs.
  - Locally, that did fix the retained white Fast seam `l9,4;l8,3` vs shipping `l9,4;l8,5`, and it also aligned the older vulnerable white mana-only board `l8,4;l7,3` to shipping `l8,4;l8,5`.
  - The package still cleared `guardrails`, retained `pro-triage` at `target_changed=4 / off_target_changed=0`, exact-lite, and advisory stage-1 CPU at `1.551 / 1.527 / 1.365`.
  - It still failed retained `pro-reliability` at `0.9167 / 0.7500 / 0.9167`. The Normal non-win trace rotated onto engine-disabled early white boards, including `l8,5;l7,6` vs shipping `l8,7;l7,8`, `l9,4;l8,5` vs `l9,4;l9,3`, and `l8,5;l7,6` vs `l9,5;l8,3;l7,4`, so the code was discarded.
- This iteration spent the white search-only split directly and killed that line too:
  - Both runtime-variant forms targeted the same turn-three mana-only board `l9,4;l8,3` vs shipping `l9,4;l8,5`: first by re-querying shipping locally, then by choosing the nearby safe `DrainerSafetyRecovery` challenger from frontier's own ranked roots.
  - Both versions fixed the local `ply9` board and kept the nearby retained white confirm, white Fast, and black late-fast walls clean.
  - Both still failed retained `pro-reliability` at `0.9167 / 0.8333 / 0.9167`, with Normal below the floor at confidence `0.9807`.
  - The Normal non-win trace still included the engine-disabled early-white split `l8,5;l7,6` vs shipping `l9,5;l8,3;l7,4`, so fixing `l9,4;l8,3` in isolation is not enough. The code was discarded.
- Advisory stage-1 CPU remains elevated but unchanged in character: `1.552 / 1.526 / 1.363` versus `shipping_pro_search`.
- With the new black late-fast repair, the small-loop advisory stage-1 CPU stayed in the same band at `1.562 / 1.529 / 1.362`.
- With the white early followup repair added, the small-loop advisory stage-1 CPU stayed in the same band at `1.559 / 1.523 / 1.361`.
- This iteration spent one narrow black-only runtime reroute and killed it:
  - Inside `pro_v2_root_advisor_select_root`, swapping the ProV1 legacy selector from `shortlist_config` to the full runtime `config` re-enabled the reply-risk guard for that selector.
  - The live non-win root probe then aligned `vs_shipping_pro_black_recovery_branch` to shipping `l6,0;l6,1` through `ApprovedLegacySelector`, while the other four live walls stayed on the same surfaces as before.
  - The package cleared `guardrails`, retained `pro-triage` at `target_changed=4 / off_target_changed=0`, exact-lite, and advisory stage-1 CPU at `1.566 / 1.534 / 1.364`.
  - It still failed retained `pro-reliability` at `0.8333 / 0.9167 / 0.8333`, so the code was discarded.
- This iteration spent an even narrower black-only runtime fallback and killed it too:
  - Instead of changing the legacy selector globally, the local candidate only let `pro_v2_root_advisor_black_legacy_alignment_override` search the current `reply_risk_shortlist` for the best vulnerable mana challenger on the weak plain-spirit black seam.
  - That aligned `vs_shipping_pro_black_recovery_branch` to shipping `l6,0;l6,1`, passed a focused retained board assertion, and left the nearby white confirm and black post-search retained checks intact.
  - The package still cleared `guardrails`, retained `pro-triage` at `target_changed=4 / off_target_changed=0`, exact-lite, and advisory stage-1 CPU at `1.561 / 1.522 / 1.367`.
  - It still failed retained `pro-reliability` at `0.9167 / 0.9167 / 0.8333`, so the code was discarded.
- This iteration traced that `0.8333` Fast failure instead of reopening another black chooser:
  - Replaying `smart_automove_pro_reliability_nonwin_trace_probe` with the shortlist-local fallback and `duel_filter=vs_shipping_fast` produced exactly `2` non-wins.
  - Both non-wins collapsed to the same already-pinned late black head-accept seam on `3 1 b 1 0 2 0 0 14 ...`, where frontier accepts `l1,8;l1,9` at `engine_post_search` while shipping stays on `l1,8;l0,8`.
  - So the shortlist-local black fallback did not introduce a new Fast regression surface; it still failed because that existing late black Fast seam remained untouched.
- This turn did not spend another canonical loop. A board-local probe on the rotated Pro white confirm seam `l10,4;l9,3` vs shipping `l7,8;l6,9` showed frontier already prefers the incumbent on its own metrics:
  - Reply floor: incumbent `431`, shipping root `338`.
  - Selected override utility: incumbent `TurnEngineUtility { ... eval_score: 431 }`, shipping root `TurnEngineUtility { ... eval_score: 338 }`.
  - The shortlist is still just those two safe `ManaTempo` roots, and neither gets a turn-engine projection.
- This iteration spent one narrow black-only runtime candidate and killed it:
  - A turn-six black spirit-reentry safety gate stopped the unsafe preserved-spirit fallback on `black_recovery_branch` and aligned that board to shipping `l6,0;l6,1`.
  - The cut cleared `guardrails`, retained `pro-triage` at `target_changed=4 / off_target_changed=0`, exact-lite, and advisory stage-1 CPU at `1.563 / 1.531 / 1.368`.
  - It still failed retained `pro-reliability` at `0.9167 / 0.9167 / 0.8333`, so the code was discarded.
- This iteration stopped at diagnostics instead of spending another gate:
  - On `black_recovery_branch`, a direct call to `pro_v2_root_advisor_black_legacy_alignment_override` already returns the shipping mana root `l6,0;l6,1`.
  - The focused ignored probe in `black_recovery_branch_legacy_alignment_probe` also shows the exact mismatch surface: a local ProV1 candidate replay on the same `candidate_indices` resolves to `l1,5;l2,7;l1,8`, while `pro_v2_legacy_selector_probe` still reports `l6,0;l6,1`.
  - A naive fallback that scanned qualifying mana siblings picked the wrong root `l6,0;l7,0`, so no production change was kept.
- The two rotated white turn-three retained misses are fixed:
  - Pro white turn-3 no-action board `0 0 w 1 0 3 0 0 3 ...` now aligns to `l9,3;l10,4`.
  - Normal white turn-3 board `0 0 w 1 0 4 0 0 3 ...` now aligns to `l7,7;l6,6`.
- Confirm traces show the local repair was real but not sufficient:
  - Pro still loses on `black_recovery_branch`.
  - The earlier white confirm board `l9,7;l8,7` vs `l9,7;l10,8` is gone from the current Pro confirm trace.
  - Pro confirm rotated onto the next white turn-three no-action seam: `l10,4;l9,3` vs shipping `l7,8;l6,9`.
  - The new probe shows that rotated white board is not another approval-order or head-acceptance miss; it is a root-scoring/model mismatch under current frontier metrics.
  - Normal and Fast stayed below the floor at the same aggregate rates, so the broader confirm-only white/black seam family remains unresolved.

## Next Hypothesis

- The current package is promotable. The next spend is no longer about confirm rescue.
- Keep the new white turn-three no-action recovery guard, the black late-fast safe-mana override, the white early setup repair, the black late reply-risk setup rescue, the exact white early engine-disabled wrapper fallback, and the new black late weak-window safe-progress setup rescue together; they now form the promoted retained package.
- Do not reopen the resolved black confirm seam `l6,2;l5,3` vs `l1,5;l3,7;l2,8`; that board is now covered by the retained suite and confirm passed with it in place.
- Do not reopen the direct runtime-variant white search-only recovery fallback on `l9,4;l8,3` vs `l9,4;l8,5`. Both the shipping-assisted and frontier-local versions still fail retained `pro-reliability` at `0.9167 / 0.8333 / 0.9167`.
- Do not reopen the resolved engine-disabled early-white Normal seam `l8,5;l7,6` vs shipping `l9,5;l8,3;l7,4` unless a future challenger regresses it. The kept fix is the exact runtime wrapper fallback, not another broader white recovery override.
- Do not reopen that white seam by broadening the turn-three no-action recovery override to `mons_moves_count == 1`. That line still fails retained `pro-reliability` at `0.9167 / 0.7500 / 0.9167` by rotating Normal onto engine-disabled early-white boards.
- Do not reopen the direct black engine-disabled shipping fallback on `l7,1;l9,3` vs `l1,5;l2,7;l1,8`. It fixes the local residue board and still fails `pro-reliability-confirm` at `0.9375 / 0.9062 / 0.8750`.
- Do not reopen the already-rotated black confirm Fast lane split `l0,0;l1,1` vs `l7,1;l8,0`; frontier already matches shipping there on the current retained package.
- Do not reopen the resolved black confirm Fast setup split `l0,5;l1,5` vs `l2,5;l3,7;l2,8`; that board is now covered by `frontier_pro_v2_guarded_profile_prefers_shipping_black_confirm_fast_setup_root` and confirm passed with it in place.
- The remaining black residue `l7,1;l9,3` vs `l1,5;l2,7;l1,8` is no longer a good place for another blind advisor spend.
  - The improved ignored `black_progress_vs_setup_residue_probe` now shows the shipping root is missing from `reply_risk_shortlist`, but that still is not the whole story.
  - Across the full frontier candidate pool, the strongest spirit-own-setup progress challenger under the current utility/followup model is actually `l1,5;l3,7;l2,8`, not shipping `l1,5;l2,7;l1,8`.
  - Shipping therefore is not doing a simple “pick the best full-pool own-setup progress root” step that frontier could safely mirror with another advisor family-competition override.
  - Do not spend canonical gates on that board again unless the local probe first explains the engine-disabled ordering that prefers shipping's `l1,5;l2,7;l1,8` over the stronger full-pool challenger.
- This iteration reopened the older shortlist-local black legacy fallback on the cleaned promoted package and killed it again.
  - The updated `black_recovery_branch_legacy_alignment_probe` now prints shortlist root details and confirms the key local fact: among the vulnerable mana roots already in `reply_risk_shortlist`, shipping `l6,0;l6,1` is the best-ranked challenger, while the earlier wrong score-leader `l6,0;l7,0` is worse-ranked.
  - Reinstating that shortlist-local fallback inside `pro_v2_root_advisor_black_legacy_alignment_override` did move the intended seam. `black_recovery_branch` aligned to shipping `l6,0;l6,1`, and the five-board live nonwin root probe collapsed to the older white seams.
  - The package still failed retained `pro-reliability` at `0.9167 / 0.9167 / 0.8333`, so the runtime change was discarded.
  - The new `pro` non-win was a later black lane split `l1,6;l1,7` vs shipping `l1,6;l1,5`; `normal` stayed on the old white `ply9` search-only split `l9,4;l8,3` vs `l9,4;l8,5`; and the two Fast non-wins had `first_diff=none`, so the gate still failed without a clean new behavioral target to keep.
  - Do not reopen that shortlist-local black legacy fallback again unless a future probe first explains both the later black lane split and the no-diff Fast gate failure.
- Do not reopen the resolved late black Fast seam `l1,8;l1,9` vs `l1,8;l0,8`, the resolved early white Fast seam `l9,7;l8,6` vs `l9,7;l7,6;l7,7`, the resolved black late setup seam `l6,2;l5,3` vs `l1,5;l3,7;l2,8`, the resolved black confirm Fast setup seam `l0,5;l1,5` vs `l2,5;l3,7;l2,8`, or the resolved white early engine-disabled seam `l8,5;l7,6` vs `l9,5;l8,3;l7,4` unless a future challenger regresses them.
- Any future challenger still has to respect stage-1 CPU pressure; a package that wins local seams while drifting further into the `1.5x+` advisory band is not an upgrade.

## No-Go Notes

- Do not reopen archived profiles, archived seams, or archived stages.
- Do not treat retained `primary_pro` churn by itself as promotion evidence.
- Do not spend canonical gates on a challenger that stays behaviorally inert at `target_changed=0 off_target_changed=0`.
- Do not treat “all live walls aligned” as enough if duel strength or CPU cost still fails.
- Do not reopen partial three-wall packages that only fix `opening_reply_white`, `black_recovery_branch`, and `white_head_acceptance`; that line can still fail retained duels and regress the currently clean fast pack.
- Do not reopen the blunt black turn-six spirit safety gate that aligns `black_recovery_branch` by banning unsafe preserved-spirit reentry; it already failed retained `pro-reliability` on Fast at `0.8333`.
- Do not paper over `black_recovery_branch` with a score-only mana fallback. The first scan-based attempt picked `l6,0;l7,0` instead of shipping `l6,0;l6,1`.
- Do not globally switch the ProV1 legacy selector from `shortlist_config` to full `config`; that reply-risk-on reroute aligns `black_recovery_branch` locally but still fails retained `pro-reliability` at `0.8333 / 0.9167 / 0.8333`.
- Do not reopen the reply-risk-shortlist-only black legacy fallback that picks the best-ranked vulnerable mana root from the local shortlist; it aligns `black_recovery_branch` and still dies on retained Fast at `0.8333`.
- Do not reopen that shortlist-local black legacy fallback just because the earlier late-Fast blocker was repaired. Replaying it on the cleaned promoted package still fails retained `pro-reliability` at `0.9167 / 0.9167 / 0.8333`.
- Do not read that `0.8333` Fast result as new shortlist collateral unless the trace says so. The traced Fast pack was just the pre-existing late black head-accept seam repeated twice.
- Do not broaden white turn-three no-action recovery from `mons_moves_count == 0` to `<= 1`, even with a paired head reject. That line fixes `l9,4;l8,3` locally and still dies at retained `pro-reliability` `0.9167 / 0.7500 / 0.9167` because Normal rotates onto engine-disabled early-white losses.
- Do not reopen packages that are already archived in `docs/automove-archive.md` unless there is a brand-new shared hypothesis.
