# Automove Knowledge

This file keeps only durable automove lessons and kill rules.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the live workflow and `AUTOMOVE_IDEAS.md` for the current live state.

## Stable Runtime Truths

- Shipping Pro is `shipping_pro_search`.
- `frontier_pro_v2_guarded` is the only retained Pro frontier.
- `shipping_pro_search` is still the deployed search-only path.
- `frontier_pro_v2_guarded` keeps the guarded opening-book and early-white fallback behavior, but it remains offline.
- Probe paths are diagnostics only; they do not describe shipping behavior.
- Promotion proof is direct frontier-vs-shipping evidence: shipping Pro, Normal, and Fast all must clear `win_rate >= 0.90`, `confidence >= 0.99`, and frontier average move time `<= 700ms`.
- The promotion bottleneck was root composition quality, not speed.
- `runtime-preflight` still matters after promotion: exact-lite diagnostics are a hard gate and stage-1 CPU remains advisory-only for Pro.
- When a live miss appears, separate `pre_accept` search choice from final `engine_post_search` output before changing shared heuristics.
- Matching a forced head is weaker than matching the full forced-turn-engine probe stage shape.
- Quiet play is only acceptable after checking immediate opponent-drainer pressure.
- Safe supermana and safe opponent-mana progress remain high-value tempo goals.
- Leaving your own drainer vulnerable is only acceptable for an immediate winning or scoring payoff.
- Spirit deployment should create progress or setup value, not idle on base.

## Retained Seam And Fixture Map

- `human_win_pro_c`: retained white safe-progress drift. Useful for triage and selector hygiene, but not promotion proof by itself.
- `primary_white_safe_progress_rerank_ply27`: retained white accepted `ManaTempo` rerank over a safer progress baseline.
- `primary_black_turn_four_action_mana_ply15`: retained later-black forced-engine `ManaTempo` seam on action+mana boards.
- `primary_black_mana_bridge_ply20`: retained black injected mana-bridge seam on shipping baseline `l4,1;l5,0;mb`.
- `primary_black_spirit_bridge_ply19`: retained black injected spirit-bridge seam on the same shipping baseline.
- `primary_black_negative_deny_ply4`: retained early-black negative-deny selector seam.
- Closed surfaces that should stay closed: `primary_spirit_setup`, `primary_pvs_sensitive_search`, and `primary_black_reliability_opening_3_ply4`.

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
- Kill any line that clears retained fixtures but does not move direct duel evidence against `shipping_pro_search`.
- Do not reopen archive profiles or retired branch families without a brand-new shared hypothesis.
- Do not treat hotspot output or one replay seed as production proof without a retained foothold.
- Wrapper-only reroutes and local fallback widening saturate quickly; shared selector/search changes are the real frontier.
- Speed-only wins are not enough once quality is flat.
