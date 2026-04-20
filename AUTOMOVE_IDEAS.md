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
  - `frontier_pro_v3_live_seam_override` exact-matched the live seam boards, moved retained `primary_pro` by `2 / 62` with `off_target_changed=0`, and cleared `runtime-preflight`, but direct `pro-reliability` vs shipped `frontier_pro_v2_guarded` still failed at `0.5000 / 0.7500 / 0.8333`, so the candidate code was discarded.
- Retained confirmation that still matters:
  - `2026-04-10` `pro-reliability-confirm`: `0.9062 / 0.9062 / 0.9062` with confidence `1.0000 / 1.0000 / 1.0000`

## Next Hypothesis

- There is no remaining cheap live-seam spend. The white turn-3 vulnerable-window recovery was real, but even combining it with exact live seam alignment was still far short of direct frontier-vs-shipping reliability.
- The next credible Pro challenger has to improve broader duel strength, not just patch retained live seams or probe boards.

## No-Go Notes

- Do not reopen archive profiles as active candidates.
- Do not spend from wrapper-only reroutes, hotspot-only output, or one traced seam without retained surface evidence.
- Do not reopen exact live-seam shipping-alignment overrides. They can clear triage and preflight without being remotely promotable in direct duels.
- Do not treat the relaxed `700ms` cap as permission to keep quality-flat changes.
