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

## Interview Guidance That Still Matters

- Attack the opponent drainer whenever a real same-turn attack exists.
- Safe supermana and safe opponent-mana captures outrank routine tactical pressure.
- Do not leave your own drainer vulnerable unless the move wins immediately or scores decisive mana.
- Move spirit off base aggressively when it creates score, denial, or real setup.
- Avoid mana movement that helps the opponent’s side of the board.
- Prefer short real routes and punish roundtrips or handoff-like progress that lose tempo.

## Current Improvement Direction

- Keep passive summaries lightweight and tactical exactness active-turn focused.
- Reuse cached tactical answers across root ranking, efficiency scoring, and tactical prepasses.
- Probe exact child progress only on boards that plausibly have target mana or spirit-assisted conversion worth preserving.
- Add fixtures around spirit scoring, safe supermana, opponent-mana conversion, drainer exposure, and opening black replies.
- Promote only candidates that pass `preflight`, prove a deterministic surface in `triage` or `pro-triage`, and then clear the earned duel path plus the release speed gates.
