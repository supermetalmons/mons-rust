#![cfg(any(target_arch = "wasm32", test))]

use crate::models::scoring::{
    evaluate_preferability_with_weights_and_exact_policy, ScoringWeights, DEFAULT_SCORING_WEIGHTS,
};
use crate::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

const TURN_PLANNER_CACHE_MAX_ENTRIES: usize = 4096;
const TURN_PLANNER_CHAIN_LIMIT: usize = 8;
const TURN_PLANNER_EMERGENCY_NODE_CAP: usize = 256;
const TURN_PLANNER_INTENT_CAP: usize = 16;
const TURN_PLANNER_ROUTE_CAP: usize = 24;
const TURN_PLANNER_SPIRIT_TOP_K: usize = 6;
const TURN_PLANNER_UTILITY_MEMO_MAX_ENTRIES: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TurnPlannerMode {
    ProV1,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TurnPlannerSearchConfig {
    pub max_nodes: usize,
    pub beam_width: usize,
    pub response_beam_width: usize,
    pub step_cap: usize,
    pub route_cap: usize,
    pub per_node_route_cap: usize,
    pub scoring_weights: &'static ScoringWeights,
    pub allow_exact_static_evaluation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TurnPlannerCacheKey {
    state_hash: u64,
    mode: TurnPlannerMode,
}

#[derive(Debug, Clone)]
struct PlannerRoute {
    inputs: Vec<Input>,
    kind: PlannerRouteKind,
    priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PlannerRouteKind {
    ModelTactical,
    DrainerScore,
    DrainerKill,
    SpiritImpact,
    DrainerSafety,
    ManaMove,
    TacticalDeny,
    Fallback,
}

#[derive(Debug, Clone, Copy)]
struct TurnResourceBudget {
    remaining_mon_moves: i32,
    can_use_action: bool,
    can_move_mana: bool,
}

#[derive(Debug, Clone, Copy)]
struct TurnResourceRequirement {
    mon_moves_needed: i32,
    needs_action: bool,
    needs_mana_move: bool,
}

#[derive(Debug, Clone, Copy)]
struct SecureManaIntent {
    actor: Location,
    wanted: Mana,
    next_step: Option<Location>,
    estimated_gain: i32,
    safety_delta: i32,
    resources: TurnResourceRequirement,
}

#[derive(Debug, Clone, Copy)]
struct DrainerKillIntent {
    actor: Location,
    target: Location,
    setup_step: Option<Location>,
    estimated_gain: i32,
    safety_delta: i32,
    resources: TurnResourceRequirement,
}

#[derive(Debug, Clone, Copy)]
struct SafetyRecoverIntent {
    actor: Location,
    target_step: Location,
    estimated_gain: i32,
    safety_delta: i32,
    resources: TurnResourceRequirement,
}

#[derive(Debug, Clone, Copy)]
struct SpiritImpactIntent {
    actor: Location,
    target: Location,
    destination: Location,
    estimated_gain: i32,
    safety_delta: i32,
    resources: TurnResourceRequirement,
}

#[derive(Debug, Clone, Copy)]
struct ManaTempoIntent {
    actor: Location,
    destination: Location,
    estimated_gain: i32,
    safety_delta: i32,
    resources: TurnResourceRequirement,
}

#[derive(Debug, Clone, Copy)]
struct TacticalDenyIntent {
    actor: Location,
    target: Location,
    setup_step: Option<Location>,
    estimated_gain: i32,
    safety_delta: i32,
    resources: TurnResourceRequirement,
}

#[derive(Debug, Clone, Copy)]
enum PlannerIntent {
    SecureMana(SecureManaIntent),
    DrainerKill(DrainerKillIntent),
    SafetyRecover(SafetyRecoverIntent),
    SpiritImpact(SpiritImpactIntent),
    ManaTempo(ManaTempoIntent),
    TacticalDeny(TacticalDenyIntent),
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TurnPlannerDiagnostics {
    pub intent_generation_calls: usize,
    pub intent_generation_hits: usize,
    pub compile_fallbacks: usize,
    pub injected_root_attempts: usize,
    pub injected_root_accepts: usize,
    pub route_model_tactical: usize,
    pub route_drainer_score: usize,
    pub route_drainer_kill: usize,
    pub route_spirit_impact: usize,
    pub route_drainer_safety: usize,
    pub route_mana_move: usize,
    pub route_tactical_deny: usize,
    pub route_fallback: usize,
    pub expansions: usize,
}

#[derive(Debug, Clone)]
struct PlannerNode {
    game: MonsGame,
    steps: Vec<Vec<Input>>,
}

#[derive(Debug, Clone)]
struct TurnPlan {
    game: MonsGame,
    steps: Vec<Vec<Input>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UtilityMemoKey {
    game_hash: u64,
    start_hash: u64,
    perspective: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlannerUtility {
    win_state: i32,
    avoid_immediate_loss: i32,
    high_value_progress: i32,
    drainer_attack: i32,
    drainer_safety: i32,
    eval_score: i32,
}

impl Ord for PlannerUtility {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.win_state,
            self.avoid_immediate_loss,
            self.high_value_progress,
            self.drainer_attack,
            self.drainer_safety,
            self.eval_score,
        )
            .cmp(&(
                other.win_state,
                other.avoid_immediate_loss,
                other.high_value_progress,
                other.drainer_attack,
                other.drainer_safety,
                other.eval_score,
            ))
    }
}

impl PartialOrd for PlannerUtility {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlanBuildStatus {
    NoPlan,
    BudgetExceeded,
}

#[derive(Debug, Clone, Copy)]
enum IntentCompileMode {
    Path,
    Spirit,
    Attack,
}

thread_local! {
    static TURN_PLANNER_CONTINUATION_CACHE: RefCell<HashMap<TurnPlannerCacheKey, Vec<Input>>> =
        RefCell::new(HashMap::new());
    static TURN_PLANNER_DIAGNOSTICS: RefCell<TurnPlannerDiagnostics> =
        RefCell::new(TurnPlannerDiagnostics::default());
}

pub(crate) fn clear_turn_opportunity_plan_cache() {
    TURN_PLANNER_CONTINUATION_CACHE.with(|cache| cache.borrow_mut().clear());
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn clear_turn_planner_diagnostics() {
    TURN_PLANNER_DIAGNOSTICS.with(|diagnostics| {
        *diagnostics.borrow_mut() = TurnPlannerDiagnostics::default();
    });
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn turn_planner_diagnostics_snapshot() -> TurnPlannerDiagnostics {
    TURN_PLANNER_DIAGNOSTICS.with(|diagnostics| *diagnostics.borrow())
}

#[inline]
fn update_turn_planner_diagnostics(update: impl FnOnce(&mut TurnPlannerDiagnostics)) {
    TURN_PLANNER_DIAGNOSTICS.with(|diagnostics| update(&mut diagnostics.borrow_mut()));
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn record_turn_planner_injected_root_attempt(accepted: bool) {
    update_turn_planner_diagnostics(|diagnostics| {
        diagnostics.injected_root_attempts = diagnostics.injected_root_attempts.saturating_add(1);
        if accepted {
            diagnostics.injected_root_accepts = diagnostics.injected_root_accepts.saturating_add(1);
        }
    });
}

#[cfg(test)]
pub(crate) fn turn_opportunity_cache_entry_for_test(
    state_hash: u64,
    mode: TurnPlannerMode,
) -> Option<Vec<Input>> {
    TURN_PLANNER_CONTINUATION_CACHE.with(|cache| {
        cache
            .borrow()
            .get(&TurnPlannerCacheKey { state_hash, mode })
            .cloned()
    })
}

#[cfg(test)]
pub(crate) fn insert_turn_opportunity_cache_entry_for_test(
    state_hash: u64,
    mode: TurnPlannerMode,
    inputs: Vec<Input>,
) {
    TURN_PLANNER_CONTINUATION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() >= TURN_PLANNER_CACHE_MAX_ENTRIES
            && !cache.contains_key(&TurnPlannerCacheKey { state_hash, mode })
        {
            cache.clear();
        }
        cache.insert(TurnPlannerCacheKey { state_hash, mode }, inputs);
    });
}

pub(crate) fn turn_opportunity_planner_next_inputs(
    game: &MonsGame,
    perspective: Color,
    mode: TurnPlannerMode,
    config: TurnPlannerSearchConfig,
) -> Option<Vec<Input>> {
    turn_opportunity_planner_next_inputs_filtered(game, perspective, mode, config, None)
}

pub(crate) fn turn_opportunity_planner_next_inputs_from_allowed(
    game: &MonsGame,
    perspective: Color,
    mode: TurnPlannerMode,
    config: TurnPlannerSearchConfig,
    allowed_first_steps: &[Vec<Input>],
) -> Option<Vec<Input>> {
    turn_opportunity_planner_next_inputs_filtered(
        game,
        perspective,
        mode,
        config,
        Some(allowed_first_steps),
    )
}

pub(crate) fn turn_opportunity_planner_intent_root_candidates(
    game: &MonsGame,
    perspective: Color,
    _mode: TurnPlannerMode,
    config: TurnPlannerSearchConfig,
    limit: usize,
) -> Vec<Vec<Input>> {
    if limit == 0 || game.active_color != perspective || !planner_should_activate(game, perspective)
    {
        return Vec::new();
    }

    let mut routes = intent_first_routes(game, perspective, config);
    routes.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.inputs.cmp(&b.inputs))
    });
    let mut seen = HashSet::new();
    routes
        .into_iter()
        .filter_map(|route| seen.insert(route.inputs.clone()).then_some(route.inputs))
        .take(limit)
        .collect()
}

fn turn_opportunity_planner_next_inputs_filtered(
    game: &MonsGame,
    perspective: Color,
    mode: TurnPlannerMode,
    config: TurnPlannerSearchConfig,
    allowed_first_steps: Option<&[Vec<Input>]>,
) -> Option<Vec<Input>> {
    if game.active_color != perspective {
        return None;
    }

    update_turn_planner_diagnostics(|diagnostics| {
        diagnostics.expansions = 0;
    });

    let allowed_set =
        allowed_first_steps.map(|steps| steps.iter().cloned().collect::<HashSet<Vec<Input>>>());
    let allowed_rank = allowed_first_steps.map(|steps| {
        steps
            .iter()
            .enumerate()
            .map(|(rank, inputs)| (inputs.clone(), rank))
            .collect::<HashMap<Vec<Input>, usize>>()
    });
    let allowed_len = allowed_first_steps.map_or(0, |steps| steps.len());

    if let Some(cached) = cached_step_if_legal(game, mode) {
        if allowed_set
            .as_ref()
            .map_or(true, |allowed| allowed.contains(&cached))
        {
            return Some(cached);
        }
    }

    if !planner_should_activate(game, perspective) {
        return None;
    }

    let mut adaptive = config;
    if planner_tactical_emergency_state(game, perspective) {
        adaptive.max_nodes = adaptive
            .max_nodes
            .saturating_mul(4)
            .checked_div(3)
            .unwrap_or(adaptive.max_nodes)
            .max(adaptive.max_nodes)
            .min(TURN_PLANNER_EMERGENCY_NODE_CAP)
            .max(32);
        adaptive.beam_width = adaptive.beam_width.saturating_add(1).clamp(2, 6);
        adaptive.response_beam_width = adaptive.response_beam_width.saturating_add(1).clamp(1, 2);
    }
    let mut utility_memo: HashMap<UtilityMemoKey, PlannerUtility> = HashMap::new();

    let plan_build_result = if let Some(allowed_steps) = allowed_first_steps {
        generate_turn_plans_from_allowed_first_steps(
            game,
            perspective,
            adaptive,
            adaptive.max_nodes.max(1),
            adaptive.beam_width.max(1),
            allowed_steps,
            &mut utility_memo,
        )
    } else {
        generate_turn_plans(
            game,
            perspective,
            adaptive,
            adaptive.max_nodes.max(1),
            adaptive.beam_width.max(1),
            &mut utility_memo,
        )
    };
    let plans = match plan_build_result {
        Ok(plans) if !plans.is_empty() => plans,
        Ok(_) | Err(PlanBuildStatus::NoPlan) | Err(PlanBuildStatus::BudgetExceeded) => return None,
    };

    let mut best_plan_index: Option<usize> = None;
    let mut best_utility = PlannerUtility {
        win_state: i32::MIN,
        avoid_immediate_loss: i32::MIN,
        high_value_progress: i32::MIN,
        drainer_attack: i32::MIN,
        drainer_safety: i32::MIN,
        eval_score: i32::MIN,
    };

    for (index, plan) in plans.iter().enumerate() {
        let Some(first_step) = plan.steps.first() else {
            continue;
        };
        if !allowed_set
            .as_ref()
            .map_or(true, |allowed| allowed.contains(first_step))
        {
            continue;
        }

        let utility =
            evaluate_plan_with_response(game, plan, perspective, adaptive, &mut utility_memo);
        let rank_bonus = allowed_rank
            .as_ref()
            .and_then(|rank_map| rank_map.get(first_step))
            .map_or(0, |rank| {
                let span = allowed_len.saturating_sub(*rank).min(96) as i32;
                span.saturating_mul(12)
            });
        let first_step_safety_penalty = apply_inputs_for_planner(game, first_step.as_slice())
            .map_or(0, |(after, _)| {
                let safety = own_drainer_safety_score(&after.board, perspective);
                let safety_penalty = if safety < 0 {
                    safety.saturating_abs().saturating_mul(2_400)
                } else {
                    0
                };
                let immediate_reply_penalty = if opponent_can_win_immediately(&after, perspective) {
                    18_000
                } else {
                    0
                };
                safety_penalty.saturating_add(immediate_reply_penalty)
            });
        let selection_utility = PlannerUtility {
            high_value_progress: utility
                .high_value_progress
                .saturating_sub(first_step_safety_penalty),
            eval_score: utility.eval_score.saturating_add(rank_bonus),
            ..utility
        };
        if best_plan_index.is_none() || selection_utility > best_utility {
            best_utility = selection_utility;
            best_plan_index = Some(index);
        } else if selection_utility == best_utility
            && compare_step_chains(
                plans[index].steps.as_slice(),
                plans[best_plan_index.expect("best plan index")]
                    .steps
                    .as_slice(),
            ) == Ordering::Less
        {
            best_plan_index = Some(index);
        }
    }

    let Some(best_plan_index) = best_plan_index else {
        return None;
    };
    let best_plan = &plans[best_plan_index];
    if best_plan.steps.is_empty() {
        return None;
    }

    register_plan_continuations(game, mode, best_plan.steps.as_slice());
    best_plan.steps.first().cloned()
}

fn planner_should_activate(game: &MonsGame, perspective: Color) -> bool {
    let turn = exact_turn_summary(game, perspective);
    let drainer_safety = own_drainer_safety_score(&game.board, perspective);

    if turn.can_attack_opponent_drainer
        || turn.safe_supermana_progress
        || turn.safe_opponent_mana_progress
        || turn.same_turn_score_window_value >= 2
    {
        return true;
    }

    if drainer_safety <= -2 {
        return true;
    }

    false
}

pub(crate) fn planner_tactical_emergency_state(game: &MonsGame, perspective: Color) -> bool {
    if opponent_can_win_immediately(game, perspective) {
        return true;
    }

    if own_drainer_safety_score(&game.board, perspective) <= -2 {
        return true;
    }

    let needed = Config::TARGET_SCORE.saturating_sub(score_for_color(game, perspective));
    if needed > 0 {
        let same_turn = exact_turn_summary(game, perspective).same_turn_score_window_value;
        if same_turn >= needed {
            return true;
        }
    }

    false
}

fn cached_step_if_legal(game: &MonsGame, mode: TurnPlannerMode) -> Option<Vec<Input>> {
    let key = TurnPlannerCacheKey {
        state_hash: MonsGameModel::search_state_hash(game),
        mode,
    };
    TURN_PLANNER_CONTINUATION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let cached = cache.get(&key).cloned();
        match cached {
            Some(inputs) => {
                if apply_inputs_for_planner(game, inputs.as_slice()).is_some() {
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

fn register_plan_continuations(game: &MonsGame, mode: TurnPlannerMode, steps: &[Vec<Input>]) {
    if steps.is_empty() {
        return;
    }

    let mut state = game.clone_for_simulation();
    let start_color = game.active_color;

    for step in steps {
        let key = TurnPlannerCacheKey {
            state_hash: MonsGameModel::search_state_hash(&state),
            mode,
        };
        TURN_PLANNER_CONTINUATION_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if cache.len() >= TURN_PLANNER_CACHE_MAX_ENTRIES && !cache.contains_key(&key) {
                cache.clear();
            }
            cache.insert(key, step.clone());
        });

        let Some((next_state, _)) = apply_inputs_for_planner(&state, step.as_slice()) else {
            break;
        };
        if next_state.active_color != start_color {
            break;
        }
        state = next_state;
    }
}

fn generate_turn_plans(
    game: &MonsGame,
    perspective: Color,
    config: TurnPlannerSearchConfig,
    node_budget: usize,
    beam_width: usize,
    utility_memo: &mut HashMap<UtilityMemoKey, PlannerUtility>,
) -> Result<Vec<TurnPlan>, PlanBuildStatus> {
    let mut expansions = 0usize;
    let mut frontier = vec![PlannerNode {
        game: game.clone_for_simulation(),
        steps: Vec::new(),
    }];
    let mut terminal = Vec::new();

    for _ in 0..config.step_cap.max(1) {
        let mut candidates: Vec<(i64, PlannerNode)> = Vec::new();
        let mut expanded_any = false;
        let current_frontier = std::mem::take(&mut frontier);

        for node in current_frontier.into_iter() {
            if node.game.winner_color().is_some() || node.game.active_color != perspective {
                terminal.push(TurnPlan {
                    game: node.game,
                    steps: node.steps,
                });
                continue;
            }

            let routes = collect_atomic_routes(&node.game, perspective, config);
            if routes.is_empty() {
                terminal.push(TurnPlan {
                    game: node.game,
                    steps: node.steps,
                });
                continue;
            }

            for route in routes.into_iter().take(config.per_node_route_cap.max(1)) {
                expansions = expansions.saturating_add(1);
                if expansions > node_budget.max(1) {
                    update_turn_planner_diagnostics(|diagnostics| {
                        diagnostics.expansions = diagnostics.expansions.saturating_add(expansions);
                    });
                    return Err(PlanBuildStatus::BudgetExceeded);
                }

                let Some((after, _)) =
                    apply_inputs_for_planner(&node.game, route.inputs.as_slice())
                else {
                    continue;
                };

                let mut steps = node.steps.clone();
                steps.push(route.inputs.clone());
                let order = quick_node_order_score(
                    game,
                    &after,
                    perspective,
                    steps.len(),
                    route.kind,
                    route.priority,
                    config,
                    utility_memo,
                );
                candidates.push((order, PlannerNode { game: after, steps }));
                expanded_any = true;
            }
        }

        if !expanded_any || candidates.is_empty() {
            break;
        }

        candidates.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| compare_step_chains(a.1.steps.as_slice(), b.1.steps.as_slice()))
        });
        frontier = candidates
            .into_iter()
            .take(beam_width.max(1))
            .map(|(_, node)| node)
            .collect();
    }

    terminal.extend(frontier.into_iter().map(|node| TurnPlan {
        game: node.game,
        steps: node.steps,
    }));

    if terminal.is_empty() {
        return Err(PlanBuildStatus::NoPlan);
    }

    terminal.sort_by(|a, b| {
        let a_utility =
            utility_for_perspective_cached(&a.game, game, perspective, config, utility_memo);
        let b_utility =
            utility_for_perspective_cached(&b.game, game, perspective, config, utility_memo);
        b_utility
            .cmp(&a_utility)
            .then_with(|| compare_step_chains(a.steps.as_slice(), b.steps.as_slice()))
    });

    update_turn_planner_diagnostics(|diagnostics| {
        diagnostics.expansions = diagnostics.expansions.saturating_add(expansions);
    });

    Ok(terminal)
}

fn generate_turn_plans_from_allowed_first_steps(
    game: &MonsGame,
    perspective: Color,
    config: TurnPlannerSearchConfig,
    node_budget: usize,
    beam_width: usize,
    allowed_first_steps: &[Vec<Input>],
    utility_memo: &mut HashMap<UtilityMemoKey, PlannerUtility>,
) -> Result<Vec<TurnPlan>, PlanBuildStatus> {
    if allowed_first_steps.is_empty() {
        return Err(PlanBuildStatus::NoPlan);
    }

    let mut expansions = 0usize;
    let seed_limit = config
        .route_cap
        .max(config.beam_width.max(1).saturating_mul(2))
        .clamp(4, 10)
        .min(TURN_PLANNER_ROUTE_CAP)
        .min(allowed_first_steps.len());
    let mut seeded: Vec<(i64, PlannerNode)> = Vec::new();

    for (seed_rank, step) in allowed_first_steps.iter().take(seed_limit).enumerate() {
        expansions = expansions.saturating_add(1);
        if expansions > node_budget.max(1) {
            update_turn_planner_diagnostics(|diagnostics| {
                diagnostics.expansions = diagnostics.expansions.saturating_add(expansions);
            });
            return Err(PlanBuildStatus::BudgetExceeded);
        }
        let Some((after, _)) = apply_inputs_for_planner(game, step.as_slice()) else {
            continue;
        };
        let rank_bonus = (seed_limit.saturating_sub(seed_rank) as i32).saturating_mul(12);
        let order = quick_node_order_score(
            game,
            &after,
            perspective,
            1,
            PlannerRouteKind::Fallback,
            rank_bonus,
            config,
            utility_memo,
        );
        seeded.push((
            order,
            PlannerNode {
                game: after,
                steps: vec![step.clone()],
            },
        ));
    }

    if seeded.is_empty() {
        return Err(PlanBuildStatus::NoPlan);
    }

    seeded.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| compare_step_chains(a.1.steps.as_slice(), b.1.steps.as_slice()))
    });

    let mut frontier: Vec<PlannerNode> = seeded
        .into_iter()
        .take(beam_width.max(1))
        .map(|(_, node)| node)
        .collect();
    let mut terminal = Vec::new();

    for _ in 1..config.step_cap.max(1) {
        let mut candidates: Vec<(i64, PlannerNode)> = Vec::new();
        let mut expanded_any = false;
        let current_frontier = std::mem::take(&mut frontier);

        for node in current_frontier.into_iter() {
            if node.game.winner_color().is_some() || node.game.active_color != perspective {
                terminal.push(TurnPlan {
                    game: node.game,
                    steps: node.steps,
                });
                continue;
            }

            let routes = collect_atomic_routes(&node.game, perspective, config);
            if routes.is_empty() {
                terminal.push(TurnPlan {
                    game: node.game,
                    steps: node.steps,
                });
                continue;
            }

            for route in routes.into_iter().take(config.per_node_route_cap.max(1)) {
                expansions = expansions.saturating_add(1);
                if expansions > node_budget.max(1) {
                    update_turn_planner_diagnostics(|diagnostics| {
                        diagnostics.expansions = diagnostics.expansions.saturating_add(expansions);
                    });
                    return Err(PlanBuildStatus::BudgetExceeded);
                }

                let Some((after, _)) =
                    apply_inputs_for_planner(&node.game, route.inputs.as_slice())
                else {
                    continue;
                };

                let mut steps = node.steps.clone();
                steps.push(route.inputs.clone());
                let order = quick_node_order_score(
                    game,
                    &after,
                    perspective,
                    steps.len(),
                    route.kind,
                    route.priority,
                    config,
                    utility_memo,
                );
                candidates.push((order, PlannerNode { game: after, steps }));
                expanded_any = true;
            }
        }

        if !expanded_any || candidates.is_empty() {
            break;
        }

        candidates.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| compare_step_chains(a.1.steps.as_slice(), b.1.steps.as_slice()))
        });
        frontier = candidates
            .into_iter()
            .take(beam_width.max(1))
            .map(|(_, node)| node)
            .collect();
    }

    terminal.extend(frontier.into_iter().map(|node| TurnPlan {
        game: node.game,
        steps: node.steps,
    }));

    if terminal.is_empty() {
        return Err(PlanBuildStatus::NoPlan);
    }

    terminal.sort_by(|a, b| {
        let a_utility =
            utility_for_perspective_cached(&a.game, game, perspective, config, utility_memo);
        let b_utility =
            utility_for_perspective_cached(&b.game, game, perspective, config, utility_memo);
        b_utility
            .cmp(&a_utility)
            .then_with(|| compare_step_chains(a.steps.as_slice(), b.steps.as_slice()))
    });

    update_turn_planner_diagnostics(|diagnostics| {
        diagnostics.expansions = diagnostics.expansions.saturating_add(expansions);
    });

    Ok(terminal)
}

fn evaluate_plan_with_response(
    root_game: &MonsGame,
    plan: &TurnPlan,
    perspective: Color,
    config: TurnPlannerSearchConfig,
    utility_memo: &mut HashMap<UtilityMemoKey, PlannerUtility>,
) -> PlannerUtility {
    if plan.game.winner_color().is_some() {
        return utility_for_perspective_cached(
            &plan.game,
            root_game,
            perspective,
            config,
            utility_memo,
        );
    }

    if plan.game.active_color != perspective.other() {
        return utility_for_perspective_cached(
            &plan.game,
            root_game,
            perspective,
            config,
            utility_memo,
        );
    }

    let response_budget = (config.max_nodes / 3).max(20);
    let response_plans = match generate_turn_plans(
        &plan.game,
        perspective.other(),
        config,
        response_budget,
        config.response_beam_width.max(1),
        utility_memo,
    ) {
        Ok(plans) if !plans.is_empty() => plans,
        _ => {
            return utility_for_perspective_cached(
                &plan.game,
                root_game,
                perspective,
                config,
                utility_memo,
            );
        }
    };

    let mut best_response = &response_plans[0];
    let mut best_response_utility = utility_for_perspective_cached(
        &best_response.game,
        &plan.game,
        perspective.other(),
        config,
        utility_memo,
    );

    for response in response_plans.iter().skip(1) {
        let response_utility = utility_for_perspective_cached(
            &response.game,
            &plan.game,
            perspective.other(),
            config,
            utility_memo,
        );
        if response_utility > best_response_utility {
            best_response_utility = response_utility;
            best_response = response;
        } else if response_utility == best_response_utility
            && compare_step_chains(response.steps.as_slice(), best_response.steps.as_slice())
                == Ordering::Less
        {
            best_response = response;
        }
    }

    utility_for_perspective_cached(
        &best_response.game,
        root_game,
        perspective,
        config,
        utility_memo,
    )
}

fn utility_for_perspective(
    game: &MonsGame,
    start: &MonsGame,
    perspective: Color,
    config: TurnPlannerSearchConfig,
) -> PlannerUtility {
    let my_score = score_for_color(game, perspective);
    let start_score = score_for_color(start, perspective);
    let score_delta = my_score.saturating_sub(start_score);

    let strategic = exact_strategic_analysis(game).color_summary(perspective);
    let path_bonus = strategic
        .score_path_window
        .best_steps
        .map(|steps| (Config::BOARD_SIZE * 3 - steps).max(0) * 24)
        .unwrap_or(0);
    let immediate_bonus = strategic.immediate_window.best_score.saturating_mul(130)
        + strategic.immediate_window.multi_pressure.saturating_mul(20);

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
    let drainer_safety = own_drainer_safety_score(&game.board, perspective);
    let unsafe_progress_penalty = if drainer_safety < 0 {
        drainer_safety.saturating_abs().saturating_mul(900)
    } else {
        0
    };
    let opponent = perspective.other();
    let opponent_window_before = exact_turn_summary(start, opponent).same_turn_score_window_value;
    let opponent_window_after = exact_turn_summary(game, opponent).same_turn_score_window_value;
    let opponent_window_deny_gain = opponent_window_before.saturating_sub(opponent_window_after);
    let opponent_needed_before =
        Config::TARGET_SCORE.saturating_sub(score_for_color(start, opponent));
    let opponent_needed_after =
        Config::TARGET_SCORE.saturating_sub(score_for_color(game, opponent));
    let denied_immediate_window = opponent_needed_before > 0
        && opponent_window_before >= opponent_needed_before
        && (opponent_needed_after <= 0 || opponent_window_after < opponent_needed_after);

    let high_value_progress = score_delta
        .saturating_mul(2_400)
        .saturating_add(path_bonus)
        .saturating_add(immediate_bonus)
        .saturating_add(safe_supermana_bonus)
        .saturating_add(safe_opponent_mana_bonus)
        .saturating_add(opponent_window_deny_gain.saturating_mul(210))
        .saturating_add(if denied_immediate_window { 1_500 } else { 0 })
        .saturating_sub(unsafe_progress_penalty);

    let drainer_attack = if find_awake_drainer_location(&game.board, perspective.other()).is_none()
    {
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

    PlannerUtility {
        win_state: winner_state(game, perspective),
        avoid_immediate_loss: if opponent_can_win_immediately(game, perspective) {
            -1
        } else {
            1
        },
        high_value_progress,
        drainer_attack,
        drainer_safety,
        eval_score,
    }
}

fn utility_for_perspective_cached(
    game: &MonsGame,
    start: &MonsGame,
    perspective: Color,
    config: TurnPlannerSearchConfig,
    utility_memo: &mut HashMap<UtilityMemoKey, PlannerUtility>,
) -> PlannerUtility {
    let key = UtilityMemoKey {
        game_hash: MonsGameModel::search_state_hash(game),
        start_hash: MonsGameModel::search_state_hash(start),
        perspective,
    };
    if let Some(cached) = utility_memo.get(&key) {
        return *cached;
    }

    let computed = utility_for_perspective(game, start, perspective, config);
    if utility_memo.len() >= TURN_PLANNER_UTILITY_MEMO_MAX_ENTRIES {
        utility_memo.clear();
    }
    utility_memo.insert(key, computed);
    computed
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

fn own_drainer_carries_safe_mana(board: &Board, color: Color, wanted: Mana) -> bool {
    let Some(drainer_location) = find_awake_drainer_location(board, color) else {
        return false;
    };
    matches!(
        board.item(drainer_location),
        Some(Item::MonWithMana { mana, .. }) if *mana == wanted
    ) && is_drainer_exactly_safe_next_turn_on_board(board, color, drainer_location)
}

fn quick_node_order_score(
    root: &MonsGame,
    game: &MonsGame,
    perspective: Color,
    step_len: usize,
    route_kind: PlannerRouteKind,
    route_priority: i32,
    config: TurnPlannerSearchConfig,
    utility_memo: &mut HashMap<UtilityMemoKey, PlannerUtility>,
) -> i64 {
    let utility = utility_for_perspective_cached(game, root, perspective, config, utility_memo);
    let kind_bonus = match route_kind {
        PlannerRouteKind::ModelTactical => 980,
        PlannerRouteKind::DrainerScore => 900,
        PlannerRouteKind::DrainerKill => 860,
        PlannerRouteKind::TacticalDeny => 830,
        PlannerRouteKind::SpiritImpact => 740,
        PlannerRouteKind::DrainerSafety => 700,
        PlannerRouteKind::ManaMove => 520,
        PlannerRouteKind::Fallback => 80,
    };

    i64::from(utility.win_state) * 10_000_000
        + i64::from(utility.avoid_immediate_loss) * 5_000_000
        + i64::from(route_priority + kind_bonus) * 5_000
        + i64::from(utility.high_value_progress)
        + i64::from(utility.drainer_attack) * 3_000
        + i64::from(utility.drainer_safety) * 2_000
        + i64::from(utility.eval_score / 8)
        - step_len as i64 * 300
}

fn compare_step_chains(a: &[Vec<Input>], b: &[Vec<Input>]) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

fn collect_atomic_routes(
    game: &MonsGame,
    perspective: Color,
    config: TurnPlannerSearchConfig,
) -> Vec<PlannerRoute> {
    let mut routes = Vec::new();
    routes.extend(intent_first_routes(game, perspective, config));
    routes.extend(model_tactical_routes(game, perspective, config));
    routes.extend(drainer_score_routes(game, perspective));
    routes.extend(drainer_kill_routes(game, perspective));
    routes.extend(tactical_deny_routes(game, perspective, config));
    routes.extend(spirit_impact_routes(game, perspective));
    routes.extend(drainer_safety_routes(game, perspective));
    routes.extend(mana_move_routes(game, perspective));

    if routes.is_empty() {
        if let Some(inputs) = first_legal_atomic_step(game) {
            routes.push(PlannerRoute {
                inputs,
                kind: PlannerRouteKind::Fallback,
                priority: 1,
            });
        }
    }

    let mut dedup = HashSet::new();
    routes.retain(|route| dedup.insert(route.inputs.clone()));

    routes.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.inputs.cmp(&b.inputs))
    });
    if routes.len() > config.route_cap.max(1).min(TURN_PLANNER_ROUTE_CAP) {
        routes.truncate(config.route_cap.max(1).min(TURN_PLANNER_ROUTE_CAP));
    }
    record_route_family_contribution(routes.as_slice());
    routes
}

fn record_route_family_contribution(routes: &[PlannerRoute]) {
    let mut model_tactical = 0usize;
    let mut drainer_score = 0usize;
    let mut drainer_kill = 0usize;
    let mut spirit_impact = 0usize;
    let mut drainer_safety = 0usize;
    let mut mana_move = 0usize;
    let mut tactical_deny = 0usize;
    let mut fallback = 0usize;

    for route in routes {
        match route.kind {
            PlannerRouteKind::ModelTactical => model_tactical = model_tactical.saturating_add(1),
            PlannerRouteKind::DrainerScore => drainer_score = drainer_score.saturating_add(1),
            PlannerRouteKind::DrainerKill => drainer_kill = drainer_kill.saturating_add(1),
            PlannerRouteKind::SpiritImpact => spirit_impact = spirit_impact.saturating_add(1),
            PlannerRouteKind::DrainerSafety => drainer_safety = drainer_safety.saturating_add(1),
            PlannerRouteKind::ManaMove => mana_move = mana_move.saturating_add(1),
            PlannerRouteKind::TacticalDeny => tactical_deny = tactical_deny.saturating_add(1),
            PlannerRouteKind::Fallback => fallback = fallback.saturating_add(1),
        }
    }

    update_turn_planner_diagnostics(|diagnostics| {
        diagnostics.route_model_tactical = diagnostics
            .route_model_tactical
            .saturating_add(model_tactical);
        diagnostics.route_drainer_score = diagnostics
            .route_drainer_score
            .saturating_add(drainer_score);
        diagnostics.route_drainer_kill =
            diagnostics.route_drainer_kill.saturating_add(drainer_kill);
        diagnostics.route_spirit_impact = diagnostics
            .route_spirit_impact
            .saturating_add(spirit_impact);
        diagnostics.route_drainer_safety = diagnostics
            .route_drainer_safety
            .saturating_add(drainer_safety);
        diagnostics.route_mana_move = diagnostics.route_mana_move.saturating_add(mana_move);
        diagnostics.route_tactical_deny = diagnostics
            .route_tactical_deny
            .saturating_add(tactical_deny);
        diagnostics.route_fallback = diagnostics.route_fallback.saturating_add(fallback);
    });
}

fn current_turn_resource_budget(game: &MonsGame, perspective: Color) -> TurnResourceBudget {
    if game.active_color != perspective {
        return TurnResourceBudget {
            remaining_mon_moves: 0,
            can_use_action: false,
            can_move_mana: false,
        };
    }

    TurnResourceBudget {
        remaining_mon_moves: remaining_moves_for_color(game, perspective),
        can_use_action: game.player_can_use_action(),
        can_move_mana: game.player_can_move_mana(),
    }
}

fn budget_satisfies_requirement(
    budget: TurnResourceBudget,
    requirement: TurnResourceRequirement,
) -> bool {
    requirement.mon_moves_needed <= budget.remaining_mon_moves
        && (!requirement.needs_action || budget.can_use_action)
        && (!requirement.needs_mana_move || budget.can_move_mana)
}

fn intent_first_routes(
    game: &MonsGame,
    perspective: Color,
    config: TurnPlannerSearchConfig,
) -> Vec<PlannerRoute> {
    let budget = current_turn_resource_budget(game, perspective);
    let mut intents = planner_intents(game, perspective, budget);
    if intents.is_empty() {
        update_turn_planner_diagnostics(|diagnostics| {
            diagnostics.intent_generation_calls =
                diagnostics.intent_generation_calls.saturating_add(1);
        });
        return Vec::new();
    }

    intents.sort_by(|left, right| {
        intent_priority(*right)
            .cmp(&intent_priority(*left))
            .then_with(|| intent_stable_key(*left).cmp(&intent_stable_key(*right)))
    });
    if intents.len() > TURN_PLANNER_INTENT_CAP {
        intents.truncate(TURN_PLANNER_INTENT_CAP);
    }

    update_turn_planner_diagnostics(|diagnostics| {
        diagnostics.intent_generation_calls = diagnostics.intent_generation_calls.saturating_add(1);
        diagnostics.intent_generation_hits = diagnostics
            .intent_generation_hits
            .saturating_add(intents.len());
    });

    let route_cap = config.route_cap.max(1).min(TURN_PLANNER_ROUTE_CAP);
    let mut routes = Vec::new();
    for intent in intents {
        if routes.len() >= route_cap {
            break;
        }
        if !budget_satisfies_requirement(budget, intent_requirement(intent)) {
            continue;
        }
        if let Some(route) = compile_intent_route(game, perspective, intent) {
            routes.push(route);
        }
    }
    routes
}

fn intent_requirement(intent: PlannerIntent) -> TurnResourceRequirement {
    match intent {
        PlannerIntent::SecureMana(intent) => intent.resources,
        PlannerIntent::DrainerKill(intent) => intent.resources,
        PlannerIntent::SafetyRecover(intent) => intent.resources,
        PlannerIntent::SpiritImpact(intent) => intent.resources,
        PlannerIntent::ManaTempo(intent) => intent.resources,
        PlannerIntent::TacticalDeny(intent) => intent.resources,
    }
}

fn intent_stable_key(
    intent: PlannerIntent,
) -> (
    PlannerRouteKind,
    Location,
    Option<Location>,
    Option<Location>,
) {
    match intent {
        PlannerIntent::SecureMana(intent) => (
            PlannerRouteKind::DrainerScore,
            intent.actor,
            intent.next_step,
            None,
        ),
        PlannerIntent::DrainerKill(intent) => (
            PlannerRouteKind::DrainerKill,
            intent.actor,
            Some(intent.target),
            intent.setup_step,
        ),
        PlannerIntent::SafetyRecover(intent) => (
            PlannerRouteKind::DrainerSafety,
            intent.actor,
            Some(intent.target_step),
            None,
        ),
        PlannerIntent::SpiritImpact(intent) => (
            PlannerRouteKind::SpiritImpact,
            intent.actor,
            Some(intent.target),
            Some(intent.destination),
        ),
        PlannerIntent::ManaTempo(intent) => (
            PlannerRouteKind::ManaMove,
            intent.actor,
            Some(intent.destination),
            None,
        ),
        PlannerIntent::TacticalDeny(intent) => (
            PlannerRouteKind::TacticalDeny,
            intent.actor,
            Some(intent.target),
            intent.setup_step,
        ),
    }
}

fn intent_priority(intent: PlannerIntent) -> i32 {
    match intent {
        PlannerIntent::SecureMana(intent) => {
            9_200
                + intent.estimated_gain.saturating_mul(240)
                + intent.safety_delta.saturating_mul(80)
        }
        PlannerIntent::DrainerKill(intent) => {
            8_900
                + intent.estimated_gain.saturating_mul(260)
                + intent.safety_delta.saturating_mul(60)
        }
        PlannerIntent::SafetyRecover(intent) => {
            8_300
                + intent.estimated_gain.saturating_mul(220)
                + intent.safety_delta.saturating_mul(180)
        }
        PlannerIntent::SpiritImpact(intent) => {
            7_600
                + intent.estimated_gain.saturating_mul(200)
                + intent.safety_delta.saturating_mul(60)
        }
        PlannerIntent::ManaTempo(intent) => {
            6_900
                + intent.estimated_gain.saturating_mul(180)
                + intent.safety_delta.saturating_mul(60)
        }
        PlannerIntent::TacticalDeny(intent) => {
            9_100
                + intent.estimated_gain.saturating_mul(250)
                + intent.safety_delta.saturating_mul(90)
        }
    }
}

fn planner_intents(
    game: &MonsGame,
    perspective: Color,
    budget: TurnResourceBudget,
) -> Vec<PlannerIntent> {
    let mut intents = Vec::new();
    intents.extend(secure_mana_intents(game, perspective));
    intents.extend(drainer_kill_intents(game, perspective, budget));
    intents.extend(safety_recover_intents(game, perspective));
    intents.extend(spirit_impact_intents(game, perspective));
    intents.extend(mana_tempo_intents(game, perspective));
    intents.extend(tactical_deny_intents(game, perspective, budget));
    intents
}

fn secure_mana_intents(game: &MonsGame, perspective: Color) -> Vec<PlannerIntent> {
    let mut intents = Vec::new();
    let Some(drainer_location) = find_awake_drainer_location(&game.board, perspective) else {
        return intents;
    };
    let safety_before = own_drainer_safety_score(&game.board, perspective);
    let wanted_manas = [Mana::Supermana, Mana::Regular(perspective.other())];

    for wanted in wanted_manas {
        let Some(path) =
            exact_secure_specific_mana_path_from(game, perspective, drainer_location, wanted)
        else {
            continue;
        };
        let estimated_gain = if wanted == Mana::Supermana { 6 } else { 5 };
        let safety_delta = if safety_before < 0 { 1 } else { 0 };
        let next_step = path.first().copied();
        intents.push(PlannerIntent::SecureMana(SecureManaIntent {
            actor: drainer_location,
            wanted,
            next_step,
            estimated_gain,
            safety_delta,
            resources: TurnResourceRequirement {
                mon_moves_needed: if next_step.is_some() { 1 } else { 0 },
                needs_action: false,
                needs_mana_move: false,
            },
        }));
    }

    intents
}

fn drainer_kill_intents(
    game: &MonsGame,
    perspective: Color,
    budget: TurnResourceBudget,
) -> Vec<PlannerIntent> {
    let mut intents = Vec::new();
    let Some(target_drainer) = find_awake_drainer_location(&game.board, perspective.other()) else {
        return intents;
    };
    if !budget.can_use_action {
        return intents;
    }

    for (actor, item) in game.board.occupied() {
        let Some(mon) = item.mon().copied() else {
            continue;
        };
        if mon.color != perspective || mon.is_fainted() {
            continue;
        }

        if can_attack_target_on_board(
            &game.board,
            perspective,
            perspective.other(),
            target_drainer,
            remaining_moves_for_color(game, perspective),
            budget.can_use_action,
        ) {
            intents.push(PlannerIntent::DrainerKill(DrainerKillIntent {
                actor,
                target: target_drainer,
                setup_step: None,
                estimated_gain: 7,
                safety_delta: 1,
                resources: TurnResourceRequirement {
                    mon_moves_needed: 0,
                    needs_action: true,
                    needs_mana_move: false,
                },
            }));
        }

        if budget.remaining_mon_moves <= 0 {
            continue;
        }
        for &next in actor.nearby_locations_ref() {
            let Some(inputs) = compile_seeded_inputs_with_mode(
                game,
                vec![Input::Location(actor), Input::Location(next)],
                IntentCompileMode::Path,
            ) else {
                continue;
            };
            let Some((after, _)) = apply_inputs_for_planner(game, inputs.as_slice()) else {
                continue;
            };
            if after.active_color != perspective {
                continue;
            }
            if can_attack_target_on_board(
                &after.board,
                perspective,
                perspective.other(),
                target_drainer,
                remaining_moves_for_color(&after, perspective),
                after.player_can_use_action(),
            ) {
                intents.push(PlannerIntent::DrainerKill(DrainerKillIntent {
                    actor,
                    target: target_drainer,
                    setup_step: Some(next),
                    estimated_gain: 6,
                    safety_delta: 0,
                    resources: TurnResourceRequirement {
                        mon_moves_needed: 1,
                        needs_action: true,
                        needs_mana_move: false,
                    },
                }));
            }
        }
    }

    intents
}

fn safety_recover_intents(game: &MonsGame, perspective: Color) -> Vec<PlannerIntent> {
    let mut intents = Vec::new();
    let Some(drainer_location) = find_awake_drainer_location(&game.board, perspective) else {
        return intents;
    };
    let safety_before = own_drainer_safety_score(&game.board, perspective);
    if safety_before >= 2 {
        return intents;
    }

    for &next in drainer_location.nearby_locations_ref() {
        let Some(inputs) = compile_seeded_inputs_with_mode(
            game,
            vec![Input::Location(drainer_location), Input::Location(next)],
            IntentCompileMode::Path,
        ) else {
            continue;
        };
        let Some((after, _)) = apply_inputs_for_planner(game, inputs.as_slice()) else {
            continue;
        };
        let safety_after = own_drainer_safety_score(&after.board, perspective);
        if safety_after <= safety_before {
            continue;
        }
        intents.push(PlannerIntent::SafetyRecover(SafetyRecoverIntent {
            actor: drainer_location,
            target_step: next,
            estimated_gain: safety_after.saturating_sub(safety_before),
            safety_delta: safety_after.saturating_sub(safety_before),
            resources: TurnResourceRequirement {
                mon_moves_needed: 1,
                needs_action: false,
                needs_mana_move: false,
            },
        }));
    }

    intents
}

fn spirit_impact_intents(game: &MonsGame, perspective: Color) -> Vec<PlannerIntent> {
    if !game.player_can_use_action() {
        return Vec::new();
    }

    let opponent_drainer = find_awake_drainer_location(&game.board, perspective.other());
    let mut spirit_intents: Vec<SpiritImpactIntent> = Vec::new();
    for (spirit_location, item) in game.board.occupied() {
        let Some(mon) = item.mon().copied() else {
            continue;
        };
        if mon.color != perspective || mon.kind != MonKind::Spirit || mon.is_fainted() {
            continue;
        }

        for &target in spirit_location.reachable_by_spirit_action_ref() {
            for &destination in target.nearby_locations_ref() {
                let mut estimated_gain: i32 = 2;
                let mut safety_delta: i32 = 0;
                if opponent_drainer == Some(target) {
                    estimated_gain = estimated_gain.saturating_add(4);
                }
                if let Some(item) = game.board.item(target) {
                    if let Some(mon) = item.mon() {
                        if mon.color == perspective.other() {
                            estimated_gain = estimated_gain.saturating_add(2);
                        }
                    }
                    if matches!(item, Item::MonWithMana { mon, .. } if mon.color == perspective.other())
                    {
                        estimated_gain = estimated_gain.saturating_add(2);
                        safety_delta = safety_delta.saturating_add(1);
                    }
                }
                spirit_intents.push(SpiritImpactIntent {
                    actor: spirit_location,
                    target,
                    destination,
                    estimated_gain,
                    safety_delta,
                    resources: TurnResourceRequirement {
                        mon_moves_needed: 0,
                        needs_action: true,
                        needs_mana_move: false,
                    },
                });
            }
        }
    }

    spirit_intents.sort_by(|left, right| {
        right
            .estimated_gain
            .cmp(&left.estimated_gain)
            .then_with(|| left.actor.cmp(&right.actor))
            .then_with(|| left.target.cmp(&right.target))
            .then_with(|| left.destination.cmp(&right.destination))
    });
    if spirit_intents.len() > TURN_PLANNER_SPIRIT_TOP_K {
        spirit_intents.truncate(TURN_PLANNER_SPIRIT_TOP_K);
    }
    spirit_intents
        .into_iter()
        .map(PlannerIntent::SpiritImpact)
        .collect()
}

fn mana_tempo_intents(game: &MonsGame, perspective: Color) -> Vec<PlannerIntent> {
    if !game.player_can_move_mana() {
        return Vec::new();
    }

    let mut intents = Vec::new();
    for (mana_location, item) in game.board.occupied() {
        let Item::Mana { mana } = item else {
            continue;
        };
        if *mana != Mana::Regular(perspective) {
            continue;
        }

        for &destination in mana_location.nearby_locations_ref() {
            let own_before = distance_to_nearest_pool(mana_location, perspective);
            let own_after = distance_to_nearest_pool(destination, perspective);
            let opp_before = distance_to_nearest_pool(mana_location, perspective.other());
            let opp_after = distance_to_nearest_pool(destination, perspective.other());
            let own_gain = own_before.saturating_sub(own_after);
            let opp_gain = opp_before.saturating_sub(opp_after);
            if own_gain <= 0 || opp_gain > 0 {
                continue;
            }

            intents.push(PlannerIntent::ManaTempo(ManaTempoIntent {
                actor: mana_location,
                destination,
                estimated_gain: own_gain.saturating_sub(opp_gain.min(0)),
                safety_delta: 0,
                resources: TurnResourceRequirement {
                    mon_moves_needed: 0,
                    needs_action: false,
                    needs_mana_move: true,
                },
            }));
        }
    }
    intents
}

fn tactical_deny_intents(
    game: &MonsGame,
    perspective: Color,
    budget: TurnResourceBudget,
) -> Vec<PlannerIntent> {
    if !planner_tactical_emergency_state(game, perspective) {
        return Vec::new();
    }

    let mut intents = Vec::new();
    let Some(target_drainer) = find_awake_drainer_location(&game.board, perspective.other()) else {
        return intents;
    };
    let deny_pressure = exact_turn_summary(game, perspective.other()).same_turn_score_window_value;

    for (actor, item) in game.board.occupied() {
        let Some(mon) = item.mon().copied() else {
            continue;
        };
        if mon.color != perspective || mon.is_fainted() {
            continue;
        }

        if budget.can_use_action
            && can_attack_target_on_board(
                &game.board,
                perspective,
                perspective.other(),
                target_drainer,
                remaining_moves_for_color(game, perspective),
                budget.can_use_action,
            )
        {
            intents.push(PlannerIntent::TacticalDeny(TacticalDenyIntent {
                actor,
                target: target_drainer,
                setup_step: None,
                estimated_gain: deny_pressure.saturating_add(5),
                safety_delta: 1,
                resources: TurnResourceRequirement {
                    mon_moves_needed: 0,
                    needs_action: true,
                    needs_mana_move: false,
                },
            }));
        }

        if budget.remaining_mon_moves <= 0 || !budget.can_use_action {
            continue;
        }
        for &next in actor.nearby_locations_ref() {
            let Some(inputs) = compile_seeded_inputs_with_mode(
                game,
                vec![Input::Location(actor), Input::Location(next)],
                IntentCompileMode::Path,
            ) else {
                continue;
            };
            let Some((after, _)) = apply_inputs_for_planner(game, inputs.as_slice()) else {
                continue;
            };
            if after.active_color != perspective {
                continue;
            }
            if can_attack_target_on_board(
                &after.board,
                perspective,
                perspective.other(),
                target_drainer,
                remaining_moves_for_color(&after, perspective),
                after.player_can_use_action(),
            ) {
                intents.push(PlannerIntent::TacticalDeny(TacticalDenyIntent {
                    actor,
                    target: target_drainer,
                    setup_step: Some(next),
                    estimated_gain: deny_pressure.saturating_add(4),
                    safety_delta: 0,
                    resources: TurnResourceRequirement {
                        mon_moves_needed: 1,
                        needs_action: true,
                        needs_mana_move: false,
                    },
                }));
            }
        }
    }

    intents
}

fn compile_intent_route(
    game: &MonsGame,
    perspective: Color,
    intent: PlannerIntent,
) -> Option<PlannerRoute> {
    let (kind, priority, inputs) = match intent {
        PlannerIntent::SecureMana(intent) => {
            let inputs = if let Some(step) = intent.next_step {
                compile_intent_inputs_with_mode(
                    game,
                    vec![Input::Location(intent.actor), Input::Location(step)],
                    IntentCompileMode::Path,
                )
            } else {
                best_drainer_pool_step(game, perspective, intent.actor, intent.wanted)
            }?;
            (
                PlannerRouteKind::DrainerScore,
                intent_priority(PlannerIntent::SecureMana(intent)),
                inputs,
            )
        }
        PlannerIntent::DrainerKill(intent) => {
            let seed = if let Some(step) = intent.setup_step {
                vec![Input::Location(intent.actor), Input::Location(step)]
            } else {
                vec![
                    Input::Location(intent.actor),
                    Input::Location(intent.target),
                ]
            };
            let inputs = compile_intent_inputs_with_mode(game, seed, IntentCompileMode::Attack)?;
            (
                PlannerRouteKind::DrainerKill,
                intent_priority(PlannerIntent::DrainerKill(intent)),
                inputs,
            )
        }
        PlannerIntent::SafetyRecover(intent) => {
            let inputs = compile_intent_inputs_with_mode(
                game,
                vec![
                    Input::Location(intent.actor),
                    Input::Location(intent.target_step),
                ],
                IntentCompileMode::Path,
            )?;
            (
                PlannerRouteKind::DrainerSafety,
                intent_priority(PlannerIntent::SafetyRecover(intent)),
                inputs,
            )
        }
        PlannerIntent::SpiritImpact(intent) => {
            let inputs = compile_intent_inputs_with_mode(
                game,
                vec![
                    Input::Location(intent.actor),
                    Input::Location(intent.target),
                    Input::Location(intent.destination),
                ],
                IntentCompileMode::Spirit,
            )?;
            (
                PlannerRouteKind::SpiritImpact,
                intent_priority(PlannerIntent::SpiritImpact(intent)),
                inputs,
            )
        }
        PlannerIntent::ManaTempo(intent) => {
            let inputs = compile_intent_inputs_with_mode(
                game,
                vec![
                    Input::Location(intent.actor),
                    Input::Location(intent.destination),
                ],
                IntentCompileMode::Path,
            )?;
            (
                PlannerRouteKind::ManaMove,
                intent_priority(PlannerIntent::ManaTempo(intent)),
                inputs,
            )
        }
        PlannerIntent::TacticalDeny(intent) => {
            let seed = if let Some(step) = intent.setup_step {
                vec![Input::Location(intent.actor), Input::Location(step)]
            } else {
                vec![
                    Input::Location(intent.actor),
                    Input::Location(intent.target),
                ]
            };
            let inputs = compile_intent_inputs_with_mode(game, seed, IntentCompileMode::Attack)?;
            (
                PlannerRouteKind::TacticalDeny,
                intent_priority(PlannerIntent::TacticalDeny(intent)),
                inputs,
            )
        }
    };

    Some(PlannerRoute {
        inputs,
        kind,
        priority,
    })
}

fn compile_intent_inputs_with_mode(
    game: &MonsGame,
    seed: Vec<Input>,
    mode: IntentCompileMode,
) -> Option<Vec<Input>> {
    if let Some(inputs) = compile_seeded_inputs_with_mode(game, seed.clone(), mode) {
        if apply_inputs_for_planner(game, inputs.as_slice()).is_some() {
            return Some(inputs);
        }
    }

    update_turn_planner_diagnostics(|diagnostics| {
        diagnostics.compile_fallbacks = diagnostics.compile_fallbacks.saturating_add(1);
    });
    let fallback = compile_seeded_inputs(game, seed)?;
    apply_inputs_for_planner(game, fallback.as_slice())?;
    Some(fallback)
}

fn tactical_deny_routes(
    game: &MonsGame,
    perspective: Color,
    config: TurnPlannerSearchConfig,
) -> Vec<PlannerRoute> {
    if !planner_tactical_emergency_state(game, perspective) {
        return Vec::new();
    }

    let mut seeds = Vec::new();
    seeds.extend(model_tactical_routes(
        game,
        perspective,
        TurnPlannerSearchConfig {
            route_cap: config.route_cap.clamp(2, 8),
            per_node_route_cap: config.per_node_route_cap.clamp(2, 4),
            ..config
        },
    ));
    seeds.extend(drainer_kill_routes(game, perspective));
    seeds.extend(drainer_safety_routes(game, perspective));

    let opponent_window_before =
        exact_turn_summary(game, perspective.other()).same_turn_score_window_value;
    let mut routes = Vec::new();
    let mut seen = HashSet::new();
    for seed in seeds {
        if !seen.insert(seed.inputs.clone()) {
            continue;
        }
        let Some((after, _)) = apply_inputs_for_planner(game, seed.inputs.as_slice()) else {
            continue;
        };
        let opponent_window_after =
            exact_turn_summary(&after, perspective.other()).same_turn_score_window_value;
        let deny_gain = opponent_window_before.saturating_sub(opponent_window_after);
        if deny_gain <= 0
            && opponent_can_win_immediately(game, perspective)
                == opponent_can_win_immediately(&after, perspective)
        {
            continue;
        }
        routes.push(PlannerRoute {
            inputs: seed.inputs,
            kind: PlannerRouteKind::TacticalDeny,
            priority: 8_600
                + deny_gain.saturating_mul(260)
                + if opponent_can_win_immediately(game, perspective)
                    && !opponent_can_win_immediately(&after, perspective)
                {
                    1_400
                } else {
                    0
                },
        });
    }
    routes
}

fn model_tactical_routes(
    game: &MonsGame,
    perspective: Color,
    config: TurnPlannerSearchConfig,
) -> Vec<PlannerRoute> {
    let candidate_cap = config.per_node_route_cap.clamp(2, 12);
    let mut routes = Vec::new();
    let mut tactical_inputs =
        MonsGameModel::turn_planner_targeted_inputs(game, perspective, candidate_cap);
    tactical_inputs.sort();
    tactical_inputs.dedup();

    let before_score = score_for_color(game, perspective);
    for inputs in tactical_inputs.into_iter().take(candidate_cap) {
        let Some((after, events)) = apply_inputs_for_planner(game, inputs.as_slice()) else {
            continue;
        };
        let score_gain = score_for_color(&after, perspective).saturating_sub(before_score);
        let safety = own_drainer_safety_score(&after.board, perspective);
        let kill_bonus = if events_include_opponent_drainer_faint(events.as_slice(), perspective) {
            900
        } else {
            0
        };
        let immediate_reply_penalty = if opponent_can_win_immediately(&after, perspective) {
            1_800
        } else {
            0
        };
        let priority =
            8_200 + score_gain.saturating_mul(220) + safety.saturating_mul(140) + kill_bonus
                - immediate_reply_penalty;
        routes.push(PlannerRoute {
            inputs,
            kind: PlannerRouteKind::ModelTactical,
            priority,
        });
    }

    routes
}

fn drainer_score_routes(game: &MonsGame, perspective: Color) -> Vec<PlannerRoute> {
    let mut routes = Vec::new();
    let Some(drainer_location) = find_awake_drainer_location(&game.board, perspective) else {
        return routes;
    };

    let wanted_manas = [Mana::Supermana, Mana::Regular(perspective.other())];
    for wanted in wanted_manas {
        let Some(path) =
            exact_secure_specific_mana_path_from(game, perspective, drainer_location, wanted)
        else {
            continue;
        };

        if let Some(next_step) = path.first().copied() {
            if let Some(inputs) = compile_seeded_inputs(
                game,
                vec![
                    Input::Location(drainer_location),
                    Input::Location(next_step),
                ],
            ) {
                let priority = if wanted == Mana::Supermana {
                    9_800
                } else {
                    9_200
                };
                routes.push(PlannerRoute {
                    inputs,
                    kind: PlannerRouteKind::DrainerScore,
                    priority,
                });
            }
        } else if let Some(inputs) =
            best_drainer_pool_step(game, perspective, drainer_location, wanted)
        {
            let priority = if wanted == Mana::Supermana {
                9_500
            } else {
                8_900
            };
            routes.push(PlannerRoute {
                inputs,
                kind: PlannerRouteKind::DrainerScore,
                priority,
            });
        }
    }

    routes
}

fn best_drainer_pool_step(
    game: &MonsGame,
    perspective: Color,
    drainer_location: Location,
    wanted: Mana,
) -> Option<Vec<Input>> {
    let Some(Item::MonWithMana { mana, .. }) = game.board.item(drainer_location) else {
        return None;
    };
    if *mana != wanted {
        return None;
    }

    let before_score = score_for_color(game, perspective);
    let before_dist = distance_to_nearest_pool(drainer_location, perspective);
    let mut best: Option<(i32, Vec<Input>)> = None;

    for &next in drainer_location.nearby_locations_ref() {
        let Some(inputs) = compile_seeded_inputs(
            game,
            vec![Input::Location(drainer_location), Input::Location(next)],
        ) else {
            continue;
        };
        let Some((after, _)) = apply_inputs_for_planner(game, inputs.as_slice()) else {
            continue;
        };
        let score_gain = score_for_color(&after, perspective).saturating_sub(before_score);
        let after_drainer =
            find_awake_drainer_location(&after.board, perspective).unwrap_or(drainer_location);
        let dist_gain =
            before_dist.saturating_sub(distance_to_nearest_pool(after_drainer, perspective));
        let metric = score_gain
            .saturating_mul(500)
            .saturating_add(dist_gain.saturating_mul(30));
        if best
            .as_ref()
            .map_or(true, |(best_metric, _)| metric > *best_metric)
        {
            best = Some((metric, inputs));
        }
    }

    best.map(|(_, inputs)| inputs)
}

fn drainer_kill_routes(game: &MonsGame, perspective: Color) -> Vec<PlannerRoute> {
    let mut routes = Vec::new();
    let Some(target_drainer) = find_awake_drainer_location(&game.board, perspective.other()) else {
        return routes;
    };

    for (start_location, item) in game.board.occupied() {
        let Some(mon) = item.mon().copied() else {
            continue;
        };
        if mon.color != perspective || mon.is_fainted() {
            continue;
        }

        if let Some(inputs) = compile_seeded_inputs(
            game,
            vec![
                Input::Location(start_location),
                Input::Location(target_drainer),
            ],
        ) {
            if let Some((_, events)) = apply_inputs_for_planner(game, inputs.as_slice()) {
                if events_include_opponent_drainer_faint(events.as_slice(), perspective) {
                    routes.push(PlannerRoute {
                        inputs,
                        kind: PlannerRouteKind::DrainerKill,
                        priority: 8_700,
                    });
                }
            }
        }

        for &next in start_location.nearby_locations_ref() {
            let Some(inputs) = compile_seeded_inputs(
                game,
                vec![Input::Location(start_location), Input::Location(next)],
            ) else {
                continue;
            };
            let Some((after, _)) = apply_inputs_for_planner(game, inputs.as_slice()) else {
                continue;
            };
            if after.active_color != perspective {
                continue;
            }
            if can_attack_target_on_board(
                &after.board,
                perspective,
                perspective.other(),
                target_drainer,
                remaining_moves_for_color(&after, perspective),
                after.player_can_use_action(),
            ) {
                routes.push(PlannerRoute {
                    inputs,
                    kind: PlannerRouteKind::DrainerKill,
                    priority: 7_700,
                });
            }
        }
    }

    routes
}

fn spirit_impact_routes(game: &MonsGame, perspective: Color) -> Vec<PlannerRoute> {
    if !game.player_can_use_action() {
        return Vec::new();
    }

    let mut scored = Vec::new();
    let before_score = score_for_color(game, perspective);

    for (spirit_location, item) in game.board.occupied() {
        let Some(mon) = item.mon().copied() else {
            continue;
        };
        if mon.color != perspective || mon.kind != MonKind::Spirit || mon.is_fainted() {
            continue;
        }

        for &target in spirit_location.reachable_by_spirit_action_ref() {
            for &dest in target.nearby_locations_ref() {
                let Some(inputs) = compile_seeded_inputs(
                    game,
                    vec![
                        Input::Location(spirit_location),
                        Input::Location(target),
                        Input::Location(dest),
                    ],
                ) else {
                    continue;
                };
                let Some((after, events)) = apply_inputs_for_planner(game, inputs.as_slice())
                else {
                    continue;
                };
                if !events_include_spirit_move(events.as_slice()) {
                    continue;
                }

                let score_gain = score_for_color(&after, perspective).saturating_sub(before_score);
                let same_turn_window = if after.active_color == perspective {
                    exact_turn_summary(&after, perspective).same_turn_score_window_value
                } else {
                    0
                };
                let strategic_gain = exact_strategic_analysis(&after)
                    .color_summary(perspective)
                    .spirit
                    .next_turn_setup_gain;
                let utility = score_gain
                    .saturating_mul(380)
                    .saturating_add(same_turn_window.saturating_mul(180))
                    .saturating_add(strategic_gain.saturating_mul(90));

                scored.push((
                    utility,
                    PlannerRoute {
                        inputs,
                        kind: PlannerRouteKind::SpiritImpact,
                        priority: 7_000 + utility,
                    },
                ));
            }
        }
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.inputs.cmp(&b.1.inputs)));
    scored
        .into_iter()
        .take(TURN_PLANNER_SPIRIT_TOP_K)
        .map(|(_, route)| route)
        .collect()
}

fn drainer_safety_routes(game: &MonsGame, perspective: Color) -> Vec<PlannerRoute> {
    let mut routes = Vec::new();
    let Some(drainer_location) = find_awake_drainer_location(&game.board, perspective) else {
        return routes;
    };

    let safety_before = own_drainer_safety_score(&game.board, perspective);
    if safety_before >= 2 {
        return routes;
    }

    for &next in drainer_location.nearby_locations_ref() {
        let Some(inputs) = compile_seeded_inputs(
            game,
            vec![Input::Location(drainer_location), Input::Location(next)],
        ) else {
            continue;
        };
        let Some((after, events)) = apply_inputs_for_planner(game, inputs.as_slice()) else {
            continue;
        };

        let safety_after = own_drainer_safety_score(&after.board, perspective);
        if safety_after <= safety_before {
            continue;
        }

        let attack_bonus = if events_include_opponent_drainer_faint(events.as_slice(), perspective)
        {
            100
        } else {
            0
        };
        let priority = 6_900 + safety_after.saturating_mul(220) + attack_bonus;
        routes.push(PlannerRoute {
            inputs,
            kind: PlannerRouteKind::DrainerSafety,
            priority,
        });
    }

    routes
}

fn mana_move_routes(game: &MonsGame, perspective: Color) -> Vec<PlannerRoute> {
    if !game.player_can_move_mana() {
        return Vec::new();
    }

    let mut routes = Vec::new();

    for (mana_location, item) in game.board.occupied() {
        let Item::Mana { mana } = item else {
            continue;
        };
        if *mana != Mana::Regular(perspective) {
            continue;
        }

        for &next in mana_location.nearby_locations_ref() {
            let Some(inputs) = compile_seeded_inputs(
                game,
                vec![Input::Location(mana_location), Input::Location(next)],
            ) else {
                continue;
            };
            let Some((after, events)) = apply_inputs_for_planner(game, inputs.as_slice()) else {
                continue;
            };
            let Some(progress) =
                mana_progress_without_opponent_help(events.as_slice(), perspective)
            else {
                continue;
            };

            let tactical = if after.active_color == perspective {
                exact_turn_summary(&after, perspective).same_turn_score_window_value
            } else {
                0
            };

            routes.push(PlannerRoute {
                inputs,
                kind: PlannerRouteKind::ManaMove,
                priority: 5_200 + progress.saturating_mul(80) + tactical.saturating_mul(30),
            });
        }
    }

    routes
}

fn mana_progress_without_opponent_help(events: &[Event], perspective: Color) -> Option<i32> {
    let Event::ManaMove { from, to, .. } = events
        .iter()
        .find(|event| matches!(event, Event::ManaMove { .. }))?
    else {
        return None;
    };

    let own_before = distance_to_nearest_pool(*from, perspective);
    let own_after = distance_to_nearest_pool(*to, perspective);
    let opp_before = distance_to_nearest_pool(*from, perspective.other());
    let opp_after = distance_to_nearest_pool(*to, perspective.other());

    let own_gain = own_before.saturating_sub(own_after);
    let opp_gain = opp_before.saturating_sub(opp_after);

    if own_gain > 0 && opp_gain <= 0 {
        Some(own_gain)
    } else {
        None
    }
}

fn first_legal_atomic_step(game: &MonsGame) -> Option<Vec<Input>> {
    let mut probe = game.clone_for_simulation();
    let start_options = Some(SuggestedStartInputOptions::for_automove());
    let starts = match probe.process_input_with_start_options_slice(&[], true, false, start_options)
    {
        Output::LocationsToStartFrom(locations) if !locations.is_empty() => locations,
        _ => return None,
    };

    let mut sorted_starts = starts;
    sorted_starts.sort_unstable();

    for start in sorted_starts {
        let second_options = match probe.process_input_with_start_options_slice(
            &[Input::Location(start)],
            true,
            false,
            start_options,
        ) {
            Output::NextInputOptions(options) if !options.is_empty() => options,
            _ => continue,
        };

        let mut scored_second = second_options;
        scored_second.sort_by(|a, b| {
            let a_score = followup_priority(a.input);
            let b_score = followup_priority(b.input);
            b_score.cmp(&a_score).then_with(|| a.input.cmp(&b.input))
        });

        for option in scored_second {
            if let Some(inputs) =
                compile_seeded_inputs(game, vec![Input::Location(start), option.input])
            {
                return Some(inputs);
            }
        }
    }

    None
}

fn compile_seeded_inputs(game: &MonsGame, mut inputs: Vec<Input>) -> Option<Vec<Input>> {
    let mut probe = game.clone_for_simulation();
    let start_options = Some(SuggestedStartInputOptions::for_automove());

    for _ in 0..TURN_PLANNER_CHAIN_LIMIT {
        match probe.process_input_with_start_options_slice(
            inputs.as_slice(),
            true,
            false,
            start_options,
        ) {
            Output::InvalidInput => return None,
            Output::Events(_) => return Some(inputs),
            Output::LocationsToStartFrom(locations) => {
                let mut sorted = locations;
                sorted.sort_unstable();
                let next = sorted.first().copied()?;
                inputs.push(Input::Location(next));
            }
            Output::NextInputOptions(options) => {
                let next = choose_followup_input(options.as_slice())?;
                inputs.push(next);
            }
        }
    }

    None
}

fn compile_seeded_inputs_with_mode(
    game: &MonsGame,
    mut inputs: Vec<Input>,
    mode: IntentCompileMode,
) -> Option<Vec<Input>> {
    let mut probe = game.clone_for_simulation();
    let start_options = Some(SuggestedStartInputOptions::for_automove());

    for _ in 0..TURN_PLANNER_CHAIN_LIMIT {
        match probe.process_input_with_start_options_slice(
            inputs.as_slice(),
            true,
            false,
            start_options,
        ) {
            Output::InvalidInput => return None,
            Output::Events(_) => return Some(inputs),
            Output::LocationsToStartFrom(locations) => {
                let mut sorted = locations;
                sorted.sort_unstable();
                let next = sorted.first().copied()?;
                inputs.push(Input::Location(next));
            }
            Output::NextInputOptions(options) => {
                let next = choose_followup_input_with_mode(options.as_slice(), mode)?;
                inputs.push(next);
            }
        }
    }

    None
}

fn choose_followup_input(options: &[NextInput]) -> Option<Input> {
    let mut best: Option<(i32, Input)> = None;

    for option in options {
        let priority = followup_priority(option.input);
        match best {
            Some((best_priority, best_input)) => {
                if priority > best_priority
                    || (priority == best_priority && option.input < best_input)
                {
                    best = Some((priority, option.input));
                }
            }
            None => best = Some((priority, option.input)),
        }
    }

    best.map(|(_, input)| input)
}

fn choose_followup_input_with_mode(
    options: &[NextInput],
    mode: IntentCompileMode,
) -> Option<Input> {
    let mut best: Option<(i32, Input)> = None;

    for option in options {
        let priority = followup_priority_for_mode(option.input, mode);
        match best {
            Some((best_priority, best_input)) => {
                if priority > best_priority
                    || (priority == best_priority && option.input < best_input)
                {
                    best = Some((priority, option.input));
                }
            }
            None => best = Some((priority, option.input)),
        }
    }

    best.map(|(_, input)| input)
}

fn followup_priority(input: Input) -> i32 {
    match input {
        Input::Modifier(Modifier::SelectPotion) => 300,
        Input::Modifier(Modifier::SelectBomb) => 260,
        Input::Location(_) => 200,
        Input::Modifier(Modifier::Cancel) => 0,
        Input::Takeback => -100,
    }
}

fn followup_priority_for_mode(input: Input, mode: IntentCompileMode) -> i32 {
    match mode {
        IntentCompileMode::Path => match input {
            Input::Location(_) => 320,
            Input::Modifier(Modifier::SelectPotion) => 180,
            Input::Modifier(Modifier::SelectBomb) => 140,
            Input::Modifier(Modifier::Cancel) => 0,
            Input::Takeback => -120,
        },
        IntentCompileMode::Spirit => match input {
            Input::Location(_) => 340,
            Input::Modifier(Modifier::SelectPotion) => 120,
            Input::Modifier(Modifier::SelectBomb) => 110,
            Input::Modifier(Modifier::Cancel) => 0,
            Input::Takeback => -120,
        },
        IntentCompileMode::Attack => match input {
            Input::Modifier(Modifier::SelectBomb) => 330,
            Input::Location(_) => 300,
            Input::Modifier(Modifier::SelectPotion) => 160,
            Input::Modifier(Modifier::Cancel) => 0,
            Input::Takeback => -120,
        },
    }
}

fn apply_inputs_for_planner(game: &MonsGame, inputs: &[Input]) -> Option<(MonsGame, Vec<Event>)> {
    let mut simulated = game.clone_for_simulation();
    match simulated.process_input_slice(inputs, false, false) {
        Output::Events(events) => Some((simulated, events)),
        Output::InvalidInput | Output::LocationsToStartFrom(_) | Output::NextInputOptions(_) => {
            None
        }
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

fn events_include_spirit_move(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::SpiritTargetMove { .. }))
}

fn events_include_opponent_drainer_faint(events: &[Event], perspective: Color) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            Event::MonFainted { mon, .. } if mon.color == perspective.other() && mon.kind == MonKind::Drainer
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planner_config() -> TurnPlannerSearchConfig {
        TurnPlannerSearchConfig {
            max_nodes: 512,
            beam_width: 6,
            response_beam_width: 3,
            step_cap: 6,
            route_cap: 16,
            per_node_route_cap: 6,
            scoring_weights: &DEFAULT_SCORING_WEIGHTS,
            allow_exact_static_evaluation: true,
        }
    }

    fn game_with_items(items: Vec<(Location, Item)>, active_color: Color) -> MonsGame {
        let mut game = MonsGame::new(false);
        game.board = Board::new_with_items(items.into_iter().collect());
        game.active_color = active_color;
        game.turn_number = 2;
        game.actions_used_count = 0;
        game.mana_moves_count = 0;
        game.mons_moves_count = 0;
        game.white_score = 0;
        game.black_score = 0;
        game
    }

    #[test]
    fn turn_planner_produces_legal_inputs() {
        clear_turn_opportunity_plan_cache();
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(10, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
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
        );

        let selected = turn_opportunity_planner_next_inputs(
            &game,
            Color::White,
            TurnPlannerMode::ProV1,
            planner_config(),
        )
        .expect("planner should produce a legal move");
        assert!(matches!(
            game.clone_for_simulation()
                .process_input_slice(selected.as_slice(), false, false),
            Output::Events(_)
        ));
    }

    #[test]
    fn turn_planner_cache_replays_on_matching_state_and_replans_on_invalid_cache() {
        clear_turn_opportunity_plan_cache();
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
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
        );

        let first = turn_opportunity_planner_next_inputs(
            &game,
            Color::White,
            TurnPlannerMode::ProV1,
            planner_config(),
        )
        .expect("first planner step");

        let (after_first, _) =
            apply_inputs_for_planner(&game, first.as_slice()).expect("legal first step");
        let after_hash = MonsGameModel::search_state_hash(&after_first);
        let cached_expected =
            turn_opportunity_cache_entry_for_test(after_hash, TurnPlannerMode::ProV1)
                .expect("second-step cache should exist");

        let second = turn_opportunity_planner_next_inputs(
            &after_first,
            Color::White,
            TurnPlannerMode::ProV1,
            planner_config(),
        )
        .expect("cached second step");
        assert_eq!(second, cached_expected);

        let illegal = vec![
            Input::Location(Location::new(0, 0)),
            Input::Location(Location::new(0, 0)),
        ];
        insert_turn_opportunity_cache_entry_for_test(
            MonsGameModel::search_state_hash(&after_first),
            TurnPlannerMode::ProV1,
            illegal.clone(),
        );
        let repaired = turn_opportunity_planner_next_inputs(
            &after_first,
            Color::White,
            TurnPlannerMode::ProV1,
            planner_config(),
        )
        .expect("planner should recover from invalid cache entry");
        assert_ne!(repaired, illegal);
    }

    #[test]
    fn turn_planner_prefers_safe_supermana_progress_over_quiet_move() {
        clear_turn_opportunity_plan_cache();
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(10, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
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
        );

        let mut state = game.clone_for_simulation();
        let mut observed_safe_progress = false;

        for _ in 0..Config::MONS_MOVES_PER_TURN {
            if state.active_color != Color::White {
                break;
            }
            let selected = turn_opportunity_planner_next_inputs(
                &state,
                Color::White,
                TurnPlannerMode::ProV1,
                planner_config(),
            )
            .expect("planner move");
            let (next_state, _) =
                apply_inputs_for_planner(&state, selected.as_slice()).expect("legal move");
            if score_for_color(&next_state, Color::White) > score_for_color(&state, Color::White)
                || own_drainer_carries_safe_mana(&next_state.board, Color::White, Mana::Supermana)
            {
                observed_safe_progress = true;
                break;
            }
            state = next_state;
        }

        assert!(
            observed_safe_progress,
            "planner should convert this turn into safe supermana progress instead of only quiet lines"
        );
    }

    #[test]
    fn turn_planner_avoids_immediate_reply_loss_when_attack_available() {
        clear_turn_opportunity_plan_cache();
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
        );
        game.black_score = Config::TARGET_SCORE - 1;

        let mut state = game.clone_for_simulation();
        let mut killed_drainer = false;

        for _ in 0..4 {
            if state.active_color != Color::White {
                break;
            }

            let selected = turn_opportunity_planner_next_inputs(
                &state,
                Color::White,
                TurnPlannerMode::ProV1,
                planner_config(),
            )
            .expect("planner move");

            let (after, events) =
                apply_inputs_for_planner(&state, selected.as_slice()).expect("selected legal");
            if events_include_opponent_drainer_faint(events.as_slice(), Color::White) {
                killed_drainer = true;
            }
            state = after;
        }

        assert!(
            killed_drainer || !opponent_can_win_immediately(&state, Color::White),
            "planner continuation must either kill the scoring drainer or avoid immediate reply loss"
        );
    }

    #[test]
    fn turn_planner_diagnostics_capture_intent_and_route_activity() {
        clear_turn_opportunity_plan_cache();
        clear_turn_planner_diagnostics();
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(10, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
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
        );

        let selected = turn_opportunity_planner_next_inputs(
            &game,
            Color::White,
            TurnPlannerMode::ProV1,
            planner_config(),
        );
        assert!(selected.is_some(), "planner should produce a move");

        let diagnostics = turn_planner_diagnostics_snapshot();
        assert!(diagnostics.intent_generation_calls > 0);
        assert!(diagnostics.intent_generation_hits > 0);
        assert!(diagnostics.expansions > 0);
        assert!(
            diagnostics.route_model_tactical
                + diagnostics.route_drainer_score
                + diagnostics.route_drainer_kill
                + diagnostics.route_spirit_impact
                + diagnostics.route_drainer_safety
                + diagnostics.route_mana_move
                + diagnostics.route_tactical_deny
                + diagnostics.route_fallback
                > 0
        );
    }
}
