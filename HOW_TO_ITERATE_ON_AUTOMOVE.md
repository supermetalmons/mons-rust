# How To Iterate On Automove

This is the canonical runbook for automove work.

Archived profile IDs are invalid by design. New iteration work must stay on the retained profile surface below.

## Quick Reference

1. Default to Pro work.
2. Treat `runtime_current` as the shipping Pro baseline.
3. Treat `runtime_pro_turn_engine_v30` as the only live Pro challenger.
4. Run the cheap-to-expensive Pro loop in order.
5. Kill flat lines immediately and compress the lesson before starting another split.

## Retained Profile Surface

Current runtime and baselines:
- `base`
- `runtime_current`
- `runtime_release_safe_pre_exact`

Calibration anchors and curated references:
- `runtime_eff_exact_lite_v1`
- `runtime_pre_fast_root_quality_v1_normal_conversion_v3`
- `swift_2024_eval_reference`
- `swift_2024_style_reference`
- `runtime_normal_from_fast_reference_v1`

Retained Pro references:
- `runtime_pro_turn_engine_v1`
- `runtime_pro_turn_engine_v30`

If a profile ID is not in this list, it is archive-only context and must not be used for new work.

## Current Reality

- Shipping Pro is `runtime_current`. Its Pro path is the planner-plus-quiescence runtime selected by the live game context.
- The only live Pro challenger is `runtime_pro_turn_engine_v30`. It is a guarded `ProV2` turn-engine path, not a raw always-on engine.
- `runtime_pro_turn_engine_v30` deliberately falls back on opening-book positions and several early white turn shapes. That is expected behavior, not a bug.
- `runtime_pro_turn_engine_v1` stays only as regression history and comparison material.
- Do not reopen archive profiles, retired planner lines, or old quiescence branches without a brand-new hypothesis strong enough to justify new code.

## Canonical Pro Loop

Use this exact order for real Pro iteration work.

```sh
./scripts/run-automove-experiment.sh guardrails runtime_pro_turn_engine_v30
SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage runtime_pro_turn_engine_v30 runtime_current
./scripts/run-automove-experiment.sh runtime-preflight runtime_pro_turn_engine_v30
./scripts/run-automove-experiment.sh pro-reliability runtime_pro_turn_engine_v30 runtime_current
```

Operator defaults:
- Candidate profile: `SMART_PRO_CANDIDATE_PROFILE` or `SMART_CANDIDATE_PROFILE`
- Pro baseline: `runtime_current`
- Default Pro triage surface: `SMART_TRIAGE_SURFACE=primary_pro`
- Opening-only fallback surface: `SMART_TRIAGE_SURFACE=opening_reply`

Baseline rules:
- For Pro work, compare directly against `runtime_current`.
- The script default baseline, `runtime_release_safe_pre_exact`, exists for compatibility and non-Pro stages. Do not let it drive Pro examples or Pro decisions.
- Use `opening_reply` only when the change touches opening-book fallback ordering, early-turn fallback guards, or an opening-specific latency regression.

## Step Rules

### 1. `guardrails`

What it means:
- Cheap tactical and interview-policy validation.
- First kill gate before any surface or duel work.

Kill the line if:
- `guardrails` fails.
- The change only buys speed but loses obvious tactical quality.

### 2. `pro-triage`

What it means:
- Fixed-cost deterministic Pro triage against `runtime_current`.
- The harness always compares both Pro fixture packs: the target surface and the off-target surface.

Pass rule:
- Pass only when the target surface changes and off-target churn stays at `<= 1`.

Default interpretation:
- `primary_pro` is the default live surface. It is the main Pro fixture pack and should carry almost all ongoing work.
- `opening_reply` is a narrow opening/fallback-order check, not the default Pro surface.

Kill the line if:
- The target surface does not move.
- Off-target churn is larger than `1`.
- The change needs a broader story than the current hypothesis can justify.

### 3. `runtime-preflight`

What it means:
- Required stamp before duel stages unless you intentionally skip it for diagnostics.
- Runs the stage-1 CPU non-regression gate and exact-lite diagnostics gate.

Kill the line if:
- Stage-1 CPU regresses.
- Exact-lite diagnostics regress.
- The candidate needs more wrapper knobs just to survive preflight.

### 4. `pro-reliability`

What it means:
- First real Pro promotion gate.
- Direct Pro-vs-Pro duel evidence against `runtime_current`.

Use it to decide:
- Whether the candidate is promotable.
- Whether the live wall moved to a new code surface.
- Whether the next split should be a shared exact/search cut or a minimal fixture addition.

Kill the line if:
- The first direct duel signal is flat or negative.
- The run still does not finish in a practical window and you do not have a new code hypothesis.
- The wall stays where it already was after a focused split.

## Diagnostic Ladder

Use diagnostics only after the canonical loop tells you why they are needed.

1. Prefer a fresh live `pro-reliability` sample when the wall is unclear or has moved.
2. Use `triage-calibrate` only when the triage surface itself is new or no longer calibrated.
3. Use `pro-opening-speed-probe` only for opening-specific regressions.
4. Use `pro-audit-screen` only as a cheap sanity check on a clean `pro-triage` reject.
5. Use the hotspot probe only after a real duel stall, and only to narrow the next code surface.

Do not do these by default:
- archive reopenings
- wrapper-only knob sweeps
- hotspot-first micro-optimization loops
- broad split families without a new code hypothesis

## Promotion Follow-Up Only

These stages are not part of the default Pro loop. Use them only after `pro-reliability` earns more spend.

```sh
./scripts/run-automove-experiment.sh pro-fast-screen runtime_pro_turn_engine_v30 runtime_current
./scripts/run-automove-experiment.sh pro-progressive runtime_pro_turn_engine_v30 runtime_current
./scripts/run-automove-experiment.sh pro-ladder runtime_pro_turn_engine_v30 runtime_current
```

## Compatibility Surface

These commands still exist, but they are not the main story and they are not promotion proof for the Pro frontier:
- `triage-calibrate`
- `triage`
- `preflight`
- `audit-screen`
- `pre-screen`
- `fast-screen`
- `progressive`
- `ladder`
- `pro-opening-speed-probe`
- `pro-audit-screen`
- `pro-pre-screen`
- `docs/automove-experiments.md`

## Artifacts

- Candidate logs: `target/experiment-runs/<candidate>/`
- Workflow-only logs: `target/experiment-runs/misc/`
- Runtime-preflight stamps: `target/experiment-stamps/`
- Process samples: `target/experiment-runs/misc/samples/`

Logs and stamps are disposable evidence, not durable memory.

## Session End

1. Update `AUTOMOVE_IDEAS.md` with the next live split or the kill result.
2. Move durable lessons into `docs/automove-knowledge.md`.
3. Move retired branch history into `docs/automove-archive.md`.
4. Clean logs and stamps intentionally.
5. Start the next session with one live hypothesis and one retained candidate.
