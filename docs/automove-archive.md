# Automove Archive

This file keeps only short summaries of retired automove waves.

Everything here is archive-only context. Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the live workflow and `docs/automove-knowledge.md` for durable rules that still matter.

## Reference Frontier Wave

- Early retained turn-engine work established the shared infrastructure that later made guarded `ProV2` possible.
- `runtime_pro_turn_engine_v1` belongs to this wave. It is archive-only reference history now, not an active experiment target.

## Replay-Diary Wave

- Many Apr 8-9 duel replays (`v7` through `v75`) were useful for classification but not for direct code spend.
- The common stop reason was the same: exact move pairs stayed count `1`, or the repeated pair had no retained Pro foothold.
- Durable outcome: keep the retained probes, compress the diary, and let git history hold the seed-by-seed detail.

## Wrapper And Fallback Wave

- Several opening-book, early-white, and forced-prepass wrapper cuts were tried after promotion work stalled.
- Some narrow guards survived as part of the shipping guarded path.
- The broader lesson from this wave is negative: wrapper-only repairs were rarely promotable on their own and often failed to move the direct `vs shipping Normal` wall.

## Retained Seam-Mapping Wave

- This wave produced the durable retained fixtures that future work still uses: black action+mana, black mana bridge, black spirit bridge, negative-deny, white safe-progress, and the closed regression seams.
- Most production cuts from this wave were killed because they solved only one local family.
- Durable outcome: keep the fixtures and the probes, not the abandoned production branches.

## Unified Root-Advisor Promotion Wave

- The winning structural change was the unified `ProV2` root advisor that centralized shortlist shaping, family preservation, omitted-root handling, macro-root injection, and conservative post-search verification.
- The final promotable cut was narrow: on quiet early-black boards, advisor approval had to stop preferring a weaker plain-spirit sibling over a stronger own-setup `SpiritImpact` root already in the shortlist.
- Durable outcome: `frontier_pro_v2_guarded` survived as the retained guarded frontier, but shipping stayed on the separate search-only path.

## Pro-Only Surface Cleanup Wave

- After promotion, the active experiment surface was shrunk to two selectable profiles: `shipping_pro_search` and `frontier_pro_v2_guarded`.
- Calibration/reference profiles, curated-pool smoke plumbing, and compatibility-only docs were archived.
- Legacy flat experiment logs and the old `target/experiment-runs/runtime_preflight_*.stamp` compatibility path were removed.
- Durable outcome: future work starts from a smaller Pro-only workflow; archived profiles and stages stay documented here, not in the live runbook.

## Closed-Surface Archive Cleanup Wave

- On `2026-04-21`, the archived regression seams `primary_spirit_setup`, `primary_pvs_sensitive_search`, and `primary_black_reliability_opening_3_ply4` were removed from the live `primary_pro` pack and from the default retained probes.
- Their history stays here only. They are no longer part of the live retained experiment surface or the default operator diagnostics.
- Durable outcome: `primary_pro` now means current live retained seams only, and closed-surface history no longer leaks into the active workflow.

## Quiet-Guarded Challenger Wave

- `frontier_pro_v3_quiet_guarded` tried to spend on live non-win seams around quiet mana acceptance and vulnerable plain-spirit reentry.
- The cut only passed `pro-triage` after 2 duel-derived live non-win boards were promoted into retained `primary_pro`.
- Direct evidence killed it: `pro-reliability` vs shipped `frontier_pro_v2_guarded` came back `0.3333 / 0.8333 / 0.8333`, so the candidate code was discarded.
- Durable outcome: keep the new retained seams and the live non-win root probe, but require direct frontier-vs-shipping wins before reopening this hypothesis family.

## White-Guarded Challenger Wave

- `frontier_pro_v3_white_guarded` spent on three white-only live seams: late quiet head acceptance, safe mana sibling selection on the exact split trace, and turn-3 vulnerable-window recovery.
- The cut really did fix the first two probe boards: `vs_shipping_pro_opening_reply_white` and `vs_shipping_pro_white_split_trace` matched shipping after the local guards landed.
- It was still not promotable because `vs_shipping_normal_white_head_acceptance` never left `search_only_engine_allowed_head`; shipping still reached `search_only_forced_prepass` on the same board.
- Durable outcome: keep the probe improvements and the lesson that the unresolved turn-3 search-only handoff is the real remaining seam. Discard the candidate code.

## Live-Seam Override Wave

- `frontier_pro_v3_live_seam_override` explicitly aligned the four known live seam boards to shipping behavior while keeping the white turn-3 vulnerable-window recovery.
- The cut did what it was supposed to do locally: retained `primary_pro` moved cleanly by `2 / 62` with `off_target_changed=0`, and `runtime-preflight` passed.
- Direct evidence still killed it: `pro-reliability` vs shipped `frontier_pro_v2_guarded` only reached `0.5000 / 0.7500 / 0.8333`, so even exact seam coverage was nowhere near promotable.
- Durable outcome: treat exact live-seam alignment as a dead end for Pro promotion. Keep the knowledge, discard the candidate code.

## Quiet-Score-Guarded Wave

- `frontier_pro_v3_quiet_score_guarded` tried a candidate-only quiet lower-scored root guard aimed at live non-win mana-head acceptance.
- The cut really did move the retained surface: `primary_pro` changed by `5 / 62` with `off_target_changed=0`, `guardrails` passed, and `runtime-preflight` passed.
- It fixed `vs_shipping_pro_opening_reply_white`, but the other live probe walls still stood: `vs_shipping_pro_black_recovery_branch`, `vs_shipping_pro_white_split_trace`, `vs_shipping_normal_black_bridge_nonwin`, and `vs_shipping_normal_white_head_acceptance`.
- Direct evidence still killed it: `pro-reliability` vs shipped `frontier_pro_v2_guarded` only reached `0.5833 / 0.7500 / 0.9167`, so the candidate code was discarded.
- Durable outcome: quiet-score suppression alone is not a promotable Pro frontier. Keep the lesson, discard the candidate code.

## Progress-Rescue Probe Wave

- `frontier_pro_v3_progress_rescue_guarded` first turned on the dormant mid-turn white progress guard and late-black setup-progress rescue, then added a candidate-only unsafe plain-spirit floor guard.
- The probe-only result was negative: the live non-win root probe remained unchanged on `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, `vs_shipping_pro_white_split_trace`, `vs_shipping_normal_black_bridge_nonwin`, and `vs_shipping_normal_white_head_acceptance`.
- Because the candidate never changed the intended live walls, it never earned `guardrails`, `pro-triage`, or `runtime-preflight`.
- Durable outcome: when the live non-win root probe does not move, kill the line immediately and keep the codebase clean.

## Forced-Prepass Priority Wave

- `frontier_pro_v3_forced_prepass_priority` tried to prioritize `forced_tactical_prepass` ahead of search-only head acceptance and was explicitly threaded through the white scoring-window fallback.
- The probe-only result was still negative: `vs_shipping_normal_white_head_acceptance` stayed on `search_only_engine_allowed_head`, and the other live probe walls also stayed unchanged.
- Because the candidate never changed the intended live walls, it never earned `guardrails`, `pro-triage`, or `runtime-preflight`.
- Durable outcome: if the exact search-only handoff stage does not move, the missing spend is deeper than prepass ordering. Kill the line and keep the codebase clean.

## White Reply-Head Guarded Wave

- `frontier_pro_v3_white_reply_head_guarded` tried a candidate-only white vulnerable-window head reject plus a quiet-mana reply-score guard, and the candidate config was explicitly threaded through the white scoring-window fallback.
- The probe-only result was still negative on the targeted white walls: `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_white_split_trace`, and `vs_shipping_normal_white_head_acceptance` all kept the same selected roots as shipping misses.
- The only visible movement was metadata-level: `vs_shipping_pro_white_split_trace` changed the approved reason label to `ApprovedFamilyCompetition` without changing the selected root.
- Because the candidate never changed the intended live walls, it never earned `guardrails`, `pro-triage`, or `runtime-preflight`.
- Durable outcome: if a white candidate only changes advisor reason labels or leaves `search_only_engine_allowed_head` intact, the real spend is deeper than generic quiet-mana score guards. Kill the line and keep the codebase clean.

## White Presearch-Reentry Guarded Wave

- `frontier_pro_v3_white_presearch_reentry_guarded` tried three white-only spends together: a vulnerable-window presearch approval path, a late quiet-mana head reject, and a stricter white mana-sibling same-lane gap.
- The probe result was mixed but still not promotable. It did fix `vs_shipping_pro_white_split_trace`, moving the selected root from `l8,0;l7,1` to shipping `l10,8;l9,7`.
- The other white walls did not move: `vs_shipping_pro_opening_reply_white` still kept `l10,10;l10,9`, and `vs_shipping_normal_white_head_acceptance` still stayed on `search_only_engine_allowed_head` instead of shipping `search_only_forced_prepass`.
- Because the candidate only repaired one white seam and left the opening-reply plus search-only handoff walls intact, it never earned `guardrails`, `pro-triage`, or `runtime-preflight`.
- Durable outcome: `white_split_trace` is a real white sibling reentry seam, but fixing it alone is not enough. Keep the lesson, discard the candidate code, and keep the worktree clean.

## White Head And Search-Only Guarded Wave

- `frontier_pro_v3_white_head_search_only_guarded` tried three narrow spends together: a late-white low-budget selector exception, a late quiet-mana head reject, and a search-only white vulnerable-window top-head conflict.
- The probe-only result was fully negative. `vs_shipping_pro_opening_reply_white` still stayed `engine_disabled`, `vs_shipping_pro_white_split_trace` still kept `l8,0;l7,1`, and `vs_shipping_normal_white_head_acceptance` still stayed on `search_only_engine_allowed_head`.
- Because the candidate never changed the intended white walls, it never earned `guardrails`, `pro-triage`, or `runtime-preflight`.
- Durable outcome: the remaining white spend is deeper than the guessed low-budget selector gate or a simple search-only top-head conflict. Kill the line and keep the codebase clean.

## Selector-PreDisabled Probe Wave

- `frontier_pro_v3_selector_predisabled_probe` did not cut a new challenger. The spend for this wave was diagnostic-only: the retained live non-win probe now records the actual frontier wrapper branch and the selector disable reason.
- That first reading was later found to be contaminated by an unconditional extra shipping-search fallback on non-black boards. The corrected probe residue from the next wave kept the useful top-level selector fields, but the specific `frontier_execute + pre_disabled` conclusion from this entry should no longer be treated as ground truth on the white boards.
- Durable outcome: keep the improved probe instrumentation, but do not reopen the old `pre_disabled` interpretation without first verifying that no extra fallback search is overwriting the top-level selector diagnostics.

## Advisor-Window Guarded Wave

- `frontier_pro_v3_advisor_window_guarded` was cut after correcting the live probe contamination from the late-black shipping fallback. Once the probe was truthful, the two active white walls split cleanly: `opening_reply_white` was a post-search head-over-advisor seam and `normal_white_head_acceptance` was an early-white vulnerable-window recovery miss.
- The candidate fixed both of those walls together. `opening_reply_white` stayed on the advisor-approved `l9,5;l8,6`, and `normal_white_head_acceptance` stayed on the safe recovery root `l9,4;l8,5` with the risky window head rejected.
- It earned the smaller gates: `guardrails` passed, retained `primary_pro` moved by `5 / 62` with `off_target_changed=0`, exact-lite passed, and stage-1 CPU stayed advisory-only even though the Pro ratios drifted to about `1.65x`, `1.70x`, and `1.90x`.
- Direct retained evidence still killed it. `pro-reliability` vs `shipping_pro_search` failed uniformly at `0.6667 / 0.6667 / 0.6667` with confidence `0.8062 / 0.8062 / 0.8062`, so the candidate code was discarded.
- Durable outcome: even fixing both corrected white live walls and moving retained `primary_pro` cleanly is still not enough. Keep the corrected probe residue and the lesson; discard the candidate code.

## Reply-Risk Injection Guarded Wave

- `frontier_pro_v3_reply_risk_injection_guarded` widened reply-risk shortlist coverage, enabled lazy score-window projection, and allowed two injected roots under the existing Pro V2 selector path.
- The probe result was negative on every real live wall. `opening_reply_white` still accepted the same head over the advisor-approved mana continuation, `black_recovery_branch` still approved the preserved spirit reentry even after the shipping `l6,0;l6,1` mana root entered the reply-risk shortlist, `normal_black_bridge_nonwin` still stayed on the spirit-impact root, and `normal_white_head_acceptance` still stayed on the risky vulnerable-window root.
- Root injection was not the missing mechanism on those boards: `injected_root` stayed `None` through the live probe, so the extra root budget did not translate into a changed approved root.
- The only visible movement was diagnostic-only and not promotable. A white plain-spirit split board changed root ordering, but the real live non-win walls stayed unchanged.
- Durable outcome: shortlist width, lazy score-window projection, and small root injection are not the bottleneck. If the black recovery fallback can already appear inside the shortlist and still not win approval, the next spend has to land inside approval or head logic rather than coverage.

## Approval-Escape Guarded Wave

- `frontier_pro_v3_approval_escape_guarded` turned on lazy score-window projection and spent on candidate-only approval escapes in white followup-mana competition, white mana sibling competition, black legacy alignment, a turn-3 white recovery override, and a late-white head reject.
- The cut did move two real seams. `vs_shipping_pro_white_split_trace` finally approved shipping `l10,8;l9,7`, and `vs_shipping_normal_black_bridge_nonwin` moved off the spirit-own-mana setup onto shipping `l6,1;l5,0;mb`.
- It was still not promotable because the remaining blockers stayed unchanged. `vs_shipping_pro_opening_reply_white` still accepted `l10,10;l10,9` over the advisor-approved `l9,5;l8,6`, `vs_shipping_pro_black_recovery_branch` still approved the preserved spirit reentry instead of shortlist legacy mana `l6,0;l6,1`, and `vs_shipping_normal_white_head_acceptance` still stayed on the vulnerable window root `l9,4;l8,3` instead of the safe recovery root.
- Because those surviving live walls never moved together, the candidate did not earn `guardrails`, `pro-triage`, or `runtime-preflight`. The code was discarded and only the lesson was kept.

## Reply-Risk Reentry Guarded Wave

- `frontier_pro_v3_reply_risk_reentry_guarded` enabled lazy score-window projection, widened the late-white post-search reject so it could also block vulnerable heads over safe-recovery preaccept roots, and relaxed the black vulnerable-spirit escape so vulnerable mana challengers could win approval.
- The white result was still negative. `vs_shipping_pro_opening_reply_white` stayed on `l10,10;l10,9`, and `vs_shipping_normal_white_head_acceptance` still finished on vulnerable `l9,4;l8,3` even though advisor approval had already moved to safe recovery `l9,4;l8,5`.
- The black result was worse, not better. `vs_shipping_pro_black_recovery_branch` flipped onto legacy mana `l6,0;l6,1` while shipping still stayed on spirit `l1,5;l3,3;l2,3`, so removing the safety requirement overcorrected the wrong wall.
- Because the surviving white walls did not move and the black recovery wall moved away from shipping, the candidate never earned `guardrails`, `pro-triage`, or `runtime-preflight`. The code was discarded and only the lesson was kept.

## Safe-Progress Head-Guarded Wave

- `frontier_pro_v3_safe_progress_head_guarded` added family-specific white safe-progress head rejects plus a turn-3 vulnerable-window recovery override, and it was cut only after the live probe gained `head_family` and `goal_family` output.
- The probe confirmed the targeted white walls precisely. `vs_shipping_pro_opening_reply_white` is a `SafeSupermanaProgress -> DrainerSafetyRecovery` post-search head-over-advisor seam, and `vs_shipping_normal_white_head_acceptance` is a `SafeSupermanaProgress -> ImmediateScore` vulnerable-window head-over-recovery seam.
- The candidate did move the intended walls. It fixed both white seams, kept `vs_shipping_pro_black_recovery_branch` aligned with shipping spirit `l1,5;l3,3;l2,3`, passed `smart_automove_tactical_selected_profile`, moved retained `primary_pro` by `5 / 62` with `off_target_changed=0`, and passed exact-lite.
- Retained duel strength still killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.8333 / 0.7500 / 0.9167`, so the candidate code was discarded.
- Durable outcome: even precise `SafeSupermanaProgress` family-specific white head guards plus the turn-3 recovery override are still not promotable. Keep the probe-family diagnostics and the lesson; do not reopen the candidate code.

## Live Nonwin Family Guarded Wave

- `frontier_pro_v3_live_nonwin_family_guarded` extended the family-aware white package with a tighter black turn-6 spirit-reentry filter aimed at the retained vulnerable-spirit seam.
- The candidate did move the intended live walls together. It fixed `vs_shipping_pro_opening_reply_white`, `vs_shipping_normal_white_head_acceptance`, and `vs_shipping_pro_black_recovery_branch`, passed `smart_automove_tactical_selected_profile`, moved retained `primary_pro` by `4 / 62` with `off_target_changed=0`, and passed exact-lite.
- Runtime cost was still weak: `smart_automove_pool_stage1_cpu_non_regression_gate` only cleared in advisory mode at `1.502x`, `1.548x`, and `1.608x` vs `shipping_pro_search`.
- Retained duel strength still killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.8333 / 0.7500 / 0.7500` with confidence `0.9807 / 0.9270 / 0.9270`, so the candidate code was discarded.
- Durable outcome: even fixing both white live walls and the black spirit-reentry wall together is still not enough. Keep the lesson; do not reopen the candidate code.

## White Window Recovery Guarded Wave

- `frontier_pro_v3_white_window_recovery_guarded` tried a narrower white-only spend: a turn-3 no-action vulnerable-window recovery redirect plus a late white weak-window recovery override on action+mana boards.
- The candidate did move the vulnerable-window seam at the advisor layer. On `vs_shipping_normal_white_head_acceptance`, `pre_accept_input_fen` and advisor approval changed from vulnerable `l9,4;l8,3` to safe `DrainerSafetyRecovery l9,4;l8,5`.
- That movement never reached the actual frontier output. Final selected roots on all live walls stayed unchanged against active `frontier_pro_v2_guarded`, because post-search head acceptance still snapped `vs_shipping_normal_white_head_acceptance` back to vulnerable `l9,4;l8,3`.
- Direct challenger evidence killed it immediately: retained `pro-triage` vs active `frontier_pro_v2_guarded` returned `target_changed=0 off_target_changed=0`, so the line was behaviorally inert and never earned `runtime-preflight` or retained reliability.
- Durable outcome: approval-only white recovery is not enough if the final head step still wins. Do not spend canonical gates on candidates that only improve advisor or `pre_accept` metadata.

## Reply-Order Guarded Wave

- `frontier_pro_v3_reply_order_guarded` tried two shared comparator changes together: a stricter risky-recovery progress sibling override and a bounded late-black vulnerable non-spirit followup escape.
- The line stayed fully inert. The live non-win probe left `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, and `vs_shipping_normal_white_head_acceptance` unchanged, and the retained live seams `primary_white_safe_progress_rerank_ply27` plus `primary_live_nonwin_black_vulnerable_spirit_reentry` also stayed unchanged.
- Direct challenger evidence killed it immediately: retained `pro-triage` vs active `frontier_pro_v2_guarded` returned `target_changed=0 off_target_changed=0`, so the line never earned `runtime-preflight` or retained reliability.
- Durable outcome: tightening those shared reply-order thresholds alone is not the missing spend. Keep the lesson, discard the candidate code, and keep the worktree clean.

## Family-Competition Guarded Wave

- `frontier_pro_v3_family_competition_guarded` paired a tighter black turn-6 spirit-reentry filter with a tighter white turn-3 mana sibling competition and a candidate-only turn-3 white recovery override.
- The package did move two real live seams together. `vs_shipping_pro_black_recovery_branch` aligned to shipping `l6,0;l6,1`, and `vs_shipping_pro_white_split_trace` aligned to shipping `l10,8;l9,7`.
- The surviving white seams still blocked promotion. `vs_shipping_pro_opening_reply_white` stayed on `l10,10;l10,9`, and `vs_shipping_normal_white_head_acceptance` again only moved at the advisor layer: `pre_accept_input_fen` changed to safe `DrainerSafetyRecovery l9,4;l8,5`, but the final selected root still snapped back to vulnerable `l9,4;l8,3`.
- Because one surviving wall stayed completely unchanged and the other still failed at final head acceptance, the line never earned `pro-triage`, `runtime-preflight`, or retained reliability. The code was discarded and only the lesson was kept.

## Live Wall Combo Guarded Wave

- `frontier_pro_v3_live_wall_combo_guarded` combined a late-white quiet head reject, a turn-3 white weak-window recovery redirect, a tighter black turn-6 spirit-reentry filter, and a safer white split-trace mana competition.
- The package did align all four active live walls together. `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, `vs_shipping_pro_white_split_trace`, and `vs_shipping_normal_white_head_acceptance` all moved onto the intended shipping roots in the live probe.
- The smaller gates also stayed clean. `smart_automove_tactical_selected_profile` passed, exact-lite passed, and retained `primary_pro` triage stayed at `target_changed=2 / off_target_changed=0`.
- Canonical cost killed it immediately anyway. Against `shipping_pro_search`, `smart_automove_pool_pro_reliability_gate` failed on `stage1_cpu_v1` at `1.687 / 1.696 / 1.732`, with median ratio `1.696x` versus the `1.300x` limit, so the candidate code was discarded.
- Durable outcome: even perfect live-wall alignment plus clean retained triage is not promotion evidence if canonical CPU cost regresses this hard.

## Retained Surface Guarded Wave

- `frontier_pro_v3_retained_surface_guarded` combined a late-white quiet mana head reject, a turn-3 white vulnerable-window recovery override, and a black vulnerable plain-spirit reentry override.
- The package did move the intended retained live seams. It fixed `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, and `vs_shipping_normal_white_head_acceptance`, while retained `primary_pro` triage stayed clean at `target_changed=2 / off_target_changed=0`.
- The cheap gates also stayed clean. `smart_automove_tactical_selected_profile` passed, exact-lite passed, and no off-target retained churn appeared.
- Canonical cost still killed it immediately. `smart_automove_pool_stage1_cpu_non_regression_gate` only cleared in advisory mode at `1.617 / 1.763 / 1.624`, and retained `smart_automove_pool_pro_reliability_gate` died on its embedded `stage1_cpu` precheck at `1.611855221929612 / 1.621475467583131 / 1.6299568403679077`, with median ratio `1.621x` against the `1.300x` limit.
- Durable outcome: even a narrower retained-surface package that fixes three real live walls and keeps retained churn clean is still not promotable if runtime cost regresses back into the `1.6x+` range. Candidate code should be discarded and only the lesson kept.

## Opening Reentry Guarded Wave

- `frontier_pro_v3_opening_reentry_guarded` kept only the two retained live-seam spends from the broader retained-surface package: a late-white quiet mana head reject and a black vulnerable plain-spirit reentry override.
- The package moved the intended retained seams and nothing broader. It fixed `vs_shipping_pro_opening_reply_white` and `vs_shipping_pro_black_recovery_branch`, intentionally left `vs_shipping_normal_white_head_acceptance` unchanged, and retained `primary_pro` triage stayed clean at `target_changed=2 / off_target_changed=0`.
- The cheap gates still stayed clean. `smart_automove_tactical_selected_profile` passed, exact-lite passed, and no off-target retained churn appeared.
- Canonical cost still killed it immediately. `smart_automove_pool_stage1_cpu_non_regression_gate` only cleared in advisory mode at `1.586 / 1.619 / 1.625`, and retained `smart_automove_pool_pro_reliability_gate` died on its embedded `stage1_cpu` precheck at `1.5837620164231196 / 1.5857045402338734 / 1.6051744579914184`, with median ratio `1.586x` against the `1.300x` limit.
- Durable outcome: removing the turn-3 white vulnerable-window recovery override did not fix the runtime-cost regression. The expensive part is at least the late-white opening head reject plus black reentry combo, so candidate code should be discarded and only the lesson kept.

## Retained Gate Alignment Wave

- No new frontier challenger was cut from this wave. The useful code change landed in the retained harness instead: frontier Pro stage-1 CPU is now advisory by default, matching the runbook instead of requiring an explicit env override.
- That harness correction exposed the deeper blocker immediately. On the default retained `pro_turn_planner_reliability_v1` corpus, shipped `frontier_pro_v2_guarded` itself now reaches the duel stage and fails retained `pro-reliability` at `0.7500 / 0.8333 / 1.0000` with confidence `0.9270 / 0.9807 / 0.9998`.
- Durable outcome at that point: keep the harness fix, but the exact retained duel surface still needed to be traced before cutting another challenger.

## White Mid-Turn Recovery Broadening Wave

- This wave tried to spend directly on the remaining white Fast search-only split `l9,4;l8,3` vs shipping `l9,4;l8,5`.
- The runtime cut widened `pro_v2_root_advisor_white_turn_three_no_action_recovery_override` from `mons_moves_count == 0` to `<= 1` and paired it with a post-search head reject for same-lane vulnerable `ManaTempo -> DrainerSafetyRecovery` pairs.
- Locally, the line was real: it fixed the white Fast `ply9` seam, aligned the older vulnerable white mana-only board `l8,4;l7,3` to shipping `l8,4;l8,5`, passed `guardrails`, retained `pro-triage` at `target_changed=4 / off_target_changed=0`, exact-lite, and advisory stage-1 CPU at `1.551 / 1.527 / 1.365`.
- Retained duel strength still killed it. `pro-reliability` failed at `0.9167 / 0.7500 / 0.9167`, and the Normal non-win trace rotated onto engine-disabled early-white boards such as `l8,5;l7,6` vs shipping `l8,7;l7,8`, `l9,4;l8,5` vs `l9,4;l9,3`, and `l8,5;l7,6` vs `l9,5;l8,3;l7,4`. The code was discarded.

## Default Non-Win Surface Alignment Wave

- No new frontier challenger was cut from this wave either. The useful spend was replaying the full default retained duel corpus on shipped `frontier_pro_v2_guarded` and collapsing the exact non-win openings.
- The shipped frontier miss is now fully mapped to the existing live non-win probe surface. `vs_shipping_pro` only loses on `opening_reply_white`, `black_recovery_branch`, and `white_split_trace`; `vs_shipping_normal` only loses on `black_bridge_nonwin` and `white_head_acceptance`; `vs_shipping_fast` is clean at `0` non-wins.
- The live root probe was cleaned to match that exact five-board retained duel surface by dropping the stale extra Pro split board that is not part of the current default non-win pack.
- Durable outcome: the next credible Pro challenger should target those five boards directly. The retained duel boundary is no longer an unexplained seed-mismatch story; it is a concrete five-wall frontier problem.

## Partial Three-Wall Guarded Wave

- No new named frontier challenger survived this wave. The local candidate combined a late-white quiet head reject, a turn-3 white vulnerable-window recovery override, and a black turn-6 preserved-spirit reentry override against active `frontier_pro_v2_guarded`.
- The live probe did move the intended three walls. It fixed `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, and `vs_shipping_normal_white_head_acceptance`; retained `primary_pro` triage stayed clean at `target_changed=3 / off_target_changed=0`, and `runtime-preflight` passed with advisory stage-1 CPU at `1.554 / 1.522 / 1.379`.
- Retained duel strength still killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.8333 / 0.7500 / 0.7500` with confidence `0.9807 / 0.9270 / 0.9270`, so the package still was not promotable even before the untouched `white_split_trace` and `black_bridge_nonwin` seams moved.
- It also regressed duel packs that were clean before the edit. `vs_shipping_fast` picked up three non-wins, including a late white post-search snap from shipping `l8,6;l6,5;l6,4` to `l8,7;l9,8` and a repeated late black tail mismatch `l1,8;l1,9` vs shipping `l1,8;l0,8`; `vs_shipping_normal` also reintroduced a white post-search miss (`l8,5;l7,6` vs shipping `l9,5;l8,3;l7,4`).
- Durable outcome: do not reopen partial three-wall approval/head packages. If `white_split_trace` and `black_bridge_nonwin` stay untouched, the retained duel can still fail and previously clean fast/normal packs can regress, so discard the candidate code and keep only the lesson.

## Black Spirit Safety-Gate Wave

- No new frontier challenger survived this wave. The local candidate only tightened `pro_v2_black_turn_six_spirit_reentry` so unsafe preserved-spirit reentry could not beat the available shipping mana root on `black_recovery_branch`.
- The local board and the small gates both looked real. `vs_shipping_pro_black_recovery_branch` aligned to shipping `l6,0;l6,1`, retained `primary_pro` triage stayed clean at `target_changed=4 / off_target_changed=0`, and `runtime-preflight` passed with advisory stage-1 CPU at `1.563 / 1.531 / 1.368`.
- Retained duel strength still killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.9167 / 0.9167 / 0.8333`, so the candidate code was discarded.
- Durable outcome: `black_recovery_branch` is not solved by a blunt unsafe-spirit ban. Keep the lesson, discard the candidate code, and keep the worktree clean.

## Black Legacy-Path Probe Wave

- No new frontier challenger survived this wave either. The useful spend was diagnostic-only on `black_recovery_branch`.
- The ignored probe `black_recovery_branch_legacy_alignment_probe` shows that a direct call to `pro_v2_root_advisor_black_legacy_alignment_override` already returns shipping `l6,0;l6,1` on the live black seam.
- The same probe also captured the path mismatch: a local ProV1 candidate replay on the board resolved to `l1,5;l2,7;l1,8`, while `pro_v2_legacy_selector_probe` still reported `l6,0;l6,1`.
- A naive fallback that scanned qualifying mana roots picked the wrong sibling `l6,0;l7,0`, so the candidate code was discarded before any canonical gate spend.
- Durable outcome: treat `black_recovery_branch` as a legacy-selector plumbing mismatch, not another score-threshold problem. Keep the diagnostic probe and the lesson; discard the production attempt.

## Black Legacy-Selector Config-Swap Wave

- No new frontier challenger survived this wave either. The local candidate changed one line in `pro_v2_root_advisor_select_root`: the ProV1 legacy selector stopped inheriting `shortlist_config` and instead reused the full runtime `config`, which re-enabled the root reply-risk guard for that selector.
- The local board movement was real. The live non-win probe aligned `vs_shipping_pro_black_recovery_branch` to shipping `l6,0;l6,1` through `ApprovedLegacySelector`, while the earlier white turn-three retained fixes stayed intact.
- The small gates also stayed clean enough to justify the spend. `guardrails` passed, retained `pro-triage` stayed at `target_changed=4 / off_target_changed=0`, exact-lite passed, and advisory stage-1 CPU came back at `1.566 / 1.534 / 1.364`.
- Retained duel strength still killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.8333 / 0.9167 / 0.8333`, so the code was discarded.
- Durable outcome: the black legacy-selector mismatch is real, but globally re-enabling reply-risk for that selector is too broad. Keep the lesson, discard the code, and do not reopen this exact config swap.

## Black Reply-Risk-Shortlist Fallback Wave

- No new frontier challenger survived this wave either. The local candidate left the legacy selector alone and only tightened `pro_v2_root_advisor_black_legacy_alignment_override` so the weak plain-spirit black seam could choose the best-ranked vulnerable mana root from the current `reply_risk_shortlist`.
- The local board movement was real. The retained black seam assertion and the live non-win probe both aligned `vs_shipping_pro_black_recovery_branch` to shipping `l6,0;l6,1`, while nearby retained checks for the white confirm board and the black post-search spirit-reentry board still passed.
- The cheap gates also stayed clean enough to justify the duel spend. `guardrails` passed, retained `pro-triage` stayed at `target_changed=4 / off_target_changed=0`, exact-lite passed, and advisory stage-1 CPU came back at `1.561 / 1.522 / 1.367`.
- Retained duel strength still killed it in the same place as the broader black-only lines. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.9167 / 0.9167 / 0.8333`, with Fast still below the floor.
- The follow-up trace showed that Fast loss was not a new collateral surface from the shortlist fallback. Replaying `smart_automove_pro_reliability_nonwin_trace_probe` with `duel_filter=vs_shipping_fast` produced exactly two non-wins, and both were the already-pinned late black head-accept seam on `3 1 b 1 0 2 0 0 14 ...`, where frontier accepts `l1,8;l1,9` and shipping stays on `l1,8;l0,8`.
- Durable outcome: even the shortlist-local black fallback is too broad to keep. Aligning `black_recovery_branch` alone is still not enough; keep the lesson, discard the code, and do not reopen this exact shortlist fallback.

## White Search-Only Recovery Fallback Wave

- No new frontier challenger survived this wave. The local candidate tried the remaining white search-only split `l9,4;l8,3` vs shipping `l9,4;l8,5` in two runtime-variant forms: first by re-querying shipping locally after frontier execution, then by choosing the same nearby safe `DrainerSafetyRecovery` challenger directly from frontier's own ranked roots.
- The local board movement was real. Both variants fixed the retained white `ply9` board and kept the nearby white confirm, white Fast, and black late-fast retained checks clean.
- The cheap gates also stayed clean enough to justify the duel spend. `guardrails` passed, retained `pro-triage` stayed at `target_changed=5 / off_target_changed=0`, exact-lite passed, and advisory stage-1 CPU stayed in the same band at `1.563 / 1.527 / 1.363`.
- Retained duel strength still killed both versions in the same place. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.9167 / 0.8333 / 0.9167`, with Normal below the floor.
- The follow-up Normal non-win trace showed why the direct fallback is not enough. The pack still included the engine-disabled early-white split `l8,5;l7,6` vs shipping `l9,5;l8,3;l7,4`, so fixing the earlier `ply9` recovery board in isolation still leaves the retained Normal blocker alive.
- Durable outcome: do not reopen direct runtime-variant white search-only recovery fallbacks for `l9,4;l8,3`. Keep the lesson, discard the code, and move the next white spend onto the remaining engine-disabled early-white seam instead.

## Black Residue Shipping Fallback Wave

- No new frontier challenger survived this wave. The local candidate added `select_black_progress_setup_engine_disabled_fallback_inputs`, mirroring shipping on the exact black turn-six action+mana weak-window residue board where frontier keeps `l7,1;l9,3` and shipping disables engine selection to play `l1,5;l2,7;l1,8`.
- The local board movement was real. The new retained assertion aligned that residue board to shipping, while the nearby retained white engine-disabled, late black Fast, and late black reply-risk setup walls all stayed clean.
- The cheap gates and the smaller retained duel also stayed clean enough to justify confirm. `guardrails` passed, retained `pro-triage` stayed at `target_changed=5 / off_target_changed=0`, exact-lite passed, advisory stage-1 CPU came back at `1.555 / 1.526 / 1.369`, and retained `pro-reliability` passed at `0.9167 / 0.9167 / 1.0000`.
- The larger confirm duel killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.9375 / 0.9062 / 0.8750`, with Fast below the floor.
- The confirm-sized Fast non-win trace showed why the wrapper is too shallow. The pack still included the old white search-only split `l9,4;l8,3` vs shipping `l9,4;l8,5`, and it also rotated onto two later black engine-disabled seams: `l0,0;l1,1` vs shipping `l7,1;l8,0`, and `l0,5;l1,5` vs shipping `l2,5;l3,7;l2,8`.
- Durable outcome: do not keep or reopen the direct black engine-disabled shipping fallback on `l7,1;l9,3`. Preserve the improved residue probe and move the next black spend onto the later Fast seams instead of mirroring shipping on the earlier turn-six board.

## Black Residue Full-Pool Advisor Probe Wave

- No new runtime challenger survived this wave either. The only kept change is a stronger ignored `black_progress_vs_setup_residue_probe` that now reports shortlist membership plus utility, reply-floor, and followup metrics for the competing `SpiritImpact` own-setup progress roots on `l7,1;l9,3` vs shipping `l1,5;l2,7;l1,8`.
- The useful lesson is that the remaining black residue is not a safe “promote the shipping root from the full pool” advisor fix.
- The probe confirms the shipping root is still absent from `reply_risk_shortlist`, but it also shows the strongest full-pool own-setup progress challenger under frontier's current metrics is `l1,5;l3,7;l2,8`, not shipping `l1,5;l2,7;l1,8`.
- Durable outcome: do not reopen this board with another blind advisor family-competition override. Any future black spend here has to explain the engine-disabled ordering that makes shipping choose `l1,5;l2,7;l1,8` instead of the stronger full-pool spirit candidate first.

## Black Recovery Shortlist Revisit Wave

- No new runtime challenger survived this revisit either. The local candidate reinstated the shortlist-local vulnerable-mana fallback inside `pro_v2_root_advisor_black_legacy_alignment_override`, choosing the best-ranked vulnerable `ManaTempo` root already present in `reply_risk_shortlist` on `black_recovery_branch`.
- The local seam movement was real on the cleaned promoted package. `black_recovery_branch` aligned to shipping `l6,0;l6,1`, and the five-board live nonwin root probe collapsed back to the older white seams.
- The refreshed ignored `black_recovery_branch_legacy_alignment_probe` is worth keeping: it now prints shortlist root details and confirms why the local fallback picks shipping `l6,0;l6,1` instead of the earlier wrong score-leader `l6,0;l7,0`.
- The broader retained duel still killed the line. The canonical loop passed `guardrails`, `pro-triage`, exact-lite, and advisory stage-1 CPU, then failed retained `pro-reliability` at `0.9167 / 0.9167 / 0.8333`.
- The failure surface also changed enough to rule out keeping the runtime cut as a live challenger. The new `pro` miss rotated to a later black lane split `l1,6;l1,7` vs shipping `l1,6;l1,5`, `normal` stayed on the old white `ply9` search-only split `l9,4;l8,3` vs `l9,4;l8,5`, and the two Fast non-wins had `first_diff=none`.
- Durable outcome: even after the earlier late-Fast blocker was repaired, the shortlist-local black legacy fallback is still not promotable. Keep the stronger diagnostic, discard the runtime/test change, and do not reopen this exact line again without explaining the later black lane split plus the no-diff Fast failure.

## Black Later Lane Probe Wave

- No new runtime challenger survived this wave either. The only kept change is a focused ignored `black_pro_lane_split_probe` for the later black `pro` miss `l1,6;l1,7` vs shipping `l1,6;l1,5`.
- The useful lesson is that this seam is not another shortlist omission and not another head-acceptance miss.
- The probe shows shipping `l1,6;l1,5` is already in the frontier `reply_risk_shortlist` beside frontier `l1,6;l1,7` and the rejected head `l2,6;l1,5`. All three are the same safe-progress family.
- It also shows shipping's root loses on frontier's own selector metrics: the reply floor is tied, `shipping_vs_frontier=false`, and frontier keeps better local utility because it preserves `drainer_safety=2` while shipping drops to `-1`.
- Durable outcome: do not reopen the later black lane split with another advisor or reply-risk ordering tweak. Shipping only reaches `l1,6;l1,5` because it disables the turn-engine selector on that board, so this is another shipping-disabled lower-safety ordering mismatch rather than a live frontier bug.

## Fast Hotspot Trace Wave

- No new runtime challenger survived this wave either. The only kept change is a focused ignored `fast_hotspot_trace_probe` for the retained Fast hotspot opening `0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/...`.
- The main value of that probe is that it killed the lazy “Fast is still failing only on no-diff gate noise” read for the current promoted package.
- On the current promoted tree, the hotspot opening really does diverge when frontier is white. The first drift is at `ply=57` on `1 1 w 0 0 1 0 0 9 n04s1xn06/...`, where frontier plays `l9,5;l8,6` from `engine_post_search` and shipping plays `l8,5;l7,7;l8,8` from `engine_disabled`. Frontier's head is rejected, so this is not another head-acceptance issue.
- The same opening with frontier as black is not a fresh target. It rotates back onto the known `black_recovery_branch` split `l1,5;l3,3;l2,3` vs shipping `l6,0;l6,1`.
- Durable outcome: do not spend the next wave treating the retained Fast hotspot as identical-behavior noise. The efficient next step is to classify that late white engine-disabled divergence board before reopening any older black or white no-go seam.

## White Late Fast Hotspot Wave

- No new runtime challenger survived this wave either. The only kept change is a focused ignored `white_late_fast_hotspot_probe` for the late white Fast hotspot board `1 1 w 0 0 1 0 0 9 n04s1xn06/...`.
- The useful lesson is that this board does not admit the obvious shipping mirror.
- Frontier's approved root `l9,5;l8,6` is the only reply-risk-shortlisted root and preserves the stronger reply floor (`921` vs shipping `651`).
- Shipping's move `l8,5;l7,7;l8,8` is outside the frontier shortlist, loses under frontier's own reply-risk comparator (`shipping_vs_frontier=false`), and is not even the strongest spirit-progress candidate in the full pool.
- Durable outcome: treat this as another shipping-only search ordering mismatch, not a live wrapper-fallback or advisor omission. Do not reopen it with direct shipping mirroring, shortlist widening, or a simple spirit-family override without a new explanation for shipping's search-only ordering.

## White Ply9 Search Ordering Wave

- No new runtime challenger survived this wave either. The only kept result is the stronger read from the existing ignored `white_fast_ply9_search_only_split_probe` on `l9,4;l8,3` vs shipping `l9,4;l8,5`.
- The useful lesson is that the remaining white `ply9` seam is not just “previously disproved” in aggregate; it is locally another shipping-only ordering mismatch.
- Frontier's approved `l9,4;l8,3` is the only reply-risk-shortlisted root and keeps the stronger floor (`1191` vs shipping `730`).
- Shipping's `l9,4;l8,5` is a full-pool `DrainerSafetyRecovery` root outside the frontier shortlist and still loses under frontier's own reply-risk comparator (`shipping_vs_frontier=false`).
- Durable outcome: do not reopen that `ply9` seam with direct shipping mirroring, shortlist widening, or another simple recovery override without a new explanation for shipping's `search_only_engine_allowed_head` ordering.

## White Ordering Config Wave

- No new runtime challenger survived this wave either. The only kept change is a focused ignored `white_profile_config_ordering_probe` for the two remaining white ordering boards: `white_ply9_search_ordering` and `white_late_fast_hotspot`.
- The useful lesson is that there is no hidden per-board config branch separating those seams from the promoted package.
- On both boards, shipping and frontier use the same depth (`4`), node budget (`15774`), reply-risk shortlist budget (`9 / 24 / 2000`), and scoring weights.
- The remaining difference is structural and profile-level: shipping stays `selector=false`, `head_rerank=true`, `mode=ProV1`, while frontier stays `selector=true`, `head_rerank=false`, `mode=ProV2` with the extra ProV2 guards.
- Durable outcome: treat the remaining white seams as search-profile semantics, not board-local config misses. Do not reopen them with another board-local wrapper or guard tweak unless there is a brand-new shared hypothesis.

## White Rerank Semantics Wave

- No new runtime challenger survived this wave either. The kept diagnostic is `white_ordering_rerank_semantics_probe`, and the discarded runtime cut was a frontier-local rerank-semantics fallback for `white_ply9_search_ordering`.
- The probe usefully split the two remaining white ordering boards instead of merging them. On `white_ply9_search_ordering`, shipping `l9,4;l8,5` is rank `0` on both the shipping and frontier root sets, is `Accepted` by `classify_turn_engine_rerank_override`, is allowed by `turn_engine_allowed_rerank_override_candidate`, and does not conflict with the ProV2 advisor.
- The late Fast hotspot is not the same class. On `white_late_fast_hotspot`, shipping `l8,5;l7,7;l8,8` is rejected by `ProgressGate` and is not an allowed rerank candidate even on shipping's own root set.
- The runtime cut was real but still too shallow. It fixed `l9,4;l8,3` vs shipping `l9,4;l8,5`, passed `guardrails`, `pro-triage` at `target_changed=5 / off_target_changed=0`, exact-lite, and advisory stage-1 CPU at `1.551 / 1.523 / 1.363`, then failed retained `pro-reliability` at `0.9167 / 0.8333 / 0.9167`.
- The retained Normal trace showed why it cannot stay live: instead of stopping at `ply9`, the pack rotated onto other early-white engine-disabled seams, including `l8,5;l7,6` vs shipping `l8,7;l7,8` and `l8,5;l7,6` vs shipping `l8,5;l7,4`.
- Durable outcome: do not reopen `white_ply9_search_ordering` with another narrow rerank-semantics wrapper fallback. Even when the shipping root is rerank-admissible and advisor-compatible on frontier, the local repair still is not enough to promote.

## White Normal Residue Wave

- No new runtime challenger survived this wave. The kept additions are a fresh retained `vs_shipping_normal` non-win trace on the cleaned promoted package and the new ignored `white_normal_ply11_search_only_split_probe`.
- The useful result is that the discarded rerank fallback's rotated engine-disabled boards are not the live retained Normal surface on the clean tree.
- Replaying `smart_automove_pro_reliability_nonwin_trace_probe` with `SMART_PRO_RELIABILITY_DUEL_FILTER=vs_shipping_normal` produced `total_nonwins=1`, and the sole drift was a sibling of the white search-order family: `0 0 w 1 0 1 0 0 3 n06a0xn04/...`, first diff at `ply=11`, frontier `l9,4;l8,3`, shipping `l9,4;l8,5`.
- The new board-local probe shows the same structural shape as the earlier Fast probe. Frontier's shortlist is still just `l9,4;l8,3`; shipping still reaches `l9,4;l8,5` through `search_only_engine_allowed_head`; the shipping recovery root is still outside the frontier shortlist; and `shipping_vs_frontier=false` under frontier's own reply-risk comparator.
- Durable outcome: until a future challenger changes the clean retained trace, treat current Normal residue as the white `l9,4;l8,3` vs `l9,4;l8,5` search-order family, not as the rotated early-white engine-disabled boards from the discarded rerank fallback.

## White Allowed-Head Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_search_order_allowed_head_probe`.
- The useful result is that the live white search-order family is not blocked by root-set reachability. On both `white_ply9_search_ordering` and `white_normal_ply11_search_ordering`, shipping's rerank engine still chooses `l9,4;l8,5` when it is fed the frontier root set.
- Frontier's own rerank engine config on the same allowed heads still prefers `l9,4;l8,3`, with `l9,4;l8,5` only appearing as a lower-ranked rerank plan.
- Durable outcome: treat the current white residue as a rerank-engine profile split (`shipping` ProV1 rerank vs `frontier` ProV2 rerank), not as a selector-disabled root omission. Another wrapper fallback that only patches root choice is not a real explanation.

## White Rerank Mode Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_search_order_rerank_mode_probe`.
- The useful result is that the white rerank split is narrower than “ProV2 everywhere is wrong” but stronger than a pure root-set or admissibility issue.
- On both white search-order siblings, forcing only the frontier rerank engine mode from `ProV2` to `ProV1` already flips the best allowed-head plan to shipping `l9,4;l8,5`.
- Shipping still chooses `l9,4;l8,5` even when its rerank mode is forced to `ProV2`, so the mismatch is not a generic mode-only story either.
- Durable outcome: treat the live white residue as a frontier-`ProV2` rerank semantics split. Do not jump straight from that fact to another narrow runtime mode-swap fallback without fresh retained-duel evidence.

## White Rerank Budget Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_search_order_rerank_budget_probe`.
- The useful result is that the live white rerank split is not spread across all of frontier's rerank budget knobs.
- On both white search-order siblings, swapping only frontier's rerank own-search caps (`own_seed_cap`, `own_beam`, `per_node_family_cap`, `step_cap`) to the shipping `ProV2` values already flips the best allowed-head plan from `l9,4;l8,3` to shipping `l9,4;l8,5`.
- Swapping only frontier's reply caps (`opponent_seed_cap`, `opponent_beam`, `reply_seed_cap`, `reply_beam`) does nothing, and swapping only the expansion cap does nothing.
- Durable outcome: treat the current white residue as a frontier-`ProV2` rerank own-search-breadth split. If there is ever a runtime spend here, it should be justified against that exact surface rather than against generic reply breadth or expansion arguments.

## White Own-Cap Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_search_order_rerank_own_cap_probe`.
- The useful result is that the frontier-side rerank own-search split is not spread evenly across the four own caps.
- `step_cap` alone flips both white search-order siblings from `l9,4;l8,3` to shipping `l9,4;l8,5`, and it reproduces the shipping rerank utility on both boards.
- `own_seed_cap` alone also flips both siblings to `l9,4;l8,5`, but it does not reproduce the same rerank utility on the Fast board, so it is a weaker explanation than `step_cap`.
- `own_beam` alone and `per_node_family_cap` alone do nothing; frontier stays on `l9,4;l8,3`.
- Durable outcome: treat frontier `ProV2` rerank `step_cap` as the cleanest single-cap explanation for the live white residue seen so far. Do not jump from that directly to a runtime patch on an already promotable package without fresh duel evidence.

## White Seed-Step Scope Wave

- No new runtime challenger survived this wave either. The kept diagnostic is `white_search_order_seed_step_scope_probe`.
- The useful result is that the two remaining “active” white rerank caps are still too broad to spend directly.
- On the two live white search-order siblings, shrinking either frontier rerank `own_seed_cap` or frontier rerank `step_cap` still flips the allowed-head plan from `l9,4;l8,3` to shipping `l9,4;l8,5`.
- On the late white Fast hotspot, though, the same broad cap shrink does not leave the board stable and does not reproduce shipping. `own_seed_cap` shrink moves frontier to `l8,5;l6,5;l5,4`, `step_cap` shrink moves it to `l8,5;l6,5;l6,4`, and shipping's rerank on the frontier head set still stays on `l9,5;l8,6`.
- Durable outcome: broad white rerank `own_seed_cap` or `step_cap` shrink is not a safe runtime answer. Any future white rerank spend has to gate more tightly than “lower frontier rerank seed/step breadth.”

## White Runtime Step Clamp Wave

- No new runtime challenger survived this wave. The attempted cut was the narrowest live follow-up to the seed/step probes: clamp frontier `ProV2` rerank `step_cap` to `1` only on the exact white `turn=3 / mons_moves=1 / no-action / mana-only / window=1 / deny=1 / drainer_safety<0` board class.
- The useful result is that the live white search-order residue is not controlled by `turn_engine_rerank_config` alone.
- The focused retained slice failed immediately. On both white siblings, frontier still selected `l9,4;l8,3` instead of shipping `l9,4;l8,5`.
- The runtime shape did not budge: selector stage stayed `engine_post_search`, the accepted head stayed `l9,4;l8,3`, and the approved shortlist stayed a singleton on that same root.
- Durable outcome: do not reopen the white search-order family with another board-local rerank-cap runtime tweak unless a future probe first shows how that tweak can move shortlist or approved-root behavior instead of only changing allowed-head rerank plans in isolation.

## White Shortlist Gate Wave

- No new runtime challenger survived this wave either. The kept diagnostic is `white_search_order_shortlist_gate_probe`.
- The useful result is that the remaining white search-order seam is not blocked by candidate focus and is not waiting on the current safe-progress shortlist extension.
- On both white siblings, shipping `l9,4;l8,5` is already present in frontier `candidate_indices`.
- It still never reaches the approved shortlist. The shortlist stays a singleton on vulnerable `l9,4;l8,3` because that root's score is about `809k` above shipping's, far beyond the `165` shortlist margin, and `pro_v2_safe_progress_sibling_shortlist_extension` returns `None`.
- Durable outcome: do not reopen the white search-order family with another simple shortlist tweak or another rerank-cap tweak. Any future white spend has to explain a new shortlist/approved-root reentry theory, or a deeper root-scoring normalization, instead of assuming the current focus/extension machinery just missed `l9,4;l8,5`.

## White Selector-Disable Wrapper Probe Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_search_order_selector_disable_probe`.
- The useful result is that shallow config mirroring does not actually test selector-disabled white semantics through the guarded frontier wrapper.
- On both white search-order siblings and on `white_late_fast_hotspot`, forcing the incoming frontier runtime config to `selector=false`, `head_rerank=true`, shipping-like own caps, or even `TurnEngineMode::ProV1` still leaves the live decision unchanged on frontier `engine_post_search`.
- The code path explains why: `select_frontier_pro_v2_guarded_inputs` always routes frontier execution back through `apply_frontier_pro_v2_guarded_config`, so those config-only selector-disable toggles are reapplied away before search runs.
- Durable outcome: do not reopen the white search-order family with another wrapper-local config-only selector-disable or head-rerank mirror. Any future white wrapper spend has to change wrapper branching itself, or move deeper into shortlist/root-scoring behavior.

## White Raw Search-Only ProV1 Scope Wave

- No new runtime challenger survived this wave. The kept diagnostics are `white_search_order_wrapper_branch_probe` and `white_search_order_raw_prov1_scope_probe`.
- The useful result is that once the guarded wrapper is truly bypassed, raw search-only branch choices become locally meaningful.
- Raw `search-only + ProV2` still keeps frontier `l9,4;l8,3` on the two white search-order siblings and keeps frontier `l9,5;l8,6` on the late Fast hotspot.
- Raw `search-only + shipping own caps + ProV2` fixes the two `ply9/ply11` siblings but still leaves the hotspot on frontier.
- Raw `search-only + ProV1` matches shipping on all three local white seams, so search-only `ProV1` semantics are a real explanation rather than a dead config artifact.
- That broad line is still not safe enough to keep. The scope probe shows the same raw `search-only + ProV1` reroute also flips the retained vulnerable white turn-three guard from `l8,4;l7,3` to shipping `l8,4;l8,5`, even though it shares the same coarse `turn=3 / mons_moves=1 / no-action / mana-only / window=1 / deny=1 / drainer_safety<0` context as the unresolved white siblings.
- Durable outcome: do not reopen the white search-order family with a broad wrapper-level search-only `ProV1` gate on that coarse white context. Any future white wrapper spend has to distinguish the retained vulnerable guard from the unresolved siblings with a narrower theory than “use shipping-like search-only semantics here.”

## White Vulnerable-Guard Search-Order Comparison Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_vulnerable_guard_search_order_probe`.
- The useful result is that the retained vulnerable white turn-three guard is not a different class from the unresolved white search-order siblings at the current shortlist/reply-risk layer.
- On the retained guard board, frontier still keeps a singleton vulnerable `ManaTempo` shortlist and shipping still only wins through `search_only_engine_allowed_head` on an outside-shortlist rank-0 `DrainerSafetyRecovery` root, just like the unresolved sibling boards.
- `shipping_vs_frontier` is still `false` there too, and there is still no projection or advisor reentry signal that would justify a simple wrapper-local split between the guard board and the unresolved siblings.
- Durable outcome: do not reopen the white search-order family with another wrapper-level gate based on the current reply-risk/shortlist surface. Any future white spend has to distinguish boards below that surface or change the root scoring that makes both boards disagree in the same way.

## White Negative-Deny Rank Surface Wave

- No new runtime challenger survived this wave. Two narrow negative-deny white wrapper cuts were tried and both died in the focused retained slice before any canonical gate spend.
- The first cut replayed the remaining Normal sibling through raw `search-only + shipping own caps + ProV2` and tried to keep it only when that replayed move was `root_rank=0`.
- The second cut used direct shipping fallback on the same board and tried to keep it only when the shipping move was `root_rank=0` in the shipping search.
- Both cuts still reopened the retained vulnerable guard `l8,4;l7,3` to shipping `l8,4;l8,5`, so both runtime changes were discarded.
- The kept diagnostic is `white_search_order_rank_surface_probe`.
- Its useful result is that the current rank surfaces are fake separation for the negative-deny white seam.
- On the unresolved Normal sibling, shipping `l9,4;l8,5` is both `selected_rank=0` and `root_rank=0`.
- On the retained vulnerable guard, shipping `l8,4;l8,5` still shows `selected_rank=4` under the shipping runtime, but its underlying `root_rank` is already `0`; the raw shipping-own-caps replay also gives that same guard move `root_rank=0`.
- Durable outcome: do not reopen the negative-deny white search-order family with another runtime gate based on current selected-rank or root-rank surfaces. Any future white spend there has to distinguish a deeper ranking layer than “shipping root becomes top-ranked.”
