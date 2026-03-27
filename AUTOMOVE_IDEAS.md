# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired branch history to `docs/automove-archive.md`.

## Current Gate Snapshot (2026-03-27)

- Shipping Pro stays `runtime_current`.
- The only live Pro challenger is `runtime_pro_turn_engine_v30`.
- Focused gate result:
  - `pro-reliability`
  - `12` games
  - `win_rate=0.6667`
  - `confidence=0.8062`
  - `candidate_avg_ms=79.61`
- Confirmation gate result:
  - `pro-reliability-confirm`
  - `32` games
  - `win_rate=0.5938`
  - `confidence=0.8115`
  - `candidate_avg_ms=84.52`
- Direct conclusion: speed is already acceptable. Promotion is blocked by quality against `runtime_current`, not by the `700ms` move-time budget.

## Promotion Rule

- `pro-reliability` is now the focused Pro gate.
- `pro-reliability-confirm` is the final promotion proof.
- Promote only after a completed confirmation run clears all three:
  - `win_rate >= 0.90`
  - `confidence >= 0.99`
  - `candidate_avg_move_ms <= 700`
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

## Reopen Under The New Goal

Rank revived work only by its chance to improve direct-vs-`runtime_current` results while staying under `700ms`.

1. `curated dominance` and very small direct-duel quality probes
   - Use the existing curated opportunity dominance probe as the first cheap read on revived branches.
   - Treat it as diagnostic ranking evidence only, never as promotion proof.
   - Use it to screen branches before spending the full Pro duel loop.

2. Budget-`2` and `>1` after-window tactical-summary families
   - Reopen only as quality-first experiments.
   - These branches drastically changed tactical query structure and local decision context.
   - They were previously rejected mostly because hotspot wall-clock got worse under the old completion-first framing, not because they were proven tactically worse.
   - Accept them only if they improve direct duel evidence or curated dominance while staying well under the move-time cap.

3. Mixed scoring/search quality branches that change root choice
   - Focus on behavior-level families, not cache-shape micro-optimizations:
     - scoring-side tactical preference
     - child ordering / preferability interaction
     - move-class / transition-order behavior
     - omitted concrete spirit-setup / progress competitors versus plain-spirit or safe non-spirit shortlist winners
   - Only spend here when the branch has a direct story for why `runtime_pro_turn_engine_v30` should beat `runtime_current` more often, not merely do the same work with a different memo layout.

## Do Not Reopen Just Because 700ms Is Relaxed

- The relaxed move-time cap does not make pure parity-preserving speed regressions interesting again.
- Keep these categories parked unless a new branch also has a clear quality story:
  - larger caches or reserve-heavy tables
  - custom hashers
  - reverse carrier maps or reverse pickup maps
  - board-scoped memo-shape experiments
  - broad search-budget or reply-budget increases without a new selector/root-choice hypothesis
  - wrapper-level `runtime_current` fallback widening or narrowing
  - projected-completion acceptance clamps for traced plain-spirit followup bundles
  - attack-target setup tables
  - sort removal
  - full-board direct scans on sparse search helpers
- Rule: if a branch preserved behavior and was only slower or flat, the new `700ms` ceiling does not justify reopening it.
- Rule: disagreement-count shrink by itself is not enough to retain a branch. If traced losses move but direct duel win rate stays flat or regresses, park the wrapper/fallback idea and move deeper into selector/search logic.

## Iteration Rules

- Accept a revived line if direct-vs-`runtime_current` evidence improves and the candidate stays under `700ms`, even if hotspot probes are somewhat slower than the retained baseline.
- Kill a line immediately if it is only a speed regression with no quality story.
- Kill a line immediately if it preserves behavior and only shifts counters.
- Prefer cheap quality-ranking probes first, then the canonical Pro loop, then confirmation only after the focused gate earns more spend.
