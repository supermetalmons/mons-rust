# How To Iterate On Automove

This is the canonical automove runbook.

Archived profiles, archived seams, and archived stages are not valid experiment targets. New work stays on the retained Pro surface below.

## Quick Reference

1. Default to Pro work.
2. Treat `frontier_pro_v2_guarded` as both the shipped Pro path and the only retained frontier.
3. Treat `shipping_pro_search` as the retained search-only baseline.
4. Use `./scripts/run-automove-canonical-loop.sh` for the default loop.
5. Clean logs and stamps before ending the session.

## Retained Surface

- Retained profiles: `shipping_pro_search`, `frontier_pro_v2_guarded`
- Canonical stages: `guardrails`, `pro-triage`, `runtime-preflight`, `pro-reliability`, `pro-reliability-confirm`
- Canonical triage surfaces: `opening_reply`, `primary_pro`

## Canonical Loop

```sh
CANDIDATE=<retained_profile_id>
./scripts/run-automove-canonical-loop.sh "$CANDIDATE"
```

- Default shipping profile: `shipping_pro_search`
- Default triage surface inside the loop: `primary_pro`
- Add `--confirm` only after `pro-reliability` earns the spend:

```sh
./scripts/run-automove-canonical-loop.sh --confirm "$CANDIDATE"
```

## Single-Stage Runs

Use `./scripts/run-automove-experiment.sh` when you need one stage at a time or a targeted rerun.

```sh
./scripts/run-automove-experiment.sh guardrails frontier_pro_v2_guarded
SMART_TRIAGE_SURFACE=opening_reply ./scripts/run-automove-experiment.sh pro-triage frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh runtime-preflight frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh pro-reliability frontier_pro_v2_guarded
```

## Gate Rules

- `guardrails`: run first; kill the line on tactical or interview-policy regressions.
- `pro-triage`: this is the cheap deterministic retained surface gate; pass only when the target surface moves with `off_target_changed <= 1`, or when the shipped `frontier_pro_v2_guarded` surface is intentionally stable on the probed target.
- `runtime-preflight`: required before duel stages unless you are doing diagnostics only; exact-lite is hard, stage-1 CPU is advisory for Pro.
- `pro-reliability`: compare the frontier against `shipping_pro_search` in Pro, Normal, and Fast; pass only with `win_rate >= 0.90`, `confidence >= 0.99`, and frontier average move time `<= 700ms` in all three matchups.
- `pro-reliability-confirm`: run only after the smaller retained duel gate earns the spend.

## Diagnostic Toolbox

Use diagnostics only after the canonical loop shows what is still missing.

- `smart_automove_pro_reliability_duel_trace_probe`
- `smart_automove_pro_reliability_nonwin_trace_probe`
- `smart_automove_pro_reliability_hotspot_probe`
- `smart_automove_pro_triage_retained_churn_probe`
- `smart_automove_pro_forced_turn_engine_retained_churn_probe`
- `smart_automove_pro_root_advisor_trace_probe`
- `black_recovery_branch_reply_floor_attribution_probe`
- `black_progress_residual_weight_attribution_probe`

All diagnostics run through the ignored test harness:

```sh
cargo test --release --lib <test_name> -- --ignored --nocapture
```

## Artifacts

- Selected-profile logs: `target/experiment-runs/<profile>/`
- Workflow-only logs: `target/experiment-runs/misc/`
- Runtime-preflight stamps: `target/experiment-stamps/`
- Logs and stamps are disposable evidence, not durable memory.

Standard cleanup:

```sh
./scripts/clean-experiment-artifacts.sh --dry-run
./scripts/clean-experiment-artifacts.sh
```

## Session End

1. Update `AUTOMOVE_IDEAS.md` with the current live state or next frontier.
2. Move durable lessons into `docs/automove-knowledge.md`.
3. Move retired wave detail into `docs/automove-archive.md`.
4. Clean disposable artifacts once validation is complete.
5. Leave exactly one clear next hypothesis, or explicitly record that there is no live challenger.
