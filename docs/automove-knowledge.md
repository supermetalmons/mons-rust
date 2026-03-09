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
