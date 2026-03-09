# Smart Automove Experiments

This file is a short compatibility entrypoint. The canonical workflow now lives in [`../HOW_TO_ITERATE_ON_AUTOMOVE.md`](../HOW_TO_ITERATE_ON_AUTOMOVE.md).

## Start Here

1. Read [`../HOW_TO_ITERATE_ON_AUTOMOVE.md`](../HOW_TO_ITERATE_ON_AUTOMOVE.md).
2. Pick or add the next item in [`../AUTOMOVE_IDEAS.md`](../AUTOMOVE_IDEAS.md).
3. For `reply_risk`, `opponent_mana`, and `supermana`, run `./scripts/run-automove-experiment.sh triage-calibrate <surface>` before candidate work. If calibration fails, do fixture work instead of candidate work.
4. Use `guardrails -> triage -> runtime-preflight` as the default front half of the earned loop so weak ideas die before the CPU gate.
5. `runtime-preflight` is the only CPU/exact-lite gate. Earned duel stages are duel-only and require the stamp it writes.
6. Default new candidates to `runtime_current`. Use retained non-production profiles only for calibration, references, or explicit audits.
7. Use `./scripts/run-automove-experiment.sh` for the full earned loop. Most candidates should die at `triage`, `pro-triage`, or the first earned duel stage.
8. Use `audit-screen` or `pro-audit-screen` only as occasional preflight-free spot checks for clean `changed=0/N` triage rejects. They are not promotion evidence. If audit contradicts triage, fix the surface before trying more candidates.
9. For mode-specific `fast` or `normal` ideas, run duel stages with `SMART_PROMOTION_TARGET_MODE=<fast|normal>`. The target mode must improve; the other client mode only has to stay non-regressed.
10. Treat `pre-screen` and `pro-pre-screen` as legacy diagnostics only.
11. Treat `smart_automove_release_opening_black_reply_speed_gate` as a promotion-time production check, not a candidate-aware pre-triage gate.
12. Promote durable conclusions into [`automove-knowledge.md`](automove-knowledge.md) or [`automove-archive.md`](automove-archive.md) before cleaning artifacts.

## Key References

- Active experiment registry: `src/models/automove_experiments/profiles.rs`
- Promotion gates and diagnostics: `src/models/automove_experiments/tests.rs`
- Harness and artifact handling: `src/models/automove_experiments/harness.rs`
- Production runtime promotion point: `src/models/mons_game_model.rs`
- Raw pro strategy notes: [`automove-pro-strategy-interview.md`](automove-pro-strategy-interview.md)

## Main Commands

```sh
# fast or normal candidate
./scripts/run-automove-experiment.sh triage-calibrate <reply_risk|supermana|opponent_mana>
./scripts/run-automove-experiment.sh guardrails <candidate>
SMART_TRIAGE_SURFACE=<reply_risk|supermana|opponent_mana|spirit_setup|drainer_safety|cache_reuse> \
  ./scripts/run-automove-experiment.sh triage <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh audit-screen <candidate>
./scripts/run-automove-experiment.sh runtime-preflight <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh fast-screen <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh progressive <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh ladder <candidate>

# pro candidate
./scripts/run-automove-experiment.sh guardrails <candidate>
SMART_TRIAGE_SURFACE=<opening_reply|primary_pro> \
  ./scripts/run-automove-experiment.sh pro-triage <candidate>
./scripts/run-automove-experiment.sh pro-audit-screen <candidate>
./scripts/run-automove-experiment.sh runtime-preflight <candidate>
./scripts/run-automove-experiment.sh pro-fast-screen <candidate>
./scripts/run-automove-experiment.sh pro-progressive <candidate>
./scripts/run-automove-experiment.sh pro-ladder <candidate>

# optional legacy diagnostics
./scripts/run-automove-experiment.sh pre-screen <candidate>
./scripts/run-automove-experiment.sh pro-pre-screen <candidate>

./scripts/clean-experiment-artifacts.sh
```
