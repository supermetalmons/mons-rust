# Automove Knowledge

This document keeps only durable lessons that should outlast the current session.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the operator loop and `AUTOMOVE_IDEAS.md` for the next live split.

## Stable Runtime Truths

- Shipping Pro is `runtime_current`.
- `runtime_pro_turn_engine_v30` is the only live Pro challenger.
- `runtime_pro_turn_engine_v1` is reference-only regression history.
- `runtime_pro_turn_engine_v30` is a guarded `ProV2` path with deliberate opening-book and early-white fallbacks, not a raw always-on engine.
- Full-turn planning can beat per-input expansion, but only if it survives the real ranked-root surface and direct duel evidence.
- Opportunity-context extraction is worth keeping separate from raw input search. It remains useful shared structure for planner seeds and selector guards.
- Continuation reuse and no-plan reuse matter, but cache keys must include a config fingerprint.
- Drainer safety, root reply-risk guards, and efficiency tie-breaks still earn their keep because they cheaply reject fake-good roots.
- Opening-specific latency still matters. A candidate that stalls on the first real black reply is not promotable.
- Hybrid fallbacks must respect retained opening and eligibility guards before they call expensive plan probes.
- Wrapper-only tuning saturates quickly. Once the obvious knobs are in place, the next gain usually has to come from shared engine or search code.
- Global `SmartSearchConfig` knob space is effectively exhausted. Future gains need new code or a sharper selector/search hypothesis, not another broad retune.
- Production wasm still needs single-shot, predictable search. Deferred or post-return work is not release-safe.

## Durable Workflow Rules

- Keep the active frontier small: one live idea, one retained candidate, one canonical path.
- For Pro work, compare directly against `runtime_current`.
- The canonical Pro path is `guardrails -> pro-triage(primary_pro) -> runtime-preflight -> pro-reliability`.
- `opening_reply` is a narrow fallback-order and opening-regression surface, not the default Pro surface.
- `pro-triage` only passes when the target surface changes and off-target churn stays at `<= 1`.
- `runtime-preflight` is the required stamp before duel stages unless the run is intentionally diagnostic.
- `pro-reliability` is the first real promotion gate. It must clear current `Pro`, current `Normal`, and current `Fast` at `win_rate >= 0.90`, `confidence >= 0.99`, and `candidate_avg_move_ms <= 700`.
- `pro-reliability-confirm` is the final promotion proof. Do not promote on smaller-corpus evidence.
- Prefer a fresh live `pro-reliability` sample over diagnostic probes when the wall is unclear.
- Use `triage-calibrate` only when a retained triage surface is new or no longer calibrated.
- Use `pro-opening-speed-probe` only for opening-specific regressions.
- Use the hotspot probe only after a real duel stall, and only to narrow the next code surface.
- Logs, stamps, and process samples are disposable evidence, not durable memory.
- Keep ignored harness test names unique; `cargo test` substring filters can hit the wrong stage.

## Mistakes Not To Repeat

- Do not reopen archived profile IDs or retired branch families without a brand-new hypothesis.
- Do not spend another loop on wrapper-only reroutes, current-Normal fallbacks, or replay-specific acceptance clamps just because they fix a traced exact.
- Do not reopen cache-size, memo-shape, reserve-heavy, or hasher experiments without a direct quality story tied to the live duel wall.
- Do not reopen broad search-budget, reply-budget, or generic search-knob clamps without evidence that the real wall lives on that surface.
- Do not keep branches just because disagreement counts shrink or hotspot counters move. If direct duel quality stays flat or regresses, kill the line.
- Do not retain white-only or black-only local seam repairs that fail to move the broader `vs current Normal` wall.
- Do not treat the relaxed `700ms` move-time cap as permission to reopen parity-preserving speed regressions.

## Current Durable Direction

- Speed is already acceptable on the retained Pro challenger. Promotion is blocked by quality, especially versus current Normal, not by the `700ms` move-time budget.
- The remaining work should favor broader in-path selector and root-choice changes over wrapper fallbacks, fallback widening, or exact replay fixes.
- The most credible future wins are shared selector/search families that move direct `runtime_pro_turn_engine_v30` vs `runtime_current` results, not local acceptance or wrapper repairs that only clear one traced seam.
