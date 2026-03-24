# Automove Ideas

This is the active backlog for upcoming automove iterations.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution playbook. Keep this file lean: current state, live frontier, workflow backlog, and compact recent outcomes only. Move durable lessons to `docs/automove-knowledge.md` and wave history to `docs/automove-archive.md`.

## Current State (2026-03-24)

- Production Pro in `runtime_current` still uses the promoted turn-opportunity planner from March 18, 2026.
- The large ProV2 turn-engine wave has been compressed. Durable shared work is retained in the engine/runtime code:
  - opportunity-context extraction
  - best-plan / no-plan / continuation caching
  - config-fingerprinted cache reuse
  - selector utility and followup-floor caching
  - low-budget / eligibility / resume logic
  - Pro-aware `runtime-preflight`, `pro-reliability`, and duel-progress workflow support
- `runtime_pro_turn_engine_v30` is the sole active ProV2 frontier.
- `runtime_pro_turn_engine_v1` remains only as a retained reference/baseline, not the live frontier.
- `runtime_pro_turn_engine_v30` is stronger than the earlier wrapper splits and cleared direct/faster earned stages during the wave, but it is still not promotable because the earned path is incomplete and strict direct-gate thresholds have now been restored.
- Default artifact layout is now:
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
- CPU risk: medium (earned-path runtime already improved, but progressive/larger duels still need proof)
- Cheapest falsifier: strict `pro-reliability` or the next earned duel stage stays flat after one exact live-loss fix
- Current checkpoint:
  - shared ProV2 planning/cache infrastructure is retained in the common engine path
  - `runtime-preflight` is Pro-aware and `pro-reliability` is part of the canonical workflow
  - `runtime_pro_turn_engine_v30` is the retained frontier after the compressed `v2`..`v30` wave
  - earlier wrapper splits and branch-specific clutter have been archived out of the active registry
- Current blocker: the frontier still has not completed the full earned path under the restored strict direct gate and progressive/ladder proof remains incomplete
- Recent outcome:
  - the stronger macro-head acceptance blockers (`primary_spirit_setup`, `primary_black_loss_opening_a_ply19`, `human_win_pro_a`) are now covered by direct regressions and stay green on the v30 suite
  - the practical earned-path blocker is now CPU spend in duel-scale `pro-reliability`, with hot samples concentrated in `oracle_walk_seeds` / exact turn-summary work during main head planning
- Next split: keep the single v30 frontier and cut duel-scale CPU from the remaining `oracle_walk_seeds` / exact-summary path before retrying the full earned path
- How to test:
  - `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight`
  - `pro-reliability` against `runtime_current`
  - only after the strict direct gate is green: `pro-fast-screen -> pro-progressive -> pro-ladder`
- Status: active

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

- Pro turn-engine wave compression: `runtime_pro_turn_engine_v2`..`v30` were reduced to one retained frontier plus archived lessons; see `docs/automove-archive.md`.
- Pro intent planner v2 stabilization: early gates and bounded ladder speed could be kept green in the emergency-only shape, but direct reliability remained flat and the branch did not justify live-frontier space.
- Fast tactical uplift against current Normal: repeated reply-risk, spirit-setup, opponent-mana, and scoring-only splits either failed triage, stayed flat at first duel, or hit progressive runtime cliffs; reopen only with a genuinely new code path.
- Pro turn-opportunity planner v1: promoted to production Pro on March 18, 2026; keep the rollout Pro-only because direct Fast/Normal transplants regressed Normal.
- Shared reply-risk / exact-lite cache reuse line: closed at `cache_reuse` triage.
