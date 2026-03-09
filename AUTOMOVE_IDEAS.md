# Automove Ideas

This is the working backlog for future automove loops.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution workflow and use this file to decide what to try next. When an idea is tried, promoted, ruled out, or split into follow-up ideas, update this file instead of relying on memory or raw logs.

If every current item here has been tried, add new ideas and keep going.

## Template

### Idea: <short name>
- Target mode:
- Triage surface:
- Triage pass signal:
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

## Backlog

### Idea: Pro primary vs confirmation retune
- Target mode: `pro`
- Triage surface: `primary_pro`
- Triage pass signal: `pro-triage` changes at least one `primary_pro` fixture while keeping `opening_reply` mostly stable.
- Candidate budget: 1
- Expected upside: stronger `pro` promotion candidate by separating independent-search strength from opening-reply confirmation behavior.
- CPU risk: medium
- Cheapest falsifier: `preflight`, then `SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage <candidate>`.
- Escalate only if: `pro-triage` passes cleanly and `pro-fast-screen` shows a clear positive story, not just a non-negative result.
- Kill if: `pro-triage` fails, `opening_reply` moves too much for a primary-path edit, or the first duel is flat or noisy.
- Next split if rejected: split into a pure `opening_reply` candidate or a narrower `primary_pro` candidate, but not both in the same retry.
- How to test: `preflight`, `pro-triage`, `pro-fast-screen`, `pro-progressive`, `pro-ladder`, then the two release speed gates.
- Status: backlog
- Notes: Keep opening-book confirmation cheaper and safer than the fully independent `pro` path.

### Idea: Normal conversion stability from proven pro signals
- Target mode: `normal`
- Triage surface: `opponent_mana`
- Triage pass signal: `triage` changes at least one deterministic `opponent_mana` fixture versus baseline without guardrail regressions.
- Candidate budget: 1
- Expected upside: better opponent-mana and supermana conversion without importing the full `pro` budget.
- CPU risk: low to medium
- Cheapest falsifier: `preflight`, then `SMART_TRIAGE_SURFACE=opponent_mana ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: `triage` passes and `fast-screen` shows a clear `normal` lift without dragging `fast` backward.
- Kill if: `triage` does not move the target surface, or the first duel is flat overall.
- Next split if rejected: break the idea into separate `opponent_mana` and `supermana` probes.
- How to test: `preflight`, `triage`, `fast-screen`, `progressive`, `ladder`, then mode comparison against `runtime_release_safe_pre_exact` and `runtime_current` only if the first duel is borderline.
- Status: backlog
- Notes: Port only the cheapest signals that already proved useful in stronger `pro` candidates.

### Idea: Fast reply-risk cleanup without normal-style overreach
- Target mode: `fast`
- Triage surface: `reply_risk`
- Triage pass signal: `triage` changes the reply-risk fixture pack instead of leaving it identical to baseline.
- Candidate budget: 1
- Expected upside: cheap strength gain from removing fake-good replies while preserving fast latency.
- CPU risk: low
- Cheapest falsifier: `preflight`, then `SMART_TRIAGE_SURFACE=reply_risk ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: `triage` passes and `fast-screen` shows a visible `fast` improvement with no sign that `normal` is carrying the result.
- Kill if: `triage` is unchanged, or the first duel is flat, or the only promising result needs wider normal-style shortlist tuning.
- Next split if rejected: isolate a smaller reply-risk guard or root tie-break instead of another combined cleanup wave.
- How to test: `preflight`, `triage`, `fast-screen`, and one focused mode comparison only if the first duel is borderline.
- Status: backlog
- Notes: Keep shortlist sizes and reply budgets tight. Avoid reusing aggressive normal-side penalties that already regressed.

### Idea: Exact-lite only on high-value conversion windows
- Target mode: `normal`, `pro`
- Triage surface: `supermana`
- Triage pass signal: `triage` moves the deterministic `supermana` fixtures, or the split `pro` follow-up moves `primary_pro` without touching `opening_reply`.
- Candidate budget: 1
- Expected upside: recover tactical strength only where safe supermana, safe opponent-mana, or spirit-assisted score windows are plausibly available.
- CPU risk: medium
- Cheapest falsifier: exact-lite diagnostics gate, `preflight`, then `SMART_TRIAGE_SURFACE=supermana ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: diagnostics stay bounded and triage shows a deterministic change on the intended conversion window.
- Kill if: the exact-lite trigger leaks into routine turns, or triage does not move the intended surface.
- Next split if rejected: split by trigger family such as `supermana`, `opponent_mana`, or a dedicated `primary_pro` follow-up.
- How to test: exact-lite diagnostics gate, `preflight`, `triage` or `pro-triage` after the idea is split to one surface, then the earned promotion path for the target mode.
- Status: backlog
- Notes: No broad exact-lite activation. Trigger selectively and keep explicit per-move budgets.

### Idea: Shared tactical and exact-lite cache reuse
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
- How to test: speed probes, exact-lite diagnostics gate, `preflight`, `triage`, then the earned promotion path for the target mode.
- Status: backlog
- Notes: Prefer reuse before deeper search. Strength that comes from duplicated work is unlikely to be promotable.

### Idea: Stronger own-drainer exposure filtering
- Target mode: `fast`, `normal`, `pro`
- Triage surface: `drainer_safety`
- Triage pass signal: `triage` changes the drainer-safety fixtures before any duel is allowed.
- Candidate budget: 1
- Expected upside: eliminate fake-good moves that immediately expose the drainer unless the move wins or scores decisive mana.
- CPU risk: low
- Cheapest falsifier: `preflight`, then `SMART_TRIAGE_SURFACE=drainer_safety ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage moves the safety surface and `fast-screen` shows measurable lift instead of just sounding correct.
- Kill if: triage is unchanged, or the first duel stays near zero or turns negative.
- Next split if rejected: combine exposure awareness with harder filtering or reply cleanup instead of another standalone penalty wave.
- How to test: tactical guardrails first, then `triage`, `fast-screen`, and only the earned promotion path for the target mode.
- Status: tried — not promotable
- Notes: Tested as `drainer_exposure_v1` (Mar 9, 2026). Heuristic penalty (200/300) plus wider safety margin (2800/5200) showed zero measurable strength gain — fast-screen δ=+0.009, progressive δ=-0.007. The existing late-stage safety filter already handles most cases. Future drainer work should combine exposure awareness with other signals or use harder filtering, not standalone penalties.

### Idea: Spirit-off-base development that still respects tempo
- Target mode: `normal`, `pro`
- Triage surface: `spirit_setup`
- Triage pass signal: `triage` changes at least one fixed spirit-setup fixture without regressing the generic guardrails.
- Candidate budget: 1
- Expected upside: better setup play from earlier spirit deployment without drifting into low-value wandering.
- CPU risk: low to medium
- Cheapest falsifier: the target mode tactical fixtures, `preflight`, then `SMART_TRIAGE_SURFACE=spirit_setup ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage passes and the first duel shows concrete score, denial, or setup value rather than extra wandering.
- Kill if: triage is unchanged, or the idea mostly adds motion without conversion in the first duel.
- Next split if rejected: separate spirit setup for score, denial, and conversion so only the useful branch survives.
- How to test: add or strengthen spirit fixtures first if needed, then run `preflight`, `triage` or `pro-triage` after the idea is split to one surface, and only the earned promotion path.
- Status: backlog
- Notes: Reward spirit movement only when it creates concrete score, denial, or conversion value.

### Idea: Opening black-reply strength under hard latency budget
- Target mode: `fast`, `normal`, `pro`
- Triage surface: `opening_reply`
- Triage pass signal: `pro-triage` changes at least one fixed opening black-reply fixture while `primary_pro` stays mostly stable.
- Candidate budget: 1
- Expected upside: stronger early replies without breaking release-safe opening latency.
- CPU risk: medium
- Cheapest falsifier: `preflight`, the release opening speed gate, then `SMART_TRIAGE_SURFACE=opening_reply ./scripts/run-automove-experiment.sh pro-triage <candidate>`.
- Escalate only if: the opening speed gate remains clean and `pro-triage` shows deterministic opening-reply change inside the fixed budget.
- Kill if: the idea needs more opening latency to work, or `primary_pro` moves too much for an opening-only edit.
- Next split if rejected: isolate cheaper opening-reply ranking signals from any broader search-budget changes.
- How to test: `preflight`, release opening speed gate, `pro-triage`, then the earned promotion pipeline for the target mode.
- Status: backlog
- Notes: Treat the opening-reply budget as fixed and improve move quality inside it rather than expanding it.

### Idea: Anti-help and anti-roundtrip cheap root scoring
- Target mode: `fast`, `normal`
- Triage surface: `reply_risk`
- Triage pass signal: `triage` changes the fixed reply-risk/root-choice fixture pack before any duel.
- Candidate budget: 1
- Expected upside: quick strength gain from refusing mana moves that help the opponent or waste tempo.
- CPU risk: low
- Cheapest falsifier: tactical guardrails, `preflight`, then `SMART_TRIAGE_SURFACE=reply_risk ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: triage shows the root-only score changes are actually moving the deterministic surface.
- Kill if: triage is unchanged, or the first duel is flat, or the only lift comes from wider search rather than the root score itself.
- Next split if rejected: split anti-help from anti-roundtrip so each can be tested as a narrower root-only rule.
- How to test: tactical guardrails, `preflight`, `triage`, `fast-screen`, and one focused mode comparison if the first duel is borderline.
- Status: backlog
- Notes: This is a good candidate for low-cost root-only scoring or tie-break adjustments.

### Idea: Stuck-state and bounded-progress safety fixtures
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
- How to test: targeted fixtures first, then `preflight`, then the relevant `triage` or `pro-triage`, then the normal release gates.
- Status: backlog
- Notes: Safety work is promotion work. A candidate that can stall or behave unpredictably is not ready.

### Idea: Promotion-focused artifact summaries
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
