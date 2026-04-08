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

## Apr 5, 2026: Reliability Hotspot Compare Diagnostic Killed

- What was tried: extended `smart_automove_pro_reliability_hotspot_probe` so it compares `runtime_pro_turn_engine_v30` against `runtime_current` on the bounded reliability hotspot corpus and prints candidate-vs-baseline move plus selector/exact deltas.
- Why it stopped: every real hotspot case was move-identical to current (`primary_spirit_setup`, `primary_black_loss_opening_a_ply19`, `human_win_pro_a`, `loss_opening_a`, `loss_opening_b`). The only changed move was the synthetic `quiet_positional` sample, so there was no credible duel-linked production seam to justify another canonical loop.
- Durable lesson: use hotspot compare to kill flat lines quickly. Counter inflation or selector-stage differences without real candidate-vs-current move divergence are not promotion evidence.

## Apr 8, 2026: PVS Acceptance Repair Retained But Not Promotable

- What was tried: a retained shared-code repair on `runtime_pro_turn_engine_v30` that blocks lower-scored unsafe `Safe*Progress` late heads from overriding the selected `primary_pvs_sensitive_search` root unless they bring a material non-eval override win, plus a retained regression fixture for that exact runtime seam.
- Why it stopped: the fix was real and collapsed `primary_pro` retained churn from `2/52` to `1/52`, but `pro-reliability` stayed flat at `win_rate=0.8333` vs current Pro, `0.5000` vs current Normal, and `0.6667` vs current Fast.
- Durable lesson: even the last live `engine_post_search` seam can be too local to matter. Once a retained late-head repair lands and the duel wall still does not move, stop the acceptance-only loop and reopen only with a broader duel-linked selector story.

## Apr 8, 2026: Human-Win-Pro-C Selector Split Killed Before Code Edits

- What was tried: reran the retained `human_win_pro_c` selector probe and the bounded `smart_automove_pro_reliability_hotspot_probe` after the PVS repair to check whether the remaining `1/52` triage drift had turned into a real duel seam.
- Why it stopped: the selector probe still showed a pure `pre_accept` safe-progress bias, but the hotspot compare remained move-identical to current on every real duel hotspot. Only the synthetic `quiet_positional` sample changed.
- Durable lesson: do not spend another shared ProV2 selector split on `human_win_pro_c` alone. If the real hotspot corpus stays unchanged, kill the line before code edits.

## Apr 8, 2026: Duel-Replay Progress Clamp Killed Before Preflight

- What was tried: added `smart_automove_pro_reliability_duel_trace_probe` to replay the exact `pro-reliability` seed corpus, compare candidate turns against a shadow `runtime_current` Pro continuation, and trace the first real divergence. Then tried a bounded large-search-deficit `Safe*Progress` acceptance clamp on one repeated fast-duel seam.
- Why it stopped: the probe was useful, but the production clamp left `pro-triage(primary_pro)` unchanged at `1/52`, still only `human_win_pro_c`. Per the retained Pro loop, the split was killed before `runtime-preflight` and `pro-reliability`.
- Durable lesson: keep the duel-replay probe, but do not retain another traced acceptance-only repair unless it also moves the cheap target surface.

## Apr 8, 2026: White Turn-Three Guard Plus Own-Setup Override Killed After Reliability

- What was tried: first narrowed the `runtime_pro_turn_engine_v30` white `turn=3`, mana-only mid-turn wrapper so it routed traced duel boards back to the current Pro surface instead of the broad fast fallback. Then paired that local guard repair with shared ProV2 own-setup-vs-progress overrides so `human_win_pro_c` would collapse and the traced normal-duel spirit-setup board would keep the current root.
- Why it stopped: the wrapper repair fixed the replayed white turn-three boards, but `pro-triage(primary_pro)` stayed flat at `1/52`, so it was too local on its own. The combined selector split moved `pro-triage(primary_pro)` to `2/52` by clearing `human_win_pro_c`, but it reopened `primary_black_reliability_opening_3_ply4` and regressed `pro-reliability` to `win_rate=0.7500` vs current Pro and `0.4167` vs current Normal/Fast.
- Durable lesson: traced white turn-three mana-only wrapper seams are real but do not deserve retained code unless they also move the cheap target surface, and broad same-lane own-setup overrides are too blunt. Do not retain either split without a cleaner selector story that keeps black reliability stable.

## Apr 8, 2026: Wrapper Bundle Plus Late-White Human Guard Killed Before Preflight

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe`, then bundled the traced wrapper/guard repairs under `runtime_pro_turn_engine_v30`: route all white `turn=3` mid-turn states and black `turn=2`/`turn=4` one-move `action+mana` states back to the current Pro surface. After that, tried one extra late white full-resource current-Pro guard aimed directly at the remaining `human_win_pro_c` triage drift.
- Why it stopped: the traced-board regression tests all passed and `guardrails` stayed green, but `pro-triage(primary_pro)` still stayed unchanged at `1/52`, with only `human_win_pro_c`, so the duel-backed wrapper bundle was still too local. The late white full-resource guard was even weaker: it did not change the selected move on `human_win_pro_c` at all.
- Durable lesson: a wrapper bundle can fix several real duel boards and still be non-promotable if the cheap target surface does not move, and late white full-resource current-Pro guards are not a useful shortcut for clearing `human_win_pro_c`.

## Apr 8, 2026: Late-White Omitted-Root Rescue Killed Before Preflight

- What was tried: kept the traced white `turn=3` plus black `turn=2`/`turn=4` wrapper bundle, then replaced the dead late-white exploration carve-out with a narrower reply-risk omitted-root rescue so late white turn-start `spirit_own_mana_setup_now + supermana_progress` roots could still beat safe non-setup progress heads on `human_win_pro_c`.
- Why it stopped: the targeted human regression and the black reliability selector probe both passed, `guardrails` stayed green, and `human_win_pro_c` matched current again, but that only collapsed `pro-triage(primary_pro)` to `0/52`, which still fails the gate because the cheap target surface no longer changes. A fresh duel replay on the same line showed the wrapper misses were gone and `vs current Pro` improved to `0` regressions / `2` improvements, but the live wall remained later `engine_post_search` drift: `vs current Normal` stayed at `2` regressions / `1` improvement and `vs current Fast` stayed at `3` regressions / `3` improvements.
- Durable lesson: do not retain a `human_win_pro_c`-only omitted-root rescue just because it neutralizes the last cheap drift. If the split drives `pro-triage(primary_pro)` to `0/52`, it is still non-promotable, and the next hypothesis has to target the remaining post-search duel wall rather than more wrapper or human-only fixes.

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
