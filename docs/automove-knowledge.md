# Automove Knowledge

This document keeps only durable lessons that should outlast the current session.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the operator loop and `AUTOMOVE_IDEAS.md` for the next live split.

## Stable Runtime Truths

- Shipping Pro is `runtime_current`, not the retained challenger profiles.
- `runtime_pro_turn_engine_v30` is the only live Pro challenger. Treat it as a guarded `ProV2` path with deliberate opening and early-white fallbacks, not a raw always-on engine.
- `runtime_pro_turn_engine_v1` is reference-only regression history.
- Full-turn planning can beat per-input expansion, but only when it survives the real ranked-root surface and direct Pro-vs-Pro duel evidence.
- Opportunity-context extraction is worth keeping separate from raw input search. It is useful shared structure for planner seeds and selector guards.
- Continuation and no-plan reuse matter, but cache keys must include a config fingerprint.
- Drainer safety, root reply-risk guards, and efficiency tie-breaks still earn their keep because they kill fake-good roots cheaply.
- Opening-specific latency still matters. A candidate that stalls on the first real black reply is not promotable.
- Hybrid fallbacks must respect retained opening and eligibility guards before they call expensive plan probes; otherwise the fallback machinery itself becomes the regression.
- Wrapper-only tuning saturates quickly. Once the obvious knobs are already in place, the next gain usually has to come from shared engine/search code.
- Config knob space is effectively exhausted. Future gains need new code, not more `SmartSearchConfig` permutations.
- Production wasm still needs single-shot, predictable search. Deferred or post-return work is not release-safe.

## Durable Workflow Rules

- Keep the active frontier small: one live idea, one candidate, one canonical path.
- For Pro work, compare directly against `runtime_current`, not the script's compatibility default baseline.
- The canonical Pro path is `guardrails -> pro-triage(primary_pro) -> runtime-preflight -> pro-reliability`.
- `opening_reply` is a narrow fallback-order and opening-regression surface, not the default Pro surface.
- `pro-triage` only counts as a pass when the target surface changes and off-target churn stays at `<= 1`.
- `runtime-preflight` is the required stamp before duel stages unless the run is intentionally diagnostic.
- `pro-reliability` is the first real promotion gate. `pro-fast-screen`, `pro-progressive`, and `pro-ladder` come later.
- If a surface is already calibrated, do not move the CPU gate back in front of triage.
- `audit-screen` and `pro-audit-screen` are reject diagnostics, not promotion evidence.
- Prefer a fresh live `pro-reliability` sample over the hotspot probe when the wall is unclear or has moved.
- Use the hotspot probe only after a real duel stall, and only to narrow the next code surface.
- Compress the lesson immediately when a run matters. Logs, stamps, and process samples are disposable evidence, not memory.
- Keep ignored harness test names unique; `cargo test` substring filters can run the wrong stage.

## Engine And Search Lessons Worth Keeping

- If a caller only consumes spirit-assisted exact fields plus safe-progress fields, do not request the exact tactical `SCORE_WINDOW` payload there.
- If a caller only needs the active-turn score window, route it through a tactical score-window projection instead of a full turn summary.
- Cheap lower-bound drainer-attack screens are worth keeping when they can reject impossible child vulnerability probes before exact reach work.
- On one-shot spirit-preview paths, local work can beat repeated filtered global cache queries.
- Mutate/undo preview application is often better than clone-per-preview on exact tactical spirit paths.
- After a cut moves the wall, follow the moved wall immediately instead of iterating again on the old surface.
- When a new helper removes one hotspot, verify that the helper itself did not become the new wall before going deeper.
- If a retained optimization shifts cost from summary construction to direct point queries on the same caller surface, follow the caller that still emits those queries.
- Low-budget exact fast paths are worth keeping when they can prove a drainer pickup or immediate window is impossible without entering the full BFS / pickup-window path.

## Mistakes Not To Repeat

- Do not keep historical profiles live after their lesson is absorbed.
- Do not treat `target/experiment-runs` or `target/experiment-stamps` as durable memory.
- Do not reopen a branch just because it looked good in a bounded screen; reopen only with a new hypothesis.
- Do not spend ladder budget on branches that are still flat in the first real duel stage.
- Do not trust pooled or sampled losses as selector evidence when direct reliability tracing shows `disagreements=0`.
- Do not let Pro-specific selector flow bypass opening-book fallback ordering.
- Do not inject planner roots globally. Crisis-only gating is safer, and even then it must prove direct value against `runtime_current`.
- Do not keep sprawling wrapper split families alive once one retained frontier is clearly dominant.
- Do not use fixture additions or hotspot work as an excuse to avoid live Pro evidence.

## Current Durable Direction

- The current problem is not proving that Pro full-turn planning can ever be stronger. The retained code already shows that it can be.
- The current problem is finishing the earned path cleanly with one retained challenger under strict gates.
- The latest shared exact cuts moved the wall from planner/oracle summary construction into tactical projection itself.
- `runtime_pro_turn_engine_v30` is still not promotable after the current cuts because `pro-reliability` continues to stall in a practical window.
- Next code should target `exact_tactical_spirit_summary` after-window followups and the remaining `exact_best_immediate_tactical_window_on_board_with_hash` / pickup-window work, not planner-oracle wrapper code.
- Keep Fast work parked until there is a genuinely new code path. Minor search-order retunes and scoring-only tweaks are already saturated.
