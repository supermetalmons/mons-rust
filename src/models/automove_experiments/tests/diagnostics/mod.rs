use super::*;
use crate::models::mons_game_model::automove_runtime_variants::{
    apply_frontier_pro_v2_guarded_config, clear_frontier_runtime_variant_branch,
    frontier_runtime_variant_branch_snapshot, select_frontier_pro_v2_guarded_inputs,
    select_frontier_pro_v2_guarded_inputs_with_frontier_runtime, select_shipping_search_inputs,
};
use crate::models::scoring::{
    evaluate_preferability_breakdown_with_weights, evaluate_preferability_with_context,
    evaluate_preferability_with_weights_and_exact_policy, ScoringEvalContext, ScoringWeights,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, OnceLock};

struct AttributionWorstReply {
    input_fen: String,
    score: i32,
    events: String,
    game: MonsGame,
}

fn attribution_root_index(scored_roots: &[RootEvaluation], root_fen: &str) -> Option<usize> {
    scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == root_fen)
}

fn zeroed_attribution_weights_like(base: &ScoringWeights) -> ScoringWeights {
    let mut weights = *base;
    weights.double_confirmed_score = false;
    weights.confirmed_score = 0;
    weights.fainted_mon = 0;
    weights.fainted_drainer = 0;
    weights.fainted_cooldown_step = 0;
    weights.drainer_at_risk = 0;
    weights.mana_close_to_same_pool = 0;
    weights.mon_with_mana_close_to_any_pool = 0;
    weights.extra_for_supermana = 0;
    weights.extra_for_opponents_mana = 0;
    weights.drainer_close_to_mana = 0;
    weights.drainer_holding_mana = 0;
    weights.drainer_close_to_own_pool = 0;
    weights.drainer_close_to_supermana = 0;
    weights.mon_close_to_center = 0;
    weights.spirit_close_to_enemy = 0;
    weights.spirit_on_own_base_penalty = 0;
    weights.angel_guarding_drainer = 0;
    weights.angel_close_to_friendly_drainer = 0;
    weights.has_consumable = 0;
    weights.active_mon = 0;
    weights.regular_mana_to_owner_pool = 0;
    weights.regular_mana_drainer_control = 0;
    weights.supermana_drainer_control = 0;
    weights.supermana_race_control = 0;
    weights.opponent_mana_denial = 0;
    weights.mana_carrier_at_risk = 0;
    weights.mana_carrier_guarded = 0;
    weights.mana_carrier_one_step_from_pool = 0;
    weights.supermana_carrier_one_step_from_pool_extra = 0;
    weights.immediate_winning_carrier = 0;
    weights.drainer_best_mana_path = 0;
    weights.drainer_pickup_score_this_turn = 0;
    weights.mana_carrier_score_this_turn = 0;
    weights.drainer_immediate_threat = 0;
    weights.score_race_path_progress = 0;
    weights.opponent_score_race_path_progress = 0;
    weights.score_race_multi_path = 0;
    weights.opponent_score_race_multi_path = 0;
    weights.immediate_score_window = 0;
    weights.opponent_immediate_score_window = 0;
    weights.immediate_score_multi_window = 0;
    weights.opponent_immediate_score_multi_window = 0;
    weights.spirit_action_utility = 0;
    weights.drainer_danger_boolean = 0;
    weights.mana_carrier_danger_boolean = 0;
    weights.drainer_walk_threat_boolean = 0;
    weights.mana_carrier_walk_threat_boolean = 0;
    weights.opponent_drainer_attack_bonus = 0;
    weights.attacker_close_to_opponent_drainer = 0;
    weights
}

fn attribution_residual_score(
    game: &MonsGame,
    perspective: Color,
    weights: &ScoringWeights,
) -> i32 {
    evaluate_preferability_breakdown_with_weights(game, perspective, weights)
        .terms
        .residual_board_state
}

fn attribution_residual_field_scores(
    game: &MonsGame,
    perspective: Color,
    base: &ScoringWeights,
) -> Vec<(&'static str, i32)> {
    let mut scores = Vec::new();
    macro_rules! add_field {
        ($field:ident) => {{
            let mut weights = zeroed_attribution_weights_like(base);
            weights.$field = base.$field;
            scores.push((
                stringify!($field),
                attribution_residual_score(game, perspective, &weights),
            ));
        }};
    }

    add_field!(confirmed_score);
    add_field!(fainted_mon);
    add_field!(fainted_drainer);
    add_field!(fainted_cooldown_step);
    add_field!(drainer_at_risk);
    add_field!(mana_close_to_same_pool);
    add_field!(mon_with_mana_close_to_any_pool);
    add_field!(extra_for_supermana);
    add_field!(extra_for_opponents_mana);
    add_field!(drainer_close_to_mana);
    add_field!(drainer_holding_mana);
    add_field!(drainer_close_to_own_pool);
    add_field!(drainer_close_to_supermana);
    add_field!(mon_close_to_center);
    add_field!(spirit_close_to_enemy);
    add_field!(spirit_on_own_base_penalty);
    add_field!(angel_guarding_drainer);
    add_field!(angel_close_to_friendly_drainer);
    add_field!(has_consumable);
    add_field!(active_mon);
    add_field!(regular_mana_to_owner_pool);
    add_field!(regular_mana_drainer_control);
    add_field!(supermana_drainer_control);
    add_field!(supermana_race_control);
    add_field!(opponent_mana_denial);
    add_field!(mana_carrier_at_risk);
    add_field!(mana_carrier_guarded);
    add_field!(mana_carrier_one_step_from_pool);
    add_field!(supermana_carrier_one_step_from_pool_extra);
    add_field!(immediate_winning_carrier);
    add_field!(drainer_best_mana_path);
    add_field!(drainer_pickup_score_this_turn);
    add_field!(mana_carrier_score_this_turn);
    add_field!(drainer_immediate_threat);
    add_field!(score_race_path_progress);
    add_field!(opponent_score_race_path_progress);
    add_field!(score_race_multi_path);
    add_field!(opponent_score_race_multi_path);
    add_field!(immediate_score_window);
    add_field!(opponent_immediate_score_window);
    add_field!(immediate_score_multi_window);
    add_field!(opponent_immediate_score_multi_window);
    add_field!(spirit_action_utility);
    add_field!(drainer_danger_boolean);
    add_field!(mana_carrier_danger_boolean);
    add_field!(drainer_walk_threat_boolean);
    add_field!(mana_carrier_walk_threat_boolean);
    add_field!(opponent_drainer_attack_bonus);
    add_field!(attacker_close_to_opponent_drainer);

    scores
}

fn top_attribution_residual_deltas(
    left_label: &str,
    left_game: &MonsGame,
    right_label: &str,
    right_game: &MonsGame,
    perspective: Color,
    config: AutomoveSearchConfig,
) -> String {
    let left_breakdown = evaluate_preferability_breakdown_with_weights(
        left_game,
        perspective,
        config.scoring_weights,
    );
    let right_breakdown = evaluate_preferability_breakdown_with_weights(
        right_game,
        perspective,
        config.scoring_weights,
    );
    let left_search_eval =
        MonsGameModel::evaluate_search_preferability(left_game, perspective, config);
    let right_search_eval =
        MonsGameModel::evaluate_search_preferability(right_game, perspective, config);
    let left_scores =
        attribution_residual_field_scores(left_game, perspective, config.scoring_weights);
    let right_scores =
        attribution_residual_field_scores(right_game, perspective, config.scoring_weights);
    let mut deltas = left_scores
        .into_iter()
        .zip(right_scores)
        .map(|((left_name, left_score), (right_name, right_score))| {
            assert_eq!(left_name, right_name);
            (left_name, left_score - right_score, left_score, right_score)
        })
        .collect::<Vec<_>>();
    let field_sum_delta = deltas.iter().map(|(_, delta, _, _)| *delta).sum::<i32>();
    deltas.sort_by(|left, right| {
        right
            .1
            .abs()
            .cmp(&left.1.abs())
            .then_with(|| left.0.cmp(right.0))
    });
    let top = deltas
        .iter()
        .filter(|(_, delta, _, _)| *delta != 0)
        .take(14)
        .map(|(name, delta, left_score, right_score)| {
            format!("{name}:{delta}({left_score}-{right_score})")
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{left_label}_minus_{right_label} search_eval_delta={} search_evals={}/{} breakdown_total_delta={} residual_delta={} field_sum_delta={} left_terms={:?} right_terms={:?} left_features={:?} right_features={:?} top_residual_fields=[{}]",
        left_search_eval - right_search_eval,
        left_search_eval,
        right_search_eval,
        left_breakdown.total - right_breakdown.total,
        left_breakdown.terms.residual_board_state - right_breakdown.terms.residual_board_state,
        field_sum_delta,
        left_breakdown.terms,
        right_breakdown.terms,
        left_breakdown.features,
        right_breakdown.features,
        top,
    )
}

#[derive(Clone, Copy)]
struct AttributionSearchEvalFlags {
    allow_exact_static_evaluation: bool,
    enable_local_ctx: bool,
    enable_attack_reach_summary: bool,
    enable_attack_reach_target_narrowing: bool,
    enable_attack_reach_drainer_target_narrowing: bool,
}

fn attribution_search_eval_with_flags(
    game: &MonsGame,
    perspective: Color,
    config: AutomoveSearchConfig,
    flags: AttributionSearchEvalFlags,
) -> i32 {
    if flags.enable_local_ctx {
        let context = ScoringEvalContext::new_with_flags(
            game,
            flags.allow_exact_static_evaluation,
            flags.enable_attack_reach_summary,
            flags.enable_attack_reach_target_narrowing,
            flags.enable_attack_reach_drainer_target_narrowing,
        );
        evaluate_preferability_with_context(
            game,
            perspective,
            config.scoring_weights,
            flags.allow_exact_static_evaluation,
            &context,
        )
    } else {
        evaluate_preferability_with_weights_and_exact_policy(
            game,
            perspective,
            config.scoring_weights,
            flags.allow_exact_static_evaluation,
        )
    }
}

fn attribution_search_eval_variant_deltas(
    left_label: &str,
    left_game: &MonsGame,
    right_label: &str,
    right_game: &MonsGame,
    perspective: Color,
    config: AutomoveSearchConfig,
) -> String {
    let variants = [
        (
            "config_no_local",
            AttributionSearchEvalFlags {
                allow_exact_static_evaluation: config.enable_static_exact_evaluation,
                enable_local_ctx: false,
                enable_attack_reach_summary: false,
                enable_attack_reach_target_narrowing: false,
                enable_attack_reach_drainer_target_narrowing: false,
            },
        ),
        (
            "exact_on_no_local",
            AttributionSearchEvalFlags {
                allow_exact_static_evaluation: true,
                enable_local_ctx: false,
                enable_attack_reach_summary: false,
                enable_attack_reach_target_narrowing: false,
                enable_attack_reach_drainer_target_narrowing: false,
            },
        ),
        (
            "exact_off_no_local",
            AttributionSearchEvalFlags {
                allow_exact_static_evaluation: false,
                enable_local_ctx: false,
                enable_attack_reach_summary: false,
                enable_attack_reach_target_narrowing: false,
                enable_attack_reach_drainer_target_narrowing: false,
            },
        ),
        (
            "exact_on_local_no_reach",
            AttributionSearchEvalFlags {
                allow_exact_static_evaluation: true,
                enable_local_ctx: true,
                enable_attack_reach_summary: false,
                enable_attack_reach_target_narrowing: false,
                enable_attack_reach_drainer_target_narrowing: false,
            },
        ),
        (
            "exact_off_local_no_reach",
            AttributionSearchEvalFlags {
                allow_exact_static_evaluation: false,
                enable_local_ctx: true,
                enable_attack_reach_summary: false,
                enable_attack_reach_target_narrowing: false,
                enable_attack_reach_drainer_target_narrowing: false,
            },
        ),
        (
            "exact_on_local_drainer",
            AttributionSearchEvalFlags {
                allow_exact_static_evaluation: true,
                enable_local_ctx: true,
                enable_attack_reach_summary: true,
                enable_attack_reach_target_narrowing: true,
                enable_attack_reach_drainer_target_narrowing: true,
            },
        ),
        (
            "exact_off_local_drainer",
            AttributionSearchEvalFlags {
                allow_exact_static_evaluation: false,
                enable_local_ctx: true,
                enable_attack_reach_summary: true,
                enable_attack_reach_target_narrowing: true,
                enable_attack_reach_drainer_target_narrowing: true,
            },
        ),
    ];
    variants
        .iter()
        .map(|(label, flags)| {
            let left_score =
                attribution_search_eval_with_flags(left_game, perspective, config, *flags);
            let right_score =
                attribution_search_eval_with_flags(right_game, perspective, config, *flags);
            format!(
                "{}:{}_minus_{}={}({}-{})",
                label,
                left_label,
                right_label,
                left_score - right_score,
                left_score,
                right_score,
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn attribution_worst_reply_state(
    state_after_move: &MonsGame,
    perspective: Color,
    config: AutomoveSearchConfig,
) -> Option<AttributionWorstReply> {
    let reply_limit = config.root_reply_risk_reply_limit.clamp(1, 24);
    let replies = MonsGameModel::enumerate_legal_transitions(
        state_after_move,
        reply_limit,
        MonsGameModel::automove_start_input_options(config),
    );
    replies
        .into_iter()
        .map(|reply| {
            let score = match reply.game.winner_color() {
                Some(winner) if winner == perspective => SMART_TERMINAL_SCORE / 2,
                Some(_) => -SMART_TERMINAL_SCORE / 2,
                None => {
                    MonsGameModel::evaluate_search_preferability(&reply.game, perspective, config)
                }
            };
            (score, Input::fen_from_array(&reply.inputs), reply)
        })
        .min_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)))
        .map(|(score, input_fen, reply)| AttributionWorstReply {
            input_fen,
            score,
            events: format!("{:?}", reply.events),
            game: reply.game,
        })
}

#[derive(Clone, Copy)]
struct ProProfileSweepCandidate {
    id: &'static str,
    selector: AutomoveSelector,
}

fn select_sweep_frontier_config_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    if config.enable_turn_engine_selector {
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
    }
    clear_turn_engine_selector_diagnostics();
    select_shipping_search_inputs(game, config)
}

fn select_sweep_shipping_pro_search_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    select_shipping_search_inputs(game, config)
}

fn select_sweep_frontier_pro_v2_guarded_counted_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    clear_frontier_runtime_variant_branch();
    let inputs = select_frontier_pro_v2_guarded_inputs(game, config);
    record_profile_sweep_branch(frontier_runtime_variant_branch_snapshot());
    inputs
}

fn select_sweep_frontier_pro_v2_raw_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    select_sweep_frontier_config_inputs(game, apply_frontier_pro_v2_guarded_config(config))
}

fn select_sweep_frontier_guarded_with_runtime_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
    runtime: AutomoveSearchConfig,
) -> Vec<Input> {
    clear_frontier_runtime_variant_branch();
    let inputs = select_frontier_pro_v2_guarded_inputs_with_frontier_runtime(game, config, runtime);
    record_profile_sweep_branch(frontier_runtime_variant_branch_snapshot());
    inputs
}

fn select_sweep_frontier_pro_v2_no_selected_followup_projection_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    let mut runtime = apply_frontier_pro_v2_guarded_config(config);
    runtime.enable_turn_engine_selected_followup_projection = false;
    select_sweep_frontier_guarded_with_runtime_inputs(game, config, runtime)
}

fn select_sweep_frontier_pro_v3_full_scored_reply_guard_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    let runtime = apply_frontier_pro_v2_guarded_config(config);
    let (runtime, scored_roots, _, _) = runtime_scored_roots_with_config(game, runtime);
    let candidate_indices = (0..scored_roots.len()).collect::<Vec<_>>();
    if candidate_indices.is_empty() {
        return select_sweep_frontier_pro_v2_guarded_counted_inputs(game, config);
    }

    MonsGameModel::pick_root_move_with_reply_risk_guard_from_shortlist(
        game,
        &scored_roots,
        candidate_indices.as_slice(),
        Some(candidate_indices.as_slice()),
        game.active_color,
        runtime,
    )
    .and_then(|index| scored_roots.get(index))
    .map(|root| root.inputs.clone())
    .unwrap_or_else(|| select_sweep_frontier_pro_v2_guarded_counted_inputs(game, config))
}

fn select_sweep_frontier_pro_v2_no_low_budget_guard_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    let mut runtime = apply_frontier_pro_v2_guarded_config(config);
    runtime.enable_turn_engine_low_budget_guard = false;
    select_sweep_frontier_guarded_with_runtime_inputs(game, config, runtime)
}

fn select_sweep_frontier_pro_v3_alternating_white_edge_mana_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    if game.variant() == GameVariant::AlternatingManaRows
        && game.active_color == Color::White
        && game.turn_number == 1
        && game.mons_moves_count == 3
        && !game.player_can_use_action()
        && !game.player_can_move_mana()
    {
        let runtime = apply_frontier_pro_v2_guarded_config(config);
        let (_, scored_roots, _, _) = runtime_scored_roots_with_config(game, runtime);
        for preferred in [
            "l9,7;l9,8",
            "l10,3;l9,3",
            "l9,7;l8,6",
            "l9,7;l8,7",
            "l9,4;l10,4",
        ] {
            if let Some(root) = scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == preferred)
            {
                return root.inputs.clone();
            }
        }
    }

    select_sweep_frontier_pro_v2_guarded_counted_inputs(game, config)
}

fn select_sweep_frontier_pro_v3_white_opening_utility_mana_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    let white_turn_one_mana_followup = game.active_color == Color::White
        && game.turn_number == 1
        && game.is_first_turn()
        && game.mons_moves_count == 1
        && !game.player_can_use_action()
        && !game.player_can_move_mana();
    if !white_turn_one_mana_followup {
        return select_sweep_frontier_pro_v2_guarded_counted_inputs(game, config);
    }

    let runtime = apply_frontier_pro_v2_guarded_config(config);
    let (_, scored_roots, _, _) = runtime_scored_roots_with_config(game, runtime);
    let best_score = scored_roots
        .iter()
        .map(|root| root.score)
        .max()
        .unwrap_or(i32::MIN);
    let selected = scored_roots
        .iter()
        .filter(|root| {
            !root.wins_immediately
                && !root.attacks_opponent_drainer
                && !root.own_drainer_vulnerable
                && !root.own_drainer_walk_vulnerable
                && !root.spirit_development
                && !root.spirit_same_turn_score_setup_now
                && !root.spirit_own_mana_setup_now
                && !root.mana_handoff_to_opponent
                && !root.has_roundtrip
                && !root.scores_supermana_this_turn
                && !root.scores_opponent_mana_this_turn
                && !root.safe_supermana_pickup_now
                && !root.safe_opponent_mana_pickup_now
                && root.same_turn_score_window_value == 0
                && !root.supermana_progress
                && !root.opponent_mana_progress
                && matches!(
                    MonsGameModel::turn_engine_root_evaluation_family(root),
                    TurnPlanFamily::ManaTempo
                )
                && root.score.saturating_add(96) >= best_score
        })
        .max_by(|left, right| {
            let left_family = MonsGameModel::turn_engine_root_evaluation_family(left);
            let right_family = MonsGameModel::turn_engine_root_evaluation_family(right);
            let left_utility = MonsGameModel::turn_engine_selected_override_utility(
                game,
                left,
                game.active_color,
                runtime,
                left_family,
            );
            let right_utility = MonsGameModel::turn_engine_selected_override_utility(
                game,
                right,
                game.active_color,
                runtime,
                right_family,
            );
            left_utility
                .cmp(&right_utility)
                .then_with(|| left.score.cmp(&right.score))
                .then_with(|| right.root_rank.cmp(&left.root_rank))
        });

    if let Some(root) = selected {
        #[cfg(test)]
        record_profile_sweep_branch("white_opening_utility_mana");
        return root.inputs.clone();
    }

    select_sweep_frontier_pro_v2_guarded_counted_inputs(game, config)
}

fn pro_profile_sweep_candidates() -> Vec<ProProfileSweepCandidate> {
    vec![
        ProProfileSweepCandidate {
            id: "shipping_pro_search_control",
            selector: select_sweep_shipping_pro_search_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_guarded",
            selector: select_sweep_frontier_pro_v2_guarded_counted_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_raw",
            selector: select_sweep_frontier_pro_v2_raw_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_no_selected_followup_projection",
            selector: select_sweep_frontier_pro_v2_no_selected_followup_projection_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v3_full_scored_reply_guard",
            selector: select_sweep_frontier_pro_v3_full_scored_reply_guard_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_no_low_budget_guard",
            selector: select_sweep_frontier_pro_v2_no_low_budget_guard_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v3_alternating_white_edge_mana",
            selector: select_sweep_frontier_pro_v3_alternating_white_edge_mana_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v3_white_opening_utility_mana",
            selector: select_sweep_frontier_pro_v3_white_opening_utility_mana_inputs,
        },
    ]
}

fn pro_sweep_filter_tokens(name: &str, default: &str) -> Vec<String> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
        .split(',')
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

fn pro_sweep_filter_allows(tokens: &[String], id: &str) -> bool {
    tokens.iter().any(|token| token == "all" || token == id)
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn profile_sweep_branch_counts() -> &'static Mutex<BTreeMap<&'static str, usize>> {
    static COUNTS: OnceLock<Mutex<BTreeMap<&'static str, usize>>> = OnceLock::new();
    COUNTS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn clear_profile_sweep_branch_counts() {
    profile_sweep_branch_counts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}

fn record_profile_sweep_branch(branch: &'static str) {
    let mut counts = profile_sweep_branch_counts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *counts.entry(branch).or_default() += 1;
}

fn print_profile_sweep_branch_counts(candidate_id: &str, duel_label: &str) {
    let counts = profile_sweep_branch_counts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for (branch, turns) in counts.iter() {
        println!(
            "PRO_PROFILE_SWEEP_BRANCH {{\"candidate\":\"{}\",\"duel\":\"{}\",\"branch\":\"{}\",\"turns\":{}}}",
            json_escape(candidate_id),
            json_escape(duel_label),
            json_escape(branch),
            turns
        );
    }
}

fn print_profile_sweep_summary(
    candidate_id: &str,
    opponent_profile: &str,
    duel_label: &str,
    opponent_mode: SmartAutomovePreference,
    stats: &TimedMatchupStats,
) {
    let metrics = pro_reliability_metrics(stats);
    println!(
        "PRO_PROFILE_SWEEP_RESULT {{\"candidate\":\"{}\",\"opponent_profile\":\"{}\",\"duel\":\"{}\",\"opponent_mode\":\"{}\",\"total_games\":{},\"wins\":{},\"losses\":{},\"draws\":{},\"win_rate\":{:.4},\"confidence\":{:.4},\"candidate_avg_ms\":{:.2},\"opponent_avg_ms\":{:.2},\"candidate_turns\":{},\"opponent_turns\":{},\"duel_passes\":{}}}",
        json_escape(candidate_id),
        json_escape(opponent_profile),
        json_escape(duel_label),
        opponent_mode.as_api_value(),
        stats.matchup.total_games(),
        stats.matchup.wins,
        stats.matchup.losses,
        stats.matchup.draws,
        metrics.win_rate,
        metrics.confidence,
        metrics.frontier_avg_ms,
        stats.timing.profile_b_avg_ms(),
        stats.timing.profile_a_turns,
        stats.timing.profile_b_turns,
        pro_reliability_duel_passes(metrics)
    );
    for variant_stats in stats.per_variant_stats() {
        println!(
            "PRO_PROFILE_SWEEP_VARIANT {{\"candidate\":\"{}\",\"duel\":\"{}\",\"variant\":\"{}\",\"total_games\":{},\"wins\":{},\"losses\":{},\"draws\":{},\"win_rate\":{:.4},\"confidence\":{:.4},\"candidate_avg_ms\":{:.2},\"opponent_avg_ms\":{:.2}}}",
            json_escape(candidate_id),
            json_escape(duel_label),
            automove_variant_label(variant_stats.variant),
            variant_stats.matchup.total_games(),
            variant_stats.matchup.wins,
            variant_stats.matchup.losses,
            variant_stats.matchup.draws,
            variant_stats.matchup.win_rate_points(),
            variant_stats.matchup.confidence_better_than_even(),
            variant_stats.timing.profile_a_avg_ms(),
            variant_stats.timing.profile_b_avg_ms(),
        );
    }
}

#[derive(Clone, Copy)]
struct ProPromotionDashboardPanelSpec {
    label: &'static str,
    variant_policy: &'static str,
    variants: &'static str,
    default_repeats: usize,
    default_games: usize,
    seed_tag: &'static str,
}

#[derive(Debug, Clone)]
struct ProPromotionDashboardPanelSummary {
    panel: &'static str,
    shipping_duels: usize,
    shipping_strict_passes: usize,
    shipping_directional_passes: usize,
    min_shipping_win_rate: f64,
    min_shipping_confidence: f64,
    max_candidate_avg_ms: f64,
    weak_variant_rows: usize,
    guarded_win_rate: f64,
    guarded_confidence: f64,
}

impl ProPromotionDashboardPanelSummary {
    fn new(panel: &'static str) -> Self {
        Self {
            panel,
            shipping_duels: 0,
            shipping_strict_passes: 0,
            shipping_directional_passes: 0,
            min_shipping_win_rate: 1.0,
            min_shipping_confidence: 1.0,
            max_candidate_avg_ms: 0.0,
            weak_variant_rows: 0,
            guarded_win_rate: 0.5,
            guarded_confidence: 0.0,
        }
    }

    fn record_shipping_duel(&mut self, stats: &TimedMatchupStats) {
        let metrics = pro_reliability_metrics(stats);
        self.shipping_duels += 1;
        if pro_reliability_duel_passes(metrics) {
            self.shipping_strict_passes += 1;
        }
        if metrics.win_rate >= 0.90 && metrics.frontier_avg_ms <= 700.0 {
            self.shipping_directional_passes += 1;
        }
        self.min_shipping_win_rate = self.min_shipping_win_rate.min(metrics.win_rate);
        self.min_shipping_confidence = self.min_shipping_confidence.min(metrics.confidence);
        self.max_candidate_avg_ms = self.max_candidate_avg_ms.max(metrics.frontier_avg_ms);
        self.weak_variant_rows += stats
            .per_variant_stats()
            .into_iter()
            .filter(|variant_stats| variant_stats.matchup.win_rate_points() < 0.50)
            .count();
    }

    fn record_guarded_duel(&mut self, stats: &TimedMatchupStats) {
        self.guarded_win_rate = stats.matchup.win_rate_points();
        self.guarded_confidence = stats.matchup.confidence_better_than_even();
        self.max_candidate_avg_ms = self
            .max_candidate_avg_ms
            .max(stats.timing.profile_a_avg_ms());
    }

    fn shipping_strict_passes_all(&self) -> bool {
        self.shipping_duels > 0 && self.shipping_strict_passes == self.shipping_duels
    }

    fn shipping_directional_passes_all(&self) -> bool {
        self.shipping_duels > 0 && self.shipping_directional_passes == self.shipping_duels
    }
}

fn pro_promotion_dashboard_panel_specs() -> Vec<ProPromotionDashboardPanelSpec> {
    vec![
        ProPromotionDashboardPanelSpec {
            label: "sampled",
            variant_policy: "sampled",
            variants: "",
            default_repeats: 3,
            default_games: 2,
            seed_tag: "pro_profile_sweep_v1",
        },
        ProPromotionDashboardPanelSpec {
            label: "active_blockers",
            variant_policy: "sampled",
            variants: "outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows",
            default_repeats: 1,
            default_games: 3,
            seed_tag: "pro_profile_active_blockers_v1",
        },
    ]
}

fn with_pro_promotion_dashboard_panel<T>(
    panel: ProPromotionDashboardPanelSpec,
    f: impl FnOnce() -> T,
) -> T {
    with_env_override("SMART_AUTOMOVE_VARIANTS", panel.variants, || {
        with_env_override("SMART_AUTOMOVE_VARIANT_POLICY", panel.variant_policy, f)
    })
}

fn pro_promotion_dashboard_directional_label(
    sampled: &ProPromotionDashboardPanelSummary,
    active: &ProPromotionDashboardPanelSummary,
) -> &'static str {
    if sampled.shipping_strict_passes_all()
        && active.shipping_directional_passes_all()
        && sampled.weak_variant_rows == 0
        && active.weak_variant_rows == 0
    {
        "promotable_scout"
    } else if sampled.shipping_directional_passes_all() && active.shipping_directional_passes_all()
    {
        "directional_both_panels"
    } else if active.shipping_directional_passes_all() {
        "active_blocker_only"
    } else if sampled.shipping_directional_passes_all() {
        "sampled_only"
    } else {
        "not_promising"
    }
}

fn pro_promotion_dashboard_stoplight_label(
    classification: &str,
    panel_summaries: &[ProPromotionDashboardPanelSummary],
) -> &'static str {
    let cost_blocked = panel_summaries
        .iter()
        .any(|summary| summary.max_candidate_avg_ms >= 650.0);
    if cost_blocked {
        "cost_blocked"
    } else {
        match classification {
            "promotable_scout" => "promotable_shape",
            "sampled_only" => "sampled_only",
            "active_blocker_only" => "active_only",
            "directional_both_panels" => "broad_pressure",
            _ => "not_promising",
        }
    }
}

fn print_pro_promotion_dashboard_result(
    panel: &str,
    candidate_id: &str,
    comparison: &str,
    duel_label: &str,
    opponent_profile: &str,
    opponent_mode: SmartAutomovePreference,
    stats: &TimedMatchupStats,
) {
    let metrics = pro_reliability_metrics(stats);
    println!(
        "PRO_PROMOTION_DASHBOARD_RESULT {{\"panel\":\"{}\",\"candidate\":\"{}\",\"comparison\":\"{}\",\"duel\":\"{}\",\"opponent_profile\":\"{}\",\"opponent_mode\":\"{}\",\"total_games\":{},\"wins\":{},\"losses\":{},\"draws\":{},\"win_rate\":{:.4},\"confidence\":{:.4},\"candidate_avg_ms\":{:.2},\"opponent_avg_ms\":{:.2},\"candidate_turns\":{},\"opponent_turns\":{},\"strict_passes\":{},\"directional_passes\":{}}}",
        json_escape(panel),
        json_escape(candidate_id),
        json_escape(comparison),
        json_escape(duel_label),
        json_escape(opponent_profile),
        opponent_mode.as_api_value(),
        stats.matchup.total_games(),
        stats.matchup.wins,
        stats.matchup.losses,
        stats.matchup.draws,
        metrics.win_rate,
        metrics.confidence,
        metrics.frontier_avg_ms,
        stats.timing.profile_b_avg_ms(),
        stats.timing.profile_a_turns,
        stats.timing.profile_b_turns,
        pro_reliability_duel_passes(metrics),
        metrics.win_rate >= 0.90 && metrics.frontier_avg_ms <= 700.0,
    );
}

fn print_pro_promotion_dashboard_variants(
    panel: &str,
    candidate_id: &str,
    comparison: &str,
    duel_label: &str,
    stats: &TimedMatchupStats,
) {
    let mut variant_stats = stats.per_variant_stats();
    variant_stats.sort_by(|left, right| {
        left.matchup
            .win_rate_points()
            .partial_cmp(&right.matchup.win_rate_points())
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.variant.id().cmp(&right.variant.id()))
    });
    for (rank, variant_stats) in variant_stats.into_iter().enumerate() {
        println!(
            "PRO_PROMOTION_DASHBOARD_VARIANT {{\"panel\":\"{}\",\"candidate\":\"{}\",\"comparison\":\"{}\",\"duel\":\"{}\",\"weakness_rank\":{},\"variant\":\"{}\",\"total_games\":{},\"wins\":{},\"losses\":{},\"draws\":{},\"win_rate\":{:.4},\"confidence\":{:.4},\"candidate_avg_ms\":{:.2},\"opponent_avg_ms\":{:.2}}}",
            json_escape(panel),
            json_escape(candidate_id),
            json_escape(comparison),
            json_escape(duel_label),
            rank + 1,
            automove_variant_label(variant_stats.variant),
            variant_stats.matchup.total_games(),
            variant_stats.matchup.wins,
            variant_stats.matchup.losses,
            variant_stats.matchup.draws,
            variant_stats.matchup.win_rate_points(),
            variant_stats.matchup.confidence_better_than_even(),
            variant_stats.timing.profile_a_avg_ms(),
            variant_stats.timing.profile_b_avg_ms(),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProProfileSweepAttributionTurn {
    ply: usize,
    board_fen: String,
    move_fen: String,
    candidate_branch: &'static str,
    active_color: Color,
    turn_number: i32,
    mons_moves_count: i32,
    can_use_action: bool,
    can_move_mana: bool,
    exact_context: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProProfileSweepAttributionTrace {
    result: MatchResult,
    final_fen: String,
    candidate_turns: Vec<ProProfileSweepAttributionTurn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProProfileSweepFirstDivergence {
    ply: usize,
    board_fen: String,
    left_move_fen: String,
    right_move_fen: String,
    left_branch: &'static str,
    right_branch: &'static str,
    active_color: Color,
    turn_number: i32,
    mons_moves_count: i32,
    can_use_action: bool,
    can_move_mana: bool,
    exact_context: String,
}

fn pro_profile_sweep_color_label(color: Color) -> &'static str {
    match color {
        Color::White => "white",
        Color::Black => "black",
    }
}

fn pro_profile_sweep_divergence_context_key(divergence: &ProProfileSweepFirstDivergence) -> String {
    format!(
        "{} branch={} turn={} mons_moves={} can_action={} can_mana={} {}",
        pro_profile_sweep_color_label(divergence.active_color),
        divergence.left_branch,
        divergence.turn_number,
        divergence.mons_moves_count,
        divergence.can_use_action,
        divergence.can_move_mana,
        divergence.exact_context,
    )
}

fn pro_policy_matrix_ply_bucket(ply: usize) -> &'static str {
    match ply {
        0..=7 => "ply0_7",
        8..=15 => "ply8_15",
        16..=31 => "ply16_31",
        _ => "ply32_plus",
    }
}

fn pro_policy_matrix_followup_count_bucket(count: usize) -> &'static str {
    match count {
        0 => "followups0",
        1 => "followups1",
        2 => "followups2",
        _ => "followups3_plus",
    }
}

fn pro_policy_matrix_continuation_rejoin_bucket(
    left: &ProProfileSweepAttributionTrace,
    right: &ProProfileSweepAttributionTrace,
    divergence: &ProProfileSweepFirstDivergence,
) -> &'static str {
    let mut left_after = left
        .candidate_turns
        .iter()
        .filter(|turn| turn.ply > divergence.ply);
    let mut right_after = right
        .candidate_turns
        .iter()
        .filter(|turn| turn.ply > divergence.ply);

    match (left_after.next(), right_after.next()) {
        (None, None) => "no_followup",
        (Some(_), None) | (None, Some(_)) => "one_sided_followup",
        (Some(left_next), Some(right_next)) if left_next.board_fen == right_next.board_fen => {
            "next_rejoin"
        }
        (Some(_), Some(_)) => {
            let mut left_later = left
                .candidate_turns
                .iter()
                .filter(|turn| turn.ply > divergence.ply);
            let right_later = right
                .candidate_turns
                .iter()
                .filter(|turn| turn.ply > divergence.ply)
                .collect::<Vec<_>>();
            if left_later.any(|left_turn| {
                right_later
                    .iter()
                    .any(|right_turn| left_turn.board_fen == right_turn.board_fen)
            }) {
                "later_rejoin"
            } else {
                "no_rejoin"
            }
        }
    }
}

fn pro_policy_matrix_timing_continuation_axes(
    first_divergence: Option<&ProProfileSweepFirstDivergence>,
    baseline_trace: &ProProfileSweepAttributionTrace,
    candidate_trace: &ProProfileSweepAttributionTrace,
) -> String {
    let Some(divergence) = first_divergence else {
        return [
            "axis=decision_timing first_diff=none".to_string(),
            "axis=continuation_stability first_diff=none".to_string(),
        ]
        .join("|");
    };

    let baseline_followups = baseline_trace
        .candidate_turns
        .iter()
        .filter(|turn| turn.ply > divergence.ply)
        .count();
    let candidate_followups = candidate_trace
        .candidate_turns
        .iter()
        .filter(|turn| turn.ply > divergence.ply)
        .count();
    let branch_transition = if divergence.left_branch == divergence.right_branch {
        "same_branch"
    } else {
        "branch_changed"
    };
    let final_state = if baseline_trace.final_fen == candidate_trace.final_fen {
        "same_final"
    } else {
        "different_final"
    };

    [
        format!(
            "axis=decision_timing ply_bucket={} color={} turn_bucket={} mons_moves={} can_action={} can_mana={}",
            pro_policy_matrix_ply_bucket(divergence.ply),
            pro_profile_sweep_color_label(divergence.active_color),
            pro_policy_mechanism_turn_bucket(divergence.turn_number),
            pro_policy_mechanism_mons_moves_bucket(divergence.mons_moves_count),
            divergence.can_use_action,
            divergence.can_move_mana,
        ),
        format!(
            "axis=decision_stage baseline_branch={} candidate_branch={} transition={}",
            divergence.left_branch, divergence.right_branch, branch_transition,
        ),
        format!(
            "axis=continuation_stability rejoin={} final_state={} baseline_followups={} candidate_followups={}",
            pro_policy_matrix_continuation_rejoin_bucket(
                baseline_trace,
                candidate_trace,
                divergence,
            ),
            final_state,
            pro_policy_matrix_followup_count_bucket(baseline_followups),
            pro_policy_matrix_followup_count_bucket(candidate_followups),
        ),
    ]
    .join("|")
}

fn select_profile_sweep_candidate_inputs_with_branch(
    candidate: ProProfileSweepCandidate,
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> (Vec<Input>, &'static str) {
    clear_frontier_runtime_variant_branch();
    let inputs = select_inputs_with_runtime_fallback(candidate.selector, game, config);
    let branch = match frontier_runtime_variant_branch_snapshot() {
        "unset" => "candidate_execute",
        branch => branch,
    };
    let branch = if inputs.is_empty() {
        "candidate_execute"
    } else {
        branch
    };
    (inputs, branch)
}

fn play_profile_sweep_attribution_trace(
    candidate: ProProfileSweepCandidate,
    opponent_selector: AutomoveSelector,
    opponent_budget: SearchBudget,
    opening_fen: &str,
    candidate_is_white: bool,
    max_plies: usize,
) -> ProProfileSweepAttributionTrace {
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_engine_plan_cache();
    clear_turn_engine_diagnostics();
    clear_turn_engine_selector_diagnostics();

    let mut candidate_turns = Vec::new();
    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return ProProfileSweepAttributionTrace {
                result: match_result_from_winner(winner_color, candidate_is_white),
                final_fen: game.fen(),
                candidate_turns,
            };
        }

        let board_fen = game.fen();
        let candidate_to_move = if candidate_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };
        let (inputs, guarded_branch) = if candidate_to_move {
            select_profile_sweep_candidate_inputs_with_branch(
                candidate,
                &game,
                pro_budget().runtime_config_for_game(&game),
            )
        } else {
            (
                select_inputs_with_runtime_fallback(
                    opponent_selector,
                    &game,
                    opponent_budget.runtime_config_for_game(&game),
                ),
                "opponent_execute",
            )
        };

        if candidate_to_move {
            candidate_turns.push(ProProfileSweepAttributionTurn {
                ply,
                board_fen,
                move_fen: Input::fen_from_array(&inputs),
                candidate_branch: guarded_branch,
                active_color: game.active_color,
                turn_number: game.turn_number,
                mons_moves_count: game.mons_moves_count,
                can_use_action: game.player_can_use_action(),
                can_move_mana: game.player_can_move_mana(),
                exact_context: exact_opportunity_context_probe(&game),
            });
        }

        if inputs.is_empty() {
            return ProProfileSweepAttributionTrace {
                result: if candidate_to_move {
                    MatchResult::ProfileBWin
                } else {
                    MatchResult::ProfileAWin
                },
                final_fen: game.fen(),
                candidate_turns,
            };
        }
        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return ProProfileSweepAttributionTrace {
                result: if candidate_to_move {
                    MatchResult::ProfileBWin
                } else {
                    MatchResult::ProfileAWin
                },
                final_fen: game.fen(),
                candidate_turns,
            };
        }
    }

    ProProfileSweepAttributionTrace {
        result: match adjudicate_non_terminal_game(&game) {
            Some(winner_color) => match_result_from_winner(winner_color, candidate_is_white),
            None => MatchResult::Draw,
        },
        final_fen: game.fen(),
        candidate_turns,
    }
}

fn play_profile_sweep_forced_first_candidate_turn(
    candidate: ProProfileSweepCandidate,
    opponent_selector: AutomoveSelector,
    opponent_budget: SearchBudget,
    opening_fen: &str,
    candidate_is_white: bool,
    max_plies: usize,
    forced_inputs: &[Input],
) -> ProProfileSweepAttributionTrace {
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_engine_plan_cache();
    clear_turn_engine_diagnostics();
    clear_turn_engine_selector_diagnostics();

    let mut candidate_turns = Vec::new();
    let mut forced_turn_spent = false;
    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return ProProfileSweepAttributionTrace {
                result: match_result_from_winner(winner_color, candidate_is_white),
                final_fen: game.fen(),
                candidate_turns,
            };
        }

        let board_fen = game.fen();
        let candidate_to_move = if candidate_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };
        let (inputs, guarded_branch) = if candidate_to_move && !forced_turn_spent {
            forced_turn_spent = true;
            (forced_inputs.to_vec(), "forced_root")
        } else if candidate_to_move {
            select_profile_sweep_candidate_inputs_with_branch(
                candidate,
                &game,
                pro_budget().runtime_config_for_game(&game),
            )
        } else {
            (
                select_inputs_with_runtime_fallback(
                    opponent_selector,
                    &game,
                    opponent_budget.runtime_config_for_game(&game),
                ),
                "opponent_execute",
            )
        };

        if candidate_to_move {
            candidate_turns.push(ProProfileSweepAttributionTurn {
                ply,
                board_fen,
                move_fen: Input::fen_from_array(&inputs),
                candidate_branch: guarded_branch,
                active_color: game.active_color,
                turn_number: game.turn_number,
                mons_moves_count: game.mons_moves_count,
                can_use_action: game.player_can_use_action(),
                can_move_mana: game.player_can_move_mana(),
                exact_context: exact_opportunity_context_probe(&game),
            });
        }

        if inputs.is_empty() {
            return ProProfileSweepAttributionTrace {
                result: if candidate_to_move {
                    MatchResult::ProfileBWin
                } else {
                    MatchResult::ProfileAWin
                },
                final_fen: game.fen(),
                candidate_turns,
            };
        }
        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return ProProfileSweepAttributionTrace {
                result: if candidate_to_move {
                    MatchResult::ProfileBWin
                } else {
                    MatchResult::ProfileAWin
                },
                final_fen: game.fen(),
                candidate_turns,
            };
        }
    }

    ProProfileSweepAttributionTrace {
        result: match adjudicate_non_terminal_game(&game) {
            Some(winner_color) => match_result_from_winner(winner_color, candidate_is_white),
            None => MatchResult::Draw,
        },
        final_fen: game.fen(),
        candidate_turns,
    }
}

fn first_profile_sweep_candidate_divergence(
    left: &ProProfileSweepAttributionTrace,
    right: &ProProfileSweepAttributionTrace,
) -> Option<ProProfileSweepFirstDivergence> {
    left.candidate_turns
        .iter()
        .zip(right.candidate_turns.iter())
        .find_map(|(left_turn, right_turn)| {
            if left_turn.board_fen != right_turn.board_fen {
                return None;
            }
            if left_turn.move_fen == right_turn.move_fen {
                return None;
            }
            Some(ProProfileSweepFirstDivergence {
                ply: left_turn.ply,
                board_fen: left_turn.board_fen.clone(),
                left_move_fen: left_turn.move_fen.clone(),
                right_move_fen: right_turn.move_fen.clone(),
                left_branch: left_turn.candidate_branch,
                right_branch: right_turn.candidate_branch,
                active_color: left_turn.active_color,
                turn_number: left_turn.turn_number,
                mons_moves_count: left_turn.mons_moves_count,
                can_use_action: left_turn.can_use_action,
                can_move_mana: left_turn.can_move_mana,
                exact_context: left_turn.exact_context.clone(),
            })
        })
}

fn pro_profile_sweep_outcome_label(delta: i32) -> &'static str {
    if delta > 0 {
        "right_better"
    } else if delta < 0 {
        "left_better"
    } else {
        "same_outcome"
    }
}

fn pro_profile_sweep_candidate_by_id(candidate_id: &str) -> ProProfileSweepCandidate {
    pro_profile_sweep_candidates()
        .into_iter()
        .find(|candidate| candidate.id == candidate_id)
        .unwrap_or_else(|| panic!("unknown sweep candidate '{}'", candidate_id))
}

fn pro_profile_sweep_candidate_list_from_env(
    name: &str,
    default: &str,
) -> Vec<ProProfileSweepCandidate> {
    let tokens = pro_sweep_filter_tokens(name, default);
    if tokens.iter().any(|token| token == "all") {
        return pro_profile_sweep_candidates();
    }
    tokens
        .iter()
        .map(|candidate_id| pro_profile_sweep_candidate_by_id(candidate_id))
        .collect()
}

fn pro_policy_matrix_outcome_label(delta: i32) -> &'static str {
    if delta > 0 {
        "candidate_better"
    } else if delta < 0 {
        "baseline_better"
    } else {
        "same_outcome"
    }
}

fn pro_policy_matrix_sorted_counts(
    counts: &BTreeMap<String, usize>,
    limit: usize,
) -> Vec<(&String, &usize)> {
    let mut entries = counts.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
    entries.into_iter().take(limit).collect()
}

fn pro_policy_matrix_move_decision_probe(board_fen: &str, move_fen: &str) -> String {
    let game = MonsGame::from_fen(board_fen, false).expect("valid policy matrix board fen");
    let config = apply_frontier_pro_v2_guarded_config(pro_budget().runtime_config_for_game(&game));

    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_engine_plan_cache();
    clear_turn_engine_diagnostics();
    clear_turn_engine_selector_diagnostics();
    let selected = select_sweep_frontier_config_inputs(&game, config);
    let selected_fen = Input::fen_from_array(&selected);
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (_, scored_roots, _, _) = runtime_scored_roots_with_config(&game, config);
    let root_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == move_fen);
    let advisor_ordered = advisor
        .as_ref()
        .and_then(|decision| {
            decision_record_root_advisor_entry_status(&decision.ordered_shortlist, move_fen)
        })
        .unwrap_or_else(|| "none".to_string());
    let advisor_preserved = advisor
        .as_ref()
        .and_then(|decision| {
            decision_record_root_advisor_entry_status(
                &decision.preserved_family_representatives,
                move_fen,
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let advisor_approved = advisor
        .as_ref()
        .and_then(|decision| decision.approved_root.as_ref())
        .filter(|entry| Input::fen_from_array(&entry.inputs) == move_fen)
        .map(format_root_advisor_entry_probe)
        .unwrap_or_else(|| "none".to_string());

    let Some(root_index) = root_index else {
        return format!(
            "selected={} root=omitted advisor_ordered={} advisor_preserved={} advisor_approved={}",
            selected_fen, advisor_ordered, advisor_preserved, advisor_approved
        );
    };

    let root = &scored_roots[root_index];
    let family = MonsGameModel::turn_engine_root_evaluation_family(root);
    let full_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        root,
        game.active_color,
        config,
        family,
    );
    let mut no_followup_config = config;
    no_followup_config.enable_turn_engine_selected_followup_projection = false;
    let no_followup_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        root,
        game.active_color,
        no_followup_config,
        family,
    );
    let followup_primary_order = crate::models::automove_turn_engine::compare_utility_primary_axes(
        full_utility,
        no_followup_utility,
    );

    format!(
        "selected={} root_index={} root_rank={} score={} family={:?} wins={} attack={} vulnerable={} walk_vulnerable={} spirit_setup={} spirit_dev={} super_progress={} opp_progress={} safe_super_steps={} safe_opp_steps={} score_path_steps={} same_turn_window={} full_utility={:?} no_followup_utility={:?} full_vs_no_followup_primary={:?} advisor_ordered={} advisor_preserved={} advisor_approved={}",
        selected_fen,
        root_index,
        root.root_rank,
        root.score,
        family,
        root.wins_immediately,
        root.attacks_opponent_drainer,
        root.own_drainer_vulnerable,
        root.own_drainer_walk_vulnerable,
        root.spirit_same_turn_score_setup_now || root.spirit_own_mana_setup_now,
        root.spirit_development,
        root.supermana_progress,
        root.opponent_mana_progress,
        root.safe_supermana_progress_steps,
        root.safe_opponent_mana_progress_steps,
        root.score_path_best_steps,
        root.same_turn_score_window_value,
        full_utility,
        no_followup_utility,
        followup_primary_order,
        advisor_ordered,
        advisor_preserved,
        advisor_approved,
    )
}

fn pro_policy_mechanism_profile_for_baseline(baseline_id: &str) -> &str {
    if profile_selector_from_name(baseline_id).is_some() {
        baseline_id
    } else {
        "frontier_pro_v2_guarded"
    }
}

fn pro_policy_target_root_utility_status_from_scored_roots(
    game: &MonsGame,
    config: AutomoveSearchConfig,
    scored_roots: &[RootEvaluation],
    move_fen: &str,
) -> String {
    let Some((index, root)) = scored_roots
        .iter()
        .enumerate()
        .find(|(_, root)| Input::fen_from_array(&root.inputs) == move_fen)
    else {
        return "root=omitted".to_string();
    };

    let family = MonsGameModel::turn_engine_root_evaluation_family(root);
    let utility = MonsGameModel::turn_engine_selected_override_utility(
        game,
        root,
        game.active_color,
        config,
        family,
    );

    format!(
        "root_index={} root_rank={} score={} family={:?} wins={} attack={} vulnerable={} walk_vulnerable={} spirit_setup={} spirit_dev={} super_progress={} opp_progress={} safe_super_steps={} safe_opp_steps={} score_path_steps={} same_turn_window={} utility={:?}",
        index,
        root.root_rank,
        root.score,
        family,
        root.wins_immediately,
        root.attacks_opponent_drainer,
        root.own_drainer_vulnerable,
        root.own_drainer_walk_vulnerable,
        root.spirit_same_turn_score_setup_now || root.spirit_own_mana_setup_now,
        root.spirit_development,
        root.supermana_progress,
        root.opponent_mana_progress,
        root.safe_supermana_progress_steps,
        root.safe_opponent_mana_progress_steps,
        root.score_path_best_steps,
        root.same_turn_score_window_value,
        utility,
    )
}

#[derive(Clone)]
struct ProPolicyMechanismRootClass {
    role: String,
    live: &'static str,
    family: String,
    advisor: String,
    safety: &'static str,
    progress: &'static str,
    rank: Option<usize>,
    score: Option<i32>,
    utility: Option<TurnEngineUtility>,
}

fn pro_policy_mechanism_rank_bucket(rank: Option<usize>) -> &'static str {
    match rank {
        Some(0) => "rank0",
        Some(1 | 2) => "rank1_2",
        Some(3..=5) => "rank3_5",
        Some(_) => "rank6_plus",
        None => "omitted",
    }
}

fn pro_policy_mechanism_rank_delta_bucket(
    winner_rank: Option<usize>,
    baseline_rank: Option<usize>,
) -> &'static str {
    match (winner_rank, baseline_rank) {
        (Some(winner), Some(baseline)) if winner == baseline => "same_rank",
        (Some(winner), Some(baseline)) if winner < baseline && baseline - winner <= 2 => {
            "winner_rank_better_1_2"
        }
        (Some(winner), Some(baseline)) if winner < baseline => "winner_rank_better_3_plus",
        (Some(winner), Some(baseline)) if winner > baseline && winner - baseline <= 2 => {
            "winner_rank_worse_1_2"
        }
        (Some(_), Some(_)) => "winner_rank_worse_3_plus",
        (Some(_), None) => "winner_live_baseline_omitted",
        (None, Some(_)) => "winner_omitted_baseline_live",
        (None, None) => "both_omitted",
    }
}

fn pro_policy_mechanism_score_order(left: Option<i32>, right: Option<i32>) -> &'static str {
    match (left, right) {
        (Some(left), Some(right)) => format_ordering_probe(left.cmp(&right)),
        (Some(_), None) => "left_live",
        (None, Some(_)) => "right_live",
        (None, None) => "both_omitted",
    }
}

fn pro_policy_mechanism_score_delta_bucket(
    winner_score: Option<i32>,
    baseline_score: Option<i32>,
) -> &'static str {
    match (winner_score, baseline_score) {
        (Some(winner), Some(baseline)) => {
            let delta = winner.saturating_sub(baseline);
            match delta {
                512.. => "winner_score_better_512_plus",
                96..=511 => "winner_score_better_96_511",
                0..=95 => "winner_score_same_or_close_better",
                -95..=-1 => "winner_score_worse_1_95",
                -511..=-96 => "winner_score_worse_96_511",
                _ => "winner_score_worse_512_plus",
            }
        }
        (Some(_), None) => "winner_live_baseline_omitted",
        (None, Some(_)) => "winner_omitted_baseline_live",
        (None, None) => "both_omitted",
    }
}

fn pro_policy_mechanism_utility_primary_order(
    left: Option<TurnEngineUtility>,
    right: Option<TurnEngineUtility>,
) -> &'static str {
    match (left, right) {
        (Some(left), Some(right)) => format_ordering_probe(
            crate::models::automove_turn_engine::compare_utility_primary_axes(left, right),
        ),
        (Some(_), None) => "left_live",
        (None, Some(_)) => "right_live",
        (None, None) => "both_omitted",
    }
}

fn pro_policy_mechanism_root_presence_bucket(root: &ProPolicyMechanismRootClass) -> &'static str {
    match root.live {
        "top3_live" => "top3_considered",
        "lower_live" => "lower_considered",
        _ => "omitted",
    }
}

fn pro_policy_mechanism_root_path_bucket(role: &str) -> &'static str {
    if role.split('+').any(|part| {
        matches!(
            part,
            "selected" | "pre_accept" | "head" | "legacy" | "legacy_full_pool"
        )
    }) {
        "selected_path"
    } else {
        "off_selected_path"
    }
}

fn pro_policy_mechanism_advisor_bucket(advisor: &str) -> &'static str {
    if advisor.starts_with("approved:") {
        "approved"
    } else if advisor.starts_with("ordered:") {
        "ordered"
    } else if advisor.starts_with("preserved:") {
        "preserved"
    } else if advisor.starts_with("injected_admitted:") {
        "injected_admitted"
    } else if advisor.starts_with("injected_rejected:") {
        "injected_rejected"
    } else if advisor == "advisor_none" {
        "advisor_none"
    } else {
        "unlisted"
    }
}

fn pro_policy_mechanism_root_preservation_signal(
    root: &ProPolicyMechanismRootClass,
) -> &'static str {
    let presence = pro_policy_mechanism_root_presence_bucket(root);
    let path = pro_policy_mechanism_root_path_bucket(&root.role);
    let advisor = pro_policy_mechanism_advisor_bucket(&root.advisor);

    if path == "selected_path" {
        "selected_path"
    } else if presence != "omitted" {
        match advisor {
            "approved" => "considered_approved_off_path",
            "ordered" => "considered_ordered_off_path",
            "preserved" => "considered_preserved_off_path",
            "injected_admitted" => "considered_injected_admitted_off_path",
            "injected_rejected" => "considered_injected_rejected_off_path",
            _ => "considered_unlisted_off_path",
        }
    } else {
        match advisor {
            "approved" => "omitted_approved",
            "ordered" => "omitted_ordered",
            "preserved" => "omitted_preserved",
            "injected_admitted" => "omitted_injected_admitted",
            "injected_rejected" => "omitted_injected_rejected",
            _ => "omitted_unlisted",
        }
    }
}

fn pro_policy_mechanism_value_bucket(value: i32, zero: &'static str) -> &'static str {
    match value {
        0 => zero,
        1 => "one",
        _ => "two_plus",
    }
}

fn pro_policy_mechanism_drainer_safety_bucket(value: i32) -> &'static str {
    match value {
        ..=-2 => "danger_2_plus",
        -1 => "danger_1",
        0 => "neutral",
        1.. => "safe",
    }
}

fn pro_policy_mechanism_turn_bucket(turn_number: i32) -> &'static str {
    match turn_number {
        0..=2 => "turn0_2",
        3..=4 => "turn3_4",
        5..=7 => "turn5_7",
        _ => "turn8_plus",
    }
}

fn pro_policy_mechanism_mons_moves_bucket(mons_moves_count: i32) -> &'static str {
    match mons_moves_count {
        0 => "mons0",
        1 => "mons1",
        2 => "mons2",
        _ => "mons3_plus",
    }
}

fn pro_policy_mechanism_exact_context_keys(game: &MonsGame) -> Vec<String> {
    let context = crate::models::automove_exact::exact_opportunity_context(game, game.active_color);
    let window =
        pro_policy_mechanism_value_bucket(context.delta.same_turn_score_window_value, "window0");
    let deny = pro_policy_mechanism_value_bucket(context.delta.opponent_window_deny_gain, "deny0");
    let drainer_safety = pro_policy_mechanism_drainer_safety_bucket(context.delta.drainer_safety);

    vec![
        format!(
            "axis=exact_pressure window={} deny={} attack={} drainer_safety={}",
            window, deny, context.delta.drainer_attack_available, drainer_safety,
        ),
        format!(
            "axis=exact_timing color={} turn_bucket={} mons_moves={} can_action={} can_mana={} opp_win={}",
            pro_profile_sweep_color_label(game.active_color),
            pro_policy_mechanism_turn_bucket(game.turn_number),
            pro_policy_mechanism_mons_moves_bucket(game.mons_moves_count),
            game.player_can_use_action(),
            game.player_can_move_mana(),
            context.opponent_can_win_immediately,
        ),
    ]
}

fn pro_policy_mechanism_advisor_class(
    advisor: Option<&crate::models::mons_game_model::ProV2RootAdvisorDecision>,
    move_fen: &str,
) -> String {
    let Some(advisor) = advisor else {
        return "advisor_none".to_string();
    };

    if let Some(entry) = advisor
        .approved_root
        .as_ref()
        .filter(|entry| Input::fen_from_array(&entry.inputs) == move_fen)
    {
        return format!("approved:{:?}:{:?}", entry.reason, entry.family);
    }
    if let Some(entry) = advisor
        .ordered_shortlist
        .iter()
        .find(|entry| Input::fen_from_array(&entry.inputs) == move_fen)
    {
        return format!("ordered:{:?}:{:?}", entry.reason, entry.family);
    }
    if let Some(entry) = advisor
        .preserved_family_representatives
        .iter()
        .find(|entry| Input::fen_from_array(&entry.inputs) == move_fen)
    {
        return format!("preserved:{:?}:{:?}", entry.reason, entry.family);
    }
    if let Some(injected) = advisor
        .injected_root
        .as_ref()
        .filter(|root| Input::fen_from_array(&root.inputs) == move_fen)
    {
        return format!(
            "injected_{}:{:?}:{:?}",
            if injected.admitted {
                "admitted"
            } else {
                "rejected"
            },
            injected.reason,
            injected.family,
        );
    }

    "advisor_unlisted".to_string()
}

fn pro_policy_mechanism_root_class(
    game: &MonsGame,
    config: AutomoveSearchConfig,
    scored_roots: &[RootEvaluation],
    probe: &RuntimeDecisionProbe,
    advisor: Option<&crate::models::mons_game_model::ProV2RootAdvisorDecision>,
    move_fen: &str,
) -> ProPolicyMechanismRootClass {
    let mut role_parts = Vec::new();
    if move_fen == probe.selected_input_fen {
        role_parts.push("selected");
    }
    if move_fen == probe.pre_accept_input_fen {
        role_parts.push("pre_accept");
    }
    if probe
        .head_input_fen
        .as_ref()
        .is_some_and(|head| head == move_fen)
    {
        role_parts.push("head");
    }
    if move_fen == probe.legacy_selected_input_fen {
        role_parts.push("legacy");
    }
    if move_fen == probe.legacy_full_pool_selected_input_fen {
        role_parts.push("legacy_full_pool");
    }
    if role_parts.is_empty() {
        role_parts.push("other");
    }

    let advisor_class = pro_policy_mechanism_advisor_class(advisor, move_fen);
    let Some((index, root)) = scored_roots
        .iter()
        .enumerate()
        .find(|(_, root)| Input::fen_from_array(&root.inputs) == move_fen)
    else {
        return ProPolicyMechanismRootClass {
            role: role_parts.join("+"),
            live: "candidate_omitted",
            family: "omitted".to_string(),
            advisor: advisor_class,
            safety: "omitted",
            progress: "omitted",
            rank: None,
            score: None,
            utility: None,
        };
    };

    let family = MonsGameModel::turn_engine_root_evaluation_family(root);
    let utility = MonsGameModel::turn_engine_selected_override_utility(
        game,
        root,
        game.active_color,
        config,
        family,
    );
    let safety = if root.mana_handoff_to_opponent || root.has_roundtrip {
        "handoff_or_roundtrip"
    } else if root.own_drainer_walk_vulnerable {
        "walk_vulnerable"
    } else if root.own_drainer_vulnerable {
        "vulnerable"
    } else {
        "safe"
    };
    let progress = if root.wins_immediately {
        "wins"
    } else if root.attacks_opponent_drainer {
        "attacks_drainer"
    } else if root.spirit_same_turn_score_setup_now || root.spirit_own_mana_setup_now {
        "spirit_setup"
    } else if root.spirit_development {
        "spirit_development"
    } else if root.supermana_progress && root.opponent_mana_progress {
        "both_mana_progress"
    } else if root.supermana_progress {
        "supermana_progress"
    } else if root.opponent_mana_progress {
        "opponent_mana_progress"
    } else if root.safe_supermana_progress_steps > 0 || root.safe_opponent_mana_progress_steps > 0 {
        "safe_step_progress"
    } else if root.same_turn_score_window_value > 0 {
        "score_window"
    } else if root.score_path_best_steps > 0 {
        "score_path"
    } else {
        "quiet"
    };

    ProPolicyMechanismRootClass {
        role: role_parts.join("+"),
        live: if index <= 2 {
            "top3_live"
        } else {
            "lower_live"
        },
        family: format!("{family:?}"),
        advisor: advisor_class,
        safety,
        progress,
        rank: Some(root.root_rank),
        score: Some(root.score),
        utility: Some(utility),
    }
}

fn pro_policy_mechanism_class_keys(
    mechanism_profile: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
    probe: &RuntimeDecisionProbe,
    advisor: Option<&crate::models::mons_game_model::ProV2RootAdvisorDecision>,
    baseline_move_fen: &str,
    winner_move_fen: &str,
) -> Vec<String> {
    let (config, scored_roots, _, _) =
        profile_runtime_scored_roots_with_forced_engine_inputs(mechanism_profile, mode, game);
    pro_policy_mechanism_class_keys_from_scored_roots(
        game,
        config,
        scored_roots.as_slice(),
        probe,
        advisor,
        baseline_move_fen,
        winner_move_fen,
    )
}

fn pro_policy_mechanism_class_keys_from_scored_roots(
    game: &MonsGame,
    config: AutomoveSearchConfig,
    scored_roots: &[RootEvaluation],
    probe: &RuntimeDecisionProbe,
    advisor: Option<&crate::models::mons_game_model::ProV2RootAdvisorDecision>,
    baseline_move_fen: &str,
    winner_move_fen: &str,
) -> Vec<String> {
    let baseline = pro_policy_mechanism_root_class(
        game,
        config,
        scored_roots,
        probe,
        advisor,
        baseline_move_fen,
    );
    let winner = pro_policy_mechanism_root_class(
        game,
        config,
        scored_roots,
        probe,
        advisor,
        winner_move_fen,
    );
    let winner_vs_baseline_utility =
        pro_policy_mechanism_utility_primary_order(winner.utility, baseline.utility);
    let winner_vs_baseline_score = pro_policy_mechanism_score_order(winner.score, baseline.score);
    let winner_rank_delta = pro_policy_mechanism_rank_delta_bucket(winner.rank, baseline.rank);
    let winner_score_delta = pro_policy_mechanism_score_delta_bucket(winner.score, baseline.score);
    let winner_presence = pro_policy_mechanism_root_presence_bucket(&winner);
    let winner_path = pro_policy_mechanism_root_path_bucket(&winner.role);
    let winner_advisor = pro_policy_mechanism_advisor_bucket(&winner.advisor);
    let winner_preservation_signal = pro_policy_mechanism_root_preservation_signal(&winner);

    let mut keys = vec![
        format!(
            "axis=stage baseline_stage={} head_accepted={} head_primary={:?} pre_family={:?} head_family={:?}",
            probe.selector_last_stage,
            probe.head_accepted,
            probe.head_plan_primary_axes_vs_pre_accept,
            probe.pre_accept_family,
            probe.head_family,
        ),
        format!(
            "axis=role baseline_role={} baseline_live={} winner_role={} winner_live={}",
            baseline.role, baseline.live, winner.role, winner.live,
        ),
        format!(
            "axis=family baseline_family={} winner_family={} winner_vs_baseline_primary={} winner_vs_baseline_score={}",
            baseline.family,
            winner.family,
            winner_vs_baseline_utility,
            winner_vs_baseline_score,
        ),
        format!(
            "axis=advisor baseline_advisor={} winner_advisor={}",
            baseline.advisor, winner.advisor,
        ),
        format!(
            "axis=rank baseline_rank={} winner_rank={} winner_vs_baseline_primary={} winner_vs_baseline_score={}",
            pro_policy_mechanism_rank_bucket(baseline.rank),
            pro_policy_mechanism_rank_bucket(winner.rank),
            winner_vs_baseline_utility,
            winner_vs_baseline_score,
        ),
        format!(
            "axis=rank_score_delta winner_rank_delta={} winner_score_delta={} winner_vs_baseline_primary={}",
            winner_rank_delta, winner_score_delta, winner_vs_baseline_utility,
        ),
        format!(
            "axis=root_preservation winner_presence={} winner_path={} winner_advisor={} winner_signal={} winner_rank_delta={}",
            winner_presence,
            winner_path,
            winner_advisor,
            winner_preservation_signal,
            winner_rank_delta,
        ),
        format!(
            "axis=safety_progress baseline_safety={} baseline_progress={} winner_safety={} winner_progress={}",
            baseline.safety, baseline.progress, winner.safety, winner.progress,
        ),
        format!(
            "axis=winner_root role={} live={} family={} advisor={} safety={} progress={} rank={}",
            winner.role,
            winner.live,
            winner.family,
            winner.advisor,
            winner.safety,
            winner.progress,
            pro_policy_mechanism_rank_bucket(winner.rank),
        ),
    ];
    keys.extend(pro_policy_mechanism_exact_context_keys(game));
    keys
}

fn pro_sweep_candidate_record_context_key(
    duel_label: &str,
    variant: GameVariant,
    outcome: &str,
    divergence: &ProProfileSweepFirstDivergence,
) -> String {
    format!(
        "outcome={} duel={} variant={} color={} branch={} turn={} mons_moves={} can_action={} can_mana={} {}",
        outcome,
        duel_label,
        automove_variant_label(variant),
        pro_profile_sweep_color_label(divergence.active_color),
        divergence.left_branch,
        divergence.turn_number,
        divergence.mons_moves_count,
        divergence.can_use_action,
        divergence.can_move_mana,
        divergence.exact_context,
    )
}

#[test]
#[ignore = "diagnostic: force each root once on a blocker board and continue with Pro policy"]
fn smart_automove_pro_forced_root_oracle_probe() {
    const DEFAULT_ALTERNATING_WHITE_NO_POLICY_FEN: &str =
        "0 0 w 0 0 3 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n11/n01xxmn01xxmn01xxmn01xxmn01xxmn01/xxQn04xxUn04xxQ/n01xxMn01xxMn01xxMn01xxMn01xxMn01/n11/n11/n04A0xn01S0xY0xn03/n03E0xn01D0xn05 4";

    let board_fen = env_raw_string_value("SMART_PRO_FORCED_ROOT_ORACLE_FEN")
        .unwrap_or_else(|| DEFAULT_ALTERNATING_WHITE_NO_POLICY_FEN.to_string());
    let label = env_string_value("SMART_PRO_FORCED_ROOT_ORACLE_LABEL")
        .unwrap_or_else(|| "sampled_alternating_white_no_policy".to_string());
    let continuation_id = env_string_value("SMART_PRO_FORCED_ROOT_ORACLE_CONTINUATION")
        .unwrap_or_else(|| "frontier_pro_v2_guarded".to_string());
    let root_source_id = env_string_value("SMART_PRO_FORCED_ROOT_ORACLE_ROOT_SOURCE")
        .unwrap_or_else(|| continuation_id.clone());
    let opponent_mode = env_string_value("SMART_PRO_FORCED_ROOT_ORACLE_OPPONENT_MODE")
        .map(|mode| mode.to_ascii_lowercase())
        .map(|mode| match mode.as_str() {
            "pro" => SmartAutomovePreference::Pro,
            "normal" => SmartAutomovePreference::Normal,
            "fast" => SmartAutomovePreference::Fast,
            _ => panic!(
                "unknown SMART_PRO_FORCED_ROOT_ORACLE_OPPONENT_MODE '{}'",
                mode
            ),
        })
        .unwrap_or(SmartAutomovePreference::Pro);
    let root_limit = env_usize("SMART_PRO_FORCED_ROOT_ORACLE_ROOT_LIMIT")
        .unwrap_or(32)
        .max(1);
    let max_plies = env_usize("SMART_PRO_FORCED_ROOT_ORACLE_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let start_ply = env_usize("SMART_PRO_FORCED_ROOT_ORACLE_START_PLY").unwrap_or(0);
    let rollout_max_plies = max_plies.saturating_sub(start_ply).max(1);
    let print_limit = env_usize("SMART_PRO_FORCED_ROOT_ORACLE_PRINT_LIMIT")
        .unwrap_or(32)
        .max(1);

    let game = MonsGame::from_fen(board_fen.as_str(), false).expect("valid oracle board fen");
    let candidate_is_white = game.active_color == Color::White;
    let continuation = pro_profile_sweep_candidate_by_id(continuation_id.as_str());
    let shipping_profile = reliability_shipping_profile_id();
    let shipping_selector = profile_selector_from_name(shipping_profile.as_str())
        .unwrap_or_else(|| panic!("shipping '{}' not found", shipping_profile));
    let opponent_budget = SearchBudget::from_preference(opponent_mode);
    let (runtime, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        root_source_id.as_str(),
        SmartAutomovePreference::Pro,
        &game,
    );

    println!(
        "forced root oracle: label={} continuation={} root_source={} shipping={} opponent_mode={:?} variant={} active_color={} roots={} root_limit={} max_plies={} start_ply={} rollout_max_plies={} fen={}",
        label,
        continuation.id,
        root_source_id,
        shipping_profile,
        opponent_mode,
        automove_variant_label(game.variant()),
        pro_profile_sweep_color_label(game.active_color),
        scored_roots.len(),
        root_limit,
        max_plies,
        start_ply,
        rollout_max_plies,
        game.fen(),
    );

    if scored_roots.is_empty() {
        let mut legal_transitions = MonsGameModel::enumerate_legal_transitions(
            &game,
            root_limit,
            MonsGameModel::automove_start_input_options(runtime),
        );
        if legal_transitions.is_empty() {
            legal_transitions = MonsGameModel::enumerate_legal_transitions(
                &game,
                root_limit,
                SuggestedStartInputOptions::for_automove(),
            );
        }
        let mut rows = legal_transitions
            .into_iter()
            .map(|transition| {
                let inputs = Input::fen_from_array(&transition.inputs);
                let trace = play_profile_sweep_forced_first_candidate_turn(
                    continuation,
                    shipping_selector,
                    opponent_budget,
                    board_fen.as_str(),
                    candidate_is_white,
                    rollout_max_plies,
                    transition.inputs.as_slice(),
                );
                (
                    trace.result,
                    inputs,
                    format!("{:?}", transition.events),
                    trace.final_fen,
                )
            })
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| {
            match_result_points(right.0)
                .cmp(&match_result_points(left.0))
                .then_with(|| left.1.cmp(&right.1))
        });

        let wins = rows
            .iter()
            .filter(|row| matches!(row.0, MatchResult::ProfileAWin))
            .count();
        let draws = rows
            .iter()
            .filter(|row| matches!(row.0, MatchResult::Draw))
            .count();
        println!(
            "FORCED_ROOT_ORACLE_SUMMARY {{\"label\":\"{}\",\"continuation\":\"{}\",\"root_source\":\"{}\",\"opponent_mode\":\"{:?}\",\"variant\":\"{}\",\"active_color\":\"{}\",\"source\":\"legal_transitions\",\"max_plies\":{},\"start_ply\":{},\"rollout_max_plies\":{},\"tested_roots\":{},\"wins\":{},\"draws\":{},\"losses\":{}}}",
            json_escape(&label),
            json_escape(continuation.id),
            json_escape(&root_source_id),
            opponent_mode,
            automove_variant_label(game.variant()),
            pro_profile_sweep_color_label(game.active_color),
            max_plies,
            start_ply,
            rollout_max_plies,
            rows.len(),
            wins,
            draws,
            rows.len().saturating_sub(wins + draws),
        );

        for (result, inputs, events, final_fen) in rows.into_iter().take(print_limit) {
            println!(
                "FORCED_ROOT_ORACLE_LEGAL_ROOT {{\"label\":\"{}\",\"result\":\"{}\",\"inputs\":\"{}\",\"events\":\"{}\",\"final\":\"{}\"}}",
                json_escape(&label),
                format_match_result(result),
                json_escape(&inputs),
                json_escape(&events),
                json_escape(&final_fen),
            );
        }
        return;
    }

    let mut rows = scored_roots
        .iter()
        .take(root_limit)
        .map(|root| {
            let trace = play_profile_sweep_forced_first_candidate_turn(
                continuation,
                shipping_selector,
                opponent_budget,
                board_fen.as_str(),
                candidate_is_white,
                rollout_max_plies,
                root.inputs.as_slice(),
            );
            let family = MonsGameModel::turn_engine_root_evaluation_family(root);
            let utility = MonsGameModel::turn_engine_selected_override_utility(
                &game,
                root,
                game.active_color,
                runtime,
                family,
            );
            (
                trace.result,
                root.root_rank,
                root.score,
                Input::fen_from_array(&root.inputs),
                family,
                root.wins_immediately,
                root.attacks_opponent_drainer,
                root.own_drainer_vulnerable,
                root.spirit_development,
                root.spirit_same_turn_score_setup_now || root.spirit_own_mana_setup_now,
                root.supermana_progress,
                root.opponent_mana_progress,
                root.safe_supermana_progress_steps,
                root.safe_opponent_mana_progress_steps,
                root.same_turn_score_window_value,
                utility,
                trace.final_fen,
            )
        })
        .collect::<Vec<_>>();

    rows.sort_by(|left, right| {
        match_result_points(right.0)
            .cmp(&match_result_points(left.0))
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| right.2.cmp(&left.2))
            .then_with(|| left.3.cmp(&right.3))
    });

    let wins = rows
        .iter()
        .filter(|row| matches!(row.0, MatchResult::ProfileAWin))
        .count();
    let draws = rows
        .iter()
        .filter(|row| matches!(row.0, MatchResult::Draw))
        .count();
    println!(
        "FORCED_ROOT_ORACLE_SUMMARY {{\"label\":\"{}\",\"continuation\":\"{}\",\"root_source\":\"{}\",\"opponent_mode\":\"{:?}\",\"variant\":\"{}\",\"active_color\":\"{}\",\"max_plies\":{},\"start_ply\":{},\"rollout_max_plies\":{},\"tested_roots\":{},\"wins\":{},\"draws\":{},\"losses\":{}}}",
        json_escape(&label),
        json_escape(continuation.id),
        json_escape(&root_source_id),
        opponent_mode,
        automove_variant_label(game.variant()),
        pro_profile_sweep_color_label(game.active_color),
        max_plies,
        start_ply,
        rollout_max_plies,
        rows.len(),
        wins,
        draws,
        rows.len().saturating_sub(wins + draws),
    );

    for (
        result,
        root_rank,
        score,
        inputs,
        family,
        wins_immediately,
        attacks,
        vulnerable,
        spirit_development,
        spirit_setup,
        supermana_progress,
        opponent_mana_progress,
        safe_super_steps,
        safe_opp_steps,
        same_turn_window,
        utility,
        final_fen,
    ) in rows.into_iter().take(print_limit)
    {
        println!(
            "FORCED_ROOT_ORACLE_ROOT {{\"label\":\"{}\",\"result\":\"{}\",\"root_rank\":{},\"score\":{},\"inputs\":\"{}\",\"family\":\"{:?}\",\"wins_immediately\":{},\"attacks\":{},\"vulnerable\":{},\"spirit_development\":{},\"spirit_setup\":{},\"supermana_progress\":{},\"opponent_mana_progress\":{},\"safe_super_steps\":{},\"safe_opp_steps\":{},\"same_turn_window\":{},\"utility\":\"{:?}\",\"final\":\"{}\"}}",
            json_escape(&label),
            format_match_result(result),
            root_rank,
            score,
            json_escape(&inputs),
            family,
            wins_immediately,
            attacks,
            vulnerable,
            spirit_development,
            spirit_setup,
            supermana_progress,
            opponent_mana_progress,
            safe_super_steps,
            safe_opp_steps,
            same_turn_window,
            utility,
            json_escape(&final_fen),
        );
    }
}

#[test]
#[ignore = "diagnostic: attribute ProV2 candidate outcome changes to first left-candidate branch divergence"]
fn smart_automove_pro_profile_attribution_probe() {
    #[derive(Clone)]
    struct AttributionDuelSpec {
        label: &'static str,
        opponent_mode: SmartAutomovePreference,
        seed_tag: String,
    }

    let shipping_profile = reliability_shipping_profile_id();
    let shipping_selector = profile_selector_from_name(shipping_profile.as_str())
        .unwrap_or_else(|| panic!("shipping '{}' not found", shipping_profile));
    let repeats = env_usize("SMART_PRO_SWEEP_REPEATS").unwrap_or(1).max(1);
    let games = env_usize("SMART_PRO_SWEEP_GAMES").unwrap_or(3).max(1);
    let max_plies = env_usize("SMART_PRO_SWEEP_MAX_PLIES").unwrap_or(96).max(56);
    let trace_limit = env_usize("SMART_PRO_SWEEP_TRACE_LIMIT")
        .unwrap_or(16)
        .max(1);
    let pair_limit = env_usize("SMART_PRO_SWEEP_PAIR_LIMIT").unwrap_or(12).max(1);
    let seed_tag = env_string_value("SMART_PRO_SWEEP_SEED_TAG")
        .unwrap_or_else(|| "pro_profile_sweep_v1".to_string());
    let duel_filter = pro_sweep_filter_tokens("SMART_PRO_SWEEP_DUEL_FILTER", "all");
    let left_id = env_string_value("SMART_PRO_SWEEP_ATTRIBUTION_LEFT")
        .unwrap_or_else(|| "frontier_pro_v2_guarded".to_string());
    let right_id = env_string_value("SMART_PRO_SWEEP_ATTRIBUTION_RIGHT")
        .unwrap_or_else(|| "frontier_pro_v2_raw".to_string());
    let duel_specs = [
        AttributionDuelSpec {
            label: "vs_shipping_pro",
            opponent_mode: SmartAutomovePreference::Pro,
            seed_tag: seed_tag.clone(),
        },
        AttributionDuelSpec {
            label: "vs_shipping_normal",
            opponent_mode: SmartAutomovePreference::Normal,
            seed_tag: format!("{}_vs_normal", seed_tag),
        },
        AttributionDuelSpec {
            label: "vs_shipping_fast",
            opponent_mode: SmartAutomovePreference::Fast,
            seed_tag: format!("{}_vs_fast", seed_tag),
        },
    ];
    let left = pro_profile_sweep_candidate_by_id(left_id.as_str());
    let right = pro_profile_sweep_candidate_by_id(right_id.as_str());

    println!(
        "pro profile attribution: left={} right={} shipping={} duels={} repeats={} games={} max_plies={} variants={}",
        left.id,
        right.id,
        shipping_profile,
        duel_specs
            .iter()
            .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
            .map(|duel| duel.label)
            .collect::<Vec<_>>()
            .join(","),
        repeats,
        games,
        max_plies,
        env::var("SMART_AUTOMOVE_VARIANTS").unwrap_or_else(|_| "<default>".to_string())
    );

    for duel in duel_specs
        .iter()
        .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
    {
        let opponent_budget = SearchBudget::from_preference(duel.opponent_mode);
        let mut total_games = 0usize;
        let mut right_better = 0usize;
        let mut left_better = 0usize;
        let mut same_outcome = 0usize;
        let mut missing_first_diff = 0usize;
        let mut printed = 0usize;
        let mut branch_counts = BTreeMap::<(&'static str, &'static str), usize>::new();
        let mut context_counts = BTreeMap::<(&'static str, String), usize>::new();
        let mut pair_counts =
            BTreeMap::<(&'static str, &'static str, String, String), usize>::new();

        for repeat_index in 0..repeats {
            let seed = seed_for_budget_duel_repeat_and_tag(
                pro_budget(),
                opponent_budget,
                repeat_index,
                duel.seed_tag.as_str(),
            );
            let opening_fens = generate_opening_fens_cached(seed, games);
            for (game_index, opening_fen) in opening_fens.iter().enumerate() {
                let variant = MonsGame::from_fen(opening_fen.as_str(), false)
                    .expect("valid opening fen")
                    .variant();
                for candidate_is_white in [true, false] {
                    total_games += 1;
                    let left_trace = play_profile_sweep_attribution_trace(
                        left,
                        shipping_selector,
                        opponent_budget,
                        opening_fen.as_str(),
                        candidate_is_white,
                        max_plies,
                    );
                    let right_trace = play_profile_sweep_attribution_trace(
                        right,
                        shipping_selector,
                        opponent_budget,
                        opening_fen.as_str(),
                        candidate_is_white,
                        max_plies,
                    );
                    let delta = match_result_points(right_trace.result)
                        - match_result_points(left_trace.result);
                    let outcome = pro_profile_sweep_outcome_label(delta);
                    match delta.cmp(&0) {
                        std::cmp::Ordering::Greater => right_better += 1,
                        std::cmp::Ordering::Less => left_better += 1,
                        std::cmp::Ordering::Equal => same_outcome += 1,
                    }

                    if delta == 0 {
                        continue;
                    }

                    let first_divergence =
                        first_profile_sweep_candidate_divergence(&left_trace, &right_trace);
                    let Some(divergence) = first_divergence else {
                        missing_first_diff += 1;
                        continue;
                    };
                    *branch_counts
                        .entry((outcome, divergence.left_branch))
                        .or_default() += 1;
                    let context_key = format!(
                        "variant={} {}",
                        automove_variant_label(variant),
                        pro_profile_sweep_divergence_context_key(&divergence)
                    );
                    *context_counts
                        .entry((outcome, context_key.clone()))
                        .or_default() += 1;
                    *pair_counts
                        .entry((
                            outcome,
                            divergence.left_branch,
                            divergence.left_move_fen.clone(),
                            divergence.right_move_fen.clone(),
                        ))
                        .or_default() += 1;

                    if printed < trace_limit {
                        println!(
                            "PRO_PROFILE_SWEEP_ATTRIBUTION {{\"left\":\"{}\",\"right\":\"{}\",\"duel\":\"{}\",\"repeat\":{},\"opening_index\":{},\"variant\":\"{}\",\"candidate_is_white\":{},\"outcome\":\"{}\",\"right_delta\":{},\"left_result\":\"{}\",\"right_result\":\"{}\",\"first_diff_ply\":{},\"left_branch\":\"{}\",\"active_color\":\"{}\",\"turn\":{},\"mons_moves\":{},\"can_action\":{},\"can_mana\":{},\"exact_context\":\"{}\",\"board\":\"{}\",\"left_move\":\"{}\",\"right_move\":\"{}\",\"left_final\":\"{}\",\"right_final\":\"{}\"}}",
                            json_escape(left.id),
                            json_escape(right.id),
                            json_escape(duel.label),
                            repeat_index,
                            game_index,
                            automove_variant_label(variant),
                            candidate_is_white,
                            outcome,
                            delta,
                            format_match_result(left_trace.result),
                            format_match_result(right_trace.result),
                            divergence.ply,
                            json_escape(divergence.left_branch),
                            pro_profile_sweep_color_label(divergence.active_color),
                            divergence.turn_number,
                            divergence.mons_moves_count,
                            divergence.can_use_action,
                            divergence.can_move_mana,
                            json_escape(&divergence.exact_context),
                            json_escape(&divergence.board_fen),
                            json_escape(&divergence.left_move_fen),
                            json_escape(&divergence.right_move_fen),
                            json_escape(&left_trace.final_fen),
                            json_escape(&right_trace.final_fen)
                        );
                        printed += 1;
                    }
                }
            }
        }

        println!(
            "PRO_PROFILE_SWEEP_ATTRIBUTION_SUMMARY {{\"left\":\"{}\",\"right\":\"{}\",\"duel\":\"{}\",\"total_games\":{},\"right_better\":{},\"left_better\":{},\"same_outcome\":{},\"missing_first_diff\":{}}}",
            json_escape(left.id),
            json_escape(right.id),
            json_escape(duel.label),
            total_games,
            right_better,
            left_better,
            same_outcome,
            missing_first_diff
        );
        for ((outcome, branch), games) in branch_counts.iter() {
            println!(
                "PRO_PROFILE_SWEEP_ATTRIBUTION_BRANCH {{\"left\":\"{}\",\"right\":\"{}\",\"duel\":\"{}\",\"outcome\":\"{}\",\"left_branch\":\"{}\",\"games\":{}}}",
                json_escape(left.id),
                json_escape(right.id),
                json_escape(duel.label),
                json_escape(outcome),
                json_escape(branch),
                games
            );
        }
        for ((outcome, context), games) in context_counts.iter() {
            println!(
                "PRO_PROFILE_SWEEP_ATTRIBUTION_CONTEXT {{\"left\":\"{}\",\"right\":\"{}\",\"duel\":\"{}\",\"outcome\":\"{}\",\"context\":\"{}\",\"games\":{}}}",
                json_escape(left.id),
                json_escape(right.id),
                json_escape(duel.label),
                json_escape(outcome),
                json_escape(context),
                games
            );
        }
        for ((outcome, branch, left_move, right_move), games) in pair_counts.iter().take(pair_limit)
        {
            println!(
                "PRO_PROFILE_SWEEP_ATTRIBUTION_PAIR {{\"left\":\"{}\",\"right\":\"{}\",\"duel\":\"{}\",\"outcome\":\"{}\",\"left_branch\":\"{}\",\"left_move\":\"{}\",\"right_move\":\"{}\",\"games\":{}}}",
                json_escape(left.id),
                json_escape(right.id),
                json_escape(duel.label),
                json_escape(outcome),
                json_escape(branch),
                json_escape(left_move),
                json_escape(right_move),
                games
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: compare multiple Pro sweep policies on identical sampled and active panels"]
fn smart_automove_pro_policy_matrix_probe() {
    #[derive(Clone)]
    struct PolicyMatrixDuelSpec {
        label: &'static str,
        opponent_mode: SmartAutomovePreference,
        seed_suffix: &'static str,
    }

    #[derive(Default)]
    struct PolicyMatrixCandidateStats {
        total_games: usize,
        candidate_better: usize,
        baseline_better: usize,
        same_outcome: usize,
        candidate_nonwins: usize,
        baseline_nonwins: usize,
        first_move_diffs: usize,
        recorded: usize,
        missing_first_diff: usize,
    }

    #[derive(Default)]
    struct PolicyMatrixPortfolioStats {
        total_games: usize,
        baseline_wins: usize,
        candidate_any_wins: usize,
        any_policy_wins: usize,
        shared_wins: usize,
        baseline_only_wins: usize,
        candidate_only_wins: usize,
        no_policy_wins: usize,
    }

    #[derive(Default)]
    struct PolicyMatrixMechanismRouteCoverage {
        candidate_only_games: usize,
        baseline_better_games: usize,
        candidate_only_states: BTreeSet<String>,
        baseline_better_states: BTreeSet<String>,
        candidate_only_panels: BTreeSet<String>,
        candidate_only_duels: BTreeSet<String>,
        candidate_only_policies: BTreeSet<String>,
        candidate_only_variants: BTreeSet<String>,
        candidate_only_colors: BTreeSet<String>,
        candidate_only_branches: BTreeSet<String>,
        candidate_only_pairs: BTreeSet<String>,
        baseline_better_panels: BTreeSet<String>,
        baseline_better_duels: BTreeSet<String>,
        baseline_better_policies: BTreeSet<String>,
        baseline_better_variants: BTreeSet<String>,
        baseline_better_colors: BTreeSet<String>,
        baseline_better_branches: BTreeSet<String>,
        baseline_better_pairs: BTreeSet<String>,
    }

    #[derive(Default)]
    struct PolicyMatrixRecordFilterStats {
        corpus_records: usize,
        trace_records: usize,
        panels: BTreeSet<String>,
        duels: BTreeSet<String>,
        candidates: BTreeSet<String>,
        outcomes: BTreeSet<String>,
        portfolio_classes: BTreeSet<String>,
        variants: BTreeSet<String>,
        colors: BTreeSet<String>,
        branches: BTreeSet<String>,
        pairs: BTreeSet<String>,
        duel_counts: BTreeMap<String, usize>,
        candidate_counts: BTreeMap<String, usize>,
        outcome_counts: BTreeMap<String, usize>,
        portfolio_class_counts: BTreeMap<String, usize>,
        variant_counts: BTreeMap<String, usize>,
        color_counts: BTreeMap<String, usize>,
        branch_counts: BTreeMap<String, usize>,
        pair_counts: BTreeMap<String, usize>,
    }

    let shipping_profile = reliability_shipping_profile_id();
    let shipping_selector = profile_selector_from_name(shipping_profile.as_str())
        .unwrap_or_else(|| panic!("shipping '{}' not found", shipping_profile));
    let candidates = pro_profile_sweep_candidate_list_from_env(
        "SMART_PRO_POLICY_MATRIX_CANDIDATES",
        "frontier_pro_v2_guarded,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard",
    );
    assert!(
        candidates.len() >= 2,
        "SMART_PRO_POLICY_MATRIX_CANDIDATES must name at least a baseline and one candidate"
    );
    let baseline = candidates[0];
    let panel_filter = pro_sweep_filter_tokens("SMART_PRO_POLICY_MATRIX_PANEL_FILTER", "all");
    let duel_filter = pro_sweep_filter_tokens("SMART_PRO_POLICY_MATRIX_DUEL_FILTER", "all");
    let max_plies = env_usize("SMART_PRO_POLICY_MATRIX_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let trace_limit = env_usize("SMART_PRO_POLICY_MATRIX_TRACE_LIMIT")
        .unwrap_or(24)
        .max(1);
    let state_limit = env_usize("SMART_PRO_POLICY_MATRIX_STATE_LIMIT").map(|limit| limit.max(1));
    let total_state_limit =
        env_usize("SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT").map(|limit| limit.max(1));
    let aggregate_limit = env_usize("SMART_PRO_POLICY_MATRIX_AGGREGATE_LIMIT")
        .unwrap_or(64)
        .max(1);
    let route_bucket_limit = env_usize("SMART_PRO_POLICY_MATRIX_ROUTE_BUCKET_LIMIT")
        .unwrap_or(3)
        .max(1);
    let record_filter_detail_limit =
        env_usize("SMART_PRO_POLICY_MATRIX_RECORD_FILTER_DETAIL_LIMIT")
            .unwrap_or(8)
            .max(1);
    let include_decision_probe =
        env_bool("SMART_PRO_POLICY_MATRIX_INCLUDE_DECISION_PROBE").unwrap_or(false);
    let include_mechanism_class =
        env_bool("SMART_PRO_POLICY_MATRIX_INCLUDE_MECHANISM_CLASS").unwrap_or(false);
    let include_portfolio_mechanism_class =
        env_bool("SMART_PRO_POLICY_MATRIX_INCLUDE_PORTFOLIO_MECHANISM_CLASS").unwrap_or(false);
    let include_corpus_records =
        env_bool("SMART_PRO_POLICY_MATRIX_INCLUDE_CORPUS_RECORDS").unwrap_or(false);
    let global_only = env_bool("SMART_PRO_POLICY_MATRIX_GLOBAL_ONLY").unwrap_or(false);
    let record_axis_filter =
        pro_sweep_filter_tokens("SMART_PRO_POLICY_MATRIX_RECORD_AXIS_FILTER", "all");
    let record_axis_filter_all = record_axis_filter.iter().any(|token| token == "all");
    let record_axis_filter_label = record_axis_filter.join(",");
    let candidate_ids = candidates
        .iter()
        .skip(1)
        .map(|candidate| candidate.id)
        .collect::<Vec<_>>()
        .join(",");
    let duel_specs = [
        PolicyMatrixDuelSpec {
            label: "vs_shipping_pro",
            opponent_mode: SmartAutomovePreference::Pro,
            seed_suffix: "",
        },
        PolicyMatrixDuelSpec {
            label: "vs_shipping_normal",
            opponent_mode: SmartAutomovePreference::Normal,
            seed_suffix: "_vs_normal",
        },
        PolicyMatrixDuelSpec {
            label: "vs_shipping_fast",
            opponent_mode: SmartAutomovePreference::Fast,
            seed_suffix: "_vs_fast",
        },
    ];

    let portfolio_stoplight_label = |stats: &PolicyMatrixPortfolioStats,
                                     winner_counts: &BTreeMap<String, usize>,
                                     winner_context_counts: &BTreeMap<String, usize>,
                                     winner_pair_counts: &BTreeMap<String, usize>,
                                     winner_mechanism_class_counts: &BTreeMap<String, usize>|
     -> &'static str {
        if stats.no_policy_wins > 0 {
            "coverage_gap"
        } else if stats.baseline_only_wins > 0 {
            "baseline_save_risk"
        } else if stats.candidate_only_wins == 0 {
            "shared_only"
        } else if max_count(winner_pair_counts) > 1 {
            "repeated_winner_pair"
        } else if max_count(winner_context_counts) > 1 {
            "repeated_winner_context"
        } else if include_portfolio_mechanism_class && max_count(winner_mechanism_class_counts) > 2
        {
            "repeated_mechanism_class"
        } else if max_count(winner_counts) > 1 {
            "repeated_winner_policy"
        } else {
            "singleton_selector_pressure"
        }
    };
    let record_axis_filter_allows = |mechanism_axes: &str,
                                     baseline_better_mechanism_axes: &str,
                                     timing_continuation_axes: &str|
     -> bool {
        if record_axis_filter_all {
            return true;
        }
        let mechanism_axes = mechanism_axes.to_ascii_lowercase();
        let baseline_better_mechanism_axes = baseline_better_mechanism_axes.to_ascii_lowercase();
        let timing_continuation_axes = timing_continuation_axes.to_ascii_lowercase();
        record_axis_filter.iter().any(|token| {
            mechanism_axes.contains(token)
                || baseline_better_mechanism_axes.contains(token)
                || timing_continuation_axes.contains(token)
        })
    };
    let add_record_filter_breakdown = |stats: &mut PolicyMatrixRecordFilterStats,
                                       panel: &str,
                                       duel: &str,
                                       candidate: &str,
                                       outcome: &str,
                                       portfolio_class: &str,
                                       variant: &str,
                                       color: &str,
                                       branch: String,
                                       pair: String| {
        stats.panels.insert(panel.to_string());
        stats.duels.insert(duel.to_string());
        stats.candidates.insert(candidate.to_string());
        stats.outcomes.insert(outcome.to_string());
        stats.portfolio_classes.insert(portfolio_class.to_string());
        stats.variants.insert(variant.to_string());
        stats.colors.insert(color.to_string());
        stats.branches.insert(branch.clone());
        stats.pairs.insert(pair.clone());

        *stats.duel_counts.entry(duel.to_string()).or_default() += 1;
        *stats
            .candidate_counts
            .entry(candidate.to_string())
            .or_default() += 1;
        *stats.outcome_counts.entry(outcome.to_string()).or_default() += 1;
        *stats
            .portfolio_class_counts
            .entry(portfolio_class.to_string())
            .or_default() += 1;
        *stats.variant_counts.entry(variant.to_string()).or_default() += 1;
        *stats.color_counts.entry(color.to_string()).or_default() += 1;
        *stats.branch_counts.entry(branch).or_default() += 1;
        *stats.pair_counts.entry(pair).or_default() += 1;
    };

    println!(
        "pro policy matrix: baseline={} candidates={} panels={} duels={} max_plies={} state_limit={:?} total_state_limit={:?} include_decision_probe={} include_mechanism_class={} include_portfolio_mechanism_class={} include_corpus_records={} global_only={} record_axis_filter={}",
        baseline.id,
        candidate_ids,
        pro_promotion_dashboard_panel_specs()
            .into_iter()
            .filter(|panel| pro_sweep_filter_allows(&panel_filter, panel.label))
            .map(|panel| panel.label)
            .collect::<Vec<_>>()
            .join(","),
        duel_specs
            .iter()
            .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
            .map(|duel| duel.label)
            .collect::<Vec<_>>()
            .join(","),
        max_plies,
        state_limit,
        total_state_limit,
        include_decision_probe,
        include_mechanism_class,
        include_portfolio_mechanism_class,
        include_corpus_records,
        global_only,
        record_axis_filter_label,
    );

    let mut global_portfolio_stats = PolicyMatrixPortfolioStats::default();
    let mut global_portfolio_class_counts = BTreeMap::<String, usize>::new();
    let mut global_portfolio_winner_counts = BTreeMap::<String, usize>::new();
    let mut global_portfolio_winner_context_counts = BTreeMap::<String, usize>::new();
    let mut global_portfolio_winner_pair_counts = BTreeMap::<String, usize>::new();
    let mut global_portfolio_winner_mechanism_class_counts = BTreeMap::<String, usize>::new();
    let mut global_portfolio_baseline_better_mechanism_class_counts =
        BTreeMap::<String, usize>::new();
    let mut global_mechanism_axis_routes =
        BTreeMap::<String, PolicyMatrixMechanismRouteCoverage>::new();
    let mut record_filter_stats = PolicyMatrixRecordFilterStats::default();
    let mut global_state_limit_hit = false;
    let mut total_states_seen = 0usize;

    'panels: for panel in pro_promotion_dashboard_panel_specs()
        .into_iter()
        .filter(|panel| pro_sweep_filter_allows(&panel_filter, panel.label))
    {
        if total_state_limit.is_some_and(|limit| total_states_seen >= limit) {
            global_state_limit_hit = true;
            break 'panels;
        }
        let repeats = env_usize("SMART_PRO_POLICY_MATRIX_REPEATS")
            .unwrap_or(panel.default_repeats)
            .max(1);
        let games = env_usize("SMART_PRO_POLICY_MATRIX_GAMES")
            .unwrap_or(panel.default_games)
            .max(1);
        let panel_seed_tag = env_string_value("SMART_PRO_POLICY_MATRIX_SEED_TAG")
            .unwrap_or_else(|| panel.seed_tag.to_string());

        with_pro_promotion_dashboard_panel(panel, || {
            for duel in duel_specs
                .iter()
                .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
            {
                if total_state_limit.is_some_and(|limit| total_states_seen >= limit) {
                    global_state_limit_hit = true;
                    break;
                }
                let opponent_budget = SearchBudget::from_preference(duel.opponent_mode);
                let duel_seed_tag = format!("{}{}", panel_seed_tag, duel.seed_suffix);
                let mut stats_by_candidate = candidates
                    .iter()
                    .skip(1)
                    .map(|candidate| (*candidate, PolicyMatrixCandidateStats::default()))
                    .collect::<Vec<_>>();
                let mut branch_counts = BTreeMap::<String, usize>::new();
                let mut context_counts = BTreeMap::<String, usize>::new();
                let mut pair_counts = BTreeMap::<String, usize>::new();
                let mut mechanism_class_counts = BTreeMap::<String, usize>::new();
                let mut portfolio_class_counts = BTreeMap::<String, usize>::new();
                let mut portfolio_winner_counts = BTreeMap::<String, usize>::new();
                let mut portfolio_winner_context_counts = BTreeMap::<String, usize>::new();
                let mut portfolio_winner_pair_counts = BTreeMap::<String, usize>::new();
                let mut portfolio_winner_mechanism_class_counts = BTreeMap::<String, usize>::new();
                let mut portfolio_baseline_better_mechanism_class_counts =
                    BTreeMap::<String, usize>::new();
                let mut portfolio_mechanism_axis_routes =
                    BTreeMap::<String, PolicyMatrixMechanismRouteCoverage>::new();
                let mut portfolio_stats = PolicyMatrixPortfolioStats::default();
                let mut printed = 0usize;
                let mut state_limit_hit = false;

                'states: for repeat_index in 0..repeats {
                    let seed = seed_for_budget_duel_repeat_and_tag(
                        pro_budget(),
                        opponent_budget,
                        repeat_index,
                        duel_seed_tag.as_str(),
                    );
                    let opening_fens = generate_opening_fens_cached(seed, games);
                    for (game_index, opening_fen) in opening_fens.iter().enumerate() {
                        let variant = MonsGame::from_fen(opening_fen.as_str(), false)
                            .expect("valid opening fen")
                            .variant();
                        for candidate_is_white in [true, false] {
                            if state_limit.is_some_and(|limit| portfolio_stats.total_games >= limit)
                            {
                                state_limit_hit = true;
                                break 'states;
                            }
                            if total_state_limit.is_some_and(|limit| total_states_seen >= limit) {
                                state_limit_hit = true;
                                global_state_limit_hit = true;
                                break 'states;
                            }
                            let traces = candidates
                                .iter()
                                .map(|candidate| {
                                    (
                                        *candidate,
                                        play_profile_sweep_attribution_trace(
                                            *candidate,
                                            shipping_selector,
                                            opponent_budget,
                                            opening_fen.as_str(),
                                            candidate_is_white,
                                            max_plies,
                                        ),
                                    )
                                })
                                .collect::<Vec<_>>();
                            let baseline_trace = &traces[0].1;
                            let baseline_won =
                                matches!(baseline_trace.result, MatchResult::ProfileAWin);
                            let candidate_any_won = traces
                                .iter()
                                .skip(1)
                                .any(|(_, trace)| matches!(trace.result, MatchResult::ProfileAWin));
                            portfolio_stats.total_games += 1;
                            if baseline_won {
                                portfolio_stats.baseline_wins += 1;
                            }
                            if candidate_any_won {
                                portfolio_stats.candidate_any_wins += 1;
                            }
                            if baseline_won || candidate_any_won {
                                portfolio_stats.any_policy_wins += 1;
                            }
                            match (baseline_won, candidate_any_won) {
                                (true, true) => portfolio_stats.shared_wins += 1,
                                (true, false) => portfolio_stats.baseline_only_wins += 1,
                                (false, true) => portfolio_stats.candidate_only_wins += 1,
                                (false, false) => portfolio_stats.no_policy_wins += 1,
                            }
                            let portfolio_class = match (baseline_won, candidate_any_won) {
                                (true, true) => "shared_win",
                                (true, false) => "baseline_only_win",
                                (false, true) => "candidate_only_win",
                                (false, false) => "no_policy_win",
                            };
                            let portfolio_class_key = format!(
                                "class={} variant={} candidate_is_white={}",
                                portfolio_class,
                                automove_variant_label(variant),
                                candidate_is_white,
                            );
                            total_states_seen += 1;
                            let portfolio_state_key = format!(
                                "panel={} duel={} seed_tag={} repeat={} opening_index={} variant={} candidate_is_white={}",
                                panel.label,
                                duel.label,
                                duel_seed_tag,
                                repeat_index,
                                game_index,
                                automove_variant_label(variant),
                                candidate_is_white,
                            );
                            *portfolio_class_counts
                                .entry(portfolio_class_key)
                                .or_default() += 1;
                            let policy_results = traces
                                .iter()
                                .map(|(candidate, trace)| {
                                    format!(
                                        "{}={}",
                                        candidate.id,
                                        format_match_result(trace.result)
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("|");
                            let winning_policies = traces
                                .iter()
                                .filter(|(_, trace)| {
                                    matches!(trace.result, MatchResult::ProfileAWin)
                                })
                                .map(|(candidate, _)| candidate.id)
                                .collect::<Vec<_>>()
                                .join(",");

                            for (winner, winner_trace) in traces.iter().filter(|(_, trace)| {
                                matches!(trace.result, MatchResult::ProfileAWin)
                            }) {
                                let winner_key = format!(
                                    "class={} policy={} variant={} candidate_is_white={}",
                                    portfolio_class,
                                    winner.id,
                                    automove_variant_label(variant),
                                    candidate_is_white,
                                );
                                *portfolio_winner_counts.entry(winner_key).or_default() += 1;

                                if winner.id == baseline.id {
                                    continue;
                                }

                                let Some(divergence) = first_profile_sweep_candidate_divergence(
                                    baseline_trace,
                                    winner_trace,
                                ) else {
                                    let missing_key = format!(
                                        "class={} policy={} duel={} variant={} candidate_is_white={} no_first_divergence",
                                        portfolio_class,
                                        winner.id,
                                        duel.label,
                                        automove_variant_label(variant),
                                        candidate_is_white,
                                    );
                                    *portfolio_winner_context_counts
                                        .entry(missing_key)
                                        .or_default() += 1;
                                    continue;
                                };

                                let context_key = format!(
                                    "class={} policy={} duel={} variant={} candidate_is_white={} color={} baseline_branch={} policy_branch={} turn={} mons_moves={} can_action={} can_mana={} {}",
                                    portfolio_class,
                                    winner.id,
                                    duel.label,
                                    automove_variant_label(variant),
                                    candidate_is_white,
                                    pro_profile_sweep_color_label(divergence.active_color),
                                    divergence.left_branch,
                                    divergence.right_branch,
                                    divergence.turn_number,
                                    divergence.mons_moves_count,
                                    divergence.can_use_action,
                                    divergence.can_move_mana,
                                    divergence.exact_context,
                                );
                                let pair_key = format!(
                                    "class={} policy={} duel={} variant={} candidate_is_white={} baseline_move={} policy_move={}",
                                    portfolio_class,
                                    winner.id,
                                    duel.label,
                                    automove_variant_label(variant),
                                    candidate_is_white,
                                    divergence.left_move_fen,
                                    divergence.right_move_fen,
                                );
                                *portfolio_winner_context_counts
                                    .entry(context_key)
                                    .or_default() += 1;
                                *portfolio_winner_pair_counts.entry(pair_key).or_default() += 1;

                                if include_portfolio_mechanism_class
                                    && portfolio_class == "candidate_only_win"
                                {
                                    let board =
                                        MonsGame::from_fen(divergence.board_fen.as_str(), false)
                                            .expect("policy matrix board fen should be valid");
                                    let mechanism_profile =
                                        pro_policy_mechanism_profile_for_baseline(baseline.id);
                                    let baseline_probe = runtime_decision_probe(
                                        mechanism_profile,
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    let baseline_advisor = pro_v2_root_advisor_decision_snapshot();
                                    for class_key in pro_policy_mechanism_class_keys(
                                        mechanism_profile,
                                        SmartAutomovePreference::Pro,
                                        &board,
                                        &baseline_probe,
                                        baseline_advisor.as_ref(),
                                        divergence.left_move_fen.as_str(),
                                        divergence.right_move_fen.as_str(),
                                    ) {
                                        let key =
                                            format!("class={} {}", portfolio_class, class_key);
                                        let axis_key =
                                            pro_policy_matrix_mechanism_axis_key(&key).to_string();
                                        *portfolio_winner_mechanism_class_counts
                                            .entry(key)
                                            .or_default() += 1;
                                        let route = portfolio_mechanism_axis_routes
                                            .entry(axis_key)
                                            .or_default();
                                        route.candidate_only_games += 1;
                                        route
                                            .candidate_only_states
                                            .insert(portfolio_state_key.clone());
                                        route.candidate_only_panels.insert(panel.label.to_string());
                                        route.candidate_only_duels.insert(duel.label.to_string());
                                        route.candidate_only_policies.insert(winner.id.to_string());
                                        route
                                            .candidate_only_variants
                                            .insert(automove_variant_label(variant).to_string());
                                        route.candidate_only_colors.insert(
                                            pro_profile_sweep_color_label(divergence.active_color)
                                                .to_string(),
                                        );
                                        route.candidate_only_branches.insert(format!(
                                            "{}->{}",
                                            divergence.left_branch, divergence.right_branch
                                        ));
                                        route.candidate_only_pairs.insert(format!(
                                            "{}->{}",
                                            divergence.left_move_fen, divergence.right_move_fen
                                        ));
                                    }
                                }
                            }

                            for trace_index in 1..traces.len() {
                                let (candidate, candidate_trace) = &traces[trace_index];
                                let (_, stats) = &mut stats_by_candidate[trace_index - 1];
                                stats.total_games += 1;
                                let candidate_won =
                                    matches!(candidate_trace.result, MatchResult::ProfileAWin);
                                if !candidate_won {
                                    stats.candidate_nonwins += 1;
                                }
                                if !baseline_won {
                                    stats.baseline_nonwins += 1;
                                }
                                let delta = match_result_points(candidate_trace.result)
                                    - match_result_points(baseline_trace.result);
                                if delta > 0 {
                                    stats.candidate_better += 1;
                                } else if delta < 0 {
                                    stats.baseline_better += 1;
                                } else {
                                    stats.same_outcome += 1;
                                }
                                let outcome = pro_policy_matrix_outcome_label(delta);

                                let first_divergence = first_profile_sweep_candidate_divergence(
                                    baseline_trace,
                                    candidate_trace,
                                );
                                if first_divergence.is_some() {
                                    stats.first_move_diffs += 1;
                                }
                                if include_corpus_records && !global_only {
                                    let include_record_mechanism_axes = include_mechanism_class
                                        || include_portfolio_mechanism_class
                                        || !record_axis_filter_all;
                                    let first_diff_ply = first_divergence
                                        .as_ref()
                                        .map(|divergence| divergence.ply as i32)
                                        .unwrap_or(-1);
                                    let baseline_branch = first_divergence
                                        .as_ref()
                                        .map(|divergence| divergence.left_branch)
                                        .unwrap_or("none");
                                    let candidate_branch = first_divergence
                                        .as_ref()
                                        .map(|divergence| divergence.right_branch)
                                        .unwrap_or("none");
                                    let active_color = first_divergence
                                        .as_ref()
                                        .map(|divergence| {
                                            pro_profile_sweep_color_label(divergence.active_color)
                                        })
                                        .unwrap_or("none");
                                    let turn_number = first_divergence
                                        .as_ref()
                                        .map(|divergence| divergence.turn_number)
                                        .unwrap_or(-1);
                                    let mons_moves_count = first_divergence
                                        .as_ref()
                                        .map(|divergence| divergence.mons_moves_count)
                                        .unwrap_or(-1);
                                    let can_use_action = first_divergence
                                        .as_ref()
                                        .is_some_and(|divergence| divergence.can_use_action);
                                    let can_move_mana = first_divergence
                                        .as_ref()
                                        .is_some_and(|divergence| divergence.can_move_mana);
                                    let exact_context = first_divergence
                                        .as_ref()
                                        .map(|divergence| divergence.exact_context.as_str())
                                        .unwrap_or("none");
                                    let board_fen = first_divergence
                                        .as_ref()
                                        .map(|divergence| divergence.board_fen.as_str())
                                        .unwrap_or("none");
                                    let baseline_move = first_divergence
                                        .as_ref()
                                        .map(|divergence| divergence.left_move_fen.as_str())
                                        .unwrap_or("none");
                                    let candidate_move = first_divergence
                                        .as_ref()
                                        .map(|divergence| divergence.right_move_fen.as_str())
                                        .unwrap_or("none");
                                    let mechanism_axes = first_divergence
                                        .as_ref()
                                        .filter(|_| include_record_mechanism_axes)
                                        .map(|divergence| {
                                            pro_policy_matrix_mechanism_axes_for_moves(
                                                baseline.id,
                                                divergence.board_fen.as_str(),
                                                divergence.left_move_fen.as_str(),
                                                divergence.right_move_fen.as_str(),
                                            )
                                        })
                                        .unwrap_or_else(|| {
                                            if include_record_mechanism_axes {
                                                "none".to_string()
                                            } else {
                                                "disabled".to_string()
                                            }
                                        });
                                    let baseline_better_mechanism_axes = first_divergence
                                        .as_ref()
                                        .filter(|_| include_record_mechanism_axes && delta < 0)
                                        .map(|divergence| {
                                            pro_policy_matrix_mechanism_axes_for_moves(
                                                baseline.id,
                                                divergence.board_fen.as_str(),
                                                divergence.right_move_fen.as_str(),
                                                divergence.left_move_fen.as_str(),
                                            )
                                        })
                                        .unwrap_or_else(|| {
                                            if include_record_mechanism_axes {
                                                "none".to_string()
                                            } else {
                                                "disabled".to_string()
                                            }
                                        });
                                    let timing_continuation_axes =
                                        pro_policy_matrix_timing_continuation_axes(
                                            first_divergence.as_ref(),
                                            baseline_trace,
                                            candidate_trace,
                                        );
                                    if record_axis_filter_allows(
                                        &mechanism_axes,
                                        &baseline_better_mechanism_axes,
                                        &timing_continuation_axes,
                                    ) {
                                        record_filter_stats.corpus_records += 1;
                                        add_record_filter_breakdown(
                                            &mut record_filter_stats,
                                            panel.label,
                                            duel.label,
                                            candidate.id,
                                            outcome,
                                            portfolio_class,
                                            automove_variant_label(variant),
                                            active_color,
                                            format!("{}->{}", baseline_branch, candidate_branch),
                                            format!("{}->{}", baseline_move, candidate_move),
                                        );
                                        println!(
                                            "PRO_POLICY_MATRIX_CORPUS_RECORD {{\"panel\":\"{}\",\"baseline\":\"{}\",\"candidate\":\"{}\",\"candidates\":\"{}\",\"duel\":\"{}\",\"seed_tag\":\"{}\",\"repeat\":{},\"opening_index\":{},\"variant\":\"{}\",\"candidate_is_white\":{},\"portfolio_class\":\"{}\",\"outcome\":\"{}\",\"delta\":{},\"baseline_result\":\"{}\",\"candidate_result\":\"{}\",\"policy_results\":\"{}\",\"winning_policies\":\"{}\",\"first_diff_ply\":{},\"baseline_branch\":\"{}\",\"candidate_branch\":\"{}\",\"active_color\":\"{}\",\"turn\":{},\"mons_moves\":{},\"can_action\":{},\"can_mana\":{},\"exact_context\":\"{}\",\"mechanism_axes\":\"{}\",\"baseline_better_mechanism_axes\":\"{}\",\"timing_continuation_axes\":\"{}\",\"board\":\"{}\",\"opening\":\"{}\",\"baseline_move\":\"{}\",\"candidate_move\":\"{}\",\"baseline_final\":\"{}\",\"candidate_final\":\"{}\"}}",
                                            json_escape(panel.label),
                                            json_escape(baseline.id),
                                            json_escape(candidate.id),
                                            json_escape(&candidate_ids),
                                            json_escape(duel.label),
                                            json_escape(&duel_seed_tag),
                                            repeat_index,
                                            game_index,
                                            automove_variant_label(variant),
                                            candidate_is_white,
                                            portfolio_class,
                                            outcome,
                                            delta,
                                            format_match_result(baseline_trace.result),
                                            format_match_result(candidate_trace.result),
                                            json_escape(&policy_results),
                                            json_escape(&winning_policies),
                                            first_diff_ply,
                                            json_escape(baseline_branch),
                                            json_escape(candidate_branch),
                                            active_color,
                                            turn_number,
                                            mons_moves_count,
                                            can_use_action,
                                            can_move_mana,
                                            json_escape(exact_context),
                                            json_escape(&mechanism_axes),
                                            json_escape(&baseline_better_mechanism_axes),
                                            json_escape(&timing_continuation_axes),
                                            json_escape(board_fen),
                                            json_escape(opening_fen),
                                            json_escape(baseline_move),
                                            json_escape(candidate_move),
                                            json_escape(&baseline_trace.final_fen),
                                            json_escape(&candidate_trace.final_fen),
                                        );
                                    }
                                }
                                if delta == 0 && first_divergence.is_none() {
                                    continue;
                                }
                                stats.recorded += 1;

                                let Some(divergence) = first_divergence else {
                                    stats.missing_first_diff += 1;
                                    continue;
                                };

                                let branch_key = format!(
                                    "candidate={} outcome={} duel={} variant={} baseline_branch={} candidate_branch={}",
                                    candidate.id,
                                    outcome,
                                    duel.label,
                                    automove_variant_label(variant),
                                    divergence.left_branch,
                                    divergence.right_branch,
                                );
                                let context_key = format!(
                                    "candidate={} outcome={} duel={} variant={} color={} baseline_branch={} candidate_branch={} turn={} mons_moves={} can_action={} can_mana={} {}",
                                    candidate.id,
                                    outcome,
                                    duel.label,
                                    automove_variant_label(variant),
                                    pro_profile_sweep_color_label(divergence.active_color),
                                    divergence.left_branch,
                                    divergence.right_branch,
                                    divergence.turn_number,
                                    divergence.mons_moves_count,
                                    divergence.can_use_action,
                                    divergence.can_move_mana,
                                    divergence.exact_context,
                                );
                                let pair_key = format!(
                                    "candidate={} outcome={} duel={} variant={} baseline_move={} candidate_move={}",
                                    candidate.id,
                                    outcome,
                                    duel.label,
                                    automove_variant_label(variant),
                                    divergence.left_move_fen,
                                    divergence.right_move_fen,
                                );
                                *branch_counts.entry(branch_key).or_default() += 1;
                                *context_counts.entry(context_key).or_default() += 1;
                                *pair_counts.entry(pair_key).or_default() += 1;
                                if include_mechanism_class {
                                    let board =
                                        MonsGame::from_fen(divergence.board_fen.as_str(), false)
                                            .expect("policy matrix board fen should be valid");
                                    let mechanism_profile =
                                        pro_policy_mechanism_profile_for_baseline(baseline.id);
                                    let baseline_probe = runtime_decision_probe(
                                        mechanism_profile,
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    let baseline_advisor = pro_v2_root_advisor_decision_snapshot();
                                    for class_key in pro_policy_mechanism_class_keys(
                                        mechanism_profile,
                                        SmartAutomovePreference::Pro,
                                        &board,
                                        &baseline_probe,
                                        baseline_advisor.as_ref(),
                                        divergence.left_move_fen.as_str(),
                                        divergence.right_move_fen.as_str(),
                                    ) {
                                        let key = format!(
                                            "candidate={} outcome={} {}",
                                            candidate.id, outcome, class_key,
                                        );
                                        *mechanism_class_counts.entry(key).or_default() += 1;
                                    }
                                }
                                if include_portfolio_mechanism_class && delta < 0 {
                                    let board =
                                        MonsGame::from_fen(divergence.board_fen.as_str(), false)
                                            .expect("policy matrix board fen should be valid");
                                    let mechanism_profile =
                                        pro_policy_mechanism_profile_for_baseline(baseline.id);
                                    let baseline_probe = runtime_decision_probe(
                                        mechanism_profile,
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    let baseline_advisor = pro_v2_root_advisor_decision_snapshot();
                                    for class_key in pro_policy_mechanism_class_keys(
                                        mechanism_profile,
                                        SmartAutomovePreference::Pro,
                                        &board,
                                        &baseline_probe,
                                        baseline_advisor.as_ref(),
                                        divergence.right_move_fen.as_str(),
                                        divergence.left_move_fen.as_str(),
                                    ) {
                                        let key = format!(
                                            "candidate={} class=baseline_better {}",
                                            candidate.id, class_key,
                                        );
                                        let axis_key =
                                            pro_policy_matrix_mechanism_axis_key(&key).to_string();
                                        *portfolio_baseline_better_mechanism_class_counts
                                            .entry(key)
                                            .or_default() += 1;
                                        let route = portfolio_mechanism_axis_routes
                                            .entry(axis_key)
                                            .or_default();
                                        route.baseline_better_games += 1;
                                        route
                                            .baseline_better_states
                                            .insert(portfolio_state_key.clone());
                                        route
                                            .baseline_better_panels
                                            .insert(panel.label.to_string());
                                        route.baseline_better_duels.insert(duel.label.to_string());
                                        route
                                            .baseline_better_policies
                                            .insert(candidate.id.to_string());
                                        route
                                            .baseline_better_variants
                                            .insert(automove_variant_label(variant).to_string());
                                        route.baseline_better_colors.insert(
                                            pro_profile_sweep_color_label(divergence.active_color)
                                                .to_string(),
                                        );
                                        route.baseline_better_branches.insert(format!(
                                            "{}->{}",
                                            divergence.left_branch, divergence.right_branch
                                        ));
                                        route.baseline_better_pairs.insert(format!(
                                            "{}->{}",
                                            divergence.left_move_fen, divergence.right_move_fen
                                        ));
                                    }
                                }

                                if printed < trace_limit && !global_only {
                                    let include_record_mechanism_axes = include_mechanism_class
                                        || include_portfolio_mechanism_class
                                        || !record_axis_filter_all;
                                    let mechanism_axes = if include_record_mechanism_axes {
                                        pro_policy_matrix_mechanism_axes_for_moves(
                                            baseline.id,
                                            divergence.board_fen.as_str(),
                                            divergence.left_move_fen.as_str(),
                                            divergence.right_move_fen.as_str(),
                                        )
                                    } else {
                                        "disabled".to_string()
                                    };
                                    let baseline_better_mechanism_axes =
                                        if include_record_mechanism_axes && delta < 0 {
                                            pro_policy_matrix_mechanism_axes_for_moves(
                                                baseline.id,
                                                divergence.board_fen.as_str(),
                                                divergence.right_move_fen.as_str(),
                                                divergence.left_move_fen.as_str(),
                                            )
                                        } else if include_record_mechanism_axes {
                                            "none".to_string()
                                        } else {
                                            "disabled".to_string()
                                        };
                                    let baseline_decision = if include_decision_probe {
                                        pro_policy_matrix_move_decision_probe(
                                            &divergence.board_fen,
                                            &divergence.left_move_fen,
                                        )
                                    } else {
                                        "disabled".to_string()
                                    };
                                    let candidate_decision = if include_decision_probe {
                                        pro_policy_matrix_move_decision_probe(
                                            &divergence.board_fen,
                                            &divergence.right_move_fen,
                                        )
                                    } else {
                                        "disabled".to_string()
                                    };
                                    let timing_continuation_axes =
                                        pro_policy_matrix_timing_continuation_axes(
                                            Some(&divergence),
                                            baseline_trace,
                                            candidate_trace,
                                        );
                                    if record_axis_filter_allows(
                                        &mechanism_axes,
                                        &baseline_better_mechanism_axes,
                                        &timing_continuation_axes,
                                    ) {
                                        record_filter_stats.trace_records += 1;
                                        if !include_corpus_records {
                                            add_record_filter_breakdown(
                                                &mut record_filter_stats,
                                                panel.label,
                                                duel.label,
                                                candidate.id,
                                                outcome,
                                                "trace",
                                                automove_variant_label(variant),
                                                pro_profile_sweep_color_label(
                                                    divergence.active_color,
                                                ),
                                                format!(
                                                    "{}->{}",
                                                    divergence.left_branch, divergence.right_branch
                                                ),
                                                format!(
                                                    "{}->{}",
                                                    divergence.left_move_fen,
                                                    divergence.right_move_fen
                                                ),
                                            );
                                        }
                                        println!(
                                            "PRO_POLICY_MATRIX_RECORD {{\"panel\":\"{}\",\"baseline\":\"{}\",\"candidate\":\"{}\",\"duel\":\"{}\",\"repeat\":{},\"opening_index\":{},\"variant\":\"{}\",\"candidate_is_white\":{},\"outcome\":\"{}\",\"delta\":{},\"baseline_result\":\"{}\",\"candidate_result\":\"{}\",\"first_diff_ply\":{},\"baseline_branch\":\"{}\",\"candidate_branch\":\"{}\",\"active_color\":\"{}\",\"turn\":{},\"mons_moves\":{},\"can_action\":{},\"can_mana\":{},\"exact_context\":\"{}\",\"mechanism_axes\":\"{}\",\"baseline_better_mechanism_axes\":\"{}\",\"timing_continuation_axes\":\"{}\",\"board\":\"{}\",\"baseline_move\":\"{}\",\"candidate_move\":\"{}\",\"baseline_decision\":\"{}\",\"candidate_decision\":\"{}\",\"baseline_final\":\"{}\",\"candidate_final\":\"{}\"}}",
                                            json_escape(panel.label),
                                            json_escape(baseline.id),
                                            json_escape(candidate.id),
                                            json_escape(duel.label),
                                            repeat_index,
                                            game_index,
                                            automove_variant_label(variant),
                                            candidate_is_white,
                                            outcome,
                                            delta,
                                            format_match_result(baseline_trace.result),
                                            format_match_result(candidate_trace.result),
                                            divergence.ply,
                                            json_escape(divergence.left_branch),
                                            json_escape(divergence.right_branch),
                                            pro_profile_sweep_color_label(divergence.active_color),
                                            divergence.turn_number,
                                            divergence.mons_moves_count,
                                            divergence.can_use_action,
                                            divergence.can_move_mana,
                                            json_escape(&divergence.exact_context),
                                            json_escape(&mechanism_axes),
                                            json_escape(&baseline_better_mechanism_axes),
                                            json_escape(&timing_continuation_axes),
                                            json_escape(&divergence.board_fen),
                                            json_escape(&divergence.left_move_fen),
                                            json_escape(&divergence.right_move_fen),
                                            json_escape(&baseline_decision),
                                            json_escape(&candidate_decision),
                                            json_escape(&baseline_trace.final_fen),
                                            json_escape(&candidate_trace.final_fen),
                                        );
                                        printed += 1;
                                    }
                                }
                            }
                        }
                    }
                }

                if !global_only {
                    for (candidate, stats) in stats_by_candidate.iter() {
                        let candidate_stoplight =
                            if stats.candidate_better == 0 && stats.baseline_better == 0 {
                                "no_delta"
                            } else if stats.candidate_better > 0 && stats.baseline_better == 0 {
                                "nonregressing_delta"
                            } else if stats.candidate_better > 0 && stats.baseline_better > 0 {
                                "mixed_delta"
                            } else {
                                "regression_only"
                            };
                        println!(
                            "PRO_POLICY_MATRIX_SUMMARY {{\"panel\":\"{}\",\"baseline\":\"{}\",\"candidate\":\"{}\",\"duel\":\"{}\",\"total_games\":{},\"candidate_better\":{},\"baseline_better\":{},\"same_outcome\":{},\"candidate_nonwins\":{},\"baseline_nonwins\":{},\"first_move_diffs\":{},\"recorded\":{},\"missing_first_diff\":{}}}",
                            json_escape(panel.label),
                            json_escape(baseline.id),
                            json_escape(candidate.id),
                            json_escape(duel.label),
                            stats.total_games,
                            stats.candidate_better,
                            stats.baseline_better,
                            stats.same_outcome,
                            stats.candidate_nonwins,
                            stats.baseline_nonwins,
                            stats.first_move_diffs,
                            stats.recorded,
                            stats.missing_first_diff,
                        );
                        println!(
                            "PRO_POLICY_MATRIX_CANDIDATE_STOPLIGHT {{\"panel\":\"{}\",\"baseline\":\"{}\",\"candidate\":\"{}\",\"duel\":\"{}\",\"label\":\"{}\",\"candidate_better\":{},\"baseline_better\":{},\"candidate_nonwins\":{},\"baseline_nonwins\":{},\"recorded\":{},\"missing_first_diff\":{}}}",
                            json_escape(panel.label),
                            json_escape(baseline.id),
                            json_escape(candidate.id),
                            json_escape(duel.label),
                            candidate_stoplight,
                            stats.candidate_better,
                            stats.baseline_better,
                            stats.candidate_nonwins,
                            stats.baseline_nonwins,
                            stats.recorded,
                            stats.missing_first_diff,
                        );
                    }
                }
                let max_portfolio_mechanism_class_games = max_count(&mechanism_class_counts)
                    .max(max_count(&portfolio_winner_mechanism_class_counts));
                let max_portfolio_mechanism_class_key = if max_count(&mechanism_class_counts)
                    > max_count(&portfolio_winner_mechanism_class_counts)
                {
                    max_count_key(&mechanism_class_counts)
                } else {
                    max_count_key(&portfolio_winner_mechanism_class_counts)
                };
                let portfolio_stoplight = portfolio_stoplight_label(
                    &portfolio_stats,
                    &portfolio_winner_counts,
                    &portfolio_winner_context_counts,
                    &portfolio_winner_pair_counts,
                    &portfolio_winner_mechanism_class_counts,
                );
                if !global_only {
                    println!(
                        "PRO_POLICY_MATRIX_PORTFOLIO {{\"panel\":\"{}\",\"baseline\":\"{}\",\"candidates\":\"{}\",\"duel\":\"{}\",\"total_games\":{},\"baseline_wins\":{},\"candidate_any_wins\":{},\"any_policy_wins\":{},\"shared_wins\":{},\"baseline_only_wins\":{},\"candidate_only_wins\":{},\"no_policy_wins\":{},\"state_limit_hit\":{}}}",
                        json_escape(panel.label),
                        json_escape(baseline.id),
                        json_escape(&candidate_ids),
                        json_escape(duel.label),
                        portfolio_stats.total_games,
                        portfolio_stats.baseline_wins,
                        portfolio_stats.candidate_any_wins,
                        portfolio_stats.any_policy_wins,
                        portfolio_stats.shared_wins,
                        portfolio_stats.baseline_only_wins,
                        portfolio_stats.candidate_only_wins,
                        portfolio_stats.no_policy_wins,
                        state_limit_hit,
                    );
                    println!(
                        "PRO_POLICY_MATRIX_PORTFOLIO_STOPLIGHT {{\"panel\":\"{}\",\"baseline\":\"{}\",\"duel\":\"{}\",\"label\":\"{}\",\"baseline_only_wins\":{},\"candidate_only_wins\":{},\"no_policy_wins\":{},\"max_winner_policy_games\":{},\"max_winner_context_games\":{},\"max_winner_pair_games\":{},\"max_mechanism_class_games\":{},\"max_mechanism_class_key\":\"{}\",\"max_baseline_better_mechanism_class_games\":{},\"max_baseline_better_mechanism_class_key\":\"{}\",\"state_limit_hit\":{}}}",
                        json_escape(panel.label),
                        json_escape(baseline.id),
                        json_escape(duel.label),
                        portfolio_stoplight,
                        portfolio_stats.baseline_only_wins,
                        portfolio_stats.candidate_only_wins,
                        portfolio_stats.no_policy_wins,
                        max_count(&portfolio_winner_counts),
                        max_count(&portfolio_winner_context_counts),
                        max_count(&portfolio_winner_pair_counts),
                        max_portfolio_mechanism_class_games,
                        json_escape(max_portfolio_mechanism_class_key),
                        max_count(&portfolio_baseline_better_mechanism_class_counts),
                        json_escape(max_count_key(
                            &portfolio_baseline_better_mechanism_class_counts
                        )),
                        state_limit_hit,
                    );
                    for (key, games) in
                        pro_policy_matrix_sorted_counts(&portfolio_class_counts, aggregate_limit)
                    {
                        println!(
                            "PRO_POLICY_MATRIX_PORTFOLIO_CLASS {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                            json_escape(panel.label),
                            json_escape(duel.label),
                            json_escape(key),
                            games,
                        );
                    }
                    for (key, games) in
                        pro_policy_matrix_sorted_counts(&portfolio_winner_counts, aggregate_limit)
                    {
                        println!(
                            "PRO_POLICY_MATRIX_PORTFOLIO_WINNER {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                            json_escape(panel.label),
                            json_escape(duel.label),
                            json_escape(key),
                            games,
                        );
                    }
                    for (key, games) in pro_policy_matrix_sorted_counts(
                        &portfolio_winner_context_counts,
                        aggregate_limit,
                    ) {
                        println!(
                            "PRO_POLICY_MATRIX_PORTFOLIO_WINNER_CONTEXT {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                            json_escape(panel.label),
                            json_escape(duel.label),
                            json_escape(key),
                            games,
                        );
                    }
                    for (key, games) in pro_policy_matrix_sorted_counts(
                        &portfolio_winner_pair_counts,
                        aggregate_limit,
                    ) {
                        println!(
                            "PRO_POLICY_MATRIX_PORTFOLIO_WINNER_PAIR {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                            json_escape(panel.label),
                            json_escape(duel.label),
                            json_escape(key),
                            games,
                        );
                    }
                    if include_portfolio_mechanism_class {
                        for (key, games) in pro_policy_matrix_sorted_counts(
                            &portfolio_winner_mechanism_class_counts,
                            aggregate_limit,
                        ) {
                            println!(
                                "PRO_POLICY_MATRIX_PORTFOLIO_WINNER_MECHANISM_CLASS {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                                json_escape(panel.label),
                                json_escape(duel.label),
                                json_escape(key),
                                games,
                            );
                        }
                        for (key, games) in pro_policy_matrix_sorted_counts(
                            &portfolio_baseline_better_mechanism_class_counts,
                            aggregate_limit,
                        ) {
                            println!(
                                "PRO_POLICY_MATRIX_PORTFOLIO_BASELINE_BETTER_MECHANISM_CLASS {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                                json_escape(panel.label),
                                json_escape(duel.label),
                                json_escape(key),
                                games,
                            );
                        }
                    }
                    for (key, games) in
                        pro_policy_matrix_sorted_counts(&branch_counts, aggregate_limit)
                    {
                        println!(
                            "PRO_POLICY_MATRIX_BRANCH {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                            json_escape(panel.label),
                            json_escape(duel.label),
                            json_escape(key),
                            games,
                        );
                    }
                    if include_mechanism_class {
                        for (key, games) in pro_policy_matrix_sorted_counts(
                            &mechanism_class_counts,
                            aggregate_limit,
                        ) {
                            println!(
                                "PRO_POLICY_MATRIX_MECHANISM_CLASS {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                                json_escape(panel.label),
                                json_escape(duel.label),
                                json_escape(key),
                                games,
                            );
                        }
                    }
                    for (key, games) in
                        pro_policy_matrix_sorted_counts(&context_counts, aggregate_limit)
                    {
                        println!(
                            "PRO_POLICY_MATRIX_CONTEXT {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                            json_escape(panel.label),
                            json_escape(duel.label),
                            json_escape(key),
                            games,
                        );
                    }
                    for (key, games) in
                        pro_policy_matrix_sorted_counts(&pair_counts, aggregate_limit)
                    {
                        println!(
                            "PRO_POLICY_MATRIX_PAIR {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                            json_escape(panel.label),
                            json_escape(duel.label),
                            json_escape(key),
                            games,
                        );
                    }
                }

                global_portfolio_stats.total_games += portfolio_stats.total_games;
                global_portfolio_stats.baseline_wins += portfolio_stats.baseline_wins;
                global_portfolio_stats.candidate_any_wins += portfolio_stats.candidate_any_wins;
                global_portfolio_stats.any_policy_wins += portfolio_stats.any_policy_wins;
                global_portfolio_stats.shared_wins += portfolio_stats.shared_wins;
                global_portfolio_stats.baseline_only_wins += portfolio_stats.baseline_only_wins;
                global_portfolio_stats.candidate_only_wins += portfolio_stats.candidate_only_wins;
                global_portfolio_stats.no_policy_wins += portfolio_stats.no_policy_wins;
                global_state_limit_hit |= state_limit_hit;
                for (key, games) in &portfolio_class_counts {
                    *global_portfolio_class_counts
                        .entry(key.clone())
                        .or_default() += *games;
                }
                for (key, games) in &portfolio_winner_counts {
                    *global_portfolio_winner_counts
                        .entry(key.clone())
                        .or_default() += *games;
                }
                for (key, games) in &portfolio_winner_context_counts {
                    *global_portfolio_winner_context_counts
                        .entry(key.clone())
                        .or_default() += *games;
                }
                for (key, games) in &portfolio_winner_pair_counts {
                    *global_portfolio_winner_pair_counts
                        .entry(key.clone())
                        .or_default() += *games;
                }
                for (key, games) in &portfolio_winner_mechanism_class_counts {
                    *global_portfolio_winner_mechanism_class_counts
                        .entry(key.clone())
                        .or_default() += *games;
                }
                for (key, games) in &portfolio_baseline_better_mechanism_class_counts {
                    *global_portfolio_baseline_better_mechanism_class_counts
                        .entry(key.clone())
                        .or_default() += *games;
                }
                for (key, route) in portfolio_mechanism_axis_routes {
                    let global_route = global_mechanism_axis_routes.entry(key).or_default();
                    global_route.candidate_only_games += route.candidate_only_games;
                    global_route.baseline_better_games += route.baseline_better_games;
                    global_route
                        .candidate_only_states
                        .extend(route.candidate_only_states);
                    global_route
                        .baseline_better_states
                        .extend(route.baseline_better_states);
                    global_route
                        .candidate_only_panels
                        .extend(route.candidate_only_panels);
                    global_route
                        .candidate_only_duels
                        .extend(route.candidate_only_duels);
                    global_route
                        .candidate_only_policies
                        .extend(route.candidate_only_policies);
                    global_route
                        .candidate_only_variants
                        .extend(route.candidate_only_variants);
                    global_route
                        .candidate_only_colors
                        .extend(route.candidate_only_colors);
                    global_route
                        .candidate_only_branches
                        .extend(route.candidate_only_branches);
                    global_route
                        .candidate_only_pairs
                        .extend(route.candidate_only_pairs);
                    global_route
                        .baseline_better_panels
                        .extend(route.baseline_better_panels);
                    global_route
                        .baseline_better_duels
                        .extend(route.baseline_better_duels);
                    global_route
                        .baseline_better_policies
                        .extend(route.baseline_better_policies);
                    global_route
                        .baseline_better_variants
                        .extend(route.baseline_better_variants);
                    global_route
                        .baseline_better_colors
                        .extend(route.baseline_better_colors);
                    global_route
                        .baseline_better_branches
                        .extend(route.baseline_better_branches);
                    global_route
                        .baseline_better_pairs
                        .extend(route.baseline_better_pairs);
                }
            }
        });
    }

    let global_max_portfolio_mechanism_class_games =
        max_count(&global_portfolio_winner_mechanism_class_counts);
    println!(
        "PRO_POLICY_MATRIX_GLOBAL_SUMMARY {{\"baseline\":\"{}\",\"candidates\":\"{}\",\"total_games\":{},\"baseline_wins\":{},\"candidate_any_wins\":{},\"any_policy_wins\":{},\"shared_wins\":{},\"baseline_only_wins\":{},\"candidate_only_wins\":{},\"no_policy_wins\":{},\"state_limit_hit\":{}}}",
        json_escape(baseline.id),
        json_escape(&candidate_ids),
        global_portfolio_stats.total_games,
        global_portfolio_stats.baseline_wins,
        global_portfolio_stats.candidate_any_wins,
        global_portfolio_stats.any_policy_wins,
        global_portfolio_stats.shared_wins,
        global_portfolio_stats.baseline_only_wins,
        global_portfolio_stats.candidate_only_wins,
        global_portfolio_stats.no_policy_wins,
        global_state_limit_hit,
    );
    println!(
        "PRO_POLICY_MATRIX_GLOBAL_STOPLIGHT {{\"label\":\"{}\",\"baseline_only_wins\":{},\"candidate_only_wins\":{},\"no_policy_wins\":{},\"max_winner_policy_games\":{},\"max_winner_context_games\":{},\"max_winner_pair_games\":{},\"max_mechanism_class_games\":{},\"max_mechanism_class_key\":\"{}\",\"max_baseline_better_mechanism_class_games\":{},\"max_baseline_better_mechanism_class_key\":\"{}\",\"state_limit_hit\":{}}}",
        portfolio_stoplight_label(
            &global_portfolio_stats,
            &global_portfolio_winner_counts,
            &global_portfolio_winner_context_counts,
            &global_portfolio_winner_pair_counts,
            &global_portfolio_winner_mechanism_class_counts,
        ),
        global_portfolio_stats.baseline_only_wins,
        global_portfolio_stats.candidate_only_wins,
        global_portfolio_stats.no_policy_wins,
        max_count(&global_portfolio_winner_counts),
        max_count(&global_portfolio_winner_context_counts),
        max_count(&global_portfolio_winner_pair_counts),
        global_max_portfolio_mechanism_class_games,
        json_escape(max_count_key(
            &global_portfolio_winner_mechanism_class_counts
        )),
        max_count(&global_portfolio_baseline_better_mechanism_class_counts),
        json_escape(max_count_key(
            &global_portfolio_baseline_better_mechanism_class_counts
        )),
        global_state_limit_hit,
    );
    if !global_only && (include_corpus_records || !record_axis_filter_all) {
        let join_record_set = |values: &BTreeSet<String>| -> String {
            values.iter().cloned().collect::<Vec<_>>().join("|")
        };
        let record_filter_breakdown_source = if include_corpus_records {
            "corpus"
        } else {
            "trace"
        };
        let record_filter_breakdown_records = if include_corpus_records {
            record_filter_stats.corpus_records
        } else {
            record_filter_stats.trace_records
        };
        println!(
            "PRO_POLICY_MATRIX_RECORD_FILTER_SUMMARY {{\"record_axis_filter\":\"{}\",\"corpus_records\":{},\"trace_records\":{},\"breakdown_source\":\"{}\",\"breakdown_records\":{},\"panel_count\":{},\"duel_count\":{},\"candidate_count\":{},\"outcome_count\":{},\"portfolio_class_count\":{},\"variant_count\":{},\"color_count\":{},\"branch_count\":{},\"pair_count\":{},\"panels\":\"{}\",\"duels\":\"{}\",\"candidates\":\"{}\",\"outcomes\":\"{}\",\"portfolio_classes\":\"{}\",\"variants\":\"{}\",\"colors\":\"{}\",\"branches\":\"{}\"}}",
            json_escape(&record_axis_filter_label),
            record_filter_stats.corpus_records,
            record_filter_stats.trace_records,
            record_filter_breakdown_source,
            record_filter_breakdown_records,
            record_filter_stats.panels.len(),
            record_filter_stats.duels.len(),
            record_filter_stats.candidates.len(),
            record_filter_stats.outcomes.len(),
            record_filter_stats.portfolio_classes.len(),
            record_filter_stats.variants.len(),
            record_filter_stats.colors.len(),
            record_filter_stats.branches.len(),
            record_filter_stats.pairs.len(),
            json_escape(&join_record_set(&record_filter_stats.panels)),
            json_escape(&join_record_set(&record_filter_stats.duels)),
            json_escape(&join_record_set(&record_filter_stats.candidates)),
            json_escape(&join_record_set(&record_filter_stats.outcomes)),
            json_escape(&join_record_set(&record_filter_stats.portfolio_classes)),
            json_escape(&join_record_set(&record_filter_stats.variants)),
            json_escape(&join_record_set(&record_filter_stats.colors)),
            json_escape(&join_record_set(&record_filter_stats.branches)),
        );
        let print_record_filter_details = |dimension: &str, counts: &BTreeMap<String, usize>| {
            for (rank, (key, records)) in
                pro_policy_matrix_sorted_counts(counts, record_filter_detail_limit)
                    .into_iter()
                    .enumerate()
            {
                println!(
                        "PRO_POLICY_MATRIX_RECORD_FILTER_DETAIL {{\"record_axis_filter\":\"{}\",\"breakdown_source\":\"{}\",\"dimension\":\"{}\",\"rank\":{},\"key\":\"{}\",\"records\":{}}}",
                        json_escape(&record_axis_filter_label),
                        record_filter_breakdown_source,
                        dimension,
                        rank + 1,
                        json_escape(key),
                        records,
                    );
            }
        };
        print_record_filter_details("duel", &record_filter_stats.duel_counts);
        print_record_filter_details("candidate_policy", &record_filter_stats.candidate_counts);
        print_record_filter_details("outcome", &record_filter_stats.outcome_counts);
        print_record_filter_details(
            "portfolio_class",
            &record_filter_stats.portfolio_class_counts,
        );
        print_record_filter_details("variant", &record_filter_stats.variant_counts);
        print_record_filter_details("color", &record_filter_stats.color_counts);
        print_record_filter_details("branch", &record_filter_stats.branch_counts);
        print_record_filter_details("first_move_pair", &record_filter_stats.pair_counts);
    }
    if !global_only {
        for (key, games) in
            pro_policy_matrix_sorted_counts(&global_portfolio_class_counts, aggregate_limit)
        {
            println!(
                "PRO_POLICY_MATRIX_GLOBAL_CLASS {{\"key\":\"{}\",\"games\":{}}}",
                json_escape(key),
                games,
            );
        }
        for (key, games) in
            pro_policy_matrix_sorted_counts(&global_portfolio_winner_counts, aggregate_limit)
        {
            println!(
                "PRO_POLICY_MATRIX_GLOBAL_WINNER {{\"key\":\"{}\",\"games\":{}}}",
                json_escape(key),
                games,
            );
        }
        for (key, games) in pro_policy_matrix_sorted_counts(
            &global_portfolio_winner_context_counts,
            aggregate_limit,
        ) {
            println!(
                "PRO_POLICY_MATRIX_GLOBAL_WINNER_CONTEXT {{\"key\":\"{}\",\"games\":{}}}",
                json_escape(key),
                games,
            );
        }
        for (key, games) in
            pro_policy_matrix_sorted_counts(&global_portfolio_winner_pair_counts, aggregate_limit)
        {
            println!(
                "PRO_POLICY_MATRIX_GLOBAL_WINNER_PAIR {{\"key\":\"{}\",\"games\":{}}}",
                json_escape(key),
                games,
            );
        }
        if include_portfolio_mechanism_class {
            for (key, games) in pro_policy_matrix_sorted_counts(
                &global_portfolio_winner_mechanism_class_counts,
                aggregate_limit,
            ) {
                println!(
                    "PRO_POLICY_MATRIX_GLOBAL_WINNER_MECHANISM_CLASS {{\"key\":\"{}\",\"games\":{}}}",
                    json_escape(key),
                    games,
                );
            }
            for (key, games) in pro_policy_matrix_sorted_counts(
                &global_portfolio_baseline_better_mechanism_class_counts,
                aggregate_limit,
            ) {
                println!(
                    "PRO_POLICY_MATRIX_GLOBAL_BASELINE_BETTER_MECHANISM_CLASS {{\"key\":\"{}\",\"games\":{}}}",
                    json_escape(key),
                    games,
                );
            }
        }
    }
    if include_portfolio_mechanism_class {
        let route_label = |route: &PolicyMatrixMechanismRouteCoverage| -> &'static str {
            if route.candidate_only_states.is_empty() {
                "no_candidate_signal"
            } else if !route.baseline_better_states.is_empty() {
                "baseline_save_risk"
            } else if route.candidate_only_panels.len() > 1 {
                "cross_panel_clean"
            } else if route.candidate_only_duels.len() > 1 {
                "cross_budget_clean"
            } else if route.candidate_only_states.len() > 2 {
                "single_scope_repeat"
            } else {
                "singleton_or_pair"
            }
        };
        let low_fragmentation_route = |route: &PolicyMatrixMechanismRouteCoverage| -> bool {
            route.candidate_only_states.len() >= 2
                && route.baseline_better_states.is_empty()
                && route.candidate_only_policies.len() <= 1
                && route.candidate_only_branches.len() <= 1
                && route.candidate_only_pairs.len() <= 1
        };
        let route_fragmentation_score = |route: &PolicyMatrixMechanismRouteCoverage| -> usize {
            route.candidate_only_policies.len().saturating_sub(1)
                + route.candidate_only_branches.len().saturating_sub(1)
                + route.candidate_only_pairs.len().saturating_sub(1)
        };
        let route_bucket = |route: &PolicyMatrixMechanismRouteCoverage| -> Option<&'static str> {
            if low_fragmentation_route(route) {
                Some("clean_low_fragmentation")
            } else if route.candidate_only_states.len() >= 2
                && route.baseline_better_states.is_empty()
            {
                Some("clean_fragmented")
            } else if !route.candidate_only_states.is_empty()
                && !route.baseline_better_states.is_empty()
            {
                Some("baseline_risk")
            } else if !route.candidate_only_states.is_empty() {
                Some("singleton_candidate")
            } else {
                None
            }
        };
        let join_route_set = |values: &BTreeSet<String>| -> String {
            values.iter().cloned().collect::<Vec<_>>().join("|")
        };
        let clean_low_fragmentation_routes = global_mechanism_axis_routes
            .values()
            .filter(|route| low_fragmentation_route(route))
            .count();
        let clean_fragmented_routes = global_mechanism_axis_routes
            .values()
            .filter(|route| {
                route.candidate_only_states.len() >= 2
                    && route.baseline_better_states.is_empty()
                    && !low_fragmentation_route(route)
            })
            .count();
        let baseline_risk_routes = global_mechanism_axis_routes
            .values()
            .filter(|route| {
                !route.candidate_only_states.is_empty() && !route.baseline_better_states.is_empty()
            })
            .count();
        let candidate_signal_routes = global_mechanism_axis_routes
            .values()
            .filter(|route| !route.candidate_only_states.is_empty())
            .count();
        let mut clean_entries = global_mechanism_axis_routes
            .iter()
            .filter(|(_, route)| {
                route.candidate_only_states.len() >= 2 && route.baseline_better_states.is_empty()
            })
            .collect::<Vec<_>>();
        clean_entries.sort_by(|(left_key, left_route), (right_key, right_route)| {
            right_route
                .candidate_only_states
                .len()
                .cmp(&left_route.candidate_only_states.len())
                .then_with(|| {
                    right_route
                        .candidate_only_games
                        .cmp(&left_route.candidate_only_games)
                })
                .then_with(|| {
                    route_fragmentation_score(left_route)
                        .cmp(&route_fragmentation_score(right_route))
                })
                .then_with(|| left_key.cmp(right_key))
        });
        let best_clean_route = clean_entries.first().copied();
        let mut baseline_risk_entries = global_mechanism_axis_routes
            .iter()
            .filter(|(_, route)| {
                !route.candidate_only_states.is_empty() && !route.baseline_better_states.is_empty()
            })
            .collect::<Vec<_>>();
        baseline_risk_entries.sort_by(|(left_key, left_route), (right_key, right_route)| {
            right_route
                .candidate_only_states
                .len()
                .cmp(&left_route.candidate_only_states.len())
                .then_with(|| {
                    right_route
                        .baseline_better_states
                        .len()
                        .cmp(&left_route.baseline_better_states.len())
                })
                .then_with(|| {
                    right_route
                        .candidate_only_games
                        .cmp(&left_route.candidate_only_games)
                })
                .then_with(|| left_key.cmp(right_key))
        });
        let best_baseline_risk_route = baseline_risk_entries.first().copied();
        let recommendation_label = if clean_low_fragmentation_routes > 0 {
            "narrow_low_fragmentation_route"
        } else if clean_fragmented_routes > 0 {
            "build_outcome_corpus_v2"
        } else if baseline_risk_routes > 0 {
            "baseline_save_risk_only"
        } else if candidate_signal_routes > 0 {
            "singleton_candidate_routes"
        } else {
            "no_candidate_route"
        };
        let best_clean_key = best_clean_route.map(|(key, _)| key.as_str()).unwrap_or("");
        let best_clean_label = best_clean_route
            .map(|(_, route)| route_label(route))
            .unwrap_or("none");
        let best_clean_candidate_only_states = best_clean_route
            .map(|(_, route)| route.candidate_only_states.len())
            .unwrap_or_default();
        let best_clean_candidate_only_games = best_clean_route
            .map(|(_, route)| route.candidate_only_games)
            .unwrap_or_default();
        let best_clean_policy_count = best_clean_route
            .map(|(_, route)| route.candidate_only_policies.len())
            .unwrap_or_default();
        let best_clean_branch_count = best_clean_route
            .map(|(_, route)| route.candidate_only_branches.len())
            .unwrap_or_default();
        let best_clean_pair_count = best_clean_route
            .map(|(_, route)| route.candidate_only_pairs.len())
            .unwrap_or_default();
        let best_baseline_risk_key = best_baseline_risk_route
            .map(|(key, _)| key.as_str())
            .unwrap_or("");
        let best_baseline_risk_candidate_only_states = best_baseline_risk_route
            .map(|(_, route)| route.candidate_only_states.len())
            .unwrap_or_default();
        let best_baseline_risk_baseline_better_states = best_baseline_risk_route
            .map(|(_, route)| route.baseline_better_states.len())
            .unwrap_or_default();
        println!(
            "PRO_POLICY_MATRIX_GLOBAL_ROUTE_RECOMMENDATION {{\"label\":\"{}\",\"candidate_signal_routes\":{},\"clean_low_fragmentation_routes\":{},\"clean_fragmented_routes\":{},\"baseline_risk_routes\":{},\"best_clean_key\":\"{}\",\"best_clean_label\":\"{}\",\"best_clean_candidate_only_states\":{},\"best_clean_candidate_only_games\":{},\"best_clean_policy_count\":{},\"best_clean_branch_count\":{},\"best_clean_pair_count\":{},\"best_baseline_risk_key\":\"{}\",\"best_baseline_risk_candidate_only_states\":{},\"best_baseline_risk_baseline_better_states\":{}}}",
            recommendation_label,
            candidate_signal_routes,
            clean_low_fragmentation_routes,
            clean_fragmented_routes,
            baseline_risk_routes,
            json_escape(best_clean_key),
            json_escape(best_clean_label),
            best_clean_candidate_only_states,
            best_clean_candidate_only_games,
            best_clean_policy_count,
            best_clean_branch_count,
            best_clean_pair_count,
            json_escape(best_baseline_risk_key),
            best_baseline_risk_candidate_only_states,
            best_baseline_risk_baseline_better_states,
        );
        for bucket in [
            "clean_low_fragmentation",
            "clean_fragmented",
            "baseline_risk",
            "singleton_candidate",
        ] {
            let mut bucket_entries = global_mechanism_axis_routes
                .iter()
                .filter(|(_, route)| route_bucket(route) == Some(bucket))
                .collect::<Vec<_>>();
            bucket_entries.sort_by(|(left_key, left_route), (right_key, right_route)| {
                right_route
                    .candidate_only_states
                    .len()
                    .cmp(&left_route.candidate_only_states.len())
                    .then_with(|| {
                        right_route
                            .candidate_only_games
                            .cmp(&left_route.candidate_only_games)
                    })
                    .then_with(|| {
                        right_route
                            .baseline_better_states
                            .len()
                            .cmp(&left_route.baseline_better_states.len())
                    })
                    .then_with(|| {
                        route_fragmentation_score(left_route)
                            .cmp(&route_fragmentation_score(right_route))
                    })
                    .then_with(|| left_key.cmp(right_key))
            });
            for (rank, (key, route)) in bucket_entries
                .into_iter()
                .take(route_bucket_limit)
                .enumerate()
            {
                println!(
                    "PRO_POLICY_MATRIX_GLOBAL_ROUTE_BUCKET {{\"bucket\":\"{}\",\"rank\":{},\"key\":\"{}\",\"label\":\"{}\",\"candidate_only_games\":{},\"baseline_better_games\":{},\"candidate_only_states\":{},\"baseline_better_states\":{},\"candidate_only_policy_count\":{},\"candidate_only_branch_count\":{},\"candidate_only_pair_count\":{},\"candidate_only_panels\":\"{}\",\"candidate_only_duels\":\"{}\",\"candidate_only_policies\":\"{}\",\"candidate_only_branches\":\"{}\",\"baseline_better_policies\":\"{}\",\"baseline_better_branches\":\"{}\"}}",
                    bucket,
                    rank + 1,
                    json_escape(key),
                    route_label(route),
                    route.candidate_only_games,
                    route.baseline_better_games,
                    route.candidate_only_states.len(),
                    route.baseline_better_states.len(),
                    route.candidate_only_policies.len(),
                    route.candidate_only_branches.len(),
                    route.candidate_only_pairs.len(),
                    json_escape(&join_route_set(&route.candidate_only_panels)),
                    json_escape(&join_route_set(&route.candidate_only_duels)),
                    json_escape(&join_route_set(&route.candidate_only_policies)),
                    json_escape(&join_route_set(&route.candidate_only_branches)),
                    json_escape(&join_route_set(&route.baseline_better_policies)),
                    json_escape(&join_route_set(&route.baseline_better_branches)),
                );
            }
        }
        let mut separation_entries = global_mechanism_axis_routes.iter().collect::<Vec<_>>();
        separation_entries.sort_by(|(left_key, left_counts), (right_key, right_counts)| {
            right_counts
                .candidate_only_states
                .len()
                .cmp(&left_counts.candidate_only_states.len())
                .then_with(|| {
                    right_counts
                        .candidate_only_games
                        .cmp(&left_counts.candidate_only_games)
                })
                .then_with(|| {
                    left_counts
                        .baseline_better_states
                        .len()
                        .cmp(&right_counts.baseline_better_states.len())
                })
                .then_with(|| {
                    left_counts
                        .baseline_better_games
                        .cmp(&right_counts.baseline_better_games)
                })
                .then_with(|| left_key.cmp(right_key))
        });
        for (key, route) in separation_entries.into_iter().take(aggregate_limit) {
            println!(
                "PRO_POLICY_MATRIX_GLOBAL_MECHANISM_SEPARATION {{\"key\":\"{}\",\"candidate_only_games\":{},\"baseline_better_games\":{},\"net_candidate_games\":{},\"candidate_only_states\":{},\"baseline_better_states\":{},\"net_candidate_states\":{}}}",
                json_escape(key),
                route.candidate_only_games,
                route.baseline_better_games,
                route.candidate_only_games as isize - route.baseline_better_games as isize,
                route.candidate_only_states.len(),
                route.baseline_better_states.len(),
                route.candidate_only_states.len() as isize
                    - route.baseline_better_states.len() as isize,
            );
            let routing_label = route_label(route);
            let candidate_only_policies = route
                .candidate_only_policies
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("|");
            let candidate_only_variants = route
                .candidate_only_variants
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("|");
            let candidate_only_colors = route
                .candidate_only_colors
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("|");
            let candidate_only_branches = route
                .candidate_only_branches
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("|");
            let baseline_better_policies = route
                .baseline_better_policies
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("|");
            let baseline_better_variants = route
                .baseline_better_variants
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("|");
            let baseline_better_colors = route
                .baseline_better_colors
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("|");
            let baseline_better_branches = route
                .baseline_better_branches
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("|");
            println!(
                "PRO_POLICY_MATRIX_GLOBAL_MECHANISM_ROUTE {{\"key\":\"{}\",\"label\":\"{}\",\"candidate_only_games\":{},\"baseline_better_games\":{},\"candidate_only_states\":{},\"baseline_better_states\":{},\"candidate_only_policy_count\":{},\"candidate_only_variant_count\":{},\"candidate_only_color_count\":{},\"candidate_only_branch_count\":{},\"candidate_only_pair_count\":{},\"baseline_better_policy_count\":{},\"baseline_better_variant_count\":{},\"baseline_better_color_count\":{},\"baseline_better_branch_count\":{},\"baseline_better_pair_count\":{},\"candidate_only_panels\":\"{}\",\"candidate_only_duels\":\"{}\",\"candidate_only_policies\":\"{}\",\"candidate_only_variants\":\"{}\",\"candidate_only_colors\":\"{}\",\"candidate_only_branches\":\"{}\",\"baseline_better_panels\":\"{}\",\"baseline_better_duels\":\"{}\",\"baseline_better_policies\":\"{}\",\"baseline_better_variants\":\"{}\",\"baseline_better_colors\":\"{}\",\"baseline_better_branches\":\"{}\"}}",
                json_escape(key),
                routing_label,
                route.candidate_only_games,
                route.baseline_better_games,
                route.candidate_only_states.len(),
                route.baseline_better_states.len(),
                route.candidate_only_policies.len(),
                route.candidate_only_variants.len(),
                route.candidate_only_colors.len(),
                route.candidate_only_branches.len(),
                route.candidate_only_pairs.len(),
                route.baseline_better_policies.len(),
                route.baseline_better_variants.len(),
                route.baseline_better_colors.len(),
                route.baseline_better_branches.len(),
                route.baseline_better_pairs.len(),
                json_escape(&route.candidate_only_panels.iter().cloned().collect::<Vec<_>>().join("|")),
                json_escape(&route.candidate_only_duels.iter().cloned().collect::<Vec<_>>().join("|")),
                json_escape(&candidate_only_policies),
                json_escape(&candidate_only_variants),
                json_escape(&candidate_only_colors),
                json_escape(&candidate_only_branches),
                json_escape(&route.baseline_better_panels.iter().cloned().collect::<Vec<_>>().join("|")),
                json_escape(&route.baseline_better_duels.iter().cloned().collect::<Vec<_>>().join("|")),
                json_escape(&baseline_better_policies),
                json_escape(&baseline_better_variants),
                json_escape(&baseline_better_colors),
                json_escape(&baseline_better_branches),
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: compare one policy choice across Pro, Normal, and Fast opponents on the same openings"]
fn smart_automove_pro_policy_cross_budget_probe() {
    #[derive(Clone)]
    struct CrossBudgetDuelSpec {
        label: &'static str,
        opponent_mode: SmartAutomovePreference,
    }

    struct CrossBudgetPolicyOutcome {
        candidate: ProProfileSweepCandidate,
        results: Vec<MatchResult>,
        traces: Vec<ProProfileSweepAttributionTrace>,
    }

    #[derive(Default)]
    struct CrossBudgetStats {
        total_states: usize,
        baseline_all_budget_wins: usize,
        candidate_any_all_budget_wins: usize,
        shared_all_budget_win_states: usize,
        baseline_only_all_budget_win_states: usize,
        candidate_only_all_budget_win_states: usize,
        clean_repair_states: usize,
        nonregressing_repair_states: usize,
        budget_conflict_states: usize,
        no_policy_help_states: usize,
        state_limit_hit: bool,
    }

    fn labelled_results(duels: &[CrossBudgetDuelSpec], results: &[MatchResult]) -> String {
        duels
            .iter()
            .zip(results.iter())
            .map(|(duel, result)| format!("{}={}", duel.label, format_match_result(*result)))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn policy_list(policies: &[&str]) -> String {
        if policies.is_empty() {
            "none".to_string()
        } else {
            policies.join(",")
        }
    }

    fn cross_budget_stoplight_label(
        stats: &CrossBudgetStats,
        stable_mechanism_class_counts: &BTreeMap<String, usize>,
        include_mechanism_class: bool,
        mechanism_class_limit_hit: bool,
    ) -> &'static str {
        if mechanism_class_limit_hit {
            "partial_mechanism_corpus"
        } else if stats.baseline_only_all_budget_win_states > 0 {
            "baseline_save_risk"
        } else if stats.no_policy_help_states > 0
            && (stats.clean_repair_states > 0 || stats.nonregressing_repair_states > 0)
        {
            "partial_repair_coverage_gap"
        } else if stats.no_policy_help_states > 0 {
            "coverage_gap"
        } else if (stats.clean_repair_states > 0 || stats.nonregressing_repair_states > 0)
            && stats.budget_conflict_states > 0
        {
            "mixed_cross_budget"
        } else if include_mechanism_class
            && (stats.clean_repair_states > 0 || stats.nonregressing_repair_states > 0)
            && max_count(stable_mechanism_class_counts) > 2
        {
            "repeated_cross_budget_mechanism_class"
        } else if stats.clean_repair_states > 0 {
            "clean_repair"
        } else if stats.nonregressing_repair_states > 0 {
            "nonregressing_repair"
        } else if stats.budget_conflict_states > 0 {
            "budget_conflict"
        } else {
            "no_policy_help"
        }
    }

    fn cross_budget_mechanism_class_filter_allows(filter: &str, candidate_class: &str) -> bool {
        match filter {
            "all" => true,
            "stable" => matches!(
                candidate_class,
                "candidate_clean_all_budget_repair" | "candidate_nonregressing_repair"
            ),
            "conflicts" => candidate_class == "budget_conflict",
            _ => panic!(
                "unknown SMART_PRO_POLICY_CROSS_BUDGET_MECHANISM_CLASS_FILTER '{}'",
                filter
            ),
        }
    }

    let shipping_profile = reliability_shipping_profile_id();
    let shipping_selector = profile_selector_from_name(shipping_profile.as_str())
        .unwrap_or_else(|| panic!("shipping '{}' not found", shipping_profile));
    let candidates = pro_profile_sweep_candidate_list_from_env(
        "SMART_PRO_POLICY_CROSS_BUDGET_CANDIDATES",
        "frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,frontier_pro_v3_white_opening_utility_mana,shipping_pro_search_control,frontier_pro_v2_raw,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard,frontier_pro_v2_no_low_budget_guard",
    );
    assert!(
        candidates.len() >= 2,
        "SMART_PRO_POLICY_CROSS_BUDGET_CANDIDATES must name at least a baseline and one candidate"
    );
    let baseline = candidates[0];
    let panel_filter =
        pro_sweep_filter_tokens("SMART_PRO_POLICY_CROSS_BUDGET_PANEL_FILTER", "sampled");
    let max_plies = env_usize("SMART_PRO_POLICY_CROSS_BUDGET_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let trace_limit = env_usize("SMART_PRO_POLICY_CROSS_BUDGET_TRACE_LIMIT")
        .unwrap_or(24)
        .max(1);
    let aggregate_limit = env_usize("SMART_PRO_POLICY_CROSS_BUDGET_AGGREGATE_LIMIT")
        .unwrap_or(96)
        .max(1);
    let state_limit =
        env_usize("SMART_PRO_POLICY_CROSS_BUDGET_STATE_LIMIT").map(|limit| limit.max(1));
    let include_mechanism_class =
        env_bool("SMART_PRO_POLICY_CROSS_BUDGET_INCLUDE_MECHANISM_CLASS").unwrap_or(false);
    let mechanism_class_filter =
        env_string_value("SMART_PRO_POLICY_CROSS_BUDGET_MECHANISM_CLASS_FILTER")
            .unwrap_or_else(|| "stable".to_string())
            .to_ascii_lowercase();
    let mechanism_class_limit = env_usize("SMART_PRO_POLICY_CROSS_BUDGET_MECHANISM_CLASS_LIMIT")
        .unwrap_or(16)
        .max(1);
    let seed_opponent_mode = env_string_value("SMART_PRO_POLICY_CROSS_BUDGET_SEED_OPPONENT_MODE")
        .map(|mode| mode.to_ascii_lowercase())
        .map(|mode| match mode.as_str() {
            "pro" => SmartAutomovePreference::Pro,
            "normal" => SmartAutomovePreference::Normal,
            "fast" => SmartAutomovePreference::Fast,
            _ => panic!(
                "unknown SMART_PRO_POLICY_CROSS_BUDGET_SEED_OPPONENT_MODE '{}'",
                mode
            ),
        })
        .unwrap_or(SmartAutomovePreference::Pro);
    let duel_specs = [
        CrossBudgetDuelSpec {
            label: "vs_shipping_pro",
            opponent_mode: SmartAutomovePreference::Pro,
        },
        CrossBudgetDuelSpec {
            label: "vs_shipping_normal",
            opponent_mode: SmartAutomovePreference::Normal,
        },
        CrossBudgetDuelSpec {
            label: "vs_shipping_fast",
            opponent_mode: SmartAutomovePreference::Fast,
        },
    ];

    println!(
        "pro policy cross budget: baseline={} candidates={} panels={} max_plies={} seed_opponent_mode={} duels={} include_mechanism_class={} mechanism_class_filter={} mechanism_class_limit={}",
        baseline.id,
        candidates
            .iter()
            .skip(1)
            .map(|candidate| candidate.id)
            .collect::<Vec<_>>()
            .join(","),
        pro_promotion_dashboard_panel_specs()
            .into_iter()
            .filter(|panel| pro_sweep_filter_allows(&panel_filter, panel.label))
            .map(|panel| panel.label)
            .collect::<Vec<_>>()
            .join(","),
        max_plies,
        seed_opponent_mode.as_api_value(),
        duel_specs
            .iter()
            .map(|duel| duel.label)
            .collect::<Vec<_>>()
            .join(","),
        include_mechanism_class,
        mechanism_class_filter,
        mechanism_class_limit,
    );

    for panel in pro_promotion_dashboard_panel_specs()
        .into_iter()
        .filter(|panel| pro_sweep_filter_allows(&panel_filter, panel.label))
    {
        let repeats = env_usize("SMART_PRO_POLICY_CROSS_BUDGET_REPEATS")
            .unwrap_or(1)
            .max(1);
        let games = env_usize("SMART_PRO_POLICY_CROSS_BUDGET_GAMES")
            .unwrap_or(1)
            .max(1);
        let panel_seed_tag = env_string_value("SMART_PRO_POLICY_CROSS_BUDGET_SEED_TAG")
            .unwrap_or_else(|| panel.seed_tag.to_string());

        with_pro_promotion_dashboard_panel(panel, || {
            let mut stats = CrossBudgetStats::default();
            let mut class_counts = BTreeMap::<String, usize>::new();
            let mut all_win_policy_counts = BTreeMap::<String, usize>::new();
            let mut nonregressing_policy_counts = BTreeMap::<String, usize>::new();
            let mut mechanism_class_counts = BTreeMap::<String, usize>::new();
            let mut stable_mechanism_class_counts = BTreeMap::<String, usize>::new();
            let mut mechanism_class_traces = 0usize;
            let mut mechanism_class_limit_hit = false;
            let mut printed = 0usize;

            'cross_budget_samples: for repeat_index in 0..repeats {
                let seed = seed_for_budget_duel_repeat_and_tag(
                    pro_budget(),
                    SearchBudget::from_preference(seed_opponent_mode),
                    repeat_index,
                    panel_seed_tag.as_str(),
                );
                let opening_fens = generate_opening_fens_cached(seed, games);
                for (game_index, opening_fen) in opening_fens.iter().enumerate() {
                    let variant = MonsGame::from_fen(opening_fen.as_str(), false)
                        .expect("valid opening fen")
                        .variant();
                    for candidate_is_white in [true, false] {
                        if state_limit.is_some_and(|limit| stats.total_states >= limit) {
                            stats.state_limit_hit = true;
                            break 'cross_budget_samples;
                        }
                        stats.total_states += 1;
                        let outcomes = candidates
                            .iter()
                            .map(|candidate| {
                                let traces = duel_specs
                                    .iter()
                                    .map(|duel| {
                                        play_profile_sweep_attribution_trace(
                                            *candidate,
                                            shipping_selector,
                                            SearchBudget::from_preference(duel.opponent_mode),
                                            opening_fen.as_str(),
                                            candidate_is_white,
                                            max_plies,
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                let results =
                                    traces.iter().map(|trace| trace.result).collect::<Vec<_>>();
                                CrossBudgetPolicyOutcome {
                                    candidate: *candidate,
                                    results,
                                    traces,
                                }
                            })
                            .collect::<Vec<_>>();
                        let baseline_results = &outcomes[0].results;
                        let baseline_traces = &outcomes[0].traces;
                        let baseline_points = baseline_results
                            .iter()
                            .map(|result| match_result_points(*result))
                            .collect::<Vec<_>>();
                        let baseline_all_wins = baseline_results
                            .iter()
                            .all(|result| matches!(result, MatchResult::ProfileAWin));
                        let all_win_policies = outcomes
                            .iter()
                            .skip(1)
                            .filter(|outcome| {
                                outcome
                                    .results
                                    .iter()
                                    .all(|result| matches!(result, MatchResult::ProfileAWin))
                            })
                            .map(|outcome| outcome.candidate.id)
                            .collect::<Vec<_>>();
                        let nonregressing_policies = outcomes
                            .iter()
                            .skip(1)
                            .filter(|outcome| {
                                let candidate_points = outcome
                                    .results
                                    .iter()
                                    .map(|result| match_result_points(*result))
                                    .collect::<Vec<_>>();
                                candidate_points
                                    .iter()
                                    .zip(baseline_points.iter())
                                    .all(|(candidate, baseline)| candidate >= baseline)
                                    && candidate_points
                                        .iter()
                                        .zip(baseline_points.iter())
                                        .any(|(candidate, baseline)| candidate > baseline)
                            })
                            .map(|outcome| outcome.candidate.id)
                            .collect::<Vec<_>>();
                        let improving_policies = outcomes
                            .iter()
                            .skip(1)
                            .filter(|outcome| {
                                outcome.results.iter().zip(baseline_results.iter()).any(
                                    |(candidate, baseline)| {
                                        match_result_points(*candidate)
                                            > match_result_points(*baseline)
                                    },
                                )
                            })
                            .map(|outcome| outcome.candidate.id)
                            .collect::<Vec<_>>();

                        if baseline_all_wins {
                            stats.baseline_all_budget_wins += 1;
                        }
                        if !all_win_policies.is_empty() {
                            stats.candidate_any_all_budget_wins += 1;
                        }
                        let class = if baseline_all_wins && !all_win_policies.is_empty() {
                            stats.shared_all_budget_win_states += 1;
                            "shared_all_budget_win"
                        } else if baseline_all_wins {
                            stats.baseline_only_all_budget_win_states += 1;
                            "baseline_only_all_budget_win"
                        } else if !all_win_policies.is_empty() {
                            stats.candidate_only_all_budget_win_states += 1;
                            stats.clean_repair_states += 1;
                            "candidate_clean_all_budget_repair"
                        } else if !nonregressing_policies.is_empty() {
                            stats.nonregressing_repair_states += 1;
                            "candidate_nonregressing_repair"
                        } else if !improving_policies.is_empty() {
                            stats.budget_conflict_states += 1;
                            "budget_conflict"
                        } else {
                            stats.no_policy_help_states += 1;
                            "no_policy_help"
                        };

                        if include_mechanism_class {
                            let mechanism_profile =
                                pro_policy_mechanism_profile_for_baseline(baseline.id);
                            for outcome in outcomes.iter().skip(1) {
                                let candidate_points = outcome
                                    .results
                                    .iter()
                                    .map(|result| match_result_points(*result))
                                    .collect::<Vec<_>>();
                                let candidate_all_wins = outcome
                                    .results
                                    .iter()
                                    .all(|result| matches!(result, MatchResult::ProfileAWin));
                                let candidate_nonregressing = candidate_points
                                    .iter()
                                    .zip(baseline_points.iter())
                                    .all(|(candidate, baseline)| candidate >= baseline)
                                    && candidate_points
                                        .iter()
                                        .zip(baseline_points.iter())
                                        .any(|(candidate, baseline)| candidate > baseline);
                                let candidate_improves = candidate_points
                                    .iter()
                                    .zip(baseline_points.iter())
                                    .any(|(candidate, baseline)| candidate > baseline);
                                let candidate_class = if baseline_all_wins && candidate_all_wins {
                                    "shared_all_budget_win"
                                } else if !baseline_all_wins && candidate_all_wins {
                                    "candidate_clean_all_budget_repair"
                                } else if candidate_nonregressing {
                                    "candidate_nonregressing_repair"
                                } else if candidate_improves {
                                    "budget_conflict"
                                } else {
                                    continue;
                                };
                                if !cross_budget_mechanism_class_filter_allows(
                                    mechanism_class_filter.as_str(),
                                    candidate_class,
                                ) {
                                    continue;
                                }

                                for (duel_index, duel) in duel_specs.iter().enumerate() {
                                    let candidate_point = candidate_points[duel_index];
                                    let baseline_point = baseline_points[duel_index];
                                    if candidate_point <= baseline_point {
                                        continue;
                                    }
                                    if mechanism_class_traces >= mechanism_class_limit {
                                        mechanism_class_limit_hit = true;
                                        break;
                                    }
                                    let Some(divergence) = first_profile_sweep_candidate_divergence(
                                        &baseline_traces[duel_index],
                                        &outcome.traces[duel_index],
                                    ) else {
                                        continue;
                                    };
                                    mechanism_class_traces += 1;
                                    let board =
                                        MonsGame::from_fen(divergence.board_fen.as_str(), false)
                                            .expect(
                                                "policy cross-budget board fen should be valid",
                                            );
                                    let baseline_probe = runtime_decision_probe(
                                        mechanism_profile,
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    let baseline_advisor = pro_v2_root_advisor_decision_snapshot();
                                    for class_key in pro_policy_mechanism_class_keys(
                                        mechanism_profile,
                                        SmartAutomovePreference::Pro,
                                        &board,
                                        &baseline_probe,
                                        baseline_advisor.as_ref(),
                                        divergence.left_move_fen.as_str(),
                                        divergence.right_move_fen.as_str(),
                                    ) {
                                        let key = format!(
                                            "class={} policy={} duel={} {}",
                                            candidate_class,
                                            outcome.candidate.id,
                                            duel.label,
                                            class_key
                                        );
                                        if matches!(
                                            candidate_class,
                                            "candidate_clean_all_budget_repair"
                                                | "candidate_nonregressing_repair"
                                        ) {
                                            *stable_mechanism_class_counts
                                                .entry(key.clone())
                                                .or_default() += 1;
                                        }
                                        *mechanism_class_counts.entry(key).or_default() += 1;
                                    }
                                }
                            }
                        }

                        let class_key = format!(
                            "class={} variant={} candidate_is_white={}",
                            class,
                            automove_variant_label(variant),
                            candidate_is_white,
                        );
                        *class_counts.entry(class_key).or_default() += 1;
                        for policy in all_win_policies.iter() {
                            let key = format!(
                                "class={} policy={} variant={} candidate_is_white={}",
                                class,
                                policy,
                                automove_variant_label(variant),
                                candidate_is_white,
                            );
                            *all_win_policy_counts.entry(key).or_default() += 1;
                        }
                        for policy in nonregressing_policies.iter() {
                            let key = format!(
                                "class={} policy={} variant={} candidate_is_white={}",
                                class,
                                policy,
                                automove_variant_label(variant),
                                candidate_is_white,
                            );
                            *nonregressing_policy_counts.entry(key).or_default() += 1;
                        }

                        if printed < trace_limit
                            && (!baseline_all_wins || all_win_policies.is_empty())
                        {
                            let policy_results = outcomes
                                .iter()
                                .skip(1)
                                .map(|outcome| {
                                    format!(
                                        "{}:{}",
                                        outcome.candidate.id,
                                        labelled_results(&duel_specs, &outcome.results)
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("|");
                            println!(
                                "PRO_POLICY_CROSS_BUDGET_RECORD {{\"panel\":\"{}\",\"seed_tag\":\"{}\",\"baseline\":\"{}\",\"repeat\":{},\"opening_index\":{},\"variant\":\"{}\",\"candidate_is_white\":{},\"class\":\"{}\",\"baseline_results\":\"{}\",\"all_win_policies\":\"{}\",\"nonregressing_policies\":\"{}\",\"improving_policies\":\"{}\",\"policy_results\":\"{}\",\"opening\":\"{}\"}}",
                                json_escape(panel.label),
                                json_escape(panel_seed_tag.as_str()),
                                json_escape(baseline.id),
                                repeat_index,
                                game_index,
                                automove_variant_label(variant),
                                candidate_is_white,
                                class,
                                json_escape(&labelled_results(&duel_specs, baseline_results)),
                                json_escape(&policy_list(&all_win_policies)),
                                json_escape(&policy_list(&nonregressing_policies)),
                                json_escape(&policy_list(&improving_policies)),
                                json_escape(&policy_results),
                                json_escape(opening_fen),
                            );
                            printed += 1;
                        }
                    }
                }
            }

            println!(
                "PRO_POLICY_CROSS_BUDGET_SUMMARY {{\"panel\":\"{}\",\"seed_tag\":\"{}\",\"baseline\":\"{}\",\"candidates\":\"{}\",\"seed_opponent_mode\":\"{}\",\"total_states\":{},\"baseline_all_budget_wins\":{},\"candidate_any_all_budget_wins\":{},\"shared_all_budget_win_states\":{},\"baseline_only_all_budget_win_states\":{},\"candidate_only_all_budget_win_states\":{},\"clean_repair_states\":{},\"nonregressing_repair_states\":{},\"budget_conflict_states\":{},\"no_policy_help_states\":{},\"state_limit_hit\":{}}}",
                json_escape(panel.label),
                json_escape(panel_seed_tag.as_str()),
                json_escape(baseline.id),
                json_escape(
                    &candidates
                        .iter()
                        .skip(1)
                        .map(|candidate| candidate.id)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                seed_opponent_mode.as_api_value(),
                stats.total_states,
                stats.baseline_all_budget_wins,
                stats.candidate_any_all_budget_wins,
                stats.shared_all_budget_win_states,
                stats.baseline_only_all_budget_win_states,
                stats.candidate_only_all_budget_win_states,
                stats.clean_repair_states,
                stats.nonregressing_repair_states,
                stats.budget_conflict_states,
                stats.no_policy_help_states,
                stats.state_limit_hit,
            );
            println!(
                "PRO_POLICY_CROSS_BUDGET_STOPLIGHT {{\"panel\":\"{}\",\"seed_tag\":\"{}\",\"baseline\":\"{}\",\"label\":\"{}\",\"shared_all_budget_win_states\":{},\"baseline_only_all_budget_win_states\":{},\"candidate_only_all_budget_win_states\":{},\"clean_repair_states\":{},\"nonregressing_repair_states\":{},\"budget_conflict_states\":{},\"no_policy_help_states\":{},\"max_stable_mechanism_class_states\":{},\"max_mechanism_class_states\":{},\"mechanism_class_traces\":{},\"mechanism_class_limit_hit\":{},\"state_limit_hit\":{}}}",
                json_escape(panel.label),
                json_escape(panel_seed_tag.as_str()),
                json_escape(baseline.id),
                cross_budget_stoplight_label(
                    &stats,
                    &stable_mechanism_class_counts,
                    include_mechanism_class,
                    mechanism_class_limit_hit,
                ),
                stats.shared_all_budget_win_states,
                stats.baseline_only_all_budget_win_states,
                stats.candidate_only_all_budget_win_states,
                stats.clean_repair_states,
                stats.nonregressing_repair_states,
                stats.budget_conflict_states,
                stats.no_policy_help_states,
                max_count(&stable_mechanism_class_counts),
                max_count(&mechanism_class_counts),
                mechanism_class_traces,
                mechanism_class_limit_hit,
                stats.state_limit_hit,
            );
            for (key, states) in pro_policy_matrix_sorted_counts(&class_counts, aggregate_limit) {
                println!(
                    "PRO_POLICY_CROSS_BUDGET_CLASS {{\"panel\":\"{}\",\"key\":\"{}\",\"states\":{}}}",
                    json_escape(panel.label),
                    json_escape(key),
                    states,
                );
            }
            for (key, states) in
                pro_policy_matrix_sorted_counts(&all_win_policy_counts, aggregate_limit)
            {
                println!(
                    "PRO_POLICY_CROSS_BUDGET_ALL_WIN_POLICY {{\"panel\":\"{}\",\"key\":\"{}\",\"states\":{}}}",
                    json_escape(panel.label),
                    json_escape(key),
                    states,
                );
            }
            for (key, states) in
                pro_policy_matrix_sorted_counts(&nonregressing_policy_counts, aggregate_limit)
            {
                println!(
                    "PRO_POLICY_CROSS_BUDGET_NONREGRESSING_POLICY {{\"panel\":\"{}\",\"key\":\"{}\",\"states\":{}}}",
                    json_escape(panel.label),
                    json_escape(key),
                    states,
                );
            }
            if include_mechanism_class {
                for (key, states) in
                    pro_policy_matrix_sorted_counts(&mechanism_class_counts, aggregate_limit)
                {
                    println!(
                        "PRO_POLICY_CROSS_BUDGET_MECHANISM_CLASS {{\"panel\":\"{}\",\"key\":\"{}\",\"states\":{}}}",
                        json_escape(panel.label),
                        json_escape(key),
                        states,
                    );
                }
            }
        });
    }
}

fn max_count(counts: &BTreeMap<String, usize>) -> usize {
    counts.values().copied().max().unwrap_or(0)
}

fn max_count_key(counts: &BTreeMap<String, usize>) -> &str {
    counts
        .iter()
        .max_by(|left, right| left.1.cmp(right.1).then_with(|| right.0.cmp(left.0)))
        .map(|(key, _)| key.as_str())
        .unwrap_or("none")
}

fn pro_policy_matrix_mechanism_axis_key(key: &str) -> &str {
    key.find("axis=").map(|index| &key[index..]).unwrap_or(key)
}

fn pro_policy_matrix_mechanism_axes_for_moves(
    baseline_id: &str,
    board_fen: &str,
    baseline_move_fen: &str,
    candidate_move_fen: &str,
) -> String {
    let board = MonsGame::from_fen(board_fen, false)
        .expect("policy matrix mechanism axis board fen should be valid");
    let mechanism_profile = pro_policy_mechanism_profile_for_baseline(baseline_id);
    let baseline_probe =
        runtime_decision_probe(mechanism_profile, SmartAutomovePreference::Pro, &board);
    let baseline_advisor = pro_v2_root_advisor_decision_snapshot();

    pro_policy_mechanism_class_keys(
        mechanism_profile,
        SmartAutomovePreference::Pro,
        &board,
        &baseline_probe,
        baseline_advisor.as_ref(),
        baseline_move_fen,
        candidate_move_fen,
    )
    .into_iter()
    .map(|key| pro_policy_matrix_mechanism_axis_key(&key).to_string())
    .collect::<Vec<_>>()
    .join("|")
}

#[derive(Default)]
struct PolicyWinnerStats {
    total_games: usize,
    baseline_wins: usize,
    policy_wins: usize,
    no_policy_wins: usize,
    candidate_traces: usize,
    missing_first_diff: usize,
    candidate_trace_limit_hit: bool,
    state_limit_hit: bool,
}

struct SweepDecisionRecordStoplightInput<'a> {
    regressions: usize,
    nonwins: usize,
    recorded: usize,
    missing_first_diff: usize,
    branch_counts: &'a BTreeMap<String, usize>,
    context_counts: &'a BTreeMap<String, usize>,
    pair_counts: &'a BTreeMap<String, usize>,
    mechanism_class_counts: &'a BTreeMap<String, usize>,
    include_mechanism_class: bool,
}

fn pro_policy_winner_stoplight_label(
    stats: &PolicyWinnerStats,
    winner_counts: &BTreeMap<String, usize>,
    winner_mechanism_counts: &BTreeMap<String, usize>,
    winner_mechanism_class_counts: &BTreeMap<String, usize>,
    include_mechanism: bool,
) -> &'static str {
    if stats.candidate_trace_limit_hit || stats.state_limit_hit {
        "partial_corpus"
    } else if stats.no_policy_wins > 0 {
        "coverage_gap"
    } else if stats.policy_wins == 0 {
        "baseline_only"
    } else if include_mechanism && max_count(winner_mechanism_counts) > 1 {
        "repeated_mechanism"
    } else if include_mechanism && max_count(winner_mechanism_class_counts) > 2 {
        "repeated_mechanism_class"
    } else if max_count(winner_counts) > 1 {
        "repeated_policy"
    } else {
        "singleton_residue"
    }
}

fn pro_sweep_decision_record_stoplight_label(
    input: &SweepDecisionRecordStoplightInput,
) -> &'static str {
    if input.recorded > 0 && input.recorded == input.missing_first_diff {
        "missing_first_diff"
    } else if input.nonwins == 0 && input.regressions == 0 {
        "clean"
    } else if input.include_mechanism_class && max_count(input.mechanism_class_counts) > 2 {
        "repeated_mechanism_class"
    } else if max_count(input.pair_counts) > 1 {
        "repeated_pair"
    } else if max_count(input.context_counts) > 1 {
        "repeated_context"
    } else if max_count(input.branch_counts) > 1 && input.regressions > 0 {
        "branch_only_with_regressions"
    } else if max_count(input.branch_counts) > 1 {
        "branch_only"
    } else if input.regressions > 0 {
        "singleton_regression_pressure"
    } else {
        "singleton_residue"
    }
}

#[test]
#[ignore = "diagnostic: short-circuit policy winner contexts for selector design"]
fn smart_automove_pro_policy_winner_probe() {
    #[derive(Clone)]
    struct PolicyWinnerDuelSpec {
        label: &'static str,
        opponent_mode: SmartAutomovePreference,
        seed_suffix: &'static str,
    }

    let shipping_profile = reliability_shipping_profile_id();
    let shipping_selector = profile_selector_from_name(shipping_profile.as_str())
        .unwrap_or_else(|| panic!("shipping '{}' not found", shipping_profile));
    let candidates = pro_profile_sweep_candidate_list_from_env(
        "SMART_PRO_POLICY_WINNER_CANDIDATES",
        "frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,shipping_pro_search_control,frontier_pro_v2_raw,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard,frontier_pro_v2_no_low_budget_guard",
    );
    assert!(
        candidates.len() >= 2,
        "SMART_PRO_POLICY_WINNER_CANDIDATES must name at least a baseline and one candidate"
    );
    let baseline = candidates[0];
    let panel_filter = pro_sweep_filter_tokens("SMART_PRO_POLICY_WINNER_PANEL_FILTER", "all");
    let duel_filter = pro_sweep_filter_tokens("SMART_PRO_POLICY_WINNER_DUEL_FILTER", "all");
    let max_plies = env_usize("SMART_PRO_POLICY_WINNER_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let trace_limit = env_usize("SMART_PRO_POLICY_WINNER_TRACE_LIMIT")
        .unwrap_or(24)
        .max(1);
    let aggregate_limit = env_usize("SMART_PRO_POLICY_WINNER_AGGREGATE_LIMIT")
        .unwrap_or(96)
        .max(1);
    let candidate_trace_limit = env_usize("SMART_PRO_POLICY_WINNER_CANDIDATE_TRACE_LIMIT");
    let state_limit = env_usize("SMART_PRO_POLICY_WINNER_STATE_LIMIT").map(|limit| limit.max(1));
    let include_mechanism = env_bool("SMART_PRO_POLICY_WINNER_INCLUDE_MECHANISM").unwrap_or(false);
    let duel_specs = [
        PolicyWinnerDuelSpec {
            label: "vs_shipping_pro",
            opponent_mode: SmartAutomovePreference::Pro,
            seed_suffix: "",
        },
        PolicyWinnerDuelSpec {
            label: "vs_shipping_normal",
            opponent_mode: SmartAutomovePreference::Normal,
            seed_suffix: "_vs_normal",
        },
        PolicyWinnerDuelSpec {
            label: "vs_shipping_fast",
            opponent_mode: SmartAutomovePreference::Fast,
            seed_suffix: "_vs_fast",
        },
    ];

    println!(
        "pro policy winner: baseline={} candidates={} panels={} duels={} max_plies={} include_mechanism={}",
        baseline.id,
        candidates
            .iter()
            .skip(1)
            .map(|candidate| candidate.id)
            .collect::<Vec<_>>()
            .join(","),
        pro_promotion_dashboard_panel_specs()
            .into_iter()
            .filter(|panel| pro_sweep_filter_allows(&panel_filter, panel.label))
            .map(|panel| panel.label)
            .collect::<Vec<_>>()
            .join(","),
        duel_specs
            .iter()
            .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
            .map(|duel| duel.label)
            .collect::<Vec<_>>()
            .join(","),
        max_plies,
        include_mechanism,
    );

    let mut global_stats = PolicyWinnerStats::default();
    let mut global_class_counts = BTreeMap::<String, usize>::new();
    let mut global_winner_counts = BTreeMap::<String, usize>::new();
    let mut global_winner_context_counts = BTreeMap::<String, usize>::new();
    let mut global_winner_pair_counts = BTreeMap::<String, usize>::new();
    let mut global_winner_mechanism_counts = BTreeMap::<String, usize>::new();
    let mut global_winner_mechanism_class_counts = BTreeMap::<String, usize>::new();

    for panel in pro_promotion_dashboard_panel_specs()
        .into_iter()
        .filter(|panel| pro_sweep_filter_allows(&panel_filter, panel.label))
    {
        let repeats = env_usize("SMART_PRO_POLICY_WINNER_REPEATS")
            .unwrap_or(panel.default_repeats)
            .max(1);
        let games = env_usize("SMART_PRO_POLICY_WINNER_GAMES")
            .unwrap_or(panel.default_games)
            .max(1);
        let panel_seed_tag = env_string_value("SMART_PRO_POLICY_WINNER_SEED_TAG")
            .unwrap_or_else(|| panel.seed_tag.to_string());

        with_pro_promotion_dashboard_panel(panel, || {
            for duel in duel_specs
                .iter()
                .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
            {
                let opponent_budget = SearchBudget::from_preference(duel.opponent_mode);
                let duel_seed_tag = format!("{}{}", panel_seed_tag, duel.seed_suffix);
                let mut stats = PolicyWinnerStats::default();
                let mut class_counts = BTreeMap::<String, usize>::new();
                let mut winner_counts = BTreeMap::<String, usize>::new();
                let mut winner_context_counts = BTreeMap::<String, usize>::new();
                let mut winner_pair_counts = BTreeMap::<String, usize>::new();
                let mut winner_mechanism_counts = BTreeMap::<String, usize>::new();
                let mut winner_mechanism_class_counts = BTreeMap::<String, usize>::new();
                let mut printed = 0usize;

                'duel_samples: for repeat_index in 0..repeats {
                    let seed = seed_for_budget_duel_repeat_and_tag(
                        pro_budget(),
                        opponent_budget,
                        repeat_index,
                        duel_seed_tag.as_str(),
                    );
                    let opening_fens = generate_opening_fens_cached(seed, games);
                    for (game_index, opening_fen) in opening_fens.iter().enumerate() {
                        let variant = MonsGame::from_fen(opening_fen.as_str(), false)
                            .expect("valid opening fen")
                            .variant();
                        for candidate_is_white in [true, false] {
                            if state_limit.is_some_and(|limit| stats.total_games >= limit) {
                                stats.state_limit_hit = true;
                                break 'duel_samples;
                            }
                            if candidate_trace_limit
                                .is_some_and(|limit| stats.candidate_traces >= limit)
                            {
                                stats.candidate_trace_limit_hit = true;
                                break 'duel_samples;
                            }
                            stats.total_games += 1;
                            let baseline_trace = play_profile_sweep_attribution_trace(
                                baseline,
                                shipping_selector,
                                opponent_budget,
                                opening_fen.as_str(),
                                candidate_is_white,
                                max_plies,
                            );
                            if matches!(baseline_trace.result, MatchResult::ProfileAWin) {
                                stats.baseline_wins += 1;
                                let key = format!(
                                    "class=baseline_win policy={} variant={} candidate_is_white={}",
                                    baseline.id,
                                    automove_variant_label(variant),
                                    candidate_is_white,
                                );
                                *class_counts.entry(key).or_default() += 1;
                                continue;
                            }

                            let mut winning_trace = None;
                            for candidate in candidates.iter().skip(1) {
                                if candidate_trace_limit
                                    .is_some_and(|limit| stats.candidate_traces >= limit)
                                {
                                    stats.candidate_trace_limit_hit = true;
                                    break;
                                }
                                stats.candidate_traces += 1;
                                let candidate_trace = play_profile_sweep_attribution_trace(
                                    *candidate,
                                    shipping_selector,
                                    opponent_budget,
                                    opening_fen.as_str(),
                                    candidate_is_white,
                                    max_plies,
                                );
                                if matches!(candidate_trace.result, MatchResult::ProfileAWin) {
                                    winning_trace = Some((*candidate, candidate_trace));
                                    break;
                                }
                            }
                            if stats.candidate_trace_limit_hit && winning_trace.is_none() {
                                break 'duel_samples;
                            }

                            let Some((winner, winner_trace)) = winning_trace else {
                                stats.no_policy_wins += 1;
                                let key = format!(
                                    "class=no_policy_win variant={} candidate_is_white={}",
                                    automove_variant_label(variant),
                                    candidate_is_white,
                                );
                                *class_counts.entry(key).or_default() += 1;
                                if printed < trace_limit {
                                    println!(
                                        "PRO_POLICY_WINNER_NO_POLICY_RECORD {{\"panel\":\"{}\",\"baseline\":\"{}\",\"duel\":\"{}\",\"repeat\":{},\"opening_index\":{},\"variant\":\"{}\",\"candidate_is_white\":{},\"baseline_result\":\"{}\",\"opening\":\"{}\",\"baseline_final\":\"{}\"}}",
                                        json_escape(panel.label),
                                        json_escape(baseline.id),
                                        json_escape(duel.label),
                                        repeat_index,
                                        game_index,
                                        automove_variant_label(variant),
                                        candidate_is_white,
                                        format_match_result(baseline_trace.result),
                                        json_escape(opening_fen),
                                        json_escape(&baseline_trace.final_fen),
                                    );
                                    printed += 1;
                                }
                                continue;
                            };

                            stats.policy_wins += 1;
                            let winner_key = format!(
                                "class=policy_win policy={} variant={} candidate_is_white={}",
                                winner.id,
                                automove_variant_label(variant),
                                candidate_is_white,
                            );
                            *winner_counts.entry(winner_key).or_default() += 1;

                            let Some(divergence) = first_profile_sweep_candidate_divergence(
                                &baseline_trace,
                                &winner_trace,
                            ) else {
                                stats.missing_first_diff += 1;
                                let key = format!(
                                    "class=policy_win policy={} duel={} variant={} candidate_is_white={} no_first_divergence",
                                    winner.id,
                                    duel.label,
                                    automove_variant_label(variant),
                                    candidate_is_white,
                                );
                                *winner_context_counts.entry(key).or_default() += 1;
                                continue;
                            };

                            let context_key = format!(
                                "class=policy_win policy={} duel={} variant={} candidate_is_white={} color={} baseline_branch={} policy_branch={} turn={} mons_moves={} can_action={} can_mana={} {}",
                                winner.id,
                                duel.label,
                                automove_variant_label(variant),
                                candidate_is_white,
                                pro_profile_sweep_color_label(divergence.active_color),
                                divergence.left_branch,
                                divergence.right_branch,
                                divergence.turn_number,
                                divergence.mons_moves_count,
                                divergence.can_use_action,
                                divergence.can_move_mana,
                                divergence.exact_context,
                            );
                            let pair_key = format!(
                                "class=policy_win policy={} duel={} variant={} candidate_is_white={} baseline_move={} policy_move={}",
                                winner.id,
                                duel.label,
                                automove_variant_label(variant),
                                candidate_is_white,
                                divergence.left_move_fen,
                                divergence.right_move_fen,
                            );
                            *winner_context_counts.entry(context_key).or_default() += 1;
                            *winner_pair_counts.entry(pair_key).or_default() += 1;

                            if include_mechanism {
                                let board =
                                    MonsGame::from_fen(divergence.board_fen.as_str(), false)
                                        .expect("policy winner board fen should be valid");
                                let mechanism_profile =
                                    pro_policy_mechanism_profile_for_baseline(baseline.id);
                                let baseline_probe = runtime_decision_probe(
                                    mechanism_profile,
                                    SmartAutomovePreference::Pro,
                                    &board,
                                );
                                let baseline_advisor = pro_v2_root_advisor_decision_snapshot();
                                let (mechanism_config, mechanism_scored_roots, _, _) =
                                    profile_runtime_scored_roots_with_forced_engine_inputs(
                                        mechanism_profile,
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                let baseline_status =
                                    decision_record_baseline_status_from_scored_roots(
                                        mechanism_scored_roots.as_slice(),
                                        &baseline_probe,
                                        baseline_advisor.as_ref(),
                                        divergence.left_move_fen.as_str(),
                                    );
                                let winner_status =
                                    decision_record_baseline_status_from_scored_roots(
                                        mechanism_scored_roots.as_slice(),
                                        &baseline_probe,
                                        baseline_advisor.as_ref(),
                                        divergence.right_move_fen.as_str(),
                                    );
                                let approved_status =
                                    decision_record_approved_status(baseline_advisor.as_ref());
                                let baseline_root =
                                    pro_policy_target_root_utility_status_from_scored_roots(
                                        &board,
                                        mechanism_config,
                                        mechanism_scored_roots.as_slice(),
                                        divergence.left_move_fen.as_str(),
                                    );
                                let winner_root =
                                    pro_policy_target_root_utility_status_from_scored_roots(
                                        &board,
                                        mechanism_config,
                                        mechanism_scored_roots.as_slice(),
                                        divergence.right_move_fen.as_str(),
                                    );
                                let mechanism_key = format!(
                                    "class=policy_win mechanism_profile={} policy={} variant={} candidate_is_white={} color={} baseline_branch={} policy_branch={} baseline_stage={} turn={} mons_moves={} can_action={} can_mana={} selected_rank={:?} pre_family={:?} head_family={:?} head_accepted={} head_primary={:?} baseline_status={} winner_status={} approved={} baseline_root=[{}] winner_root=[{}] {}",
                                    mechanism_profile,
                                    winner.id,
                                    automove_variant_label(variant),
                                    candidate_is_white,
                                    pro_profile_sweep_color_label(board.active_color),
                                    divergence.left_branch,
                                    divergence.right_branch,
                                    baseline_probe.selector_last_stage,
                                    board.turn_number,
                                    board.mons_moves_count,
                                    board.player_can_use_action(),
                                    board.player_can_move_mana(),
                                    baseline_probe.selected_rank,
                                    baseline_probe.pre_accept_family,
                                    baseline_probe.head_family,
                                    baseline_probe.head_accepted,
                                    baseline_probe.head_plan_primary_axes_vs_pre_accept,
                                    baseline_status,
                                    winner_status,
                                    approved_status,
                                    baseline_root,
                                    winner_root,
                                    baseline_probe.exact_context,
                                );
                                *winner_mechanism_counts.entry(mechanism_key).or_default() += 1;
                                for class_key in pro_policy_mechanism_class_keys_from_scored_roots(
                                    &board,
                                    mechanism_config,
                                    mechanism_scored_roots.as_slice(),
                                    &baseline_probe,
                                    baseline_advisor.as_ref(),
                                    divergence.left_move_fen.as_str(),
                                    divergence.right_move_fen.as_str(),
                                ) {
                                    *winner_mechanism_class_counts.entry(class_key).or_default() +=
                                        1;
                                }
                            }

                            if printed < trace_limit {
                                println!(
                                    "PRO_POLICY_WINNER_RECORD {{\"panel\":\"{}\",\"baseline\":\"{}\",\"winner\":\"{}\",\"duel\":\"{}\",\"repeat\":{},\"opening_index\":{},\"variant\":\"{}\",\"candidate_is_white\":{},\"baseline_result\":\"{}\",\"winner_result\":\"{}\",\"first_diff_ply\":{},\"baseline_branch\":\"{}\",\"winner_branch\":\"{}\",\"active_color\":\"{}\",\"turn\":{},\"mons_moves\":{},\"can_action\":{},\"can_mana\":{},\"exact_context\":\"{}\",\"board\":\"{}\",\"baseline_move\":\"{}\",\"winner_move\":\"{}\",\"baseline_final\":\"{}\",\"winner_final\":\"{}\"}}",
                                    json_escape(panel.label),
                                    json_escape(baseline.id),
                                    json_escape(winner.id),
                                    json_escape(duel.label),
                                    repeat_index,
                                    game_index,
                                    automove_variant_label(variant),
                                    candidate_is_white,
                                    format_match_result(baseline_trace.result),
                                    format_match_result(winner_trace.result),
                                    divergence.ply,
                                    json_escape(divergence.left_branch),
                                    json_escape(divergence.right_branch),
                                    pro_profile_sweep_color_label(divergence.active_color),
                                    divergence.turn_number,
                                    divergence.mons_moves_count,
                                    divergence.can_use_action,
                                    divergence.can_move_mana,
                                    json_escape(&divergence.exact_context),
                                    json_escape(&divergence.board_fen),
                                    json_escape(&divergence.left_move_fen),
                                    json_escape(&divergence.right_move_fen),
                                    json_escape(&baseline_trace.final_fen),
                                    json_escape(&winner_trace.final_fen),
                                );
                                printed += 1;
                            }
                        }
                    }
                }

                println!(
                    "PRO_POLICY_WINNER_SUMMARY {{\"panel\":\"{}\",\"duel\":\"{}\",\"seed_tag\":\"{}\",\"baseline\":\"{}\",\"candidates\":\"{}\",\"total_games\":{},\"baseline_wins\":{},\"policy_wins\":{},\"no_policy_wins\":{},\"candidate_traces\":{},\"candidate_trace_limit_hit\":{},\"state_limit_hit\":{},\"missing_first_diff\":{}}}",
                    json_escape(panel.label),
                    json_escape(duel.label),
                    json_escape(duel_seed_tag.as_str()),
                    json_escape(baseline.id),
                    json_escape(
                        &candidates
                            .iter()
                            .skip(1)
                            .map(|candidate| candidate.id)
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    stats.total_games,
                    stats.baseline_wins,
                    stats.policy_wins,
                    stats.no_policy_wins,
                    stats.candidate_traces,
                    stats.candidate_trace_limit_hit,
                    stats.state_limit_hit,
                    stats.missing_first_diff,
                );
                println!(
                    "PRO_POLICY_WINNER_STOPLIGHT {{\"panel\":\"{}\",\"duel\":\"{}\",\"seed_tag\":\"{}\",\"label\":\"{}\",\"policy_wins\":{},\"no_policy_wins\":{},\"max_policy_games\":{},\"max_mechanism_games\":{},\"max_mechanism_class_games\":{},\"candidate_trace_limit_hit\":{},\"state_limit_hit\":{}}}",
                    json_escape(panel.label),
                    json_escape(duel.label),
                    json_escape(duel_seed_tag.as_str()),
                    pro_policy_winner_stoplight_label(
                        &stats,
                        &winner_counts,
                        &winner_mechanism_counts,
                        &winner_mechanism_class_counts,
                        include_mechanism,
                    ),
                    stats.policy_wins,
                    stats.no_policy_wins,
                    max_count(&winner_counts),
                    max_count(&winner_mechanism_counts),
                    max_count(&winner_mechanism_class_counts),
                    stats.candidate_trace_limit_hit,
                    stats.state_limit_hit,
                );
                for (key, games) in pro_policy_matrix_sorted_counts(&class_counts, aggregate_limit)
                {
                    println!(
                        "PRO_POLICY_WINNER_CLASS {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                        json_escape(panel.label),
                        json_escape(duel.label),
                        json_escape(key),
                        games,
                    );
                }
                for (key, games) in pro_policy_matrix_sorted_counts(&winner_counts, aggregate_limit)
                {
                    println!(
                        "PRO_POLICY_WINNER_POLICY {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                        json_escape(panel.label),
                        json_escape(duel.label),
                        json_escape(key),
                        games,
                    );
                }
                for (key, games) in
                    pro_policy_matrix_sorted_counts(&winner_context_counts, aggregate_limit)
                {
                    println!(
                        "PRO_POLICY_WINNER_CONTEXT {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                        json_escape(panel.label),
                        json_escape(duel.label),
                        json_escape(key),
                        games,
                    );
                }
                for (key, games) in
                    pro_policy_matrix_sorted_counts(&winner_pair_counts, aggregate_limit)
                {
                    println!(
                        "PRO_POLICY_WINNER_PAIR {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                        json_escape(panel.label),
                        json_escape(duel.label),
                        json_escape(key),
                        games,
                    );
                }
                if include_mechanism {
                    for (key, games) in pro_policy_matrix_sorted_counts(
                        &winner_mechanism_class_counts,
                        aggregate_limit,
                    ) {
                        println!(
                            "PRO_POLICY_WINNER_MECHANISM_CLASS {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                            json_escape(panel.label),
                            json_escape(duel.label),
                            json_escape(key),
                            games,
                        );
                    }
                    for (key, games) in
                        pro_policy_matrix_sorted_counts(&winner_mechanism_counts, aggregate_limit)
                    {
                        println!(
                            "PRO_POLICY_WINNER_MECHANISM {{\"panel\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                            json_escape(panel.label),
                            json_escape(duel.label),
                            json_escape(key),
                            games,
                        );
                    }
                }

                global_stats.total_games += stats.total_games;
                global_stats.baseline_wins += stats.baseline_wins;
                global_stats.policy_wins += stats.policy_wins;
                global_stats.no_policy_wins += stats.no_policy_wins;
                global_stats.candidate_traces += stats.candidate_traces;
                global_stats.missing_first_diff += stats.missing_first_diff;
                global_stats.candidate_trace_limit_hit |= stats.candidate_trace_limit_hit;
                global_stats.state_limit_hit |= stats.state_limit_hit;
                for (key, games) in &class_counts {
                    *global_class_counts.entry(key.clone()).or_default() += *games;
                }
                for (key, games) in &winner_counts {
                    *global_winner_counts.entry(key.clone()).or_default() += *games;
                }
                for (key, games) in &winner_context_counts {
                    *global_winner_context_counts.entry(key.clone()).or_default() += *games;
                }
                for (key, games) in &winner_pair_counts {
                    *global_winner_pair_counts.entry(key.clone()).or_default() += *games;
                }
                for (key, games) in &winner_mechanism_counts {
                    *global_winner_mechanism_counts
                        .entry(key.clone())
                        .or_default() += *games;
                }
                for (key, games) in &winner_mechanism_class_counts {
                    *global_winner_mechanism_class_counts
                        .entry(key.clone())
                        .or_default() += *games;
                }
            }
        });
    }

    println!(
        "PRO_POLICY_WINNER_GLOBAL_SUMMARY {{\"baseline\":\"{}\",\"candidates\":\"{}\",\"total_games\":{},\"baseline_wins\":{},\"policy_wins\":{},\"no_policy_wins\":{},\"candidate_traces\":{},\"candidate_trace_limit_hit\":{},\"state_limit_hit\":{},\"missing_first_diff\":{}}}",
        json_escape(baseline.id),
        json_escape(
            &candidates
                .iter()
                .skip(1)
                .map(|candidate| candidate.id)
                .collect::<Vec<_>>()
                .join(",")
        ),
        global_stats.total_games,
        global_stats.baseline_wins,
        global_stats.policy_wins,
        global_stats.no_policy_wins,
        global_stats.candidate_traces,
        global_stats.candidate_trace_limit_hit,
        global_stats.state_limit_hit,
        global_stats.missing_first_diff,
    );
    println!(
        "PRO_POLICY_WINNER_GLOBAL_STOPLIGHT {{\"label\":\"{}\",\"policy_wins\":{},\"no_policy_wins\":{},\"max_policy_games\":{},\"max_context_games\":{},\"max_pair_games\":{},\"max_mechanism_games\":{},\"max_mechanism_class_games\":{},\"candidate_trace_limit_hit\":{},\"state_limit_hit\":{}}}",
        pro_policy_winner_stoplight_label(
            &global_stats,
            &global_winner_counts,
            &global_winner_mechanism_counts,
            &global_winner_mechanism_class_counts,
            include_mechanism,
        ),
        global_stats.policy_wins,
        global_stats.no_policy_wins,
        max_count(&global_winner_counts),
        max_count(&global_winner_context_counts),
        max_count(&global_winner_pair_counts),
        max_count(&global_winner_mechanism_counts),
        max_count(&global_winner_mechanism_class_counts),
        global_stats.candidate_trace_limit_hit,
        global_stats.state_limit_hit,
    );
    for (key, games) in pro_policy_matrix_sorted_counts(&global_class_counts, aggregate_limit) {
        println!(
            "PRO_POLICY_WINNER_GLOBAL_CLASS {{\"key\":\"{}\",\"games\":{}}}",
            json_escape(key),
            games,
        );
    }
    for (key, games) in pro_policy_matrix_sorted_counts(&global_winner_counts, aggregate_limit) {
        println!(
            "PRO_POLICY_WINNER_GLOBAL_POLICY {{\"key\":\"{}\",\"games\":{}}}",
            json_escape(key),
            games,
        );
    }
    for (key, games) in
        pro_policy_matrix_sorted_counts(&global_winner_context_counts, aggregate_limit)
    {
        println!(
            "PRO_POLICY_WINNER_GLOBAL_CONTEXT {{\"key\":\"{}\",\"games\":{}}}",
            json_escape(key),
            games,
        );
    }
    for (key, games) in pro_policy_matrix_sorted_counts(&global_winner_pair_counts, aggregate_limit)
    {
        println!(
            "PRO_POLICY_WINNER_GLOBAL_PAIR {{\"key\":\"{}\",\"games\":{}}}",
            json_escape(key),
            games,
        );
    }
    if include_mechanism {
        for (key, games) in
            pro_policy_matrix_sorted_counts(&global_winner_mechanism_class_counts, aggregate_limit)
        {
            println!(
                "PRO_POLICY_WINNER_GLOBAL_MECHANISM_CLASS {{\"key\":\"{}\",\"games\":{}}}",
                json_escape(key),
                games,
            );
        }
        for (key, games) in
            pro_policy_matrix_sorted_counts(&global_winner_mechanism_counts, aggregate_limit)
        {
            println!(
                "PRO_POLICY_WINNER_GLOBAL_MECHANISM {{\"key\":\"{}\",\"games\":{}}}",
                json_escape(key),
                games,
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: aggregate arbitrary Pro sweep-candidate nonwins and deltas against shipping control"]
fn smart_automove_pro_sweep_decision_record_probe() {
    #[derive(Clone)]
    struct SweepDecisionRecordDuelSpec {
        label: &'static str,
        opponent_mode: SmartAutomovePreference,
        seed_tag: String,
    }

    let shipping_profile = reliability_shipping_profile_id();
    let shipping_selector = profile_selector_from_name(shipping_profile.as_str())
        .unwrap_or_else(|| panic!("shipping '{}' not found", shipping_profile));
    let candidate_id = env_string_value("SMART_PRO_SWEEP_DECISION_RECORD_CANDIDATE")
        .or_else(|| env_string_value("SMART_PRO_SWEEP_CANDIDATE"))
        .unwrap_or_else(|| "frontier_pro_v2_guarded".to_string());
    let candidate = pro_profile_sweep_candidate_by_id(candidate_id.as_str());
    let shipping_control = pro_profile_sweep_candidate_by_id("shipping_pro_search_control");
    let repeats = env_usize("SMART_PRO_SWEEP_DECISION_RECORD_REPEATS")
        .unwrap_or(3)
        .max(1);
    let games = env_usize("SMART_PRO_SWEEP_DECISION_RECORD_GAMES")
        .unwrap_or(2)
        .max(1);
    let max_plies = env_usize("SMART_PRO_SWEEP_DECISION_RECORD_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let trace_limit = env_usize("SMART_PRO_SWEEP_DECISION_RECORD_TRACE_LIMIT")
        .unwrap_or(24)
        .max(1);
    let aggregate_limit = env_usize("SMART_PRO_SWEEP_DECISION_RECORD_AGGREGATE_LIMIT")
        .unwrap_or(64)
        .max(1);
    let seed_tag = env_string_value("SMART_PRO_SWEEP_DECISION_RECORD_SEED_TAG")
        .unwrap_or_else(|| "pro_profile_sweep_v1".to_string());
    let duel_filter = pro_sweep_filter_tokens("SMART_PRO_SWEEP_DECISION_RECORD_DUEL_FILTER", "all");
    let outcome_filter = pro_sweep_filter_tokens("SMART_PRO_SWEEP_DECISION_RECORD_OUTCOME", "all");
    let scope = env_string_value("SMART_PRO_SWEEP_DECISION_RECORD_SCOPE")
        .unwrap_or_else(|| "nonwins".to_string());
    let include_mechanism_class =
        env_bool("SMART_PRO_SWEEP_DECISION_RECORD_INCLUDE_MECHANISM_CLASS").unwrap_or(false);
    let outcome_scope = format!("{} scope={}", outcome_filter.join(","), scope);
    let duel_specs = [
        SweepDecisionRecordDuelSpec {
            label: "vs_shipping_pro",
            opponent_mode: SmartAutomovePreference::Pro,
            seed_tag: seed_tag.clone(),
        },
        SweepDecisionRecordDuelSpec {
            label: "vs_shipping_normal",
            opponent_mode: SmartAutomovePreference::Normal,
            seed_tag: format!("{}_vs_normal", seed_tag),
        },
        SweepDecisionRecordDuelSpec {
            label: "vs_shipping_fast",
            opponent_mode: SmartAutomovePreference::Fast,
            seed_tag: format!("{}_vs_fast", seed_tag),
        },
    ];

    println!(
        "pro sweep decision record: candidate={} shipping={} repeats={} games={} max_plies={} duels={} outcome_filter={} variants={} include_mechanism_class={}",
        candidate.id,
        shipping_profile,
        repeats,
        games,
        max_plies,
        duel_specs
            .iter()
            .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
            .map(|duel| duel.label)
            .collect::<Vec<_>>()
            .join(","),
        outcome_scope,
        env::var("SMART_AUTOMOVE_VARIANTS").unwrap_or_else(|_| "<default>".to_string()),
        include_mechanism_class,
    );

    for duel in duel_specs
        .iter()
        .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
    {
        let opponent_budget = SearchBudget::from_preference(duel.opponent_mode);
        let mut total_games = 0usize;
        let mut regressions = 0usize;
        let mut improvements = 0usize;
        let mut flat = 0usize;
        let mut nonwins = 0usize;
        let mut recorded = 0usize;
        let mut missing_first_diff = 0usize;
        let mut printed = 0usize;
        let mut branch_counts = BTreeMap::<String, usize>::new();
        let mut context_counts = BTreeMap::<String, usize>::new();
        let mut pair_counts = BTreeMap::<String, usize>::new();
        let mut mechanism_class_counts = BTreeMap::<String, usize>::new();

        for repeat_index in 0..repeats {
            let seed = seed_for_budget_duel_repeat_and_tag(
                pro_budget(),
                opponent_budget,
                repeat_index,
                duel.seed_tag.as_str(),
            );
            let opening_fens = generate_opening_fens_cached(seed, games);
            for (game_index, opening_fen) in opening_fens.iter().enumerate() {
                let variant = MonsGame::from_fen(opening_fen.as_str(), false)
                    .expect("valid opening fen")
                    .variant();
                for candidate_is_white in [true, false] {
                    total_games += 1;
                    let candidate_trace = play_profile_sweep_attribution_trace(
                        candidate,
                        shipping_selector,
                        opponent_budget,
                        opening_fen.as_str(),
                        candidate_is_white,
                        max_plies,
                    );
                    let shipping_trace = play_profile_sweep_attribution_trace(
                        shipping_control,
                        shipping_selector,
                        opponent_budget,
                        opening_fen.as_str(),
                        candidate_is_white,
                        max_plies,
                    );
                    let delta = match_result_points(candidate_trace.result)
                        - match_result_points(shipping_trace.result);
                    let candidate_won = matches!(candidate_trace.result, MatchResult::ProfileAWin);
                    if !candidate_won {
                        nonwins += 1;
                    }
                    if delta < 0 {
                        regressions += 1;
                    } else if delta > 0 {
                        improvements += 1;
                    } else {
                        flat += 1;
                    }
                    let outcome = if scope == "nonwins" && !candidate_won {
                        match delta.cmp(&0) {
                            std::cmp::Ordering::Less => "nonwin_regression",
                            std::cmp::Ordering::Greater => "nonwin_improvement",
                            std::cmp::Ordering::Equal => "nonwin_flat",
                        }
                    } else if delta < 0 {
                        "regression"
                    } else if delta > 0 {
                        "improvement"
                    } else {
                        "flat"
                    };
                    let should_record = if scope == "nonwins" {
                        !candidate_won
                    } else {
                        delta != 0
                    };
                    if !should_record || !pro_sweep_filter_allows(&outcome_filter, outcome) {
                        continue;
                    }
                    recorded += 1;

                    let Some(divergence) =
                        first_profile_sweep_candidate_divergence(&candidate_trace, &shipping_trace)
                    else {
                        missing_first_diff += 1;
                        continue;
                    };
                    let context_key = pro_sweep_candidate_record_context_key(
                        duel.label,
                        variant,
                        outcome,
                        &divergence,
                    );
                    let pair_key = format!(
                        "outcome={} duel={} variant={} branch={} candidate_move={} shipping_move={}",
                        outcome,
                        duel.label,
                        automove_variant_label(variant),
                        divergence.left_branch,
                        divergence.left_move_fen,
                        divergence.right_move_fen,
                    );
                    let branch_key = format!(
                        "outcome={} duel={} variant={} branch={}",
                        outcome,
                        duel.label,
                        automove_variant_label(variant),
                        divergence.left_branch,
                    );
                    *context_counts.entry(context_key).or_default() += 1;
                    *pair_counts.entry(pair_key).or_default() += 1;
                    *branch_counts.entry(branch_key).or_default() += 1;
                    if include_mechanism_class {
                        let board = MonsGame::from_fen(divergence.board_fen.as_str(), false)
                            .expect("decision-record board fen should be valid");
                        let mechanism_profile =
                            pro_policy_mechanism_profile_for_baseline(candidate.id);
                        let candidate_probe = runtime_decision_probe(
                            mechanism_profile,
                            SmartAutomovePreference::Pro,
                            &board,
                        );
                        let candidate_advisor = pro_v2_root_advisor_decision_snapshot();
                        for class_key in pro_policy_mechanism_class_keys(
                            mechanism_profile,
                            SmartAutomovePreference::Pro,
                            &board,
                            &candidate_probe,
                            candidate_advisor.as_ref(),
                            divergence.left_move_fen.as_str(),
                            divergence.right_move_fen.as_str(),
                        ) {
                            let key = format!("outcome={} {}", outcome, class_key);
                            *mechanism_class_counts.entry(key).or_default() += 1;
                        }
                    }

                    if printed < trace_limit {
                        println!(
                            "PRO_SWEEP_DECISION_RECORD {{\"candidate\":\"{}\",\"shipping\":\"{}\",\"duel\":\"{}\",\"repeat\":{},\"opening_index\":{},\"variant\":\"{}\",\"candidate_is_white\":{},\"outcome\":\"{}\",\"delta\":{},\"candidate_result\":\"{}\",\"shipping_control_result\":\"{}\",\"first_diff_ply\":{},\"candidate_branch\":\"{}\",\"active_color\":\"{}\",\"turn\":{},\"mons_moves\":{},\"can_action\":{},\"can_mana\":{},\"candidate_move\":\"{}\",\"shipping_control_move\":\"{}\",\"exact_context\":\"{}\",\"board\":\"{}\"}}",
                            json_escape(candidate.id),
                            json_escape(&shipping_profile),
                            json_escape(duel.label),
                            repeat_index,
                            game_index,
                            automove_variant_label(variant),
                            candidate_is_white,
                            outcome,
                            delta,
                            format_match_result(candidate_trace.result),
                            format_match_result(shipping_trace.result),
                            divergence.ply,
                            json_escape(divergence.left_branch),
                            pro_profile_sweep_color_label(divergence.active_color),
                            divergence.turn_number,
                            divergence.mons_moves_count,
                            divergence.can_use_action,
                            divergence.can_move_mana,
                            json_escape(&divergence.left_move_fen),
                            json_escape(&divergence.right_move_fen),
                            json_escape(&divergence.exact_context),
                            json_escape(&divergence.board_fen),
                        );
                        printed += 1;
                    }
                }
            }
        }

        println!(
            "PRO_SWEEP_DECISION_RECORD_SUMMARY {{\"candidate\":\"{}\",\"shipping\":\"{}\",\"duel\":\"{}\",\"scope\":\"{}\",\"total_games\":{},\"regressions\":{},\"improvements\":{},\"flat\":{},\"nonwins\":{},\"recorded\":{},\"missing_first_diff\":{}}}",
            json_escape(candidate.id),
            json_escape(&shipping_profile),
            json_escape(duel.label),
            json_escape(&scope),
            total_games,
            regressions,
            improvements,
            flat,
            nonwins,
            recorded,
            missing_first_diff,
        );
        println!(
            "PRO_SWEEP_DECISION_RECORD_STOPLIGHT {{\"candidate\":\"{}\",\"duel\":\"{}\",\"scope\":\"{}\",\"label\":\"{}\",\"nonwins\":{},\"regressions\":{},\"recorded\":{},\"missing_first_diff\":{},\"max_branch_games\":{},\"max_context_games\":{},\"max_pair_games\":{},\"max_mechanism_class_games\":{}}}",
            json_escape(candidate.id),
            json_escape(duel.label),
            json_escape(&scope),
            pro_sweep_decision_record_stoplight_label(&SweepDecisionRecordStoplightInput {
                regressions,
                nonwins,
                recorded,
                missing_first_diff,
                branch_counts: &branch_counts,
                context_counts: &context_counts,
                pair_counts: &pair_counts,
                mechanism_class_counts: &mechanism_class_counts,
                include_mechanism_class,
            }),
            nonwins,
            regressions,
            recorded,
            missing_first_diff,
            max_count(&branch_counts),
            max_count(&context_counts),
            max_count(&pair_counts),
            max_count(&mechanism_class_counts),
        );
        for (key, games) in branch_counts.iter().take(aggregate_limit) {
            println!(
                "PRO_SWEEP_DECISION_RECORD_BRANCH {{\"candidate\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                json_escape(candidate.id),
                json_escape(duel.label),
                json_escape(key),
                games,
            );
        }
        for (key, games) in context_counts.iter().take(aggregate_limit) {
            println!(
                "PRO_SWEEP_DECISION_RECORD_CONTEXT {{\"candidate\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                json_escape(candidate.id),
                json_escape(duel.label),
                json_escape(key),
                games,
            );
        }
        for (key, games) in pair_counts.iter().take(aggregate_limit) {
            println!(
                "PRO_SWEEP_DECISION_RECORD_PAIR {{\"candidate\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                json_escape(candidate.id),
                json_escape(duel.label),
                json_escape(key),
                games,
            );
        }
        if include_mechanism_class {
            for (key, games) in
                pro_policy_matrix_sorted_counts(&mechanism_class_counts, aggregate_limit)
            {
                println!(
                    "PRO_SWEEP_DECISION_RECORD_MECHANISM_CLASS {{\"candidate\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                    json_escape(candidate.id),
                    json_escape(duel.label),
                    json_escape(key),
                    games,
                );
            }
        }
    }
}

#[test]
#[ignore = "diagnostic: broad Pro profile sweep with structured per-duel and per-variant summaries"]
fn smart_automove_pro_profile_sweep_probe() {
    #[derive(Clone)]
    struct SweepDuelSpec {
        label: &'static str,
        opponent_mode: SmartAutomovePreference,
        seed_tag: String,
    }

    let shipping_profile = reliability_shipping_profile_id();
    let shipping_selector = profile_selector_from_name(shipping_profile.as_str())
        .unwrap_or_else(|| panic!("shipping '{}' not found", shipping_profile));
    let candidate_filter = pro_sweep_filter_tokens(
        "SMART_PRO_SWEEP_CANDIDATES",
        "frontier_pro_v2_guarded,frontier_pro_v2_raw",
    );
    let duel_filter = pro_sweep_filter_tokens("SMART_PRO_SWEEP_DUEL_FILTER", "all");
    let repeats = env_usize("SMART_PRO_SWEEP_REPEATS").unwrap_or(2).max(1);
    let games = env_usize("SMART_PRO_SWEEP_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_SWEEP_MAX_PLIES").unwrap_or(96).max(56);
    let seed_tag = env_string_value("SMART_PRO_SWEEP_SEED_TAG")
        .unwrap_or_else(|| "pro_profile_sweep_v1".to_string());
    let duel_specs = [
        SweepDuelSpec {
            label: "vs_shipping_pro",
            opponent_mode: SmartAutomovePreference::Pro,
            seed_tag: seed_tag.clone(),
        },
        SweepDuelSpec {
            label: "vs_shipping_normal",
            opponent_mode: SmartAutomovePreference::Normal,
            seed_tag: format!("{}_vs_normal", seed_tag),
        },
        SweepDuelSpec {
            label: "vs_shipping_fast",
            opponent_mode: SmartAutomovePreference::Fast,
            seed_tag: format!("{}_vs_fast", seed_tag),
        },
    ];
    let candidates = pro_profile_sweep_candidates()
        .into_iter()
        .filter(|candidate| pro_sweep_filter_allows(&candidate_filter, candidate.id))
        .collect::<Vec<_>>();
    assert!(
        !candidates.is_empty(),
        "SMART_PRO_SWEEP_CANDIDATES did not match any sweep candidates"
    );

    println!(
        "pro profile sweep: candidates={} duels={} repeats={} games={} max_plies={} variants={}",
        candidates
            .iter()
            .map(|candidate| candidate.id)
            .collect::<Vec<_>>()
            .join(","),
        duel_specs
            .iter()
            .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
            .map(|duel| duel.label)
            .collect::<Vec<_>>()
            .join(","),
        repeats,
        games,
        max_plies,
        env::var("SMART_AUTOMOVE_VARIANTS").unwrap_or_else(|_| "<default>".to_string())
    );

    for candidate in candidates {
        for duel in duel_specs
            .iter()
            .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
        {
            clear_profile_sweep_branch_counts();
            let stats = run_cross_model_duel_with_timing(CrossModelDuelConfig {
                label_a: candidate.id,
                model_a: AutomoveModel {
                    select_inputs: candidate.selector,
                },
                budget_a: pro_budget(),
                label_b: shipping_profile.as_str(),
                model_b: AutomoveModel {
                    select_inputs: shipping_selector,
                },
                budget_b: SearchBudget::from_preference(duel.opponent_mode),
                seed_tag: duel.seed_tag.as_str(),
                repeats,
                games_per_repeat: games,
                max_plies,
            });
            print_profile_sweep_branch_counts(candidate.id, duel.label);
            print_profile_sweep_summary(
                candidate.id,
                shipping_profile.as_str(),
                duel.label,
                duel.opponent_mode,
                &stats,
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: one-stop Pro promotion dashboard over sampled and active-blocker panels"]
fn smart_automove_pro_promotion_dashboard_probe() {
    #[derive(Clone)]
    struct DashboardDuelSpec {
        label: &'static str,
        opponent_mode: SmartAutomovePreference,
        seed_suffix: &'static str,
    }

    let shipping_profile = reliability_shipping_profile_id();
    let shipping_selector = profile_selector_from_name(shipping_profile.as_str())
        .unwrap_or_else(|| panic!("shipping '{}' not found", shipping_profile));
    let guarded_candidate = pro_profile_sweep_candidate_by_id("frontier_pro_v2_guarded");
    let candidate_filter =
        pro_sweep_filter_tokens("SMART_PRO_DASHBOARD_CANDIDATES", "frontier_pro_v2_raw");
    let panel_filter = pro_sweep_filter_tokens("SMART_PRO_DASHBOARD_PANEL_FILTER", "all");
    let duel_filter = pro_sweep_filter_tokens("SMART_PRO_DASHBOARD_DUEL_FILTER", "all");
    let include_guarded_delta = env_bool("SMART_PRO_DASHBOARD_INCLUDE_GUARDED").unwrap_or(true);
    let skip_guarded_after_shipping_fail =
        env_bool("SMART_PRO_DASHBOARD_SKIP_GUARDED_AFTER_SHIPPING_FAIL").unwrap_or(true);
    let promotion_fast_fail = env_bool("SMART_PRO_DASHBOARD_PROMOTION_FAST_FAIL").unwrap_or(false);
    let max_plies = env_usize("SMART_PRO_DASHBOARD_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let duel_specs = [
        DashboardDuelSpec {
            label: "vs_shipping_pro",
            opponent_mode: SmartAutomovePreference::Pro,
            seed_suffix: "",
        },
        DashboardDuelSpec {
            label: "vs_shipping_normal",
            opponent_mode: SmartAutomovePreference::Normal,
            seed_suffix: "_vs_normal",
        },
        DashboardDuelSpec {
            label: "vs_shipping_fast",
            opponent_mode: SmartAutomovePreference::Fast,
            seed_suffix: "_vs_fast",
        },
    ];
    let candidates = pro_profile_sweep_candidates()
        .into_iter()
        .filter(|candidate| pro_sweep_filter_allows(&candidate_filter, candidate.id))
        .collect::<Vec<_>>();
    assert!(
        !candidates.is_empty(),
        "SMART_PRO_DASHBOARD_CANDIDATES did not match any sweep candidates"
    );

    println!(
        "pro promotion dashboard: candidates={} panels={} duels={} max_plies={} include_guarded_delta={} skip_guarded_after_shipping_fail={} promotion_fast_fail={}",
        candidates
            .iter()
            .map(|candidate| candidate.id)
            .collect::<Vec<_>>()
            .join(","),
        pro_promotion_dashboard_panel_specs()
            .into_iter()
            .filter(|panel| pro_sweep_filter_allows(&panel_filter, panel.label))
            .map(|panel| panel.label)
            .collect::<Vec<_>>()
            .join(","),
        duel_specs
            .iter()
            .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
            .map(|duel| duel.label)
            .collect::<Vec<_>>()
            .join(","),
        max_plies,
        include_guarded_delta,
        skip_guarded_after_shipping_fail,
        promotion_fast_fail,
    );

    for candidate in candidates {
        let mut panel_summaries = Vec::<ProPromotionDashboardPanelSummary>::new();
        'panels: for panel in pro_promotion_dashboard_panel_specs()
            .into_iter()
            .filter(|panel| pro_sweep_filter_allows(&panel_filter, panel.label))
        {
            let repeats = env_usize("SMART_PRO_DASHBOARD_REPEATS")
                .unwrap_or(panel.default_repeats)
                .max(1);
            let games = env_usize("SMART_PRO_DASHBOARD_GAMES")
                .unwrap_or(panel.default_games)
                .max(1);
            let panel_seed_tag = env_string_value("SMART_PRO_DASHBOARD_SEED_TAG")
                .unwrap_or_else(|| panel.seed_tag.to_string());
            let mut summary = ProPromotionDashboardPanelSummary::new(panel.label);

            with_pro_promotion_dashboard_panel(panel, || {
                for duel in duel_specs
                    .iter()
                    .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
                {
                    let duel_seed_tag = format!("{}{}", panel_seed_tag, duel.seed_suffix);
                    clear_profile_sweep_branch_counts();
                    let stats = run_cross_model_duel_with_timing(CrossModelDuelConfig {
                        label_a: candidate.id,
                        model_a: AutomoveModel {
                            select_inputs: candidate.selector,
                        },
                        budget_a: pro_budget(),
                        label_b: shipping_profile.as_str(),
                        model_b: AutomoveModel {
                            select_inputs: shipping_selector,
                        },
                        budget_b: SearchBudget::from_preference(duel.opponent_mode),
                        seed_tag: duel_seed_tag.as_str(),
                        repeats,
                        games_per_repeat: games,
                        max_plies,
                    });
                    print_profile_sweep_branch_counts(candidate.id, duel.label);
                    print_pro_promotion_dashboard_result(
                        panel.label,
                        candidate.id,
                        "shipping",
                        duel.label,
                        shipping_profile.as_str(),
                        duel.opponent_mode,
                        &stats,
                    );
                    print_pro_promotion_dashboard_variants(
                        panel.label,
                        candidate.id,
                        "shipping",
                        duel.label,
                        &stats,
                    );
                    let metrics = pro_reliability_metrics(&stats);
                    summary.record_shipping_duel(&stats);
                    let shipping_gate_failed = match panel.label {
                        "sampled" => !pro_reliability_duel_passes(metrics),
                        "active_blockers" => {
                            metrics.win_rate < 0.90 || metrics.frontier_avg_ms > 700.0
                        }
                        _ => false,
                    };
                    if promotion_fast_fail && shipping_gate_failed {
                        println!(
                            "PRO_PROMOTION_DASHBOARD_FAST_FAIL {{\"panel\":\"{}\",\"candidate\":\"{}\",\"duel\":\"{}\",\"reason\":\"shipping_gate_failed\",\"shipping_duels\":{},\"shipping_strict_passes\":{},\"shipping_directional_passes\":{}}}",
                            json_escape(panel.label),
                            json_escape(candidate.id),
                            json_escape(duel.label),
                            summary.shipping_duels,
                            summary.shipping_strict_passes,
                            summary.shipping_directional_passes,
                        );
                        break;
                    }
                }

                let run_guarded_delta = include_guarded_delta
                    && (!skip_guarded_after_shipping_fail
                        || summary.shipping_directional_passes_all());
                if run_guarded_delta {
                    let guarded_seed_tag = format!("{}_vs_guarded", panel_seed_tag);
                    let stats = run_cross_model_duel_with_timing(CrossModelDuelConfig {
                        label_a: candidate.id,
                        model_a: AutomoveModel {
                            select_inputs: candidate.selector,
                        },
                        budget_a: pro_budget(),
                        label_b: guarded_candidate.id,
                        model_b: AutomoveModel {
                            select_inputs: guarded_candidate.selector,
                        },
                        budget_b: pro_budget(),
                        seed_tag: guarded_seed_tag.as_str(),
                        repeats,
                        games_per_repeat: games,
                        max_plies,
                    });
                    print_pro_promotion_dashboard_result(
                        panel.label,
                        candidate.id,
                        "guarded",
                        "vs_frontier_pro_v2_guarded",
                        guarded_candidate.id,
                        SmartAutomovePreference::Pro,
                        &stats,
                    );
                    print_pro_promotion_dashboard_variants(
                        panel.label,
                        candidate.id,
                        "guarded",
                        "vs_frontier_pro_v2_guarded",
                        &stats,
                    );
                    summary.record_guarded_duel(&stats);
                } else if include_guarded_delta {
                    println!(
                        "PRO_PROMOTION_DASHBOARD_GUARDED_SKIPPED {{\"panel\":\"{}\",\"candidate\":\"{}\",\"reason\":\"shipping_direction_failed\",\"shipping_duels\":{},\"shipping_directional_passes\":{}}}",
                        json_escape(panel.label),
                        json_escape(candidate.id),
                        summary.shipping_duels,
                        summary.shipping_directional_passes,
                    );
                }
            });

            println!(
                "PRO_PROMOTION_DASHBOARD_PANEL {{\"candidate\":\"{}\",\"panel\":\"{}\",\"shipping_duels\":{},\"shipping_strict_passes\":{},\"shipping_directional_passes\":{},\"min_shipping_win_rate\":{:.4},\"min_shipping_confidence\":{:.4},\"max_candidate_avg_ms\":{:.2},\"weak_variant_rows\":{},\"guarded_win_rate\":{:.4},\"guarded_confidence\":{:.4}}}",
                json_escape(candidate.id),
                json_escape(summary.panel),
                summary.shipping_duels,
                summary.shipping_strict_passes,
                summary.shipping_directional_passes,
                summary.min_shipping_win_rate,
                summary.min_shipping_confidence,
                summary.max_candidate_avg_ms,
                summary.weak_variant_rows,
                summary.guarded_win_rate,
                summary.guarded_confidence,
            );
            panel_summaries.push(summary);
            if promotion_fast_fail
                && panel_summaries
                    .last()
                    .is_some_and(|summary| match summary.panel {
                        "sampled" => !summary.shipping_strict_passes_all(),
                        "active_blockers" => !summary.shipping_directional_passes_all(),
                        _ => false,
                    })
            {
                break 'panels;
            }
        }

        let classification = match (
            panel_summaries
                .iter()
                .find(|summary| summary.panel == "sampled"),
            panel_summaries
                .iter()
                .find(|summary| summary.panel == "active_blockers"),
        ) {
            (Some(sampled), Some(active)) => {
                pro_promotion_dashboard_directional_label(sampled, active)
            }
            _ => "partial_dashboard",
        };
        println!(
            "PRO_PROMOTION_DASHBOARD_CANDIDATE {{\"candidate\":\"{}\",\"classification\":\"{}\",\"panels\":{}}}",
            json_escape(candidate.id),
            json_escape(classification),
            panel_summaries.len(),
        );
        println!(
            "PRO_PROMOTION_DASHBOARD_STOPLIGHT {{\"candidate\":\"{}\",\"label\":\"{}\",\"classification\":\"{}\",\"panels\":{},\"max_candidate_avg_ms\":{:.2}}}",
            json_escape(candidate.id),
            pro_promotion_dashboard_stoplight_label(classification, &panel_summaries),
            json_escape(classification),
            panel_summaries.len(),
            panel_summaries
                .iter()
                .map(|summary| summary.max_candidate_avg_ms)
                .fold(0.0, f64::max),
        );
    }
}

fn decision_record_root_advisor_entry_status(
    entries: &[crate::models::mons_game_model::ProV2RootAdvisorEntry],
    root_fen: &str,
) -> Option<String> {
    entries.iter().find_map(|entry| {
        (Input::fen_from_array(&entry.inputs) == root_fen).then(|| {
            format!(
                "{:?}:{:?}:rank{}",
                entry.reason, entry.family, entry.root_rank
            )
        })
    })
}

fn decision_record_baseline_status(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
    probe: &RuntimeDecisionProbe,
    advisor: Option<&crate::models::mons_game_model::ProV2RootAdvisorDecision>,
    baseline_move_fen: &str,
) -> String {
    let (_, scored_roots, _, _) =
        profile_runtime_scored_roots_with_forced_engine_inputs(profile_name, mode, game);
    decision_record_baseline_status_from_scored_roots(
        scored_roots.as_slice(),
        probe,
        advisor,
        baseline_move_fen,
    )
}

fn decision_record_baseline_status_from_scored_roots(
    scored_roots: &[RootEvaluation],
    probe: &RuntimeDecisionProbe,
    advisor: Option<&crate::models::mons_game_model::ProV2RootAdvisorDecision>,
    baseline_move_fen: &str,
) -> String {
    let mut parts = Vec::<String>::new();

    if baseline_move_fen == probe.selected_input_fen {
        parts.push("selected".to_string());
    }
    if baseline_move_fen == probe.pre_accept_input_fen {
        parts.push("pre_accept".to_string());
    }
    if probe
        .head_input_fen
        .as_ref()
        .is_some_and(|head| head == baseline_move_fen)
    {
        parts.push("head".to_string());
    }
    if baseline_move_fen == probe.legacy_selected_input_fen {
        parts.push("legacy_selected".to_string());
    }
    if baseline_move_fen == probe.legacy_full_pool_selected_input_fen {
        parts.push("legacy_full_pool_selected".to_string());
    }

    if let Some((index, root)) = scored_roots
        .iter()
        .enumerate()
        .find(|(_, root)| Input::fen_from_array(&root.inputs) == baseline_move_fen)
    {
        parts.push(format!(
            "candidate_live:index{}:rank{}:{:?}",
            index,
            root.root_rank,
            MonsGameModel::turn_engine_root_evaluation_family(root)
        ));
    } else {
        parts.push("candidate_omitted".to_string());
    }

    if let Some(advisor) = advisor {
        if advisor
            .approved_root
            .as_ref()
            .is_some_and(|entry| Input::fen_from_array(&entry.inputs) == baseline_move_fen)
        {
            parts.push("advisor_approved".to_string());
        }
        if let Some(status) =
            decision_record_root_advisor_entry_status(&advisor.ordered_shortlist, baseline_move_fen)
        {
            parts.push(format!("advisor_ordered:{status}"));
        }
        if let Some(status) = decision_record_root_advisor_entry_status(
            &advisor.preserved_family_representatives,
            baseline_move_fen,
        ) {
            parts.push(format!("advisor_preserved:{status}"));
        }
        if advisor.injected_root.as_ref().is_some_and(|root| {
            Input::fen_from_array(&root.inputs) == baseline_move_fen && root.admitted
        }) {
            parts.push("advisor_injected_admitted".to_string());
        }
        if advisor.injected_root.as_ref().is_some_and(|root| {
            Input::fen_from_array(&root.inputs) == baseline_move_fen && !root.admitted
        }) {
            parts.push("advisor_injected_rejected".to_string());
        }
    } else {
        parts.push("advisor_none".to_string());
    }

    parts.join("|")
}

fn decision_record_approved_status(
    advisor: Option<&crate::models::mons_game_model::ProV2RootAdvisorDecision>,
) -> String {
    advisor
        .and_then(|advisor| advisor.approved_root.as_ref())
        .map(format_root_advisor_entry_probe)
        .unwrap_or_else(|| "none".to_string())
}

#[test]
#[ignore = "diagnostic: aggregate first-divergence decision records for Pro reliability duels"]
fn smart_automove_pro_decision_record_aggregation_probe() {
    #[derive(Clone)]
    struct DecisionRecordDuelSpec {
        label: &'static str,
        opponent_mode: SmartAutomovePreference,
        seed_tag: String,
    }

    let frontier_profile = reliability_frontier_profile_id();
    let shipping_profile = reliability_shipping_profile_id();
    let repeats = env_usize("SMART_PRO_DECISION_RECORD_REPEATS")
        .unwrap_or(3)
        .max(1);
    let games = env_usize("SMART_PRO_DECISION_RECORD_GAMES")
        .unwrap_or(2)
        .max(1);
    let max_plies = env_usize("SMART_PRO_DECISION_RECORD_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let trace_limit = env_usize("SMART_PRO_DECISION_RECORD_TRACE_LIMIT")
        .unwrap_or(24)
        .max(1);
    let aggregate_limit = env_usize("SMART_PRO_DECISION_RECORD_AGGREGATE_LIMIT")
        .unwrap_or(64)
        .max(1);
    let seed_tag = env_string_value("SMART_PRO_DECISION_RECORD_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());
    let duel_filter = pro_sweep_filter_tokens("SMART_PRO_DECISION_RECORD_DUEL_FILTER", "all");
    let outcome_filter = pro_sweep_filter_tokens("SMART_PRO_DECISION_RECORD_OUTCOME", "all");
    let scope =
        env_string_value("SMART_PRO_DECISION_RECORD_SCOPE").unwrap_or_else(|| "delta".to_string());
    let outcome_scope = format!("{} scope={}", outcome_filter.join(","), scope);
    let duel_specs = [
        DecisionRecordDuelSpec {
            label: "vs_shipping_pro",
            opponent_mode: SmartAutomovePreference::Pro,
            seed_tag: seed_tag.clone(),
        },
        DecisionRecordDuelSpec {
            label: "vs_shipping_normal",
            opponent_mode: SmartAutomovePreference::Normal,
            seed_tag: format!("{}_vs_normal", seed_tag),
        },
        DecisionRecordDuelSpec {
            label: "vs_shipping_fast",
            opponent_mode: SmartAutomovePreference::Fast,
            seed_tag: format!("{}_vs_fast", seed_tag),
        },
    ];

    println!(
        "pro decision record aggregation: frontier={} shipping={} repeats={} games={} max_plies={} duels={} outcome_filter={} variants={}",
        frontier_profile,
        shipping_profile,
        repeats,
        games,
        max_plies,
        duel_specs
            .iter()
            .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
            .map(|duel| duel.label)
            .collect::<Vec<_>>()
            .join(","),
        outcome_scope,
        env::var("SMART_AUTOMOVE_VARIANTS").unwrap_or_else(|_| "<default>".to_string())
    );

    for duel in duel_specs
        .iter()
        .filter(|duel| pro_sweep_filter_allows(&duel_filter, duel.label))
    {
        let opponent_budget = SearchBudget::from_preference(duel.opponent_mode);
        let mut total_games = 0usize;
        let mut regressions = 0usize;
        let mut improvements = 0usize;
        let mut flat = 0usize;
        let mut nonwins = 0usize;
        let mut recorded = 0usize;
        let mut missing_first_diff = 0usize;
        let mut printed = 0usize;
        let mut mechanism_counts = BTreeMap::<String, usize>::new();
        let mut baseline_counts = BTreeMap::<String, usize>::new();
        let mut stage_counts = BTreeMap::<String, usize>::new();

        for repeat_index in 0..repeats {
            let seed = seed_for_budget_duel_repeat_and_tag(
                pro_budget(),
                opponent_budget,
                repeat_index,
                duel.seed_tag.as_str(),
            );
            let opening_fens = generate_opening_fens_cached(seed, games);
            for (game_index, opening_fen) in opening_fens.iter().enumerate() {
                let variant = MonsGame::from_fen(opening_fen.as_str(), false)
                    .expect("valid opening fen")
                    .variant();
                for frontier_is_white in [true, false] {
                    total_games += 1;
                    let frontier_trace = play_profile_duel_trace(
                        frontier_profile.as_str(),
                        shipping_profile.as_str(),
                        duel.opponent_mode,
                        opening_fen.as_str(),
                        frontier_is_white,
                        max_plies,
                    );
                    let shipping_trace = play_profile_duel_trace(
                        shipping_profile.as_str(),
                        shipping_profile.as_str(),
                        duel.opponent_mode,
                        opening_fen.as_str(),
                        frontier_is_white,
                        max_plies,
                    );
                    let delta = match_result_points(frontier_trace.result)
                        - match_result_points(shipping_trace.result);
                    let frontier_won = matches!(frontier_trace.result, MatchResult::ProfileAWin);
                    if !frontier_won {
                        nonwins += 1;
                    }
                    if delta < 0 {
                        regressions += 1;
                    } else if delta > 0 {
                        improvements += 1;
                    } else {
                        flat += 1;
                    }
                    let outcome = if scope == "nonwins" && !frontier_won {
                        match delta.cmp(&0) {
                            std::cmp::Ordering::Less => "nonwin_regression",
                            std::cmp::Ordering::Greater => "nonwin_improvement",
                            std::cmp::Ordering::Equal => "nonwin_flat",
                        }
                    } else if delta < 0 {
                        "regression"
                    } else if delta > 0 {
                        "improvement"
                    } else {
                        "flat"
                    };
                    let should_record = if scope == "nonwins" {
                        !frontier_won
                    } else {
                        delta != 0
                    };
                    if !should_record || !pro_sweep_filter_allows(&outcome_filter, outcome) {
                        continue;
                    }
                    recorded += 1;

                    let Some(divergence) =
                        first_duel_trace_divergence(&frontier_trace, &shipping_trace)
                    else {
                        missing_first_diff += 1;
                        continue;
                    };
                    let board = MonsGame::from_fen(divergence.board_fen.as_str(), false)
                        .expect("trace board fen should be valid");
                    let frontier_probe = runtime_decision_probe(
                        frontier_profile.as_str(),
                        SmartAutomovePreference::Pro,
                        &board,
                    );
                    let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
                    let shipping_probe = runtime_decision_probe(
                        shipping_profile.as_str(),
                        SmartAutomovePreference::Pro,
                        &board,
                    );
                    let baseline_status = decision_record_baseline_status(
                        frontier_profile.as_str(),
                        SmartAutomovePreference::Pro,
                        &board,
                        &frontier_probe,
                        frontier_advisor.as_ref(),
                        divergence.profile_b_move_fen.as_str(),
                    );
                    let approved_status =
                        decision_record_approved_status(frontier_advisor.as_ref());
                    let mechanism_key = format!(
                        "outcome={} duel={} variant={} color={} turn={} mons_moves={} branch={} stage={} selected_rank={:?} pre_family={:?} head_family={:?} head_accepted={} baseline_status={} approved={}",
                        outcome,
                        duel.label,
                        automove_variant_label(variant),
                        pro_profile_sweep_color_label(board.active_color),
                        board.turn_number,
                        board.mons_moves_count,
                        frontier_probe.runtime_variant_branch,
                        frontier_probe.selector_last_stage,
                        frontier_probe.selected_rank,
                        frontier_probe.pre_accept_family,
                        frontier_probe.head_family,
                        frontier_probe.head_accepted,
                        baseline_status,
                        approved_status,
                    );
                    let baseline_key = format!(
                        "outcome={} duel={} baseline_status={}",
                        outcome, duel.label, baseline_status
                    );
                    let stage_key = format!(
                        "outcome={} duel={} branch={} stage={} pre_family={:?} head_family={:?} head_accepted={}",
                        outcome,
                        duel.label,
                        frontier_probe.runtime_variant_branch,
                        frontier_probe.selector_last_stage,
                        frontier_probe.pre_accept_family,
                        frontier_probe.head_family,
                        frontier_probe.head_accepted,
                    );
                    *mechanism_counts.entry(mechanism_key.clone()).or_default() += 1;
                    *baseline_counts.entry(baseline_key).or_default() += 1;
                    *stage_counts.entry(stage_key).or_default() += 1;

                    if printed < trace_limit {
                        println!(
                            "PRO_DECISION_RECORD {{\"frontier\":\"{}\",\"shipping\":\"{}\",\"duel\":\"{}\",\"repeat\":{},\"opening_index\":{},\"variant\":\"{}\",\"frontier_is_white\":{},\"outcome\":\"{}\",\"delta\":{},\"frontier_result\":\"{}\",\"shipping_result\":\"{}\",\"first_diff_ply\":{},\"active_color\":\"{}\",\"turn\":{},\"mons_moves\":{},\"can_action\":{},\"can_mana\":{},\"frontier_move\":\"{}\",\"shipping_move\":\"{}\",\"frontier_selected\":\"{}\",\"frontier_pre_accept\":\"{}\",\"frontier_head\":{:?},\"frontier_stage\":\"{}\",\"frontier_branch\":\"{}\",\"frontier_pre_family\":\"{:?}\",\"frontier_head_family\":\"{:?}\",\"frontier_head_accepted\":{},\"baseline_status\":\"{}\",\"approved_status\":\"{}\",\"shipping_stage\":\"{}\",\"shipping_selected\":\"{}\",\"exact_context\":\"{}\",\"board\":\"{}\"}}",
                            json_escape(&frontier_profile),
                            json_escape(&shipping_profile),
                            json_escape(duel.label),
                            repeat_index,
                            game_index,
                            automove_variant_label(variant),
                            frontier_is_white,
                            outcome,
                            delta,
                            format_match_result(frontier_trace.result),
                            format_match_result(shipping_trace.result),
                            divergence.ply,
                            pro_profile_sweep_color_label(board.active_color),
                            board.turn_number,
                            board.mons_moves_count,
                            board.player_can_use_action(),
                            board.player_can_move_mana(),
                            json_escape(&divergence.profile_a_move_fen),
                            json_escape(&divergence.profile_b_move_fen),
                            json_escape(&frontier_probe.selected_input_fen),
                            json_escape(&frontier_probe.pre_accept_input_fen),
                            frontier_probe.head_input_fen.as_ref().map(|head| json_escape(head)),
                            json_escape(frontier_probe.selector_last_stage),
                            json_escape(frontier_probe.runtime_variant_branch),
                            frontier_probe.pre_accept_family,
                            frontier_probe.head_family,
                            frontier_probe.head_accepted,
                            json_escape(&baseline_status),
                            json_escape(&approved_status),
                            json_escape(shipping_probe.selector_last_stage),
                            json_escape(&shipping_probe.selected_input_fen),
                            json_escape(&frontier_probe.exact_context),
                            json_escape(&divergence.board_fen),
                        );
                        printed += 1;
                    }
                }
            }
        }

        println!(
            "PRO_DECISION_RECORD_SUMMARY {{\"frontier\":\"{}\",\"shipping\":\"{}\",\"duel\":\"{}\",\"scope\":\"{}\",\"total_games\":{},\"regressions\":{},\"improvements\":{},\"flat\":{},\"nonwins\":{},\"recorded\":{},\"missing_first_diff\":{}}}",
            json_escape(&frontier_profile),
            json_escape(&shipping_profile),
            json_escape(duel.label),
            json_escape(&scope),
            total_games,
            regressions,
            improvements,
            flat,
            nonwins,
            recorded,
            missing_first_diff,
        );
        for (key, games) in stage_counts.iter().take(aggregate_limit) {
            println!(
                "PRO_DECISION_RECORD_STAGE {{\"frontier\":\"{}\",\"shipping\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                json_escape(&frontier_profile),
                json_escape(&shipping_profile),
                json_escape(duel.label),
                json_escape(key),
                games,
            );
        }
        for (key, games) in baseline_counts.iter().take(aggregate_limit) {
            println!(
                "PRO_DECISION_RECORD_BASELINE {{\"frontier\":\"{}\",\"shipping\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                json_escape(&frontier_profile),
                json_escape(&shipping_profile),
                json_escape(duel.label),
                json_escape(key),
                games,
            );
        }
        for (key, games) in mechanism_counts.iter().take(aggregate_limit) {
            println!(
                "PRO_DECISION_RECORD_MECHANISM {{\"frontier\":\"{}\",\"shipping\":\"{}\",\"duel\":\"{}\",\"key\":\"{}\",\"games\":{}}}",
                json_escape(&frontier_profile),
                json_escape(&shipping_profile),
                json_escape(duel.label),
                json_escape(key),
                games,
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: replay exact pro-reliability duel seeds against shipping_pro_search and log first regression divergence"]
fn smart_automove_pro_reliability_duel_trace_probe() {
    use std::collections::BTreeMap;

    #[derive(Clone)]
    struct DuelSpec {
        label: &'static str,
        opponent_mode: SmartAutomovePreference,
        seed_tag: String,
    }

    let frontier_profile = reliability_frontier_profile_id();
    let shipping_profile = reliability_shipping_profile_id();
    let repeats = env_usize("SMART_PRO_RELIABILITY_REPEATS")
        .unwrap_or(3)
        .max(1);
    let games = env_usize("SMART_PRO_RELIABILITY_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_RELIABILITY_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let trace_limit = env_usize("SMART_PRO_RELIABILITY_TRACE_LIMIT")
        .unwrap_or(12)
        .max(1);
    let seed_tag = env_string_value("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());
    let duel_filter = env::var("SMART_PRO_RELIABILITY_DUEL_FILTER").ok();
    let duel_specs = [
        DuelSpec {
            label: "vs_shipping_pro",
            opponent_mode: SmartAutomovePreference::Pro,
            seed_tag: seed_tag.clone(),
        },
        DuelSpec {
            label: "vs_shipping_normal",
            opponent_mode: SmartAutomovePreference::Normal,
            seed_tag: format!("{}_vs_normal", seed_tag),
        },
        DuelSpec {
            label: "vs_shipping_fast",
            opponent_mode: SmartAutomovePreference::Fast,
            seed_tag: format!("{}_vs_fast", seed_tag),
        },
    ];

    println!(
        "pro reliability duel trace probe: frontier={} shipping={} repeats={} games_per_repeat={} max_plies={} trace_limit={} duel_filter={:?}",
        frontier_profile,
        shipping_profile,
        repeats,
        games,
        max_plies,
        trace_limit,
        duel_filter,
    );

    for duel in duel_specs {
        if duel_filter
            .as_deref()
            .is_some_and(|filter| filter != duel.label)
        {
            continue;
        }

        let opponent_budget = SearchBudget::from_preference(duel.opponent_mode);
        let mut regressions = 0usize;
        let mut improvements = 0usize;
        let mut total_games = 0usize;
        let mut printed = 0usize;
        let mut move_pair_counts = BTreeMap::<(String, String), usize>::new();

        for repeat_index in 0..repeats {
            let seed = seed_for_budget_duel_repeat_and_tag(
                pro_budget(),
                opponent_budget,
                repeat_index,
                duel.seed_tag.as_str(),
            );
            let opening_fens = generate_opening_fens_cached(seed, games);
            for (game_index, opening_fen) in opening_fens.iter().enumerate() {
                for frontier_is_white in [true, false] {
                    total_games += 1;
                    let frontier_trace = play_profile_duel_trace(
                        frontier_profile.as_str(),
                        shipping_profile.as_str(),
                        duel.opponent_mode,
                        opening_fen.as_str(),
                        frontier_is_white,
                        max_plies,
                    );
                    let shipping_trace = play_profile_duel_trace(
                        shipping_profile.as_str(),
                        shipping_profile.as_str(),
                        duel.opponent_mode,
                        opening_fen.as_str(),
                        frontier_is_white,
                        max_plies,
                    );
                    let delta = match_result_points(frontier_trace.result)
                        - match_result_points(shipping_trace.result);
                    if delta < 0 {
                        regressions += 1;
                        let first_divergence =
                            first_duel_trace_divergence(&frontier_trace, &shipping_trace);
                        if let Some(divergence) = first_divergence.as_ref() {
                            *move_pair_counts
                                .entry((
                                    divergence.profile_a_move_fen.clone(),
                                    divergence.profile_b_move_fen.clone(),
                                ))
                                .or_default() += 1;
                        }
                        if printed < trace_limit {
                            let detail = first_divergence.as_ref().map(|divergence| {
                                    let board = MonsGame::from_fen(
                                        divergence.board_fen.as_str(),
                                        false,
                                    )
                                    .expect("trace board fen should be valid");
                                    let frontier_probe = runtime_decision_probe(
                                        frontier_profile.as_str(),
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    let shipping_probe = runtime_decision_probe(
                                        shipping_profile.as_str(),
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    format!(
                                        "first_diff_ply={} board={} frontier_move={} shipping_move={} frontier(selected={} rank={:?} pre_accept={} pre_rank={:?} stage={} head={:?} head_rank={:?} accepted={} top={:?} selected_root=\"{}\" head_root=\"{}\") shipping(selected={} rank={:?} pre_accept={} pre_rank={:?} stage={} head={:?} head_rank={:?} accepted={} top={:?} selected_root=\"{}\" head_root=\"{}\")",
                                        divergence.ply,
                                        divergence.board_fen,
                                        divergence.profile_a_move_fen,
                                        divergence.profile_b_move_fen,
                                        frontier_probe.selected_input_fen,
                                        frontier_probe.selected_rank,
                                        frontier_probe.pre_accept_input_fen,
                                        frontier_probe.pre_accept_rank,
                                        frontier_probe.selector_last_stage,
                                        frontier_probe.head_input_fen,
                                        frontier_probe.head_rank,
                                        frontier_probe.head_accepted,
                                        frontier_probe.top_root_fens,
                                        frontier_probe.selected_root,
                                        frontier_probe.head_root,
                                        shipping_probe.selected_input_fen,
                                        shipping_probe.selected_rank,
                                        shipping_probe.pre_accept_input_fen,
                                        shipping_probe.pre_accept_rank,
                                        shipping_probe.selector_last_stage,
                                        shipping_probe.head_input_fen,
                                        shipping_probe.head_rank,
                                        shipping_probe.head_accepted,
                                        shipping_probe.top_root_fens,
                                        shipping_probe.selected_root,
                                        shipping_probe.head_root,
                                    )
                                });

                            println!(
                                    "PRO_RELIABILITY_TRACE duel={} repeat={} opening_index={} frontier_is_white={} opening={} frontier_result={} shipping_result={} frontier_final={} shipping_final={} {}",
                                    duel.label,
                                    repeat_index,
                                    game_index,
                                    frontier_is_white,
                                    opening_fen,
                                    format_match_result(frontier_trace.result),
                                    format_match_result(shipping_trace.result),
                                    frontier_trace.final_fen,
                                    shipping_trace.final_fen,
                                    detail.unwrap_or_else(|| "first_diff=none".to_string()),
                                );
                            printed += 1;
                        }
                    } else if delta > 0 {
                        improvements += 1;
                    }
                }
            }
        }

        println!(
                "PRO_RELIABILITY_TRACE_SUMMARY duel={} total_games={} regressions={} improvements={} flat={} repeated_move_pairs={:?}",
                duel.label,
                total_games,
                regressions,
                improvements,
                total_games.saturating_sub(regressions + improvements),
                move_pair_counts,
            );
    }
}

#[test]
#[ignore = "diagnostic: attribute black recovery branch reply-floor scoring"]
fn black_recovery_branch_reply_floor_attribution_probe() {
    fn root_attribution_details(
        game: &MonsGame,
        scored_roots: &[RootEvaluation],
        index: usize,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> String {
        let root = &scored_roots[index];
        let family = MonsGameModel::turn_engine_root_evaluation_family(root);
        let utility = MonsGameModel::turn_engine_selected_override_utility(
            game,
            root,
            perspective,
            config,
            family,
        );
        let snapshot = MonsGameModel::root_reply_risk_snapshot(
            &root.game,
            perspective,
            config,
            config.root_reply_risk_reply_limit.clamp(1, 24),
        );

        format!(
            "{} index={} family={:?} utility={} floor={} match_point={} immediate_win={} followup={} details={}",
            Input::fen_from_array(&root.inputs),
            index,
            family,
            format_turn_engine_utility_probe(utility),
            snapshot.worst_reply_score,
            snapshot.opponent_reaches_match_point,
            snapshot.allows_immediate_opponent_win,
            MonsGameModel::pro_v2_spirit_followup_floor_score(&root.game, perspective, config),
            format_root_probe(Some(root)),
        )
    }

    fn pair_attribution(
        left_label: &str,
        left_root: &RootEvaluation,
        right_label: &str,
        right_root: &RootEvaluation,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> String {
        let after_root = top_attribution_residual_deltas(
            &format!("{left_label}_after_root"),
            &left_root.game,
            &format!("{right_label}_after_root"),
            &right_root.game,
            perspective,
            config,
        );
        let after_root_variants = attribution_search_eval_variant_deltas(
            &format!("{left_label}_after_root"),
            &left_root.game,
            &format!("{right_label}_after_root"),
            &right_root.game,
            perspective,
            config,
        );
        let worst_reply = match (
            attribution_worst_reply_state(&left_root.game, perspective, config),
            attribution_worst_reply_state(&right_root.game, perspective, config),
        ) {
            (Some(left_reply), Some(right_reply)) => format!(
                "{}_worst_reply={} {}_worst_score={} {}_worst_events={} {}_worst_reply={} {}_worst_score={} {}_worst_events={} search_variants=[{}] {}",
                left_label,
                left_reply.input_fen,
                left_label,
                left_reply.score,
                left_label,
                left_reply.events,
                right_label,
                right_reply.input_fen,
                right_label,
                right_reply.score,
                right_label,
                right_reply.events,
                attribution_search_eval_variant_deltas(
                    &format!("{left_label}_worst_reply"),
                    &left_reply.game,
                    &format!("{right_label}_worst_reply"),
                    &right_reply.game,
                    perspective,
                    config,
                ),
                top_attribution_residual_deltas(
                    &format!("{left_label}_worst_reply"),
                    &left_reply.game,
                    &format!("{right_label}_worst_reply"),
                    &right_reply.game,
                    perspective,
                    config,
                ),
            ),
            (None, None) => "worst_reply=no_replies".to_string(),
            (None, Some(right_reply)) => format!(
                "{}_worst_reply=none {}_worst_reply={} {}_worst_score={} {}_worst_events={}",
                left_label,
                right_label,
                right_reply.input_fen,
                right_label,
                right_reply.score,
                right_label,
                right_reply.events,
            ),
            (Some(left_reply), None) => format!(
                "{}_worst_reply={} {}_worst_score={} {}_worst_events={} {}_worst_reply=none",
                left_label,
                left_reply.input_fen,
                left_label,
                left_reply.score,
                left_label,
                left_reply.events,
                right_label,
            ),
        };

        format!(
            "{}={} {}={} after_root={{ search_variants=[{}] {} }} worst_reply={{ {} }}",
            left_label,
            Input::fen_from_array(&left_root.inputs),
            right_label,
            Input::fen_from_array(&right_root.inputs),
            after_root_variants,
            after_root,
            worst_reply,
        )
    }

    let game = MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        false,
    )
    .expect("valid black recovery branch fen");
    let perspective = game.active_color;
    let frontier_probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_probe =
        runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    let (config, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let candidate_indices = MonsGameModel::filtered_root_candidate_indices(
        &game,
        scored_roots.as_slice(),
        perspective,
        config,
    );
    let shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
        scored_roots.as_slice(),
        candidate_indices.as_slice(),
        config,
    );
    let frontier_index = attribution_root_index(scored_roots.as_slice(), "l1,5;l3,3;l2,3")
        .expect("frontier spirit root should exist");
    let shipping_index = attribution_root_index(scored_roots.as_slice(), "l6,0;l6,1")
        .expect("shipping mana root should exist");
    let pro_v1_candidate_index = attribution_root_index(scored_roots.as_slice(), "l1,5;l2,7;l1,8")
        .expect("no-guard ProV1 spirit replay root should exist");
    let score_leader_index = attribution_root_index(scored_roots.as_slice(), "l6,0;l7,0")
        .expect("score-leading mana sibling should exist");

    println!(
        "BLACK_RECOVERY_BRANCH_REPLY_FLOOR_ATTRIBUTION context={} frontier_selected={} shipping_selected={} shortlist={:?} roots={:?} comparisons={:?}",
        exact_opportunity_context_probe(&game),
        frontier_probe.selected_input_fen,
        shipping_probe.selected_input_fen,
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        [
            ("frontier", frontier_index),
            ("shipping", shipping_index),
            ("pro_v1_candidate", pro_v1_candidate_index),
            ("score_leader", score_leader_index),
        ]
        .iter()
        .map(|(label, index)| {
            format!(
                "{}={}",
                label,
                root_attribution_details(&game, scored_roots.as_slice(), *index, perspective, config)
            )
        })
        .collect::<Vec<_>>(),
        [
            ("frontier", frontier_index, "shipping", shipping_index),
            (
                "pro_v1_candidate",
                pro_v1_candidate_index,
                "shipping",
                shipping_index,
            ),
            ("shipping", shipping_index, "score_leader", score_leader_index),
        ]
        .iter()
        .map(|(left_label, left_index, right_label, right_index)| {
            pair_attribution(
                left_label,
                &scored_roots[*left_index],
                right_label,
                &scored_roots[*right_index],
                perspective,
                config,
            )
        })
        .collect::<Vec<_>>(),
    );
}

#[test]
#[ignore = "diagnostic: attribute black progress-vs-setup residual board-state scoring and selector outcome"]
fn black_progress_residual_weight_attribution_probe() {
    use crate::models::scoring::{evaluate_preferability_breakdown_with_weights, ScoringWeights};

    #[derive(Clone, Copy)]
    struct AttributionCase {
        label: &'static str,
        board_fen: &'static str,
        safe_progress_root: &'static str,
        setup_roots: &'static [&'static str],
    }

    struct WorstReply {
        input_fen: String,
        score: i32,
        events: String,
        game: MonsGame,
    }

    fn root_index(scored_roots: &[RootEvaluation], root_fen: &str) -> Option<usize> {
        scored_roots
            .iter()
            .position(|root| Input::fen_from_array(&root.inputs) == root_fen)
    }

    fn zeroed_like(base: &ScoringWeights) -> ScoringWeights {
        ScoringWeights {
            use_legacy_formula: base.use_legacy_formula,
            include_regular_mana_move_windows: base.include_regular_mana_move_windows,
            include_match_point_window: base.include_match_point_window,
            next_turn_window_scale_bp: base.next_turn_window_scale_bp,
            double_confirmed_score: false,
            confirmed_score: 0,
            fainted_mon: 0,
            fainted_drainer: 0,
            fainted_cooldown_step: 0,
            drainer_at_risk: 0,
            mana_close_to_same_pool: 0,
            mon_with_mana_close_to_any_pool: 0,
            extra_for_supermana: 0,
            extra_for_opponents_mana: 0,
            drainer_close_to_mana: 0,
            drainer_holding_mana: 0,
            drainer_close_to_own_pool: 0,
            drainer_close_to_supermana: 0,
            mon_close_to_center: 0,
            spirit_close_to_enemy: 0,
            spirit_on_own_base_penalty: 0,
            angel_guarding_drainer: 0,
            angel_close_to_friendly_drainer: 0,
            has_consumable: 0,
            active_mon: 0,
            regular_mana_to_owner_pool: 0,
            regular_mana_drainer_control: 0,
            supermana_drainer_control: 0,
            supermana_race_control: 0,
            opponent_mana_denial: 0,
            mana_carrier_at_risk: 0,
            mana_carrier_guarded: 0,
            mana_carrier_one_step_from_pool: 0,
            supermana_carrier_one_step_from_pool_extra: 0,
            immediate_winning_carrier: 0,
            drainer_best_mana_path: 0,
            drainer_pickup_score_this_turn: 0,
            mana_carrier_score_this_turn: 0,
            drainer_immediate_threat: 0,
            score_race_path_progress: 0,
            opponent_score_race_path_progress: 0,
            score_race_multi_path: 0,
            opponent_score_race_multi_path: 0,
            immediate_score_window: 0,
            opponent_immediate_score_window: 0,
            immediate_score_multi_window: 0,
            opponent_immediate_score_multi_window: 0,
            spirit_action_utility: 0,
            drainer_danger_boolean: 0,
            mana_carrier_danger_boolean: 0,
            drainer_walk_threat_boolean: 0,
            mana_carrier_walk_threat_boolean: 0,
            opponent_drainer_attack_bonus: 0,
            attacker_close_to_opponent_drainer: 0,
        }
    }

    fn residual_score(game: &MonsGame, perspective: Color, weights: &ScoringWeights) -> i32 {
        evaluate_preferability_breakdown_with_weights(game, perspective, weights)
            .terms
            .residual_board_state
    }

    fn residual_field_scores(
        game: &MonsGame,
        perspective: Color,
        base: &ScoringWeights,
    ) -> Vec<(&'static str, i32)> {
        let mut scores = Vec::new();
        macro_rules! add_field {
            ($field:ident) => {{
                let mut weights = zeroed_like(base);
                weights.$field = base.$field;
                scores.push((
                    stringify!($field),
                    residual_score(game, perspective, &weights),
                ));
            }};
        }

        add_field!(confirmed_score);
        add_field!(fainted_mon);
        add_field!(fainted_drainer);
        add_field!(fainted_cooldown_step);
        add_field!(drainer_at_risk);
        add_field!(mana_close_to_same_pool);
        add_field!(mon_with_mana_close_to_any_pool);
        add_field!(extra_for_supermana);
        add_field!(extra_for_opponents_mana);
        add_field!(drainer_close_to_mana);
        add_field!(drainer_holding_mana);
        add_field!(drainer_close_to_own_pool);
        add_field!(drainer_close_to_supermana);
        add_field!(mon_close_to_center);
        add_field!(spirit_close_to_enemy);
        add_field!(spirit_on_own_base_penalty);
        add_field!(angel_guarding_drainer);
        add_field!(angel_close_to_friendly_drainer);
        add_field!(has_consumable);
        add_field!(active_mon);
        add_field!(regular_mana_to_owner_pool);
        add_field!(regular_mana_drainer_control);
        add_field!(supermana_drainer_control);
        add_field!(supermana_race_control);
        add_field!(opponent_mana_denial);
        add_field!(mana_carrier_at_risk);
        add_field!(mana_carrier_guarded);
        add_field!(mana_carrier_one_step_from_pool);
        add_field!(supermana_carrier_one_step_from_pool_extra);
        add_field!(immediate_winning_carrier);
        add_field!(drainer_best_mana_path);
        add_field!(drainer_pickup_score_this_turn);
        add_field!(mana_carrier_score_this_turn);
        add_field!(drainer_immediate_threat);
        add_field!(score_race_path_progress);
        add_field!(opponent_score_race_path_progress);
        add_field!(score_race_multi_path);
        add_field!(opponent_score_race_multi_path);
        add_field!(immediate_score_window);
        add_field!(opponent_immediate_score_window);
        add_field!(immediate_score_multi_window);
        add_field!(opponent_immediate_score_multi_window);
        add_field!(spirit_action_utility);
        add_field!(drainer_danger_boolean);
        add_field!(mana_carrier_danger_boolean);
        add_field!(drainer_walk_threat_boolean);
        add_field!(mana_carrier_walk_threat_boolean);
        add_field!(opponent_drainer_attack_bonus);
        add_field!(attacker_close_to_opponent_drainer);

        scores
    }

    fn top_residual_deltas(
        left_label: &str,
        left_game: &MonsGame,
        right_label: &str,
        right_game: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> String {
        let left_breakdown = evaluate_preferability_breakdown_with_weights(
            left_game,
            perspective,
            config.scoring_weights,
        );
        let right_breakdown = evaluate_preferability_breakdown_with_weights(
            right_game,
            perspective,
            config.scoring_weights,
        );
        let left_scores = residual_field_scores(left_game, perspective, config.scoring_weights);
        let right_scores = residual_field_scores(right_game, perspective, config.scoring_weights);
        let mut deltas = left_scores
            .into_iter()
            .zip(right_scores)
            .map(|((left_name, left_score), (right_name, right_score))| {
                assert_eq!(left_name, right_name);
                (left_name, left_score - right_score, left_score, right_score)
            })
            .collect::<Vec<_>>();
        let field_sum_delta = deltas.iter().map(|(_, delta, _, _)| *delta).sum::<i32>();
        deltas.sort_by(|left, right| {
            right
                .1
                .abs()
                .cmp(&left.1.abs())
                .then_with(|| left.0.cmp(right.0))
        });
        let top = deltas
            .iter()
            .filter(|(_, delta, _, _)| *delta != 0)
            .take(14)
            .map(|(name, delta, left_score, right_score)| {
                format!("{name}:{delta}({left_score}-{right_score})")
            })
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{left_label}_minus_{right_label} total_delta={} residual_delta={} field_sum_delta={} left_terms={:?} right_terms={:?} left_features={:?} right_features={:?} top_residual_fields=[{}]",
            left_breakdown.total - right_breakdown.total,
            left_breakdown.terms.residual_board_state - right_breakdown.terms.residual_board_state,
            field_sum_delta,
            left_breakdown.terms,
            right_breakdown.terms,
            left_breakdown.features,
            right_breakdown.features,
            top,
        )
    }

    fn worst_reply_state(
        state_after_move: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> Option<WorstReply> {
        let reply_limit = config.root_reply_risk_reply_limit.clamp(1, 24);
        let replies = MonsGameModel::enumerate_legal_transitions(
            state_after_move,
            reply_limit,
            MonsGameModel::automove_start_input_options(config),
        );
        replies
            .into_iter()
            .map(|reply| {
                let score = match reply.game.winner_color() {
                    Some(winner) if winner == perspective => SMART_TERMINAL_SCORE / 2,
                    Some(_) => -SMART_TERMINAL_SCORE / 2,
                    None => MonsGameModel::evaluate_search_preferability(
                        &reply.game,
                        perspective,
                        config,
                    ),
                };
                (score, Input::fen_from_array(&reply.inputs), reply)
            })
            .min_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)))
            .map(|(score, input_fen, reply)| WorstReply {
                input_fen,
                score,
                events: format!("{:?}", reply.events),
                game: reply.game,
            })
    }

    fn pair_attribution(
        safe_root: &RootEvaluation,
        setup_root: &RootEvaluation,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> String {
        let safe_fen = Input::fen_from_array(&safe_root.inputs);
        let setup_fen = Input::fen_from_array(&setup_root.inputs);
        let after_root = top_residual_deltas(
            "safe_after_root",
            &safe_root.game,
            "setup_after_root",
            &setup_root.game,
            perspective,
            config,
        );
        let worst_reply = match (
            worst_reply_state(&safe_root.game, perspective, config),
            worst_reply_state(&setup_root.game, perspective, config),
        ) {
            (Some(safe_reply), Some(setup_reply)) => format!(
                "safe_worst_reply={} safe_worst_score={} safe_worst_events={} setup_worst_reply={} setup_worst_score={} setup_worst_events={} {}",
                safe_reply.input_fen,
                safe_reply.score,
                safe_reply.events,
                setup_reply.input_fen,
                setup_reply.score,
                setup_reply.events,
                top_residual_deltas(
                    "safe_worst_reply",
                    &safe_reply.game,
                    "setup_worst_reply",
                    &setup_reply.game,
                    perspective,
                    config,
                ),
            ),
            (None, None) => "worst_reply=no_replies".to_string(),
            (None, Some(setup_reply)) => format!(
                "safe_worst_reply=none setup_worst_reply={} setup_worst_score={} setup_worst_events={}",
                setup_reply.input_fen, setup_reply.score, setup_reply.events,
            ),
            (Some(safe_reply), None) => format!(
                "safe_worst_reply={} safe_worst_score={} safe_worst_events={} setup_worst_reply=none",
                safe_reply.input_fen, safe_reply.score, safe_reply.events,
            ),
        };

        format!(
            "safe_root={} safe_probe={} setup_root={} setup_probe={} after_root={{ {} }} worst_reply={{ {} }}",
            safe_fen,
            format_root_probe(Some(safe_root)),
            setup_fen,
            format_root_probe(Some(setup_root)),
            after_root,
            worst_reply,
        )
    }

    fn material_dampened_selection_probe(
        game: &MonsGame,
        setup_root_fens: &[&'static str],
    ) -> String {
        static MATERIAL_DAMPENED_WEIGHTS: std::sync::OnceLock<ScoringWeights> =
            std::sync::OnceLock::new();

        let selector = profile_selector_from_name("frontier_pro_v2_guarded")
            .expect("frontier profile selector should exist");
        let mut config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            game,
            SmartAutomovePreference::Pro,
        );
        config.scoring_weights = MATERIAL_DAMPENED_WEIGHTS.get_or_init(|| {
            let mut weights = *config.scoring_weights;
            weights.fainted_mon = 0;
            weights.fainted_cooldown_step = 0;
            weights
        });

        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();
        clear_frontier_runtime_variant_branch();

        let selected = select_inputs_with_runtime_fallback(selector, game, config);
        let selected_input_fen = Input::fen_from_array(&selected);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let runtime_variant_branch = frontier_runtime_variant_branch_snapshot();
        let advisor = pro_v2_root_advisor_decision_snapshot();
        let approved = advisor
            .as_ref()
            .and_then(|decision| decision.approved_root.as_ref())
            .map(format_root_advisor_entry_probe)
            .unwrap_or_else(|| "none".to_string());
        let shortlist = advisor
            .as_ref()
            .map(|decision| {
                decision
                    .ordered_shortlist
                    .iter()
                    .take(8)
                    .map(format_root_advisor_entry_probe)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let preserved = advisor
            .as_ref()
            .map(|decision| {
                decision
                    .preserved_family_representatives
                    .iter()
                    .map(format_root_advisor_entry_probe)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let (dampened_config, dampened_roots, _, _) =
            runtime_scored_roots_with_config(game, config);
        let approved_index = root_index(dampened_roots.as_slice(), selected_input_fen.as_str());
        let candidate_gate = approved_index
            .map(|approved_index| {
                material_dampened_setup_gate_probe(
                    game,
                    dampened_config,
                    dampened_roots.as_slice(),
                    approved_index,
                    setup_root_fens,
                )
            })
            .unwrap_or_else(|| "setup_gate=approved_root_missing".to_string());

        format!(
            "material_dampened_selected={} branch={} selector(stage={} top_stage={} disable={} top_disable={} head_calls={} head_hits={}) approved={} shortlist={:?} preserved={:?} {}",
            selected_input_fen,
            runtime_variant_branch,
            selector_diag.last_return_stage,
            selector_diag.top_level_last_return_stage,
            selector_diag.selector_disable_reason,
            selector_diag.top_level_selector_disable_reason,
            selector_diag.head_plan_calls,
            selector_diag.head_plan_hits,
            approved,
            shortlist,
            preserved,
            candidate_gate,
        )
    }

    fn material_dampened_setup_gate_probe(
        game: &MonsGame,
        mut config: AutomoveSearchConfig,
        scored_roots: &[RootEvaluation],
        approved_index: usize,
        setup_root_fens: &[&'static str],
    ) -> String {
        static MATERIAL_DAMPENED_WEIGHTS: std::sync::OnceLock<ScoringWeights> =
            std::sync::OnceLock::new();

        config.scoring_weights = MATERIAL_DAMPENED_WEIGHTS.get_or_init(|| {
            let mut weights = *config.scoring_weights;
            weights.fainted_mon = 0;
            weights.fainted_cooldown_step = 0;
            weights
        });

        let Some(approved) = scored_roots.get(approved_index) else {
            return "setup_gate=approved_index_out_of_range".to_string();
        };
        let approved_family = MonsGameModel::turn_engine_root_evaluation_family(approved);
        let approved_utility = MonsGameModel::turn_engine_selected_override_utility(
            game,
            approved,
            game.active_color,
            config,
            approved_family,
        );
        let reply_limit = config.root_reply_risk_reply_limit.clamp(1, 24);
        let approved_snapshot = MonsGameModel::root_reply_risk_snapshot(
            &approved.game,
            game.active_color,
            config,
            reply_limit,
        );
        let approved_followup = MonsGameModel::pro_v2_spirit_followup_floor_score(
            &approved.game,
            game.active_color,
            config,
        );
        let exact_context =
            crate::models::automove_exact::exact_opportunity_context(game, game.active_color);
        let weak_window_context = exact_context.delta.same_turn_score_window_value <= 1
            && exact_context.delta.opponent_window_deny_gain <= 1
            && !exact_context.delta.drainer_attack_available
            && (exact_context.delta.same_turn_score_window_value > 0
                || exact_context.delta.opponent_window_deny_gain > 0);

        let entries = setup_root_fens
            .iter()
            .map(|setup_root_fen| {
                let Some(index) = root_index(scored_roots, setup_root_fen) else {
                    return format!("{setup_root_fen}:missing");
                };
                let challenger = &scored_roots[index];
                let challenger_family =
                    MonsGameModel::turn_engine_root_evaluation_family(challenger);
                let challenger_utility = MonsGameModel::turn_engine_selected_override_utility(
                    game,
                    challenger,
                    game.active_color,
                    config,
                    challenger_family,
                );
                let challenger_snapshot = MonsGameModel::root_reply_risk_snapshot(
                    &challenger.game,
                    game.active_color,
                    config,
                    reply_limit,
                );
                let challenger_followup = MonsGameModel::pro_v2_spirit_followup_floor_score(
                    &challenger.game,
                    game.active_color,
                    config,
                );
                let utility_order =
                    crate::models::automove_turn_engine::compare_utility_primary_axes(
                        challenger_utility,
                        approved_utility,
                    );
                let utility_competes = MonsGameModel::pro_v2_root_advisor_utility_competes(
                    challenger_utility,
                    approved_utility,
                );
                let score_gap = approved.score.saturating_sub(challenger.score);
                let setup_gain_gap =
                    challenger.spirit_setup_gain.saturating_sub(approved.spirit_setup_gain);
                let progress_surface =
                    MonsGameModel::turn_engine_root_evaluation_has_progress_surface(challenger);
                let safe = MonsGameModel::pro_v2_root_advisor_root_evaluation_is_safe(challenger);
                let base_shape = challenger_family == TurnPlanFamily::SpiritImpact
                    && challenger.spirit_own_mana_setup_now
                    && !challenger.spirit_same_turn_score_setup_now
                    && progress_surface
                    && !challenger.wins_immediately
                    && !challenger.attacks_opponent_drainer
                    && challenger.same_turn_score_window_value == 0
                    && !challenger.scores_supermana_this_turn
                    && !challenger.scores_opponent_mana_this_turn
                    && !challenger.safe_supermana_pickup_now
                    && !challenger.safe_opponent_mana_pickup_now
                    && challenger.mana_handoff_to_opponent == approved.mana_handoff_to_opponent
                    && challenger.has_roundtrip == approved.has_roundtrip
                    && safe
                    && challenger.own_drainer_vulnerable == approved.own_drainer_vulnerable
                    && challenger.own_drainer_walk_vulnerable
                        == approved.own_drainer_walk_vulnerable
                    && challenger.supermana_progress == approved.supermana_progress
                    && challenger.opponent_mana_progress == approved.opponent_mana_progress;
                let current_rank_gate =
                    challenger.root_rank <= approved.root_rank.saturating_add(4);
                let higher_scoring_rank_candidate = challenger.score >= approved.score
                    && challenger.root_rank <= approved.root_rank.saturating_add(12);
                let current_passes_override = base_shape
                    && utility_competes
                    && score_gap <= 32
                    && setup_gain_gap >= 64
                    && current_rank_gate;

                format!(
                    "{} idx={} rank={} score={} score_gap={} setup_gain={} setup_gap={} family={:?} base_shape={} utility_order={} utility_competes={} score_gate={} setup_gate={} current_rank_gate={} higher_scoring_rank_candidate={} current_passes_override={} reply(worst={} imm_loss={} match_point={}) followup={} followup_gap={} utility={}",
                    setup_root_fen,
                    index,
                    challenger.root_rank,
                    challenger.score,
                    score_gap,
                    challenger.spirit_setup_gain,
                    setup_gain_gap,
                    challenger_family,
                    base_shape,
                    format_ordering_probe(utility_order),
                    utility_competes,
                    score_gap <= 32,
                    setup_gain_gap >= 64,
                    current_rank_gate,
                    higher_scoring_rank_candidate,
                    current_passes_override,
                    challenger_snapshot.worst_reply_score,
                    challenger_snapshot.allows_immediate_opponent_win,
                    challenger_snapshot.opponent_reaches_match_point,
                    challenger_followup,
                    challenger_followup - approved_followup,
                    format_turn_engine_utility_probe(challenger_utility),
                )
            })
            .collect::<Vec<_>>();

        format!(
            "setup_gate approved(rank={} score={} family={:?} weak_window={} reply(worst={} imm_loss={} match_point={}) followup={} utility={}) candidates={:?}",
            approved.root_rank,
            approved.score,
            approved_family,
            weak_window_context,
            approved_snapshot.worst_reply_score,
            approved_snapshot.allows_immediate_opponent_win,
            approved_snapshot.opponent_reaches_match_point,
            approved_followup,
            format_turn_engine_utility_probe(approved_utility),
            entries,
        )
    }

    fn surface_for_case(case: AttributionCase) -> String {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{} should have a valid fen", case.label));
        let perspective = game.active_color;
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
        let (config, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let Some(safe_index) = root_index(scored_roots.as_slice(), case.safe_progress_root) else {
            return format!(
                "label={} context={} frontier_selected={} shipping_selected={} safe_root={} missing",
                case.label,
                exact_opportunity_context_probe(&game),
                frontier_probe.selected_input_fen,
                shipping_probe.selected_input_fen,
                case.safe_progress_root,
            );
        };
        let safe_root = &scored_roots[safe_index];
        let material_dampened_selector = if case.label == "black_progress_vs_setup_residue" {
            material_dampened_selection_probe(&game, case.setup_roots)
        } else {
            "material_dampened=out_of_scope".to_string()
        };

        format!(
            "label={} context={} frontier_selected={} shipping_selected={} {} comparisons={:?}",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selected_input_fen,
            shipping_probe.selected_input_fen,
            material_dampened_selector,
            case.setup_roots
                .iter()
                .map(|setup_root_fen| {
                    let Some(setup_index) = root_index(scored_roots.as_slice(), setup_root_fen)
                    else {
                        return format!("setup_root={} missing", setup_root_fen);
                    };
                    pair_attribution(safe_root, &scored_roots[setup_index], perspective, config)
                })
                .collect::<Vec<_>>(),
        )
    }

    let cases = [
        AttributionCase {
            label: "black_progress_vs_setup_residue",
            board_fen: "1 0 b 0 0 0 0 0 6 n05d0xn05/n05s0xa0xe0xn03/n02xxmn04xxmn03/n07xxmn03/n03xxmn01xxmn05/n05xxUn04xxQ/n05xxMn05/n01y0xn01S0xE0xn01xxMxxMn03/n01xxMn09/n03A0xn03Y0xn03/n05D1xn05",
            safe_progress_root: "l7,1;l9,3",
            setup_roots: &["l1,5;l2,7;l1,8", "l1,5;l3,7;l2,8"],
        },
        AttributionCase {
            label: "black_confirm_fast_setup_control",
            board_fen: "2 1 b 0 0 0 0 0 10 n05d0xn03xxmn01/n04a0xn02e0xn03/n05s0xn05/E0xn02xxmn03xxmn03/n05xxmn05/n05xxUn04xxQ/n05xxMn05/n03S0xn01Y0xxxMn04/n03y0xn04xxMn02/n03A0xn07/n05D1xn05",
            safe_progress_root: "l0,5;l1,5",
            setup_roots: &["l2,5;l3,7;l2,8"],
        },
    ];

    for case in cases {
        println!(
            "BLACK_PROGRESS_RESIDUAL_WEIGHT_ATTRIBUTION {}",
            surface_for_case(case)
        );
    }
}

#[test]
#[ignore = "diagnostic: replay exact pro-reliability duel seeds and log frontier non-win openings"]
fn smart_automove_pro_reliability_nonwin_trace_probe() {
    let frontier_profile = reliability_frontier_profile_id();
    let shipping_profile = reliability_shipping_profile_id();
    let repeats = env_usize("SMART_PRO_RELIABILITY_REPEATS")
        .unwrap_or(3)
        .max(1);
    let games = env_usize("SMART_PRO_RELIABILITY_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_RELIABILITY_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let trace_limit = env_usize("SMART_PRO_RELIABILITY_TRACE_LIMIT")
        .unwrap_or(12)
        .max(1);
    let seed_tag = env_string_value("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());
    let duel_filter = env::var("SMART_PRO_RELIABILITY_DUEL_FILTER").ok();
    let duel_specs = [
        (
            "vs_shipping_pro",
            SmartAutomovePreference::Pro,
            seed_tag.clone(),
        ),
        (
            "vs_shipping_normal",
            SmartAutomovePreference::Normal,
            format!("{}_vs_normal", seed_tag),
        ),
        (
            "vs_shipping_fast",
            SmartAutomovePreference::Fast,
            format!("{}_vs_fast", seed_tag),
        ),
    ];

    println!(
        "pro reliability non-win trace probe: frontier={} shipping={} repeats={} games_per_repeat={} max_plies={} trace_limit={} duel_filter={:?}",
        frontier_profile,
        shipping_profile,
        repeats,
        games,
        max_plies,
        trace_limit,
        duel_filter,
    );

    for (duel_label, opponent_mode, duel_seed_tag) in duel_specs {
        if duel_filter
            .as_deref()
            .is_some_and(|filter| filter != duel_label)
        {
            continue;
        }

        let opponent_budget = SearchBudget::from_preference(opponent_mode);
        let mut nonwins = 0usize;
        let mut printed = 0usize;

        for repeat_index in 0..repeats {
            let seed = seed_for_budget_duel_repeat_and_tag(
                pro_budget(),
                opponent_budget,
                repeat_index,
                duel_seed_tag.as_str(),
            );
            let opening_fens = generate_opening_fens_cached(seed, games);
            for (game_index, opening_fen) in opening_fens.iter().enumerate() {
                for frontier_is_white in [true, false] {
                    let frontier_trace = play_profile_duel_trace(
                        frontier_profile.as_str(),
                        shipping_profile.as_str(),
                        opponent_mode,
                        opening_fen.as_str(),
                        frontier_is_white,
                        max_plies,
                    );
                    if !matches!(frontier_trace.result, MatchResult::ProfileAWin) {
                        nonwins += 1;
                        if printed < trace_limit {
                            let shipping_trace = play_profile_duel_trace(
                                shipping_profile.as_str(),
                                shipping_profile.as_str(),
                                opponent_mode,
                                opening_fen.as_str(),
                                frontier_is_white,
                                max_plies,
                            );
                            let detail = first_duel_trace_divergence(
                                    &frontier_trace,
                                    &shipping_trace,
                                )
                                .map(|divergence| {
                                    let board = MonsGame::from_fen(
                                        divergence.board_fen.as_str(),
                                        false,
                                    )
                                    .expect("trace board fen should be valid");
                                    let frontier_probe = runtime_decision_probe(
                                        frontier_profile.as_str(),
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    let shipping_probe = runtime_decision_probe(
                                        shipping_profile.as_str(),
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    format!(
                                        "first_diff_ply={} board={} frontier_move={} shipping_move={} frontier(selected={} pre_accept={} stage={} head={:?} accepted={} top={:?}) shipping(selected={} pre_accept={} stage={} head={:?} accepted={} top={:?})",
                                        divergence.ply,
                                        divergence.board_fen,
                                        divergence.profile_a_move_fen,
                                        divergence.profile_b_move_fen,
                                        frontier_probe.selected_input_fen,
                                        frontier_probe.pre_accept_input_fen,
                                        frontier_probe.selector_last_stage,
                                        frontier_probe.head_input_fen,
                                        frontier_probe.head_accepted,
                                        frontier_probe.top_root_fens,
                                        shipping_probe.selected_input_fen,
                                        shipping_probe.pre_accept_input_fen,
                                        shipping_probe.selector_last_stage,
                                        shipping_probe.head_input_fen,
                                        shipping_probe.head_accepted,
                                        shipping_probe.top_root_fens,
                                    )
                                })
                                .unwrap_or_else(|| "first_diff=none".to_string());

                            println!(
                                    "PRO_RELIABILITY_NONWIN duel={} repeat={} opening_index={} frontier_is_white={} opening={} frontier_result={} shipping_result={} frontier_final={} shipping_final={} {}",
                                    duel_label,
                                    repeat_index,
                                    game_index,
                                    frontier_is_white,
                                    opening_fen,
                                    format_match_result(frontier_trace.result),
                                    format_match_result(shipping_trace.result),
                                    frontier_trace.final_fen,
                                    shipping_trace.final_fen,
                                    detail,
                                );
                            printed += 1;
                        }
                    }
                }
            }
        }

        println!(
            "PRO_RELIABILITY_NONWIN_SUMMARY duel={} total_nonwins={} trace_limit={}",
            duel_label, nonwins, trace_limit,
        );
    }
}

#[test]
#[ignore = "diagnostic: bounded selector/exact hotspot probe for pro reliability corpus"]
fn smart_automove_pro_reliability_hotspot_probe() {
    use std::collections::BTreeMap;
    use std::time::Instant;

    #[derive(Clone)]
    struct ProbeCase {
        label: String,
        game: MonsGame,
        mode: SmartAutomovePreference,
        config_tweak: Option<fn(AutomoveSearchConfig) -> AutomoveSearchConfig>,
    }

    fn probe_case_from_fixture(label: &'static str, fixture: TriageFixture) -> ProbeCase {
        ProbeCase {
            label: label.to_string(),
            game: fixture.game,
            mode: fixture.mode,
            config_tweak: fixture.config_tweak,
        }
    }

    fn hotspot_mode_from_env() -> SmartAutomovePreference {
        env::var("SMART_PRO_RELIABILITY_HOTSPOT_MODE")
            .ok()
            .and_then(|value| {
                SmartAutomovePreference::from_api_value(value.trim().to_lowercase().as_str())
            })
            .unwrap_or(SmartAutomovePreference::Pro)
    }

    fn hotspot_case_from_env() -> Option<ProbeCase> {
        let fen = env::var("SMART_PRO_RELIABILITY_HOTSPOT_FEN").ok()?;
        let label = env::var("SMART_PRO_RELIABILITY_HOTSPOT_LABEL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "env_hotspot".to_string());

        Some(ProbeCase {
            label,
            game: MonsGame::from_fen(fen.trim(), false)
                .expect("SMART_PRO_RELIABILITY_HOTSPOT_FEN should be a valid MonsGame FEN"),
            mode: hotspot_mode_from_env(),
            config_tweak: None,
        })
    }

    fn game_with_items(items: Vec<(Location, Item)>, active_color: Color) -> MonsGame {
        let mut game = MonsGame::new(false, GameVariant::Classic);
        game.replace_board_items(items);
        game.active_color = active_color;
        game.turn_number = 2;
        game.actions_used_count = 0;
        game.mana_moves_count = 0;
        game.mons_moves_count = 0;
        game.white_score = 0;
        game.black_score = 0;
        game.white_potions_count = 0;
        game.black_potions_count = 0;
        game
    }

    #[derive(Clone)]
    struct ProbeResult {
        move_fen: String,
        elapsed_ms: f64,
        selector_diag: TurnEngineSelectorDiagnostics,
        exact_diag: ExactQueryDiagnostics,
        engine_diag: TurnEngineDiagnostics,
    }

    fn run_probe_for_profile(profile_name: &str, case: &ProbeCase) -> ProbeResult {
        let selector = profile_selector_from_name(profile_name)
            .unwrap_or_else(|| panic!("profile '{}' not found", profile_name));
        let base_config = case
            .config_tweak
            .map(|tweak| {
                tweak(SearchBudget::from_preference(case.mode).runtime_config_for_game(&case.game))
            })
            .unwrap_or_else(|| {
                SearchBudget::from_preference(case.mode).runtime_config_for_game(&case.game)
            });
        let config = profile_runtime_config_for_name(profile_name, &case.game, base_config)
            .unwrap_or(base_config);

        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let start = Instant::now();
        let inputs = select_inputs_with_runtime_fallback(selector, &case.game, config);
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        assert!(
            !inputs.is_empty(),
            "hotspot probe profile '{}' produced no legal move for '{}'",
            profile_name,
            case.label
        );

        ProbeResult {
            move_fen: Input::fen_from_array(&inputs),
            elapsed_ms,
            selector_diag: turn_engine_selector_diagnostics_snapshot(),
            exact_diag: exact_query_diagnostics_snapshot(),
            engine_diag: turn_engine_diagnostics_snapshot(),
        }
    }

    fn selector_metrics(result: &ProbeResult) -> BTreeMap<&'static str, u64> {
        BTreeMap::from([
            (
                "child_calls",
                result.selector_diag.ranked_child_states_calls as u64,
            ),
            (
                "children",
                result.selector_diag.ranked_child_states_children_enumerated as u64,
            ),
            (
                "fully_scored",
                result
                    .selector_diag
                    .ranked_child_states_children_fully_scored as u64,
            ),
            (
                "shortlist",
                result.selector_diag.child_ordering_shortlist_children as u64,
            ),
            (
                "full_pass",
                result.selector_diag.child_ordering_full_pass_children as u64,
            ),
            (
                "move_eff_builds",
                result.selector_diag.move_efficiency_snapshot_builds as u64,
            ),
            (
                "move_eff_hits",
                result.selector_diag.move_efficiency_snapshot_cache_hits as u64,
            ),
            (
                "prefer_builds",
                result.selector_diag.search_preferability_builds as u64,
            ),
            (
                "prefer_hits",
                result.selector_diag.search_preferability_cache_hits as u64,
            ),
            ("head_calls", result.selector_diag.head_plan_calls as u64),
            ("head_hits", result.selector_diag.head_plan_hits as u64),
        ])
    }

    fn exact_metrics(result: &ProbeResult) -> BTreeMap<&'static str, u64> {
        BTreeMap::from([
            (
                "attack_summary_builds",
                result.exact_diag.attack_reach_summary_builds as u64,
            ),
            ("attack_calls", result.exact_diag.attack_reach_calls as u64),
            (
                "attack_hits",
                result.exact_diag.attack_reach_cache_hits as u64,
            ),
            (
                "threat_calls",
                result.exact_diag.drainer_immediate_threat_calls as u64,
            ),
            (
                "payload_calls",
                result.exact_diag.actor_payload_after_move_calls as u64,
            ),
            (
                "tactical_spirit_calls",
                result.exact_diag.tactical_spirit_summary_calls as u64,
            ),
            (
                "tactical_spirit_hits",
                result.exact_diag.tactical_spirit_summary_cache_hits as u64,
            ),
            (
                "immediate_window_queries",
                result.exact_diag.immediate_tactical_window_queries as u64,
            ),
            (
                "tactical_window_calls",
                result.exact_diag.tactical_spirit_after_window_calls as u64,
            ),
            (
                "secure_mana_calls",
                result.exact_diag.exact_secure_mana_calls as u64,
            ),
            (
                "secure_mana_hits",
                result.exact_diag.exact_secure_mana_cache_hits as u64,
            ),
            ("pickup_calls", result.exact_diag.pickup_path_calls as u64),
            (
                "pickup_hits",
                result.exact_diag.pickup_path_cache_hits as u64,
            ),
        ])
    }

    fn engine_metrics(result: &ProbeResult) -> BTreeMap<&'static str, u64> {
        BTreeMap::from([
            ("cache_hits", result.engine_diag.cache_hits as u64),
            ("cache_misses", result.engine_diag.cache_misses as u64),
            ("accepted", result.engine_diag.accepted_plans as u64),
            ("reply_calls", result.engine_diag.reply_search_calls as u64),
        ])
    }

    fn format_metric_delta(
        frontier: &BTreeMap<&'static str, u64>,
        shipping: &BTreeMap<&'static str, u64>,
    ) -> String {
        frontier
            .iter()
            .map(|(label, frontier_value)| {
                let shipping_value = shipping.get(label).copied().unwrap_or_default();
                let delta = *frontier_value as i64 - shipping_value as i64;
                format!("{label}={frontier_value}/{shipping_value}({delta:+})")
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    let frontier_profile = probe_frontier_profile_id();
    let shipping_profile = probe_shipping_profile_id();

    let mut cases = vec![
        probe_case_from_fixture(
            "primary_black_loss_opening_a_ply19",
            primary_pro_fixture_by_id("primary_black_loss_opening_a_ply19"),
        ),
        probe_case_from_fixture("human_win_pro_a", primary_pro_fixture_by_id("human_win_pro_a")),
        ProbeCase {
            label: "loss_opening_a".to_string(),
            game: MonsGame::from_fen(
                "0 0 w 0 0 1 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n02E0xn01A0xD0xS0xY0xn03",
                false,
            )
            .expect("loss opening a fen should be valid"),
            mode: SmartAutomovePreference::Pro,
            config_tweak: None,
        },
        ProbeCase {
            label: "loss_opening_b".to_string(),
            game: MonsGame::from_fen(
                "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03",
                false,
            )
            .expect("loss opening b fen should be valid"),
            mode: SmartAutomovePreference::Pro,
            config_tweak: None,
        },
        ProbeCase {
            label: "quiet_positional".to_string(),
            game: game_with_items(
                vec![
                    (
                        Location::new(10, 5),
                        Item::Mon {
                            mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        },
                    ),
                    (
                        Location::new(8, 5),
                        Item::Mon {
                            mon: Mon::new(MonKind::Spirit, Color::White, 0),
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
            ),
            mode: SmartAutomovePreference::Pro,
            config_tweak: None,
        },
    ];
    if let Some(case) = hotspot_case_from_env() {
        cases.push(case);
    }

    println!(
        "pro reliability hotspot probe: frontier={} shipping={} positions={}",
        frontier_profile,
        shipping_profile,
        cases.len()
    );
    for case in cases {
        let frontier = run_probe_for_profile(frontier_profile.as_str(), &case);
        let shipping = run_probe_for_profile(shipping_profile.as_str(), &case);

        println!(
            "HOTSPOT label={} changed={} frontier_move={} shipping_move={} ms={:.2}/{:.2} selector(last_stage={}/{}) exact={} engine={} fen={}",
            case.label,
            frontier.move_fen != shipping.move_fen,
            frontier.move_fen,
            shipping.move_fen,
            frontier.elapsed_ms,
            shipping.elapsed_ms,
            frontier.selector_diag.last_return_stage,
            shipping.selector_diag.last_return_stage,
            format_metric_delta(&exact_metrics(&frontier), &exact_metrics(&shipping)),
            format_metric_delta(&engine_metrics(&frontier), &engine_metrics(&shipping)),
            case.game.fen(),
        );
        println!(
            "HOTSPOT_SELECTOR label={} {}",
            case.label,
            format_metric_delta(&selector_metrics(&frontier), &selector_metrics(&shipping)),
        );
    }
}

#[test]
#[ignore = "diagnostic: trace unified ProV2 root advisor decisions on retained footholds and duel boards"]
fn smart_automove_pro_root_advisor_trace_probe() {
    #[derive(Clone)]
    struct AdvisorTraceCase {
        label: &'static str,
        game: MonsGame,
        mode: SmartAutomovePreference,
        expect_selected_matches_approved: bool,
    }

    fn case_from_fixture(id: &'static str) -> AdvisorTraceCase {
        let fixture = primary_pro_fixture_by_id(id);
        AdvisorTraceCase {
            label: id,
            game: fixture.game,
            mode: fixture.mode,
            expect_selected_matches_approved: true,
        }
    }

    let cases = vec![
        case_from_fixture("human_win_pro_c"),
        case_from_fixture("primary_white_safe_progress_rerank_ply27"),
        case_from_fixture("primary_black_turn_four_action_mana_ply15"),
        case_from_fixture("primary_black_mana_bridge_ply20"),
        case_from_fixture("primary_black_spirit_bridge_ply19"),
        case_from_fixture("primary_black_negative_deny_ply4"),
        AdvisorTraceCase {
            label: "duel_trace_pro_white_opening_tail",
            game: MonsGame::from_fen(
                "0 0 w 0 0 3 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xn01S0xn04/n02E0xn01A0xn02Y0xn03",
                false,
            )
            .expect("valid pro white duel-trace fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_normal_white_safe_progress",
            game: MonsGame::from_fen(
                "1 1 w 0 0 0 0 0 7 n11/n06a0xn04/n04y0xd0xe0xn04/n02s0xn01xxmn01xxmn04/n01E0xn02xxUxxmn01xxmn03/n10xxQ/n05xxMn01xxMn03/n06xxMn04/n02xxMn08/n05S0xn01Y0xn03/D0xn03A0xn06",
                false,
            )
            .expect("valid normal white duel-trace fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_normal_black_mana",
            game: MonsGame::from_fen(
                "0 0 b 1 0 0 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmxxmn07/n05xxmn01xxmn03/E0xn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn06/n05D0xn02xxMn02/n04A0xn01S0xn04/n08Y0xn02",
                false,
            )
            .expect("valid normal black duel-trace fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_fast_black_mana",
            game: primary_pro_fixture_by_id("primary_black_turn_four_action_mana_ply15").game,
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_normal_black_plain_followup",
            game: MonsGame::from_fen(
                "0 0 b 0 0 1 0 0 4 n03y0xn01d1xa0xe0xn03/n04s0xn06/n04xxmn06/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn05/n05xxMn01xxMn03/n03xxMxxMn01xxMn01Y0xn02/n03E0xn07/n05S0xn05/n04A0xD1xn05",
                false,
            )
            .expect("valid normal black plain-followup duel-trace fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_normal_white_mana_sibling_v92",
            game: MonsGame::from_fen(
                "0 0 w 1 0 4 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMY0xn03/n05S0xn05/n04A0xD0xn05/n02E0xn08",
                false,
            )
            .expect("valid normal white mana sibling v92 duel-trace fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_fast_white_forced_prepass",
            game: MonsGame::from_fen(
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn01Y0xn03",
                false,
            )
            .expect("valid fast white duel-trace fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_fast_white_mana_sibling_v94",
            game: MonsGame::from_fen(
                "0 0 w 1 0 4 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn03Y0xn03/n03E0xn01S0xn05/n04A0xD0xn05",
                false,
            )
            .expect("valid fast white mana sibling v94 duel-trace fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_fast_black_nonwin_v1",
            game: MonsGame::from_fen(
                "1 0 b 0 0 1 0 0 4 n06a0xn04/n05s0xd0xe0xn03/n07xxmn03/n02y0xxxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n11/n02E0xA0xn01S0xn01Y0xn03/D0xn10",
                false,
            )
            .expect("valid fast black non-win v1 duel-trace fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_pro_black_shared_late_post_search_nonwin",
            game: MonsGame::from_fen(
                "1 0 b 1 0 0 0 0 8 n05d0xn05/n05s0xa0xe0xxxmn02/n11/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
                false,
            )
            .expect("valid pro black shared late post-search nonwin fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_pro_black_turn_four_followup_nonwin",
            game: MonsGame::from_fen(
                "0 0 b 1 0 1 0 0 4 n03y0xn03e0xn03/n05a0xn05/n02xxmn01s0xn02d0mn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/E0xn03xxMS0xn05/n05D0xn01Y0xn03/n04A0xn06",
                false,
            )
            .expect("valid pro black turn-four followup nonwin fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_pro_white_late_post_search_nonwin",
            game: MonsGame::from_fen(
                "2 1 w 0 0 4 0 0 7 n11/n01xxmn01y0xn03a0xd0mn02/n06s0xn01e0xn02/n04xxmn06/n05xxmn05/xxQn04xxUn04Y0B/n04xxMn02xxMn03/n05S0xxxMn04/n11/n05A0xn05/D0xn02E0xn07",
                false,
            )
            .expect("valid pro white late post-search nonwin fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_pro_white_harvest_followup_nonwin",
            game: MonsGame::from_fen(
                "0 0 w 0 0 2 0 0 3 n03y0xn03e0xn03/n05s0xa0xn01d0mn02/n11/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn01Y0xn02/n01E0xn09/n04D0xn01S0xn04/n04A0xn06",
                false,
            )
            .expect("valid pro white harvest followup nonwin fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_pro_white_late_cluster_nonwin",
            game: MonsGame::from_fen(
                "1 1 w 0 0 0 0 0 5 d0xn10/n05s0xa0xe0xn03/n03y0xn03xxmn03/n11/n04xxmxxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n05S0xn05/n04E0xA0xn05/n07Y0xn02D0x",
                false,
            )
            .expect("valid pro white late cluster nonwin fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_pro_black_turn_ten_nonwin",
            game: MonsGame::from_fen(
                "3 0 b 1 0 0 0 0 10 n09xxmn01/n05a0xn01e0xn03/n05s0xd0mn04/n02xxmxxmn07/n05xxmn02Y0xn02/n05xxUn05/y0xn04xxMn05/n03xxMn07/n04S0xn06/n02E0xn08/n04A0xn05D0x",
                false,
            )
            .expect("valid pro black turn ten nonwin fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_fast_black_late_mana_lane_nonwin",
            game: MonsGame::from_fen(
                "3 1 b 1 0 2 0 0 14 n11/n07a0xd0xxxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
                false,
            )
            .expect("valid fast black late mana lane nonwin fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_fast_black_late_second_lane_nonwin",
            game: MonsGame::from_fen(
                "3 1 b 1 0 3 0 0 14 n08d0xn02/n07a0xn01xxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
                false,
            )
            .expect("valid fast black late second lane nonwin fen"),
            mode: SmartAutomovePreference::Pro,
            expect_selected_matches_approved: true,
        },
    ];

    for case in cases {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let configured_runtime =
            calibration_runtime_config("frontier_pro_v2_guarded", &case.game, case.mode);
        let selected = MonsGameModel::smart_search_best_inputs(&case.game, configured_runtime);
        let decision = pro_v2_root_advisor_decision_snapshot()
            .unwrap_or_else(|| panic!("advisor decision missing for {}", case.label));
        let approved_root = decision
            .approved_root
            .as_ref()
            .unwrap_or_else(|| panic!("approved root missing for {}", case.label));

        let ordered_shortlist = decision
            .ordered_shortlist
            .iter()
            .map(format_root_advisor_entry_probe)
            .collect::<Vec<_>>()
            .join(" | ");
        let preserved = decision
            .preserved_family_representatives
            .iter()
            .map(format_root_advisor_entry_probe)
            .collect::<Vec<_>>()
            .join(" | ");
        let injected = decision.injected_root.as_ref().map_or_else(
            || "none".to_string(),
            |root| {
                format!(
                    "{}:{:?}:admitted={}:reason={:?}",
                    Input::fen_from_array(&root.inputs),
                    root.family,
                    root.admitted,
                    root.reason,
                )
            },
        );
        println!(
                    "ROOT_ADVISOR_TRACE label={} mode={:?} selected={} approved={} injected={} shortlist=[{}] preserved=[{}] fen={}",
                    case.label,
                    case.mode,
                    Input::fen_from_array(&selected),
                    format_root_advisor_entry_probe(approved_root),
                    injected,
                    ordered_shortlist,
                    preserved,
                    case.game.fen(),
                );
        if case.expect_selected_matches_approved {
            assert_eq!(
                approved_root.inputs, selected,
                "advisor-approved root must match the selected runtime move on {}",
                case.label,
            );
        }
        assert!(
            !decision.ordered_shortlist.is_empty(),
            "advisor shortlist must be non-empty on {}",
            case.label,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect retained pro-triage churn fixtures for frontier_pro_v2_guarded"]
fn smart_automove_pro_triage_retained_churn_probe() {
    let frontier_profile = "frontier_pro_v2_guarded";
    let shipping_profile = "shipping_pro_search";
    let fixture_ids = [
        "primary_black_negative_deny_ply4",
        "primary_black_late_accepted_head_ply4",
        "primary_black_turn_four_action_mana_ply15",
        "primary_black_mana_bridge_ply20",
        "primary_black_spirit_bridge_ply19",
        "primary_white_mana_sibling_ply9",
        "primary_white_safe_progress_rerank_ply27",
        "primary_white_harvest_loss_c_ply24",
        "primary_white_fast_accepted_head_ply13",
        "human_win_pro_c",
    ];

    println!(
        "retained churn probe: frontier={} shipping={} fixtures={}",
        frontier_profile,
        shipping_profile,
        fixture_ids.len()
    );
    for fixture_id in fixture_ids {
        let fixture = primary_pro_fixture_by_id(fixture_id);
        for profile_name in [frontier_profile, shipping_profile] {
            clear_exact_state_analysis_cache();
            clear_exact_query_diagnostics();
            clear_turn_engine_plan_cache();
            clear_turn_engine_diagnostics();
            clear_turn_engine_selector_diagnostics();

            let snapshot = pro_triage_fixture_snapshot(
                profile_name,
                profile_selector_from_name(profile_name)
                    .unwrap_or_else(|| panic!("profile '{}' not found", profile_name)),
                &fixture,
            );
            let (config, scored_roots) =
                profile_scored_roots(profile_name, fixture.mode, &fixture.game);
            let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
                &fixture.game,
                scored_roots.as_slice(),
                fixture.game.active_color,
                config,
            );
            let pre_accept_selected_fen = Input::fen_from_array(&pre_accept_selected);
            let pre_accept_selected_rank = scored_roots
                .iter()
                .position(|root| root.inputs == pre_accept_selected)
                .unwrap_or(scored_roots.len());
            let head_plan = if config.enable_turn_engine_selector {
                turn_engine_candidate_plan(
                    &fixture.game,
                    fixture.game.active_color,
                    MonsGameModel::turn_engine_config_for_game(&fixture.game, config),
                )
            } else {
                None
            };
            let selector_diag = turn_engine_selector_diagnostics_snapshot();
            let engine_diag = turn_engine_diagnostics_snapshot();
            let exact_diag = exact_query_diagnostics_snapshot();
            let selected_root = scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == snapshot.selected_input_fen);
            let head_root = head_plan
                .as_ref()
                .and_then(|plan| plan.compiled_chunks.first())
                .and_then(|chunk| {
                    scored_roots
                        .iter()
                        .find(|root| root.inputs.as_slice() == chunk.as_slice())
                });
            let reply_limit = config.node_enum_limit.clamp(
                SMART_NORMAL_ROOT_SAFETY_REPLY_LIMIT_MIN,
                SMART_NORMAL_ROOT_SAFETY_REPLY_LIMIT_MAX,
            );
            let my_score_before =
                MonsGameModel::score_for_color(&fixture.game, fixture.game.active_color);
            let start_options = MonsGameModel::automove_start_input_options(config);
            let selected_normal_snapshot = selected_root.map(|root| {
                MonsGameModel::normal_root_safety_snapshot(
                    &root.game,
                    fixture.game.active_color,
                    my_score_before,
                    config,
                    reply_limit,
                    start_options,
                )
            });
            let head_normal_snapshot = head_root.map(|root| {
                MonsGameModel::normal_root_safety_snapshot(
                    &root.game,
                    fixture.game.active_color,
                    my_score_before,
                    config,
                    reply_limit,
                    start_options,
                )
            });

            println!(
                    "RETAINED_CHURN fixture={} profile={} selected_rank={} selected={} pre_accept_rank={} pre_accept={} top_roots={:?} selector(last_stage={} head_calls={} head_hits={} child_calls={} children={} shortlist={} full_pass={} prefer_builds={} prefer_hits={}) head_plan(first_chunk={:?} selected_matches_head={} head_family={:?} goal_family={:?} utility={:?} head_utility={:?}) selected_root=\"{}\" head_root=\"{}\" normal_safety(selected=\"{}\" head=\"{}\") engine(accepted={} cache_hits={} cache_misses={} reply_calls={}) exact(tactical_spirit_calls={} tactical_spirit_hits={} secure_mana_calls={} secure_mana_hits={} pickup_calls={} pickup_hits={}) fen={}",
                    fixture.id,
                    profile_name,
                    snapshot.selected_rank,
                    snapshot.selected_input_fen,
                    pre_accept_selected_rank,
                    pre_accept_selected_fen,
                    snapshot.top_root_fens,
                    selector_diag.last_return_stage,
                    selector_diag.head_plan_calls,
                    selector_diag.head_plan_hits,
                    selector_diag.ranked_child_states_calls,
                    selector_diag.ranked_child_states_children_enumerated,
                    selector_diag.child_ordering_shortlist_children,
                    selector_diag.child_ordering_full_pass_children,
                    selector_diag.search_preferability_builds,
                    selector_diag.search_preferability_cache_hits,
                    head_plan
                        .as_ref()
                        .and_then(|plan| plan.compiled_chunks.first())
                        .map(|chunk| Input::fen_from_array(chunk)),
                    head_plan.as_ref().and_then(|plan| plan.compiled_chunks.first()).is_some_and(
                        |chunk| Input::fen_from_array(chunk) == snapshot.selected_input_fen
                    ),
                    head_plan.as_ref().map(|plan| plan.head_family),
                    head_plan.as_ref().map(|plan| plan.goal_family),
                    head_plan.as_ref().map(|plan| plan.utility),
                    head_plan.as_ref().map(|plan| plan.head_utility),
                    format_root_probe(selected_root),
                    format_root_probe(head_root),
                    format_normal_safety_probe(selected_normal_snapshot),
                    format_normal_safety_probe(head_normal_snapshot),
                    engine_diag.accepted_plans,
                    engine_diag.cache_hits,
                    engine_diag.cache_misses,
                    engine_diag.reply_search_calls,
                    exact_diag.tactical_spirit_summary_calls,
                    exact_diag.tactical_spirit_summary_cache_hits,
                    exact_diag.exact_secure_mana_calls,
                    exact_diag.exact_secure_mana_cache_hits,
                    exact_diag.pickup_path_calls,
                    exact_diag.pickup_path_cache_hits,
                    fixture.game.fen(),
                );
        }
    }
}

#[test]
#[ignore = "diagnostic: inspect forced-turn-engine probe acceptance on retained churn fixtures"]
fn smart_automove_pro_forced_turn_engine_retained_churn_probe() {
    let fixture_ids = [
        "primary_black_negative_deny_ply4",
        "primary_black_late_accepted_head_ply4",
        "primary_black_turn_four_action_mana_ply15",
        "primary_black_mana_bridge_ply20",
        "primary_black_spirit_bridge_ply19",
        "primary_white_mana_sibling_ply9",
        "primary_white_safe_progress_rerank_ply27",
        "primary_white_harvest_loss_c_ply24",
        "primary_white_fast_accepted_head_ply13",
        "human_win_pro_c",
    ];

    for fixture_id in fixture_ids {
        let fixture = primary_pro_fixture_by_id(fixture_id);
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let (config, scored_roots, head_plan, forced_engine_inputs) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "frontier_pro_v2_guarded",
                fixture.mode,
                &fixture.game,
            );
        let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
            &fixture.game,
            scored_roots.as_slice(),
            fixture.game.active_color,
            config,
        );
        let pre_accept_rank = scored_roots
            .iter()
            .position(|root| root.inputs == pre_accept_selected);
        let head_rank = head_plan.as_ref().and_then(|plan| {
            plan.compiled_chunks.first().and_then(|chunk| {
                scored_roots
                    .iter()
                    .position(|root| root.inputs.as_slice() == chunk.as_slice())
            })
        });
        let accepted = head_plan.as_ref().is_some_and(|plan| {
            MonsGameModel::accept_turn_engine_head_after_search(
                &fixture.game,
                fixture.game.active_color,
                config,
                scored_roots.as_slice(),
                pre_accept_selected.as_slice(),
                plan,
            )
        });
        let selected_root = pre_accept_rank.and_then(|index| scored_roots.get(index));
        let head_root = head_rank.and_then(|index| scored_roots.get(index));
        let reply_limit = config.node_enum_limit.clamp(
            SMART_NORMAL_ROOT_SAFETY_REPLY_LIMIT_MIN,
            SMART_NORMAL_ROOT_SAFETY_REPLY_LIMIT_MAX,
        );
        let my_score_before =
            MonsGameModel::score_for_color(&fixture.game, fixture.game.active_color);
        let start_options = MonsGameModel::automove_start_input_options(config);
        let selected_normal_snapshot = selected_root.map(|root| {
            MonsGameModel::normal_root_safety_snapshot(
                &root.game,
                fixture.game.active_color,
                my_score_before,
                config,
                reply_limit,
                start_options,
            )
        });
        let head_normal_snapshot = head_root.map(|root| {
            MonsGameModel::normal_root_safety_snapshot(
                &root.game,
                fixture.game.active_color,
                my_score_before,
                config,
                reply_limit,
                start_options,
            )
        });

        println!(
            "FORCED_TURN_ENGINE_PROBE fixture={} forced_inputs={:?} pre_accept_rank={:?} pre_accept={} head_rank={:?} head={:?} accepted={} selected_root=\"{}\" head_root=\"{}\" normal_safety(selected=\"{}\" head=\"{}\")",
            fixture.id,
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            pre_accept_rank,
            Input::fen_from_array(&pre_accept_selected),
            head_rank,
            head_plan
                .as_ref()
                .and_then(|plan| plan.compiled_chunks.first())
                .map(|chunk| Input::fen_from_array(chunk)),
            accepted,
            format_root_probe(selected_root),
            format_root_probe(head_root),
            format_normal_safety_probe(selected_normal_snapshot),
            format_normal_safety_probe(head_normal_snapshot),
        );
    }
}
