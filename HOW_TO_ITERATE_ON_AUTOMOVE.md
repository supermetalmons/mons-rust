# How To Iterate On Automove

This is the canonical automove runbook.

Archived profile IDs and archived stages are invalid experiment targets. New work stays on the retained surface below.

## Quick Reference

1. Default to Pro work.
2. Treat `runtime_current` as the shipping baseline.
3. Treat `runtime_pro_turn_engine_v30` as the only retained Pro frontier.
4. Run the cheap-to-expensive loop in order.
5. Clean logs and stamps at the end of the session.

## Retained Profile Surface

- `runtime_current`
- `runtime_pro_turn_engine_v30`

If a profile ID is not in this list, it is archive-only.

## Current Reality

- Shipping Pro is `runtime_current`.
- `runtime_current` delegates to the promoted guarded `runtime_pro_turn_engine_v30` path.
- The guarded path intentionally keeps opening-book and early-white fallback guards.
- Promotion proof is still direct evidence against `runtime_current`, not fixture churn or hotspot output.

## Canonical Pro Loop

```sh
CANDIDATE=<new_retained_pro_profile>
./scripts/run-automove-experiment.sh guardrails "$CANDIDATE"
SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage "$CANDIDATE" runtime_current
./scripts/run-automove-experiment.sh runtime-preflight "$CANDIDATE"
./scripts/run-automove-experiment.sh pro-reliability "$CANDIDATE" runtime_current
```

Operator defaults:
- Pro baseline: `runtime_current`
- Default Pro triage surface: `SMART_TRIAGE_SURFACE=primary_pro`
- Opening-only fallback surface: `SMART_TRIAGE_SURFACE=opening_reply`

## Gate Rules

### `guardrails`

- Run first.
- Kill the line on tactical regressions or interview-policy regressions.

### `pro-triage`

- This is the cheap deterministic Pro surface gate.
- For a real challenger, pass only when the target surface changes and off-target churn stays at `<= 1`.
- For post-promotion maintenance on `runtime_pro_turn_engine_v30` vs `runtime_current`, a stable `0/0` result is valid because that retained frontier is intentionally shipping-equivalent.
- Kill the line if it only moves one stale seam or does not move the target surface at all.

### `runtime-preflight`

- Required before duel stages unless you are doing diagnostics only.
- Exact-lite diagnostics remain a hard gate.
- Stage-1 CPU is advisory for Pro and still hard for non-Pro work.

### `pro-reliability`

- This is the real duel gate: Pro vs current Pro, Normal, and Fast.
- Pass only when all three runs complete with `win_rate >= 0.90`, `confidence >= 0.99`, and `candidate_avg_move_ms <= 700`.
- Kill the line if the wall stays on old fragmented churn after a focused split.

### `pro-reliability-confirm`

- Run only after `pro-reliability` earns the spend.
- Promotion proof is still the same three-duel rule on the larger confirm corpus.

## Diagnostic Toolbox

Use diagnostics only after the canonical loop tells you what is missing.

- `smart_automove_pro_reliability_hotspot_probe`: bounded compare-oriented hotspot check.
- `smart_automove_pro_reliability_duel_trace_probe`: replay duel seeds and inspect first divergences.
- `smart_automove_pro_reliability_nonwin_trace_probe`: collapse non-win openings from a duel corpus.
- `smart_automove_pro_triage_retained_churn_probe`: separate retained selector churn stories.
- `smart_automove_pro_runtime_faithful_retained_churn_probe`: inspect runtime-faithful forced-engine acceptance on retained churn fixtures.
- `smart_automove_pro_root_advisor_trace_probe`: inspect unified ProV2 root-advisor decisions directly.

All experiment probes run through the ignored test harness:

```sh
cargo test --release --lib <test_name> -- --ignored --nocapture
```

## Artifacts And Cleanup

- Candidate logs: `target/experiment-runs/<candidate>/`
- Workflow-only logs: `target/experiment-runs/misc/`
- Runtime-preflight stamps: `target/experiment-stamps/`
- Logs and stamps are disposable evidence, not durable memory.
- Standard cleanup step:
  - `./scripts/clean-experiment-artifacts.sh --dry-run`
  - `./scripts/clean-experiment-artifacts.sh`

## Session End

1. Update `AUTOMOVE_IDEAS.md` with the current live state or next frontier.
2. Move durable lessons into `docs/automove-knowledge.md`.
3. Move retired wave summaries into `docs/automove-archive.md`.
4. Run `./scripts/clean-experiment-artifacts.sh --dry-run`, then the real cleanup when validation is complete.
5. Leave one clear next hypothesis, or explicitly record that no live challenger exists yet.
