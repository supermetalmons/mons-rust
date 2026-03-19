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
