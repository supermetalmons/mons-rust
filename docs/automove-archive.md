# Automove Archive

This file keeps only short summaries of retired automove waves.

Everything here is archive-only context. Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the live workflow and `docs/automove-knowledge.md` for durable rules that still matter.

## Iterative-Deepening Recheck And Advisor Reply-Floor Policy

- Temporary test-only candidate source was cut and removed in the same session.
- Recreating the sampled-pass guarded iterative-deepening row composite reconfirmed the old active-blocker failure: Pro `3-3`, Normal `3-3`, Fast `5-1`. Active Normal still had `outer_edge_mana_rows` at `0-2`, while active Pro split `outer_edge_mana_rows`, `alternating_mana_rows`, and `forward_bridge_mana_rows`.
- Narrow outer-edge shipping fallbacks did not rescue it. The white-opening shipping branch was mostly inert, and the combined white-opening plus black weak-window shipping branch only moved the active dashboard to Pro `3-3`, Normal `4-2`, Fast `5-1`.
- Active decision records for the repaired row composite stayed singleton-heavy: remaining Pro misses split across an `outer_edge` flat, an `alternating` flat, and a `forward_bridge` early-white fallback regression.
- A test-only snapshot-based advisor reply-floor policy then preserved guarded selection, captured the already-computed root-selection snapshot, and reranked only advisor/top roots by primary utility, progress/setup value, and reply floor. It failed the sampled dashboard before active spend: Pro `5-7`, Normal `9-3`, Fast `8-4`, with override counts `104 / 127 / 162` and max average move time `190.26ms`.
- Durable outcome: the root-selection snapshot is cheap enough for sampled-dashboard scouts, but advisor/top-root reply-floor plus progress reranking is still too broad. Do not retry this as another hand-tuned comparator; it needs a corpus-trained feature or a repeated mechanism below the current root fields.

## Policy-Winner Mechanism-Class and Legacy-Preaccept Head Veto

- This wave kept a harness improvement and removed the test-only runtime scout before ending.
- `pro-policy-corpus` / `smart_automove_pro_policy_winner_probe` now emits `PRO_POLICY_WINNER_MECHANISM_CLASS` when mechanism tracing is enabled. These are coarse axes over stage/head state, baseline-vs-winner role, family, advisor status, rank, safety/progress, and winner-root shape, printed next to exact `PRO_POLICY_WINNER_MECHANISM` records.
- The first sampled Pro mechanism-class corpus over the full reset portfolio still had exact singleton mechanisms: baseline wins `7`, policy wins `5`, no-policy wins `0`, `max_mechanism_games=1`, `max_mechanism_class_games=2`. The only repeated class was a broad safe-step-progress shape, which is not enough for runtime code.
- The active Fast corpus also stayed exact-singleton with oracle coverage: baseline wins `2`, policy wins `4`, no-policy wins `0`, `max_mechanism_games=1`, `max_mechanism_class_games=2`. Its repeated class was selected accepted-head baseline roots losing to pre-accept/legacy winner roots.
- A temporary test-only `frontier_pro_v4_legacy_preaccept_head_veto` candidate vetoed an accepted head only when the guarded selected move was exactly the head and the pre-accept root matched the legacy or legacy-full-pool selection. It fired `26` times in sampled Pro and failed the quick dashboard at `8-4` (`0.6667`, confidence `0.8062`, avg `195.76ms`), with weak rows in `center_spoke_mana_rows`, `alternating_mana_rows`, `inner_wedge_mana_rows`, and `forward_bridge_mana_rows`.
- Durable outcome: coarse mechanism classes are useful routing evidence, but a repeated selected-head/pre-accept class is still not a promotable head-veto feature by itself. Future head work needs a new discriminator below pre-accept/legacy status and must pass sampled plus active panels before runtime source is retained.

## Shallow ProV4 Unified Root-Value Comparator

- Temporary test-only candidate source was cut and removed in the same session.
- The structural scout reconfirmed retained guarded as non-promotable: sampled Pro/Normal/Fast `7-5 / 7-5 / 6-6`, active blockers `3-3 / 5-1 / 2-4`, and `PRO_PROMOTION_DASHBOARD_STOPLIGHT` `not_promising`.
- The policy corpus again had oracle coverage without selector evidence: every sampled and active Pro/Normal/Fast duel had `no_policy_wins=0`, `max_policy_games=1`, `max_mechanism_games=1`, and `PRO_POLICY_WINNER_STOPLIGHT` `singleton_residue`.
- A test-only shallow ProV4 comparator ranked scored roots by existing tactical value, `TurnEngineUtility`, safety, progress, family priority, score, and root rank while preserving guarded as the incumbent. The loose version improved sampled Pro to `9-3`, fixed `inner_wedge_mana_rows` and `forward_bridge_mana_rows`, but rotated losses into `center_spoke_mana_rows`, `alternating_mana_rows`, and `split_flank_mana_rows`.
- A score-drop-tightened version reduced overrides from `139` to `116` and fixed `split_flank_mana_rows`, but still failed sampled Pro at `9-3`; `center_spoke_mana_rows` fell to `0-2`, `alternating_mana_rows` stayed split, and the sampled Pro nonwin probe emitted `singleton_regression_pressure` with max branch/context/pair counts of `1`.
- Durable outcome: do not retry a single shallow root-value tuple over existing root fields. The next ProV4 attempt needs a new corpus-trained feature, preserved/omitted/root-timing evidence, or a harness change that finds repeated mechanisms below the current singleton records.

## Policy-Rollout and Dormant-Toggle Quick Kill

- Temporary test-only candidates were cut and removed in the same session.
- The online policy-rollout scout preserved guarded fallbacks, gathered guarded, shipping-control, raw, no-selected-followup, and full-scored reply-guard outputs on early `frontier_execute` turns, and tried short continuation rollouts before switching. It was killed on cost: the broad version did not complete sampled Pro-only validation after several minutes, and the tightened two-ply Pro/Fast version remained too slow for a practical quick gate.
- Two wrapper-preserving dormant toggle checks were also killed on sampled Pro-only promotion dashboard. Enabling `enable_turn_engine_mid_turn_progress_guard` reproduced `7-5`, `inner_wedge_mana_rows` `0-2`, candidate average `141.61ms`; enabling `enable_turn_engine_late_black_setup_progress_rescue` reproduced `7-5`, `inner_wedge_mana_rows` `0-2`, candidate average `142.20ms`.
- Durable outcome: do not use online rollout over the current policy portfolio without a cheaper precomputed feature, and do not reopen mid-turn progress guard or late-black setup-progress rescue as direct Pro challengers.

## Forced-Root FEN and Policy-Portfolio Selector Refresh

- This diagnostic wave fixed a harness bug and kept the fix: `SMART_PRO_FORCED_ROOT_ORACLE_FEN` now uses a raw env string helper so case-sensitive FEN payloads are not lowercased.
- The fix invalidated the earlier sampled Fast `corner_chain_mana_rows` white zero-root no-go. With case preserved, the forced-root oracle found `16` scored roots and `13` winning first roots; `frontier_pro_v3_white_opening_utility_mana` was kept as a test-only policy component that selects `l10,5;l9,4` over the shipped losing `l10,3;l9,4` on that board.
- The expanded policy portfolio reached oracle coverage on the checked sampled and active panels, but a context selector over those labels was discarded. Its best dashboard shape was sampled `11-1 / 11-1 / 11-1` and active Pro/Normal/Fast `6-0 / 6-0 / 5-1`, so active Fast remained below directional promotion shape.
- A wider `outer_edge_mana_rows` delta check killed the selector line rather than just the single dashboard miss: the same white turn-three `window=0/deny=0/drainer_safety=2` context was a Pro improvement in one opening and a Fast regression in another, and the wider Pro slice exposed a separate black regression.
- Durable outcome: keep the FEN raw-env harness fix and the narrow white-opening policy component for future matrices, but do not retain the `frontier_pro_v3_context_policy_portfolio` selector or replace it with guarded-selected-move/FEN gates.

## Iterative-Deepening Row-Composite Wave

- Temporary test-only sweep candidates were cut and removed in the same session. The wave tested guarded ProV2 with iterative deepening, alpha-window variants, node compensation, lazy-oracle projection, and row composites across the promotion dashboard.
- Plain iterative deepening was directionally useful but not broad enough: sampled Pro passed at `11-1`, while sampled Normal failed at `9-3`. Offset-1 plus `1.25x` nodes inverted that shape, passing sampled Normal at `11-1` while failing sampled Pro at `8-4`.
- Search-order tuning did not solve sampled Fast by itself. The best standalone Fast tuning was iterative deepening with alpha margin `320` and `1.25x` nodes, which reached `10-2` but still split `offset_arc_mana_rows` and `corner_chain_mana_rows`; other node and offset variants stayed at `7-5` to `9-3`.
- The best sampled-only row composite used guarded iterative deepening by default, raw ProV2 only for `offset_arc_mana_rows`, alpha-window iterative deepening plus `1.25x` nodes only for `inner_wedge_mana_rows`, and offset-1 plus `1.25x` nodes for `outer_edge_mana_rows`, `forward_bridge_mana_rows`, and `corner_chain_mana_rows`. It passed the sampled dashboard in all three duels: Pro `11-1`, Normal `11-1`, Fast `11-1`; max average move time was `183.71ms`.
- The active-blocker dashboard killed the composite before promotion: Pro `3-3`, Normal `3-3`, Fast `5-1`. Active Normal `outer_edge_mana_rows` fell to `0-2`, active Pro split `outer_edge_mana_rows`, `alternating_mana_rows`, and `forward_bridge_mana_rows`, and active Fast still split `outer_edge_mana_rows`.
- Follow-up active checks on older profiles exposed the underlying conflict. Raw ProV2 fixes active Normal `outer_edge_mana_rows` at `2-0`, but loses active Pro and Fast `outer_edge_mana_rows` at `0-2`; no-late-black fallback also loses active Pro/Fast `outer_edge_mana_rows` while improving Normal.
- Durable outcome: do not promote sampled-only row composites or variant-only raw fallbacks. A viable next candidate needs a below-variant `outer_edge_mana_rows` feature that distinguishes the active Normal contexts from the active Pro/Fast contexts without relying on opponent mode.

## ProV3 Utility Selector/Switch Wave

- A temporary test-only ProV3 utility wave was cut and removed in the same session. It tried selecting roots by `TurnEngineUtility` order and switching away from retained guarded only when a utility candidate was not worse on primary axes.
- Pure utility root selection failed the active blocker panel. Candidate-set utility scored Pro `3-3`, Normal `3-3`, Fast `4-2`; full-pool utility scored Pro `3-3`, Normal `4-2`, Fast `4-2`.
- Full-pool utility switches were not promotable. The dominant-primary switch failed Pro at `1-5`; the nonstrict switch reached Pro `5-1`, but Normal and Fast only reached `4-2`.
- Candidate-set nonstrict utility switching was the best shape on active blockers: Pro `5-1`, Normal `5-1`, Fast `5-1`. Attribution versus retained guarded showed candidate saves in several black/full-turn and mana-only contexts, one Pro candidate regression on an early white quiet turn, and shared Normal/Fast misses that guarding back to retained would not solve.
- Follow-up tuning did not rescue it. Score tolerance `64` fell to Pro `3-3`; score tolerance `128` fell to Pro `4-2`; utility eval tolerance `64` fell to Pro `3-3`; an early-white quiet-turn guard fell to Pro `4-2` and introduced an `alternating_mana_rows` split.
- Canonical sampled Pro killed the line before Normal/Fast spend: `5-7`, with `alternating_mana_rows` `0-2`, `inner_wedge_mana_rows` `0-2`, and `forward_bridge_mana_rows` `3-1`.
- Durable outcome: do not retry shallow root-level utility ordering or guarded utility switching as the next ProV3 spend. A viable ProV3 candidate needs new utility features or a dashboard-driven mechanism that is strong on both sampled variants and active blockers.

## Decision-Record Aggregation Wave

- A diagnostic-only decision-record aggregator was added and kept. It replays Pro, Normal, and Fast shipping duels and classifies each first divergence by runtime branch, selector stage, pre-accept/head family, head acceptance, approved advisor root, exact context, and baseline-root status.
- Delta-scope records showed that many promotion misses are not frontier-worse-than-shipping deltas. On the active blockers, Pro had `0` regressions / `2` improvements / `4` flat, Normal had `1` / `2` / `3`, and Fast had `2` / `2` / `2`; canonical sampled Pro had no regressions.
- Nonwin-scope records exposed why a simple shipping-root selector is unsafe. Active nonwins split across alternating black recovery-vs-mana flat loss, forward-bridge white SpiritImpact-vs-ManaTempo regression, outer-edge white SpiritImpact-vs-DrainerSafetyRecovery regression, and forward-bridge white safe-progress-vs-spirit regression. Canonical sampled nonwins added outer-edge black same-family ManaTempo flat loss plus the archived forward-bridge white accepted-head spirit regression.
- A temporary test-only live-root shipping meta-selector was cut and removed in the same session. Routing to shipping when the shipping root was candidate-live at rank `0` failed the active Pro panel at `1-5`; broader top-2 and advisor-live variants also failed at `2-4`, with `outer_edge_mana_rows` and `forward_bridge_mana_rows` both `0-2`.
- Durable outcome: do not use candidate-live, top-ranked, or advisor-live shipping status as the next meta-selector by itself. The status is useful evidence for records, but the mechanisms are still split; the next runtime spend needs a true ProV3 utility change or another aggregate that collapses the split below advisor/head/final-selection status.

## Attribution-Informed Context Gate Wave

- A test-only context-gated sweep candidate was cut and killed in this wave, then removed before ending the session.
- The wave first refreshed canonical sampled guarded-vs-raw attribution. Guarded still had sampled Pro saves and regressions under the same coarse fallback branches: Pro raw-better `2`, guarded-better `4`, same `6`; Normal raw-better `3`, guarded-better `0`, same `9`; Fast raw-better `4`, guarded-better `0`, same `8`.
- The active-blocker attribution and direct profile refresh confirmed there was no hidden raw challenger on the current harness. On `outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows`, retained guarded scored Pro `3-3`, Normal `5-1`, Fast `2-4`, while raw scored Pro `2-4`, Normal `6-0`, Fast `4-2`.
- The broader exact-context gate switched to raw on observed raw-better white opening and black unsafe-context surfaces. It failed structural scout: canonical sampled Pro `9-3`, Normal `8-4`, Fast `6-6`; active blockers Pro `3-3`, Normal `5-1`, Fast `3-3`.
- The narrowed gate removed the riskiest white forward-bridge and inner-wedge opening switches. It also failed structural scout: canonical sampled Pro `8-4`, Normal `8-4`, Fast `6-6`; active blockers Pro `3-3`, Normal `5-1`, Fast `3-3`.
- Durable outcome: do not retry context gates built only from variant/color/turn/resource/exact-opportunity fields. The next useful spend needs a decision record below the context label, including candidate-live/shortlist-live status, advisor reason, head acceptance, final selector stage, and baseline-root position.

## Profile-Level ProV2 Ablation Wave

- Runtime challengers were cut and killed in this wave; each was reverted before ending the session.
- Enabling `enable_turn_head_rerank` inside `frontier_pro_v2_guarded` passed guardrails, variant-smoke, triage, and runtime-preflight, then failed sampled `pro-reliability`: Pro `1.0000`, Normal `0.8333`, Fast `0.7500`. It added Normal `outer_wedge_mana_rows` and Fast `corner_chain_mana_rows` blockers.
- Disabling `turn_engine_enable_spirit_family` passed early gates, then failed sampled `pro-reliability`: Pro `0.6667`, Normal `0.8333`, Fast `0.7500`. It broadened Pro losses across `outer_edge_mana_rows`, `outer_wedge_mana_rows`, and `corner_chain_mana_rows`.
- Disabling `enable_turn_engine_mid_turn_tactical_guard` passed early gates, then failed sampled `pro-reliability` at the baseline-shaped Pro `1.0000`, Normal `0.9167`, Fast `0.8333`; it cleaned sampled Fast `corner_chain_mana_rows` but left Normal `outer_edge_mana_rows` and Fast `alternating_mana_rows` / `forward_bridge_mana_rows`.
- Raising only `turn_engine_expansion_cap` from `176` to `224` passed early gates, then failed sampled `pro-reliability` at Pro `1.0000`, Normal `0.9167`, Fast `0.8333`; the extra capacity was strength-inert on the live blockers.
- Disabling `enable_turn_engine_low_budget_guard` passed early gates, then failed sampled `pro-reliability`: Pro `0.7500`, Normal `0.9167`, Fast `0.8333`. It improved sampled Fast `forward_bridge_mana_rows` but broadened Pro losses across `outer_wedge_mana_rows`, `outer_edge_mana_rows`, and `corner_chain_mana_rows`, and Fast `alternating_mana_rows` fell to `0.0000`.
- Durable outcome: do not reopen broad ProV2 profile toggles without a new trace tying the toggle to the live blocker mechanism. The sampled blockers did not respond as one guard/capacity problem, and guard removals can broaden variant losses while staying under the time budget.

## Reply-Risk Recovery Scope Wave

- A runtime challenger was cut and killed in this wave.
- The candidate restricted `pro_v2_root_advisor_black_turn_four_weak_window_recovery_override` so it could only promote `DrainerSafetyRecovery` roots that were already present in the reply-risk shortlist.
- Focused retained Fast controls still passed: `frontier_pro_v2_guarded_profile_prefers_shipping_black_recovery_duel_fast_root` and `frontier_pro_v2_guarded_profile_prefers_shipping_black_branch_duel_sampled_fast_root`.
- The local signal moved but did not promote. The full isolated `alternating_mana_rows` Fast trace improved from the archived `7` nonwins to `6`, and the repeated `l2,7;l1,6` vs `l2,7;l1,8` pair disappeared, but mixed singleton residue remained.
- Canonical sampled `pro-reliability` then failed: Pro `1.0000` / confidence `0.9998` / `143.32ms`, Normal `0.8333` / `0.9807` / `186.18ms`, Fast `0.8333` / `0.9807` / `165.40ms`.
- The failure rotated Normal down to `center_spoke_mana_rows` and `outer_edge_mana_rows` at `0.5000` each, while Fast still had `alternating_mana_rows` and `forward_bridge_mana_rows` at `0.5000` each.
- Durable outcome: do not keep or retry reply-risk-shortlist scoping for the black turn-four weak-window recovery override by itself. It can remove one repeated `alternating` seam, but it is not a promotable multi-variant strength mechanism.

## Root Advisor Trace Refresh Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave ran `smart_automove_pro_root_advisor_trace_probe` against retained footholds and duel-trace boards near the current sampled blocker families.
- The trace did not expose one shared advisor-layer failure. Blocker-adjacent boards split across `ApprovedReplyRiskGuard`, `ApprovedFamilyCompetition`, omitted-root reentry, preserved representatives, and rejected injected macro roots.
- Several cases also showed injected macro roots being admitted on unrelated retained controls while nearby blocker boards rejected injection or selected through preserved/reentry paths, so a broad metadata or injection change would be another speculative churn source.
- Durable outcome: do not spend on a shared advisor-label, preserved-representative, omitted-root reentry, or injected-root admission patch from this trace. The remaining sampled blockers still require a narrower mechanism than "advisor trace differs from shipping."

## Canonical Sampled Gate Refresh Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave ran the canonical `runtime-preflight` stage for `frontier_pro_v2_guarded`; both stage-1 CPU advisory and exact-lite diagnostics passed, and the runtime-preflight stamp was written.
- The follow-up canonical sampled `pro-reliability` gate still failed promotion. Pro passed at `1.0000` with confidence `0.9998` and `143.82ms` frontier average move time, but Normal stayed at `0.9167` with confidence `0.9968`, and Fast stayed at `0.8333` with confidence `0.9807`; both failed the `0.90` win-rate / `0.99` confidence requirement.
- The variant failures matched the targeted refreshes rather than revealing a new mechanism: Normal `outer_edge_mana_rows` was `0.5000`, Fast `alternating_mana_rows` was `0.5000`, and Fast `forward_bridge_mana_rows` was `0.5000`; sampled `classic`, `corner_chain_mana_rows`, `offset_arc_mana_rows`, `outer_wedge_mana_rows`, `split_flank_mana_rows`, `swapped_mana_rows`, and `center_spoke_mana_rows` were clean in this gate sample.
- Durable outcome: public Pro remains on `frontier_pro_v2_guarded`, but there is no promotable automove candidate from this wave. The next spend still needs a focused mechanism below the mixed `outer_edge` Normal plus `alternating`/`forward_bridge` Fast residue.

## Fast Blocker Refresh Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave reran `SMART_AUTOMOVE_VARIANTS=alternating_mana_rows,forward_bridge_mana_rows` on `smart_automove_pro_reliability_nonwin_trace_probe` with `duel_filter=vs_shipping_fast`, `repeats=4`, and `games=3`.
- The refreshed Fast replay logged `9` nonwins. It did not improve on the archived shape: the only repeated pair was still the white `forward_bridge` accepted-head miss `l9,6;l7,4;l7,3` vs shipping `l9,6;l7,6;l7,7` (`2x`).
- Every other Fast miss stayed singleton, including archived copied-board classes: black `l0,10;l0,9` vs `l4,0;l5,0;mb`, white `l9,6;l8,7` vs `l9,6;l7,7;l8,8`, black `l0,6;l1,6` vs `l2,3;l3,4`, black `l2,5;l0,5;l1,6` vs `l2,5;l4,7;l3,8`, white `l8,5;l10,5;l9,4` vs `l8,5;l6,3;l7,2`, black `l2,7;l1,6` vs `l2,7;l1,8`, and black `l2,4;l1,5` vs `l2,4;l1,3`.
- Durable outcome: do not reopen the isolated Fast head-accept or alternating mana-sibling patches from this refresh. The trace is still mixed and mostly replays already-killed copied-board or retained-extension no-go surfaces.

## Outer-Edge Normal Refresh Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave reran `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows` on `smart_automove_pro_reliability_nonwin_trace_probe` with `duel_filter=vs_shipping_normal`, `repeats=6`, and `games=3`.
- The refreshed replay again logged `10` Normal nonwins. The distribution still matched the archived mixed bucket instead of collapsing to one local patch target.
- The only repeated move pair was still black `l2,7;l3,8` vs shipping `l1,5;l0,3;l1,3` (`3x`). The rest remained singleton drift across late black same-family mana, early white post-search mana, early black copied-board recovery, and white `search_only_forced_prepass`.
- The head-accept source review did not reveal a clean general rule worth cutting: blocking the repeated black miss would require another special-case head guard, while the sampled blocker set still includes unrelated pre-accept and forced-prepass surfaces.
- Durable outcome: do not reopen `outer_edge_mana_rows` from the refreshed Normal trace alone. It is still mixed residue with one repeated accepted-head pair, not a promotable Pro automove hypothesis.

## Alternating Singleton Copied Board Replay Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave reran `SMART_AUTOMOVE_VARIANTS=alternating_mana_rows` on `smart_automove_pro_reliability_nonwin_trace_probe` with `duel_filter=vs_shipping_fast`, `repeats=4`, and `games=3`.
- The replay again logged `7` Fast nonwins and recovered the two previously unverified singleton pairs: white `l8,5;l10,5;l9,4` vs shipping `l8,5;l6,3;l7,2`, and black `l2,4;l1,5` vs shipping `l2,4;l1,3`.
- The follow-up direct replay killed both as stable local targets. The white copied board collapsed to shared engine-disabled `l8,5;l6,3;l7,3` for both profiles, which was neither traced side. The black copied board collapsed to shared `search_only_forced_prepass` `l2,4;l1,5`.
- Durable outcome: do not spend runtime code or retained coverage on those copied `alternating` singleton boards until they reproduce cleanly. They are trace artifacts, not stable local seams.

## Black Forward-Bridge Followup Spirit Repro Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave continued the Fast `forward_bridge_mana_rows` replay after the copied post-search board failed clean reproduction.
- The current replay recovered a black spirit/setup copied seam where frontier drifted to `l0,3;l1,3` while shipping stayed on `l1,5;l3,4;l3,3`.
- The follow-up direct structure probe compared that copied board against the nearby retained black followup-spirit control.
- The copied board did not replay the traced drift. On a clean direct probe, both frontier and shipping collapsed to shared engine-disabled `l0,3;l1,3`, with no head or advisor residue left as a stable target.
- Durable outcome: do not extend retained black followup-spirit controls from that copied board. Shared spirit/setup geometry is not enough when the copied board does not reproduce the shipping-selected root.

## Black Forward-Bridge Post-Search Repro Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave first widened `SMART_AUTOMOVE_VARIANTS=forward_bridge_mana_rows` on `smart_automove_pro_reliability_nonwin_trace_probe` with `duel_filter=vs_shipping_fast`, `repeats=4`, and `games=3`.
- That replay still logged `10` Fast nonwins and recovered a black copied seam where frontier drifted to `l0,6;l1,6` while shipping stayed on `l2,3;l3,4`.
- The follow-up direct structure probe compared that copied board against the nearby retained `BLACK_POST_SEARCH_DUEL_PRO` control.
- The copied board did not replay the traced seam. On a clean direct probe, both frontier and shipping collapsed to shared engine-disabled `l2,3;l3,4`, with no head or advisor residue left to compare against the retained post-search control.
- Durable outcome: do not extend `BLACK_POST_SEARCH_DUEL_PRO` or any other retained black post-search control from that copied board. Shared frontier move shape was a false lead; the copied board is another nonreproducible local artifact.

## Black Alternating Retained Recovery Search-Only Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave recovered the singleton Fast `alternating_mana_rows` seam `l1,6;l1,7` vs shipping `l1,6;l0,5` and compared it directly against the nearby retained `BLACK_RECOVERY_DUEL_FAST` control.
- The coarse lane looked close: both boards touch the same black recovery move `l1,6;l0,5`, and both are black weak-window Fast surfaces.
- The selector/advisor path still diverged. On the live seam, frontier keeps approved `ManaTempo l1,6;l1,7` through `ApprovedReplyRiskGuard`, shipping only wins through `search_only_engine_allowed_head`, and there is no accepted recovery head.
- Retained `BLACK_RECOVERY_DUEL_FAST` is different: it approves `DrainerSafetyRecovery l1,6;l0,5`, accepts that head, and then both frontier and shipping collapse to engine-disabled mana `l4,1;l5,0;mb`.
- Durable outcome: do not extend `BLACK_RECOVERY_DUEL_FAST` into the singleton `alternating` search-only seam. Shared recovery geometry was a false lead; the approved family, shipping win path, and final runtime stage are different.

## Black Alternating Retained Fast Spirit Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave compared the repeated Fast `alternating_mana_rows` seam `l2,7;l1,6` vs shipping `l2,7;l1,8` directly against the last nearby retained black Fast spirit controls: `BLACK_FAST_FLAT_NONWIN` and `BLACK_SPIRIT_BRIDGE_DUEL_FAST`.
- `BLACK_FAST_FLAT_NONWIN` did not match. It is `window=0/deny=0`, pure `SpiritImpact`, and frontier already matches shipping through `ApprovedReplyRiskGuard`.
- `BLACK_SPIRIT_BRIDGE_DUEL_FAST` also did not match. It accepts a `SpiritImpact` head through `ApprovedLegacySelector` and still collapses to engine-disabled mana `l4,9;l5,10;mb`.
- The live repeated seam is different. It is a turn-four black `window=1/deny=1` board where frontier approves outside-shortlist `DrainerSafetyRecovery l2,7;l1,6` through `ApprovedFamilyCompetition`, shipping stays on `ManaTempo l2,7;l1,8`, and the shortlist contains only the two `ManaTempo` siblings beneath rejected head `l2,7;l2,8`.
- Durable outcome: do not extend the nearby retained black Fast spirit package into the repeated `alternating` seam. Shared Fast/black proximity was a false lead; the family, approval path, shortlist shape, and final runtime stage still diverge.

## White Forward-Bridge Retained Fast-Ply10 Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave compared the singleton Fast `forward_bridge_mana_rows` fallback seam `l9,7;l8,6` vs shipping `l9,7;l7,6;l7,7` directly against the nearby retained `WHITE_FAST_PLY10` control, which already ships the same spirit root.
- The coarse surface looked close: both boards are white turn-three `window=1/deny=1` positions, shipping selects `SpiritImpact l9,7;l7,6;l7,7`, and frontier considers the same rejected head `l8,5;l7,6`.
- The approval layer still diverged. On the live seam, frontier approves `SafeSupermanaProgress l9,7;l8,6` through `ApprovedReplyRiskGuard` and keeps the spirit root only shortlist-live, with a preserved safe-progress representative.
- On retained `WHITE_FAST_PLY10`, frontier keeps shipping `SpiritImpact l9,7;l7,6;l7,7` through `ApprovedFamilyCompetition` and preserves the safe-progress root only as a representative.
- Durable outcome: do not extend `WHITE_FAST_PLY10` into the singleton `forward_bridge` fallback seam. Shared shipping root and rejected head were not enough; the live board is a different safe-progress-vs-spirit approval surface.

## White Forward-Bridge Retained Spirit Head Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave reran the narrow Fast nonwin trace on `SMART_AUTOMOVE_VARIANTS=forward_bridge_mana_rows` only far enough to recover the exact repeated white head-accept board.
- On that live board, frontier keeps shipped `SpiritImpact l9,6;l7,6;l7,7` as pre-accept, then accepts same-family head `l9,6;l7,4;l7,3` out of `engine_post_search`; shipping stays on `l9,6;l7,6;l7,7`.
- The follow-up structure probe compared that live board against the nearby retained white head/spirit controls `WHITE_HEAD_FLAT_NONWIN` and `WHITE_TURN_FIVE_SPIRIT_SETUP_PRE_ACCEPT_FAST`.
- `WHITE_HEAD_FLAT_NONWIN` still did not match. It is also a spirit accepted-head surface, but it runs on `window=1/deny=1`, the advisor path is `ApprovedReplyRiskGuard`, and shipping already matches the accepted head.
- `WHITE_TURN_FIVE_SPIRIT_SETUP_PRE_ACCEPT_FAST` also did not match. It is another `window=1/deny=1` spirit setup surface, but frontier keeps shipping and rejects a `ManaTempo` head instead of accepting a same-family spirit head.
- Durable outcome: do not extend the nearby retained white head/spirit package into the repeated Fast `forward_bridge` seam. Shared white spirit setup shape was a false lead; the live board is a different accepted-head mechanism.

## Outer-Edge Widened Normal Replay Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave widened `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows` on `smart_automove_pro_reliability_nonwin_trace_probe` with `duel_filter=vs_shipping_normal`, `repeats=6`, and `games=3`.
- That replay logged `10` Normal nonwins, so `outer_edge` is no longer just the old late-black plus early-white pair.
- The only repeated pair was black `l2,7;l3,8` vs shipping `l1,5;l0,3;l1,3` (`3x`). The other nonwins stayed singleton, including late black `l1,6;l1,5` vs `l2,6;l3,7`, early white `l10,4;l9,5` vs `l9,4;l8,3`, white `l7,4;l6,4` vs `l9,4;l8,3`, black `l1,4;l2,5` vs `l1,4;l1,6;l2,7`, black `l1,4;l2,4` vs `l0,5;l1,6`, and white `l9,3;l8,3` vs `l7,2;l6,1`.
- The follow-up structure probe compared that repeated black seam directly against the nearby retained `BLACK_HEAD_DUEL_SAMPLED_NORMAL` control, which already ships `l1,5;l0,3;l1,3`.
- The retained control still did not match. It is also `window=0/deny=0`, but it keeps the shipped `SpiritImpact` root and rejects head `l0,5;l1,6`; its advisor approves the shipped root through `ApprovedFamilyCompetition`.
- The live repeated seam is different. It already has the shipped `SpiritImpact l1,5;l0,3;l1,3` approved through `ApprovedReplyRiskGuard`, then later accepts head `ManaTempo l2,7;l3,8` out of `engine_post_search`.
- Durable outcome: do not reopen `outer_edge_mana_rows` as a late-black-only spend, and do not extend `BLACK_HEAD_DUEL_SAMPLED_NORMAL` into the new repeated black seam. The widened Normal replay is still mixed, and the only repeated black seam has a different head-accept mechanism from the retained control.

## Black Alternating Retained Fast-Recovery Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave compared the repeated Fast `alternating` seam directly against the retained `BLACK_RECOVERY_DUEL_FAST` control.
- The overlap was only at frontier pre-accept/head family. On the retained control, frontier pre-accept and head both land on `DrainerSafetyRecovery l1,6;l0,5`, but final selected and shipping both collapse to engine-disabled mana `l4,1;l5,0;mb`.
- The retained control is therefore a different surface: `window=0/deny=0`, singleton shortlist, and accepted head into an engine-disabled mana finish.
- The repeated `alternating` seam is different. It is a turn-four black `window=1/deny=1` board where frontier approves `DrainerSafetyRecovery l2,7;l1,6` through `ApprovedFamilyCompetition`, shipping stays on `ManaTempo l2,7;l1,8`, and the shortlist contains only the two `ManaTempo` siblings beneath rejected head `l2,7;l2,8`.
- Durable outcome: do not extend `BLACK_RECOVERY_DUEL_FAST` into the repeated `alternating` seam. Shared `DrainerSafetyRecovery` at pre-accept was a false lead; the runtime stage, final selected root, shortlist shape, and exact-opportunity context are different.

## Black Alternating Retained Branch Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave first reran `smart_automove_pro_reliability_nonwin_trace_probe` on `SMART_AUTOMOVE_VARIANTS=alternating_mana_rows` with `duel_filter=vs_shipping_fast` to recover the exact repeated black board on the current corpus.
- The repeated seam still reproduced cleanly: on the recovered turn-four board, frontier played `l2,7;l1,6` while shipping stayed on `l2,7;l1,8`.
- The follow-up structure probe compared that board against the nearby retained black fast/pro controls: `BLACK_BRANCH_DUEL_SAMPLED_FAST`, `BLACK_SIBLING_DUEL_SAMPLED_PRO`, and `BLACK_FOLLOWUP_DUEL_SAMPLED_NORMAL`.
- None of them matched. The live repeated seam is a `window=1/deny=1` board where frontier approves `DrainerSafetyRecovery l2,7;l1,6` through `ApprovedFamilyCompetition`, shipping stays on `ManaTempo l2,7;l1,8`, and the reply-risk shortlist contains only the two `ManaTempo` siblings beneath a rejected head `l2,7;l2,8`.
- `BLACK_BRANCH_DUEL_SAMPLED_FAST` is different: it is a wide-shortlist `SpiritImpact` own-setup board approved through `ApprovedLegacySelector`.
- `BLACK_SIBLING_DUEL_SAMPLED_PRO` is different again: it is `SpiritImpact` through `ApprovedFamilyCompetition` with one preserved `ManaTempo` representative.
- `BLACK_FOLLOWUP_DUEL_SAMPLED_NORMAL` is also different: it is a `window=0/deny=0` `ManaTempo` family-competition board that still keeps shipping.
- Durable outcome: do not extend the retained black branch/followup package into the repeated `alternating` seam. Shared black-early labels were not enough; the family, context, shortlist density, and approval path still diverge.

## Black Outer-Edge Retained Late Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave first reran `smart_automove_pro_reliability_nonwin_trace_probe` on `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows` with `duel_filter=vs_shipping_normal` to recover the exact late black board on the current corpus.
- The clean late seam still reproduced: on the recovered turn-eight board, frontier played `l1,6;l1,5` while shipping stayed on `l2,6;l3,7`.
- The follow-up structure probe compared that board against the retained sampled black late package: `BLACK_LATE_MANA_DUEL_SAMPLED_NORMAL`, `BLACK_LATE_HEAD_ACCEPT_DUEL_SAMPLED_NORMAL`, and `BLACK_HEAD_ACCEPT_SEARCH_ONLY_DUEL_SAMPLED_NORMAL`.
- The closest retained control still did not match. `BLACK_LATE_MANA_DUEL_SAMPLED_NORMAL` is also late black `ManaTempo` on `window=1/deny=1`, but it keeps shipping `l0,7;l1,6` through `ApprovedFamilyCompetition` on a smaller shortlist that mixes `SafeOpponentManaProgress`; frontier does not drift off shipping there.
- The other retained controls are further away: `BLACK_LATE_HEAD_ACCEPT_DUEL_SAMPLED_NORMAL` is a `SafeSupermanaProgress` preserved-root board, and `BLACK_HEAD_ACCEPT_SEARCH_ONLY_DUEL_SAMPLED_NORMAL` is a `DrainerSafetyRecovery` search-order board.
- The live `outer_edge` seam is different. Shipping `l2,6;l3,7` stays inside a dense reply-risk shortlist with several spirit siblings, frontier still approves lower-ranked `l1,6;l1,5` through `ApprovedReplyRiskGuard`, and the head is rejected.
- Durable outcome: do not extend the retained sampled black late package into the clean late `outer_edge` seam. Shared “late black” labeling was not enough; the approval path, shortlist shape, and family mix still diverge.

## White Forward-Bridge Retained Sampled Head-Accept Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave compared the repeated white `forward_bridge` seam directly against the retained sampled white head-accept controls in Pro, Normal, and Fast.
- The retained sampled controls did not collapse to one reusable package.
- The retained Pro control is a pure `ManaTempo` `ApprovedReplyRiskGuard` board on `window=0/deny=0`; frontier and shipping both stay on `l9,6;l8,5`, and the head is rejected.
- The retained Normal control is closer but still different: it is `SpiritImpact` on `window=1/deny=1`, shipping `l9,5;l7,4;l8,3` is advisor-approved through `ApprovedFamilyCompetition`, and frontier still keeps shipping while rejecting a lower-ranked spirit head.
- The retained Fast control is different again: it is `SpiritImpact` through `ApprovedReplyRiskGuard` with a singleton shortlist, one preserved sibling representative, and a rejected vulnerable `ManaTempo` head.
- The repeated `forward_bridge` seam is none of those. It is a `window=0/deny=0` `SpiritImpact` own-setup board where shipping `l9,6;l7,6;l7,7` is advisor-approved through `ApprovedFamilyCompetition`, the shortlist stays full of spirit own-setup siblings, and frontier only loses later by accepting head `l9,6;l7,4;l7,3`.
- Durable outcome: do not extend the retained sampled white head-accept package into the repeated `forward_bridge` seam. Matching “head-accept” labels were not enough; the family mix, context, shortlist density, and approval path still diverge.

## White Forward-Bridge Head Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave first reran `smart_automove_pro_reliability_nonwin_trace_probe` on `SMART_AUTOMOVE_VARIANTS=forward_bridge_mana_rows` with `duel_filter=vs_shipping_fast`, `repeats=4`, and `games=3` to recover the live board on the current corpus.
- The repeated white seam still reproduced cleanly: `l9,6;l7,4;l7,3` vs shipping `l9,6;l7,6;l7,7` appeared `3x` inside `10` Fast nonwins.
- The follow-up structure probe then compared that board against the retained white turn-five head-reject controls.
- The retained controls did not match. They are `window=1/deny=1` pure `ManaTempo` boards approved through `ApprovedReplyRiskGuard`, and frontier keeps shipping while rejecting a weaker head.
- The repeated `forward_bridge` seam is different. It is a `window=0/deny=0` `SpiritImpact` board where shipping `l9,6;l7,6;l7,7` is advisor-approved through `ApprovedFamilyCompetition`, the shortlist stays full of spirit own-setup siblings, and frontier only loses later by accepting head `l9,6;l7,4;l7,3`.
- Durable outcome: do not extend the retained white turn-five head-reject package into the repeated `forward_bridge` seam. Matching head labels were a false lead; the family, context, advisor path, and shortlist shape are different.

## White Forward-Bridge Search-Order Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave tested the last plausible white reuse path by comparing the `forward_bridge` safe-progress seam directly against the retained early-white engine-disabled/search-only controls.
- The retained boards did not match. `white_early_engine_disabled_normal` still approved `ManaTempo l8,5;l7,6` through `ApprovedReplyRiskGuard`, while shipping `SpiritImpact l9,5;l8,3;l7,4` stayed candidate-live but outside shortlist alongside many other spirit roots. `white_negative_deny_search_only_selected_rank_normal` was different again: it approved `ManaTempo l9,4;l8,3` through `ApprovedReplyRiskGuard` and had no spirit candidates live at all.
- The `forward_bridge` seam is different from both. It approves `SafeSupermanaProgress l9,6;l8,7`, while shipping `SpiritImpact l9,6;l7,7;l8,8` stays candidate-live and shortlist-live with other spirit siblings.
- Durable outcome: do not extend the retained white engine-disabled/search-only package into `forward_bridge`. Matching stage labels were a false lead; the approved family and spirit-candidate shape are different.

## White Safe-Progress Retained Extension Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave tested the only plausible retained white reuse path left on the `forward_bridge` safe-progress seam by comparing it directly against the retained Pro board that already ships `l9,6;l8,7`.
- The overlap was only the final move. The retained `WHITE_POST_SEARCH_DUEL_PRO` board is a quiet `ManaTempo` approval on `window=0/deny=0`; frontier and shipping both select `l9,6;l8,7`, and there are no shortlist-live `SpiritImpact` roots at all.
- The `forward_bridge` seam is different. It is a `window=1/deny=1` board where frontier approves `SafeSupermanaProgress l9,6;l8,7`, while shipping `SpiritImpact l9,6;l7,7;l8,8` stays candidate-live and shortlist-live with several spirit siblings.
- Durable outcome: do not extend the retained white `l9,6;l8,7` post-search control into the `forward_bridge` seam. Matching final moves were a false lead; the family mix and shortlist shape are different.

## White-Black Safe-Progress Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave tested the only plausible cross-color follow-up left after the black safe-progress split: whether the white `forward_bridge` safe-progress seam `l9,6;l8,7` vs shipping `l9,6;l7,7;l8,8` and the Pro black setup-lane seam `l1,6;l2,7` vs shipping `l1,5;l2,3;l1,2` were really the same `SafeSupermanaProgress`-over-`SpiritImpact` mechanism.
- They matched only at the approved family. On both boards, frontier still approved `SafeSupermanaProgress` through `ApprovedReplyRiskGuard`.
- Under that, they split again. On the white seam, shipping `SpiritImpact l9,6;l7,7;l8,8` stayed candidate-live and shortlist-live, and several other spirit siblings stayed live with it. On the Pro black seam, the entire `SpiritImpact` family was filtered out before shortlist, and shipping `l1,5;l2,3;l1,2` survived only as a preserved outside-candidate representative.
- Durable outcome: do not reopen a shared cross-color safe-progress-vs-spirit runtime spend across these two seams. Shared `ApprovedReplyRiskGuard` approval was a false lead; the candidate-set failure modes are different.

## Black Safe-Progress Setup Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave tested the only plausible shared black follow-up left after the Pro black recurrence split: whether the old black progress residue `l7,1;l9,3` vs shipping `l1,5;l2,7;l1,8` and the newer Pro black setup-lane seam `l1,6;l2,7` vs shipping `l1,5;l2,3;l1,2` were really the same `SafeSupermanaProgress`-over-`SpiritImpact` mechanism.
- They matched only at the approved family. On both boards, frontier still approved `SafeSupermanaProgress` through `ApprovedReplyRiskGuard`.
- Under that, they split again. On `black_progress_vs_setup_residue`, the shipping root `l1,5;l2,7;l1,8` was still inside the candidate set but outside the shortlist, and a stronger full-pool own-setup `SpiritImpact l1,5;l3,7;l2,8` was also candidate-live above it. On the Pro setup-lane seam, shipped `SpiritImpact l1,5;l2,3;l1,2` was already a preserved rank-2 representative outside the candidate set, and the whole `SpiritImpact` family was filtered out before shortlist.
- Durable outcome: do not reopen a shared black safe-progress-vs-spirit runtime spend across these two seams. Shared `ApprovedReplyRiskGuard` approval was a false lead; the candidate-set failure modes are different.

## Pro Black Recurrence Structure Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave tested the only plausible shared Pro follow-up left after the widened blocker replay by comparing the two repeated black seams directly: repeated `alternating_mana_rows` black `l2,7;l1,6` vs `l2,7;l1,8`, and repeated Pro black `l1,6;l2,7` vs shipping `l1,5;l2,3;l1,2`.
- The seams only matched on the surface. Both boards were `window=1/deny=1`, frontier still returned through `engine_post_search`, and shipping still stayed `engine_disabled`.
- Under that, they split again. The `alternating` board still approved outside-shortlist `DrainerSafetyRecovery l2,7;l1,6` through `ApprovedFamilyCompetition` while the shortlist itself stayed on `ManaTempo`. The `l1,6;l2,7` board instead approved shortlisted `SafeSupermanaProgress l1,6;l2,7` through `ApprovedReplyRiskGuard`, while shipping `SpiritImpact l1,5;l2,3;l1,2` survived only as a preserved family representative outside the candidate set.
- Durable outcome: do not reopen a shared Pro-only black runtime spend across these two repeated seams. Shared stage/context was a false lead; the advisor and candidate-set failure modes are different.

## Pro-Only Blocker Replay Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave widened the Pro-only replay over the active blocker set instead of probing one seam at a time: `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows` with `smart_automove_pro_reliability_nonwin_trace_probe`, `duel_filter=vs_shipping_pro`, `repeats=6`, and `games=3`.
- The replay logged `11` Pro nonwins across `36` games. It did not promote the new white seams: `l9,7;l8,8` vs `l9,7;l8,7` and `l6,7;l7,6` vs `l6,7;l7,7` each appeared once.
- Instead, the Pro surface broadened into mixed black residue. Black `l2,7;l1,6` vs `l2,7;l1,8` repeated twice, black `l1,6;l2,7` vs shipping `l1,5;l2,3;l1,2` repeated twice, and the rest stayed singleton across both colors and across `engine_post_search` plus `engine_disabled` surfaces.
- Durable outcome: do not reopen the new white Pro seams as standalone spends. The widened Pro-only blocker surface is still mixed, and the repeated recurrence is on the black side rather than the white side.

## White Late Mana Sibling Surface Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave tested the only nearby retained extension candidate for the new Pro white `l6,7;l7,6` vs `l6,7;l7,7` seam by comparing it directly against the retained late white mana sibling duel-normal board.
- They were not the same surface. The retained board ships and frontier-aligns on `l7,7;l6,5;l6,6` through `engine_post_search`, keeps a live head plan, and mixes `SpiritImpact` against `SafeSupermanaProgress` under `window=2/deny=2`.
- The new seam is different. It still ships `l6,7;l7,7`, frontier drifts to `l6,7;l7,6`, the selector is already `engine_disabled`, there is no head plan, and the whole top root pool stays pure `ManaTempo` under `window=0/deny=0`.
- Durable outcome: do not reopen the retained late white mana sibling path as an extension candidate for `l6,7;l7,6` vs `l6,7;l7,7`. The retained board is a late spirit/setup surface, and the new seam is an early pure-ManaTempo ordering surface.

## Same-Family Late Mana Residual Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave tested the only plausible shared scoring angle left inside the current same-family `ManaTempo` drifts: late black `outer_edge` plus the new Pro white `l9,7;l8,8` vs `l9,7;l8,7` and `l6,7;l7,6` vs `l6,7;l7,7` seams.
- The black `outer_edge` seam and the white `l9,7;l8,8` vs `l9,7;l8,7` seam both came back as exact ties at the scoring layer. They had zero residual delta, zero `search_eval` delta, and identical `TurnEngineUtility` despite different selected roots.
- The white `l6,7;l7,6` vs `l6,7;l7,7` seam did not match that shape. It carried a real residual delta (`-112`) concentrated in `spirit_action_utility` and `mana_close_to_same_pool`.
- Durable outcome: do not reopen a shared same-family late mana scoring spend. Two seams are tie-order surfaces and the third is a different residual-scoring surface, so there is still no common mechanism.

## All-Blocker Recurrence Trace Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave widened recurrence tracing over the full active blocker set instead of probing one blocker at a time: `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows` with `smart_automove_pro_reliability_duel_trace_probe`, `repeats=4`, and `games=3`.
- The broader trace did not collapse the residue. Across `24` games per duel, Pro logged `3` regressions, Normal `3`, and Fast `7`.
- Every per-duel `repeated_move_pairs` entry stayed singleton. The isolated repeated white head-accept seam and isolated black mana sibling seam stopped dominating once the full blocker set was traced together.
- The widened trace also broadened the blocker surface instead of shrinking it. Alongside the known `alternating` and `forward_bridge` misses, it exposed extra singleton Pro/Normal seams such as `l9,7;l8,8` vs `l9,7;l8,7`, `l6,7;l7,6` vs `l6,7;l7,7`, and `l9,6;l9,4;l8,4` vs `l9,6;l7,8;l8,8`.
- Durable outcome: do not reopen a shared spend across `outer_edge_mana_rows`, `alternating_mana_rows`, and `forward_bridge_mana_rows`. Keep the broadened recurrence counts and no-go lesson, discard the logs.

## Blocker Hotspot Fingerprint Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave first reran bounded nonwin traces only to recover the current black blocker boards on clean logs: `outer_edge_mana_rows` on `vs_shipping_normal` still logged `2` nonwins, including late black `l1,6;l1,5` vs shipping `l2,6;l3,7`, and `alternating_mana_rows` on `vs_shipping_fast` logged `4` nonwins, including black `l2,7;l1,6` vs shipping `l2,7;l1,8`.
- A temporary three-board hotspot fingerprint probe then compared late black `outer_edge`, black `alternating`, and the repeated white `forward_bridge` head-accept board.
- The commonality was only the expected frontier-vs-search-only cost shape. On all three boards, frontier and shipping enumerated the same selector pool sizes, while frontier paid extra exact/pickup/secure-mana/tactical-spirit work on top of the shipping search-only baseline.
- That was not a shared runtime mechanism. `outer_edge` still drifted at `pre_accept` into lower-ranked same-family `ManaTempo`, `alternating` still drifted at `pre_accept` into outside-family `DrainerSafetyRecovery`, and repeated white `forward_bridge` still kept shipping at `pre_accept` and only flipped at head acceptance. The exact contexts also stayed split between `window=1/deny=1` and `window=0/deny=0`.
- Durable outcome: do not reopen a shared exact/selector hotspot spend across the remaining repeated seams. Keep the clean-board FEN recovery and the no-go lesson, discard the temporary probe code.

## White Forward-Bridge Structure Probe Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave targeted the only plausible shared white runtime angle left after the isolated Fast traces: whether the repeated `forward_bridge_mana_rows` white seam and its sibling white misses were really one late spirit/setup mechanism.
- They were not. On the repeated seam, shipping `l9,6;l7,6;l7,7` was both legacy-selected and advisor-approved as `SpiritImpact`, and frontier only lost later by accepting head `l9,6;l7,4;l7,3`.
- The `l9,6;l8,7` seam was different. Shipping `l9,6;l7,7;l8,8` stayed in the shortlist, but frontier approved `SafeSupermanaProgress l9,6;l8,7` through `ApprovedReplyRiskGuard`.
- The `l9,7;l8,6` seam was different again. It routed through `score_window_tactical_fallback`, preserved a safe-progress representative, and did not even stay on the same runtime path as the repeated head-accept seam.
- Durable outcome: do not reopen a shared white `forward_bridge_mana_rows` spend. The white seams look related from the move list, but they split across head acceptance, reply-risk approval, and runtime fallback layers, so there is still no common runtime mechanism.

## Black Late-Mana Structure Probe Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The wave targeted the only plausible shared black runtime angle left after the isolated Fast traces: whether the unresolved Normal `outer_edge_mana_rows` black seam and the repeated Fast `alternating_mana_rows` black seam were really the same late-mana mechanism.
- They were not. On `outer_edge_mana_rows` black, shipping `l2,6;l3,7` was still the legacy-selected, reply-risk-shortlisted `ManaTempo` root, and frontier still drifted to lower-ranked `ManaTempo l1,6;l1,5` through `ApprovedReplyRiskGuard`.
- On the repeated `alternating_mana_rows` black seam, shipping `l2,7;l1,8` was also the legacy-selected `ManaTempo` root, but frontier did not lose to another shortlisted `ManaTempo`. It approved outside-shortlist `DrainerSafetyRecovery l2,7;l1,6` through `ApprovedFamilyCompetition`, while the ordered shortlist itself stayed on `ManaTempo`.
- Durable outcome: do not reopen a shared black late-mana spend across `outer_edge_mana_rows` and `alternating_mana_rows`. The move pairs look related, but the advisor/shortlist failure modes are different, so there is still no common runtime mechanism.

## Fast Variant-Isolation Trace Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- `forward_bridge_mana_rows` was isolated first with `smart_automove_pro_reliability_duel_trace_probe`. Across `24` Fast games it logged `8` regressions, `4` improvements, and `12` flat games. The repeated pair did get stronger: the white head-accept seam `l9,6;l7,4;l7,3` vs shipping `l9,6;l7,6;l7,7` appeared `3` times.
- That still was not one-variant collapse. The same `forward_bridge_mana_rows` trace kept five singleton seams behind the repeated pair, including black mana/progress misses (`l0,6;l1,6` vs `l2,3;l3,4`, `l7,1;l9,3` vs `l1,5;l3,4;l2,3`) and extra white spirit/setup siblings (`l9,6;l8,7` vs `l9,6;l7,7;l8,8`, `l9,7;l8,6` vs `l9,7;l7,6;l7,7`, `l9,5;l9,6` vs `l7,3;l6,2`).
- `alternating_mana_rows` was then isolated with `smart_automove_pro_reliability_nonwin_trace_probe` on `vs_shipping_fast`. It logged `7` Fast nonwins. The black mana sibling seam `l2,7;l1,6` vs `l2,7;l1,8` repeated `2` times, but the rest of the variant still stayed singleton: `l0,10;l0,9` vs `l4,0;l5,0;mb`, `l1,6;l1,7` vs `l1,6;l0,5`, `l2,5;l0,5;l1,6` vs `l2,5;l4,7;l3,8`, `l8,5;l10,5;l9,4` vs `l8,5;l6,3;l7,2`, and `l2,4;l1,5` vs `l2,4;l1,3`.
- Durable outcome: do not spend isolated `forward_bridge_mana_rows` or `alternating_mana_rows` runtime code yet. Each variant now has one repeated pair, but neither variant collapsed to a single runtime mechanism.

## Fast Recurrence Trace Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The widened `smart_automove_pro_reliability_duel_trace_probe` reran `alternating_mana_rows,forward_bridge_mana_rows` against `vs_shipping_fast` with `repeats=4` and `games=3`, for `24` Fast games total. The result was still mixed: `8` regressions, `7` improvements, and `9` flat games.
- Only one move pair repeated more than once: the already-archived white branch head-accept seam `l9,6;l7,4;l7,3` vs shipping `l9,6;l7,6;l7,7`, which appeared `2` times.
- Every other Fast regression pair stayed singleton, including the black `alternating_mana_rows` mana/recovery misses, the black `forward_bridge_mana_rows` mana sibling misses, and the white `l9,6;l8,7` vs `l9,6;l7,7;l8,8` spirit/setup seam.
- Durable outcome: do not reopen a standalone white late head-accept blocker from this trace alone. The repeated pair is real, but the wider Fast blocker set still does not collapse to one runtime mechanism.

## Late Black Outer-Edge Probe Wave

- No runtime challenger was cut in this wave. The spend stayed diagnostic-only.
- The remaining Normal `outer_edge_mana_rows` miss narrowed to one late black turn-eight advisor seam. Shipping `l2,6;l3,7` was already the legacy-selected, reply-risk-shortlisted `ManaTempo` root, while frontier still approved lower-ranked `l1,6;l1,5` through `ApprovedReplyRiskGuard`.
- That seam is real but still too local to spend on by itself. It only explains one remaining Normal board and does not collapse the sampled Fast residue.
- Explicit Fast replay on `alternating_mana_rows,forward_bridge_mana_rows` still logged `5` nonwins across both colors. The residue mixed `engine_post_search` divergences with head-accepted divergences and included bomb/setup-style misses, so it still does not support one shared runtime hypothesis.
- Durable outcome: do not spend a direct late-black same-family legacy-alignment override yet. Keep the lesson, discard the probe code, and clean logs/stamps.

## White Mana-Only Legacy-Progress Wave

- No runtime challenger survived this wave. The local candidate added a narrow `ApprovedLegacySelector` carve-out inside the white turn-three legacy-alignment advisor path for mana-only, `window=0`, `deny=0`, positive-safety boards where legacy already preferred a vulnerable safe-progress root over approved non-vulnerable `ManaTempo`.
- The line did move the clean local seam. On the explicit `outer_edge_mana_rows` white board, frontier aligned from `l10,4;l9,5` to shipping `l9,4;l8,3`, and the explicit `vs_shipping_normal` outer-edge trace dropped from `2` nonwins to `1`, leaving only the late black miss.
- That still was not promotable. The canonical loop cleared `guardrails`, `variant-smoke`, `pro-triage`, and `runtime-preflight`, then sampled `pro-reliability` still failed at Pro `1.0000`, Normal `0.9167`, Fast `0.8333`; confidence `0.9998 / 0.9968 / 0.9807`; frontier average move times stayed below `200ms`.
- The remaining sampled blockers on that line were still broad enough to kill it: Normal `outer_edge_mana_rows` stayed at `0.5000`, and Fast `alternating_mana_rows` plus `forward_bridge_mana_rows` stayed at `0.5000`; `classic` and `corner_chain_mana_rows` were clean on that sample.
- Durable outcome: do not reopen the direct white turn-three mana-only legacy-progress override on the positive-safety `window=0/deny=0` surface. Keep the lesson that it only removed the clean white half of `outer_edge_mana_rows`, discard the runtime code and temporary retained board.

## Variant-Blocker Trace Wave

- No runtime challenger was cut in this wave. The spend was diagnostic-only: explicit-variant `smart_automove_pro_reliability_nonwin_trace_probe` replays against the current sampled blockers on the kept `frontier_pro_v2_guarded` tree.
- Normal `outer_edge_mana_rows` did reproduce, but not as one seam. The bounded `vs_shipping_normal` replay logged `2` nonwins: a late black turn-eight `engine_post_search` miss (`l1,6;l1,5` vs shipping `l2,6;l3,7`) and an early white turn-three action+mana `engine_post_search` miss (`l10,4;l9,5` vs shipping `l9,4;l8,3`).
- Fast explicit `classic,forward_bridge_mana_rows,corner_chain_mana_rows` also stayed mixed. The bounded `vs_shipping_fast` replay logged `5` nonwins, all on `forward_bridge_mana_rows` and `corner_chain_mana_rows`; `forward_bridge` stayed mainly `engine_post_search`, while `corner_chain` also exposed an `engine_disabled` ordering miss.
- Standalone Fast `classic` explicit replay produced `0` nonwins in the same bounded probe, so the sampled `classic` miss is not yet a stable direct target.
- Durable outcome: do not spend runtime code on the mixed `outer_edge_mana_rows` plus Fast `forward_bridge_mana_rows` / `corner_chain_mana_rows` residue until a later clean probe collapses it to one shared mechanism. Keep the trace counts and stage split, discard the logs and stamps.

## Late Black Shipping-Fallback Expansion Wave

- No runtime challenger survived this wave. The local candidate extended `select_late_black_search_fallback_inputs` into late weak-window black turn-start and mana-only states.
- The line did fix the traced sampled black boards directly. The turn-eight weak-window Normal board aligned to shipping mana `l2,6;l3,7`, and the turn-ten weak-window Fast board aligned to shipping recovery `l1,6;l0,5`.
- That still was not promotable. The canonical loop cleared `guardrails`, `variant-smoke`, `pro-triage`, and `runtime-preflight`, then sampled `pro-reliability` failed at Pro `1.0000`, Normal `0.9167`, Fast `0.7500`; confidence `0.9998 / 0.9968 / 0.9270`; frontier average move times stayed below `200ms`.
- The sampled failure rotated instead of shrinking. `alternating_mana_rows` recovered, but Fast residue moved to `classic`, `corner_chain_mana_rows`, and `forward_bridge_mana_rows`, while Normal `outer_edge_mana_rows` still failed at `0.5000`.
- Durable outcome: do not reopen a broad late-black shipping-fallback expansion just because it fixes the currently traced black sampled boards. Keep the lesson, discard the runtime code and temporary retained boards.

## Sampled-Pass Late-Ply Override Wave

- No runtime challenger survived this wave. The local candidate paired a white late spirit-setup branch-head reject with black weak-window mana-lane advisor overrides and a weak-window carve-out in black followup competition.
- The line did move the targeted sampled late boards. Two new retained boards were briefly promoted locally, and sampled `pro-reliability` passed at Pro `1.0000`, Normal `0.9167`, Fast `0.9167`, with confidence `0.9998 / 0.9968 / 0.9968`.
- All-variant `pro-reliability-confirm` killed it hard: Pro `0.6667`, Normal `0.7292`, Fast `0.6667`, with confidence `0.9853 / 0.9990 / 0.9853`. Frontier average move times stayed below `200ms`, so the failure was duel strength, not runtime cost.
- Follow-up `smart_automove_pro_reliability_nonwin_trace_probe` on the confirm corpus logged `16` Pro nonwins across multiple variants and both colors, with repeated late `engine_post_search` / head-accept divergences from shipping. That was broad enough to kill the line instead of stacking more local exceptions.
- Durable outcome: do not reopen this combined late white head-block plus black weak-window mana-lane package. Keep the confirm metrics and lesson, discard the runtime code and temporary retained boards.

## Black Progress Material Plus Rank Wave

- No runtime challenger survived this wave. The local candidate combined frontier-only scoped material/cooldown dampening on the black turn-six weak-window residue with a higher-scoring setup-rank exception in the advisor path.
- The focused probe moved the local target: frontier aligned from safe progress `l7,1;l9,3` to shipping setup `l1,5;l2,7;l1,8`, and the retained turn-ten setup-control board stayed aligned.
- Cheap gates were clean enough to spend the sampled duel: `guardrails`, `variant-smoke`, `pro-triage`, and `runtime-preflight` all passed.
- Sampled retained reliability killed it hard: Pro `0.5000`, Normal `0.5000`, Fast `0.8333`, with confidence `0.0000 / 0.0000 / 0.9807`. Frontier average move times were `148.34ms / 179.83ms / 161.71ms`, so cost was not the failure.
- Durable outcome: even a mechanism that both removes the material gap and moves advisor approval is only local board repair. Do not reopen this residue with scoped material-plus-rank advisor changes unless there is new evidence for retained multi-variant duel strength.

## Black Progress Selector-Layer Probe

- No runtime challenger was cut in this wave. The retained change is diagnostic-only: `black_progress_residual_weight_attribution_probe` now also replays the target under material-dampened weights and reports the advisor/selector layer.
- The replay kept selecting frontier safe progress `l7,1;l9,3` through `frontier_execute` / `engine_post_search` even with `fainted_mon` and `fainted_cooldown_step` zeroed. The advisor approved that same root as `SafeSupermanaProgress:ApprovedReplyRiskGuard:rank0`.
- The shipping setup root `l1,5;l2,7;l1,8` was not missing; it appeared in the reply-risk shortlist at rank `10`.
- Durable outcome: after the material/cooldown gap is removed, this residue is an advisor reply-risk approval problem, not a final-selector mystery. Do not spend on final-selector-only or material-only changes; any future spend must prove why the reply-risk guard should prefer the setup root while preserving retained setup-control boards.

## Black Progress Material Dampening Wave

- No runtime challenger survived this wave. The target was the black Fast progress-vs-setup residue where frontier keeps safe progress `l7,1;l9,3` and shipping chooses the setup root `l1,5;l2,7;l1,8`.
- The existing attribution probe confirmed the local residual score story: on the target, safe progress beat setup mostly through `fainted_mon` and `fainted_cooldown_step`.
- A narrow scoring cut zeroed those two material/cooldown terms only for the scoped black turn-six action+mana window/deny state. It did reduce the target residual deltas from `843/778` after-root to `83/18`, and from `862/797` at worst reply to `102/37`.
- The line was still behaviorally inert. Final frontier selection stayed on `l7,1;l9,3`, while the retained turn-ten setup-control board stayed on shipping/setup as intended. Because `target_changed=0`, the runtime code was discarded before canonical gate spend.
- Durable outcome: material/cooldown explains the local residual valuation, but material-only scoring is not a live challenger. Future work must explain the selector layer that keeps safe progress after the residual gap is mostly removed.

## Black Recovery Static-Exact Wave

- No runtime challenger survived this wave. The target was the remaining `black_recovery_branch` seam where pairwise attribution showed static exact evaluation could flip the local reply-floor ordering toward shipping.
- A broad scoped static-exact cut was killed before canonical spend because it did not recover shipping; on the local retained board it selected the new spirit sibling `l1,5;l2,6` instead of shipping `l6,0;l6,1`.
- A narrower reply-floor-only exact cut did align that board to shipping and passed `guardrails`, `variant-smoke`, retained `pro-triage`, and `runtime-preflight`.
- Retained sampled duel strength killed it. `pro-reliability` failed at Pro `0.5000`, Normal `0.5000`, Fast `0.9167`, with confidence `0.0000 / 0.0000 / 0.9968`; frontier average move times were `153.71ms / 180.61ms / 159.35ms`, so cost was acceptable but strength was not.
- Durable outcome: static exact remains useful as an attribution signal but not as a direct `black_recovery_branch` runtime patch. Do not reopen broad static exact or reply-floor-only exact without a new mechanism that improves retained multi-variant duel strength.

## Promoted Retained Package And Residual Classification Wave

- On `2026-04-23`, the retained package around `frontier_pro_v2_guarded` was refreshed and promoted as the only shipped Pro path.
- The promoted package kept the narrow white and black repairs that survived retained and confirm gates: white turn-three no-action recovery, early-white engine-disabled wrapper fallback, nonnegative-deny and selected-rank search-only fallbacks, white turn-five head guards, white confirm ProV1 tiebreaks, black late safe-mana/setup advisor repairs, and the black turn-six guarded legacy mana override.
- Release verification passed with public Pro still wired through `select_frontier_pro_v2_guarded_inputs`; experimentation stayed gated behind `#[cfg(test)]`.
- The full canonical confirm loop passed with Pro `0.9688`, Normal `1.0000`, Fast `0.9688`, confidence `1.0000`, and average move times below `200ms`.
- The remaining residuals were classified as no-go at the current selection layer:
  - `black_recovery_branch` is only reopened by a scoped static-exact cost/reliability hypothesis; local legacy and shipping-mirror fallbacks were killed.
  - The black Fast progress-vs-setup residue is explained by residual material/cooldown valuation; shortlist, advisor, and wrapper mirrors were killed.
  - White search-order residue is not solved by root reachability, root rank, broad ProV1 reroute, wrapper config mirroring, or rerank-cap clamps; future work must separate unresolved siblings from retained vulnerable guards below the current shortlist/reply-risk surface.
- Durable outcome: the live board was compressed back to one retained frontier, one baseline, one next-hypothesis slot, and explicit no-go notes. Detailed probe diaries remain in git history, not in the live operator flow.

## Reference Frontier Wave

- Early retained turn-engine work established the shared infrastructure that later made guarded `ProV2` possible.
- `runtime_pro_turn_engine_v1` belongs to this wave. It is archive-only reference history now, not an active experiment target.

## Replay-Diary Wave

- Many Apr 8-9 duel replays (`v7` through `v75`) were useful for classification but not for direct code spend.
- The common stop reason was the same: exact move pairs stayed count `1`, or the repeated pair had no retained Pro foothold.
- Durable outcome: keep the retained probes, compress the diary, and let git history hold the seed-by-seed detail.

## Wrapper And Fallback Wave

- Several opening-book, early-white, and forced-prepass wrapper cuts were tried after promotion work stalled.
- Some narrow guards survived as part of the shipping guarded path.
- The broader lesson from this wave is negative: wrapper-only repairs were rarely promotable on their own and often failed to move the direct `vs shipping Normal` wall.

## Retained Seam-Mapping Wave

- This wave produced the durable retained fixtures that future work still uses: black action+mana, black mana bridge, black spirit bridge, negative-deny, white safe-progress, and the closed regression seams.
- Most production cuts from this wave were killed because they solved only one local family.
- Durable outcome: keep the fixtures and the probes, not the abandoned production branches.

## Unified Root-Advisor Promotion Wave

- The winning structural change was the unified `ProV2` root advisor that centralized shortlist shaping, family preservation, omitted-root handling, macro-root injection, and conservative post-search verification.
- The final promotable cut was narrow: on quiet early-black boards, advisor approval had to stop preferring a weaker plain-spirit sibling over a stronger own-setup `SpiritImpact` root already in the shortlist.
- Durable outcome: `frontier_pro_v2_guarded` survived as the retained guarded frontier, but shipping stayed on the separate search-only path.

## Pro-Only Surface Cleanup Wave

- After promotion, the active experiment surface was shrunk to two selectable profiles: `shipping_pro_search` and `frontier_pro_v2_guarded`.
- Calibration/reference profiles, curated-pool smoke plumbing, and compatibility-only docs were archived.
- Legacy flat experiment logs and the old `target/experiment-runs/runtime_preflight_*.stamp` compatibility path were removed.
- Durable outcome: future work starts from a smaller Pro-only workflow; archived profiles and stages stay documented here, not in the live runbook.

## Closed-Surface Archive Cleanup Wave

- On `2026-04-21`, the archived regression seams `primary_spirit_setup`, `primary_pvs_sensitive_search`, and `primary_black_reliability_opening_3_ply4` were removed from the live `primary_pro` pack and from the default retained probes.
- Their history stays here only. They are no longer part of the live retained experiment surface or the default operator diagnostics.
- Durable outcome: `primary_pro` now means current live retained seams only, and closed-surface history no longer leaks into the active workflow.

## Quiet-Guarded Challenger Wave

- `frontier_pro_v3_quiet_guarded` tried to spend on live non-win seams around quiet mana acceptance and vulnerable plain-spirit reentry.
- The cut only passed `pro-triage` after 2 duel-derived live non-win boards were promoted into retained `primary_pro`.
- Direct evidence killed it: `pro-reliability` vs shipped `frontier_pro_v2_guarded` came back `0.3333 / 0.8333 / 0.8333`, so the candidate code was discarded.
- Durable outcome: keep the new retained seams and the live non-win root probe, but require direct frontier-vs-shipping wins before reopening this hypothesis family.

## White-Guarded Challenger Wave

- `frontier_pro_v3_white_guarded` spent on three white-only live seams: late quiet head acceptance, safe mana sibling selection on the exact split trace, and turn-3 vulnerable-window recovery.
- The cut really did fix the first two probe boards: `vs_shipping_pro_opening_reply_white` and `vs_shipping_pro_white_split_trace` matched shipping after the local guards landed.
- It was still not promotable because `vs_shipping_normal_white_head_acceptance` never left `search_only_engine_allowed_head`; shipping still reached `search_only_forced_prepass` on the same board.
- Durable outcome: keep the probe improvements and the lesson that the unresolved turn-3 search-only handoff is the real remaining seam. Discard the candidate code.

## Live-Seam Override Wave

- `frontier_pro_v3_live_seam_override` explicitly aligned the four known live seam boards to shipping behavior while keeping the white turn-3 vulnerable-window recovery.
- The cut did what it was supposed to do locally: retained `primary_pro` moved cleanly by `2 / 62` with `off_target_changed=0`, and `runtime-preflight` passed.
- Direct evidence still killed it: `pro-reliability` vs shipped `frontier_pro_v2_guarded` only reached `0.5000 / 0.7500 / 0.8333`, so even exact seam coverage was nowhere near promotable.
- Durable outcome: treat exact live-seam alignment as a dead end for Pro promotion. Keep the knowledge, discard the candidate code.

## Quiet-Score-Guarded Wave

- `frontier_pro_v3_quiet_score_guarded` tried a candidate-only quiet lower-scored root guard aimed at live non-win mana-head acceptance.
- The cut really did move the retained surface: `primary_pro` changed by `5 / 62` with `off_target_changed=0`, `guardrails` passed, and `runtime-preflight` passed.
- It fixed `vs_shipping_pro_opening_reply_white`, but the other live probe walls still stood: `vs_shipping_pro_black_recovery_branch`, `vs_shipping_pro_white_split_trace`, `vs_shipping_normal_black_bridge_nonwin`, and `vs_shipping_normal_white_head_acceptance`.
- Direct evidence still killed it: `pro-reliability` vs shipped `frontier_pro_v2_guarded` only reached `0.5833 / 0.7500 / 0.9167`, so the candidate code was discarded.
- Durable outcome: quiet-score suppression alone is not a promotable Pro frontier. Keep the lesson, discard the candidate code.

## Progress-Rescue Probe Wave

- `frontier_pro_v3_progress_rescue_guarded` first turned on the dormant mid-turn white progress guard and late-black setup-progress rescue, then added a candidate-only unsafe plain-spirit floor guard.
- The probe-only result was negative: the live non-win root probe remained unchanged on `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, `vs_shipping_pro_white_split_trace`, `vs_shipping_normal_black_bridge_nonwin`, and `vs_shipping_normal_white_head_acceptance`.
- Because the candidate never changed the intended live walls, it never earned `guardrails`, `pro-triage`, or `runtime-preflight`.
- Durable outcome: when the live non-win root probe does not move, kill the line immediately and keep the codebase clean.

## Forced-Prepass Priority Wave

- `frontier_pro_v3_forced_prepass_priority` tried to prioritize `forced_tactical_prepass` ahead of search-only head acceptance and was explicitly threaded through the white scoring-window fallback.
- The probe-only result was still negative: `vs_shipping_normal_white_head_acceptance` stayed on `search_only_engine_allowed_head`, and the other live probe walls also stayed unchanged.
- Because the candidate never changed the intended live walls, it never earned `guardrails`, `pro-triage`, or `runtime-preflight`.
- Durable outcome: if the exact search-only handoff stage does not move, the missing spend is deeper than prepass ordering. Kill the line and keep the codebase clean.

## White Reply-Head Guarded Wave

- `frontier_pro_v3_white_reply_head_guarded` tried a candidate-only white vulnerable-window head reject plus a quiet-mana reply-score guard, and the candidate config was explicitly threaded through the white scoring-window fallback.
- The probe-only result was still negative on the targeted white walls: `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_white_split_trace`, and `vs_shipping_normal_white_head_acceptance` all kept the same selected roots as shipping misses.
- The only visible movement was metadata-level: `vs_shipping_pro_white_split_trace` changed the approved reason label to `ApprovedFamilyCompetition` without changing the selected root.
- Because the candidate never changed the intended live walls, it never earned `guardrails`, `pro-triage`, or `runtime-preflight`.
- Durable outcome: if a white candidate only changes advisor reason labels or leaves `search_only_engine_allowed_head` intact, the real spend is deeper than generic quiet-mana score guards. Kill the line and keep the codebase clean.

## White Presearch-Reentry Guarded Wave

- `frontier_pro_v3_white_presearch_reentry_guarded` tried three white-only spends together: a vulnerable-window presearch approval path, a late quiet-mana head reject, and a stricter white mana-sibling same-lane gap.
- The probe result was mixed but still not promotable. It did fix `vs_shipping_pro_white_split_trace`, moving the selected root from `l8,0;l7,1` to shipping `l10,8;l9,7`.
- The other white walls did not move: `vs_shipping_pro_opening_reply_white` still kept `l10,10;l10,9`, and `vs_shipping_normal_white_head_acceptance` still stayed on `search_only_engine_allowed_head` instead of shipping `search_only_forced_prepass`.
- Because the candidate only repaired one white seam and left the opening-reply plus search-only handoff walls intact, it never earned `guardrails`, `pro-triage`, or `runtime-preflight`.
- Durable outcome: `white_split_trace` is a real white sibling reentry seam, but fixing it alone is not enough. Keep the lesson, discard the candidate code, and keep the worktree clean.

## White Head And Search-Only Guarded Wave

- `frontier_pro_v3_white_head_search_only_guarded` tried three narrow spends together: a late-white low-budget selector exception, a late quiet-mana head reject, and a search-only white vulnerable-window top-head conflict.
- The probe-only result was fully negative. `vs_shipping_pro_opening_reply_white` still stayed `engine_disabled`, `vs_shipping_pro_white_split_trace` still kept `l8,0;l7,1`, and `vs_shipping_normal_white_head_acceptance` still stayed on `search_only_engine_allowed_head`.
- Because the candidate never changed the intended white walls, it never earned `guardrails`, `pro-triage`, or `runtime-preflight`.
- Durable outcome: the remaining white spend is deeper than the guessed low-budget selector gate or a simple search-only top-head conflict. Kill the line and keep the codebase clean.

## Selector-PreDisabled Probe Wave

- `frontier_pro_v3_selector_predisabled_probe` did not cut a new challenger. The spend for this wave was diagnostic-only: the retained live non-win probe now records the actual frontier wrapper branch and the selector disable reason.
- That first reading was later found to be contaminated by an unconditional extra shipping-search fallback on non-black boards. The corrected probe residue from the next wave kept the useful top-level selector fields, but the specific `frontier_execute + pre_disabled` conclusion from this entry should no longer be treated as ground truth on the white boards.
- Durable outcome: keep the improved probe instrumentation, but do not reopen the old `pre_disabled` interpretation without first verifying that no extra fallback search is overwriting the top-level selector diagnostics.

## Advisor-Window Guarded Wave

- `frontier_pro_v3_advisor_window_guarded` was cut after correcting the live probe contamination from the late-black shipping fallback. Once the probe was truthful, the two active white walls split cleanly: `opening_reply_white` was a post-search head-over-advisor seam and `normal_white_head_acceptance` was an early-white vulnerable-window recovery miss.
- The candidate fixed both of those walls together. `opening_reply_white` stayed on the advisor-approved `l9,5;l8,6`, and `normal_white_head_acceptance` stayed on the safe recovery root `l9,4;l8,5` with the risky window head rejected.
- It earned the smaller gates: `guardrails` passed, retained `primary_pro` moved by `5 / 62` with `off_target_changed=0`, exact-lite passed, and stage-1 CPU stayed advisory-only even though the Pro ratios drifted to about `1.65x`, `1.70x`, and `1.90x`.
- Direct retained evidence still killed it. `pro-reliability` vs `shipping_pro_search` failed uniformly at `0.6667 / 0.6667 / 0.6667` with confidence `0.8062 / 0.8062 / 0.8062`, so the candidate code was discarded.
- Durable outcome: even fixing both corrected white live walls and moving retained `primary_pro` cleanly is still not enough. Keep the corrected probe residue and the lesson; discard the candidate code.

## Reply-Risk Injection Guarded Wave

- `frontier_pro_v3_reply_risk_injection_guarded` widened reply-risk shortlist coverage, enabled lazy score-window projection, and allowed two injected roots under the existing Pro V2 selector path.
- The probe result was negative on every real live wall. `opening_reply_white` still accepted the same head over the advisor-approved mana continuation, `black_recovery_branch` still approved the preserved spirit reentry even after the shipping `l6,0;l6,1` mana root entered the reply-risk shortlist, `normal_black_bridge_nonwin` still stayed on the spirit-impact root, and `normal_white_head_acceptance` still stayed on the risky vulnerable-window root.
- Root injection was not the missing mechanism on those boards: `injected_root` stayed `None` through the live probe, so the extra root budget did not translate into a changed approved root.
- The only visible movement was diagnostic-only and not promotable. A white plain-spirit split board changed root ordering, but the real live non-win walls stayed unchanged.
- Durable outcome: shortlist width, lazy score-window projection, and small root injection are not the bottleneck. If the black recovery fallback can already appear inside the shortlist and still not win approval, the next spend has to land inside approval or head logic rather than coverage.

## Approval-Escape Guarded Wave

- `frontier_pro_v3_approval_escape_guarded` turned on lazy score-window projection and spent on candidate-only approval escapes in white followup-mana competition, white mana sibling competition, black legacy alignment, a turn-3 white recovery override, and a late-white head reject.
- The cut did move two real seams. `vs_shipping_pro_white_split_trace` finally approved shipping `l10,8;l9,7`, and `vs_shipping_normal_black_bridge_nonwin` moved off the spirit-own-mana setup onto shipping `l6,1;l5,0;mb`.
- It was still not promotable because the remaining blockers stayed unchanged. `vs_shipping_pro_opening_reply_white` still accepted `l10,10;l10,9` over the advisor-approved `l9,5;l8,6`, `vs_shipping_pro_black_recovery_branch` still approved the preserved spirit reentry instead of shortlist legacy mana `l6,0;l6,1`, and `vs_shipping_normal_white_head_acceptance` still stayed on the vulnerable window root `l9,4;l8,3` instead of the safe recovery root.
- Because those surviving live walls never moved together, the candidate did not earn `guardrails`, `pro-triage`, or `runtime-preflight`. The code was discarded and only the lesson was kept.

## Reply-Risk Reentry Guarded Wave

- `frontier_pro_v3_reply_risk_reentry_guarded` enabled lazy score-window projection, widened the late-white post-search reject so it could also block vulnerable heads over safe-recovery preaccept roots, and relaxed the black vulnerable-spirit escape so vulnerable mana challengers could win approval.
- The white result was still negative. `vs_shipping_pro_opening_reply_white` stayed on `l10,10;l10,9`, and `vs_shipping_normal_white_head_acceptance` still finished on vulnerable `l9,4;l8,3` even though advisor approval had already moved to safe recovery `l9,4;l8,5`.
- The black result was worse, not better. `vs_shipping_pro_black_recovery_branch` flipped onto legacy mana `l6,0;l6,1` while shipping still stayed on spirit `l1,5;l3,3;l2,3`, so removing the safety requirement overcorrected the wrong wall.
- Because the surviving white walls did not move and the black recovery wall moved away from shipping, the candidate never earned `guardrails`, `pro-triage`, or `runtime-preflight`. The code was discarded and only the lesson was kept.

## Safe-Progress Head-Guarded Wave

- `frontier_pro_v3_safe_progress_head_guarded` added family-specific white safe-progress head rejects plus a turn-3 vulnerable-window recovery override, and it was cut only after the live probe gained `head_family` and `goal_family` output.
- The probe confirmed the targeted white walls precisely. `vs_shipping_pro_opening_reply_white` is a `SafeSupermanaProgress -> DrainerSafetyRecovery` post-search head-over-advisor seam, and `vs_shipping_normal_white_head_acceptance` is a `SafeSupermanaProgress -> ImmediateScore` vulnerable-window head-over-recovery seam.
- The candidate did move the intended walls. It fixed both white seams, kept `vs_shipping_pro_black_recovery_branch` aligned with shipping spirit `l1,5;l3,3;l2,3`, passed `smart_automove_tactical_selected_profile`, moved retained `primary_pro` by `5 / 62` with `off_target_changed=0`, and passed exact-lite.
- Retained duel strength still killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.8333 / 0.7500 / 0.9167`, so the candidate code was discarded.
- Durable outcome: even precise `SafeSupermanaProgress` family-specific white head guards plus the turn-3 recovery override are still not promotable. Keep the probe-family diagnostics and the lesson; do not reopen the candidate code.

## Live Nonwin Family Guarded Wave

- `frontier_pro_v3_live_nonwin_family_guarded` extended the family-aware white package with a tighter black turn-6 spirit-reentry filter aimed at the retained vulnerable-spirit seam.
- The candidate did move the intended live walls together. It fixed `vs_shipping_pro_opening_reply_white`, `vs_shipping_normal_white_head_acceptance`, and `vs_shipping_pro_black_recovery_branch`, passed `smart_automove_tactical_selected_profile`, moved retained `primary_pro` by `4 / 62` with `off_target_changed=0`, and passed exact-lite.
- Runtime cost was still weak: `smart_automove_pool_stage1_cpu_non_regression_gate` only cleared in advisory mode at `1.502x`, `1.548x`, and `1.608x` vs `shipping_pro_search`.
- Retained duel strength still killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.8333 / 0.7500 / 0.7500` with confidence `0.9807 / 0.9270 / 0.9270`, so the candidate code was discarded.
- Durable outcome: even fixing both white live walls and the black spirit-reentry wall together is still not enough. Keep the lesson; do not reopen the candidate code.

## White Window Recovery Guarded Wave

- `frontier_pro_v3_white_window_recovery_guarded` tried a narrower white-only spend: a turn-3 no-action vulnerable-window recovery redirect plus a late white weak-window recovery override on action+mana boards.
- The candidate did move the vulnerable-window seam at the advisor layer. On `vs_shipping_normal_white_head_acceptance`, `pre_accept_input_fen` and advisor approval changed from vulnerable `l9,4;l8,3` to safe `DrainerSafetyRecovery l9,4;l8,5`.
- That movement never reached the actual frontier output. Final selected roots on all live walls stayed unchanged against active `frontier_pro_v2_guarded`, because post-search head acceptance still snapped `vs_shipping_normal_white_head_acceptance` back to vulnerable `l9,4;l8,3`.
- Direct challenger evidence killed it immediately: retained `pro-triage` vs active `frontier_pro_v2_guarded` returned `target_changed=0 off_target_changed=0`, so the line was behaviorally inert and never earned `runtime-preflight` or retained reliability.
- Durable outcome: approval-only white recovery is not enough if the final head step still wins. Do not spend canonical gates on candidates that only improve advisor or `pre_accept` metadata.

## Reply-Order Guarded Wave

- `frontier_pro_v3_reply_order_guarded` tried two shared comparator changes together: a stricter risky-recovery progress sibling override and a bounded late-black vulnerable non-spirit followup escape.
- The line stayed fully inert. The live non-win probe left `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, and `vs_shipping_normal_white_head_acceptance` unchanged, and the retained live seams `primary_white_safe_progress_rerank_ply27` plus `primary_live_nonwin_black_vulnerable_spirit_reentry` also stayed unchanged.
- Direct challenger evidence killed it immediately: retained `pro-triage` vs active `frontier_pro_v2_guarded` returned `target_changed=0 off_target_changed=0`, so the line never earned `runtime-preflight` or retained reliability.
- Durable outcome: tightening those shared reply-order thresholds alone is not the missing spend. Keep the lesson, discard the candidate code, and keep the worktree clean.

## Family-Competition Guarded Wave

- `frontier_pro_v3_family_competition_guarded` paired a tighter black turn-6 spirit-reentry filter with a tighter white turn-3 mana sibling competition and a candidate-only turn-3 white recovery override.
- The package did move two real live seams together. `vs_shipping_pro_black_recovery_branch` aligned to shipping `l6,0;l6,1`, and `vs_shipping_pro_white_split_trace` aligned to shipping `l10,8;l9,7`.
- The surviving white seams still blocked promotion. `vs_shipping_pro_opening_reply_white` stayed on `l10,10;l10,9`, and `vs_shipping_normal_white_head_acceptance` again only moved at the advisor layer: `pre_accept_input_fen` changed to safe `DrainerSafetyRecovery l9,4;l8,5`, but the final selected root still snapped back to vulnerable `l9,4;l8,3`.
- Because one surviving wall stayed completely unchanged and the other still failed at final head acceptance, the line never earned `pro-triage`, `runtime-preflight`, or retained reliability. The code was discarded and only the lesson was kept.

## Live Wall Combo Guarded Wave

- `frontier_pro_v3_live_wall_combo_guarded` combined a late-white quiet head reject, a turn-3 white weak-window recovery redirect, a tighter black turn-6 spirit-reentry filter, and a safer white split-trace mana competition.
- The package did align all four active live walls together. `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, `vs_shipping_pro_white_split_trace`, and `vs_shipping_normal_white_head_acceptance` all moved onto the intended shipping roots in the live probe.
- The smaller gates also stayed clean. `smart_automove_tactical_selected_profile` passed, exact-lite passed, and retained `primary_pro` triage stayed at `target_changed=2 / off_target_changed=0`.
- Canonical cost killed it immediately anyway. Against `shipping_pro_search`, `smart_automove_pool_pro_reliability_gate` failed on `stage1_cpu_v1` at `1.687 / 1.696 / 1.732`, with median ratio `1.696x` versus the `1.300x` limit, so the candidate code was discarded.
- Durable outcome: even perfect live-wall alignment plus clean retained triage is not promotion evidence if canonical CPU cost regresses this hard.

## Retained Surface Guarded Wave

- `frontier_pro_v3_retained_surface_guarded` combined a late-white quiet mana head reject, a turn-3 white vulnerable-window recovery override, and a black vulnerable plain-spirit reentry override.
- The package did move the intended retained live seams. It fixed `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, and `vs_shipping_normal_white_head_acceptance`, while retained `primary_pro` triage stayed clean at `target_changed=2 / off_target_changed=0`.
- The cheap gates also stayed clean. `smart_automove_tactical_selected_profile` passed, exact-lite passed, and no off-target retained churn appeared.
- Canonical cost still killed it immediately. `smart_automove_pool_stage1_cpu_non_regression_gate` only cleared in advisory mode at `1.617 / 1.763 / 1.624`, and retained `smart_automove_pool_pro_reliability_gate` died on its embedded `stage1_cpu` precheck at `1.611855221929612 / 1.621475467583131 / 1.6299568403679077`, with median ratio `1.621x` against the `1.300x` limit.
- Durable outcome: even a narrower retained-surface package that fixes three real live walls and keeps retained churn clean is still not promotable if runtime cost regresses back into the `1.6x+` range. Candidate code should be discarded and only the lesson kept.

## Opening Reentry Guarded Wave

- `frontier_pro_v3_opening_reentry_guarded` kept only the two retained live-seam spends from the broader retained-surface package: a late-white quiet mana head reject and a black vulnerable plain-spirit reentry override.
- The package moved the intended retained seams and nothing broader. It fixed `vs_shipping_pro_opening_reply_white` and `vs_shipping_pro_black_recovery_branch`, intentionally left `vs_shipping_normal_white_head_acceptance` unchanged, and retained `primary_pro` triage stayed clean at `target_changed=2 / off_target_changed=0`.
- The cheap gates still stayed clean. `smart_automove_tactical_selected_profile` passed, exact-lite passed, and no off-target retained churn appeared.
- Canonical cost still killed it immediately. `smart_automove_pool_stage1_cpu_non_regression_gate` only cleared in advisory mode at `1.586 / 1.619 / 1.625`, and retained `smart_automove_pool_pro_reliability_gate` died on its embedded `stage1_cpu` precheck at `1.5837620164231196 / 1.5857045402338734 / 1.6051744579914184`, with median ratio `1.586x` against the `1.300x` limit.
- Durable outcome: removing the turn-3 white vulnerable-window recovery override did not fix the runtime-cost regression. The expensive part is at least the late-white opening head reject plus black reentry combo, so candidate code should be discarded and only the lesson kept.

## Retained Gate Alignment Wave

- No new frontier challenger was cut from this wave. The useful code change landed in the retained harness instead: frontier Pro stage-1 CPU is now advisory by default, matching the runbook instead of requiring an explicit env override.
- That harness correction exposed the deeper blocker immediately. On the default retained `pro_turn_planner_reliability_v1` corpus, shipped `frontier_pro_v2_guarded` itself now reaches the duel stage and fails retained `pro-reliability` at `0.7500 / 0.8333 / 1.0000` with confidence `0.9270 / 0.9807 / 0.9998`.
- Durable outcome at that point: keep the harness fix, but the exact retained duel surface still needed to be traced before cutting another challenger.

## White Mid-Turn Recovery Broadening Wave

- This wave tried to spend directly on the remaining white Fast search-only split `l9,4;l8,3` vs shipping `l9,4;l8,5`.
- The runtime cut widened `pro_v2_root_advisor_white_turn_three_no_action_recovery_override` from `mons_moves_count == 0` to `<= 1` and paired it with a post-search head reject for same-lane vulnerable `ManaTempo -> DrainerSafetyRecovery` pairs.
- Locally, the line was real: it fixed the white Fast `ply9` seam, aligned the older vulnerable white mana-only board `l8,4;l7,3` to shipping `l8,4;l8,5`, passed `guardrails`, retained `pro-triage` at `target_changed=4 / off_target_changed=0`, exact-lite, and advisory stage-1 CPU at `1.551 / 1.527 / 1.365`.
- Retained duel strength still killed it. `pro-reliability` failed at `0.9167 / 0.7500 / 0.9167`, and the Normal non-win trace rotated onto engine-disabled early-white boards such as `l8,5;l7,6` vs shipping `l8,7;l7,8`, `l9,4;l8,5` vs `l9,4;l9,3`, and `l8,5;l7,6` vs `l9,5;l8,3;l7,4`. The code was discarded.

## Default Non-Win Surface Alignment Wave

- No new frontier challenger was cut from this wave either. The useful spend was replaying the full default retained duel corpus on shipped `frontier_pro_v2_guarded` and collapsing the exact non-win openings.
- The shipped frontier miss is now fully mapped to the existing live non-win probe surface. `vs_shipping_pro` only loses on `opening_reply_white`, `black_recovery_branch`, and `white_split_trace`; `vs_shipping_normal` only loses on `black_bridge_nonwin` and `white_head_acceptance`; `vs_shipping_fast` is clean at `0` non-wins.
- The live root probe was cleaned to match that exact five-board retained duel surface by dropping the stale extra Pro split board that is not part of the current default non-win pack.
- Durable outcome: the next credible Pro challenger should target those five boards directly. The retained duel boundary is no longer an unexplained seed-mismatch story; it is a concrete five-wall frontier problem.

## Partial Three-Wall Guarded Wave

- No new named frontier challenger survived this wave. The local candidate combined a late-white quiet head reject, a turn-3 white vulnerable-window recovery override, and a black turn-6 preserved-spirit reentry override against active `frontier_pro_v2_guarded`.
- The live probe did move the intended three walls. It fixed `vs_shipping_pro_opening_reply_white`, `vs_shipping_pro_black_recovery_branch`, and `vs_shipping_normal_white_head_acceptance`; retained `primary_pro` triage stayed clean at `target_changed=3 / off_target_changed=0`, and `runtime-preflight` passed with advisory stage-1 CPU at `1.554 / 1.522 / 1.379`.
- Retained duel strength still killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.8333 / 0.7500 / 0.7500` with confidence `0.9807 / 0.9270 / 0.9270`, so the package still was not promotable even before the untouched `white_split_trace` and `black_bridge_nonwin` seams moved.
- It also regressed duel packs that were clean before the edit. `vs_shipping_fast` picked up three non-wins, including a late white post-search snap from shipping `l8,6;l6,5;l6,4` to `l8,7;l9,8` and a repeated late black tail mismatch `l1,8;l1,9` vs shipping `l1,8;l0,8`; `vs_shipping_normal` also reintroduced a white post-search miss (`l8,5;l7,6` vs shipping `l9,5;l8,3;l7,4`).
- Durable outcome: do not reopen partial three-wall approval/head packages. If `white_split_trace` and `black_bridge_nonwin` stay untouched, the retained duel can still fail and previously clean fast/normal packs can regress, so discard the candidate code and keep only the lesson.

## Black Spirit Safety-Gate Wave

- No new frontier challenger survived this wave. The local candidate only tightened `pro_v2_black_turn_six_spirit_reentry` so unsafe preserved-spirit reentry could not beat the available shipping mana root on `black_recovery_branch`.
- The local board and the small gates both looked real. `vs_shipping_pro_black_recovery_branch` aligned to shipping `l6,0;l6,1`, retained `primary_pro` triage stayed clean at `target_changed=4 / off_target_changed=0`, and `runtime-preflight` passed with advisory stage-1 CPU at `1.563 / 1.531 / 1.368`.
- Retained duel strength still killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.9167 / 0.9167 / 0.8333`, so the candidate code was discarded.
- Durable outcome: `black_recovery_branch` is not solved by a blunt unsafe-spirit ban. Keep the lesson, discard the candidate code, and keep the worktree clean.

## Black Legacy-Path Probe Wave

- No new frontier challenger survived this wave either. The useful spend was diagnostic-only on `black_recovery_branch`.
- The ignored probe `black_recovery_branch_legacy_alignment_probe` shows that a direct call to `pro_v2_root_advisor_black_legacy_alignment_override` already returns shipping `l6,0;l6,1` on the live black seam.
- The same probe also captured the path mismatch: a local ProV1 candidate replay on the board resolved to `l1,5;l2,7;l1,8`, while `pro_v2_legacy_selector_probe` still reported `l6,0;l6,1`.
- A naive fallback that scanned qualifying mana roots picked the wrong sibling `l6,0;l7,0`, so the candidate code was discarded before any canonical gate spend.
- Durable outcome: treat `black_recovery_branch` as a legacy-selector plumbing mismatch, not another score-threshold problem. Keep the diagnostic probe and the lesson; discard the production attempt.

## Black Legacy-Selector Config-Swap Wave

- No new frontier challenger survived this wave either. The local candidate changed one line in `pro_v2_root_advisor_select_root`: the ProV1 legacy selector stopped inheriting `shortlist_config` and instead reused the full runtime `config`, which re-enabled the root reply-risk guard for that selector.
- The local board movement was real. The live non-win probe aligned `vs_shipping_pro_black_recovery_branch` to shipping `l6,0;l6,1` through `ApprovedLegacySelector`, while the earlier white turn-three retained fixes stayed intact.
- The small gates also stayed clean enough to justify the spend. `guardrails` passed, retained `pro-triage` stayed at `target_changed=4 / off_target_changed=0`, exact-lite passed, and advisory stage-1 CPU came back at `1.566 / 1.534 / 1.364`.
- Retained duel strength still killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.8333 / 0.9167 / 0.8333`, so the code was discarded.
- Durable outcome: the black legacy-selector mismatch is real, but globally re-enabling reply-risk for that selector is too broad. Keep the lesson, discard the code, and do not reopen this exact config swap.

## Black Reply-Risk-Shortlist Fallback Wave

- No new frontier challenger survived this wave either. The local candidate left the legacy selector alone and only tightened `pro_v2_root_advisor_black_legacy_alignment_override` so the weak plain-spirit black seam could choose the best-ranked vulnerable mana root from the current `reply_risk_shortlist`.
- The local board movement was real. The retained black seam assertion and the live non-win probe both aligned `vs_shipping_pro_black_recovery_branch` to shipping `l6,0;l6,1`, while nearby retained checks for the white confirm board and the black post-search spirit-reentry board still passed.
- The cheap gates also stayed clean enough to justify the duel spend. `guardrails` passed, retained `pro-triage` stayed at `target_changed=4 / off_target_changed=0`, exact-lite passed, and advisory stage-1 CPU came back at `1.561 / 1.522 / 1.367`.
- Retained duel strength still killed it in the same place as the broader black-only lines. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.9167 / 0.9167 / 0.8333`, with Fast still below the floor.
- The follow-up trace showed that Fast loss was not a new collateral surface from the shortlist fallback. Replaying `smart_automove_pro_reliability_nonwin_trace_probe` with `duel_filter=vs_shipping_fast` produced exactly two non-wins, and both were the already-pinned late black head-accept seam on `3 1 b 1 0 2 0 0 14 ...`, where frontier accepts `l1,8;l1,9` and shipping stays on `l1,8;l0,8`.
- Durable outcome: even the shortlist-local black fallback is too broad to keep. Aligning `black_recovery_branch` alone is still not enough; keep the lesson, discard the code, and do not reopen this exact shortlist fallback.

## White Search-Only Recovery Fallback Wave

- No new frontier challenger survived this wave. The local candidate tried the remaining white search-only split `l9,4;l8,3` vs shipping `l9,4;l8,5` in two runtime-variant forms: first by re-querying shipping locally after frontier execution, then by choosing the same nearby safe `DrainerSafetyRecovery` challenger directly from frontier's own ranked roots.
- The local board movement was real. Both variants fixed the retained white `ply9` board and kept the nearby white confirm, white Fast, and black late-fast retained checks clean.
- The cheap gates also stayed clean enough to justify the duel spend. `guardrails` passed, retained `pro-triage` stayed at `target_changed=5 / off_target_changed=0`, exact-lite passed, and advisory stage-1 CPU stayed in the same band at `1.563 / 1.527 / 1.363`.
- Retained duel strength still killed both versions in the same place. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.9167 / 0.8333 / 0.9167`, with Normal below the floor.
- The follow-up Normal non-win trace showed why the direct fallback is not enough. The pack still included the engine-disabled early-white split `l8,5;l7,6` vs shipping `l9,5;l8,3;l7,4`, so fixing the earlier `ply9` recovery board in isolation still leaves the retained Normal blocker alive.
- Durable outcome: do not reopen direct runtime-variant white search-only recovery fallbacks for `l9,4;l8,3`. Keep the lesson, discard the code, and move the next white spend onto the remaining engine-disabled early-white seam instead.

## Black Residue Shipping Fallback Wave

- No new frontier challenger survived this wave. The local candidate added `select_black_progress_setup_engine_disabled_fallback_inputs`, mirroring shipping on the exact black turn-six action+mana weak-window residue board where frontier keeps `l7,1;l9,3` and shipping disables engine selection to play `l1,5;l2,7;l1,8`.
- The local board movement was real. The new retained assertion aligned that residue board to shipping, while the nearby retained white engine-disabled, late black Fast, and late black reply-risk setup walls all stayed clean.
- The cheap gates and the smaller retained duel also stayed clean enough to justify confirm. `guardrails` passed, retained `pro-triage` stayed at `target_changed=5 / off_target_changed=0`, exact-lite passed, advisory stage-1 CPU came back at `1.555 / 1.526 / 1.369`, and retained `pro-reliability` passed at `0.9167 / 0.9167 / 1.0000`.
- The larger confirm duel killed it. `smart_automove_pool_pro_reliability_gate` vs `shipping_pro_search` failed at `0.9375 / 0.9062 / 0.8750`, with Fast below the floor.
- The confirm-sized Fast non-win trace showed why the wrapper is too shallow. The pack still included the old white search-only split `l9,4;l8,3` vs shipping `l9,4;l8,5`, and it also rotated onto two later black engine-disabled seams: `l0,0;l1,1` vs shipping `l7,1;l8,0`, and `l0,5;l1,5` vs shipping `l2,5;l3,7;l2,8`.
- Durable outcome: do not keep or reopen the direct black engine-disabled shipping fallback on `l7,1;l9,3`. Preserve the improved residue probe and move the next black spend onto the later Fast seams instead of mirroring shipping on the earlier turn-six board.

## Black Residue Full-Pool Advisor Probe Wave

- No new runtime challenger survived this wave either. The only kept change is a stronger ignored `black_progress_vs_setup_residue_probe` that now reports shortlist membership plus utility, reply-floor, and followup metrics for the competing `SpiritImpact` own-setup progress roots on `l7,1;l9,3` vs shipping `l1,5;l2,7;l1,8`.
- The useful lesson is that the remaining black residue is not a safe “promote the shipping root from the full pool” advisor fix.
- The probe confirms the shipping root is still absent from `reply_risk_shortlist`, but it also shows the strongest full-pool own-setup progress challenger under frontier's current metrics is `l1,5;l3,7;l2,8`, not shipping `l1,5;l2,7;l1,8`.
- Durable outcome: do not reopen this board with another blind advisor family-competition override. Any future black spend here has to explain the engine-disabled ordering that makes shipping choose `l1,5;l2,7;l1,8` instead of the stronger full-pool spirit candidate first.

## Black Recovery Shortlist Revisit Wave

- No new runtime challenger survived this revisit either. The local candidate reinstated the shortlist-local vulnerable-mana fallback inside `pro_v2_root_advisor_black_legacy_alignment_override`, choosing the best-ranked vulnerable `ManaTempo` root already present in `reply_risk_shortlist` on `black_recovery_branch`.
- The local seam movement was real on the cleaned promoted package. `black_recovery_branch` aligned to shipping `l6,0;l6,1`, and the five-board live nonwin root probe collapsed back to the older white seams.
- The refreshed ignored `black_recovery_branch_legacy_alignment_probe` is worth keeping: it now prints shortlist root details and confirms why the local fallback picks shipping `l6,0;l6,1` instead of the earlier wrong score-leader `l6,0;l7,0`.
- The broader retained duel still killed the line. The canonical loop passed `guardrails`, `pro-triage`, exact-lite, and advisory stage-1 CPU, then failed retained `pro-reliability` at `0.9167 / 0.9167 / 0.8333`.
- The failure surface also changed enough to rule out keeping the runtime cut as a live challenger. The new `pro` miss rotated to a later black lane split `l1,6;l1,7` vs shipping `l1,6;l1,5`, `normal` stayed on the old white `ply9` search-only split `l9,4;l8,3` vs `l9,4;l8,5`, and the two Fast non-wins had `first_diff=none`.
- Durable outcome: even after the earlier late-Fast blocker was repaired, the shortlist-local black legacy fallback is still not promotable. Keep the stronger diagnostic, discard the runtime/test change, and do not reopen this exact line again without explaining the later black lane split plus the no-diff Fast failure.

## Black Later Lane Probe Wave

- No new runtime challenger survived this wave either. The only kept change is a focused ignored `black_pro_lane_split_probe` for the later black `pro` miss `l1,6;l1,7` vs shipping `l1,6;l1,5`.
- The useful lesson is that this seam is not another shortlist omission and not another head-acceptance miss.
- The probe shows shipping `l1,6;l1,5` is already in the frontier `reply_risk_shortlist` beside frontier `l1,6;l1,7` and the rejected head `l2,6;l1,5`. All three are the same safe-progress family.
- It also shows shipping's root loses on frontier's own selector metrics: the reply floor is tied, `shipping_vs_frontier=false`, and frontier keeps better local utility because it preserves `drainer_safety=2` while shipping drops to `-1`.
- Durable outcome: do not reopen the later black lane split with another advisor or reply-risk ordering tweak. Shipping only reaches `l1,6;l1,5` because it disables the turn-engine selector on that board, so this is another shipping-disabled lower-safety ordering mismatch rather than a live frontier bug.

## Fast Hotspot Trace Wave

- No new runtime challenger survived this wave either. The only kept change is a focused ignored `fast_hotspot_trace_probe` for the retained Fast hotspot opening `0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/...`.
- The main value of that probe is that it killed the lazy “Fast is still failing only on no-diff gate noise” read for the current promoted package.
- On the current promoted tree, the hotspot opening really does diverge when frontier is white. The first drift is at `ply=57` on `1 1 w 0 0 1 0 0 9 n04s1xn06/...`, where frontier plays `l9,5;l8,6` from `engine_post_search` and shipping plays `l8,5;l7,7;l8,8` from `engine_disabled`. Frontier's head is rejected, so this is not another head-acceptance issue.
- The same opening with frontier as black is not a fresh target. It rotates back onto the known `black_recovery_branch` split `l1,5;l3,3;l2,3` vs shipping `l6,0;l6,1`.
- Durable outcome: do not spend the next wave treating the retained Fast hotspot as identical-behavior noise. The efficient next step is to classify that late white engine-disabled divergence board before reopening any older black or white no-go seam.

## White Late Fast Hotspot Wave

- No new runtime challenger survived this wave either. The only kept change is a focused ignored `white_late_fast_hotspot_probe` for the late white Fast hotspot board `1 1 w 0 0 1 0 0 9 n04s1xn06/...`.
- The useful lesson is that this board does not admit the obvious shipping mirror.
- Frontier's approved root `l9,5;l8,6` is the only reply-risk-shortlisted root and preserves the stronger reply floor (`921` vs shipping `651`).
- Shipping's move `l8,5;l7,7;l8,8` is outside the frontier shortlist, loses under frontier's own reply-risk comparator (`shipping_vs_frontier=false`), and is not even the strongest spirit-progress candidate in the full pool.
- Durable outcome: treat this as another shipping-only search ordering mismatch, not a live wrapper-fallback or advisor omission. Do not reopen it with direct shipping mirroring, shortlist widening, or a simple spirit-family override without a new explanation for shipping's search-only ordering.

## White Ply9 Search Ordering Wave

- No new runtime challenger survived this wave either. The only kept result is the stronger read from the existing ignored `white_fast_ply9_search_only_split_probe` on `l9,4;l8,3` vs shipping `l9,4;l8,5`.
- The useful lesson is that the remaining white `ply9` seam is not just “previously disproved” in aggregate; it is locally another shipping-only ordering mismatch.
- Frontier's approved `l9,4;l8,3` is the only reply-risk-shortlisted root and keeps the stronger floor (`1191` vs shipping `730`).
- Shipping's `l9,4;l8,5` is a full-pool `DrainerSafetyRecovery` root outside the frontier shortlist and still loses under frontier's own reply-risk comparator (`shipping_vs_frontier=false`).
- Durable outcome: do not reopen that `ply9` seam with direct shipping mirroring, shortlist widening, or another simple recovery override without a new explanation for shipping's `search_only_engine_allowed_head` ordering.

## White Ordering Config Wave

- No new runtime challenger survived this wave either. The only kept change is a focused ignored `white_profile_config_ordering_probe` for the two remaining white ordering boards: `white_ply9_search_ordering` and `white_late_fast_hotspot`.
- The useful lesson is that there is no hidden per-board config branch separating those seams from the promoted package.
- On both boards, shipping and frontier use the same depth (`4`), node budget (`15774`), reply-risk shortlist budget (`9 / 24 / 2000`), and scoring weights.
- The remaining difference is structural and profile-level: shipping stays `selector=false`, `head_rerank=true`, `mode=ProV1`, while frontier stays `selector=true`, `head_rerank=false`, `mode=ProV2` with the extra ProV2 guards.
- Durable outcome: treat the remaining white seams as search-profile semantics, not board-local config misses. Do not reopen them with another board-local wrapper or guard tweak unless there is a brand-new shared hypothesis.

## White Rerank Semantics Wave

- No new runtime challenger survived this wave either. The kept diagnostic is `white_ordering_rerank_semantics_probe`, and the discarded runtime cut was a frontier-local rerank-semantics fallback for `white_ply9_search_ordering`.
- The probe usefully split the two remaining white ordering boards instead of merging them. On `white_ply9_search_ordering`, shipping `l9,4;l8,5` is rank `0` on both the shipping and frontier root sets, is `Accepted` by `classify_turn_engine_rerank_override`, is allowed by `turn_engine_allowed_rerank_override_candidate`, and does not conflict with the ProV2 advisor.
- The late Fast hotspot is not the same class. On `white_late_fast_hotspot`, shipping `l8,5;l7,7;l8,8` is rejected by `ProgressGate` and is not an allowed rerank candidate even on shipping's own root set.
- The runtime cut was real but still too shallow. It fixed `l9,4;l8,3` vs shipping `l9,4;l8,5`, passed `guardrails`, `pro-triage` at `target_changed=5 / off_target_changed=0`, exact-lite, and advisory stage-1 CPU at `1.551 / 1.523 / 1.363`, then failed retained `pro-reliability` at `0.9167 / 0.8333 / 0.9167`.
- The retained Normal trace showed why it cannot stay live: instead of stopping at `ply9`, the pack rotated onto other early-white engine-disabled seams, including `l8,5;l7,6` vs shipping `l8,7;l7,8` and `l8,5;l7,6` vs shipping `l8,5;l7,4`.
- Durable outcome: do not reopen `white_ply9_search_ordering` with another narrow rerank-semantics wrapper fallback. Even when the shipping root is rerank-admissible and advisor-compatible on frontier, the local repair still is not enough to promote.

## White Normal Residue Wave

- No new runtime challenger survived this wave. The kept additions are a fresh retained `vs_shipping_normal` non-win trace on the cleaned promoted package and the new ignored `white_normal_ply11_search_only_split_probe`.
- The useful result is that the discarded rerank fallback's rotated engine-disabled boards are not the live retained Normal surface on the clean tree.
- Replaying `smart_automove_pro_reliability_nonwin_trace_probe` with `SMART_PRO_RELIABILITY_DUEL_FILTER=vs_shipping_normal` produced `total_nonwins=1`, and the sole drift was a sibling of the white search-order family: `0 0 w 1 0 1 0 0 3 n06a0xn04/...`, first diff at `ply=11`, frontier `l9,4;l8,3`, shipping `l9,4;l8,5`.
- The new board-local probe shows the same structural shape as the earlier Fast probe. Frontier's shortlist is still just `l9,4;l8,3`; shipping still reaches `l9,4;l8,5` through `search_only_engine_allowed_head`; the shipping recovery root is still outside the frontier shortlist; and `shipping_vs_frontier=false` under frontier's own reply-risk comparator.
- Durable outcome: until a future challenger changes the clean retained trace, treat current Normal residue as the white `l9,4;l8,3` vs `l9,4;l8,5` search-order family, not as the rotated early-white engine-disabled boards from the discarded rerank fallback.

## White Allowed-Head Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_search_order_allowed_head_probe`.
- The useful result is that the live white search-order family is not blocked by root-set reachability. On both `white_ply9_search_ordering` and `white_normal_ply11_search_ordering`, shipping's rerank engine still chooses `l9,4;l8,5` when it is fed the frontier root set.
- Frontier's own rerank engine config on the same allowed heads still prefers `l9,4;l8,3`, with `l9,4;l8,5` only appearing as a lower-ranked rerank plan.
- Durable outcome: treat the current white residue as a rerank-engine profile split (`shipping` ProV1 rerank vs `frontier` ProV2 rerank), not as a selector-disabled root omission. Another wrapper fallback that only patches root choice is not a real explanation.

## White Rerank Mode Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_search_order_rerank_mode_probe`.
- The useful result is that the white rerank split is narrower than “ProV2 everywhere is wrong” but stronger than a pure root-set or admissibility issue.
- On both white search-order siblings, forcing only the frontier rerank engine mode from `ProV2` to `ProV1` already flips the best allowed-head plan to shipping `l9,4;l8,5`.
- Shipping still chooses `l9,4;l8,5` even when its rerank mode is forced to `ProV2`, so the mismatch is not a generic mode-only story either.
- Durable outcome: treat the live white residue as a frontier-`ProV2` rerank semantics split. Do not jump straight from that fact to another narrow runtime mode-swap fallback without fresh retained-duel evidence.

## White Rerank Budget Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_search_order_rerank_budget_probe`.
- The useful result is that the live white rerank split is not spread across all of frontier's rerank budget knobs.
- On both white search-order siblings, swapping only frontier's rerank own-search caps (`own_seed_cap`, `own_beam`, `per_node_family_cap`, `step_cap`) to the shipping `ProV2` values already flips the best allowed-head plan from `l9,4;l8,3` to shipping `l9,4;l8,5`.
- Swapping only frontier's reply caps (`opponent_seed_cap`, `opponent_beam`, `reply_seed_cap`, `reply_beam`) does nothing, and swapping only the expansion cap does nothing.
- Durable outcome: treat the current white residue as a frontier-`ProV2` rerank own-search-breadth split. If there is ever a runtime spend here, it should be justified against that exact surface rather than against generic reply breadth or expansion arguments.

## White Own-Cap Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_search_order_rerank_own_cap_probe`.
- The useful result is that the frontier-side rerank own-search split is not spread evenly across the four own caps.
- `step_cap` alone flips both white search-order siblings from `l9,4;l8,3` to shipping `l9,4;l8,5`, and it reproduces the shipping rerank utility on both boards.
- `own_seed_cap` alone also flips both siblings to `l9,4;l8,5`, but it does not reproduce the same rerank utility on the Fast board, so it is a weaker explanation than `step_cap`.
- `own_beam` alone and `per_node_family_cap` alone do nothing; frontier stays on `l9,4;l8,3`.
- Durable outcome: treat frontier `ProV2` rerank `step_cap` as the cleanest single-cap explanation for the live white residue seen so far. Do not jump from that directly to a runtime patch on an already promotable package without fresh duel evidence.

## White Seed-Step Scope Wave

- No new runtime challenger survived this wave either. The kept diagnostic is `white_search_order_seed_step_scope_probe`.
- The useful result is that the two remaining “active” white rerank caps are still too broad to spend directly.
- On the two live white search-order siblings, shrinking either frontier rerank `own_seed_cap` or frontier rerank `step_cap` still flips the allowed-head plan from `l9,4;l8,3` to shipping `l9,4;l8,5`.
- On the late white Fast hotspot, though, the same broad cap shrink does not leave the board stable and does not reproduce shipping. `own_seed_cap` shrink moves frontier to `l8,5;l6,5;l5,4`, `step_cap` shrink moves it to `l8,5;l6,5;l6,4`, and shipping's rerank on the frontier head set still stays on `l9,5;l8,6`.
- Durable outcome: broad white rerank `own_seed_cap` or `step_cap` shrink is not a safe runtime answer. Any future white rerank spend has to gate more tightly than “lower frontier rerank seed/step breadth.”

## White Runtime Step Clamp Wave

- No new runtime challenger survived this wave. The attempted cut was the narrowest live follow-up to the seed/step probes: clamp frontier `ProV2` rerank `step_cap` to `1` only on the exact white `turn=3 / mons_moves=1 / no-action / mana-only / window=1 / deny=1 / drainer_safety<0` board class.
- The useful result is that the live white search-order residue is not controlled by `turn_engine_rerank_config` alone.
- The focused retained slice failed immediately. On both white siblings, frontier still selected `l9,4;l8,3` instead of shipping `l9,4;l8,5`.
- The runtime shape did not budge: selector stage stayed `engine_post_search`, the accepted head stayed `l9,4;l8,3`, and the approved shortlist stayed a singleton on that same root.
- Durable outcome: do not reopen the white search-order family with another board-local rerank-cap runtime tweak unless a future probe first shows how that tweak can move shortlist or approved-root behavior instead of only changing allowed-head rerank plans in isolation.

## White Shortlist Gate Wave

- No new runtime challenger survived this wave either. The kept diagnostic is `white_search_order_shortlist_gate_probe`.
- The useful result is that the remaining white search-order seam is not blocked by candidate focus and is not waiting on the current safe-progress shortlist extension.
- On both white siblings, shipping `l9,4;l8,5` is already present in frontier `candidate_indices`.
- It still never reaches the approved shortlist. The shortlist stays a singleton on vulnerable `l9,4;l8,3` because that root's score is about `809k` above shipping's, far beyond the `165` shortlist margin, and `pro_v2_safe_progress_sibling_shortlist_extension` returns `None`.
- Durable outcome: do not reopen the white search-order family with another simple shortlist tweak or another rerank-cap tweak. Any future white spend has to explain a new shortlist/approved-root reentry theory, or a deeper root-scoring normalization, instead of assuming the current focus/extension machinery just missed `l9,4;l8,5`.

## White Selector-Disable Wrapper Probe Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_search_order_selector_disable_probe`.
- The useful result is that shallow config mirroring does not actually test selector-disabled white semantics through the guarded frontier wrapper.
- On both white search-order siblings and on `white_late_fast_hotspot`, forcing the incoming frontier runtime config to `selector=false`, `head_rerank=true`, shipping-like own caps, or even `TurnEngineMode::ProV1` still leaves the live decision unchanged on frontier `engine_post_search`.
- The code path explains why: `select_frontier_pro_v2_guarded_inputs` always routes frontier execution back through `apply_frontier_pro_v2_guarded_config`, so those config-only selector-disable toggles are reapplied away before search runs.
- Durable outcome: do not reopen the white search-order family with another wrapper-local config-only selector-disable or head-rerank mirror. Any future white wrapper spend has to change wrapper branching itself, or move deeper into shortlist/root-scoring behavior.

## White Raw Search-Only ProV1 Scope Wave

- No new runtime challenger survived this wave. The kept diagnostics are `white_search_order_wrapper_branch_probe` and `white_search_order_raw_prov1_scope_probe`.
- The useful result is that once the guarded wrapper is truly bypassed, raw search-only branch choices become locally meaningful.
- Raw `search-only + ProV2` still keeps frontier `l9,4;l8,3` on the two white search-order siblings and keeps frontier `l9,5;l8,6` on the late Fast hotspot.
- Raw `search-only + shipping own caps + ProV2` fixes the two `ply9/ply11` siblings but still leaves the hotspot on frontier.
- Raw `search-only + ProV1` matches shipping on all three local white seams, so search-only `ProV1` semantics are a real explanation rather than a dead config artifact.
- That broad line is still not safe enough to keep. The scope probe shows the same raw `search-only + ProV1` reroute also flips the retained vulnerable white turn-three guard from `l8,4;l7,3` to shipping `l8,4;l8,5`, even though it shares the same coarse `turn=3 / mons_moves=1 / no-action / mana-only / window=1 / deny=1 / drainer_safety<0` context as the unresolved white siblings.
- Durable outcome: do not reopen the white search-order family with a broad wrapper-level search-only `ProV1` gate on that coarse white context. Any future white wrapper spend has to distinguish the retained vulnerable guard from the unresolved siblings with a narrower theory than “use shipping-like search-only semantics here.”

## White Vulnerable-Guard Search-Order Comparison Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_vulnerable_guard_search_order_probe`.
- The useful result is that the retained vulnerable white turn-three guard is not a different class from the unresolved white search-order siblings at the current shortlist/reply-risk layer.
- On the retained guard board, frontier still keeps a singleton vulnerable `ManaTempo` shortlist and shipping still only wins through `search_only_engine_allowed_head` on an outside-shortlist rank-0 `DrainerSafetyRecovery` root, just like the unresolved sibling boards.
- `shipping_vs_frontier` is still `false` there too, and there is still no projection or advisor reentry signal that would justify a simple wrapper-local split between the guard board and the unresolved siblings.
- Durable outcome: do not reopen the white search-order family with another wrapper-level gate based on the current reply-risk/shortlist surface. Any future white spend has to distinguish boards below that surface or change the root scoring that makes both boards disagree in the same way.

## White Negative-Deny Rank Surface Wave

- No new runtime challenger survived this wave. Two narrow negative-deny white wrapper cuts were tried and both died in the focused retained slice before any canonical gate spend.
- The first cut replayed the remaining Normal sibling through raw `search-only + shipping own caps + ProV2` and tried to keep it only when that replayed move was `root_rank=0`.
- The second cut used direct shipping fallback on the same board and tried to keep it only when the shipping move was `root_rank=0` in the shipping search.
- Both cuts still reopened the retained vulnerable guard `l8,4;l7,3` to shipping `l8,4;l8,5`, so both runtime changes were discarded.
- The kept diagnostic is `white_search_order_rank_surface_probe`.
- Its useful result is that root rank is fake separation for the negative-deny white seam.
- On the then-unresolved Normal sibling, shipping `l9,4;l8,5` is both `selected_rank=0` and `root_rank=0`.
- On the retained vulnerable guard, shipping `l8,4;l8,5` still shows `selected_rank=4` under the shipping runtime, but its underlying `root_rank` is already `0`; the raw shipping-own-caps replay also gives that same guard move `root_rank=0`.
- Later supersession: selected rank under the raw shipping-own-caps replay, not root rank, did become a safe separator. The promoted follow-up only keeps that replay when the move is the top scored focused candidate; the Normal sibling is `selected_rank=0`, while the retained vulnerable guard stays `selected_rank=4`.
- Durable outcome: do not reopen the negative-deny white search-order family with another runtime gate based on `root_rank == 0`. The useful layer is the scored focused selected rank after the exact capped replay, not the underlying root rank.

## Sampled Late-Ply Repair Wave

- No promotable challenger came out of this wave. The kept runtime package repaired several stable retained late Normal/Fast seams around `frontier_pro_v2_guarded`, but sampled `pro-reliability` still failed at Pro `1.0000`, Normal `0.9167`, Fast `0.8333`, with confidence `0.9998 / 0.9968 / 0.9807`; frontier average move times were `151.76ms / 190.31ms / 170.39ms`.
- The useful local repairs that survived retained replay were narrow: black late search/head-accept Normal guards, a black turn-six vulnerable-progress mana override, an early black setup-branch legacy spirit override, and a white turn-start spirit-setup head blocker. Those changes fixed the sampled retained boards they were derived from.
- Promotion still died on rotated late-ply sampled boards. The remaining sampled blockers at the end of the wave were Normal `outer_edge_mana_rows` and Fast `alternating_mana_rows` plus `forward_bridge_mana_rows`.
- The clean reproducible live blockers were deeper late-ply search/head splits, not the earlier retained seams:
  - Normal black late search board: frontier approved quiet `l1,6;l1,5` while shipping stayed on `l2,6;l3,7`.
  - Fast white branch head-accept board: frontier accepted `l9,6;l7,4;l7,3` over the advisor-approved shipping root `l9,6;l7,6;l7,7`.
- One traced Fast black nonwin was not safe to keep as retained coverage. The copied board snapshot did not reproduce the live shipping-selected root on a clean retained replay and collapsed back to frontier instead.
- Durable outcome: when sampled losses rotate into late-ply boards, keep only the stable retained repairs and archive the reproducibility rule. Do not retain copied `first_diff_ply` boards unless the retained harness reproduces the same final shipping-selected root.

## Black Alternating Retained Engine-Disabled Structure Wave

- No new runtime challenger survived this wave. The kept diagnostic is `black_alternating_vs_retained_engine_disabled_fast_structure_probe`.
- The useful result is that the singleton Fast `alternating_mana_rows` seam `l0,10;l0,9` vs shipping `l4,0;l5,0;mb` is not covered by the nearby retained black engine-disabled package.
- `BLACK_BRIDGE_NONWIN_DUEL_FAST` only matches at the coarse `window=0/deny=0` singleton-shortlist `ManaTempo` surface. On that retained board, frontier accepts `l1,6;l2,7` and still collapses to engine-disabled mana `l4,1;l5,0;mb`, matching shipping in the final selector.
- `BLACK_ENGINE_DISABLED_DUEL_FAST` is different again: it is a dense-shortlist `SpiritImpact` preserved-representative board where frontier and shipping both keep `l1,5;l2,3;l1,2` and no accepted head survives.
- Durable outcome: do not extend the retained black bridge or engine-disabled Fast controls into `alternating_mana_rows` just because they share `window=0/deny=0` or an engine-disabled shipping finish. The live seam is a pure `ManaTempo` `ApprovedReplyRiskGuard` board where frontier never collapses off the approved root.

## Outer Edge Forward Bridge Shared Head Surface Wave

- No new runtime challenger survived this wave. The kept diagnostics are the direct widened replays:
  - `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows` with `smart_automove_pro_reliability_nonwin_trace_probe`, `duel_filter=vs_shipping_normal`, `repeats=6`, `games=3`
  - `SMART_AUTOMOVE_VARIANTS=forward_bridge_mana_rows` with `smart_automove_pro_reliability_nonwin_trace_probe`, `duel_filter=vs_shipping_fast`, `repeats=4`, `games=3`
- The useful result is that the remaining Normal `outer_edge` and Fast `forward_bridge` residue do not share one head-accept mechanism.
- `outer_edge` widened back to `10` Normal nonwins and split across several surfaces:
  - rejected-head post-search drift: late black `l1,6;l1,5` vs shipping `l2,6;l3,7`
  - rejected-head post-search drift: early white `l10,4;l9,5` vs shipping `l9,4;l8,3`
  - rejected-head post-search drift: early black `l1,4;l2,5` vs shipping `l1,4;l1,6;l2,7`
  - accepted-head post-search drift: repeated black `l2,7;l3,8` vs shipping `l1,5;l0,3;l1,3`
  - accepted-head `search_only_forced_prepass`: white `l9,3;l8,3` vs shipping `l7,2;l6,1`
- `forward_bridge` also logged `10` Fast nonwins, but it stayed a different mixed spirit/setup bucket:
  - repeated white accepted-head pair `l9,6;l7,4;l7,3` vs shipping `l9,6;l7,6;l7,7`
  - safe-progress approval seam `l9,6;l8,7` vs shipping `l9,6;l7,7;l8,8`
  - fallback/setup seam `l9,7;l8,6` vs shipping `l9,7;l7,6;l7,7`
  - extra white setup drift `l9,5;l9,6` vs shipping `l7,3;l6,2`
  - black setup drift `l7,1;l9,3` vs shipping `l1,5;l3,4;l2,3`
- Durable outcome: do not reopen a shared `outer_edge` plus `forward_bridge` head-accept patch. One repeated accepted-head pair still exists inside `forward_bridge`, but the widened `outer_edge` replay no longer supports a single shared head surface.

## White Outer Edge Harvest Structure Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_outer_edge_forced_prepass_vs_retained_harvest_structure_probe`.
- The useful result is that retained `primary_white_harvest_loss_c_ply24` is not an extension candidate for the white `outer_edge_mana_rows` forced-prepass seam.
- The traced live board had looked adjacent because the trace logged shipping `l7,2;l6,1`. On a clean direct probe, that exact move only survived at `pre_accept`: both frontier and shipping collapsed to `search_only_forced_prepass` with final selected `l9,3;l8,3`, no head, and no advisor decision.
- The retained harvest board is a different surface entirely: Pro mode, `window=2/deny=2`, attack-enabled, shortlist-live `SafeSupermanaProgress l7,2;l6,1` approved through `ApprovedReplyRiskGuard`, with frontier rejecting non-progress head `l8,5;l7,4`.
- Durable outcome: do not spend on the white `outer_edge` forced-prepass seam by extending the retained harvest control. The exact move overlap was only a `pre_accept` coincidence, not a stable shared selector surface.

## Black Outer Edge Early Recovery Repro Wave

- No new runtime challenger survived this wave. The kept diagnostic is `black_outer_edge_early_recovery_structure_probe`.
- The useful result is that the copied singleton early-black `outer_edge` boards are not stable local seams.
- The copied `l1,4;l2,4` vs `l0,5;l1,6` board did not replay the traced drift. On a clean direct probe, both frontier and shipping collapsed to engine-disabled `l0,5;l1,6`.
- The copied `l1,4;l2,5` vs `l1,4;l1,6;l2,7` board also failed reproduction. On a clean direct probe, both frontier and shipping instead collapsed to engine-disabled `l1,5;l2,7;l3,8`, so even the traced shipping final did not survive.
- Durable outcome: do not spend on copied singleton early-black `outer_edge` seams until they reproduce cleanly. If the copied board collapses to shared shipping or to a different shared final, it is not a defensible local runtime target and it is not a good retained-extension anchor.

## Black Alternating Late Fast Recovery Repro Wave

- No new runtime challenger survived this wave. The kept diagnostics were:
  - `SMART_AUTOMOVE_VARIANTS=alternating_mana_rows` with `smart_automove_pro_reliability_nonwin_trace_probe`, `duel_filter=vs_shipping_fast`, `repeats=4`, `games=3`
  - `black_alternating_late_fast_recovery_structure_probe`
- The useful result is that the old Fast `alternating` late-fast recovery singleton still exists in the live trace, but its copied board is not stable enough to spend on.
- The replay still logged the seam `l2,5;l0,5;l1,6` vs shipping `l2,5;l4,7;l3,8` on the current Fast corpus.
- The copied board did not replay that seam on a clean direct probe. Both frontier and shipping instead collapsed to shared engine-disabled `l2,5;l4,5;l3,4`, with no head or advisor residue left to compare against retained `BLACK_LATE_FAST_RECOVERY_TRACE`.
- Durable outcome: do not extend the retained late-fast recovery trace into the Fast `alternating` singleton and do not spend runtime code on that copied board until it reproduces cleanly.

## White Forward Bridge Setup Repro Wave

- No new runtime challenger survived this wave. The kept diagnostic is `white_forward_bridge_setup_structure_probe`.
- The useful result is that the copied Fast `forward_bridge` white setup singleton is not a stable local seam.
- The archived singleton had looked like a plausible retained-extension candidate because the widened Fast trace logged `l9,5;l9,6` vs shipping `l7,3;l6,2` and the nearby retained white setup controls live on similar spirit/setup territory.
- On a clean direct probe, that copied board did not replay the traced drift. Both frontier and shipping collapsed to shared engine-disabled `l7,3;l6,2`, with no head or advisor residue left to compare against `WHITE_TURN_FIVE_SPIRIT_SETUP_PRE_ACCEPT_FAST` or `WHITE_LATE_CLUSTER_NONWIN`.
- Durable outcome: do not extend the retained white setup controls into the copied `forward_bridge` singleton and do not spend runtime code on that board until it reproduces cleanly.

## Forward Bridge Low-Budget Guard Scope Wave

- No new runtime challenger survived this wave. The attempted cut disabled `enable_turn_engine_low_budget_guard` only when `game.variant() == GameVariant::ForwardBridgeManaRows`.
- The useful result is that the low-budget guard can be a sampled Fast forward-bridge lever without being a promotable strength lever.
- The candidate passed focused checks and canonical sampled `pro-reliability`: Pro `1.0000`, Normal `0.9167`, Fast `0.9167`; sampled Fast `forward_bridge_mana_rows` moved to `1.0000`, while Normal `outer_edge_mana_rows` and Fast `alternating_mana_rows` remained the sampled weak rows.
- All-variant `pro-reliability-confirm` failed broadly: Pro `0.6458`, Normal `0.7083`, Fast `0.6667`; average move times stayed well below the `700ms` cap, so the failure was strength rather than cost.
- Durable outcome: do not reopen a variant-scoped low-budget guard disable for `ForwardBridgeManaRows`. It overfits sampled forward-bridge and broadens all-variant losses.

## Alternate-Seed All-Blocker Trace Wave

- No runtime challenger was attempted in this wave. The diagnostic was `smart_automove_pro_reliability_duel_trace_probe` with `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows`, `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_alt_v1`, `SMART_PRO_RELIABILITY_REPEATS=2`, and `SMART_PRO_RELIABILITY_GAMES=3`.
- The useful result is that the active blocker set still does not hide one repeated move-pair mechanism under a different deterministic seed.
- Pro produced `1` regression, `2` improvements, and `9` flat results; the only regression pair was `l8,8;l7,7` vs `l9,4;l8,3`.
- Normal produced `2` regressions, `5` improvements, and `5` flat results; both regression pairs were singletons: `l10,7;l9,8` vs `l9,6;l10,4;l9,5`, and `l9,6;l8,7` vs `l9,6;l10,5`.
- Fast produced `4` regressions, `3` improvements, and `5` flat results; all four regression pairs were singletons, including the known `forward_bridge` head-accept pair `l9,6;l7,4;l7,3` vs `l9,6;l7,6;l7,7`.
- Durable outcome: do not spend runtime code on alternate-seed singleton pairs from the current blocker variants. Use alternate seeds to prove recurrence, not to chase one-off seams.

## Focused Fast-Blocker Alternate-Seed Trace Wave

- No runtime challenger was attempted in this wave. The diagnostic was `smart_automove_pro_reliability_duel_trace_probe` with `SMART_AUTOMOVE_VARIANTS=alternating_mana_rows,forward_bridge_mana_rows`, `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_alt_v2`, `SMART_PRO_RELIABILITY_REPEATS=2`, and `SMART_PRO_RELIABILITY_GAMES=3`.
- The useful result is that narrowing the alternate seed to the two sampled Fast blocker variants still did not recover one stable Fast mechanism.
- Pro produced `3` regressions, `1` improvement, and `8` flat results; all three regression pairs were singletons, including the known `alternating` black `l2,7;l1,6` vs `l2,7;l1,8` pair.
- Normal produced `5` regressions, `3` improvements, and `4` flat results; all five regression pairs were singleton and split across black and white, head-accepted and engine-disabled surfaces.
- Fast produced only `1` regression, `5` improvements, and `6` flat results; its single regression was `l10,5;l9,5` vs `l8,1;l7,0`.
- Durable outcome: do not reopen the sampled Fast blockers from this alternate seed. The focused run argues against a stable `alternating_mana_rows` or `forward_bridge_mana_rows` runtime spend.

## Outer Edge Alt-V2 Normal Trace Wave

- No runtime challenger was attempted in this wave. The diagnostic was `smart_automove_pro_reliability_nonwin_trace_probe` with `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows`, `SMART_PRO_RELIABILITY_DUEL_FILTER=vs_shipping_normal`, `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_alt_v2`, `SMART_PRO_RELIABILITY_REPEATS=4`, and `SMART_PRO_RELIABILITY_GAMES=3`.
- The useful result is that the Normal `outer_edge_mana_rows` blocker still does not collapse to one mechanism under a different seed.
- The trace logged `9` nonwins. Two black pairs repeated: late black `l1,6;l1,5` vs shipping `l2,6;l3,7`, and early black accepted-spirit `l1,5;l0,3;l1,3` vs shipping `l1,6;l2,7`.
- The same run also logged singleton white post-search drift, black accepted-head drift, white accepted-head drift, early black engine-disabled drift, and a white safe-progress ordering seam.
- Durable outcome: do not spend on either repeated `outer_edge` black pair as a standalone patch. Repetition inside a trace that still spans colors and selector stages is not promotable evidence.

## Alt-V3 All-Blocker Trace Wave

- No runtime challenger was attempted in this wave. The diagnostic was `smart_automove_pro_reliability_duel_trace_probe` with `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows`, `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_alt_v3`, `SMART_PRO_RELIABILITY_REPEATS=3`, and `SMART_PRO_RELIABILITY_GAMES=3`.
- The useful result is that raising the alternate-seed sample to `18` games per duel bucket still did not collapse the active blockers into a shared runtime mechanism.
- Pro produced `5` regressions, `5` improvements, and `8` flat results. Every Pro regression pair was singleton: `l2,5;l3,5` vs `l1,6;l1,5`, `l9,2;l8,2` vs `l9,2;l8,1`, `l9,4;l8,3` vs `l7,5;l6,4`, `l9,3;l8,2` vs `l7,2;l6,1`, and `l0,5;l1,4` vs `l0,5;l1,5`.
- Normal produced `4` regressions, `3` improvements, and `11` flat results. Only one pair repeated, black `l2,3;l3,3` vs shipping `l2,3;l3,2` (`2x`), while `l8,5;l7,3;l6,4` vs `l8,5;l6,5;l6,4` and `l1,6;l2,5` vs `l1,6;l2,6` stayed singleton.
- Fast produced `2` regressions, `4` improvements, and `12` flat results. Both Fast regression pairs were singleton: white `l10,4;l9,3` vs shipping `l9,5;l7,6;l8,7` and black `l2,4;l4,5;l3,6` vs shipping `l2,4;l3,2;l2,2`.
- Durable outcome: do not spend on the Normal black `l2,3;l3,3` pair as a standalone patch. The broader trace still balances regressions with improvements and keeps Pro/Fast singleton-only, so the blocker set remains mixed rather than promotable.

## Alt-V3 Normal Hotspot Follow-Up

- No runtime challenger survived this wave. A temporary diagnostic case was added to `smart_automove_pro_reliability_hotspot_probe` for the repeated Normal black board from the alt-v3 trace, then removed before commit.
- The useful result is that the repeated pair `l2,3;l3,3` vs shipping `l2,3;l3,2` is not a root-pool availability seam. Frontier and shipping both enumerated `10,941` selector children with identical shortlist/full-pass counts.
- The actual split was the known frontier-extra-work shape: frontier selected `l2,3;l3,3` through `engine_post_search`, shipping selected `l2,3;l3,2` through `engine_disabled`, and frontier paid extra exact/engine work (`payload_calls +163,990`, `pickup_calls +6,177`, `secure_mana_calls +1,784`, `tactical_spirit_calls +2,914`, engine accepted `6` vs `1`).
- Durable outcome: do not reopen the alt-v3 Normal black pair through root-pool widening, shortlist widening, or generic exact-cost reduction. The board reproduces a known hotspot false lead inside an already mixed all-blocker trace.

## Env-Driven Hotspot Probe Hook

- No runtime challenger was attempted in this wave. The kept code is diagnostic-only under the ignored test harness.
- `smart_automove_pro_reliability_hotspot_probe` now accepts one extra ad-hoc board through `SMART_PRO_RELIABILITY_HOTSPOT_FEN`, with optional `SMART_PRO_RELIABILITY_HOTSPOT_LABEL` and `SMART_PRO_RELIABILITY_HOTSPOT_MODE`.
- The hook was validated by rerunning the alt-v3 Normal black board without a temporary source case. It reproduced the previous no-go: frontier `l2,3;l3,3`, shipping `l2,3;l3,2`, identical `10,941` selector children, and frontier-only extra `engine_post_search`/exact work.
- Durable outcome: use the env hook for future one-off hotspot inspections instead of adding and later removing ad-hoc probe cases.

## Alt-V4 All-Blocker Trace Wave

- No runtime challenger was attempted in this wave. The diagnostic was `smart_automove_pro_reliability_duel_trace_probe` with `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows`, `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_alt_v4`, `SMART_PRO_RELIABILITY_REPEATS=3`, and `SMART_PRO_RELIABILITY_GAMES=3`.
- The useful result is that another fresh deterministic seed still did not collapse the active blockers into one promotable mechanism.
- Pro produced `3` regressions, `3` improvements, and `12` flat results. One pair repeated, black `l2,7;l3,8` vs shipping `l1,5;l0,3;l1,3` (`2x`), while white `l8,7;l7,7` vs shipping `l5,10;l4,10` stayed singleton.
- Normal produced `5` regressions, `2` improvements, and `11` flat results. One pair repeated, the already-archived black `l2,7;l1,6` vs shipping `l2,7;l1,8` recovery jump (`2x`), while `l0,5;l1,4` vs `l1,6;l0,6`, `l2,3;l3,3` vs `l2,3;l3,2`, and `l9,4;l9,3` vs `l9,4;l8,3` stayed singleton.
- Fast produced `4` regressions, `3` improvements, and `11` flat results. Every Fast regression pair was singleton.
- The env-driven hotspot hook was used on the repeated Pro black pair. Frontier selected `l2,7;l3,8` through `engine_post_search`, shipping stayed engine-disabled on `l1,5;l0,3;l1,3`, frontier accepted `23` engine heads vs shipping `0`, and frontier expanded more selector children (`12,473` vs `11,612`). That is extra frontier search/selector work inside a balanced mixed trace, not a clean root-pool or shortlist repair.
- Kept code outcome: `smart_automove_pro_reliability_duel_trace_probe` now accepts `SMART_PRO_RELIABILITY_DUEL_FILTER`, matching the existing nonwin trace filter and avoiding full three-bucket runs when only one duel bucket needs recurrence evidence.
- Durable outcome: do not reopen either alt-v4 repeated pair as a standalone patch. Use filtered duel traces for future focused recurrence checks, then hotspot only the repeated board if the surrounding bucket is also promotable-looking.

## Alt-V5 Focused Fast-Blocker Trace Wave

- No runtime challenger was attempted in this wave. The diagnostic used the filtered duel trace path: `smart_automove_pro_reliability_duel_trace_probe` with `SMART_AUTOMOVE_VARIANTS=alternating_mana_rows,forward_bridge_mana_rows`, `SMART_PRO_RELIABILITY_DUEL_FILTER=vs_shipping_fast`, `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_alt_v5_fast_focus`, `SMART_PRO_RELIABILITY_REPEATS=4`, and `SMART_PRO_RELIABILITY_GAMES=3`.
- The useful result is that the focused Fast blocker bucket still did not produce one repeated mechanism. Across `24` Fast games, the trace produced `4` regressions, `6` improvements, and `14` flat results.
- Every regression move pair was singleton: white `l10,3;l9,2` vs `l10,3;l9,3`, white `l10,4;l9,4` vs `l9,3;l8,2`, black `l2,5;l4,3;l3,2` vs `l2,5;l4,3;l3,3`, and white `l9,4;l8,4` vs `l9,4;l8,3`.
- The printed regressions split across white safe-progress/mana ordering and black spirit/setup surfaces. There was no repeated board worth sending to the hotspot probe and no runtime code was changed.
- Durable outcome: do not spend on the alt-v5 focused Fast singleton pairs. The filtered trace path is useful for cheap recurrence checks, but this seed gives no promotable Fast repair.

## Alt-V5 Focused Normal Outer-Edge Trace Wave

- No runtime challenger was attempted in this wave. The diagnostic used `smart_automove_pro_reliability_duel_trace_probe` with `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows`, `SMART_PRO_RELIABILITY_DUEL_FILTER=vs_shipping_normal`, `SMART_PRO_RELIABILITY_SEED_TAG=pro_turn_planner_reliability_alt_v5_normal_focus`, `SMART_PRO_RELIABILITY_REPEATS=4`, and `SMART_PRO_RELIABILITY_GAMES=3`.
- The useful result is that the focused Normal `outer_edge_mana_rows` bucket still did not produce one repeated mechanism. Across `24` Normal games, the trace produced `3` regressions, `5` improvements, and `16` flat results.
- Every regression move pair was singleton: black `l1,4;l2,5` vs `l1,4;l1,6;l2,7`, black `l1,5;l0,3;l1,3` vs `l1,6;l2,7`, and white `l9,4;l8,3` vs `l9,6;l9,8;l8,8`.
- The printed regressions split across white safe-progress over spirit setup, black spirit head-accept over mana, and early black mana-vs-spirit setup ordering. There was no repeated board worth hotspotting and no runtime code was changed.
- Durable outcome: do not spend on the alt-v5 focused Normal singleton pairs. This seed again argues that `outer_edge_mana_rows` is not currently exposing a promotable standalone repair.

## Sampled Gate Refresh After Alt-V5 Focused Traces

- No runtime challenger was attempted in this wave. A clean `runtime-preflight` passed, then sampled `pro-reliability` was rerun for retained `frontier_pro_v2_guarded` against `shipping_pro_search`.
- Promotion failed. Pro passed at win rate `1.0000`, confidence `0.9998`, and frontier average `154.74ms`; Normal was below the confidence target at `0.9167`, `0.9968`, `192.00ms`; Fast failed at `0.8333`, `0.9807`, `172.51ms`.
- The weak variant rows were the same as before: Normal `outer_edge_mana_rows` at `0.5000`, Fast `alternating_mana_rows` at `0.5000`, and Fast `forward_bridge_mana_rows` at `0.5000`.
- Durable outcome: the focused alt-v5 singleton-only traces did not make the retained frontier promotable. Keep runtime code untouched until a focused probe exposes a repeated mechanism under one of those three weak rows.

## Focused Fast Nonwin Trace Wave

- No runtime challenger was attempted in this wave. The diagnostic was `smart_automove_pro_reliability_nonwin_trace_probe` with `SMART_AUTOMOVE_VARIANTS=alternating_mana_rows,forward_bridge_mana_rows`, `SMART_PRO_RELIABILITY_DUEL_FILTER=vs_shipping_fast`, `SMART_PRO_RELIABILITY_REPEATS=3`, and `SMART_PRO_RELIABILITY_GAMES=2`.
- The useful result is that targeting the exact failed Fast rows by frontier nonwins still did not collapse to one mechanism. The trace logged `5` nonwins and all printed move pairs were singleton.
- The nonwins were black `l0,10;l0,9` vs shipping `l4,0;l5,0;mb`, white `l9,6;l8,7` vs shipping `l9,6;l7,7;l8,8`, black `l0,6;l1,6` vs shipping `l2,3;l3,4`, black `l2,5;l0,5;l1,6` vs shipping `l2,5;l4,7;l3,8`, and white `l9,6;l7,4;l7,3` vs shipping `l9,6;l7,6;l7,7`.
- Durable outcome: do not spend runtime code on this focused Fast nonwin set. It spans already-known singleton black bridge/mana, black setup, black late recovery, white safe-progress, and white head-accept surfaces.

## Focused Normal Nonwin Trace Wave

- No runtime challenger was attempted in this wave. The diagnostic was `smart_automove_pro_reliability_nonwin_trace_probe` with `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows`, `SMART_PRO_RELIABILITY_DUEL_FILTER=vs_shipping_normal`, `SMART_PRO_RELIABILITY_REPEATS=3`, and `SMART_PRO_RELIABILITY_GAMES=2`.
- The useful result is that the exact failed Normal row still did not produce one repair mechanism. The trace logged only `2` nonwins, both singleton and different.
- The nonwins were late black `l1,6;l1,5` vs shipping `l2,6;l3,7`, and early white `l10,4;l9,5` vs shipping `l9,4;l8,3`. In both games, shipping also lost, so these are not direct frontier-only win blockers.
- Durable outcome: do not spend runtime code on this focused Normal nonwin set. It is too small, split across different first-diff surfaces, and not enough to explain the sampled gate failure by one local repair.

## Active Context-Shipping Row Gate Wave

- No runtime challenger survived this wave. Temporary test-only sweep candidates were removed before commit.
- The diagnostics compared `frontier_pro_v2_guarded`, `frontier_pro_v2_raw`, and `shipping_pro_search_control` through `smart_automove_pro_profile_attribution_probe` on the active-blocker seed with `SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows`, then checked candidates through `smart_automove_pro_promotion_dashboard_probe` with `SMART_PRO_DASHBOARD_PANEL_FILTER=active_blockers`.
- Guarded-vs-raw attribution on isolated `outer_edge_mana_rows` produced only one outcome split: a raw-better Normal white turn-three move with action unavailable, mana available, `window=0`, `deny=0`, and `drainer_safety=-1`. It did not explain why raw later loses active Pro/Fast `outer_edge_mana_rows`.
- Shipping-control-vs-raw attribution over the full active blockers showed raw harms active `outer_edge_mana_rows` in Pro/Fast on action-plus-mana states with `window=0`, `deny=0`, and `drainer_safety=2`, while raw's useful Normal evidence was not isolated by the same feature.
- A row composite using raw ProV2 for `outer_edge_mana_rows` was not promotable: active Pro `2-4`, active Normal `5-1`, active Fast `4-2`. The exact-context shipping fallback variant repaired active Fast to `6-0`, but active Pro stayed at `3-3` and active Normal stayed at `5-1`.
- Broadening the `forward_bridge_mana_rows` white turn-three shipping fallback to the full mana-available `window=0` / `deny=0` / `drainer_safety=-1` context did not move active Pro; it remained `3-3`.
- Durable outcome: do not continue first-divergence context-shipping gates on the sampled-pass row composite. They can align individual early moves and clean up active Fast, but they have not improved the active Pro outcome. The next spend needs either arbitrary-candidate decision-record/nonwin tracing or a genuinely new utility feature that clears active Pro and sampled Pro together.

## Sweep-Candidate Decision Record Harness Wave

- No runtime challenger survived this wave. The retained work is diagnostic-only under the ignored test harness.
- `smart_automove_pro_sweep_decision_record_probe` now compares any registered `ProProfileSweepCandidate` against a same-seed `shipping_pro_search_control` replay and aggregates either nonwins or outcome deltas. The runner exposes it as `./scripts/run-automove-experiment.sh pro-sweep-decision-record <candidate>`.
- The probe was smoke-validated on `frontier_pro_v2_guarded` with a tiny Fast `alternating_mana_rows` slice, then used on the active-blocker panel for guarded. Active guarded remained mixed: Pro had `3/6` candidate nonwins, Normal had `1/6`, and Fast had `4/6`. Fast had two `outer_edge_mana_rows` regressions on action-plus-mana `window=0` / `deny=0` / `drainer_safety=2`, plus one `forward_bridge_mana_rows` regression and one `alternating_mana_rows` flat nonwin.
- The same harness checked raw ProV2's canonical sampled Pro failure. Raw produced `7/12` nonwins across `alternating_mana_rows`, `center_spoke_mana_rows`, `forward_bridge_mana_rows`, `split_flank_mana_rows`, and `inner_wedge_mana_rows`. The two regressions were singleton contexts: `forward_bridge_mana_rows` white turn one and `inner_wedge_mana_rows` black turn two.
- Durable outcome: raw's sampled Pro failure is still not a one-feature rescue target, and active guarded still does not justify another context-shipping patch. Use the new sweep-candidate decision record stage to kill singleton-heavy candidates before writing runtime code.

## ProV3 Utility Switch Active Fast No-Go

- No runtime challenger survived this wave. A temporary test-only sweep candidate preserved guarded wrapper branches and only overrode `frontier_execute` roots when a safe candidate had non-worse primary utility axes inside a guarded score window of `96`.
- The active Fast dashboard killed the candidate before sampled validation. Against shipping Fast it scored `4-2`, but `outer_edge_mana_rows` was `0-2`, so it did not solve the active blocker set.
- The same active panel against current guarded scored only `2-4`. It improved `outer_edge_mana_rows` to `2-0`, but lost `alternating_mana_rows` and `forward_bridge_mana_rows` at `0-2` each.
- The tighter guard variants (`no_alt_inner`, stable-openings) were not run because the base failure already showed the tradeoff: the score-window switch helps active `outer_edge` against guarded, still loses active `outer_edge` against shipping, and harms the two other active Fast rows against guarded.
- Durable outcome: do not retry score-window utility switches, surface-gain tie breakers, or guarded-wrapper-preserved utility overrides as the next Pro challenger. The next utility feature must explain this row tradeoff before any runtime edit is worth keeping.

## No-Late-Black Fallback Active Dashboard Refresh

- No runtime challenger survived this wave. The existing test-only `frontier_pro_v2_no_late_black_fallback` candidate was rerun through `smart_automove_pro_promotion_dashboard_probe` with `SMART_PRO_DASHBOARD_PANEL_FILTER=active_blockers`.
- The refreshed active shipping panel failed broadly: Pro `3-3`, Normal `5-1`, Fast `3-3`. Pro and Fast both had `outer_edge_mana_rows` at `0-2`; Normal split `alternating_mana_rows` at `1-1`.
- Candidate-vs-guarded was also only a mixed tradeoff: `4-2` overall, with `outer_edge_mana_rows` and `forward_bridge_mana_rows` both split `1-1`, and only `alternating_mana_rows` at `2-0`.
- Durable outcome: do not use no-late-black fallback as a current structural baseline or near-promotion scaffold. It no longer has an active panel shape worth tracing further unless a new decision-record aggregate exposes a repeated mechanism under the active `outer_edge` losses.

## Iterdeep Row Composite Active Refresh

- No runtime challenger survived this refresh. The sampled-pass guarded iterative-deepening row composite was recreated only as a test-only diagnostic scaffold: guarded iterative deepening by default, raw ProV2 for `offset_arc_mana_rows`, alpha-window iterative deepening plus `1.25x` nodes for `inner_wedge_mana_rows`, and offset-1 plus `1.25x` nodes for `outer_edge_mana_rows`, `forward_bridge_mana_rows`, and `corner_chain_mana_rows`.
- The active-blocker dashboard killed the recreated composite again: vs shipping Pro `3-3`, Normal `3-3`, Fast `5-1`; vs guarded `5-1`. All active Pro variants split `1-1`, active Normal `outer_edge_mana_rows` stayed `0-2`, and active Fast still split `outer_edge_mana_rows`.
- A narrow outer-edge context-shipping repair did not make the line promotable. It improved active Normal to `4-2`, but active Pro stayed `3-3` and Fast stayed `5-1`.
- Decision-record follow-up showed no single repair surface: active Pro nonwins split across `outer_edge_mana_rows` flat, `alternating_mana_rows` flat, and a `forward_bridge_mana_rows` early-white-fallback regression.
- Durable outcome: do not spend another iteration on the sampled-pass iterative-deepening row composite or outer-edge context-shipping repair. The next challenger needs a new ProV3 utility feature or structural selector that clears sampled Pro and active Pro together rather than rotating the active blocker rows across opponent modes.

## Exact Evidence And Portfolio Scout No-Go

- No runtime challenger survived this wave. Temporary test-only sweep candidates were removed before commit.
- Full exact static evaluation and root exact tactics were stopped as cost failures before producing useful active-panel evidence.
- Broad exact-lite checks did finish the active Pro row but failed immediately: `4-2` overall, with `outer_edge_mana_rows` and `forward_bridge_mana_rows` both split `1-1`.
- Guarded/raw/shipping portfolio majority was too expensive and still failed active Pro: `3-3` overall, `outer_edge_mana_rows` `0-2`, average candidate move time `356.18ms`.
- Raw-on-divergence was also killed on active Pro: `2-4` overall, `outer_edge_mana_rows` `0-2`, average candidate move time `255.32ms`.
- Durable outcome: do not continue exact-evidence toggles or multi-search guarded/raw/shipping agreement portfolios as the next challenger. They either hit cost before strength or reproduce the active Pro `outer_edge_mana_rows` failure before sampled validation is worth running.

## ProV3 Reply-Risk Selector Scout No-Go

- No runtime challenger survived this wave. Temporary test-only sweep candidates were removed before commit.
- `frontier_pro_v3_reply_floor_utility` preserved guarded wrapper branches and only reconsidered `frontier_execute` roots with better reply-floor safety plus comparable `TurnEngineUtility`. The active Pro dashboard killed it immediately: `3-3` overall, `outer_edge_mana_rows` `0-2`, `alternating_mana_rows` `1-1`, `forward_bridge_mana_rows` `2-0`, average `212.86ms`.
- The active Pro decision record showed the reply-floor scout did not address the sharp opening regression. The remaining `outer_edge_mana_rows` loss still diverged on black turn two through `frontier_execute` / `engine_post_search`, `l0,5;l1,6` vs shipping `l0,4;l1,5`.
- `frontier_pro_v3_ranked_reply_guard` only changed roots when the advisor approved guarded through `ApprovedReplyRiskGuard` and a safer, better-ranked shortlist root had comparable utility. It improved active Pro to `4-2`, removed candidate regressions against shipping on that slice, and moved `outer_edge_mana_rows` from `0-2` to `1-1`.
- The ranked reply-guard line still failed the full active dashboard: Pro `4-2`, Normal `5-1`, Fast `2-4`. Fast `outer_edge_mana_rows` remained `0-2`, while Pro still had flat losses on `outer_edge_mana_rows` and `alternating_mana_rows` where shipping also lost.
- Durable outcome: do not retry reply-floor-only switching or generic ranked `ApprovedReplyRiskGuard` deference. Rank deference is a useful diagnostic signal for the active Pro `outer_edge` regression, but by itself it rotates the active panel and does not supply the missing winning policy for flat Pro/Fast losses.

## ProV3 Pressure And Dirty Reply-Risk No-Go

- No runtime challenger survived this wave. Temporary test-only sweep candidates were removed before commit.
- `frontier_pro_v3_pressure_blend` treated advisor ordered and preserved roots as one pool, then tried to override `ApprovedReplyRiskGuard` selections when a safe root had stronger concrete pressure features: scoring windows, mana progress, pickups, or spirit setup. The active Pro dashboard killed it at `3-3`, with `outer_edge_mana_rows` still `0-2`, `alternating_mana_rows` `1-1`, and `forward_bridge_mana_rows` `2-0`.
- `frontier_pro_v3_dirty_reply_roots` disabled `prefer_clean_reply_risk_roots` under the guarded ProV2 config. It made the active Pro panel worse: `2-4` overall, `outer_edge_mana_rows` `0-2`, `alternating_mana_rows` `1-1`, and `forward_bridge_mana_rows` `1-1`.
- Durable outcome: do not retry pressure-blend scoring over advisor ordered/preserved roots or dirty reply-risk root preference toggles. They fail before sampled validation and do not address the active Pro `outer_edge_mana_rows` blocker.

## Black Turn-Two And White Head Active Scout No-Go

- No runtime challenger survived this wave. Temporary test-only sweep candidates were removed before commit.
- `frontier_pro_v3_black_t2_action_mana_fallback` preserved guarded ProV2 except for black turn two with one mon move already spent and both action and mana available, where it mirrored retained shipping search.
- The active Pro dashboard improved but still failed: `4-2` overall, `outer_edge_mana_rows` `1-1`, `alternating_mana_rows` `1-1`, and `forward_bridge_mana_rows` `2-0`.
- Sweep decision records on the same active Pro seed showed no remaining candidate regressions. The two remaining nonwins were flat losses where shipping also lost: white turn five `outer_edge_mana_rows`, `window=1`, `deny=1`, rejected `SpiritImpact` head; and white turn three `alternating_mana_rows`, mana-only, `window=0`, `deny=0`, rejected `SafeOpponentManaProgress` head.
- `frontier_pro_v3_black_t2_plus_white_head` kept the black turn-two fallback and returned the retained ProV2 rejected head on those two white flat-loss shapes. It improved active Pro to `5-1` and active Normal to `5-1`, with active Pro `outer_edge_mana_rows` fixed to `2-0`.
- The same head scout collapsed active Fast to `1-5`: `outer_edge_mana_rows` `0-2`, `alternating_mana_rows` `0-2`, and `forward_bridge_mana_rows` `1-1`. Fast decision records split across white and black `frontier_execute` contexts, including regressions where shipping control won and flat losses where shipping also lost.
- Durable outcome: black turn-two shipping fallback and scoped white rejected-head acceptance are useful diagnostic signals but not promotable mechanisms. They rotate the active panel across opponent modes and still lack a winning policy for active Fast flats, so do not spend runtime code on these gates without a broader selector feature that clears Fast at the same time.

## Variant-Scoped White Head Scout No-Go

- No runtime challenger survived this wave. Temporary test-only sweep candidates were removed before commit.
- The rerun first measured `frontier_pro_v3_black_t2_action_mana_fallback` across the full active dashboard instead of only Pro. It failed at Pro `4-2`, Normal `5-1`, and Fast `2-4`; Fast `outer_edge_mana_rows` was still `0-2`.
- `frontier_pro_v3_black_t2_plus_outer_edge_head` added only the outer-edge white turn-five rejected-head case. It was behaviorally identical on the active dashboard: Pro `4-2`, Normal `5-1`, Fast `2-4`.
- `frontier_pro_v3_black_t2_plus_alternating_head` added only the alternating white turn-three mana-only rejected-head case. It was also behaviorally identical: Pro `4-2`, Normal `5-1`, Fast `2-4`.
- Durable outcome: narrowing the white-head layer by variant does not recover the previous broad-head Pro/Normal improvement and does not help active Fast. Do not keep spending on variant-scoped rejected-head gates in this family.

## Guarded Preaccept Utility Floor No-Go

- No runtime challenger survived this wave. A temporary test-only sweep candidate preserved guarded wrapper branches, then only on `frontier_execute` replaced an accepted head with the preaccept root when that preaccept root was legacy-selected or legacy-full-pool-selected, non-vulnerable, and not worse on primary `TurnEngineUtility` axes.
- The retained decision-record aggregate before the candidate stayed mixed. Sampled retained decisions had no Pro regressions but still showed a Normal nonwin and Fast regressions split across `outer_edge_mana_rows`, `alternating_mana_rows`, and `forward_bridge_mana_rows`. The active-blocker aggregate split across rejected-head, accepted-head, advisor-approved, legacy, and candidate-live statuses, so there was no single preaccept mechanism.
- The active dashboard killed the candidate before sampled validation: vs shipping Pro `3-3`, Normal `5-1`, Fast `2-4`; vs guarded `3-3`. Pro and Fast both left `outer_edge_mana_rows` at `0-2`, while active Fast also split `alternating_mana_rows` and `forward_bridge_mana_rows` at `1-1`.
- Average move time rose to roughly `217ms` Pro, `295ms` Normal, and `282ms` Fast on the active dashboard, so the candidate added cost without creating a promotable row.
- Durable outcome: do not retry generic head/root utility-floor or legacy preaccept vetoes. The remaining blocker requires a new utility feature that explains when head acceptance, reply-risk approval, and preaccept preservation are one-sided across sampled and active Pro/Normal/Fast; replaying legacy preaccept roots is not that feature.

## Expanded Policy-Winner Active Refresh No-Go

- No runtime challenger survived this wave. The run was diagnostic-only through `smart_automove_pro_policy_winner_probe` with the expanded portfolio: retained guarded, alternating-white edge mana, white-opening utility mana, shipping-control, raw ProV2, no-selected-followup, full-scored reply guard, and no-low-budget.
- The active-blocker panel still showed oracle coverage but not selector shape: Pro had `3` guarded baseline wins and `3` policy wins, Normal `5` / `1`, and Fast `2` / `4`, with `no_policy_wins=0` in all three duels.
- The policy wins conflict by opponent mode. Active Pro wants full-scored reply guard for white `outer_edge_mana_rows` turn three under action+mana, `window=0`, `deny=0`, and `drainer_safety=2`; active Fast wants shipping-control on a matching white `outer_edge_mana_rows` context. Fast also needs shipping-control for black outer-edge and white forward-bridge, plus raw ProV2 only for a black alternating late-fallback row.
- Durable outcome: the expanded policy portfolio is an oracle diagnostic, not a promotable selector. Do not build another static active policy-winner selector over these labels; add a new utility/root-evaluation feature first, then use panel-filtered policy-winner runs to check whether the new policy separates Pro/Normal/Fast instead of memorizing row openings.

## Dual-Progress Spirit Feature No-Go

- No runtime challenger survived this wave. Temporary test-only candidates were removed before commit.
- Forced-root oracles on the active white `outer_edge_mana_rows` Pro/Fast conflict were later corrected for full-opening horizon. The original unadjusted probe granted a fresh 96 plies from first-diff boards and overstated wins at `12/16` Pro and `15/16` Fast. With `SMART_PRO_FORCED_ROOT_ORACLE_START_PLY` set to the recorded first-diff ply, the same boards are `8/16` Pro and `9/16` Fast, and the guarded-selected roots reproduce the full-opening losses.
- A direct dual-progress SpiritImpact preference selected safe `SpiritImpact` roots with both `supermana_progress` and `opponent_mana_progress` plus short safe paths. It failed the active dashboard at Pro `3-3`, Normal `5-1`, Fast `3-3`; Pro `outer_edge_mana_rows` stayed `0-2`, while Fast split every blocker variant at `1-1`.
- A dual-progress-triggered shipping fallback was worse on Pro and still not promotable: active Pro `2-4`, Normal `5-1`, Fast `3-3`. It improved the guarded delta on `outer_edge_mana_rows`, but not the shipping-strength gate.
- Durable outcome: do not use dual-progress SpiritImpact as the next shared utility feature or as a shipping-fallback trigger. It is another active Normal/guarded-delta repair that leaves active Pro and Fast blocker strength unresolved.

## Policy Matrix Mechanism-Class And Fallback Relaxation No-Go

- No runtime challenger survived this wave. The retained source change is harness-only: `smart_automove_pro_policy_matrix_probe` can now emit `PRO_POLICY_MATRIX_MECHANISM_CLASS` when `SMART_PRO_POLICY_MATRIX_INCLUDE_MECHANISM_CLASS=true`.
- A broad active Fast full-portfolio matrix with mechanism classes was stopped for cost after printing only the first record. Keep this flag on filtered panel/duel/candidate slices until a cheap run shows a mechanism worth widening.
- The first completed narrow run used the active-blocker Fast slice with guarded as baseline and `shipping_pro_search_control` plus raw ProV2 as candidates. Shipping control was mixed (`candidate_better=3`, `baseline_better=1`), while raw ProV2 was non-regressing on the slice (`candidate_better=2`, `baseline_better=0`).
- The raw gains came from bypassing guarded wrapper fallbacks: one early-white fallback and one late-black shipping fallback. That made fallback relaxation look plausible but also matched a historically risky selector family.
- A temporary test-only candidate relaxed only those two surfaces: white turn-one start used a safe quiet mana-utility root, and black turn-four weak-window action+mana used raw guarded-config frontier. The sampled dashboard killed it immediately against shipping Pro at `7-5` (`0.5833`, confidence `0.6128`), with weak rows spread across `center_spoke_mana_rows`, `alternating_mana_rows`, `inner_wedge_mana_rows`, and `forward_bridge_mana_rows`. The remaining Normal/Fast sampled work was stopped and the source candidate was discarded.
- Durable outcome: `PRO_POLICY_MATRIX_MECHANISM_CLASS` is useful for narrowing which candidate deltas deserve a focused probe, but active Fast raw fallback wins are not enough to relax guarded fallbacks. Do not retry early-white or late-black fallback removal unless a new utility/root-evaluation feature first clears sampled Pro/Normal/Fast.

## Live Board Compression And Sweep Surface Prune

- The live board was compressed back to current reset state, reset portfolio, next commands, stoplight rules, and a short no-go summary. Historical probe diaries remain in this archive and in `docs/automove-knowledge.md`.
- No runtime behavior changed. Public Pro still routes through `frontier_pro_v2_guarded`, and `shipping_pro_search` remains the retained search-only baseline.
- Stale test-only sweep candidates were removed from the active experiment runner: `frontier_pro_v2_no_late_black_fallback`, `frontier_pro_v2_head_rerank`, `frontier_pro_v2_no_spirit_family`, `frontier_pro_v2_no_mid_tactical_guard`, and `frontier_pro_v2_expansion_224`.
- The retained reset portfolio remains guarded, alternating-white edge mana, white-opening utility mana, shipping-control, raw ProV2, no-selected-followup, full-scored reply guard, and no-low-budget.
- Durable outcome: future agents should treat removed candidate IDs as archived evidence only. Recreate a pruned surface only as a new, named test-only candidate after outcome-corpus or ProV4 evidence shows a clean mechanism that the old ablation could not provide.

## Outcome Corpus Route Coverage Scout

- No runtime challenger survived this iteration. The retained source change is harness-only: `smart_automove_pro_policy_matrix_probe` now emits `PRO_POLICY_MATRIX_GLOBAL_MECHANISM_ROUTE` next to global mechanism separation lines, including route labels and the panels/duels behind candidate-only wins and baseline-better saves.
- The structural scout dashboard for retained guarded was `not_promising`: sampled vs shipping measured Pro `7-5`, Normal `7-5`, Fast `6-6`; active blockers measured Pro `3-3`, Normal `5-1`, Fast `2-4`.
- The bounded global outcome corpus over the reset portfolio still had oracle coverage but not source permission: `total_games=12`, guarded baseline wins `5`, candidate-any wins `12`, candidate-only wins `7`, no-policy wins `0`, and stoplight `repeated_mechanism_class`.
- Route coverage confirmed the broad safe zero-window pressure key is contaminated: `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe` had `candidate_only_games=10`, `baseline_better_games=5`, and route label `baseline_save_risk` across both sampled and active panels.
- The cleanest remaining signals are timing/stage diagnostics, not selectors. White `exact_timing color=white turn_bucket=turn3_4 mons_moves=mons0` was `cross_panel_clean` at `3` candidate-only games; rejected-head `engine_post_search` stage classes also reached cross-panel or cross-budget clean at `3`; black early timing and ManaTempo-to-SpiritImpact stage classes were active-only cross-budget clean at `3`.
- Durable outcome: do not implement a selector from broad exact pressure or from policy labels. The next useful step is a narrow corpus-record plus decision-probe rerun over sampled/active Normal/Fast slices to see whether the clean timing/stage routes collapse to exact boards or point to a below-branch timing/continuation feature.

## State-Aware Outcome Route Rerun

- No runtime challenger survived the state-aware rerun. The retained source change is harness-only: route coverage now deduplicates mechanism evidence by portfolio state before assigning route labels.
- The focused reset portfolio outcome corpus over sampled and active Normal/Fast slices still had oracle coverage: `total_games=8`, guarded baseline wins `5`, candidate-any wins `8`, candidate-only wins `3`, no-policy wins `0`, and global stoplight `repeated_mechanism_class`.
- State-aware route coverage confirmed that broad exact pressure is worse than the raw emission count implied. `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe` had candidate-only games `6`, baseline-better games `5`, candidate-only states `2`, baseline-better states `3`, and label `baseline_save_risk`.
- The only route that stayed clean across deduplicated states was coarse white timing: `axis=exact_timing color=white turn_bucket=turn3_4 mons_moves=mons0 can_action=true can_mana=true opp_win=false` had candidate-only games `3`, baseline-better games `0`, candidate-only states `2`, and label `cross_panel_clean`.
- Raw timing/stage repeats collapsed after deduplication. The white timing records split between sampled Normal `inner_wedge_mana_rows` and active Fast `outer_edge_mana_rows`, with different pressure windows, first moves, and winning policies, so this is not source permission.
- Durable outcome: state counts trump mechanism emission counts. Future reset work should either run a broad state-aware global scan or build Outcome Corpus V2 persistent records with explicit timing, continuation, pressure-window, and root-feature fields before adding another selector.

## Mechanism-Axis Record Output

- No runtime challenger survived this iteration. The retained source change is harness-only: `PRO_POLICY_MATRIX_RECORD` and `PRO_POLICY_MATRIX_CORPUS_RECORD` now include `mechanism_axes` and `baseline_better_mechanism_axes` when mechanism classification is enabled.
- The broad reset-portfolio global scan had oracle coverage but no promotion permission: `total_games=12`, guarded baseline wins `5`, candidate-any wins `12`, candidate-only wins `7`, no-policy wins `0`, and stoplight `repeated_mechanism_class`.
- Broad exact pressure remained contaminated. `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe` had candidate-only games `10`, baseline-better games `5`, candidate-only states `5`, baseline-better states `3`, and label `baseline_save_risk`.
- The strongest clean route was active-only: `axis=stage baseline_stage=engine_post_search head_accepted=false head_primary=Some("equal") pre_family=ManaTempo head_family=Some(SpiritImpact)` had candidate-only games `3`, baseline-better games `0`, candidate-only states `3`, and label `cross_budget_clean` across active Pro/Fast only.
- A focused active Pro/Fast rerun with decision probes showed the route is still diagnostic. Candidate-only wins were all `outer_edge_mana_rows` active blockers and split by color, turn, winning policy, branch, and advisor status; the records did not identify one source feature below the existing policy labels.
- Durable outcome: do not write runtime code for this route. The next useful step is to rerun the focused route with the new record-level mechanism axes, then either retire it or build a small Outcome Corpus V2 postprocessor that groups records by mechanism axes, policy, branch, color, and first move automatically.

## Route Fragmentation Counts

- No runtime challenger survived the follow-up active Pro/Fast rerun. The retained source change is harness-only: `PRO_POLICY_MATRIX_GLOBAL_MECHANISM_ROUTE` now reports candidate-only and baseline-better fragmentation counts for policy, variant, color, branch, and first-move pair.
- Record-level mechanism axes confirmed that the active-only `engine_post_search` route is not source-ready. The `pre_family=ManaTempo head_family=Some(SpiritImpact)` route still had candidate-only games `3`, baseline-better games `0`, and candidate-only states `3`, but the matching records split across three winning policies, both colors, two branch transitions, and three first-move pairs.
- The focused slice had no baseline-better saves, but it was active-blocker-only and all candidate-only states were `outer_edge_mana_rows`; broad exact-pressure remains killed by the earlier sampled/active baseline-save evidence.
- Durable outcome: use route fragmentation counts before opening raw records. A clean state route with multiple policies, branches, or first-move pairs is routing evidence only; do not write runtime code until a lower-level shared feature survives this fragmentation check.

## Broad Route Recommendation Scan

- No runtime challenger survived this iteration. The retained source change is harness-only: `smart_automove_pro_policy_matrix_probe` now emits `PRO_POLICY_MATRIX_GLOBAL_ROUTE_RECOMMENDATION` whenever portfolio mechanism-class route coverage is enabled.
- The broad global reset scan over the retained portfolio still had oracle coverage but no source permission: `total_games=12`, guarded baseline wins `5`, candidate-any wins `12`, candidate-only wins `7`, no-policy wins `0`, and stoplight `repeated_mechanism_class`.
- Broad exact pressure stayed contaminated. `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe` had candidate-only games `10`, baseline-better games `5`, candidate-only states `5`, and baseline-better states `3`.
- There were zero clean low-fragmentation routes. The top clean route remained the active-only `engine_post_search` stage route with candidate-only states `3`, but it split across three policies, two branch transitions, and three first-move pairs. Other clean timing/stage routes were similarly fragmented or singleton/pair evidence.
- Durable outcome: the next useful work is Outcome Corpus V2/postprocessor output that ranks and joins mechanism records automatically. Do not write runtime selectors from the current broad route scan unless a future recommendation line reports `narrow_low_fragmentation_route`.

## Route Bucket Shortlist Output

- No runtime challenger survived this iteration. The retained source change is harness-only: `smart_automove_pro_policy_matrix_probe` now emits `PRO_POLICY_MATRIX_GLOBAL_ROUTE_BUCKET` lines that shortlist top routes by `clean_low_fragmentation`, `clean_fragmented`, `baseline_risk`, and `singleton_candidate` buckets.
- The broad scan result that motivated the change was `build_outcome_corpus_v2`: `candidate_signal_routes=109`, `clean_low_fragmentation_routes=0`, `clean_fragmented_routes=8`, `baseline_risk_routes=14`, best clean route `axis=stage baseline_stage=engine_post_search head_accepted=false head_primary=Some("equal") pre_family=ManaTempo head_family=Some(SpiritImpact)`, and best baseline risk `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe`.
- The bucket output was smoke-validated on a bounded active Fast global-only slice with `SMART_PRO_POLICY_MATRIX_ROUTE_BUCKET_LIMIT=2`; it emitted `singleton_candidate` bucket rows and preserved the global recommendation output.
- Durable outcome: future broad route scans should read the recommendation line and bucket shortlist first. If the recommendation stays `build_outcome_corpus_v2`, preserve postprocessor/harness work and skip runtime source selection.

## Outcome Corpus Record Axis Filter

- No runtime challenger survived this iteration. The retained source change is harness-only: `smart_automove_pro_policy_matrix_probe` now accepts `SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER`, a comma-separated list of mechanism-axis substrings used to filter printed `PRO_POLICY_MATRIX_CORPUS_RECORD` and `PRO_POLICY_MATRIX_RECORD` lines.
- The filter computes mechanism axes for printed records even when the aggregate mechanism-class flags are off, but it does not change outcome totals, route aggregation, or stoplight labels.
- The filter was smoke-validated on the active Fast `engine_post_search` route with `SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER='axis=stage baseline_stage=engine_post_search'`. The run emitted only the matching `PRO_POLICY_MATRIX_CORPUS_RECORD` and `PRO_POLICY_MATRIX_RECORD` pair plus the normal summaries.
- Durable outcome: use route bucket output to pick an axis, then use the record-axis filter for focused corpus inspection. The filtered records are Outcome Corpus V2 input, not runtime source permission.

## Record Filter Summary Output

- No runtime challenger survived this iteration. The retained source change is harness-only: filtered matrix runs now emit `PRO_POLICY_MATRIX_RECORD_FILTER_SUMMARY`, summarizing matching corpus and trace records by panel, duel, policy, outcome, portfolio class, variant, color, branch transition, and first-move pair count.
- The focused active Pro/Fast query for `axis=stage baseline_stage=engine_post_search head_accepted=false head_primary=Some("equal") pre_family=ManaTempo head_family=Some(SpiritImpact)` still returned `build_outcome_corpus_v2`: `total_games=6`, guarded baseline wins `1`, candidate-any wins `6`, candidate-only wins `5`, no-policy wins `0`, and `clean_low_fragmentation_routes=0`.
- The selected route remained no-source. It matched four corpus records across `vs_shipping_pro` and `vs_shipping_fast`, black and white, policies `frontier_pro_v2_no_selected_followup_projection`, `frontier_pro_v3_full_scored_reply_guard`, and `shipping_pro_search_control`, two branch transitions, and four first-move pairs.
- The summary output was smoke-validated on a one-state active Fast slice. It reported one corpus record, one trace record, and one policy/branch/pair for the filtered route, preserving the normal recommendation output.
- Durable outcome: `engine_post_search` remains an Outcome Corpus V2 fixture, not source permission. Use `PRO_POLICY_MATRIX_RECORD_FILTER_SUMMARY` to retire or prioritize future bucket axes without manually counting raw filtered records.

## Record Filter Detail Output

- No runtime challenger survived this iteration. The retained source change is harness-only: filtered matrix runs now emit capped `PRO_POLICY_MATRIX_RECORD_FILTER_DETAIL` rows for the filtered population by duel, candidate policy, outcome, portfolio class, variant, color, branch transition, and first-move pair.
- The focused active Pro/Fast query for `axis=safety_progress baseline_safety=safe baseline_progress=safe_step_progress winner_safety=safe winner_progress=spirit_development` returned `build_outcome_corpus_v2`: `total_games=6`, guarded baseline wins `1`, candidate-any wins `6`, candidate-only wins `5`, no-policy wins `0`, and `clean_low_fragmentation_routes=0`.
- The top clean route was no-source despite zero baseline-better saves. It matched four candidate-only corpus records across `vs_shipping_pro` and `vs_shipping_fast`, black and white, policies `frontier_pro_v2_no_selected_followup_projection`, `frontier_pro_v3_full_scored_reply_guard`, and `shipping_pro_search_control`, two branch transitions, and four first-move pairs.
- The detail output was smoke-validated on a one-state active Fast slice. It reported one corpus record, one trace record, and detail rows for duel, candidate policy, outcome, portfolio class, variant, color, branch, and first-move pair.
- Durable outcome: the safety/progress route joins `engine_post_search` as an Outcome Corpus V2 fixture. Use detail rows to identify fragmentation before opening raw filtered records; do not write runtime selectors unless a future route recommendation reports `narrow_low_fragmentation_route`.

## Policy Matrix Log Summarizer

- No runtime challenger survived this iteration. The retained source change is a postprocess script: `scripts/summarize-automove-policy-matrix-log.py` reads logged `PRO_POLICY_MATRIX_*` JSON lines and emits one digest containing event counts, global summary, stoplight, route recommendation, route buckets, and filtered-record permissions.
- The focused active Pro/Fast safety/progress detail rerun still returned `build_outcome_corpus_v2`: `total_games=6`, guarded baseline wins `1`, candidate-any wins `6`, candidate-only wins `5`, no-policy wins `0`, `clean_low_fragmentation_routes=0`, `clean_fragmented_routes=9`, and `baseline_risk_routes=6`.
- The summarizer classified the route as `postprocess_only` and the filtered records as `fragmented_no_source`. The filtered details were candidate policies `shipping_pro_search_control=2`, `frontier_pro_v2_no_selected_followup_projection=1`, `frontier_pro_v3_full_scored_reply_guard=1`; duels `vs_shipping_pro=3`, `vs_shipping_fast=1`; variants `outer_edge_mana_rows=3`, `alternating_mana_rows=1`; colors `black=2`, `white=2`; branches `frontier_execute->candidate_execute=3`, `frontier_execute->frontier_execute=1`; and four singleton first-move pairs.
- Durable outcome: keep the safety/progress route as a postprocessor fixture and use the summarizer on the next bounded global-only outcome-corpus run before opening raw route records.

## Policy Matrix Total State Cap

- No runtime challenger survived this iteration. The retained source change is harness-only: `smart_automove_pro_policy_matrix_probe` now accepts `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT`, a true global cap across panels and duels.
- The attempted all-panel/all-budget reset digest with only `SMART_PRO_POLICY_MATRIX_STATE_LIMIT=2` was stopped after about fourteen minutes. That cap is per panel/duel, so the broad reset portfolio still fans out across sampled and active panels plus Pro/Normal/Fast duels.
- The total cap was smoke-validated with `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=1`, where global summary reported exactly `total_games=1` and `state_limit_hit=true`.
- A focused active Fast full-portfolio digest with `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=2`, `SMART_PRO_POLICY_MATRIX_GLOBAL_ONLY=true`, `SMART_PRO_POLICY_MATRIX_INCLUDE_CORPUS_RECORDS=false`, and `SMART_PRO_POLICY_MATRIX_MAX_PLIES=56` completed successfully. It stayed no-source: `baseline_save_risk_only`, `candidate_signal_routes=19`, `clean_low_fragmentation_routes=0`, `clean_fragmented_routes=0`, and `baseline_risk_routes=1`.
- Durable outcome: use total-capped, panel/duel-filtered digests to rank reset routes before any raw record inspection. The next slice should be active blockers vs shipping Pro with the same total cap and summarizer workflow.

## Policy Matrix Corpus Decision

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now emits `corpus_decision`, which combines global summary, stoplight, and route recommendation into a first-read decision such as `coverage_gap`, `baseline_save_risk`, `singleton_no_source`, `postprocess_only`, or `inspect_for_source`.
- The total-capped active Pro full-portfolio digest completed successfully with two total states and stayed no-source: global stoplight `coverage_gap`, `candidate_only_wins=1`, `no_policy_wins=1`, route recommendation `singleton_candidate_routes`, zero clean routes, and summarizer `corpus_decision=coverage_gap`.
- The top active Pro route was broad zero-window safe exact pressure as singleton evidence only: one candidate-only state, two candidate policies (`frontier_pro_v3_full_scored_reply_guard` and `shipping_pro_search_control`), one branch transition, and two first-move pairs. The other checked state was a no-policy win, so current policy labels cannot cover the slice.
- Durable outcome: the next bounded digest should check active blockers vs shipping Normal with the same total cap and summarizer workflow. Do not write runtime selectors from active Pro singleton routes.

## Policy Matrix Next Action Digest

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now emits `next_action`, derived from `corpus_decision`, so the first-read digest names the operator action instead of only the no-source label.
- The total-capped active Normal full-portfolio digest completed successfully with two total states and stayed no-source: global stoplight `shared_only`, `candidate_only_wins=0`, `no_policy_wins=0`, route recommendation `no_candidate_route`, `candidate_signal_routes=0`, summarizer `corpus_decision=no_candidate_route`, and `next_action=try_next_slice`.
- Guarded shared both checked wins. The only route pressure came from full-scored reply guard baseline-better rows on one active outer-edge white state, so this slice provides no runtime source permission and no route to inspect.
- Durable outcome: total-capped active Fast, Pro, and Normal slices all stayed no-source for different reasons. Keep the reset work in Outcome Corpus V2/postprocess and use the next sampled Normal digest to validate whether the sampled false-positive surface still reports save risk or singleton residue under the retained summarizer.

## Sampled Normal Singleton Digest

- No runtime challenger survived this iteration. No source code was retained; the useful artifact is the sampled Normal no-go captured in the live board and durable knowledge.
- The total-capped sampled Normal full-portfolio digest completed successfully with two total states and stayed no-source: global stoplight `singleton_selector_pressure`, `candidate_only_wins=1`, `no_policy_wins=0`, route recommendation `singleton_candidate_routes`, summarizer `corpus_decision=singleton_no_source`, and `next_action=widen_or_archive_singleton`.
- The top route was a single white `inner_wedge_mana_rows` state. Its strongest class was `axis=advisor baseline_advisor=approved:ApprovedReplyRiskGuard:SpiritImpact winner_advisor=ordered:ReplyRiskShortlist:SpiritImpact`, but the route split across `frontier_pro_v2_no_selected_followup_projection` and `shipping_pro_search_control`, `frontier_execute->frontier_execute` and `frontier_execute->candidate_execute`, and two first-move pairs.
- Durable outcome: archive this as sampled Normal singleton residue. Do not write a SpiritImpact advisor/order selector from this slice; the next low-cost refresh is sampled Fast with the same total cap and summarizer workflow.

## Sampled Fast Shared-Only Digest

- No runtime challenger survived this iteration. No source code was retained; the useful artifact is the sampled Fast shared-only no-go captured in the live board and durable knowledge.
- The total-capped sampled Fast full-portfolio digest completed successfully with two total states and stayed no-source: global stoplight `shared_only`, `candidate_only_wins=0`, `no_policy_wins=0`, route recommendation `no_candidate_route`, summarizer `corpus_decision=no_candidate_route`, and `next_action=try_next_slice`.
- Guarded shared both checked wins. Existing policy components only added baseline-better pressure on sampled `split_flank_mana_rows`, including `frontier_pro_v2_no_selected_followup_projection`, `frontier_pro_v2_raw`, and `frontier_pro_v3_full_scored_reply_guard` rows across black late-shipping and white early-fallback branches.
- Durable outcome: keep sampled Fast as guarded-covered save-risk evidence for the current portfolio. The next low-cost refresh is sampled Pro with the same total cap and summarizer workflow, after which the standardized sampled/active reset digest set should be complete enough to move back to Outcome Corpus V2/postprocess design.

## Sampled Pro Shared-Only Digest

- No runtime challenger survived this iteration. No source code was retained; the useful artifact is the sampled Pro shared-only no-go captured in the live board and durable knowledge.
- The total-capped sampled Pro full-portfolio digest completed successfully with two total states and stayed no-source: global stoplight `shared_only`, `candidate_only_wins=0`, `no_policy_wins=0`, route recommendation `no_candidate_route`, summarizer `corpus_decision=no_candidate_route`, and `next_action=try_next_slice`.
- Guarded shared both checked `inner_wedge_mana_rows` wins. Existing policy components only added baseline-better pressure, led by `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe` with `baseline_better_games=5` and `baseline_better_states=2` across raw, no-selected-followup, full-scored, and shipping-control rows.
- Durable outcome: the standardized two-state sampled Pro/Normal/Fast and active Pro/Normal/Fast reset refresh stayed no-source in every slice. Use the next true-global capped digest to consolidate the route picture, then move the work back to Outcome Corpus V2/postprocess design rather than runtime selectors.

## True-Global Source Blocker Digest

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now emits `source_blocker`, a compact reason to keep runtime source untouched for the summarizer's `corpus_decision`.
- The true-global full-portfolio digest with `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=6` completed successfully and stayed no-source: total games `6`, guarded baseline wins `5`, candidate-only wins `1`, no-policy wins `0`, stoplight `singleton_selector_pressure`, route recommendation `baseline_save_risk_only`, summarizer `corpus_decision=baseline_save_risk`, and `next_action=avoid_selector`.
- `source_blocker` identified baseline-save risk on `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe`: candidate-only states `1`, baseline-better states `3`. The matching bucket had candidate-only evidence only in sampled `vs_shipping_pro` split-flank black, while baseline-better saves spanned center-spoke and inner-wedge plus raw, no-selected-followup, full-scored, and shipping-control rows.
- Durable outcome: broad zero-window safe exact pressure remains a source-work kill. The next work should stay in Outcome Corpus V2/postprocess; do not widen or encode exact-pressure selectors unless a future corpus feature separates candidate wins from guarded saves below policy, branch, and first-move labels.

## Policy Matrix Multi-Log Rollup

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now emits `log_rollup` when more than one log is passed, including per-log summaries plus decision, next-action, and source-blocker counts.
- The current true-global full-portfolio digest was rerun with `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=6`, `SMART_PRO_POLICY_MATRIX_GLOBAL_ONLY=true`, corpus records disabled, portfolio mechanism classes enabled, route buckets capped at five, and max plies capped at `56`. It completed successfully and stayed no-source: total games `6`, guarded baseline wins `5`, candidate-any wins `6`, candidate-only wins `1`, no-policy wins `0`, stoplight `singleton_selector_pressure`, and route recommendation `baseline_save_risk_only`.
- The summarizer reported `corpus_decision=baseline_save_risk`, `next_action=avoid_selector`, and `source_blocker.kind=baseline_save_risk` on `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe`, with candidate-only states `1` versus baseline-better states `3`.
- The multi-log rollup was smoke-validated by passing the same capped log twice. It reported `decision_counts=[baseline_save_risk: 2]`, `next_action_counts=[avoid_selector: 2]`, and one repeated source blocker for the exact-pressure baseline-save-risk route.
- Durable outcome: the next Outcome Corpus V2 iteration should compare fresh small slice logs with one summarizer invocation and read `log_rollup` before raw buckets. Runtime source remains untouched unless a future rollup shows a repeated source blocker has cleared baseline-save risk and fragmentation.

## Pro-Budget Coverage-Gap Rollup

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now adds `log_rollup.rollup_decision`, `rollup_next_action`, and `rollup_permission` so mixed multi-log no-source outcomes do not require manual interpretation from counts.
- The sampled and active Pro-budget reset slices were run with `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=2`, `SMART_PRO_POLICY_MATRIX_DUEL_FILTER=vs_shipping_pro`, global-only output, corpus records disabled, portfolio mechanism classes enabled, route buckets capped at five, and max plies capped at `56`.
- The combined rollup stayed no-source: sampled Pro was `corpus_decision=no_candidate_route`, `next_action=try_next_slice`, guarded baseline wins `2`, candidate-only wins `0`, and no-policy wins `0`; active Pro was `corpus_decision=coverage_gap`, `next_action=add_policy_or_root_feature`, guarded baseline wins `0`, candidate-only wins `1`, and no-policy wins `1`. The rollup reported `rollup_decision=coverage_gap`, `rollup_next_action=add_policy_or_root_feature`, and `rollup_permission=no_source`.
- A focused active Pro rerun with corpus records enabled confirmed the gap shape. The candidate-only state was active `outer_edge_mana_rows`, candidate black, opening `0 0 b 0 0 1 0 0 2 n03y0xs0xd0xa0xn04/n08e0xn02/n11/n11/xxmxxmn07xxmxxm/xxQxxmn03xxUn03xxMxxQ/xxMxxMn07xxMxxM/n11/n07Y0xn03/n04A0xn06/n03E0xn01D0xS0xn04 8`, with `shipping_pro_search_control` and `frontier_pro_v3_full_scored_reply_guard` winning. The same opening as candidate white was a true no-policy state: every current portfolio policy lost.
- Durable outcome: active Pro coverage now points below the current policy portfolio. Keep runtime source untouched; the next useful work is an Outcome Corpus V2 postprocess view that compacts `portfolio_class=no_policy_win` corpus records into per-state coverage-gap entries before designing a new policy/root feature.

## Coverage Gap Entries

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now emits `coverage_gap_entry_count` and `coverage_gap_entries`, compacting `PRO_POLICY_MATRIX_CORPUS_RECORD` rows with `portfolio_class=no_policy_win` by panel, duel, seed tag, repeat, opening index, variant, and side.
- The focused active Pro record-bearing slice was rerun with `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=2`, `SMART_PRO_POLICY_MATRIX_PANEL_FILTER=active_blockers`, `SMART_PRO_POLICY_MATRIX_DUEL_FILTER=vs_shipping_pro`, `SMART_PRO_POLICY_MATRIX_GLOBAL_ONLY=false`, corpus records enabled, portfolio mechanism classes enabled, route buckets capped at five, and max plies capped at `56`. The run stayed `corpus_decision=coverage_gap` and `next_action=add_policy_or_root_feature`.
- The summarizer emitted `coverage_gap_entry_count=1`. The compact entry identified the active `outer_edge_mana_rows` candidate-white no-policy state from opening `0 0 b 0 0 1 0 0 2 n03y0xs0xd0xa0xn04/n08e0xn02/n11/n11/xxmxxmn07xxmxxm/xxQxxmn03xxUn03xxMxxQ/xxMxxMn07xxMxxM/n11/n07Y0xn03/n04A0xn06/n03E0xn01D0xS0xn04 8`: all seven current portfolio policies lost, `record_count=7`, `candidate_count=7`, `first_diff_count=3`, `pair_count=4`, and `branch_count=3`.
- The top compact mechanism axes were `none` for four non-divergent same-outcome records and `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe` for two divergent records. The earliest listed divergence was `frontier_pro_v3_full_scored_reply_guard` at first-diff ply `7`, board `0 0 w 0 0 1 0 0 3 n05d0xa0xn04/n08e0xn02/n06s0xn04/n04y0xn03xxmn02/xxmxxmn08xxm/xxQxxmn03xxUn03xxMxxQ/xxMxxMn07xxMxxM/n11/n07Y0xn03/n04A0xS0xn05/n03E0xn01D0xn05 8`.
- Durable outcome: the no-policy gap is now visible without raw corpus inspection. The next useful diagnostic is a forced-root oracle on the earliest compact divergence board with `SMART_PRO_FORCED_ROOT_ORACLE_START_PLY=7`; keep runtime source untouched unless that root-set probe finds a repeated below-policy feature.

## Active Pro Forced-Root Root Source Oracle

- No runtime challenger survived this iteration. The retained source change is harness-only: `smart_automove_pro_forced_root_oracle_probe` now accepts `SMART_PRO_FORCED_ROOT_ORACLE_ROOT_SOURCE`, letting scored roots come from a runtime profile while a test-only continuation policy plays out the forced root.
- The first forced-root pass on the active Pro coverage-gap board was discarded as horizon-inflated. It used the oracle default `SMART_PRO_FORCED_ROOT_ORACLE_MAX_PLIES=96` even though the source outcome corpus used `56` plies. After setting `SMART_PRO_FORCED_ROOT_ORACLE_START_PLY` to the first-diff ply and `SMART_PRO_FORCED_ROOT_ORACLE_MAX_PLIES=56`, the tempting rank-0 ply-7 root `l9,5;l8,5` correctly remained a loss.
- The corrected ply-7 active `outer_edge_mana_rows` board with guarded continuation had `7/16` winning roots, while the same guarded root source continued by `frontier_pro_v3_full_scored_reply_guard` had `4/16` winning roots. The full-scored reply-guard continuation is weaker on this board, but both runs show the current root set contains winning choices beneath the selected losing root.
- Two later compact no-policy divergence boards were also root-covered under the corrected horizon. The ply-20 board had `4/16` winning roots, all printed winners in `SpiritImpact`; the ply-40 board had `1/17`, a rank-9 `ManaTempo` root. Across the three checked boards, winners were low/mid ranked and did not collapse to one family, rank, or source-selector rule.
- Durable outcome: the active Pro coverage gap is not a missing-root ceiling; it is root-ranking or feature-design pressure. Keep runtime source untouched until a postprocess or ProV4 root-feature pass finds a repeated below-policy feature across corrected-horizon winning roots.

## Forced-Root Oracle Log Summarizer

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-forced-root-oracle-log.py` reads `FORCED_ROOT_ORACLE_*` JSON lines and emits a compact digest with root coverage, per-board winner summaries, repeated winner axes, and winner-vs-nonwinner separability.
- The summarizer was validated on fresh corrected-horizon active Pro oracle logs for the three compact `outer_edge_mana_rows` no-policy boards. The ignored release harness passed for all three board probes, and the digest reported `summary_count=3`, `tested_roots=49`, `wins=12`, `losses=37`, and `groups_with_wins=3`.
- The digest decision stayed no-source: `oracle_decision=fragmented_root_features`, `next_action=return_to_outcome_corpus_feature_extraction`, and `promising_repeated_axes=[]`. Broad repeats were contaminated by losing roots: `rank_band=rank8_plus` covered all three winner groups but also `16` nonwinner roots, and the all-group utility/safety axes had `37` nonwinner roots.
- Durable outcome: do not build a ProV4/root comparator from current forced-root rank, family, same-window, safety, or `TurnEngineUtility` axes alone. The next useful work returns to Outcome Corpus V2 feature extraction for a discriminator not already visible in the forced-root rows.

## Coverage Gap Sibling State Summaries

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now adds `same_opening_sibling_states` to each `coverage_gap_entries` item, summarizing other candidate-side states from the same panel, duel, seed tag, repeat, opening index, and variant.
- The active Pro outcome-corpus command from the live board was rerun with `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=2`, corpus records enabled, portfolio mechanism classes enabled, route buckets capped at five, and max plies capped at `56`. The run passed and stayed `corpus_decision=coverage_gap`: total games `2`, guarded baseline wins `0`, candidate-only wins `1`, no-policy wins `1`, route recommendation `singleton_candidate_routes`.
- The old compact coverage-gap entry showed only the candidate-white no-policy state. With sibling summaries, the same entry now reports one same-opening sibling: candidate black on the same `outer_edge_mana_rows` opening, with `portfolio_class_counts=[candidate_only_win: 7]`, winning policies `shipping_pro_search_control,frontier_pro_v3_full_scored_reply_guard`, top axes led by zero-window safe exact pressure, and four first divergences.
- Durable outcome: the active Pro gap is a cross-side asymmetry hypothesis, not source permission. Widen only enough to see whether this same-opening candidate-only/no-policy pairing repeats before adding any runtime or ProV4 comparator logic.

## Active Pro Sibling Pairing Widening

- No runtime challenger survived this iteration. No source or harness code was retained; the useful artifact is the no-source result from the wider active Pro same-opening pairing check.
- The active Pro outcome-corpus command was rerun with `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=4`, corpus records enabled, portfolio mechanism classes enabled, route buckets capped at five, and max plies capped at `56`. The run passed and stayed `corpus_decision=coverage_gap`: total games `4`, guarded baseline wins `1`, candidate-any wins `3`, candidate-only wins `2`, no-policy wins `1`, shared wins `1`, and route recommendation `build_outcome_corpus_v2`.
- The widened compact view still emitted `coverage_gap_entry_count=1`. The only no-policy entry was the same active `outer_edge_mana_rows` candidate-white state, and it still had one same-opening candidate-black sibling with `portfolio_class_counts=[candidate_only_win: 7]` and winning policies `shipping_pro_search_control,frontier_pro_v3_full_scored_reply_guard`.
- The extra checked states added candidate-only and shared evidence but did not add a second same-opening no-policy/candidate-only pair. The best clean route was still postprocess-only exact pressure: `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe`, with candidate-only states `2`, two candidate policies, and three first-move pairs.
- Durable outcome: archive the cross-side pairing as singleton evidence. The next useful work is broader record-bearing Outcome Corpus V2 feature extraction that ranks lower-level axes across candidate-only wins, baseline saves, shared wins, and no-policy gaps before any runtime or ProV4 selector work.

## Corpus Axis Summary

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now emits `corpus_axis_summary`, a compact record-level axis report built from `PRO_POLICY_MATRIX_CORPUS_RECORD` rows.
- The new report groups record axes into `candidate_better`, `baseline_better`, `no_policy`, and `same_outcome` classes, deduplicates by corpus state, and labels each axis as `baseline_save_risk`, `coverage_gap_axis`, `repeated_candidate_axis`, `singleton_candidate_axis`, `baseline_better_only`, or `shared_or_neutral`.
- The unfiltered record-bearing Outcome Corpus V2 slice was run with `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=8`, corpus records enabled, portfolio mechanism classes enabled, route buckets capped at eight, and max plies capped at `56`. It completed successfully but the true global cap was consumed by sampled `vs_shipping_pro` states: total games `8`, guarded baseline wins `6`, candidate-only wins `2`, shared wins `6`, no-policy wins `0`, and route recommendation `baseline_save_risk_only`.
- The digest stayed no-source: `corpus_decision=baseline_save_risk`, `next_action=avoid_selector`, and `source_blocker.kind=baseline_save_risk`. The new `corpus_axis_summary` showed the same blocker without raw records: `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe` had candidate-better states `2` but baseline-better states `4`, with `branch_count=4`, `pair_count=13`, and all four sampled variants represented.
- Durable outcome: exact zero-window safe pressure remains a selector kill. Use `corpus_axis_summary` before raw records on the next explicit active-blocker Pro slice; runtime source remains untouched unless the summarizer reports `inspect_for_source`.

## Active Pro Corpus Axis Decision View

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now adds `corpus_axis_summary.axis_decision_counts` and `corpus_axis_summary.top_axes_by_decision`, so coverage-gap, repeated-candidate, baseline-better, and save-risk axes can be read without manually scanning separate top lists.
- The explicit active-blocker Pro record-bearing slice was run with `SMART_PRO_POLICY_MATRIX_PANEL_FILTER=active_blockers`, `SMART_PRO_POLICY_MATRIX_DUEL_FILTER=vs_shipping_pro`, `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=8`, corpus records enabled, portfolio mechanism classes enabled, route buckets capped at eight, and max plies capped at `56`. It completed all available states and stayed no-source: total games `6`, guarded baseline wins `2`, candidate-only wins `3`, shared wins `2`, no-policy wins `1`, and route recommendation `build_outcome_corpus_v2`.
- The digest stayed `corpus_decision=coverage_gap`, `next_action=add_policy_or_root_feature`, `route_permission=postprocess_only`, and `clean_low_fragmentation_routes=0`. The remaining no-policy entry is still the active `outer_edge_mana_rows` candidate-white state with all seven policies losing and one same-opening candidate-black sibling.
- The top `coverage_gap_axis` was broad exact pressure: `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe`, with candidate-better states `3`, no-policy states `1`, baseline-better states `0`, branch count `3`, and pair count `13`. It is active-clean but fragmented and already sampled-killed as baseline-save risk, so it is not source permission.
- The only `repeated_candidate_axis` rows were active-only leads: `axis=safety_progress baseline_safety=safe baseline_progress=safe_step_progress winner_safety=safe winner_progress=spirit_development` with candidate-better states `2`, and `axis=role baseline_role=selected baseline_live=top3_live winner_role=pre_accept+legacy+legacy_full_pool winner_live=top3_live` with candidate-better states `2`. Both had zero active baseline-better and zero no-policy states but still split by branch/pair.
- Durable outcome: keep runtime source untouched. The next useful check is a focused sampled Pro record-axis run for those two repeated active candidate axes; if either shows sampled baseline saves, archive the axis and return to ProV4/root-feature or continuation-stability feature work.

## Sampled Pro Axis Filter Match Split

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now adds `axis_filter_matches` to each summarized `record_filters` entry, splitting comma-separated `SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER` tokens into separate corpus-axis summaries.
- The focused sampled Pro run checked the two active-only repeated candidate axes together with `SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=8`, corpus records enabled, portfolio mechanism classes enabled, route buckets capped at eight, max plies capped at `56`, and the full reset portfolio.
- The global sampled result stayed no-source: `corpus_decision=baseline_save_risk`, `next_action=avoid_selector`, route recommendation `baseline_save_risk_only`, guarded baseline wins `6`, candidate-any wins `8`, candidate-only wins `2`, shared wins `6`, no-policy wins `0`, and zero clean low-fragmentation routes. The source blocker remained broad zero-window safe exact pressure, with candidate-only states `2` versus baseline-better states `4`.
- Per-token filter matches split the active leads cleanly. The safety/progress token `axis=safety_progress baseline_safety=safe baseline_progress=safe_step_progress winner_safety=safe winner_progress=spirit_development` was `baseline_save_risk` with record count `4`, state count `2`, candidate-better states `1`, baseline-better states `1`, and same-outcome states `2`. The role token `axis=role baseline_role=selected baseline_live=top3_live winner_role=pre_accept+legacy+legacy_full_pool winner_live=top3_live` was only `singleton_candidate_axis` with record count `1`, state count `1`, and candidate-better states `1`.
- Durable outcome: retire both active repeated-candidate axes as runtime selector leads. The safety/progress axis has sampled save contamination, and the role axis is singleton evidence. The next useful work is Outcome Corpus V2 decision-stage timing and continuation-stability feature extraction before any ProV4/root-feature comparator or runtime selector.

## Timing Continuation Axis Records

- No runtime challenger survived this iteration. The retained source change is harness/postprocess-only: policy-matrix corpus and trace records now emit `timing_continuation_axes`, and the summarizer includes those axes in `corpus_axis_summary`, `axis_filter_matches`, and coverage-gap state summaries.
- The one-state active Pro validation passed through the ignored `smart_automove_pro_policy_matrix_probe` harness and printed timing axes on `PRO_POLICY_MATRIX_CORPUS_RECORD` / `PRO_POLICY_MATRIX_RECORD`. A focused filter on `axis=decision_timing ply_bucket=ply0_7` printed exactly one matching corpus record and one matching trace record, proving `SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER` can now target timing axes.
- The wider active Pro timing run completed all six available states and stayed no-source: `corpus_decision=coverage_gap`, `next_action=add_policy_or_root_feature`, guarded baseline wins `2`, candidate-only wins `3`, shared wins `2`, no-policy wins `1`, route permission `postprocess_only`, and one coverage-gap entry. Repeated timing axes either included the no-policy state, baseline-better saves, or only singleton candidate evidence.
- The matching sampled Pro timing run stayed no-source: `corpus_decision=baseline_save_risk`, `next_action=avoid_selector`, guarded baseline wins `6`, candidate-only wins `2`, shared wins `6`, no-policy wins `0`, and source blocker `axis=exact_pressure window=window0 deny=deny0 attack=false drainer_safety=safe` with candidate-only states `2` versus baseline-better states `4`.
- The combined sampled/active timing rollup reported `rollup_decision=baseline_save_risk`, `rollup_next_action=avoid_selector`, and `rollup_permission=no_source`. Top timing axes were save-contaminated or coverage-gap axes: no-rejoin/different-final continuation had candidate-better, baseline-better, same-outcome, and no-policy states; `frontier_execute->candidate_execute`, early-white fallback, late-black fallback, and black ply-0-to-7 decision timing all picked up sampled baseline-better saves.
- Durable outcome: timing/continuation axes are useful corpus features, but they are not source permission in their first sampled/active Pro pass. The next useful work is a cross-budget axis view that joins these axes across Pro/Normal/Fast before any ProV4 comparator or runtime selector.

## Cross-Budget Axis Summary

- No runtime challenger survived this iteration. The retained source change is postprocess-only: `scripts/summarize-automove-policy-matrix-log.py` now emits `cross_budget_axis_summary`, joining `PRO_POLICY_MATRIX_CORPUS_RECORD` axes by panel, normalized seed family, repeat, opening index, variant, and side across Pro/Normal/Fast duels.
- The active-blocker validation used `SMART_PRO_POLICY_MATRIX_STATE_LIMIT=1` with Pro, Normal, and Fast duels, corpus records enabled, portfolio mechanism classes enabled, route buckets capped at five, max plies capped at `56`, and the full reset portfolio. The ignored `smart_automove_pro_policy_matrix_probe` harness passed in `171.64s`.
- The digest stayed no-source: `corpus_decision=coverage_gap`, `next_action=add_policy_or_root_feature`, `route_permission=no_source`, and `source_blocker.kind=coverage_gap`. The new cross-budget view reported `record_count=21`, `joined_state_count=1`, `axis_state_group_count=106`, and decision counts `shared_or_neutral=39`, `coverage_gap=33`, `non_regressing_repair=20`, `baseline_save_risk=9`, `partial_repair_coverage_gap=3`, and `budget_conflict=2`.
- The two budget-conflicted axes were broad zero-window safe exact pressure and no-rejoin/different-final continuation stability. Both joined the same active `outer_edge_mana_rows` candidate-white opening side across all three budgets: Fast had candidate-better evidence, Normal had baseline-better save evidence, and Pro carried no-policy pressure.
- Durable outcome: the first cross-budget view confirms the current timing/continuation and exact-pressure leads are not source candidates. The next useful check is a two-state active-blocker cross-budget widening through the same summarizer; runtime source stays untouched unless `cross_budget_axis_summary` shows repeated all-budget or non-regressing repairs with no baseline-save and no coverage-gap contamination.
