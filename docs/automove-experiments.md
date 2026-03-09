# Smart Automove Experiments

This is the canonical automove runbook. Production runtime logic stays in the main game code. Experimentation stays test-only under `src/models/automove_experiments/`.

Raw interview notes remain in `docs/automove-pro-strategy-interview.md`. Retired profile waves live in `docs/automove-archive.md`. Durable lessons stay in `docs/automove-knowledge.md`.

## Quick Start

Use the wrapper script for active client-mode iteration:

1. Preflight once for the candidate.

```sh
./scripts/run-automove-experiment.sh preflight <candidate>
```

2. Run the 3-command promotion pipeline.

```sh
./scripts/run-automove-experiment.sh fast-screen <candidate>
./scripts/run-automove-experiment.sh progressive <candidate>
./scripts/run-automove-experiment.sh ladder <candidate>
```

Default ladder baseline is `runtime_release_safe_pre_exact`. Override it with the optional third argument when needed:

```sh
./scripts/run-automove-experiment.sh fast-screen <candidate> <baseline>
```

## Exploration Discipline

- Keep `main` as the last known clean promotion checkpoint.
- Run active exploration on a `codex/*` branch until a candidate is promotable.
- Use hypothesis-driven names in the `runtime_root_*` line for new pre-exact waves.
- Keep only one live `runtime_root_*` candidate in the active registry at a time.
- Keep archived low-CPU waves in `docs/automove-archive.md`; do not revive them by default.
- Do not start an `runtime_exact_lite_probe_*` wave until a pre-exact candidate clears preflight plus the client promotion ladder.

## What Each Stage Does

- `preflight`: runs tactical guardrails, strict client-mode (`fast` and `normal`) stage-1 CPU non-regression vs `runtime_current`, and the exact-lite diagnostics gate. The stage-1 gate uses repeated speed probes and a median ratio so one noisy sample does not kill a candidate. The exact-lite gate is a no-op for non-exact candidates.
- `fast-screen`: quick active-pool screen against `runtime_release_safe_pre_exact`.
- `progressive`: geometric duel against `runtime_release_safe_pre_exact` with artifact output under `target/experiment-runs`.
- `ladder`: full promotion gate with tactical checks, speed checks, progressive duel, confirmation duel, and pool non-regression.

## Release Prerequisites

Run these before any promotion decision that could change shipped runtime behavior:

```sh
cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture
cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture
```

For experiment-only stage-1 speed checks that also include `pro`, set `SMART_STAGE1_INCLUDE_PRO=true`.

The shipped wasm API remains:

- `smartAutomoveAsync("fast")`
- `smartAutomoveAsync("normal")`
- `smartAutomoveAsync("pro")`

## Active Profiles

Only these profiles are part of the active experiment surface:

- `base`
- `runtime_current`
- `runtime_release_safe_pre_exact`
- `runtime_eff_non_exact_v1`
- `runtime_eff_non_exact_v2`
- `runtime_efficient_v1`
- `runtime_eff_exact_lite_v1`
- `swift_2024_eval_reference`
- `swift_2024_style_reference`
- `runtime_pre_fast_root_quality_v1_normal_conversion_v3`
- `runtime_pre_pro_promotion_v1`

Notes:

- `runtime_current` is the shipped runtime profile.
- `runtime_release_safe_pre_exact` is the immutable default promotion baseline.
- `runtime_efficient_v1` is the compatibility alias for `runtime_eff_non_exact_v1`.
- `runtime_eff_exact_lite_v1` enables only small exact-lite progress checks with explicit call budgets.

## Curated Pool

The active pool is fixed to:

- `runtime_current`
- `runtime_release_safe_pre_exact`
- `swift_2024_eval_reference`
- `swift_2024_style_reference`
- `runtime_pre_fast_root_quality_v1_normal_conversion_v3`

Default integrity tests enforce:

- retained IDs resolve from the registry
- the active retained ID list stays exactly aligned with the supported profile surface
- curated pool IDs are unique
- curated pool selectors are unique

## Artifacts And Cleanup

- Logged experiment runs go to `target/experiment-runs` by default.
- Raw logs are disposable local artifacts, not durable knowledge.
- Promote conclusions into `docs/automove-knowledge.md` or `docs/automove-archive.md` before cleaning.
- Use the cleanup script when the local artifact pile gets noisy:

```sh
./scripts/clean-experiment-artifacts.sh
./scripts/clean-experiment-artifacts.sh --dry-run
```

## Pro-Specific Tests

Pro comparison and promotion tests still exist, but they are manual ignored tests outside the client wrapper flow. Use them only for active profiles and active baselines. Do not use archived profiles for new promotion decisions.
