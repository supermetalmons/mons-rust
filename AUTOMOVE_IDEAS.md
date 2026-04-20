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
  - `frontier_pro_v3_quiet_score_guarded` fixed `vs_shipping_pro_opening_reply_white`, moved retained `primary_pro` by `5 / 62` with `off_target_changed=0`, and passed `guardrails` plus `runtime-preflight`, but direct `pro-reliability` vs shipped `frontier_pro_v2_guarded` still failed at `0.5833 / 0.7500 / 0.9167`, so the candidate code was discarded.
- Retained confirmation that still matters:
  - `2026-04-10` `pro-reliability-confirm`: `0.9062 / 0.9062 / 0.9062` with confidence `1.0000 / 1.0000 / 1.0000`

## Next Hypothesis

- Quiet lower-scored root suppression is a real local repair, but it only fixed `vs_shipping_pro_opening_reply_white` and still lost the direct frontier-vs-shipping duel.
- The remaining live wall still includes `vs_shipping_pro_black_recovery_branch`, `vs_shipping_pro_white_split_trace`, `vs_shipping_normal_black_bridge_nonwin`, and the `vs_shipping_normal_white_head_acceptance` handoff where shipping still reaches `search_only_forced_prepass`.
- The next credible Pro challenger has to improve broader duel strength, not just patch quiet-root acceptance or isolated probe boards.

## No-Go Notes

- Do not reopen archive profiles as active candidates.
- Do not spend from wrapper-only reroutes, hotspot-only output, or one traced seam without retained surface evidence.
- Do not reopen exact live-seam shipping-alignment overrides. They can clear triage and preflight without being remotely promotable in direct duels.
- Do not reopen quiet-score-only root guards as a standalone challenger. They can move retained `primary_pro` by `5 / 62` and still fail direct duels.
- Do not treat the relaxed `700ms` cap as permission to keep quality-flat changes.
