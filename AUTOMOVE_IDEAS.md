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
- Kill result from 2026-04-04: a narrow early-black reply-floor near-tie search comparator was also not worth keeping.
  - I traced the two remaining early-black `turn=2`, `mons_moves=2`, mana-only losses (`repeat=0 opening=1 mirror=ab` and `repeat=1 opening=0 mirror=ba`) with the exact duel-config selector probe and confirmed the same diagnostic shape: inside `root_search_probe_for_test(...)`, ProV2 search already preferred the safer Normal-like root (`l0,7;l1,8` or `l0,6;l1,7`), but the real engine-disabled selector still returned the older root (`l0,7;l1,7` or `l0,3;l1,3`).
  - I tried a narrow ProV2 comparator that let those early-black safe flat mana-only roots keep a real search-score lead despite a modest reply-floor deficit. It did move the diagnostic search lane on the traced FENs: the acceptance/root-search probe flipped to the safer root on `repeat=1 opening=0 mirror=ba`.
  - The branch was still dead immediately because `loss_probe_decision(...)` did not move. The live selector path on those states still came back through the existing engine-disabled selected-root lane, so changing deeper reply-risk/search ordering below that surface did not change the actual move at all.
- Next hypothesis after that kill: do not spend more time on root-search-only comparators for the early-black mana-only family. If that family is revisited, only an actual in-path selector adoption/reroute can matter, and it is too small a slice to be a promotion line by itself. The next live split still needs to operate on a broader engine-disabled selector surface that can change final moves, not just diagnostic `root_search_probe_for_test(...)` output.
- Kill result from 2026-04-04: a narrow early-white turn-one three-move current-Normal contender adoption was also not worth keeping.
  - I targeted the two confirmed white `ply=0` losses at `turn=1`, `mons_moves=3`, no action or mana and only adopted the current-Normal move when it was already a near-top safe Pro contender on the v30 root surface (`root_rank <= 3`, heuristic gap `<= 64`, no vulnerability / handoff / roundtrip / immediate score window edge).
  - The branch fixed both exact loss FENs, preserved the existing white three-move tail regression, and passed `guardrails`, `pro-triage` (`target_changed=10`, `off_target_changed=0`), and `runtime-preflight`.
  - It was still dead at the real gate: `pro-reliability` cleared current Pro at `win_rate=0.9167`, `confidence=0.9968`, `candidate_avg_ms=96.38`, but stayed exactly flat against current Normal at `win_rate=0.5000`, `confidence=0.0000`, `candidate_avg_ms=97.11`.
  - Useful lesson: even a safe, rank-clamped current-Normal adoption on the early-white `mons_moves=3` family is still a wrapper-local repair. It can patch the traced openings without moving the broader `vs current Normal` wall.
- Next hypothesis after that kill: do not reopen early-white current-Normal wrapper adoptions, even when the Normal move is already a near-top safe Pro root. If the white `turn=1`, `mons_moves=3` family is worth more spend, it needs an in-path engine-disabled selector/search mechanism or a broader shared selector fix that changes more than those two openings.
- Kill result from 2026-04-04: a shared singleton plain-spirit reply-risk pickup was also not worth keeping.
  - I fixed the exact in-path control-flow hole in `pick_root_move_with_reply_risk_guard(...)`: when the reply-risk shortlist was all plain-spirit roots and only one such root survived, the helper returned `None` and let selector fallback choose an unrelated non-spirit root. On the live early-white opening FEN (`turn=1`, `mons_moves=2`) that flipped `runtime_pro_turn_engine_v30` from `l10,5;l9,4` to the searched plain-spirit root `l10,6;l9,7`.
  - The branch was real enough to matter. It passed the new function-level singleton test, the focused opening regression, `guardrails`, `pro-triage` (`target_changed=10`, `off_target_changed=0`), and `runtime-preflight`.
  - It still died at the focused duel: `pro-reliability` improved the current-Normal matchup to `win_rate=0.5833`, `confidence=0.6128`, `candidate_avg_ms=98.26`, but it regressed the same-budget Pro duel to `win_rate=0.8333`, `confidence=0.9807`, `candidate_avg_ms=103.73`.
  - Useful lesson: the singleton plain-spirit blind spot is a real selector lever, not a fake replay. But taking that shortcut globally is too broad; it recovers some current-Normal openings while reopening stronger current-Pro losses elsewhere.
- Next hypothesis after that kill: do not keep a global singleton plain-spirit fallback in reply-risk guard. If that family is revisited, clamp it to a narrower in-path early-opening selector surface or pair it with a Pro-preserving competition filter; the raw shared pickup is not promotable by itself.
- Kill result from 2026-04-04: a narrow white turn-one two-move late-tail admit was also not worth keeping.
  - I narrowed the dead singleton plain-spirit reply-risk pickup into an exact white opening seam. Selector-side singleton rescue only fired on `turn=1`, `mons_moves=2`, with no action or mana, when the lone plain-spirit shortlist root was already the safe `root_rank=0` choice and held a large score / efficiency lead over the best non-spirit alternative.
  - I paired that with a matching `runtime_pro_turn_engine_v30_guarded_inputs(...)` wrapper admit that only let the candidate v30 root through when the top safe plain-spirit root already beat the shared current-Pro fallback by a large heuristic / efficiency margin.
  - The branch fixed the exact traced regression, preserved the existing white three-move tail regression, and still passed `guardrails`, `pro-triage` (`target_changed=10`, `off_target_changed=0`), and `runtime-preflight`.
  - It was still dead at promotion time: `pro-reliability` reached `win_rate=0.9167`, `confidence=0.9968`, and `candidate_avg_ms=98.16` against current Pro, but stayed flat at `win_rate=0.5000`, `confidence=0.0000`, and `candidate_avg_ms=99.78` against current Normal.
  - Useful lesson: the white turn-one two-move late-tail seam is real and can be fixed narrowly without reopening the current-Pro duel, but it is too local to move the broader current-Normal wall by itself.
- Next hypothesis after that kill: do not spend more time on the white turn-one two-move late-tail seam in isolation. If the white opening family is revisited, combine it with another shared selector/search lever or a different white opening cluster instead of another standalone admit on this exact seam.
- Kill result from 2026-04-04: a narrow late-white unsafe progress-head rejection was also not worth keeping.
  - I targeted a repeated late-white `turn>=5`, `mons_moves=0`, action+mana seam where `runtime_pro_turn_engine_v30` accepted a `Safe*Progress` macro head even though the searched root already had the stronger eval on the same primary utility axes. On the traced FEN the head was `l7,5;l8,6`, the searched root was `l7,5;l7,6`, and forcing the searched root converted the representative `repeat=1 opening=0 mirror=ab` replay to a White win while forcing current-Normal's `l9,5;l8,3;l7,2` still lost.
  - The in-path branch was real enough to fix that exact seam. It passed the new late-white acceptance and decision regressions, preserved the existing white progress-tail and white three-move opening-tail regressions, and still passed `guardrails`, `pro-triage` (`target_changed=10`, `off_target_changed=0`), and `runtime-preflight`.
  - It still died at promotion time: `pro-reliability` stayed healthy against current Pro at `win_rate=0.9167`, `confidence=0.9968`, and `candidate_avg_ms=97.19`, but remained exactly flat against current Normal at `win_rate=0.5000`, `confidence=0.0000`, and `candidate_avg_ms=98.01`.
  - Useful lesson: this late-white unsafe progress-head seam is real and the searched root can be the better move locally, but isolated acceptance-only repairs on that seam are still too small to move the broader `vs current Normal` wall.
- Next hypothesis after that kill: do not spend more time on late-white unsafe progress-head acceptance-only repairs in isolation. If this family is revisited, target the broader late-white selector/root-choice lane that keeps surfacing the searched non-head root cluster across multiple openings, or combine it with another shared loss family instead of another exact head rejection.
- Kill result from 2026-04-04: a narrow late-white spirit-setup followup filter was also not worth keeping.
  - I targeted the later white `turn=7`, `mons_moves=2..3`, action+mana family where `runtime_pro_turn_engine_v30` let ProV2's extra spirit/plain-spirit competition keep quiet flat non-spirit roots alive in the search-scored filter, while current Normal collapsed the same safe lane to spirit-own-mana followups like `l8,6;l6,5;l6,4`.
  - The branch was real enough to matter: it fixed the two traced late-white followup FENs, passed the exact regressions, preserved the existing white progress-tail and white three-move opening-tail guards, and still passed `guardrails`, `pro-triage` (`target_changed=10`, `off_target_changed=0`), and `runtime-preflight`.
  - It still died immediately at the focused duel gate: `pro-reliability` stayed healthy against current Pro at `win_rate=0.9167`, `confidence=0.9968`, and `candidate_avg_ms=97.22`, but remained exactly flat against current Normal at `win_rate=0.5000`, `confidence=0.0000`, and `candidate_avg_ms=97.43`.
  - Useful lesson: even an in-path late-white search-scored filter that restores the quiet spirit-own-mana setup cluster is still too local by itself. It patches the traced later-white setup followups without moving the broader `vs current Normal` wall.
- Next hypothesis after that kill: do not spend more time on late-white quiet spirit-own-mana followup filters in isolation. If that family is revisited, target the broader search-scored spirit/setup competition surface that lets ProV2's extra plain-spirit competition suppress safe setup bundles across multiple later-white states, or pair it with another shared loss family instead of another exact late-white filter.
- Kill result from 2026-04-04: a broader late-white plain-spirit competition suppression cut was also not worth keeping.
  - I kept the same later-white `turn=7`, `mons_moves=2..3`, action+mana family, but moved the change deeper than the dead filter branch: instead of forcing setup bundles directly, I tried suppressing the ProV2 plain-spirit `negative_deny` / projection competition gates whenever a safe, close-score quiet spirit-own-mana setup bundle already existed in the same late-white lane.
  - That branch was dead immediately. It did not change either traced exact regression at all: `runtime_pro_turn_engine_v30` still chose the flat non-spirit roots `l5,10;l4,9` and `l4,9;l3,8` instead of the desired setup followup `l8,6;l6,5;l6,4`, while the existing white progress-tail and white three-move opening-tail guards still passed.
  - Useful lesson: the controlling blocker on this later-white setup family is not just the plain-spirit `negative_deny` or projection competition gate. Suppressing those probes alone leaves the exact decisions unchanged.
- Next hypothesis after that kill: if the later-white setup family is revisited again, inspect the remaining competition surfaces or the deeper search score itself instead of reopening only the plain-spirit competition guards. This branch proved that the blocker sits elsewhere on the search-scored spirit/setup lane.
- Kill result from 2026-04-04: a later-white quiet-state mid-turn engine disable was also not worth keeping.
  - I moved from selector/search competition cuts to the live ProV2 guard surface and tried extending the existing mid-turn tactical disable so later white `turn>=6`, `mons_moves>=2`, action+mana states with quiet exact-opportunity context would drop back to the non-turn-engine selector path that current Pro/Normal already use.
  - That branch was dead immediately. It did not move either traced exact regression at all: the setup-bundle tests still failed with `runtime_pro_turn_engine_v30` choosing `l5,10;l4,9` and `l4,9;l3,8` instead of `l8,6;l6,5;l6,4`, while the kept white progress-tail and white three-move opening-tail guards still passed.
  - Useful lesson: the later-white setup family is not captured by a simple exact-opportunity quiet-state mid-turn disable. State-level context alone is too weak; the blocker sits deeper in the root-surface evidence.
- Next hypothesis after that kill: if this later-white family is revisited again, derive the guard or reroute from actual root-surface structure instead of only the exact-opportunity state summary. The exact traced states still require a root-aware split, not a broad quiet-state engine disable.
- Kill result from 2026-04-04: a combined current-Normal reroute bundle was also not worth keeping.
  - I combined three lines in one branch: broad current-Normal reroutes on the existing guarded black surfaces (`turn=2` mana-only and `turn=4` turn-start action+mana), plus exact current-Normal fallbacks for the two live white opening FENs and the late-black `turn=2`, `mons_moves=5`, action+mana tactical FEN.
  - The branch was real enough to matter locally. It kept the full `runtime_pro_turn_engine_v30_profile_` focused fixture bundle green, and the fresh six-opening `pro_turn_planner_reliability_v1_vs_normal` sweep improved from `6/12` wins to `8/12`.
  - It still died before the canonical loop because the remaining four losses proved the opening/current-Normal reroutes were not the real wall:
    - `repeat=0 opening=0 mirror=ba` still lost later on black through `engine_post_search` at `ply=17..18` (`l1,5;l1,6` / `l1,6;l2,7` vs current-Normal's deeper disabled line).
    - `repeat=0 opening=1 mirror=ba` still lost later on white through `engine_disabled` at `ply=1` and again at `ply=16`, even after the opening fallback.
    - `repeat=2 opening=1 mirror=ab` still lost later on black through `engine_post_search` at `ply=17..18`, with accepted engine heads continuing to override the deeper disabled line.
    - `repeat=2 opening=1 mirror=ba` still lost later on white through `engine_disabled` at `ply=1` and `ply=12`.
  - Useful lesson: fixing the opening/current-Normal seams is not enough anymore. The live `vs current Normal` wall has moved deeper into later-game `engine_disabled` and `engine_post_search` branches across both colors, so combining more wrapper reroutes just repairs the seed positions and then loses again later.
- Next hypothesis after that kill: do not reopen combined current-Normal wrapper reroutes on the opening/early-black families. If this wall is revisited, target the deeper later-game branches surfaced by the fresh opening probes: the late-black `engine_post_search` lane around `ply=17..18` and the deeper white `engine_disabled` selector branches after the opening move, not more opening fallbacks.
