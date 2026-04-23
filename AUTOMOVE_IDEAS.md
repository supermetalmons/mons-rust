# Automove Ideas

This is the live decision board for automove work. Keep it short.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` for the operator flow, `docs/automove-knowledge.md` for durable rules, and `docs/automove-archive.md` for retired wave detail.

## Current State

- Public Pro routes through `frontier_pro_v2_guarded`.
- `shipping_pro_search` remains the retained search-only baseline.
- The live experiment surface is Pro-only and multi-variant: 2 retained profiles and 6 canonical stages.
- The default operator entrypoint is `./scripts/run-automove-canonical-loop.sh`.
- Quick automove iteration uses seeded sampled game variants; final promotion confirmation uses all current variants.
- There is no second live challenger today.
- Release readiness was refreshed on `2026-04-23`: public Pro wiring still ships `frontier_pro_v2_guarded`, experimentation remains test-only, and the full canonical confirm loop passed without selection-behavior changes.
- The current local package around `frontier_pro_v2_guarded` is still not promotable on the sampled Pro gate, even after fixing several retained late Normal/Fast boards.
- Remaining sampled blockers at the end of this wave are variant-level, not just the earlier retained boards:
  - Normal `outer_edge_mana_rows`
  - Fast `alternating_mana_rows`
  - Fast `forward_bridge_mana_rows`

## Latest Gate Snapshot

- Date: `2026-04-23`
- Shipping decision: public Pro remains on `frontier_pro_v2_guarded`.
- Release verification passed under the previous Classic-era gate shape: `cargo fmt --check`, host and wasm `cargo build --release --lib`, `git diff --check`, `cargo test --release --lib smart_automove_tactical_selected_profile -- --ignored --nocapture`, and `./scripts/run-automove-canonical-loop.sh --confirm frontier_pro_v2_guarded`.
- Previous confirm duel metrics: Pro `0.9688`, Normal `1.0000`, Fast `0.9688`, each with confidence `1.0000`; frontier average move times stayed below `200ms`.
- Release containment: public `Pro` dispatch still routes through `select_frontier_pro_v2_guarded_inputs`; `automove_experiments` remains under `#[cfg(test)]`, so diagnostics and experiment harness code are not production selection code.
- Latest sampled `pro-reliability` gate for the kept local package still failed promotion: Pro `1.0000`, Normal `0.9167`, Fast `0.8333`; confidence `0.9998 / 0.9968 / 0.9807`; frontier average move times `151.76ms / 190.31ms / 170.39ms`.
- This iteration killed the `black_recovery_branch` static-exact spend. A reply-floor-only exact cut passed `guardrails`, `variant-smoke`, `pro-triage`, and `runtime-preflight`, then failed sampled `pro-reliability` at Pro `0.5000`, Normal `0.5000`, Fast `0.9167`; average move times stayed under `200ms`. The runtime code was discarded.
- This iteration also killed black progress material/cooldown-only scoring. Zeroing `fainted_mon` and `fainted_cooldown_step` only on the scoped black turn-six window/deny state shrank the target residual delta from `843/778` to `83/18` and preserved the turn-ten setup-control board, but final selection still stayed on `l7,1;l9,3`.
- Follow-up selector-layer probe for black progress retained only diagnostic code. Under the material-dampened replay, frontier still selected `l7,1;l9,3` through `frontier_execute` / `engine_post_search`, with advisor approval `l7,1;l9,3:SafeSupermanaProgress:ApprovedReplyRiskGuard:rank0`; shipping setup `l1,5;l2,7;l1,8` was already present in the reply-risk shortlist at rank `10`.
- This iteration killed the combined black progress material-plus-rank line. The runtime cut aligned the local residue to shipping setup `l1,5;l2,7;l1,8`, passed `guardrails`, `variant-smoke`, `pro-triage`, and `runtime-preflight`, then failed sampled `pro-reliability`: Pro `0.5000`, Normal `0.5000`, Fast `0.8333`; average move times stayed below `200ms`. The runtime code was discarded.
- The kept runtime edits from this wave repaired several stable retained late boards, including black search/head-accept Normal seams and two Fast seams, but the sampled gate still rotated to deeper late-ply losses.
- One traced Fast nonwin from this wave was not safe to retain. The copied board snapshot replayed to frontier's own move instead of the live shipping-selected root, so only the reproducibility lesson is kept.

## Next Hypothesis

- There is no promotable challenger after the latest sampled no-go. Default to no broad runtime change until a focused probe produces a new shared mechanism below the current late-ply search/head layer.
- For `black_recovery_branch`, do not retry broad static exact or reply-floor-only static exact. Reopen only with a new mechanism that improves direct multi-variant duel strength, not just the local board.
- For black progress-vs-setup residue, do not retry material/cooldown-only scoring, final-selector-only patches, or the scoped material-plus-higher-rank advisor exception. Reopen only with a mechanism that improves sampled multi-variant duel strength, not just the local residue.
- For white search-order residue, do not retry wrapper config mirroring, broad `ProV1` reroutes, root-rank gates, or simple shortlist widening. Reopen only with a mechanism that separates unresolved siblings from the retained vulnerable guard below the current shortlist/reply-risk surface.
- For the current late-ply sampled residue, do not promote copied `first_diff_ply` boards into retained until the clean retained replay reproduces the same final shipping-selected root.

## Active No-Go Notes

- Do not reopen archived profiles, archived seams, or archived stages.
- Do not treat retained `primary_pro` churn by itself as promotion evidence.
- Do not treat Classic retained fixtures as broad variant evidence.
- Do not spend canonical gates on a challenger that stays behaviorally inert at `target_changed=0 off_target_changed=0`.
- Do not treat “all live walls aligned” as enough if duel strength or CPU cost still fails.
- Do not add board-local shipping mirrors for residuals that lose under frontier reply-risk, selector, or residual-score metrics.
- Do not reopen the direct black engine-disabled shipping fallback on `l7,1;l9,3` vs `l1,5;l2,7;l1,8`; it fixed the local board and still failed confirm.
- Do not reopen the black shortlist-local, guarded-legacy, or direct `black_recovery_branch` fallback variants; they align the board locally and still fail retained reliability.
- Do not reopen `black_recovery_branch` with broad scoped static exact or reply-floor-only static exact; the former selected `l1,5;l2,6`, and the latter still failed sampled retained reliability.
- Do not reopen black progress-vs-setup with material/cooldown-only scoring; it reduces the local residual gap but leaves the selected root unchanged.
- Do not reopen black progress-vs-setup with final-selector-only changes; the material-dampened replay still approves safe progress at the advisor `ApprovedReplyRiskGuard` layer.
- Do not reopen black progress-vs-setup with the combined scoped material dampening plus higher-scoring setup-rank exception; it fixes the board locally and still fails sampled retained reliability.
- Do not broaden white turn-three no-action recovery from turn-start to `mons_moves_count == 1`; that line already failed retained reliability.
- Do not broaden white search-order fallback into raw `search-only + ProV1`, shipping own caps, root-rank checks, or wrapper config toggles; each has a documented retained-control failure or inert wrapper path.
