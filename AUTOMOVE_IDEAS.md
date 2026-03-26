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
- The latest retained exact cut is also worth keeping: a budget-1 after-window tactical fast path updates touched-neighborhood one-move contributions instead of re-running the immediate-window helper, which dropped `human_win_pro_a` immediate-window / after-window query volume again from about `168068` / `166138` to about `41740` / `39810` and moved the hotspot from about `433ms` to about `413ms`, but `pro-reliability` still had to be killed at `2:35` on 2026-03-27.
- The latest local budget-2 after-window summary experiment was not worth keeping: it collapsed `human_win_pro_a` immediate-window / after-window query volume again from about `41740` / `39810` to about `7590` / `5660`, but the hotspot itself regressed from about `420ms` to about `452ms` and `pro-reliability` still had to be killed at `2:35` on 2026-03-27.
- The latest shared actor-move-memo experiment was not worth keeping either: sharing payload transition memos across immediate-window and budget-1 summary searches trimmed `human_win_pro_a` payload calls only slightly and cut pickup-window calls from about `2910` to about `2218`, but the hotspot regressed from about `420ms` to about `433ms` and `pro-reliability` still had to be killed at `2:34` on 2026-03-27.
- The latest carrier-specific move-memo plus small-budget carrier fast-path experiment was not worth keeping either: replacing the generic actor memo with a carrier-only memo and a local `0..=3` frontier path left `human_win_pro_a` effectively flat at about `411.32ms` versus the retained `411.56ms`, with `payload_calls` still about `5083818`, so carrier memo/init overhead is not the remaining wall.
- The latest retained exact cut is worth keeping too: a bounded reverse carrier-distance map, built only after a board/budget pair proves it has multiple distinct carrier queries, moved `human_win_pro_a` from the retained `411.56ms` down to about `404.74ms` and then `398.73ms`, with `payload_calls` down to about `4922742`, but `pro-reliability` still had to be killed at about `2:45` on 2026-03-27.
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
- budget-1 after-window tactical queries were also worth bypassing locally, but once they drop out the wall-clock gain is modest and the remaining wall looks deeper than simple one-move query volume
- budget-2 after-window tactical query volume is no longer the right target by itself; collapsing it with a local summary moved counters but not wall-clock
- board-scoped payload-transition memo reuse is also not enough by itself once the obvious after-window fanout is gone
- carrier-specific memo shrink plus small-budget carrier search is also not enough by itself; leaving `payload_calls` flat means the remaining wall is deeper than generic carrier BFS setup cost
- board-scoped bounded reverse carrier maps can help, but only when built lazily after repeated queries on the same board/budget; building them unconditionally on the first bounded query is too broad and regresses wall-clock
- future cuts need to reduce the number of after-window tactical queries or spirit-preview fanout itself, not just make hashing or pickup subroutines cheaper
- the next live target is likely the remaining payload reachability algorithm itself, not more memo reuse or budget-count collapse alone
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
