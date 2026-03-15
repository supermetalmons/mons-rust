# Automove Ideas

This is the active backlog for upcoming automove iterations.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution playbook. Keep this file lean: active hypotheses only. Move long history to `docs/automove-archive.md` and durable lessons to `docs/automove-knowledge.md`.

## Current State (2026-03-15)

- Latest promotion: Normal mode moved to a fast-derived core with bounded extra Normal spend.
- Pro recovery status: the opening-reply transplant wave and the primary quiescence-shaping wave are now closed.
- Latest Pro promotion: primary-context tactical quiescence narrowing landed in production (`quiescence_node_budget=120`, tactical-only quiescence children, enum limit `12`).
- Promotion validation snapshot:
  - medium bounded `pro-progressive` (`2->4`, repeats `1`, max plies `64`): vs normal `delta=+0.1944` / vs fast `delta=+0.4444`
  - bounded `pro-ladder` (`speed=12`, `primary=3x3@64`, `confirm=3x3@64`): pass; confirmation `vs_fast=-0.0556` within `-0.10` tolerance
  - release speed gates: pass (`mixed median ms fast=5.95, normal=32.00, pro=180.32`)
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
- Status: closed (CPU/strength tradeoff unresolved in this family)
  - Notes:
    - Keep minimized ladder as directional-only.
    - First keep/kill ladder should be the bounded `3x3 @64` form.
    - 2026-03-15 run:
    - `runtime_pro_confirmation_reply_policy_v1`: first revision failed `pro-triage` (`target_changed=0`), second revision passed triage (`target_changed=1`, `off_target_changed=0`), passed fast-screen/progressive, then failed bounded ladder primary gate (`pro primary vs normal delta=0.0556 < 0.0800`).
    - `runtime_pro_confirmation_reply_policy_v2` (stronger primary reply-risk shape): passed triage (`target_changed=1`, `off_target_changed=0`), passed fast-screen/progressive, then failed bounded ladder primary gate harder (`delta=0.0370 < 0.0800`).
    - `runtime_pro_confirmation_reply_policy_v1` follow-up (primary movement `target_changed=3`, plus primary selective-extension disable) failed earlier at first duel (`pro-fast-screen vs normal delta=-0.1250`), so this branch over-corrects against the normal baseline.
    - `runtime_pro_confirmation_reply_policy_v3` (same reply-policy shape plus `+10%` primary node budget): passed `pro-triage`, `runtime-preflight`, `pro-fast-screen`, and bounded `pro-progressive`, but bounded `pro-ladder` exceeded practical runtime envelope (>20m, exact-heavy hot path) and was killed on CPU-fit.
    - `runtime_pro_confirmation_reply_policy_v4` (baseline two-pass kept, no node bump): passed `pro-triage` and `runtime-preflight`, fast-screen was `+0.250` vs normal and `0.000` vs fast, then bounded progressive still exceeded practical runtime for this loop and was killed on runtime-fit risk.
    - `runtime_pro_confirmation_reply_policy_v5` (v1-style primary plus clean reply-risk preference): passed `pro-triage`, `runtime-preflight`, fast-screen (`+0.250` vs normal, `+0.125` vs fast), and ultra-bounded progressive (`+0.250` vs normal, `+0.1667` vs fast), but failed minimum ladder CPU gate (`ratio 10.470 > 10.000`) and was killed.
    - `runtime_pro_confirmation_reply_policy_v6` (v1-style primary plus tiny primary exact-lite `1/1`): kept first-duel strength (`+0.250` vs normal, `+0.125` vs fast) but failed minimum ladder CPU gate harder (`ratio 10.527 > 10.000`) and was killed.
    - `runtime_pro_confirmation_reply_policy_v7` (pure v1 baseline calibration): failed minimum ladder CPU gate (`ratio 10.464 > 10.000`), confirming the core v1 reply-policy family is currently over CPU cap.
    - `runtime_pro_confirmation_reply_policy_v8` (trimmed v1 reply-risk workload): still failed minimum ladder CPU gate (`ratio 10.438 > 10.000`), so this branch needs a larger CPU reduction.
    - `runtime_pro_confirmation_reply_policy_v9` (stronger CPU cuts: `event ordering off`, `shortlist=5`, `reply_limit=12`, `node_share=900`): cleared CPU gate but failed primary ladder badly (`pro primary vs normal delta=-0.1250`), so strength collapsed.
    - `runtime_pro_confirmation_reply_policy_v10` (v9 plus primary event-ordering restored): failed minimum ladder CPU gate (`ratio 10.589 > 10.000`), so event-ordering restore is too expensive in this branch.
    - `runtime_pro_confirmation_reply_policy_v11` (event-ordering off with medium reply-risk workload): still failed minimum ladder CPU gate (`ratio 10.454 > 10.000`), so medium reply-risk remains too expensive even without event-ordering.
    - `runtime_pro_confirmation_reply_policy_v12` (event-ordering restored with fast-like reply-risk limits): still failed minimum ladder CPU gate (`ratio 10.491 > 10.000`).
    - Current live split: none in this family; reply-policy-only branch is closed for now because we can either satisfy CPU or primary strength, but not both.
  - Current takeaway: this reply-policy-only line no longer stalls at confirmation; it now stalls at primary-vs-normal lift. Next split should add primary strength signal, not more confirmation-only tuning.

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
- Status: closed (no opening surface movement)
- Notes:
  - Previous opening-reply fast-policy family is closed; do not revive old IDs.
  - 2026-03-15 probe candidate `runtime_pro_opening_reply_quality_v1` passed guardrails and opening speed probe but failed `pro-triage` on `opening_reply` (`target_changed=0`, `off_target_changed=0`), so this simplified opening-only shape does not move the target surface.
  - 2026-03-15 probe candidate `runtime_pro_opening_reply_quality_v2` (fast-like scoring/reply-risk opening-only shape) again failed `pro-triage` on `opening_reply` with `target_changed=0`, `off_target_changed=0`.
  - 2026-03-15 probe candidate `runtime_pro_opening_reply_quality_v3` (opening-only supermana-progress-biased scoring) still failed `pro-triage` on `opening_reply` with `target_changed=0`, `off_target_changed=0`.
  - Conclusion: opening-only scoring/reply-risk retunes are not moving the opening surface; this lane is closed for now.

### Idea: Pro primary tactical scoring rebalance
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro` (off-target guard: `opening_reply`)
- Triage pass signal: `primary_pro target_changed>0` with `opening_reply off_target_changed<=1`
- Calibration gate: none
- Candidate budget: 1
- Expected upside: recover primary-vs-normal ladder lift with CPU-neutral scoring changes instead of heavier reply-policy churn.
- CPU risk: low
- Cheapest falsifier: fail `pro-triage` on `primary_pro`, or fail `pro-fast-screen` vs normal.
- Escalate only if: `guardrails -> pro-triage -> runtime-preflight -> pro-fast-screen` is positive.
- Kill if: first duel is flat/negative or minimum ladder fails CPU hard cap.
- Next split if rejected: move to one targeted selective-extension/quiescence gate in primary context only (no opening-side edits).
- How to test: `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight -> pro-fast-screen -> bounded pro-progressive/pro-ladder`.
- Status: closed (promoted to production)
- Notes:
  - `runtime_pro_primary_signal_v1` passed `pro-triage`, `runtime-preflight`, and first duel (`+0.250` vs normal, `+0.250` vs fast), but failed bounded progressive vs normal (`delta=-0.0833`) and was killed.
  - `runtime_pro_primary_signal_v2` (primary-only tactical quiescence narrowing) passed bounded earned path and was transplanted to production Pro primary runtime config.
  - Promotion validation:
    - medium bounded `pro-progressive`: vs normal `delta=+0.1944`, vs fast `delta=+0.4444`
    - bounded `pro-ladder`: pass; confirmation `vs_normal=0.0000`, `vs_fast=-0.0556`, tolerance `-0.10`
    - release speed gates: pass

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
