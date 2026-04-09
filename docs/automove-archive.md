# Automove Archive

This document keeps short history for retired automove waves.

Everything here is archive-only context. These IDs are not valid experiment targets. Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the live workflow and `docs/automove-knowledge.md` for durable lessons that still matter. Full branch-by-branch detail lives in git history rather than this file.

## Apr 9, 2026: White Fast V73 Rerank Classified, Fresh Seed v74 Killed

- What was tried: widened `smart_automove_pro_white_score_route_probe` with the repeated `v73` Fast white board `l10,4;l9,3` vs current `l9,5;l7,6;l8,7` to determine whether that repeat finally matched a retained white surface, then refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v74`.
- Why it stopped: the widened probe showed the repeated Fast white board is another distinct vulnerable runtime surface, not a retained white seam. Runtime-faithful v30 already has `selected=pre_accept=l10,4;l9,3`, `forced_inputs=None`, `stage=engine_post_search`, and `accepted=false`, with a top cluster of vulnerable `ManaTempo` siblings, while current only appears at rank `4` as own-setup `SpiritImpact` `l9,5;l7,6;l8,7`. The fresh `v74` replay also stayed fragmented: direct Pro finished `1` regression / `3` improvements / `8` flat, Normal `2` / `2` / `8`, and Fast `3` / `0` / `9`, with every exact move pair count `1`.
- Durable lesson: keep the widened white score-route probe, but do not spend code from `v74`; the `l10,4;l9,3` repeat only widened the white seam map, and the next fresh seed still split across unrelated white one-offs plus already-classified black bridge and `ManaTempo` churn.

## Apr 9, 2026: Retained Churn Probe Confirmed Split, Fresh Seed v73 Killed

- What was tried: ran `smart_automove_pro_triage_retained_churn_probe` to see whether the retained `primary_pro` churn had finally collapsed onto one shared selector/root-choice family before spending another duel seed, then refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v73`.
- Why it stopped: the retained churn probe still split into separate stories: three black seams were late accepted injected heads over current rank-0 mana roots, `primary_white_safe_progress_rerank_ply27` stayed a separate vulnerable accepted white rerank, and `human_win_pro_c` stayed separate safe-progress drift. The fresh duel replay also failed to line up behind one branch: direct Pro finished `2` regressions / `3` improvements / `7` flat, Normal `2` / `0` / `10`, and Fast `5` / `0` / `7`. The only repeated exact pair was Fast white `l10,4;l9,3` vs current `l9,5;l7,6;l8,7` twice, while every other exact pair stayed at count `1`.
- Durable lesson: do not spend code from `v73`; even when Fast finally repeats the familiar white `l10,4;l9,3` rerank, that is still weaker than a shared cross-bucket retained family, and the retained churn probe still shows no single selector family behind the live `primary_pro` wall.

## Apr 9, 2026: Fresh Seed v72 Replay Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v72` to see whether a cleaner Normal bucket would finally expose one exact family worth real code.
- Why it stopped: the replay stayed fully fragmented. Direct Pro finished `3` regressions / `4` improvements / `5` flat, Normal `1` / `1` / `10`, and Fast `5` / `2` / `5`, with every exact move pair count `1`. Direct Pro only mixed one-off white `l9,4;l8,5` vs current `l7,7;l6,8`, one-off black `l0,10;l1,9` vs `l0,6;l1,6`, and one-off black `l0,5;l1,6` vs `l1,5;l1,3;l2,3`. Normal only showed one-off white `l9,5;l8,4` vs `l9,6;l7,7;l7,8`. Fast then fractured across one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one-off black spirit rerank `l1,5;l1,7;l0,7` vs `l3,2;l4,1`, one-off black action+mana `l1,6;l2,7` vs `l2,3;l3,2`, one-off white `l10,4;l9,3` vs `l9,5;l7,6;l8,7`, and one-off white forced-prepass `l7,4;l8,5` vs `l7,4;l8,3`.
- Durable lesson: do not spend code from `v72`; a cleaner Normal bucket is still not enough when direct Pro and Fast split across already-classified bridge, action+mana, engine-disabled, and forced-prepass families.

## Apr 9, 2026: Fresh Seed v71 Replay Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v71` to see whether a quieter Normal/Fast sample would finally collapse onto one known exact family.
- Why it stopped: the replay stayed split across already-killed families. Direct Pro finished `2` regressions / `2` improvements / `8` flat, Normal `2` / `0` / `10`, and Fast `1` / `4` / `7`, with every exact move pair count `1`. Direct Pro only mixed the old black spirit sibling `l0,4;l1,3` vs current `l0,4;l1,4` and a white `engine_disabled` `ManaTempo` tie `l8,7;l8,8` vs `l9,6;l8,5`. Normal only replayed two distinct black variants on the old `l4,1;l5,0;mb` baseline, `l0,5;l1,4` and `l2,5;l0,5;l1,5`. Fast only replayed the old white forced-prepass shell `l8,4;l8,5` vs `l8,4;l9,3`.
- Durable lesson: do not spend code from `v71`; narrower losing buckets are still not enough when they still reduce to already-killed bridge, spirit-sibling, and forced-prepass families.

## Apr 9, 2026: Fresh Seed v70 Replay Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v70` after the `v69` black runtime classification, to see whether a cleaner direct-Pro sample would finally line up the other buckets behind one exact family.
- Why it stopped: the seed still fractured across old one-offs. Direct Pro finished `1` regression / `5` improvements / `6` flat, Normal `4` / `3` / `5`, and Fast `2` / `4` / `6`, with every exact move pair count `1`. Direct Pro only had a white `engine_disabled` rerank `l10,6;l9,5` vs current `l10,5;l9,4`. Normal mixed one-off white forced-prepass `l9,6;l8,5` vs `l9,6;l8,7`, one-off white accepted spirit `l8,5;l6,5;l6,4` vs `l7,6;l8,7`, one-off black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`, and one-off early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7`. Fast only added one-off black spirit sibling `l0,4;l1,3` vs `l0,4;l1,4` and one-off white forced-prepass `l9,6;l8,5` vs `l9,6;l8,7`.
- Durable lesson: do not spend code from `v70`; a cleaner direct-Pro bucket is still weaker than one repeated exact retained family when Normal and Fast stay spread across already-classified seams.

## Apr 9, 2026: Fresh Seed v69 Replay Killed After Later-Black Runtime Classification

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v69`, then widened `smart_automove_pro_black_forced_runtime_probe` with the traced Fast black board `l1,6;l2,7` vs current `l2,3;l3,4` because that exact pair finally appeared across buckets.
- Why it stopped: the replay still did not converge on one code-ready family. Direct Pro finished `2` regressions / `6` improvements / `4` flat, Normal `2` / `2` / `8`, and Fast `3` / `1` / `8`, with every exact move pair count `1`. Direct Pro only mixed early-black `negative_deny` `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` plus the old white spirit cluster `l4,9;l4,7;l5,7` vs `l9,4;l8,3`; Normal only replayed the two already-killed `l4,1;l5,0;mb` bridge variants `l0,5;l1,4` and `l1,5;l1,7;l0,7`; Fast only added one-off later-black `ManaTempo` `l1,6;l2,7` vs `l2,3;l3,4` plus one-off white `l9,4;l8,5` vs `l7,7;l6,8`. The widened black runtime probe showed the Fast `l1,6;l2,7` board is not the retained `primary_black_turn_four_action_mana_ply15` seam: runtime-faithful v30 still selects and heads on forced `l1,6;l2,7`, but `pre_accept` and baseline stay on current `l2,3;l3,4`, so it is another distinct current-baseline branch.
- Durable lesson: keep the widened black runtime probe, but do not spend code from `v69`; the current `l2,3;l3,4` baseline still branches beyond the retained later-black seam, and one cross-bucket replay is still weaker than one repeated exact retained family.

## Apr 9, 2026: Fresh Seed v68 Replay Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v68` after the black risky-window inject cut died, to see whether the clean challenger would finally expose a repeated cross-bucket seam strong enough for another production split.
- Why it stopped: the replay still did not converge on one code-ready family. Direct Pro finished `1` regression / `3` improvements / `8` flat, Normal `2` / `4` / `6`, and Fast `5` / `2` / `5`, with every exact move pair count `1`. The only cross-bucket neighborhood was the already-classified white forced-prepass shell: Normal surfaced `l9,4;l8,5` vs current `l9,4;l8,3` and Fast surfaced `l8,4;l8,5` vs current `l8,4;l8,3`, but both still live only in `search_only_forced_prepass` with `pre_accept` already on current. The remaining losses split across one-off later-black reranks, one-off black mana-bridge, one-off white safe-progress and `ManaTempo` ties, and one-off black `ManaTempo` tie.
- Durable lesson: do not spend code from `v68`; reviving the old white forced-prepass shell in two buckets is still weaker than one repeated exact retained family, and the rest of the seed remained fragmented.

## Apr 9, 2026: Black Turn-Four Bridge Fallback Killed After Full Loop

- What was tried: added a narrow `runtime_pro_turn_engine_v30` profile guard that routed only black `turn=4`, `mons_moves=2`, `action+mana` states back to current Pro when current already selected `l4,1;l5,0;mb`. The branch specifically targeted the retained bridge seams `primary_black_mana_bridge_ply20` and `primary_black_spirit_bridge_ply19`.
- Why it stopped: the branch was clean but still not promotable. It closed both retained bridge seams, passed the focused regression tests, `guardrails`, `pro-triage(primary_pro)=3/60` with `off_target_changed=0`, and `runtime-preflight`, but `pro-reliability` still failed at `0.8333` vs current Pro, `0.5000` vs current Normal, and `0.7500` vs current Fast.
- Durable lesson: do not reopen that bridge-family current fallback. Even collapsing both retained `l4,1;l5,0;mb` bridge seams to current is still too local to clear the real duel wall.

## Apr 8, 2026: Black Spirit-Bridge Injection Comparison Killed At Diagnostics

- What was tried: widened `smart_automove_pro_black_forced_root_probe` with retained `primary_black_spirit_bridge_ply19` to compare its raw, injected, and focused root ranks directly against the retained late-head and mana-bridge black seams.
- Why it stopped: the new spirit-bridge seam matched the retained mana-bridge seam at injection stage rather than the retained late-head seam. Both `primary_black_spirit_bridge_ply19` and `primary_black_mana_bridge_ply20` kept current `l4,1;l5,0;mb` at raw rank `0`, injected the forced bridge root only at rank `1`, and then promoted it to focused rank `0`. The late-head seam stayed different: `primary_black_late_accepted_head_ply4` injected `l1,5;l1,7;l0,7` directly to rank `0` and remained the rejected `ImmediateScore` head story.
- Durable lesson: keep the widened black forced-root probe, but do not reopen a shared black production branch from this alone. The `l4,1;l5,0;mb` current baseline now has a small bridge-injection family, but that still is not one promotable rule.

## Apr 8, 2026: Fresh Seed v16 Replay Retained Black Spirit-Bridge Foothold

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v16`, then widened the black forced-runtime comparison when the Normal black pair `l1,5;l1,7;l0,7` vs current `l4,1;l5,0;mb` repeated twice.
- Why it stopped: the seed still was not a production branch. Direct Pro finished `2` regressions / `4` improvements / `6` flat and Fast `3` / `2` / `7`, with every exact pair count `1`; only Normal repeated one exact pair twice. The widened runtime probe showed that repeated board is not the retained late-black accepted-head seam and not the retained mana-bridge seam. Runtime-faithful v30 already has `selected=pre_accept=head=l1,5;l1,7;l0,7`, `accepted=true`, `head_family=SpiritImpact`, and `goal_family=SpiritImpact`, while current stays on `l4,1;l5,0;mb`.
- Durable lesson: keep `primary_black_spirit_bridge_ply19` and the widened black forced-runtime probe, but do not reopen a shared black production branch from that repeat alone. The same current baseline `l4,1;l5,0;mb` now supports at least two distinct accepted black runtime stories.

## Apr 8, 2026: Traced Normal V15 White Score-Route Win-B Revival Killed At Diagnostics

- What was tried: widened `smart_automove_pro_white_score_route_probe` so it compares the fresh `v15` Normal white board `l10,5;l9,4` vs current `l4,9;l4,7;l5,7` against the retained `primary_harvest_white_score_route_win_b` fixture, which already carries the same forced head `l10,5;l9,4`.
- Why it stopped: the traced board was not the retained `win_b` harvest surface. Runtime-faithful v30 already had `selected=pre_accept=head=l10,5;l9,4`, `forced_inputs=Some("l10,5;l9,4")`, `stage=engine_post_search`, and `goal_family=DrainerSafetyRecovery`, while current stayed on `l4,9;l4,7;l5,7`. The retained `primary_harvest_white_score_route_win_b` fixture stayed different: `selected=baseline=l10,7;l9,8`, `pre_accept=l10,6;l9,5`, `head=l10,5;l9,4`, `stage=engine_disabled`, and `goal_family=ImmediateScore`.
- Durable lesson: keep the widened white score-route probe, but do not reopen a white harvest/score-route branch just because a fresh board reuses the same forced head. Runtime-faithful stage, selected root, and goal family still matter more than the injected move string.

## Apr 8, 2026: Traced Pro V15 White L8,4 Sibling Split Killed At Diagnostics

- What was tried: widened `smart_automove_pro_white_fast_forced_prepass_probe` so it compares the fresh `v15` direct-Pro white board `l8,4;l7,3` vs current `l8,4;l9,3` against the older traced forced-prepass boards and the retained `primary_white_fast_screen_opening_0_ply9` fixture.
- Why it stopped: the traced board was neither retained white family. Runtime-faithful v30 already had `selected=l8,4;l7,3`, `pre_accept=baseline=head=l8,4;l9,3`, `forced_inputs=Some("l8,4;l9,3")`, `stage=engine_disabled`, and all nearby `l8,4;*` roots stayed the same vulnerable `ManaTempo` family. That differs from the older forced-prepass shell, which used `stage=search_only_forced_prepass` and a safe `DrainerSafetyRecovery` selected root, and from the retained fast-screen fixture, which stayed `drainer_vulnerable=false` on spirit-progress roots with no `l8,4;*` sibling roots present at all.
- Durable lesson: keep the widened white fast forced-prepass probe, but do not reopen a white opening-family branch just because a fresh board lands in the same `l8,4` neighborhood. Runtime-faithful stage shape still matters more than the move neighborhood.

## Apr 8, 2026: Fresh Seed v15 Replay Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v15` after the `v14` direct-Pro sibling board died against the retained opening surface, to see whether the next replay would finally repeat one exact retained family strongly enough for a real branch.
- Why it stopped: the replay mixed retained seams with nearby sibling reranks, but every exact pair still stayed count `1`. `vs current Pro` finished `3` regressions / `3` improvements / `6` flat, `vs current Normal` `3` / `2` / `7`, and `vs current Fast` `2` / `2` / `8`. Direct Pro replayed the retained mana-bridge seam `l0,5;l1,4` vs `l4,1;l5,0;mb` plus sibling ties `l0,4;l1,3` vs `l0,4;l1,5` and `l8,4;l7,3` vs `l8,4;l9,3`; Normal added later-black accepted-head `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, black spirit rerank `l1,5;l2,3;l1,4` vs `l1,5;l2,3;l1,2`, and white safe-progress rerank `l10,5;l9,4` vs `l4,9;l4,7;l5,7`; Fast added only white `l10,4;l9,3` vs `l9,5;l7,6;l8,7` and black `l2,7;l2,8` vs `l2,7;l1,8`.
- Durable lesson: even a replay that mixes real retained seams with nearby sibling ties is still only diagnostics when every exact pair stays count `1`. Do not cut code from that blend; wait for one exact family to repeat strongly enough to justify a real branch.

## Apr 8, 2026: Traced Pro V14 Black Spirit Sibling Killed At Diagnostics

- What was tried: widened `smart_automove_pro_black_spirit_sibling_probe` so it compares the fresh `v14` direct-Pro black board `l0,4;l1,4` vs current `l0,4;l1,5` against the older traced `v12` sibling board and the retained early-black opening fixtures, including `primary_black_loss_opening_b_black_turn`.
- Why it stopped: the traced `v14` board was not the retained opening-`b` family. Runtime-faithful v30 already had `selected=pre_accept=l0,4;l1,4`, `forced_inputs=None`, `stage=engine_post_search`, `accepted=false`, and a rejected head `l0,5;l1,6` under `goal_family=DrainerSafetyRecovery`, while the retained `primary_black_loss_opening_b_black_turn` surface stayed `engine_disabled` on current `l0,4;l1,5` with no head at all. The other retained opening fixtures also remained different: some stayed fully unforced on current, while others still carried forced `SafeSupermanaProgress` heads like `l0,5;l1,6`.
- Durable lesson: keep the widened sibling probe, but do not reopen a shared early-black opening branch just because a fresh direct-Pro board lands on a nearby spirit sibling like `l0,4;l1,4`. Runtime-faithful stage shape still matters more than the baseline cluster or the neighboring move string.

## Apr 8, 2026: Fresh Seed v14 Replay Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v14` after classifying the `v13` Normal black drainer-safety board, to see whether the cleaner direct-Pro bucket would finally line up with a repeated Normal or Fast exact family.
- Why it stopped: the replay stayed cleaner, but still not actionable. `vs current Pro` finished `1` regression / `4` improvements / `7` flat, `vs current Normal` `2` / `0` / `10`, and `vs current Fast` `2` / `5` / `5`, with every exact move pair staying count `1`. Direct Pro had only black spirit-sibling `l0,4;l1,4` vs current `l0,4;l1,5`; Normal had later-black accepted-head `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb` plus accepted black spirit rerank `l1,4;l0,6;l1,6` vs `l1,4;l3,4;l3,3`; Fast had only the white forced-prepass drift `l9,6;l8,5` vs `l9,6;l8,7` and the retained black mana-bridge seam `l0,5;l1,4` vs `l4,1;l5,0;mb`.
- Durable lesson: a cleaner direct-Pro bucket still is not enough when Normal and Fast remain pure count-`1` churn. Keep only the replay note and wait for a seed that repeats one exact family strongly enough to justify code.

## Apr 8, 2026: Traced Normal V13 Black Drainer-Safety Rerank Killed At Diagnostics

- What was tried: widened `smart_automove_pro_black_forced_runtime_probe` so it compares the fresh `v13` Normal black board `l1,6;l1,5` vs current `l3,2;l4,1` against the retained black seams plus the earlier traced black reranks on that same baseline.
- Why it stopped: the traced board was another distinct black runtime surface, not a retained family. Runtime-faithful v30 already had `selected=pre_accept=head=l1,6;l1,5`, `accepted=true`, `stage=engine_post_search`, and both `head_family` and `goal_family` equal to `DrainerSafetyRecovery`, while current stayed on vulnerable `ManaTempo` `l3,2;l4,1`. That differs from the retained later-black rejected-head surface, the retained mana-bridge seam, the traced fast `v10` accepted `ManaTempo` rerank, and the traced `v12` accepted spirit-head board.
- Durable lesson: keep the widened runtime probe, but do not reopen a shared black branch just because a fresh board reuses current `l3,2;l4,1`. That baseline now supports multiple distinct accepted and rejected black stories, so runtime-faithful stage and family matching still matter more than the current move string.

## Apr 8, 2026: Fresh Seed v13 Replay Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v13` after the `v12` Pro/Normal one-offs died at diagnostics, to see whether the next duel sample would finally concentrate the wall onto one retained black family strongly enough for a real production retry.
- Why it stopped: the replay briefly resurfaced several retained black seams, but still not in a code-ready way. `vs current Pro` finished `3` regressions / `4` improvements / `5` flat, `vs current Normal` `1` / `2` / `9`, and `vs current Fast` `3` / `2` / `7`, with every exact move pair staying count `1`. Direct Pro replayed the retained mana-bridge seam `l0,5;l1,4` vs current `l4,1;l5,0;mb` and the retained early-black `negative_deny` seam `l0,5;l1,6` vs current `l1,5;l3,6;l2,7`, Fast replayed the retained black turn-four action+mana seam `l1,6;l2,7` vs current `l3,2;l4,1`, and Normal added only another one-off accepted black drainer-safety rerank `l1,6;l1,5` vs `l3,2;l4,1`.
- Durable lesson: even a replay that brings back multiple retained black seams is still only diagnostics if each exact pair stays count `1`. Do not reopen a shared black branch from that shape; exact-pair repeat count still matters more than the broad family label.

## Apr 8, 2026: Traced Normal V12 Black Mana Rerank Killed At Diagnostics

- What was tried: widened `smart_automove_pro_black_forced_runtime_probe` so it compares the fresh `v12` Normal black board `l1,5;l2,5` vs current `l1,6;l0,6` against the retained black forced-engine seams and the traced fast `v10` black mana rerank.
- Why it stopped: the traced board was another accepted non-vulnerable black `ManaTempo` rerank, but not a retained family. Runtime-faithful v30 kept `pre_accept` on current `l1,6;l0,6`, then accepted `l1,5;l2,5` under `head_family=ManaTempo`, `goal_family=DrainerSafetyRecovery`, and `stage=engine_cached_resume`. That differs from the retained mana-bridge seam, which uses a different current baseline and `goal_family=SpiritImpact`, and from the traced fast `v10` rerank, which also used a different current baseline and stage.
- Durable lesson: keep the widened runtime probe, but do not reopen a shared black branch just because a fresh board is another accepted non-vulnerable `ManaTempo` rerank. Goal family, baseline move, and late selector stage still matter.

## Apr 8, 2026: Traced Normal V12 White Safe-Progress Rerank Killed At Diagnostics

- What was tried: added `smart_automove_pro_white_safe_progress_probe` to compare the fresh `v12` Normal board `l9,5;l8,5` vs current `l10,7;l9,8` against the retained `primary_white_safe_progress_rerank_ply27` and `primary_white_fast_screen_opening_0_ply9` white surfaces.
- Why it stopped: the traced board was not either retained white family. Runtime-faithful v30 already had `selected=pre_accept=head=l9,5;l8,5`, `accepted=true`, `head_family=SafeSupermanaProgress`, and `goal_family=DrainerSafetyRecovery`, with a non-vulnerable safe-progress root over current non-vulnerable `ManaTempo` `l10,7;l9,8`. The retained `primary_white_safe_progress_rerank_ply27` seam remains an accepted vulnerable `ManaTempo` rerank under `goal_family=ImmediateScore`, while `primary_white_fast_screen_opening_0_ply9` keeps `pre_accept` and baseline on spirit-progress lines and leaves the same `l9,5;l8,5` head rejected.
- Durable lesson: keep the new white safe-progress probe, but do not reopen a white production branch just because `l9,5;l8,5` appears again on a non-vulnerable board. Runtime-faithful stage shape and goal family still matter more than the move string.

## Apr 8, 2026: Traced Pro V12 Black Spirit Sibling Killed At Diagnostics

- What was tried: added `smart_automove_pro_black_spirit_sibling_probe` to compare the fresh `v12` direct-Pro board `l0,4;l1,3` vs current `l0,4;l1,5` against existing early black opening fixtures that already anchor on current `l0,4;l1,5`.
- Why it stopped: the traced board was not the existing opening family. On the traced board, `runtime_pro_turn_engine_v30` had `forced_inputs=None`, `stage=engine_post_search`, `accepted=false`, and `pre_accept` already on `l0,4;l1,3`, while the opening fixtures either stayed fully unforced on current `l0,4;l1,5` or carried forced `SafeSupermanaProgress` heads like `l0,5;l1,6` or `l0,5;l1,5`. Sharing the same current move string did not produce a shared runtime-faithful surface.
- Durable lesson: keep the new probe, but do not reopen a shared early-black opening branch just because a fresh board reuses current `l0,4;l1,5`. Runtime-faithful stage shape and forced-input story still matter more than the baseline move string.

## Apr 8, 2026: Traced Pro V12 Black Spirit Head Killed At Diagnostics

- What was tried: widened `smart_automove_pro_black_forced_runtime_probe` so it compares the retained black seams against the fresh `v12` direct-Pro board `l2,5;l0,5;l1,6` vs current `l3,2;l4,1`.
- Why it stopped: the traced board was not any retained black seam. It was not the retained later-black accepted-head family, because runtime-faithful v30 already had `selected=pre_accept=head=l2,5;l0,5;l1,6`, `accepted=true`, and a vulnerable `SpiritImpact -> ImmediateScore` plan, while the retained `primary_black_late_accepted_head_ply4` surface still keeps `pre_accept` on current `l3,2;l4,1` and rejects `l1,5;l1,7;l0,7`. It was not the retained mana-bridge family either: both differ from current `l3,2;l4,1`, but the retained bridge is an accepted non-vulnerable `ManaTempo` root under `head_family=SafeSupermanaProgress`, not an accepted vulnerable spirit head.
- Durable lesson: keep the widened runtime probe, but do not reopen a shared black branch just because a fresh board reuses current `l3,2;l4,1`. That baseline now supports multiple distinct black live families, and runtime-faithful stage shape still matters more than the baseline move string.

## Apr 8, 2026: Fresh Seed v12 Replay Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v12` after the `v11` Normal repeat died off the Pro surface, to see whether the next duel sample would finally produce a repeatable retained family or at least collapse the wall into one exact code target.
- Why it stopped: the replay got cleaner, but not actionable. `vs current Pro` finished `2` regressions / `2` improvements / `8` flat, `vs current Normal` `2` / `0` / `10`, and `vs current Fast` `0` / `4` / `8`, with no repeated move pair anywhere. The only regressions were one-off boards: direct-Pro black spirit reranks `l0,4;l1,3` vs current `l0,4;l1,5` and `l2,5;l0,5;l1,6` vs `l3,2;l4,1`, a Normal black `ManaTempo` rerank `l1,5;l2,5` vs `l1,6;l0,6`, and a Normal white non-vulnerable safe-progress rerank `l9,5;l8,5` vs `l10,7;l9,8`.
- Durable lesson: even a replay that goes clean in Fast is still only diagnostics if Pro and Normal remain count-`1` churn. Do not retain new fixtures or cut code from that shape until one exact family repeats strongly enough to survive on the retained Pro surface.

## Apr 8, 2026: Fresh Seed v11 Normal Repeat Killed At Diagnostics

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v11`, then briefly widened `smart_automove_pro_white_score_route_probe` around the only exact repeated pair from that replay, the Normal white move `l9,5;l7,4;l8,3` vs current `l9,5;l7,6;l7,7`.
- Why it stopped: the replay still was not code-ready. `vs current Pro` finished `3` regressions / `3` improvements / `6` flat, `vs current Normal` `6` / `0` / `6`, and `vs current Fast` `4` / `4` / `4`; only the Normal white pair repeated, and even that died immediately when checked on the retained Pro surface. On the traced board in Pro mode, `runtime_pro_turn_engine_v30` and current both selected `l10,4;l9,4`, `forced_inputs` stayed `None`, `accepted=false`, and the repeated `l9,5;...` targets were absent from the Pro shortlist.
- Durable lesson: a repeated Normal duel pair still is not enough to retain as `primary_pro` unless the same board reproduces as an actual Pro-mode seam. If the traced board collapses to current and the repeated targets vanish from the Pro shortlist, kill the branch at diagnostics and keep only the note.

## Apr 8, 2026: Retain White Safe-Progress Rerank Seam

- What was tried: widened `smart_automove_pro_white_score_route_probe` with the fresh `v10` Normal white board `l9,4;l8,3` vs current `l5,2;l4,1`, then retained that exact board as `primary_white_safe_progress_rerank_ply27` and added it to the retained churn probes to see whether it moved the clean cheap surface.
- Why it stopped: this turn did not justify production code, but it did produce a real new white foothold. The widened probe showed the board is not the retained harvest white score-route family: `runtime_pro_turn_engine_v30` already has `selected=pre_accept=head=l9,4;l8,3`, `accepted=true`, `forced_inputs=Some("l9,4;l8,3")`, `drainer_vulnerable=false`, and a vulnerable `ManaTempo` root, while current stays on a non-vulnerable `SafeSupermanaProgress` baseline `l5,2;l4,1`. Adding the fixture moved `pro-triage(primary_pro)` to `4/59`, and only `primary_white_safe_progress_rerank_ply27`, `primary_black_turn_four_action_mana_ply15`, `primary_black_mana_bridge_ply20`, and `human_win_pro_c` changed, with `off_target_changed=0`.
- Durable lesson: keep `primary_white_safe_progress_rerank_ply27` as a separate retained white accepted rerank. Do not merge it into the harvest-family white score-route story from this evidence alone; the stage shape, safety story, and selected/root family all differ.

## Apr 8, 2026: Traced Fast V10 Black Rerank Killed At Diagnostics

- What was tried: widened `smart_automove_pro_black_forced_runtime_probe` so it compares the retained black seams against the fresh `v10` Fast black board `l1,5;l1,4` vs current `l3,2;l4,1`.
- Why it stopped: the traced board was not any retained black seam. It was not the retained later-black accepted-head family, because `pre_accept` already stayed on current `l3,2;l4,1` and the accepted head was a different non-vulnerable `ManaTempo` rerank `l1,5;l1,4` under `goal_family=SpiritImpact`, not the retained rejected `SpiritImpact -> ImmediateScore` head `l1,5;l1,7;l0,7`. It was not the retained mana-bridge family either: both boards ended as accepted non-vulnerable `ManaTempo` reranks, but the traced board used a different forced head and current-shaped five-step root instead of the retained `l0,5;l1,4` bridge.
- Durable lesson: keep the widened runtime probe, but do not reopen a shared black branch just because a fresh board reuses the same current baseline `l3,2;l4,1` or lands in the same broad black `ManaTempo` bucket. Runtime-faithful stage shape still has to match the retained seam, not just the baseline move.

## Apr 8, 2026: Fresh Seed v10 Duel Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v10` after the retained black forced-runtime comparison died at diagnostics, to see whether the next duel sample would finally repeat a retained family strongly enough to justify new code.
- Why it stopped: the replay resurfaced several familiar stories, but still not in a code-ready way. `vs current Pro` finished `2` regressions / `4` improvements / `6` flat, `vs current Normal` `4` / `0` / `8`, and `vs current Fast` `4` / `0` / `8`, with every exact move pair staying count `1`. The sample brought back the retained early-black `negative_deny` move pair `l0,5;l1,6` vs current `l1,5;l3,6;l2,7`, the retained later-black accepted-head pair `l1,5;l1,7;l0,7` vs current `l4,1;l5,0;mb`, and the known white forced-prepass family around `l9,4;l8,5` or `l8,4;l8,5`, but each bucket still fragmented into separate one-offs with different exact baselines.
- Durable lesson: do not treat “the same family is back in multiple duel buckets” as enough on its own. If the exact move pairs still do not repeat, keep only the replay note and wait for a seed that repeats on one retained surface cleanly enough to justify a real branch.

## Apr 8, 2026: Black Forced-Runtime Comparison Killed At Diagnostics

- What was tried: added `smart_automove_pro_black_forced_runtime_probe` so the two retained black forced-engine footholds, `primary_black_turn_four_action_mana_ply15` and `primary_black_mana_bridge_ply20`, could be compared at runtime-faithful selection stage instead of only at raw, injected, and focused root ranks.
- Why it stopped: the probe showed the two seams are closer late than early, but still not in a code-ready way. In both cases `runtime_pro_turn_engine_v30` reaches `stage=engine_post_search`, accepts the head, and collapses `selected`, `pre_accept`, and `head` onto the same forced `ManaTempo` root under a `SafeSupermanaProgress` head. But the remaining differences are exactly the kind of broad preference split that is not safe to codify: `primary_black_turn_four_action_mana_ply15` stays a vulnerable `SafeSupermanaProgress -> ImmediateScore` drift, while `primary_black_mana_bridge_ply20` is a non-vulnerable `SafeSupermanaProgress -> SpiritImpact` rerank, and on both boards the selected root utility already narrowly beats current.
- Durable lesson: keep the runtime-faithful comparison probe, but do not reopen a shared black current-forcing repair from these two seams alone. If both retained boards already converge on accepted `ManaTempo` roots and differ mainly by narrow utility and risk shape, the next production branch still needs a stronger duel-backed explanation than “pick current instead.”

## Apr 8, 2026: Retain White Turn-Three Full-Resources Fallback

- What was tried: traced the retained `primary_white_mana_sibling_ply9` seam back to a white `turn=3`, `mons_moves=3`, `action+mana` wrapper miss. Guarded `runtime_pro_turn_engine_v30` was falling through to `runtime_release_safe_pre_exact` and selecting `l5,0;l5,1`, while current Pro still chose `l5,0;l4,1`. Kept a narrow production repair by lowering the existing white turn-three full-resources current-Pro fallback from `mons_moves>=5` to `mons_moves>=3`, and added `runtime_pro_turn_engine_v30_profile_prefers_current_white_turn_three_full_resources_root`.
- Why it stopped: the seam itself closed cleanly, `guardrails` passed, `pro-triage(primary_pro)` returned to the familiar `human_win_pro_c`-only `1/56`, `runtime-preflight` passed, and `pro-reliability` still failed at `0.8333` vs current Pro, `0.5000` vs current Normal, and `0.7500` vs current Fast. The default duel trace did not expose a new repeated family; it stayed on the same one-off Pro/Normal/Fast wall as before.
- Durable lesson: a retained white mana-sibling seam can still be only another white turn-three full-resource wrapper miss. Closing it is safe enough to keep when it cleanly collapses the retained fixture with no churn, but it does not justify more wrapper-only spend if the default live wall remains unchanged one-offs.

## Apr 8, 2026: Retain Black Mana-Bridge Seam

- What was tried: after the `v7`, `v8`, and `v9` replays kept reusing the black move pair `l0,5;l1,4` vs current `l4,1;l5,0;mb`, added `primary_black_mana_bridge_ply20` and widened `smart_automove_pro_black_forced_root_probe` so it compares the new retained board against the existing traced fast `v7` board and the two older retained black seams.
- Why it stopped: this turn still did not justify production code, but it did produce a real retained foothold. The widened probe showed the new retained board matches the old traced fast board exactly at raw, injected, and focused root stages: forced root absent from raw roots, injected to rank `1`, promoted to focused rank `0`, `head_family=SafeSupermanaProgress`, and `goal_family=SpiritImpact`. The cheap gate moved to `3/58` with only `primary_black_turn_four_action_mana_ply15`, `primary_black_mana_bridge_ply20`, and `human_win_pro_c` changed, with `off_target_changed=0`.
- Durable lesson: keep `primary_black_mana_bridge_ply20` as the second retained foothold for the later black forced-engine family. It is not enough to cut production code alone, but it gives the next branch a real second black surface to compare against `primary_black_turn_four_action_mana_ply15`.

## Apr 8, 2026: Fresh Seed v8 Duel Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v8` after the shared black injected-root block was killed, to see whether a fresh duel sample would finally repeat one seam on a retained surface.
- Why it stopped: the replay was still all count-`1` churn. `vs current Pro` finished `2` regressions / `4` improvements / `6` flat, `vs current Normal` `2` / `1` / `9`, and `vs current Fast` `2` / `2` / `8`, with every move pair unique inside its duel bucket. The traces were interesting but still too sparse: a Pro black top-ranked `SafeSupermanaProgress` root `l1,4;l2,5` vs current `l2,3;l3,4`, a Pro white mana rerank `l8,7;l8,8` vs `l9,6;l8,5`, a Normal white spirit-own-setup rerank `l3,8;l1,7;l2,8` vs `l3,8;l3,6;l4,7`, a Normal black `l0,5;l1,4` vs `l4,1;l5,0;mb`, and two Fast black spirit-family misses `l1,5;l1,7;l0,7` vs `l5,0;l4,1` and `l1,5;l3,3;l2,4` vs `l1,5;l2,7;l1,8`.
- Durable lesson: even a fresh replay that surfaces a top-ranked safe-progress candidate or multiple black spirit-family drifts is still a diagnostics-only kill if every move pair stays count `1`. Wait for repetition on a retained surface before cutting code.

## Apr 8, 2026: Fresh Seed v9 Duel Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v9` to see whether a cleaner live sample could finally narrow the wall enough to justify a retained seam or production split.
- Why it stopped: the seed was cleaner but still not actionable. `vs current Pro` finished `0` regressions / `4` improvements / `8` flat, `vs current Normal` `1` / `0` / `11`, and `vs current Fast` `6` / `0` / `6`, with no repeated move pair anywhere. Normal had only the one-off black `l0,5;l1,4` vs `l4,1;l5,0;mb` seam, but Fast fragmented across six different misses: black spirit heads `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb` and `l1,5;l1,7;l2,6` vs `l5,0;l4,1`, black `ManaTempo` siblings `l0,4;l1,3` vs `l0,4;l1,4` and `l2,7;l2,8` vs `l2,7;l1,8`, plus white `l9,6;l10,4;l9,4` vs `l9,6;l7,6;l7,7` and white forced-prepass `l9,6;l8,5` vs `l9,6;l8,7`.
- Durable lesson: even a strong seed with a clean direct-Pro bucket is still a diagnostics-only kill if the remaining wall breaks only into unrelated Fast one-offs. Treat that as evidence that the wall is narrowing, not as permission to invent a code branch without one repeated retained family.

## Apr 8, 2026: Shared Black Injected-Root Block Killed After Reliability

- What was tried: widened `smart_automove_pro_black_forced_root_probe` so it compares three boards at raw, injected, and focused root stages: the retained `primary_black_turn_four_action_mana_ply15` seam, the retained `primary_black_late_accepted_head_ply4` seam, and the traced fast `v7` black replay `l0,5;l1,4` vs current `l4,1;l5,0;mb`. Then tried one shared production cut on `runtime_pro_turn_engine_v30`: reject black macro progress-head injections when the forced root is absent from raw roots and still has no progress, tactical, safety, or spirit surface.
- Why it stopped: the focused cut was real but still too local. It collapsed the retained black action+mana seam, also removed the traced fast `v7` forced root, passed `guardrails`, returned `pro-triage(primary_pro)` to `1/57` with only `human_win_pro_c`, and passed `runtime-preflight`, but `pro-reliability` still failed at `0.5833` vs current Pro, `0.4167` vs current Normal, and `0.6667` vs current Fast. The widened probe also showed the traced fast board was only adjacent to the retained action+mana family, not identical to either retained black seam: unlike the retained action+mana board it injected only to rank `1` before focused rank `0`, and unlike the retained late-black family it still stayed under a `SafeSupermanaProgress` head rather than a true spirit-progress root.
- Durable lesson: keep the widened black forced-root comparison probe, but do not reopen a shared black injected-root block from these boards alone. Similar outer injection shape is not enough when the injected-root semantics and duel outcomes still diverge.

## Apr 8, 2026: Retain Black Turn-Four Action+Mana Seam

- What was tried: retained the later black duel seam `l1,6;l2,7` vs current `l2,3;l3,2` as `primary_black_turn_four_action_mana_ply15`, added `smart_automove_pro_black_turn_four_action_mana_probe`, and briefly retried the narrow current-Pro wrapper reroute for black `turn=4`, `mons_moves=1`, `action+mana` boards. That wrapper fix closed the new retained seam, passed `guardrails`, returned `pro-triage(primary_pro)` to `1/57` with only `human_win_pro_c`, and passed `runtime-preflight`.
- Why it stopped: the wrapper line still failed `pro-reliability` at `0.7500` vs current Pro, `0.5000` vs current Normal, and `0.5833` vs current Fast, so the production guard was reverted. The widened retained probe then showed why a follow-up accept clamp would also be the wrong lever: raw ranked roots and current still prefer `l2,3;l3,2`, but runtime-faithful v30 injects `l1,6;l2,7` and already collapses selected, `pre_accept`, and head onto that same vulnerable `ManaTempo` root under a `SafeSupermanaProgress -> ImmediateScore` plan.
- Durable lesson: keep the retained fixture and widened probe, but do not reopen either a wrapper-only reroute or an acceptance-only clamp from this seam alone. The live behavior is earlier forced-engine injection, and the isolated black repair still is not enough to move the direct duel wall.

## Apr 8, 2026: Fresh Seed v7 Duel Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v7` after retaining the black `primary_black_turn_four_action_mana_ply15` foothold, to check whether the live wall had converged onto a repeated family worth code.
- Why it stopped: the replay was still all count-`1` churn. `vs current Pro` finished `1` regression / `4` improvements / `7` flat, `vs current Normal` `3` / `1` / `8`, and `vs current Fast` `2` / `1` / `9`, with every move pair unique inside its duel bucket. The traces included a direct-Pro white vulnerable `ManaTempo` rerank `l10,0;l9,1` vs current `l9,2;l9,1`, two different white Normal reranks, one later-black accepted-head resurfacing `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one Fast black `l0,5;l1,4` vs `l4,1;l5,0;mb`, and one Fast white `l10,4;l9,3` vs `l9,5;l7,6;l8,7`, but none repeated strongly enough to justify a retained fixture or production split.
- Durable lesson: even after a new retained black foothold lands, a fresh replay that only remixes one-off white and black seams is still a diagnostics-only kill. Do not reopen code until a replay repeats one family on a retained surface.

## Apr 8, 2026: Black Forced-Root Comparison Killed At Diagnostics

- What was tried: added `smart_automove_pro_black_forced_root_probe` to compare the retained `primary_black_turn_four_action_mana_ply15` seam against the retained `primary_black_late_accepted_head_ply4` seam at raw, injected, and focused root stages.
- Why it stopped: the probe showed only a superficial match. In both cases the forced root was absent from raw roots and got injected directly to rank `0`, demoting the current raw top to rank `1`. But the injected roots themselves diverged sharply: the action+mana seam injects a vulnerable non-spirit `ManaTempo` root `l1,6;l2,7` even though the head family is `SafeSupermanaProgress`, while the later black family injects a genuine `SpiritImpact` progress root `l1,5;l1,7;l0,7`.
- Durable lesson: do not reopen a shared black injection-family production split from these two seams alone. The useful retained artifact is the comparison probe; the code path still branches earlier than a single shared injected-root rule.

## Apr 8, 2026: Retain White Mana-Sibling Seam

- What was tried: after the `v6` replay left only one repeated multi-bucket family, added `primary_white_mana_sibling_ply9` and `smart_automove_pro_white_mana_sibling_probe` to compare the direct-Pro `l5,0;l5,1` vs current `l5,0;l4,1` board against the sibling Normal `l5,0;l6,1` vs `l5,0;l4,1` board.
- Why it stopped: the seam was real enough to retain but still not ready for production code. Both boards were `engine_disabled`, `drainer_vulnerable=false`, and stayed entirely inside the same non-spirit `ManaTempo` family. Current and `pre_accept` already matched `l5,0;l4,1`, while the challenger picked a lower-ranked sibling (`l5,0;l5,1` or `l5,0;l6,1`). The cheap gate moved to `2/56` changed primary-Pro fixtures with only this new seam plus `human_win_pro_c`, but there was still no bounded shared fix to try safely.
- Durable lesson: repeated sibling reranks can be worth retaining as fixtures even before there is a production hypothesis. Preserve the foothold and wait for a clear selector rule, rather than inventing a risky tie-break change.

## Apr 8, 2026: White V6 Spirit-Rerank Comparison Killed At Diagnostics

- What was tried: widened `smart_automove_pro_white_score_route_probe` so it compares the fresh `pro_turn_planner_reliability_v6` direct-Pro seam `l9,6;l8,4;l7,4` vs current `l9,6;l7,4;l7,3` against the retained harvest fixture `primary_harvest_white_score_route_win_a`.
- Why it stopped: the traced `v6` board was still not the retained harvest surface. On the fresh board, `runtime_pro_turn_engine_v30` accepted and selected the spirit head `l9,6;l8,4;l7,4`, `pre_accept` already sat on the nearby spirit sibling `l9,6;l7,4;l8,4`, and shipping differed only by another nearby vulnerable spirit-own-setup root `l9,6;l7,4;l7,3`. On the retained harvest fixture, current, candidate, and `pre_accept` still matched on `l9,6;l7,4;l8,3`, while the injected head `l9,6;l7,4;l6,3` remained rejected.
- Durable lesson: do not treat another vulnerable white `l9,6;...` spirit rerank as proof that the retained harvest score-route seam is live again. If the fresh board is an accepted spirit-sibling reshuffle while the retained fixture is still a rejected-head cluster, kill the production idea and keep only the widened probe.

## Apr 8, 2026: Fresh Seed v6 Duel Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v6` after the `v5` replay seams were fully classified and killed.
- Why it stopped: the replay still produced only count-`1` seams. `vs current Pro` finished `3` regressions / `4` improvements / `5` flat, `vs current Normal` `1` / `0` / `11`, and `vs current Fast` `2` / `4` / `6`, with no repeated move pair in any duel bucket. The recorded regressions were all already-classified live-only shapes: a white forced-prepass drainer-safety board `l8,4;l8,5` vs current `l8,4;l9,3`, white `ManaTempo` sibling reranks around `l5,0;*` and `l8,3;*`, one white accepted spirit rerank `l9,6;l8,4;l7,4` vs current `l9,6;l7,4;l7,3`, the non-retained late-black `l1,5;l1,7;l0,7` spirit-head case, and one black mana rerank `l1,5;l2,5` vs current `l1,6;l0,6`.
- Durable lesson: another fresh replay that only remixes already-classified one-off seam shapes is still not a reason to spend code. Wait for a replay that repeats one family on a retained surface, not for a broader list of count-`1` misses.

## Apr 8, 2026: White Mana-Sibling Tie Killed At Diagnostics

- What was tried: used a temporary traced-board probe on the fresh `pro_turn_planner_reliability_v5` direct-Pro seam `l8,3;l8,2` vs current `l8,3;l9,2`, then removed the probe after classification.
- Why it stopped: the board had no retained fixture footprint and turned out to be a live-only `ManaTempo` sibling tie. `stage=engine_disabled`, `forced_inputs` and the traced head already matched current `l8,3;l9,2`, `accepted=true`, and both candidate and baseline roots had identical utility and identical vulnerability; the challenger differed only by picking the equal-utility sibling `l8,3;l8,2`.
- Durable lesson: do not spend a production split on a fresh white sibling rerank when the traced board is only an equal-utility `ManaTempo` tie and there is no retained surface. Keep the lesson, not the probe.

## Apr 8, 2026: Late-Black Accepted-Head Comparison Killed At Diagnostics

- What was tried: widened `smart_automove_pro_black_late_accepted_head_probe` so it compares the retained `primary_black_late_accepted_head_ply4` board against the fresh `pro_turn_planner_reliability_v5` Normal drift `l1,5;l1,7;l0,7` vs current `l4,1;l5,0;mb`.
- Why it stopped: the traced board was not the retained late-black family. On the fresh board, `pre_accept` already chose `l1,5;l1,7;l0,7`, the head was accepted, and the plan stayed in `goal_family=SpiritImpact`, with shipping differing only because it still selected the weaker `ManaTempo` sibling `l4,1;l5,0;mb`. On the retained fixture, current and `pre_accept` still chose `l3,2;l4,1`, while the injected `l1,5;l1,7;l0,7` head remained rejected as a `SpiritImpact -> ImmediateScore` override.
- Durable lesson: do not treat a repeated head move string as proof that the retained late-black accepted-head seam is live again. If the fresh board already selects that spirit head at `pre_accept`, it is a different story than the retained rejected-head family and does not justify reopening the production guard.

## Apr 8, 2026: White Score-Route Comparison Killed At Diagnostics

- What was tried: added `smart_automove_pro_white_score_route_probe` to compare the fresh `pro_turn_planner_reliability_v5` direct-Pro seam `l7,4;l8,3` vs current `l9,7;l7,6;l8,7` against the retained harvest fixture `primary_harvest_white_score_route_win_a`.
- Why it stopped: the traced board was not the retained harvest surface. On the traced board, `runtime_pro_turn_engine_v30` accepted a forced `SafeSupermanaProgress -> ImmediateScore` head and returned the plain `ManaTempo` root `l7,4;l8,3`, while current and `pre_accept` both stayed on the same vulnerable spirit-own-setup route. On the retained harvest fixture, current, candidate, and `pre_accept` already matched on `l9,6;l7,4;l8,3`, the injected head was rejected, and the plain `l7,4;l8,3` mana route sat only as a lower-ranked sibling.
- Durable lesson: do not treat a fresh white `l7,4;l8,3` replay as evidence that the retained harvest score-route surface is live again. If the traced board is an accepted progress-head override while the retained fixture is an unaccepted spirit cluster, kill the production idea and keep only the probe.

## Apr 8, 2026: White Forced-Prepass Comparison Killed At Diagnostics

- What was tried: widened `smart_automove_pro_white_fast_forced_prepass_probe` so it compares the older traced Fast board with the fresh `pro_turn_planner_reliability_v5` Normal drift `l9,4;l8,5` vs current `l9,4;l8,3`, while keeping `primary_white_fast_screen_opening_0_ply9` as the retained comparison fixture.
- Why it stopped: the new Normal board was not a new family. Both traced boards showed the same live-only `search_only_forced_prepass` shape: `drainer_vulnerable=true`, `drainer_walk_vulnerable=false`, a safe `DrainerSafetyRecovery` selected root, and vulnerable `ManaTempo` pre-accept/current roots underneath. The retained fixture still stayed `drainer_vulnerable=false` and `engine_disabled`, so there was still no retained cheap-surface foothold for a production split.
- Durable lesson: when a fresh white replay only reproduces the existing live-only forced-prepass drainer-safety pattern, keep the widened probe and kill the production idea. Wait for that family to land on a retained surface before spending code.

## Apr 8, 2026: Fresh Seed v5 Duel Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v5` after the cheap gate reconfirmed the retained frontier was stalled at `human_win_pro_c` alone.
- Why it stopped: the replay was still all count-`1` churn. `vs current Pro` finished `5` regressions / `4` improvements / `3` flat, `vs current Normal` `2` / `2` / `8`, and `vs current Fast` `1` / `1` / `10`, with every move pair unique inside its duel bucket. The replay did bring back familiar shapes, including early-black `negative_deny`, later-black accepted-head `l1,5;l1,7;l0,7` vs current `l4,1;l5,0;mb`, and a white `search_only_forced_prepass` drift `l9,4;l8,5` vs current `l9,4;l8,3`, but none repeated strongly enough to justify another production cut.
- Durable lesson: do not treat the return of known live families as actionable by itself. If a fresh replay still spreads them across count-`1` move pairs, keep only the lesson and wait for a stronger repeated seam or a retained-surface foothold.

## Apr 8, 2026: Cheap Frontier Reconfirmed As Human-Only Stall

- What was tried: refreshed `smart_automove_pro_human_win_pro_c_selector_probe` again on the retained challenger, then reran the canonical cheap gate via `SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage runtime_pro_turn_engine_v30 runtime_current` to see whether any bounded production split still had a credible path.
- Why it stopped: the retained human fixture still showed the same safe-progress / followup-floor selector shape, and the cheap gate explicitly reconfirmed the frontier stall: `opening_reply` stayed `0/3`, `primary_pro` stayed `1/55`, `off_target_changed=0`, and the only changed fixture was still `human_win_pro_c`. That meant there was no new cheap-surface foothold beyond the already-rejected human-only drift.
- Durable lesson: once the canonical cheap gate itself reconfirms a `human_win_pro_c`-only `1/55` stall with no off-target churn, stop reopening human-only production ideas. Wait for a fresh duel replay to land on a retained surface again.

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

## Apr 9, 2026: Seed v17 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v17` to look for the next repeatable direct-duel seam after the black bridge fallback died.
- Why it stopped: the replay stayed non-actionable. Direct Pro finished `2` regressions / `6` improvements / `4` flat, Normal `0` / `1` / `11`, and Fast `3` / `3` / `6`, but every exact move pair still stayed at count `1`. The surviving regressions were only live-only white forced-prepass drift `l8,4;l8,5` vs current `l8,4;l8,3`, white `ManaTempo` sibling drift `l8,3;l8,2` vs `l8,3;l9,2`, and a late black spirit-head rerank `l1,5;l1,7;l0,7` vs `l1,6;l2,7`, with no repeated retained `primary_pro` foothold.
- Durable lesson: do not spend from a cleaner replay just because Normal nearly clears. If Pro and Fast still fragment into count-`1` white sibling/forced-prepass drift plus a one-off late black spirit-head rerank, keep only the note and wait for a replay that actually repeats on the retained surface.

## Apr 9, 2026: Seed v18 White Forced-Prepass Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v18`, then widened `smart_automove_pro_white_fast_forced_prepass_probe` after Normal repeated white `l9,6;l8,5` vs current `l9,6;l8,7` twice.
- Why it stopped: direct Pro finished clean at `0` regressions / `5` improvements / `7` flat and Fast also finished clean at `0` / `5` / `7`, but Normal still lost `5` / `1` / `6`. The repeated Normal white pair looked promising until the widened probe showed it was still the old live-only forced-prepass family: runtime-faithful v30 had `selected=l9,6;l8,5`, `pre_accept=baseline=head=l9,6;l8,7`, `stage=search_only_forced_prepass`, `accepted=true`, and `drainer_vulnerable=true`, while the retained `primary_white_fast_screen_opening_0_ply9` fixture stayed `drainer_vulnerable=false`, `engine_disabled`, and on spirit-progress roots. That left no retained `primary_pro` foothold for a production split.
- Durable lesson: do not spend from a replay just because one Normal white pair repeats while Pro and Fast are clean. If the repeated pair still maps to the old live-only forced-prepass drainer-safety override and the retained comparison fixture stays on a different non-vulnerable spirit-progress surface, keep only the widened probe and the note.

## Apr 9, 2026: Seed v19 Black Remix Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v19` to see whether the cleaner `v18` wall would converge onto one retained black family in the next seed.
- Why it stopped: the replay stayed fragmented. Direct Pro finished `1` regression / `5` improvements / `6` flat, Normal `3` / `2` / `7`, and Fast `2` / `5` / `5`, but every exact move pair still stayed at count `1`. The seed only resurfaced already-known retained black seams in different buckets: accepted non-vulnerable `ManaTempo` rerank `l1,5;l2,5` vs current `l1,6;l0,6`, mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`, action+mana `l1,6;l2,7` vs `l3,2;l4,1`, and late spirit-head `l1,5;l1,7;l0,7` vs `l1,6;l2,7`, plus one white spirit-own-setup rerank `l9,5;l8,5` vs `l9,5;l7,6;l7,7`.
- Durable lesson: do not spend from a replay just because several retained black seams reappear together. If they still arrive as count-`1` exact pairs in different duel buckets, keep only the note and wait for a seed that repeats one exact black family strongly enough to justify a shared in-path split.

## Apr 9, 2026: Spirit-Only Black Bridge Fallback Killed After Full Loop

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v20`, which repeated black `l1,5;l1,7;l0,7` vs current `l4,1;l5,0;mb` twice in Normal and once in Fast. Widened `smart_automove_pro_black_forced_runtime_probe` with the traced Normal board, confirmed it matches retained `primary_black_spirit_bridge_ply19` at runtime-faithful selection stage, then cut one narrow production guard that only routed that exact spirit-bridge pair back to current while leaving `primary_black_mana_bridge_ply20` alone.
- Why it stopped: the branch closed `primary_black_spirit_bridge_ply19`, passed the focused bridge tests, `guardrails`, `pro-triage(primary_pro)=4/60` with `off_target_changed=0`, and `runtime-preflight`, but `pro-reliability` still failed at `0.8333` vs current Pro, `0.5000` vs current Normal, and `0.7500` vs current Fast. Those are the exact same duel scores the older broad black bridge fallback already hit, which means over-clamping the mana-bridge seam was not the reason that family failed to promote.
- Durable lesson: do not reopen a spirit-only fallback on the `l4,1;l5,0;mb` baseline. If closing only `l1,5;l1,7;l0,7` still lands on the same full-loop duel scores as the broader bridge fallback, the bridge family itself is too local to move promotion.

## Apr 9, 2026: Seed v21 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v21` immediately after killing the spirit-only black bridge fallback, to see whether the wall stayed on that retained family or broke back into churn.
- Why it stopped: the replay broke back into pure count-`1` churn. Direct Pro finished `3` regressions / `1` improvement / `8` flat, Normal `3` / `2` / `7`, and Fast `1` / `4` / `7`, with every exact move pair staying at count `1`. The seed only remixed one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7`, black mana rerank `l1,5;l2,5` vs `l1,6;l0,6`, white mana-sibling `l8,3;l8,2` vs `l8,3;l9,2`, white spirit-own-setup `l9,5;l7,4;l7,3` vs `l9,5;l7,6;l7,7`, and a direct-Pro black tie `l1,2;l1,1` vs `l1,2;l0,1`.
- Durable lesson: do not spend from the next seed just because the previous one finally repeated a retained seam. If the follow-up replay immediately falls back into count-`1` mixed churn across Pro, Normal, and Fast, keep only the note and wait for a cleaner repeated family.

## Apr 9, 2026: Seed v22 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v22` to see whether the post-`v21` wall would finally settle on one repeated exact family.
- Why it stopped: the replay stayed in the same churn bucket. Direct Pro finished `2` regressions / `6` improvements / `4` flat, Normal `3` / `1` / `8`, and Fast `1` / `4` / `7`, but every exact move pair still stayed at count `1`. The seed only remixed one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, white mana-sibling `l8,3;l8,2` vs `l8,3;l9,2`, a direct-Pro black `ManaTempo` tie `l1,2;l1,1` vs `l1,2;l0,1`, early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7`, black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`, and white spirit-own-setup `l9,5;l7,4;l7,3` vs `l9,5;l7,6;l7,7`.
- Durable lesson: if the next replay after a dead local fix still mixes only count-`1` versions of the same black and white families, keep only the note. That is replay churn again, not a code-ready continuation.

## Apr 9, 2026: Seed v23 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v23` to see whether the post-`v22` wall would finally settle on one repeated exact family.
- Why it stopped: the replay stayed in the same churn bucket again. Direct Pro finished `3` regressions / `1` improvement / `8` flat, Normal `3` / `2` / `7`, and Fast `1` / `4` / `7`, but every exact move pair still stayed at count `1`. The seed only remixed one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, a direct-Pro black `ManaTempo` tie `l1,2;l1,1` vs `l1,2;l0,1`, early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7`, black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`, black mana rerank `l1,5;l2,5` vs `l1,6;l0,6`, white mana-sibling `l8,3;l8,2` vs `l8,3;l9,2`, and white spirit-own-setup `l9,5;l7,4;l7,3` vs `l9,5;l7,6;l7,7`.
- Durable lesson: if the next replay still comes back as the same mixed black-bridge, early-black, white-sibling, and white setup families at count `1`, keep only the note. Adding one more direct-Pro black `ManaTempo` tie is still replay churn, not a code-ready continuation.

## Apr 9, 2026: Seed v24 Bridge Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v24` to see whether the post-`v23` wall would finally settle onto a repeated retained seam worth reopening.
- Why it stopped: the replay did settle, but only onto an already-killed local family. Direct Pro finished `1` regression / `4` improvements / `7` flat, Normal `2` / `1` / `9`, and Fast `4` / `2` / `6`. Fast repeated black spirit-bridge `l1,5;l1,7;l0,7` vs current `l4,1;l5,0;mb` three times and added one black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`, while direct Pro and Normal only contributed one-off early-black `negative_deny`, accepted black mana rerank `l1,5;l2,5` vs `l3,2;l4,1`, and white mana-sibling `l9,5;l8,6` vs `l9,6;l8,6` drift. That would normally justify a bridge-family reopen, except both the older broad bridge fallback and the later spirit-only bridge fallback already closed those retained seams and still failed full loops.
- Durable lesson: if a fresh seed repeats the retained `l4,1;l5,0;mb` bridge family again, do not reopen it with another fallback-to-current branch by itself. That replay only reconfirms that the bridge baseline is live, not that the previously killed fallback strategy has become promotable.

## Apr 9, 2026: v24 White Engine-Disabled Revival Killed Before Code Edits

- What was tried: added `smart_automove_pro_white_engine_disabled_runtime_probe` to compare the fresh `v24` Normal white drift `l9,5;l8,6` vs current `l9,6;l8,6` against the closed `primary_spirit_setup` surface and the nearby engine-disabled white opening board.
- Why it stopped: the traced board was neither nearby retained surface. Runtime-faithful v30 already had `selected=l9,5;l8,6`, `pre_accept=baseline=l9,6;l8,6`, `stage=engine_disabled`, `accepted=false`, and a nearby head `l8,5;l7,4` under `SafeSupermanaProgress -> ImmediateScore`. `primary_spirit_setup` stayed different: `engine_post_search`, spirit-root selected/current `l9,7;...`, and head `l9,7;l7,8;l8,7`. The nearby engine-disabled opening board also stayed different: `selected=baseline=l9,6;l8,6`, `pre_accept=l10,6;l9,5`, head `l9,6;l8,5`, and `l9,5;l8,6` was absent from the shortlist entirely.
- Durable lesson: do not reopen a white engine-disabled drift just because it shares `l9,5;l8,6` or `l9,6;l8,6` with older retained probes. Shared neighborhood is still weaker than runtime-faithful stage shape; this was another local white `ManaTempo` sibling seam, not a real `primary_spirit_setup` or opening-family revival.

## Apr 9, 2026: v24 Normal Black Baseline Revival Killed Before Code Edits

- What was tried: widened `smart_automove_pro_black_forced_runtime_probe` with the fresh `v24` Normal black drift `l1,5;l2,5` vs current `l3,2;l4,1` to compare it directly against the retained late-head seam and the previously traced accepted black runtime branches on the same current baseline.
- Why it stopped: the traced board was another distinct accepted runtime surface, not a shared black family. Runtime-faithful v30 already had `selected=pre_accept=head=l1,5;l2,5`, `forced_inputs=Some("l1,5;l2,5")`, `stage=engine_post_search`, `accepted=true`, `head_family=ManaTempo`, and `goal_family=SpiritImpact`, while current stayed on non-vulnerable `ManaTempo` `l3,2;l4,1`. That differs from the retained late-head seam, from the traced fast `v10` black rerank where `pre_accept` stayed on current, and from the traced `v13` drainer-safety branch.
- Durable lesson: do not reopen a black drift just because it shares current `l3,2;l4,1` with several older probes. That baseline still branches into multiple accepted black runtime surfaces, so shared baseline alone is not enough for another production split.

## Apr 9, 2026: Seed v25 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v25` to see whether the post-`v24` wall would move onto a new retained exact family after the black and white side branches were classified.
- Why it stopped: the replay fell back into mixed count-`1` churn again. Direct Pro finished `4` regressions / `3` improvements / `5` flat, Normal `3` / `3` / `6`, and Fast `1` / `3` / `8`, with every exact move pair still at count `1`. The seed only remixed one-off direct-Pro black safe-progress `l0,6;l1,6` vs current `l1,5;l2,7;l1,8`, white engine-disabled `ManaTempo` ties `l8,3;l8,2` vs `l8,3;l9,2`, white forced-prepass `l9,6;l8,5` vs `l9,6;l8,7`, white spirit-own-setup `l9,5;l7,4;l7,3` vs `l9,5;l7,6;l7,7`, Normal white ties `l9,2;l9,1` vs `l9,2;l10,1` and `l9,4;l8,4` vs `l9,2;l8,2`, one Normal black spirit-bridge replay `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, and one Fast black drainer-safety rerank `l1,4;l0,5` vs `l4,1;l5,0;mb`.
- Durable lesson: do not spend from a seed like `v25` just because it introduces a new black rerank on a familiar baseline. If Pro, Normal, and Fast all still end with only count-`1` exact pairs across unrelated black and white families, keep only the note and wait for a replay that actually repeats on one retained surface.

## Apr 9, 2026: v26 Repeated White Safe-Progress Revival Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v26`, then widened `smart_automove_pro_white_safe_progress_probe` after Normal repeated white `l9,5;l8,5` vs current `l9,5;l7,6;l7,7` twice.
- Why it stopped: the replay still mixed multiple one-off families across buckets. Direct Pro finished `2` regressions / `4` improvements / `6` flat and Fast `2` / `5` / `5`, with every exact pair in those buckets still at count `1`; only Normal repeated one exact pair. The widened white safe-progress probe showed that repeated board is not the retained white safe-progress rerank and not the retained fast-screen surface: runtime-faithful v30 already has `selected=pre_accept=l9,5;l8,5`, `forced_inputs=None`, `stage=engine_post_search`, `accepted=false`, and a `SafeOpponentManaProgress -> DrainerSafetyRecovery` head shell, while `primary_white_safe_progress_rerank_ply27` stays an accepted vulnerable `ManaTempo` rerank under `ImmediateScore`, and `primary_white_fast_screen_opening_0_ply9` keeps `l9,5;l8,5` only as a rejected head over spirit-progress pre-accept/current roots.
- Durable lesson: do not reopen a repeated white `l9,5;l8,5` seam just because it repeats in Normal. If the traced board still fails both retained white-surface matches on runtime-faithful stage, acceptance, and selected-root shape, keep only the widened probe and the note.

## Apr 9, 2026: Seed v27 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v27` to see whether the post-`v26` wall would finally collapse onto one repeated retained seam.
- Why it stopped: the replay was cleaner, but still fragmented. Direct Pro finished `1` regression / `3` improvements / `8` flat, Normal `2` / `2` / `8`, and Fast `2` / `7` / `3`, yet every exact move pair still stayed at count `1`. The seed only produced one-off direct-Pro white engine-disabled `ManaTempo` sibling drift `l7,4;l8,3` vs current `l6,7;l6,6`, one-off Normal black spirit reranks `l1,5;l2,7;l3,7` vs `l1,5;l2,7;l1,8` and `l2,5;l4,3;l5,3` vs `l2,5;l2,7;l1,6`, and one-off Fast white safe-progress / mana reranks `l8,5;l8,6` vs `l7,0;l6,1` and `l10,5;l9,6` vs `l9,7;l8,6`.
- Durable lesson: do not spend from a replay just because it is cleaner or because one bucket improves more often than it regresses. If Pro, Normal, and Fast still end with only count-`1` exact pairs, keep only the note and wait for a seed that repeats on one retained surface.

## Apr 9, 2026: v28 Mixed Early-Black And White Forced-Prepass Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v28`, then widened `smart_automove_pro_white_fast_forced_prepass_probe` after Normal repeated white `l8,4;l8,5` vs current `l8,4;l8,3` twice.
- Why it stopped: the replay split across previously known families instead of collapsing onto one branch story. Direct Pro finished `3` regressions / `6` improvements / `3` flat and repeated early-black `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` twice, but Normal finished `4` / `0` / `8` and repeated white forced-prepass `l8,4;l8,5` vs `l8,4;l8,3` twice, while Fast finished `2` / `3` / `7` and added only one-off black bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb` plus one-off white `l10,4;l9,3` vs `l9,5;l7,6;l8,7`. The widened forced-prepass probe showed the repeated Normal white board is still the old live-only drainer-safety shell: runtime-faithful v30 has `selected=l8,4;l8,5`, `pre_accept=baseline=l8,4;l8,3`, `forced_inputs=Some("l8,4;l8,3")`, `stage=search_only_forced_prepass`, `accepted=true`, and `drainer_vulnerable=true`, while `primary_white_fast_screen_opening_0_ply9` remains `drainer_vulnerable=false`, `engine_disabled`, and on spirit-progress roots.
- Durable lesson: do not reopen early-black `negative_deny` just because it repeats again if the same seed also repeats the old white forced-prepass family. That is still a mixed replay, not a single promotable branch, so keep only the widened probe and the note.

## Apr 9, 2026: Seed v29 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v29` to see whether the post-`v28` wall would stay on one retained black or white family after the mixed replay was ruled out.
- Why it stopped: the replay got cleaner again, but still stayed count-`1` across every bucket. Direct Pro finished `2` regressions / `4` improvements / `6` flat, Normal `2` / `3` / `7`, and Fast `3` / `2` / `7`; every exact move pair still stayed at count `1`. The seed only remixed one-off direct-Pro early-black `negative_deny` `l0,5;l1,6` vs current `l1,5;l3,6;l2,7`, one-off direct-Pro black spirit sibling `l0,4;l1,3` vs `l0,4;l1,4`, one-off Normal black spirit rerank `l2,4;l0,5;l1,5` vs `l2,4;l4,2;l3,2`, one-off Normal white engine-disabled tie `l8,7;l8,8` vs `l8,7;l9,8`, and one-off Fast black bridge plus white forced-prepass drift.
- Durable lesson: do not spend from a replay just because it gets cleaner after a mixed-seam kill. If Pro, Normal, and Fast still end with only count-`1` exact pairs, keep only the note and wait for a seed that repeats on one retained surface.

## Apr 9, 2026: Seed v30 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v30` to see whether the post-`v29` wall would finally collapse once Pro and Fast cleaned up.
- Why it stopped: the replay got cleaner again, but there was still no repeated retained seam. Direct Pro finished clean at `0` regressions / `4` improvements / `8` flat, Normal `3` / `3` / `6`, and Fast clean at `0` / `2` / `10`; every exact move pair still stayed at count `1`. The only regressions were one-off Normal white `ManaTempo` reranks `l7,4;l8,3` vs `l7,6;l8,7`, `l6,3;l7,3` vs `l6,7;l7,7`, and `l9,4;l8,3` vs `l5,2;l4,1`.
- Durable lesson: do not spend from a replay just because Pro and Fast go clean. If Normal is still losing only on count-`1` local reranks, keep only the note and wait for a seed that actually repeats on one retained surface.

## Apr 9, 2026: Seed v31 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v31` to see whether the post-`v30` wall would finally justify reopening one of the retained black bridge seams.
- Why it stopped: the replay still split across the already-killed `l4,1;l5,0;mb` bridge family and unrelated one-off drift. Direct Pro finished `1` regression / `5` improvements / `6` flat, Normal `2` / `3` / `7`, and Fast `3` / `0` / `9`. Normal did repeat black spirit-bridge `l1,5;l1,7;l0,7` vs current `l4,1;l5,0;mb` twice, but Fast only added one old black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb` plus one-off white `l10,4;l9,3` vs `l9,5;l7,6;l8,7` and one-off black `ManaTempo` rerank `l2,7;l2,8` vs `l7,1;l6,1`, while direct Pro only added one-off black `l0,5;l1,4` vs `l1,5;l2,4`.
- Durable lesson: do not reopen the `l4,1;l5,0;mb` bridge fallback family just because Normal repeats the spirit-bridge seam again. If the same seed still splits across bridge variants and unrelated one-off drift in the other buckets, keep only the note and wait for a genuinely new branch story.

## Apr 9, 2026: Seed v32 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v32` to see whether the post-`v31` wall would finally converge onto one retained seam after the latest black bridge replay died.
- Why it stopped: the replay still stayed fragmented across related black baselines and unrelated white churn. Direct Pro finished `2` regressions / `5` improvements / `5` flat, Normal `3` / `0` / `9`, and Fast `2` / `3` / `7`, but every exact move pair still stayed at count `1`. Normal only mixed the two old `l4,1;l5,0;mb` bridge variants `l0,5;l1,4` and `l1,5;l1,7;l0,7`, Fast only remixed `l0,5;l1,4` against current `l1,6;l0,6` plus the old white forced-prepass `l9,4;l8,5` vs `l9,4;l8,3`, and direct Pro added only one-off black spirit rerank `l1,5;l2,7;l3,6` vs `l1,5;l2,7;l1,8` plus white `engine_disabled` tie `l8,4;l8,5` vs `l8,4;l9,4`.
- Durable lesson: do not spend from a seed just because it surfaces multiple related black bridge baselines at once. If every exact pair still stays at count `1` and the seed also mixes in old white forced-prepass or engine-disabled churn, keep only the note and wait for one repeated exact family with a retained foothold.

## Apr 9, 2026: Seed v33 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v33` to see whether the post-`v32` wall would finally settle onto one retained seam instead of another mixed replay.
- Why it stopped: the replay stayed count-`1` across every bucket again. Direct Pro finished `3` regressions / `3` improvements / `6` flat and only mixed one old early-black `negative_deny` replay `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` with two unrelated white reranks `l9,4;l8,5` vs `l9,6;l8,7` and `l8,7;l8,8` vs `l8,7;l9,8`. Normal finished `4` / `2` / `6` and only mixed the two old `l4,1;l5,0;mb` bridge variants `l0,5;l1,4` and `l1,5;l1,7;l0,7` with one white sibling tie `l10,4;l9,3` vs `l9,4;l9,3` and one black sibling tie `l2,7;l2,8` vs `l2,7;l1,8`. Fast finished `1` / `2` / `9` and added only one old `l0,5;l1,4` mana-bridge replay.
- Durable lesson: do not spend from a seed just because it replays several already-known families at once. If direct Pro, Normal, and Fast all stay count-`1` and no exact seam repeats across buckets, keep only the note and wait for one repeated exact family with a retained match.

## Apr 9, 2026: Seed v34 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v34` to see whether the cleaner post-`v33` wall would justify reopening the retained black mana-bridge seam.
- Why it stopped: the seed still reduced to the same already-killed bridge-fallback family. Direct Pro finished `1` regression / `3` improvements / `8` flat and only had one white `ManaTempo` rerank `l6,7;l7,7` vs current `l6,4;l7,3`. Normal finished `2` / `2` / `8` and repeated only the old black mana-bridge seam `l0,5;l1,4` vs current `l4,1;l5,0;mb` twice. Fast finished `1` / `1` / `10` and added only one white `ManaTempo` tie `l9,4;l8,3` vs `l9,4;l8,4`. There was no new repeated exact family beyond the bridge seam, and the older broad bridge fallback already proved that routing that family back to current is too local to clear `pro-reliability`.
- Durable lesson: do not reopen the black mana-bridge fallback just because a cleaner seed repeats it twice while Pro and Fast are otherwise quiet. If the only repeated exact pair is still the already-killed bridge family, keep only the note and wait for a new repeated seam with a fresh in-path explanation.

## Apr 9, 2026: Seed v35 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v35`, then reran `smart_automove_pro_white_safe_progress_probe` after Normal repeated `l9,4;l8,3` vs current `l5,2;l4,1` twice.
- Why it stopped: the repeated Normal pair is real, but it was still too local to justify code. Direct Pro finished `2` regressions / `3` improvements / `7` flat, Normal `3` / `2` / `7`, and Fast `1` / `3` / `8`. The white safe-progress probe confirmed the repeated Normal pair matches retained `primary_white_safe_progress_rerank_ply27` exactly: runtime-faithful v30 still has `selected=pre_accept=head=l9,4;l8,3`, `accepted=true`, and a vulnerable `ManaTempo` root over current non-vulnerable `SafeSupermanaProgress` `l5,2;l4,1`. But the rest of the replay did not line up behind it: direct Pro only had one-off white reranks `l10,6;l9,5` vs `l10,5;l9,4` and `l8,5;l7,6` vs `l8,5;l7,4`, Fast only had one-off white `l10,4;l9,3` vs `l9,5;l7,6;l8,7`, and the only black regression was a one-off early `negative_deny` replay `l0,5;l1,6` vs `l1,5;l3,6;l2,7`.
- Durable lesson: do not spend from a seed just because one retained Normal seam finally repeats twice. If direct Pro and Fast still stay on unrelated one-off reranks, keep only the note and wait for a cross-bucket duel story with a fresh shared lever.

## Apr 9, 2026: Seed v37 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v37` after the white safe-progress story from `v35` failed to carry through `v36`.
- Why it stopped: the replay stayed too local and still failed to produce a repeated exact family. Direct Pro finished `1` regression / `3` improvements / `8` flat, Normal `2` / `2` / `8`, and Fast `1` / `3` / `8`, with every exact move pair still at count `1`. Direct Pro only had one white opening rerank `l10,6;l9,5` vs `l10,5;l9,4`; Normal mixed one old black spirit-bridge `l1,5;l1,7;l0,7` vs current `l4,1;l5,0;mb` with one white rerank `l10,5;l9,4` vs `l9,6;l7,6;l7,7`; Fast only added one copy of that same old black spirit-bridge seam.
- Durable lesson: do not spend from a seed just because direct Pro stays clean and both losing buckets each replay a familiar one-off seam. If every exact move pair still stays at count `1`, keep only the note and wait for one repeated exact family with a retained foothold and a fresh shared lever.

## Apr 9, 2026: Seed v38 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v38` to see whether the post-`v37` wall would finally collapse onto one retained family after the cleaner direct-Pro result.
- Why it stopped: Fast finally repeated one retained black seam, but it was still the already-killed broad bridge-fallback family and the rest of the seed stayed fragmented. Direct Pro finished `1` regression / `5` improvements / `6` flat and only replayed early-black `negative_deny` `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` once. Normal finished `5` / `1` / `6` and mixed white forced-prepass `l9,6;l9,4;l8,5` vs `l9,6;l7,4;l8,4`, old black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`, white engine-disabled `l9,3;l8,3` vs `l9,5;l8,5`, white spirit rerank `l9,5;l7,4;l8,3` vs `l9,5;l7,6;l7,7`, and black spirit rerank `l2,5;l0,6;l1,6` vs `l2,5;l2,3;l1,2`, each only once. Fast finished `6` / `1` / `5` and repeated black mana-bridge `l0,5;l1,4` vs current `l4,1;l5,0;mb` three times, but only added one-off black spirit-bridge `l1,5;l1,7;l0,7`, white `l8,5;l8,6` vs `l8,5;l7,4`, and black spirit sibling `l0,4;l1,3` vs `l0,4;l1,4`.
- Durable lesson: do not reopen the black mana-bridge fallback just because a fresh seed finally repeats it several times in Fast. If the repeated seam is still the already-killed bridge family and Pro/Normal continue to mix unrelated one-offs, keep only the note and wait for a new repeated seam with a fresh shared lever.

## Apr 9, 2026: Seed v39 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v39` to see whether the post-`v38` wall would simplify once the repeated Fast bridge family fell away.
- Why it stopped: the replay got quieter in Fast, but Pro and Normal still mixed unrelated count-`1` seams. Direct Pro finished `2` regressions / `4` improvements / `6` flat and split immediately across one-off black spirit rerank `l1,5;l2,3;l3,3` vs current `l1,5;l2,7;l1,8` and one-off black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`. Normal finished `4` / `1` / `7` and added one-off white `ManaTempo` sibling `l8,5;l7,4` vs `l8,5;l9,4`, one-off white forced-prepass `l9,4;l8,5` vs `l9,4;l8,3`, one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, and one-off black action+mana `l1,6;l2,7` vs `l3,2;l4,1`. Fast finished `1` / `2` / `9` and only added one-off black `ManaTempo` rerank `l2,7;l2,8` vs `l7,1;l6,1`.
- Durable lesson: do not spend from a quieter replay just because Fast mostly goes flat again. If direct Pro and Normal are still mixing unrelated count-`1` black and white seams, keep only the note and wait for one repeated exact family with a retained foothold and a fresh shared lever.

## Apr 9, 2026: Seed v40 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v40` to see whether the post-`v39` wall would finally collapse once direct Pro returned to one old black seam.
- Why it stopped: the replay stayed count-`1` across all three buckets again. Direct Pro finished `1` regression / `4` improvements / `7` flat and only replayed early-black `negative_deny` `l0,5;l1,6` vs current `l1,5;l3,6;l2,7`. Normal finished `3` / `1` / `8` and mixed one-off white safe-progress `l10,7;l9,6` vs `l9,8;l8,7`, one-off white `l9,5;l8,5` vs `l9,5;l7,6;l7,7`, and one-off black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`. Fast finished `2` / `2` / `8` and added one-off white forced-prepass `l9,6;l8,5` vs `l9,6;l8,7` plus one-off white `ManaTempo` sibling `l6,3;l7,2` vs `l6,3;l7,3`.
- Durable lesson: do not spend from a seed just because direct Pro drops back to one known regression. If Normal and Fast still contribute only unrelated count-`1` white and black seams, keep only the note and wait for one repeated exact family with a retained foothold and a fresh shared lever.

## Apr 9, 2026: Seed v41 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v41` after the accidental `v40` rerun was discarded, to see whether the wall would finally line up behind one new exact family.
- Why it stopped: the replay stayed count-`1` across all three buckets again. Direct Pro finished `1` regression / `5` improvements / `6` flat and only replayed white spirit-own-setup rerank `l9,6;l7,4;l8,3` vs current `l9,6;l7,6;l7,7`. Normal finished `4` / `1` / `7` and mixed one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb` with three separate white `ManaTempo` boards `l7,4;l7,3` vs `l7,4;l7,5`, `l9,2;l9,1` vs `l9,2;l10,1`, and `l9,8;l8,9` vs `l9,2;l8,1`. Fast finished `3` / `4` / `5` and added one-off white `l10,4;l9,3` vs `l9,5;l7,6;l8,7`, one-off black `ManaTempo` rerank `l2,7;l2,8` vs `l2,7;l1,8`, and one-off white `l10,3;l9,2` vs `l7,3;l7,2`.
- Durable lesson: do not spend from a replay just because direct Pro stays near-clean and Fast improves more often than it regresses. If every exact move pair still stays at count `1` and Normal/Fast split across several unrelated white and black seams, keep only the note and wait for one repeated exact family with a retained foothold and a fresh shared lever.

## Apr 9, 2026: Seed v42 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v42` to see whether another direct-Pro replay of early-black `negative_deny` would finally line up across the other buckets.
- Why it stopped: direct Pro did repeat the old early-black seam, but the rest of the replay still broke the shared story. Direct Pro finished `3` regressions / `3` improvements / `6` flat and repeated `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` twice, while also adding one-off white spirit-own-setup rerank `l9,5;l7,4;l7,3` vs `l9,5;l7,6;l7,7`. Normal finished `3` / `0` / `9` and mixed one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one-off early-black `l0,5;l1,6` vs `l1,5;l3,6;l2,7`, and one-off white spirit-own-setup rerank `l9,6;l7,4;l8,3` vs `l9,6;l7,6;l7,7`. Fast finished `2` / `4` / `6` and added the old white forced-prepass shell `l8,4;l8,5` vs `l8,4;l8,3` plus one-off black `l0,5;l1,4` vs `l3,2;l4,1`.
- Durable lesson: do not reopen early-black `negative_deny` just because direct Pro repeats it again. If Normal and Fast still split across the old bridge, white spirit-own-setup, and white forced-prepass stories, keep only the note and wait for one repeated exact family with a retained foothold and a fresh shared lever.

## Apr 9, 2026: Seed v43 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v43` to see whether a quieter replay with clean-or-near-clean buckets would finally expose one bounded seam worth code.
- Why it stopped: Normal did go clean, but Pro and Fast still did not line up behind one exact family. Direct Pro finished `1` regression / `4` improvements / `7` flat, with its only loss on the old black action+mana seam `l1,6;l2,7` vs current `l3,2;l4,1`. Normal finished clean at `0` / `4` / `8`. Fast finished `3` / `3` / `6` and split across one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one-off white `l10,4;l9,3` vs `l9,5;l7,6;l8,7`, and one-off white `l8,3;l8,2` vs `l8,3;l9,2`.
- Durable lesson: do not spend from a replay just because Normal goes clean while Pro and Fast stay relatively quiet. If the only remaining seams are still unrelated count-`1` Pro/Fast drifts with no repeated exact family, keep only the note and wait for one shared lever.

## Apr 9, 2026: Seed v44 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v44` after the quieter `v43` result, to see whether the next seed would finally collapse onto one retained exact family.
- Why it stopped: the replay stayed too fragmented to justify code. Direct Pro finished `2` regressions / `4` improvements / `6` flat and split across one-off black spirit sibling `l0,4;l1,3` vs current `l0,4;l1,4` plus one-off accepted safe-progress `l3,5;l4,4` vs `l1,5;l3,5;l3,6`. Normal finished `1` / `3` / `8` and only added one white `l9,4;l8,5` vs `l9,4;l8,3` trace where runtime-faithful `selected` and `pre_accept` already matched current. Fast finished `3` / `3` / `6` and split across one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one-off black action+mana `l1,6;l2,7` vs `l3,2;l4,1`, and one-off white spirit head `l4,9;l4,7;l3,7` vs `l4,9;l3,9`. Every exact move pair still stayed at count `1`.
- Durable lesson: do not spend from a replay just because direct Pro stays relatively quiet and Normal nearly goes clean. If every exact move pair still stays at count `1` and Fast keeps mixing old black seams with an unrelated white head, keep only the note and wait for one repeated exact family with a retained foothold and a shared lever.

## Apr 9, 2026: Seed v45 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v45` to see whether the quieter post-`v44` wall would finally line up behind one retained black family.
- Why it stopped: Fast did go clean, but the losing buckets still did not share one fresh exact story. Direct Pro finished `3` regressions / `2` improvements / `7` flat and split across one-off black spirit sibling `l0,4;l1,3` vs current `l0,4;l1,5`, one-off black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`, and one-off black spirit rerank `l1,5;l2,3;l3,3` vs `l1,5;l2,7;l1,8`. Normal finished `4` / `2` / `6` and did repeat the old black spirit-bridge seam `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb` twice, but that is still the already-killed bridge-fallback family; its other regressions were only one-off white spirit rerank `l9,5;l7,4;l7,3` vs `l8,7;l7,8` and one-off white `ManaTempo` tie `l9,2;l9,1` vs `l9,2;l10,1` where runtime-faithful selection already matched current. Fast finished clean at `0` / `4` / `8`.
- Durable lesson: do not spend from a replay just because Fast goes clean and one old Normal seam repeats twice. If direct Pro still splits across unrelated one-off black seams and the only repeated loser is an already-killed bridge family, keep only the note and wait for one repeated exact family with a fresh shared lever.

## Apr 9, 2026: Seed v46 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v46` to see whether the post-`v45` wall would simplify once Fast stopped contributing losses.
- Why it stopped: the replay did get quieter in Pro and Normal, but it still failed to produce one repeated exact family. Direct Pro finished `2` regressions / `5` improvements / `5` flat and split across one-off white `l9,6;l8,7` vs current `l9,5;l7,6;l8,7` plus one-off early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7`. Normal finished `1` / `0` / `11` and only added one-off white spirit-own-setup `l9,5;l7,4;l8,3` vs `l9,5;l7,6;l7,7`. Fast finished `3` / `5` / `4` and split across one-off black spirit sibling `l0,4;l1,3` vs `l0,4;l1,4`, one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, and one-off black `ManaTempo` tie `l2,7;l2,8` vs `l2,7;l1,8`. Every exact move pair still stayed at count `1`.
- Durable lesson: do not spend from a replay just because Pro and Normal both get quieter again. If every exact move pair still stays at count `1` and Fast reverts to a mix of unrelated black one-offs, keep only the note and wait for one repeated exact family with a retained foothold and a fresh shared lever.

## Apr 9, 2026: Seed v47 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v47` to see whether a very quiet direct-Pro bucket would finally expose one bounded follow-up seam worth code.
- Why it stopped: direct Pro did fall to a single loss, but the losing buckets still did not share one fresh exact story. Direct Pro finished `1` regression / `4` improvements / `7` flat, with its only loss on one-off black accepted `ManaTempo` rerank `l0,5;l1,5` vs current `l2,2;l3,3`. Normal finished `3` / `3` / `6` and repeated the old black spirit-bridge seam `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb` twice, but that is still the already-killed bridge-fallback family; its other loss was one-off black `ManaTempo` tie `l2,7;l2,8` vs `l2,7;l1,8`. Fast finished `4` / `2` / `6` and split across one-off black drainer-safety `l1,4;l0,5` vs `l4,1;l5,0;mb`, one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one-off white `l10,4;l9,3` vs `l9,5;l7,6;l8,7`, and one-off black `ManaTempo` tie `l2,7;l2,8` vs `l7,1;l6,1`.
- Durable lesson: do not spend from a replay just because direct Pro drops to one regression. If the only repeated loser is still an already-killed bridge family and Fast fragments into unrelated one-offs, keep only the note and wait for one repeated exact family with a fresh shared lever.

## Apr 9, 2026: Seed v48 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v48` to see whether the next sample would convert the cleaner `v47` buckets into a real promotion path.
- Why it stopped: Normal and Fast both went clean, but direct Pro alone still broke into unrelated one-offs. Direct Pro finished `4` regressions / `4` improvements / `4` flat, with losses on one-off black `ManaTempo` tie `l2,3;l2,2` vs current `l2,3;l1,2`, one-off white `l10,5;l9,4` vs `l9,8;l7,6;l7,7`, one-off white `l4,9;l2,7;l2,8` vs `l4,9;l2,7;l3,6`, and one-off white `l6,3;l7,2` vs `l6,5;l7,4`. Normal finished clean at `0` / `0` / `12`. Fast finished clean at `0` / `4` / `8`. Every exact move pair still stayed at count `1`.
- Durable lesson: do not spend from a replay just because Normal and Fast both go clean. If the only losing bucket is still direct Pro and it fractures into unrelated count-`1` one-offs, keep only the note and wait for one repeated exact family with a fresh shared lever.

## Apr 9, 2026: Seed v49 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v49` to see whether the next sample would turn the very clean direct-Pro bucket into a real branch hypothesis.
- Why it stopped: direct Pro nearly cleaned up, but the other buckets exploded into unrelated one-offs. Direct Pro finished `1` regression / `6` improvements / `5` flat, with its only loss on one-off black bridge-like rerank `l1,5;l1,7;l0,7` vs current `l5,1;l6,0`. Normal finished `7` / `0` / `5` and mixed one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one-off black `l1,5;l1,7;l0,7` vs `l4,2;l4,1`, one-off black `l2,5;l0,6;l1,6` vs `l2,5;l2,3;l1,2`, one-off black spirit sibling `l0,4;l1,3` vs `l0,4;l1,4`, one-off black `ManaTempo` tie `l2,7;l2,8` vs `l2,7;l1,8`, one-off white `l8,7;l7,8` vs `l10,7;l10,8`, and one-off white spirit-own-setup `l9,5;l7,4;l8,3` vs `l9,5;l7,6;l7,7`. Fast finished `1` / `3` / `8` and only added one-off black spirit sibling `l0,4;l1,3` vs `l0,4;l1,5`. Every exact move pair still stayed at count `1`.
- Durable lesson: do not spend from a replay just because direct Pro nearly goes clean. If Normal and Fast break into unrelated count-`1` seams and the only Pro loss is a one-off on a new baseline, keep only the note and wait for one repeated exact family with a fresh shared lever.

## Apr 9, 2026: Seed v50 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v50` to see whether a repeated Normal white family would finally line up across the other buckets.
- Why it stopped: Normal did repeat a new white `ManaTempo` tie, but the other buckets still diverged. Direct Pro finished `3` regressions / `4` improvements / `5` flat and split across one-off white `l10,5;l9,4` vs `l9,2;l8,1`, one-off white `l9,4;l8,5` vs `l8,1;l7,0`, and one-off white forced-prepass `l9,5;l8,3;l7,2` vs `l9,2;l8,1`. Normal finished `3` / `3` / `6` and repeated `l6,5;l6,6` vs current `l6,5;l7,4` twice, while also adding one-off black `l1,5;l2,4` vs `l1,6;l2,7`. Fast finished `3` / `2` / `7` and split across one-off black `l2,7;l1,8` vs `l2,3;l3,2`, one-off white `l9,6;l8,5` vs `l9,5;l8,5`, and one-off black `l0,6;l1,6` vs `l1,5;l2,5`. Every exact move pair still stayed at count `1` outside that one Normal pair.
- Durable lesson: do not spend from a replay just because Normal repeats a new exact seam. If direct Pro and Fast still split across unrelated one-off families, keep only the note and wait for one repeated exact family with a fresh shared lever.

## Apr 9, 2026: Seed v51 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v51` to check whether the old black bridge baseline would finally line up into a promotable shared family.
- Why it stopped: Fast did repeat the old black spirit-bridge seam `l1,5;l1,7;l0,7` vs current `l4,1;l5,0;mb` twice, but the rest of the seed still broke the story. Direct Pro finished `3` regressions / `2` improvements / `7` flat and split across one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one-off early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7`, and one-off white spirit-own-setup `l9,7;l8,5;l7,4` vs `l9,7;l7,6;l7,7`. Normal finished `3` / `2` / `7` and split across one-off black drainer-safety `l1,6;l0,5` vs `l4,1;l5,0;mb`, one-off white engine-disabled tie `l8,7;l8,8` vs `l9,5;l8,6`, and the old white forced-prepass shell `l9,4;l8,5` vs `l9,4;l8,3`. Fast finished `3` / `5` / `4` and added only one other one-off white engine-disabled rerank `l10,4;l9,3` vs `l9,2;l8,1`.
- Durable lesson: do not spend from a replay just because Fast repeats the old black spirit-bridge seam twice. If Pro and Normal still split across unrelated one-offs and the only repeated family already failed both the broad bridge fallback and the later spirit-only bridge fallback, keep only the note and wait for a repeated exact family with a fresh shared lever.

## Apr 9, 2026: Seed v52 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v52` to check whether the newer retained black seams would finally cluster into one promotable replay family.
- Why it stopped: all three buckets fractured into count-`1` seams. Direct Pro finished `3` regressions / `3` improvements / `6` flat and split across one-off early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7`, one-off black action+mana `l1,6;l2,7` vs `l2,3;l3,2`, and one-off black spirit sibling `l0,4;l1,4` vs `l0,4;l1,5`. Normal finished `3` / `5` / `4` and split across one-off white engine-disabled tie `l6,3;l7,3` vs `l7,7;l8,8`, one-off white forced-prepass `l9,6;l8,5` vs `l9,6;l8,7`, and one-off black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`. Fast finished `4` / `4` / `4` and split across one-off black spirit sibling `l0,4;l1,3` vs `l0,4;l1,4`, one-off early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7`, one-off black drainer-safety `l1,4;l0,5` vs `l4,1;l5,0;mb`, and one-off black `ManaTempo` tie `l2,7;l2,8` vs `l2,7;l1,8`.
- Durable lesson: do not spend from a replay just because it replays several retained black families across all three buckets. If none of them repeats exactly and the seed still mixes in unrelated white engine-disabled and forced-prepass one-offs, keep only the note and wait for one exact family to lead the sample.

## Apr 9, 2026: Seed v53 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v53` to see whether the bridge baseline would finally condense into one usable replay family after the `v52` mixed black-seam sample.
- Why it stopped: direct Pro nearly cleaned up and Fast went clean, but Normal broke apart instead. Direct Pro finished `1` regression / `4` improvements / `7` flat, with its only loss on the old black mana-bridge seam `l0,5;l1,4` vs current `l4,1;l5,0;mb`. Normal finished `5` / `2` / `5` and split across one-off bridge-family losses `l0,5;l1,4` vs `l4,1;l5,0;mb` and `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one-off black action+mana `l1,6;l2,7` vs `l3,2;l4,1`, one-off black `ManaTempo` tie `l2,7;l2,8` vs `l2,7;l1,8`, and one-off white forced-prepass rerank `l9,6;l10,5` vs `l9,6;l8,7`. Fast finished clean at `0` / `4` / `8`.
- Durable lesson: do not spend from a replay just because Fast goes clean and direct Pro drops to one loss again. If the only Pro loss is still an already-killed bridge family and Normal fractures into unrelated count-`1` seams, keep only the note and wait for one exact family with a fresh shared lever.

## Apr 9, 2026: Seed v54 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v54` to see whether the bridge family would extend onto the new direct-Pro `l5,1;l6,0` baseline strongly enough to justify a retained follow-up.
- Why it stopped: all three buckets fractured into count-`1` seams again. Direct Pro finished `4` regressions / `4` improvements / `4` flat and mixed one-off black bridge-like `l1,5;l1,7;l0,7` vs current `l5,1;l6,0`, one-off early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7`, and two one-off white `ManaTempo` reranks `l9,3;l8,2` vs `l8,5;l9,4` and `l8,5;l8,6` vs `l8,5;l9,4`. Normal finished `4` / `1` / `7` and mixed one-off black spirit rerank `l2,5;l0,6;l1,6` vs `l2,5;l2,3;l1,2`, one-off black `ManaTempo` tie `l2,7;l2,8` vs `l2,7;l1,8`, and one-off white reranks `l8,6;l7,7` vs `l8,6;l8,5` and `l9,5;l7,4;l8,3` vs `l9,5;l7,6;l7,7`. Fast finished `5` / `4` / `3` and mixed one-off white forced-prepass and `ManaTempo` drifts `l8,4;l8,5` vs `l8,4;l9,3`, `l8,4;l8,5` vs `l8,4;l8,3`, and `l9,4;l8,3` vs `l9,4;l8,4`, plus one-off black drainer-safety `l1,6;l0,5` vs `l4,1;l5,0;mb` and one-off black `ManaTempo` tie `l2,7;l2,8` vs `l2,7;l1,8`.
- Durable lesson: do not spend from a replay just because one direct-Pro board surfaces a new bridge-like baseline. If Pro, Normal, and Fast still fracture into unrelated count-`1` seams, keep only the note and wait for one exact family with a retained foothold and a fresh shared lever.

## Apr 9, 2026: Seed v55 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v55` to see whether the new direct-Pro white rerank family around `l10,5;l9,4` could become a real shared story.
- Why it stopped: all three buckets stayed count-`1`. Direct Pro finished `3` regressions / `5` improvements / `4` flat and mixed one-off black spirit sibling `l0,4;l1,3` vs `l0,4;l1,5`, plus one-off white reranks `l10,5;l9,4` vs `l9,2;l8,1` and `l10,6;l9,5` vs `l10,5;l9,4`. Normal finished `4` / `5` / `3` and mixed one-off black spirit rerank `l2,5;l0,6;l1,6` vs `l2,5;l2,3;l1,2`, one-off black `ManaTempo` tie `l2,7;l2,8` vs `l2,7;l1,8`, one-off black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`, and one-off white spirit-own-setup `l9,5;l7,4;l8,3` vs `l9,5;l7,6;l7,7`. Fast finished `4` / `2` / `6` and mixed one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one-off black action+mana `l1,6;l2,7` vs `l2,3;l3,2`, and one-off white forced-prepass drifts `l9,4;l8,5` vs `l8,4;l7,6;l6,6` and `l9,4;l8,5` vs `l9,4;l8,3`.
- Durable lesson: do not spend from a replay just because it surfaces two fresh direct-Pro white reranks at once. If Normal and Fast do not line up behind the same exact family and every move pair stays count-`1`, keep only the note and wait for one exact seam with a fresh shared lever.

## Apr 9, 2026: Seed v56 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v56` to see whether a repeated direct-Pro white pair could finally become a shared duel family worth classification.
- Why it stopped: direct Pro did repeat one exact pair, but the other buckets did not line up. Direct Pro finished `3` regressions / `3` improvements / `6` flat and repeated `l10,5;l9,4` vs current `l9,5;l7,4;l8,3` twice, while also adding one-off black `ManaTempo` tie `l2,7;l2,6` vs `l2,7;l3,8`. Normal finished `2` / `4` / `6` and only showed one-off black spirit rerank `l2,5;l0,6;l1,6` vs `l2,5;l2,3;l1,2` plus one-off white spirit-own-setup `l9,5;l7,4;l8,3` vs `l9,5;l7,6;l7,7`. Fast finished `3` / `0` / `9` and only showed one-off black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`, one-off black `ManaTempo` tie `l2,7;l2,8` vs `l2,7;l1,8`, and one-off white forced-prepass `l8,4;l8,5` vs `l8,4;l8,3`.
- Durable lesson: do not spend from a replay just because direct Pro repeats one white pair twice. If Normal and Fast do not share the same exact seam and every non-Pro pair stays count-`1`, keep only the note and wait for a repeated family with a retained foothold and a fresh shared lever.

## Apr 9, 2026: Seed v57 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v57` to see whether an apparent direct-Pro white spirit-own-setup family would survive a full replay and line up across other buckets.
- Why it stopped: the apparent repeat disappeared by the summaries. Direct Pro finished `2` regressions / `3` improvements / `7` flat and only mixed one-off white reranks `l9,5;l7,4;l7,3` vs current `l9,5;l7,6;l7,7` and `l8,1;l7,0` vs `l9,4;l8,3`. Normal finished `2` / `2` / `8` and only mixed one-off white `l9,6;l8,5` vs `l9,7;l7,6;l7,7` and one-off black `l3,2;l3,4;l2,3` vs `l3,2;l4,1`. Fast finished clean at `0` / `2` / `10`. Every exact move pair across all three buckets stayed at count `1`.
- Durable lesson: do not spend from a replay just because the first direct-Pro trace looks like a repeated white family. If the final summaries collapse back to count-`1` seams and Fast goes clean, keep only the note and wait for one exact family that survives the whole sample.

## Apr 9, 2026: Seed v58 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v58` to see whether a replay that repeated retained early-black `negative_deny` in direct Pro would finally line up behind one promotable cross-bucket seam.
- Why it stopped: the seed still split across already-known families instead of converging. Direct Pro finished `3` regressions / `5` improvements / `4` flat and did repeat the retained early-black `negative_deny` seam `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` twice, but its third loss was a separate one-off late white spirit rerank `l4,9;l2,7;l2,8` vs `l4,9;l2,7;l3,6`. Normal finished `5` / `2` / `5` and stayed pure count-`1` churn across white `l9,5;l8,5` vs `l9,5;l7,6;l7,7`, white forced-prepass `l9,4;l8,5` vs `l9,4;l8,3`, white spirit-own-setup `l9,5;l7,4;l8,3` vs `l9,5;l7,6;l7,7`, white `ManaTempo` tie `l8,5;l8,4` vs `l8,5;l9,4`, and white `l9,5;l8,4` vs `l9,6;l7,7;l7,8`. Fast finished `4` / `3` / `5` and repeated the already-killed black spirit-bridge seam `l1,5;l1,7;l0,7` vs current `l4,1;l5,0;mb` twice, while also adding one-off white forced-prepass `l8,4;l8,5` vs `l8,4;l8,3` and one-off white rerank `l10,4;l9,3` vs `l9,5;l7,6;l8,7`.
- Durable lesson: do not spend from a replay just because direct Pro repeats retained `negative_deny` and Fast repeats the old black spirit-bridge seam in the same sample. If Normal still fractures entirely into count-`1` white reranks and ties, keep only the note and wait for one exact family that actually carries through the losing buckets.

## Apr 9, 2026: Seed v59 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v59` to see whether a quieter replay would finally turn the retained early-black seam into a real cross-bucket promotion story.
- Why it stopped: the seed never converged on one shared family. Direct Pro finished `2` regressions / `2` improvements / `8` flat and only mixed one-off retained early-black `negative_deny` `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` with one-off white forced-prepass `l8,4;l8,5` vs `l8,4;l9,3`. Normal finished `1` / `1` / `10` and only added one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`. Fast finished `5` / `3` / `4` and fractured across one-off white `l10,4;l9,3` vs `l9,5;l7,6;l8,7`, one-off black `l2,7;l1,8` vs `l2,3;l3,2`, one-off white forced-prepass `l8,4;l8,5` vs `l8,4;l9,3`, one-off white forced-prepass `l9,4;l8,5` vs `l9,4;l8,3`, and one-off white safe-progress `l9,5;l8,6` vs `l10,6;l9,6`. Every exact move pair across all three buckets stayed at count `1`.
- Durable lesson: do not spend from a replay just because direct Pro and Normal each show one retained black seam while staying relatively quiet overall. If Fast still breaks apart into unrelated white forced-prepass, white safe-progress, and black `ManaTempo` one-offs, keep only the note and wait for one exact family that actually survives the whole sample.

## Apr 9, 2026: Seed v60 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v60` to see whether a quieter Fast bucket would finally let one retained black seam or the new direct-Pro white rerank carry the sample.
- Why it stopped: the seed still fractured into count-`1` seams. Direct Pro finished `3` regressions / `3` improvements / `6` flat and split across one-off white `l8,4;l9,3` vs current `l8,5;l6,7;l7,7`, one-off retained early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7`, and one-off black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`. Normal finished `3` / `2` / `7` and split across one-off black action+mana `l1,6;l2,7` vs `l3,2;l4,1`, one-off white forced-prepass `l9,4;l8,5` vs `l9,4;l8,3`, and one-off black `ManaTempo` rerank `l0,10;l0,9` vs `l1,5;l2,5`. Fast finished `1` / `6` / `5` and only added one-off black action+mana `l1,6;l2,7` vs `l2,3;l3,2`. Every exact move pair across all three buckets stayed at count `1`.
- Durable lesson: do not spend from a replay just because Fast nearly goes clean while Pro and Normal surface familiar retained black seams. If the surviving losses still split across unrelated one-off white reranks, black bridge/action+mana seams, and local `ManaTempo` reranks, keep only the note and wait for one exact family that actually carries the whole sample.

## Apr 9, 2026: Seed v61 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v61` to see whether a cleaner replay of retained early-black `negative_deny` would finally carry through Normal and Fast.
- Why it stopped: only direct Pro repeated that seam, and the other buckets still fractured. Direct Pro finished `2` regressions / `5` improvements / `5` flat and repeated retained early-black `negative_deny` `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` twice. Normal finished `2` / `2` / `8` and only added one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb` plus one-off white `l9,5;l8,5` vs `l9,5;l7,6;l7,7`. Fast finished `4` / `1` / `7` and fractured across one-off white forced-prepass `l9,4;l8,5` vs `l9,4;l8,3`, one-off white forced-prepass `l8,4;l8,5` vs `l8,4;l9,3`, one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, and one-off black spirit variant `l1,5;l0,7;l1,7` vs `l3,2;l4,1`.
- Durable lesson: do not spend from a replay just because direct Pro repeats retained early-black `negative_deny` twice again. If Normal and Fast still split across separate white forced-prepass and bridge-like one-offs, keep only the note and wait for one exact family that actually survives the whole sample.

## Apr 9, 2026: Seed v62 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v62` to see whether a quieter direct-Pro bucket would turn the new black `ManaTempo` sibling tie into a real family or expose one repeatable Normal/Fast follow-up.
- Why it stopped: the seed stayed count-`1` everywhere. Direct Pro finished `1` regression / `2` improvements / `9` flat, with its only loss on one-off black `ManaTempo` sibling `l2,3;l2,2` vs current `l2,3;l1,2`. Normal finished `3` / `0` / `9` and split across one-off white `l8,5;l8,6` vs `l10,6;l9,6`, one-off black action+mana `l1,6;l2,7` vs `l3,2;l4,1`, and one-off white spirit-own-setup `l9,5;l7,4;l7,3` vs `l9,5;l7,6;l7,7`. Fast finished `1` / `3` / `8` and only added one-off bridge-adjacent black spirit variant `l1,5;l1,7;l2,6` vs `l5,0;l4,1`.
- Durable lesson: do not spend from a replay just because direct Pro nearly goes clean and Fast stays relatively quiet. If every exact family remains count-`1`, keep only the note and wait for one exact seam that actually survives the whole sample.

## Apr 9, 2026: Seed v63 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v63` to see whether the quieter direct-Pro bucket or the resurfaced Normal bridge seam would finally expose one cross-bucket exact family worth code.
- Why it stopped: the seed still fractured. Direct Pro finished `2` regressions / `3` improvements / `7` flat and only added one-off black `l1,5;l3,4;l2,3` vs current `l1,5;l3,6;l2,7` plus one-off late white safe-progress `l10,5;l9,6` vs `l10,6;l9,5`. Normal finished `5` / `1` / `6` and repeated only the already-killed black spirit-bridge seam `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb` twice, while also adding one-off white `l8,4;l9,3` vs `l5,9;l7,7;l8,8`, one-off black spirit sibling `l0,4;l1,4` vs `l0,4;l1,5`, and one-off white forced-prepass `l9,4;l8,5` vs `l9,4;l8,3`. Fast finished `3` / `5` / `4` and split across one-off white `l10,4;l9,3` vs `l9,5;l7,6;l8,7`, one-off white forced-prepass `l8,6;l7,5` vs `l8,6;l7,7`, and one-off black accepted `ManaTempo` rerank `l1,5;l1,4` vs `l3,2;l4,1`.
- Durable lesson: do not spend from a replay just because Normal repeats the old black spirit-bridge seam twice again. If direct Pro and Fast still break into separate one-off white and black reranks, keep only the note and wait for one exact family that actually survives the whole sample.

## Apr 9, 2026: Seed v64 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v64` to see whether a near-clean direct-Pro bucket would finally line up with a single quieter Normal/Fast seam.
- Why it stopped: the seed stayed count-`1` everywhere. Direct Pro finished `1` regression / `4` improvements / `7` flat and only had one-off black drainer-safety `l0,5;l1,4` vs current `l2,8;l3,8`. Normal finished `2` / `2` / `8` and split across one-off white spirit-own-setup `l9,5;l7,4;l8,3` vs `l9,5;l7,6;l7,7` and one-off black spirit rerank `l2,5;l0,6;l1,6` vs `l2,5;l2,3;l1,2`. Fast finished `1` / `3` / `8` and only added one-off black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`. Every exact move pair across all three buckets stayed at count `1`.
- Durable lesson: do not spend from a replay just because direct Pro nearly goes clean again. If Normal and Fast still contribute only separate one-off seams, keep only the note and wait for one exact family that actually survives the whole sample.

## Apr 9, 2026: Seed v65 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v65` to see whether a cleaner `ManaTempo`-shaped replay would finally collapse onto one exact family across the three duel buckets.
- Why it stopped: the seed stayed count-`1` everywhere. Direct Pro finished `1` regression / `3` improvements / `8` flat and only had one-off black `ManaTempo` tie `l1,2;l1,1` vs current `l1,2;l0,1`. Normal finished `2` / `3` / `7` and split across one-off black accepted `ManaTempo` rerank `l0,9;l0,10` vs `l2,5;l4,3;l3,3` and one-off white `ManaTempo` tie `l8,3;l8,2` vs `l8,3;l9,2`. Fast finished `3` / `4` / `5` and split across one-off black `ManaTempo` tie `l1,8;l1,9` vs `l1,8;l0,9`, one-off white `ManaTempo` tie `l8,5;l7,4` vs `l8,5;l9,4`, and one-off black bridge-adjacent drainer-safety `l0,5;l1,4` vs `l4,1;l5,0;mb`.
- Durable lesson: do not spend from a replay just because the surviving losses all look like tidy `ManaTempo` ties and reranks. If every exact pair still stays at count `1`, keep only the note and wait for one exact family that actually survives the whole sample.

## Apr 9, 2026: Seed v66 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v66` to see whether the repeated Normal white spirit-own-setup rerank would finally become a cross-bucket family worth code.
- Why it stopped: the seed still fractured. Direct Pro finished `1` regression / `4` improvements / `7` flat and only had one-off black accepted spirit `l1,4;l0,6;l1,6` vs current `l5,10;l6,9`. Normal finished `4` / `1` / `7` and did repeat `l9,5;l7,4;l8,3` vs `l9,5;l7,6;l7,7` twice, but also added one-off white `ManaTempo` tie `l9,2;l9,1` vs `l9,2;l10,1` and one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`. Fast finished `3` / `2` / `7` and split across one-off black mana-bridge `l0,5;l1,4` vs `l4,1;l5,0;mb`, one-off white forced-prepass `l7,4;l8,5` vs `l7,4;l8,3`, and one-off black `ManaTempo` rerank `l1,5;l2,5` vs `l1,6;l0,6`.
- Durable lesson: do not spend from a replay just because Normal repeats the white spirit-own-setup rerank twice. If direct Pro and Fast still split across unrelated one-off black and white seams, keep only the note and wait for one exact family that actually survives the whole sample.

## Apr 9, 2026: Seed v67 Replay Killed Before Code Edits

- What was tried: refreshed `smart_automove_pro_reliability_duel_trace_probe` with `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_v67` to see whether the next quiet direct-Pro sample would finally turn the old Normal white `l6,5;l6,6` tie or the latest black one-offs into one cross-bucket family worth code.
- Why it stopped: the seed still fractured into count-`1` seams. Direct Pro finished `1` regression / `4` improvements / `7` flat and only had one-off white `l9,6;l8,5` vs current `l9,6;l8,7`. Normal finished `2` / `1` / `9` and only mixed one-off early-black `negative_deny` `l0,5;l1,6` vs `l1,5;l3,6;l2,7` with one return of the already-killed white `ManaTempo` tie `l6,5;l6,6` vs `l6,5;l7,4`. Fast finished `5` / `2` / `5` and fractured across one-off black spirit-bridge `l1,5;l1,7;l0,7` vs `l4,1;l5,0;mb`, one-off white forced-prepass `l8,4;l8,5` vs `l8,4;l9,3`, one-off black drainer-safety `l1,4;l2,3` vs `l1,6;l1,5`, one-off black `ManaTempo` tie `l2,7;l2,8` vs `l7,1;l6,1`, and one-off black spirit rerank `l2,4;l0,6;l1,6` vs `l1,7;l2,6`. Every exact move pair across all three buckets stayed at count `1`.
- Durable lesson: do not spend from a replay just because direct Pro stays quiet and the old Normal white `l6,5;l6,6` tie returns. If Fast still breaks into unrelated bridge, forced-prepass, drainer-safety, and local `ManaTempo` one-offs, keep only the note and wait for one exact family that actually survives the whole sample.

## Apr 9, 2026: Black Risky-Window Inject Block Killed After Reliability

- What was tried: added a narrow inject-time cut in `runtime_pro_turn_engine_v30` to reject absent forced `ManaTempo` same-turn-window roots under `Safe*Progress -> ImmediateScore` when they only replaced an equally unsafe non-progress `ManaTempo` top with a weaker setup surface. The target seam was retained `primary_black_turn_four_action_mana_ply15`.
- Why it stopped: the change only closed that one retained later-black seam. It passed the focused regression test, `guardrails`, moved `pro-triage(primary_pro)` to `4/60` with `off_target_changed=0`, and passed `runtime-preflight`, but `pro-reliability` got worse than the clean challenger at `0.7500` vs current Pro, `0.5000` vs current Normal, and `0.5833` vs current Fast.
- Durable lesson: do not keep a narrow absent-forced black risky-window inject block just because it collapses `primary_black_turn_four_action_mana_ply15`. If the other retained wall stays on the white risky rerank, both bridge seams, and `human_win_pro_c`, the action+mana seam alone is still too local and can degrade direct duel evidence.

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
