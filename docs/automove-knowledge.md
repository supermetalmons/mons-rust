# Automove Knowledge

This document keeps only durable lessons that should outlast the current session.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the operator loop and `AUTOMOVE_IDEAS.md` for the next live split.

## Stable Runtime Truths

- Shipping Pro is `runtime_current`, not the retained challenger profiles.
- `runtime_pro_turn_engine_v30` is the only live Pro challenger. Treat it as a guarded `ProV2` path with deliberate opening and early-white fallbacks, not a raw always-on engine.
- `runtime_pro_turn_engine_v1` is reference-only regression history.
- Full-turn planning can beat per-input expansion, but only when it survives the real ranked-root surface and direct Pro-vs-Pro duel evidence.
- Opportunity-context extraction is worth keeping separate from raw input search. It is useful shared structure for planner seeds and selector guards.
- Continuation and no-plan reuse matter, but cache keys must include a config fingerprint.
- Drainer safety, root reply-risk guards, and efficiency tie-breaks still earn their keep because they kill fake-good roots cheaply.
- Opening-specific latency still matters. A candidate that stalls on the first real black reply is not promotable.
- Hybrid fallbacks must respect retained opening and eligibility guards before they call expensive plan probes; otherwise the fallback machinery itself becomes the regression.
- Wrapper-only tuning saturates quickly. Once the obvious knobs are already in place, the next gain usually has to come from shared engine/search code.
- Config knob space is effectively exhausted. Future gains need new code, not more `SmartSearchConfig` permutations.
- Production wasm still needs single-shot, predictable search. Deferred or post-return work is not release-safe.

## Durable Workflow Rules

- Keep the active frontier small: one live idea, one candidate, one canonical path.
- For Pro work, compare directly against `runtime_current`, not the script's compatibility default baseline.
- The canonical Pro path is `guardrails -> pro-triage(primary_pro) -> runtime-preflight -> pro-reliability`.
- `opening_reply` is a narrow fallback-order and opening-regression surface, not the default Pro surface.
- `pro-triage` only counts as a pass when the target surface changes and off-target churn stays at `<= 1`.
- `runtime-preflight` is the required stamp before duel stages unless the run is intentionally diagnostic.
- `pro-reliability` is the first real promotion gate. `pro-fast-screen`, `pro-progressive`, and `pro-ladder` come later.
- If a surface is already calibrated, do not move the CPU gate back in front of triage.
- `audit-screen` and `pro-audit-screen` are reject diagnostics, not promotion evidence.
- Prefer a fresh live `pro-reliability` sample over the hotspot probe when the wall is unclear or has moved.
- Use the hotspot probe only after a real duel stall, and only to narrow the next code surface.
- Compress the lesson immediately when a run matters. Logs, stamps, and process samples are disposable evidence, not memory.
- Keep ignored harness test names unique; `cargo test` substring filters can run the wrong stage.

## Engine And Search Lessons Worth Keeping

- If a caller only consumes spirit-assisted exact fields plus safe-progress fields, do not request the exact tactical `SCORE_WINDOW` payload there.
- If a caller only needs the active-turn score window, route it through a tactical score-window projection instead of a full turn summary.
- Cheap lower-bound drainer-attack screens are worth keeping when they can reject impossible child vulnerability probes before exact reach work.
- On one-shot spirit-preview paths, local work can beat repeated filtered global cache queries.
- Mutate/undo preview application is often better than clone-per-preview on exact tactical spirit paths.
- After a cut moves the wall, follow the moved wall immediately instead of iterating again on the old surface.
- When a new helper removes one hotspot, verify that the helper itself did not become the new wall before going deeper.
- If a retained optimization shifts cost from summary construction to direct point queries on the same caller surface, follow the caller that still emits those queries.
- Low-budget exact fast paths are worth keeping when they can prove a drainer pickup or immediate window is impossible without entering the full BFS / pickup-window path.
- Low-budget drainer pickup windows are also worth direct fast paths. They can remove most uncached pickup BFS work from the live hotspot, even if that does not guarantee a duel-stage wall-clock win by itself.
- Sharing local after-window cache entries across score/denial flag variants is not enough by itself when the underlying after-window projection still runs at nearly the same frequency.
- When a tactical result can be masked exactly, reuse cached superset flag results for smaller tactical spirit, immediate-window, pickup-window, and projection queries instead of rebuilding each flag subset.
- Budgeted carrier-to-pool queries should not compute full shortest paths. When the caller only needs reachability within `N`, use a bounded search and do not hash the whole board just to initialize an actor move memo that ignores that hash.
- Once pickup BFS cost drops, full-board exact hash rebuilds on spirit preview surfaces become a real wall. Keep exact board hashing incrementally maintainable so preview code can update touched squares instead of rescanning the board.
- Zero-move after-window tactical spirit queries do not need the full immediate-window helper. When a spirit preview leaves no remaining mon moves, derive the exact tactical window from touched mana-pool count deltas instead of re-entering the cached search path.
- One-move after-window tactical spirit queries also admit a local exact path. If the spirit preview leaves one remaining mon move, update the touched-neighborhood one-move tactical contributions instead of rescanning the whole immediate-window helper surface.
- Do not keep a local budget-2 after-window tactical summary just because it collapses helper-call counts. If building that summary raises hotspot wall-clock and the duel still stalls, the real wall has moved into the remaining payload reachability work, not the raw query counter.
- Do not assume board-scoped actor-move memo reuse will clear the remaining wall. Sharing payload transition memos across immediate-window and budget-1 summary searches can reduce some repeated pickup and carrier transition work without moving the duel if the underlying payload reachability algorithm is still the bottleneck.
- Do not assume carrier-only memo shrink or a local small-budget carrier frontier will clear the remaining wall either. If `payload_calls` stay flat and hotspot wall-clock barely moves, the bottleneck is deeper than generic carrier memo initialization or queue setup.
- If you try board-scoped reverse carrier distance maps, keep them bounded and lazy. Building a reverse map on the first bounded carrier query is too broad and can regress wall-clock even while `payload_calls` drop; only build it after the board/budget pair proves it has multiple distinct carrier queries.

## Mistakes Not To Repeat

- Do not keep historical profiles live after their lesson is absorbed.
- Do not treat `target/experiment-runs` or `target/experiment-stamps` as durable memory.
- Do not reopen a branch just because it looked good in a bounded screen; reopen only with a new hypothesis.
- Do not spend ladder budget on branches that are still flat in the first real duel stage.
- Do not trust pooled or sampled losses as selector evidence when direct reliability tracing shows `disagreements=0`.
- Do not let Pro-specific selector flow bypass opening-book fallback ordering.
- Do not inject planner roots globally. Crisis-only gating is safer, and even then it must prove direct value against `runtime_current`.
- Do not keep sprawling wrapper split families alive once one retained frontier is clearly dominant.
- Do not use fixture additions or hotspot work as an excuse to avoid live Pro evidence.

## Current Durable Direction

- The current problem is not proving that Pro full-turn planning can ever be stronger. The retained code already shows that it can be.
- The current problem is finishing the earned path cleanly with one retained challenger under strict gates.
- The latest shared exact cuts moved the wall from planner/oracle summary construction into tactical projection itself.
- `runtime_pro_turn_engine_v30` is still not promotable after the current cuts because `pro-reliability` continues to stall in a practical window.
- The latest `exact_tactical_spirit_summary` cache-axis-sharing tweak only shaved the hotspot slightly; it did not materially reduce tactical projection call volume.
- The follow-up superset-cache cuts were worth keeping and moved `human_win_pro_a` again, from about `1425ms` down to about `1305ms`, but `pro-reliability` still stalled past `2:17`.
- The follow-up bounded carrier-window cut was also worth keeping: it collapsed `human_win_pro_a` payload work from about `32.2m` to about `6.3m`, but `pro-reliability` still stalled past `2:17`.
- The follow-up low-budget pickup-window, score-floor after-window, and incremental board-hash cut was also worth keeping: it removed most uncached pickup BFS from the hotspot and moved `human_win_pro_a` from about `570ms` to about `520ms`, but `pro-reliability` still stalled past `2:20` on 2026-03-27.
- The follow-up zero-move after-window tactical cut was also worth keeping: exact touched-item mana-pool updates moved `human_win_pro_a` from about `520ms` to about `433ms` and dropped immediate-window / after-window query volume from about `428999` / `427069` to about `168068` / `166138`, but `pro-reliability` still had to be killed at `3:37` on 2026-03-27.
- The follow-up budget-1 after-window tactical cut was also worth keeping: touched-neighborhood one-move updates dropped `human_win_pro_a` immediate-window / after-window query volume from about `168068` / `166138` to about `41740` / `39810` and moved the hotspot from about `433ms` to about `413ms`, but `pro-reliability` still had to be killed at `2:35` on 2026-03-27.
- A follow-up local budget-2 after-window summary was not worth keeping: it dropped `human_win_pro_a` immediate-window / after-window query volume from about `41740` / `39810` to about `7590` / `5660`, but the hotspot regressed from about `420ms` to about `452ms` and `pro-reliability` still had to be killed at `2:35` on 2026-03-27.
- A follow-up shared actor-move-memo cut was not worth keeping either: it only trimmed `human_win_pro_a` payload calls slightly and reduced pickup-window calls from about `2910` to about `2218`, but the hotspot regressed from about `420ms` to about `433ms` and `pro-reliability` still had to be killed at `2:34` on 2026-03-27.
- A follow-up carrier-specific move-memo plus small-budget carrier fast path was not worth keeping either: it left `human_win_pro_a` effectively flat at about `411.32ms` versus the retained `411.56ms` with `payload_calls` still about `5083818`, so shrinking generic carrier BFS setup cost did not move the real wall.
- A follow-up lazy bounded reverse carrier-distance map was worth keeping: once it only built after repeated bounded carrier queries on the same board/budget, `human_win_pro_a` moved from the retained `411.56ms` down to about `404.74ms` and then `398.73ms`, with `payload_calls` down to about `4922742`, but `pro-reliability` still had to be killed at about `2:45` on 2026-03-27.
- A local uncached after-window path that bypassed the retained exact caches was not worth keeping; it increased pickup work and did not beat the retained bounded-cache line.
- A score-floor threshold on after-window tactical queries can trim some off-target immediate-window work, but the live `human_win_pro_a` wall still emits roughly the same after-window query volume.
- Once the zero-move and budget-1 branches are removed, the remaining live wall is the payload reachability work that survives under the remaining tactical paths; further budget-count collapse alone is not enough.
- Next code should target the number of remaining after-window tactical queries or the spirit-preview fanout that creates them, not planner-oracle wrapper code or more per-query micro-optimizations alone.
- Keep Fast work parked until there is a genuinely new code path. Minor search-order retunes and scoring-only tweaks are already saturated.
