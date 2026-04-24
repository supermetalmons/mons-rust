# How To Iterate On Automove

This is the canonical automove runbook.

Archived profiles, archived seams, and archived stages are not valid experiment targets. New work stays on the retained Pro surface below.

## Quick Reference

1. Default to Pro work.
2. Optimize for reliable strength across all game variants, not only Classic.
3. Treat `frontier_pro_v2_guarded` as both the shipped Pro path and the only retained frontier.
4. Treat `shipping_pro_search` as the retained search-only baseline.
5. Use `./scripts/run-automove-canonical-loop.sh` for the default loop.
6. Pick exactly one live hypothesis before editing runtime code.
7. Probe first when the mechanism is unclear.
8. When there is no live hypothesis, switch to the structural reset in `docs/automove-strategy.md` instead of running another seam loop.
9. Archive or kill the line before starting another.
10. Clean logs and stamps before ending the session.

## Variant Policy

- Quick iteration uses deterministic seeded variant samples. A failed sampled-variant gate is enough to kill a line.
- Final promotion confirmation uses all current `GameVariant`s by default.
- Retained `primary_pro` fixtures are Classic regression controls only. They are not broad variant evidence.
- Use `SMART_AUTOMOVE_VARIANTS=classic,swapped_mana_rows` for a targeted variant rerun.
- Use `SMART_AUTOMOVE_VARIANT_POLICY=classic`, `sampled`, or `all` only when you need to override the stage default.
- Variant randomness is seeded and reproducible from logs.

## Retained Surface

- Retained profiles: `shipping_pro_search`, `frontier_pro_v2_guarded`
- Canonical stages: `guardrails`, `variant-smoke`, `pro-triage`, `runtime-preflight`, `pro-reliability`, `pro-reliability-confirm`
- Canonical triage surface: retained Classic `primary_pro`

## Canonical Loop

```sh
CANDIDATE=<retained_profile_id>
./scripts/run-automove-canonical-loop.sh "$CANDIDATE"
```

- Default shipping profile: `shipping_pro_search`
- Default quick duel variant policy: seeded `sampled`
- Default triage surface inside the loop: retained Classic `primary_pro`
- Add `--confirm` only after `pro-reliability` earns the spend:

```sh
./scripts/run-automove-canonical-loop.sh --confirm "$CANDIDATE"
```

## Single-Stage Runs

Use `./scripts/run-automove-experiment.sh` when you need one stage at a time or a targeted rerun.

```sh
./scripts/run-automove-experiment.sh guardrails frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh variant-smoke frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh pro-triage frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh runtime-preflight frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh pro-reliability frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh pro-reliability-confirm frontier_pro_v2_guarded
./scripts/run-automove-experiment.sh pro-profile-sweep frontier_pro_v2_raw
```

## Structural Reset

Use this path when `AUTOMOVE_IDEAS.md` says there is no live challenger or when recent work keeps passing narrow sampled slices and failing broader promotion gates.

```sh
./scripts/run-automove-structural-scout.sh <sweep-candidate[,candidate...]>
```

- Read `docs/automove-strategy.md` first.
- The scout is diagnostic-only and runs both canonical sampled and active-blocker panels.
- Do not edit runtime code for a broad Pro change unless the candidate is strong on both panels.
- Add `--confirm` only after the default scout panels look promotable.

## Gate Rules

- `guardrails`: run first; kill the line on tactical or interview-policy regressions.
- `variant-smoke`: cheap all-variant legality check for public Fast, Normal, and Pro paths.
- `pro-triage`: cheap deterministic retained Classic surface gate; pass only when the target surface moves with `off_target_changed <= 1`, or when the shipped `frontier_pro_v2_guarded` surface is intentionally stable on the probed target.
- `runtime-preflight`: required before duel stages unless you are doing diagnostics only; exact-lite is hard, stage-1 CPU is advisory for Pro, and openings use sampled variants.
- `pro-reliability`: sampled-variant frontier-vs-`shipping_pro_search` duels in Pro, Normal, and Fast; pass only with `win_rate >= 0.90`, `confidence >= 0.99`, and frontier average move time `<= 700ms` in all three matchups.
- `pro-reliability-confirm`: all-variant confirmation. Run only after the sampled duel gate earns the spend; it also enforces a per-variant non-regression floor.
- `pro-profile-sweep` and `pro-profile-attribution`: diagnostic-only stages for test-only Pro candidates. They do not add retained profiles and are not promotion stages.

## Iteration Lifecycle

1. Read `AUTOMOVE_IDEAS.md` and select the single current hypothesis.
2. If there is no current hypothesis, run the structural reset path and create a test-only candidate or diagnostic before touching runtime code.
3. If the mechanism is not already proven, run one targeted diagnostic before editing runtime code.
4. Make the narrowest runtime or test-only change that can falsify the hypothesis.
5. Run the canonical stages in order; stop immediately on a failed hard gate.
6. If the line fails, discard runtime code and record the no-go in `docs/automove-archive.md` or `docs/automove-knowledge.md`.
7. If the line passes, promote retained Classic regression coverage before confirm.
8. End by compressing `AUTOMOVE_IDEAS.md` back to current state plus one next hypothesis.

## Diagnostic Toolbox

Use diagnostics only after the canonical loop shows what is still missing.

- `smart_automove_pro_reliability_duel_trace_probe`
- `smart_automove_pro_reliability_nonwin_trace_probe`
- `smart_automove_pro_reliability_hotspot_probe`
- `smart_automove_pro_profile_sweep_probe`
- `smart_automove_pro_profile_attribution_probe`
- `smart_automove_pro_triage_retained_churn_probe`
- `smart_automove_pro_forced_turn_engine_retained_churn_probe`
- `smart_automove_pro_root_advisor_trace_probe`
- `black_recovery_branch_reply_floor_attribution_probe`
- `black_progress_residual_weight_attribution_probe`

`smart_automove_pro_reliability_hotspot_probe` can take one extra ad-hoc board without a source edit:

```sh
SMART_PRO_RELIABILITY_HOTSPOT_LABEL=<label> \
SMART_PRO_RELIABILITY_HOTSPOT_MODE=pro \
SMART_PRO_RELIABILITY_HOTSPOT_FEN='<fen>' \
cargo test --release --lib smart_automove_pro_reliability_hotspot_probe -- --ignored --nocapture
```

`smart_automove_pro_reliability_duel_trace_probe` and `smart_automove_pro_reliability_nonwin_trace_probe` can focus on one duel bucket:

```sh
SMART_PRO_RELIABILITY_DUEL_FILTER=vs_shipping_fast \
cargo test --release --lib smart_automove_pro_reliability_duel_trace_probe -- --ignored --nocapture
```

`smart_automove_pro_profile_sweep_probe` compares test-only Pro candidates against the retained shipping baseline without adding them to the retained profile registry. It prints structured `PRO_PROFILE_SWEEP_RESULT`, `PRO_PROFILE_SWEEP_VARIANT`, and guarded-branch `PRO_PROFILE_SWEEP_BRANCH` lines.

```sh
SMART_PRO_SWEEP_CANDIDATES=frontier_pro_v2_guarded,frontier_pro_v2_raw \
SMART_PRO_SWEEP_DUEL_FILTER=vs_shipping_fast \
SMART_AUTOMOVE_VARIANTS=alternating_mana_rows,forward_bridge_mana_rows \
cargo test --release --lib smart_automove_pro_profile_sweep_probe -- --ignored --nocapture
```

The same sweep is available through the experiment runner:

```sh
SMART_PRO_SWEEP_DUEL_FILTER=vs_shipping_fast \
SMART_AUTOMOVE_VARIANTS=alternating_mana_rows,forward_bridge_mana_rows \
./scripts/run-automove-experiment.sh pro-profile-sweep frontier_pro_v2_raw
```

`smart_automove_pro_profile_attribution_probe` replays the same opening seeds with two sweep candidates against the same shipping opponent, then prints outcome-changing first divergences as `PRO_PROFILE_SWEEP_ATTRIBUTION`, `PRO_PROFILE_SWEEP_ATTRIBUTION_SUMMARY`, `PRO_PROFILE_SWEEP_ATTRIBUTION_BRANCH`, and `PRO_PROFILE_SWEEP_ATTRIBUTION_PAIR` lines. It defaults to `frontier_pro_v2_guarded` vs `frontier_pro_v2_raw`; override with `SMART_PRO_SWEEP_ATTRIBUTION_LEFT` and `SMART_PRO_SWEEP_ATTRIBUTION_RIGHT`.

```sh
SMART_PRO_SWEEP_ATTRIBUTION_LEFT=frontier_pro_v2_no_late_black_fallback \
SMART_PRO_SWEEP_ATTRIBUTION_RIGHT=frontier_pro_v2_raw \
SMART_PRO_SWEEP_DUEL_FILTER=all \
SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows \
cargo test --release --lib smart_automove_pro_profile_attribution_probe -- --ignored --nocapture
```

The attribution wrapper sets the left candidate from the stage argument and reads the right candidate from `SMART_PRO_SWEEP_ATTRIBUTION_RIGHT`:

```sh
SMART_PRO_SWEEP_ATTRIBUTION_RIGHT=frontier_pro_v2_raw \
SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows \
./scripts/run-automove-experiment.sh pro-profile-attribution frontier_pro_v2_no_late_black_fallback
```

All diagnostics run through the ignored test harness:

```sh
cargo test --release --lib <test_name> -- --ignored --nocapture
```

## Artifacts

- Selected-profile logs: `target/experiment-runs/<profile>/`
- Workflow-only logs: `target/experiment-runs/misc/`
- Runtime-preflight stamps: `target/experiment-stamps/`
- Run metadata records the stage variant policy and any explicit variant override.
- Rust gate logs print resolved variant policy, sample size, and per-variant duel summaries.
- Logs and stamps are disposable evidence, not durable memory.

Standard cleanup:

```sh
./scripts/clean-experiment-artifacts.sh --dry-run
./scripts/clean-experiment-artifacts.sh
```

Full local cache cleanup after validation:

```sh
./scripts/clean-experiment-artifacts.sh --dry-run --all-target
./scripts/clean-experiment-artifacts.sh --all-target
```

## Session End

1. Update `AUTOMOVE_IDEAS.md` with the current live state or next frontier.
2. Move durable lessons into `docs/automove-knowledge.md`.
3. Move retired wave detail into `docs/automove-archive.md`.
4. Clean disposable artifacts once validation is complete.
5. Leave exactly one clear next hypothesis, or explicitly record that there is no live challenger.
6. Do not leave unarchived probe diaries or failed runtime branches in the live board.
