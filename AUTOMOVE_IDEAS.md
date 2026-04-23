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
- The current package keeps a narrow live challenger:
  - It adds a white turn-three no-action recovery override so ProV2 can keep the safe `DrainerSafetyRecovery` root on the confirm board `l9,7;l8,7` vs shipping `l9,7;l10,8`.
  - The retained regression for that board now lives in `frontier_pro_v2_guarded_profile_prefers_shipping_white_confirm_pro_ply9_root`.
- This iteration kept a second narrow black late-fast repair inside the same live challenger:
  - It extends `pro_v2_root_advisor_black_late_window_mana_safety_override` so a rank-zero vulnerable one-window black `ManaTempo` root can yield to a same-lane safe sibling when that sibling restores the lane's progress surface.
  - The retained regression for that board now lives in `frontier_pro_v2_guarded_profile_prefers_shipping_black_late_fast_trace_root`.
- This iteration kept a third narrow early-white repair in the same package:
  - It extends `pro_v2_root_advisor_white_early_safe_progress_setup_competition_override` so a turn-three action+mana safe-progress incumbent can still yield to a spirit-own-setup challenger across a larger score gap when the challenger keeps the same progress surface, gains at least `+64` setup, and is at least six root-rank slots better.
  - The retained regression for that board now lives in `frontier_pro_v2_guarded_profile_prefers_shipping_white_fast_ply10_root`.
- The package still clears the small retained loop: `guardrails` passed, `pro-triage` stayed at `target_changed=5 / off_target_changed=0`, exact-lite passed, and retained `pro-reliability` passed at `0.9167 / 0.9167 / 1.0000`.
- The local retained slice for the new black fix stayed clean:
  - `frontier_pro_v2_guarded_profile_prefers_shipping_black_late_fast_trace_root` now aligns to shipping `l1,8;l0,8`.
  - `frontier_pro_v2_guarded_keeps_recovery_on_black_late_fast_trace_root` stayed aligned to shipping `l2,5;l0,5;l1,6`.
  - `frontier_pro_v2_guarded_profile_prefers_shipping_black_late_fast_second_lane_nonwin_root` stayed aligned to shipping `l0,8;l1,9`.
  - `frontier_pro_v2_guarded_rejects_late_black_plain_spirit_progress_head_without_concrete_gain` still passes.
- The package now clears confirm as well: `pro-reliability-confirm` passed at `0.9375 / 0.9062 / 0.9062`.
- The repaired white and black Fast walls are both gone from the `4x4` Fast non-win trace:
  - `l1,8;l1,9` vs shipping `l1,8;l0,8` no longer appears.
  - `l9,7;l8,6` vs shipping `l9,7;l7,6;l7,7` no longer appears.
- The remaining Fast non-wins in the confirm trace dropped to `3` boards:
  - White search-only split `l9,4;l8,3` vs `l9,4;l8,5`.
  - Black legacy/search seam `l7,1;l9,3` vs `l1,5;l2,7;l1,8`.
  - Black legacy/search seam `l6,2;l5,3` vs `l1,5;l3,7;l2,8`.
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

- The next real blocker is confirm, not the small retained loop.
- Keep the new white turn-three no-action recovery guard; it fixes a real confirm board without disturbing the retained small loop.
- Keep the new black late-fast safe-mana override as well; it removes the retained `l1,8;l1,9` vs `l1,8;l0,8` Fast seam without disturbing the small loop.
- Keep the new white early followup setup repair as well; it removes the retained `l9,7;l8,6` vs `l9,7;l7,6;l7,7` Fast seam and is now part of the promoted package.
- Do not spend another advisor/head patch on the rotated Pro white no-action seam `l10,4;l9,3` vs `l7,8;l6,9`; the incumbent already wins on frontier reply-floor and selected-utility metrics, so another approval override would be fighting the current model rather than fixing a local routing bug.
- If that board matters again, start from root scoring/root-rank generation for quiet early white mana-only roots instead of shortlist or post-search acceptance.
- Treat `black_recovery_branch` as an active seam, but do not reopen the simple turn-six spirit safety gate; that line fixes the local board and retained triage while still regressing Fast duel strength.
- Do not reopen the resolved late black Fast seam `l1,8;l1,9` vs `l1,8;l0,8`; that board is now covered by the retained suite and should stay fixed while confirm work moves elsewhere.
- Do not reopen the resolved early white Fast seam `l9,7;l8,6` vs `l9,7;l7,6;l7,7`; that board is now covered by the retained suite and should stay fixed while future confirm work moves elsewhere.
- If `black_recovery_branch` is reopened, start from the traced legacy-selector config difference instead of another threshold tweak. The live ProV1 legacy selector currently inherits `shortlist_config`, which disables the reply-risk guard; simply switching that selector to full `config` does align `l6,0;l6,1`, but it also fails retained `pro-reliability` at `0.8333 / 0.9167 / 0.8333`, so any future black fix has to be narrower than that global config swap.
- The narrower reply-risk-shortlist fallback is also not enough. Even when the black legacy-alignment override only searches the local `reply_risk_shortlist`, the line still dies at `0.9167 / 0.9167 / 0.8333`, so the next black attempt has to explain the retained Fast regression instead of just tightening the black chooser again.
- The next spend should start from the reduced `4x4` Fast residue instead:
  - White search-only split `l9,4;l8,3` vs `l9,4;l8,5`.
  - Black legacy/search seams `l7,1;l9,3` vs `l1,5;l2,7;l1,8` and `l6,2;l5,3` vs `l1,5;l3,7;l2,8`.
- The white `l9,4;l8,3` vs `l9,4;l8,5` board is not another advisor shortlist miss. Shipping changes that board through the search-only rerank path while frontier stays on the same vulnerable pre-accept root, so future work there should start from search-only/head routing rather than another reply-risk override.
- Do not reopen the resolved white turn-three sibling boards unless a future challenger regresses them.
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
- Do not read that `0.8333` Fast result as new shortlist collateral unless the trace says so. The traced Fast pack was just the pre-existing late black head-accept seam repeated twice.
- Do not reopen packages that are already archived in `docs/automove-archive.md` unless there is a brand-new shared hypothesis.
