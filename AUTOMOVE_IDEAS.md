# Automove Ideas

This is the live decision board for automove work.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the runbook. Keep this file short. Move durable lessons to `docs/automove-knowledge.md` and retired branch history to `docs/automove-archive.md`.

## Current State (2026-03-27)

- Shipping Pro stays `runtime_current`.
- `runtime_current` is the planner-plus-quiescence Pro runtime.
- `runtime_pro_turn_engine_v30` is the only live Pro challenger.
- `runtime_pro_turn_engine_v30` is a guarded `ProV2` engine path with deliberate opening and early-white fallbacks.
- The latest shared exact cuts improved `runtime_pro_turn_engine_v30`, but it is still not promotable because `pro-reliability` keeps stalling past a practical window.
- The latest local after-window cache-axis-sharing cut only shaved the hotspot slightly; it did not materially reduce tactical projection call volume or clear the promotion gate.
- The latest tactical cache-sharing cuts did move the live wall again: `human_win_pro_a` dropped from about `1425ms` to about `1305ms`, but `pro-reliability` still stalled past `2:17`.
- The latest bounded carrier-window cut is worth keeping: it collapsed `human_win_pro_a` payload work from about `32.2m` to about `6.3m` and materially sped up the probe, but the real duel gate still stalled past `2:17`.
- The latest retained exact cut is also worth keeping: low-budget drainer pickup fast paths removed most uncached pickup BFS from the hotspot, score-floor after-window filtering trimmed some off-target tactical queries, and incremental board-hash reuse across spirit previews moved `human_win_pro_a` from about `570ms` to about `520ms`, but `pro-reliability` was still live past `2:20` on 2026-03-27.
- The latest retained exact cut is worth keeping too: a zero-move after-window tactical fast path updates touched mana-pool counts directly instead of re-querying the immediate-window helper, which moved `human_win_pro_a` again from about `520ms` to about `433ms` and dropped immediate-window / after-window query volume from about `428999` / `427069` to about `168068` / `166138`, but `pro-reliability` still had to be killed at `3:37` on 2026-03-27.
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
- cross-flag cache reuse at the tactical spirit, immediate-window, pickup-window, and projection layers is worth keeping, but it still does not clear the duel gate by itself
- bounded carrier reach checks are worth keeping on budgeted immediate-window callers, but they still leave the live wall inside after-window tactical queries and drainer pickup work
- low-budget pickup fast paths and incremental exact board-hash reuse are worth keeping, but the main `human_win_pro_a` wall still emits about `428999` immediate-window queries and about `427069` after-window calls
- zero-move after-window tactical queries were a real wall and are now worth bypassing with exact touched-item updates, but the live duel wall still sits in the remaining budgeted after-window fanout
- future cuts need to reduce the number of after-window tactical queries or spirit-preview fanout itself, not just make hashing or pickup subroutines cheaper
- the next live target is likely the remaining budget-1 / budget-2 after-window tactical query volume, not more budget-0 or hashing cleanup
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
