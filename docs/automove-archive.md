# Automove Archive

This document keeps short history for retired automove waves.

Everything here is archive-only context. These IDs are not valid experiment targets. Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the live workflow and `docs/automove-knowledge.md` for durable lessons that still matter. Full branch-by-branch detail lives in git history rather than this file.

## Mar 9-16, 2026: Config Knob Exhaustion

- What was tried: remaining `SmartSearchConfig` tuning across Fast, Normal, and Pro, including scoring retunes, quiescence, null-move, history heuristic, and wider-search variants.
- Why it stopped: one Normal promotion landed, but the remaining knob space was flat, noisy, or too expensive.
- Durable lesson: the easy global retune space is spent. Future gains need narrower tactical signals or shared structural changes.

## Mar 18-20, 2026: Fast Tactical Uplift Parked

- What was tried: reply-risk, spirit-setup, drainer-safety, attacker-proximity, and exact-lite top-offs aimed at closing the `normal` vs `fast` gap.
- Why it stopped: most candidates died in triage, and the few that moved first-duel evidence stalled or hit runtime cliffs.
- Durable lesson: do not reopen Fast for another micro-retune in the same exhausted families. Reopen only with a genuinely new code path.

## Mar 19-20, 2026: Pro Intent-Planner Line Retired

- What was tried: intent-first planner work, emergency-only injected roots, and extra diagnostics around `runtime_pro_intent_planner_v2`.
- Why it stopped: front gates could stay green, but direct reliability against the shipping baseline stayed flat.
- Durable lesson: crisis-gated injection is safer than global injection, but a planner line without direct selector lift does not deserve live-frontier space.

## Mar 20-24, 2026: Pro Turn-Engine Consolidation

- What was tried: the `runtime_pro_turn_engine_v2` through `runtime_pro_turn_engine_v30` wave, including selector caching, continuation reuse, utility guards, and many wrapper-local splits.
- Why it stopped: useful shared engine and workflow code landed, but too many branch IDs accumulated and the line needed one retained frontier instead of many live variants.
- Durable lesson: keep shared infrastructure, retire wrapper-local branch IDs, and retain only one Pro turn-engine frontier at a time. The retained output of this wave is `runtime_pro_turn_engine_v30`, with `runtime_pro_turn_engine_v1` kept only as reference history.

## Mar 25-Apr 4, 2026: Post-v30 Follow-Through Retired

- What was tried: `runtime_pro_turn_engine_v31` through `runtime_pro_turn_engine_v62`, then a broad set of v30-local replay repairs across white openings, black mana-only openings, later black `engine_post_search`, and later white selector/search seams.
- Why it stopped: many branches fixed real traced seams and some even cleared focused Pro-vs-Pro evidence, but they stayed too local. The broader `vs current Normal` wall did not move enough, and wrapper or acceptance repairs often gave quality back in the same-budget Pro duel.
- Durable lesson: real exact seams are not enough on their own. Once a branch is clearly local, does not move `pro-triage`, or does not improve direct duel quality, retire the family instead of extending it.

## Retired Families Worth Remembering

- Wrapper-only current-Normal reroutes and search-surface swaps
- Acceptance-only macro-head clamps
- Cache-shape and memo-shape micro-optimizations without a quality story
- Generic search-knob clamps without a shared live-family explanation
- Local white-only or black-only replay repairs that do not move the broader wall

## Historical References

- `runtime_historical_*` checkpoints are historical baselines only.
- Mixed historical snapshots such as `runtime_eff_non_exact_v3` answered one-time comparison questions and were retired to reduce noise.
- Removed registry aliases and dead compatibility profiles belong here, not in the active registry.
