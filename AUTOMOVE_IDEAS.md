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
- The package still clears the small retained loop: `guardrails` passed, `pro-triage` stayed at `target_changed=5 / off_target_changed=0`, exact-lite passed, and retained `pro-reliability` passed at `0.9167 / 0.9167 / 1.0000`.
- `pro-reliability-confirm` failed: `0.9375 / 0.9062 / 0.8750`, with Fast falling below the `0.90` floor.
- Advisory stage-1 CPU remains elevated but unchanged in character: `1.552 / 1.526 / 1.363` versus `shipping_pro_search`.
- This iteration spent one narrow black-only runtime reroute and killed it:
  - Inside `pro_v2_root_advisor_select_root`, swapping the ProV1 legacy selector from `shortlist_config` to the full runtime `config` re-enabled the reply-risk guard for that selector.
  - The live non-win root probe then aligned `vs_shipping_pro_black_recovery_branch` to shipping `l6,0;l6,1` through `ApprovedLegacySelector`, while the other four live walls stayed on the same surfaces as before.
  - The package cleared `guardrails`, retained `pro-triage` at `target_changed=4 / off_target_changed=0`, exact-lite, and advisory stage-1 CPU at `1.566 / 1.534 / 1.364`.
  - It still failed retained `pro-reliability` at `0.8333 / 0.9167 / 0.8333`, so the code was discarded.
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
- Do not spend another advisor/head patch on the rotated Pro white no-action seam `l10,4;l9,3` vs `l7,8;l6,9`; the incumbent already wins on frontier reply-floor and selected-utility metrics, so another approval override would be fighting the current model rather than fixing a local routing bug.
- If that board matters again, start from root scoring/root-rank generation for quiet early white mana-only roots instead of shortlist or post-search acceptance.
- Treat `black_recovery_branch` as an active seam, but do not reopen the simple turn-six spirit safety gate; that line fixes the local board and retained triage while still regressing Fast duel strength.
- If `black_recovery_branch` is reopened, start from the traced legacy-selector config difference instead of another threshold tweak. The live ProV1 legacy selector currently inherits `shortlist_config`, which disables the reply-risk guard; simply switching that selector to full `config` does align `l6,0;l6,1`, but it also fails retained `pro-reliability` at `0.8333 / 0.9167 / 0.8333`, so any future black fix has to be narrower than that global config swap.
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
- Do not reopen packages that are already archived in `docs/automove-archive.md` unless there is a brand-new shared hypothesis.
