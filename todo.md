# Supermana Priority v1 (Fast-Only) — COMPLETED

## Status

**Promoted to production.** All changes are live in the `model_current_best` / `from_preference` production code path.

Gate test `runtime_supermana_priority_v1` passed (gate v2): fast 90W-54L (δ=+0.125, confidence=0.998), normal 73W-71L (neutral), pool 3/10 vs 1/10. See `docs/automove-experiments.md` for full promotion snapshot, gate redesign details, and iteration history.

### Changes
- `scoring.rs`: Added `supermana_race_control: 30` to `RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS`
- `mons_game_model.rs`: Fast mode `enable_supermana_prepass_exception = true`, boosted interview bonuses (600/320), prepass exception logic in `forced_tactical_prepass_choice`
- `mons_game_model_automove_experiments.rs`: Gate v2 redesign (per-mode non-regression + at-least-one-improves), supermana experiment profile (marked promoted)

### Next Priority Areas (from pro-strategy interview)
1. Get opponent's mana if safe — not yet implemented
2. Hold potion for scoring threats — not yet implemented
3. Spirit always off base — partially addressed, not promoted
4. Spirit to move own mana closer — not in search eval
5. Attack opponent spirit when quick — not yet implemented

---

# Boolean Drainer Protection/Attack System — COMPLETED

## Status

**Promoted to production.** All changes are live in the `model_current_best` / `from_preference` production code path.

Gate test `runtime_fast_boost_v1` passed: fast 72-72 (50%), normal 90-54 (62.5%), aggregate δ=0.062, speed ≤1.019x. See `docs/automove-experiments.md` for full promotion snapshot and experimental findings.

## Original Request

> let's remake the current automove implementation, run it through experiments flow, and iterate on it until we can promote it to production
>
> the current major weakness of it when playing versus human players is it doesn't properly protect drainer — either free or carriying anything, and keeping drainer alive and actively using it to carry mana is important to win.
>
> so we need to implement a reliable and efficient check wether our drainer can be attacked or not on the next opponent's turn, and we need to check as well if we can attack opponent drainer this very turn. this should include mystic attack, demon attack, or bomb attack. for mystic and demon attacks we need to account for angel protection.
>
> when we can run this quick and reliable check, we should remake board scoring so it doesn't calculate vague drainer danger, but instead uses boolean — wether there's drainer danger or not.
>
> using this check, we should always attack opponent drainer when possible on the same turn (we should skip that only if we can score supermana or opponent mana this very turn), and we should absolutely make sure we are never leaving our drainer vurnarable for opponents attack (again, this should be ok only if we score opponent mana or supermana this very same turn)
>
> the goal is to make sure the new implementation doesn't start using to much cpu and is promotable based on win rates. if cpu usage grows to much, let's optimize the implementation evaluating attack distances directly instead of using bfs. once you verify this with experiments and finish iterating, promote this implementation and report it's new cpu stats and wl ratio.

## Context

The automove AI's biggest weakness against human players is poor drainer protection. The root cause: board scoring uses a **vague continuous signal** (`drainer_at_risk / danger`) where `danger` is Chebyshev distance to the nearest threatening piece. A mystic at distance 4 contributes a penalty of `-420/4 = -105` even though it cannot attack until it reaches exact distance 2. This imprecision causes the AI to undervalue real threats and overvalue non-threats.

The fix: replace the continuous danger signal with a **boolean** — is the drainer exactly attackable or not — and enforce hard policies for attacking opponent drainer and protecting own drainer.

## Changes

### 1. `src/models/scoring.rs` — Boolean drainer danger in evaluation

**Add new weight fields** to `ScoringWeights` (near line 20):
```rust
pub drainer_danger_boolean: i32,      // flat penalty when drainer is exactly attackable
pub mana_carrier_danger_boolean: i32,  // flat penalty when mana carrier is exactly attackable
```

Set both to `0` in all existing presets (backward compat).

**Add `is_drainer_under_exact_threat()` function** (near `drainer_immediate_threats()` at line 2275). This checks exact attack geometry including MonBase exclusion (which `drainer_immediate_threats` misses) and covers all mon-containing items (`Mon`, `MonWithMana`, `MonWithConsumable`):
- Mystic: `|di| == 2 && |dj| == 2`, not on MonBase, not angel-guarded
- Demon: `(|di| == 2 && dj == 0) || (di == 0 && |dj| == 2)`, clear midpoint (no item, no SupermanaBase, no MonBase at midpoint), not on MonBase, not angel-guarded
- Bomb: `MonWithConsumable(Bomb)` within Chebyshev distance ≤ 3 (ignores angel protection)
- Returns `bool` with early return on first threat found

**Wire into evaluation** — in the three drainer scoring blocks:
1. `Item::Mon` drainer block (~line 814): after existing `drainer_at_risk / danger`, add boolean check
2. `Item::MonWithConsumable` drainer block (~line 894): same
3. `Item::MonWithMana` carrier block (~line 1044): after `mana_carrier_at_risk / danger`, add `mana_carrier_danger_boolean`

Guard with `weights.drainer_danger_boolean != 0` so zero-cost for presets that don't use it.

**Create new weight presets** for the candidate. Both fast and normal get boolean weights. The new presets zero out `drainer_at_risk` and `drainer_immediate_threat` and use `drainer_danger_boolean` instead:

For fast mode:
```
RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS = {
    ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS,
    drainer_at_risk: 0,
    drainer_immediate_threat: 0,
    drainer_danger_boolean: -600,
    mana_carrier_danger_boolean: -400,
}
```

For normal mode, create corresponding variants of each phase-adaptive preset (`RUNTIME_NORMAL_*_SPIRIT_BASE_SCORING_WEIGHTS`) with the same boolean substitution.

### 2. `src/models/mons_game_model.rs` — Hard drainer policies

**a) Harden drainer safety filter** in `filtered_root_candidate_indices()` (~line 3919).

Current: soft filter with 2,200-point score margin.
New: hard filter (no margin), with exceptions for supermana/opponent mana scoring:

```rust
if config.enable_root_drainer_safety_prefilter && !forced_attack_applied {
    let safer_indices = candidate_indices.iter().copied().filter(|index| {
        let root = &scored_roots[*index];
        !root.own_drainer_vulnerable
            || root.scores_supermana_this_turn
            || root.scores_opponent_mana_this_turn
    }).collect::<Vec<_>>();
    if !safer_indices.is_empty() {
        candidate_indices = safer_indices;
    }
}
```

**b) Add supermana/opponent-mana exceptions to forced drainer attack** (~line 3869).

Current: retains ONLY drainer-attacking candidates.
New: also allows candidates that score supermana or opponent mana:

```rust
if config.enable_forced_drainer_attack
    && candidate_indices.iter().any(|index| scored_roots[*index].attacks_opponent_drainer)
{
    candidate_indices.retain(|index| {
        let root = &scored_roots[*index];
        root.attacks_opponent_drainer
            || root.scores_supermana_this_turn
            || root.scores_opponent_mana_this_turn
    });
    forced_attack_applied = true;
}
```

**c) Update `forced_tactical_prepass_choice()`** (~line 2162) with same exception logic. If a winning-immediate or drainer-attacking candidate exists, still prefer it. But for the drainer safety path (~line 2186), add the supermana/opponent-mana exception.

**d) Fix `is_own_drainer_immediately_vulnerable()`** (~line 3696) to also check `MonWithMana` attackers. Currently only matches `Item::Mon`, but a mystic carrying mana can still attack. Change the match to include `Item::MonWithMana { mon, .. }`.

### 3. `src/models/mons_game_model_automove_experiments.rs` — Candidate profile

**Add candidate function** `model_runtime_boolean_drainer_v1`:
```rust
fn model_runtime_boolean_drainer_v1(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    // Override scoring weights with boolean variants
    runtime.scoring_weights = if config.depth < 3
        && config.enable_mana_start_mix_with_potion_actions
    {
        &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF
    } else {
        MonsGameModel::runtime_phase_adaptive_boolean_drainer_scoring_weights(game, config.depth)
    };
    MonsGameModel::smart_search_best_inputs(game, runtime)
}
```

**Add `runtime_pre_boolean_drainer` snapshot** (identical to `candidate_model_base`, captures pre-change baseline).

**Register** both in `candidate_model()` and `all_profile_variants()`.

### 4. Experiment Flow

Run using the logged wrapper. All commands from workspace root.

**Step 1: Smoke tests**
```bash
cargo test --lib smart_automove_pool_smoke_runs
cargo test --lib smart_automove_pool_keeps_ten_models
cargo check --release --target wasm32-unknown-unknown
```

**Step 2: Fast pipeline (speed + quick strength)**
```bash
./scripts/run-experiment-logged.sh fast_pipeline_boolean_drainer_v1 -- \
  env SMART_FAST_PROFILES=runtime_current,runtime_boolean_drainer_v1 \
      SMART_FAST_BASELINE=runtime_current \
      SMART_FAST_USE_CLIENT_MODES=true \
  cargo test --release --lib smart_automove_pool_fast_pipeline -- --ignored --nocapture
```
Gate: CPU fast ratio ≤ 1.15x, normal ratio ≤ 1.15x.

**Step 3: Quick mirrored duels (both orientations)**
```bash
./scripts/run-experiment-logged.sh quick_duel_a_boolean_drainer_v1 -- \
  env SMART_DUEL_A=runtime_boolean_drainer_v1 SMART_DUEL_B=runtime_current \
      SMART_DUEL_GAMES=2 SMART_DUEL_REPEATS=2 SMART_DUEL_MAX_PLIES=72 \
      SMART_DUEL_SEED_TAG=quick_v1 \
  cargo test --release --lib smart_automove_pool_profile_duel -- --ignored --nocapture

./scripts/run-experiment-logged.sh quick_duel_b_boolean_drainer_v1 -- \
  env SMART_DUEL_A=runtime_current SMART_DUEL_B=runtime_boolean_drainer_v1 \
      SMART_DUEL_GAMES=2 SMART_DUEL_REPEATS=2 SMART_DUEL_MAX_PLIES=72 \
      SMART_DUEL_SEED_TAG=quick_v1 \
  cargo test --release --lib smart_automove_pool_profile_duel -- --ignored --nocapture
```
Gate: aggregate delta win-rate ≥ +0.04.

**Step 4: Reduced gate**
```bash
./scripts/run-experiment-logged.sh reduced_gate_boolean_drainer_v1 -- \
  env SMART_CANDIDATE_PROFILE=runtime_boolean_drainer_v1 \
      SMART_GATE_BASELINE_PROFILE=runtime_current \
      SMART_GATE_PRIMARY_GAMES=2 SMART_GATE_PRIMARY_REPEATS=2 \
      SMART_GATE_CONFIRM_GAMES=2 SMART_GATE_CONFIRM_REPEATS=2 \
  cargo test --release --lib smart_automove_pool_promotion_gate_v2 -- --ignored --nocapture
```

**Step 5: Ladder**
```bash
./scripts/run-experiment-logged.sh ladder_boolean_drainer_v1 -- \
  env SMART_CANDIDATE_PROFILE=runtime_boolean_drainer_v1 \
      SMART_GATE_BASELINE_PROFILE=runtime_current \
      SMART_LADDER_ARTIFACT_PATH=target/smart_ladder_artifacts_boolean_drainer_v1.jsonl \
  cargo test --release --lib smart_automove_pool_promotion_ladder -- --ignored --nocapture
```

**Step 6: Full gate (only if reduced gate promising)**
```bash
./scripts/run-experiment-logged.sh full_gate_boolean_drainer_v1 -- \
  env SMART_CANDIDATE_PROFILE=runtime_boolean_drainer_v1 \
      SMART_GATE_BASELINE_PROFILE=runtime_current \
  cargo test --release --lib smart_automove_pool_promotion_gate_v2 -- --ignored --nocapture
```

### 5. If CPU Too High

The boolean check iterates `board.items` once with early return — should be cheaper than the existing `drainer_distances()` which always scans all items. If CPU is still too high:

1. **Merge into `drainer_distances()`**: add a 4th return value `(i32, i32, bool, bool)` — the boolean threat. This avoids a second board iteration entirely.
2. **Cache per color**: compute once per color per `evaluate_preferability_with_weights` call.

### 6. Iteration If v1 Doesn't Pass

| Knob | v1 | v2 (if weak) | v3 (if too aggressive) |
|------|-----|-------------|----------------------|
| `drainer_danger_boolean` | -600 | -750 | -450 |
| `mana_carrier_danger_boolean` | -400 | -500 | -300 |
| `drainer_at_risk` | 0 | 0 | -150 (partial blend) |

### 7. Promotion

After full gate passes:
1. Update `with_runtime_scoring_weights()` in `mons_game_model.rs` to use boolean presets
2. Make `runtime_phase_adaptive_boolean_drainer_scoring_weights()` the production path
3. Keep hardened safety/attack filter changes (already in production code)
4. Run final validation: `cargo test --lib && cargo check --release && cargo check --release --target wasm32-unknown-unknown`
5. Update `docs/automove-experiments.md` with promotion snapshot

## Files Modified

- `src/models/scoring.rs` — new weight fields, `is_drainer_under_exact_threat()`, boolean penalty in eval, new weight presets
- `src/models/mons_game_model.rs` — hardened filters in `filtered_root_candidate_indices()`, `forced_tactical_prepass_choice()`, fix `is_own_drainer_immediately_vulnerable()`, `runtime_phase_adaptive_boolean_drainer_scoring_weights()` helper
- `src/models/mons_game_model_automove_experiments.rs` — candidate profile, snapshot profile, registration
- `docs/automove-experiments.md` — new candidate docs, promotion snapshot

## Verification

1. `cargo test --lib smart_automove_pool_smoke_runs` — smoke test
2. `cargo test --lib smart_automove_pool_keeps_ten_models` — model registration
3. `cargo check --release --target wasm32-unknown-unknown` — WASM build
4. Full experiment flow (steps 2-6 above)
5. `cargo test --lib` + `cargo check --release` — final validation
