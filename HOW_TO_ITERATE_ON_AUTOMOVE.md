# How To Iterate On Automove

This is the canonical runbook for automove work.

Archived profile IDs are invalid by design. New iteration work must stay on the retained profile surface below.

## Quick Reference

1. Pick one target mode: `pro`, `normal`, or `fast`.
2. Pick one active idea from `AUTOMOVE_IDEAS.md`.
3. Start from one retained profile ID.
4. Run the earned path in order.
5. Kill flat candidates early and record the lesson before cleanup.

## Supported Profile IDs

Current runtime and baselines:
- `base`
- `runtime_current`
- `runtime_release_safe_pre_exact`

Calibration anchors and curated-pool references:
- `runtime_eff_exact_lite_v1`
- `runtime_pre_fast_root_quality_v1_normal_conversion_v3`
- `swift_2024_eval_reference`
- `swift_2024_style_reference`
- `runtime_normal_from_fast_reference_v1`

Retained Pro references:
- `runtime_pro_turn_engine_v1`
- `runtime_pro_turn_engine_v30`

If a profile ID is not in this list, it is archive-only context and must not be used for new experiments.

## Earned Paths

### Calibration

Run this before candidate work when the surface is not already known-good.

```sh
./scripts/run-automove-experiment.sh triage-calibrate
./scripts/run-automove-experiment.sh triage-calibrate reply_risk
./scripts/run-automove-experiment.sh triage-calibrate opponent_mana
./scripts/run-automove-experiment.sh triage-calibrate supermana
```

### Fast / Normal

```sh
./scripts/run-automove-experiment.sh guardrails <candidate>
SMART_TRIAGE_SURFACE=<surface> ./scripts/run-automove-experiment.sh triage <candidate>
./scripts/run-automove-experiment.sh runtime-preflight <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh fast-screen <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh progressive <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh ladder <candidate>
```

### Pro

```sh
./scripts/run-automove-experiment.sh guardrails <candidate>
SMART_TRIAGE_SURFACE=<opening_reply|primary_pro> ./scripts/run-automove-experiment.sh pro-triage <candidate>
./scripts/run-automove-experiment.sh runtime-preflight <candidate>
./scripts/run-automove-experiment.sh pro-reliability <candidate> runtime_current
./scripts/run-automove-experiment.sh pro-fast-screen <candidate> runtime_current
./scripts/run-automove-experiment.sh pro-progressive <candidate> runtime_current
./scripts/run-automove-experiment.sh pro-ladder <candidate> runtime_current
```

### Promotion-Time Only

```sh
cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture
cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture
```

## Default Baselines

- Use `runtime_current` as the direct reference for new work unless the idea explicitly depends on another retained reference.
- The script default baseline remains `runtime_release_safe_pre_exact` for compatibility.
- Use `runtime_pro_turn_engine_v30` only as retained Pro history and comparison material. It is not shipping runtime.
- Use `runtime_pro_turn_engine_v1` only for shared regression coverage and historical comparison.

## Stop Conditions

- `guardrails` fails.
- `triage` or `pro-triage` fails.
- `runtime-preflight` fails.
- First earned duel stage is flat or negative.
- Progressive stalls or turns into a runtime cliff.
- One focused diagnostic split produces no credible next move.

When any of these happens, kill the candidate, compress the lesson, and clean the artifacts.

## Artifacts

- Candidate logs: `target/experiment-runs/<candidate>/`
- Workflow-only logs: `target/experiment-runs/misc/`
- Runtime-preflight stamps: `target/experiment-stamps/`
- Process samples: `target/experiment-runs/misc/samples/`

Useful cleanup commands:

```sh
./scripts/clean-experiment-artifacts.sh --dry-run
./scripts/clean-experiment-artifacts.sh --candidate <candidate> --logs-only --dry-run
./scripts/clean-experiment-artifacts.sh --candidate <candidate> --stamps-only --dry-run
./scripts/clean-experiment-artifacts.sh
./scripts/clean-process-samples.sh --dry-run
./scripts/clean-process-samples.sh
```

Logs and stamps are disposable evidence, not durable memory.

## Compatibility Surface

These stages still exist, but they are not promotion evidence on their own:
- `preflight`
- `audit-screen`
- `pre-screen`
- `pro-opening-speed-probe`
- `pro-audit-screen`
- `pro-pre-screen`
- `docs/automove-experiments.md`

## Session End

1. Update `AUTOMOVE_IDEAS.md` with the current state or next split.
2. Move durable lessons into `docs/automove-knowledge.md`.
3. Move retired branch history into `docs/automove-archive.md`.
4. Clean logs and stamps intentionally.
5. Start the next session with one fresh idea and one retained candidate.
