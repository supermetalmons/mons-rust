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
- There is no second live challenger today.

## Latest Gate Snapshot

- Date: `2026-04-16`
- Shipping decision:
  - public Pro switched to `frontier_pro_v2_guarded`
- Fresh retained-baseline spot check before the switch:
  - vs `shipping_pro_search` Pro: `9-3`
  - vs `shipping_pro_search` Normal: `12-0`
  - vs `shipping_pro_search` Fast: `12-0`
- Last larger retained confirmation before the switch:
  - `2026-04-10` `pro-reliability-confirm`: `0.9062 / 0.9062 / 0.9062` with confidence `1.0000 / 1.0000 / 1.0000`

## Next Hypothesis

- None yet. The next live challenger should beat shipped `frontier_pro_v2_guarded`; `shipping_pro_search` is now only the retained baseline/reference.

## No-Go Notes

- Do not reopen archive profiles as active candidates.
- Do not spend from wrapper-only reroutes, hotspot-only output, or one traced seam without retained surface evidence.
- Do not treat the relaxed `700ms` cap as permission to keep quality-flat changes.
