# Automove Ideas

This is the working backlog for future automove loops.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution workflow and use this file to decide what to try next. When an idea is tried, promoted, ruled out, or split into follow-up ideas, update this file instead of relying on memory or raw logs.

If every current item here has been tried, add new ideas and keep going.

## Template

### Idea: <short name>
- Target mode:
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
- Expected upside: stronger `pro` promotion candidate by separating independent-search strength from opening-reply confirmation behavior.
- CPU risk: medium
- Cheapest falsifier: `preflight`, then `pro-pre-screen` against both `normal` and `fast` baselines.
- Escalate only if: both cheap screens pass and at least one baseline shows a clear positive signal that is not just opening-book confirmation noise.
- Kill if: either pro cheap screen is negative or flat, or the gain only appears in confirmation-heavy openings.
- Next split if rejected: separate primary independent-search tuning from opening-reply confirmation tuning and retry only the surviving half.
- How to test: `preflight`, `pro-pre-screen`, `pro-fast-screen`, `pro-progressive`, `pro-ladder`, then the two release speed gates.
- Status: backlog
- Notes: Keep opening-book driven confirmation cheaper and safer than the fully independent `pro` path.

### Idea: Normal conversion stability from proven pro signals
- Target mode: `normal`
- Expected upside: better opponent-mana and supermana conversion without importing the full `pro` budget.
- CPU risk: low to medium
- Cheapest falsifier: `preflight`, then `pre-screen` against `runtime_release_safe_pre_exact`.
- Escalate only if: the cheap screen shows clear lift in `normal` without dragging `fast` backward.
- Kill if: the candidate is flat overall, or any apparent gain only survives as noise while `normal` remains explainably weak.
- Next split if rejected: break the idea into separate opponent-mana and supermana conversion probes.
- How to test: `preflight`, `pre-screen`, `fast-screen`, `progressive`, `ladder`, then mode comparison against `runtime_release_safe_pre_exact` and `runtime_current` if results are borderline.
- Status: backlog
- Notes: Port only the cheapest signals that already proved useful in stronger `pro` candidates.

### Idea: Fast reply-risk cleanup without normal-style overreach
- Target mode: `fast`
- Expected upside: cheap strength gain from removing fake-good replies while preserving fast latency.
- CPU risk: low
- Cheapest falsifier: `preflight`, then `pre-screen` with tight reply-risk edits only.
- Escalate only if: the cheap screen shows a visible `fast` improvement with no sign that `normal` is carrying the result.
- Kill if: the screen is flat, or the only promising result needs wider normal-style shortlist tuning.
- Next split if rejected: isolate a smaller reply-risk guard or root tie-break instead of another combined cleanup wave.
- How to test: `preflight`, `pre-screen`, `fast-screen`, and focused mode comparisons for `fast` vs baseline `fast` and `normal` if the screen is borderline.
- Status: backlog
- Notes: Keep shortlist sizes and reply budgets tight. Avoid reusing aggressive normal-side penalties that already regressed.

### Idea: Exact-lite only on high-value conversion windows
- Target mode: `normal`, `pro`
- Expected upside: recover tactical strength only where safe supermana, safe opponent-mana, or spirit-assisted score windows are plausibly available.
- CPU risk: medium
- Cheapest falsifier: exact-lite diagnostics gate, `preflight`, then the target mode cheap screen.
- Escalate only if: diagnostics stay bounded and the cheap screen shows that selective activation beats the baseline in the intended conversion window.
- Kill if: the exact-lite trigger leaks into routine turns, or any lift disappears once the cheap screen leaves the target positions.
- Next split if rejected: split by trigger family such as safe supermana, opponent-mana conversion, or spirit-assisted windows.
- How to test: exact-lite diagnostics gate, `preflight`, `pre-screen` or `pro-pre-screen`, then the normal or pro promotion path depending on the target mode.
- Status: backlog
- Notes: No broad exact-lite activation. Trigger selectively and keep explicit per-move budgets.

### Idea: Shared tactical and exact-lite cache reuse
- Target mode: `normal`, `pro`
- Expected upside: more strength from the same CPU budget by reusing cached summaries across root ranking, tie-breaks, and tactical prepasses.
- CPU risk: low to medium
- Cheapest falsifier: speed probes, exact-lite diagnostics gate, then the target mode cheap screen.
- Escalate only if: cache reuse lowers duplicated work and the cheap screen shows strength from the reclaimed budget instead of from extra search.
- Kill if: reuse adds bookkeeping without measurable lift, or the cheap screen stays flat after the speed gain.
- Next split if rejected: keep only the cheapest cache-sharing point and drop the rest of the reuse surface.
- How to test: speed probes, exact-lite diagnostics gate, `preflight`, `pre-screen` or `pro-pre-screen`, then the standard promotion pipeline for the target mode.
- Status: backlog
- Notes: Prefer reuse before deeper search. Strength that comes from duplicated work is unlikely to be promotable.

### Idea: Stronger own-drainer exposure filtering
- Target mode: `fast`, `normal`, `pro`
- Expected upside: eliminate fake-good moves that immediately expose the drainer unless the move wins or scores decisive mana.
- CPU risk: low
- Cheapest falsifier: `preflight`, `pre-screen`, then `fast-screen` if the candidate changes move choice often enough to matter.
- Escalate only if: a harder filter or a combined signal shows measurable cheap-screen lift instead of just sounding correct.
- Kill if: a standalone penalty or wider margin stays near zero or turns negative by `fast-screen`.
- Next split if rejected: combine exposure awareness with harder filtering or reply cleanup instead of another standalone penalty wave.
- How to test: tactical guardrails first, then `pre-screen`, `fast-screen`, and only the earned promotion path for the target mode.
- Status: tried — not promotable
- Notes: Tested as `drainer_exposure_v1` (Mar 9, 2026). Heuristic penalty (200/300) plus wider safety margin (2800/5200) showed zero measurable strength gain — fast-screen δ=+0.009, progressive δ=-0.007. The existing late-stage safety filter already handles most cases. Future drainer work should combine exposure awareness with other signals or use harder filtering, not standalone penalties.

### Idea: Spirit-off-base development that still respects tempo
- Target mode: `normal`, `pro`
- Expected upside: better setup play from earlier spirit deployment without drifting into low-value wandering.
- CPU risk: low to medium
- Cheapest falsifier: the target mode tactical fixtures, `preflight`, then the cheap screen.
- Escalate only if: the screen shows concrete score, denial, or setup value rather than extra wandering.
- Kill if: the idea mostly adds motion without conversion, or only looks good in fixtures while the screen stays flat.
- Next split if rejected: separate spirit setup for score, denial, and conversion so only the useful branch survives.
- How to test: add or strengthen fixtures around spirit scoring and setup, then run `preflight`, `pre-screen` or `pro-pre-screen`, and the earned promotion path.
- Status: backlog
- Notes: Reward spirit movement only when it creates concrete score, denial, or conversion value.

### Idea: Opening black-reply strength under hard latency budget
- Target mode: `fast`, `normal`, `pro`
- Expected upside: stronger early replies without breaking release-safe opening latency.
- CPU risk: medium
- Cheapest falsifier: `preflight`, the release opening speed gate, then the target mode cheap screen.
- Escalate only if: the opening speed gate remains clean and the cheap screen shows stronger early replies within the fixed budget.
- Kill if: the idea needs more opening latency to work, or the cheap screen gain disappears once the hard latency budget is enforced.
- Next split if rejected: isolate cheaper opening-reply ranking signals from any broader search-budget changes.
- How to test: `preflight`, release opening speed gate, `pre-screen` or `pro-pre-screen`, then the earned promotion pipeline for the target mode.
- Status: backlog
- Notes: Treat the opening-reply budget as fixed and improve move quality inside it rather than expanding it.

### Idea: Anti-help and anti-roundtrip cheap root scoring
- Target mode: `fast`, `normal`
- Expected upside: quick strength gain from refusing mana moves that help the opponent or waste tempo.
- CPU risk: low
- Cheapest falsifier: tactical guardrails, `preflight`, then `pre-screen`.
- Escalate only if: the cheap screen shows that the root-only score changes are removing obvious anti-help or roundtrip mistakes.
- Kill if: the candidate is flat, or the only lift comes from wider search rather than the cheap root score itself.
- Next split if rejected: split anti-help from anti-roundtrip so each can be tested as a narrower root-only rule.
- How to test: tactical guardrails, `preflight`, `pre-screen`, `fast-screen`, and focused mode comparison if the screen is borderline.
- Status: backlog
- Notes: This is a good candidate for low-cost root-only scoring or tie-break adjustments.

### Idea: Stuck-state and bounded-progress safety fixtures
- Target mode: `fast`, `normal`, `pro`
- Expected upside: stronger release confidence by catching empty-selector, repeated-position, and no-progress edge cases before promotion.
- CPU risk: low
- Cheapest falsifier: new or strengthened fixtures that fail the candidate immediately.
- Escalate only if: the safety fixtures pass and the candidate still clears the relevant cheap screen without added instability.
- Kill if: the candidate needs unsafe fallback behavior to stay strong, or any stuck-state fixture remains unresolved.
- Next split if rejected: isolate the single unsafe edge case and fix that before touching strength again.
- How to test: targeted fixtures, `preflight`, `pre-screen` or `pro-pre-screen`, and the normal release gates.
- Status: backlog
- Notes: Safety work is promotion work. A candidate that can stall or behave unpredictably is not ready.

### Idea: Promotion-focused artifact summaries
- Target mode: workflow
- Expected upside: faster iteration because doc-worthy outcomes become obvious and disposable logs stay disposable.
- CPU risk: low
- Cheapest falsifier: one failed loop where the lesson still requires digging through raw logs.
- Escalate only if: each loop can produce a backlog update, archive note, or durable lesson directly from the standard artifacts.
- Kill if: the summary step adds ceremony without shortening the time from run result to decision.
- Next split if rejected: keep only the smallest summary artifact that directly feeds backlog, archive, or knowledge updates.
- How to test: verify each loop can move a conclusion into `docs/automove-knowledge.md`, `docs/automove-archive.md`, or a new backlog item with no dependence on raw logs.
- Status: backlog
- Notes: Improve signal extraction, not permanent logging volume.
