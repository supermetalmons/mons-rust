# Automove Ideas

This is the active backlog for upcoming automove iterations.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution playbook. Keep this file lean: active hypotheses only. Move long history to `docs/automove-archive.md` and durable lessons to `docs/automove-knowledge.md`.

## Current State (2026-03-18)

- Latest production promotion: Pro now uses the turn-opportunity planner path in `runtime_current` (full-turn abstract opportunity planning + bounded 2-turn response layer + hash-guarded continuation replay).
- Production Pro validation snapshot (`runtime_current` vs `runtime_release_safe_pre_exact`, bounded `pro-ladder`):
  - speed gate ratio: `5.889` (pass under cap `10.0`)
  - primary summary: `vs_normal delta=+0.1667`, `vs_fast delta=+0.4444`
  - confirmation summary: `vs_normal delta=0.0000`, `vs_fast delta=+0.2500` (tolerance `-0.10`)
  - pool summary: candidate `vs_normal=0.0000`, baseline `-0.3000`; candidate `vs_fast=+0.2000`, baseline `0.0000`
- Release speed gates after promotion: pass (`opening black reply`: fast `3.87ms`, normal `3.99ms`, pro `4.06ms`; `mixed`: fast `5.89ms`, normal `31.82ms`, pro `121.68ms`).
- Fast/Normal planner port status: still not promotable; repeated mixed-ladder attempts regressed Normal (typical `delta=-0.0556` lane), so planner rollout remains Pro-only.
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

### Idea: Pro turn-opportunity planner v1
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro` (off-target guard: `opening_reply`)
- Triage pass signal: strong bounded `pro-fast-screen` / `pro-progressive` and bounded `pro-ladder` pass with release speed gates green
- Calibration gate: none
- Candidate budget: 1
- Expected upside: replace per-input tree behavior with stronger full-turn tactical conversion
- CPU risk: medium-to-high (bounded by planner beam/node caps and strict Pro-only activation)
- Cheapest falsifier: fail bounded `pro-fast-screen` or exceed bounded `pro-ladder` speed ratio cap
- Escalate only if: `guardrails -> pro-triage -> runtime-preflight -> pro-fast-screen -> pro-progressive` stays positive
- Kill if: bounded ladder primary/confirm misses or release speed gates regress
- Next split if rejected: tighten planner route activation and fallback acceptance thresholds before adding new route families
- How to test: `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight -> pro-fast-screen -> pro-progressive -> pro-ladder`, then release speed gates
- Status: closed (promoted to production Pro in `runtime_current`)
- Notes:
  - Candidate profile `runtime_pro_turn_planner_v1` cleared bounded Pro ladder and produced stronger pool deltas than baseline.
  - Production mapping was then transplanted into Pro runtime config (`enable_turn_opportunity_planner=true`, bounded branch/node budget lift, deterministic tie-break + event-ordering bonus).
  - Opening-book behavior remains unchanged; planner remains gated to non-opening Pro runtime context.
  - Fast/Normal transplant attempts with the same planner family failed mixed-ladder non-regression; rollout stays Pro-only.

### Idea: Pro intent planner v2 stabilization
- Base profile: `runtime_release_safe_pre_exact`
- Target mode: `pro`
- Triage surface: `primary_pro` (off-target guard: `opening_reply`)
- Triage pass signal: `primary_pro target_changed>0` with `opening_reply off_target_changed<=1`
- Calibration gate: none
- Candidate budget: 1
- Expected upside: keep intent-planner tactical signal while preserving pro fast-screen non-regression
- CPU risk: high (progressive/ladder runtime fit remains the main risk)
- Cheapest falsifier: fail `pro-fast-screen` vs fast or lose `primary_pro` movement
- Escalate only if: `guardrails -> pro-triage -> runtime-preflight -> pro-fast-screen` stays green and runtime remains practical
- Kill if: progressive/ladder runtime cliff persists or first duel regresses
- Next split if rejected: selectively re-enable injected roots only in explicit emergency states; keep `limit=0` baseline as control
- How to test: `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight -> pro-fast-screen -> bounded pro-progressive/ladder`, then full gates only if runtime-fit is stable
- Status: in_progress
- Notes:
  - 2026-03-19: active injected-root settings (`turn_planner_intent_root_injection_limit>0`) repeatedly failed first duel vs fast (`delta=-0.1250` to `-0.2500`).
  - 2026-03-19: `runtime_pro_turn_planner_v1` control passed fast-screen vs fast (`delta=+0.1250`), isolating the regression to active injection behavior.
  - 2026-03-19: current candidate shape with intent flag on and injection effectively disabled (`limit=0`) restored first-duel signal while keeping triage movement:
    - `pro-triage primary_pro`: `target_changed=1`, `off_target_changed=0`
    - `pro-fast-screen`: vs normal `delta=0.0000`, vs fast `delta=+0.1250`
  - Bounded directional progressive probes (`initial=1`, `max=1`, `repeats=1`, `max_plies=40`) were positive (`vs normal +0.3333`, `vs fast +0.5000`) but default progressive/ladder runtime remains expensive and unresolved.
  - 2026-03-19: new split implemented emergency-only injection instead of global injection:
    - profile now uses `limit=1`, `max_heuristic_gap=200`, `turn_planner_intent_root_emergency_only=true`
    - injected roots in emergency mode now require crisis-resolving tactical content (drainer kill/safety recover/immediate score conversion) and reject vulnerable handoff lines.
  - Current gate checks on this split:
    - `guardrails`: pass
    - `pro-triage primary_pro`: pass (`target_changed=1`, `off_target_changed=0`)
    - `runtime-preflight`: pass
    - direct full `pro-fast-screen vs fast`: pass (`delta=+0.1250`, `confidence=0.637`)
    - bounded `pro-fast-screen vs normal` (`1x1`) stayed non-negative (`delta=0.0000`)
    - bounded progressive (`1x1 @56`) stayed positive (`vs normal +0.3333`, `vs fast +0.5000`)
    - bounded ladder speed gate remained under cap (`ratio=8.589`)
  - Remaining blocker: still need stable full-capture `pro-fast-screen vs normal` and ladder summaries in this environment before promotion decision.
  - 2026-03-19 rerun:
    - revalidated `guardrails -> pro-triage -> runtime-preflight` (all pass; `primary_pro target_changed=1`, `off_target_changed=0`)
    - full `pro-fast-screen vs fast` recheck still passed (`delta=+0.1250`)
    - bounded progressive recheck remained positive (`vs normal +0.3333`, `vs fast +0.5000`)
    - bounded ladder speed gate remained under cap (`ratio=8.650`)
    - full `pro-fast-screen vs normal` and full ladder summary capture are still flaky in this environment; keep using direct lane runs + bounded summaries for signal until capture is stable.
  - Reliability probe update (`smart_automove_pro_reliability_loss_probe`, candidate=`runtime_pro_intent_planner_v2`, baseline=`runtime_release_safe_pre_exact`, `repeats=1`, `games=2`, `max_plies=24`):
    - summary: `total_games=4`, `wins=2`, `losses=2`, `win_rate=0.5000`
    - `losses_with_disagreement=0` and no trace disagreements logged, so sampled losses were not caused by traced root-choice divergence against baseline.
  - Rejected micro-split in same loop:
    - attempted lower planner budget uplift under `turn_planner_intent_root_emergency_only`
    - killed immediately after repeated directional `vs fast` failures (`delta=-0.5000` in `1x1` fast-screen probes)
    - reverted; keep current candidate shape.
  - 2026-03-19 additional split checks:
    - Split A (broader emergency injection: `limit=2`, `max_heuristic_gap=220`) was killed immediately after directional `vs fast` failure (`delta=-0.5000` in `1x1` probe), then reverted.
    - Split B (allow emergency injected roots on strict safety-upgrade signal over top root) showed no measurable gain in sampled gates:
      - `pro-triage primary_pro` unchanged (`target_changed=1`)
      - full `pro-fast-screen vs fast` unchanged pass (`delta=+0.1250`)
      - bounded progressive unchanged (`vs normal +0.3333`, `vs fast +0.5000`)
      - reliability-loss probe sample (`repeats=1`, `games=3`, `max_plies=24`) remained `win_rate=0.5000` with `losses_with_disagreement=0`.
    - Split B reverted; keep prior stable emergency-only strict candidate shape.

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

### Idea: Pro primary tactical follow-up exact-lite
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro` (off-target guard: `opening_reply`)
- Triage pass signal: `primary_pro target_changed>0` with `opening_reply off_target_changed<=1`
- Calibration gate: none
- Candidate budget: 1
- Expected upside: stack low-budget exact-lite signal on the promoted primary tactical quiescence core to improve tactical conversion without reopening confirmation churn.
- CPU risk: low-to-medium
- Cheapest falsifier: fail `pro-triage` on `primary_pro`, or fail first duel.
- Escalate only if: `guardrails -> pro-triage -> runtime-preflight -> pro-fast-screen` is positive.
- Kill if: first duel is flat/negative, or bounded ladder fails CPU/primary gates.
- Next split if rejected: keep the promoted primary quiescence core and try one smaller exact-lite trigger gate before any broader changes.
- How to test: `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight -> pro-fast-screen -> bounded pro-progressive/pro-ladder`.
- Status: closed (no deterministic surface movement)
- Notes:
  - Probe `runtime_pro_confirmation_tactical_quiescence_v1` was killed immediately on `opening_reply` triage (`target_changed=0`, `off_target_changed=0`), so confirmation-side quiescence-only edits remain non-moving on opening fixtures.
  - Probe `runtime_pro_primary_signal_exact_lite_v1` was also killed immediately on `primary_pro` triage (`target_changed=0`, `off_target_changed=0`), so tiny exact-lite top-off on the promoted primary core is currently non-moving on fixture surfaces.
  - Current live split: none in this line.

### Idea: Pro primary selective-extension gate
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro` (off-target guard: `opening_reply`)
- Triage pass signal: `primary_pro target_changed>0` with `opening_reply off_target_changed<=1`
- Calibration gate: none
- Candidate budget: 1
- Expected upside: improve primary tactical conversion by reducing extension-path noise on top of the promoted primary tactical quiescence core.
- CPU risk: low-to-medium
- Cheapest falsifier: fail `pro-triage` on `primary_pro`, or fail `pro-fast-screen` vs fast.
- Escalate only if: `guardrails -> pro-triage -> runtime-preflight -> pro-fast-screen -> bounded pro-progressive` is positive.
- Kill if: first duel flat/negative, progressive flat/negative, or bounded ladder fails runtime-fit/primary gates.
- Next split if rejected: keep selective extensions enabled and split a single primary scoring/risk lever.
- How to test: `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight -> pro-fast-screen -> bounded pro-progressive -> bounded pro-ladder`.
- Status: closed (primary ladder miss persists in this branch)
- Notes:
  - Probe `runtime_pro_primary_signal_no_ext_v1` (primary-context selective extensions disabled) passed `guardrails`, `pro-triage` (`target_changed=4`, `off_target_changed=0`), and `runtime-preflight`.
  - First duel (`pro-fast-screen`) was positive on both lanes: vs normal `delta=+0.2500` (`conf=0.855`), vs fast `delta=+0.1250` (`conf=0.637`).
  - Bounded directional progressive (`initial=1`, `max=2`, `repeats=1`, `max_plies=56`) was positive on both lanes: vs normal `delta=+0.3333` (`conf=0.996`), vs fast `delta=+0.2778` (`conf=0.985`).
  - Bounded ladder eventually failed primary gate (`delta=-0.0833 < +0.0800`), so `runtime_pro_primary_signal_no_ext_v1` is killed.
  - Follow-up `runtime_pro_primary_signal_ext_share_v1` (keep selective extensions enabled, reduce node share only) failed `pro-triage` immediately (`target_changed=0`, `off_target_changed=0`).
  - Follow-up `runtime_pro_primary_signal_no_ext_eff_v1` (no-extension core plus higher root-efficiency margin) matched the same fast-screen/progressive positives as `no_ext_v1`, then failed bounded ladder with the identical primary gate miss (`delta=-0.0833 < +0.0800`).
  - Current live split: none in this line.

### Idea: Pro primary broad quiescence probe
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro` (off-target guard: `opening_reply`)
- Triage pass signal: `primary_pro target_changed>0` with `opening_reply off_target_changed<=1`
- Calibration gate: none
- Candidate budget: 1
- Expected upside: recover additional primary tactical lift by broadening quiescence child generation on top of the promoted Pro primary core.
- CPU risk: medium
- Cheapest falsifier: fail `pro-triage` on `primary_pro`, or fail `pro-fast-screen` vs normal.
- Escalate only if: `guardrails -> pro-triage -> runtime-preflight -> pro-fast-screen` is positive.
- Kill if: first duel is flat/negative, or bounded ladder fails CPU/primary gates.
- Next split if rejected: keep tactical-only quiescence in production and explore non-quiescence primary policy changes.
- How to test: `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight -> pro-fast-screen -> bounded pro-progressive/pro-ladder`.
- Status: closed (first duel flat vs fast)
- Notes:
  - `runtime_pro_primary_broad_quiescence_v1` passed `guardrails`, `pro-triage` (`target_changed=3`, `off_target_changed=0`), and `runtime-preflight`.
  - First duel (`pro-fast-screen`) result was mixed: vs normal `delta=+0.1250`, vs fast `delta=0.0000`.
  - Killed on first-duel flat signal per playbook (do not escalate flat candidates to progressive/ladder).

### Idea: Pro primary broad quiescence + reply-risk top-off
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro` (off-target guard: `opening_reply`)
- Triage pass signal: `primary_pro target_changed>0` with `opening_reply off_target_changed<=1`
- Calibration gate: none
- Candidate budget: 1
- Expected upside: preserve broad-primary movement while lifting first-duel `vs fast` above flat via a controlled primary reply-risk boost.
- CPU risk: medium-to-high
- Cheapest falsifier: fail `pro-triage` or first duel.
- Escalate only if: `guardrails -> pro-triage -> runtime-preflight -> pro-fast-screen` is positive on both normal and fast lanes.
- Kill if: first duel is flat/negative on any lane, or preflight/CPU signal regresses materially.
- Next split if rejected: keep tactical-only quiescence baseline and abandon broad-primary expansion family.
- How to test: `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight -> pro-fast-screen`.
- Status: closed (flat/negative vs fast and non-moving follow-ups)
- Notes:
  - `runtime_pro_primary_broad_quiescence_reply_risk_v1` passed `guardrails`, `pro-triage` (`target_changed=3`, `off_target_changed=0`), and `runtime-preflight`, then failed first duel with mixed result: vs normal `delta=+0.1250`, vs fast `delta=0.0000`.
  - Follow-up `runtime_pro_primary_signal_reply_risk_v1` (same promoted tactical core + moderate primary reply-risk top-off) failed immediately at `pro-triage` (`target_changed=0`, `off_target_changed=0`).
  - Follow-up `runtime_pro_primary_signal_tactical_enum_v1` (same promoted tactical core + small tactical enum/budget lift) also failed `pro-triage` (`target_changed=0`, `off_target_changed=0`).
  - One `pro-audit-screen` spot-check on `runtime_pro_primary_signal_tactical_enum_v1` produced mixed tiny-sample signal (`vs normal delta=+0.5000`, `vs fast delta=-0.5000`), so it remains a kill under the playbook.
  - Probe `runtime_pro_primary_signal_q_order_v1` (primary tactical-child ordering split) needed a larger quiescence bump to move `primary_pro` (`target_changed=1` at `180/20`), but then failed first duel on `pro-fast-screen` vs fast (`delta=-0.2500`); lowering to `150/16` removed movement (`target_changed=0`). Lane killed.
  - Current live split: none; broad-primary expansion family is closed.

### Idea: Fast tactical uplift against current Normal
- Base profile: `runtime_current`
- Target mode: `fast`
- Triage surface: `reply_risk` (off-target guard: `supermana`)
- Triage pass signal: `reply_risk target_changed>0` with `supermana off_target_changed<=1`
- Calibration gate: `triage-calibrate` only if first candidate is non-moving on `reply_risk`
- Candidate budget: 1
- Expected upside: recover the current `normal-vs-fast` gap while preserving Fast latency envelope.
- CPU risk: medium
- Cheapest falsifier: fail `triage` on `reply_risk`, or fail `fast-screen` with `SMART_PROMOTION_TARGET_MODE=fast`.
- Escalate only if: `guardrails -> triage -> runtime-preflight -> fast-screen` is positive on Fast target lanes.
- Kill if: first duel is flat/negative or speed envelope regresses materially.
- Next split if rejected: keep Fast CPU flat and split one Fast-only policy family at a time (reply-risk, root ordering, or tactical top-off), never combine two levers before first duel win.
- How to test: `guardrails -> SMART_TRIAGE_SURFACE=reply_risk triage -> runtime-preflight -> SMART_PROMOTION_TARGET_MODE=fast fast-screen -> progressive/ladder`.
- Status: in_progress
- Notes:
  - Directional trigger: latest 16-game sweep showed `normal-vs-fast delta=+0.1875`.
  - Probe `runtime_fast_reply_risk_v1` passed `guardrails`, `triage` on `reply_risk` (`changed=3/3`), and `runtime-preflight`.
  - Bounded `fast-screen` (`target=fast`, `screen 2->4`, repeats `1`, max plies `64`) was initially positive (`delta=+0.0833`, fast lane `delta=+0.1667`, normal lane `delta=0.0000`).
  - Follow-up bounded `progressive` (directional `1->2`, repeats `1`, max plies `56`) immediately flipped to reject (`delta=-0.0833`, fast lane `delta=-0.1667`, normal lane `delta=0.0000`), so `runtime_fast_reply_risk_v1` is killed.
  - Directional reference probe `runtime_pre_fast_root_quality_v1_normal_conversion_v3` moved `reply_risk` triage (`changed=3/3`) and passed `runtime-preflight`, but first duel was flat (`fast-screen delta=0.0000`, fast `2W-2L`, normal `2W-2L`) and was killed.
  - Probe `runtime_fast_reply_risk_tiebreak_v1` (Fast-only deterministic tie-break split on the same reply-risk shape) again moved `reply_risk` triage (`changed=3/3`) and passed `runtime-preflight`, but first duel repeated the same flat result (`delta=0.0000`, fast `2W-2L`, normal `2W-2L`) and was killed.
  - Probe `runtime_fast_drainer_safety_v1` (Fast-only stronger drainer safety/handoff/backtrack penalties) failed `drainer_safety` triage immediately (`changed=0/2`) and was killed without duel escalation.
  - Probe `runtime_fast_exact_lite_progress_v1` (Fast-only minimal exact-lite progress checks) failed `opponent_mana` triage (`changed=0/18`); `triage-calibrate opponent_mana` remained green, so this was a true non-moving kill.
  - Probe `runtime_fast_spirit_setup_v1` (Fast-only hard spirit deploy + deterministic tie-break + spirit-progress soft top-off) moved `spirit_setup` triage (`changed=1/2`) and passed `runtime-preflight`, but first duel was again flat (`delta=0.0000`, fast `2W-2L`, normal `2W-2L`) and was killed.
  - Probe `runtime_fast_tactical_quiescence_order_v1` (Fast tactical-only quiescence with ordered children) failed `reply_risk` triage (`changed=0/3`), but moved `opponent_mana` strongly (`changed=14/18`) and passed `runtime-preflight`; first duel rejected hard twice (`delta=-0.1875` then `-0.2500`, normal lane `-0.3750` both runs), so this lane is killed.
  - Probe `runtime_fast_root_shape_rebalance_v1` (Fast root/node branching rebalance with two-pass root allocation) failed `reply_risk` triage (`changed=0/3`) with calibration still green, and only weakly moved fallback `opponent_mana` (`changed=3/18`), so it was killed before duel escalation.
  - Probe `runtime_fast_history_killer_v1` (Fast history heuristic + killer ordering only) failed `reply_risk` triage (`changed=0/3`) with calibration green and also failed fallback `opponent_mana` triage (`changed=0/18`), so it was killed immediately.
  - Probe `runtime_fast_wider_reply_risk_v1` (Fast-only wider reply-risk envelope: shortlist `7`, reply limit `16`, node share `1350`) moved `reply_risk` triage (`changed=3/3`) and passed `runtime-preflight`; bounded `fast-screen` was positive (`delta=+0.0833`, fast lane `+0.1667`, normal lane `0.0000`), but bounded progressive opened flat at tier-0 (`delta=0.0000`, fast `6W-6L`, normal `6W-6L`) and runtime stayed heavy (>5m for this bounded run), so the branch was killed on flat + runtime-fit risk.
  - Probe `runtime_fast_spirit_deploy_only_v1` (Fast-only `hard_spirit_deploy` single lever) failed `spirit_setup` triage immediately (`changed=0/2`) and was killed before preflight/duel spend.
  - Probe `runtime_fast_walk_threat_prefilter_v1` (Fast-only `enable_walk_threat_prefilter` single lever) failed `drainer_safety` triage immediately (`changed=0/2`) and was killed before preflight/duel spend.
  - Probe `runtime_fast_reply_risk_clean_v1` (wider reply-risk envelope plus `prefer_clean_reply_risk_roots=true`) matched `runtime_fast_wider_reply_risk_v1` exactly through bounded `fast-screen` (`delta=+0.0833`, fast lane `+0.1667`, normal lane `0.0000`) and then again opened bounded progressive flat (`delta=0.0000`, fast `6W-6L`, normal `6W-6L`), so it was killed as a duplicate/non-converting split.
  - Probe `runtime_fast_reply_risk_margin_v1` (reply-risk margin-focused envelope: margin `150`, shortlist `6`, reply limit `14`, node share `1100`) again matched the same bounded `fast-screen` story (`delta=+0.0833`, fast lane `+0.1667`, normal lane `0.0000`) and replayed the same progressive trajectory; this branch was killed as duplicate/non-converting.
  - Probe `runtime_fast_tactical_quiescence_lite_v1` (Fast-only tiny tactical quiescence top-off: budget `8`, tactical-only, enum `8`) failed `opponent_mana` triage immediately (`changed=0/18`) with `triage-calibrate opponent_mana` still green, so it was killed as a true non-mover.
  - Probe `runtime_fast_no_tactical_prepass_v1` (Fast-only disable forced tactical prepass) also failed `opponent_mana` triage immediately (`changed=0/18`), so it was killed as another true non-mover.
  - Probe `runtime_fast_per_mon_drainer_fallback_v1` (Fast-only per-mon drainer attack fallback plus drainer priority enum) failed `drainer_safety` triage immediately (`changed=0/2`) and was killed before preflight/duel spend.
  - Probe `runtime_fast_per_mon_drainer_fallback_v2` (same profile knobs plus code-path attempt to backfill safer drainer attacks when only unsafe attack roots were initially enumerated) still failed `drainer_safety` triage immediately (`changed=0/2`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_conditional_forced_attack_v1` (Fast-only conditional forced drainer attack gate, only force when not ahead) failed `reply_risk` triage immediately (`changed=0/3`); `triage-calibrate reply_risk` stayed green, so this was another true non-mover.
  - Retained-profile probe `runtime_eff_non_exact_v2` passed `guardrails` and moved `reply_risk` (`changed=3/3`) but also moved off-target `supermana` heavily (`changed=3/3`), so it was killed on off-target guard failure before duel escalation.
  - Retained-profile probe `runtime_eff_non_exact_v1` passed `guardrails`, moved `reply_risk` (`changed=3/3`), held off-target `supermana` flat (`changed=0/3`), and passed `runtime-preflight`, but first duel (`fast-screen`, target `fast`) was exactly flat (`delta=0.0000`, fast `4W-4L`, normal `4W-4L`) and was killed on early reject.
  - Probe `runtime_fast_reply_risk_no_event_ordering_v1` (same `runtime_eff_non_exact_v1` fast lane plus single root-ordering lever `event_ordering_bonus=false`) matched the same pattern: `reply_risk changed=3/3`, off-target `supermana changed=0/3`, `runtime-preflight` pass, then first duel exactly flat (`fast-screen delta=0.0000`, fast `4W-4L`, normal `4W-4L`), so it was killed on early reject.
  - Probe `runtime_fast_drainer_full_pool_v1` (Fast-only `enable_drainer_attack_full_pool=true` to stop forced attack filtering) failed `reply_risk` triage immediately (`changed=0/3`); `triage-calibrate reply_risk` stayed green, so this was another true non-mover.
  - Probe `runtime_fast_reply_risk_medium_v1` (Fast-only medium reply-risk envelope: margin `130`, shortlist `5`, reply limit `10`, node share `900`) passed `guardrails`, moved `reply_risk` (`changed=3/3`), and held off-target `supermana` flat (`changed=0/3`), but first duel (`fast-screen`, target `fast`) blew past practical runtime envelope (run still incomplete after several minutes with only tier-0 output `delta=+0.0625`, fast `5W-3L`, normal `4W-4L`), so it was killed on runtime-fit risk.
  - Probe `runtime_fast_drainer_minimax_v1` (Fast-only `enable_drainer_attack_minimax_selection=true`) failed `reply_risk` triage immediately (`changed=0/3`); `triage-calibrate reply_risk` stayed green, so this was another true non-mover.
  - Probe `runtime_fast_reply_risk_medium_lite_v1` (Fast-only lighter medium reply-risk envelope with node share `780`) passed `guardrails`, moved `reply_risk` (`changed=3/3`), held off-target `supermana` flat (`changed=0/3`), and passed `runtime-preflight`; bounded `fast-screen` was positive (`delta=+0.0833`, fast lane `+0.1667`, normal lane `0.0000`), but bounded `progressive` still flattened completely (`delta=0.0000`, fast `9W-9L`, normal `9W-9L`, `MaxGamesReached`), so it was killed as a non-converting branch.
  - Probe `runtime_fast_reply_risk_hard_floor_v1` (Fast-only disabled interview soft/tiebreak) passed `guardrails` and `runtime-preflight`, but failed `reply_risk` triage (`changed=0/3`) and then repeated the first-duel flat reject pattern (`fast-screen delta=0.0000`, fast `2W-2L`, normal `2W-2L`).
  - Probe `runtime_fast_reply_risk_floor_guard_v2` (Fast-only widened reply-risk shortlist plus candidate-only hard reply-floor guard with margin `120`) passed `guardrails`, moved `reply_risk` (`changed=3/3`), held off-target `supermana` flat (`changed=0/3`), and passed `runtime-preflight`, but first duel was exactly flat again (`fast-screen delta=0.0000`, fast `2W-2L`, normal `2W-2L`), so it was killed.
  - Probe `runtime_fast_reply_risk_floor_guard_v3` (same lane with zero hard-floor margin) reproduced the same path: `reply_risk changed=3/3`, `supermana changed=0/3`, `runtime-preflight` pass, then identical early flat reject on first duel (`delta=0.0000`, fast `2W-2L`, normal `2W-2L`); branch killed.
  - Probe `runtime_fast_forced_attack_safety_v1` (code split: allow forced-attack lanes to run drainer-safety prefilter) passed `guardrails` but failed `drainer_safety` triage immediately (`changed=0/2`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_targeted_drainer_fallback_v1` (Fast-only `enable_targeted_drainer_attack_fallback=true`) passed `guardrails` but failed `reply_risk` triage immediately (`changed=0/3`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_drainer_exposure_penalty_v1` (Fast-only `root_drainer_exposure_penalty=900`) passed `guardrails` but failed `reply_risk` triage immediately (`changed=0/3`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_no_strict_anti_help_v1` (Fast-only `enable_strict_anti_help_filter=false`) passed `guardrails` but failed `reply_risk` triage immediately (`changed=0/3`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_forced_attack_safe_filter_v1` (code split: forced-attack lanes prefer safe forced-attack roots when available) passed `guardrails` but failed `drainer_safety` triage immediately (`changed=0/2`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_opponent_mana_prepass_v1` (Fast-only `enable_opponent_mana_prepass_exception=true`) passed `guardrails` but failed `opponent_mana` triage immediately (`changed=0/18`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_no_reply_risk_guard_v1` (Fast-only `enable_root_reply_risk_guard=false`) passed `guardrails`, moved `reply_risk` (`changed=3/3`), held off-target `supermana` flat (`changed=0/3`), and passed `runtime-preflight`; bounded `fast-screen` looked strong (`tier0 delta=+0.1250`, `tier1 delta=+0.1667`), but bounded `progressive` immediately rejected (`delta=-0.0833`, fast lane `-0.1667`), so it was killed on first earned-duel regression.
  - Probe `runtime_fast_reply_risk_two_pass_v1` (reply-risk-moving `runtime_eff_non_exact_v1` base plus Fast two-pass root allocation/focus) passed `guardrails`, moved `reply_risk` (`changed=3/3`), held off-target `supermana` flat (`changed=0/3`), and passed `runtime-preflight`, but first duel was exactly flat (`fast-screen delta=0.0000`, fast `2W-2L`, normal `2W-2L`); branch killed on early reject.
  - Probe `runtime_fast_search_structure_v1` (Fast-only aspiration/PVS/futility/killer/history/TT-depth toggles) passed `guardrails` but failed primary `reply_risk` triage (`changed=0/3`); one fallback diagnostic on `opponent_mana` was only weakly moving (`changed=2/18`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_reply_risk_floor_score_guard_v1` (code split: Fast clean-reply mode only lets reply-floor wins override when heuristic score is not materially worse, candidate profile on `runtime_eff_non_exact_v1`) passed `guardrails`, moved `reply_risk` (`changed=3/3`), held off-target `supermana` flat (`changed=0/3`), and passed `runtime-preflight`, but first duel was exactly flat again (`fast-screen delta=0.0000`, fast `2W-2L`, normal `2W-2L`); branch killed on early reject.
  - Probe `runtime_fast_conditional_forced_attack_pickup_v1` (code split: conditional forced-attack lane skips attack-only filtering when a safe high-value pickup line is available, candidate profile enabling `enable_conditional_forced_drainer_attack`) passed `guardrails` but failed primary `opponent_mana` triage immediately (`changed=0/18`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_pickup_override_v1` (code split: Fast targeted-drainer-fallback lane bypasses forced-attack-only filtering when safe high-value pickup roots already exist) passed `guardrails` but again failed primary `opponent_mana` triage immediately (`changed=0/18`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_reply_risk_narrow_margin_v1` (Fast-only tighter reply-risk guard envelope: margin `80`, shortlist `3`, reply limit `8`, node share `500`) passed `guardrails`, moved `reply_risk` (`changed=3/3`), and passed `runtime-preflight`, but first duel still rejected flat (`fast-screen`, target-only fast lane `2W-2L`, `delta=0.0000`, `EarlyReject`), so it was killed as another non-converting reply-risk retune.
  - Probe `runtime_fast_conditional_forced_attack_safety_v2` (code split: conditional forced-attack lane was allowed to skip attack-only filtering when all available attack roots still exposed own drainer and a safe non-attack root existed) passed `guardrails` but failed `drainer_safety` triage immediately (`changed=0/2`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_safety_rerank_v1` (Fast-only enable Normal-style root safety rerank + deep floor) passed `guardrails` but failed `drainer_safety` triage immediately (`changed=0/2`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_clean_reply_tiebreak_v1` (Fast-only `prefer_clean_reply_risk_roots=true` plus deterministic tie-break) passed `guardrails` but failed `reply_risk` triage immediately (`changed=0/3`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_opponent_mana_prepass_progress_v2` (code split: opponent-mana prepass exception also considered safe opponent-mana-progress lines, candidate profile enabled `enable_opponent_mana_prepass_exception`) passed `guardrails` but still failed `opponent_mana` triage immediately (`changed=0/18`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_reply_risk_counter_probe_v1` (code split: reply-risk snapshot added a lightweight counter-response probe to reduce over-penalizing trivially recoverable opponent replies) passed `guardrails`, moved `reply_risk` triage (`changed=3/3`), and passed `runtime-preflight`, but first duel was again exactly flat (`fast-screen delta=0.0000`, fast `4W-4L`, normal `4W-4L`), so it was killed on early reject.
  - Probe `runtime_fast_forced_attack_reply_risk_v1` (code split: forced-attack lane kept only attack roots that passed a quick reply-risk safety check, candidate profile enabled `enable_forced_attack_reply_risk_filter`) passed `guardrails`, moved `reply_risk` triage (`changed=3/3`), and passed `runtime-preflight`, but first duel stayed exactly flat (`fast-screen delta=0.0000`, fast `4W-4L`, normal `4W-4L`), so it was killed on early reject.
  - Probe `runtime_fast_forced_attack_non_losing_v1` (code split: forced-attack lane added a non-losing override to allow near-best non-attack roots when every forced attack allowed immediate-loss replies, candidate profile enabled `enable_forced_attack_non_losing_override`) passed `guardrails`, moved `reply_risk` triage (`changed=3/3`), and passed `runtime-preflight`, but first duel was still exactly flat (`fast-screen delta=0.0000`, fast `4W-4L`, normal `4W-4L`), so it was killed on early reject.
  - Probe `runtime_fast_forced_attack_immediate_score_v1` (code split: forced-attack lane allowed safe immediate-score non-attack roots near forced-attack score to override attack-only filtering, candidate profile enabled `enable_forced_attack_immediate_score_override`) passed `guardrails` but failed `reply_risk` triage immediately (`changed=0/3`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_forced_attack_score_guard_v1` (code split: forced-attack filtering applied only when best attack root stayed within a configurable score margin from overall best root, candidate profile enabled `enable_forced_attack_score_guard`) passed `guardrails`, moved `reply_risk` triage (`changed=3/3`), and passed `runtime-preflight`, but first duel remained exactly flat (`fast-screen delta=0.0000`, fast `4W-4L`, normal `4W-4L`), so it was killed on early reject.
  - Probe `runtime_fast_no_reply_risk_guard_clean_v1` (Fast-only `enable_root_reply_risk_guard=false` plus `prefer_clean_reply_risk_roots=true`) passed `guardrails`, moved `reply_risk` triage (`changed=3/3`), and passed `runtime-preflight`; first duel was positive (`fast-screen delta=+0.1458`, fast `19W-5L`, normal `12W-12L`, `stop=EarlyPromote`), but bounded progressive (`duel 1->2`, repeats `1`, `max_plies=56`) rejected immediately (`delta=-0.0833`, fast lane `-0.1667`, normal lane `0.0000`), so it was killed on first earned-duel regression.
  - Probe `runtime_fast_no_reply_risk_guard_clean_antihelp_v1` (Fast-only `enable_root_reply_risk_guard=false`, `prefer_clean_reply_risk_roots=true`, plus stronger anti-help top-off `score_margin=320/reply_limit=10`) passed `guardrails`, moved `reply_risk` triage (`changed=3/3`), and passed `runtime-preflight`; first duel stayed strong (`fast-screen delta=+0.1458`, fast `19W-5L`, normal `12W-12L`, `stop=EarlyPromote`), but bounded progressive again rejected immediately (`duel 1->2`, repeats `1`, `max_plies=56`, `delta=-0.0833`, fast lane `-0.1667`, normal lane `0.0000`), so it was killed as a non-converting no-guard follow-up.
  - Probe `runtime_fast_reply_risk_guard_safe_attack_bypass_v1` (code split: keep Fast reply-risk guard enabled but allow a narrow safe drainer-attack bypass before guard selection) passed `guardrails` but failed primary `reply_risk` triage immediately (`changed=0/3`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_reply_risk_safe_score_override_v1` (code split: keep guard enabled but allow score-led overrides when candidate reply-risk stays within a safe tolerance band) passed `guardrails` but failed primary `reply_risk` triage immediately (`changed=0/3`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_tactical_quiescence_mid_v1` (Fast-only tactical quiescence top-off: budget `20`, tactical-only, enum `12`) passed `guardrails` but failed primary `opponent_mana` triage immediately (`changed=0/18`); `triage-calibrate opponent_mana` remained green, so it was killed as a true non-mover.
  - Probe `runtime_fast_spirit_soft_priority_v1` (Fast-only spirit soft-priority top-off: higher soft bonuses for supermana/opponent-mana progress and score) passed `guardrails`, moved `spirit_setup` triage (`changed=1/2`), and passed `runtime-preflight`, but first duel rejected exactly flat (`fast-screen delta=0.0000`, fast `4W-4L`, normal `4W-4L`), so it was killed on early reject.
  - Probe `runtime_fast_attacker_proximity_v1` (Fast-only attacker-proximity scoring weights while keeping current runtime policy) passed `guardrails`, moved `opponent_mana` triage (`changed=2/18`), and passed `runtime-preflight`, but first duel rejected negative (`fast-screen delta=-0.1250`, fast `2W-6L`, normal `4W-4L`), so it was killed on early reject.
  - Probe `runtime_fast_attacker_proximity_safety_v1` (Fast-only attacker-proximity scoring plus stronger drainer-safety margin/exposure penalties) moved `opponent_mana` triage more strongly (`changed=4/18`) with off-target `supermana` clean (`changed=0/3`) and passed `runtime-preflight`, but first duel repeated the same negative reject (`fast-screen delta=-0.1250`, fast `2W-6L`, normal `4W-4L`), so this safety top-off did not rescue the attacker-proximity lane.
  - Probe `runtime_fast_no_potion_compensation_v1` (Fast-only disable potion-progress compensation) passed `guardrails` but failed primary `opponent_mana` triage immediately (`changed=0/18`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_handoff_backtrack_penalty_v1` (Fast-only stronger handoff/backtrack penalties: `handoff=380`, `backtrack=300`) passed `guardrails` but failed primary `reply_risk` triage immediately (`changed=0/3`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_reply_risk_progress_tiebreak_v1` (code split: reply-risk guard tiebreak preferred supermana/opponent-mana progress when worst-reply floors were within a tight tolerance) passed `guardrails` but failed primary `reply_risk` triage immediately (`changed=0/3`), so it was killed before preflight/duel spend.
  - Probe `runtime_fast_no_reply_risk_guard_drainer_safety_v1` (Fast-only disable reply-risk guard plus stronger drainer safety margin/exposure penalty) passed `guardrails`, moved primary `reply_risk` triage (`changed=3/3`), held off-target `supermana` flat (`changed=0/3`), and passed `runtime-preflight`; full `fast-screen` passed with early promote (`delta=+0.1458`, fast `19W-5L`, normal `12W-12L`), but full `progressive` with `SMART_PROMOTION_TARGET_MODE=fast` hit a runtime cliff (only tier-0 finished after ~11 minutes: `delta=+0.0833`, fast `16W-8L`, normal `12W-12L`), so it was killed on runtime-fit risk before ladder.
  - Probe `runtime_fast_no_reply_risk_guard_drainer_safety_light_v1` (same no-guard + drainer-safety lane, but `enable_enhanced_drainer_vulnerability=false`) reproduced the exact same keep signal through `fast-screen` (`delta=+0.1458`, fast `19W-5L`, normal `12W-12L`) and the same `progressive` tier-0 signal (`delta=+0.0833`, fast `16W-8L`, normal `12W-12L`), while still taking multiple minutes just to reach tier-0, so it was killed as a duplicate runtime-fit failure.
  - Current branch takeaway: this reply-risk shortlist movement family remains non-converting at first duel against `runtime_release_safe_pre_exact`; next split should leave this family and test a different Fast-only policy lane.
  - Updated takeaway: even non-reply-risk Fast splits are either non-moving on triage (`drainer_safety`, minimal exact-lite, `hard_spirit_deploy` only, `walk_threat_prefilter` only) or lose first duel despite tactical-surface movement (`spirit_setup`, tactical quiescence ordering). Fast tactical-surface movement alone is not a promotion signal.
  - Current live split: none in this family.

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
