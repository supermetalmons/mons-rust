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
