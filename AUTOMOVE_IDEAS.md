# Automove Ideas

This is the live decision board for automove work. Keep it decision-oriented; move probe diaries to the archive.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the operator flow, `docs/automove-major-reset-plan.md` for the current reset handoff, `docs/automove-knowledge.md` for durable rules, and `docs/automove-archive.md` for retired wave detail.

## Current State

- Public Pro routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` remains the retained search-only baseline.
- The live experiment surface is Pro-only and multi-variant.
- Retained profiles are only `shipping_pro_search` and `frontier_pro_v2_guarded`.
- The current mode is `structural-reset`.
- There is no live runtime hypothesis and no promotable challenger.
- Current diagnostic hypothesis: there is no source-level selector yet. The broad reset route scan found no clean low-fragmentation route; the active-only `engine_post_search` route is retired as source evidence because its `pre_family=ManaTempo head_family=Some(SpiritImpact)` states split by policy, color, branch, first move, and advisor status.
- Recent stagnation is from the loop where local selectors are cheap to invent, broad promotion proof is expensive, and singleton-heavy corpus evidence still leaves room to try "one more gate".
- Do not reopen archived profiles, archived seams, archived stages, or pruned sweep candidates as direct experiment targets.

## Reset Portfolio

The retained reset portfolio for policy corpus and outcome corpus work is:

```text
frontier_pro_v2_guarded,
frontier_pro_v3_alternating_white_edge_mana,
frontier_pro_v3_white_opening_utility_mana,
shipping_pro_search_control,
frontier_pro_v2_raw,
frontier_pro_v2_no_selected_followup_projection,
frontier_pro_v3_full_scored_reply_guard,
frontier_pro_v2_no_low_budget_guard
```

The following stale test-only sweep candidates are intentionally pruned from the active runner surface:

```text
frontier_pro_v2_no_late_black_fallback,
frontier_pro_v2_head_rerank,
frontier_pro_v2_no_spirit_family,
frontier_pro_v2_no_mid_tactical_guard,
frontier_pro_v2_expansion_224
```

Their historical no-go evidence remains in `docs/automove-knowledge.md` and `docs/automove-archive.md`.

## Next Command Sequence

Current next sequence: use the record-axis filter summary on the current top clean-fragmented route before any broader postprocessor work. This is Outcome Corpus V2 validation only, not runtime source permission.

```sh
SMART_PRO_POLICY_MATRIX_PANEL_FILTER=active_blockers \
SMART_PRO_POLICY_MATRIX_DUEL_FILTER=vs_shipping_pro,vs_shipping_fast \
SMART_PRO_POLICY_MATRIX_STATE_LIMIT=3 \
SMART_PRO_POLICY_MATRIX_INCLUDE_PORTFOLIO_MECHANISM_CLASS=true \
SMART_PRO_POLICY_MATRIX_INCLUDE_CORPUS_RECORDS=true \
SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER='axis=safety_progress baseline_safety=safe baseline_progress=safe_step_progress winner_safety=safe winner_progress=spirit_development' \
./scripts/run-automove-experiment.sh pro-policy-outcome-corpus frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,frontier_pro_v3_white_opening_utility_mana,shipping_pro_search_control,frontier_pro_v2_raw,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard,frontier_pro_v2_no_low_budget_guard shipping_pro_search
```

Read `PRO_POLICY_MATRIX_GLOBAL_MECHANISM_ROUTE` by state counts first, then inspect `candidate_only_policy_count`, `candidate_only_branch_count`, and `candidate_only_pair_count`. A clean route that is fragmented on those dimensions is diagnostic only. Only a clean route with positive state-level separation and low fragmentation should earn a narrow record/probe rerun.
Read `PRO_POLICY_MATRIX_GLOBAL_ROUTE_RECOMMENDATION` before raw route lines. `build_outcome_corpus_v2` means preserve harness/postprocess work and do not write a runtime selector.
Read `PRO_POLICY_MATRIX_GLOBAL_ROUTE_BUCKET` next. Its bucketed shortlist should replace manual grepping through all raw route lines.
For focused record inspection, copy the bucket `key` into `SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER`. The filtered records are for grouping/postprocess design; they do not override route-fragmentation no-go rules.
Read `PRO_POLICY_MATRIX_RECORD_FILTER_SUMMARY` before raw records; if it still has multiple policies, branches, or first-move pairs, keep the work in postprocess/harness.

## Major Idea Backlog

### 1. Outcome Corpus V2 Workbench

Structural change: make corpus output a persistent, queryable artifact instead of stdout that humans manually scan. Emit normalized JSONL records for each policy decision, then add a postprocessor that ranks mechanisms by candidate-only wins, baseline-better saves, no-policy gaps, cross-budget stability, cost, and state-limit confidence.

First proof: use the retained reset portfolio and current `pro-policy-outcome-corpus` feed. Add only harness/postprocess code until the report can answer "which mechanism is clean enough to become a feature?" without reading raw logs. Current progress: global outcome-corpus output now includes state-aware `PRO_POLICY_MATRIX_GLOBAL_MECHANISM_ROUTE` labels, route fragmentation counts, `PRO_POLICY_MATRIX_GLOBAL_ROUTE_RECOMMENDATION`, and bucketed `PRO_POLICY_MATRIX_GLOBAL_ROUTE_BUCKET` shortlists; record output includes `mechanism_axes` / `baseline_better_mechanism_axes`, `SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER`, and `PRO_POLICY_MATRIX_RECORD_FILTER_SUMMARY` so route lines can be matched back to divergences without dumping or manually counting the full corpus.

Promotion signal: one mechanism repeats across deduplicated states in at least two panels or opponent budgets, has positive state-level separation after baseline saves, and points to a feature below policy labels.

Kill signal: repeated keys remain exact-context, pair, branch, or broad `axis=exact_pressure` classes with comparable baseline-better counts.

### 2. Test-Only ProV4 Unified Root Pool

Structural change: stop treating guarded, head, pre-accept, advisor, preserved, raw, shipping-control, and ablation outputs as separate routing branches. Build a test-only `ProV4RootCandidate` pool with origin labels, root rank/score/family, advisor state, head/pre-accept/legacy status, liveness, reply-risk summary, exact pressure, utility axes, and continuation features. Select from that pool with one comparator.

First proof: implement in diagnostics/sweep only. Register one candidate after the pool can explain current guarded decisions and produce a corpus record for every considered root.

Promotion signal: dashboard is strong on sampled and active panels before attribution, and nonwins share a below-branch mechanism rather than a policy label.

Kill signal: the comparator only reorders existing score/rank/family/safety/progress/`TurnEngineUtility` fields, or it improves sampled Pro while rotating Normal/Fast or active blockers.

### 3. Corpus-Calibrated Utility Feature

Structural change: use the policy portfolio as supervision to add one measured utility feature below selectors. Candidate feature families are continuation stability after the selected root, root preservation/omission as a soft prior, reply-risk floor interacted with progress/setup class, budget-invariant safety deltas, and timing pressure for roots that must enter before first printed divergence.

First proof: extend corpus records with the missing feature value for baseline, guarded, and winning-policy roots; only then add a test-only sweep candidate that changes selection through that feature.

Promotion signal: the feature separates candidate-only wins from baseline saves on both sampled and active evidence, then survives `pro-promotion-dashboard`.

Kill signal: the feature is just another broad utility gate, fires on many guarded saves, or raises cost before strength moves.

### 4. Decision-Timing And Continuation Stability

Structural change: model "when the winning root must enter" as a first-class feature. Current first-divergence records often show the winning policy too late; add probes that compare root choice at selected, pre-accept, head, reply-risk approval, and final output with a cheap next-turn continuation-stability score.

First proof: augment outcome-corpus records with decision-stage timing and cached continuation stability, then re-rank only boards where portfolio winners disagree with guarded.

Promotion signal: the same timing/continuation class explains repairs across more than one variant or budget without hitting known guarded saves.

Kill signal: timing labels are singleton-heavy or collapse to branch labels like `frontier_execute`, `head`, or `pre_accept`.

### 5. Cross-Budget Invariant Mechanism Gate

Structural change: make cross-budget stability a source permission gate, not a follow-up after a selector is already attractive. A proposed mechanism must show all-budget repair or non-regressing repair on the same openings before it can become runtime code.

First proof: join outcome-corpus records by panel, seed tag, opening index, variant, side, and first-divergence ply across Pro/Normal/Fast opponents. Surface budget conflicts directly in the global report.

Promotion signal: candidate-only mechanisms are stable or non-regressing across budgets and do not create `baseline_save_risk`.

Kill signal: a mechanism is active-only, sampled-only, or budget-conflicted even if one panel looks strong.

### 6. Faster Structural Scout Defaults

Structural change: make broad reset scans cheap and decisive by default. Use global-only summaries, state caps, fast-fail dashboard routing, and mechanism-separation tables before printing long record streams.

First proof: update the scout flow so reset mode produces one top-level recommendation: widen a clean mechanism, build a new root feature, run a ProV4 candidate dashboard, or record no-go.

Promotion signal: future sessions stop with one of those decisions instead of another ambiguous matrix dump.

Kill signal: the scout still requires manual interpretation of hundreds of lines before choosing the next action.

### 7. Candidate Lifecycle And Registry Hygiene

Structural change: require every new sweep candidate to declare its mechanism, expected invariant, risk rows, and kill condition next to its registration. Keep pruned candidate IDs archived and force new names for materially different ideas.

First proof: add a small candidate metadata table in the diagnostic harness or docs, then have scripts print the metadata before running dashboards/corpus probes.

Promotion signal: stale ablations stop reappearing as "new" experiments, and failed candidates archive into knowledge instead of accumulating in the live runner.

Kill signal: the metadata becomes a diary or duplicates `docs/automove-archive.md` instead of changing run decisions.

### 8. Cheap Active-Blocker Shadow Panel

Structural change: keep active blockers as a compact shadow panel for architecture triage, not as retained promotion evidence or exact-selector fuel. It should catch obvious sampled-only false positives before full dashboard spend.

First proof: derive a small, deterministic active-blocker sample from existing dashboard seeds and run it before expensive corpus widening.

Promotion signal: the shadow panel kills bad ideas earlier while final decisions still rely on canonical sampled, active, and confirm gates.

Kill signal: it encourages exact-board patching or diverges from the canonical active-blocker dashboard.

For a new test-only ProV4/root-policy candidate, register it as a sweep candidate first, then use the structural scout corpus path from `HOW_TO_ITERATE_ON_AUTOMOVE.md`; that is not the current next command while no candidate exists.

## Stoplight Rules

- `promotable_shape`: run confirm before retaining runtime source.
- `sampled_only` or `active_only`: no runtime retention; use decision records only if one miss is explainable.
- `cost_blocked`: kill or redesign before tuning strength.
- `coverage_gap`: add a policy/root feature before selector work.
- `baseline_save_risk`: do not encode a selector until baseline saves separate from candidate wins by mechanism.
- `mixed_delta` or `regression_only`: kill the candidate as a direct source path.
- `singleton_selector_pressure`: oracle coverage exists, but there is no selector mechanism.
- `repeated_winner_policy`: too coarse by itself; require repeated context, pair, or mechanism with clean save separation.
- `repeated_mechanism_class` at count 2 is routing evidence only, not runtime permission.

## Current No-Go Summary

- Static selectors over existing policy labels, exact contexts, branches, variants, and first moves are retired unless a new corpus feature changes the evidence.
- The expanded reset portfolio has shown useful oracle coverage, but repeated exact winner context/pair evidence has stayed singleton-heavy.
- Broad zero-window safe-pressure classes are contaminated by baseline-better saves and cannot justify runtime selectors.
- The latest broad state-aware route summary confirmed that zero-window safe-pressure remains `baseline_save_risk` despite positive raw emissions; it had candidate-only games `10`, baseline-better games `5`, candidate-only states `5`, and baseline-better states `3`.
- Cleaner route signals are timing/stage-level and still diagnostic only. The top clean active route was `engine_post_search` with `pre_family=ManaTempo head_family=Some(SpiritImpact)`: candidate-only games `3`, baseline-better games `0`, candidate-only states `3`, spanning active Pro/Fast only. Focused records showed three winning policies, both colors, two branch transitions, and three first-move pairs, so it is retired as source permission.
- The latest full reset portfolio route scan found zero clean low-fragmentation routes. All clean repeated routes were fragmented by policy, branch, or first-move pair, so the retained next step is Outcome Corpus V2/postprocess work rather than runtime selection.
- The latest route recommendation scan emitted `build_outcome_corpus_v2`: `candidate_signal_routes=109`, `clean_low_fragmentation_routes=0`, `clean_fragmented_routes=8`, and `baseline_risk_routes=14`. The best clean route was still the active-only fragmented `engine_post_search` stage route; best baseline risk was still zero-window safe exact pressure.
- The record-axis filter was smoke-validated on the active Fast slice for the top `engine_post_search` bucket route. It printed only the matching `PRO_POLICY_MATRIX_CORPUS_RECORD` / `PRO_POLICY_MATRIX_RECORD` pair and preserved the existing summary/recommendation output.
- The focused active Pro/Fast record-filter run for `engine_post_search` stayed no-source: `candidate_signal_routes=82`, `clean_low_fragmentation_routes=0`, `clean_fragmented_routes=9`, `baseline_risk_routes=6`. The selected stage route had four matching corpus records split across Pro/Fast, black/white, three policies, two branch transitions, and four first-move pairs; it remains a postprocess fixture only.
- Raw ProV2, no-selected-followup, full-scored reply guard, no-low-budget, alternating-white, and white-opening utility policies are diagnostic components, not retained challengers.
- Root-origin and continuation-probe ProV4 attempts are retired unless they add a new discriminator below current score, rank, family, safety, progress, and `TurnEngineUtility` fields.
- Future source-bearing work should be one of: Outcome Corpus V2, a test-only ProV4 unified root policy, or a corpus-calibrated utility feature.
- The fastest path to a promotable automove is probably not another named ProV3 component; it is a shorter evidence loop that ranks mechanisms, then one larger root/utility change measured on the dashboard before retained runtime code.

## Latest Gate Snapshot

- Date: `2026-04-29`
- Shipping decision: public Pro remains on `frontier_pro_v2_guarded`.
- Release containment: public `Pro` dispatch still routes through retained runtime code; `automove_experiments` remains under `#[cfg(test)]`.
- Latest retained package direction: no runtime source retained from recent structural reset work.
- Latest reset evidence: the focused route-filter scan has oracle coverage but no source permission. The `engine_post_search` route remains fragmented across policy, branch, color, budget, and first-move pair. The retained source change is harness-only record-filter summary output.

## Session End Checklist

1. Update this file with the current state and exactly one next command sequence.
2. Move durable rules into `docs/automove-knowledge.md`.
3. Move retired wave detail into `docs/automove-archive.md`.
4. Clean disposable artifacts after validation.
5. Leave exactly one clear next hypothesis, or explicitly record that there is no live challenger.
