# Automove Knowledge

This document keeps only durable lessons that should outlast the current session.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the operator loop and `AUTOMOVE_IDEAS.md` for the next live split.

## Stable Runtime Truths

- Shipping Pro is `runtime_current`.
- `runtime_pro_turn_engine_v30` is the only live Pro challenger.
- `runtime_pro_turn_engine_v1` is reference-only regression history.
- `runtime_pro_turn_engine_v30` is a guarded `ProV2` path with deliberate opening-book and early-white fallbacks, not a raw always-on engine.
- Full-turn planning can beat per-input expansion, but only if it survives the real ranked-root surface and direct duel evidence.
- Opportunity-context extraction is worth keeping separate from raw input search. It remains useful shared structure for planner seeds and selector guards.
- Continuation reuse and no-plan reuse matter, but cache keys must include a config fingerprint.
- Drainer safety, root reply-risk guards, and efficiency tie-breaks still earn their keep because they cheaply reject fake-good roots.
- Opening-specific latency still matters. A candidate that stalls on the first real black reply is not promotable.
- Hybrid fallbacks must respect retained opening and eligibility guards before they call expensive plan probes.
- Wrapper-only tuning saturates quickly. Once the obvious knobs are in place, the next gain usually has to come from shared engine or search code.
- Global `SmartSearchConfig` knob space is effectively exhausted. Future gains need new code or a sharper selector/search hypothesis, not another broad retune.
- Weak deferred progress heads are a repeat failure mode. Keep blocking them when the real selected root already offers a safer immediate pickup, a concrete setup window, or an unsafe non-progress root that the head does not concretely improve.
- Production wasm still needs single-shot, predictable search. Deferred or post-return work is not release-safe.
- The retained churn probe is worth keeping. For live ProV2 misses, distinguish `pre_accept` search choice from final `engine_post_search` output before changing shared heuristics.
- The runtime-faithful retained churn probe is worth keeping too. It must inject forced engine inputs before `focused_root_candidates_with_forced_inputs(...)`, or it can misclassify injected-root seams as selector churn.
- The compare-oriented hotspot probe is worth keeping. On Apr 5, 2026 it showed the bounded reliability hotspot corpus was move-identical to `runtime_current` on every real duel hotspot, with only a synthetic `quiet_positional` difference.
- The duel-replay probe is worth keeping too. On Apr 8, 2026 `smart_automove_pro_reliability_duel_trace_probe` replayed the exact `pro-reliability` seeds, exposed a repeated real fast-duel `engine_post_search` `SafeSupermanaProgress` override, and showed that the hotspot corpus was no longer sufficient by itself to explain the live wall.
- Re-running the hotspot compare after the retained `primary_pvs_sensitive_search` repair did not expose a new real duel seam. On Apr 8, 2026 every real hotspot was still move-identical to `runtime_current`, so `human_win_pro_c` remained a triage-only selector drift rather than promotion evidence.
- The current retained seam map is stable enough to plan around: `primary_pvs_sensitive_search` now has a retained late `engine_post_search` fix, `human_win_pro_c` is the only remaining retained `pre_accept` safe-progress bias, and the previously live `primary_white_harvest_loss_c_ply24`, `primary_spirit_setup`, and `primary_black_reliability_opening_3_ply4` seams still hold their retained fixes.
- `primary_spirit_setup` was a two-step bug, not one seam: the engine was force-pinning an existing low-ranked plain `SpiritImpact` head into the focused shortlist, then a completed-plan override was still allowed to replace an equivalent stronger selected plain spirit sibling. Both checks have to stay aligned.
- The black turn-two low-budget clamp was too broad. On `turn=2`, `mons_moves=1`, `action+mana` states it could suppress a stronger spirit-own-setup root; the clamp should only fire on truly resource-constrained turn-two black states.
- White `turn=3`, mana-only mid-turn wrapper regressions are real, but still not enough by themselves. On Apr 8, 2026 routing those traced boards back to the current Pro surface fixed the replayed duel choices, yet `pro-triage(primary_pro)` stayed flat at `1/52`, so the guard repair was still too local to earn more spend.
- Even a broader duel-backed wrapper bundle can still be too local. On Apr 8, 2026 routing all traced white `turn=3` mid-turn boards plus black `turn=2`/`turn=4` one-move `action+mana` boards back to the current Pro surface fixed the targeted replay boards and cleared `guardrails`, but `pro-triage(primary_pro)` still stayed flat at `1/52`.
- Simple speculative immediate-score non-regression clamps and setup-gain-only spirit-setup promotion are not enough. They can reshuffle `primary_pro` fixtures and regress direct Pro-vs-Pro without moving the `vs current Normal` wall.
- Broad same-lane `spirit_own_mana_setup_now` overrides are too blunt as well. On Apr 8, 2026 a shared ProV2 override collapsed `human_win_pro_c` and fixed one traced normal-duel board, but it reopened `primary_black_reliability_opening_3_ply4` and regressed `pro-reliability` to `0.7500` vs current Pro and `0.4167` vs current Normal/Fast.
- Late-white full-resource current-Pro guards are not a useful `human_win_pro_c` lever either. On Apr 8, 2026 a late white turn-start safe-supermana wrapper did not change the selected move on the live `human_win_pro_c` fixture.
- A late-white omitted-root reply-risk rescue is too local as well. On Apr 8, 2026 it cleared `human_win_pro_c` without reopening black reliability, but that only collapsed `pro-triage(primary_pro)` to `0/52`, which still fails because the cheap target surface must move.
- Soft followup-tolerance for `spirit_own_mana_setup_now` roots and close quiet-root normal-safety blocks are also not enough by themselves. If the runtime-faithful seam stays unchanged, kill the split before spending the canonical loop.
- Eval-only progress-head wins are too soft for unsafe late overrides. When both the selected root and the `Safe*Progress` head stay unsafe, do not let a lower-scored progress head replace the selected non-progress root unless it brings a non-eval strategic gain or a forced `score_delta` jump.
- Even a real duel-traced acceptance seam can still be too local. On Apr 8, 2026 a bounded fast-duel `Safe*Progress` override clamp matched the traced seed, but it left `pro-triage(primary_pro)` unchanged at `1/52`, so the production split was killed before `runtime-preflight` and `pro-reliability`.
- Once the wrapper-local misses are gone, the live wall can shift cleanly into later `engine_post_search` drift. On Apr 8, 2026 the wrapper bundle plus late-white omitted-root rescue left `vs current Pro` at `0` regressions / `2` improvements, but `vs current Normal` still sat at `2` regressions / `1` improvement and `vs current Fast` at `3` regressions / `3` improvements, so the next spend would have to attack post-search head acceptance rather than more wrapper or human-only fixes.
- When debugging `engine_post_search` head acceptance, print both `turn_engine_selected_override_utility(...)` and the raw selected-root `turn_engine_root_plan_utility(...)`. On Apr 8, 2026 a retained fast white turn-seven seam looked like a weak raw selected root, but the real gate was comparing against a much stronger projected `selected_override_utility`.
- A multi-chunk `ImmediateScore` near-tie `ManaTempo` clamp can still be too local. On Apr 8, 2026 it cleared `human_win_pro_c`, fixed the bounded duel-accept seams, and passed `guardrails`, `pro-triage(primary_pro)=1/52`, and `runtime-preflight`, but `pro-reliability` still failed at `0.7500` vs current Pro, `0.6667` vs current Normal, and `0.5000` vs current Fast. A sampled duel trace still diverged on a later black turn-four `engine_post_search` choice (`l1,6;l2,7` vs current `l2,3;l3,2`), so future spend has to target the broader later duel wall rather than another near-tie accept clamp.
- The sampled fast-duel black turn-four seam is wrapper-shaped but still too local on its own. On Apr 8, 2026 an isolated `turn=4`, `mons_moves=1`, `action+mana` current-Pro guard did route the live board back from `l1,6;l2,7` to current `l2,3;l3,2`, but `pro-triage(primary_pro)` still stayed unchanged at the same `human_win_pro_c`-only `1/52`.
- The remaining human drift and the sampled fast black turn-four duel seam do not currently share one projection-family fix. On Apr 8, 2026 the shared probe showed `human_win_pro_c` winning through a higher followup floor on a root whose post-root projection became `SpiritImpact -> ImmediateScore`, while the fast duel board stayed an injected vulnerable `ManaTempo` root whose post-root projection became `SafeSupermanaProgress -> ImmediateScore`.
- The live duel wall can shift into a broader white turn-three family without touching the retained cheap surface. On Apr 8, 2026 a fresh duel trace showed three white `turn=3`, `mons_moves>0`, `action=false`, `mana=true` boards across Pro/Normal/Fast were all broad fallback drifts, and one repeated white `turn=3`, `mons_moves=1`, `action=true`, `mana=true` board was still a separate accepted-head seam.
- Even a broader white turn-three mana-only reroute can still be too local. On Apr 8, 2026 routing all white `turn=3`, `mons_moves>0`, `action=false`, `mana=true` states back to the current Pro surface fixed three live duel boards, but `pro-triage(primary_pro)` still stayed unchanged at the same `human_win_pro_c`-only `1/52`.
- A repeated live accepted-head seam still is not enough without a retained target-surface foothold. On Apr 8, 2026 the white `turn=3`, `mons_moves=1`, `action+mana` fast-duel seam (`l9,4;l8,4` vs `l8,7;l7,8`) repeated twice, but the runtime-faithful retained churn probe still showed `accepted=false` on every retained `primary_pro` fixture.
- A fresh early black spirit-own-setup drift can still be the wrong merge. On Apr 8, 2026 the new normal-duel board `l0,5;l1,6` vs current `l1,5;l3,6;l2,7` looked superficially like the retained `human_win_pro_c` setup-vs-progress bias, but the focused selector probe showed `negative_deny_competes=true` and `followup_progress_competes=false`, so it was a separate negative-deny spirit-preference seam rather than the retained human followup-progress story.
- A lone `vs current Pro` white regression can still be only wrapper churn. On Apr 8, 2026 the board `l9,2;l8,3` vs current `l10,7;l9,7` looked like a new Pro-only selector miss, but the focused probe showed the configured v30 path still pre-accepted the current-style root and the outer `runtime_pro_turn_engine_v30_guarded_inputs(...)` wrapper was the piece rerouting the board through the broad white `turn=3`, `action=false`, `mana=true` fast fallback.

## Durable Workflow Rules

- Keep the active frontier small: one live idea, one retained candidate, one canonical path.
- For Pro work, compare directly against `runtime_current`.
- The canonical Pro path is `guardrails -> pro-triage(primary_pro) -> runtime-preflight -> pro-reliability`.
- `opening_reply` is a narrow fallback-order and opening-regression surface, not the default Pro surface.
- `pro-triage` only passes when the target surface changes and off-target churn stays at `<= 1`.
- `runtime-preflight` is the required stamp before duel stages unless the run is intentionally diagnostic.
- `pro-reliability` is the first real promotion gate. It must clear current `Pro`, current `Normal`, and current `Fast` at `win_rate >= 0.90`, `confidence >= 0.99`, and `candidate_avg_move_ms <= 700`.
- `pro-reliability-confirm` is the final promotion proof. Do not promote on smaller-corpus evidence.
- Prefer a fresh live `pro-reliability` sample over diagnostic probes when the wall is unclear.
- Use `triage-calibrate` only when a retained triage surface is new or no longer calibrated.
- Use `pro-opening-speed-probe` only for opening-specific regressions.
- Use the hotspot probe only after a real duel stall, and only to narrow the next code surface.
- When the hotspot corpus stays flat but the direct wall is still unclear, use the duel-replay probe before changing shared code.
- If the compare hotspot probe shows decision parity on the real hotspot cases, kill the line immediately. Selector/exact counter deltas without candidate-vs-current move differences are not promotion evidence.
- Logs, stamps, and process samples are disposable evidence, not durable memory.
- Keep ignored harness test names unique; `cargo test` substring filters can hit the wrong stage.

## Mistakes Not To Repeat

- Do not reopen archived profile IDs or retired branch families without a brand-new hypothesis.
- Do not spend another loop on wrapper-only reroutes, current-Normal fallbacks, or replay-specific acceptance clamps just because they fix a traced exact.
- Do not retain a traced white turn-three mana-only wrapper reroute unless it also moves `primary_pro`; fixing replay boards alone was not enough.
- Do not retain a broader white-turn-three plus black one-move wrapper bundle if it still leaves `pro-triage(primary_pro)` unchanged at `1/52`, even when multiple traced duel boards now match current.
- Do not reopen cache-size, memo-shape, reserve-heavy, or hasher experiments without a direct quality story tied to the live duel wall.
- Do not reopen broad search-budget, reply-budget, or generic search-knob clamps without evidence that the real wall lives on that surface.
- Do not keep branches just because disagreement counts shrink or hotspot counters move. If direct duel quality stays flat or regresses, kill the line.
- Do not retain a duel-traced acceptance clamp just because it fixes one repeated seed. If `pro-triage(primary_pro)` does not move, kill the production split and keep only the probe or lesson.
- Do not assume `human_win_pro_c` and the sampled fast black turn-four divergence can be fixed by one shared projection clamp. If the probe says the human drift is a followup-floor `SpiritImpact` story and the duel board is a vulnerable `ManaTempo` progress-head story, kill the shared idea before code edits.
- Do not retain a broader white `turn=3`, `mons_moves>0`, `action=false`, `mana=true` reroute just because it fixes several live duel boards. If `pro-triage(primary_pro)` does not move off the stale `human_win_pro_c`-only `1/52`, kill it before `runtime-preflight`.
- Do not retain a repeated white `turn=3`, `mons_moves=1`, `action+mana` accepted-head clamp just because it repeats in fast duels. If the retained churn probe still shows `accepted=false` on every retained `primary_pro` fixture, kill the idea before code edits.
- Do not merge an early black `negative_deny_competes` spirit-setup drift with `human_win_pro_c` just because both are spirit-vs-non-spirit selector stories. If the black board shows `followup_progress_competes=false`, treat it as a separate one-off seam and kill the shared idea before code edits.
- Do not treat a lone `vs current Pro` white mana-only regression as a new selector seam until the probe proves it is not just another `runtime_pro_turn_engine_v30_guarded_inputs(...)` fast-fallback reroute. If configured v30 already pre-accepts the current root, kill the idea before code edits.
- Do not treat late white full-resource current-Pro wrappers as a safe way to clear `human_win_pro_c`; if the fixture does not move immediately, kill the idea.
- Do not retain a late-white omitted-root rescue that only drives `pro-triage(primary_pro)` to `0/52`; matching current everywhere on the cheap surface is still a dead line.
- Do not retain a same-lane own-setup-vs-progress override just because it clears `human_win_pro_c`; if it reopens black reliability fixtures or tanks direct duels, kill it immediately.
- Do not retain white-only or black-only local seam repairs that fail to move the broader `vs current Normal` wall.
- Do not treat the relaxed `700ms` move-time cap as permission to reopen parity-preserving speed regressions.
- Do not retain new scratch profile IDs when the shared fix can live under `runtime_pro_turn_engine_v30` and the scratch line still fails direct duel evidence.

## Current Durable Direction

- Speed is already acceptable on the retained Pro challenger. Promotion is blocked by quality, especially versus current Normal, not by the `700ms` move-time budget.
- The remaining work should favor broader in-path selector and root-choice changes over wrapper fallbacks, fallback widening, or exact replay fixes.
- The most credible future wins are shared selector/search families that move direct `runtime_pro_turn_engine_v30` vs `runtime_current` results, not local acceptance or wrapper repairs that only clear one traced seam.
- `human_win_pro_c` may still be a retained triage drift, but it is not a hotspot-backed duel seam today. Do not reopen shared ProV2 selector work on that fixture alone without fresh direct duel or hotspot evidence.
- The current human drift and sampled fast black turn-four duel seam still look separate. Treat them as different hypotheses unless a fresh duel replay proves they have converged onto the same projection surface.
- The current white turn-three wall is split in two: the mana-only mid-turn wrapper family is closed again as too local, while the repeated `action+mana` accepted-head seam (`l9,4;l8,4` vs `l8,7;l7,8`) remains live.
- The repeated white `action+mana` seam still does not have a retained cheap-surface foothold today. Do not spend another production split on it until a fresh target-surface probe shows the same accepted-head family on `primary_pro`.
- The fresh black normal-duel `l0,5;l1,6` vs `l1,5;l3,6;l2,7` seam is not a retained foothold today. It is a one-off `negative_deny` spirit-preference drift, not the retained human followup-progress seam, so it does not justify a new production split by itself.
- The lone `vs current Pro` white `l9,2;l8,3` vs `l10,7;l9,7` miss is not a new foothold today either. It is another member of the already-closed white `turn=3`, `action=false`, `mana=true` wrapper family, so it does not justify reopening that production split.
- The latest retained shared win was a selector hygiene fix, not a new branch family: reject non-concrete one-chunk progress heads when they only override an unsafe non-progress selected root without a completed plan.
- Reducing `primary_pro` churn is not enough on its own. The retained line now reaches `2/52` changed fixtures on `primary_pro`, but `pro-reliability` is still flat at `0.8333` vs current Pro, `0.5000` vs current Normal, and `0.6667` vs current Fast. Future spend needs a direct duel-quality hypothesis.
- Closing the late `primary_pvs_sensitive_search` seam was still not enough. The retained line is now down to `1/52` changed `primary_pro` fixtures, but `pro-reliability` remained flat at `0.8333` vs current Pro, `0.5000` vs current Normal, and `0.6667` vs current Fast on Apr 8, 2026.
- Fixing the retained duel-accept seams and clearing `human_win_pro_c` is still not enough on its own. The latest retained attempt kept `primary_pro` at `1/52` changed fixtures and stayed preflight-clean, but `pro-reliability` still failed at `0.7500` vs current Pro, `0.6667` vs current Normal, and `0.5000` vs current Fast.
