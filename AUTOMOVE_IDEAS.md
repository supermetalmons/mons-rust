# Automove Ideas

This is the live decision board for automove work. Keep it short.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the operator flow, `docs/automove-major-reset-plan.md` for the current reset handoff, `docs/automove-knowledge.md` for durable rules, and `docs/automove-archive.md` for retired wave detail.

## Current State

- Public Pro routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` remains the retained search-only baseline.
- The live experiment surface is Pro-only and multi-variant.
- Retained profiles are only `shipping_pro_search` and `frontier_pro_v2_guarded`.
- The current mode is `structural-reset`.
- There is no live runtime hypothesis and no promotable challenger.
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

Default reset-mode entrypoint:

```sh
./scripts/run-automove-structural-scout.sh --outcome-corpus frontier_pro_v2_guarded
```

Start filtered before widening:

```sh
SMART_PRO_POLICY_MATRIX_PANEL_FILTER=active_blockers \
SMART_PRO_POLICY_MATRIX_DUEL_FILTER=vs_shipping_fast \
./scripts/run-automove-structural-scout.sh --outcome-corpus frontier_pro_v2_guarded
```

Use `--corpus` when the next question is first-winning policy coverage or mechanism classes:

```sh
./scripts/run-automove-structural-scout.sh --corpus frontier_pro_v2_guarded
```

For a new test-only ProV4/root-policy candidate, register it as a sweep candidate first, then run:

```sh
./scripts/run-automove-structural-scout.sh --corpus <candidate>
```

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
- Broad zero-window safe-pressure classes are contaminated by baseline-better saves and cannot justify runtime selectors.
- Raw ProV2, no-selected-followup, full-scored reply guard, no-low-budget, alternating-white, and white-opening utility policies are diagnostic components, not retained challengers.
- Root-origin and continuation-probe ProV4 attempts are retired unless they add a new discriminator below current score, rank, family, safety, progress, and `TurnEngineUtility` fields.
- Future source-bearing work should be one of: Outcome Corpus V2, a test-only ProV4 unified root policy, or a corpus-calibrated utility feature.

## Latest Gate Snapshot

- Date: `2026-04-28`
- Shipping decision: public Pro remains on `frontier_pro_v2_guarded`.
- Release containment: public `Pro` dispatch still routes through retained runtime code; `automove_experiments` remains under `#[cfg(test)]`.
- Latest retained package direction: no runtime source retained from recent structural reset work.
- Latest reset evidence: current policy/output corpus routes are useful diagnostics but not source permission while exact contexts, move pairs, or clean cross-budget mechanisms remain singleton-heavy.

## Session End Checklist

1. Update this file with the current state and exactly one next command sequence.
2. Move durable rules into `docs/automove-knowledge.md`.
3. Move retired wave detail into `docs/automove-archive.md`.
4. Clean disposable artifacts after validation:

```sh
./scripts/clean-experiment-artifacts.sh --dry-run
./scripts/clean-experiment-artifacts.sh
```

5. Leave exactly one clear next hypothesis, or explicitly record that there is no live challenger.
