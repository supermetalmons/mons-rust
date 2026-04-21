# Automove Knowledge

This file keeps only durable automove lessons and kill rules.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the live workflow and `AUTOMOVE_IDEAS.md` for the current live state.

## Stable Runtime Truths

- Shipping Pro now routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` is the retained search-only baseline profile.
- `frontier_pro_v2_guarded` is the retained Pro frontier and the deployed Pro path.
- Probe paths are diagnostics only; they do not describe shipping behavior.
- Promotion happened off direct frontier-vs-baseline duel evidence, not fixture churn or hotspot output.
- The promotion bottleneck was root composition quality, not speed.
- `runtime-preflight` still matters after promotion: exact-lite diagnostics are a hard gate and stage-1 CPU remains advisory-only for Pro.
- When a live miss appears, separate `pre_accept` search choice from final `engine_post_search` output before changing shared heuristics.
- Matching a forced head is weaker than matching the full forced-turn-engine probe stage shape.
- Quiet play is only acceptable after checking immediate opponent-drainer pressure.
- Safe supermana and safe opponent-mana progress remain high-value tempo goals.
- Leaving your own drainer vulnerable is only acceptable for an immediate winning or scoring payoff.
- Spirit deployment should create progress or setup value, not idle on base.
- Promoting duel-derived live misses into retained triage is useful, but fixture churn alone is not promotion evidence.
- On white turn-3 split boards, a safe mana sibling correction only sticks if it runs after white mana competition selection.
- Quiet late-white head-accept guards can fix a real live reply-risk seam, but that repair alone does not move the `vs shipping Normal` wall.
- If `search_only_engine_allowed_head` keeps surviving on a white vulnerable-window board where the final surfaced roots also include `DrainerSafetyRecovery`, do not trust a seemingly-matching rerank block until probe output proves the stage actually changed.
- Exact live-seam shipping-alignment can clear `pro-triage` and `runtime-preflight` while still failing direct `pro-reliability` badly (`0.5000 / 0.7500 / 0.8333` vs shipped `frontier_pro_v2_guarded`); seam coverage is not duel strength.
- Candidate-only quiet lower-scored root-score guards can fix `vs_shipping_pro_opening_reply_white` and move retained `primary_pro` broadly, but they still fail direct `pro-reliability` vs shipped `frontier_pro_v2_guarded` (`0.5833 / 0.7500 / 0.9167`); suppressing quiet mana heads alone is not a promotable frontier.
- If dormant shared toggles or a candidate-only unsafe plain-spirit floor guard leave the live non-win root probe completely unchanged, kill the line before canonical gates; the real spend still sits higher in the approval or wrapper path.
- Search-only `forced_tactical_prepass` priority under Pro config can stay completely inert even when threaded through the white scoring-window fallback; if `vs_shipping_normal_white_head_acceptance` still stays on `search_only_engine_allowed_head`, the missing spend is deeper than prepass ordering.
- Candidate-only white vulnerable-window head rejects and quiet-mana reply-score guards can still stay inert even when threaded through the white scoring-window fallback; if `white_split_trace` only flips the approval reason label without changing the selected root, the live spend sits inside a deeper white family-competition or root-move signal.
- A stricter white mana-sibling same-lane gap can align `vs_shipping_pro_white_split_trace` to shipping without touching `vs_shipping_pro_opening_reply_white` or `vs_shipping_normal_white_head_acceptance`; those remaining walls are not controlled by the same white sibling reentry seam.
- A late-white low-budget selector exception plus a late quiet-mana head reject can still leave `vs_shipping_pro_opening_reply_white` on `engine_disabled`; that wall is not fixed by the guessed low-budget board shape alone.
- A simple search-only white vulnerable-window top-head conflict can still leave `vs_shipping_normal_white_head_acceptance` on `search_only_engine_allowed_head` even with an in-scope `DrainerSafetyRecovery` alternative; the rerank acceptance gap is deeper than a top-root conflict check.
- If a live probe board reports `runtime_variant_branch=frontier_execute` together with `selector_disable_reason=pre_disabled`, first prove the probe is not being contaminated by a second shipping-search fallback call. An unconditional extra search can overwrite the top-level selector diagnostics on unrelated boards.
- After fixing that contamination, `vs_shipping_pro_opening_reply_white` and `vs_shipping_normal_white_head_acceptance` both really stayed on `frontier_execute + engine_post_search + selector not_disabled`; those white walls were not wrapper mismatches.
- On the corrected probe, `opening_reply_white` is a post-search head-over-advisor seam, while `normal_white_head_acceptance` is an early-white vulnerable-window recovery miss where a safe `DrainerSafetyRecovery` alternative already exists in the scored roots.
- The `head_family` and `goal_family` probe fields make those white walls concrete: `opening_reply_white` is `SafeSupermanaProgress -> DrainerSafetyRecovery`, while `normal_white_head_acceptance` is `SafeSupermanaProgress -> ImmediateScore`.
- The probe is much more useful once it shows the actual pre-accept root versus the accepted head-plan utility. Keeping `pre_accept_family`, `pre_accept_score`, `pre_accept_utility`, `head_score`, `head_plan_utility`, `head_plan_head_utility`, and `head_plan_primary_axes_vs_pre_accept` makes late-white head seams inspectable without guessing from final selected roots alone.
- Even when a candidate fixes both of those white walls, passes `guardrails`, moves retained `primary_pro` by `5 / 62` with `off_target_changed=0`, and clears exact-lite, it can still fail retained `pro-reliability` vs `shipping_pro_search` at `0.6667 / 0.6667 / 0.6667`; local white seam coverage plus clean triage is still not duel proof.
- A family-aware white safe-progress head guard plus turn-3 vulnerable-window recovery override can fix both white live walls and preserve the black recovery seam, yet still fail retained `pro-reliability` vs `shipping_pro_search` at `0.8333 / 0.7500 / 0.9167`; those white family-specific repairs are still overfitting local seams rather than lifting the real frontier.
- Adding a tighter black turn-6 spirit-reentry filter to that family-aware white package can also align `vs_shipping_pro_black_recovery_branch`, clear retained `primary_pro` at `4 / 62` with `off_target_changed=0`, and still fail retained `pro-reliability` vs `shipping_pro_search` at `0.8333 / 0.7500 / 0.7500` while only clearing `stage1_cpu` in advisory mode at about `1.50x` to `1.61x`; live non-win family alignment is still not promotable duel strength.
- Even a challenger that aligns all four active live walls can still be non-promotable on canonical cost alone. `frontier_pro_v3_live_wall_combo_guarded` fixed `opening_reply_white`, `black_recovery_branch`, `white_split_trace`, and `normal_white_head_acceptance`, kept retained `primary_pro` clean at `target_changed=2 / off_target_changed=0`, passed `smart_automove_tactical_selected_profile`, and passed exact-lite, yet still died immediately on canonical `stage1_cpu` at `1.696x` median against `shipping_pro_search`.
- Widening reply-risk shortlist and node-share coverage, enabling lazy score-window projection, and allowing small root injection can surface the black late mana fallback directly inside the shortlist while still leaving `approved_root` on the preserved spirit reentry and `injected_root` at `None`; if the live walls stay unchanged, the spend still sits in approval or head logic rather than shortlist width.
- Letting white turn-3 mana competition consider a higher-rank sibling with a real score edge can move `vs_shipping_pro_white_split_trace` onto shipping `l10,8;l9,7`, but that repair still does not transfer to `opening_reply_white` or `normal_white_head_acceptance`.
- On `vs_shipping_normal_black_bridge_nonwin`, a candidate-only white followup-mana escape can beat the spirit-own-mana setup and land on shipping `l6,1;l5,0;mb`, but that same approval-escape package still leaves the black recovery and late white head seams unchanged.
- If `opening_reply_white` still accepts the same late white head after a generic lower-score sibling reject, the missing spend is not a simple candidate-vs-selected shape check; it still needs more direct post-search head-over-advisor evidence.
- Broadening that late-white reject to also cover safe-recovery preaccept roots still may not move `opening_reply_white` or `normal_white_head_acceptance`; a generic vulnerable-vs-safe post-search shape check is still not enough evidence for those white walls.
- If `black_recovery_branch` still preserves the spirit reentry after the shipping mana root is both the full-pool legacy choice and already present in the reply-risk shortlist, the missing spend is not legacy reachability alone. Approval is still anchored on the preserved spirit path.
- Removing challenger-safety from a black vulnerable-spirit escape can overcorrect `black_recovery_branch` onto legacy mana `l6,0;l6,1` while shipping still stays on spirit `l1,5;l3,3;l2,3`; that seam is not solved by simply allowing weaker mana challengers through.
- Tightening the black turn-6 spirit-reentry filter and the white turn-3 mana sibling competition together can align `vs_shipping_pro_black_recovery_branch` plus `vs_shipping_pro_white_split_trace`, but that shared package still leaves `opening_reply_white` untouched and can still let `normal_white_head_acceptance` snap back from advisor-approved safe recovery onto vulnerable `l9,4;l8,3`; the surviving spend is still in white post-search head acceptance.
- A narrower retained-surface package is still not safe by default. Late-white quiet mana head rejection plus turn-3 white vulnerable-window recovery plus black vulnerable spirit-reentry can fix `opening_reply_white`, `black_recovery_branch`, and `normal_white_head_acceptance`, keep retained `primary_pro` at `2 / 62` with `off_target_changed=0`, and pass exact-lite, yet still regress canonical `stage1_cpu` to about `1.62x / 1.76x / 1.62x` against `shipping_pro_search`; retained seam coverage still is not promotion evidence if the runtime cost jumps back into live-wall-combo territory.
- Removing the turn-3 white vulnerable-window recovery override still does not make that retained-seam package cheap enough. Late-white opening head rejection plus black vulnerable spirit-reentry alone still moves retained `primary_pro` by `2 / 62` with `off_target_changed=0`, yet canonical `stage1_cpu` remains around `1.586x / 1.619x / 1.625x` against `shipping_pro_search`; the cost regression is not coming only from the early-white recovery spend.
- The retained harness must treat stage-1 CPU as advisory by default for `frontier_pro_*`, not only when an env override is set. After fixing that mismatch, the deeper blocker became visible: the default retained `pro-reliability` corpus (`pro_turn_planner_reliability_v1`) currently fails shipped `frontier_pro_v2_guarded` at `0.7500 / 0.8333 / 1.0000`, so that corpus is not a valid promotion boundary until it is reconciled with the shipped frontier.
- If `normal_white_head_acceptance` still stays on the vulnerable window root even after turn-3 recovery logic searches beyond the shortlist, the blocker is not shortlist coverage alone. The reply-risk approval path still dominates that board.
- On `normal_white_head_acceptance`, candidate-only white vulnerable-window recovery can move `pre_accept_input_fen` and advisor approval to safe `DrainerSafetyRecovery l9,4;l8,5` while the final selected root still snaps back to vulnerable `l9,4;l8,3`; that seam is not fixed until post-search head acceptance itself moves.
- Tightening `pro_v2_risky_recovery_progress_sibling_order` and adding a bounded late-black vulnerable non-spirit followup escape was completely inert against active `frontier_pro_v2_guarded`; if both the live probe and retained `primary_pro` stay at `0 / 0`, the real spend is not in those shared reply-order thresholds alone.

## Retained Seam And Fixture Map

- `human_win_pro_c`: retained white safe-progress drift. Useful for triage and selector hygiene, but not promotion proof by itself.
- `primary_white_safe_progress_rerank_ply27`: retained white accepted `ManaTempo` rerank over a safer progress baseline.
- `primary_black_turn_four_action_mana_ply15`: retained later-black forced-engine `ManaTempo` seam on action+mana boards.
- `primary_black_mana_bridge_ply20`: retained black injected mana-bridge seam on shipping baseline `l4,1;l5,0;mb`.
- `primary_black_spirit_bridge_ply19`: retained black injected spirit-bridge seam on the same shipping baseline.
- `primary_black_negative_deny_ply4`: retained early-black negative-deny selector seam.
- `primary_live_nonwin_opening_reply_white`: retained live duel seam where quiet mana-head acceptance beat a stronger reply-risk-aware mana continuation.
- `primary_live_nonwin_black_vulnerable_spirit_reentry`: retained live duel seam where vulnerable plain-spirit reentry competed against a quieter mana continuation.
- Closed surfaces that should stay closed: `primary_spirit_setup`, `primary_pvs_sensitive_search`, and `primary_black_reliability_opening_3_ply4`.
- Live probe seams that should stay diagnostic-only: `vs_shipping_pro_black_recovery_branch`, `vs_shipping_normal_white_head_acceptance`, `vs_shipping_pro_white_split_trace`, and `vs_shipping_normal_black_bridge_nonwin`.

## Retained Diagnostic Toolbox

- `smart_automove_pro_reliability_duel_trace_probe`: replay duel seeds and show first divergence.
- `smart_automove_pro_reliability_nonwin_trace_probe`: collapse exact non-win openings from a duel corpus.
- `smart_automove_pro_reliability_hotspot_probe`: bounded hotspot compare for real duel trouble spots.
- `smart_automove_pro_triage_retained_churn_probe`: show which retained fixtures are moving and why.
- `smart_automove_pro_forced_turn_engine_retained_churn_probe`: inspect forced-turn-engine probe acceptance on retained churn fixtures.
- `smart_automove_pro_root_advisor_trace_probe`: print unified root-advisor shortlist, approvals, preserved families, and injected-root decisions.

## Proven Kill Rules And Anti-Patterns

- Kill any line that fails `guardrails`, reopens closed retained seams, or pushes off-target churn above `1`.
- Kill any line that only fixes one traced seam and leaves the cheap surface on stale churn.
- Kill any line that clears retained fixtures but does not move direct duel evidence on the candidate-vs-baseline matchup.
- Kill any line that only starts winning after the fixture surface is expanded but then loses the direct frontier-vs-shipping duel.
- Kill any line whose main gain is suppressing lower-scored quiet heads without broader duel lift.
- Kill any line that does not move at least one retained live root-probe wall when the hypothesis was explicitly built to do that.
- Kill any line that is supposed to fix a search-only handoff but leaves the exact handoff stage unchanged in the probe output.
- Kill any line that only changes root-advisor reason labels while the selected live probe root stays the same.
- Kill any line whose main gain is only exposing extra shortlist entries or enabling small root injection while the approved live probe root still does not move.
- Kill any line that only repairs the white split-trace sibling reentry if the opening-reply and search-only vulnerable-window walls still stay unchanged.
- Kill any line that fixes `white_split_trace` and `normal_black_bridge_nonwin` but still leaves `opening_reply_white`, `black_recovery_branch`, and `normal_white_head_acceptance` unchanged. That approval-escape package is still not the real frontier.
- Kill any line that repairs both corrected white live walls but still stalls at `0.6667` retained reliability across Pro, Normal, and Fast. That line is still overfitting live seams instead of improving the real frontier.
- Kill any line that only proves “all four live walls aligned” plus clean retained triage. `frontier_pro_v3_live_wall_combo_guarded` still failed canonical `stage1_cpu` at `1.696x`, so live-wall coverage alone is not promotion evidence.
- Kill any line that only changes advisor or `pre_accept_input_fen` metadata while direct `pro-triage` vs active `frontier_pro_v2_guarded` still returns `target_changed=0 off_target_changed=0`; that challenger is behaviorally inert even if the probe looks safer before final head acceptance.
- Kill any line that only tightens shared reply-order thresholds without moving a single selected root on the live probe or retained `primary_pro`. If `strict risky-recovery progress` plus `late-black vulnerable non-spirit followup` still returns `0 / 0` vs active `frontier_pro_v2_guarded`, stop there.
- Kill any line that leaves both white live walls unchanged and flips `black_recovery_branch` onto legacy mana `l6,0;l6,1`; that is overcorrection, not progress toward the shipped frontier.
- Kill any line that fixes `black_recovery_branch` and `white_split_trace` together but still leaves `opening_reply_white` unchanged and only moves `normal_white_head_acceptance` at the advisor layer. That package still does not touch the surviving white post-search head seam.
- Do not reopen archive profiles or retired branch families without a brand-new shared hypothesis.
- Do not treat hotspot output or one replay seed as production proof without a retained foothold.
- Wrapper-only reroutes and local fallback widening saturate quickly; shared selector/search changes are the real frontier.
- Speed-only wins are not enough once quality is flat.
