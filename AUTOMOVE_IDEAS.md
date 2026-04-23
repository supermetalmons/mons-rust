# Automove Ideas

This is the live decision board for automove work. Keep it short.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the operator flow, `docs/automove-knowledge.md` for durable rules, and `docs/automove-archive.md` for retired wave detail.

## Current State

- Public Pro routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` remains the retained search-only baseline.
- The live experiment surface is Pro-only: 2 retained profiles and 5 canonical stages.
- The default operator entrypoint is `./scripts/run-automove-canonical-loop.sh`.
- There is no second live challenger today.
- Release readiness was refreshed on `2026-04-23`: public Pro wiring still ships `frontier_pro_v2_guarded`, experimentation remains test-only, and the full canonical confirm loop passed without selection-behavior changes.
- The remaining classified residuals are not current selection-layer targets:
  - `black_recovery_branch`: static exact evaluation is the only fresh signal; local shipping mirrors and legacy-fallback variants are killed.
  - black Fast progress-vs-setup residue: explained as material/cooldown residual valuation, not a shortlist/advisor/wrapper miss.

## Latest Gate Snapshot

- Date: `2026-04-23`
- Shipping decision: public Pro remains on `frontier_pro_v2_guarded`.
- Release verification passed: `cargo fmt --check`, host and wasm `cargo build --release --lib`, `git diff --check`, `cargo test --release --lib smart_automove_tactical_selected_profile -- --ignored --nocapture`, and `./scripts/run-automove-canonical-loop.sh --confirm frontier_pro_v2_guarded`.
- Confirm duel metrics: Pro `0.9688`, Normal `1.0000`, Fast `0.9688`, each with confidence `1.0000`; frontier average move times stayed below `200ms`.
- Release containment: public `Pro` dispatch still routes through `select_frontier_pro_v2_guarded_inputs`; `automove_experiments` remains under `#[cfg(test)]`, so diagnostics and experiment harness code are not production selection code.

## Next Hypothesis

- Default to no runtime change until a focused probe produces a new shared mechanism below the current no-go layer.
- For `black_recovery_branch`, the only plausible next spend is a scoped static-exact experiment that proves both runtime cost and retained reliability before any selector change.
- For black progress-vs-setup residue, reopen only with a narrow material/cooldown valuation hypothesis proven against retained setup-control boards.
- For white search-order residue, do not retry wrapper config mirroring, broad `ProV1` reroutes, root-rank gates, or simple shortlist widening. Reopen only with a mechanism that separates unresolved siblings from the retained vulnerable guard below the current shortlist/reply-risk surface.

## Active No-Go Notes

- Do not reopen archived profiles, archived seams, or archived stages.
- Do not treat retained `primary_pro` churn by itself as promotion evidence.
- Do not spend canonical gates on a challenger that stays behaviorally inert at `target_changed=0 off_target_changed=0`.
- Do not treat “all live walls aligned” as enough if duel strength or CPU cost still fails.
- Do not add board-local shipping mirrors for residuals that lose under frontier reply-risk, selector, or residual-score metrics.
- Do not reopen the direct black engine-disabled shipping fallback on `l7,1;l9,3` vs `l1,5;l2,7;l1,8`; it fixed the local board and still failed confirm.
- Do not reopen the black shortlist-local, guarded-legacy, or direct `black_recovery_branch` fallback variants; they align the board locally and still fail retained reliability.
- Do not broaden white turn-three no-action recovery from turn-start to `mons_moves_count == 1`; that line already failed retained reliability.
- Do not broaden white search-order fallback into raw `search-only + ProV1`, shipping own caps, root-rank checks, or wrapper config toggles; each has a documented retained-control failure or inert wrapper path.
