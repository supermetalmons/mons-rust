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
- Current diagnostic hypothesis: there is no source-level selector yet. The broad reset route scan found no clean low-fragmentation route; the filtered safety/progress and `engine_post_search` routes are retired as source evidence because their matching states split by policy, color, branch, first move, and advisor status. Outcome Corpus V2 now has a log summarizer with `corpus_decision` / `next_action` / `source_blocker`, multi-log `log_rollup` including rollup-level decisions, compact `coverage_gap_entries`, and a true total-state cap; use those before reading raw matrix logs.
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

Current next sequence: do not run another selector or widening experiment first. Use the compact active Pro coverage-gap entry to run a forced-root oracle on the earliest no-policy divergence board. This tests whether the current root set already contains a winning move before designing a new policy/root feature; it is diagnostic only, not runtime source permission.

```sh
SMART_PRO_FORCED_ROOT_ORACLE_FEN='0 0 w 0 0 1 0 0 3 n05d0xa0xn04/n08e0xn02/n06s0xn04/n04y0xn03xxmn02/xxmxxmn08xxm/xxQxxmn03xxUn03xxMxxQ/xxMxxMn07xxMxxM/n11/n07Y0xn03/n04A0xS0xn05/n03E0xn01D0xn05 8' \
SMART_PRO_FORCED_ROOT_ORACLE_CONTINUATION=frontier_pro_v2_guarded \
SMART_PRO_FORCED_ROOT_ORACLE_START_PLY=7 \
cargo test --release --lib smart_automove_pro_forced_root_oracle_probe -- --ignored --nocapture
```

If the oracle finds no winning forced root, preserve the no-policy gap and move to a new root feature or broader no-policy corpus view. If it finds a winning root, inspect whether the root repeats below policy labels before creating a test-only ProV4/root-feature candidate.
Read `PRO_POLICY_MATRIX_GLOBAL_MECHANISM_ROUTE` by state counts only after compact coverage-gap and oracle evidence, then inspect `candidate_only_policy_count`, `candidate_only_branch_count`, and `candidate_only_pair_count`. A clean route that is fragmented on those dimensions is diagnostic only. Only a clean route with positive state-level separation and low fragmentation should earn a narrow record/probe rerun.
Read `PRO_POLICY_MATRIX_GLOBAL_ROUTE_RECOMMENDATION` before raw route lines. `build_outcome_corpus_v2` means preserve harness/postprocess work and do not write a runtime selector.
Read `PRO_POLICY_MATRIX_GLOBAL_ROUTE_BUCKET` next. Its bucketed shortlist should replace manual grepping through all raw route lines.
For focused record inspection, copy the bucket `key` into `SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER`. The filtered records are for grouping/postprocess design; they do not override route-fragmentation no-go rules.
Read `PRO_POLICY_MATRIX_RECORD_FILTER_SUMMARY` and `PRO_POLICY_MATRIX_RECORD_FILTER_DETAIL` before raw records; if the detail rows still have multiple policies, branches, or first-move pairs, keep the work in postprocess/harness.
When a log exists, read the summarizer's `corpus_decision`, `next_action`, `source_blocker`, `route_permission`, and per-filter `permission` fields first. `coverage_gap`, `baseline_save_risk`, `singleton_no_source`, `no_candidate_route`, `postprocess_only`, or `fragmented_no_source` means update knowledge and keep runtime source untouched.
Use `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT` for global caps. `SMART_PRO_POLICY_MATRIX_STATE_LIMIT` is per panel/duel and can still fan out across the full panel/budget matrix.

## Major Idea Backlog

### 1. Outcome Corpus V2 Workbench

Structural change: make corpus output a persistent, queryable artifact instead of stdout that humans manually scan. Emit normalized JSONL records for each policy decision, then add a postprocessor that ranks mechanisms by candidate-only wins, baseline-better saves, no-policy gaps, cross-budget stability, cost, and state-limit confidence.

First proof: use the retained reset portfolio and current `pro-policy-outcome-corpus` feed. Add only harness/postprocess code until the report can answer "which mechanism is clean enough to become a feature?" without reading raw logs. Current progress: global outcome-corpus output now includes state-aware `PRO_POLICY_MATRIX_GLOBAL_MECHANISM_ROUTE` labels, route fragmentation counts, `PRO_POLICY_MATRIX_GLOBAL_ROUTE_RECOMMENDATION`, and bucketed `PRO_POLICY_MATRIX_GLOBAL_ROUTE_BUCKET` shortlists; record output includes `mechanism_axes` / `baseline_better_mechanism_axes`, `SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER`, `PRO_POLICY_MATRIX_RECORD_FILTER_SUMMARY`, and capped `PRO_POLICY_MATRIX_RECORD_FILTER_DETAIL` rows so route lines can be matched back to divergences without dumping or manually counting the full corpus. `scripts/summarize-automove-policy-matrix-log.py` now turns logged policy-matrix JSON lines into a digest with `corpus_decision`, `next_action`, `source_blocker`, route and filter permissions, compact `coverage_gap_entries`, and a multi-log `log_rollup` with rollup-level decision fields; `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT` provides a true global cap.

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
- The focused active Pro/Fast record-filter run for the safety/progress route also stayed no-source: `candidate_signal_routes=82`, `clean_low_fragmentation_routes=0`, `clean_fragmented_routes=9`, `baseline_risk_routes=6`. Its best clean key had four matching candidate-only records across Pro/Fast, black/white, three policies, two branch transitions, four first-move pairs, and zero baseline-better saves, so it remains a postprocess fixture only.
- The safety/progress detail rerun confirmed the no-source split in the retained summarizer: `route_permission=postprocess_only`, route recommendation `build_outcome_corpus_v2`, filter permission `fragmented_no_source`, and candidate/branch/pair counts `3 / 2 / 4`. Detail counts were shipping-control `2`, no-selected `1`, full-scored `1`; Pro/Fast `3 / 1`; outer-edge/alternating `3 / 1`; first-move pairs all singleton.
- A broad all-panel/all-budget reset digest with only `SMART_PRO_POLICY_MATRIX_STATE_LIMIT=2` was stopped after about fourteen minutes because that cap is per panel/duel. The retained harness fix is `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT`.
- A total-capped active Fast digest over the full reset portfolio completed successfully with two total states and stayed no-source: `baseline_save_risk_only`, `candidate_signal_routes=19`, `clean_low_fragmentation_routes=0`, `clean_fragmented_routes=0`, `baseline_risk_routes=1`. The only baseline-risk route was broad zero-window safe exact pressure, with candidate-only states `1` and baseline-better states `1`.
- A total-capped active Pro digest over the full reset portfolio completed successfully with two total states and stayed no-source: summarizer `corpus_decision=coverage_gap`, route recommendation `singleton_candidate_routes`, `candidate_only_wins=1`, `no_policy_wins=1`, and zero clean routes. The only candidate route was zero-window safe exact pressure as singleton evidence split across two policies and two first-move pairs.
- A total-capped active Normal digest over the full reset portfolio completed successfully with two total states and stayed no-source: summarizer `corpus_decision=no_candidate_route`, `next_action=try_next_slice`, route recommendation `no_candidate_route`, `candidate_only_wins=0`, `no_policy_wins=0`, and `candidate_signal_routes=0`. Guarded shared both checked wins, while full-scored reply guard only emitted baseline-better save-risk rows on one outer-edge white state.
- A total-capped sampled Normal digest over the full reset portfolio completed successfully with two total states and stayed no-source: summarizer `corpus_decision=singleton_no_source`, `next_action=widen_or_archive_singleton`, route recommendation `singleton_candidate_routes`, `candidate_only_wins=1`, `no_policy_wins=0`, and `candidate_signal_routes=14`. The top route was one white `inner_wedge_mana_rows` state split across no-selected-followup and shipping-control, two branch transitions, and two first-move pairs.
- A total-capped sampled Fast digest over the full reset portfolio completed successfully with two total states and stayed no-source: summarizer `corpus_decision=no_candidate_route`, `next_action=try_next_slice`, route recommendation `no_candidate_route`, `candidate_only_wins=0`, `no_policy_wins=0`, and `candidate_signal_routes=0`. Guarded shared both checked wins; other policies only emitted baseline-better pressure on split-flank states, including zero-window safe exact pressure.
- A total-capped sampled Pro digest over the full reset portfolio completed successfully with two total states and stayed no-source: summarizer `corpus_decision=no_candidate_route`, `next_action=try_next_slice`, route recommendation `no_candidate_route`, `candidate_only_wins=0`, `no_policy_wins=0`, and `candidate_signal_routes=0`. Guarded shared both checked inner-wedge wins; other policies only emitted baseline-better pressure, led by zero-window safe exact pressure with `baseline_better_games=5` and `baseline_better_states=2`.
- A true-global total-capped digest over the full reset portfolio completed successfully with six total states and stayed no-source: summarizer `corpus_decision=baseline_save_risk`, `next_action=avoid_selector`, `source_blocker.kind=baseline_save_risk`, and route recommendation `baseline_save_risk_only`. The blocker was `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe`, with candidate-only states `1` and baseline-better states `3`.
- A postprocess rollup validation reran the same true-global cap and stayed no-source with the same blocker. The retained summarizer now emits `log_rollup` for multi-log inputs; the smoke check reported repeated `baseline_save_risk` / `avoid_selector` decisions and a repeated exact-pressure source blocker when the capped log was passed twice.
- A sampled/active Pro-budget rollup over the full reset portfolio stayed no-source. Sampled Pro was guarded-covered (`corpus_decision=no_candidate_route`, guarded wins `2`, no candidate-only wins), while active Pro was `coverage_gap` (`candidate_only_wins=1`, `no_policy_wins=1`). The retained summarizer now emits `rollup_decision=coverage_gap`, `rollup_next_action=add_policy_or_root_feature`, and `rollup_permission=no_source` for that mixed no-source shape.
- The focused active Pro coverage-gap record run showed the active `outer_edge_mana_rows` fixture: candidate black had `shipping_pro_search_control` and `frontier_pro_v3_full_scored_reply_guard` wins, but the same opening as candidate white was a true no-policy state where every current portfolio policy lost. The next postprocess gap is compacting `portfolio_class=no_policy_win` corpus records into per-state coverage-gap summaries.
- The coverage-gap compact view is now implemented and validated on the focused active Pro fixture. It emits `coverage_gap_entry_count=1` and identifies the candidate-white `outer_edge_mana_rows` no-policy state with all seven policies losing, `first_diff_count=3`, and an earliest listed divergence at first-diff ply `7` on board `0 0 w 0 0 1 0 0 3 n05d0xa0xn04/n08e0xn02/n06s0xn04/n04y0xn03xxmn02/xxmxxmn08xxm/xxQxxmn03xxUn03xxMxxQ/xxMxxMn07xxMxxM/n11/n07Y0xn03/n04A0xS0xn05/n03E0xn01D0xn05 8`.
- Raw ProV2, no-selected-followup, full-scored reply guard, no-low-budget, alternating-white, and white-opening utility policies are diagnostic components, not retained challengers.
- Root-origin and continuation-probe ProV4 attempts are retired unless they add a new discriminator below current score, rank, family, safety, progress, and `TurnEngineUtility` fields.
- Future source-bearing work should be one of: Outcome Corpus V2, a test-only ProV4 unified root policy, or a corpus-calibrated utility feature.
- The fastest path to a promotable automove is probably not another named ProV3 component; it is a shorter evidence loop that ranks mechanisms, then one larger root/utility change measured on the dashboard before retained runtime code.

## Latest Gate Snapshot

- Date: `2026-04-29`
- Shipping decision: public Pro remains on `frontier_pro_v2_guarded`.
- Release containment: public `Pro` dispatch still routes through retained runtime code; `automove_experiments` remains under `#[cfg(test)]`.
- Latest retained package direction: no runtime source retained from recent structural reset work.
- Latest reset evidence: the focused route-filter scans have oracle coverage but no source permission. The safety/progress and `engine_post_search` routes remain fragmented across policy, branch, color, budget, and first-move pair; the latest sampled/active Pro-budget rollup was no-source with `rollup_decision=coverage_gap`, and the focused active Pro compact view exposes a no-policy state where every current policy loses. The retained change is `coverage_gap_entries` in the Outcome Corpus V2 summarizer; no runtime source is retained from this reset pass.

## Session End Checklist

1. Update this file with the current state and exactly one next command sequence.
2. Move durable rules into `docs/automove-knowledge.md`.
3. Move retired wave detail into `docs/automove-archive.md`.
4. Clean disposable artifacts after validation.
5. Leave exactly one clear next hypothesis, or explicitly record that there is no live challenger.
