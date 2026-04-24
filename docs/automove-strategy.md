# Automove Structural Strategy

This document is the reset point when local automove iteration stalls. Use it before cutting another Pro runtime patch.

## Diagnosis

The current loop is stuck because it is optimizing local seams faster than it is improving global policy strength.

- Public Pro is `frontier_pro_v2_guarded`: a guarded ProV2 turn-engine path plus a wrapper chain of search-only fallbacks.
- The strongest recent test-only profile direction, `frontier_pro_v2_raw`, improves the known active blocker slice but fails the canonical sampled Pro-vs-Pro panel at `5-7`.
- Guarded wrapper branches are not globally bad. On sampled Pro guarded-vs-raw attribution, both `early_white_fallback` and `late_black_shipping_fallback` produced saves and regressions.
- The current weak rows are multi-variant and mixed-stage: Normal `outer_edge_mana_rows`, Fast `alternating_mana_rows`, and Fast `forward_bridge_mana_rows`.
- Repeated local boards have repeatedly failed to generalize. Several sampled-pass runtime patches cleared one row and then failed all-variant confirm with broad Pro/Normal/Fast strength loss.
- The docs have also accumulated too many no-go details in the live board. Future work should preserve live state and strategic decisions, not probe diaries.

## Position

Do not continue the old seam loop as the default. A future automove patch should start from one of the structural paths below and must pass the structural scout before runtime code is retained.

## Structural Scout

Before editing runtime code for a broad Pro change, run:

```sh
./scripts/run-automove-structural-scout.sh <sweep-candidate[,candidate...]>
```

This runs two diagnostic panels:

- canonical sampled variants with `repeats=3`, `games=2`
- active blocker variants with `repeats=1`, `games=3`

Run the optional all-variant scout only after both default panels look promotable:

```sh
./scripts/run-automove-structural-scout.sh --confirm <sweep-candidate[,candidate...]>
```

Interpretation:

- Kill a candidate that is strong only on active blockers but fails canonical sampled variants.
- Kill a candidate that is strong only on canonical sampled variants but fails active blockers.
- Do not spend runtime code unless every Pro/Normal/Fast duel in the scout is at least directionally promotable: win rate `>= 0.90`, confidence trending toward `0.99`, and no obvious weak variant row.
- Use attribution only after the scout identifies a candidate that is globally promising but has one explainable panel failure.

## Major Paths

### 1. Context-Gated Meta-Selector

Build a test-only candidate that can choose between guarded, raw ProV2, and selected retained fallback behavior from decision context, not branch labels.

Required evidence:

- Use guarded-vs-raw attribution to collect both saves and regressions.
- Classify by context fields below the wrapper branch: variant, color, turn, `window/deny`, selector stage, pre-accept family, head family, advisor reason, shortlist shape, and selected-vs-head rank.
- Only promote a gate when one context class is mostly one-sided across canonical sampled and active-blocker panels.

Avoid:

- Removing `early_white_fallback` or `late_black_shipping_fallback` wholesale.
- Variant-only gates that simply mirror shipping on blocker variants.

### 2. Evaluation Portfolio Before Runtime

Treat candidate work as portfolio selection before code. Compare multiple test-only candidates across the same panels before keeping any one line.

Recommended portfolio:

- current retained `frontier_pro_v2_guarded`
- `frontier_pro_v2_raw`
- any new ProV3 or meta-selector candidate
- one deliberately conservative candidate that preserves guarded saves

Promote only candidates that beat retained guarded on both the canonical sampled panel and active-blocker panel. Active-blocker-only wins are not enough.

### 3. Trace Corpus And Decision Records

Stop relying on copied `first_diff_ply` boards as primary evidence. Add or extend diagnostics so every duel divergence can emit a compact decision record:

- opening seed, variant, mode, color, turn, ply
- selected move, opponent/baseline move, final selected move
- runtime branch and selector stage
- pre-accept root family, head root family, advisor reason
- shortlist size and whether the baseline root was candidate-live, shortlist-live, preserved, or omitted
- outcome delta

Then aggregate records before writing runtime code. A patch needs a repeated mechanism at the decision-record layer, not just a repeated move pair.

### 4. ProV3 Search Policy, Not More Wrapper Surgery

If the scout keeps showing mixed guarded/raw behavior, create a new test-only ProV3 candidate that changes the core selector policy rather than adding fallback wrappers.

Likely areas:

- unify head acceptance, reply-risk approval, and final root scoring into one utility comparison
- make preserved and omitted roots first-class candidates instead of late exceptions
- calibrate `TurnEngineUtility` against actual duel outcomes across variants
- reduce special-case fallbacks by moving their lessons into shared utility axes

Promotion shape:

- ProV3 must pass structural scout before any retained profile entry is added.
- ProV3 must beat `shipping_pro_search` and not materially regress `frontier_pro_v2_guarded` on the same panels.

### 5. Harness-Level Promotion Dashboard

The harness should eventually produce one promotion dashboard instead of scattered logs. The dashboard should summarize:

- Pro/Normal/Fast win rate, confidence, and average move time
- per-variant rows sorted by weakest result
- candidate-vs-guarded and candidate-vs-shipping deltas
- branch/context classes responsible for outcome-changing divergences
- whether the candidate is active-blocker-only, sampled-only, or broadly promising

Until that dashboard exists, the structural scout plus profile attribution is the minimum replacement.

## Stop Conditions

Stop the iteration and update docs instead of patching runtime code when any of these are true:

- the best candidate only wins on the active blockers
- attribution shows the same branch label has both saves and regressions
- the weak rows rotate to different variants after a local fix
- a copied trace board does not reproduce cleanly
- a sampled-pass change fails all-variant confirm with broad Pro/Normal/Fast loss
- the proposed change is another board-local fallback, shortlist widening, branch toggle, or variant mirror without a decision-record mechanism

## Next Concrete Step

The next useful implementation is a test-only context-gated meta-selector candidate or a decision-record aggregation diagnostic. Do not cut another runtime seam patch until one of those produces a scout-pass candidate.
