# Automove Ideas

This is the active backlog for upcoming automove iterations.

Use `HOW_TO_ITERATE_ON_AUTOMOVE.md` as the execution playbook. Keep this file lean: current state, live frontier, workflow backlog, and compact recent outcomes only. Move durable lessons to `docs/automove-knowledge.md` and wave history to `docs/automove-archive.md`.

## Current State (2026-03-24)

- Production Pro in `runtime_current` still uses the promoted turn-opportunity planner from March 18, 2026.
- The large ProV2 turn-engine wave has been compressed. Durable shared work is retained in the engine/runtime code:
  - opportunity-context extraction
  - best-plan / no-plan / continuation caching
  - config-fingerprinted cache reuse
  - selector utility and followup-floor caching
  - low-budget / eligibility / resume logic
  - Pro-aware `runtime-preflight`, `pro-reliability`, and duel-progress workflow support
- `runtime_pro_turn_engine_v30` is the sole active ProV2 frontier.
- `runtime_pro_turn_engine_v1` remains only as a retained reference/baseline, not the live frontier.
- `runtime_pro_turn_engine_v30` is stronger than the earlier wrapper splits and cleared direct/faster earned stages during the wave, but it is still not promotable because the earned path is incomplete and strict direct-gate thresholds have now been restored.
- Default artifact layout is now:
  - logs: `target/experiment-runs/<candidate>/`
  - workflow-only logs: `target/experiment-runs/misc/`
  - runtime-preflight stamps: `target/experiment-stamps/`

## Idea Template

### Idea: <short name>

- Base profile: `runtime_current`
- Target mode:
- Triage surface:
- Triage pass signal:
- Calibration gate:
- Expected upside:
- CPU risk:
- Cheapest falsifier:
- Current blocker:
- Next split:
- How to test:
- Status:

## Active Frontier

### Idea: Pro turn engine v30 completion

- Base profile: `runtime_current`
- Target mode: `pro`
- Triage surface: `primary_pro`
- Triage pass signal: `runtime_pro_turn_engine_v30` keeps moving `primary_pro` fixtures while preserving `runtime-preflight` and re-earning direct `runtime_pro_turn_engine_v30` vs `runtime_current` reliability under the restored strict gate
- Calibration gate: none
- Expected upside: stronger full-turn planning and continuation reuse than shipping Pro without reopening the old CPU-heavy branch
- CPU risk: medium (earned-path runtime already improved, but progressive/larger duels still need proof)
- Cheapest falsifier: strict `pro-reliability` or the next earned duel stage stays flat after one exact live-loss fix
- Current checkpoint:
  - shared ProV2 planning/cache infrastructure is retained in the common engine path
  - `runtime-preflight` is Pro-aware and `pro-reliability` is part of the canonical workflow
  - `runtime_pro_turn_engine_v30` is the retained frontier after the compressed `v2`..`v30` wave
  - earlier wrapper splits and branch-specific clutter have been archived out of the active registry
- Current blocker: the frontier still has not completed the full earned path under the restored strict direct gate and progressive/ladder proof remains incomplete
- Recent outcome:
  - the stronger macro-head acceptance blockers (`primary_spirit_setup`, `primary_black_loss_opening_a_ply19`, `human_win_pro_a`) are now covered by direct regressions and stay green on the v30 suite
  - shared-hash exact helpers now feed the turn oracle / eligibility gate, and secure-mana recursion prunes impossible drainer walks before `process_input`
  - `guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, and `runtime-preflight` still pass after those cuts, with `primary_pro` unchanged at `target_changed=14` and `off_target_changed=0`
  - a retained tactical projection path now lets `oracle_walk_seeds` / `spirit_impact_seeds` read only the spirit/progress fields they use instead of full `ExactTurnSummary`; focused exact tests were added for projection parity and the front of the earned path stayed green on that tree
  - secure-mana recursion now uses an exact-only incremental state key instead of full search-state hashing, and the narrow drainer-walk helper now applies its transition directly instead of routing through generic event application; focused parity tests cover pickup, score, invalid consumable pickup, and next-turn key parity
  - on the latest retained tree, `guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, and `runtime-preflight` still pass with `primary_pro` unchanged at `target_changed=14` and `off_target_changed=0`, but `pro-reliability` still does not finish in a practical promotion window
  - fresh duel samples on the latest tree no longer center on `exact_search_state_hash` or generic event application; the hot stack moved into broader search / reach work, especially `search_score`, `can_attack_target_on_board`, `actor_payload_after_move`, and exact/scoring hash-map inserts
  - a follow-up attempt to narrow secure-mana caching to `board_hash + color + remaining moves + wanted mana` made canonical `pro-triage` slow enough to be SIGTERM'd, so that split was discarded
  - a follow-up corridor / endpoint screen for non-drainer `oracle_walk_seeds` also regressed the front gate: live `primary_pro` triage drifted to off-target changes (`primary_supermana_progress`, `primary_opponent_mana_progress`, `primary_spirit_setup`, `primary_pvs_sensitive_search`, `human_win_pro_a`, `primary_black_gate_loss_b_ply31`, `primary_white_fast_screen_opening_0_ply9`) while samples still stayed in the same exact hotspot, so that screen was discarded too
  - fixed-size exact visitation tables now replace hash-backed seen sets for attack reach, spirit reach, drainer pickup, and generic payload BFS helpers; the direct v30 suite and `primary_pro` front gate stayed green on that tree
  - secure-mana step counting now reuses one cloned synthetic game and recurses in place with explicit undo instead of cloning a full `MonsGame` for every drainer step; the secure drainer-walk helper was split into an in-place transition plus the older clone-based parity wrapper
  - secure drainer-walk touched-location tracking now uses a fixed stack record with O(1) bitset dedupe instead of per-step heap `Vec` churn; `guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, and `runtime-preflight` all still pass on that final tree with `primary_pro` unchanged at `target_changed=14` and `off_target_changed=0`
  - even after those cuts, the final `pro-reliability` run still did not complete in a practical promotion window; the latest sample no longer centers on touched-item heap churn and instead spreads across `evaluate_preferability_with_weights_and_exact_policy`, `move_efficiency_snapshot`, `exact_apply_secure_drainer_walk_in_place`, `actor_payload_after_move`, `can_attack_target_on_board`, and `exact_secure_specific_mana_steps_in_game_with_key_at_mut`
  - retained on the latest tree: selector-side caches for `move_efficiency_snapshot` and `evaluate_search_preferability`, direct exact-summary reuse inside `scoring.rs`, and a cheaper turn-end wakeup scan inside `exact_apply_secure_drainer_walk_in_place`; `guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, and `runtime-preflight` still pass on that tree with `primary_pro` unchanged at `target_changed=14` / `off_target_changed=0`, and stage-1 CPU stayed well under the `1.300` limit (`0.286`, `0.328`, `0.476`)
  - a follow-up attempt to route main-search `score_state` through the same static-eval cache was backed out: live `pro-reliability` samples shifted the hotspot into thread-local cache access and hash-map inserts without completing the gate in a practical window
  - after backing that search-path cache back out, the blocker snapped back to secure exact drainer-walk recursion; the latest live sample was led by `exact_apply_secure_drainer_walk_in_place`, then `actor_payload_after_move`, `exact_best_drainer_pickup_path_filtered`, `exact_secure_specific_mana_steps_in_game_with_key_at_mut`, and `exact_board_hash`
  - latest retained exact-path cut: same-turn secure-mana search now stops doing full turn-switch wakeup work when the move hands control away, and `actor_payload_after_move` / touched-hash updates now use direct board-slot reads on their hot path; `guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, and `runtime-preflight` still pass on that tree with `primary_pro` unchanged at `target_changed=14` / `off_target_changed=0`, and stage-1 CPU stayed well under the `1.300` limit (`0.287`, `0.338`, `0.493`)
  - even after that cut, `pro-reliability` still did not finish in a practical promotion window; the latest live sample still centers on secure exact recursion, led by `exact_apply_secure_drainer_walk_in_place`, then `actor_payload_after_move`, `exact_secure_specific_mana_steps_in_game_with_key_at_mut`, `exact_best_drainer_pickup_path_filtered`, `exact_board_hash`, and `exact_secure_board_hash_after_touched_items`
  - latest retained correction/cut: `ExactSecureManaStateKey` now carries per-color free regular-mana counts, so secure drainer-walk turn-end checks stop rescanning the board for active-color mana while preserving last-move pickup correctness; a focused regression now covers “last move picks supermana” states
  - on that tree, `guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, and `runtime-preflight` still pass with `primary_pro` unchanged at `target_changed=14` / `off_target_changed=0`, and stage-1 CPU stayed well under the `1.300` limit (`0.290`, `0.330`, `0.472`)
  - `pro-reliability` still did not finish in a practical promotion window even after the incremental-count key; the latest live sample still leads with `exact_apply_secure_drainer_walk_in_place`, but the top-of-stack count dropped again, with the remaining hotspot now spreading across `actor_payload_after_move`, `exact_secure_specific_mana_steps_on_board`, `exact_secure_specific_mana_steps_in_game_with_key_at_mut`, `exact_best_drainer_pickup_path_filtered`, and `exact_board_hash`
  - latest retained exact-path cuts: secure same-turn synthetic states now use `MonsGame::new_simulation_state` instead of `MonsGame::new(false)` field patching; secure board hashing and free regular-mana counting were fused into one scan and reused directly when building the secure recursion key; secure-mana recursion now borrows `EXACT_SECURE_MANA_CACHE` once per top-level query instead of re-entering thread-local cache storage on every recursive step; and tactical-projection caching now keys directly off the caller-provided search-state hash instead of recomputing an exact board hash in hot `oracle_walk_seeds` callers
  - on the latest retained tree, `guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, and `runtime-preflight` still pass with `primary_pro` unchanged at `target_changed=14` / `off_target_changed=0`, and stage-1 CPU stayed well under the `1.300` limit (`0.287`, `0.329`, `0.474`)
  - even after those cuts, `pro-reliability` still did not finish in a practical promotion window; a fresh live sample after a two-minute duel run moved the visible wall upward but not out of the exact path, concentrating in `oracle_walk_seeds -> build_exact_turn_tactical_projection`, then `exact_tactical_spirit_summary`, `exact_secure_specific_mana_steps_on_board`, `exact_secure_specific_mana_steps_in_game_with_key_at_mut`, `exact_apply_secure_drainer_walk_in_place`, and hash-map insert/remove churn inside the secure-mana cache
  - latest retained tactical-projection cut: `exact_tactical_spirit_summary` is now field-scoped, so `ExactTurnTacticalProjection` only asks spirit tactical code for score/denial fields when projection flags do not request progress; a focused diagnostics regression now asserts that score-only tactical projection no longer emits secure-mana queries through spirit progress probing
  - latest retained tactical-window cut: the tactical spirit path now computes immediate score and immediate opponent-mana windows together, reusing one direct drainer pickup BFS when both are needed instead of paying separate cached pickup-path traversals for the same preview board
  - on the latest retained tree, `guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, and `runtime-preflight` still pass with `primary_pro` unchanged at `target_changed=14` / `off_target_changed=0`, and stage-1 CPU stayed well under the `1.300` limit (`0.296`, `0.330`, `0.472`)
  - even after those two tactical cuts, `pro-reliability` still did not finish in a practical promotion window; the final live sample no longer centers on exact secure-mana or tactical pickup-path work, and instead shifts into main search scoring and child ranking: `search_score`, `ranked_child_states`, `move_efficiency_snapshot`, `evaluate_preferability_with_weights_and_exact_policy`, with some remaining `can_attack_target_on_board` / `actor_payload_after_move`
  - latest retained selector/search cuts: `move_efficiency_snapshot` now reads active-side tactical fields from `exact_turn_tactical_projection` instead of full `exact_turn_summary`; root and child ranking reuse precomputed state hashes plus one cached parent efficiency snapshot; transient child efficiency snapshots no longer pay global cache insert/lookup cost; carrier-progress classification now uses a direct board scan instead of full move-efficiency snapshots; and transient root/child ordering evals now bypass `SEARCH_PREFERABILITY_CACHE`
  - on the latest retained tree, `guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, and `runtime-preflight` still pass with `primary_pro` unchanged at `target_changed=14` / `off_target_changed=0`, and stage-1 CPU stayed under the `1.300` limit (`0.405`, `0.444`, `0.608`)
  - even after those search-side cuts, a clean `pro-reliability` run still did not finish in a practical promotion window; the fresh final sample moved the visible wall back into the macro-planner exact oracle, concentrating in `oracle_walk_seeds -> build_exact_turn_tactical_projection`, then `exact_tactical_spirit_summary`, `exact_best_immediate_tactical_window_on_board`, `exact_secure_specific_mana_steps_on_board`, `exact_secure_specific_mana_steps_in_game_with_key_at_mut`, `exact_apply_secure_drainer_walk_in_place`, and `actor_payload_after_move`
  - latest retained projection split: tactical projection flags now distinguish spirit-score vs spirit-denial demand, so non-spirit opponent-progress walks only request denial fields, spirit walks request score fields, and score-window callers only include denial when they also need denial semantics; focused projection parity tests cover score-only, denial-only, and full-spirit combinations
  - latest retained safe-progress cut: `safe_progress_seeds` no longer pays full `exact_turn_summary`; it now reads only the wanted progress steps from `exact_turn_tactical_projection` plus `exact_best_score_steps_on_board`, preserving the same front gate (`guardrails`, `SMART_TRIAGE_SURFACE=primary_pro pro-triage`, `runtime-preflight`) with `primary_pro` unchanged at `target_changed=14` / `off_target_changed=0`, and stage-1 CPU still under the `1.300` limit (`0.409`, `0.441`, `0.602`)
  - even after those two cuts, a fresh clean `pro-reliability` run still did not finish in a practical promotion window; the latest live sample shows the safe-progress wall largely gone, but the duel hotspot is still dominated by `discover_macro_opportunities_v2 -> oracle_walk_seeds -> build_exact_turn_tactical_projection`, then `exact_tactical_spirit_summary`, `exact_best_immediate_tactical_window_on_board`, `actor_payload_after_move`, `exact_secure_specific_mana_steps_on_board`, and `exact_apply_secure_drainer_walk_in_place`
- Next split: keep the single v30 frontier, keep the retained secure-path key/direct-transition cuts, and target the new broader duel hotspot instead of revisiting full-state exact hashing; the live candidates now are tighter `can_attack_target_on_board` / reach caching, cheaper scoring-side path snapshots, or a proof-driven v30 screen that suppresses exact reach work where the macro planner cannot use it
  - after the latest in-place secure-search cuts, the next concrete split should target the mixed duel hotspot as one surface: either cache or batch `move_efficiency_snapshot` / scoring-side snapshots in the reply-risk path, or suppress exact secure-walk / attack-reach work earlier when a root cannot survive selector scoring anyway
  - after the latest backed-out score-state cache attempt, the concrete next split is narrower again: either carry incremental free-regular-mana presence / wakeup state through the secure drainer-walk key so turn-end recursion stops rescanning the board, or cut `exact_best_drainer_pickup_path_filtered` / `actor_payload_after_move` in the secure-mana path before revisiting broader scoring caches
  - after the latest retained exact cut, the most direct next split is still inside secure recursion: either fold free-regular-mana presence into `ExactSecureManaStateKey` so last-step turn-end checks stop touching the board, or rewrite the remaining `exact_apply_secure_drainer_walk_in_place` state updates around direct slot mutation / rollback to take more work out of the helper that still dominates the sample
  - after the latest incremental-count key lands, the next split should move one layer deeper into the same hotspot: direct slot mutation/rollback inside `exact_apply_secure_drainer_walk_in_place`, or a narrower cache/projection around `exact_best_drainer_pickup_path_filtered` so the secure recursion stops paying full pickup/path cost on every branch
  - after the latest constructor/cache-key wave, the concrete next split should target projection churn at the `oracle_walk_seeds` entry point instead of another isolated secure-walk micro-cut: either add a local tactical-projection memo / borrowed cache path for `oracle_walk_seeds` and `spirit_impact_seeds`, or narrow `build_exact_turn_tactical_projection` so walk-seed discovery only pays for the exact secure-mana and spirit fields that the active actor family can still convert into a macro opportunity
  - after the latest tactical-projection and tactical-window cuts, the next concrete split is no longer in `automove_turn_engine` exact projection; it should target the selector/search side that now dominates the duel sample, especially `move_efficiency_snapshot`, `ranked_child_states`, reply-risk preferability evaluation, or the remaining attack-reach probes that still surface under `search_score`
  - after the latest retained search-side cuts, the next concrete split moves back to the macro oracle itself: narrow `build_exact_turn_tactical_projection` per `oracle_walk_seeds` caller again, or take more allocation / preview churn out of `exact_tactical_spirit_summary`, `exact_best_immediate_tactical_window_on_board`, and secure-mana recursion under the walk-seed path
  - after the latest spirit-score/denial split and safe-progress cut, the most direct next split is still inside `oracle_walk_seeds`: either memoize `exact_best_immediate_tactical_window_on_board` / spirit preview work inside one tactical spirit query, or cut `actor_payload_after_move` / drainer-pickup churn inside `exact_tactical_spirit_summary` before touching broader selector code again
- How to test:
  - `guardrails -> SMART_TRIAGE_SURFACE=primary_pro pro-triage -> runtime-preflight`
  - `pro-reliability` against `runtime_current`
  - only after the strict direct gate is green: `pro-fast-screen -> pro-progressive -> pro-ladder`
- Status: active

## Workflow Backlog

### Idea: Stuck-state and bounded-progress safety fixtures

- Base profile: `runtime_current`
- Target mode: `fast`, `normal`, `pro`
- Triage surface: blocked until fixtures exist
- Expected upside: catch empty-selector, repeat-loop, and no-progress regressions before promotion
- CPU risk: low
- Cheapest falsifier: fixtures land but do not reject unsafe candidates any earlier than the current guardrails
- Current blocker: fixture pack does not yet cover these edge cases directly
- Next split: add the smallest promotable fixture pack and wire it into guardrails or triage
- How to test: add the fixtures, then confirm unsafe branches fail before duel spend
- Status: backlog

### Idea: Promotion-time rollup summary

- Base profile: workflow-only
- Target mode: workflow
- Triage surface: none
- Expected upside: faster promote/kill decisions without opening multiple raw logs
- CPU risk: low
- Cheapest falsifier: metadata and cleanup improvements are already enough, and no operator time is saved by adding a summary layer
- Current blocker: logs are better organized now, but promotion evidence still lives across multiple command outputs
- Next split: emit one compact per-stage rollup after progressive or ladder without changing any gate behavior
- How to test: add the summary output and confirm it replaces manual log spelunking on one live candidate
- Status: backlog

## Recently Closed / Parked

- Pro turn-engine wave compression: `runtime_pro_turn_engine_v2`..`v30` were reduced to one retained frontier plus archived lessons; see `docs/automove-archive.md`.
- Pro intent planner v2 stabilization: early gates and bounded ladder speed could be kept green in the emergency-only shape, but direct reliability remained flat and the branch did not justify live-frontier space.
- Fast tactical uplift against current Normal: repeated reply-risk, spirit-setup, opponent-mana, and scoring-only splits either failed triage, stayed flat at first duel, or hit progressive runtime cliffs; reopen only with a genuinely new code path.
- Pro turn-opportunity planner v1: promoted to production Pro on March 18, 2026; keep the rollout Pro-only because direct Fast/Normal transplants regressed Normal.
- Shared reply-risk / exact-lite cache reuse line: closed at `cache_reuse` triage.
