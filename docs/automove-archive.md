# Automove Archive

This document keeps short history for retired automove waves.

Everything here is archive-only context. These IDs are not valid experiment targets. Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the live workflow and `docs/automove-knowledge.md` for durable lessons that still matter. Full branch-by-branch detail lives in git history rather than this file.

## Apr 8, 2026: Human-Only Revival Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_human_win_pro_c_selector_probe` after the fresh direct-Pro white spirit-cluster probe, to check whether the only remaining retained cheap drift now justified a bounded human-only production retry.
- Why it stopped: `human_win_pro_c` still showed the same safe-progress / followup-floor selector shape, with `progress_competes=true` and `followup_progress_competes=true`, while the fresh direct-Pro white one-off remained a different risky-score spirit cluster. That meant the retained cheap surface was still isolated to `human_win_pro_c` and the live duel churn was still off-surface, so another human-only production split had no credible path through the cheap gate.
- Durable lesson: once the retained cheap surface is down to `human_win_pro_c` alone, do not treat that as a reason to reopen a human-only fix. If fresh duel seams still stay off-surface and count-`1`, kill the idea before code edits.

## Apr 8, 2026: Direct-Pro White Spirit Cluster Probe Killed At Diagnostics

- What was tried: after ruling out the fresh black direct-Pro seam, drilled into the one-off white regression `l4,9;l4,7;l5,7` vs current `l9,4;l8,3` and compared it to the retained `human_win_pro_c` selector surface.
- Why it stopped: the traced board was not the retained human seam. The focused probe showed `followup_progress_competes=false`, `progress_competes=false`, and `risky_score_competes=true`, with both the candidate spirit-own-setup root and the current non-spirit root still vulnerable; the forced head `l4,9;l4,7;l3,7` was present but still rejected. That made it another isolated risky-score spirit cluster rather than a duel-backed version of `human_win_pro_c`.
- Durable lesson: do not reopen `human_win_pro_c`-style production fixes from a one-off direct-Pro white spirit-own-setup drift unless the probe actually shows the retained followup-progress pattern. Reverse-polarity spirit-vs-progress intuition is not enough.

## Apr 8, 2026: Direct-Pro Black Spirit Cluster Probe Killed At Diagnostics

- What was tried: after the fresh `pro_turn_planner_reliability_v4` replay, drilled into the one-off direct-Pro black regression `l1,5;l2,3;l3,3` vs current `l1,5;l2,7;l1,8` and compared it to both retained black footholds, `primary_black_negative_deny_ply4` and `primary_black_late_accepted_head_ply4`.
- Why it stopped: the traced board did not match either retained black family. All spirit-pref competition gates stayed `false`, the forced head `l1,5;l1,7;l0,7` was present but still `accepted=false`, and the shortlist was just a vulnerable plain-spirit `supermana_progress` cluster with identical setup gain, while current differed only by taking a lower-ranked own-setup spirit root. That made it another isolated live seam rather than a retained early-black `negative_deny` or later-black accepted-head story.
- Durable lesson: do not reopen black production seams from a one-off spirit-vs-spirit drift unless a focused probe shows it actually lands on one of the retained black competition families. A rejected forced head plus all-false competition flags is a kill signal.

## Apr 8, 2026: Fresh Seed v4 Duel Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with another new seed tag, `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v4`, after the retained white forced-prepass probe ruled out the previous Fast family as live-only shell churn.
- Why it stopped: the replay again produced only count-`1` seams. `vs current Pro` finished at `2` regressions / `4` improvements / `6` flat, `vs current Normal` at `3` / `1` / `8`, and `vs current Fast` at `2` / `4` / `6`, with every recorded move pair unique. The traced regressions ranged across black accepted-spirit drift, white spirit-own-setup reranks, a white accepted head, and a white mana-tempo rerank, but none repeated and none landed on a retained cheap-surface foothold.
- Durable lesson: once two fresh replays in the same retained branch collapse into count-`1` churn, treat the apparent new families as seed-local noise unless they repeat and land on a retained surface. Do not cut production code from one-off accepted-head or spirit-setup-looking seams.

## Apr 8, 2026: White Fast Forced-Prepass Family Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` on the retained challenger with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v3`, which exposed one repeated white fast candidate family in `vs current Fast`: `l8,4;l8,5` appeared twice, once against current `l8,4;l7,3` and once against current `l8,4;l8,3`. Added a focused diagnostic probe, `smart_automove_pro_white_fast_forced_prepass_probe`, to compare the traced fast-duel board with the nearby retained `primary_white_fast_screen_opening_0_ply9` surface.
- Why it stopped: the probe showed the traced board already matched current at the real search surface, with both current and `pre_accept` choosing `l8,4;l7,3`; the challenger only diverged because `search_only_forced_prepass` overrode that search result and returned `l8,4;l8,5`. The nearby retained white fast screen fixture did not share the same targets or selector stage, so there still was no retained cheap-surface foothold strong enough to justify a production split.
- Durable lesson: even a repeated candidate-only early white family is not enough if the divergence lives only in `search_only_forced_prepass` and does not appear on a retained cheap surface. Keep the probe, kill the production idea.

## Apr 8, 2026: Negative-Deny Full-Loop Retry Killed At Reliability

- What was tried: reran the first shared early-black `negative_deny` selector override and, unlike the first diagnostic-only pass, carried it through the full canonical loop. The retained `primary_black_negative_deny_ply4` seam closed to current, `guardrails` passed, `pro-triage(primary_pro)` moved to `1/53` with only `human_win_pro_c`, `off_target_changed=0`, and `runtime-preflight` passed.
- Why it stopped: `pro-reliability` still failed at `0.8333` vs current Pro, `0.4167` vs current Normal, and `0.5833` vs current Fast. A fresh `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v3` duel trace showed the live wall had shifted to a later black accepted-head seam `l1,5;l1,7;l0,7` vs current `l3,2;l4,1`, repeated `2x` in direct Pro and `3x` in Normal, while the old early-black `l0,5;l1,6` seam still resurfaced once in Pro.
- Durable lesson: clearing the retained early-black foothold is not enough if the live wall simply moves later in the turn-engine path. When the first full-loop retry trades one black family for another and direct duel quality stays weak, kill the production change and keep only the knowledge.

## Apr 8, 2026: Combined Black Retained-Seam Retry Killed At Reliability

- What was tried: added a retained foothold for the later black accepted-head seam, `primary_black_late_accepted_head_ply4` plus `smart_automove_pro_black_late_accepted_head_probe`, then combined two black production cuts under `runtime_pro_turn_engine_v30`: the earlier shared early-black `negative_deny` override and a new late-black plain-spirit-progress rejection so `l1,5;l1,7;l0,7` would stop overriding current `l3,2;l4,1`.
- Why it stopped: both retained black seams collapsed to current, `guardrails` passed, `pro-triage(primary_pro)` moved to `1/54` with only `human_win_pro_c`, `off_target_changed=0`, and `runtime-preflight` passed, but `pro-reliability` still failed at `0.8333` vs current Pro, `0.5000` vs current Normal, and `0.5833` vs current Fast. A fresh default-seed duel trace showed the branch had not found a broader live wall: direct Pro regressed only once on the old white wrapper `l9,2;l8,3` vs `l10,7;l9,7`, Normal showed three one-off seams, and Fast still centered on the old white accepted-head pair `l9,4;l8,4` vs `l8,7;l7,8` repeated twice.
- Durable lesson: retaining a new later-black foothold was worthwhile, but a branch that closes both retained black seams can still be the wrong spend if the live duel wall simply snaps back to white wrapper and accepted-head churn. Keep the retained fixture/probe, kill the black production overrides.

## Apr 8, 2026: Repeated Black Negative-Deny Selector Override Killed At Triage

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v3`, which finally repeated the early black `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` family across all three direct duels. Added a new retained foothold for that seam, `primary_black_negative_deny_ply4`, plus `smart_automove_pro_black_negative_deny_selector_probe`, then tried one narrow shared selector override that ignored the plain-spirit `negative_deny` block for spirit-setup narrowing when a spirit-setup root already beat every non-spirit challenger.
- Why it stopped: the new retained fixture did collapse to current, the focused selector probe matched the intended shape, and `guardrails` passed, but `pro-triage(primary_pro)` still returned only the stale `human_win_pro_c` drift at `1/53` with `off_target_changed=0`. Per the retained runbook, that meant the selector cut was still too local, so it died before `runtime-preflight` and `pro-reliability`.
- Durable lesson: a repeated cross-duel family can deserve a retained fixture addition, but the first shared fix for that family still has to move the cheap frontier in a broader way. If the line closes the new retained seam and immediately snaps back to stale human-only churn, keep the fixture and probe but kill the production change.

## Apr 8, 2026: Fresh Seed v2 Duel Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with a new seed tag, `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v2`, to see whether the live wall had moved onto a new repeated family that could justify another retained `runtime_pro_turn_engine_v30` split.
- Why it stopped: the replay finished cleanly but every recorded move pair stayed at count `1`: `vs current Pro` ended at `3` regressions / `5` improvements / `4` flat, `vs current Normal` at `5` / `2` / `5`, and `vs current Fast` at `3` / `2` / `7`. The closest thing to a new family was a later black `SpiritImpact` takeover (`l1,5;l1,7;l0,7` vs current mana-tempo replies) that appeared once each in Pro and Fast, while the old early black `l0,5;l1,6` negative-deny seam resurfaced once each in Pro and Normal. Per the retained runbook, that still was not enough: there was no repeated duel seam and no retained `primary_pro` foothold, so the line died before code edits.
- Durable lesson: a fresh replay that produces only count-`1` seams is itself a stop signal. Do not force a production split out of cross-duel similarities alone; wait for a repeated family that also lands on retained `primary_pro`.

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

## Apr 8, 2026: ImmediateScore Near-Tie Acceptance Repair Killed After Reliability

- What was tried: added retained shared-code post-search guards for non-concrete deferred progress heads, unsafe delayed recovery heads, and a multi-chunk `ImmediateScore` near-tie `ManaTempo` sibling override, plus bounded duel-accept regression fixtures and a diagnostic post-search probe.
- Why it stopped: the split fixed the bounded duel-accept seams, cleared `human_win_pro_c`, passed `guardrails`, moved `pro-triage(primary_pro)` to `1/52` with `off_target_changed=0`, and passed `runtime-preflight`, but `pro-reliability` still failed at `win_rate=0.7500` vs current Pro, `0.6667` vs current Normal, and `0.5000` vs current Fast. A sampled duel trace still diverged later in a fast black turn-four `engine_post_search` decision (`l1,6;l2,7` vs current `l2,3;l3,2`).
- Durable lesson: even a real accepted-head repair can be too local. If the cheap gates are green but the live duel wall stays broad, kill the line and reopen only from the remaining duel-trace divergence, not another `human_win_pro_c` or near-tie acceptance clamp.

## Apr 8, 2026: Isolated Black Turn-Four Wrapper Repair Killed At Triage

- What was tried: added a narrow `runtime_pro_turn_engine_v30` current-Pro wrapper for black `turn=4`, `mons_moves=1`, `action+mana` states so the sampled fast-duel board would route from the injected `l1,6;l2,7` back to current `l2,3;l3,2`, plus a bounded regression test and focused selector probe.
- Why it stopped: the live duel board was fixed and `guardrails` passed, but `pro-triage(primary_pro)` still stayed unchanged at `1/52`, with only the old `human_win_pro_c` drift. Per the retained runbook, that meant the target surface had not moved, so the split was killed before `runtime-preflight` and `pro-reliability`.
- Durable lesson: even a real black one-move `action+mana` wrapper miss can be too local. Do not retain the isolated guard without a broader selector or duel story that also moves the cheap target surface.

## Apr 8, 2026: Shared Human Plus Fast-Duel Projection Clamp Killed Before Code Edits

- What was tried: added a shared diagnostic probe to compare the remaining `human_win_pro_c` drift against the sampled fast-duel black turn-four divergence before cutting more ProV2 code, looking specifically for one common `Safe*Progress -> ImmediateScore` projection seam.
- Why it stopped: the probe showed the seam was not actually shared. On `human_win_pro_c`, selected `l10,5;l9,6` won mainly on followup floor (`810871` vs `810407`) and its post-root projection became `SpiritImpact -> ImmediateScore`; on the fast duel board, injected `l1,6;l2,7` was instead a vulnerable `ManaTempo` root whose post-root projection became `SafeSupermanaProgress -> ImmediateScore` against another vulnerable `ManaTempo` current root.
- Durable lesson: do not bundle `human_win_pro_c` and the sampled fast black turn-four board into one shared projection clamp unless fresh duel evidence proves the same projection family is responsible.

## Apr 8, 2026: Broad White Turn-Three Mana-Only Reroute Killed At Triage

- What was tried: refreshed the live duel trace, then grouped three fresh white turn-three regressions under one wrapper family: `turn=3`, `mons_moves>0`, `action=false`, `mana=true` states that were still routing through the broad fallback instead of the challenger’s own pre-accept/current root. Rerouted that whole family back to the current Pro surface and added focused regression tests for the live Pro/Normal/Fast boards.
- Why it stopped: the replayed duel boards all matched current after the reroute and `guardrails` passed, but `pro-triage(primary_pro)` still stayed unchanged at the same stale `human_win_pro_c`-only `1/52`. Per the retained runbook, that meant the target surface still had not moved, so the split was killed before `runtime-preflight` and `pro-reliability`.
- Durable lesson: even a broader white turn-three wrapper family that fixes three live duel boards can still be too local. Do not retain it without a target-surface story that also moves `primary_pro`.

## Apr 8, 2026: White Turn-Three Accepted-Head Idea Killed Before Code Edits

- What was tried: after the broader white mana-only reroute died, checked the only remaining repeated live lead before touching production code: the white `turn=3`, `mons_moves=1`, `action+mana` fast-duel accepted-head seam `l9,4;l8,4` vs current `l8,7;l7,8`.
- Why it stopped: the runtime-faithful retained churn probe still showed `accepted=false` on every retained `primary_pro` fixture (`primary_spirit_setup`, `primary_pvs_sensitive_search`, `primary_black_reliability_opening_3_ply4`, `primary_white_harvest_loss_c_ply24`, and `human_win_pro_c`). That meant the repeated fast-duel seam still had no retained target-surface foothold, so per the runbook the idea was killed before code edits.
- Durable lesson: do not spend another production split on a repeated accepted-head duel seam unless the same acceptance family also appears on the retained `primary_pro` surface.

## Apr 8, 2026: Early Black Negative-Deny Spirit Merge Killed Before Code Edits

- What was tried: refreshed the full `smart_automove_pro_reliability_duel_trace_probe` corpus again, then drilled into the new early black normal-duel miss `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` to see whether it was the same family as the retained `human_win_pro_c` spirit-vs-progress drift.
- Why it stopped: the full replay still showed the same broad wall, and the only repeated live seam remained the white fast-duel accepted-head pair with no retained foothold. The focused selector probe on the black normal board showed `negative_deny_competes=true` and `followup_progress_competes=false`, while `human_win_pro_c` is the opposite shape: a followup-progress bias rather than a negative-deny one-off. That meant the black seam was both separate and still off the retained `primary_pro` surface, so the shared spirit-setup idea was killed before code edits.
- Durable lesson: do not merge an early black `negative_deny` spirit-preference drift with the retained human followup-progress seam unless fresh retained evidence proves they share the same selector gate.

## Apr 8, 2026: White Pro-Duel Wrapper Probe Killed Before Code Edits

- What was tried: after the black negative-deny merge died, drilled into the lone `vs current Pro` regression `l9,2;l8,3` vs current `l10,7;l9,7` to see whether it was a fresh Pro-only selector surface.
- Why it stopped: the board turned out to be another member of the already-closed white mid-turn wrapper family: `turn=3`, `mons_moves=4`, `action=false`, `mana=true`. The focused probe showed the configured `runtime_pro_turn_engine_v30` path itself still pre-accepted the current-style root `l10,7;l9,7`, but the outer `runtime_pro_turn_engine_v30_guarded_inputs(...)` wrapper rerouted the board through the broad fast pre-exact fallback and returned `l9,2;l8,3`. That meant the Pro regression was not a new in-path selector seam, so the idea was killed before code edits.
- Durable lesson: do not reopen a lone white Pro-duel regression if the probe shows configured v30 already matches current and only the outer white turn-three mana-only wrapper is still responsible.

## Apr 8, 2026: Later Black ManaTempo Shortlist Probe Killed Before Code Edits

- What was tried: drilled into the later black normal-duel drift `l2,4;l3,3` vs current `l3,6;l2,7` to see whether it was a new retained in-path selector seam.
- Why it stopped: the focused probe showed a one-off forced-shortlist distortion instead. `runtime_pro_turn_engine_v30` injected a terrible `ManaTempo` head `l3,6;l3,7` at the top of the forced root set, left that head unaccepted, and still pre-accepted `l2,4;l3,3` over current `l3,6;l2,7`. A fresh `smart_automove_pro_runtime_faithful_retained_churn_probe` confirmed the retained surface still had forced inputs only on the already-closed `primary_pvs_sensitive_search` fixture, so this later black seam had no retained `primary_pro` foothold.
- Durable lesson: do not reopen a later black `ManaTempo` shortlist repair without a matching retained forced-input seam on `primary_pro`; a one-off injected head with `accepted=false` is still just diagnostic noise.

## Apr 8, 2026: White Fast-Duel Opening-Family Probe Killed Before Code Edits

- What was tried: drilled into the white fast-duel one-off `l9,5;l8,5` vs current `l7,5;l6,4` to see whether it was a fresh opening-family selector seam.
- Why it stopped: the focused probe showed another outer-wrapper mismatch instead. The board was `turn=3`, `mons_moves=2`, `action=false`, `mana=true`; configured `runtime_pro_turn_engine_v30` still pre-accepted current `l7,5;l6,4`, while the live path returned `l9,5;l8,5`, which was not present in the configured root shortlist at all. That made it another member of the already-closed white turn-three mana-only wrapper family, so the idea died before code edits.
- Durable lesson: if a white fast-duel move is missing from the configured v30 shortlist, treat it as wrapper churn, not a new retained selector seam.

## Apr 8, 2026: White Normal-Duel Opening-Family Probe Killed Before Code Edits

- What was tried: drilled into the white normal-duel one-off `l10,4;l9,4` vs current `l8,6;l7,7` to see whether it was a fresh opening-family selector seam.
- Why it stopped: the focused probe showed another wrapper mismatch. The board was again `turn=3`, `mons_moves=2`, `action=false`, `mana=true`; configured `runtime_pro_turn_engine_v30` still pre-accepted current `l8,6;l7,7`, while the live path returned lower-ranked `l10,4;l9,4`. That made it another member of the already-closed white turn-three mana-only wrapper family, so the idea died before code edits.
- Durable lesson: if configured v30 still pre-accepts current on a white `turn=3`, mana-only board, treat the live mismatch as wrapper churn rather than a new retained selector seam.

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
