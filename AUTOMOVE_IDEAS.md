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
  - `frontier_pro_v3_selector_predisabled_probe` did not cut a new challenger. The retained live non-win probe now prints the actual frontier wrapper branch and selector disable reason, and that result killed the next guessed white spend before candidate code: `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, `vs_shipping_pro_white_plain_spirit_split`, `vs_shipping_pro_white_split_trace`, and `vs_shipping_normal_white_head_acceptance` all still ran through `frontier_execute` with `selector_disable_reason=pre_disabled`, so the runtime-effective path is still not the calibrated `selector=true` frontier shape.
- Retained confirmation that still matters:
  - `2026-04-10` `pro-reliability-confirm`: `0.9062 / 0.9062 / 0.9062` with confidence `1.0000 / 1.0000 / 1.0000`

## Next Hypothesis

- The blocker is now sharper than “which white guard should change”: on multiple live walls the frontier wrapper really does stay on `frontier_execute`, but the runtime still enters `smart_search_best_inputs_internal` with the selector already `pre_disabled`.
- That `pre_disabled` result is incompatible with the calibrated probe config, which still reports `profile_turn_engine_selector=true` on the same boards. The next live spend is explaining that mismatch before touching root heuristics again.
- `vs_shipping_normal_white_head_acceptance` is still the cleanest search-only handoff wall, but the same new diagnostic means its missing spend is not just a rerank acceptance rule. The runtime path is already different from the forced probe assumptions earlier in the stack.
- `vs_shipping_pro_white_split_trace` is still only a partial seam. Even though a stricter white mana-sibling clamp can move it, there is still no credible challenger until the shared `pre_disabled` frontier-execute mismatch is understood.
- The next credible Pro challenger has to prove why these live boards reach `frontier_execute` with `selector_disable_reason=pre_disabled`, then show which shared path change fixes `opening_reply_white` and `normal_white_head_acceptance` without reopening the retained Pro surface.

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
- Do not treat the relaxed `700ms` cap as permission to keep quality-flat changes.
