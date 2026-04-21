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
  - `frontier_pro_v3_white_reply_head_guarded` tried a candidate-only white vulnerable-window head reject plus quiet-mana reply-score guard, and the candidate config was threaded through the white scoring-window fallback, but the live non-win root probe still stayed unchanged on `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_white_split_trace`, and `vs_shipping_normal_white_head_acceptance`; `white_split_trace` only changed the approval reason label to `ApprovedFamilyCompetition`, so the candidate code was discarded before canonical gates.
- Retained confirmation that still matters:
  - `2026-04-10` `pro-reliability-confirm`: `0.9062 / 0.9062 / 0.9062` with confidence `1.0000 / 1.0000 / 1.0000`

## Next Hypothesis

- Generic white quiet-mana score guards are not enough by themselves here: even after threading the candidate through the white scoring-window fallback, the live white probe boards stayed behavior-identical.
- The remaining white wall still includes `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_white_split_trace`, and the `vs_shipping_normal_white_head_acceptance` handoff where shipping still reaches `search_only_forced_prepass`.
- The next credible Pro challenger has to target the actual white family-competition override or the pre-search root-move features that keep the vulnerable window head admissible; a reason-label change without a root change is not enough to spend canonical gates.

## No-Go Notes

- Do not reopen archive profiles as active candidates.
- Do not spend from wrapper-only reroutes, hotspot-only output, or one traced seam without retained surface evidence.
- Do not reopen exact live-seam shipping-alignment overrides. They can clear triage and preflight without being remotely promotable in direct duels.
- Do not reopen quiet-score-only root guards as a standalone challenger. They can move retained `primary_pro` by `5 / 62` and still fail direct duels.
- Do not reopen search-only forced-prepass-priority as a standalone challenger. It can stay completely inert even when threaded through the scoring-window fallback.
- Do not reopen candidate-only white vulnerable-window head rejects or quiet-mana reply-score guards as a standalone challenger. They can reach the targeted white boards and still leave the selected roots unchanged.
- Do not spend canonical gates on a candidate that leaves the live non-win root probe unchanged.
- Do not treat the relaxed `700ms` cap as permission to keep quality-flat changes.
