# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired branch history to `docs/automove-archive.md`.

## Current Gate Snapshot

- Shipping Pro stays `runtime_current`.
- The only live Pro challenger is `runtime_pro_turn_engine_v30`.
- Last retained focused gate on the clean v30 line:
  - `pro-reliability`
  - `12` games
  - `win_rate=0.9167`
  - `confidence=0.9968`
  - `candidate_avg_ms=99.64`
- Last retained larger confirmation result on the same line:
  - `pro-reliability-confirm`
  - `32` games
  - `win_rate=0.7812`
  - `confidence=0.9989`
  - `candidate_avg_ms=100.11`
- Direct conclusion: speed is already acceptable. Promotion is blocked by larger-corpus quality and by the current-Normal matchup, not by the `700ms` move-time budget.

## Promotion Rule

- `pro-reliability` is the focused Pro gate.
- `pro-reliability-confirm` is the final promotion proof.
- Promote only after a completed confirmation run clears all three direct duels:
  - candidate Pro vs current Pro: `win_rate >= 0.90`, `confidence >= 0.99`, `candidate_avg_move_ms <= 700`
  - candidate Pro vs current Normal: `win_rate >= 0.90`, `confidence >= 0.99`, `candidate_avg_move_ms <= 700`
  - candidate Pro vs current Fast: `win_rate >= 0.90`, `confidence >= 0.99`, `candidate_avg_move_ms <= 700`
- `candidate_avg_move_ms` means candidate decision-selection time on candidate turns only. Do not count compile time, harness startup, or `game.process_input(...)`.
- A stalled or incomplete duel run is not promotable evidence.

## Live Code Surfaces

- Exact tactical / attack:
  - `exact_tactical_spirit_summary`
  - `exact_budget_one_tactical_counts_after_touched_locations`
  - `can_attack_target_on_board_with_hash`
  - `drainer_immediate_threats_uncached`
- Scoring / evaluation:
  - `ScoringBoardSummary::from_board`
  - `evaluate_preferability_with_context`
- Search / move generation:
  - `process_input_internal`
  - `collect_legal_transitions`
  - `classify_move_classes`

## Do Not Reopen

- Wrapper-only current-Normal fallbacks or reroutes
- Exact replay repairs without a broader duel story
- Acceptance-only macro-head clamps
- Cache-size, memo-shape, reserve-heavy, or hasher experiments without a direct quality hypothesis
- Generic search-budget or search-knob retunes without evidence that the live wall is on that surface
- White-only or black-only local seam repairs that do not move the broader `vs current Normal` wall
- Branches that only shift counters, disagreement counts, or hotspot timing while duel quality stays flat

## Iteration Rules

- Accept a revived line only if direct-vs-`runtime_current` evidence improves while staying under `700ms`.
- Kill a line immediately if it is only a speed regression with no quality story.
- Kill a line immediately if it preserves behavior and only shifts counters.
- Prefer cheap quality-ranking probes first, then the canonical Pro loop, then confirmation only after the focused gate earns more spend.

## Next Live Split

- Keep the retained challenger unchanged for now.
- Stay out of wrapper/fallback, current-Normal reroute, and early replay-fix branches.
- The next hypothesis should target the later white non-engine selector/search family that still survives after earlier replay seams are patched.
- The branch needs a direct story for why `runtime_pro_turn_engine_v30` should beat `runtime_current` more often, not merely mimic current Normal on a few traced openings.
