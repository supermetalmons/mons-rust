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
