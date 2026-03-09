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
- Status: tried — killed at pro-triage
- Notes: Keep opening-book confirmation cheaper and safer than the fully independent `pro` path. Mar 9, 2026: candidate `runtime_pro_primary_exact_lite_v1` enabled bounded exact-lite progress/spirit checks and slightly stronger opponent-mana conversion only for depth-4 `pro` primary contexts. It passed `guardrails` in about 6s, then failed `SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage runtime_pro_primary_exact_lite_v1` with `target_changed=0 off_target_changed=0`. No `runtime-preflight` or duel work was run, and the transient candidate was removed from the registry.

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
- Status: backlog
- Notes: Keep shortlist sizes and reply budgets tight. Avoid reusing aggressive normal-side penalties that already regressed.

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
- Status: tried — stalled at target-aware fast-screen
- Notes: Continuation of the audit-validated candidate, not a same-idea retry. Candidate `runtime_fast_clean_reply_risk_v1` stayed fast-only and changed only `prefer_clean_reply_risk_roots`, `root_reply_risk_score_margin`, `root_reply_risk_shortlist_max`, `root_reply_risk_reply_limit`, and `root_reply_risk_node_share_bp`. Mar 9, 2026: target-aware duel policy fixed the pooled false reject. `triage-calibrate reply_risk`, `guardrails`, `SMART_TRIAGE_SURFACE=reply_risk ./scripts/run-automove-experiment.sh triage runtime_fast_clean_reply_risk_v1`, and `runtime-preflight` all passed. Under `SMART_PROMOTION_TARGET_MODE=fast`, the first `fast-screen` replay no longer died at tier 0: it reached tier 1 with aggregate `δ=+0.0208`, `fast δ=+0.0417`, and `normal δ=0.0000` over `48` games, but then the `8`-games-per-seed expansion turned into a workflow stall and the run was stopped after about `19m` wall-clock with no final candidate decision. The stage was then tightened so target-aware `fast-screen` caps at the `48`-game tier, but a rerun under current machine load still did not return promptly enough to be a useful loop, so the batch stopped on workflow grounds rather than on candidate strength. The transient candidate was removed from the registry again. The next workflow cut is cached top-off or another cheaper target-aware first duel stage, not another fast reply-risk variant.

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
