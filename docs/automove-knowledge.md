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
- Do not reopen archive profiles or retired branch families without a brand-new shared hypothesis.
- Do not treat hotspot output or one replay seed as production proof without a retained foothold.
- Wrapper-only reroutes and local fallback widening saturate quickly; shared selector/search changes are the real frontier.
- Speed-only wins are not enough once quality is flat.
