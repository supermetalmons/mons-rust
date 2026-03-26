# Automove Archive

This document keeps compressed history for retired automove waves. These IDs are archive-only context and are no longer valid experiment targets.

## Mar 9-16, 2026: Config Knob Exhaustion

- What was tried: remaining `SmartSearchConfig` structural toggles across Fast, Normal, and Pro; scoring retunes; quiescence; null-move; history heuristic; wider search variants.
- Why it stopped: one Normal promotion landed, but the rest of the knob space was either flat, noisy, or too expensive.
- Durable lesson: most remaining strength will not come from more global knob retuning. New progress needs narrower tactical signals or structural changes.

## Mar 18-20, 2026: Fast Tactical Uplift Parked

- What was tried: reply-risk, spirit-setup, drainer-safety, attacker-proximity, and exact-lite top-offs aimed at closing the `normal` vs `fast` gap.
- Why it stopped: many candidates failed triage, and the ones that moved first-duel evidence stalled or hit runtime cliffs.
- Durable lesson: do not reopen Fast with another micro-retune in the exhausted families. Reopen only with a genuinely new path.

## Mar 19-20, 2026: Pro Intent Planner V2 Stabilization

- What was tried: intent-first planner work, emergency-only injected roots, and extra Pro diagnostics around `runtime_pro_intent_planner_v2`.
- Why it stopped: front gates could be kept green, but direct reliability against the baseline stayed flat.
- Durable lesson: crisis-gated injection is safer than global injection, but a planner branch without direct selector lift does not deserve live-frontier space.

## Mar 20-24, 2026: Pro Turn-Engine Compression

- What was tried: the `runtime_pro_turn_engine_v2` through `v30` line, including selector caching, continuation reuse, utility guards, and many narrow wrapper splits.
- Why it stopped: useful engine/workflow code had landed, but too many branch IDs accumulated and the wave needed one retained frontier instead of more live variants.
- Durable lesson: keep shared engine infrastructure, archive wrapper-local branch IDs, and retain only one Pro turn-engine frontier at a time. The retained output of this wave is `runtime_pro_turn_engine_v30`, with `runtime_pro_turn_engine_v1` kept only as reference history.

## Mar 25-26, 2026: Stronger Pro Roadmap Follow-Through

- What was tried: `runtime_pro_turn_engine_v31` through `v52`, covering two-stage child ordering, reach cleanup, projection-profile narrowing, and later search/oracle follow-ups.
- Why it stopped: the line produced useful follow-through evidence but never finished direct `pro-reliability` in a practical promotion window.
- Durable lesson: these branches were a bounded follow-through wave, not separate long-lived frontiers.

## Mar 26, 2026: Post-Roadmap Exact-Window Follow-Ups

- What was tried: `v53` and `v54`, focused on spirit-preview fast paths and immediate tactical-window caching.
- Why it stopped: bounded hotspot and stage-1 CPU numbers improved, but the direct reliability wall stayed deeper in the exact path.
- Durable lesson: local window reuse alone is not enough if the returned wall is still in secure recursion and payload churn.

## Mar 26, 2026: Secure-Recursion Wrapper Follow-Ups

- What was tried: `v55` and `v56`, aimed at secure-mana prechecks and cached secure drainer-walk metadata.
- Why it stopped: both candidates lost at the bounded hotspot stage without earning the full path.
- Durable lesson: wrapper-local secure-recursion caches were the wrong cut. If the hotspot does not move immediately, discard the family.

## Mar 26, 2026: Secure-Recursion, Search-Summary, And Pickup-Window Follow-Ups

- What was tried: `v59` through `v62`, covering secure-mana dead-end skips, search-side board-summary reuse, pickup-window caching, and a deeper secure-pickup prune.
- Why it stopped: the front gates could stay green, but even the best branch still failed to finish direct `pro-reliability` in a practical window.
- Durable lesson: the next useful split has to hit the remaining exact wall directly. Do not reopen discarded prune or cache families just because they improved one bounded sample.

## Historical References

- `runtime_historical_*` checkpoints are historical baselines only.
- `runtime_eff_non_exact_v3` and similar mixed historical snapshots answered one-time comparison questions and were retired to reduce noise.
- Removed registry aliases and dead compatibility profiles belong here, not in the active registry.
