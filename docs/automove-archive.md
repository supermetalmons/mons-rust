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

## Apr 5, 2026: Selective Allowed-Root Scratch Line Retired

- What was tried: a short `v31` scratch line that re-opened selective allowed-root planner access for close non-tactical ProV2 clusters while continuing shared deferred-progress selector repairs.
- Why it stopped: the scratch line changed `pro-triage`, but both the scratch line and the retained challenger stayed flat at `0.5000` on the cheap direct duel gate versus current Pro, Normal, and Fast.
- Durable lesson: keep the shared selector repairs, but do not retain a new profile ID unless it moves direct duel evidence. Unsupported scratch profiles should be collapsed back into retained-shared code or retired immediately.

## Apr 5, 2026: Speculative Immediate-Score Clamp Retired

- What was tried: retained shared-code clamps that forced speculative `SpiritImpact` / `Safe*Progress -> ImmediateScore` heads to preserve first-chunk score or normal-safety, plus a setup-gain-only promotion for `spirit_own_mana_setup_now` roots.
- Why it stopped: `pro-triage(primary_pro)` stayed at `5/52`, only reshuffling `primary_spirit_setup`, and `pro-reliability` regressed to `win_rate=0.7500` vs current Pro while staying flat at `0.5000` vs current Normal.
- Durable lesson: on live ProV2 misses, inspect `pre_accept` vs final `engine_post_search` output first. Broad first-chunk non-regression clamps and setup-gain-only setup promotion are too blunt for the current wall.

## Apr 5, 2026: Probe-Led Followup/Safety Split Retired

- What was tried: a retained shared-code split that loosened `spirit_own_mana_setup_now` followup competition and added a close `Safe*Progress` head normal-safety block after the runtime-faithful retained-churn probe identified exact seams.
- Why it stopped: the runtime-faithful retained-churn probe was unchanged on the targeted seams, so the split was killed before `guardrails` and `pro-reliability`.
- Durable lesson: when a shared selector or acceptance tweak does not fire on the exact runtime-faithful seam, stop immediately and keep only the probe or seam map.

## Apr 5, 2026: Runtime-Faithful Spirit/Clamp Repairs Retained But Not Promotable

- What was tried: two retained shared-code repairs on `runtime_pro_turn_engine_v30`: reject a forced low-ranked plain `SpiritImpact` sibling on `primary_spirit_setup`, and skip the broad black turn-two low-budget clamp on full-resource `action+mana` states that regressed `primary_black_reliability_opening_3_ply4`.
- Why it stopped: both fixes were real and reduced `primary_pro` churn from `4/52` to `2/52`, but `pro-reliability` stayed flat at `win_rate=0.8333` vs current Pro, `0.5000` vs current Normal, and `0.6667` vs current Fast.
- Durable lesson: local runtime-faithful seam repairs can be correct and still fail promotion. Once the duel gate stays flat after the seam map collapses, stop the line and only reopen with a duel-linked hypothesis, not another local selector cleanup.

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
