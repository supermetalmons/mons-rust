# Automove Ideas

This is the working backlog for future automove loops.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution workflow and use this file to decide what to try next. When an idea is tried, promoted, ruled out, or split into follow-up ideas, update this file instead of relying on memory or raw logs.

If every current item here has been tried, add new ideas and keep going.

For `reply_risk`, `opponent_mana`, and `supermana`, run `./scripts/run-automove-experiment.sh triage-calibrate <surface>` before candidate work. If calibration fails, the next task is fixture work, not a new candidate.
Unless an idea explicitly says otherwise, new candidates must start as a delta on `runtime_current`. Retained non-production profiles are for calibration, references, or audit only.

## Template

### Idea: <short name>
- Base profile: `runtime_current`
- Target mode:
- Triage surface:
- Triage pass signal:
- Calibration gate:
- Candidate budget: 1
- Expected upside:
- CPU risk:
- Cheapest falsifier: `guardrails`, then triage; run `runtime-preflight` only if triage passes.
- Escalate only if:
- Kill if:
- Next split if rejected:
- How to test: `guardrails`, triage, `runtime-preflight`, then the earned duel path only if the candidate is still alive.
- Status: backlog
- Notes:

## In-Progress

### Idea: Pro quiescence search
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `opening_reply` (target), `primary_pro` (off-target guard)
- Triage pass signal: `pro-triage` reports `target_changed>=1` with `off_target_changed<=1`.
- Calibration gate: none
- Candidate budget: 1
- Expected upside: recover the proven quiescence strength signal in Pro without breaking release CPU gates.
- CPU risk: medium to high — current quiescence still pays full `ranked_child_states()` cost at depth 0.
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=opening_reply ./scripts/run-automove-experiment.sh pro-triage <candidate>`.
- Escalate only if: `pro-triage` passes and `runtime-preflight` clears CPU and exact-lite diagnostics.
- Kill if: `pro-triage` misses target change, or first `pro-fast-screen` lane is flat/regressed.
- Next split if rejected: tactical-only quiescence child generation.
- How to test: `guardrails`, `pro-triage`, `runtime-preflight`, `pro-fast-screen`, then `pro-progressive`/`pro-ladder` only with clear target-mode signal.
- Status: in-progress
- Notes: Existing retained candidates: `runtime_pro_quiescence_v1` (budget 200) and `runtime_pro_quiescence_v2` (budget 30).

## Curated Proposals (March 2026)

### Idea: Tactical-only child generation for quiescence
- Base profile: `runtime_current`
- Target mode: `pro` first, `normal` only if Pro path is safe
- Triage surface: `opening_reply`
- Triage pass signal: candidate still changes `opening_reply` fixtures while reducing runtime-preflight CPU cost versus current quiescence variants.
- Calibration gate: none
- Candidate budget: 1
- Expected upside: unblock quiescence by removing the biggest known bottleneck (`ranked_child_states()` over full child sets).
- CPU risk: medium
- Cheapest falsifier: `guardrails`, `pro-triage` (`opening_reply`), then `runtime-preflight`.
- Escalate only if: first `pro-fast-screen` lane is positive and non-noisy.
- Kill if: no measurable CPU win or `pro-triage` stays flat.
- Next split if rejected: capture-only quiescence expansion (skip non-capture tactical classes).
- How to test: one candidate delta on `runtime_current`, then the earned Pro pipeline.
- Status: backlog
- Notes: Keep this as the top follow-up if `runtime_pro_quiescence_v1/v2` stay blocked.

### Idea: Volatility-gated quiescence trigger
- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `opening_reply`
- Triage pass signal: quiescence only activates on volatile/tactical frontier nodes and still changes `opening_reply` fixtures.
- Calibration gate: none
- Candidate budget: 1
- Expected upside: preserve most tactical upside while reducing needless quiescence calls on quiet leaves.
- CPU risk: medium
- Cheapest falsifier: `guardrails`, `pro-triage` (`opening_reply`), `runtime-preflight`.
- Escalate only if: `pro-fast-screen` beats baseline without vs_fast regression.
- Kill if: CPU drops but signal disappears, or signal remains with no CPU relief.
- Next split if rejected: combine trigger gate with tactical-only child generation.
- How to test: single-profile delta, then standard Pro earned loop.
- Status: backlog
- Notes: Distinct from tactical-only generation; this changes when quiescence runs, not how children are generated.

### Idea: Reply-risk shortlist cache reuse
- Base profile: `runtime_current`
- Target mode: `normal`, `pro`
- Triage surface: `cache_reuse`
- Triage pass signal: `triage` shows deterministic cache win (`avg_ms` drop or hit-rate lift) with no guardrail regression.
- Calibration gate: none
- Candidate budget: 1
- Expected upside: reclaim budget from repeated reply-risk and exact-lite summaries, then convert that budget into duel strength.
- CPU risk: low to medium
- Cheapest falsifier: `guardrails`, then `SMART_TRIAGE_SURFACE=cache_reuse ./scripts/run-automove-experiment.sh triage <candidate>`.
- Escalate only if: `cache_reuse` triage passes and first duel stage is positive.
- Kill if: cache metrics improve but duel stays flat.
- Next split if rejected: isolate one cache-sharing point (reply-risk shortlist only or exact-lite summary only).
- How to test: cache-reuse triage first, then `runtime-preflight` and earned duel path.
- Status: backlog
- Notes: This keeps the existing cache-reuse direction but narrows it to a concrete first split.

### Idea: Candidate-aware opening-reply speed probe
- Base profile: workflow-only
- Target mode: `pro`
- Triage surface: blocked until probe exists
- Triage pass signal: new probe reports stable candidate-vs-baseline opening reply latency deltas on fixed seeds.
- Calibration gate: none
- Candidate budget: 1
- Expected upside: catch opening latency regressions early for `opening_reply` ideas without misusing the production-only release gate.
- CPU risk: low
- Cheapest falsifier: implement probe and verify it cannot reliably separate known retained profiles.
- Escalate only if: probe is stable enough to become a pre-duel diagnostic for Pro candidates.
- Kill if: probe is noisy or adds overhead without better reject decisions.
- Next split if rejected: keep release gate only and shrink opening fixture pack instead.
- How to test: add ignored harness test, run side-by-side against `runtime_current` and one known slower/faster retained profile.
- Status: backlog
- Notes: This is workflow infrastructure, but it directly improves promotion quality for Pro opening work.

## Backlog

### Idea: Stuck-state and bounded-progress safety fixtures
- Base profile: `runtime_current`
- Target mode: `fast`, `normal`, `pro`
- Triage surface: blocked until new safety fixture exists
- Candidate budget: 1
- Expected upside: stronger release confidence by catching empty-selector, repeated-position, and no-progress edge cases before promotion.
- CPU risk: low
- Cheapest falsifier: new or strengthened fixtures that fail the candidate immediately.
- Status: backlog
- Notes: Safety work is promotion work. A candidate that can stall or behave unpredictably is not ready.

### Idea: Promotion-focused artifact summaries
- Base profile: workflow-only
- Target mode: workflow
- Candidate budget: 1
- Expected upside: faster iteration because doc-worthy outcomes become obvious and disposable logs stay disposable.
- Status: backlog
- Notes: Improve signal extraction, not permanent logging volume.

## Tried — Killed (Wave 3, Mar 9–16, 2026)

Full details archived in `docs/automove-archive.md` under "Wave 3".

### Config knob exhaustion assessment
- Status: confirmed
- Summary: The config knob space is completely exhausted across all three modes. The only successful promotion (Normal no-extensions, +19.4%) was a structural evaluation change. All remaining SmartSearchConfig features have been individually tested and are either triage-invisible or duel-flat. Future improvement requires new code.

### Normal disable selective extensions
- Status: **PROMOTED** — shipped to production runtime (+19.4% Normal strength, 50W-22L)
- Summary: Breadth-over-depth principle proven. `enable_selective_extensions = false` for Normal depth≥3.

### Pro disable selective extensions
- Status: killed at pro-ladder (CPU ratio 1.28x vs 1.60x minimum, confirmation regression post Normal promotion)
- Summary: +13% across 1,488 games — strongest candidate ever. Failed CPU ratio gate. After Normal-no-ext shipped, Pro-no-ext no longer adds strength on top.

### Pro flat search (no-ext + no-quiet-reductions)
- Status: killed at progressive (vs_fast δ=-0.056, 72 games)
- Summary: Combined no-ext + no-quiet for Pro. Strong vs_normal (+12.4%) but vs_fast regression.

### Pro more extensions (deeper tactical chains)
- Status: killed at pro-fast-screen (vs_fast δ=-0.250)
- Summary: max_extensions_per_path=2, 2500bp budget. vs_normal strongest ever (+0.375) but vs_fast regression.

### Normal quiescence search
- Status: killed at ladder pool non-regression (0 beaten vs baseline 1)
- Summary: Concept proven (46W-26L, δ=0.139, conf=0.988) but `ranked_child_states()` too expensive. budget=30 → CPU 1.290x (limit 1.30). Triage 2/18.

### Normal/Pro history heuristic
- Status: killed at triage (Normal 0/18, Pro 0/12+0/3)
- Summary: TT best-child (+2400) and killer (+1200) bonuses dominate ordering. History cap (+800) cannot change ordering at either depth.

### Normal scoring weight changes (5+ candidates)
- Status: all killed at fast-screen or progressive (FadingSignal)
- Summary: spirit_race, scoring_upgrade, tactical_finish, node_boost, supermana_boost, attacker_proximity — all pass triage but are noise at duel scale. Pattern: ±30 tweaks to nonzero weights produce triage-visible but duel-flat results.

### Normal structural features (6+ candidates)
- Status: all killed at triage (0/14 to 0/18)
- Summary: safety_rerank, two_pass, iterative_deepening, class_coverage, aspiration_windows, forced_tactical_prepass, futility_pruning — all invisible to triage due to dominant top moves.

### Pro structural features (4+ candidates)
- Status: all killed at pro-triage (0/12)
- Summary: no_futility, killer_ordering, search_combo (killer+no-futility+PVS), tight_ext_budget, wider_roots — all invisible to triage.

### Fast mode candidates (5+ candidates)
- Status: all killed at triage or fast-screen (0/14 to 0/18, or duel-flat)
- Summary: reply_risk cleanup, root_alloc, boolean_drainer_scoring, no_quiet_reductions, two_pass, spirit_deploy. Only 2% of positions sensitive to any config perturbation at depth 0-2.

### Normal conversion/opening candidates (3 candidates)
- Status: all killed at triage or pro-triage
- Summary: opponent_mana_conversion, spirit_window, opening_attacker_reply — none moved triage surfaces.

### Normal exact tactics activation
- Status: killed at runtime-preflight (CPU 1.396–5.345x vs 1.30x limit)
- Summary: Static exact eval moved ALL fixtures but costs 5.3x. Root exact marginal (1.396x) and audit δ=-0.125. No bounded approximation reproduced the signal.

### Normal dormant feature activation sweep
- Status: all killed at triage or audit
- Summary: walk_threat_prefilter, deterministic_tiebreak, killer_move_ordering, event_ordering, node_boost, wider_reply, supermana/opponent_mana prepass exceptions — all invisible or noise.

### Infrastructure (completed)
- Pro triage fixture expansion (close-decision positions): 4 gap=0 fixtures, 9→12 total. Completed.
- Triage fixture expansion (known-mistake positions): 2 positions, 4→6 total. Completed.
- Audit-driven fixture creation from random positions: 4 depth-disagreement fixtures, 10→14 total. Completed.
- Pro human-win fixture expansion: 3 fixtures from human wins, 9→12 total. Completed.
- Fast-mode fixture expansion: 4 Fast-mode fixtures, 14→18 total. Completed.
- Normal fixture sensitivity probe: found 25/300 positions sensitive, 14 from no_extensions. Led to promotion.
