#![cfg(any(target_arch = "wasm32", test))]

use crate::models::scoring::{
    evaluate_preferability_with_weights_and_exact_policy, ScoringWeights, DEFAULT_SCORING_WEIGHTS,
};
use crate::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

const TURN_ENGINE_CACHE_MAX_ENTRIES: usize = 4096;
const TURN_ENGINE_COMPILE_LIMIT_MAX: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TurnEngineMode {
    ProV1,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TurnEngineConfig {
    pub own_seed_cap: usize,
    pub own_beam: usize,
    pub per_node_family_cap: usize,
    pub step_cap: usize,
    pub opponent_seed_cap: usize,
    pub opponent_beam: usize,
    pub reply_seed_cap: usize,
    pub reply_beam: usize,
    pub expansion_cap: usize,
    pub enable_spirit_family: bool,
    pub scoring_weights: &'static ScoringWeights,
    pub allow_exact_static_evaluation: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct TurnSnapshot {
    pub state_hash: u64,
    #[allow(dead_code)]
    pub active_color: Color,
    #[allow(dead_code)]
    pub white_score: i32,
    #[allow(dead_code)]
    pub black_score: i32,
    #[allow(dead_code)]
    pub remaining_mon_moves: i32,
    #[allow(dead_code)]
    pub can_use_action: bool,
    #[allow(dead_code)]
    pub can_move_mana: bool,
    #[allow(dead_code)]
    pub occupied: Vec<(Location, Item)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TurnAction {
    Walk {
        actor: Location,
        to: Location,
    },
    Attack {
        actor: Location,
        target: Location,
    },
    SpiritShift {
        actor: Location,
        target: Location,
        destination: Location,
    },
    Bomb {
        actor: Location,
        target: Location,
    },
    MoveMana {
        from: Location,
        to: Location,
    },
    ScoreCarry {
        actor: Location,
        wanted: Mana,
        step: Location,
    },
    SafetyRetreat {
        actor: Location,
        to: Location,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum TurnPlanFamily {
    ImmediateScore,
    DenyOpponentWindow,
    DrainerKill,
    SafeSupermanaProgress,
    SafeOpponentManaProgress,
    DrainerSafetyRecovery,
    SpiritImpact,
    ManaTempo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TurnEngineUtility {
    win_state: i32,
    avoid_immediate_loss: i32,
    score_delta: i32,
    deny_gain: i32,
    drainer_attack: i32,
    drainer_safety: i32,
    eval_score: i32,
}

impl Ord for TurnEngineUtility {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.win_state,
            self.avoid_immediate_loss,
            self.score_delta,
            self.deny_gain,
            self.drainer_attack,
            self.drainer_safety,
            self.eval_score,
        )
            .cmp(&(
                other.win_state,
                other.avoid_immediate_loss,
                other.score_delta,
                other.deny_gain,
                other.drainer_attack,
                other.drainer_safety,
                other.eval_score,
            ))
    }
}

impl PartialOrd for TurnEngineUtility {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
impl TurnEngineUtility {
    pub(crate) fn from_eval_score_for_test(eval_score: i32) -> Self {
        Self {
            win_state: 0,
            avoid_immediate_loss: 0,
            score_delta: 0,
            deny_gain: 0,
            drainer_attack: 0,
            drainer_safety: 0,
            eval_score,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TurnPlan {
    pub actions: Vec<TurnAction>,
    pub compiled_chunks: Vec<Vec<Input>>,
    pub end_game: MonsGame,
    pub end_snapshot: TurnSnapshot,
    pub utility: TurnEngineUtility,
    pub family: TurnPlanFamily,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TurnEngineDiagnostics {
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub seed_immediate_score: usize,
    pub seed_deny_window: usize,
    pub seed_drainer_kill: usize,
    pub seed_safe_supermana_progress: usize,
    pub seed_safe_opponent_mana_progress: usize,
    pub seed_safety_recovery: usize,
    pub seed_spirit_impact: usize,
    pub seed_mana_tempo: usize,
    pub accepted_plans: usize,
    pub accepted_immediate_score: usize,
    pub accepted_deny_window: usize,
    pub accepted_drainer_kill: usize,
    pub accepted_safe_supermana_progress: usize,
    pub accepted_safe_opponent_mana_progress: usize,
    pub accepted_safety_recovery: usize,
    pub accepted_spirit_impact: usize,
    pub accepted_mana_tempo: usize,
    pub compile_attempts: usize,
    pub compile_failures: usize,
    pub compile_failures_at_limit: usize,
    pub compile_state_mismatches: usize,
    pub compile_walk_attempts: usize,
    pub compile_walk_failures: usize,
    pub compile_attack_attempts: usize,
    pub compile_attack_failures: usize,
    pub compile_spirit_shift_attempts: usize,
    pub compile_spirit_shift_failures: usize,
    pub compile_bomb_attempts: usize,
    pub compile_bomb_failures: usize,
    pub compile_move_mana_attempts: usize,
    pub compile_move_mana_failures: usize,
    pub compile_score_attempts: usize,
    pub compile_score_failures: usize,
    pub compile_retreat_attempts: usize,
    pub compile_retreat_failures: usize,
    pub reply_search_calls: usize,
    pub fallback_no_plan: usize,
    pub fallback_budget_exceeded: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TurnEngineCacheKey {
    state_hash: u64,
    mode: TurnEngineMode,
}

#[derive(Debug, Clone)]
struct ActionSeed {
    family: TurnPlanFamily,
    action: TurnAction,
    priority: i32,
}

#[derive(Debug, Clone)]
struct PlanNode {
    game: MonsGame,
    actions: Vec<TurnAction>,
    compiled_chunks: Vec<Vec<Input>>,
    family: TurnPlanFamily,
}

#[derive(Clone)]
struct TransitionCompilePool {
    transitions: Vec<LegalInputTransition>,
    limit: usize,
    priority_locations: Vec<Location>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlanBuildStatus {
    NoPlan,
    BudgetExceeded,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TurnEngineProbeStatus {
    InactiveColor,
    CachedOnly,
    Planned,
    NoPlan,
    BudgetExceeded,
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) struct TurnEngineProbe {
    pub status: TurnEngineProbeStatus,
    pub cached_step: Option<Vec<Input>>,
    pub candidate_family: Option<TurnPlanFamily>,
    pub candidate_chunk: Option<Vec<Input>>,
    pub chunk_count: usize,
}

thread_local! {
    static TURN_ENGINE_CONTINUATION_CACHE: RefCell<HashMap<TurnEngineCacheKey, Vec<Input>>> =
        RefCell::new(HashMap::new());
    static TURN_ENGINE_DIAGNOSTICS: RefCell<TurnEngineDiagnostics> =
        RefCell::new(TurnEngineDiagnostics::default());
}

pub(crate) fn clear_turn_engine_plan_cache() {
    TURN_ENGINE_CONTINUATION_CACHE.with(|cache| cache.borrow_mut().clear());
}

pub(crate) fn clear_turn_engine_diagnostics() {
    TURN_ENGINE_DIAGNOSTICS.with(|diagnostics| {
        *diagnostics.borrow_mut() = TurnEngineDiagnostics::default();
    });
}

pub(crate) fn turn_engine_diagnostics_snapshot() -> TurnEngineDiagnostics {
    TURN_ENGINE_DIAGNOSTICS.with(|diagnostics| *diagnostics.borrow())
}

fn update_turn_engine_diagnostics(update: impl FnOnce(&mut TurnEngineDiagnostics)) {
    TURN_ENGINE_DIAGNOSTICS.with(|diagnostics| update(&mut diagnostics.borrow_mut()));
}

fn record_accepted_plan_family(family: TurnPlanFamily) {
    update_turn_engine_diagnostics(|diagnostics| match family {
        TurnPlanFamily::ImmediateScore => diagnostics.accepted_immediate_score += 1,
        TurnPlanFamily::DenyOpponentWindow => diagnostics.accepted_deny_window += 1,
        TurnPlanFamily::DrainerKill => diagnostics.accepted_drainer_kill += 1,
        TurnPlanFamily::SafeSupermanaProgress => diagnostics.accepted_safe_supermana_progress += 1,
        TurnPlanFamily::SafeOpponentManaProgress => {
            diagnostics.accepted_safe_opponent_mana_progress += 1
        }
        TurnPlanFamily::DrainerSafetyRecovery => diagnostics.accepted_safety_recovery += 1,
        TurnPlanFamily::SpiritImpact => diagnostics.accepted_spirit_impact += 1,
        TurnPlanFamily::ManaTempo => diagnostics.accepted_mana_tempo += 1,
    });
}

fn record_compile_attempt_for_action(action: TurnAction) {
    update_turn_engine_diagnostics(|diagnostics| match action {
        TurnAction::Walk { .. } => diagnostics.compile_walk_attempts += 1,
        TurnAction::Attack { .. } => diagnostics.compile_attack_attempts += 1,
        TurnAction::SpiritShift { .. } => diagnostics.compile_spirit_shift_attempts += 1,
        TurnAction::Bomb { .. } => diagnostics.compile_bomb_attempts += 1,
        TurnAction::MoveMana { .. } => diagnostics.compile_move_mana_attempts += 1,
        TurnAction::ScoreCarry { .. } => diagnostics.compile_score_attempts += 1,
        TurnAction::SafetyRetreat { .. } => diagnostics.compile_retreat_attempts += 1,
    });
}

fn record_compile_failure_for_action(action: TurnAction, at_limit: bool) {
    update_turn_engine_diagnostics(|diagnostics| {
        if at_limit {
            diagnostics.compile_failures_at_limit += 1;
        }
        match action {
            TurnAction::Walk { .. } => diagnostics.compile_walk_failures += 1,
            TurnAction::Attack { .. } => diagnostics.compile_attack_failures += 1,
            TurnAction::SpiritShift { .. } => diagnostics.compile_spirit_shift_failures += 1,
            TurnAction::Bomb { .. } => diagnostics.compile_bomb_failures += 1,
            TurnAction::MoveMana { .. } => diagnostics.compile_move_mana_failures += 1,
            TurnAction::ScoreCarry { .. } => diagnostics.compile_score_failures += 1,
            TurnAction::SafetyRetreat { .. } => diagnostics.compile_retreat_failures += 1,
        }
    });
}

impl TurnSnapshot {
    pub(crate) fn from_game(game: &MonsGame) -> Self {
        let mut occupied = game
            .board
            .occupied()
            .map(|(location, item)| (location, *item))
            .collect::<Vec<_>>();
        occupied.sort_by(|a, b| a.0.cmp(&b.0));
        Self {
            state_hash: MonsGameModel::search_state_hash(game),
            active_color: game.active_color,
            white_score: game.white_score,
            black_score: game.black_score,
            remaining_mon_moves: remaining_moves_for_color(game, game.active_color),
            can_use_action: game.player_can_use_action(),
            can_move_mana: game.player_can_move_mana(),
            occupied,
        }
    }
}

pub(crate) fn turn_engine_next_inputs(
    game: &MonsGame,
    perspective: Color,
    mode: TurnEngineMode,
    config: TurnEngineConfig,
) -> Option<Vec<Input>> {
    if game.active_color != perspective {
        return None;
    }

    if let Some(cached) = turn_engine_cached_step(game, mode) {
        return Some(cached);
    }
    let best_plan = turn_engine_candidate_plan(game, perspective, config)?;

    register_plan_continuations(game, mode, best_plan.compiled_chunks.as_slice());
    best_plan.compiled_chunks.first().cloned()
}

#[cfg(test)]
pub(crate) fn turn_engine_best_plan_for_test(
    game: &MonsGame,
    perspective: Color,
    config: TurnEngineConfig,
) -> Option<TurnPlan> {
    turn_engine_candidate_plan(game, perspective, config)
}

pub(crate) fn turn_engine_cached_step(game: &MonsGame, mode: TurnEngineMode) -> Option<Vec<Input>> {
    let cached = cached_step_if_legal(game, mode);
    update_turn_engine_diagnostics(|diagnostics| {
        if cached.is_some() {
            diagnostics.cache_hits += 1;
        } else {
            diagnostics.cache_misses += 1;
        }
    });
    cached
}

pub(crate) fn turn_engine_candidate_plan(
    game: &MonsGame,
    perspective: Color,
    config: TurnEngineConfig,
) -> Option<TurnPlan> {
    if game.active_color != perspective {
        return None;
    }
    match build_best_plan(game, perspective, config) {
        Ok(Some(plan)) => Some(plan),
        Ok(None) | Err(PlanBuildStatus::NoPlan) => {
            update_turn_engine_diagnostics(|diagnostics| diagnostics.fallback_no_plan += 1);
            None
        }
        Err(PlanBuildStatus::BudgetExceeded) => {
            update_turn_engine_diagnostics(|diagnostics| diagnostics.fallback_budget_exceeded += 1);
            None
        }
    }
}

pub(crate) fn turn_engine_commit_plan(game: &MonsGame, mode: TurnEngineMode, plan: &TurnPlan) {
    register_plan_continuations(game, mode, plan.compiled_chunks.as_slice());
}

pub(crate) fn turn_engine_compare_plans(left: &TurnPlan, right: &TurnPlan) -> Ordering {
    compare_plans(left, right)
}

pub(crate) fn turn_engine_evaluate_state_utility(
    game: &MonsGame,
    start: &MonsGame,
    perspective: Color,
    config: TurnEngineConfig,
) -> TurnEngineUtility {
    evaluate_state_utility(game, start, perspective, config)
}

pub(crate) fn turn_engine_evaluate_plan_with_replies(
    root: &MonsGame,
    plan: &TurnPlan,
    perspective: Color,
    config: TurnEngineConfig,
) -> TurnEngineUtility {
    evaluate_plan_with_replies(root, plan, perspective, config)
}

#[cfg(test)]
pub(crate) fn turn_engine_probe(
    game: &MonsGame,
    perspective: Color,
    mode: TurnEngineMode,
    config: TurnEngineConfig,
) -> TurnEngineProbe {
    if game.active_color != perspective {
        return TurnEngineProbe {
            status: TurnEngineProbeStatus::InactiveColor,
            cached_step: None,
            candidate_family: None,
            candidate_chunk: None,
            chunk_count: 0,
        };
    }

    let cached_step = cached_step_if_legal(game, mode);
    match build_best_plan(game, perspective, config) {
        Ok(Some(plan)) => TurnEngineProbe {
            status: TurnEngineProbeStatus::Planned,
            cached_step,
            candidate_family: Some(plan.family),
            candidate_chunk: plan.compiled_chunks.first().cloned(),
            chunk_count: plan.compiled_chunks.len(),
        },
        Ok(None) | Err(PlanBuildStatus::NoPlan) => TurnEngineProbe {
            status: if cached_step.is_some() {
                TurnEngineProbeStatus::CachedOnly
            } else {
                TurnEngineProbeStatus::NoPlan
            },
            cached_step,
            candidate_family: None,
            candidate_chunk: None,
            chunk_count: 0,
        },
        Err(PlanBuildStatus::BudgetExceeded) => TurnEngineProbe {
            status: TurnEngineProbeStatus::BudgetExceeded,
            cached_step,
            candidate_family: None,
            candidate_chunk: None,
            chunk_count: 0,
        },
    }
}

fn build_best_plan(
    game: &MonsGame,
    perspective: Color,
    config: TurnEngineConfig,
) -> Result<Option<TurnPlan>, PlanBuildStatus> {
    let plans = match generate_turn_plans(
        game,
        perspective,
        config,
        config.own_seed_cap.max(1),
        config.own_beam.max(1),
        config.step_cap.max(1),
        config.expansion_cap.max(1),
    ) {
        Ok(plans) if !plans.is_empty() => plans,
        Ok(_) | Err(PlanBuildStatus::NoPlan) => {
            return Ok(fallback_single_action_plan(game, perspective, config));
        }
        Err(PlanBuildStatus::BudgetExceeded) => return Err(PlanBuildStatus::BudgetExceeded),
    };

    let mut best_plan: Option<TurnPlan> = None;
    for mut plan in plans {
        plan.utility = evaluate_plan_with_replies(game, &plan, perspective, config);
        let replace = best_plan.as_ref().map_or(true, |current| {
            compare_plans(&plan, current) == Ordering::Greater
        });
        if replace {
            best_plan = Some(plan);
        }
    }

    if let Some(best_plan) = best_plan.as_ref() {
        update_turn_engine_diagnostics(|diagnostics| diagnostics.accepted_plans += 1);
        record_accepted_plan_family(best_plan.family);
    }
    Ok(best_plan)
}

fn fallback_single_action_plan(
    game: &MonsGame,
    perspective: Color,
    config: TurnEngineConfig,
) -> Option<TurnPlan> {
    let mut seeds = generate_action_seeds(
        game,
        perspective,
        config,
        config
            .own_seed_cap
            .max(1)
            .saturating_mul(2)
            .min(TURN_ENGINE_COMPILE_LIMIT_MAX),
    );
    if seeds.is_empty() {
        seeds = fallback_walk_seeds(game, perspective);
    }
    if seeds.is_empty() {
        return None;
    }

    let mut compile_pool = TransitionCompilePool::new(game, seeds.as_slice(), config);
    let mut best_plan: Option<TurnPlan> = None;
    for seed in seeds {
        let Some((after, chunk)) =
            compile_action_from_pool(game, perspective, seed.action, &mut compile_pool)
        else {
            continue;
        };
        let mut plan = TurnPlan {
            actions: vec![seed.action],
            compiled_chunks: vec![chunk],
            end_game: after.clone_for_simulation(),
            end_snapshot: TurnSnapshot::from_game(&after),
            utility: evaluate_state_utility(&after, game, perspective, config),
            family: seed.family,
        };
        plan.utility = evaluate_plan_with_replies(game, &plan, perspective, config);
        let replace = best_plan.as_ref().map_or(true, |current| {
            compare_plans(&plan, current) == Ordering::Greater
        });
        if replace {
            best_plan = Some(plan);
        }
    }
    best_plan
}

fn compare_plans(left: &TurnPlan, right: &TurnPlan) -> Ordering {
    left.utility
        .cmp(&right.utility)
        .then_with(|| family_rank(right.family).cmp(&family_rank(left.family)))
        .then_with(|| right.actions.len().cmp(&left.actions.len()))
        .then_with(|| left.compiled_chunks.cmp(&right.compiled_chunks))
}

fn generate_turn_plans(
    game: &MonsGame,
    perspective: Color,
    config: TurnEngineConfig,
    seed_cap: usize,
    beam_width: usize,
    step_cap: usize,
    expansion_cap: usize,
) -> Result<Vec<TurnPlan>, PlanBuildStatus> {
    let mut expansions = 0usize;
    let mut frontier = Vec::new();
    let seeds = generate_action_seeds(game, perspective, config, seed_cap);
    if seeds.is_empty() {
        return Err(PlanBuildStatus::NoPlan);
    }
    let mut compile_pool = TransitionCompilePool::new(game, seeds.as_slice(), config);

    let mut seen = HashMap::<u64, i64>::new();
    for seed in seeds {
        let Some((after, chunk)) =
            compile_action_from_pool(game, perspective, seed.action, &mut compile_pool)
        else {
            continue;
        };
        expansions += 1;
        if expansions > expansion_cap {
            return Err(PlanBuildStatus::BudgetExceeded);
        }
        let order = quick_order_score(game, &after, perspective, seed.family, 1, config);
        let snapshot = TurnSnapshot::from_game(&after);
        let should_keep = seen
            .get(&snapshot.state_hash)
            .map_or(true, |existing| order > *existing);
        if !should_keep {
            continue;
        }
        seen.insert(snapshot.state_hash, order);
        frontier.push((
            order,
            PlanNode {
                game: after,
                actions: vec![seed.action],
                compiled_chunks: vec![chunk],
                family: seed.family,
            },
        ));
    }

    if frontier.is_empty() {
        return Err(PlanBuildStatus::NoPlan);
    }

    frontier.sort_by(|a, b| {
        b.0.cmp(&a.0).then_with(|| {
            compare_chunks(
                a.1.compiled_chunks.as_slice(),
                b.1.compiled_chunks.as_slice(),
            )
        })
    });
    let mut frontier = frontier
        .into_iter()
        .take(beam_width.max(1))
        .map(|(_, node)| node)
        .collect::<Vec<_>>();
    let mut terminal = Vec::new();

    for _ in 1..step_cap.max(1) {
        let mut candidates = Vec::<(i64, PlanNode)>::new();
        let mut expanded_any = false;
        let current_frontier = std::mem::take(&mut frontier);

        for node in current_frontier {
            if node.game.winner_color().is_some() || node.game.active_color != perspective {
                terminal.push(node);
                continue;
            }

            let seeds = generate_action_seeds(&node.game, perspective, config, seed_cap);
            if seeds.is_empty() {
                terminal.push(node);
                continue;
            }
            let mut compile_pool = TransitionCompilePool::new(&node.game, seeds.as_slice(), config);
            let mut node_expanded = false;

            for seed in seeds {
                let Some((after, chunk)) = compile_action_from_pool(
                    &node.game,
                    perspective,
                    seed.action,
                    &mut compile_pool,
                ) else {
                    continue;
                };
                expansions += 1;
                if expansions > expansion_cap {
                    return Err(PlanBuildStatus::BudgetExceeded);
                }
                let mut actions = node.actions.clone();
                actions.push(seed.action);
                let mut compiled_chunks = node.compiled_chunks.clone();
                compiled_chunks.push(chunk);
                let order = quick_order_score(
                    game,
                    &after,
                    perspective,
                    node.family,
                    actions.len(),
                    config,
                );
                let snapshot = TurnSnapshot::from_game(&after);
                let should_keep = seen
                    .get(&snapshot.state_hash)
                    .map_or(true, |existing| order > *existing);
                if !should_keep {
                    continue;
                }
                seen.insert(snapshot.state_hash, order);
                candidates.push((
                    order,
                    PlanNode {
                        game: after,
                        actions,
                        compiled_chunks,
                        family: node.family,
                    },
                ));
                expanded_any = true;
                node_expanded = true;
            }

            if !node_expanded {
                terminal.push(node);
            }
        }

        if !expanded_any || candidates.is_empty() {
            break;
        }

        candidates.sort_by(|a, b| {
            b.0.cmp(&a.0).then_with(|| {
                compare_chunks(
                    a.1.compiled_chunks.as_slice(),
                    b.1.compiled_chunks.as_slice(),
                )
            })
        });
        frontier = candidates
            .into_iter()
            .take(beam_width.max(1))
            .map(|(_, node)| node)
            .collect();
    }

    terminal.extend(frontier);
    if terminal.is_empty() {
        return Err(PlanBuildStatus::NoPlan);
    }

    let mut plans = terminal
        .into_iter()
        .map(|node| TurnPlan {
            actions: node.actions,
            compiled_chunks: node.compiled_chunks,
            end_game: node.game.clone_for_simulation(),
            end_snapshot: TurnSnapshot::from_game(&node.game),
            utility: evaluate_state_utility(&node.game, game, perspective, config),
            family: node.family,
        })
        .collect::<Vec<_>>();
    plans.sort_by(|a, b| compare_plans(b, a));
    Ok(plans)
}

fn evaluate_plan_with_replies(
    root: &MonsGame,
    plan: &TurnPlan,
    perspective: Color,
    config: TurnEngineConfig,
) -> TurnEngineUtility {
    let after = &plan.end_game;

    if after.winner_color().is_some() || after.active_color != perspective.other() {
        return evaluate_state_utility(after, root, perspective, config);
    }

    update_turn_engine_diagnostics(|diagnostics| diagnostics.reply_search_calls += 1);
    let opponent_config = TurnEngineConfig {
        own_seed_cap: config.opponent_seed_cap.max(1),
        own_beam: config.opponent_beam.max(1),
        per_node_family_cap: config.per_node_family_cap.max(1),
        step_cap: config.step_cap.min(4).max(1),
        opponent_seed_cap: config.reply_seed_cap.max(1),
        opponent_beam: config.reply_beam.max(1),
        reply_seed_cap: 0,
        reply_beam: 0,
        expansion_cap: (config.expansion_cap / 2).max(24),
        enable_spirit_family: config.enable_spirit_family,
        scoring_weights: config.scoring_weights,
        allow_exact_static_evaluation: config.allow_exact_static_evaluation,
    };
    let opponent_plans = match generate_turn_plans(
        &after,
        perspective.other(),
        opponent_config,
        opponent_config.own_seed_cap,
        opponent_config.own_beam,
        opponent_config.step_cap,
        opponent_config.expansion_cap,
    ) {
        Ok(plans) if !plans.is_empty() => plans,
        _ => return evaluate_state_utility(after, root, perspective, config),
    };

    let opponent_shortlist = reply_shortlist_len(opponent_plans.len(), opponent_config.own_beam);
    let mut best_opponent = &opponent_plans[0];
    let mut best_opponent_utility = evaluate_state_utility(
        &best_opponent.end_game,
        after,
        perspective.other(),
        opponent_config,
    );
    for opponent_plan in opponent_plans.iter().take(opponent_shortlist).skip(1) {
        let utility = evaluate_state_utility(
            &opponent_plan.end_game,
            after,
            perspective.other(),
            opponent_config,
        );
        if utility > best_opponent_utility
            || (utility == best_opponent_utility
                && compare_chunks(
                    opponent_plan.compiled_chunks.as_slice(),
                    best_opponent.compiled_chunks.as_slice(),
                ) == Ordering::Less)
        {
            best_opponent = opponent_plan;
            best_opponent_utility = utility;
        }
    }

    let after_opponent = &best_opponent.end_game;

    if after_opponent.winner_color().is_some()
        || after_opponent.active_color != perspective
        || config.reply_seed_cap == 0
    {
        return evaluate_state_utility(after_opponent, root, perspective, config);
    }

    let reply_config = TurnEngineConfig {
        own_seed_cap: config.reply_seed_cap.max(1),
        own_beam: config.reply_beam.max(1),
        per_node_family_cap: config.per_node_family_cap.max(1),
        step_cap: config.step_cap.min(3).max(1),
        opponent_seed_cap: 0,
        opponent_beam: 0,
        reply_seed_cap: 0,
        reply_beam: 0,
        expansion_cap: (config.expansion_cap / 3).max(16),
        enable_spirit_family: config.enable_spirit_family,
        scoring_weights: config.scoring_weights,
        allow_exact_static_evaluation: config.allow_exact_static_evaluation,
    };
    let reply_plans = match generate_turn_plans(
        &after_opponent,
        perspective,
        reply_config,
        reply_config.own_seed_cap,
        reply_config.own_beam,
        reply_config.step_cap,
        reply_config.expansion_cap,
    ) {
        Ok(plans) if !plans.is_empty() => plans,
        _ => return evaluate_state_utility(after_opponent, root, perspective, config),
    };
    let reply_shortlist = reply_shortlist_len(reply_plans.len(), reply_config.own_beam);
    reply_plans
        .into_iter()
        .take(reply_shortlist)
        .map(|plan| evaluate_state_utility(&plan.end_game, root, perspective, config))
        .max()
        .unwrap_or_else(|| evaluate_state_utility(after_opponent, root, perspective, config))
}

fn reply_shortlist_len(total: usize, beam: usize) -> usize {
    total.min(beam.saturating_mul(2).max(4).min(8))
}

fn evaluate_state_utility(
    game: &MonsGame,
    start: &MonsGame,
    perspective: Color,
    config: TurnEngineConfig,
) -> TurnEngineUtility {
    let my_score = score_for_color(game, perspective);
    let start_score = score_for_color(start, perspective);
    let score_delta = my_score.saturating_sub(start_score);
    let strategic = exact_strategic_analysis(game).color_summary(perspective);
    let path_bonus = strategic
        .score_path_window
        .best_steps
        .map(|steps| (Config::BOARD_SIZE * 3 - steps).max(0) * 22)
        .unwrap_or(0);
    let immediate_bonus = strategic.immediate_window.best_score.saturating_mul(110)
        + strategic.immediate_window.multi_pressure.saturating_mul(18);
    let safe_supermana_bonus =
        if own_drainer_carries_safe_mana(&game.board, perspective, Mana::Supermana) {
            380
        } else {
            0
        };
    let safe_opponent_mana_bonus = if own_drainer_carries_safe_mana(
        &game.board,
        perspective,
        Mana::Regular(perspective.other()),
    ) {
        300
    } else {
        0
    };
    let opponent = perspective.other();
    let opponent_window_before = exact_turn_summary(start, opponent).same_turn_score_window_value;
    let opponent_window_after = if game.active_color == opponent {
        exact_turn_summary(game, opponent).same_turn_score_window_value
    } else {
        exact_strategic_analysis(game)
            .color_summary(opponent)
            .immediate_window
            .best_score
    };
    let deny_gain = opponent_window_before.saturating_sub(opponent_window_after);
    let drainer_safety = own_drainer_safety_score(&game.board, perspective);
    let unsafe_progress_penalty = if drainer_safety < 0 {
        drainer_safety.saturating_abs().saturating_mul(900)
    } else {
        0
    };
    let opponent_needed_before =
        Config::TARGET_SCORE.saturating_sub(score_for_color(start, opponent));
    let opponent_needed_after =
        Config::TARGET_SCORE.saturating_sub(score_for_color(game, opponent));
    let denied_immediate_window = opponent_needed_before > 0
        && opponent_window_before >= opponent_needed_before
        && (opponent_needed_after <= 0 || opponent_window_after < opponent_needed_after);
    let drainer_attack = if find_awake_drainer_location(&game.board, opponent).is_none() {
        1
    } else {
        0
    };
    let eval_score = evaluate_preferability_with_weights_and_exact_policy(
        game,
        perspective,
        config.scoring_weights,
        config.allow_exact_static_evaluation,
    );
    TurnEngineUtility {
        win_state: winner_state(game, perspective),
        avoid_immediate_loss: if opponent_can_win_immediately(game, perspective) {
            -1
        } else {
            1
        },
        score_delta: score_delta
            .saturating_mul(2_400)
            .saturating_add(path_bonus)
            .saturating_add(immediate_bonus)
            .saturating_add(safe_supermana_bonus)
            .saturating_add(safe_opponent_mana_bonus)
            .saturating_sub(unsafe_progress_penalty),
        deny_gain: deny_gain
            .saturating_mul(220)
            .saturating_add(if denied_immediate_window { 1_500 } else { 0 }),
        drainer_attack,
        drainer_safety,
        eval_score,
    }
}

fn quick_order_score(
    root: &MonsGame,
    game: &MonsGame,
    perspective: Color,
    family: TurnPlanFamily,
    step_len: usize,
    config: TurnEngineConfig,
) -> i64 {
    let utility = evaluate_state_utility(game, root, perspective, config);
    let family_bonus = match family {
        TurnPlanFamily::ImmediateScore => 1_000,
        TurnPlanFamily::DenyOpponentWindow => 960,
        TurnPlanFamily::DrainerKill => 920,
        TurnPlanFamily::DrainerSafetyRecovery => 860,
        TurnPlanFamily::SpiritImpact => 820,
        TurnPlanFamily::SafeSupermanaProgress => 760,
        TurnPlanFamily::SafeOpponentManaProgress => 720,
        TurnPlanFamily::ManaTempo => 560,
    };
    i64::from(utility.win_state) * 10_000_000
        + i64::from(utility.avoid_immediate_loss) * 5_000_000
        + i64::from(utility.score_delta)
        + i64::from(utility.deny_gain)
        + i64::from(utility.drainer_attack) * 3_500
        + i64::from(utility.drainer_safety) * 2_200
        + i64::from(utility.eval_score / 8)
        + i64::from(family_bonus) * 2_000
        - step_len as i64 * 350
}

fn generate_action_seeds(
    game: &MonsGame,
    perspective: Color,
    config: TurnEngineConfig,
    seed_cap: usize,
) -> Vec<ActionSeed> {
    if game.active_color != perspective {
        return Vec::new();
    }

    let mut seeds = Vec::new();
    seeds.extend(immediate_score_seeds(game, perspective));
    seeds.extend(deny_window_seeds(game, perspective));
    seeds.extend(drainer_kill_seeds(game, perspective));
    seeds.extend(safe_supermana_progress_seeds(game, perspective));
    seeds.extend(safe_opponent_mana_progress_seeds(game, perspective));
    seeds.extend(safety_recovery_seeds(game, perspective));
    seeds.extend(oracle_walk_seeds(game, perspective));
    seeds.extend(spirit_impact_seeds(game, perspective, config));
    seeds.extend(mana_tempo_seeds(game, perspective));

    let mut dedup = HashSet::<TurnAction>::new();
    let mut per_family = HashMap::<TurnPlanFamily, Vec<ActionSeed>>::new();
    for seed in seeds {
        per_family.entry(seed.family).or_default().push(seed);
    }
    for family_seeds in per_family.values_mut() {
        family_seeds.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| action_key(a.action).cmp(&action_key(b.action)))
        });
    }

    let family_order = [
        TurnPlanFamily::ImmediateScore,
        TurnPlanFamily::DenyOpponentWindow,
        TurnPlanFamily::DrainerKill,
        TurnPlanFamily::DrainerSafetyRecovery,
        TurnPlanFamily::SpiritImpact,
        TurnPlanFamily::SafeSupermanaProgress,
        TurnPlanFamily::SafeOpponentManaProgress,
        TurnPlanFamily::ManaTempo,
    ];
    let mut filtered = Vec::new();
    let mut family_indices = HashMap::<TurnPlanFamily, usize>::new();
    for _round in 0..config.per_node_family_cap.max(1) {
        let mut added_any = false;
        for family in family_order {
            let Some(family_seeds) = per_family.get(&family) else {
                continue;
            };
            let start_index = *family_indices.get(&family).unwrap_or(&0);
            let mut selected = None;
            for (offset, seed) in family_seeds.iter().enumerate().skip(start_index) {
                if dedup.insert(seed.action) {
                    selected = Some((offset + 1, seed.clone()));
                    break;
                }
            }
            if let Some((next_index, seed)) = selected {
                family_indices.insert(family, next_index);
                filtered.push(seed);
                added_any = true;
                if filtered.len() >= seed_cap.max(1) {
                    return filtered;
                }
            } else {
                family_indices.insert(family, family_seeds.len());
            }
        }
        if !added_any {
            break;
        }
    }
    filtered
}

fn family_rank(family: TurnPlanFamily) -> i32 {
    match family {
        TurnPlanFamily::ImmediateScore => 0,
        TurnPlanFamily::DenyOpponentWindow => 1,
        TurnPlanFamily::DrainerKill => 2,
        TurnPlanFamily::DrainerSafetyRecovery => 3,
        TurnPlanFamily::SpiritImpact => 4,
        TurnPlanFamily::SafeSupermanaProgress => 5,
        TurnPlanFamily::SafeOpponentManaProgress => 6,
        TurnPlanFamily::ManaTempo => 7,
    }
}

fn action_key(action: TurnAction) -> (i32, Location, Option<Location>, Option<Location>) {
    match action {
        TurnAction::Walk { actor, to } => (0, actor, Some(to), None),
        TurnAction::Attack { actor, target } => (1, actor, Some(target), None),
        TurnAction::SpiritShift {
            actor,
            target,
            destination,
        } => (2, actor, Some(target), Some(destination)),
        TurnAction::Bomb { actor, target } => (3, actor, Some(target), None),
        TurnAction::MoveMana { from, to } => (4, from, Some(to), None),
        TurnAction::ScoreCarry { actor, step, .. } => (5, actor, Some(step), None),
        TurnAction::SafetyRetreat { actor, to } => (6, actor, Some(to), None),
    }
}

fn immediate_score_seeds(game: &MonsGame, perspective: Color) -> Vec<ActionSeed> {
    let mut seeds = Vec::new();
    for (location, item) in game.board.occupied() {
        let Item::MonWithMana { mon, mana } = item else {
            continue;
        };
        if mon.color != perspective || mon.is_fainted() {
            continue;
        }
        let before_dist = distance_to_nearest_pool(location, perspective);
        for &next in location.nearby_locations_ref() {
            let after_dist = distance_to_nearest_pool(next, perspective);
            if after_dist > before_dist {
                continue;
            }
            let priority = 9_800
                + before_dist.saturating_sub(after_dist).saturating_mul(180)
                + mana.score(perspective).saturating_mul(120);
            seeds.push(ActionSeed {
                family: TurnPlanFamily::ImmediateScore,
                action: TurnAction::ScoreCarry {
                    actor: location,
                    wanted: *mana,
                    step: next,
                },
                priority,
            });
        }
    }
    update_turn_engine_diagnostics(|diagnostics| diagnostics.seed_immediate_score += seeds.len());
    seeds
}

fn deny_window_seeds(game: &MonsGame, perspective: Color) -> Vec<ActionSeed> {
    let opponent = perspective.other();
    let deny_pressure = exact_turn_summary(game, opponent).same_turn_score_window_value;
    if deny_pressure <= 0 && !opponent_can_win_immediately(game, perspective) {
        return Vec::new();
    }

    let mut seeds = attack_family_seeds(
        game,
        perspective,
        TurnPlanFamily::DenyOpponentWindow,
        9_400 + deny_pressure.saturating_mul(240),
    );
    if let Some(drainer) = find_awake_drainer_location(&game.board, perspective) {
        for &next in drainer.nearby_locations_ref() {
            let before_safety = own_drainer_safety_score(&game.board, perspective);
            let before_dist = distance_to_nearest_pool(drainer, perspective);
            let after_dist = distance_to_nearest_pool(next, perspective);
            if after_dist > before_dist.saturating_add(1) && before_safety >= 0 {
                continue;
            }
            seeds.push(ActionSeed {
                family: TurnPlanFamily::DenyOpponentWindow,
                action: TurnAction::SafetyRetreat {
                    actor: drainer,
                    to: next,
                },
                priority: 9_100 + before_safety.saturating_abs().saturating_mul(220),
            });
        }
    }
    update_turn_engine_diagnostics(|diagnostics| diagnostics.seed_deny_window += seeds.len());
    seeds
}

fn drainer_kill_seeds(game: &MonsGame, perspective: Color) -> Vec<ActionSeed> {
    let Some(target) = find_awake_drainer_location(&game.board, perspective.other()) else {
        return Vec::new();
    };
    if !opponent_drainer_kill_is_high_value(game, perspective, target) {
        return Vec::new();
    }
    let seeds = attack_family_seeds(game, perspective, TurnPlanFamily::DrainerKill, 9_000);
    update_turn_engine_diagnostics(|diagnostics| diagnostics.seed_drainer_kill += seeds.len());
    seeds
}

fn attack_family_seeds(
    game: &MonsGame,
    perspective: Color,
    family: TurnPlanFamily,
    base_priority: i32,
) -> Vec<ActionSeed> {
    let Some(target) = find_awake_drainer_location(&game.board, perspective.other()) else {
        return Vec::new();
    };
    let mut seeds = Vec::new();
    let can_use_action = game.player_can_use_action();
    let remaining_moves = remaining_moves_for_color(game, perspective);
    for (location, item) in game.board.occupied() {
        match item {
            Item::Mon { mon }
            | Item::MonWithMana { mon, .. }
            | Item::MonWithConsumable { mon, .. } => {
                if mon.color != perspective || mon.is_fainted() {
                    continue;
                }
            }
            Item::Mana { .. } | Item::Consumable { .. } => continue,
        }

        let can_attack = can_use_action && actor_can_attack_from_item(item);
        let can_bomb = can_use_action && actor_can_bomb_from_item(item);

        if can_attack
            && actor_can_attack_target_now(&game.board, location, target, item, perspective)
        {
            seeds.push(ActionSeed {
                family,
                action: TurnAction::Attack {
                    actor: location,
                    target,
                },
                priority: base_priority,
            });
        }

        if can_bomb && actor_can_bomb_target_now(&game.board, location, target, item, perspective) {
            seeds.push(ActionSeed {
                family,
                action: TurnAction::Bomb {
                    actor: location,
                    target,
                },
                priority: base_priority.saturating_sub(80),
            });
        }

        if remaining_moves <= 0 || !(can_attack || can_bomb) {
            continue;
        }
        for &next in location.nearby_locations_ref() {
            if next.distance(&target) >= location.distance(&target) {
                continue;
            }
            if family == TurnPlanFamily::DrainerKill {
                let Some(mon) = item.mon().copied() else {
                    continue;
                };
                let moved_item = match item {
                    Item::Mon { .. } => Item::Mon { mon },
                    Item::MonWithMana { mana, .. } => Item::MonWithMana { mon, mana: *mana },
                    Item::MonWithConsumable { consumable, .. } => Item::MonWithConsumable {
                        mon,
                        consumable: *consumable,
                    },
                    Item::Mana { .. } | Item::Consumable { .. } => continue,
                };
                let mut preview = game.board.clone();
                preview.remove_item(location);
                preview.put(moved_item, next);
                let threatens_now = (can_attack
                    && actor_can_attack_target_now(
                        &preview,
                        next,
                        target,
                        &moved_item,
                        perspective,
                    ))
                    || (can_bomb
                        && actor_can_bomb_target_now(
                            &preview,
                            next,
                            target,
                            &moved_item,
                            perspective,
                        ));
                if !threatens_now {
                    continue;
                }
            }
            seeds.push(ActionSeed {
                family,
                action: TurnAction::Walk {
                    actor: location,
                    to: next,
                },
                priority: base_priority.saturating_sub(200).saturating_add(
                    location
                        .distance(&target)
                        .saturating_sub(next.distance(&target))
                        * 80,
                ),
            });
        }
    }
    seeds
}

fn safe_supermana_progress_seeds(game: &MonsGame, perspective: Color) -> Vec<ActionSeed> {
    let seeds = safe_progress_seeds(
        game,
        perspective,
        Mana::Supermana,
        TurnPlanFamily::SafeSupermanaProgress,
        8_900,
    );
    update_turn_engine_diagnostics(|diagnostics| {
        diagnostics.seed_safe_supermana_progress += seeds.len()
    });
    seeds
}

fn safe_opponent_mana_progress_seeds(game: &MonsGame, perspective: Color) -> Vec<ActionSeed> {
    let seeds = safe_progress_seeds(
        game,
        perspective,
        Mana::Regular(perspective.other()),
        TurnPlanFamily::SafeOpponentManaProgress,
        8_600,
    );
    update_turn_engine_diagnostics(|diagnostics| {
        diagnostics.seed_safe_opponent_mana_progress += seeds.len()
    });
    seeds
}

fn safe_progress_seeds(
    game: &MonsGame,
    perspective: Color,
    wanted: Mana,
    family: TurnPlanFamily,
    base_priority: i32,
) -> Vec<ActionSeed> {
    let Some(drainer) = find_awake_drainer_location(&game.board, perspective) else {
        return Vec::new();
    };
    let mut seeds = Vec::new();
    let before_turn = exact_turn_summary(game, perspective);
    let before_safety = own_drainer_safety_score(&game.board, perspective);
    if let Some(path) = exact_secure_specific_mana_path_from(game, perspective, drainer, wanted) {
        if let Some(step) = path.first().copied() {
            seeds.push(ActionSeed {
                family,
                action: TurnAction::ScoreCarry {
                    actor: drainer,
                    wanted,
                    step,
                },
                priority: base_priority
                    .saturating_add((Config::BOARD_SIZE * 2 - path.len() as i32).max(0) * 120),
            });
        }
    }
    if remaining_moves_for_color(game, perspective) > 0 {
        if let Some(target_mana) = nearest_wanted_mana_location(&game.board, wanted) {
            let before_dist = drainer.distance(&target_mana);
            let before_exact_steps = wanted_progress_steps(before_turn, wanted, perspective)
                .unwrap_or(Config::BOARD_SIZE * 3);
            let before_score_path = before_turn
                .score_path_best_steps
                .unwrap_or(Config::BOARD_SIZE * 3);
            for &next in drainer.nearby_locations_ref() {
                if !walk_destination_plausible(&game.board, drainer, next) {
                    continue;
                }
                let Some((after, _)) = MonsGameModel::apply_inputs_for_search_with_events(
                    game,
                    &[Input::Location(drainer), Input::Location(next)],
                ) else {
                    continue;
                };
                if opponent_can_win_immediately(&after, perspective) {
                    continue;
                }
                let after_turn = exact_turn_summary(&after, perspective);
                let after_safety = own_drainer_safety_score(&after.board, perspective);
                let after_exact_steps = wanted_progress_steps(after_turn, wanted, perspective)
                    .unwrap_or(Config::BOARD_SIZE * 3);
                let after_score_path = after_turn
                    .score_path_best_steps
                    .unwrap_or(Config::BOARD_SIZE * 3);
                let exact_improved = after_exact_steps < before_exact_steps
                    || (after_exact_steps <= before_exact_steps
                        && after_score_path < before_score_path);
                if !exact_improved && after_safety < before_safety {
                    continue;
                }
                let mut priority = base_priority
                    .saturating_sub(180)
                    .saturating_add(
                        before_dist
                            .saturating_sub(next.distance(&target_mana))
                            .max(0)
                            * 110,
                    )
                    .saturating_add(after_safety.saturating_sub(before_safety) * 120);
                if exact_improved {
                    priority = priority.saturating_add(
                        before_exact_steps
                            .saturating_sub(after_exact_steps)
                            .saturating_mul(220),
                    );
                    priority = priority.saturating_add(
                        before_score_path
                            .saturating_sub(after_score_path)
                            .saturating_mul(180),
                    );
                }
                if wanted == Mana::Supermana && after_turn.same_turn_score_window_value > 0 {
                    priority = priority.saturating_add(
                        after_turn.same_turn_score_window_value.saturating_mul(260),
                    );
                }
                seeds.push(ActionSeed {
                    family,
                    action: TurnAction::Walk {
                        actor: drainer,
                        to: next,
                    },
                    priority,
                });
            }
        }
    }
    if let Some(Item::MonWithMana { mana, .. }) = game.board.item(drainer) {
        if *mana == wanted {
            let before_dist = distance_to_nearest_pool(drainer, perspective);
            for &next in drainer.nearby_locations_ref() {
                let after_dist = distance_to_nearest_pool(next, perspective);
                if after_dist > before_dist {
                    continue;
                }
                seeds.push(ActionSeed {
                    family,
                    action: TurnAction::ScoreCarry {
                        actor: drainer,
                        wanted,
                        step: next,
                    },
                    priority: base_priority
                        .saturating_add(before_dist.saturating_sub(after_dist).saturating_mul(150)),
                });
            }
        }
    }
    seeds
}

fn safety_recovery_seeds(game: &MonsGame, perspective: Color) -> Vec<ActionSeed> {
    let Some(drainer) = find_awake_drainer_location(&game.board, perspective) else {
        return Vec::new();
    };
    let before_safety = own_drainer_safety_score(&game.board, perspective);

    let mut seeds = Vec::new();
    for &next in drainer.nearby_locations_ref() {
        let Some((after, _)) = MonsGameModel::apply_inputs_for_search_with_events(
            game,
            &[Input::Location(drainer), Input::Location(next)],
        ) else {
            continue;
        };
        let safety_after = own_drainer_safety_score(&after.board, perspective);
        if safety_after <= before_safety {
            continue;
        }
        seeds.push(ActionSeed {
            family: TurnPlanFamily::DrainerSafetyRecovery,
            action: TurnAction::SafetyRetreat {
                actor: drainer,
                to: next,
            },
            priority: 8_300
                + before_safety.saturating_abs().saturating_mul(220)
                + safety_after
                    .saturating_sub(before_safety)
                    .saturating_mul(260),
        });
    }
    update_turn_engine_diagnostics(|diagnostics| diagnostics.seed_safety_recovery += seeds.len());
    seeds
}

fn fallback_walk_seeds(game: &MonsGame, perspective: Color) -> Vec<ActionSeed> {
    if remaining_moves_for_color(game, perspective) <= 0 {
        return Vec::new();
    }

    let mut seeds = Vec::new();
    let before_safety = own_drainer_safety_score(&game.board, perspective);
    if let Some(drainer) = find_awake_drainer_location(&game.board, perspective) {
        let before_pool_dist = distance_to_nearest_pool(drainer, perspective);
        for &next in drainer.nearby_locations_ref() {
            if !walk_destination_plausible(&game.board, drainer, next) {
                continue;
            }
            let Some((after, _)) = MonsGameModel::apply_inputs_for_search_with_events(
                game,
                &[Input::Location(drainer), Input::Location(next)],
            ) else {
                continue;
            };
            if opponent_can_win_immediately(&after, perspective) {
                continue;
            }
            let after_safety = own_drainer_safety_score(&after.board, perspective);
            if after_safety < before_safety {
                continue;
            }
            let after_pool_dist = distance_to_nearest_pool(next, perspective);
            let family = if after_safety > before_safety {
                TurnPlanFamily::DrainerSafetyRecovery
            } else {
                TurnPlanFamily::ManaTempo
            };
            let priority = 7_200
                + before_pool_dist
                    .saturating_sub(after_pool_dist)
                    .max(0)
                    .saturating_mul(140)
                + after_safety
                    .saturating_sub(before_safety)
                    .saturating_mul(240);
            seeds.push(ActionSeed {
                family,
                action: TurnAction::Walk {
                    actor: drainer,
                    to: next,
                },
                priority,
            });
        }
    }

    if seeds.is_empty() {
        for (actor, item) in game.board.occupied() {
            let Some(mon) = item.mon().copied() else {
                continue;
            };
            if mon.color != perspective || mon.is_fainted() {
                continue;
            }
            for &to in actor.nearby_locations_ref() {
                if !walk_destination_plausible(&game.board, actor, to) {
                    continue;
                }
                let Some((after, _)) = MonsGameModel::apply_inputs_for_search_with_events(
                    game,
                    &[Input::Location(actor), Input::Location(to)],
                ) else {
                    continue;
                };
                if opponent_can_win_immediately(&after, perspective) {
                    continue;
                }
                seeds.push(ActionSeed {
                    family: TurnPlanFamily::ManaTempo,
                    action: TurnAction::Walk { actor, to },
                    priority: 6_800,
                });
            }
        }
    }

    update_turn_engine_diagnostics(|diagnostics| diagnostics.seed_mana_tempo += seeds.len());
    seeds
}

fn oracle_walk_seeds(game: &MonsGame, perspective: Color) -> Vec<ActionSeed> {
    if remaining_moves_for_color(game, perspective) <= 0 {
        return Vec::new();
    }

    let before_turn = exact_turn_summary(game, perspective);
    let before_safety = own_drainer_safety_score(&game.board, perspective);
    let before_super_steps = before_turn
        .safe_supermana_progress_steps
        .unwrap_or(Config::BOARD_SIZE * 3);
    let before_opponent_steps = before_turn
        .safe_opponent_mana_progress_steps
        .unwrap_or(Config::BOARD_SIZE * 3);
    let mut seeds = Vec::new();

    for (actor, item) in game.board.occupied() {
        let Some(mon) = item.mon().copied() else {
            continue;
        };
        if mon.color != perspective || mon.is_fainted() {
            continue;
        }
        for &to in actor.nearby_locations_ref() {
            if !walk_destination_plausible(&game.board, actor, to) {
                continue;
            }
            let Some((after, _)) = MonsGameModel::apply_inputs_for_search_with_events(
                game,
                &[Input::Location(actor), Input::Location(to)],
            ) else {
                continue;
            };
            if opponent_can_win_immediately(&after, perspective) {
                continue;
            }

            let after_turn = exact_turn_summary(&after, perspective);
            let after_safety = own_drainer_safety_score(&after.board, perspective);
            let after_super_steps = after_turn
                .safe_supermana_progress_steps
                .unwrap_or(Config::BOARD_SIZE * 3);
            let after_opponent_steps = after_turn
                .safe_opponent_mana_progress_steps
                .unwrap_or(Config::BOARD_SIZE * 3);

            if after_super_steps < before_super_steps {
                seeds.push(ActionSeed {
                    family: TurnPlanFamily::SafeSupermanaProgress,
                    action: TurnAction::Walk { actor, to },
                    priority: 8_250
                        + before_super_steps
                            .saturating_sub(after_super_steps)
                            .saturating_mul(240)
                        + after_safety
                            .saturating_sub(before_safety)
                            .saturating_mul(100)
                        + after_turn.same_turn_score_window_value.saturating_mul(160),
                });
            }

            let opponent_progress_improved = after_opponent_steps < before_opponent_steps;
            let spirit_denial_improved =
                after_turn.spirit_assisted_denial_value > before_turn.spirit_assisted_denial_value;
            if opponent_progress_improved || spirit_denial_improved {
                let family = if mon.kind == MonKind::Spirit {
                    TurnPlanFamily::SpiritImpact
                } else {
                    TurnPlanFamily::SafeOpponentManaProgress
                };
                let mut priority = 8_000;
                if opponent_progress_improved {
                    priority += before_opponent_steps
                        .saturating_sub(after_opponent_steps)
                        .saturating_mul(240);
                }
                if spirit_denial_improved {
                    priority += after_turn
                        .spirit_assisted_denial_value
                        .saturating_sub(before_turn.spirit_assisted_denial_value)
                        .saturating_mul(180);
                }
                seeds.push(ActionSeed {
                    family,
                    action: TurnAction::Walk { actor, to },
                    priority,
                });
            }

            let spirit_score_improved = after_turn.spirit_assisted_score_value
                > before_turn.spirit_assisted_score_value
                || after_turn.same_turn_score_window_value
                    > before_turn.same_turn_score_window_value;
            if mon.kind == MonKind::Spirit && spirit_score_improved {
                seeds.push(ActionSeed {
                    family: TurnPlanFamily::SpiritImpact,
                    action: TurnAction::Walk { actor, to },
                    priority: 8_100
                        + after_turn
                            .spirit_assisted_score_value
                            .saturating_sub(before_turn.spirit_assisted_score_value)
                            .saturating_mul(200)
                        + after_turn
                            .same_turn_score_window_value
                            .saturating_sub(before_turn.same_turn_score_window_value)
                            .saturating_mul(220),
                });
            }

            if after_safety > before_safety {
                seeds.push(ActionSeed {
                    family: TurnPlanFamily::DrainerSafetyRecovery,
                    action: TurnAction::Walk { actor, to },
                    priority: 8_050
                        + after_safety
                            .saturating_sub(before_safety)
                            .saturating_mul(260),
                });
            }
        }
    }

    if !seeds.is_empty() {
        let mut seed_supermana = 0usize;
        let mut seed_opponent = 0usize;
        let mut seed_safety = 0usize;
        let mut seed_spirit = 0usize;
        for seed in seeds.iter() {
            match seed.family {
                TurnPlanFamily::SafeSupermanaProgress => seed_supermana += 1,
                TurnPlanFamily::SafeOpponentManaProgress => seed_opponent += 1,
                TurnPlanFamily::DrainerSafetyRecovery => seed_safety += 1,
                TurnPlanFamily::SpiritImpact => seed_spirit += 1,
                TurnPlanFamily::ImmediateScore
                | TurnPlanFamily::DenyOpponentWindow
                | TurnPlanFamily::DrainerKill
                | TurnPlanFamily::ManaTempo => {}
            }
        }
        update_turn_engine_diagnostics(|diagnostics| {
            diagnostics.seed_safe_supermana_progress += seed_supermana;
            diagnostics.seed_safe_opponent_mana_progress += seed_opponent;
            diagnostics.seed_safety_recovery += seed_safety;
            diagnostics.seed_spirit_impact += seed_spirit;
        });
    }

    seeds
}

fn spirit_impact_seeds(
    game: &MonsGame,
    perspective: Color,
    config: TurnEngineConfig,
) -> Vec<ActionSeed> {
    if !config.enable_spirit_family {
        return Vec::new();
    }
    if !game.player_can_use_action() {
        return Vec::new();
    }
    let mut seeds = Vec::new();
    let before_turn = exact_turn_summary(game, perspective);
    let before_safety = own_drainer_safety_score(&game.board, perspective);
    for (spirit_location, item) in game.board.occupied() {
        let Some(mon) = item.mon().copied() else {
            continue;
        };
        if mon.color != perspective || mon.kind != MonKind::Spirit || mon.is_fainted() {
            continue;
        }
        if matches!(game.board.square(spirit_location), Square::MonBase { .. }) {
            continue;
        }

        for &target in spirit_location.reachable_by_spirit_action_ref() {
            let Some(target_item) = game.board.item(target).copied() else {
                continue;
            };
            if !spirit_target_allowed(target_item) {
                continue;
            }
            for &destination in target.nearby_locations_ref() {
                if !spirit_destination_allowed(&game.board, target_item, destination) {
                    continue;
                }
                let Some((after, _)) = MonsGameModel::apply_inputs_for_search_with_events(
                    game,
                    &[
                        Input::Location(spirit_location),
                        Input::Location(target),
                        Input::Location(destination),
                    ],
                ) else {
                    continue;
                };
                let mut priority = 7_600;
                if let Some(mon) = target_item.mon() {
                    if mon.color == perspective.other() {
                        priority += 400;
                    }
                }
                if matches!(target_item, Item::Mana { mana } if mana == Mana::Supermana) {
                    priority += 600;
                }
                if matches!(target_item, Item::Mana { mana } if mana == Mana::Regular(perspective.other()))
                {
                    priority += 460;
                }
                let after_turn = exact_turn_summary(&after, perspective);
                if after_turn.same_turn_score_window_value
                    > before_turn.same_turn_score_window_value
                {
                    priority += after_turn
                        .same_turn_score_window_value
                        .saturating_sub(before_turn.same_turn_score_window_value)
                        .saturating_mul(280);
                }
                if after_turn.spirit_assisted_score {
                    priority += 900 + after_turn.spirit_assisted_score_value.saturating_mul(120);
                }
                if after_turn.safe_supermana_progress {
                    priority += 700
                        + progress_priority_bonus(
                            before_turn.safe_supermana_progress_steps,
                            after_turn.safe_supermana_progress_steps,
                        );
                }
                if after_turn.safe_opponent_mana_progress {
                    priority += 760
                        + progress_priority_bonus(
                            before_turn.safe_opponent_mana_progress_steps,
                            after_turn.safe_opponent_mana_progress_steps,
                        );
                }
                if after_turn.spirit_assisted_denial {
                    priority += 820 + after_turn.spirit_assisted_denial_value.saturating_mul(140);
                }
                let after_safety = own_drainer_safety_score(&after.board, perspective);
                if after_safety > before_safety {
                    priority += after_safety
                        .saturating_sub(before_safety)
                        .saturating_mul(160);
                }
                priority += (Config::BOARD_SIZE - destination.distance(&target)).max(0) * 20;
                seeds.push(ActionSeed {
                    family: TurnPlanFamily::SpiritImpact,
                    action: TurnAction::SpiritShift {
                        actor: spirit_location,
                        target,
                        destination,
                    },
                    priority,
                });
            }
        }
    }
    seeds.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| action_key(a.action).cmp(&action_key(b.action)))
    });
    seeds.truncate(12);
    update_turn_engine_diagnostics(|diagnostics| diagnostics.seed_spirit_impact += seeds.len());
    seeds
}

fn wanted_progress_steps(turn: ExactTurnSummary, wanted: Mana, perspective: Color) -> Option<i32> {
    match wanted {
        Mana::Supermana => turn.safe_supermana_progress_steps,
        Mana::Regular(color) if color == perspective.other() => {
            turn.safe_opponent_mana_progress_steps
        }
        Mana::Regular(_) => None,
    }
}

fn progress_priority_bonus(before: Option<i32>, after: Option<i32>) -> i32 {
    let before = before.unwrap_or(Config::BOARD_SIZE * 3);
    let after = after.unwrap_or(Config::BOARD_SIZE * 3);
    if after >= before {
        0
    } else {
        before.saturating_sub(after).saturating_mul(220)
    }
}

fn mana_tempo_seeds(game: &MonsGame, perspective: Color) -> Vec<ActionSeed> {
    if !game.player_can_move_mana() {
        return Vec::new();
    }
    if find_awake_drainer_location(&game.board, perspective).is_some() {
        return Vec::new();
    }
    let mut seeds = Vec::new();
    for (from, item) in game.board.occupied() {
        let Item::Mana { mana } = item else {
            continue;
        };
        if *mana != Mana::Regular(perspective) {
            continue;
        }
        for &to in from.nearby_locations_ref() {
            if !mana_move_destination_allowed(&game.board, to) {
                continue;
            }
            let own_before = distance_to_nearest_pool(from, perspective);
            let own_after = distance_to_nearest_pool(to, perspective);
            let opp_before = distance_to_nearest_pool(from, perspective.other());
            let opp_after = distance_to_nearest_pool(to, perspective.other());
            let own_gain = own_before.saturating_sub(own_after);
            let opp_gain = opp_before.saturating_sub(opp_after);
            if own_gain <= 0 || opp_gain > 0 {
                continue;
            }
            seeds.push(ActionSeed {
                family: TurnPlanFamily::ManaTempo,
                action: TurnAction::MoveMana { from, to },
                priority: 6_900 + own_gain.saturating_mul(200)
                    - opp_gain.max(0).saturating_mul(200),
            });
        }
    }
    update_turn_engine_diagnostics(|diagnostics| diagnostics.seed_mana_tempo += seeds.len());
    seeds
}

impl TransitionCompilePool {
    fn new(game: &MonsGame, seeds: &[ActionSeed], config: TurnEngineConfig) -> Self {
        let mut seen = HashSet::new();
        let mut priority_locations = Vec::new();
        for seed in seeds {
            for location in action_priority_locations(seed.action) {
                if seen.insert(location) {
                    priority_locations.push(location);
                }
            }
        }
        let limit = compile_limit_for_config(config);
        let transitions = MonsGameModel::enumerate_legal_transitions_with_priority(
            game,
            limit,
            SuggestedStartInputOptions::for_automove(),
            priority_locations.as_slice(),
        );
        Self {
            transitions,
            limit,
            priority_locations,
        }
    }

    fn expand(&mut self, game: &MonsGame) -> bool {
        if self.transitions.len() < self.limit || self.limit >= TURN_ENGINE_COMPILE_LIMIT_MAX {
            return false;
        }
        let next_limit = (self.limit.saturating_mul(2)).min(TURN_ENGINE_COMPILE_LIMIT_MAX);
        if next_limit <= self.limit {
            return false;
        }
        self.transitions = MonsGameModel::enumerate_legal_transitions_with_priority(
            game,
            next_limit,
            SuggestedStartInputOptions::for_automove(),
            self.priority_locations.as_slice(),
        );
        self.limit = next_limit;
        true
    }
}

fn compile_limit_for_config(config: TurnEngineConfig) -> usize {
    (config
        .own_seed_cap
        .max(config.opponent_seed_cap)
        .saturating_mul(12))
    .clamp(24, 96)
}

fn best_transition_for_action<'a>(
    game: &MonsGame,
    perspective: Color,
    action: TurnAction,
    transitions: &'a [LegalInputTransition],
) -> Option<(i32, usize)> {
    let mut best: Option<(i32, usize)> = None;
    for (index, transition) in transitions.iter().enumerate() {
        if !transition_matches_action(
            game,
            &transition.game,
            transition.events.as_slice(),
            perspective,
            action,
        ) {
            continue;
        }
        let score = transition_score(
            game,
            &transition.game,
            transition.events.as_slice(),
            perspective,
            action,
        );
        if best.as_ref().map_or(true, |(best_score, best_index)| {
            score > *best_score
                || (score == *best_score && transition.inputs < transitions[*best_index].inputs)
        }) {
            best = Some((score, index));
        }
    }
    best
}

fn compile_action_from_pool(
    game: &MonsGame,
    perspective: Color,
    action: TurnAction,
    compile_pool: &mut TransitionCompilePool,
) -> Option<(MonsGame, Vec<Input>)> {
    update_turn_engine_diagnostics(|diagnostics| diagnostics.compile_attempts += 1);
    record_compile_attempt_for_action(action);
    let mut best = best_transition_for_action(game, perspective, action, &compile_pool.transitions);
    if best.is_none() && compile_pool.expand(game) {
        best = best_transition_for_action(game, perspective, action, &compile_pool.transitions);
    }

    let Some((_, best_index)) = best else {
        update_turn_engine_diagnostics(|diagnostics| diagnostics.compile_failures += 1);
        record_compile_failure_for_action(
            action,
            compile_pool.transitions.len() >= compile_pool.limit,
        );
        return None;
    };
    let best_transition = &compile_pool.transitions[best_index];

    let consistent_hash = MonsGameModel::search_state_hash(&best_transition.game);
    let snapshot = TurnSnapshot::from_game(&best_transition.game);
    if consistent_hash != snapshot.state_hash {
        update_turn_engine_diagnostics(|diagnostics| diagnostics.compile_state_mismatches += 1);
        return None;
    }

    Some((
        best_transition.game.clone_for_simulation(),
        best_transition.inputs.clone(),
    ))
}

fn action_priority_locations(action: TurnAction) -> Vec<Location> {
    match action {
        TurnAction::Walk { actor, to } => vec![actor, to],
        TurnAction::Attack { actor, target } => vec![actor, target],
        TurnAction::SpiritShift {
            actor,
            target,
            destination,
        } => vec![actor, target, destination],
        TurnAction::Bomb { actor, target } => vec![actor, target],
        TurnAction::MoveMana { from, to } => vec![from, to],
        TurnAction::ScoreCarry { actor, step, .. } => vec![actor, step],
        TurnAction::SafetyRetreat { actor, to } => vec![actor, to],
    }
}

fn transition_matches_action(
    before: &MonsGame,
    after: &MonsGame,
    events: &[Event],
    perspective: Color,
    action: TurnAction,
) -> bool {
    match action {
        TurnAction::Walk { actor, to } => moved_actor_to(events, actor, to) && !events_include_non_walk_action(events),
        TurnAction::Attack { actor, target } => attack_events_match(events, actor, target, perspective),
        TurnAction::SpiritShift {
            actor,
            target,
            destination,
        } => events.iter().any(|event| {
            matches!(
                event,
                Event::SpiritTargetMove { by, from, to, .. }
                    if *by == actor && *from == target && *to == destination
            )
        }),
        TurnAction::Bomb { actor, target } => events.iter().any(|event| {
            matches!(
                event,
                Event::BombAttack { from, to, .. } if *from == actor && *to == target
            )
        }),
        TurnAction::MoveMana { from, to } => events.iter().any(|event| {
            matches!(event, Event::ManaMove { from: event_from, to: event_to, .. } if *event_from == from && *event_to == to)
        }),
        TurnAction::ScoreCarry { actor, wanted, step } => {
            moved_actor_to(events, actor, step)
                && (events.iter().any(|event| {
                    matches!(event, Event::ManaScored { mana, .. } if *mana == wanted)
                }) || actor_or_successor_carries(after, perspective, wanted))
        }
        TurnAction::SafetyRetreat { actor, to } => {
            moved_actor_to(events, actor, to)
                && own_drainer_safety_score(&after.board, perspective)
                    > own_drainer_safety_score(&before.board, perspective)
        }
    }
}

fn transition_score(
    before: &MonsGame,
    after: &MonsGame,
    events: &[Event],
    perspective: Color,
    action: TurnAction,
) -> i32 {
    let mut score = score_for_color(after, perspective)
        .saturating_sub(score_for_color(before, perspective))
        * 500;
    score += own_drainer_safety_score(&after.board, perspective).saturating_mul(180);
    if !opponent_can_win_immediately(before, perspective)
        && opponent_can_win_immediately(after, perspective)
    {
        score -= 2_200;
    }
    match action {
        TurnAction::Walk { actor, to } => {
            score += actor.distance(&to).saturating_mul(-20);
        }
        TurnAction::Attack { .. } => {
            if events_include_opponent_drainer_faint(events, perspective) {
                score += 1_600;
            }
            if events_include_any_faint(events, perspective) {
                score += 800;
            }
        }
        TurnAction::SpiritShift { .. } => {
            if events
                .iter()
                .any(|event| matches!(event, Event::ManaScored { .. }))
            {
                score += 1_000;
            }
            if events
                .iter()
                .any(|event| matches!(event, Event::SpiritTargetMove { .. }))
            {
                score += 600;
            }
        }
        TurnAction::Bomb { .. } => {
            if events_include_any_faint(events, perspective) {
                score += 1_000;
            }
        }
        TurnAction::MoveMana { from, to } => {
            score += distance_to_nearest_pool(from, perspective)
                .saturating_sub(distance_to_nearest_pool(to, perspective))
                .saturating_mul(160);
        }
        TurnAction::ScoreCarry { wanted, .. } => {
            score += wanted.score(perspective).saturating_mul(200);
        }
        TurnAction::SafetyRetreat { .. } => {
            score += own_drainer_safety_score(&after.board, perspective).saturating_mul(260);
        }
    }
    score
}

fn moved_actor_to(events: &[Event], actor: Location, to: Location) -> bool {
    events.iter().any(|event| match event {
        Event::MonMove {
            from, to: event_to, ..
        } => *from == actor && *event_to == to,
        Event::DemonAdditionalStep {
            from, to: event_to, ..
        } => *from == actor && *event_to == to,
        _ => false,
    })
}

fn attack_events_match(
    events: &[Event],
    actor: Location,
    target: Location,
    perspective: Color,
) -> bool {
    events.iter().any(|event| match event {
        Event::MysticAction { from, to, .. } | Event::DemonAction { from, to, .. } => {
            *from == actor && *to == target
        }
        Event::MonFainted { mon, to, .. } => mon.color == perspective.other() && *to == target,
        _ => false,
    })
}

fn actor_or_successor_carries(after: &MonsGame, perspective: Color, wanted: Mana) -> bool {
    after.board.occupied().any(|(_, item)| {
        matches!(
            item,
            Item::MonWithMana { mon, mana }
                if mon.color == perspective && !mon.is_fainted() && *mana == wanted
        )
    })
}

fn actor_can_attack_from_item(item: &Item) -> bool {
    match item {
        Item::Mon { mon } | Item::MonWithMana { mon, .. } | Item::MonWithConsumable { mon, .. } => {
            matches!(mon.kind, MonKind::Mystic | MonKind::Demon)
        }
        Item::Mana { .. } | Item::Consumable { .. } => false,
    }
}

fn actor_can_bomb_from_item(item: &Item) -> bool {
    matches!(
        item,
        Item::MonWithConsumable {
            mon,
            consumable: Consumable::Bomb,
        } if !mon.is_fainted()
    )
}

fn actor_can_attack_target_now(
    board: &Board,
    actor: Location,
    target: Location,
    item: &Item,
    perspective: Color,
) -> bool {
    if matches!(board.square(actor), Square::MonBase { .. }) {
        return false;
    }
    let Some(target_item) = board.item(target) else {
        return false;
    };
    let Some(target_mon) = target_item.mon() else {
        return false;
    };
    if target_mon.color != perspective.other() || target_mon.is_fainted() {
        return false;
    }
    if location_guarded_by_angel(board.find_awake_angel(perspective.other()), target) {
        return false;
    }
    match item {
        Item::Mon { mon } | Item::MonWithMana { mon, .. } | Item::MonWithConsumable { mon, .. } => {
            match mon.kind {
                MonKind::Mystic => actor.reachable_by_mystic_action_ref().contains(&target),
                MonKind::Demon => {
                    actor.reachable_by_demon_action_ref().contains(&target)
                        && demon_attack_path_clear(board, actor, target)
                }
                _ => false,
            }
        }
        Item::Mana { .. } | Item::Consumable { .. } => false,
    }
}

fn actor_can_bomb_target_now(
    board: &Board,
    actor: Location,
    target: Location,
    item: &Item,
    perspective: Color,
) -> bool {
    if !actor.reachable_by_bomb_ref().contains(&target) {
        return false;
    }
    let Item::MonWithConsumable {
        mon,
        consumable: Consumable::Bomb,
    } = item
    else {
        return false;
    };
    if mon.color != perspective || mon.is_fainted() {
        return false;
    }
    matches!(
        board.item(target),
        Some(
            Item::Mon { mon: target_mon }
            | Item::MonWithMana { mon: target_mon, .. }
            | Item::MonWithConsumable {
                mon: target_mon,
                ..
            }
        ) if target_mon.color == perspective.other() && !target_mon.is_fainted()
    )
}

fn location_guarded_by_angel(angel_location: Option<Location>, location: Location) -> bool {
    angel_location.map_or(false, |angel| angel.distance(&location) == 1)
}

fn demon_attack_path_clear(board: &Board, from: Location, target: Location) -> bool {
    let middle = from.location_between(&target);
    board.item(middle).is_none()
        && !matches!(
            board.square(middle),
            Square::SupermanaBase | Square::MonBase { .. }
        )
}

fn spirit_target_allowed(item: Item) -> bool {
    match item {
        Item::Mon { mon } | Item::MonWithMana { mon, .. } | Item::MonWithConsumable { mon, .. } => {
            !mon.is_fainted()
        }
        Item::Mana { .. } | Item::Consumable { .. } => true,
    }
}

fn spirit_destination_allowed(board: &Board, target_item: Item, destination: Location) -> bool {
    let destination_item = board.item(destination).copied();
    let destination_square = board.square(destination);
    let target_mon = target_item.mon().copied();
    let target_mana = target_item.mana().copied();

    let valid_destination = match destination_item {
        Some(Item::Mon {
            mon: destination_mon,
        }) => match target_item {
            Item::Mon { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. } => false,
            Item::Mana { .. } => {
                destination_mon.kind == MonKind::Drainer && !destination_mon.is_fainted()
            }
            Item::Consumable {
                consumable: Consumable::BombOrPotion,
            } => true,
            Item::Consumable { .. } => false,
        },
        Some(Item::Mana { .. }) => {
            matches!(target_mon, Some(mon) if mon.kind == MonKind::Drainer && !mon.is_fainted())
        }
        Some(Item::MonWithMana { .. }) | Some(Item::MonWithConsumable { .. }) => {
            matches!(
                target_item,
                Item::Consumable {
                    consumable: Consumable::BombOrPotion,
                }
            )
        }
        Some(Item::Consumable {
            consumable: Consumable::BombOrPotion,
        }) => matches!(
            target_item,
            Item::Mon { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. }
        ),
        Some(Item::Consumable { .. }) => false,
        None => true,
    };
    if !valid_destination {
        return false;
    }

    match destination_square {
        Square::Regular
        | Square::ConsumableBase
        | Square::ManaBase { .. }
        | Square::ManaPool { .. } => true,
        Square::SupermanaBase => {
            target_mana == Some(Mana::Supermana)
                || (target_mana.is_none()
                    && matches!(target_mon.map(|mon| mon.kind), Some(MonKind::Drainer)))
        }
        Square::MonBase { kind, color } => {
            matches!(target_mon, Some(mon) if mon.kind == kind && mon.color == color)
                && target_mana.is_none()
                && target_item.consumable().is_none()
        }
    }
}

fn mana_move_destination_allowed(board: &Board, destination: Location) -> bool {
    let item = board.item(destination);
    let square = board.square(destination);
    match item {
        Some(Item::Mon { mon }) => match square {
            Square::Regular
            | Square::ConsumableBase
            | Square::ManaBase { .. }
            | Square::ManaPool { .. } => mon.kind == MonKind::Drainer && !mon.is_fainted(),
            Square::SupermanaBase | Square::MonBase { .. } => false,
        },
        Some(Item::MonWithConsumable { .. })
        | Some(Item::Consumable { .. })
        | Some(Item::MonWithMana { .. })
        | Some(Item::Mana { .. }) => false,
        None => matches!(
            square,
            Square::Regular
                | Square::ConsumableBase
                | Square::ManaBase { .. }
                | Square::ManaPool { .. }
        ),
    }
}

fn nearest_wanted_mana_location(board: &Board, wanted: Mana) -> Option<Location> {
    board.occupied().find_map(|(location, item)| {
        matches!(item, Item::Mana { mana } if *mana == wanted).then_some(location)
    })
}

fn walk_destination_plausible(board: &Board, actor: Location, destination: Location) -> bool {
    let Some(actor_mon) = board.item(actor).and_then(|item| item.mon()).copied() else {
        return false;
    };
    match board.item(destination) {
        Some(Item::Mon { .. })
        | Some(Item::MonWithMana { .. })
        | Some(Item::MonWithConsumable { .. }) => false,
        Some(Item::Mana { .. }) | Some(Item::Consumable { .. }) | None => {
            match board.square(destination) {
                Square::Regular
                | Square::ConsumableBase
                | Square::ManaBase { .. }
                | Square::ManaPool { .. } => true,
                Square::SupermanaBase => actor_mon.kind == MonKind::Drainer,
                Square::MonBase { kind, color } => {
                    actor_mon.kind == kind && actor_mon.color == color
                }
            }
        }
    }
}

fn events_include_non_walk_action(events: &[Event]) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            Event::MysticAction { .. }
                | Event::DemonAction { .. }
                | Event::BombAttack { .. }
                | Event::SpiritTargetMove { .. }
        )
    })
}

fn events_include_any_faint(events: &[Event], perspective: Color) -> bool {
    events.iter().any(
        |event| matches!(event, Event::MonFainted { mon, .. } if mon.color == perspective.other()),
    )
}

fn events_include_opponent_drainer_faint(events: &[Event], perspective: Color) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            Event::MonFainted { mon, .. }
                if mon.color == perspective.other() && mon.kind == MonKind::Drainer
        )
    })
}

fn cached_step_if_legal(game: &MonsGame, mode: TurnEngineMode) -> Option<Vec<Input>> {
    let key = TurnEngineCacheKey {
        state_hash: MonsGameModel::search_state_hash(game),
        mode,
    };
    TURN_ENGINE_CONTINUATION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let cached = cache.get(&key).cloned();
        match cached {
            Some(inputs) => {
                if MonsGameModel::apply_inputs_for_search(game, inputs.as_slice()).is_some() {
                    Some(inputs)
                } else {
                    cache.remove(&key);
                    None
                }
            }
            None => None,
        }
    })
}

fn register_plan_continuations(game: &MonsGame, mode: TurnEngineMode, chunks: &[Vec<Input>]) {
    if chunks.is_empty() {
        return;
    }
    let mut state = game.clone_for_simulation();
    let start_color = game.active_color;
    for chunk in chunks {
        let key = TurnEngineCacheKey {
            state_hash: MonsGameModel::search_state_hash(&state),
            mode,
        };
        TURN_ENGINE_CONTINUATION_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if cache.len() >= TURN_ENGINE_CACHE_MAX_ENTRIES && !cache.contains_key(&key) {
                cache.clear();
            }
            cache.insert(key, chunk.clone());
        });
        let Some(next) = MonsGameModel::apply_inputs_for_search(&state, chunk.as_slice()) else {
            break;
        };
        if next.active_color != start_color {
            break;
        }
        state = next;
    }
}

fn compare_chunks(left: &[Vec<Input>], right: &[Vec<Input>]) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

fn winner_state(game: &MonsGame, perspective: Color) -> i32 {
    match game.winner_color() {
        Some(winner) if winner == perspective => 2,
        Some(_) => -2,
        None => 0,
    }
}

fn opponent_can_win_immediately(game: &MonsGame, perspective: Color) -> bool {
    if game.winner_color().is_some() || game.active_color != perspective.other() {
        return false;
    }
    let opponent = perspective.other();
    let needed = Config::TARGET_SCORE.saturating_sub(score_for_color(game, opponent));
    if needed <= 0 {
        return true;
    }
    exact_turn_summary(game, opponent).same_turn_score_window_value >= needed
}

fn score_for_color(game: &MonsGame, color: Color) -> i32 {
    if color == Color::White {
        game.white_score
    } else {
        game.black_score
    }
}

fn remaining_moves_for_color(game: &MonsGame, color: Color) -> i32 {
    if game.active_color == color {
        (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0)
    } else {
        Config::MONS_MOVES_PER_TURN
    }
}

fn distance_to_nearest_pool(location: Location, color: Color) -> i32 {
    Config::squares_ref()
        .iter()
        .filter_map(|(loc, square)| match square {
            Square::ManaPool { color: pool_color } if *pool_color == color => {
                Some(location.distance(loc))
            }
            _ => None,
        })
        .min()
        .unwrap_or(Config::BOARD_SIZE)
}

fn find_awake_drainer_location(board: &Board, color: Color) -> Option<Location> {
    board.occupied().find_map(|(location, item)| {
        let mon = item.mon()?;
        (mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted())
            .then_some(location)
    })
}

fn own_drainer_safety_score(board: &Board, color: Color) -> i32 {
    let Some(drainer_location) = find_awake_drainer_location(board, color) else {
        return 0;
    };
    let angel_nearby = board
        .find_awake_angel(color)
        .map_or(false, |angel| angel.distance(&drainer_location) == 1);
    let immediate = is_drainer_under_immediate_threat(board, color, drainer_location, angel_nearby);
    let walk = is_drainer_under_walk_threat(board, color, drainer_location, angel_nearby);
    let exact_safe = is_drainer_exactly_safe_next_turn_on_board(board, color, drainer_location);

    if exact_safe && !immediate && !walk {
        2
    } else if exact_safe {
        1
    } else if immediate || walk {
        -2
    } else {
        -1
    }
}

fn opponent_drainer_kill_is_high_value(
    game: &MonsGame,
    perspective: Color,
    target_drainer: Location,
) -> bool {
    let opponent = perspective.other();
    if own_drainer_safety_score(&game.board, perspective) < 0 {
        return true;
    }
    if exact_turn_summary(game, opponent).same_turn_score_window_value > 0 {
        return true;
    }
    if score_for_color(game, opponent) >= Config::TARGET_SCORE - 2 {
        return true;
    }
    if matches!(
        game.board.item(target_drainer),
        Some(Item::MonWithMana { .. })
    ) {
        return true;
    }
    distance_to_nearest_pool(target_drainer, opponent) <= 3
}

fn own_drainer_carries_safe_mana(board: &Board, color: Color, wanted: Mana) -> bool {
    let Some(drainer_location) = find_awake_drainer_location(board, color) else {
        return false;
    };
    matches!(
        board.item(drainer_location),
        Some(Item::MonWithMana { mana, .. }) if *mana == wanted
    ) && is_drainer_exactly_safe_next_turn_on_board(board, color, drainer_location)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_with_items(
        items: Vec<(Location, Item)>,
        active_color: Color,
        turn_number: i32,
    ) -> MonsGame {
        let mut game = MonsGame::new(false);
        game.board = Board::new_with_items(items.into_iter().collect());
        game.active_color = active_color;
        game.actions_used_count = 0;
        game.mana_moves_count = 0;
        game.mons_moves_count = 0;
        game.turn_number = turn_number;
        game.white_score = 0;
        game.black_score = 0;
        game.white_potions_count = 0;
        game.black_potions_count = 0;
        game
    }

    fn engine_config() -> TurnEngineConfig {
        TurnEngineConfig {
            own_seed_cap: 16,
            own_beam: 6,
            per_node_family_cap: 4,
            step_cap: 6,
            opponent_seed_cap: 8,
            opponent_beam: 3,
            reply_seed_cap: 4,
            reply_beam: 2,
            expansion_cap: 192,
            enable_spirit_family: true,
            scoring_weights: &DEFAULT_SCORING_WEIGHTS,
            allow_exact_static_evaluation: false,
        }
    }

    fn exhaustive_same_turn_reachable<F>(game: &MonsGame, color: Color, predicate: F) -> bool
    where
        F: Fn(&MonsGame, &[Event]) -> bool,
    {
        fn visit<F>(game: &MonsGame, color: Color, seen: &mut HashSet<u64>, predicate: &F) -> bool
        where
            F: Fn(&MonsGame, &[Event]) -> bool,
        {
            if game.active_color != color {
                return false;
            }
            let state_hash = MonsGameModel::search_state_hash(game);
            if !seen.insert(state_hash) {
                return false;
            }
            for transition in MonsGameModel::enumerate_legal_transitions(
                game,
                usize::MAX,
                SuggestedStartInputOptions::for_automove(),
            ) {
                if predicate(&transition.game, &transition.events) {
                    return true;
                }
                if transition.game.active_color == color
                    && visit(&transition.game, color, seen, predicate)
                {
                    return true;
                }
            }
            false
        }

        if predicate(game, &[]) {
            return true;
        }
        let mut seen = HashSet::new();
        visit(game, color, &mut seen, &predicate)
    }

    fn immediate_score_fixture() -> MonsGame {
        game_with_items(
            vec![
                (
                    Location::new(9, 1),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Regular(Color::White),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
            2,
        )
    }

    fn safe_supermana_fixture() -> MonsGame {
        game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
            2,
        )
    }

    fn safe_opponent_mana_fixture() -> MonsGame {
        game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
            2,
        )
    }

    fn spirit_impact_fixture() -> MonsGame {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
            2,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 2;
        game
    }

    fn primary_spirit_setup_fixture() -> MonsGame {
        game_with_items(
            vec![
                (
                    Location::new(9, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 8),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
            2,
        )
    }

    fn primary_pvs_sensitive_search_fixture() -> MonsGame {
        MonsGame::from_fen(
            "0 0 b 1 0 0 0 0 4 n05d0xa0xn04/n02xxmn01s0xn03e0xn02/n02y0xn08/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMxxMxxMn01xxMn03/n11/n01E0xn03D0xxxMS0xn03/n04A0xn01Y0xn04/n11",
            false,
        )
        .expect("primary_pvs_sensitive_search_fixture: valid fen")
    }

    fn drainer_kill_fixture() -> MonsGame {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(3, 2),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::White, 0),
                    },
                ),
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(1, 0),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
            2,
        );
        game.black_score = Config::TARGET_SCORE - 1;
        game
    }

    fn safety_recovery_fixture() -> MonsGame {
        let drainer = Location::new(5, 5);
        let probe_locations = [
            Location::new(6, 6),
            Location::new(6, 7),
            Location::new(7, 6),
            Location::new(7, 5),
            Location::new(5, 7),
            Location::new(4, 7),
            Location::new(7, 4),
            Location::new(6, 4),
        ];
        for kind in [MonKind::Mystic, MonKind::Demon] {
            for location in probe_locations {
                let game = game_with_items(
                    vec![
                        (
                            drainer,
                            Item::Mon {
                                mon: Mon::new(MonKind::Drainer, Color::White, 0),
                            },
                        ),
                        (
                            location,
                            Item::Mon {
                                mon: Mon::new(kind, Color::Black, 0),
                            },
                        ),
                        (
                            Location::new(0, 10),
                            Item::Mon {
                                mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                            },
                        ),
                    ],
                    Color::White,
                    2,
                );
                let before_safety = own_drainer_safety_score(&game.board, Color::White);
                if before_safety >= 0 {
                    continue;
                }
                let has_improving_move = MonsGameModel::enumerate_legal_transitions(
                    &game,
                    usize::MAX,
                    SuggestedStartInputOptions::for_automove(),
                )
                .into_iter()
                .any(|transition| {
                    transition.events.iter().any(
                        |event| matches!(event, Event::MonMove { from, .. } if *from == drainer),
                    ) && own_drainer_safety_score(&transition.game.board, Color::White)
                        > before_safety
                });
                if has_improving_move {
                    return game;
                }
            }
        }
        panic!("expected at least one deterministic safety-recovery fixture");
    }

    fn assert_plan_roundtrip(game: &MonsGame, plan: &TurnPlan) -> MonsGame {
        let mut state = game.clone_for_simulation();
        for chunk in plan.compiled_chunks.iter() {
            state = MonsGameModel::apply_inputs_for_search(&state, chunk.as_slice())
                .expect("compiled chunk should stay legal");
        }
        assert_eq!(
            plan.end_snapshot.state_hash,
            MonsGameModel::search_state_hash(&state)
        );
        state
    }

    #[test]
    fn turn_engine_finds_immediate_score_plan() {
        let game = immediate_score_fixture();
        let plan = turn_engine_best_plan_for_test(&game, Color::White, engine_config())
            .expect("immediate score plan");
        assert_eq!(plan.family, TurnPlanFamily::ImmediateScore);
        assert!(!plan.compiled_chunks.is_empty());
        let state = assert_plan_roundtrip(&game, &plan);
        assert!(state.white_score > game.white_score);
    }

    #[test]
    fn turn_engine_finds_safe_supermana_progress_plan() {
        let game = safe_supermana_fixture();
        let plan = turn_engine_best_plan_for_test(&game, Color::White, engine_config())
            .expect("supermana progress plan");
        assert!(
            matches!(
                plan.family,
                TurnPlanFamily::SafeSupermanaProgress | TurnPlanFamily::ImmediateScore
            ),
            "family={:?}",
            plan.family
        );
        let state = assert_plan_roundtrip(&game, &plan);
        assert!(
            state.white_score > 0
                || state.board.occupied().any(|(_, item)| {
                    matches!(
                        item,
                        Item::MonWithMana { mon, mana }
                            if mon.color == Color::White
                                && mon.kind == MonKind::Drainer
                                && *mana == Mana::Supermana
                    )
                })
        );
    }

    #[test]
    fn turn_engine_finds_safe_opponent_mana_progress_plan() {
        let game = safe_opponent_mana_fixture();
        let plan = turn_engine_best_plan_for_test(&game, Color::White, engine_config())
            .expect("opponent mana progress plan");
        assert!(
            matches!(
                plan.family,
                TurnPlanFamily::SafeOpponentManaProgress | TurnPlanFamily::ImmediateScore
            ),
            "family={:?}",
            plan.family
        );
        let state = assert_plan_roundtrip(&game, &plan);
        assert!(
            state.white_score >= Mana::Regular(Color::Black).score(Color::White)
                || state.board.occupied().any(|(_, item)| {
                    matches!(
                        item,
                        Item::MonWithMana { mon, mana }
                            if mon.color == Color::White
                                && mon.kind == MonKind::Drainer
                                && *mana == Mana::Regular(Color::Black)
                    )
                })
        );
    }

    #[test]
    fn turn_engine_finds_spirit_impact_plan() {
        let game = spirit_impact_fixture();
        let plan = turn_engine_best_plan_for_test(&game, Color::White, engine_config())
            .expect("spirit impact plan");
        assert_eq!(plan.family, TurnPlanFamily::SpiritImpact);
        assert!(plan
            .actions
            .iter()
            .any(|action| matches!(action, TurnAction::SpiritShift { .. })));
    }

    #[test]
    fn turn_engine_matches_primary_spirit_setup_fixture() {
        let game = primary_spirit_setup_fixture();
        let plan = turn_engine_best_plan_for_test(&game, Color::White, engine_config())
            .expect("primary spirit setup plan");
        assert_eq!(plan.family, TurnPlanFamily::SpiritImpact);
        assert_eq!(
            plan.compiled_chunks.first(),
            Some(&vec![
                Input::Location(Location::new(9, 7)),
                Input::Location(Location::new(7, 8)),
                Input::Location(Location::new(7, 7)),
            ]),
        );
        assert_plan_roundtrip(&game, &plan);
    }

    #[test]
    fn turn_engine_generate_turn_plans_retains_leaf_nodes_on_pvs_fixture() {
        let game = primary_pvs_sensitive_search_fixture();
        let plans = generate_turn_plans(&game, Color::Black, engine_config(), 16, 6, 7, 192)
            .expect("pvs fixture should produce at least one multi-step or terminal leaf plan");
        assert!(
            !plans.is_empty(),
            "pvs fixture should not collapse to no-plan inside the main generator"
        );
    }

    #[test]
    fn turn_engine_finds_drainer_kill_or_deny_plan() {
        let game = drainer_kill_fixture();
        let plan = turn_engine_best_plan_for_test(&game, Color::White, engine_config())
            .expect("drainer kill or deny plan");
        assert!(
            matches!(
                plan.family,
                TurnPlanFamily::DenyOpponentWindow | TurnPlanFamily::DrainerKill
            ),
            "family={:?}",
            plan.family
        );
        let state = assert_plan_roundtrip(&game, &plan);
        let opponent_drainer_alive =
            find_awake_drainer_location(&state.board, Color::Black).is_some();
        assert!(
            !opponent_drainer_alive
                || exact_turn_summary(&state, Color::Black).same_turn_score_window_value
                    < exact_turn_summary(&game, Color::Black).same_turn_score_window_value,
            "plan should either remove the drainer or reduce the immediate scoring window"
        );
    }

    #[test]
    fn turn_engine_finds_drainer_safety_recovery_plan() {
        let game = safety_recovery_fixture();
        let before_safety = own_drainer_safety_score(&game.board, Color::White);
        let plan = turn_engine_best_plan_for_test(&game, Color::White, engine_config())
            .expect("safety recovery plan");
        assert!(
            matches!(
                plan.family,
                TurnPlanFamily::DrainerSafetyRecovery | TurnPlanFamily::DenyOpponentWindow
            ),
            "family={:?}",
            plan.family
        );
        let state = assert_plan_roundtrip(&game, &plan);
        assert!(own_drainer_safety_score(&state.board, Color::White) > before_safety);
    }

    #[test]
    fn turn_engine_oracle_matches_safe_supermana_progress_fixture() {
        let game = safe_supermana_fixture();
        let plan = turn_engine_best_plan_for_test(&game, Color::White, engine_config());
        let exhaustive = exhaustive_same_turn_reachable(&game, Color::White, |state, events| {
            events.iter().any(
                |event| matches!(event, Event::ManaScored { mana, .. } if *mana == Mana::Supermana),
            ) || state.board.occupied().any(|(_, item)| {
                matches!(
                    item,
                    Item::MonWithMana { mon, mana }
                        if mon.color == Color::White
                            && mon.kind == MonKind::Drainer
                            && *mana == Mana::Supermana
                )
            })
        });
        assert_eq!(plan.is_some(), exhaustive);
    }

    #[test]
    fn turn_engine_compiled_chunks_roundtrip_to_planned_snapshot() {
        for game in [
            immediate_score_fixture(),
            safe_supermana_fixture(),
            safe_opponent_mana_fixture(),
            spirit_impact_fixture(),
            drainer_kill_fixture(),
            safety_recovery_fixture(),
        ] {
            let plan = turn_engine_best_plan_for_test(&game, Color::White, engine_config())
                .expect("fixture should yield a plan");
            assert_plan_roundtrip(&game, &plan);
        }
    }

    #[test]
    fn turn_engine_cache_replays_remaining_chunks() {
        clear_turn_engine_plan_cache();
        let game = game_with_items(
            vec![
                (
                    Location::new(8, 2),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Regular(Color::White),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
            2,
        );
        let plan = turn_engine_best_plan_for_test(&game, Color::White, engine_config())
            .expect("cacheable plan");
        assert!(plan.compiled_chunks.len() >= 2, "need multi-step plan");
        let first =
            turn_engine_next_inputs(&game, Color::White, TurnEngineMode::ProV1, engine_config())
                .expect("first chunk");
        assert_eq!(first, plan.compiled_chunks[0]);
        let after_first = MonsGameModel::apply_inputs_for_search(&game, first.as_slice())
            .expect("first chunk legal");
        let second = turn_engine_next_inputs(
            &after_first,
            Color::White,
            TurnEngineMode::ProV1,
            engine_config(),
        )
        .expect("second chunk");
        assert_eq!(second, plan.compiled_chunks[1]);
    }

    #[test]
    fn turn_engine_cache_invalidates_on_diverged_state() {
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        let game = game_with_items(
            vec![
                (
                    Location::new(8, 2),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Regular(Color::White),
                    },
                ),
                (
                    Location::new(4, 4),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let first =
            turn_engine_next_inputs(&game, Color::White, TurnEngineMode::ProV1, engine_config())
                .expect("first chunk");
        let after_first = MonsGameModel::apply_inputs_for_search(&game, first.as_slice())
            .expect("first chunk legal");

        clear_turn_engine_diagnostics();
        let mut diverged = after_first.clone_for_simulation();
        diverged.board = Board::new_with_items(
            diverged
                .board
                .occupied()
                .filter_map(|(location, item)| {
                    if location == Location::new(4, 4) {
                        None
                    } else {
                        Some((location, *item))
                    }
                })
                .chain(std::iter::once((
                    Location::new(4, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                )))
                .collect(),
        );

        let _ = turn_engine_next_inputs(
            &diverged,
            Color::White,
            TurnEngineMode::ProV1,
            engine_config(),
        );
        let diagnostics = turn_engine_diagnostics_snapshot();
        assert_eq!(diagnostics.cache_hits, 0);
        assert_eq!(diagnostics.cache_misses, 1);
    }
}
