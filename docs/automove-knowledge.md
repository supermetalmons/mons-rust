# Automove Knowledge

This file keeps durable automove rules and reusable heuristics only.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the workflow, `AUTOMOVE_IDEAS.md` for the live state, and `docs/automove-archive.md` for retired wave detail.

## Stable Runtime Truths

- Public Pro routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` is the retained search-only baseline.
- Release wiring is intentionally narrower than the experiment surface: public `Pro` dispatch goes through `MonsGameModel::public_runtime_inputs` to `select_frontier_pro_v2_guarded_inputs`, while `automove_experiments` and experiment profile selectors are only included under `#[cfg(test)]`.
- Probe paths are diagnostics only; they do not describe shipping behavior.
- Promotion evidence comes from direct frontier-vs-baseline duels, not fixture churn alone.
- `runtime-preflight` still matters after promotion: exact-lite is hard, stage-1 CPU is advisory for Pro.
- Automove users can play any current `GameVariant`; promotion evidence must cover variant breadth, not just Classic.

## Experiment Rules

- Pick one hypothesis before editing runtime code.
- Probe first when the mechanism is unclear. Do not spend canonical gates on a guess.
- Treat retained Classic fixtures as regression controls, not proof that a candidate is stronger across variants.
- Use seeded sampled variants for quick kill/pass evidence, then all-variant confirmation for promotion.
- Separate `pre_accept` search choice from final `engine_post_search` output before changing shared heuristics.
- A seam can move while the duel gate still fails; local seam coverage is not duel strength.
- Passing the small `pro-reliability` gate does not guarantee confirm readiness.
- Passing sampled variants does not guarantee all-variant readiness.
- Stacked narrow late-ply head/advisor overrides are especially suspect. One direct line cleared sampled `pro-reliability` at `1.0000 / 0.9167 / 0.9167` and then collapsed in all-variant confirm at `0.6667 / 0.7292 / 0.6667`.
- Treat rotated residue from a discarded challenger as provisional until a clean-tree retained trace confirms it.
- Do not promote a live `first_diff_ply` board into retained until the retained replay reproduces the same final shipping-selected root; copied late-ply snapshots can collapse back to frontier on clean replay.
- Runtime cost is a real gate. A candidate that wins local seams while drifting further into the `1.5x+` advisory band is not an upgrade.
- Wrapper-only reroutes, fallback widening, shortlist widening, and metadata-only advisor changes saturate quickly; durable progress usually needs shared approval, head, or scoring logic.
- Extending a retained shipping-fallback path to a wider black weak-window surface is not automatically safer than advisor surgery; one direct late-black shipping-fallback expansion fixed the traced sampled boards and still rotated the sampled gate down to Fast `0.7500`.
- When an explicit-variant nonwin trace splits the remaining sampled residue across different turn families or selector stages, keep the wave diagnostic-only. The current `outer_edge_mana_rows` plus Fast `forward_bridge_mana_rows` / `corner_chain_mana_rows` residue did not collapse to one mechanism, and isolated Fast `classic` did not reproduce.
- A widened recurrence trace can still fail to justify runtime code even when one pair repeats. On `alternating_mana_rows,forward_bridge_mana_rows`, the only repeated Fast pair was the archived white head-accept seam `l9,6;l7,4;l7,3` vs `l9,6;l7,6;l7,7`, while every other Fast regression pair stayed singleton; that is still mixed residue, not a shared runtime hypothesis.
- Isolating a single blocker variant can still stay too mixed for code. `forward_bridge_mana_rows` isolated to `24` Fast games repeated the white head-accept seam `l9,6;l7,4;l7,3` vs `l9,6;l7,6;l7,7` three times, and `alternating_mana_rows` repeated the black mana sibling seam `l2,7;l1,6` vs `l2,7;l1,8` twice, but both variants still kept five singleton seams; one repeated pair per variant is still not enough.
- A blocker-hotspot fingerprint can still be a false lead. The current late black `outer_edge_mana_rows`, black `alternating_mana_rows`, and repeated white `forward_bridge_mana_rows` boards all kept the same full selector pool against `shipping_pro_search` and all showed frontier paying extra exact/pickup/tactical work, but the actual move drift still split across two `pre_accept` seams and one head-accept seam with different exact-opportunity contexts.
- A widened recurrence trace over the full active blocker set can invalidate narrower repeated-pair stories. On `outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows`, the broader `repeats=4` / `games=3` duel trace produced Pro `3`, Normal `3`, and Fast `7` regressions with every per-duel move pair still singleton; earlier isolated repeated pairs stopped dominating once the whole blocker set was traced together.
- Similar-looking same-family `ManaTempo` drifts can still split between tie-order and real scoring residue. The current late black `outer_edge` seam and one new Pro white mana sibling seam had zero residual delta, zero search-eval delta, and identical `TurnEngineUtility`, while a second Pro white mana sibling seam carried a real residual split from `spirit_action_utility` and `mana_close_to_same_pool`.
- A nearby retained white mana sibling board is not automatically an extension candidate for a new white seam. The retained late white mana sibling root is a turn-eleven `SpiritImpact` / `SafeSupermanaProgress` `engine_post_search` surface with a live head plan and `window=2/deny=2`, while the new Pro `l6,7;l7,6` vs `l6,7;l7,7` seam is an early pure `ManaTempo` `engine_disabled` ordering surface with `window=0/deny=0`.
- A widened Pro-only nonwin replay can still kill a seemingly new white candidate. On the current blocker set, the new Pro white `l6,7;l7,6` vs `l6,7;l7,7` seam stayed singleton while the replay broadened into repeated black `l2,7;l1,6` vs `l2,7;l1,8` and black `l1,6;l2,7` vs shipping `l1,5;l2,3;l1,2` plus other singleton seams; that is still mixed residue, not a white runtime hypothesis.
- Similar-looking black late-mana seams can still be different mechanisms. The unresolved `outer_edge_mana_rows` black seam keeps shipping `ManaTempo l2,6;l3,7` inside the reply-risk shortlist and then drifts to lower-ranked `ManaTempo l1,6;l1,5` through `ApprovedReplyRiskGuard`, while the repeated `alternating_mana_rows` black seam keeps shipping `ManaTempo l2,7;l1,8` as legacy-selected but then jumps to outside-shortlist `DrainerSafetyRecovery l2,7;l1,6` through `ApprovedFamilyCompetition`.
- Similar-looking white `forward_bridge_mana_rows` seams can also be different mechanisms. The repeated `l9,6;l7,4;l7,3` miss is a head-accept loss on advisor-approved shipping `SpiritImpact l9,6;l7,6;l7,7`; the `l9,6;l8,7` miss is a reply-risk approval of `SafeSupermanaProgress`; and the `l9,7;l8,6` miss additionally routes through `score_window_tactical_fallback` with a preserved safe-progress representative.
- A direct white turn-three mana-only legacy-progress override on a positive-safety `window=0/deny=0` surface is still only a local repair. One cut aligned the clean `outer_edge_mana_rows` white board and reduced the explicit outer-edge Normal trace from `2` nonwins to `1`, but the sampled gate stayed at Pro `1.0000`, Normal `0.9167`, Fast `0.8333`.
- A remaining late-black outer-edge board can still be too local to spend on. The clean probe showed shipping `l2,6;l3,7` was already the legacy-selected, reply-risk-shortlisted `ManaTempo` root while frontier approved lower-ranked `l1,6;l1,5`, but the sampled Fast residue was still mixed across black/white and across `engine_post_search` plus head-accepted surfaces, so the wave stayed diagnostic-only.

## Retained Package Lessons

- The promoted retained package is a set of narrow, mutually constrained repairs around `frontier_pro_v2_guarded`; keep them together unless a future gate proves otherwise.
- White turn-three and turn-five repairs are intentionally split by mechanism: no-action recovery, selected-rank search-only fallback, equal-surface ProV1 tiebreaks, and final head rejection are not interchangeable.
- Black late setup/progress repairs belong in advisor family competition only when setup roots are close-surface, already shortlisted, and backed by retained controls.
- A direct black progress-vs-setup wrapper mirror is unsafe: it fixed one local board and failed confirm by rotating onto later Fast seams.
- A broad white turn-three recovery or raw ProV1 search-only reroute is unsafe: it fixes known siblings but flips retained vulnerable guards or rotates Normal losses.
- A root-rank separator is not enough for white search-order fallbacks; selected rank was the safe separator for the retained negative-deny sibling.

## Residual Map

- `black_recovery_branch`: not a root-availability problem. The current live model prefers the approved plain-spirit root or a no-guard ProV1 spirit replay over shipping under frontier metrics. Static exact scoring is not a promotable direct spend: broad scoped exact selected the wrong spirit sibling, and reply-floor-only exact aligned the local board but failed retained sampled reliability.
- Black Fast progress-vs-setup residue: not solved by direct wrapper mirrors, material-only scoring, final-selector-only changes, or a combined scoped material-plus-rank advisor exception. The safe-progress edge largely comes from residual board-state material/cooldown terms, but a line that zeroed those terms and let higher-scoring setup roots outrank the safe-progress root fixed the local board and still failed sampled reliability. Reopen only with a mechanism that improves retained multi-variant duel strength, not just the local residue.
- White search-order residue: not a wrapper config, root-set, or simple rerank-cap problem. The shipping recovery root can be reachable and still lose at shortlist/reply-risk under frontier metrics; any future spend must distinguish unresolved siblings from retained vulnerable guards below that surface.

## Diagnostic Toolbox

- `smart_automove_pro_reliability_duel_trace_probe`
- `smart_automove_pro_reliability_nonwin_trace_probe`
- `smart_automove_pro_reliability_hotspot_probe`
- `smart_automove_pro_triage_retained_churn_probe`
- `smart_automove_pro_forced_turn_engine_retained_churn_probe`
- `smart_automove_pro_root_advisor_trace_probe`
- `black_recovery_branch_reply_floor_attribution_probe`
- `black_progress_residual_weight_attribution_probe`

Run diagnostics through the ignored test harness:

```sh
cargo test --release --lib <test_name> -- --ignored --nocapture
```

## Kill Rules

- Kill any line that fails `guardrails` or pushes off-target retained churn above `1`.
- Kill any line that does not move direct duel evidence on the candidate-vs-baseline matchup.
- Kill any line that fails sampled-variant reliability unless the failure exposes a clearly scoped harness issue.
- Kill any line that passes Classic fixtures but fails all-variant confirmation.
- Kill any sampled-pass line that only moved through stacked narrow late-ply head/advisor overrides and then broadens all-variant nonwins; the direct white late spirit-setup head block plus black weak-window mana-lane package did exactly that.
- Kill any late black shipping-fallback expansion that only fixes the traced sampled black boards and rotates Fast losses elsewhere; the direct weak-window extension did exactly that.
- Kill any wave whose explicit-variant nonwin trace splits the remaining sampled blockers across unrelated early/late or `engine_disabled` / `engine_post_search` surfaces. Archive the probe and do not guess a shared fallback from mixed residue.
- Kill any proposed standalone white late head-accept blocker when it is only the one repeated pair inside a wider Fast trace and the rest of `alternating_mana_rows` / `forward_bridge_mana_rows` residue stays singleton. A repeated pair is still not enough when the surrounding blocker set does not collapse.
- Kill any proposed isolated-variant runtime spend when that variant shows only one repeated pair and still keeps five singleton seams behind it. The isolated `forward_bridge_mana_rows` and `alternating_mana_rows` traces both met that no-go shape.
- Kill any proposed shared hotspot spend when the commonality is only frontier doing more exact/pickup/tactical work than the shipping search-only baseline. Shared cost inflation without the same drift sublayer and exact-opportunity context is not one runtime mechanism.
- Kill any proposed shared blocker spend when a widened all-blocker recurrence trace falls back to singleton-heavy regressions across Pro, Normal, and Fast. If the earlier isolated repeated pairs stop dominating once the full blocker set is traced together, there is no stable shared residue left to spend on.
- Kill any proposed shared same-family late mana scoring spend when some candidate seams are exact scoring/utility ties and others have distinct residual deltas. Equal `search_eval` plus equal `TurnEngineUtility` is not a scoring bug, and mixed tie-order vs residual surfaces are not one mechanism.
- Kill any proposed retained-extension spend when the retained board and the new seam differ on stage, family mix, and exact-opportunity context. A late `SpiritImpact` / head-plan retained board is not evidence for an early pure `ManaTempo` engine-disabled seam.
- Kill any proposed standalone white spend when a widened Pro-only replay leaves that white seam singleton and broadens the residue into repeated black seams plus more singleton drift. One white appearance inside mixed Pro residue is not enough.
- Kill any proposed shared black late-mana spend when the candidate seams diverge at the advisor layer: shortlisted same-family `ApprovedReplyRiskGuard` drift on one board and outside-shortlist `ApprovedFamilyCompetition` on the other is not one mechanism.
- Kill any proposed shared white `forward_bridge` spend when the candidate seams split across head acceptance, reply-risk approval, and runtime fallback stages. One repeated pair is still not enough when the supporting seams disagree structurally.
- Kill any white turn-three mana-only legacy-progress override that only removes the clean white half of `outer_edge_mana_rows`; the direct positive-safety `window=0/deny=0` cut did that and still left the sampled gate blocked by late black `outer_edge` plus Fast `alternating_mana_rows` and `forward_bridge_mana_rows`.
- Kill any proposed late-black same-family legacy-alignment spend if the remaining sampled Fast blockers are still mixed across colors or stages. A clean local `ApprovedReplyRiskGuard` seam is not enough by itself.
- Kill any line that stays inert at `target_changed=0 off_target_changed=0` against active `frontier_pro_v2_guarded`.
- Kill any line that only changes advisor labels or `pre_accept` metadata while the final selected root stays unchanged.
- Kill any line that widens shortlist or injection coverage without moving the approved root on the live walls.
- Kill any line that aligns live walls but still fails retained duel strength or canonical cost.
- Kill any retained-board addition whose copied board state does not replay the same shipping-selected root on a clean retained harness run.
- Kill any `black_recovery_branch` line that only forces shipping `l6,0;l6,1` above the current reply-risk and no-guard selector ordering.
- Kill any `black_recovery_branch` line that reopens broad scoped static exact or reply-floor-only static exact without a new direct duel-strength mechanism.
- Kill any black progress-vs-setup line that only dampens `fainted_mon` / `fainted_cooldown_step`; that is local-score movement, not selection movement.
- Kill any black progress-vs-setup line that patches final selection without moving the material-dampened `ApprovedReplyRiskGuard` advisor approval.
- Kill any black progress-vs-setup line that combines scoped material dampening with a higher-rank setup advisor exception unless it brings new sampled duel-strength evidence; the direct version failed Pro `0.5000`, Normal `0.5000`, Fast `0.8333`.
- Do not reopen archived profiles, archived seams, or archived wave packages without a brand-new shared hypothesis.
