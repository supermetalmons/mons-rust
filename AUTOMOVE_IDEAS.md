# Automove Ideas

This is the active backlog for upcoming automove iterations.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution playbook. Keep this file lean: active hypotheses only. Move long history to `docs/automove-archive.md` and durable lessons to `docs/automove-knowledge.md`.

## Current State (2026-03-15)

- Latest promotion: Normal mode moved to a fast-derived core with bounded extra Normal spend.
- Pro recovery status: the opening-reply transplant wave and the primary quiescence-shaping wave are now closed.
- Current Pro blocker: bounded `pro-ladder` repeatedly fails confirmation-vs-fast by a tiny margin (`-0.1111` vs tolerance `-0.10`) across both retained and split candidates.
- Workflow update kept: `./scripts/run-automove-experiment.sh pro-opening-speed-probe <candidate> [baseline]` is available for Pro `opening_reply` ideas.

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

### Idea: Pro confirmation reply-policy rebalance
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro` (off-target guard: `opening_reply`)
- Triage pass signal: `primary_pro target_changed>0` with `opening_reply off_target_changed<=1`
- Calibration gate: none
- Candidate budget: 1
- Expected upside: fix the `pro-ladder` confirmation-vs-fast near-miss without reopening broad primary search churn.
- CPU risk: medium
- Cheapest falsifier: fail `pro-triage` on `primary_pro`, or fail `pro-fast-screen` vs fast.
- Escalate only if: `guardrails -> pro-triage -> runtime-preflight -> pro-fast-screen -> bounded pro-progressive` is clean.
- Kill if: bounded `pro-ladder` (`speed_positions=12`, `primary/confirm=3x3`, `max_plies=64`) is flat/negative on confirmation lanes.
- Next split if rejected: split one confirmation-context policy family at a time (opening-book reply policy or confirmation reply-risk shape), not quiescence budgets.
- How to test: `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight -> pro-fast-screen -> pro-progressive -> pro-ladder`.
- Status: backlog
- Notes:
  - Keep minimized ladder as directional-only.
  - First keep/kill ladder should be the bounded `3x3 @64` form.

### Idea: Pro opening-reply quality with explicit speed envelope
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `opening_reply` (off-target guard: `primary_pro`)
- Triage pass signal: `opening_reply target_changed>0` with `primary_pro off_target_changed<=1`
- Calibration gate: none
- Candidate budget: 1
- Expected upside: recover opening-reply quality if the confirmation-rebalance line stalls.
- CPU risk: low-to-medium
- Cheapest falsifier: fail `pro-opening-speed-probe` envelope or fail first duel.
- Escalate only if: `pro-opening-speed-probe` + `pro-triage` both pass.
- Kill if: first duel (`pro-fast-screen`) is flat/negative.
- Next split if rejected: close opening-reply-only lane and return to primary confirmation hypotheses.
- How to test: `guardrails -> pro-opening-speed-probe -> SMART_TRIAGE_SURFACE=opening_reply pro-triage -> runtime-preflight -> pro-fast-screen`.
- Status: backlog
- Notes:
  - Previous opening-reply fast-policy family is closed; do not revive old IDs.

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

### Idea: Promotion-focused artifact summaries
- Base profile: workflow-only
- Target mode: workflow
- Candidate budget: 1
- Expected upside: faster promote/kill decisions with less log spelunking.
- CPU risk: low
- Cheapest falsifier: add lightweight summary output and verify no decision-speed improvement.
- Status: backlog

## Recently Closed (Compact)

- Pro opening-reply fast-policy family (`runtime_pro_opening_reply_fast_policy_v1..v8`): closed after repeated ladder primary-vs-normal failures.
- Pro primary fast-policy family (`runtime_pro_primary_fast_policy_v1..v5`): closed after the same ladder primary-vs-normal failure pattern.
- Pro primary quiescence shaping family (`runtime_pro_quiescence_v3..v14`): closed; bounded ladder kept failing confirmation lanes (especially vs fast).
- Retained Pro quiescence anchor (`runtime_pro_quiescence_v2`) remains useful as a reference but is not promotable under bounded ladder settings.
- Shared reply-risk/exact-lite cache reuse line: closed at `cache_reuse` triage.

Detailed wave history is archived in `docs/automove-archive.md`. Durable guidance is tracked in `docs/automove-knowledge.md`.
