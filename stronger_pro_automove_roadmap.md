
# Stronger, promotable Pro automove roadmap for `mons-rust`

Prepared on 2026-03-25 from:
- the 7 proposal documents in this session
- the current repo playbook/backlog/docs
- the current code paths in `profiles.rs`, `mons_game_model.rs`, `scoring.rs`, `automove_exact.rs`, `automove_turn_engine.rs`, `automove_experiments/*`, and `scripts/run-automove-experiment.sh`

## How to read this document

This guide does two things at once:

1. It chooses **one main implementation direction** that I think has the highest expected value right now.
2. It preserves **every distinct useful idea** from the 7 proposal docs in an explicit alternatives / idea-bank section so nothing valuable gets lost if the main path stalls.

Throughout this document I refer to the 7 uploaded proposal docs as:

- **P1** = hybrid Pro recovery plan
- **P2** = search-side efficiency completion plan
- **P3** = search amortization + fallback plan
- **P4** = fused child-evaluation / batched reach plan
- **P5** = broader codebase audit + candidate approaches
- **P6** = PR-sequenced selector/search wall plan
- **P7** = local-reuse / lazy-escalation / oracle-profile plan

---

## 1. Executive decision

### Main recommendation

Keep **`runtime_pro_turn_engine_v30` as the sole live Pro frontier** and treat the next campaign as a **promotion-completion program centered on shared selector/search cost reduction**, not as another planner wave, wrapper-family wave, or config sweep.

The mainline should be:

1. add a bounded selector/search hotspot probe
2. remove duplicated child-evaluation work inside `ranked_child_states`
3. only then introduce lazy exact escalation / two-stage child ordering if needed
4. then clean up the remaining attack-reach / threat hot helpers
5. only after CPU headroom exists, do a **targeted** acceptance pass for the disagreement family that matters most
6. run the canonical Pro earned path exactly as written

### Why this is the main direction

This choice matches both the repo’s current state and the strongest overlap across the 7 proposals:

- the repo playbook explicitly wants **one live idea, one live candidate, one earned path**
- `AUTOMOVE_IDEAS.md` says **`runtime_pro_turn_engine_v30` is the sole active ProV2 frontier**
- the durable knowledge file says the problem is now **finishing the earned path cleanly under strict gates**, not finding another Pro concept
- the current backlog says the hotspot has migrated into **`search_score -> ranked_child_states -> evaluate_preferability_with_weights_and_exact_policy`**, with visible cost in `move_efficiency_snapshot`, `ManaPathSnapshot::from_board`, `can_attack_target_on_board`, `actor_payload_after_move`, and transition/hash-map churn
- direct code inspection confirms that `ranked_child_states` currently pays a large, overlapping per-child tax:
  - full `score_state(...)`
  - `move_efficiency_delta_from_before_snapshot(...)`
  - uncached after-state `build_move_efficiency_snapshot(...)`
  - drainer-vulnerability transition work
  - move-class classification work

That is the clearest, lowest-regret place to attack first.

### Why this is **not** the time to make the hybrid overlay the main path

P1’s hybrid `runtime_current` + ProV2 activation design is smart and worth preserving, but I would **not** lead with it yet.

Reason: the current repo state already says the retained frontier is `v30`, the latest blocker is shared selector/search cost, and the playbook strongly prefers finishing the retained frontier rather than opening a new structural branch until the current one is genuinely exhausted.

So my recommendation is:

- **mainline now** = finish `v30` through shared search-side work
- **first structural fallback** = hybrid overlay if `v30` still cannot become globally promotable after 2–3 focused shared-code splits
- **second structural fallback** = distill `v30` into a cheaper runtime_current-like signal/runtime

---

## 2. Working diagnosis

## 2.1 What is shipping vs what is experimental

### Shipping today

`runtime_current` is still the shipping runtime, and production Pro still uses the promoted turn-opportunity planner line.

### Retained experimental frontier

`runtime_pro_turn_engine_v30` is the retained ProV2 turn-engine frontier. It already contains durable shared work that the repo clearly wants to keep:

- opportunity-context extraction
- best-plan / no-plan / continuation caching
- config-fingerprinted cache reuse
- selector utility / followup-floor caching
- low-budget / eligibility / resume logic
- Pro-aware workflow support (`runtime-preflight`, `pro-reliability`, duel progress)

### Important implication

The frontier is **not** “turn engine instead of normal selector/search.”
It is “turn engine plus the normal selector/search stack.”

That means even a genuinely stronger engine head can fail to become promotable if the selector/search wall is still too expensive or too noisy.

## 2.2 What the current code says

The current code supports the same diagnosis the proposal docs reached:

### `profiles.rs`
`configure_runtime_pro_turn_engine_v30(...)`:
- disables `enable_turn_opportunity_planner`
- enables `TurnEngineMode::ProV2`
- uses bounded seed/beam/reply/expansion caps
- enables low-budget guard
- enables mid-turn tactical guard
- enables late safe-mana root preference

`model_runtime_pro_turn_engine_v30(...)` still contains important opening / early-turn fallbacks back to release-safe-pre-exact Fast/Normal shapes.

### `smart_search_best_inputs_internal(...)`
The selector already has:
- cached-step reuse
- low-budget engine disable
- mid-turn progress/tactical disable
- eligibility guard
- low-budget search clamp
- head-plan skip in some root contexts
- optional planner/root injection/forced prepass flow

This is important: the system already expects **selective activation and selective analysis**. The next gain should exploit that, not fight it.

### `ranked_child_states(...)`
The current implementation:
- enumerates legal transitions
- optionally adds exact-progress fallback children
- for **every child**:
  - computes `child_hash`
  - calls full `score_state(...)`
  - computes `move_efficiency_delta_from_before_snapshot(...)`
  - computes `own_drainer_vulnerable_after`
  - computes move classes
- then sorts
- then enforces tactical top-2 coverage
- then truncates to `node_branch_limit`

This is the single most important structural hotspot in the current code.

### `build_move_efficiency_snapshot(...)`
The current snapshot builder:
- may build `exact_strategic_analysis(game)`
- may request a tactical projection containing:
  - safe supermana progress
  - safe opponent-mana progress
  - spirit score
  - spirit denial
  - score window
- also scans the board to compute carrier counts / spirit-on-base / related fields

This is a rich summary, but it is expensive enough that it should not be rebuilt unnecessarily inside one sibling set.

### `evaluate_preferability_with_weights_and_exact_policy(...)`
The current evaluation path:
- builds `ManaPathSnapshot::from_board`
- may build `exact_strategic_analysis(game)`
- calls drainer-safety and immediate-threat logic
- uses exact/non-legacy summaries when enabled

This overlaps significantly with the move-efficiency snapshot work.

### `can_attack_target_on_board(...)`
The current exact reach helper:
- uses a cache keyed by:
  - `board_hash`
  - `attacker_color`
  - `target_color`
  - `target`
  - `remaining_moves`
  - `can_use_action`
- computes `exact_board_hash(board)` inside the helper
- answers one target query at a time

That makes the existing board-scoped attack-map idea a credible conditional follow-up, but it does **not** mean it has to be the first move.

### `accept_turn_engine_head_after_search(...)`
The current acceptance path is not cheap:
- it finds candidate and selected indices
- computes multiple override conditions
- may call `turn_engine_selected_override_utility(...)`
- may project compiled chunks into a simulated state
- may compute projected end-of-turn safety and follow-up implications
- may recurse into selected followup projection

This should become a later-phase optimization / strength pass, not the first move.

---

## 3. Campaign rules

These are the rules I would use for the whole campaign.

### One live frontier only

Keep exactly one live Pro frontier and one active candidate at a time.

### Do not reopen exhausted families first

Do **not** lead with:
- wrapper-family reopen
- old intent-planner reopen
- generic config/beam/seed sweeps
- Fast/Normal side work
- opening-reply side work
- broad new global caches in the hot path

### Promotion discipline stays unchanged

The canonical Pro path remains:

```sh
./scripts/run-automove-experiment.sh guardrails <candidate> runtime_current
SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage <candidate> runtime_current
./scripts/run-automove-experiment.sh runtime-preflight <candidate> runtime_current
./scripts/run-automove-experiment.sh pro-reliability <candidate> runtime_current
./scripts/run-automove-experiment.sh pro-fast-screen <candidate> runtime_current
./scripts/run-automove-experiment.sh pro-progressive <candidate> runtime_current
./scripts/run-automove-experiment.sh pro-ladder <candidate> runtime_current
```

Do **not** spend `pro-fast-screen`, `pro-progressive`, or `pro-ladder` budget on a branch that is still flat or impractical in `pro-reliability`.

### Production wiring stays frozen until proof exists

Do not wire turn-engine behavior into shipping `runtime_current` until the candidate clears the earned path and release-only speed gates.

---

## 4. Mainline roadmap

## Phase 0 — add the missing selector/search measurement surface

### Goal

Make each next split answer:
- did the hotspot move?
- did it move in the intended place?
- did the front gate remain stable?
- did the cost move into cache overhead instead of disappearing?

### Add a bounded hotspot probe

Add a small ignored diagnostic surface that runs on a fixed corpus of Pro positions and reports at least:

- `ranked_child_states` calls
- child transitions enumerated
- child transitions fully scored
- `score_state(...)` calls inside child ordering
- `build_move_efficiency_snapshot(...)` calls
- `ManaPathSnapshot::from_board` calls
- `evaluate_preferability_with_weights_and_exact_policy(...)` calls
- `can_attack_target_on_board(...)` calls and cache hits
- `actor_payload_after_move(...)` calls
- `drainer_immediate_threats(...)` calls
- tactical-projection calls split by flag/profile shape
- exact secure-mana calls if the hotspot drifts back there

### Corpus to use

Use a fixed position corpus, not random states:

- representative `primary_pro` fixtures
- a few recent `pro-reliability` losses
- a few disagreement-harvest positions
- one quiet positional state
- one obvious drainer-risk state
- one exact-progress state
- one spirit-development state

### Where to put it

- `src/models/mons_game_model.rs`
- `src/models/scoring.rs`
- `src/models/automove_exact.rs`
- `src/models/automove_experiments/tests.rs`

A reasonable ignored test name:

- `smart_automove_pro_reliability_hotspot_probe`

### Keep it non-invasive

Instrumentation must be:
- env-gated or ignored-test-only
- counters first, timing second
- easy to strip back if it perturbs behavior

### Phase 0 success criteria

- zero selector behavior change
- front gate remains identical
- the team can compare candidate A vs B without waiting for a giant duel run

---

## Phase 1A — semantics-preserving local reuse inside `ranked_child_states`

This is the highest-EV first change.

### Hypothesis

The current search wall is not only “too many children.” It is also “too much duplicated per-child work.”

If the team builds shared child analysis once and reuses it locally inside one sibling set, `pro-reliability` should either become practical or at least move the hotspot cleanly upward.

### The design

Add a **node-local** reuse layer rather than another broad thread-local/global cache.

Recommended types:

#### `ChildOrderingScratch` in `mons_game_model.rs`
Suggested contents:
- `before_state_hash`
- `before_efficiency_snapshot`
- compact per-child storage keyed by `child_hash`
- optional tiny per-child same-board reach/threat memo if still needed later

Implementation note:
- prefer a pre-sized `Vec` / small linear-probe structure over new hot-path `HashMap` usage
- reserve using the expected child count / branch limit
- keep lifetime local to one `ranked_child_states(...)` call

#### `ChildEvalBundle` in `mons_game_model.rs`
Suggested fields:
- `child_hash`
- prebuilt after-state `MoveEfficiencySnapshot`
- optional cached preferability score
- drainer-vulnerability transition
- move-class flags
- tactical-extension trigger
- any cheap event-derived markers the later ordering pass will need

#### `StaticEvalContext` (or `ScoringEvalContext`) in `scoring.rs`
Suggested fields:
- `state_hash`
- `ManaPathSnapshot`
- optional exact strategic analysis
- optional already-available tactical projection
- optional memo for repeated drainer-safety / immediate-threat lookups within one evaluation

### Concrete implementation steps

1. In `ranked_child_states(...)`, keep the existing `before_state_hash` and `before_efficiency_snapshot`, but stop rebuilding the same child summaries independently.
2. Split “build child analysis” from “consume child analysis.”
3. Add a context-aware scoring entrypoint, e.g.
   - `evaluate_preferability_with_context(...)`
4. Add a snapshot-aware move-efficiency path, e.g.
   - `move_efficiency_delta_from_before_snapshot_with_after_snapshot(...)`
5. Build one `ChildEvalBundle` per child and reuse it for:
   - static evaluation
   - move-efficiency ordering
   - drainer vulnerability transition
   - move-class classification

### Important design rule

Do **not** solve this phase by expanding a large global cache first.

The repo already has evidence that broad cache expansion can just move the hotspot into:
- thread-local lookup cost
- hash-map insert cost
- cache churn / clear behavior

Phase 1A should be local reuse, not another cache wave.

### Specific code touchpoints

- `src/models/mons_game_model.rs`
  - `ranked_child_states`
  - `move_efficiency_delta_from_before_snapshot`
  - `build_move_efficiency_snapshot`
  - `score_state`
- `src/models/scoring.rs`
  - `evaluate_preferability_with_weights_and_exact_policy`
  - `ManaPathSnapshot::from_board`
- optional narrow helpers in `src/models/automove_exact.rs`

### Tests to add

Parity first:
- context-aware scoring equals existing scoring on curated positions
- snapshot-aware move-efficiency delta equals existing delta
- vulnerability transition parity
- move-class parity

### Phase 1A success criteria

Keep the front gates clean while materially reducing the hotspot share of:
- `ranked_child_states`
- `evaluate_preferability_with_weights_and_exact_policy`
- `build_move_efficiency_snapshot`
- `ManaPathSnapshot::from_board`

### Kill criteria

Kill / back out Phase 1A if:
- off-target triage drift appears
- the hotspot just moves into scratch bookkeeping or map churn
- there is no clear runtime story after one focused split

---

## Phase 1B — lazy exact escalation / two-stage child ordering

Only do this if Phase 1A alone is not enough.

### Hypothesis

Not every enumerated child deserves the full expensive bundle.
A cheap pass can reject obvious non-survivors before full exact spending.

### The design

Split child ordering into two passes:

#### Pass A: cheap pass over all enumerated children

Allowed here:
- terminal score / immediate tactical outcome
- event ordering bonus
- TT-best bonus
- killer/history bonus
- a cheap approximate/static score
- cheap event-derived tactical markers
- already-available parent/child surface facts from Phase 1A

Not allowed here by default:
- full exact static evaluation
- full `move_efficiency_snapshot` rebuild
- full reach/vulnerability/classification bundle when it is not needed to decide shortlist survival

#### Pass B: expensive pass only on the shortlist

Escalate only:
- top `node_branch_limit * 2` children (start conservative)
- plus an always-retain tactical reserve

### Always-retain tactical reserve

Do not allow the cheap pass to suppress these:

- terminal / winning children
- children that score mana this turn
- children that faint the opponent drainer
- children that preserve surfaced exact progress
- TT-best child if present
- killer child if present
- obvious tactical-extension-trigger children
- any child that resolves own-drainer exposure

### Recommended candidate-only knobs

Keep them candidate-only and off by default:

- `enable_child_eval_bundle`
- `enable_local_scoring_eval_ctx`
- `enable_two_stage_child_ordering`
- `child_ordering_shortlist_multiplier`
- `child_ordering_tactical_reserve`

### Why this is not the first move

This phase changes search policy.
That is why it comes after the semantics-preserving reuse pass.

### Tests to add

Shortlist safety tests:
- exact safe supermana progress child survives shortlist
- exact safe opponent-mana progress child survives shortlist
- drainer-faint child survives shortlist
- TT/killer preserved child survives shortlist
- tactical top-2 coverage still holds

### Phase 1B success criteria

- front gates remain green
- full expensive child evaluations drop materially
- `pro-reliability` becomes clearly more practical

### Kill criteria

If this phase causes triage drift or opening regressions, revert and either:
- make the shortlist more conservative
- or move to the reach/threat cleanup phase instead

---

## Phase 2 — attack-reach / threat cleanup after shortlist reduction

After the search-side child count is under control, remove the remaining same-board helper churn.

### Start narrow, not broad

Start with low-risk same-board reuse:

#### 1. Add `_with_hash` variants
Thread precomputed board/state hashes into hot same-board helpers, especially:
- `can_attack_target_on_board(...)`
- any same-board tactical helper still recomputing a hash

#### 2. Add a tiny per-surface immediate-threat memo
Inside one child surface / one evaluation context, memoize:
- `drainer_immediate_threats(board, color, location)`

#### 3. Only if still hot, predecode square legality / direct board-slot facts
If `actor_payload_after_move(...)` and square checks remain high after Phase 1A/1B:
- predecode hot legality facts
- or use more direct board-slot reads in the hot path

### Conditional escalation: board-scoped attack map

If attack reach is **still** a dominant hotspot after the narrow fixes, then graduate to the board-scoped alternative from P2/P4.

Recommended shape:
- `AttackReachSummary`
- keyed by `(board_hash, attacker_color, target_color, remaining_moves, can_use_action)`
- stores a board-scoped attacked-target bitset / summary rather than one target answer

Start narrow:
- drainer vulnerability
- immediate threat counts
- reply-risk queries that hit many targets on the same board

Do not replace every attack query in one shot unless the narrow rollout is clean.

### Code touchpoints

- `src/models/automove_exact.rs`
  - `can_attack_target_on_board`
  - `actor_payload_after_move`
  - `drainer_immediate_threats`
- `src/models/scoring.rs`
- `src/models/mons_game_model.rs`

### Tests to add

- `_with_hash` parity vs current helper
- immediate-threat memo parity
- board-scoped attack-summary parity
- edge cases: mystic, demon, bomb, guarded target, fainted units

### Phase 2 success criteria

Meaningful reduction in:
- `can_attack_target_on_board`
- `actor_payload_after_move`
- `drainer_immediate_threats`

without moving the wall into new cache overhead.

---

## Phase 3 — oracle tactical-projection narrowing (conditional)

Only make this the next priority if a fresh clean sample moves back to:

- `oracle_walk_seeds`
- `spirit_impact_seeds`
- `build_exact_turn_tactical_projection`
- `exact_tactical_spirit_summary`
- immediate-window preview churn

### Why this is conditional

The code and backlog already show the hotspot moving back and forth between:
- search-side ordering
- macro oracle
- secure recursion

So Phase 3 is a **branching response**, not guaranteed next work.

### Recommended implementation

#### 1. Replace raw flag mixes with explicit projection profiles
Example profile enum:
- `SafeProgressOnly`
- `SpiritWindowOnly`
- `DrainerOpportunity`
- `SpiritOpportunity`
- `SelectorWindow`

#### 2. Narrow projection per caller / actor family
For example:
- non-spirit opportunity discovery should not pay for spirit-only fields
- callers that cannot convert denial this turn should not request denial
- safe-progress callers should not build more than they need

#### 3. Add a borrowed local projection memo
Memo scope:
- one oracle/root context only

Key idea:
- `(board_hash, remaining_moves, profile)` or equivalent

#### 4. Keep tightening inside spirit-preview code
If the hotspot is specifically in `exact_tactical_spirit_summary`:
- memoize after-board immediate-window results inside one query
- early-exit once score/denial maxima that matter to the caller are reached

#### 5. Optional split: separate score-window construction from other projection work
This is worth trying only if profiling says bundled score-window demand is still overpaying.

### Code touchpoints

- `src/models/automove_turn_engine.rs`
  - `oracle_walk_seeds`
  - `spirit_impact_seeds`
  - actor capability / flag selection helpers
- `src/models/automove_exact.rs`
  - tactical projection builder
  - spirit tactical summary
  - immediate-window helpers

### Phase 3 success criteria

The hotspot probe and live samples both show a clear reduction in the oracle surface without front-gate drift.

---

## Phase 4 — secure recursion structural cut (conditional)

Only take this path first if the hotspot returns to secure recursion after the earlier phases.

### Recommended implementation order

#### 1. Reuse simulation state where the caller already has one
Avoid rebuilding a synthetic `MonsGame` when a search caller already has a simulation state.

#### 2. Remove remaining turn-end rescans if possible
If turn-end secure recursion still scans too much board state:
- carry only the minimum extra state in the secure key or local scratch
- avoid broad new key expansions unless profiling proves the value

#### 3. Continue direct mutation / rollback style cuts
Keep using direct slot mutation and rollback inside secure walk helpers instead of clone-heavy wrappers.

#### 4. Only if still necessary, carry wakeup/free-mana presence data deeper
This is a later secure-specific alternative, not the first cut now.

### Code touchpoints

- `src/models/automove_exact.rs`
  - secure-mana recursion
  - secure drainer-walk helpers
  - synthetic state creation paths
  - turn-end wakeup logic

### Success criteria

The secure recursion surface becomes clearly secondary or disappears from top live samples.

---

## Phase 5 — one targeted acceptance-lane pass after CPU headroom exists

Only do this after the shared selector/search wall is smaller.

### Why it belongs later

The current acceptance logic is complex, but the current backlog says the dominant live blocker is broader search/ordering cost.
Acceptance should be a **targeted** strength conversion pass once the system is cheap enough to measure cleanly.

### Recommended acceptance strategy

Combine the best parts of P1 and P6:

#### Step 1: split acceptance into cheap and expensive phases
Cheap phase should use only:
- scored roots
- candidate rank
- score gap
- wins-immediately
- same-turn score window
- drainer attack
- drainer safety
- progress surface
- spirit surface
- head family

Expensive phase should only run for:
- `MAYBE` cases from the cheap phase
- tight rank caps
- families where projected override actually matters:
  - `ImmediateScore`
  - `DenyOpponentWindow`
  - `DrainerKill`
  - emergency `DrainerSafetyRecovery`

#### Step 2: start with one disagreement family
Target first:
- `SafeSupermanaProgress`
- `SafeOpponentManaProgress`

Reason:
- these are core strategy-interview priorities
- the acceptance logic already has specialized progress logic
- this is the cleanest likely source of “engine found the right whole-turn idea but selector rejected it”

#### Step 3: use disagreement harvest, not intuition
Use existing tooling:
- `smart_automove_pro_reliability_loss_probe`
- `smart_automove_pro_reliability_disagreement_harvest`
- `turn_engine_acceptance_probe_for_test`

### New diagnostics to add

- `expensive_override_checks`
- `expensive_override_accepts`
- `projected_plan_override_checks`
- `projected_plan_override_accepts`
- acceptance buckets by plan family

### Success criteria

A real disagreement bucket moves and `pro-reliability` improves without creating tactical-blind regressions.

### Kill criteria

If one focused acceptance split does not move a real disagreement family, stop and go back to the mainline hotspot story.

---

## Phase 6 — promotion proof and production integration

Once one serial successor is clearly stronger **and** practical:

1. run the full canonical Pro earned path
2. run the release-only speed gates
3. only then discuss wiring the improvement into the shipping Pro path

### Production invariants to add before any wiring change

- turn-engine stays disabled for Fast and Normal
- turn-engine is only enabled where intended for Pro
- opening-book / early-fallback ordering remains intact
- mixed-runtime speed gate remains green

### Useful production tests

- Pro-only activation invariant tests
- continuation cache legality under state drift
- compile-success coverage / compile-failure diagnostics by action family

---

## 5. Suggested serial candidate sequence

Keep only **one live successor at a time**.

### Instrumentation-only change
If the patch is behavior-neutral, keep the profile id as `runtime_pro_turn_engine_v30`.

### First behavioral successor
`runtime_pro_turn_engine_v31_ordering_bundle_v1`

Meaning:
- v30
- hotspot probe
- local child surface reuse
- no ordering-policy change yet

### Second behavioral successor
`runtime_pro_turn_engine_v32_two_stage_ordering_v1`

Meaning:
- v31
- cheap + expensive child ordering
- tactical always-retain reserve

### Third behavioral successor
`runtime_pro_turn_engine_v33_reach_cleanup_v1`

Meaning:
- v32
- same-board reach/threat cleanup
- possibly board-scoped attack summary if still warranted

### Fourth behavioral successor
`runtime_pro_turn_engine_v34_accept_progress_v1`

Meaning:
- v33
- cheap/expensive acceptance split
- one targeted progress-family acceptance pass

### Important serial rule

Do not keep all these live in the active registry at once.
Use them as **serial placeholders** only.

---

## 6. Exact files and functions to work in

## `src/models/mons_game_model.rs`
Highest priority:
- `smart_search_best_inputs_internal`
- `accept_turn_engine_head_after_search`
- `ranked_child_states`
- `move_efficiency_delta_from_before_snapshot`
- `build_move_efficiency_snapshot`
- `score_state`

Also important:
- selector diagnostics
- low-budget / mid-turn / eligibility guard helpers
- any root/child coverage truncation helpers

## `src/models/scoring.rs`
Highest priority:
- `evaluate_preferability_with_weights_and_exact_policy`
- `ManaPathSnapshot::from_board`
- any repeated drainer-safety / immediate-threat work inside evaluation

## `src/models/automove_exact.rs`
Highest priority:
- `can_attack_target_on_board`
- `drainer_immediate_threats`
- `actor_payload_after_move`
- tactical projection builder
- secure recursion helpers if the hotspot shifts back there

## `src/models/automove_turn_engine.rs`
Highest priority only if the hotspot shifts back there:
- `oracle_walk_seeds`
- `spirit_impact_seeds`
- tactical projection flag/profile construction
- plan caching/continuation reuse if required by the split

## `src/models/automove_experiments/profiles.rs`
Use for:
- candidate-only flags
- serial successor ids
- hybrid fallback profile if and only if the team intentionally switches to that plan

## `src/models/automove_experiments/tests.rs`
Use for:
- hotspot probe
- parity tests
- disagreement-harvest helpers
- tactical-prior regression pack
- compile / cache legality regressions

## `src/models/automove_experiments/harness.rs`
Use for:
- bounded duel/hotspot helper wiring if needed
- promotion metrics / pool evaluation support

## `scripts/run-automove-experiment.sh`
Leave canonical stage order intact.
Add only small convenience aliases if they are clearly useful.

---

## 7. Tests and diagnostics to add or expand

## New or expanded diagnostics

### Hotspot / observability
- `smart_automove_pro_reliability_hotspot_probe` (new)
- candidate notes template recorded in `AUTOMOVE_IDEAS.md`

### Existing diagnostics to lean on harder
- `smart_automove_pro_reliability_loss_probe`
- `smart_automove_pro_reliability_disagreement_harvest`
- `turn_engine_acceptance_probe_for_test`
- `smart_automove_pro_turn_engine_stage1_cpu_probe` (optional diagnostic)

## Parity tests
- scoring with and without context reuse
- move-efficiency delta with reused after-snapshot
- `_with_hash` reach helper parity
- board-scoped attack summary parity
- tactical projection parity after flag/profile narrowing
- secure recursion parity after structural cuts

## Tactical-prior regression pack
Derived from the strategy interview, add cheap fixtures that explicitly protect:

1. attack opponent drainer unless same-turn win / safe supermana / safe opponent mana is better
2. prefer safe supermana over quiet development
3. prefer safe opponent-mana scoring over quiet development
4. reject quiet moves that leave your own drainer exposed
5. move spirit off base early when no higher tactical priority dominates
6. respect potion-created scoring threats

## Legality / cache correctness tests
- continuation cache legality after state drift
- cached-step mismatch rejection
- compile-attempt / compile-failure coverage by plan family / action type

## Low-risk parallel fixture pack
From P3:
- stuck states
- bounded-progress safety
- cheap promotion-hygiene fixtures

---

## 8. Metrics and hard gates to keep in front of the team

Use these as hard “do not rationalize around this” numbers.

## Workflow / promotion metrics
- minimum opponents beaten for pool promotion: **4**
- minimum confidence to count an opponent as beaten: **0.75**
- `MAX_GAME_PLIES`: **320**

## Stage-1 CPU
- Fast max ratio: **1.30**
- Normal max ratio: **1.30**
- Pro max ratio: **1.30**

## Pro reliability
- minimum win rate: **0.90**
- minimum confidence: **0.99**

## Pro fast/progressive
- `pro-fast-screen` minimum delta: **0.0**
- `pro-progressive` meaningful delta: **0.04**
- `pro-progressive` meaningful confidence: **0.65**

## Pro ladder / primary promotion proof
- minimum delta vs Normal: **0.08**
- minimum delta vs Fast: **0.08**
- minimum confidence: **0.90**

## Pro CPU ratio target band
- minimum target: **0.50**
- maximum target: **10.00**

## Release-only speed gates

### Opening black reply guard
- Fast: **250 ms**
- Normal: **250 ms**
- Pro: **500 ms**

### Mixed runtime speed gate
- Fast: **450 ms**
- Normal: **1800 ms**
- Pro: **4800 ms**
- Normal/Fast ratio max: **14.75**
- Pro/Normal ratio must remain within the Pro CPU target band above

---

## 9. Session-by-session run protocol

## Every serious candidate

```sh
./scripts/run-automove-experiment.sh guardrails <candidate> runtime_current
SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage <candidate> runtime_current
./scripts/run-automove-experiment.sh runtime-preflight <candidate> runtime_current
```

## Bounded diagnostic direct stage before full spend

```sh
SMART_PRO_RELIABILITY_REPEATS=2 SMART_PRO_RELIABILITY_GAMES=4 SMART_PRO_RELIABILITY_MAX_PLIES=84 ./scripts/run-automove-experiment.sh pro-reliability <candidate> runtime_current
```

Then, if the story is good, run the full direct gate.

## Full direct gate

```sh
./scripts/run-automove-experiment.sh pro-reliability <candidate> runtime_current
```

## Only after direct reliability is clearly green

```sh
./scripts/run-automove-experiment.sh pro-fast-screen <candidate> runtime_current
./scripts/run-automove-experiment.sh pro-progressive <candidate> runtime_current
./scripts/run-automove-experiment.sh pro-ladder <candidate> runtime_current
```

## Promotion-time only

```sh
cargo test
cargo test --release --lib --no-run
cargo test --release --lib smart_automove_release_opening_black_reply_speed_gate -- --ignored --nocapture
cargo test --release --lib smart_automove_release_mixed_runtime_speed_gate -- --ignored --nocapture
```

---

## 10. Stop rules

Keep these brutally simple.

Kill the candidate immediately if:
- `guardrails` fail
- `pro-triage` drifts off-target without a deliberate audit reason
- `runtime-preflight` fails
- first earned duel stage is flat/negative
- progressive fades or hits a runtime cliff
- there is no clear story after one focused split

Campaign-specific additions:
- if a proposed reuse layer moves the hotspot into map/cache overhead instead of removing it, back it out
- if two-stage child ordering causes triage drift, revert before widening the branch family
- if a targeted acceptance split does not move a real disagreement family, stop tuning acceptance and return to the hotspot story

---

## 11. What to stop doing right now

- no wrapper-family reopen
- no old intent-planner reopen as the main path
- no generic `SmartSearchConfig` fishing expedition
- no Fast/Normal side campaign
- no opening-specific detour unless fresh evidence points there
- no broad new global hot-path cache as the first move
- no shipping runtime wiring changes before proof exists

---

## 12. Alternatives if the mainline stalls

These alternatives are preserved intentionally. They are not dead ideas; they are just not my recommended **first** implementation path.

## Alternative A — hybrid `runtime_current` floor + selective ProV2 overlay (P1)

### When to switch to this
Switch here if:
- 2–3 focused shared-code splits still leave `v30` too expensive globally
- or the system clearly wants ProV2 only in certain states, not as a general Pro replacement

### Design
Create a hybrid profile using `runtime_current` as the floor and enable ProV2 only in states classified as:

- `OFF`
- `STRATEGIC`
- `EMERGENCY`

Use `exact_opportunity_context(...)` / opportunity delta as the classifier input:
- `same_turn_score_window_value`
- `opponent_window_deny_gain`
- `drainer_attack_available`
- `drainer_safety`
- safe progress steps
- `opponent_can_win_immediately`

Suggested behavior:
- `OFF` → shipping `runtime_current` unchanged
- `STRATEGIC` → bounded ProV2 head-plan path, but disable `enable_turn_engine_secondary_analysis` and `enable_turn_engine_selected_followup_projection`
- `EMERGENCY` → full v30-style ProV2 behavior

### Why it is attractive
- preserves the current Pro floor
- spends ProV2 where it is most likely to matter
- fits the existing guard/eligibility/opportunity-context architecture

### Why it is not mainline now
- it opens a new structural promotion story before the current retained frontier is exhausted
- it adds more routing complexity before the shared selector/search wall is fixed

## Alternative B — board-scoped attack summary earlier (P2, P4)

If attack reach remains dominant immediately, move this up earlier:
- board-scoped `AttackReachSummary`
- narrow rollout to drainer safety / immediate threats first

## Alternative C — oracle-local memo / per-caller projection narrowing first (P3, P7)

If the hotspot returns to `oracle_walk_seeds` quickly:
- add borrowed local tactical-projection memo
- narrow by explicit projection profiles
- optionally split score-window construction out of broader projection

## Alternative D — secure-incremental structural cut first (P3, P7)

If the hotspot returns to secure recursion before search-side work clearly dominates:
- extend secure incremental state handling
- reduce synthetic state rebuilds
- remove remaining turn-end rescans
- continue direct slot mutation/rollback work

## Alternative E — distilled root signal / teacher-student fallback (P3)

If the full `v30` runtime remains too expensive even after focused code work:
1. collect disagreement sets where `runtime_current` and `v30` choose different roots
2. label which side wins
3. fit a tiny offline rule/model on cheap root features
4. compile the result into a bounded root veto/bonus stage

This is the best structural fallback if the product needs `v30`-derived strength at `runtime_current`-like cost.

## Alternative F — lightweight ML move-ordering / seed-priority model (P5)

A CPU-safe ML path exists, but should be later:
- tiny linear / shallow MLP
- use only for move ordering / seed prioritization
- keep final decision inside existing search/engine
- quantize / integerize for low runtime cost

Good future path, not first path.

## Alternative G — ensemble controller / surface-based meta-policy (P5)

Use a meta-policy to choose:
- planner
- engine
- pure search
- fallback runtime
by surface / game context

This is more attractive after either:
- hybrid activation logic has proven useful
- or distillation has produced a cheap strong signal

## Alternative H — MCTS micro-search with strict cap (P5)

Worth preserving as a research option only.
Current concerns:
- determinism
- latency variance
- release-gate risk

Do not prioritize now.

## Alternative I — crisis-triggered tactical widening after savings (P7)

Once the candidate is cheap enough:
- spend extra search/engine budget only on critical lanes:
  - immediate drainer attack
  - safe same-turn supermana
  - safe same-turn opponent mana
  - own drainer exposure rescue

This is the right way to turn reclaimed CPU into strength later.

## Alternative J — early dominance pruning / compile hardening / cap retuning as supplements (P5)

Useful supplements, not the mainline:
- earlier dominance pruning using `TurnEngineUtility`
- compile-attempt / compile-failure coverage and hardening
- continuation legality tests
- tune opponent/reply caps instead of widening breadth
- tactical quiescence/tiebreak tightening in primary context

## Alternative K — low-risk parallel fixture work (P3)

Land in parallel:
- stuck-state fixtures
- bounded-progress safety fixtures
- tactical-prior regressions from the strategy interview

This will not create the main strength gain by itself, but it improves iteration quality and prevents waste.

---

## 13. Idea bank preserving every distinct proposal from P1–P7

This section is intentionally exhaustive.

| Idea | Source(s) | Recommended status | Where it fits |
|---|---|---:|---|
| Keep `runtime_pro_turn_engine_v30` as the one live frontier and treat the work as promotion completion, not a new planner hunt | P2, P3, P4, P6, P7 | **Mainline now** | Overall campaign |
| Hybrid `runtime_current` floor with OFF / STRATEGIC / EMERGENCY ProV2 activation tiers | P1 | **Structural fallback** | Alternative A |
| Use `exact_opportunity_context` / opportunity delta as the hybrid activation classifier | P1 | **Structural fallback** | Alternative A |
| In hybrid STRATEGIC states, disable `enable_turn_engine_secondary_analysis` and `enable_turn_engine_selected_followup_projection` | P1 | **Structural fallback** | Alternative A |
| Split `accept_turn_engine_head_after_search` into cheap and expensive phases | P1, P6 | **Later mainline** | Phase 5 |
| Reserve expensive projected overrides for only a narrow set of families and tight rank caps | P1, P6 | **Later mainline** | Phase 5 |
| Add bounded duel-hotspot probe after `runtime-preflight` | P2, P4, P6 | **Mainline now** | Phase 0 |
| Build sibling-scoped / node-local board-evaluation context instead of adding more global cache | P2, P3, P4, P7 | **Mainline now** | Phase 1A |
| `SearchEvalContext` / `ScoringEvalContext` / `StaticEvalContext` for scoring-side reuse | P3, P4, P7 | **Mainline now** | Phase 1A |
| `ChildEvalBundle` / `ChildOrderingScratch` for reused child analysis | P4, P7 | **Mainline now** | Phase 1A |
| Reuse one child analysis bundle across score, efficiency, vulnerability, and class work | P4, P6, P7 | **Mainline now** | Phase 1A |
| Keep the first optimization semantics-preserving before changing ordering policy | P7 | **Mainline now** | Phase 1A |
| Two-stage child ranking: cheap pass for all children, expensive pass for shortlist | P2, P3, P4, P6, P7 | **Conditional mainline** | Phase 1B |
| Always-retain tactical reserve during two-stage ordering | P6 | **Conditional mainline** | Phase 1B |
| Planner-aware exact suppression / lazy exact escalation for low-survival children | P3, P4, P6, P7 | **Conditional mainline** | Phase 1B / Phase 2 |
| Local same-board memo for `can_attack_target_on_board` and `actor_payload_after_move` | P3, P6 | **Mainline now** | Phase 2 |
| Add `_with_hash` helper variants and thread precomputed hashes downward | P6 | **Mainline now** | Phase 2 |
| Add per-surface memo for `drainer_immediate_threats` | P6 | **Mainline now** | Phase 2 |
| Predecode square legality / use more direct board-slot reads in hot BFS helpers | P6 | **Conditional mainline** | Phase 2 |
| Board-scoped attack map / `AttackReachSummary` instead of target-scoped cache | P2, P4 | **Conditional fallback** | Alternative B / Phase 2 escalation |
| Start board-scoped attack summary narrow (drainer safety/threats first) | P4 | **Conditional fallback** | Alternative B |
| Lazily split `ManaPathSnapshot` so not every caller pays full candidate-vector cost | P6 | **Mainline now** | Phase 1A |
| Add oracle-local tactical-projection memo / borrowed cache at `oracle_walk_seeds` / `spirit_impact_seeds` | P3, P7 | **Conditional mainline** | Phase 3 |
| Split score-window building out of broader tactical projection | P3 | **Conditional fallback** | Phase 3 |
| Replace ad-hoc flag mixes with explicit projection profiles (`SafeProgressOnly`, `SpiritWindowOnly`, etc.) | P7 | **Conditional mainline** | Phase 3 |
| Narrow projection by actor family / caller-convertible outcomes | P3, P7 | **Conditional mainline** | Phase 3 |
| Memoize after-board immediate-window/spirit-preview work inside one tactical spirit query | P3, P7 | **Conditional mainline** | Phase 3 |
| Secure recursion should become more incremental, not more globally cached | P3, P7 | **Conditional mainline** | Phase 4 |
| Extend secure key with additional free-mana / wakeup state if rescans still dominate | P3, P7 | **Conditional fallback** | Phase 4 |
| Stop rebuilding synthetic game state when caller already has a simulation state | P7 | **Conditional mainline** | Phase 4 |
| Remove full-board wakeup scan on secure turn end if possible | P7 | **Conditional mainline** | Phase 4 |
| Continue direct slot mutation / rollback inside secure drainer walk | P3, P7 | **Conditional mainline** | Phase 4 |
| Use `runtime_pro_turn_engine_v30` as an offline teacher and distill a cheap online root signal / veto/bonus model | P3 | **Structural fallback** | Alternative E |
| Build the distilled signal from disagreement sets and cheap root features | P3 | **Structural fallback** | Alternative E |
| Land a low-risk stuck-state / bounded-progress fixture pack in parallel | P3 | **Parallel low-risk** | Alternative K |
| Batch attack reach / threat summaries as a dedicated exact helper type | P4 | **Conditional fallback** | Alternative B |
| Only after runtime wall moves, spend effort on strength-only tuning | P4 | **Mainline later** | Post-promotion headroom |
| Encode pro-player priors as priors/tiebreakers, not broad hard overrides | P4, P7 | **Mainline later** | Post-promotion headroom |
| Add Pro-only production wiring with explicit invariants so engine never leaks into Fast/Normal | P5 | **Required later** | Phase 6 |
| Add compile-success coverage and action-family failure diagnostics | P5 | **Supplement now** | Tests/diagnostics |
| Add continuation-cache legality / cached-step mismatch tests | P5 | **Supplement now** | Tests/diagnostics |
| Try early dominance pruning using `TurnEngineUtility` | P5 | **Supplement later** | After mainline CPU work |
| Tune opponent/reply caps instead of widening breadth | P5 | **Later tuning** | After headroom exists |
| Use tactical quiescence / deterministic tiebreak carefully in Pro primary context | P5 | **Later tuning** | After headroom exists |
| Add tiny learned evaluator / seed-priority model for move ordering only | P5 | **High-risk future** | Alternative F |
| Add MCTS micro-search under a strict simulation cap | P5 | **Parked research** | Alternative H |
| Add ensemble/meta-controller that chooses planner vs engine vs search by surface | P5 | **Future structural alternative** | Alternative G |
| Four-PR sequence: hotspot measurement → two-stage child ordering → child-surface sharing → reach cleanup → acceptance lane | P6 | **Adapted into mainline** | Phases 0–5 |
| Target the first acceptance pass at `SafeSupermanaProgress` / `SafeOpponentManaProgress` | P6 | **Mainline later** | Phase 5 |
| Use disagreement harvest and acceptance probes before changing acceptance | P6 | **Mainline later** | Phase 5 |
| Add tactical-prior regression pack from the strategy interview | P7 | **Parallel low-risk** | Tests/diagnostics |
| After CPU savings, use crisis-triggered tactical widening instead of global breadth | P7 | **Mainline later** | Alternative I / post-promotion headroom |

---

## 14. Exact next session checklist

If I were continuing the campaign myself, the next session would do this:

1. land the bounded selector/search hotspot probe
2. implement the **Phase 1A** local child-analysis reuse layer
3. keep the first split semantics-preserving
4. run:
   - `guardrails`
   - `SMART_TRIAGE_SURFACE=primary_pro pro-triage`
   - `runtime-preflight`
   - bounded `pro-reliability`
5. inspect whether the hotspot:
   - stayed in child ordering
   - moved to oracle
   - moved to secure recursion
6. pick **exactly one** next split based on that result:
   - child ordering still dominant → Phase 1B
   - reach/threat still dominant → Phase 2
   - oracle dominant → Phase 3
   - secure dominant → Phase 4
7. do not touch production wiring or config sweeps yet

---

## 15. Bottom line

The repo has already done the hard part: it found a stronger Pro direction and compressed it into one retained frontier.

The next win is **not** “find another Pro concept.”
The next win is:

- keep `runtime_pro_turn_engine_v30` as the one live frontier
- remove repeated exact/tactical work locally inside child ordering
- only then add lazy exact escalation if needed
- then clean up the remaining same-board reach/threat churn
- only after CPU headroom exists, tune one acceptance lane
- if that still is not enough, switch structurally to hybrid activation or distillation rather than reopening another wrapper family

That is the cleanest path to a stronger, promotable Pro mode within the repo’s current CPU and release constraints.

---

## 16. Repo anchors consulted

### Workflow / backlog / durable notes
- `HOW_TO_ITERATE_ON_AUTOMOVE.md`
- `AUTOMOVE_IDEAS.md`
- `docs/automove-knowledge.md`
- `docs/automove-archive.md`
- `docs/automove-pro-strategy-interview.md`

### Main implementation files
- `src/models/automove_experiments/profiles.rs`
- `src/models/mons_game_model.rs`
- `src/models/scoring.rs`
- `src/models/automove_exact.rs`
- `src/models/automove_turn_engine.rs`
- `src/models/automove_experiments/tests.rs`
- `src/models/automove_experiments/harness.rs`
- `src/models/automove_experiments/mod.rs`
- `scripts/run-automove-experiment.sh`

### Uploaded proposal docs synthesized here
- `1.md`
- `2.md`
- `3.md`
- `4.md`
- `5.md`
- `6.md`
- `7.md`
