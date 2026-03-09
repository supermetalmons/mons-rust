# Automove Ideas

This is the working backlog for future automove loops.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution workflow and use this file to decide what to try next. When an idea is tried, promoted, ruled out, or split into follow-up ideas, update this file instead of relying on memory or raw logs.

If every current item here has been tried, add new ideas and keep going.

## Template

### Idea: <short name>
- Target mode:
- Expected upside:
- CPU risk:
- How to test:
- Status: backlog
- Notes:

## Backlog

### Idea: Pro primary vs confirmation retune
- Target mode: `pro`
- Expected upside: stronger `pro` promotion candidate by separating independent-search strength from opening-reply confirmation behavior.
- CPU risk: medium
- How to test: `preflight`, `smart_automove_pool_pro_fast_screen_vs_normal`, `smart_automove_pool_pro_fast_screen_vs_fast`, `smart_automove_pool_pro_promotion_ladder`, then the two release speed gates.
- Status: backlog
- Notes: Keep opening-book driven confirmation cheaper and safer than the fully independent `pro` path.

### Idea: Normal conversion stability from proven pro signals
- Target mode: `normal`
- Expected upside: better opponent-mana and supermana conversion without importing the full `pro` budget.
- CPU risk: low to medium
- How to test: `preflight`, `fast-screen`, `progressive`, `ladder`, then mode comparison against `runtime_release_safe_pre_exact` and `runtime_current` if results are mixed.
- Status: backlog
- Notes: Port only the cheapest signals that already proved useful in stronger `pro` candidates.

### Idea: Fast reply-risk cleanup without normal-style overreach
- Target mode: `fast`
- Expected upside: cheap strength gain from removing fake-good replies while preserving fast latency.
- CPU risk: low
- How to test: `preflight`, `fast-screen`, and focused mode comparisons for `fast` vs baseline `fast` and `normal`.
- Status: backlog
- Notes: Keep shortlist sizes and reply budgets tight. Avoid reusing aggressive normal-side penalties that already regressed.

### Idea: Exact-lite only on high-value conversion windows
- Target mode: `normal`, `pro`
- Expected upside: recover tactical strength only where safe supermana, safe opponent-mana, or spirit-assisted score windows are plausibly available.
- CPU risk: medium
- How to test: exact-lite diagnostics gate, `preflight`, then the normal or pro promotion path depending on the target mode.
- Status: backlog
- Notes: No broad exact-lite activation. Trigger selectively and keep explicit per-move budgets.

### Idea: Shared tactical and exact-lite cache reuse
- Target mode: `normal`, `pro`
- Expected upside: more strength from the same CPU budget by reusing cached summaries across root ranking, tie-breaks, and tactical prepasses.
- CPU risk: low to medium
- How to test: exact-lite diagnostics gate, speed probes, `preflight`, and the standard promotion pipeline.
- Status: backlog
- Notes: Prefer reuse before deeper search. Strength that comes from duplicated work is unlikely to be promotable.

### Idea: Stronger own-drainer exposure filtering
- Target mode: `fast`, `normal`, `pro`
- Expected upside: eliminate fake-good moves that immediately expose the drainer unless the move wins or scores decisive mana.
- CPU risk: low
- How to test: tactical guardrails first, then the standard promotion path for the target mode.
- Status: backlog
- Notes: This aligns with durable interview guidance and tends to pay off before deeper search changes.

### Idea: Spirit-off-base development that still respects tempo
- Target mode: `normal`, `pro`
- Expected upside: better setup play from earlier spirit deployment without drifting into low-value wandering.
- CPU risk: low to medium
- How to test: add or strengthen fixtures around spirit scoring and setup, then run the standard promotion pipeline.
- Status: backlog
- Notes: Reward spirit movement only when it creates concrete score, denial, or conversion value.

### Idea: Opening black-reply strength under hard latency budget
- Target mode: `fast`, `normal`, `pro`
- Expected upside: stronger early replies without breaking release-safe opening latency.
- CPU risk: medium
- How to test: `preflight`, release opening speed gate, and the standard promotion pipeline for the target mode.
- Status: backlog
- Notes: Treat the opening-reply budget as fixed and improve move quality inside it rather than expanding it.

### Idea: Anti-help and anti-roundtrip cheap root scoring
- Target mode: `fast`, `normal`
- Expected upside: quick strength gain from refusing mana moves that help the opponent or waste tempo.
- CPU risk: low
- How to test: tactical guardrails, `preflight`, `fast-screen`, and focused mode comparison if the result is borderline.
- Status: backlog
- Notes: This is a good candidate for low-cost root-only scoring or tie-break adjustments.

### Idea: Stuck-state and bounded-progress safety fixtures
- Target mode: `fast`, `normal`, `pro`
- Expected upside: stronger release confidence by catching empty-selector, repeated-position, and no-progress edge cases before promotion.
- CPU risk: low
- How to test: targeted fixtures plus the normal release gates.
- Status: backlog
- Notes: Safety work is promotion work. A candidate that can stall or behave unpredictably is not ready.

### Idea: Promotion-focused artifact summaries
- Target mode: workflow
- Expected upside: faster iteration because doc-worthy outcomes become obvious and disposable logs stay disposable.
- CPU risk: low
- How to test: verify each loop can move a conclusion into `docs/automove-knowledge.md`, `docs/automove-archive.md`, or a new backlog item with no dependence on raw logs.
- Status: backlog
- Notes: Improve signal extraction, not permanent logging volume.
