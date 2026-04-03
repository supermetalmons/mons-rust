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
- Kill result from 2026-04-04: a narrow ProV2 search-only rescue on the traced early engine-disabled opening states was not worth keeping.
  - I kept `runtime_pro_turn_engine_v30` on the wrapper surface, but for the early white turn-start and black turn-two mana-only branches I routed the candidate through a Pro-budget search-only config tagged as `TurnEngineMode::ProV2`, then added narrow reply-risk rescues for the traced white omitted efficient root and the traced black flat-root near-tie.
  - The branch was real, not flat: `guardrails` passed, `pro-triage` moved the target surface cleanly at `target_changed=11` / `off_target_changed=0`, and `runtime-preflight` passed.
  - It still failed `pro-reliability`: vs current Pro it improved to `win_rate=0.8333`, `confidence=0.9807`, `candidate_avg_ms=128.59`; vs current Normal it only reached `win_rate=0.5000`, `confidence=0.0000`, `candidate_avg_ms=126.25`.
  - The useful new evidence is that the original traced early seams were only part of the wall. The remaining `vs current Normal` losses moved to later engine-disabled states where the low-budget path still rejects a stronger head/prepass line that already matches Normal. In the `pro_reliability_v1_vs_normal` opening probe:
    - `opening_index=0 mirror=ab ply=6` on white, candidate `engine_disabled` chose `l10,6;l9,5` while current Normal used the forced-prepass line `l10,5;l9,6`.
    - `opening_index=1 mirror=ab ply=9` on black, candidate `engine_disabled` chose `l0,3;l1,3` while current Normal chose `l1,6;l2,6`; at `ply=10` it stayed on `l1,3;l2,2` while Normal still stayed on `l1,6;l2,6`.
- Next hypothesis after that kill: do not spend more on search-only ProV2 reply-risk rescues for the earliest engine-disabled openings. The next split should target the later engine-disabled head-candidate / forced-prepass adoption seam, where the low-budget guard disables the engine after the head already finds the stronger Normal-like line.
- Kill result from 2026-04-04: a narrow later black turn-two mana-only Normal fallback was also not worth keeping.
  - I left the existing `black_turn_two_mana_only` wrapper in place for the early states, but for `mons_moves_count >= 3` I rerouted `runtime_pro_turn_engine_v30` to the Normal release-safe runtime so the traced later black losses would take the current-Normal line `l1,6;l2,6`.
  - The branch passed the cheap screens cleanly again: `guardrails`, `pro-triage` (`target_changed=11`, `off_target_changed=0`), and `runtime-preflight`.
  - It still failed promotion badly: `pro-reliability` fell to `win_rate=0.7500`, `confidence=0.9270`, `candidate_avg_ms=83.12` against current Pro, and only improved to `win_rate=0.5833`, `confidence=0.6128`, `candidate_avg_ms=123.49` against current Normal.
  - Useful lesson: later black mana-only current-Normal fallbacks are still wrapper trades, not promotable fixes. They recover some `vs current Normal` ground, but they give too much back in the same-budget Pro duel.
- Next hypothesis after that kill: do not reopen later black turn-two mana-only wrapper fallbacks either. If that seam is real, solve it inside the engine-disabled ProV2 search path, likely through a targeted search-only clamp or a low-budget head/prepass adoption rule that reproduces the safe Normal-like progress root without abandoning the Pro-budget selector.
- Kill result from 2026-04-04: a narrow later black turn-two mana-only low-budget prepass adoption cut was also not worth keeping.
  - I kept the wrapper surface unchanged, added a ProV2-only later-black mana-only detector, and let `forced_low_budget_turn_engine_prepass_choice(...)` adopt a safe progress-like head/prepass on that shape.
  - The branch was real on the main gate: `guardrails` passed, `pro-triage` moved `target_changed=10` with `off_target_changed=0`, and `runtime-preflight` passed.
  - It also materially improved the same-budget duel: `pro-reliability` cleared current Pro at `win_rate=0.9167`, `confidence=0.9968`, `candidate_avg_ms=98.33`.
  - It still died on the only wall that mattered: `vs current Normal` stayed flat at `win_rate=0.5000`, `confidence=0.0000`, `candidate_avg_ms=98.31`.
  - The `pro_turn_planner_reliability_v1_vs_normal` loss harvest was no longer one later-black family. The remaining six losses were split across both colors: early black turn-two root/head disagreements (`repeat=0 opening=1 mirror=ab`, `repeat=1 opening=0 mirror=ba`), a later black turn-four seam (`repeat=1 opening=1 mirror=ab`), another black late tactical loss (`repeat=2 opening=0 mirror=ba`), and two white opening-root misses at `ply=0` (`repeat=0 opening=1 mirror=ba`, `repeat=2 opening=1 mirror=ba`).
  - Useful lesson: once the later-black in-path prepass rule is present, that seam is no longer the dominant `vs current Normal` wall. Do not keep spending on later-black-only low-budget prepass adoption from here.
- Next hypothesis after that kill: if `vs current Normal` is still the live wall, restart from a fresh `pro_turn_planner_reliability_v1_vs_normal` harvest and split the remaining losses by family instead of assuming one black turn-two seam. The obvious next split is to separate white opening/root-search misses from black head/search override families, not to reopen this later-black prepass line.
- Kill result from 2026-04-04: a white-opening-only split probe was also not worth keeping.
  - I restarted from the fresh `pro_turn_planner_reliability_v1_vs_normal` harvest and isolated the two `ply=0` white losses first.
  - White loss A (`repeat=0 opening=1 mirror=ba`) is not an early-turn-start seam. The live state is `turn=1`, `mons_moves=2`, no action/mana. Search already scores the plain-spirit root `l10,6;l9,7` well above `l10,5;l9,4` (`1719` vs `1474`), but `pick_root_move_with_reply_risk_guard(...)` builds a one-item shortlist `["l10,6;l9,7"]` and still returns `None`, so selector fallback keeps `l10,5;l9,4`.
  - White loss B (`repeat=2 opening=1 mirror=ba`) does not live in acceptance anymore. The live state is `turn=1`, `mons_moves=3`, no action/mana. A narrow `SafeSupermanaProgress` acceptance clamp can flip `accepted_after_search` to `false`, but the runtime still loses because root-search reply-risk itself chooses `l10,7;l9,6` while the heuristic/root-selection lane prefers `l10,5;l9,4`.
  - Useful lesson: the white `ply=0` family is split. One loss is a sole plain-spirit shortlist blind spot at `mons_moves=2`; the other is a deeper early-white root-search / reply-risk ranking seam at `mons_moves=3`. Acceptance-only white progress clamps are dead, and a white-only pass cannot be the promotion line by itself when it can clear at most two of the six remaining `vs current Normal` losses.
- Next hypothesis after that kill: do not reopen acceptance-only early-white progress clamps. If the white family is worth more spend, target either a general sole plain-spirit shortlist pickup in reply-risk guard or an early-white root-search / reply-risk clamp for the safe non-progress opening cluster. Promotion work should probably rejoin the black family or a shared search / reply-risk mechanism, because white-only repairs are too small to carry the whole `vs current Normal` wall.
