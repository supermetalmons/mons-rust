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

- Date: `2026-04-22`
- Shipping decision: public Pro remains on `frontier_pro_v2_guarded`.
- Live retained duel blocker: the shipped frontier still fails the default retained duel corpus on the exact five-board live non-win surface.
- `vs_shipping_pro` loses only on `opening_reply_white`, `black_recovery_branch`, and `white_split_trace`.
- `vs_shipping_normal` loses only on `black_bridge_nonwin` and `white_head_acceptance`.
- `vs_shipping_fast` is currently clean.
- The latest narrow approval/head package did fix `opening_reply_white`, `black_recovery_branch`, and `white_head_acceptance`, but retained `pro-reliability` still failed at `0.8333 / 0.7500 / 0.7500` and it reintroduced fast/normal non-wins, so that partial three-wall package was discarded.
- The current live challenger is a cheap turn-three white package: it fixes `vs_shipping_pro_white_split_trace` and `vs_shipping_normal_black_bridge_nonwin`, passes `pro-triage` at `target_changed=5 / off_target_changed=0`, passes exact-lite, keeps `vs_shipping_fast` clean at `1.0000`, and holds advisory stage-1 CPU at `1.551 / 1.509 / 1.346`.
- That challenger is still not promotable: retained `pro-reliability` fails at `0.8333 / 0.8333 / 1.0000`, because the duel rotates onto two new white turn-three misses while `opening_reply_white`, `black_recovery_branch`, and `white_head_acceptance` still matter overall.
- New retained Pro miss under that package: white turn-3 no-action board `0 0 w 1 0 3 0 0 3 ...`, frontier `l9,3;l8,4`, shipping `l9,3;l10,4`.
- New retained Normal miss under that package: white turn-3 board `0 0 w 1 0 4 0 0 3 ...`, frontier `l10,2;l9,1`, shipping `l7,7;l6,6`.

## Next Hypothesis

- Target the exact five-board surface directly; do not spend more time on seed-tag reconciliation.
- The surviving white blocker is still post-search head acceptance, not just recovery reachability.
- The surviving black blocker is still approval on preserved spirit reentry, not shortlist reachability alone.
- The turn-three split/bridge pair is now cheap enough to keep. The next spend should harden the two newly surfaced white turn-three retained misses rather than reopen `white_split_trace` or `black_bridge_nonwin`.
- The next credible challenger still has to buy cheaper approval or head logic; retained seam alignment that pushes stage-1 CPU back into the `1.5x+` range is not promotable.

## No-Go Notes

- Do not reopen archived profiles, archived seams, or archived stages.
- Do not treat retained `primary_pro` churn by itself as promotion evidence.
- Do not spend canonical gates on a challenger that stays behaviorally inert at `target_changed=0 off_target_changed=0`.
- Do not treat “all live walls aligned” as enough if duel strength or CPU cost still fails.
- Do not reopen partial three-wall packages that only fix `opening_reply_white`, `black_recovery_branch`, and `white_head_acceptance`; that line can still fail retained duels and regress the currently clean fast pack.
- Do not reopen packages that are already archived in `docs/automove-archive.md` unless there is a brand-new shared hypothesis.
