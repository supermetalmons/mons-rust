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
- retained guarded decision records are singleton-heavy below `frontier_execute`; branch-level concentration alone is not a spendable mechanism

## Next Concrete Step

The first unified-policy scout has now been killed. Direct root replacement, conservative post-advisor override, full-selection reply-risk approval, selected-followup composites, reply-floor portfolios, and active-context portfolio gates all moved one active row while rotating another; the strongest active-only result still stopped at `5-1 / 5-1 / 5-1` and exceeded the CPU ceiling.

The next useful implementation is harness-first: record alternative-policy choices and outcome labels across both canonical sampled and active panels without selecting them as runtime behavior. At minimum, compare retained guarded, no-selected followup, full-selection reply-risk, and any new ProV3 utility candidate at each first divergence, then aggregate by variant, color, turn, resource state, exact-opportunity context, advisor reason, selected family, head family, and reply-risk status.

Use the policy-matrix `PRO_POLICY_MATRIX_PORTFOLIO` and `PRO_POLICY_MATRIX_PORTFOLIO_CLASS` lines before another context selector. If `no_policy_wins` remains high, the compared policies do not contain a winning move for that slice and a selector cannot promote them. If `baseline_only_wins` is nonzero, treat every proposed gate as regression-prone until its context aggregate separates those baseline saves from `candidate_only_wins` across both sampled and active panels.

The latest matrix wave changed the blocker shape: adding the test-only `frontier_pro_v3_alternating_white_edge_mana` policy plus the retained shipping-control and known ProV2/ProV3 ablations eliminated `no_policy_wins` and `baseline_only_wins` on the checked sampled-Pro and active-blocker panels. That is selector evidence, not promotion evidence. The narrow edge-mana policy itself failed structural scout and must not be promoted directly.

The policy-winner/context aggregate now exists, and the first selector over that complete policy set is killed. `frontier_pro_v3_policy_winner_context_full_outer` reached sampled Pro `11-1`, Normal `12-0`, Fast `11-1`, but failed active blockers at Pro `4-2`, Normal `6-0`, Fast `5-1`. Switching the active outer-edge white context to shipping fixed Fast and collapsed Pro; adding matrix-derived earlier-entry gates still failed and regressed Normal. Future work should not spend another static selector over the same policy labels.

The latest `pro-policy-winner` refresh found oracle coverage on sampled Pro and active Pro/Normal/Fast with the same policy portfolio, but the winning labels are singleton-heavy and conflict by evolved board. After fixing forced-root FEN case preservation, sampled Fast `corner_chain_mana_rows` white also has winning roots and a narrow utility policy component, but that only improves oracle coverage. Active `outer_edge_mana_rows` white turn-three action+mana still needs full-scored reply guard in one Pro opening and shipping-control routing in Fast under the same exact-opportunity context. Do not replace the killed context selector with a guarded-selected-move table or first-diff board fingerprint table; that would memorize dashboard openings without creating a broader automove feature.

The active-focused refresh on the expanded portfolio kept the same conclusion: active Pro was `3` baseline wins / `3` policy wins, active Normal `5` / `1`, and active Fast `2` / `4`, all with `no_policy_wins=0`. The policy wins still split across full-scored reply guard, shipping-control, alternating-white edge mana, and raw ProV2. Use panel filters for these portfolio diagnostics; the full all-panel run is slow and does not change the selector conclusion unless a new policy component is added.

The cross-budget policy diagnostic now exists for the exact selector-false-positive problem. On a one-opening sampled smoke with guarded, shipping-control, and raw, the white `inner_wedge_mana_rows` side was a `budget_conflict`: raw repaired the Pro loss but regressed Normal, while no policy was non-regressing across all three budgets. The black side of the same opening was a clean shipping-control repair across Pro/Normal/Fast. Future selector work should run this probe before implementation; if the class is `budget_conflict`, do not encode the policy label as a gate.

The first static composite from cross-budget-clean classes is killed. An active white `outer_edge_mana_rows` full-scored reply-guard repair was all-budget clean on one opening, and the sampled black `inner_wedge_mana_rows` shipping-control repair was also all-budget clean. The active-only repair still failed sampled `7-5 / 7-5 / 6-6`; stacking both repairs only reached sampled `8-4 / 7-5 / 6-6`. Treat cross-budget clean classes as feature-design signals, not as direct selector components.

The same no-go now covers a smaller sampled inner-wedge full-scored route. A minimal cross-budget smoke saw full-scored reply guard repair both sides of one sampled `inner_wedge_mana_rows` opening without regression, but a variant-scoped sampled dashboard failed Pro `7-5` and lost inner-wedge `0-2`. Do not turn one-opening non-regressing repairs into variant-scoped policy gates.

The quiet-mana utility scouts are killed too. Selecting near-top quiet `ManaTempo` roots by `TurnEngineUtility` in quiet mana-available contexts through turn six moved some sampled Pro rows but still failed sampled Pro at `9-3` with weak `center_spoke_mana_rows`, `alternating_mana_rows`, and `forward_bridge_mana_rows`; it also fired in generic early-opening contexts and raised average move time above `200ms`. Tightening the same idea to after-opening turns fixed the center row but worsened sampled Pro to `8-4`, with weak rows rotating to `alternating`, `inner_wedge`, `split_flank`, and `forward_bridge`. Do not use a wider quiet-mana gate as the next ProV3 feature.

The color-symmetric safe-mana selected-followup projection scout is killed. Broadening the existing black-only safe `ManaTempo` / `DrainerSafetyRecovery` projection path to both colors failed sampled Pro at `9-3` and created an `inner_wedge_mana_rows` `0-2` weak row, so selected-followup projection symmetry is not the missing shared utility feature.

The broad utility-admission variant is also killed. Admitting projected-utility roots into the reply-risk guard before final selection failed sampled Pro at `7-5`, so the next root-evaluation attempt needs a new cross-budget stability signal rather than another utility gate around reply-risk shortlist membership.

The simple cross-budget score-floor signal is killed too. A test-only candidate that preserved guarded wrapper branches and only overrode `frontier_execute` roots when their Pro/Normal/Fast guarded score floor beat the guarded root still failed sampled Pro at `7-5` after enough overrides to change behavior. Future stability work needs a feature below raw search score agreement, not another internal budget floor.

The latest sampled-policy refresh killed the cheap selector escape routes. A post-guard normal-root-safety reply-gain switch was inert, a full policy-portfolio rollout selector was far too slow even with early-only 24-ply rollouts, and a sampled policy-winner context selector only proved a Pro-vs-Pro false positive. The expanded portfolio with `frontier_pro_v3_white_opening_utility_mana` removed the false corner-chain no-policy row and reached sampled `11-1 / 11-1 / 11-1`, but active Fast still failed at `5-1`; a wider outer-edge-only delta run also exposed same-context Pro improvement and Fast regression. Another selector over existing policies cannot be the next structural implementation.

The latest head-layer structural scouts are killed. Skipping post-search head acceptance wholesale failed the sampled dashboard at Pro `4-8`, Normal `7-5`, Fast `8-4`; requiring a strict primary-axis improvement before accepting the head reproduced sampled Pro `4-8`. This means the head layer is both a source of regressions and a source of sampled Pro saves. Future work should not spend on broad head vetoes; it needs a deeper utility feature that separates harmful accepted heads from saved head continuations before selection.

The policy-agreement shortcut is killed as well. Asking the existing policy portfolio to override guarded only when multiple policies agree still failed sampled Pro at `5-7` under both loose and strict thresholds, and it carried `520ms+` average candidate turns before any Normal/Fast or active-panel spend. Agreement between existing policy outputs is not a shared cross-budget utility signal.

The latest sampled shipping/no-low-budget scout is killed. Exact sampled shipping-control repairs plus the retained alternating-white component moved sampled Pro from `7-5` to `10-2`, but adding broader forward-bridge quiet-white routing and a no-low-budget inner-wedge white gate did not improve that score. Full-policy traces can win those games, while one-shot first-diff routing cannot; raw early-white inner-wedge routing is also a cross-budget conflict because it fixes Pro and regresses Normal. Do not spend another direct selector on these sampled first-diff labels.

The recovery-rank/opening-utility scout is killed. Demoting `DrainerSafetyRecovery` behind `SpiritImpact` and safe-progress families was a real sampled-Pro directional signal because it fixed `inner_wedge_mana_rows`, but stacking the retained alternating-white and white-opening utility components still stopped at sampled Pro `10-2` with `forward_bridge_mana_rows` split. The follow-up decision records split across black turn-four action+mana attack/window, white turn-five mana-only calm setup, white turn-three action+mana attack, and black turn-six mana-only window contexts. Treat recovery-rank demotion as feature-design evidence only; it is not a promotable shared utility change.

The sticky policy-continuation follow-up is killed too. Letting those sampled shipping/no-low-budget policy choices continue for four follow-up turns only reached sampled Pro `10-2` when the fallthrough stayed on guarded; adding the non-promotable alternating-white policy component got Pro back to `11-1` but failed sampled Normal `8-4` and Fast `6-6`. Do not retry thread-local/profile-continuation selectors over existing policy labels unless a new utility feature first passes both sampled and active panels.

The portfolio allowed-head continuation scout is killed as well. Feeding the current policy portfolio's first moves into the turn engine as allowed heads was too expensive when broad; the narrowed early-conflict version still failed sampled Pro `8-4` with `33` overrides, `311.91ms` average move time, and `alternating_mana_rows` at `0-2`. Existing policy outputs plus turn-engine continuation planning still rotate rows instead of creating a shared cross-budget utility signal.

The latest retained guarded sampled decision-record refresh did not expose a code path to spend on. Pro, Normal, and Fast nonwins all reached `frontier_execute`, but they were singleton-heavy below that branch and split across variants, colors, turns, resource states, families, accepted-head/advisor state, and exact move pairs. Future runtime work should start from a new shared utility/root-evaluation feature or a candidate that already passes the promotion dashboard, not from another `frontier_execute` selector.

The next useful implementation should be a shared utility or root-evaluation feature that explains why the winning policy sometimes must enter earlier or later than the printed first divergence. Use `pro-policy-matrix`, `pro-policy-winner`, and `pro-policy-cross-budget` to evaluate new policy components, but runtime code should change only after a candidate passes the promotion dashboard on both sampled and active panels.
