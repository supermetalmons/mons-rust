# Automove Knowledge

This document keeps only durable lessons that should shape future automove work.

## Durable Strategy Signals

- Strong root filtering beats wider raw enumeration when the filters are tactical and cheap.
- Full-turn planning can outperform per-input tree expansion when it is routed through existing legality checks and accepted against the real ranked root surface.
- Opportunity-context extraction is worth keeping separate from raw input search. It gives cheap whole-turn structure that can guide both planner seeds and selector guards.
- Plan / no-plan / continuation cache reuse matters. Recomputing the same full-turn shape on each input request burns Pro budget for no strength gain.
- Cache keys must include a config fingerprint. Reusing continuation or no-plan results across different runtime shapes produced bad selector behavior during the wave.
- Selector utility helpers and followup-floor caches are worth keeping as shared infrastructure even when a candidate branch is retired.
- Drainer safety needs near-hard treatment in production search; soft penalties alone miss obvious blunders.
- Root reply-risk guards and efficiency tie-breaks still earn their keep because they eliminate fake-good roots before deeper search.
- Opening-specific latency guardrails matter. A stronger search that stalls on the first real black reply is not promotable.
- Hybrid profile-level fallbacks must respect the retained opening and eligibility guards before they call into expensive plan probes; otherwise the comparison step itself can become the stage-1 CPU regression even when move selection stays unchanged.
- Production wasm must stay single-shot and predictable. Deferred or post-return search is still not release-safe.
- Wrapper-only tuning can recover specific cross-budget lanes, but it saturates quickly. When many tiny wrapper branches accumulate, the next gain usually needs shared engine/search work instead.
- Config knob space is exhausted. Future gains need new code, not more `SmartSearchConfig` permutations.

## Durable Workflow Rules

- Keep the active frontier small: one live idea, one candidate, one earned path.
- Run `guardrails -> triage/pro-triage -> runtime-preflight` before paying duel budget.
- For Pro work, treat `pro-reliability` as part of the canonical earned path before `pro-fast-screen`.
- If a surface is calibrated, do not move the CPU gate back in front of triage.
- `audit-screen` and `pro-audit-screen` are spot checks for clean rejects, not promotion proof.
- Pro-aware `runtime-preflight` was a real workflow fix. Pro candidates should be judged against Pro budget, not Fast/Normal budget spillover.
- Compress the lesson immediately when a run matters. Raw logs and stamps are disposable evidence, not memory.
- Keep ignored harness test names unique; `cargo test` substring filters can accidentally run the wrong stage.
- Candidate logs belong under `target/experiment-runs/<candidate>/`; runtime-preflight state belongs under `target/experiment-stamps/`.
- When bounded `pro-reliability` stalls, prefer a fresh live duel sample over the synthetic hotspot probe when choosing the next split. On this ProV2 line the visible wall has jumped across three different surfaces in consecutive iterations: selector ordering, macro oracle projection, and turn-planner targeted score-window queries.
- When a live sample clearly moves the wall to a new surface, follow the moved wall immediately instead of spending another iteration on the old one. In the latest ProV2 wave, the useful oracle cut (`v40`) pushed the blocker back into `search_score -> ranked_child_states`, and a follow-up search-side reuse cut (`v41`) reduced that search stack further even though it still was not promotable.
- If a move-efficiency caller only reads spirit-assisted exact fields plus safe-progress fields, do not request the exact tactical `SCORE_WINDOW` flag there. On the ProV2 selector path, paying for that unused field kept `exact_best_immediate_tactical_window_on_board_with_hash` alive for no behavior gain until `v42` removed it.
- When a retained scoring-summary optimization shifts the live wall from summary construction to direct point queries on the same caller surface, follow the caller that still emits those direct queries. In the latest ProV2 wave, `v43` cut `attack_reach_summary` materially, but the next visible blocker immediately became the cheap-pass `can_attack_target_on_board_with_hash` calls under `ranked_child_states`.
- A cheap lower-bound drainer-attack screen is worth keeping when it can reject impossible child vulnerability probes before exact reach work. In the latest ProV2 wave, `v44` moved the live wall off cheap-pass `can_attack_target_on_board_with_hash` and back into the oracle exact path without disturbing the front gates.
- On one-shot spirit-preview boards, a local dual drainer pickup BFS can beat filtered global pickup-cache queries. In the latest ProV2 wave, `v45` improved stage-1 CPU and the bounded hotspot by replacing filtered pickup-cache misses inside `exact_best_immediate_tactical_window_on_board_with_hash`, but the durable lesson is to follow the wall back to `build_child_eval_bundle` / scoring attack summaries once the oracle exact hotspot recedes.

## Mistakes Not To Repeat

- Do not keep historical profiles live in the active registry after their lesson is absorbed.
- Do not treat `target/experiment-runs` or `target/experiment-stamps` as durable memory.
- Do not reopen a branch just because it had a good bounded screen; reopen only with a new hypothesis.
- Do not spend ladder budget on branches that are still flat in the first earned duel stage.
- Do not trust pooled or sampled losses as selector evidence when direct reliability tracing shows `disagreements=0`.
- Do not let Pro-specific selector flow bypass opening-book fallback ordering; that ordering mistake already produced confirmation regressions.
- Do not inject planner roots globally. Crisis-only gating is safer, and even then the branch must prove direct value against `runtime_current`.
- Do not keep sprawling wrapper split families alive once one branch has clearly become the retained frontier.

## Current Improvement Direction

- The strongest retained ProV2 frontier is `runtime_pro_turn_engine_v30`, not the old CPU-heavy ancestor.
- `runtime_pro_turn_engine_v2` remains useful only as archived evidence that the ProV2 direction can be strong but too expensive.
- The current problem is no longer “can ProV2 plan a stronger turn?”; the retained code already proves that it can on many lanes.
- The current problem is to finish the earned path cleanly with one retained frontier under strict gates, not to keep proliferating branch-local wrappers.
- Next code should target exact remaining earned-path losses on `runtime_pro_turn_engine_v30` or improve shared engine/search behavior that benefits the retained frontier directly.
- Keep Fast work parked until there is a genuinely new code path; reply-risk, scoring-only, and minor search-order retunes are already saturated.
