# How To Iterate On Automove

This is the canonical long-term workflow for automove iteration.

The goal is not to collect experiments. The goal is to make automove stronger and get the stronger version promoted into the production runtime safely. A successful loop ends with either a promotable candidate or a compressed lesson that makes the next promotion attempt better.

## Quick Reference

1. Pick one target mode to improve first: usually `pro`, `normal`, or `fast`.
2. Keep one live candidate in the experiment registry.
3. Fill in the idea entry in `AUTOMOVE_IDEAS.md` before running anything: name the base profile, the triage surface, the triage pass signal, the cheapest falsifier, the escalation bar, the kill bar, and set `Candidate budget: 1`.
4. Calibrate the declared surface before candidate tuning. If `./scripts/run-automove-experiment.sh triage-calibrate <reply_risk|opponent_mana|supermana>` fails, stop and do fixture work instead of candidate work.
5. Do not tune a candidate until its deterministic triage fixture exists. If the surface is missing, add the fixture or helper first.
6. Default every new candidate to a delta on `runtime_current`. Only retained calibration/reference profiles should start elsewhere, and that exception should be written down explicitly.
7. Run the earned loop for the target mode.

For `fast` or `normal` candidates:

```sh
./scripts/run-automove-experiment.sh triage-calibrate <reply_risk|supermana|opponent_mana>
./scripts/run-automove-experiment.sh guardrails <candidate>
SMART_TRIAGE_SURFACE=<reply_risk|supermana|opponent_mana|spirit_setup|drainer_safety|cache_reuse> \
  ./scripts/run-automove-experiment.sh triage <candidate>
# optional audit lane for about 1 in 5 clean triage rejects
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh audit-screen <candidate>
./scripts/run-automove-experiment.sh runtime-preflight <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh fast-screen <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh progressive <candidate>
SMART_PROMOTION_TARGET_MODE=<fast|normal> ./scripts/run-automove-experiment.sh ladder <candidate>
```

For `pro` candidates:

```sh
./scripts/run-automove-experiment.sh guardrails <candidate>
SMART_TRIAGE_SURFACE=<opening_reply|primary_pro> \
  ./scripts/run-automove-experiment.sh pro-triage <candidate>
# optional audit lane for about 1 in 5 clean pro-triage rejects
./scripts/run-automove-experiment.sh pro-audit-screen <candidate>
./scripts/run-automove-experiment.sh runtime-preflight <candidate>
./scripts/run-automove-experiment.sh pro-fast-screen <candidate>
./scripts/run-automove-experiment.sh pro-progressive <candidate>
./scripts/run-automove-experiment.sh pro-ladder <candidate>
```

8. `preflight` still exists as the old all-in-one wrapper, but the default loop should use `guardrails` before triage and `runtime-preflight` only after triage passes.
9. `runtime-preflight` is the only stage that should run the stage-1 CPU gate and exact-lite diagnostics. Earned duel stages are duel-only and should reuse the stamp written by `runtime-preflight`.
10. Treat `pre-screen` and `pro-pre-screen` as optional legacy diagnostics only. They are not part of the default promotion path.
11. Use `audit-screen` or `pro-audit-screen` only as a false-negative spot check for roughly 1 in 5 clean `changed=0/N` triage rejects. They are intentionally preflight-free and are not promotion evidence. If an audit clearly contradicts triage, spend the next slot on the surface fix, not on more candidate breadth.
12. For mode-specific `fast` or `normal` promotion attempts, run duel stages with `SMART_PROMOTION_TARGET_MODE=<fast|normal>`. The target mode must improve; the other client mode only has to stay non-regressed.
13. Before any runtime promotion decision, run the release speed gates:

```sh
cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture
cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture
```

14. If the candidate is not promotable, archive the lesson and return to the next promotion attempt.
15. If the candidate is promotable, update the production runtime and commit the promotion immediately.

The generic release opening speed gate is a production baseline check, not a candidate-aware experiment filter. Do not run it before `pro-triage` for `opening_reply` ideas.

All experiment harness runs use ignored `#[cfg(test)]` tests:

```sh
cargo test --release --lib <test_name> -- --ignored --nocapture
```

## Files That Matter

- Active experiment registry: `src/models/automove_experiments/profiles.rs`
- Promotion gates and comparison tests: `src/models/automove_experiments/tests.rs`
- Experiment harness and artifacts: `src/models/automove_experiments/harness.rs`
- Production runtime that promotion must change: `src/models/mons_game_model.rs`
- Wrapper script for the main loop: `scripts/run-automove-experiment.sh`
- Artifact cleanup script: `scripts/clean-experiment-artifacts.sh`
- Working backlog for future loops: `AUTOMOVE_IDEAS.md`
- Durable lessons: `docs/automove-knowledge.md`
- Archived waves and retired context: `docs/automove-archive.md`
- Raw pro strategy notes: `docs/automove-pro-strategy-interview.md`

Promotion means changing the production runtime in `src/models/mons_game_model.rs`, not just leaving a stronger candidate in the experiment registry.

## Core Rules

- Optimize for promotion, not for experiment volume.
- Improve one mode at a time and verify the other modes do not regress.
- Prefer a focused sequence such as `pro` then `normal` then `fast`, or the reverse, instead of trying to improve all modes at once.
- Keep `main` as the last clean promotion checkpoint and do active exploration on a `codex/*` branch.
- Keep the active registry small. One live candidate at a time is easier to reason about than many overlapping waves.
- Keep one candidate budget per idea. No same-idea `v2` or `v3` retries after a reject or noisy first signal.
- Default new candidates to `runtime_current`. Treat retained non-production profiles as calibration references unless the idea explicitly says otherwise.
- Do not use archived profiles for new promotion decisions unless there is a new hypothesis that cannot be tested on the active surface.
- Most candidates should die at `triage`, `pro-triage`, or `fast-screen`.
- It is fine to explore and diagnose, but every loop should return to the next concrete promotion attempt.

## Safety And CPU Budget

The strongest candidate is only valuable if it is safe to ship.

- Stay within the release-safe CPU budget for the target mode.
- Do not promote anything that risks endless exploration, stalled turns, or getting stuck in unusual game states.
- Keep exact-lite and other expensive logic explicitly bounded.
- Treat opening black-reply latency as a hard release concern, not a cosmetic one.
- If strength depends on unsafe runtime behavior, it is not promotable yet.
- When in doubt, keep runtime behavior predictable and bounded first, then recover strength with cheaper signals or more selective exactness.

## Weak Candidate Protocol

- Stop immediately on any `guardrails`, `runtime-preflight`, or `preflight` failure.
- `triage` and `pro-triage` are mandatory. If there is no deterministic surface for the idea yet, the next task is fixture work, not duel work.
- For `reply_risk`, `opponent_mana`, and `supermana`, `triage-calibrate` is also mandatory. If the retained calibration profiles do not move the surface, stop and fix the triage pack before creating or tuning a candidate.
- Do not pay the stage-1 CPU gate before a candidate proves it can move the declared surface. Run `guardrails` first, then `triage`, then `runtime-preflight`.
- `runtime-preflight` now writes the duel stamp. `fast-screen`, `progressive`, `ladder`, `pro-fast-screen`, `pro-progressive`, and `pro-ladder` should refuse to run without a fresh stamp.
- `triage` is fixed-cost and rejection-first. A pass means only that the candidate changed the declared surface without regressing the generic guardrails.
- `pro-triage` must change the declared target surface and keep the off-target surface mostly stable. If the candidate moves both `opening_reply` and `primary_pro`, split the idea instead of continuing it.
- On roughly every fifth clean `changed=0/N` triage reject, run one audit duel with `audit-screen` or `pro-audit-screen`. If the audit duel is also flat or noisy, keep the reject. If the audit duel shows clear positive signal, treat that as evidence that the fixture pack is too strict for the surface and use the next slot on the surface fix instead of another candidate.
- `audit-screen` and `pro-audit-screen` are preflight-free sensitivity checks only. Even if they pass, the candidate still has to clear the earned duel stages under the current stamp.
- If an audit contradiction has already been fixed at the surface layer, replay the candidate from the earned duel stage. Do not rerun `audit-screen` on the same candidate before `fast-screen` or `progressive`.
- For `reply_risk`, triage must compare candidate-resolved shortlist evidence, not just selected-root equality. Shortlist width, clean-root preference, and reply-floor movement can matter even when the final move stays the same.
- Pooled client aggregate is too coarse for mode-specific first-duel decisions. For `SMART_PROMOTION_TARGET_MODE=fast|normal`, treat the target mode as the only improvement bar and treat the other client mode as a non-regression floor.
- For `opening_reply` ideas, do not pay the generic release opening speed gate before `pro-triage`. It measures production runtime only, so it belongs at promotion time unless a candidate-aware opening speed probe exists.
- If a target-aware `fast-screen` still takes too long to return a first earned decision, stop and shrink or reuse the duel stage before opening another strength candidate.
- Treat `pre-screen` and `pro-pre-screen` as legacy noise diagnostics only. Do not use them to justify escalation.
- Kill a candidate after the first real duel stage if the result is negative, flat, or noisy enough that the story is unclear.
- Allow at most one focused diagnostic after a borderline duel result. Good defaults are `smart_automove_pool_mode_comparison_report` or `smart_automove_pool_pool_regression_diagnostic`.
- If that one diagnostic does not reveal a clear next split, archive the lesson and move on.
- Never run `ladder` or `pro-ladder` for a merely non-negative candidate. Promotion stages are for strong, explainable signal only.

## Recommended Iteration Loop

1. Choose one target mode, one hypothesis, and one deterministic triage surface.
2. Pull or add the next idea in `AUTOMOVE_IDEAS.md`.
3. If the surface is `reply_risk`, `opponent_mana`, or `supermana`, run `triage-calibrate` first. If it fails, stop and improve the fixture pack instead of building a candidate.
4. If the surface fixture does not exist yet, add that fixture or helper first and stop there.
5. Implement one live candidate in `src/models/automove_experiments/profiles.rs`.
6. Run `guardrails` once.
7. Run `triage` for `fast` or `normal`, or `pro-triage` for `pro`.
8. If triage fails, either archive or split the idea immediately, or use one audit-lane duel if this candidate is the selected spot check for the current batch. If the audit contradicts triage, stop candidate breadth and patch the surface before continuing. Do not make same-idea follow-up variants.
9. Only if triage passes, run `runtime-preflight`.
10. Only if `runtime-preflight` passes, run `fast-screen` or `pro-fast-screen`.
11. If the first duel is borderline, allow one focused diagnostic, then either kill the candidate or split the idea into a narrower follow-up.
12. Only if the first duel already shows a strong, mode-specific story, run `progressive` or `pro-progressive`.
13. Only if the progressive result remains clearly stronger and safe, run `ladder` or `pro-ladder`.
14. If the candidate still looks promotable, run the release speed gates.
15. Promote if it is stronger and safe. Otherwise compress the lesson into docs, clean stale artifacts, and move to the next attempt.

A good loop often spreads findings across modes. For example: learn something in `pro`, then port the safe subset to `normal`, then decide whether a cheaper version is worth trying in `fast`. That is still one candidate per idea, not one rolling family of retries.

## Recording, Cleanup, And Commits

- Commit meaningful checkpoints once in a while during exploration.
- Do not let useful conclusions remain only in `target/experiment-runs` logs.
- Promote durable conclusions into `docs/automove-knowledge.md` or `docs/automove-archive.md` before cleaning artifacts.
- Update the matching idea entry in `AUTOMOVE_IDEAS.md` as soon as the kill, split, or promotion decision becomes clear.
- Clean obsolete local artifacts once in a while with:

```sh
./scripts/clean-experiment-artifacts.sh
./scripts/clean-experiment-artifacts.sh --dry-run
```

- Update `AUTOMOVE_IDEAS.md` as ideas are added, tried, promoted, or retired.
- When promotion is achieved, commit the runtime change, the supporting docs updates, and the cleaned experiment surface together.

## When The Backlog Runs Dry

If every current item in `AUTOMOVE_IDEAS.md` has already been tried, generate new ideas, add them to the backlog, and continue iterating. The workflow does not end when the current list is exhausted.
