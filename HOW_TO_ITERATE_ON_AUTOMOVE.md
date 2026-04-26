# How To Iterate On Automove

This is the canonical automove runbook.

Archived profiles, archived seams, and archived stages are not valid experiment targets. New work stays on the retained Pro surface below.

## Quick Reference

1. Default to Pro work.
2. Optimize for reliable strength across all game variants, not only Classic.
3. Treat `frontier_pro_v2_guarded` as both the shipped Pro path and the only retained frontier.
4. Treat `shipping_pro_search` as the retained search-only baseline.
5. Use `./scripts/run-automove-canonical-loop.sh` for the default loop.
6. Pick exactly one live hypothesis before editing runtime code.
7. Probe first when the mechanism is unclear.
8. When there is no live hypothesis, switch to the structural reset in `docs/automove-strategy.md` instead of running another seam loop.
9. Archive or kill the line before starting another.
10. Clean logs and stamps before ending the session.

## Variant Policy

- Quick iteration uses deterministic seeded variant samples. A failed sampled-variant gate is enough to kill a line.
- Final promotion confirmation uses all current `GameVariant`s by default.
- Retained `primary_pro` fixtures are Classic regression controls only. They are not broad variant evidence.
- Use `SMART_AUTOMOVE_VARIANTS=classic,swapped_mana_rows` for a targeted variant rerun.
- Use `SMART_AUTOMOVE_VARIANT_POLICY=classic`, `sampled`, or `all` only when you need to override the stage default.
- Variant randomness is seeded and reproducible from logs.

## Retained Surface

- Retained profiles: `shipping_pro_search`, `frontier_pro_v2_guarded`
- Canonical stages: `guardrails`, `variant-smoke`, `pro-triage`, `runtime-preflight`, `pro-reliability`, `pro-reliability-confirm`
- Canonical triage surface: retained Classic `primary_pro`

## Canonical Loop

```sh
CANDIDATE=<retained_profile_id>
./scripts/run-automove-canonical-loop.sh "$CANDIDATE"
```

- Default shipping profile: `shipping_pro_search`
- Default quick duel variant policy: seeded `sampled`
- Default triage surface inside the loop: retained Classic `primary_pro`
- Add `--confirm` only after `pro-reliability` earns the spend:

```sh
./scripts/run-automove-canonical-loop.sh --confirm "$CANDIDATE"
```

## Single-Stage Runs

Use `./scripts/run-automove-experiment.sh` when you need one stage at a time or a targeted rerun.

```sh
./scripts/run-automove-experiment.sh guardrails frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh variant-smoke frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh pro-triage frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh runtime-preflight frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh pro-reliability frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh pro-reliability-confirm frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh pro-profile-sweep frontier_pro_v2_raw
./scripts/run-automove-experiment.sh pro-promotion-dashboard frontier_pro_v2_raw
./scripts/run-automove-experiment.sh pro-sweep-decision-record frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh pro-policy-matrix frontier_pro_v2_guarded,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard
./scripts/run-automove-experiment.sh pro-policy-winner frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,shipping_pro_search_control
```

## Structural Reset

Use this path when `AUTOMOVE_IDEAS.md` says there is no live challenger or when recent work keeps passing narrow sampled slices and failing broader promotion gates.

```sh
./scripts/run-automove-structural-scout.sh <sweep-candidate[,candidate...]>
```

- Read `docs/automove-strategy.md` first.
- The scout is diagnostic-only and runs both canonical sampled and active-blocker panels.
- Do not edit runtime code for a broad Pro change unless the candidate is strong on both panels.
- Add `--confirm` only after the default scout panels look promotable.

## Gate Rules

- `guardrails`: run first; kill the line on tactical or interview-policy regressions.
- `variant-smoke`: cheap all-variant legality check for public Fast, Normal, and Pro paths.
- `pro-triage`: cheap deterministic retained Classic surface gate; pass only when the target surface moves with `off_target_changed <= 1`, or when the shipped `frontier_pro_v2_guarded` surface is intentionally stable on the probed target.
- `runtime-preflight`: required before duel stages unless you are doing diagnostics only; exact-lite is hard, stage-1 CPU is advisory for Pro, and openings use sampled variants.
- `pro-reliability`: sampled-variant frontier-vs-`shipping_pro_search` duels in Pro, Normal, and Fast; pass only with `win_rate >= 0.90`, `confidence >= 0.99`, and frontier average move time `<= 700ms` in all three matchups.
- `pro-reliability-confirm`: all-variant confirmation. Run only after the sampled duel gate earns the spend; it also enforces a per-variant non-regression floor.
- `pro-profile-sweep` and `pro-profile-attribution`: diagnostic-only stages for test-only Pro candidates. They do not add retained profiles and are not promotion stages.

## Iteration Lifecycle

1. Read `AUTOMOVE_IDEAS.md` and select the single current hypothesis.
2. If there is no current hypothesis, run the structural reset path and create a test-only candidate or diagnostic before touching runtime code.
3. If the mechanism is not already proven, run one targeted diagnostic before editing runtime code.
4. Make the narrowest runtime or test-only change that can falsify the hypothesis.
5. Run the canonical stages in order; stop immediately on a failed hard gate.
6. If the line fails, discard runtime code and record the no-go in `docs/automove-archive.md` or `docs/automove-knowledge.md`.
7. If the line passes, promote retained Classic regression coverage before confirm.
8. End by compressing `AUTOMOVE_IDEAS.md` back to current state plus one next hypothesis.

## Diagnostic Toolbox

Use diagnostics only after the canonical loop shows what is still missing.

- `smart_automove_pro_reliability_duel_trace_probe`
- `smart_automove_pro_reliability_nonwin_trace_probe`
- `smart_automove_pro_reliability_hotspot_probe`
- `smart_automove_pro_profile_sweep_probe`
- `smart_automove_pro_profile_attribution_probe`
- `smart_automove_pro_sweep_decision_record_probe`
- `smart_automove_pro_policy_matrix_probe`
- `smart_automove_pro_policy_winner_probe`
- `smart_automove_pro_promotion_dashboard_probe`
- `smart_automove_pro_decision_record_aggregation_probe`
- `smart_automove_pro_forced_root_oracle_probe`
- `smart_automove_pro_triage_retained_churn_probe`
- `smart_automove_pro_forced_turn_engine_retained_churn_probe`
- `smart_automove_pro_root_advisor_trace_probe`
- `black_recovery_branch_reply_floor_attribution_probe`
- `black_progress_residual_weight_attribution_probe`

`smart_automove_pro_reliability_hotspot_probe` can take one extra ad-hoc board without a source edit:

```sh
SMART_PRO_RELIABILITY_HOTSPOT_LABEL=<label> \
SMART_PRO_RELIABILITY_HOTSPOT_MODE=pro \
SMART_PRO_RELIABILITY_HOTSPOT_FEN='<fen>' \
cargo test --release --lib smart_automove_pro_reliability_hotspot_probe -- --ignored --nocapture
```

`smart_automove_pro_reliability_duel_trace_probe` and `smart_automove_pro_reliability_nonwin_trace_probe` can focus on one duel bucket:

```sh
SMART_PRO_RELIABILITY_DUEL_FILTER=vs_shipping_fast \
cargo test --release --lib smart_automove_pro_reliability_duel_trace_probe -- --ignored --nocapture
```

`smart_automove_pro_profile_sweep_probe` compares test-only Pro candidates against the retained shipping baseline without adding them to the retained profile registry. It prints structured `PRO_PROFILE_SWEEP_RESULT`, `PRO_PROFILE_SWEEP_VARIANT`, and guarded-branch `PRO_PROFILE_SWEEP_BRANCH` lines.

```sh
SMART_PRO_SWEEP_CANDIDATES=frontier_pro_v2_guarded,frontier_pro_v2_raw \
SMART_PRO_SWEEP_DUEL_FILTER=vs_shipping_fast \
SMART_AUTOMOVE_VARIANTS=alternating_mana_rows,forward_bridge_mana_rows \
cargo test --release --lib smart_automove_pro_profile_sweep_probe -- --ignored --nocapture
```

The same sweep is available through the experiment runner:

```sh
SMART_PRO_SWEEP_DUEL_FILTER=vs_shipping_fast \
SMART_AUTOMOVE_VARIANTS=alternating_mana_rows,forward_bridge_mana_rows \
./scripts/run-automove-experiment.sh pro-profile-sweep frontier_pro_v2_raw
```

`smart_automove_pro_promotion_dashboard_probe` summarizes a sweep candidate on both canonical sampled and active-blocker panels. It prints `PRO_PROMOTION_DASHBOARD_RESULT`, weakness-sorted `PRO_PROMOTION_DASHBOARD_VARIANT`, per-panel `PRO_PROMOTION_DASHBOARD_PANEL`, and final `PRO_PROMOTION_DASHBOARD_CANDIDATE` lines. Use it before cutting runtime code when a candidate might be active-blocker-only, sampled-only, or broadly promising.

```sh
./scripts/run-automove-experiment.sh pro-promotion-dashboard frontier_pro_v2_raw
```

For expensive or high-risk scouts, kill quickly on the sampled panel before spending guarded deltas or active blockers:

```sh
SMART_PRO_DASHBOARD_PANEL_FILTER=sampled \
SMART_PRO_DASHBOARD_INCLUDE_GUARDED=false \
./scripts/run-automove-experiment.sh pro-promotion-dashboard <candidate>
```

`smart_automove_pro_profile_attribution_probe` replays the same opening seeds with two sweep candidates against the same shipping opponent, then prints outcome-changing first divergences as `PRO_PROFILE_SWEEP_ATTRIBUTION`, `PRO_PROFILE_SWEEP_ATTRIBUTION_SUMMARY`, `PRO_PROFILE_SWEEP_ATTRIBUTION_BRANCH`, and `PRO_PROFILE_SWEEP_ATTRIBUTION_PAIR` lines. It defaults to `frontier_pro_v2_guarded` vs `frontier_pro_v2_raw`; override with `SMART_PRO_SWEEP_ATTRIBUTION_LEFT` and `SMART_PRO_SWEEP_ATTRIBUTION_RIGHT`.

```sh
SMART_PRO_SWEEP_ATTRIBUTION_LEFT=frontier_pro_v2_no_late_black_fallback \
SMART_PRO_SWEEP_ATTRIBUTION_RIGHT=frontier_pro_v2_raw \
SMART_PRO_SWEEP_DUEL_FILTER=all \
SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows \
cargo test --release --lib smart_automove_pro_profile_attribution_probe -- --ignored --nocapture
```

The attribution wrapper sets the left candidate from the stage argument and reads the right candidate from `SMART_PRO_SWEEP_ATTRIBUTION_RIGHT`:

```sh
SMART_PRO_SWEEP_ATTRIBUTION_RIGHT=frontier_pro_v2_raw \
SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows \
./scripts/run-automove-experiment.sh pro-profile-attribution frontier_pro_v2_no_late_black_fallback
```

`smart_automove_pro_sweep_decision_record_probe` aggregates nonwins or outcome deltas for any registered sweep candidate against a same-seed `shipping_pro_search_control` replay. Use it when a test-only candidate is not a retained profile and the retained-profile decision recorder cannot inspect it.

```sh
SMART_PRO_SWEEP_DECISION_RECORD_SCOPE=nonwins \
SMART_PRO_SWEEP_DECISION_RECORD_DUEL_FILTER=vs_shipping_pro \
./scripts/run-automove-experiment.sh pro-sweep-decision-record frontier_pro_v2_raw
```

`smart_automove_pro_policy_matrix_probe` compares multiple registered sweep policies on identical openings across the sampled and active-blocker panels. The first candidate is the baseline. Use it before writing another policy selector when two ablations each fix one active row but rotate sampled or Fast/Normal losses elsewhere; it prints per-candidate outcome summaries plus first-divergence branch, context, and move-pair aggregates.

The matrix also prints `PRO_POLICY_MATRIX_PORTFOLIO` for each panel/duel, followed by weakness buckets as `PRO_POLICY_MATRIX_PORTFOLIO_CLASS`. Read these before designing a context selector: `candidate_only_wins` is the selector opportunity, `baseline_only_wins` is the regression risk, and `no_policy_wins` means the current candidate set cannot solve those openings by selection alone.

Set `SMART_PRO_POLICY_MATRIX_INCLUDE_DECISION_PROBE=true` on a narrow run when a first divergence needs deeper root evidence. This adds guarded root rank, family, score, selected/advisor status, and full-vs-no-selected-followup utility for both divergent moves. Keep this off for broad matrix runs because it reruns root scoring for printed records.

```sh
SMART_PRO_POLICY_MATRIX_PANEL_FILTER=active_blockers \
SMART_PRO_POLICY_MATRIX_DUEL_FILTER=vs_shipping_fast \
SMART_PRO_POLICY_MATRIX_REPEATS=1 \
SMART_PRO_POLICY_MATRIX_GAMES=1 \
SMART_PRO_POLICY_MATRIX_INCLUDE_DECISION_PROBE=true \
./scripts/run-automove-experiment.sh pro-policy-matrix frontier_pro_v2_guarded,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard
```

`smart_automove_pro_policy_winner_probe` is the faster selector-design companion for a policy matrix with good oracle coverage. It plays the baseline first; when the baseline loses, it tries candidate policies in the provided order until one wins, then prints `PRO_POLICY_WINNER_POLICY`, `PRO_POLICY_WINNER_CONTEXT`, and `PRO_POLICY_WINNER_PAIR`. Use it before writing a selector over an already-viable policy set, but validate the resulting selector with `pro-promotion-dashboard`; first-winning context alone can hide policy-entry timing conflicts.

For broad portfolios, keep it filtered or cap exploratory cost with `SMART_PRO_POLICY_WINNER_CANDIDATE_TRACE_LIMIT`; summaries then include `candidate_trace_limit_hit=true` when the duel was intentionally partial.

```sh
SMART_PRO_POLICY_WINNER_PANEL_FILTER=sampled \
SMART_PRO_POLICY_WINNER_DUEL_FILTER=vs_shipping_pro \
./scripts/run-automove-experiment.sh pro-policy-winner frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,shipping_pro_search_control,frontier_pro_v2_raw,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard,frontier_pro_v2_no_low_budget_guard
```

`smart_automove_pro_forced_root_oracle_probe` forces each scored root once from one blocker board, then continues with a registered sweep candidate against retained shipping Pro. Use it when the policy matrix reports `no_policy_wins` for a specific context and you need to know whether the root set already contains winning moves before creating another policy. Override `SMART_PRO_FORCED_ROOT_ORACLE_FEN`, `SMART_PRO_FORCED_ROOT_ORACLE_CONTINUATION`, and `SMART_PRO_FORCED_ROOT_ORACLE_ROOT_LIMIT` for focused boards. When the board comes from a full-opening first divergence, set `SMART_PRO_FORCED_ROOT_ORACLE_START_PLY` to that `first_diff_ply`; otherwise the oracle grants extra rollout horizon and can turn full-opening losses into false local wins.

```sh
SMART_PRO_FORCED_ROOT_ORACLE_FEN='<fen>' \
SMART_PRO_FORCED_ROOT_ORACLE_CONTINUATION=frontier_pro_v2_guarded \
SMART_PRO_FORCED_ROOT_ORACLE_START_PLY=<first_diff_ply> \
cargo test --release --lib smart_automove_pro_forced_root_oracle_probe -- --ignored --nocapture
```

`smart_automove_pro_decision_record_aggregation_probe` aggregates first-divergence records against `shipping_pro_search` and reports whether the shipping root was selected, pre-accepted, head-selected, legacy-selected, candidate-live, advisor-approved, ordered, preserved, injected, or omitted. Use `SMART_PRO_DECISION_RECORD_SCOPE=nonwins` when the promotion miss is flat losses rather than frontier-worse-than-shipping deltas.

```sh
SMART_PRO_DECISION_RECORD_SCOPE=nonwins \
SMART_PRO_DECISION_RECORD_DUEL_FILTER=vs_shipping_fast \
SMART_AUTOMOVE_VARIANTS=alternating_mana_rows,forward_bridge_mana_rows \
cargo test --release --lib smart_automove_pro_decision_record_aggregation_probe -- --ignored --nocapture
```

All diagnostics run through the ignored test harness:

```sh
cargo test --release --lib <test_name> -- --ignored --nocapture
```

## Artifacts

- Selected-profile logs: `target/experiment-runs/<profile>/`
- Workflow-only logs: `target/experiment-runs/misc/`
- Runtime-preflight stamps: `target/experiment-stamps/`
- Run metadata records the stage variant policy and any explicit variant override.
- Rust gate logs print resolved variant policy, sample size, and per-variant duel summaries.
- Logs and stamps are disposable evidence, not durable memory.

Standard cleanup:

```sh
./scripts/clean-experiment-artifacts.sh --dry-run
./scripts/clean-experiment-artifacts.sh
```

Full local cache cleanup after validation:

```sh
./scripts/clean-experiment-artifacts.sh --dry-run --all-target
./scripts/clean-experiment-artifacts.sh --all-target
```

## Session End

1. Update `AUTOMOVE_IDEAS.md` with the current live state or next frontier.
2. Move durable lessons into `docs/automove-knowledge.md`.
3. Move retired wave detail into `docs/automove-archive.md`.
4. Clean disposable artifacts once validation is complete.
5. Leave exactly one clear next hypothesis, or explicitly record that there is no live challenger.
6. Do not leave unarchived probe diaries or failed runtime branches in the live board.
