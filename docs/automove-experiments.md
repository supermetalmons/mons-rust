# Smart Automove Experiments

This is the current runbook for automove experiments. Historical logs and dead candidate catalogs were intentionally removed. The production runtime stays in the main game code; experimentation stays test-only under `src/models/automove_experiments/`.

Raw pro-player interview notes remain in `docs/automove-pro-strategy-interview.md`.

## Quick Reference

Use the same three-step pipeline for client-mode candidate work:

1. Fast screen

```sh
./scripts/run-experiment-logged.sh fast_screen_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
      SMART_GATE_BASELINE_PROFILE=runtime_release_safe_pre_exact \
  cargo test --release --lib smart_automove_pool_fast_screen -- --ignored --nocapture
```

2. Progressive duel

```sh
./scripts/run-experiment-logged.sh progressive_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
      SMART_GATE_BASELINE_PROFILE=runtime_release_safe_pre_exact \
  cargo test --release --lib smart_automove_pool_progressive_duel -- --ignored --nocapture
```

3. Promotion ladder

```sh
./scripts/run-experiment-logged.sh ladder_<candidate> -- \
  env SMART_CANDIDATE_PROFILE=<candidate> \
      SMART_GATE_BASELINE_PROFILE=runtime_release_safe_pre_exact \
  cargo test --release --lib smart_automove_pool_promotion_ladder -- --ignored --nocapture
```

For `pro` and `ultra`, use the mode-specific ignored tests listed below instead of the client pipeline.

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
- `swift_2024_eval_reference`
- `swift_2024_style_reference`
- `runtime_pre_fast_root_quality_v1_normal_conversion_v3`
- `runtime_pre_pro_promotion_v1`

`runtime_current` is the shipped runtime profile. `runtime_release_safe_pre_exact` is the frozen promotion baseline sourced from the last release-safe pre-exact runtime. `base` intentionally stays as the default candidate name for cheap local sanity checks.

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
- `smartAutomoveAsync("ultra")`

Current release discipline:

- wasm production stays single-shot; no post-return ticked search
- opening black-reply latency is guarded in `publish.sh`
- production legality, opening-book, and till-end guards stay in the main runtime tests
- experiment candidates never ship until they pass the relevant promotion ladder

## Ignored Experiment Entry Points

Client mode:

- `smart_automove_pool_profile_speed_probe`
- `smart_automove_pool_pool_regression_diagnostic`
- `smart_automove_tactical_suite`
- `smart_automove_tactical_candidate_profile`
- `smart_automove_pool_fast_screen`
- `smart_automove_pool_progressive_duel`
- `smart_automove_pool_promotion_ladder`

Pro mode:

- `smart_automove_pool_pro_fast_screen_vs_normal`
- `smart_automove_pool_pro_fast_screen_vs_fast`
- `smart_automove_pool_pro_progressive_vs_normal`
- `smart_automove_pool_pro_progressive_vs_fast`
- `smart_automove_pool_pro_promotion_ladder`

Ultra mode:

- `smart_automove_pool_ultra_fast_screen_vs_pro`
- `smart_automove_pool_ultra_fast_screen_vs_normal`
- `smart_automove_pool_ultra_fast_screen_vs_fast`
- `smart_automove_pool_ultra_progressive_vs_pro`
- `smart_automove_pool_ultra_progressive_vs_normal`
- `smart_automove_pool_ultra_progressive_vs_fast`
- `smart_automove_pool_ultra_promotion_ladder`

## Release Steps

1. Keep the candidate in the test-only registry until it passes the relevant ladder.
2. Run the main runtime validation and the publish guardrails.
3. Update production runtime code directly in the main game model or scoring code.
4. Leave experiment-only selectors and baselines inside `src/models/automove_experiments/`.
5. Do not change release packaging or `runtime_current` semantics until the ladder result is clear and repeatable.
