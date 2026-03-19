## Pro Intent Planner V2 Findings (2026-03-19)

Candidate profile: `runtime_pro_intent_planner_v2`

### Implemented
- Hybrid intent-first planner in `automove_turn_planner`:
  - Intent generation from board/resource budgets.
  - Intent-specific compile path with legality fallback.
  - Tactical emergency adaptive planner budget.
  - Planner utility memoization within one call.
  - Tactical deny intent/route family.
  - Planner diagnostics counters.
- Pro integration:
  - Candidate-only root injection path gated by strict safety acceptance.
  - Candidate-only tactical weighting hook when intent-root-injection is enabled.
- Experiment harness updates:
  - New retained profile `runtime_pro_intent_planner_v2`.
  - Cache-reuse probe now captures planner diagnostics.

### Current gate signals
- `guardrails`: pass.
- `runtime-preflight`: pass (stage1 CPU + exact-lite diagnostics).
- `pro-triage` (`SMART_TRIAGE_SURFACE=primary_pro`): pass with movement
  - `target_changed=1`, `off_target_changed=0`.
- `pro-fast-screen`:
  - vs normal: pass (`delta=+0.2500` in latest run).
  - vs fast: fail (`delta=-0.2500` in latest run).

### Next focus
- Improve candidate strength/robustness in the fast-screen `vs fast` lane without losing:
  - `primary_pro` movement signal,
  - runtime-preflight CPU envelope,
  - opening reply guardrails.

## Iteration update (2026-03-19, later pass)

### Isolation result
- The `vs fast` fast-screen regression is tied to **active injected root candidates**.
- Probe outcomes:
  - `runtime_pro_turn_planner_v1` vs fast-screen fast lane: pass (`delta=+0.1250`).
  - `runtime_pro_intent_planner_v2` with injection enabled (`limit>0`): repeatedly failed (`delta=-0.1250` to `-0.2500`).
  - `runtime_pro_intent_planner_v2` with intent flag on but injection effectively disabled (`limit=0`): pass (`delta=+0.1250`).

### Candidate shape tested now
- `configure_runtime_pro_intent_planner_v2`:
  - `enable_turn_planner_intent_root_injection = true`
  - `turn_planner_intent_root_injection_limit = 0`
  - `turn_planner_intent_root_max_heuristic_gap = 320`
- Kept a lightweight intent-root bonus (small tactical/safety nudges only; removed generic progress boosts).

### Gate signals with current shape
- `pro-triage` (`primary_pro`): pass with movement (`target_changed=1`, `off_target_changed=0`).
- `runtime-preflight`: pass.
- `pro-fast-screen`: pass
  - vs normal: `delta=0.0000`
  - vs fast: `delta=+0.1250`
- Bounded progressive directional probes (`initial=1, max=1, repeats=1, max_plies=40`) were positive:
  - vs normal: `delta=+0.3333`
  - vs fast: `delta=+0.5000`

### Remaining blocker
- Runtime-fit at higher duel stages remains the open risk:
  - default `pro-progressive` spent many minutes on `vs normal` before completion in multiple runs.
  - bounded ladder probe showed acceptable speed ratio early (`8.586 < 10.0`) but remained too slow to complete quickly in this loop.

## Iteration update (2026-03-19, emergency-only strict injection)

### Candidate shape now
- `configure_runtime_pro_intent_planner_v2`:
  - `enable_turn_planner_intent_root_injection = true`
  - `turn_planner_intent_root_injection_limit = 1`
  - `turn_planner_intent_root_max_heuristic_gap = 200`
  - `turn_planner_intent_root_emergency_only = true`
- Added planner bridge gate: root-intent injection is only active when planner tactical emergency state is true.
- Added emergency-only injected-root acceptance filter:
  - only crisis-resolving roots are allowed (drainer kill, drainer safety recover, or immediate score conversion),
  - explicit reject of handoff/vulnerable non-recover lines.

### Current signals
- `guardrails`: pass.
- `pro-triage` (`primary_pro`): pass with movement (`target_changed=1`, `off_target_changed=0`).
- `runtime-preflight`: pass.
- First duel lanes (direct cargo run):
  - `pro-fast-screen vs fast`: pass (`delta=+0.1250`, `confidence=0.637`).
  - `pro-fast-screen vs normal`: bounded directional probe (`1x1`) is non-negative (`delta=0.0000`).
- Bounded progressive probes (`1x1 @56`):
  - vs normal: `delta=+0.3333`
  - vs fast: `delta=+0.5000`
- Bounded ladder probe (`2x2`, `max_plies=56`) speed gate still passes:
  - `ratio=8.589` under cap `10.0`.

### Remaining risk
- Need stable full-capture runs for default `pro-fast-screen vs normal` and bounded/full ladder summaries; long-running wrapper/session capture remains flaky in this environment.

## Iteration update (2026-03-19, rerun + failed micro-split)

### Rerun status on current shape
- Fresh earned-path early gates stayed green:
  - `guardrails`: pass
  - `pro-triage primary_pro`: pass (`target_changed=1`, `off_target_changed=0`)
  - `runtime-preflight`: pass
- Direct full lane recheck:
  - `pro-fast-screen vs fast`: pass (`delta=+0.1250`, `confidence=0.637`)
- Directional bounded rechecks stayed positive:
  - `pro-progressive vs normal` (`1x1 @56`): `delta=+0.3333`
  - `pro-progressive vs fast` (`1x1 @56`): `delta=+0.5000`
- Bounded ladder speed gate stayed under cap:
  - `ratio=8.650` (`< 10.0`)

### Rejected tweak in this loop
- Tried reducing planner budget uplift specifically for `turn_planner_intent_root_emergency_only`.
- Result: directional `pro-fast-screen vs fast` (`1x1`) failed repeatedly (`delta=-0.5000`), so the tweak was reverted immediately.

### Current takeaway
- Keep the existing emergency-only injection shape from commit `36d92ae926`; the attempted budget-cut split weakened tactical conversion.
- Remaining blocker is still stable, full-capture ladder/normal-lane reporting in this environment, not early-gate signal quality.

## Iteration update (2026-03-19, reliability probe + rerun)

### Gate revalidation
- Re-ran early earned path on unchanged candidate shape:
  - `guardrails`: pass
  - `pro-triage primary_pro`: pass (`target_changed=1`, `off_target_changed=0`)
  - `runtime-preflight`: pass
- Re-ran full `pro-fast-screen vs fast` directly:
  - pass (`delta=+0.1250`, `confidence=0.637`)
- Re-ran bounded progressive (`1x1 @56`) directly:
  - vs normal: `delta=+0.3333`
  - vs fast: `delta=+0.5000`
- Re-ran bounded ladder speed gate:
  - `ratio=8.650` (`< 10.0`)

### Reliability-loss probe signal
- Probe: `smart_automove_pro_reliability_loss_probe`
  - candidate: `runtime_pro_intent_planner_v2`
  - baseline: `runtime_release_safe_pre_exact`
  - config sample (`repeats=1`, `games=2`, `max_plies=24`)
- Summary:
  - `total_games=4`, `wins=2`, `losses=2`, `win_rate=0.5000`
  - `losses_with_disagreement=0`, `disagreements_logged=0`
- Interpretation: sampled losses in this probe did not come from candidate-vs-baseline root-choice divergence in traced positions.

## Iteration update (2026-03-19, split checks and reverts)

### Split A (rejected): broader emergency injection
- Change tested:
  - `turn_planner_intent_root_injection_limit: 1 -> 2`
  - `turn_planner_intent_root_max_heuristic_gap: 200 -> 220`
- Result:
  - `pro-triage primary_pro`: still pass (`target_changed=1`)
  - directional `pro-fast-screen vs fast` (`1x1`) failed immediately (`delta=-0.5000`)
- Action: reverted.

### Split B (rejected): emergency acceptance safety-upgrade expansion
- Change tested:
  - allowed emergency injected roots with strict safety-upgrade signal over top root as additional crisis criterion.
- Result:
  - no observable improvement in sampled gates:
    - `pro-triage primary_pro`: unchanged (`target_changed=1`)
    - full `pro-fast-screen vs fast`: pass but unchanged (`delta=+0.1250`)
    - bounded progressive unchanged (`vs normal +0.3333`, `vs fast +0.5000`)
  - reliability probe at slightly larger sample (`repeats=1`, `games=3`, `max_plies=24`) remained:
    - `total_games=6`, `wins=3`, `losses=3`, `losses_with_disagreement=0`
- Action: reverted.

### Current standing
- Keep candidate on prior stable emergency-only strict shape from `36d92ae926`.
- Latest bounded ladder speed-gate checks remained green (`ratio ~8.55`).
- Remaining blocker is still stable full-capture reporting for `pro-fast-screen vs normal` and full ladder summaries in this environment.
