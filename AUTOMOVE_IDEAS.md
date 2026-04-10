# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired wave summaries to `docs/automove-archive.md`.

## Current State

- Shipping Pro is `runtime_current`.
- `runtime_current` now ships through the promoted guarded `runtime_pro_turn_engine_v30` path.
- `runtime_pro_turn_engine_v30` is the only retained Pro frontier.
- The live experiment surface is now Pro-only: 2 active profiles and 5 canonical stages.
- There is no second live challenger today.

## Latest Gate Snapshot

- Date: `2026-04-10`
- Retained gate state before promotion:
  - `guardrails`: pass
  - `pro-triage(primary_pro)`: `changed=3/60`, `target_changed=3`, `off_target_changed=0`
  - `runtime-preflight`: pass
- Promotion proof:
  - `pro-reliability`: `0.9167` vs current Pro, `1.0000` vs current Normal, `1.0000` vs current Fast
  - `pro-reliability-confirm`: `0.9062` vs current Pro, `0.9062` vs current Normal, `0.9062` vs current Fast
  - confirm confidence: `1.0000 / 1.0000 / 1.0000`
  - confirm candidate average move time: `130.28ms / 161.62ms / 176.11ms`
- Final promotion seam:
  - the last blocking confirm-normal miss was an early-black advisor drift where reply-risk approval picked a weaker plain-spirit sibling over a stronger own-setup `SpiritImpact` root already in the shortlist

## Next Frontier

- Future Pro iteration should start only when there is one new shared selector/search hypothesis that can challenge `runtime_current`.
- Cleanup is complete enough to work from the reduced Pro-only surface; there is no live replacement candidate yet.

## No-Go Notes

- Do not reopen archive profiles as active candidates.
- Do not spend from wrapper-only reroutes, hotspot-only output, or one traced seam without retained surface evidence.
- Do not treat the relaxed `700ms` cap as permission to keep quality-flat changes.
