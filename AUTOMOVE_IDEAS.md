# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired branch history to `docs/automove-archive.md`.

## Current Gate Snapshot (2026-03-27)

- Shipping Pro stays `runtime_current`.
- The only live Pro challenger is `runtime_pro_turn_engine_v30`.
- Focused gate result:
  - `pro-reliability`
  - `12` games
  - `win_rate=0.9167`
  - `confidence=0.9968`
  - `candidate_avg_ms=99.64`
- Last confirmation result:
  - `pro-reliability-confirm`
  - `32` games
  - `win_rate=0.7812`
  - `confidence=0.9989`
  - `candidate_avg_ms=100.11`
- Direct conclusion: speed is already acceptable and the focused Pro-vs-Pro gate clears, but promotion is still blocked by larger-corpus quality against `runtime_current`, not by the `700ms` move-time budget. Under the stronger rule, future promotion evidence must also clear the same floor against current Normal.

## Promotion Rule

- `pro-reliability` is now the focused Pro gate.
- `pro-reliability-confirm` is the final promotion proof.
- Promote only after a completed confirmation run clears both direct duels:
  - candidate Pro vs current Pro: `win_rate >= 0.90`, `confidence >= 0.99`, `candidate_avg_move_ms <= 700`
  - candidate Pro vs current Normal: `win_rate >= 0.90`, `confidence >= 0.99`, `candidate_avg_move_ms <= 700`
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
  - `pro-vs-normal` dominance-gap branches as a primary driver when the gap set is mostly black late acceptance/projection seams and only one white early root-search miss overlaps the blocked confirm surface
  - narrow white turn-three action+mana wrapper compares to the shared current line when they only drop the focused gate back to the old `0.8333 / 0.9807` band
  - narrow white turn-three mana-only wrapper reroutes, including the `mons_moves==1` and `mons_moves==3` score-window followup slices, when they only keep confirmation quality flat
  - exact first-disagreement profile fallbacks for traced live loss FENs
  - projected-completion acceptance clamps for traced plain-spirit followup bundles
  - single black projected non-concrete progress or immediate-score override blocks for traced opening-B continuations
  - dormant ProV2 config toggles without direct duel lift, including `enable_turn_engine_mid_turn_progress_guard` and `enable_turn_engine_lazy_oracle_score_window_projection`
  - positive attack/safety-edge plain-spirit acceptance clamps
  - single traced pair clamps that only fix one black safe-followup acceptance case or one late-white setup-progress comparator
  - single early-white setup-progress acceptance clamps that only clear a focused replay or small-gate seam while leaving confirmation flat
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

## Next Live Split

- Keep the retained challenger unchanged for now. On 2026-04-03 I tried a shared early-white setup-progress force/focus cut that blocked non-tactical turn-three `spirit_own_mana_setup_now + progress` heads when a safe non-spirit progress root already matched the same path surface.
- That branch really did fix the first confirm-aligned `opening 0 / ab` seam (`l9,6;l7,4;l7,3` -> `l8,7;l7,8`), passed `guardrails`, `pro-triage` (`target_changed=10`, `off_target_changed=0`), `runtime-preflight`, and the focused `pro-reliability` gate (`0.9167 / 0.9968 / 94.50ms`).
- It is still dead as a promotion line because `pro-reliability-confirm` stayed exactly flat on quality at `0.7812 / 0.9989` and slowed from the retained `100.11ms` to `103.92ms`.
- Next hypothesis: stay out of wrapper/fallback and early-white setup-progress force/focus repairs. The live wall on that lane moved later and out of engine injection: `opening 0 / ab`, `ply=18`, plain search chooses `l7,7;l8,8` while `runtime_current` prefers `l7,4;l8,4`. The next split should target that later white non-engine selector/search family instead of reopening earlier replay fixes.
- Kill result from 2026-04-03: a broader early-opening `runtime_current` Normal fallback on the white turn-one late tail and black turn-two mana-only families was not worth keeping.
  - It fixed the traced opening states, passed `guardrails`, `pro-triage` (`target_changed=12`, `off_target_changed=0`), and `runtime-preflight`.
  - It still died immediately on the focused duel gate: vs current Pro it fell to `win_rate=0.6667`, `confidence=0.8062`, `candidate_avg_ms=97.68`; vs current Normal it fell further to `win_rate=0.4167`, `confidence=0.0000`, `candidate_avg_ms=151.36`.
  - Single-opening sequence overrides also failed to convert the representative losses, so this is not a one-move replay seam.
- Next hypothesis after that kill: do not reopen early-opening current-Normal wrapper fallbacks. The shared wall is deeper in pro-budget plain-search / reply-risk selection on engine-disabled openings where `runtime_pro_turn_engine_v30` and `runtime_current` Pro agree but `runtime_current` Normal prefers a different root. The next split should target that reply-risk / root-search lane, not more wrapper reroutes.
