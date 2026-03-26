# Automove Ideas

This is the live backlog for upcoming automove iterations.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution playbook. Keep this file lean: current state, the retained frontier, the latest attempted candidate, and compact next-step ideas only. Move durable lessons to `docs/automove-knowledge.md` and branch history to `docs/automove-archive.md`.

## Current State (2026-03-26)

- Production Pro in `runtime_current` still uses the promoted turn-opportunity planner from March 18, 2026.
- `runtime_pro_turn_engine_v30` remains the retained ProV2 frontier.
- `runtime_pro_turn_engine_v1` remains reference-only history, not the live frontier.
- `runtime_pro_turn_engine_v3_shared` stays hidden experiment-only state; it recovered front-gate proof but still did not finish `pro-reliability` in a practical window.
- `runtime_pro_turn_engine_v31` through `runtime_pro_turn_engine_v52` are archived roadmap follow-through, not separate live frontiers.
- No extra Pro candidate is retained as active after the latest follow-up loop.
- The latest archived follow-up is `runtime_pro_turn_engine_v54_tactical_window_cache_v1`, built on the `v53` exact preview fast path.
- Default artifact layout is:
  - logs: `target/experiment-runs/<candidate>/`
  - workflow-only logs: `target/experiment-runs/misc/`
  - runtime-preflight stamps: `target/experiment-stamps/`

## Idea Template

### Idea: <short name>

- Base profile: `runtime_current`
- Target mode:
- Triage surface:
- Triage pass signal:
- Calibration gate:
- Expected upside:
- CPU risk:
- Cheapest falsifier:
- Current blocker:
- Next split:
- How to test:
- Status:

## Active Frontier

### Idea: Pro turn engine v30 completion

- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Triage pass signal: `runtime_pro_turn_engine_v30` keeps moving `primary_pro` fixtures while preserving `runtime-preflight` and re-earning direct `runtime_pro_turn_engine_v30` vs `runtime_current` reliability under the restored strict gate
- Calibration gate: none
- Expected upside: stronger full-turn planning and continuation reuse than shipping Pro without reopening the old CPU-heavy branch
- CPU risk: medium-high
- Cheapest falsifier: strict `pro-reliability` or the next focused exact-oracle split stays flat on `human_win_pro_a`
- Current checkpoint:
  - `runtime_pro_turn_engine_v30` still fully scores all enumerated children on the hard hotspot boards.
  - The roadmap mainline was followed far enough that it should no longer be the operator doc: the local bundle, two-stage ordering, reach cleanup, projection-profile, and `v46`..`v52` search/oracle phases all landed as bounded follow-through.
  - `runtime_pro_turn_engine_v52_spirit_preview_no_board_summary_v1` is the strongest late-roadmap technical base, but it is still not promotable.
  - `runtime_pro_turn_engine_v53_spirit_preview_window_fast_path_v1` proved the narrow preview-saturation skip, but it did not move the direct gate enough to justify retained-frontier status.
  - `runtime_pro_turn_engine_v54_tactical_window_cache_v1` is the strongest post-roadmap exact follow-up so far. It keeps the `v53` preview path and adds a candidate-only cache for exact immediate tactical windows.
- Current blocker: a clean direct `pro-reliability` run still does not finish in a practical promotion window. The returned duel wall is still the same exact-oracle surface under `discover_macro_opportunities_v2 -> oracle_walk_seeds -> build_exact_turn_tactical_projection -> exact_tactical_spirit_summary`, with the sampled top cost now spread across `exact_apply_secure_drainer_walk_in_place`, `exact_secure_specific_mana_steps_in_game_with_key_at_mut`, `ExactActorMoveMemo::payload_after_move`, the uncached remainder of `exact_best_immediate_tactical_window_on_board_with_hash`, and `exact_board_hash`.
- Recent outcome:
  - `v53` kept the front of the earned path green and validated the preview-saturation skip, but the strict direct gate still ran past the practical window and was manually stopped.
  - `v54` also kept the front of the earned path green: `guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, and `runtime-preflight` all pass.
  - Fresh front-gate numbers on `v54`: `primary_pro target_changed=15`, `off_target_changed=0`; runtime-preflight stage-1 CPU improved to `0.822`, `0.765`, and `0.785`.
  - Focused parity coverage exists for the spirit-preview helper, tactical spirit summary score-only parity, denial-only parity, full score+denial+progress parity, tactical projection parity, and immediate tactical-window parity.
  - The bounded hotspot probe shows that `v54` is useful but not decisive: on `primary_spirit_setup`, wall-clock improved from `337.10ms` on `v53` to `321.98ms` with `immediate_window_hits=21927`; on `human_win_pro_a`, it stayed effectively flat (`1300.71ms -> 1292.17ms`) even with `immediate_window_hits=35008`.
  - A follow-up `v55` fast-lookup attempt cut counted immediate-window queries further, but it did not improve the hotspot wall enough to justify keeping the code, so it was discarded.
  - A clean `pro-reliability` run for `v54` still did not complete in a practical window and was manually stopped after roughly 80 seconds with no completion summary. Until that changes, `runtime_pro_turn_engine_v30` remains the retained frontier and `v54` stays archived-only.
- Next split:
  - Stay inside the returned exact-oracle wall first, but move below the immediate-window cache layer: cut secure drainer recursion and touched-state churn inside `exact_apply_secure_drainer_walk_in_place` / `exact_secure_specific_mana_steps_in_game_with_key_at_mut`, or avoid paying that work from the tactical spirit path when the caller only needs score or denial.
  - If another bounded exact cut stays flat on `human_win_pro_a`, reopen shared search-side reuse in `ranked_child_states`, reply-risk scoring, and `move_efficiency_snapshot` rather than spending more time on preview-local window caching variants.
  - If the `v30` line still cannot earn a practical `pro-reliability` finish after 2–3 more focused splits, use the roadmap’s remaining structural fallbacks: a guarded hybrid overlay or a cheaper distilled online signal learned from `v30` decisions.
- How to test:
  - `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight`
  - `pro-reliability` against `runtime_current`
  - only after the strict direct gate is green: `pro-fast-screen -> pro-progressive -> pro-ladder`
- Status: active; retained frontier is `runtime_pro_turn_engine_v30`, latest archived follow-up is `runtime_pro_turn_engine_v54_tactical_window_cache_v1`

## Workflow Backlog

### Idea: Stuck-state and bounded-progress safety fixtures

- Base profile: `runtime_current`
- Target mode: `fast`, `normal`, `pro`
- Triage surface: blocked until fixtures exist
- Expected upside: catch empty-selector, repeat-loop, and no-progress regressions before promotion
- CPU risk: low
- Cheapest falsifier: fixtures land but do not reject unsafe candidates any earlier than the current guardrails
- Current blocker: fixture pack does not yet cover these edge cases directly
- Next split: add the smallest promotable fixture pack and wire it into guardrails or triage
- How to test: add the fixtures, then confirm unsafe branches fail before duel spend
- Status: backlog

### Idea: Promotion-time rollup summary

- Base profile: workflow-only
- Target mode: workflow
- Triage surface: none
- Expected upside: faster promote/kill decisions without opening multiple raw logs
- CPU risk: low
- Cheapest falsifier: metadata and cleanup improvements are already enough, and no operator time is saved by adding a summary layer
- Current blocker: logs are better organized now, but promotion evidence still lives across multiple command outputs
- Next split: emit one compact per-stage rollup after progressive or ladder without changing any gate behavior
- How to test: add the summary output and confirm it replaces manual log spelunking on one live candidate
- Status: backlog

## Recently Closed / Parked

- Stronger Pro roadmap follow-through: `runtime_pro_turn_engine_v31`..`runtime_pro_turn_engine_v52` are archived as one bounded follow-through wave, and the `v53`/`v54` post-roadmap exact-window follow-ups are archived as latest evidence; use this file, not `stronger_pro_automove_roadmap.md`, for live next-step decisions.
- Pro turn-engine wave compression: `runtime_pro_turn_engine_v2`..`v30` were reduced to one retained frontier plus archived lessons; see `docs/automove-archive.md`.
- Pro intent planner v2 stabilization: early gates and bounded ladder speed could be kept green in the emergency-only shape, but direct reliability remained flat and the branch did not justify live-frontier space.
- Fast tactical uplift against current Normal: repeated reply-risk, spirit-setup, opponent-mana, and scoring-only splits either failed triage, stayed flat at first duel, or hit progressive runtime cliffs; reopen only with a genuinely new code path.
- Pro turn-opportunity planner v1: promoted to production Pro on March 18, 2026; keep the rollout Pro-only because direct Fast/Normal transplants regressed Normal.
- Shared reply-risk / exact-lite cache reuse line: closed at `cache_reuse` triage.
