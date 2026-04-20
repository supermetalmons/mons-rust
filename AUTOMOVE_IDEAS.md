# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired wave summaries to `docs/automove-archive.md`.

## Current State

- Public Pro now routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` remains the retained search-only baseline profile.
- `frontier_pro_v2_guarded` is the only retained Pro frontier and the shipped Pro path.
- Probe paths remain diagnostic-only and do not imply shipping behavior.
- The live experiment surface is now Pro-only: 2 active profiles and 5 canonical stages.
- The canonical operator entrypoint is `./scripts/run-automove-canonical-loop.sh`.
- The retained `primary_pro` surface now includes 2 duel-derived live non-win seams.
- There is no second live challenger today.

## Latest Gate Snapshot

- Date: `2026-04-21`
- Shipping decision:
  - public Pro switched to `frontier_pro_v2_guarded`
- Failed challenger:
  - `frontier_pro_v3_white_guarded` fixed the live `opening_reply_white` quiet-head seam and the `white_split_trace` safe-mana sibling seam, but it never moved `vs_shipping_normal_white_head_acceptance`: the board stayed on `search_only_engine_allowed_head` instead of shipping's `search_only_forced_prepass`, so the candidate was discarded before gates.
- Retained confirmation that still matters:
  - `2026-04-10` `pro-reliability-confirm`: `0.9062 / 0.9062 / 0.9062` with confidence `1.0000 / 1.0000 / 1.0000`

## Next Hypothesis

- The remaining cheap spend is the white turn-3 vulnerable-window handoff: prove a candidate can turn `search_only_engine_allowed_head` into the shipping-style forced-prepass recovery on `vs_shipping_normal_white_head_acceptance`.
- Do not reopen quiet late-head or safe-sibling code by themselves. They were real local fixes, but they were not enough to make a promotable challenger.

## No-Go Notes

- Do not reopen archive profiles as active candidates.
- Do not spend from wrapper-only reroutes, hotspot-only output, or one traced seam without retained surface evidence.
- Do not treat the relaxed `700ms` cap as permission to keep quality-flat changes.
