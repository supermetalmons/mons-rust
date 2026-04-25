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

The harness now has a diagnostic promotion dashboard:

```sh
./scripts/run-automove-experiment.sh pro-promotion-dashboard <sweep-candidate[,candidate...]>
```

The dashboard summarizes:

- Pro/Normal/Fast win rate, confidence, and average move time
- per-variant rows sorted by weakest result
- candidate-vs-guarded and candidate-vs-shipping deltas
- whether the candidate is active-blocker-only, sampled-only, or broadly promising

Use profile attribution after the dashboard identifies a globally promising candidate with one explainable panel failure.

## Stop Conditions

Stop the iteration and update docs instead of patching runtime code when any of these are true:

- the best candidate only wins on the active blockers
- attribution shows the same branch label has both saves and regressions
- the weak rows rotate to different variants after a local fix
- a copied trace board does not reproduce cleanly
- a sampled-pass change fails all-variant confirm with broad Pro/Normal/Fast loss
- the proposed change is another board-local fallback, shortlist widening, branch toggle, or variant mirror without a decision-record mechanism

## Next Concrete Step

The first unified-policy scout has now been killed. Direct root replacement, conservative post-advisor override, full-selection reply-risk approval, selected-followup composites, reply-floor portfolios, and active-context portfolio gates all moved one active row while rotating another; the strongest active-only result still stopped at `5-1 / 5-1 / 5-1` and exceeded the CPU ceiling.

The next useful implementation is harness-first: record alternative-policy choices and outcome labels across both canonical sampled and active panels without selecting them as runtime behavior. At minimum, compare retained guarded, no-selected followup, full-selection reply-risk, and any new ProV3 utility candidate at each first divergence, then aggregate by variant, color, turn, resource state, exact-opportunity context, advisor reason, selected family, head family, and reply-risk status.

Use the policy-matrix `PRO_POLICY_MATRIX_PORTFOLIO` and `PRO_POLICY_MATRIX_PORTFOLIO_CLASS` lines before another context selector. If `no_policy_wins` remains high, the compared policies do not contain a winning move for that slice and a selector cannot promote them. If `baseline_only_wins` is nonzero, treat every proposed gate as regression-prone until its context aggregate separates those baseline saves from `candidate_only_wins` across both sampled and active panels.

The latest matrix wave changed the blocker shape: adding the test-only `frontier_pro_v3_alternating_white_edge_mana` policy plus the retained shipping-control and known ProV2/ProV3 ablations eliminated `no_policy_wins` and `baseline_only_wins` on the checked sampled-Pro and active-blocker panels. That is selector evidence, not promotion evidence. The narrow edge-mana policy itself failed structural scout and must not be promoted directly.

The policy-winner/context aggregate now exists, and the first selector over that complete policy set is killed. `frontier_pro_v3_policy_winner_context_full_outer` reached sampled Pro `11-1`, Normal `12-0`, Fast `11-1`, but failed active blockers at Pro `4-2`, Normal `6-0`, Fast `5-1`. Switching the active outer-edge white context to shipping fixed Fast and collapsed Pro; adding matrix-derived earlier-entry gates still failed and regressed Normal. Future work should not spend another static selector over the same policy labels.

The next useful implementation should be a shared utility or root-evaluation feature that explains why the winning policy sometimes must enter earlier or later than the printed first divergence. Use `pro-policy-matrix` and `pro-policy-winner` to evaluate new policy components, but runtime code should change only after a candidate passes the promotion dashboard on both sampled and active panels.
