# Stronger automove review notes

This document records the production design and validation evidence for the packed automove
changes. The executable merge contract is the test suite and the immutable decision corpora;
exploratory theory probes and rejected experiments are intentionally not part of the branch.

## Shipped design

The public Fast, Normal, and Pro selectors keep their existing 16 ms, 75 ms, and 460 ms
budgets. Their fixed-clock node ceilings are 38,400, 184,000, and 2,000,000 respectively.
Unsupported packed positions continue through the canonical fallback.

The final packed search keeps these changes:

- aspiration windows for Fast and Normal;
- bounded transposition entries for selectively pruned nodes;
- canonical ordering for commuting mon sub-moves;
- cached static evaluations and attack-origin tables;
- a winning regular-mana step at any point in the turn;
- quiet spirit pushes ordered with other quiet moves while tactical pushes retain their
  reduction and pruning exemption; and
- race-shaped evaluation terms for score state, mana delivery, drainer trips, attack exposure,
  and the relative scoring tempo.

The unrestricted form that offered every early scoring mana move was rejected. Production only
offers the early move when it wins immediately. The search configuration therefore exposes only
parameters that differ among shipped profiles; invariant production behavior is not an ablation
switch.

The rules engine also advances the turn after the mon allowance is spent when no free regular
mana has a legal destination. The canonical and packed implementations share that rule.

Published JavaScript targets ES2022 so the hot packed implementation keeps native modern syntax.
The package's documented Node requirements are unchanged.

## Decision history

Each immutable corpus describes the selector round that produced it:

- [v7](../test-data/automove-decisions/v7/README.md): Fast and Normal search retuning
- [v8](../test-data/automove-decisions/v8/README.md): search structure and ES2022 output
- [v9](../test-data/automove-decisions/v9/README.md): race evaluation and winning mana move
- [v10](../test-data/automove-decisions/v10/README.md): score shape and spirit classification
- [v11](../test-data/automove-decisions/v11/README.md): mover-aware threat and race terms
- [v12](../test-data/automove-decisions/v12/README.md): two-point drainer trip selection
- [v13](../test-data/automove-decisions/v13/README.md): quiet spirit ordering and table reuse

`v13` is the active deterministic contract: 13 source states and all three public preferences,
for 39 replayed decisions with `performance.now()` fixed at zero. Earlier versions remain
protected provenance and must not be rewritten.

## Final-round evidence

The final quiet-spirit ordering change was measured on held-out, colour-swapped, fixed-node
self-play before the corpus was recorded:

| Mode   | Pairs |   Elo |   95% interval | Invalids / cutoffs |
| ------ | ----: | ----: | -------------: | -----------------: |
| Fast   | 1,280 | +21.5 |  +9.1 to +33.9 |              0 / 0 |
| Normal | 1,280 | +25.3 | +12.2 to +38.5 |              0 / 0 |
| Pro    |   320 | +50.3 | +24.6 to +76.5 |              0 / 0 |

An eight-block ABBA latency run over 64 midpoint states measured candidate/baseline mean ratios
of 0.997 for Fast, 0.998 for Normal, and 1.000 for Pro. These historical measurements explain
the accepted change; they are not a substitute for rerunning the tracked regression and smoke
checks on the review commit.

## Review verification

Run the complete repository gate:

```sh
npm run check
```

Given separately built baseline and candidate bundles, run the tracked 13-state performance
comparison and apply the existing mean and p95 limits:

```sh
node scripts/run-automove-performance.mjs \
  --baseline /path/to/main.mjs \
  --candidate ./dist/mons-rules.js \
  --states test-data/automove-decisions/v6/decisions.jsonl \
  --modes fast,normal,pro \
  --repeat 5 \
  --out target/automove-performance.json
node scripts/check-automove-performance.mjs \
  --report target/automove-performance.json
```

Exercise cache reuse by keeping each public game alive across plies:

```sh
node scripts/run-automove-strength.mjs \
  --baseline /path/to/main.mjs \
  --candidate ./dist/mons-rules.js \
  --modes fast,normal,pro \
  --variants all \
  --game-driving held \
  --out target/automove-strength.json
```

The smoke run must complete without invalid suggestions, source mutation, illegal replay, or
cutoffs. Timing and playing-strength totals should be recorded for review rather than encoded as
machine-independent exact assertions.
