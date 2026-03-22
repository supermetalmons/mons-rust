# Automove Ideas

This is the active backlog for upcoming automove iterations.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution playbook. Keep this file lean: current state, live frontier, workflow backlog, and compact recent outcomes only. Move durable lessons to `docs/automove-knowledge.md` and wave history to `docs/automove-archive.md`.

## Current State (2026-03-22)

- Production Pro in `runtime_current` still uses the turn-opportunity planner promotion from March 18, 2026.
- Release speed gates after that promotion stayed green (`opening black reply`: fast `3.87ms`, normal `3.99ms`, pro `4.06ms`; `mixed`: fast `5.89ms`, normal `31.82ms`, pro `121.68ms`).
- `runtime_pro_turn_engine_v1` is the only live promotion candidate.
- `runtime_pro_intent_planner_v2` and the recent Fast uplift wave are no longer part of the live frontier; their lessons were compressed into `docs/automove-knowledge.md` and `docs/automove-archive.md`.
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

### Idea: Pro turn engine v1 promotion

- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro` (off-target guard: `opening_reply`)
- Triage pass signal: engine plans expand on primary Pro fixtures and direct `runtime_pro_turn_engine_v1` vs `runtime_current` reliability turns positive before full earned-path spend
- Calibration gate: none
- Expected upside: stronger full-turn continuation on spirit/setup and tactical conversion lines without regressing current Pro selectors
- CPU risk: medium-to-high (seed previews, reply beams, continuation replay)
- Cheapest falsifier: direct Pro-vs-current disagreement signal stays flat after another focused engine-coverage split
- Current checkpoint:
  - direct activity probe now adjudicates non-terminal samples instead of panicking at `total_games=0`
  - continuation replay is guarded again; reduced-budget reply-risk reranks no longer pollute the real cache
  - engine seed generation now covers spirit-impact previews, exact-preview drainer progress, and oracle walk seeds
  - regression coverage exists for immediate-score usage, spirit/setup selection, cached continuation replay, and reply-risk tiebreak protection
  - fixture progress improved on `primary_spirit_setup`, but sampled direct Pro-vs-current runs stayed neutral (`win_rate=0.5000`, `disagreements=0`)
- Current blocker: the candidate is structurally better, but it is not yet generating beneficial selector disagreements against `runtime_current`
- Next split: add richer direct disagreement tracing and target the remaining `NoPlan` / selected-mismatch fixtures before touching broader selector gates
- How to test:
  - `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight`
  - `smart_automove_pro_reliability_loss_probe` against `runtime_current`
  - only after direct reliability turns positive: `pro-fast-screen -> pro-progressive -> pro-ladder`
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

- Pro intent planner v2 stabilization: early gates and bounded ladder speed could be kept green in the emergency-only shape, but direct reliability remained flat and the branch did not justify live-frontier space.
- Fast tactical uplift against current Normal: repeated reply-risk, spirit-setup, opponent-mana, and scoring-only splits either failed triage, stayed flat at first duel, or hit progressive runtime cliffs; reopen only with a genuinely new code path.
- Pro turn-opportunity planner v1: promoted to production Pro on March 18, 2026; keep the rollout Pro-only because direct Fast/Normal transplants regressed Normal.
- Shared reply-risk / exact-lite cache reuse line: closed at `cache_reuse` triage.
