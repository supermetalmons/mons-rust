# How To Iterate On Automove

This is the operator playbook for fast, safe automove iteration and promotion.

Goal: produce a stronger, promotable runtime change. If a candidate is not promotable, compress the lesson and move on quickly.

## Quick Reference

1. Pick one target mode (`pro`, `normal`, or `fast`).
2. Pick exactly one idea from `AUTOMOVE_IDEAS.md` (`Candidate budget: 1`).
3. Confirm the triage surface and calibration requirement.
4. Run the earned path in strict order.
5. Kill flat/noisy candidates early.
6. Promote only after duel gates and release speed gates pass.

### Fast/Normal earned path

```sh
./scripts/run-automove-experiment.sh guardrails <candidate>
SMART_TRIAGE_SURFACE=<surface> ./scripts/run-automove-experiment.sh triage <candidate>
./scripts/run-automove-experiment.sh runtime-preflight <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh fast-screen <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh progressive <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh ladder <candidate>
```

### Pro earned path

```sh
./scripts/run-automove-experiment.sh guardrails <candidate>
SMART_TRIAGE_SURFACE=<opening_reply|primary_pro> ./scripts/run-automove-experiment.sh pro-triage <candidate>
./scripts/run-automove-experiment.sh runtime-preflight <candidate>
./scripts/run-automove-experiment.sh pro-fast-screen <candidate>
./scripts/run-automove-experiment.sh pro-progressive <candidate>
./scripts/run-automove-experiment.sh pro-ladder <candidate>
```

### Release speed gates (promotion-time only)

```sh
cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture
cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture
```

## Session Start Checklist

- Pick one target mode for this session.
- Pick one idea and one candidate profile.
- Confirm whether `triage-calibrate` is required (`reply_risk`, `opponent_mana`, `supermana`).
- Confirm triage surface exists and is deterministic.
- Set `SMART_PROMOTION_TARGET_MODE=<fast|normal>` for mode-targeted duel stages.
- Confirm baseline profile (default: `runtime_release_safe_pre_exact`; `normal_fast_gap` triage defaults to `runtime_current` unless baseline passed explicitly).

## Stop Conditions (Kill Fast)

- `guardrails` fail: kill candidate.
- `triage`/`pro-triage` fail: kill candidate (or run one audit spot-check if explicitly chosen).
- `runtime-preflight` fail: kill candidate.
- First duel stage (`fast-screen` / `pro-fast-screen`) is flat or negative: kill candidate.
- `progressive` fading/weak signal: kill candidate; do not escalate to ladder.
- No clear story after one focused diagnostic: kill candidate and split idea.

## Supported Script Names (Source of Truth)

### Stages

- `guardrails`
- `runtime-preflight`
- `preflight` (legacy wrapper)
- `triage-calibrate`
- `triage`
- `audit-screen`
- `pre-screen` (legacy)
- `fast-screen`
- `progressive`
- `ladder`
- `pro-triage`
- `pro-audit-screen`
- `pro-pre-screen` (legacy)
- `pro-fast-screen`
- `pro-progressive`
- `pro-ladder`

### Triage surfaces in script flow

- `opening_reply`
- `primary_pro`
- `reply_risk`
- `supermana`
- `opponent_mana`
- `normal_fast_gap`
- `normal_release_seed_gap`
- `spirit_setup`
- `drainer_safety`
- `cache_reuse`

## Cross-Mode Campaign Rule

After any production promotion:

1. Run a quick mode-comparison sanity sweep on `runtime_current`.
2. Confirm target-mode gain story still holds.
3. Re-pick the next target mode (default: `pro` first, then port safe wins).
4. Start again from one idea / one candidate.

## Session End Checklist

- Record candidate outcome in `AUTOMOVE_IDEAS.md` immediately.
- Move durable lessons to `docs/automove-knowledge.md`.
- Keep long wave history in `docs/automove-archive.md`.
- Clean local artifacts:

```sh
./scripts/clean-experiment-artifacts.sh --dry-run
./scripts/clean-experiment-artifacts.sh
```

## Promotion Checklist

- Candidate cleared: `guardrails -> triage/pro-triage -> runtime-preflight -> first duel -> progressive -> ladder`.
- Release speed gates passed.
- Production runtime updated in `src/models/mons_game_model.rs`.
- Docs updated (`AUTOMOVE_IDEAS.md` + archive/knowledge notes).
- Commit includes runtime + docs + any required surface/workflow updates.

## Files That Matter

- `src/models/mons_game_model.rs`
- `src/models/automove_experiments/profiles.rs`
- `src/models/automove_experiments/tests.rs`
- `src/models/automove_experiments/harness.rs`
- `scripts/run-automove-experiment.sh`
- `scripts/clean-experiment-artifacts.sh`
- `AUTOMOVE_IDEAS.md`
- `docs/automove-knowledge.md`
- `docs/automove-archive.md`
