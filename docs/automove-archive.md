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

## Mistakes Not To Repeat

- Do not keep historical profiles live in the active registry after their lesson is absorbed.
- Do not treat `target/experiment-runs` as durable memory.
- Do not revive archived candidates unless there is a specific new hypothesis that cannot be tested with the active profile surface.
- Do not widen the active experiment surface faster than you can maintain the runbook and guardrail tests.
