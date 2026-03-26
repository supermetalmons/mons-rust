# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired branch history to `docs/automove-archive.md`.

## Current State (2026-03-26)

- Shipping Pro stays `runtime_current`.
- `runtime_current` is the planner-plus-quiescence Pro runtime.
- `runtime_pro_turn_engine_v30` is the only live Pro challenger.
- `runtime_pro_turn_engine_v30` is a guarded `ProV2` engine path with deliberate opening and early-white fallbacks.
- The latest shared exact cuts improved `runtime_pro_turn_engine_v30`, but it is still not promotable because `pro-reliability` keeps stalling past a practical window.
- `runtime_pro_turn_engine_v1` stays reference-only regression history.
- Archive profiles and retired planner/quiescence lines are docs-only context.

## Live Objective

Promote `runtime_pro_turn_engine_v30` against `runtime_current` in Pro mode with fewer, cheaper iterations.

Success means:
- `guardrails` stays clean.
- `SMART_TRIAGE_SURFACE=primary_pro` passes `pro-triage`.
- `runtime-preflight` stays clean.
- `pro-reliability` produces real promotable evidence against `runtime_current`.

Default loop:
- `./scripts/run-automove-experiment.sh guardrails runtime_pro_turn_engine_v30`
- `SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage runtime_pro_turn_engine_v30 runtime_current`
- `./scripts/run-automove-experiment.sh runtime-preflight runtime_pro_turn_engine_v30`
- `./scripts/run-automove-experiment.sh pro-reliability runtime_pro_turn_engine_v30 runtime_current`

## Primary Split Family

Shared exact/search cuts on the current live wall.

Use this family when:
- a fresh `pro-reliability` sample points at the next wall clearly
- the wall is still inside exact tactical, projection, or search-side scoring work
- the split can plausibly improve `primary_pro` or reduce direct duel friction

Reject this family for a branch when:
- the split is only another wrapper retune
- the hotspot moved but the planned change still attacks the old wall
- the split has no direct story for `pro-triage` or `pro-reliability`

Current live wall:
- inside tactical projection itself, especially `exact_tactical_spirit_summary` after-window work and the remaining `exact_best_immediate_tactical_window_on_board_with_hash` / pickup-window cost
- not in planner/oracle summary construction anymore

## Secondary Split Family

Minimal new `primary_pro` fixtures or stuck-state / bounded-progress fixtures.

Use this family only when:
- a small fixture addition can reject bad branches before duel spend
- the current wall is already understood and the missing piece is earlier failure detection
- the fixture is directly tied to a repeated live failure mode

Do not use this family to avoid live Pro evidence.

## Parked

Do not spend time here unless the live objective changes:
- Fast work
- archive profiles
- retired planner/quiescence revivals
- wrapper-only knob sweeps
- hotspot-first iteration loops
- broad reporting cleanup that does not change promote/kill speed

## Hard Kill Condition

Stop the line and record the lesson when a focused split does not do at least one of these:
- improve `pro-triage`
- reduce `pro-reliability` friction
- move the live wall to a new code surface

If the direct duel wall is unclear, take a fresh live `pro-reliability` sample before starting another micro-optimization pass.
