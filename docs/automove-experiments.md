# Smart Automove Experiments

This is the current runbook for automove experiments. Historical logs and dead candidate catalogs were intentionally removed. The production runtime stays in the main game code; experimentation stays test-only under `src/models/automove_experiments/`.

Raw pro-player interview notes remain in `docs/automove-pro-strategy-interview.md`.

## Quick Reference

Use the promotion-first stage order for client-mode candidates:

1. Stage 0: tactical + interview guardrails

```sh
./scripts/run-experiment-logged.sh tactical_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
  cargo test --release --lib smart_automove_tactical_candidate_profile -- --ignored --nocapture
```

2. Stage 1: strict CPU non-regression vs `runtime_current` (all 3 modes)

```sh
./scripts/run-experiment-logged.sh stage1_cpu_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
  cargo test --release --lib smart_automove_pool_stage1_cpu_non_regression_gate -- --ignored --nocapture
```

Stage-1 CPU gate defaults to three seed tags (`stage1_cpu_v1`, `stage1_cpu_v2`, `stage1_cpu_v3`) and enforces per-seed caps: `fast <= 1.05x`, `normal <= 1.05x`, `pro <= 1.10x` versus `runtime_current`. Override seeds with `SMART_STAGE1_SEED_TAGS` (comma-separated, minimum 3).

3. Stage 1b: exact-lite diagnostics gate (required for exact-lite candidates, no-op for non-exact candidates)

```sh
./scripts/run-experiment-logged.sh exact_lite_diag_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
  cargo test --release --lib smart_automove_pool_exact_lite_diagnostics_gate -- --ignored --nocapture
```

4. Release speed gates (must pass before stage 2/3 promotion decisions)

```sh
cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture
cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture
```

5. Stage 2: fast screen

```sh
./scripts/run-experiment-logged.sh fast_screen_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
      SMART_GATE_BASELINE_PROFILE=runtime_release_safe_pre_exact \
  cargo test --release --lib smart_automove_pool_fast_screen -- --ignored --nocapture
```

6. Stage 2: progressive duel

```sh
./scripts/run-experiment-logged.sh progressive_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
      SMART_GATE_BASELINE_PROFILE=runtime_release_safe_pre_exact \
  cargo test --release --lib smart_automove_pool_progressive_duel -- --ignored --nocapture
```

7. Stage 2: promotion ladder

```sh
./scripts/run-experiment-logged.sh ladder_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
      SMART_GATE_BASELINE_PROFILE=runtime_release_safe_pre_exact \
  cargo test --release --lib smart_automove_pool_promotion_ladder -- --ignored --nocapture
```

For `pro`, use the mode-specific ignored tests listed below instead of the client pipeline.

Candidate naming convention for iteration waves:

- non-exact tuning: `runtime_eff_non_exact_v{N}`
- exact-lite tuning: `runtime_eff_exact_lite_v{N}`

## Module Layout

- `src/models/automove_experiments/profiles.rs`
  Retained profile registry and curated pool selection.
- `src/models/automove_experiments/harness.rs`
  Shared duel, pool, speed, seed, artifact, and tactical guardrail helpers.
- `src/models/automove_experiments/tests.rs`
  Fast default integrity tests and ignored experiment entrypoints.

The module is wired from `src/models/mons_game_model.rs` under `#[cfg(test)]`. It is not part of the shipped runtime build.

## Retained Profiles

Only these profiles remain available for experiment selection:

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

`runtime_current` is the shipped runtime profile and runs pre-exact defaults for all modes. `runtime_release_safe_pre_exact` is the immutable ladder baseline ID. `runtime_eff_non_exact_v1` is the default latency-first candidate line, and `runtime_eff_non_exact_v2` is the immediate non-exact follow-up iteration when v1 fails strength gates; `runtime_efficient_v1` is retained as a compatibility alias to the v1 selector. `runtime_eff_exact_lite_v1` enables only micro exact-lite root/static checks with explicit call budgets; full exact tactics remain disabled.

## Curated Pool

The curated pool is fixed to five distinct retained profiles:

- `runtime_current`
- `runtime_release_safe_pre_exact`
- `swift_2024_eval_reference`
- `swift_2024_style_reference`
- `runtime_pre_fast_root_quality_v1_normal_conversion_v3`

Pool integrity is enforced by default tests:

- retained IDs must resolve from the registry
- curated pool IDs must be unique
- curated pool selectors must be unique

## Current Shipped Runtime

The shipped automove API remains:

- `smartAutomoveAsync("fast")`
- `smartAutomoveAsync("normal")`
- `smartAutomoveAsync("pro")`

Current release discipline:

- wasm production stays single-shot; no post-return ticked search
- opening black-reply latency is guarded in `publish.sh`
- production legality, opening-book, and till-end guards stay in the main runtime tests
- exact-path tactics/evaluation stay disabled in shipped runtime defaults
- exact-lite work stays candidate-only until explicit promotion
- experiment candidates never ship until they pass the relevant promotion ladder

## Ignored Experiment Entry Points

Client mode:

- `smart_automove_pool_profile_speed_probe`
- `smart_automove_pool_pool_regression_diagnostic`
- `smart_automove_tactical_suite`
- `smart_automove_tactical_candidate_profile`
- `smart_automove_pool_stage1_cpu_non_regression_gate`
- `smart_automove_pool_exact_lite_diagnostics_gate`
- `smart_automove_pool_mode_comparison_report`
- `smart_automove_pool_fast_screen`
- `smart_automove_pool_progressive_duel`
- `smart_automove_pool_promotion_ladder`

Pro mode:

- `smart_automove_pool_pro_fast_screen_vs_normal`
- `smart_automove_pool_pro_fast_screen_vs_fast`
- `smart_automove_pool_pro_progressive_vs_normal`
- `smart_automove_pool_pro_progressive_vs_fast`
- `smart_automove_pool_pro_promotion_ladder`

## Release Steps

1. Keep the candidate in the test-only registry until it passes the relevant ladder.
2. Run the main runtime validation and the publish guardrails.
3. Update production runtime code directly in the main game model or scoring code.
4. Leave experiment-only selectors and baselines inside `src/models/automove_experiments/`.
5. Do not change release packaging or `runtime_current` semantics until the ladder result is clear and repeatable.
