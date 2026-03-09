# Automove Archive

This document keeps compressed history for retired automove profiles and experiment waves. These IDs are archived context only. They are not part of the active experiment registry and should not be used for new promotion decisions.

## Retired Runtime Snapshots

- `runtime_historical_0_1_109`
  Fast and normal settings proved that reply-risk guardrails, tactical prepass, and stronger drainer safety mattered more than wider search.
- `runtime_historical_0_1_110`
  Helped validate the opening-reply latency problem and the need for stricter pro-context tuning instead of one shared pro shape.
- `runtime_historical_post_0_1_110_6c3d5cb`
  Showed that post-0.1.110 pro tuning could recover strength, but still needed cleaner promotion discipline.
- `runtime_historical_post_0_1_110_a70b842`
  Confirmed the stronger pro budget direction, but remained a historical checkpoint rather than a stable baseline.
- `runtime_historical_pre_exact_e9a05ce`
  Was the strongest archived pre-exact pro wave, but it is now historical context rather than an active baseline.

## Retired Candidate Line

- `runtime_eff_non_exact_v3`
  This hybrid line mixed current fast/normal with archived pre-exact pro behavior. It answered a one-time comparison question and is now retired because keeping it selectable in the active registry only adds noise.
- `runtime_root_signal_v1`
  This March 9, 2026 low-CPU signal wave improved cheap safe-progress detection, but it failed `fast-screen` against `runtime_release_safe_pre_exact` at `24` games with aggregate delta `-0.0417`. Keep the lesson that cheap safety-aware setup signals are useful, but do not promote a signal-only wave without stronger reply selection.
- `runtime_root_reply_v1`
  This March 9, 2026 low-CPU reply-selection wave cleared `preflight` and a constrained `fast-screen`, but follow-up diagnostics did not show promotable normal-side strength. Focused mode checks showed candidate `fast` at `+0.0208` vs baseline `fast` and `0.0000` vs baseline `normal` over `48` games each, while candidate `normal` ran `-0.0417` vs baseline `fast` over `48` games and `-0.0938` vs baseline `normal` after `32` games before the report was stopped. Keep the lesson that cheap reply-risk cleanup can be acceptable in `fast`, but do not carry the aggressive normal shortlist and penalty tuning forward.
- `drainer_exposure_v1`
  This March 9, 2026 heuristic penalty wave added a root `drainer_exposure_penalty` (200 fast, 300 normal) for uncompensated own-drainer-vulnerable moves plus wider `root_drainer_safety_score_margin` (2800 fast, 5200 normal). Cleared `preflight` with zero CPU cost and passed `fast-screen` at δ=+0.0089 (112 games, conf=0.538). Progressive duel showed δ=-0.0069 at 144 games with fast dead even (36-36) and normal slightly behind (35-37). The penalty alone at these magnitudes does not shift move selection often enough to yield measurable strength. Keep the lesson that the existing late-stage drainer-safety filter already handles most exposure cases; a heuristic penalty on top adds noise without signal. Future drainer work should focus on harder filtering or combining exposure awareness with other signals rather than standalone penalties.

## Removed Registry Aliases

- `runtime_efficient_v1`
  Removed from the active registry on March 9, 2026 because it was only a compatibility alias for `runtime_eff_non_exact_v1`. Use the canonical ID instead.
- `runtime_pre_pro_promotion_v1`
  Removed from the active registry on March 9, 2026 because it was a no-op duplicate of `runtime_current`. Use `runtime_current` for that behavior.

## Mistakes Not To Repeat

- Do not keep historical profiles live in the active registry after their lesson is absorbed.
- Do not treat `target/experiment-runs` as durable memory.
- Do not revive archived candidates unless there is a specific new hypothesis that cannot be tested with the active profile surface.
- Do not widen the active experiment surface faster than you can maintain the runbook and guardrail tests.
