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
  - `frontier_pro_v3_reply_order_guarded` tried two shared reply-order spends together: a stricter risky-recovery progress sibling override and a bounded late-black vulnerable non-spirit followup escape. The line stayed fully inert. The live non-win probe left `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, and `vs_shipping_normal_white_head_acceptance` unchanged, and direct retained `pro-triage` vs active `frontier_pro_v2_guarded` failed at `target_changed=0 off_target_changed=0`, so the code was discarded before `runtime-preflight`.
  - `frontier_pro_v3_white_window_recovery_guarded` added candidate-only white vulnerable-window recovery overrides: a turn-3 no-action recovery redirect and a late white weak-window recovery override on action+mana boards. It did move `vs_shipping_normal_white_head_acceptance` at the advisor layer, changing `pre_accept_input_fen` and approval to safe `DrainerSafetyRecovery l9,4;l8,5`, but the final selected root still stayed on vulnerable `l9,4;l8,3` because post-search head acceptance overrode it. Direct comparison against active `frontier_pro_v2_guarded` left all live selected roots unchanged, and retained `pro-triage` vs the active frontier failed with `target_changed=0 off_target_changed=0`, so the code was discarded before `runtime-preflight` or retained reliability.
  - `frontier_pro_v3_live_nonwin_family_guarded` paired family-aware white `SafeSupermanaProgress` head rejects and a turn-3 vulnerable-window recovery override with a tighter black turn-6 spirit-reentry filter. It fixed `vs_shipping_pro_opening_reply_white`, `vs_shipping_normal_white_head_acceptance`, and `vs_shipping_pro_black_recovery_branch`, passed `smart_automove_tactical_selected_profile`, moved retained `primary_pro` by `4 / 62` with `off_target_changed=0`, passed exact-lite, and only cleared `stage1_cpu` in advisory mode at `1.502x / 1.548x / 1.608x`. Retained `pro-reliability` vs `shipping_pro_search` still failed at `0.8333 / 0.7500 / 0.7500` with confidence `0.9807 / 0.9270 / 0.9270`, so the code was discarded.
  - `frontier_pro_v3_safe_progress_head_guarded` used family-specific white safe-progress head guards plus a turn-3 vulnerable-window recovery override. It fixed both white live walls, preserved the black recovery seam, passed `smart_automove_tactical_selected_profile`, moved retained `primary_pro` by `5 / 62` with `off_target_changed=0`, passed exact-lite, and still failed retained `pro-reliability` vs `shipping_pro_search` at `0.8333 / 0.7500 / 0.9167`, so the code was discarded.
  - `frontier_pro_v3_reply_risk_reentry_guarded` enabled lazy score-window projection, widened the late-white post-search head reject to cover safe-recovery preaccept roots, and relaxed the black vulnerable-spirit escape so vulnerable mana challengers could win approval. It still left `vs_shipping_pro_opening_reply_white` unchanged, still let `vs_shipping_normal_white_head_acceptance` finish on vulnerable `l9,4;l8,3` even after advisor approval moved to safe recovery `l9,4;l8,5`, and flipped `vs_shipping_pro_black_recovery_branch` the wrong way onto legacy mana `l6,0;l6,1` while shipping stayed on spirit `l1,5;l3,3;l2,3`, so the code was discarded before canonical gates.
  - `frontier_pro_v3_approval_escape_guarded` turned on lazy score-window projection and candidate-only approval escapes. It did fix `vs_shipping_pro_white_split_trace` and `vs_shipping_normal_black_bridge_nonwin`, but `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, and `vs_shipping_normal_white_head_acceptance` stayed unchanged, so the code was discarded before canonical gates.
  - `frontier_pro_v3_reply_risk_injection_guarded` widened reply-risk coverage, enabled lazy score-window projection, and allowed small root injection, but the live non-win root probe stayed unchanged on the four real walls: `opening_reply_white`, `black_recovery_branch`, `normal_black_bridge_nonwin`, and `normal_white_head_acceptance`. The code was discarded before canonical gates.
  - `frontier_pro_v3_advisor_window_guarded` fixed both corrected white live walls, passed `guardrails`, moved retained `primary_pro` by `5 / 62` with `off_target_changed=0`, and passed exact-lite, but retained `pro-reliability` vs `shipping_pro_search` still failed at `0.6667 / 0.6667 / 0.6667` with confidence `0.8062 / 0.8062 / 0.8062`.
- Retained confirmation that still matters:
  - `2026-04-10` `pro-reliability-confirm`: `0.9062 / 0.9062 / 0.9062` with confidence `1.0000 / 1.0000 / 1.0000`

## Next Hypothesis

- The corrected live probe is now trustworthy: `opening_reply_white` is a post-search head-over-advisor seam, while `normal_white_head_acceptance` is an early-white vulnerable-window recovery miss where the safer `DrainerSafetyRecovery` root already exists in the scored set.
- Approval-only white vulnerable-window recovery is not enough. The latest failed challenger proved `normal_white_head_acceptance` can move the advisor and `pre_accept_input_fen` to safe `DrainerSafetyRecovery l9,4;l8,5` while the final selected root still snaps back to vulnerable `l9,4;l8,3`, so the missing spend is still in post-search head acceptance rather than recovery reachability.
- Tightening the shared reply-order comparators alone was also not enough. A stricter risky-recovery progress override plus a bounded late-black vulnerable non-spirit followup escape did not move any live selected roots or any retained `primary_pro` fixture against active `frontier_pro_v2_guarded`, so the missing spend is not in those pairwise reply-order thresholds by themselves.
- The new `head_family` and `goal_family` probe fields narrowed those white walls further. `opening_reply_white` is specifically a `SafeSupermanaProgress -> DrainerSafetyRecovery` post-search head-over-advisor seam, while `normal_white_head_acceptance` is a `SafeSupermanaProgress -> ImmediateScore` vulnerable-window head-over-recovery seam.
- Fixing those two white walls together was still not enough. The failed challenger proved that white live-seam repairs plus clean retained triage do not automatically transfer into retained duel strength.
- Fixing those two white walls plus the black turn-6 spirit-reentry wall was still not enough. A combined family-aware white head guard, turn-3 vulnerable-window recovery override, and tighter black spirit-reentry filter still failed retained duels and only cleared `stage1_cpu` in advisory mode.
- A family-aware white safe-progress head guard plus turn-3 recovery override can align both white live walls and still fail retained `pro-reliability` at `0.8333 / 0.7500 / 0.9167` vs `shipping_pro_search`.
- Lazy score-window projection plus approval-path relaxations can fix `white_split_trace` and the Normal black bridge seam together, but they still do not move the surviving blockers. `opening_reply_white` remains a post-search head seam, `black_recovery_branch` remains a preserved-spirit approval seam, and `normal_white_head_acceptance` still needs a stronger vulnerable-window recovery path.
- Widening reply-risk shortlist coverage and allowing small root injection was still not enough. On the black recovery wall it surfaced shipping `l6,0;l6,1` directly inside the reply-risk shortlist, but approval still stayed on the preserved spirit reentry and `injected_root` stayed `None`.
- Broadening the late-white post-search reject to cover safe recovery preaccept roots was still not enough. `opening_reply_white` stayed unchanged, and `normal_white_head_acceptance` still let the vulnerable head `l9,4;l8,3` override the advisor-approved safe recovery `l9,4;l8,5`.
- Relaxing black vulnerable-spirit escape by dropping challenger-safety overcorrected the wrong wall. `black_recovery_branch` flipped onto legacy mana `l6,0;l6,1` while shipping still stayed on spirit `l1,5;l3,3;l2,3`, so that seam is not solved by simply letting weaker mana challengers through.
- There is no second live challenger today.
- The next credible Pro challenger has to move the actual approval or head logic on the surviving walls, especially the black retained churn still exposed by `primary_live_nonwin_black_vulnerable_spirit_reentry` and the extension-sensitive `primary_pro` seams.

## No-Go Notes

- Do not reopen archive profiles as active candidates.
- Do not spend from wrapper-only reroutes, hotspot-only output, or one traced seam without retained surface evidence.
- Do not reopen exact live-seam shipping-alignment overrides. They can clear triage and preflight without being remotely promotable in direct duels.
- Do not reopen quiet-score-only root guards as a standalone challenger. They can move retained `primary_pro` by `5 / 62` and still fail direct duels.
- Do not reopen search-only forced-prepass-priority as a standalone challenger. It can stay completely inert even when threaded through the scoring-window fallback.
- Do not reopen candidate-only white vulnerable-window head rejects or quiet-mana reply-score guards as a standalone challenger. They can reach the targeted white boards and still leave the selected roots unchanged.
- Do not reopen late-white low-budget selector exceptions or simple search-only white top-head conflict checks as standalone challengers. They can stay fully inert on the exact white walls they target.
- Do not reopen wider reply-risk shortlist plus small root injection as a standalone challenger. It can widen black recovery coverage and still leave the actual approved roots unchanged.
- Do not reopen lazy score-window projection plus candidate-only approval escapes as a standalone challenger. It can fix `vs_shipping_pro_white_split_trace` and `vs_shipping_normal_black_bridge_nonwin` while leaving `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, and `vs_shipping_normal_white_head_acceptance` unchanged.
- Do not reopen lazy score-window projection plus broadened late-white post-search reject plus relaxed black spirit-reentry escape as a standalone challenger. It can still leave both white walls unchanged while flipping `vs_shipping_pro_black_recovery_branch` the wrong way onto legacy mana `l6,0;l6,1`.
- Do not spend canonical gates on a candidate that only fixes `vs_shipping_pro_white_split_trace` while leaving `vs_shipping_pro_opening_reply_white` and `vs_shipping_normal_white_head_acceptance` unchanged.
- Do not spend canonical gates on a candidate that leaves the live non-win root probe unchanged.
- Do not treat “fixed both white live walls” plus retained `primary_pro` movement `5 / 62` with `off_target_changed=0` as promotion evidence. That exact shape still failed retained `pro-reliability` at `0.6667` across Pro, Normal, and Fast.
- Do not spend canonical gates on a candidate that only changes advisor or `pre_accept_input_fen` metadata while the final selected live roots stay unchanged. If direct `pro-triage` vs active `frontier_pro_v2_guarded` comes back `target_changed=0 off_target_changed=0`, the challenger is inert and should die immediately.
- Do not reopen a standalone shared reply-order spend that only tightens risky-recovery progress ordering and adds a bounded late-black vulnerable non-spirit followup escape. That exact combination stayed fully inert on both the live probe walls and retained `primary_pro`.
- Do not reopen a standalone white safe-progress head guard plus turn-3 vulnerable-window recovery override as a challenger. Even with `SafeSupermanaProgress` family targeting, it still failed retained `pro-reliability` at `0.8333 / 0.7500 / 0.9167`.
- Do not reopen the combined live non-win family guard package as a challenger. Even after fixing both white live walls and `vs_shipping_pro_black_recovery_branch`, it still failed retained `pro-reliability` at `0.8333 / 0.7500 / 0.7500` and only cleared `stage1_cpu` in advisory mode.
- Do not treat the relaxed `700ms` cap as permission to keep quality-flat changes.
