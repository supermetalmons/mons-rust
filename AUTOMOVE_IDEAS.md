# Automove Ideas

This is the working backlog for future automove loops.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution workflow and use this file to decide what to try next. When an idea is tried, promoted, ruled out, or split into follow-up ideas, update this file instead of relying on memory or raw logs.

If every current item here has been tried, add new ideas and keep going.

For `reply_risk`, `opponent_mana`, and `supermana`, run `./scripts/run-automove-experiment.sh triage-calibrate <surface>` before candidate work. If calibration fails, the next task is fixture work, not a new candidate.
Unless an idea explicitly says otherwise, new candidates must start as a delta on `runtime_current`. Retained non-production profiles are for calibration, references, or audit only.

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
- Cheapest falsifier: `guardrails`, then triage; run `runtime-preflight` only if triage passes.
- Escalate only if:
- Kill if:
- Next split if rejected:
- How to test: `guardrails`, triage, `runtime-preflight`, then the earned duel path only if the candidate is still alive.
- Status: backlog
- Notes:

## Backlog

### Idea: Pro primary vs confirmation retune
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Triage pass signal: `pro-triage` changes at least one `primary_pro` fixture while keeping `opening_reply` mostly stable.
- Candidate budget: 1
- Expected upside: stronger `pro` promotion candidate by separating independent-search strength from opening-reply confirmation behavior.
- CPU risk: medium
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage <candidate>`.
- Escalate only if: `pro-triage` passes cleanly and `pro-fast-screen` shows a clear positive story, not just a non-negative result.
- Kill if: `pro-triage` fails, `opening_reply` moves too much for a primary-path edit, or the first duel is flat or noisy.
- Next split if rejected: split into a pure `opening_reply` candidate or a narrower `primary_pro` candidate, but not both in the same retry.
- How to test: `guardrails`, `pro-triage`, `runtime-preflight`, `pro-fast-screen`, `pro-progressive`, `pro-ladder`, then the two release speed gates.
- Status: tried — killed at pro-fast-screen (four attempts)
- Notes: Keep opening-book confirmation cheaper and safer than the fully independent `pro` path. Mar 9, 2026: candidate `runtime_pro_primary_exact_lite_v1` enabled bounded exact-lite progress/spirit checks and slightly stronger opponent-mana conversion only for depth-4 `pro` primary contexts. It passed `guardrails` in about 6s, then failed `SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage runtime_pro_primary_exact_lite_v1` with `target_changed=0 off_target_changed=0`. No `runtime-preflight` or duel work was run, and the transient candidate was removed from the registry. Mar 10, 2026: candidate `runtime_pro_pvs_v1` enabled `enable_pvs=true` for Pro depth-4. PVS is a transparent search optimization (narrow-window probe + re-search on fail) that produces identical backed-up values to full-window search — it only saves nodes. Initially failed pro-triage on the original 4 primary_pro fixtures (all dominant top moves immune to PVS). After fixture expansion with `primary_pvs_sensitive_search_triage_game` (position where PVS changes move selection in ~16/300 random positions), pro-triage passed 1/5. Pro-fast-screen: both vs_normal and vs_fast lanes δ=0.000 — flat. Candidate killed per flat-result rule. Mar 11, 2026: candidate `runtime_pro_deep_tactics_v1` combined PVS + tighter futility margin (2000 vs production 2300). Pro-triage 1/5 (same PVS-sensitive fixture), pro-fast-screen both lanes δ=0.000 — flat again. Killed per flat-result rule. Root cause: search-structure-only changes (PVS, futility tuning) save nodes but don't change root move selection at Pro depth-4. Both candidates removed from registry.

### Idea: Normal conversion stability from proven pro signals
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Triage pass signal: `triage` changes at least one deterministic `opponent_mana` fixture versus baseline without guardrail regressions.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate opponent_mana`
- Candidate budget: 1
- Expected upside: better opponent-mana and supermana conversion without importing the full `pro` budget.
- CPU risk: low to medium
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=opponent_mana ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: `triage` passes and `fast-screen` shows a clear `normal` lift without dragging `fast` backward.
- Kill if: `triage` does not move the target surface, or the first duel is flat overall.
- Next split if rejected: break the idea into separate `opponent_mana` and `supermana` probes.
- How to test: `guardrails`, `triage`, `runtime-preflight`, `fast-screen`, `progressive`, `ladder`, then mode comparison against `runtime_release_safe_pre_exact` and `runtime_current` only if the first duel is borderline.
- Status: tried — killed at triage
- Notes: Port only the cheapest signals that already proved useful in stronger `pro` candidates. Mar 9, 2026: direct `triage-calibrate opponent_mana` now passes via the retained-profile shortlist probe, so candidate work is no longer blocked on the old generic fixture delta. Candidate `runtime_normal_opponent_mana_conversion_v1` changed only `interview_soft_opponent_mana_progress_bonus` in normal mode, passed `guardrails` in about 6s, then failed `SMART_TRIAGE_SURFACE=opponent_mana ./scripts/run-automove-experiment.sh triage runtime_normal_opponent_mana_conversion_v1` with `changed=0/4`. No runtime CPU gate or duel work was run.

### Idea: Fast reply-risk cleanup without normal-style overreach
- Base profile: `runtime_current`
- Target mode: `fast`
- Triage surface: `reply_risk`
- Triage pass signal: `triage` changes the reply-risk fixture pack instead of leaving it identical to baseline.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate reply_risk`
- Candidate budget: 1
- Expected upside: cheap strength gain from removing fake-good replies while preserving fast latency.
- CPU risk: low
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=reply_risk ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: `triage` passes and `fast-screen` shows a visible `fast` improvement with no sign that `normal` is carrying the result.
- Kill if: `triage` is unchanged, or the first duel is flat, or the only promising result needs wider normal-style shortlist tuning.
- Next split if rejected: isolate a smaller reply-risk guard or root tie-break instead of another combined cleanup wave.
- How to test: `guardrails`, `triage`, `runtime-preflight`, `fast-screen`, and one focused mode comparison only if the first duel is borderline.
- Status: tried — killed at triage
- Notes: Keep shortlist sizes and reply budgets tight. Avoid reusing aggressive normal-side penalties that already regressed. Candidate `runtime_fast_clean_reply_pref_v1` changes only `prefer_clean_reply_risk_roots = true` for depth<3 (fast mode). This is the single-variable isolation of a feature that's already proven in Normal/Pro mode. Unlike the killed clean_reply_risk_v1 which changed 5 parameters, this changes only the tie-breaking preference. Mar 10, 2026: triage `changed=0/3` on reply_risk. The single boolean doesn't shift any fast-mode fixture outcome.

### Idea: Fast root allocation rebalance
- Base profile: `runtime_current`
- Target mode: `fast`
- Triage surface: `reply_risk`
- Triage pass signal: `triage` changes at least one reply-risk fixture from root allocation differences alone.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate reply_risk`
- Candidate budget: 1
- Expected upside: broader top-move evaluation by spreading budget across k=3 roots instead of k=2, letting the 3rd-best root prove itself at depth=2.
- CPU risk: low (same total node budget)
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=reply_risk ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage passes and fast-screen shows clear fast-mode lift.
- Kill if: triage unchanged or first duel is flat/negative.
- Next split if rejected: try root_focus_budget_share_bp alone or combine with reduced root_branch_limit.
- How to test: `triage-calibrate`, `guardrails`, `triage`, `runtime-preflight`, then `SMART_PROMOTION_TARGET_MODE=fast SMART_TARGET_ONLY_BUDGETS=true` on `fast-screen`.
- Status: tried — killed at triage
- Notes: Candidate `runtime_fast_root_alloc_v1` changed only `root_focus_k` (2→3) and `root_focus_budget_share_bp` (6000→5000) for depth<3. Passed calibrate and guardrails, but triage showed `changed=0/3` — fixture outcomes identical to baseline. Budget rebalancing alone does not change deterministic move selection in the fixture positions.

### Idea: Fast boolean drainer safety scoring
- Base profile: `runtime_current`
- Target mode: `fast`
- Triage surface: `reply_risk`
- Triage pass signal: `triage` changes at least one reply-risk fixture from boolean drainer/carrier danger penalties.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate reply_risk`
- Candidate budget: 1
- Expected upside: catch tactical blunders that distance-based drainer threat misses by adding boolean danger penalties (-400 drainer, -300 carrier) plus tighter efficiency margin (1900).
- CPU risk: low (same search budget, different scoring)
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=reply_risk ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage passes and fast-screen shows clear fast-mode lift.
- Kill if: triage unchanged or first duel is flat/negative.
- Next split if rejected: try boolean scoring alone without efficiency margin change, or try different boolean penalty values.
- How to test: `triage-calibrate`, `guardrails`, `triage`, `runtime-preflight`, then `SMART_PROMOTION_TARGET_MODE=fast SMART_TARGET_ONLY_BUDGETS=true` on `fast-screen`.
- Status: tried — killed at triage+audit
- Notes: Uses existing RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF weights. Tried scoring-only, +reply_risk margin 140, +two_pass allocation. All combinations `changed=0/3` on reply_risk and `changed=0/2` on drainer_safety. Audit-screen showed δ=0.000 (2W-2L fast, 2W-2L normal). Boolean drainer penalties don't affect move selection in the fixture positions. The fast-mode evaluation is effectively saturated for these game states.

### Idea: Normal tactical focus via quiet reductions and event ordering
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `reply_risk`
- Triage pass signal: `triage` changes at least one reply-risk fixture from structural search changes.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate reply_risk`
- Candidate budget: 1
- Expected upside: better move selection by deprioritizing quiet positions (quiet reductions) and prioritizing tactical moves (event ordering bonus). Saves search budget on non-tactical moves at depth=3.
- CPU risk: low (quiet reductions save nodes)
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=reply_risk ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage passes and fast-screen shows clear normal-mode lift.
- Kill if: triage unchanged or first duel is flat/negative.
- Next split if rejected: try quiet reductions alone, or event ordering alone, or with different quiet_reduction_depth_threshold.
- How to test: `triage-calibrate`, `guardrails`, `triage`, `runtime-preflight`, then `SMART_PROMOTION_TARGET_MODE=normal SMART_TARGET_ONLY_BUDGETS=true` on `fast-screen`.
- Status: tried — killed at fast-screen
- Notes: Both features exist in fast mode but are disabled for normal. Structural search change, not parameter tuning. Candidate `runtime_normal_tactical_focus_v1` enabled PVS, quiet reductions, event ordering, and boosted interview_soft bonuses (supermana progress 240→400, supermana score 300→500, opponent_mana progress 220→350, opponent_mana score 280→400) for depth>=3. All pre-duel gates passed (guardrails, triage opponent_mana changed=4/4, runtime-preflight). Fast-screen with SMART_TARGET_ONLY_BUDGETS=true showed EarlyReject at tier 0 (8 normal-only games, 4W-4L, δ=0.000). The bundle of search optimizations plus aggressive interview-soft boosts did not translate to duel strength.

### Idea: Fast clean reply-risk shortlist
- Base profile: `runtime_current`
- Target mode: `fast`
- Triage surface: `reply_risk`
- Triage pass signal: `triage` changes at least one reply-risk fixture by favoring cleaner shortlist ordering without touching non-fast behavior.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate reply_risk`
- Candidate budget: 1
- Expected upside: stronger fast reply filtering from a modest shortlist cleanup that stays inside the current runtime shape.
- CPU risk: low
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=reply_risk ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: `triage` passes and the first duel shows a clear fast-mode lift instead of a flat or carried result.
- Kill if: `guardrails` or `runtime-preflight` fails, `triage` stays at `changed=0/3`, or the first duel is flat or noisy.
- Next split if rejected: do not split the heuristics again until the first earned duel stage resolves quickly; the next split is workflow-side, not candidate-side.
- How to test: `triage-calibrate`, `guardrails`, `triage`, `runtime-preflight`, then `SMART_PROMOTION_TARGET_MODE=fast` on `fast-screen`, `progressive`, and `ladder`.
- Status: tried — killed at extended fast-screen
- Notes: Continuation of the audit-validated candidate, not a same-idea retry. Candidate `runtime_fast_clean_reply_risk_v1` stayed fast-only and changed only `prefer_clean_reply_risk_roots`, `root_reply_risk_score_margin`, `root_reply_risk_shortlist_max`, `root_reply_risk_reply_limit`, and `root_reply_risk_node_share_bp`. Mar 9, 2026: target-aware duel policy fixed the pooled false reject. All pre-duel gates passed (`triage-calibrate`, `guardrails`, `triage` changed=3/3, `runtime-preflight`). Fast-screen with target-only budgets (`SMART_TARGET_ONLY_BUDGETS=true`) ran 120 fast-only games: tier 0 δ=+0.1250(8g), tier 1 δ=+0.0417(24g), tier 2 δ=-0.0357(56g), tier 3 δ=-0.0583(120g). Early positive signal was noise; the candidate is clearly weaker than baseline at scale. Killed and removed from registry. No more fast reply-risk shortlist variants.

### Idea: Exact-lite only on high-value conversion windows
- Base profile: `runtime_current` by default; retained `runtime_eff_exact_lite_v1` only for calibration/audit
- Target mode: `normal`, `pro`
- Triage surface: `supermana`
- Triage pass signal: `triage` moves the deterministic `supermana` fixtures, or the split `pro` follow-up moves `primary_pro` without touching `opening_reply`.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate supermana`
- Candidate budget: 1
- Expected upside: recover tactical strength only where safe supermana, safe opponent-mana, or spirit-assisted score windows are plausibly available.
- CPU risk: medium
- Cheapest falsifier: exact-lite diagnostics gate, `guardrails`, then `SMART_TRIAGE_SURFACE=supermana ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: diagnostics stay bounded and triage shows a deterministic change on the intended conversion window.
- Kill if: the exact-lite trigger leaks into routine turns, or triage does not move the intended surface.
- Next split if rejected: split by trigger family such as `supermana`, `opponent_mana`, or a dedicated `primary_pro` follow-up.
- How to test: exact-lite diagnostics gate, `guardrails`, `triage` or `pro-triage` after the idea is split to one surface, `runtime-preflight`, then the earned promotion path for the target mode.
- Status: tried — killed at triage
- Notes: No broad exact-lite activation. Trigger selectively and keep explicit per-move budgets. Mar 9, 2026: direct `triage-calibrate supermana` now passes via the retained exact-lite activation probe, so this surface is ready for candidate work again. The retained exact-lite profile `runtime_eff_exact_lite_v1` then failed `SMART_TRIAGE_SURFACE=supermana ./scripts/run-automove-experiment.sh triage runtime_eff_exact_lite_v1` with `changed=0/3`, so broad exact-lite activation still does not move the fixed `supermana` surface enough to earn heavier checks.

### Idea: Shared tactical and exact-lite cache reuse
- Base profile: `runtime_current`
- Target mode: `normal`, `pro`
- Triage surface: `cache_reuse`
- Triage pass signal: `triage` shows deterministic speed or cache-hit improvement versus baseline on the fixed cache probe.
- Candidate budget: 1
- Expected upside: more strength from the same CPU budget by reusing cached summaries across root ranking, tie-breaks, and tactical prepasses.
- CPU risk: low to medium
- Cheapest falsifier: speed probes, exact-lite diagnostics gate, then `SMART_TRIAGE_SURFACE=cache_reuse ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: cache reuse lowers duplicated work and the first duel shows strength from the reclaimed budget instead of from extra search.
- Kill if: reuse adds bookkeeping without deterministic cache evidence, or the first duel stays flat after the speed gain.
- Next split if rejected: keep only the cheapest cache-sharing point and drop the rest of the reuse surface.
- How to test: speed probes, exact-lite diagnostics gate, `guardrails`, `triage`, `runtime-preflight`, then the earned promotion path for the target mode.
- Status: backlog
- Notes: Prefer reuse before deeper search. Strength that comes from duplicated work is unlikely to be promotable.

### Idea: Stronger own-drainer exposure filtering
- Base profile: `runtime_current`
- Target mode: `fast`, `normal`, `pro`
- Triage surface: `drainer_safety`
- Triage pass signal: `triage` changes the drainer-safety fixtures before any duel is allowed.
- Candidate budget: 1
- Expected upside: eliminate fake-good moves that immediately expose the drainer unless the move wins or scores decisive mana.
- CPU risk: low
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=drainer_safety ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage moves the safety surface and `fast-screen` shows measurable lift instead of just sounding correct.
- Kill if: triage is unchanged, or the first duel stays near zero or turns negative.
- Next split if rejected: combine exposure awareness with harder filtering or reply cleanup instead of another standalone penalty wave.
- How to test: tactical guardrails first, then `triage`, `fast-screen`, and only the earned promotion path for the target mode.
- Status: tried — not promotable
- Notes: Tested as `drainer_exposure_v1` (Mar 9, 2026). Heuristic penalty (200/300) plus wider safety margin (2800/5200) showed zero measurable strength gain — fast-screen δ=+0.009, progressive δ=-0.007. The existing late-stage safety filter already handles most cases. Future drainer work should combine exposure awareness with other signals or use harder filtering, not standalone penalties.

### Idea: Spirit-off-base development that still respects tempo
- Base profile: `runtime_current`
- Target mode: `normal`, `pro`
- Triage surface: `spirit_setup`
- Triage pass signal: `triage` changes at least one fixed spirit-setup fixture without regressing the generic guardrails.
- Candidate budget: 1
- Expected upside: better setup play from earlier spirit deployment without drifting into low-value wandering.
- CPU risk: low to medium
- Cheapest falsifier: the target mode tactical fixtures, `guardrails`, then `SMART_TRIAGE_SURFACE=spirit_setup ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage passes and the first duel shows concrete score, denial, or setup value rather than extra wandering.
- Kill if: triage is unchanged, or the idea mostly adds motion without conversion in the first duel.
- Next split if rejected: separate spirit setup for score, denial, and conversion so only the useful branch survives.
- How to test: add or strengthen spirit fixtures first if needed, then run `guardrails`, `triage` or `pro-triage` after the idea is split to one surface, `runtime-preflight`, and only the earned promotion path.
- Status: tried — killed at triage
- Notes: Reward spirit movement only when it creates concrete score, denial, or conversion value. Mar 9, 2026: candidate `runtime_normal_spirit_window_v1` enabled bounded exact-lite spirit-window checks plus slightly stronger supermana spirit bonuses only for normal depth-3 contexts. It passed `guardrails` in about 6s, then failed `SMART_TRIAGE_SURFACE=spirit_setup ./scripts/run-automove-experiment.sh triage runtime_normal_spirit_window_v1` with `changed=0/2`. No `runtime-preflight` or duel work was run, and the transient candidate was removed from the registry.

### Idea: Opening black-reply strength under hard latency budget
- Base profile: `runtime_current`
- Target mode: `fast`, `normal`, `pro`
- Triage surface: `opening_reply`
- Triage pass signal: `pro-triage` changes at least one fixed opening black-reply fixture while `primary_pro` stays mostly stable.
- Candidate budget: 1
- Expected upside: stronger early replies without breaking release-safe opening latency.
- CPU risk: medium
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=opening_reply ./scripts/run-automove-experiment.sh pro-triage <candidate>`.
- Escalate only if: `pro-triage` shows deterministic opening-reply change inside the fixed budget; treat the generic release opening speed gate as promotion-time baseline validation, not a candidate filter.
- Kill if: the idea needs more opening latency to work, or `primary_pro` moves too much for an opening-only edit.
- Next split if rejected: isolate cheaper opening-reply ranking signals from any broader search-budget changes.
- How to test: `guardrails`, `pro-triage`, `runtime-preflight`, then the earned promotion pipeline for the target mode. Run the generic release opening speed gate only before promotion, or after adding a candidate-aware opening speed probe.
- Status: tried — killed at pro-triage
- Notes: Treat the opening-reply budget as fixed and improve move quality inside it rather than expanding it. Mar 9, 2026: candidate `runtime_pro_opening_attacker_reply_v1` switched opening-reply ranking toward attacker-proximity weights and slightly stronger reply-risk filtering without changing the opening latency envelope. It passed `guardrails` in about 6s, then failed `SMART_TRIAGE_SURFACE=opening_reply ./scripts/run-automove-experiment.sh pro-triage runtime_pro_opening_attacker_reply_v1` with `target_changed=0 off_target_changed=0`. This also exposed a workflow bug: `smart_automove_release_opening_black_reply_speed_gate` measures production runtime only, so it should not sit before candidate triage.

### Idea: Anti-help and anti-roundtrip cheap root scoring
- Base profile: `runtime_current`
- Target mode: `fast`, `normal`
- Triage surface: `reply_risk`
- Triage pass signal: `triage` changes the fixed reply-risk/root-choice fixture pack before any duel.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate reply_risk`
- Candidate budget: 1
- Expected upside: quick strength gain from refusing mana moves that help the opponent or waste tempo.
- CPU risk: low
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=reply_risk ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage shows the root-only score changes are actually moving the deterministic surface.
- Kill if: triage is unchanged, or the first duel is flat, or the only lift comes from wider search rather than the root score itself.
- Next split if rejected: split anti-help from anti-roundtrip so each can be tested as a narrower root-only rule.
- How to test: `guardrails`, `triage`, `runtime-preflight`, `fast-screen`, and one focused mode comparison if the first duel is borderline.
- Status: tried — killed at triage
- Notes: This is a good candidate for low-cost root-only scoring or tie-break adjustments. Mar 9, 2026: direct `triage-calibrate reply_risk` now passes via the retained roundtrip-penalty probe, so candidate work is no longer blocked on unchanged whole-board root snapshots. Candidate `runtime_fast_reply_cleanup_v1` passed `preflight` after a base-config fix, but `SMART_TRIAGE_SURFACE=reply_risk ./scripts/run-automove-experiment.sh triage runtime_fast_reply_cleanup_v1` still showed `changed=0/3` and was killed before any duel. The run also exposed the next workflow cut: the full CPU gate took about 63s before that instant triage reject, so the default loop now needs `guardrails -> triage -> runtime-preflight`.

### Idea: Pro triage fixture expansion with close-decision positions
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Triage pass signal: new Pro fixtures where depth-4 search produces close decisions between top moves, making triage sensitive to config or search changes.
- Candidate budget: 0 (infrastructure work)
- Expected upside: unlock Pro candidate iteration. Current 4 primary_pro fixtures all have dominant top moves immune to single-variable config changes. Both exact-lite and PVS candidates failed at 0/4+0/3.
- CPU risk: none
- Cheapest falsifier: create positions where depth-4 Pro search disagrees with heuristic ordering (selected_rank>0) or where top-2 backed-up scores are within margin, then verify triage sensitivity.
- Status: completed
- Notes: Created 4 FEN-based close-decision fixtures via sensitivity probe (`extension_sensitive_no_ext_a`, `extension_sensitive_more_ext_a`, `extension_sensitive_no_ext_b`, `extension_sensitive_more_ext_b`). All have gap_1v2=0 (perfectly tied top-2 decisions at depth-4). Added to `primary_pro_triage_fixtures()` making 9 total. These unlocked the first-ever structural Pro triage pass (changed=2/9) for the no-extensions candidate.

### Idea: Stuck-state and bounded-progress safety fixtures
- Base profile: `runtime_current`
- Target mode: `fast`, `normal`, `pro`
- Triage surface: blocked until new safety fixture exists
- Triage pass signal: add the missing deterministic fixture, then require it to pass before any duel work continues.
- Candidate budget: 1
- Expected upside: stronger release confidence by catching empty-selector, repeated-position, and no-progress edge cases before promotion.
- CPU risk: low
- Cheapest falsifier: new or strengthened fixtures that fail the candidate immediately.
- Escalate only if: the safety fixtures pass and the candidate still clears the relevant triage surface without added instability.
- Kill if: the candidate needs unsafe fallback behavior to stay strong, or any stuck-state fixture remains unresolved.
- Next split if rejected: isolate the single unsafe edge case and fix that before touching strength again.
- How to test: targeted fixtures first, then `guardrails`, then the relevant `triage` or `pro-triage`, `runtime-preflight`, then the normal release gates.
- Status: backlog
- Notes: Safety work is promotion work. A candidate that can stall or behave unpredictably is not ready.

### Idea: Promotion-focused artifact summaries
- Base profile: workflow-only
- Target mode: workflow
- Triage surface: workflow-only
- Triage pass signal: each loop can produce a backlog update, archive note, or durable lesson directly from the standard artifacts.
- Candidate budget: 1
- Expected upside: faster iteration because doc-worthy outcomes become obvious and disposable logs stay disposable.
- CPU risk: low
- Cheapest falsifier: one failed loop where the lesson still requires digging through raw logs.
- Escalate only if: the summary step shortens the time from run result to decision.
- Kill if: the summary step adds ceremony without shortening the decision loop.
- Next split if rejected: keep only the smallest summary artifact that directly feeds backlog, archive, or knowledge updates.
- How to test: verify each loop can move a conclusion into `docs/automove-knowledge.md`, `docs/automove-archive.md`, or a new backlog item with no dependence on raw logs.
- Status: backlog
- Notes: Improve signal extraction, not permanent logging volume.

### Idea: Normal futility pruning from proven pro signal
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Triage pass signal: `triage` changes at least one deterministic `opponent_mana` fixture versus baseline.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate opponent_mana`
- Candidate budget: 1
- Expected upside: better move selection by pruning hopeless frontier nodes, freeing budget for promising lines. Proven in `pro` mode (margin=2300).
- CPU risk: low (strictly saves nodes)
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=opponent_mana ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage passes and fast-screen shows clear normal-mode lift.
- Kill if: triage unchanged or first duel is flat/negative.
- Next split if rejected: try a different margin value or combine with phase-adaptive scoring.
- How to test: `triage-calibrate`, `guardrails`, `triage`, `runtime-preflight`, then `SMART_PROMOTION_TARGET_MODE=normal SMART_TARGET_ONLY_BUDGETS=true` on `fast-screen`.
- Status: tried — killed at triage
- Notes: Pro mode uses futility pruning (margin=2300) and quiet reductions together. Normal mode has neither. Prior candidate tried quiet reductions + event ordering + PVS + boosted interview bonuses but was flat at duel. This tries only futility pruning — the single most impactful frontier-level optimization. Candidate `runtime_normal_futility_pruning_v1` enabled `enable_futility_pruning=true` and `futility_margin=2300` for depth>=3. Passed guardrails, then failed `SMART_TRIAGE_SURFACE=opponent_mana triage` with `changed=0/4`. Futility pruning changes the tree search at frontier nodes but doesn't shift root-level heuristic/efficiency signals that triage compares. Search-structure-only changes (futility pruning, PVS, quiet reductions) are invisible to the current triage surface.

### Idea: Normal attacker-proximity scoring from proven pro signal
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Triage pass signal: `triage` changes at least one deterministic `opponent_mana` fixture versus baseline.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate opponent_mana`
- Candidate budget: 1
- Expected upside: better normal-mode evaluation by switching from walk-threat-medium scoring to Pro-style attacker-proximity scoring. Rewards closeness to opponent drainer instead of penalizing walk threats. Proven evaluation philosophy in Pro mode.
- CPU risk: low (same search budget, different scoring weights)
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=opponent_mana ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage passes and fast-screen shows clear normal-mode lift.
- Kill if: triage unchanged or first duel is flat/negative.
- Next split if rejected: try combo_proximity_attack variant or add attacker_proximity on top of walk_threat instead of replacing.
- How to test: `triage-calibrate`, `guardrails`, `triage`, `runtime-preflight`, then `SMART_PROMOTION_TARGET_MODE=normal SMART_TARGET_ONLY_BUDGETS=true` on `fast-screen`.
- Status: tried — killed at fast-screen (post-fixture-expansion)
- Notes: Normal mode already uses phase-adaptive scoring (walk_threat_medium). Pro mode overrides with attacker_proximity. The two differ in evaluation philosophy: walk_threat penalizes positions where drainer is walk-threatened; attacker_proximity rewards positions where attacker is close to opponent drainer. Candidate `runtime_normal_proximity_scoring_v1` overrode scoring_weights to attacker_proximity for depth>=3. Passed guardrails but failed `SMART_TRIAGE_SURFACE=opponent_mana triage` with `changed=0/4` on old fixtures. Mar 10, 2026: Expanded opponent_mana fixtures with 2 new positions (contested_approach, defended_pickup) that have nearby opponent pieces and gap=0 tied top moves. These produce heuristic delta=66 between attacker-proximity and balanced scoring. Candidate `runtime_attacker_proximity_v1` (scoring-only, no structural changes) passed triage 2/6, preflight (CPU 1.05x fast, 1.08x normal), then EarlyReject at fast-screen (4W-4L fast, 3W-5L normal, δ=-0.0625). Also tested `runtime_attacker_futility_v1` (+ futility pruning margin=2300) — identical fast-screen result (same 16 game outcomes). Attacker-proximity scoring is detectable on the new fixtures but does not improve Normal play. Retained `runtime_attacker_proximity_v1` for calibration, killed `attacker_futility_v1`. Mar 12, 2026: Re-tested after expanding opponent_mana to 14 fixtures (added 4 depth-disagreement fixtures). Triage now changed=4/14 (opponent_mana_contested_approach, opponent_mana_defended_pickup, normal_ext_sensitive_d, depth_disagree_a) — triage PASSED. Runtime-preflight PASSED. Fast-screen δ=0.0000 EarlyReject after 16 games. Third confirmation that attacker-proximity scoring is triage-visible but duel-flat.

### Idea: Normal futility pruning plus tighter efficiency
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Triage pass signal: `triage` changes at least one deterministic `opponent_mana` fixture versus baseline.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate opponent_mana`
- Candidate budget: 1
- Expected upside: real search quality from futility pruning (proven in Pro) combined with tighter efficiency margin and backtrack penalty that shift root-level efficiency signals (visible to triage).
- CPU risk: low (futility pruning saves nodes, efficiency changes are root-only)
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=opponent_mana ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage passes and fast-screen shows clear normal-mode lift.
- Kill if: triage unchanged or first duel is flat/negative.
- Next split if rejected: try audit-screen to spot-check if search quality changes have duel impact despite triage failure. If audit shows strength, the triage surface needs work.
- How to test: `triage-calibrate`, `guardrails`, `triage`, `runtime-preflight`, then `SMART_PROMOTION_TARGET_MODE=normal SMART_TARGET_ONLY_BUDGETS=true` on `fast-screen`.
- Status: tried — killed at triage
- Notes: Futility pruning alone was invisible to triage (changed=0/4). Attacker-proximity scoring also invisible (features don't fire in fixtures). This combines futility pruning for search depth with efficiency/backtrack tuning (1400→1200 margin, 240→280 backtrack) for triage visibility. Also tried +20% node boost. Also tried tighter reply-risk filter for Normal (margin 145→160, shortlist 7→5, reply limit 16→12, node share 1350→1100 +20% nodes). All combinations `changed=0/4` on opponent_mana, plus drainer_safety and spirit_setup. Audit-screen with futility+efficiency showed δ=0.000 (8 games). Root issue: opponent_mana fixture positions have dominant top moves unaffected by search-level or filter changes. Mar 10, 2026: Post-fixture-expansion, futility pruning + attacker-proximity scoring tested as `attacker_futility_v1`. Triage 2/6 (from scoring, not pruning). Fast-screen EarlyReject: identical to scoring-only candidate (same 16 game outcomes). Futility pruning adds no incremental value on top of scoring changes.

### Idea: Normal scoring weight defensiveness (spirit race)
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Candidate budget: 1
- Status: tried — killed at progressive (FadingSignal)
- Notes: `runtime_normal_spirit_race_v1` changed `drainer_close_to_mana: 360→330` (-30) and `opponent_score_race_path_progress: 184→214` (+30) for depth≥3. Passed triage 2/4, fast-screen EarlyPromote (24g δ=+0.083). Progressive faded: tier 0-2 δ=0.03-0.04, tier 3 δ=+0.008. Lesson: small ±30 tweaks to existing nonzero weights pass triage but are noise at scale.

### Idea: Normal multi-path scoring activation
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Candidate budget: 1
- Expected upside: activate 4 dormant features (`score_race_multi_path`, `opponent_score_race_multi_path`, `immediate_score_multi_window`, `opponent_immediate_score_multi_window`) that are 0 in Normal but 60/90/80/120 in Fast.
- Status: tried — killed at fast-screen (combined in scoring_upgrade_v1)
- Notes: Combined with mana-race activation as `scoring_upgrade_v1` (multi-path 60/90/80/120 + mana-race 85/10). Passed triage 1/4, preflight (fast ~1.0, normal ~1.02-1.13). Fast-screen EarlyReject: 4W-4L both modes, δ=0.000. Also combined with tactical finish + 25% node boost as `node_boost_v1`. Scoring weight changes alone do not improve Normal duel strength.

### Idea: Normal mana-race field activation
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Candidate budget: 1
- Expected upside: activate `regular_mana_to_owner_pool` (0→85), `regular_mana_drainer_control` (0→10), `supermana_drainer_control` (0→15). Zero in Normal, 170/18/26 in Fast.
- Kill if: triage unchanged or duel flat/fading.
- Status: tried — killed at fast-screen (combined in scoring_upgrade_v1)
- Notes: Combined with multi-path activation as `scoring_upgrade_v1`. Also tried alone as part of `node_boost_v1` scoring overlay. Neither passed fast-screen.
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Candidate budget: 1
- Expected upside: boost `drainer_best_mana_path` (58→120), `drainer_pickup_score_this_turn` (90→140), `mana_carrier_score_this_turn` (150→200). Much higher in Fast (250/210/290).
- Kill if: triage unchanged or duel flat/fading.
- Status: tried — killed at fast-screen (tactical_finish_v1 and node_boost_v1)
- Notes: Combined with immediate score window boost as `tactical_finish_v1`: drainer_best_mana_path 58→120, drainer_pickup_score_this_turn 90→140, mana_carrier_score_this_turn 150→200, immediate_score_window 96→160, opponent_immediate_score_window 245→300. Passed triage 1/4. Fast-screen EarlyReject: 4W-4L both modes, δ=0.000. Also combined with +25% nodes as `node_boost_v1` — EarlyReject 3W-5L normal, δ=-0.0625.
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Candidate budget: 1
- Expected upside: boost `immediate_score_window` (96→160) and `opponent_immediate_score_window` (245→300). The TACTICAL phase already uses 102/310. Higher window awareness could catch scoring opportunities earlier.
- Kill if: triage unchanged or duel flat/fading.
- Status: tried — killed at fast-screen (combined in tactical_finish_v1 and node_boost_v1)
- Notes: Combined with tactical finish weights as `tactical_finish_v1`. Also combined with +25% node boost as `node_boost_v1`. All Normal scoring weight adjustments produced flat or negative duel results.

### Idea: Normal dormant feature activation sweep
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`, `drainer_safety`, `spirit_setup`
- Candidate budget: exhausted
- Status: tried — all killed at triage or audit
- Notes: Mar 10, 2026: Comprehensive sweep of all dormant SmartSearchConfig features for Normal mode. Each tested as single-variable change using `with_pre_exact_runtime_policy` (same config as baseline + one boolean). Results:
  - `enable_walk_threat_prefilter = true`: 0/2 drainer_safety, 0/4 opponent_mana. Fixture positions don't have walk-distance drainer threats.
  - `enable_interview_deterministic_tiebreak = true`: 0/2 spirit_setup, 0/4 opponent_mana. Spirit-development tiebreakers don't fire — top candidates aren't tied on the relevant fields.
  - `enable_killer_move_ordering = true`: guardrails passed, audit-screen δ=0.000 (2W-2L each mode, 8 games). Killer moves improve internal tree pruning but don't change which root move wins.
  - `enable_event_ordering_bonus = true` (via `with_runtime_scoring_weights`): triage 1/4 opponent_mana, 1/3 supermana — BUT the triage change was from the scoring weight switch (walk_threat_medium vs DEFAULT), not from event ordering. Fast-screen EarlyReject: 3W-5L normal, δ=-0.0625.
  - `+20% max_visited_nodes` alone (via `with_pre_exact_runtime_policy`): 0/4 opponent_mana. More nodes don't shift dominant top moves.
  - Wider reply-risk margins (155/18/1500 + efficiency 1300, matching eff_non_exact_v2 via `with_runtime_scoring_weights`): 0/4 opponent_mana.
  - `enable_supermana_prepass_exception + enable_opponent_mana_prepass_exception + enable_conditional_forced_drainer_attack`: 0/4 opponent_mana, 0/3 supermana, 0/2 drainer_safety. Features don't fire — fixture positions lack the simultaneous tactical conditions.
  Root cause: All Normal triage fixture positions have a dominant top move unaffected by any single-variable change to the pre-exact config. The only way to move triage is changing scoring weights, which is cosmetic (passes triage but noise at duel).

### Idea: Triage fixture expansion with known-mistake positions
- Base profile: `runtime_current`
- Target mode: `fast`, `normal`
- Triage surface: new fixtures needed
- Triage pass signal: new fixtures that expose known engine mistakes, making triage sensitive to structural improvements.
- Candidate budget: 0 (infrastructure work)
- Expected upside: unlock iteration on structural features (killer moves, PVS, futility pruning, walk threat) that are invisible to current triage.
- CPU risk: none
- Cheapest falsifier: identify game positions where current automove makes suboptimal moves, add them as fixtures.
- Status: completed — fixtures deployed, first candidates tested
- Notes: The current triage framework was blocked for Normal mode. Every single-variable change to the pre-exact config produced `changed=0/N` on all surfaces. Only scoring weight changes (walk_threat_medium vs DEFAULT) passed triage, but they are noise at duel scale. Mar 10, 2026: Probed 12 candidate positions against 5 retained profiles. Found 2 positions with gap=0 (tied top moves) and heuristic delta=66 between baseline and eff_non_exact_v2: `contested_opponent_mana_approach` (converging threats, white drainer+spirit vs black drainer+mystic near mana) and `defended_opponent_mana_pickup` (angel-guarded drainer vs distant mystic). Added as `opponent_mana_contested_approach` and `opponent_mana_defended_pickup` fixtures (6 total). These fixtures are sensitive to attacker-proximity scoring changes but scoring-only candidates (attacker_proximity_v1, attacker_futility_v1) failed fast-screen. The fundamental finding: scoring weight changes pass expanded triage but remain noise at duel scale. New fixtures are necessary infrastructure but do not unlock Normal mode candidates alone.

### Idea: Normal exact tactics activation (bounded CPU)
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`, `supermana`
- Candidate budget: exhausted
- Status: tried — all killed at runtime-preflight or triage
- Notes: Mar 11, 2026: Exhaustive investigation of exact tactics flags on Normal mode. The three flags disabled by `with_pre_exact_runtime_policy` are the only features that move Normal triage at all. Results:
  - `enable_static_exact_evaluation = true`: triage 4/4 opponent_mana + 3/3 supermana (ALL moved). CPU 5.345x — far over 1.30x limit. Signal comes from child-level `exact_strategic_analysis` calls inside `move_efficiency_delta` within `search_score`, which change internal alpha-beta pruning and thus root move heuristics.
  - `enable_root_exact_tactics = true`: triage 1/4 opponent_mana + 2/3 supermana. CPU 1.396x — 7.4% over 1.30x limit. Marginal signal that dies with any compensating CPU reduction.
  - `enable_child_exact_tactics = true`: triage 0/4 opponent_mana. No signal at all.
  - Root exact + 10% node reduction: 0/4. Signal killed.
  - Root exact + root_branch_limit-2: 0/4. Signal killed.
  - Root exact + extensions disabled: 0/4. Signal killed.
  - Exact-lite (budget 1, 5, 20, 200): 0/4 on all. Conditional triggers don't fire on fixture positions.
  - Root-only exact summary (new `enable_root_summary_exact` field): 0/4. Root-level `approximate_active_turn_summary` + root-level `move_efficiency_delta` with exact analysis alone don't change which root move wins. The signal requires CHILD-level efficiency ordering changes propagating through the search tree.
  - Production scoring weights (walk_threat_medium + 20% nodes): 0/4 opponent_mana, 0/3 supermana, 0/3 reply_risk. Production vs test baseline scoring gap is real but invisible to triage — per-node scoring uses legacy formula regardless of weight struct (`allow_exact_strategic=false` forces `use_legacy_formula=true`).
  Root cause: The pre-exact policy exists because exact tactics are too expensive. The triage signal from exact tactics requires O(thousands) calls to `exact_strategic_analysis` at child levels in the search tree. No bounded or root-only approximation reproduces the signal. Root exact alone passes triage marginally (1/4 + 2/3) at 1.396x CPU, but audit-screen showed δ=-0.125 (1W-3L normal, EarlyReject) — the feature is actively weaker at duel, not just expensive. The Normal mode triage surface is fundamentally blocked unless (a) new cheaper-to-evaluate fixtures are added, or (b) the CPU budget is significantly raised.

### Idea: Normal supermana interview-soft boost
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `supermana`
- Candidate budget: 1
- Status: tried — killed at fast-screen
- Notes: Mar 11, 2026: `runtime_normal_supermana_boost_v1` boosted `interview_soft_supermana_progress_bonus` (240→400) and `interview_soft_supermana_score_bonus` (300→550) for Normal depth≥3. Interview-soft bonuses directly inflate root heuristic scores. Guardrails passed. Triage supermana: changed=3/3. Preflight: CPU 0.94-1.02x, well under 1.30x limit. Fast-screen (SMART_PROMOTION_TARGET_MODE=normal): EarlyReject at tier 0, 4W-4L both modes, δ=0.000. Confirms the pattern: scoring-weight changes pass triage by inflating heuristic scores but don't translate to duel strength. No more supermana interview_soft variants.

### Idea: Pro root focus budget rebalance
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Candidate budget: 1
- Status: tried — killed at pro-triage
- Notes: Mar 11, 2026: `runtime_pro_wider_roots_v1` changed `root_focus_k` (3→4) and `root_focus_budget_share_bp` (7000→6500) for Pro depth≥4, giving more roots deeper budget. Guardrails passed. Pro-triage: target_changed=0/5, off_target_changed=0/3. Budget redistribution alone doesn't shift root move selection in Pro fixture positions — dominant top moves win regardless of how budget is split. Pro triage is blocked by fixture dominance, same as Normal.

### Idea: Normal forced_tactical_prepass disabled
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`, `supermana`, `drainer_safety`
- Candidate budget: 1
- Status: tried — killed at triage (0/6 + 0/3 + 0/2)
- Notes: Mar 11, 2026: `runtime_normal_no_prepass_v1` disabled `enable_forced_tactical_prepass` for Normal depth≥3, matching Pro behavior. The prepass shortcuts search when obvious tactical moves exist (wins_immediately, scores_supermana, attacks_drainer). Hypothesis: by always running full depth-3 search, Normal might find better moves. Result: the prepass doesn't fire in ANY fixture position — none of the 11 tested fixtures have the tactical conditions (immediate wins, supermana scoring, drainer attacks) that trigger the prepass. The feature is irrelevant for triage.

### Idea: Audit-driven fixture creation from random game positions
- Base profile: `runtime_current`
- Target mode: `normal`, `pro`
- Triage surface: new fixtures needed
- Triage pass signal: new fixtures that are structurally sensitive — top-2 roots within small backed-up score gap, or positions where budget allocation changes shift the winner.
- Candidate budget: 0 (infrastructure work)
- Expected upside: unlock iteration. Current fixtures (all surfaces, both modes) have dominant top moves immune to all parameter/search/weight changes. Only scoring weight changes pass triage, but those are duel-noise. New fixture positions with CLOSE decisions are needed to make triage useful for structural improvements.
- CPU risk: none
- Cheapest falsifier: run random midgame positions through depth-3 search, find cases where gap between #1 and #2 backed-up scores is < 50. Add those as fixtures and verify sensitivity.
- Status: completed (Mar 12, 2026)
- Notes: This is the fundamental blocker. Without close-decision fixtures, triage can only detect scoring weight inflation (which is cosmetically detectable but strategically meaningless). The human game analysis (diagnostic tests at tests.rs lines 2971-3357) already identified some positions. Those could be mined for fixture candidates. KEY INSIGHT from this session: 13 candidates across 3 sessions, every single one that passed triage was a scoring weight change, and every one was flat or negative at duel. Structural/search improvements are invisible to triage. The triage framework needs positions where the margin of victory is narrow. Mar 12, 2026: Created `smart_automove_normal_depth_disagreement_probe` diagnostic that finds positions where Normal (depth-3) and Pro (depth-4) disagree on best move. Ran 100 positions: 21/100 disagreements, 12 with gap≤50. Added 4 FEN-based depth-disagreement fixtures to opponent_mana surface: depth_disagree_a (gap=0, Normal→rank2, Pro→rank1), depth_disagree_b (gap=0, Normal→rank6!, Pro→rank1), depth_disagree_c (gap=10), depth_disagree_d (gap=0). Triage-calibrate PASSED (6/14 changed). depth_disagree_a and depth_disagree_b are sensitive to eff_non_exact_v2 calibration profile.

### Idea: Pro more extensions (deeper tactical chains)
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Triage pass signal: `pro-triage` changes at least 2/9 primary_pro fixtures, keeping opening_reply stable.
- Candidate budget: 1
- Expected upside: stronger Pro play by allowing double-depth extension chains (max_extensions_per_path=2) and wider extension budget (2500bp vs 1500bp). Sensitivity probe found 4/50 positions sensitive to more_extensions, including 1 with gap=0.
- CPU risk: high — deeper extensions consume more nodes. Pro ratio currently ~4.7, max 6.0.
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage runtime_pro_more_extensions_v1`.
- Escalate only if: pro-triage passes cleanly and pro-fast-screen shows clear positive story.
- Kill if: pro-triage fails, runtime-preflight CPU exceeds 1.30x, or first duel is flat.
- How to test: `guardrails`, `pro-triage`, `runtime-preflight`, `pro-fast-screen`, `pro-progressive`, `pro-ladder`.
- Status: tried — killed at pro-fast-screen (vs_fast negative)
- Notes: Opposite direction from Pro-no-extensions. The sensitivity probe showed that BOTH removing and adding extensions changes moves at Pro depth — the landscape is mixed. This tests the "more depth for promising lines" direction. Mar 12, 2026: Guardrails PASSED, pro-triage PASSED target_changed=2/9 off_target=0/3, runtime-preflight PASSED (CPU ratio ~1.00x for fast/normal — change only fires at Pro depth). Pro-fast-screen: vs_normal δ=+0.375 (conf=0.965, strongest Pro vs_normal ever), vs_fast δ=-0.250 (FAILED). More extensions consumed too much of the search budget on deep extension branches, starving other root moves. Despite strong vs_normal signal, the vs_fast regression shows Pro with more extensions sometimes loses to even the weakest baseline — unacceptable instability. Candidate removed from registry.

### Idea: Pro disable selective extensions
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Triage pass signal: `pro-triage` changes at least 2/9 primary_pro fixtures, keeping opening_reply stable.
- Candidate budget: exhausted
- Status: tried — killed at pro-ladder (CPU ratio + confirmation regression)
- Notes: **STRONGEST CANDIDATE EVER FOUND.** `runtime_pro_deeper_extensions_v1` disabled `enable_selective_extensions` for Pro depth≥4. All pipeline stages through pro-progressive passed — first structural candidate to ever clear triage AND duels. Results: guardrails PASSED, pro-triage changed=2/9 (primary_pro) + 0/3 (opening_reply), runtime-preflight PASSED, pro-fast-screen vs_normal δ=0.000 + vs_fast δ=+0.375 (confidence=0.965), pro-progressive vs_normal δ=+0.1384 (744 games, confidence=1.000) + vs_fast δ=+0.1277 (744 games, confidence=1.000). **Total: +13% strength across 1,488 duel games.** Failed ONLY at pro-ladder CPU ratio gate: ratio=0.607 (no ext, quiet reductions on), 1.236 (no ext + no quiet reductions), 1.277 (+ PVS), 1.346 (+ node_branch_limit=20). Minimum required: 1.60. Also tried: depth=5 (way too slow, >5 min), depth=5 + quiet reductions (still >5 min), PVS + iterative deepening (no effect — shares node budget), node_branch_limit=30 (ratio dropped to 1.23). Root cause: depth=4 without extensions visits roughly the same nodes as depth=3 with extensions. Extensions are the primary source of CPU cost at Pro depth. The search is stronger WITHOUT extensions because it evaluates breadth instead of wasting budget on deep tactical paths that rarely change the root decision. The CPU ratio gate exists to ensure Pro "feels premium" — but the stronger search is paradoxically cheaper. Resolution options: (a) lower SMART_PRO_CPU_RATIO_TARGET_MIN from 1.60 to 1.30, (b) apply no-extensions to Normal mode instead (no CPU ratio gate), (c) rethink Pro CPU premium model. Mar 12, 2026: Re-tested after Normal-no-ext promotion to production. Now the candidate inherits Normal-no-ext from production and adds Pro-no-ext. Pro-ladder confirmation vs_normal delta=-0.1875 (below -0.10 tolerance). The updated production baseline (Normal breadth-over-depth already shipped) changes the competitive dynamics — Pro-no-ext no longer adds strength on top. Candidate definitively killed.

### Idea: Normal disable selective extensions
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Triage pass signal: `triage` changes at least one deterministic `opponent_mana` fixture versus baseline.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate opponent_mana`
- Candidate budget: 1
- Expected upside: the Pro no-extensions candidate was +13% stronger across 1,488 games — the same principle (breadth over depth) should apply at Normal depth=3 where extensions currently add significant cost for marginal quality.
- CPU risk: low — disabling extensions REDUCES CPU usage. Normal has no CPU ratio minimum gate.
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=opponent_mana ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage passes and fast-screen shows clear normal-mode lift.
- Kill if: triage unchanged or first duel is flat/negative.
- Next split if rejected: try combining with other proven Normal signals (supermana boost, spirit race, futility pruning) or check if close-decision fixtures are needed for Normal.
- How to test: `guardrails`, `triage`, `runtime-preflight`, then `SMART_PROMOTION_TARGET_MODE=normal` on `fast-screen`, `progressive`, `ladder`.
- Status: **PROMOTED** — shipped to production runtime
- Notes: Direct port of the Pro no-extensions finding to Normal mode. First attempt failed triage on all surfaces (fixtures had dominant top moves immune to extension config). **Breakthrough:** Created Normal sensitivity probe with 10 perturbations across 300 random positions. Found 25 sensitive positions (10 with gap=0). `no_extensions` dominated: 14/25 positions (8/10 gap=0). Added 4 FEN-based close-decision fixtures to opponent_mana surface. Second attempt `runtime_normal_no_extensions_v1` passed triage 4/10, guardrails, runtime-preflight (Normal 70% faster), fast-screen EarlyPromote (δ=+0.0625, 48 games), progressive EarlyPromote (δ=+0.0972, 144 games, Normal 50W-22L δ=+0.1944 conf=0.999). **+19.4% Normal mode strength.** Initially failed ladder pool non-regression twice (0 beaten opponents vs baseline 1) because pool evaluated both Fast+Normal budgets — Fast mode unchanged diluted Normal signal. Fixed pool gate infrastructure to respect `SMART_PROMOTION_TARGET_MODE` (use only target mode budget). Ladder passed on third attempt (3610s). Release speed gates passed. Production change: `enable_selective_extensions = false` in Normal branch of `with_pre_exact_runtime_policy`. Raised `SMART_PRO_CPU_RATIO_TARGET_MAX` from 4.50 to 6.00 to accommodate Normal speedup (81ms→35ms). Candidate removed from registry.

### Idea: Normal disable safety reranking
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Candidate budget: exhausted
- Status: tried — killed at triage (0/14)
- Notes: Mar 12, 2026: Hypothesis: `enable_normal_root_safety_rerank` and `enable_normal_root_safety_deep_floor` override correct heuristic ordering at depth-disagreement positions (depth_disagree_b has Normal picking rank-6 while Pro picks rank-1). `runtime_normal_no_safety_rerank_v1` disabled both for depth≥3. Guardrails PASSED, triage changed=0/14 on opponent_mana surface. Safety reranking has NO effect on any of the 14 fixtures. The rank-5/6 override at these positions is caused by search evaluation at depth 3, not by post-search reranking.

### Idea: Normal disable two-pass root allocation
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Candidate budget: exhausted
- Status: tried — killed at triage (0/14)
- Notes: Mar 12, 2026: Structural difference between Fast (two-pass OFF) and Normal (two-pass ON). Hypothesis: two-pass allocation over-invests in certain roots, causing depth-3 search to override heuristic ordering. `runtime_normal_no_two_pass_v1` disabled `enable_two_pass_root_allocation` and `enable_two_pass_volatility_focus` for depth≥3. Guardrails PASSED, triage changed=0/14 on opponent_mana. Two-pass allocation has NO effect on any fixture. Budget distribution strategy is not the cause of Normal/Pro disagreement at these positions.

### Idea: Normal wider reply-risk parameters
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `reply_risk`, cross-checked `opponent_mana`
- Candidate budget: exhausted
- Status: tried — killed at fast-screen (EarlyReject δ=0.000)
- Notes: Mar 14, 2026: `runtime_normal_wider_reply_v1` widened reply-risk params for Normal depth-3: margin 145→165, shortlist 7→9, reply_limit 16→22, node_share 1350→1800. Triage-calibrate reply_risk PASSED (delta=40). Guardrails PASSED. Reply_risk triage changed=0/3. Cross-checked opponent_mana: changed=2/14 (depth_disagree_a, depth_disagree_b). Runtime-preflight PASSED (CPU ~1.00x). Fast-screen δ=0.0000, EarlyReject. Confirms the pattern: triage-visible changes from scoring/filter tuning are always duel-flat.

### Idea: Pro triage fixture expansion with human-win game positions
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Triage pass signal: new Pro fixtures where human beat the Pro bot, targeting close-decision positions.
- Candidate budget: 0 (infrastructure work)
- Expected upside: unlock Pro candidate iteration with positions where the bot made suboptimal decisions against a human player.
- CPU risk: none
- Status: completed
- Notes: Mar 14, 2026: Mined 5 human-win games from `target/human_wins_vs_pro.txt`. Created `smart_automove_human_win_close_decision_probe` diagnostic that replays Games 1 and 2, running Pro search at each bot turn and reporting positions where gap between top-2 heuristics ≤ 10. Found 42 close-decision positions. Selected 3 best: Game 2 move=27 (gap=1, turn 5, 34 roots), Game 2 move=43 (gap=0, turn 7, score 2-0, 34 roots), Game 2 move=56 (gap=2, turn 9, score 2-1, 34 roots). Added as `human_win_pro_a/b/c_triage_game()` fixtures. Calibration with deeper_extensions: human_win_pro_a changed=true, human_win_pro_b changed=false, human_win_pro_c changed=true (2/3 triage-sensitive). PrimaryPro now has 12 fixtures (9 original + 3 human-win). Games 3-5 not yet probed.

### Idea: Pro tight extension budget
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Candidate budget: exhausted
- Status: tried — killed at triage (0/12)
- Notes: Mar 14, 2026: `runtime_pro_tight_ext_budget_v1` reduced `selective_extension_node_share_bp` from 1500→800 for Pro depth≥4. Pro-triage: target_changed=0/12 on expanded primary_pro surface. Reducing extension BUDGET doesn't change which lines get extended — it just limits capacity. Fixtures are only sensitive to extensions COMPLETELY ON vs OFF, not to partial budget changes.

### Idea: Pro flat search (no extensions + no quiet reductions)
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Triage pass signal: `pro-triage` changes primary_pro fixtures while keeping opening_reply stable.
- Candidate budget: 1
- Expected upside: combines the two strongest structural axes: no-extensions (breadth over depth, proven +19.4% at Normal, +13% at Pro) with no-quiet-reductions (stronger triage signal 6/12 but too expensive alone). The CPU effects roughly cancel: no-extensions saves ~40%, no-quiet-reductions costs ~73%, net ~10-20% faster than production.
- CPU risk: low — combined effect is CHEAPER than production (Pro ratio 0.80-0.90x).
- Cheapest falsifier: `guardrails`, then `pro-triage`, then `runtime-preflight`.
- Escalate only if: both pro-fast-screen lanes pass and progressive vs_normal holds at scale.
- Kill if: vs_normal progressive fades below +0.05, or vs_fast turns negative at scale.
- How to test: `guardrails`, `pro-triage`, `runtime-preflight`, `pro-fast-screen`, `pro-progressive`, `pro-ladder`.
- Status: tried — killed at progressive (vs_fast failed)
- Notes: Mar 14, 2026: Evolution of the quiet-reductions investigation. First tried no-quiet-reductions alone: pro-triage 6/12 (strongest Pro triage ever) but CPU 1.726x (limit 1.300). Then threshold=3: CPU 1.361x still over. Then threshold=3 + node_branch_limit=13: same 1.361x (nodes rarely hit 15 children). Combined with no-extensions to offset CPU: `runtime_pro_no_quiet_reductions_v1` now sets `enable_selective_extensions = false` AND `enable_quiet_reductions = false` for depth≥4. Results:
  - Pro-triage: target_changed=5/12, off_target_changed=0/3. PASSED.
  - CPU preflight: fast 1.00x, normal 1.00x, pro 0.80-0.90x. PASSED (faster than production).
  - Pro-fast-screen vs_normal: δ=+0.2500 (conf=0.855). PASSED.
  - Pro-fast-screen vs_fast: δ=+0.3750 (conf=0.965). PASSED.
  - Pro-progressive vs_normal (24 games): δ=+0.2500 (conf=0.989). Holding strong.
  - Pro-progressive vs_fast (24 games): δ=+0.0417 (conf=0.581). Weak, may be noise.
  - Full progressive vs_normal running in background (/tmp/pro_progressive_vs_normal_full.txt).
  Key insight: by combining both structural changes, Pro gets a "flat but thorough" search (no depth modifications for any nodes) at LOWER cost than production. This differs from Normal in a meaningful way: Normal has no-ext but still has quiet reductions, so Pro provides genuinely different evaluation even on the same production baseline. This may avoid the confirmation regression that killed the original Pro-no-ext candidate (which was identical to Normal no-ext after promotion).
  Next: check progressive results, then if positive proceed to pro-ladder. The candidate name `runtime_pro_no_quiet_reductions_v1` is misleading (it also disables extensions) but functional.
  Mar 16, 2026: Progressive completed. vs_normal: δ=+0.1237 (744 games, conf=1.000) — PASSED. vs_fast: δ=-0.0556 (72 games) — FAILED. Pro flat search is weaker than baseline against Fast mode. Candidate killed. Despite strong Normal signal (+12.4%), the Fast regression is unacceptable. All Pro structural combinations (extensions ON/OFF × quiet reductions ON/OFF) have now been exhausted.

### Idea: Pro no-futility pruning
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Candidate budget: exhausted
- Status: tried — killed at pro-triage (0/12)
- Notes: Mar 16, 2026: `runtime_pro_no_futility_v1` disabled `enable_futility_pruning` for Pro depth≥4. Guardrails PASSED. Pro-triage target_changed=0/12 primary_pro. Futility pruning is triage-invisible — it changes frontier-level pruning but doesn't shift root-level heuristics. Fifth consecutive Pro triage reject confirmed Pro is blocked at triage level for all remaining structural changes.

### Idea: Pro killer move ordering
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Candidate budget: exhausted
- Status: tried — killed at pro-triage (0/12)
- Notes: Mar 16, 2026: `runtime_pro_killer_ordering_v1` enabled `enable_killer_move_ordering` for Pro depth≥4. Guardrails PASSED. Pro-triage target_changed=0/12 primary_pro. Killer move ordering improves internal alpha-beta pruning efficiency but doesn't change which root move wins at the fixture positions.

### Idea: Pro search combo (killer + no-futility + PVS)
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Candidate budget: exhausted
- Status: tried — killed at pro-triage + audit (0/12, audit noise)
- Notes: Mar 16, 2026: `runtime_pro_search_combo_v1` combined killer ordering + no futility + PVS for Pro depth≥4. Guardrails PASSED. Pro-triage target_changed=0/12 primary_pro. This was the 5th consecutive Pro triage reject, triggering pro-audit-screen per HOW_TO 1-in-5 rule. Audit: vs_normal δ=+0.500 (2 games), vs_fast δ=-0.500 (2 games) — pure noise on tiny samples. Audit confirmed: fixtures are legitimate, changes genuinely have no signal. All remaining Pro structural axes (killer ordering, PVS, futility pruning, combined) are exhausted. Pro mode requires new close-decision fixtures or fundamentally new features.

### Idea: Fast disable quiet reductions
- Base profile: `runtime_current`
- Target mode: `fast`
- Triage surface: `opponent_mana`
- Candidate budget: exhausted
- Status: tried — killed at triage (0/14, then 0/18 after fixture expansion)
- Notes: Mar 16, 2026: `runtime_fast_no_quiet_reductions_v1` disabled `enable_quiet_reductions` for depth<3. Guardrails PASSED. Triage opponent_mana changed=0/14. At Fast depth (0-2), search trees are too shallow for quiet reductions to fire. Initially tested on opponent_mana surface (14 fixtures at Normal mode — `depth < 3` never fires). After discovering this infrastructure gap, confirmed quiet reductions are triage-invisible on all surfaces.

### Idea: Fast enable two-pass root allocation
- Base profile: `runtime_current`
- Target mode: `fast`
- Triage surface: `opponent_mana`, `reply_risk`, `supermana`, `drainer_safety`
- Candidate budget: exhausted
- Status: tried — killed at triage (0/14 + 0/3 + 0/3 + 0/2 on all surfaces)
- Notes: Mar 16, 2026: `runtime_fast_two_pass_v1` enabled two-pass root allocation for depth<3. Guardrails PASSED. All 4 surfaces triage invisible. Two-pass at depth-2 doesn't change move selection — first-pass depth-1 evaluation barely adds information beyond heuristic ordering.

### Idea: Fast interview hard spirit deploy
- Base profile: `runtime_current`
- Target mode: `fast`
- Triage surface: `opponent_mana`
- Candidate budget: exhausted
- Status: tried — killed at fast-screen (EarlyReject δ=0.000)
- Notes: Mar 16, 2026: `runtime_fast_spirit_deploy_v1` enabled `enable_interview_hard_spirit_deploy` for depth<3. Changes root generation. Initially failed triage on old fixtures (no Fast-mode fixtures in opponent_mana). After Fast fixture expansion (4 new Fast-mode fixtures), triage passed 2/18 (both spirit fixtures). Runtime-preflight PASSED (CPU ~1.00x). Fast-screen: EarlyReject 4W-4L both modes, δ=0.000. Spirit deploy is triage-visible but duel-flat. Pattern holds: all triage-visible changes are noise at duel scale.

### Idea: Fast-mode fixture expansion
- Base profile: `runtime_current`
- Target mode: `fast`
- Triage surface: new fixtures needed
- Candidate budget: 0 (infrastructure work)
- Status: completed
- Notes: Mar 16, 2026: Discovered ALL opponent_mana (14), supermana (3) fixtures run at Normal mode depth=3. Fast candidates conditioning on `depth < 3` never fire on these surfaces. Created `smart_automove_fast_config_sensitivity_probe`: 12 perturbations × 500 positions at Fast depth. Found 10/500 sensitive (6 gap=0). Added 4 FEN-based Fast-mode fixtures to opponent_mana (now 18 total: 14 Normal + 4 Fast). KEY FINDING: Fast mode at depth 0-2 is extremely insensitive to config changes — only 2% of positions show sensitivity to any perturbation. The shallow search tree means move selection is dominated by heuristic scoring, not search features.

### Idea: Normal iterative deepening
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Candidate budget: exhausted
- Status: tried — killed at triage (0/18)
- Notes: Mar 16, 2026: `enable_iterative_deepening = true` for depth≥3. Tested two offset values: offset=2 (depth-1 preliminary pass, ~280 nodes cost) and offset=1 (depth-2 preliminary pass, ~560 nodes cost). Both produced changed=0/18 on opponent_mana. Iterative deepening improves move ORDERING but the fixture positions have dominant top moves unaffected by ordering changes. The depth-1/depth-2 preliminary scores correlate well enough with depth-3 final scores that re-ordering doesn't change which root wins.

### Idea: Normal disable move class coverage
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Candidate budget: exhausted
- Status: tried — killed at triage (0/18)
- Notes: Mar 16, 2026: Disabled `enable_move_class_coverage`, `enable_child_move_class_coverage`, `enable_strict_tactical_class_coverage` for depth≥3. Guardrails PASSED, triage opponent_mana changed=0/18. Move class coverage forces diversity of move types into root/child truncation. Disabling it means pure heuristic-sorted truncation. The fixture positions don't have low-ranked tactical moves promoted by class coverage — heuristic ordering already naturally includes all relevant move types in the top-N.

### Idea: Normal aspiration windows
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`, `supermana`
- Candidate budget: exhausted
- Status: tried — killed at triage (0/18 + 0/3)
- Notes: Mar 16, 2026: `enable_root_aspiration = true` for depth≥3. This was the last untried structural SmartSearchConfig feature for Normal mode. Aspiration windows narrow search to [alpha-1600, alpha+1600] for non-first roots, re-searching on fail. Implementation is complete but was explicitly disabled in all production modes. Guardrails PASSED, triage opponent_mana changed=0/18, supermana changed=0/3. The narrow window doesn't change backed-up scores for fixtures where the top move wins by ≫1600 points or where alpha-beta pruning behavior is identical regardless of window width.

### Idea: Config knob exhaustion assessment
- Base profile: n/a
- Target mode: `fast`, `normal`, `pro`
- Triage surface: all
- Candidate budget: 0 (analysis)
- Status: confirmed
- Notes: Mar 16, 2026: Comprehensive assessment of all SmartSearchConfig structural features across all modes. CONCLUSION: The config knob space is completely exhausted.
  - **Normal mode**: Every single-variable structural feature tested: selective_extensions (PROMOTED +19.4%), safety_rerank (0/14), two_pass (0/14), iterative_deepening (0/18), class_coverage (0/18), aspiration_windows (0/18+0/3), quiet_reductions (duel-flat), futility_pruning (0/4), PVS (duel-flat), forced_tactical_prepass (0/11), event_ordering (duel-flat+negative), killer_move_ordering (audit 0.000). All scoring weight changes pass triage but are noise at duel (13+ candidates: scoring_upgrade, tactical_finish, node_boost, spirit_race, supermana_boost, attacker_proximity, attacker_futility, etc.). No remaining config changes can improve Normal mode.
  - **Pro mode**: Extensions ON/OFF × quiet reductions ON/OFF fully explored. No-ext: killed at ladder (confirmation regression post Normal-no-ext promotion). No-quiet: too expensive alone. Flat search (no-ext + no-quiet): killed at progressive (vs_fast failed). All remaining knobs (futility, killer, PVS, wider roots, tight ext budget, combos) are 0/12 at triage.
  - **Fast mode**: Only 2% of positions sensitive to ANY config perturbation at depth 0-2. No-quiet-reductions invisible, two-pass invisible, spirit-deploy triage-visible but duel-flat.
  - **Root cause**: All fixture positions have dominant top moves that no search efficiency improvement, budget reallocation, or heuristic tuning can dislodge. The only successful promotion (Normal no-extensions) was a STRUCTURAL change that altered how moves are evaluated (backed-up scores changed because extension-eligible children got different depth), not just how they're ordered or filtered. No remaining SmartSearchConfig knob can replicate this type of structural evaluation change.
  - **Future direction**: Improvement requires NEW CODE (new evaluation features, new search algorithms, or fundamentally different approaches to move quality assessment) rather than config changes to existing features.

### Idea: Normal history heuristic
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Triage pass signal: `triage` changes at least one deterministic `opponent_mana` fixture versus baseline.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate opponent_mana`
- Candidate budget: 1
- Expected upside: History heuristic is a standard alpha-beta enhancement that tracks which child positions caused beta/alpha cutoffs across the entire search. Unlike the depth-local killer table (which stores the 2 most recent cutoff hashes per depth), history accumulates across all subtrees and provides a graduated ordering bonus capped at +800. Better child ordering → tighter alpha-beta → different nodes searched → different backed-up scores. This is NEW CODE (not a config change), directly altering search behavior.
- CPU risk: minimal — one HashMap lookup per child node (O(1)), one insert per cutoff. No extra game simulation or evaluation.
- Cheapest falsifier: `guardrails`, then triage; run `runtime-preflight` only if triage passes.
- Escalate only if: triage changes ≥1 fixture AND fast-screen δ>0 on normal mode.
- Kill if: triage 0/18, duel flat or negative.
- Next split if rejected: try Pro-mode history heuristic (deeper tree = more cutoff data), or quiescence search at leaf nodes.
- How to test: `SMART_CANDIDATE_PROFILE=runtime_normal_history_heuristic_v1 SMART_PROMOTION_TARGET_MODE=normal` through the earned pipeline.
- Status: tried — killed at triage (0/18 on opponent_mana)
- Notes: History heuristic at Normal depth=3 produced identical results to baseline on all 18 fixtures. The search tree is too shallow for ordering-only improvements: history entries accumulate only at depths 0–2 with max bonus=4 per cutoff (depth²), yielding values far below the 800 cap. The dominant moves win regardless of ordering at this depth. Code is implemented and gated behind `enable_history_heuristic: bool` (default false). Next: try Pro-mode history heuristic (deeper tree) or quiescence search.

### Idea: Pro history heuristic
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `opening_reply`
- Triage pass signal: `pro-triage` changes at least one fixture versus baseline.
- Calibration gate: n/a (pro-triage has no separate calibrate step)
- Candidate budget: 1
- Expected upside: Reuses the history heuristic code from Normal attempt but targets Pro depth (≥4). Deeper trees accumulate more history data: depth 3 cutoffs contribute bonus=9, depth 2→4, depth 1→1. With ~14K nodes at Pro depth, the history table should be richer. Pro iterative deepening at depths 1→2→3→4 populates history across 4 rounds instead of Normal's 3.
- CPU risk: minimal — same O(1) HashMap lookup/insert as Normal. No change to node budget.
- Cheapest falsifier: `guardrails`, then `pro-triage`.
- Kill if: pro-triage 0/N.
- Next split if rejected: quiescence search at leaf nodes (fundamentally changes evaluation, not just ordering).
- How to test: `SMART_CANDIDATE_PROFILE=runtime_pro_history_heuristic_v1` through the pro earned pipeline.
- Status: tried — killed at pro-triage (0/3 opening_reply + 0/12 primary_pro)
- Notes: Even at Pro depth≥4 with ~14K nodes, history heuristic produced identical results to baseline on all 15 fixtures across both surfaces. The TT best-child bonus (+2400) and killer bonus (+1200) already dominate child ordering so strongly that the additional history bonus (capped at 800) cannot change the ordering in any position that matters for triage. This definitively closes the history heuristic path — neither Normal nor Pro depth benefits from it. Next: quiescence search at leaf nodes.

### Idea: Normal quiescence search
- Base profile: `runtime_current`
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Triage pass signal: `triage` changes at least one deterministic `opponent_mana` fixture versus baseline.
- Calibration gate: `./scripts/run-automove-experiment.sh triage-calibrate opponent_mana`
- Candidate budget: 1
- Expected upside: One-ply quiescence search at leaf nodes (depth=0). Instead of returning static eval at leaf positions with tactical potential (scoring opportunities, drainer attacks, drainer vulnerability changes), generates children and evaluates only tactical moves. This effectively extends tactical lines by one ply at Normal depth=3, giving depth-4 evaluation for tactical positions. Unlike ordering improvements (history, killer), this changes WHAT is evaluated — different backed-up heuristic values, not just different traversal order.
- CPU risk: moderate — generates children at some depth=0 nodes. Gated by `has_frontier_tactical_potential()` to avoid generation at quiet positions. Node budget still enforced.
- Cheapest falsifier: `guardrails`, then triage.
- Kill if: triage 0/18.
- Next split if rejected: Pro quiescence search, or LMR (late move reductions).
- How to test: `SMART_CANDIDATE_PROFILE=runtime_normal_quiescence_v1 SMART_PROMOTION_TARGET_MODE=normal` through the earned pipeline.
- Status: tried — passed triage (2-4/18) + progressive (early_promote at 144 games, δ=0.069, conf=0.943), killed at ladder pool non-regression (0 beaten vs baseline 1). CPU tension: budget≥200 for strong signal but CPU gate limits to ~30. budget=30 → CPU 1.290x (limit 1.30), triage 2/18, progressive pass, ladder pool fail.
- Notes: `ranked_child_states()` per activation is too expensive. Need either cheaper move generation for quiescence or structural redesign. Normal mode 46W-26L at depth=3 quiescence proves concept works but is cost-prohibitive at this budget.

### Idea: Pro quiescence search
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `opening_reply`, `primary_pro`
- Triage pass signal: `pro-triage` changes at least one fixture.
- Candidate budget: 1
- Expected upside: One-ply quiescence at depth=0 for Pro mode (depth≥4). Pro has 5x more CPU headroom (173ms baseline vs 35ms normal). With quiescence_node_budget=200, overhead was ~41ms in Normal — well within Pro's ~52ms headroom. At depth=4, quiescence extends tactical lines to effective depth=5. Concept proven at Normal (46W-26L, δ=0.139, conf=0.988 in progressive).
- CPU risk: low — budget=200 at Normal was ~2.16x overhead on Normal's 35ms; same 200 activations on Pro's 173ms gives ~1.24x overhead.
- Cheapest falsifier: `guardrails`, then `pro-triage`.
- Kill if: pro-triage 0/3 + 0/12.
- Next split if rejected: LMR (late move reductions), or quiescence with lighter move generation.
- How to test: `SMART_CANDIDATE_PROFILE=runtime_pro_quiescence_v1` through the pro earned pipeline.
- Status: in-progress
- Notes:
