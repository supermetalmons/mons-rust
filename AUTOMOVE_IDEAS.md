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
- The two rotated white turn-three retained misses are fixed:
  - Pro white turn-3 no-action board `0 0 w 1 0 3 0 0 3 ...` now aligns to `l9,3;l10,4`.
  - Normal white turn-3 board `0 0 w 1 0 4 0 0 3 ...` now aligns to `l7,7;l6,6`.
- Confirm traces show the local repair was real but not sufficient:
  - Pro still loses on `black_recovery_branch`.
  - The earlier white confirm board `l9,7;l8,7` vs `l9,7;l10,8` is gone from the current Pro confirm trace.
  - Pro confirm rotated onto the next white turn-three no-action seam: `l10,4;l9,3` vs shipping `l7,8;l6,9`.
  - Normal and Fast stayed below the floor at the same aggregate rates, so the broader confirm-only white/black seam family remains unresolved.

## Next Hypothesis

- The next real blocker is confirm, not the small retained loop.
- Keep the new white turn-three no-action recovery guard; it fixes a real confirm board without disturbing the retained small loop.
- Start from the next Pro confirm white no-action seam `l10,4;l9,3` vs `l7,8;l6,9`; the current blocker is still early white confirm routing, but the surface has moved one board deeper after the recovery fix.
- Treat `black_recovery_branch` as the second Pro seam after that new white board is understood; it still does not look like the same local bug.
- Do not reopen the resolved white turn-three sibling boards unless a future challenger regresses them.
- Any future challenger still has to respect stage-1 CPU pressure; a package that wins local seams while drifting further into the `1.5x+` advisory band is not an upgrade.

## No-Go Notes

- Do not reopen archived profiles, archived seams, or archived stages.
- Do not treat retained `primary_pro` churn by itself as promotion evidence.
- Do not spend canonical gates on a challenger that stays behaviorally inert at `target_changed=0 off_target_changed=0`.
- Do not treat “all live walls aligned” as enough if duel strength or CPU cost still fails.
- Do not reopen partial three-wall packages that only fix `opening_reply_white`, `black_recovery_branch`, and `white_head_acceptance`; that line can still fail retained duels and regress the currently clean fast pack.
- Do not reopen packages that are already archived in `docs/automove-archive.md` unless there is a brand-new shared hypothesis.
