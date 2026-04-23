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
- Treat rotated residue from a discarded challenger as provisional until a clean-tree retained trace confirms it.
- Runtime cost is a real gate. A candidate that wins local seams while drifting further into the `1.5x+` advisory band is not an upgrade.
- Wrapper-only reroutes, fallback widening, shortlist widening, and metadata-only advisor changes saturate quickly; durable progress usually needs shared approval, head, or scoring logic.

## Retained Package Lessons

- The promoted retained package is a set of narrow, mutually constrained repairs around `frontier_pro_v2_guarded`; keep them together unless a future gate proves otherwise.
- White turn-three and turn-five repairs are intentionally split by mechanism: no-action recovery, selected-rank search-only fallback, equal-surface ProV1 tiebreaks, and final head rejection are not interchangeable.
- Black late setup/progress repairs belong in advisor family competition only when setup roots are close-surface, already shortlisted, and backed by retained controls.
- A direct black progress-vs-setup wrapper mirror is unsafe: it fixed one local board and failed confirm by rotating onto later Fast seams.
- A broad white turn-three recovery or raw ProV1 search-only reroute is unsafe: it fixes known siblings but flips retained vulnerable guards or rotates Normal losses.
- A root-rank separator is not enough for white search-order fallbacks; selected rank was the safe separator for the retained negative-deny sibling.

## Residual Map

- `black_recovery_branch`: not a root-availability problem. The current live model prefers the approved plain-spirit root or a no-guard ProV1 spirit replay over shipping under frontier metrics. Static exact scoring is not a promotable direct spend: broad scoped exact selected the wrong spirit sibling, and reply-floor-only exact aligned the local board but failed retained sampled reliability.
- Black Fast progress-vs-setup residue: not a shortlist/advisor target. The safe-progress edge comes from residual board-state evaluation, especially material/cooldown terms. Reopen only with a narrow scoring hypothesis that preserves retained setup-control boards.
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
- Kill any line that stays inert at `target_changed=0 off_target_changed=0` against active `frontier_pro_v2_guarded`.
- Kill any line that only changes advisor labels or `pre_accept` metadata while the final selected root stays unchanged.
- Kill any line that widens shortlist or injection coverage without moving the approved root on the live walls.
- Kill any line that aligns live walls but still fails retained duel strength or canonical cost.
- Kill any `black_recovery_branch` line that only forces shipping `l6,0;l6,1` above the current reply-risk and no-guard selector ordering.
- Kill any `black_recovery_branch` line that reopens broad scoped static exact or reply-floor-only static exact without a new direct duel-strength mechanism.
- Do not reopen archived profiles, archived seams, or archived wave packages without a brand-new shared hypothesis.
