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
- Failed iteration:
  - `frontier_pro_v3_advisor_window_guarded` fixed both corrected white live walls, passed `guardrails`, moved retained `primary_pro` by `5 / 62` with `off_target_changed=0`, and passed exact-lite, but retained `pro-reliability` vs `shipping_pro_search` still failed at `0.6667 / 0.6667 / 0.6667` with confidence `0.8062 / 0.8062 / 0.8062`.
- Retained confirmation that still matters:
  - `2026-04-10` `pro-reliability-confirm`: `0.9062 / 0.9062 / 0.9062` with confidence `1.0000 / 1.0000 / 1.0000`

## Next Hypothesis

- The corrected live probe is now trustworthy: `opening_reply_white` is a post-search head-over-advisor seam, while `normal_white_head_acceptance` is an early-white vulnerable-window recovery miss where the safer `DrainerSafetyRecovery` root already exists in the scored set.
- Fixing those two white walls together was still not enough. The failed challenger proved that white live-seam repairs plus clean retained triage do not automatically transfer into retained duel strength.
- There is no second live challenger today.
- The next credible Pro challenger has to explain the remaining duel losses that survive after those white repairs, especially the black retained churn still exposed by `primary_live_nonwin_black_vulnerable_spirit_reentry` and the extension-sensitive `primary_pro` seams.

## No-Go Notes

- Do not reopen archive profiles as active candidates.
- Do not spend from wrapper-only reroutes, hotspot-only output, or one traced seam without retained surface evidence.
- Do not reopen exact live-seam shipping-alignment overrides. They can clear triage and preflight without being remotely promotable in direct duels.
- Do not reopen quiet-score-only root guards as a standalone challenger. They can move retained `primary_pro` by `5 / 62` and still fail direct duels.
- Do not reopen search-only forced-prepass-priority as a standalone challenger. It can stay completely inert even when threaded through the scoring-window fallback.
- Do not reopen candidate-only white vulnerable-window head rejects or quiet-mana reply-score guards as a standalone challenger. They can reach the targeted white boards and still leave the selected roots unchanged.
- Do not reopen late-white low-budget selector exceptions or simple search-only white top-head conflict checks as standalone challengers. They can stay fully inert on the exact white walls they target.
- Do not spend canonical gates on a candidate that only fixes `vs_shipping_pro_white_split_trace` while leaving `vs_shipping_pro_opening_reply_white` and `vs_shipping_normal_white_head_acceptance` unchanged.
- Do not spend canonical gates on a candidate that leaves the live non-win root probe unchanged.
- Do not treat “fixed both white live walls” plus retained `primary_pro` movement `5 / 62` with `off_target_changed=0` as promotion evidence. That exact shape still failed retained `pro-reliability` at `0.6667` across Pro, Normal, and Fast.
- Do not treat the relaxed `700ms` cap as permission to keep quality-flat changes.
