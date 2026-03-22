# Automove Knowledge

This document keeps only durable lessons that should shape future automove work.

## Durable Strategy Signals

- Strong root filtering beats wider raw enumeration when the filters are tactical and cheap.
- Full-turn planning can outperform per-input tree expansion when it is routed through existing legality checks and accepted against the real ranked root surface.
- A hash-guarded continuation cache materially improves turn-level planner stability and cost, but it must be invalidated on divergence and cleared between duel games in harness runs.
- Drainer safety needs near-hard treatment in production search; soft penalties alone miss obvious blunders.
- Root reply-risk guards and efficiency tie-breaks still earn their keep because they eliminate fake-good roots before deeper search.
- Opening-specific latency guardrails matter. A stronger search that stalls on the first real black reply is not promotable.
- Production wasm must stay single-shot and predictable. Deferred or post-return search is still not release-safe.
- Default new code candidates to `runtime_current`; use retained non-production profiles only for calibration, references, or explicit audits.
- Config knob space is exhausted. Future gains need new code, not more `SmartSearchConfig` permutations.

## Durable Workflow Rules

- Keep the active frontier small: one live idea, one candidate, one earned path.
- Run `guardrails -> triage/pro-triage -> runtime-preflight` before paying duel budget.
- If a surface is calibrated, do not move the CPU gate back in front of triage.
- `audit-screen` and `pro-audit-screen` are spot checks for clean rejects, not promotion proof.
- Compress the lesson immediately when a run matters. Raw logs and stamps are disposable evidence, not memory.
- Keep ignored harness test names unique; `cargo test` substring filters can accidentally run the wrong stage.
- Candidate logs belong under `target/experiment-runs/<candidate>/`; runtime-preflight state belongs under `target/experiment-stamps/`.

## Mistakes Not To Repeat

- Do not keep historical profiles live in the active registry after their lesson is absorbed.
- Do not treat `target/experiment-runs` or `target/experiment-stamps` as durable memory.
- Do not reopen a branch just because it had a good bounded screen; reopen only with a new hypothesis.
- Do not spend ladder budget on branches that are still flat in the first earned duel stage.
- Do not trust pooled or sampled losses as selector evidence when direct reliability tracing shows `disagreements=0`.
- Do not let Pro-specific selector flow bypass opening-book fallback ordering; that ordering mistake already produced confirmation regressions.
- Do not inject planner roots globally. Crisis-only gating is safer, and even then the branch must prove direct value against `runtime_current`.

## Current Improvement Direction

- The live frontier is `runtime_pro_turn_engine_v1`.
- The current problem is not "can the engine plan?" but "does the engine create beneficial disagreements against `runtime_current`?"
- Next code should either increase direct selector impact on the remaining `NoPlan` / selected-mismatch fixtures or improve disagreement tracing so the failure mode is explicit.
- Keep Fast work parked until there is a genuinely new code path; reply-risk, scoring-only, and minor search-order retunes are already saturated.
