# Automove Ideas

This is the active backlog for upcoming automove iterations.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution playbook. Keep this file lean: active hypotheses only. Move long history to `docs/automove-archive.md` and durable lessons to `docs/automove-knowledge.md`.

## Current State (2026-03-14)

- Latest promotion: Normal mode moved to a fast-derived core with bounded extra Normal spend.
- Production now has stronger Normal-vs-Fast head-to-head while staying within release speed gates.
- Next wave emphasis: Pro-first recovery, then port proven safe wins across modes.

## Template

### Idea: <short name>
- Base profile: `runtime_current`
- Target mode:
- Triage surface:
- Triage pass signal:
- Calibration gate:
- Candidate budget: 1
- Expected upside:
- CPU risk:
- Cheapest falsifier:
- Escalate only if:
- Kill if:
- Next split if rejected:
- How to test:
- Status: backlog
- Notes:

## Active Frontier

### Idea: Pro quiescence tactical-only child generation
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `opening_reply` (target), `primary_pro` (off-target guard via `pro-triage`)
- Triage pass signal: `pro-triage` reports `target_changed>=1` with `off_target_changed<=1`.
- Calibration gate: none
- Candidate budget: 1
- Expected upside: keep quiescence tactical signal while removing full `ranked_child_states()` expansion cost.
- CPU risk: medium
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=opening_reply ./scripts/run-automove-experiment.sh pro-triage <candidate>`.
- Escalate only if: `runtime-preflight` passes and `pro-fast-screen` is positive vs both normal and fast baselines.
- Kill if: no target-surface movement, or either `pro-fast-screen` lane is flat/regressed.
- Next split if rejected: capture-only tactical child generation.
- How to test: `guardrails -> pro-triage -> runtime-preflight -> pro-fast-screen -> pro-progressive -> pro-ladder`.
- Status: backlog
- Notes: this is the first strength attempt for the Pro-first recovery wave.

### Idea: Pro quiescence trigger gating (volatility/frontier)
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `opening_reply`
- Triage pass signal: `pro-triage` still moves `opening_reply` fixtures while expected quiescence call footprint shrinks.
- Calibration gate: none
- Candidate budget: 1
- Expected upside: reduce unnecessary quiescence activations on quiet leaves while preserving tactical upside.
- CPU risk: medium
- Cheapest falsifier: `guardrails`, `pro-triage`, `runtime-preflight`.
- Escalate only if: first `pro-fast-screen` beats baseline with no lane regression.
- Kill if: CPU improves but duel signal disappears, or duel signal remains but CPU does not improve.
- Next split if rejected: combine with tactical-only child generation in one bounded follow-up.
- How to test: one candidate delta through standard Pro earned path.
- Status: backlog
- Notes: distinct from tactical-only generation; this changes when quiescence runs.

### Idea: Shared reply-risk/exact-lite cache reuse
- Base profile: `runtime_current`
- Target mode: `pro`, then `normal`
- Triage surface: `cache_reuse`
- Triage pass signal: deterministic cache-reuse triage improvement (`avg_ms` down or hit-rate up) without guardrail regressions.
- Calibration gate: none
- Candidate budget: 1
- Expected upside: reclaim CPU from duplicated shortlist/exact-lite work, then convert into strength.
- CPU risk: low to medium
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=cache_reuse ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: `runtime-preflight` passes and first earned duel stage is positive for target mode.
- Kill if: measurable cache win but first duel stage remains flat.
- Next split if rejected: isolate cache-sharing to one path (reply-risk only or exact-lite only).
- How to test: cache triage first, then earned duel path in target mode.
- Status: backlog
- Notes: preferred all-modes infrastructure-backed strength direction.

### Idea: Candidate-aware opening-reply speed probe
- Base profile: workflow-only
- Target mode: `pro`
- Triage surface: blocked until probe exists
- Triage pass signal: new probe shows stable candidate-vs-baseline opening-reply latency deltas on fixed seeds.
- Calibration gate: none
- Candidate budget: 1
- Expected upside: catch opening latency regressions early for `opening_reply` ideas without misusing production-only release gates.
- CPU risk: low
- Cheapest falsifier: implement probe and show it cannot separate known retained profiles.
- Escalate only if: probe is stable enough to become a standard pre-duel diagnostic.
- Kill if: probe is noisy or does not improve promotion decisions.
- Next split if rejected: keep release speed gates only; do not add new opening speed diagnostics.
- How to test: add ignored harness test and compare `runtime_current` with one known slower/faster retained profile.
- Status: backlog
- Notes: workflow diagnostic, not a direct strength candidate.

## Workflow Improvements

### Idea: Stuck-state and bounded-progress safety fixtures
- Base profile: `runtime_current`
- Target mode: `fast`, `normal`, `pro`
- Triage surface: blocked until fixture exists
- Candidate budget: 1
- Expected upside: catch empty-selector/repeat/no-progress edge cases before promotion.
- CPU risk: low
- Cheapest falsifier: add fixtures and confirm they reject unsafe candidates immediately.
- Status: backlog
- Notes: safety reliability is a release requirement, not optional cleanup.

### Idea: Promotion-focused artifact summaries
- Base profile: workflow-only
- Target mode: workflow
- Candidate budget: 1
- Expected upside: faster decisions with less log spelunking and cleaner cleanup cycles.
- CPU risk: low
- Cheapest falsifier: add lightweight summary output and verify it does not improve decision speed/quality.
- Status: backlog
- Notes: optimize signal extraction, not logging volume.

## Recently Closed

- Normal fast-derived core branch: promoted via `runtime_normal_fast_core_budget_spend_v1` into `runtime_current` on 2026-03-14.
- Normal release-seed alignment (`normal_release_seed_gap`): completed; retained as active deterministic surface.
- Earlier long-wave config knob details are archived in `docs/automove-archive.md` (Wave 3 section).
- Durable cross-wave lessons are kept in `docs/automove-knowledge.md`.
