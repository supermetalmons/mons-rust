# How To Iterate On Automove

This is the canonical automove runbook.

Archived profile IDs and archived stages are invalid experiment targets. New work stays on the retained surface below.

## Quick Reference

1. Default to Pro work.
2. Treat `frontier_pro_v2_guarded` as the current shipping Pro path.
3. Treat `shipping_pro_search` as the retained search-only baseline profile.
4. Use `./scripts/run-automove-canonical-loop.sh` for the default Pro loop.
5. Clean logs and stamps at the end of the session.

## Retained Profile Surface

- `shipping_pro_search`
- `frontier_pro_v2_guarded`

If a profile ID is not in this list, it is archive-only.

## Glossary

- `shipping`: the deployed Pro path, currently `frontier_pro_v2_guarded`
- `baseline`: the retained search-only comparison profile, currently `shipping_pro_search`
- `frontier`: the guarded ProV2 selector/runtime line, currently `frontier_pro_v2_guarded`
- `probe`: forced turn-engine diagnostics that inspect acceptance behavior without changing shipping

## Current Reality

- Shipping Pro now routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` remains the retained search-only baseline and still keeps the turn-engine selector disabled.
- `frontier_pro_v2_guarded` is both the shipped Pro path and the retained guarded ProV2 frontier.
- Probe paths only inspect forced turn-engine behavior; they are diagnostics, not shipping behavior.
- Direct duel evidence still matters more than fixture churn or hotspot output.

## Canonical Pro Loop

```sh
CANDIDATE=<new_retained_pro_profile>
./scripts/run-automove-canonical-loop.sh "$CANDIDATE"
```

Operator defaults:
- Baseline profile: `shipping_pro_search`
- Default Pro triage surface inside the canonical loop: `SMART_TRIAGE_SURFACE=primary_pro`
- Add `--confirm` when the smaller reliability gate already earned the spend:
  - `./scripts/run-automove-canonical-loop.sh --confirm "$CANDIDATE"`

## Single-Stage And Diagnostic Runs

Use `./scripts/run-automove-experiment.sh` only when you need one stage at a time or a diagnostic rerun.

Examples:

```sh
./scripts/run-automove-experiment.sh guardrails frontier_pro_v2_guarded
SMART_TRIAGE_SURFACE=opening_reply ./scripts/run-automove-experiment.sh pro-triage frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh runtime-preflight frontier_pro_v2_guarded
```

## Gate Rules

### `guardrails`

- Run first.
- Kill the line on tactical regressions or interview-policy regressions.

### `pro-triage`

- This is the cheap deterministic Pro surface gate.
- For a real challenger, pass only when the target surface changes and off-target churn stays at `<= 1`.
- After shipping `frontier_pro_v2_guarded`, a stable `0/0` result is acceptable only when you intentionally expect no retained-surface behavior change.
- Kill the line if it only moves one stale seam or does not move the target surface at all.

### `runtime-preflight`

- Required before duel stages unless you are doing diagnostics only.
- Exact-lite diagnostics remain a hard gate.
- Stage-1 CPU is advisory for Pro and still hard for non-Pro work.

### `pro-reliability`

- This retained duel gate compares the selected frontier against `shipping_pro_search` in Pro, Normal, and Fast modes.
- Pass only when all three runs complete with `win_rate >= 0.90`, `confidence >= 0.99`, and frontier average move time `<= 700ms`.
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
- `smart_automove_pro_forced_turn_engine_retained_churn_probe`: inspect forced-turn-engine probe acceptance on retained churn fixtures.
- `smart_automove_pro_root_advisor_trace_probe`: inspect unified ProV2 root-advisor decisions directly.

All experiment probes run through the ignored test harness:

```sh
cargo test --release --lib <test_name> -- --ignored --nocapture
```

## Artifacts And Cleanup

- Selected-profile logs: `target/experiment-runs/<profile>/`
- Workflow-only logs: `target/experiment-runs/misc/`
- Runtime-preflight stamps: `target/experiment-stamps/`
- Logs and stamps are disposable evidence, not durable memory.
- The old flat log layout and legacy `target/experiment-runs/runtime_preflight_*.stamp` path are retired.
- Standard cleanup step:
  - `./scripts/clean-experiment-artifacts.sh --dry-run`
  - `./scripts/clean-experiment-artifacts.sh`

## Session End

1. Update `AUTOMOVE_IDEAS.md` with the current live state or next frontier.
2. Move durable lessons into `docs/automove-knowledge.md`.
3. Move retired wave summaries into `docs/automove-archive.md`.
4. Run `./scripts/clean-experiment-artifacts.sh --dry-run`, then the real cleanup when validation is complete.
5. Leave one clear next hypothesis, or explicitly record that no live challenger exists yet.
