# Automove Ideas

This is the live backlog for upcoming automove iterations.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution playbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired branch history to `docs/automove-archive.md`.

## Current State (2026-03-26)

- Shipping runtime stays `runtime_current`.
- `runtime_pro_turn_engine_v30` is the only retained Pro turn-engine frontier.
- `runtime_pro_turn_engine_v1` stays as reference-only regression history.
- Archived branches from `runtime_pro_turn_engine_v31` onward, the old planner lines, and the quiescence line are docs-only context now.
- The executable experiment surface is intentionally small: current runtime, calibration anchors, curated references, and the two retained Pro references.

## Active Frontier

### Pro v30 completion

- Base profile: `runtime_current`
- Candidate profile: `runtime_pro_turn_engine_v30`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Expected upside: stronger full-turn planning and continuation reuse than shipping Pro without reopening the dead archive branches
- Current blocker: clean direct `pro-reliability` still does not finish in a practical window
- Hotspot shape: the wall is still inside the exact tactical / projection path, not in wrapper-only glue
- Next split:
  1. Cut deeper into the remaining exact wall around `payload_after_move`, immediate tactical-window work, and secure-mana recursion.
  2. Add the smallest useful stuck-state / bounded-progress fixture pack so bad candidates die before duel spend.
  3. If `v30` still cannot finish direct reliability after a few focused splits, stop grinding micro-optimizations and switch to a structural fallback.
- How to test:
  - `./scripts/run-automove-experiment.sh guardrails runtime_pro_turn_engine_v30`
  - `SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage runtime_pro_turn_engine_v30 runtime_current`
  - `./scripts/run-automove-experiment.sh runtime-preflight runtime_pro_turn_engine_v30 runtime_current`
  - `./scripts/run-automove-experiment.sh pro-reliability runtime_pro_turn_engine_v30 runtime_current`
- Status: active

## Backlog

### Stuck-state and bounded-progress fixtures

- Goal: reject empty-selector, repeat-loop, and no-progress failures before duel spend
- Cost: low
- Why it matters: the workflow is now cleaner, so the next useful improvement is earlier failure detection
- Status: backlog

### Promotion rollup summary

- Goal: emit one compact per-stage summary instead of forcing manual log spelunking
- Cost: low
- Why it matters: cleaner operator evidence and faster promote/kill decisions
- Status: backlog

## Archive Pointer

Detailed branch history for `runtime_pro_turn_engine_v31` and later, the planner experiments, the quiescence line, and earlier exhaustions lives in `docs/automove-archive.md`.
