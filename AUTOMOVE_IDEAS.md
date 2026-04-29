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
- Current diagnostic hypothesis: there is no source-level selector yet. The broad reset route scan found no clean low-fragmentation route; the filtered safety/progress and `engine_post_search` routes are retired as source evidence because their matching states split by policy, color, branch, first move, and advisor status. Outcome Corpus V2 now has a log summarizer with `corpus_decision` / `next_action` / `source_blocker`, multi-log `log_rollup` including rollup-level decisions, compact `coverage_gap_entries`, same-opening sibling summaries, record-level `corpus_axis_summary.top_axes_by_decision`, per-token `axis_filter_matches`, decision timing / continuation-stability axes, root-preservation axes, reply-risk / followup-floor axes, detailed root-safety axes, cross-budget axis source-status summaries, normalized JSONL workbench export, JSONL blocked-axis report, JSONL family-overlap drilldown, JSONL state-level discriminator drilldown, JSONL axis-token discriminator drilldown, and a true total-state cap. A corrected-horizon forced-root oracle on the active Pro no-policy coverage-gap boards found winning roots in the current root set, but enriched forced-root safety/reply/advisor/path axes still found no repeated winner-vs-nonwinner separator. The widened active Pro same-opening pairing check stayed singleton, the sampled eight-state record-bearing slice stayed baseline-save-risk, the explicit active Pro axis summary stayed coverage-gap, the focused sampled Pro axis-filter check killed the two active repeated-candidate leads, the first sampled/active timing-axis pass stayed no-source, the first active cross-budget axis validation classified zero-window exact pressure plus no-rejoin continuation as budget-conflicted rather than stable repairs, the two-state active cross-budget widening produced zero source-candidate rollups, the sampled cross-budget source-status pass also produced zero source-candidate rollups, the sampled+active root-preservation axis smoke produced zero source-candidate rollups, the sampled+active reply-floor axis smoke produced zero source-candidate rollups, the sampled+active root-safety detail smoke produced zero source-candidate rollups, the enriched forced-root feature-axis digest stayed `fragmented_root_features`, the first JSONL workbench smoke stayed `coverage_gap` with no source-candidate rollups, the first JSONL blocked-axis report found zero source-candidate axes, the first JSONL family-overlap drilldown found shared but contaminated candidate states, the JSONL state discriminator found no unique candidate family inside either target state, and the first JSONL axis-token discriminator found a repeated vulnerable-baseline safety/progress token lead but still no source-candidate rollups. Work remains Outcome Corpus V2 feature extraction rather than runtime selectors.
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

Current next sequence: do not write runtime selector code and do not rerun current-axis, root-preservation, reply-floor, root-safety, forced-root feature-axis, family-overlap, state-discriminator, or broad all-family token passes. The next useful work is a focused JSONL axis-token widening for the vulnerable-baseline safety/progress lead found by the token discriminator: `baseline_detail=vulnerable`, `baseline_safety=vulnerable`, and `baseline_signal=vulnerable_progress`. Use only `root_safety_detail` and `safety_progress` token families first. If the lead remains repeated, clean from baseline/no-policy rows, and less fragmented after widening, inspect it as a candidate below-policy feature; if it picks up save/gap contamination or stays singleton/fragmented, archive it and move to a new feature source.

```sh
python3 -m py_compile scripts/summarize-automove-outcome-jsonl.py

SMART_PRO_POLICY_MATRIX_PANEL_FILTER=sampled,active_blockers \
SMART_PRO_POLICY_MATRIX_DUEL_FILTER=vs_shipping_pro,vs_shipping_normal,vs_shipping_fast \
SMART_PRO_POLICY_MATRIX_STATE_LIMIT=2 \
SMART_PRO_POLICY_MATRIX_GLOBAL_ONLY=false \
SMART_PRO_POLICY_MATRIX_INCLUDE_CORPUS_RECORDS=true \
SMART_PRO_POLICY_MATRIX_INCLUDE_PORTFOLIO_MECHANISM_CLASS=true \
SMART_PRO_POLICY_MATRIX_ROUTE_BUCKET_LIMIT=5 \
SMART_PRO_POLICY_MATRIX_MAX_PLIES=56 \
SMART_PRO_POLICY_MATRIX_CANDIDATES=frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,frontier_pro_v3_white_opening_utility_mana,shipping_pro_search_control,frontier_pro_v2_raw,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard,frontier_pro_v2_no_low_budget_guard \
cargo test --release --lib smart_automove_pro_policy_matrix_probe -- --ignored --nocapture \
  > /tmp/automove-outcome-corpus-workbench-smoke.log 2>&1

scripts/summarize-automove-policy-matrix-log.py \
  --compact \
  --jsonl-out /tmp/automove-outcome-corpus-workbench-smoke.jsonl \
  /tmp/automove-outcome-corpus-workbench-smoke.log \
  > /tmp/automove-outcome-corpus-workbench-smoke-summary.json

scripts/summarize-automove-outcome-jsonl.py \
  --compact \
  --limit 20 \
  --state-axis-limit 20 \
  --token-families root_safety_detail,safety_progress \
  /tmp/automove-outcome-corpus-workbench-smoke.jsonl
```

Runtime source stays untouched unless a new below-policy axis creates non-empty `source_candidate_rollups` on sampled evidence and a future active rerun also clears baseline-save, coverage-gap, singleton, and fragmentation statuses.
Use `--jsonl-out` on `scripts/summarize-automove-policy-matrix-log.py` to create the normalized workbench artifact. It emits `policy_decision`, `policy_axis`, `corpus_axis_summary`, `cross_budget_axis_state`, `cross_budget_axis_rollup`, and `coverage_gap_state` rows; prefer those rows over manual stdout scanning when comparing candidate-better, baseline-better, no-policy, shared, cross-budget, and coverage-gap evidence.
Use `scripts/summarize-automove-outcome-jsonl.py <export.jsonl>` after export to rank source candidates, blocked candidate-bearing axes, blocker counts, blocked axis-family rollups, default family-overlap drilldown, state-discriminator rows, and axis-token discriminator rows directly from normalized rows. Use `--token-families` when a focused token lead should be separated from the broad contaminated family set.
Read `PRO_POLICY_MATRIX_GLOBAL_MECHANISM_ROUTE` by state counts only after compact coverage-gap and oracle evidence, then inspect `candidate_only_policy_count`, `candidate_only_branch_count`, and `candidate_only_pair_count`. A clean route that is fragmented on those dimensions is diagnostic only. Only a clean route with positive state-level separation and low fragmentation should earn a narrow record/probe rerun.
Read `PRO_POLICY_MATRIX_GLOBAL_ROUTE_RECOMMENDATION` before raw route lines. `build_outcome_corpus_v2` means preserve harness/postprocess work and do not write a runtime selector.
Read `PRO_POLICY_MATRIX_GLOBAL_ROUTE_BUCKET` next. Its bucketed shortlist should replace manual grepping through all raw route lines.
For focused record inspection, copy the bucket `key` into `SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER`. The filtered records are for grouping/postprocess design; they do not override route-fragmentation no-go rules.
Read `PRO_POLICY_MATRIX_RECORD_FILTER_SUMMARY`, summarizer `axis_filter_matches`, and `PRO_POLICY_MATRIX_RECORD_FILTER_DETAIL` before raw records; if the detail rows still have multiple policies, branches, or first-move pairs, keep the work in postprocess/harness.
When a log exists, read the summarizer's `corpus_decision`, `next_action`, `source_blocker`, `route_permission`, and per-filter `permission` fields first. `coverage_gap`, `baseline_save_risk`, `singleton_no_source`, `no_candidate_route`, `postprocess_only`, or `fragmented_no_source` means update knowledge and keep runtime source untouched.
Use `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT` for global caps. `SMART_PRO_POLICY_MATRIX_STATE_LIMIT` is per panel/duel and can still fan out across the full panel/budget matrix.

## Major Idea Backlog

### 1. Outcome Corpus V2 Workbench

Structural change: make corpus output a persistent, queryable artifact instead of stdout that humans manually scan. Emit normalized JSONL records for each policy decision, then add a postprocessor that ranks mechanisms by candidate-only wins, baseline-better saves, no-policy gaps, cross-budget stability, cost, and state-limit confidence.

First proof: use the retained reset portfolio and current `pro-policy-outcome-corpus` feed. Add only harness/postprocess code until the report can answer "which mechanism is clean enough to become a feature?" without reading raw logs. Current progress: global outcome-corpus output now includes state-aware `PRO_POLICY_MATRIX_GLOBAL_MECHANISM_ROUTE` labels, route fragmentation counts, `PRO_POLICY_MATRIX_GLOBAL_ROUTE_RECOMMENDATION`, and bucketed `PRO_POLICY_MATRIX_GLOBAL_ROUTE_BUCKET` shortlists; record output includes `mechanism_axes` / `baseline_better_mechanism_axes`, `timing_continuation_axes`, `SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER`, `PRO_POLICY_MATRIX_RECORD_FILTER_SUMMARY`, and capped `PRO_POLICY_MATRIX_RECORD_FILTER_DETAIL` rows so route lines can be matched back to divergences without dumping or manually counting the full corpus. `scripts/summarize-automove-policy-matrix-log.py` now turns logged policy-matrix JSON lines into a digest with `corpus_decision`, `next_action`, `source_blocker`, route and filter permissions, compact `coverage_gap_entries` with same-opening sibling state summaries, record-level `corpus_axis_summary.top_axes_by_decision`, cross-budget `cross_budget_axis_summary`, per-token `axis_filter_matches`, a multi-log `log_rollup` with rollup-level decision fields, and normalized JSONL export; `scripts/summarize-automove-outcome-jsonl.py` ranks blocked candidate-bearing axes, family rollups, family overlap, state-level discriminators, and axis-token discriminators from that export; `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT` provides a true global cap. `scripts/summarize-automove-forced-root-oracle-log.py` now summarizes forced-root oracle logs and checks whether repeated winner axes also appear on losing roots.

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
- The active Pro no-policy coverage-gap boards are root-covered under corrected horizon but not selector-covered: guarded continuation found `7/16`, `4/16`, and `1/17` winning roots on the checked ply-7, ply-20, and ply-40 boards, while the winning ranks and families did not collapse to one selector feature.
- The forced-root digest confirmed the same no-source shape across all three checked boards: `summary_count=3`, `tested_roots=49`, `wins=12`, `losses=37`, `promising_repeated_axes=[]`, and `oracle_decision=fragmented_root_features`. Even `rank_band=rank8_plus` repeated across all winner groups but also appeared on `16` losing roots.
- The active Pro Outcome Corpus V2 rerun with two total states stayed `coverage_gap` with `candidate_only_wins=1` and `no_policy_wins=1`, and `same_opening_sibling_states` showed the candidate-white no-policy entry paired with a candidate-black sibling from the same opening where every corpus record is `candidate_only_win` and the winning policies are `shipping_pro_search_control` plus `frontier_pro_v3_full_scored_reply_guard`.
- The active Pro four-state widening kept that same-opening cross-side pairing singleton. It stayed `coverage_gap` with `total_games=4`, `candidate_only_wins=2`, `no_policy_wins=1`, `shared_wins=1`, `coverage_gap_entry_count=1`, and only the original `outer_edge_mana_rows` no-policy entry with one same-opening sibling. Treat the pairing as archived singleton evidence, not source permission.
- The unfiltered eight-state record-bearing Outcome Corpus V2 slice did not create cross-panel evidence because the true global cap was consumed by sampled `vs_shipping_pro` states. It stayed no-source with `corpus_decision=baseline_save_risk`, `candidate_only_wins=2`, `shared_wins=6`, `no_policy_wins=0`, and zero clean routes. The new `corpus_axis_summary` shows the top record-level candidate-better axis is still `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe`, with candidate-better states `2` but baseline-better states `4`, so exact pressure remains a selector kill.
- The explicit active-blocker Pro record-bearing slice completed all available states and stayed no-source with `corpus_decision=coverage_gap`, `total_games=6`, `candidate_only_wins=3`, `shared_wins=2`, `no_policy_wins=1`, `clean_low_fragmentation_routes=0`, and `route_permission=postprocess_only`. `top_axes_by_decision` shows exact zero-window safe pressure as a `coverage_gap_axis` with candidate-better states `3` and no-policy states `1`, but it is fragmented across branches and pairs and already sampled-killed by baseline-save risk.
- The focused sampled Pro record-axis run killed the two repeated active candidate-only leads. The safety/progress token `safe_step_progress -> spirit_development` was `baseline_save_risk` inside `axis_filter_matches` with sampled candidate-better states `1`, baseline-better states `1`, and same-outcome states `2`; the role token `selected -> pre_accept+legacy+legacy_full_pool` was only a sampled singleton with candidate-better states `1` and no baseline-better evidence. The combined filter stayed `fragmented_no_source`, and global sampled Pro stayed `baseline_save_risk` on exact zero-window safe pressure.
- The first timing/continuation-axis pass stayed no-source. Active Pro completed all six available states and stayed `coverage_gap`; repeated timing axes either included the no-policy state, baseline-better saves, or singleton candidate evidence. Sampled Pro with the same eight-state cap stayed `baseline_save_risk`; the combined sampled/active timing rollup was `baseline_save_risk` / `avoid_selector` / `no_source`.
- The first active-blocker cross-budget axis validation stayed no-source with one joined opening-side across Pro/Normal/Fast. `cross_budget_axis_summary` reported zero all-budget repairs; zero-window safe exact pressure and no-rejoin/different-final continuation stability were `budget_conflict` rows with Fast candidate-better evidence, Normal baseline-better save evidence, and Pro no-policy pressure.
- The two-state active-blocker cross-budget widening also stayed no-source. It checked six games and kept `corpus_decision=coverage_gap`, route recommendation `baseline_save_risk_only`, `source_candidate_rollups=[]`, and `source_status_counts` led by `no_candidate_signal=152`, `singleton_non_regressing=25`, and `fragmented_no_source=15`. The repeated later-rejoin continuation lead had two candidate-better joined states and no baseline/no-policy joined states, but it split across three policies, four branch transitions, and four first-move pairs.
- The sampled cross-budget source-status pass stayed no-source. It checked six sampled games and kept `corpus_decision=singleton_no_source`, route recommendation `singleton_candidate_routes`, `source_candidate_rollups=[]`, and `source_status_counts` led by `no_candidate_signal=171`, `fragmented_no_source=10`, `singleton_non_regressing=5`, and `baseline_save_risk=3`. The top blocked rows were no-rejoin continuation save-risk and fragmented single-state SpiritImpact / timing / pressure leads.
- The root-preservation feature-axis smoke stayed no-source. It added `axis=root_preservation` to mechanism axes and checked one sampled plus one active state across Pro/Normal/Fast budgets. The digest stayed `coverage_gap`, `source_candidate_rollups=[]`, and source statuses were led by `no_candidate_signal=152`, `singleton_non_regressing=20`, `fragmented_no_source=14`, `baseline_save_risk=7`, and `coverage_gap=4`. The visible preservation leads were either baseline-save risk or coverage-gap/fragmented rows; a cheap replay of coarser preservation groupings also produced no source candidates.
- The reply-risk / followup-floor feature-axis smoke stayed no-source. It added `axis=reply_floor_progress` and `axis=winner_reply_floor`, then checked one sampled plus one active state across Pro/Normal/Fast budgets. The digest stayed `coverage_gap`, `source_candidate_rollups=[]`, and source statuses were led by `no_candidate_signal=183`, `singleton_non_regressing=28`, `fragmented_no_source=14`, `baseline_save_risk=7`, and `coverage_gap=4`. Coarser replay groupings over winner reply/followup, reply deltas, and reply-progress tradeoffs also produced no source candidates.
- The detailed root-safety feature-axis smoke stayed no-source. It added `axis=root_safety_detail` and `axis=winner_safety_signal`, splitting handoff, roundtrip, walk-vulnerability, vulnerable, and safe roots before crossing them with progress/setup and safety delta. One sampled plus one active state across Pro/Normal/Fast budgets stayed `coverage_gap`, `source_candidate_rollups=[]`, and source statuses were led by `no_candidate_signal=200`, `singleton_non_regressing=32`, `fragmented_no_source=17`, `baseline_save_risk=8`, and `coverage_gap=4`. A cheap replay of coarser safety-delta/signal groupings also produced no source candidates; `winner_safer` was only singleton evidence, while repeated same-safety rows were baseline-save risk or fragmented.
- The enriched forced-root feature-axis digest stayed no-source. It added detailed safety, progress, reply/followup floor, advisor bucket, and path fields to `FORCED_ROOT_ORACLE_ROOT`, then reran the three corrected active Pro `outer_edge_mana_rows` coverage-gap boards. Root coverage stayed `49` tested roots, `12` wins, `37` losses, and all three groups had wins, but `promising_repeated_axes=[]` and `oracle_decision=fragmented_root_features`. The repeated winner axes were all contaminated by losing roots: `rank_band=rank8_plus` had `16` nonwinner roots, `path_safety=lower_unlisted|safe` had `21`, and broad safety/reply/utility axes appeared on all `37` nonwinners.
- The first normalized JSONL workbench export stayed no-source. The same sampled+active one-state cross-budget smoke passed in `295.06s`, and `--jsonl-out` emitted `1341` rows: `42` `policy_decision`, `471` `policy_axis`, `261` `corpus_axis_summary`, `305` `cross_budget_axis_state`, `261` `cross_budget_axis_rollup`, and one `coverage_gap_state`. The digest stayed `corpus_decision=coverage_gap`, `route_permission=no_source`, `source_candidate_rollups=[]`, and source statuses led by `no_candidate_signal=200`, `singleton_non_regressing=32`, `fragmented_no_source=17`, `baseline_save_risk=8`, and `coverage_gap=4`.
- The first JSONL blocked-axis report stayed no-source. The same sampled+active one-state cross-budget smoke passed in `295.32s`; `scripts/summarize-automove-outcome-jsonl.py` reported `source_candidate_axis_count=0`, `blocked_candidate_axis_count=61`, `workbench_decision=blocked_candidate_axes`, and `source_permission=no_source`. Candidate-bearing blockers were `singleton_non_regressing=32`, `fragmented_no_source=17`, `baseline_save_risk=8`, and `coverage_gap=4`. The only candidate count-2 exact axes were decision-stage and no-rejoin continuation, both save/gap contaminated; clean-looking families such as `reply_floor_progress`, `role`, and `winner_reply_floor` were repeated only as exact singleton axes.
- The JSONL family-overlap drilldown stayed no-source. The same sampled+active one-state cross-budget smoke passed in `294.64s`; default overlap families `reply_floor_progress`, `role`, `winner_reply_floor`, `root_safety_detail`, and `safety_progress` all had the same two candidate state ids and pairwise Jaccard `1.0`, but both candidate states were contaminated by baseline/no-policy evidence, so `family_overlap_decision=shared_contaminated_family_states`, `next_action=add_discriminator_or_archive_family`, and `source_permission=no_source`.
- The JSONL state-level discriminator drilldown also stayed no-source. The same sampled+active one-state cross-budget smoke passed in `294.60s` and exported `1341` JSONL rows. The discriminator targeted the same two shared states and reported `state_discriminator_decision=no_state_family_discriminator`, `next_action=archive_current_families_or_add_new_feature_axis`, `target_state_count=2`, `source_candidate_axis_count=0`, and `blocked_candidate_axis_count=61`. Active `outer_edge_mana_rows` candidate-white had `29` candidate-unique exact axes, sampled `inner_wedge_mana_rows` candidate-white had `25`, but both had `candidate_unique_family_count=0` and `candidate_contaminated_family_count=18`, so the exact differences remain inside already contaminated families.
- The first JSONL axis-token discriminator found one routing lead but no source permission. The same sampled+active one-state cross-budget smoke passed in `294.59s`; the log digest stayed `corpus_decision=coverage_gap`, `route_permission=no_source`, and `source_candidate_rollups=[]`. Token scanning across the contaminated target states reported `axis_token_decision=inspect_repeated_candidate_tokens`, `candidate_token_count=156`, `clean_repeated_candidate_token_count=3`, `clean_singleton_candidate_token_count=21`, and `contaminated_candidate_token_count=132`. The repeated clean tokens were all vulnerable-baseline safety/progress variants: `baseline_detail=vulnerable`, `baseline_safety=vulnerable`, and `baseline_signal=vulnerable_progress`, with `3` candidate records across `2` states, `2` policies, `2` branches, and `3` first-move pairs.
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
- The forced-root oracle now supports `SMART_PRO_FORCED_ROOT_ORACLE_ROOT_SOURCE`, so roots can be scored by a runtime profile while a test-only continuation plays out the line. Always copy the source corpus max-plies cap into `SMART_PRO_FORCED_ROOT_ORACLE_MAX_PLIES` when probing corpus-derived boards.
- The forced-root log summarizer is now available at `scripts/summarize-automove-forced-root-oracle-log.py`; use it before reading raw `FORCED_ROOT_ORACLE_ROOT` rows. It reports `nonwinner_count` and `winner_precision` for repeated axes, so winner-only repetition does not become false source permission.
- The policy-matrix summarizer now adds `same_opening_sibling_states` to coverage-gap entries. Use this to detect cross-side candidate-only/no-policy pairings before raw corpus records.
- Raw ProV2, no-selected-followup, full-scored reply guard, no-low-budget, alternating-white, and white-opening utility policies are diagnostic components, not retained challengers.
- Root-origin and continuation-probe ProV4 attempts are retired unless they add a new discriminator below current score, rank, family, safety, progress, and `TurnEngineUtility` fields.
- Future source-bearing work should start from the Outcome Corpus V2 JSONL workbench. The clean-looking families already share the same contaminated candidate states, state-level exact-axis differences did not produce a unique candidate family, and the first token-level lead is vulnerable-baseline safety/progress only. Validate that token lead on a widened slice before justifying either a corpus-calibrated utility feature or a test-only ProV4 unified root policy.
- The fastest path to a promotable automove is probably not another named ProV3 component; it is a shorter evidence loop that ranks mechanisms, then one larger root/utility change measured on the dashboard before retained runtime code.

## Latest Gate Snapshot

- Date: `2026-04-29`
- Shipping decision: public Pro remains on `frontier_pro_v2_guarded`.
- Release containment: public `Pro` dispatch still routes through retained runtime code; `automove_experiments` remains under `#[cfg(test)]`.
- Latest retained package direction: no runtime source retained from recent structural reset work.
- Latest reset evidence: the focused route-filter scans have oracle coverage but no source permission. The safety/progress and `engine_post_search` routes remain fragmented across policy, branch, color, budget, and first-move pair; the active Pro compact view exposed a no-policy state where every current policy loses. Corrected-horizon forced-root probes show that the no-policy state is root-covered, and the enriched forced-root digest confirms current rank/family/utility/safety/reply/advisor/path axes do not cleanly separate winners from losses. The active Pro same-opening sibling check stayed singleton after widening from two to four states, the unfiltered sampled eight-state record-bearing slice stayed baseline-save-risk, the explicit active Pro axis summary stayed coverage-gap with no low-fragmentation route, the focused sampled Pro axis-filter run killed the two active repeated-candidate axes as source leads, the first timing/continuation-axis pass stayed no-source, sampled plus active cross-budget source-status passes produced no source-candidate rollups, the root-preservation, reply-floor, root-safety, and forced-root feature-axis passes all produced no source permission, the first JSONL workbench export stayed coverage-gap/no-source, the first JSONL blocked-axis report found no source candidates, the first JSONL family-overlap drilldown found shared but contaminated candidate states, the JSONL state discriminator found no unique candidate family inside those states, and the first JSONL axis-token discriminator found vulnerable-baseline safety/progress token pressure but still no source-candidate rollups. The retained direction is focused vulnerable-baseline token widening before adding another below-policy feature; no runtime source is retained from this reset pass.

## Session End Checklist

1. Update this file with the current state and exactly one next command sequence.
2. Move durable rules into `docs/automove-knowledge.md`.
3. Move retired wave detail into `docs/automove-archive.md`.
4. Clean disposable artifacts after validation.
5. Leave exactly one clear next hypothesis, or explicitly record that there is no live challenger.
