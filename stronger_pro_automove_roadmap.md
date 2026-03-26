# Stronger, promotable Pro automove roadmap for `mons-rust`

Status on 2026-03-26: historical pointer only.

This document is no longer the live operator playbook.

Use these files instead:

- `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the execution workflow
- `AUTOMOVE_IDEAS.md` for the live backlog and next split
- `docs/automove-archive.md` for compressed branch history
- `docs/automove-knowledge.md` for durable lessons

## What happened

The roadmap was followed far enough that its mainline phases already exist in code and backlog history:

- `v31`: instrumentation and local child-bundle reuse
- `v32`: two-stage child ordering / shortlist control
- `v33`: reach and payload cleanup
- `v37`, `v40`, `v44`, `v45`: planner / projection / tactical-window follow-through
- `v46`..`v52`: search-side and exact-oracle follow-ups
- `v53` and `v54`: post-roadmap exact follow-ups on top of `v52`

## Current decisions

- `runtime_current` remains the shipping runtime.
- `runtime_pro_turn_engine_v30` remains the retained Pro frontier until a newer candidate earns the full path.
- `runtime_pro_turn_engine_v3_shared` stays hidden experiment-only state.
- `runtime_pro_turn_engine_v31` through `runtime_pro_turn_engine_v52` are archived follow-through IDs, not active frontiers.
- `runtime_pro_turn_engine_v52_spirit_preview_no_board_summary_v1` is the strongest late-roadmap technical base.
- `runtime_pro_turn_engine_v54_tactical_window_cache_v1` is the strongest post-roadmap exact follow-up so far, but it still did not finish `pro-reliability` in a practical window.
- no extra Pro candidate is retained as active; `v53` and `v54` are archived follow-up evidence.

## Remaining ideas

Any still-untried residual ideas from the original roadmap now live under `Pro turn engine v30 completion` in `AUTOMOVE_IDEAS.md`:

- one more exact-oracle cut if it attacks the returned secure-recursion / payload / board-hash wall directly
- a return to shared search-side reuse if the exact cut stays flat on `human_win_pro_a`
- structural fallback only if the retained `v30` line still cannot finish `pro-reliability` after a few more focused splits
