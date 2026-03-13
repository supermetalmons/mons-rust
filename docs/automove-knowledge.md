# Automove Knowledge

This document keeps only durable lessons that should shape future automove work.

## Durable Strategy Signals

- Strong root filtering beats wider raw enumeration when the filters are tactical and cheap.
- Drainer safety needs close-to-hard treatment in production search; soft penalties alone miss obvious blunders.
- Root reply-risk guards and efficiency tie-breaks are worth keeping because they remove fake-good moves before deeper search.
- Production wasm must stay single-shot and predictable. Deferred or post-return search is still not release-safe.
- Opening-specific latency guardrails matter. A stronger search that stalls on the first real black reply is not promotable.
- Immutable reference baselines are useful even when weaker than the current runtime because they keep calibration honest.

## Repeated Failure Modes

- Large candidate catalogs slow iteration and hide the actual question. Keep the active registry small.
- Raw logs are not knowledge. If a run matters, promote the conclusion into docs immediately.
- Reject fast, archive fast, and keep only promotable signal in the live experiment surface.
- If retained profiles cannot move a triage surface in `triage-calibrate`, the fixture pack is not ready for candidate work yet.
- Calibration probes need to target the actual helper or budget lever for the surface. Whole-board selected-root equality can hide real reply-risk, shortlist, or exact-lite differences.
- If `reply_risk` audit-screen contradicts triage, patch the surface before trying more candidates. The right fix is candidate-resolved shortlist evidence, not a weaker global triage bar.
- After a confirmed audit contradiction is fixed, replay the candidate from the earned duel stage. Do not pay the audit lane twice on the same candidate.
- Once a surface is calibrated, do not pay the stage-1 CPU gate until after triage. The CPU gate is too expensive to sit in front of an instant surface reject.
- After splitting preflight from duel stages, treat any remaining audit slowness as duel-budget cost. The next workflow cut is duel-budget reduction or screen deduplication, not more preflight reshuffling.
- Pooled client aggregate can falsely reject a mode-specific lead. For `fast` or `normal` promotion attempts, the target mode should be the only improvement bar and the other client mode should stay a non-regression check.
- Even after fixing the pooled false reject, the first earned duel stage can still stall on a borderline fast-only lead. If the target-aware `fast-screen` cannot return promptly, the next workflow cut is cached top-off or a cheaper target-aware screen, not another candidate retry.
- Default new candidates to `runtime_current`. Use retained non-production profiles only for calibration, references, or explicit audit lanes.
- Keep ignored harness test names unique. `cargo test` filters by substring, so overlapping names can accidentally run extra stages and distort the loop.
- The generic release opening black-reply speed gate measures production runtime only. Do not use it as a candidate filter before `pro-triage`; keep it for promotion time unless a candidate-aware opening speed probe exists.
- Keep one audit lane, not a weaker default. Spot-check roughly every fifth clean triage reject with a cheap duel so the false-negative rate is measured instead of guessed.
- A preflight-free audit helped, but it was still expensive in practice: the reply-risk audit dropped from about `347s` to about `292s` once stage-1 CPU and exact-lite checks were removed. That proved the split worked and also showed that duel cost now dominates.
- For `reply_risk`, selected-root equality is too coarse. Compare candidate-resolved shortlist evidence, including reply-floor snapshots on the shortlisted roots, so clean-ordering and shortlist-width changes are visible in triage.
- Heavy exact evaluation on both colors in every node is too expensive for production-facing turns.
- Client-mode experiment gating gets noisy if `pro` is mixed into the default speed check. Default stage-1 gating to `fast` and `normal`, use repeated probes, and gate on the median ratio.
- Shipping experiment code before it clears the promotion pipeline is too risky.
- Generic meta-tooling for automated iteration helped less than direct code, direct gates, and short written lessons.
- Scoring weight changes (±30 tweaks, dormant feature activation, multi-path activation, mana-race fields) consistently pass triage by inflating heuristic scores but are pure noise at duel scale. Do not invest duel budget on scoring-only candidates.
- Search-structure-only changes (futility pruning, PVS, quiet reductions, aspirations) at Normal depth-3 are invisible to triage because fixture positions have dominant top moves unaffected by ordering or pruning efficiency.
- History heuristic is dominated by TT best-child (+2400) and killer (+1200) bonuses at both Normal and Pro depth. The additional history bonus (capped at 800) cannot change ordering in any position that matters.
- The config knob space is completely exhausted across all three modes (Mar 2026). Every SmartSearchConfig structural feature has been individually tested. Future improvement requires new code (new evaluation features, new search algorithms), not config permutations.
- Quiescence search concept is proven (Normal 46W-26L, δ=0.139) but `ranked_child_states()` per activation is too expensive. Need cheaper move generation for quiescence or structural redesign before it can ship.
- Breadth-over-depth (disabling extensions) is fundamentally stronger than extending promising lines. The search wins by evaluating more root candidates at nominal depth rather than extending a few children deeper.
- Pro CPU ratio gate (1.60x minimum) blocked the strongest candidate ever found (+13% across 1,488 games). The gate ensures Pro "feels premium" but the stronger search is paradoxically cheaper.
- Pro vs_fast regression is a hard gate. Two Pro candidates with strong vs_normal signals (+12-13%) failed because they regressed against Fast mode baseline.

## Interview Guidance That Still Matters

- Attack the opponent drainer whenever a real same-turn attack exists.
- Safe supermana and safe opponent-mana captures outrank routine tactical pressure.
- Do not leave your own drainer vulnerable unless the move wins immediately or scores decisive mana.
- Move spirit off base aggressively when it creates score, denial, or real setup.
- Avoid mana movement that helps the opponent’s side of the board.
- Prefer short real routes and punish roundtrips or handoff-like progress that lose tempo.

## Current Improvement Direction

- Config knob space is exhausted. All future candidates must involve new code, not config permutations.
- Pro quiescence search is the most promising active path: concept proven at Normal (46W-26L), Pro has 5x more CPU headroom.
- Cheaper move generation for quiescence (avoiding full `ranked_child_states()`) could unlock Normal quiescence too.
- Shared tactical/exact-lite cache reuse could reclaim duplicated work across root ranking and tactical prepasses.
- Safety fixtures (stuck-state, bounded-progress) remain backlog infrastructure work.
- Promote only candidates that pass `preflight`, prove a deterministic surface in `triage` or `pro-triage`, and then clear the earned duel path plus the release speed gates.
