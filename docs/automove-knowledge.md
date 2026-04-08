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
- Simple speculative immediate-score non-regression clamps and setup-gain-only spirit-setup promotion are not enough. They can reshuffle `primary_pro` fixtures and regress direct Pro-vs-Pro without moving the `vs current Normal` wall.
- Broad same-lane `spirit_own_mana_setup_now` overrides are too blunt as well. On Apr 8, 2026 a shared ProV2 override collapsed `human_win_pro_c` and fixed one traced normal-duel board, but it reopened `primary_black_reliability_opening_3_ply4` and regressed `pro-reliability` to `0.7500` vs current Pro and `0.4167` vs current Normal/Fast.
- Soft followup-tolerance for `spirit_own_mana_setup_now` roots and close quiet-root normal-safety blocks are also not enough by themselves. If the runtime-faithful seam stays unchanged, kill the split before spending the canonical loop.
- Eval-only progress-head wins are too soft for unsafe late overrides. When both the selected root and the `Safe*Progress` head stay unsafe, do not let a lower-scored progress head replace the selected non-progress root unless it brings a non-eval strategic gain or a forced `score_delta` jump.
- Even a real duel-traced acceptance seam can still be too local. On Apr 8, 2026 a bounded fast-duel `Safe*Progress` override clamp matched the traced seed, but it left `pro-triage(primary_pro)` unchanged at `1/52`, so the production split was killed before `runtime-preflight` and `pro-reliability`.

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
- Do not reopen cache-size, memo-shape, reserve-heavy, or hasher experiments without a direct quality story tied to the live duel wall.
- Do not reopen broad search-budget, reply-budget, or generic search-knob clamps without evidence that the real wall lives on that surface.
- Do not keep branches just because disagreement counts shrink or hotspot counters move. If direct duel quality stays flat or regresses, kill the line.
- Do not retain a duel-traced acceptance clamp just because it fixes one repeated seed. If `pro-triage(primary_pro)` does not move, kill the production split and keep only the probe or lesson.
- Do not retain a same-lane own-setup-vs-progress override just because it clears `human_win_pro_c`; if it reopens black reliability fixtures or tanks direct duels, kill it immediately.
- Do not retain white-only or black-only local seam repairs that fail to move the broader `vs current Normal` wall.
- Do not treat the relaxed `700ms` move-time cap as permission to reopen parity-preserving speed regressions.
- Do not retain new scratch profile IDs when the shared fix can live under `runtime_pro_turn_engine_v30` and the scratch line still fails direct duel evidence.

## Current Durable Direction

- Speed is already acceptable on the retained Pro challenger. Promotion is blocked by quality, especially versus current Normal, not by the `700ms` move-time budget.
- The remaining work should favor broader in-path selector and root-choice changes over wrapper fallbacks, fallback widening, or exact replay fixes.
- The most credible future wins are shared selector/search families that move direct `runtime_pro_turn_engine_v30` vs `runtime_current` results, not local acceptance or wrapper repairs that only clear one traced seam.
- `human_win_pro_c` may still be a retained triage drift, but it is not a hotspot-backed duel seam today. Do not reopen shared ProV2 selector work on that fixture alone without fresh direct duel or hotspot evidence.
- The latest retained shared win was a selector hygiene fix, not a new branch family: reject non-concrete one-chunk progress heads when they only override an unsafe non-progress selected root without a completed plan.
- Reducing `primary_pro` churn is not enough on its own. The retained line now reaches `2/52` changed fixtures on `primary_pro`, but `pro-reliability` is still flat at `0.8333` vs current Pro, `0.5000` vs current Normal, and `0.6667` vs current Fast. Future spend needs a direct duel-quality hypothesis.
- Closing the late `primary_pvs_sensitive_search` seam was still not enough. The retained line is now down to `1/52` changed `primary_pro` fixtures, but `pro-reliability` remained flat at `0.8333` vs current Pro, `0.5000` vs current Normal, and `0.6667` vs current Fast on Apr 8, 2026.
