# Automove Ideas

This is the live decision board for automove work. Keep it short and decision-oriented.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for workflow, `docs/automove-major-reset-plan.md` for the reset handoff, `docs/automove-knowledge.md` for durable rules, and `docs/automove-archive.md` for retired wave detail.

## Current State

- Public Pro routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` remains the retained search-only baseline.
- Retained profiles are only `shipping_pro_search` and `frontier_pro_v2_guarded`.
- The live experiment surface is Pro-only and multi-variant.
- The current mode is `structural-reset`.
- There is no live runtime hypothesis and no promotable challenger.
- Runtime source stays untouched until a new measured root feature or a test-only ProV4/root-policy candidate separates candidate wins from guarded saves across sampled and active evidence with low fragmentation.

## Latest No-Source Summary

- 2026-05-18 root-origin postprocess over the active Fast outcome-corpus produced no source permission. The dashboard stayed `not_promising` for `frontier_pro_v2_guarded` (`7-5`, `win_rate=0.5833`, `confidence=0.6128`, `129.91ms`), and postprocess ended `corpus_decision=postprocess_only`, `root_pool_decision=fragmented_repeated_root_pool_signal`, `guarded_delta_decision=fragmented_repeated_root_pool_guarded_delta`, `route_permission=postprocess_only`, `source_permission=no_source`.
- Root-origin provenance is retained as workbench evidence only: the root pool had `root_count=93`, `candidate_only_winning_policy_root_count=12`, `blocker_root_count=14`, `guarded_blocker_root_count=14`, and `same_state_blocker_root_count=3`, but root-origin exact and compound rollups had zero low-fragmentation repeated signals and remained fragmented or contaminated by blockers.
- 2026-05-18 active Fast outcome-corpus over the reset portfolio produced no source permission. The dashboard was `not_promising` for `frontier_pro_v2_guarded` (`7-5`, `win_rate=0.5833`, `confidence=0.6128`, `139.73ms`), and the postprocess ended `corpus_decision=postprocess_only`, `route_permission=postprocess_only`, `source_permission=no_source`.
- The only repeated class was active-Fast `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe` with `candidate_only_games=6` over two states, but it fragmented across `3` policies, `3` branch transitions, and `6` first-move pairs. Workbench had `blocked_candidate_axis_count=105`, `source_candidate_axis_count=0`, and top blockers were `fragmented_no_source` or `singleton_non_regressing`.
- Recent sampled/active outcome-corpus and root-pool work produced no source permission: routes stayed `coverage_gap`, `baseline_save_risk`, `singleton_no_source`, `no_candidate_route`, or fragmented by policy, branch, first move, budget, or guarded baseline saves.
- Recent ProV4/root-policy scouts were not promotable. They preserved guarded fallbacks but failed sampled dashboards or tiny Fast smokes, usually rotating weaknesses across Pro, Normal, Fast, and active blockers rather than creating a stable floor.
- Existing root-pool feature families are retained as diagnostic fields only. Do not write runtime selectors from current root-pool fields, guarded deltas, root-origin provenance, exact contexts, score terms, follow-up profiles, action profiles, carrier profiles, objective profiles, role/formation/mobility profiles, policy labels, branch labels, first moves, or singleton-heavy corpus rows.
- Detailed run notes live in `docs/automove-archive.md`; durable rules and grouped retired evidence live in `docs/automove-knowledge.md`.

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

## Next Command Sequence

First validate local hygiene:

```sh
./scripts/check-automove-hygiene.sh
```

Then continue only after adding or exposing another new below-fragmented measured corpus/root feature that is not already in the retired evidence list:

```sh
SMART_PRO_POLICY_MATRIX_PANEL_FILTER=active_blockers \
SMART_PRO_POLICY_MATRIX_DUEL_FILTER=vs_shipping_fast \
./scripts/run-automove-structural-scout.sh --outcome-corpus frontier_pro_v2_guarded
```

For a new test-only ProV4/root-policy candidate, register it as a sweep candidate with metadata first, then use:

```sh
./scripts/run-automove-structural-scout.sh --corpus <candidate>
```

Do not rerun archived root-pool slices or toggle scouts as source work. If continuing without a new candidate, first add or expose a genuinely new measured root feature that is not in the retired evidence list, then run the smallest outcome-corpus scout that emits that feature.

## Session End

1. Leave this file with one current state and one next command sequence.
2. Move durable lessons to `docs/automove-knowledge.md`.
3. Move probe diaries and failed wave detail to `docs/automove-archive.md`.
4. Run `./scripts/check-automove-hygiene.sh`.
5. Clean target logs/stamps separately with `./scripts/clean-experiment-artifacts.sh --dry-run` only when disposable evidence is no longer needed.
