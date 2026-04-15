# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired wave summaries to `docs/automove-archive.md`.

## Current State

- Shipping Pro is `shipping_pro_search`.
- `shipping_pro_search` is still the deployed search-only path.
- `frontier_pro_v2_guarded` is the only retained Pro frontier.
- Probe paths remain diagnostic-only and do not imply shipping behavior.
- The live experiment surface is now Pro-only: 2 active profiles and 5 canonical stages.
- The canonical operator entrypoint is `./scripts/run-automove-canonical-loop.sh`.
- There is no second live challenger today.

## Latest Gate Snapshot

- Date: `2026-04-10`
- Retained gate state before promotion:
  - `guardrails`: pass
  - `pro-triage(primary_pro)`: `changed=3/60`, `target_changed=3`, `off_target_changed=0`
  - `runtime-preflight`: pass
- Promotion proof:
  - `pro-reliability`: `0.9167` vs shipping Pro, `1.0000` vs shipping Normal, `1.0000` vs shipping Fast
  - `pro-reliability-confirm`: `0.9062` vs shipping Pro, `0.9062` vs shipping Normal, `0.9062` vs shipping Fast
  - confirm confidence: `1.0000 / 1.0000 / 1.0000`
  - confirm frontier average move time: `130.28ms / 161.62ms / 176.11ms`
- Final promotion seam:
  - the last blocking confirm-normal miss was an early-black advisor drift where reply-risk approval picked a weaker plain-spirit sibling over a stronger own-setup `SpiritImpact` root already in the shortlist

## Next Hypothesis

- None yet. Do not start a new Pro wave until there is one shared selector/search hypothesis that can challenge `shipping_pro_search`.

## No-Go Notes

- Do not reopen archive profiles as active candidates.
- Do not spend from wrapper-only reroutes, hotspot-only output, or one traced seam without retained surface evidence.
- Do not treat the relaxed `700ms` cap as permission to keep quality-flat changes.
