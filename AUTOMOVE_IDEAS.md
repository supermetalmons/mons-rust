# Automove Ideas

This is the live decision board for automove work. Keep it short and decision-oriented. Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for workflow, `docs/automove-major-reset-plan.md` for the reset handoff, `docs/automove-knowledge.md` for durable rules, and `docs/automove-archive.md` for retired wave detail.

## Current State

- Public Pro routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` remains the retained search-only baseline.
- Retained profiles are only `shipping_pro_search` and `frontier_pro_v2_guarded`.
- The live experiment surface is Pro-only and multi-variant.
- The current mode is `structural-reset`.
- There is no live runtime hypothesis and no promotable challenger.
- Latest sampled/active Fast scouts found no source permission: sampled Fast was `no_candidate_route`; active Fast had repeated candidate axes, but every route remained singleton or fragmented by policy, branch, or first move, and ProV4 root-pool / guarded-delta rows were also fragmented.
- Latest ProV4 cross-budget static-eval consensus scout was not promotable: it preserved guarded fallbacks after an unsafe loose smoke, then fast-failed the sampled dashboard at Pro `6-6` and never appeared as a policy-corpus winner.
- Runtime source stays untouched unless a new corpus/root feature separates candidate wins from baseline saves across sampled and active evidence with low fragmentation.

## Reset Portfolio

Use this retained portfolio for policy-corpus and outcome-corpus reset work:

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

These stale test-only sweep candidates are pruned from the active runner surface and must not be reopened as direct targets:

```text
frontier_pro_v2_no_late_black_fallback,
frontier_pro_v2_head_rerank,
frontier_pro_v2_no_spirit_family,
frontier_pro_v2_no_mid_tactical_guard,
frontier_pro_v2_expansion_224
```

## Retired Source Evidence

Do not write runtime selectors from:

- existing policy labels, branch labels, exact contexts, first moves, variants, or singleton-heavy corpus rows;
- broad zero-window exact-pressure classes or current exact-pressure deltas;
- current active Fast lower-live safe-step / ManaTempo pressure, ProV4 root-pool, guarded-delta, root-ordering profile, root-preservation, reply-floor, root-safety, utility/rank, cross-budget static-eval consensus, forced-root feature-axis, root-pool provenance, forced-root pool JSONL, root trajectory, race geometry, root-pool contrast, outcome contrast, family-overlap, state-discriminator, broad token, vulnerable-baseline token, or utility/rank token-pair evidence;
- current post-root feature families: exact pressure, board-resource custody/material, scoreboard/turn-budget, legal-transition fanout, attack-exposure, support-guard, territory, mana-path, consumable, engagement, mobility, action-threat, role-state/loadout, base-recovery, lane-shape, root-transition/event footprint, worst-reply event footprint, and immediate reply-spectrum shape.

Those paths produced no source permission because the evidence stayed `coverage_gap`, `baseline_save_risk`, `no_candidate_route`, singleton-only, policy/branch/pair fragmented, shared with blockers, or contaminated by guarded baseline saves. Their detailed run notes are archived in `docs/automove-archive.md`; durable rules live in `docs/automove-knowledge.md`.

## Next Command Sequence

First validate local hygiene:

```sh
python3 -m py_compile \
  scripts/summarize-automove-policy-matrix-log.py \
  scripts/summarize-automove-outcome-jsonl.py \
  scripts/summarize-automove-forced-root-oracle-log.py \
  scripts/summarize-automove-forced-root-pool-jsonl.py

./scripts/cleanup-automove-iteration-artifacts.sh --dry-run
```

Do not rerun the current sampled/active Fast portfolio slices as source work. They are now archived as no-source. If continuing reset-mode diagnostics without a new candidate, first add or expose a genuinely new measured root feature that is not in the retired evidence list, then run the smallest outcome-corpus scout that emits that feature.

For a new test-only ProV4/root-policy candidate, register it as a sweep candidate with metadata first, then use:

```sh
./scripts/run-automove-structural-scout.sh --corpus <candidate>
```

## Session End

1. Leave this file with one current state and one next command sequence.
2. Move durable lessons to `docs/automove-knowledge.md`.
3. Move probe diaries and failed wave detail to `docs/automove-archive.md`.
4. Clean scratch artifacts with `./scripts/cleanup-automove-iteration-artifacts.sh --dry-run`.
5. Clean target logs/stamps separately with `./scripts/clean-experiment-artifacts.sh --dry-run` only when disposable evidence is no longer needed.
