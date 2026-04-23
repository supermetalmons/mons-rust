use super::*;
use crate::models::scoring::{
    evaluate_preferability_breakdown_with_weights, evaluate_preferability_with_context,
    evaluate_preferability_with_weights_and_exact_policy, ScoringEvalContext, ScoringWeights,
};

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

fn attribution_search_eval_with_flags(
    game: &MonsGame,
    perspective: Color,
    config: AutomoveSearchConfig,
    allow_exact_static_evaluation: bool,
    enable_local_ctx: bool,
    enable_attack_reach_summary: bool,
    enable_attack_reach_target_narrowing: bool,
    enable_attack_reach_drainer_target_narrowing: bool,
) -> i32 {
    if enable_local_ctx {
        let context = ScoringEvalContext::new_with_flags(
            game,
            allow_exact_static_evaluation,
            enable_attack_reach_summary,
            enable_attack_reach_target_narrowing,
            enable_attack_reach_drainer_target_narrowing,
        );
        evaluate_preferability_with_context(
            game,
            perspective,
            config.scoring_weights,
            allow_exact_static_evaluation,
            &context,
        )
    } else {
        evaluate_preferability_with_weights_and_exact_policy(
            game,
            perspective,
            config.scoring_weights,
            allow_exact_static_evaluation,
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
            config.enable_static_exact_evaluation,
            false,
            false,
            false,
            false,
        ),
        ("exact_on_no_local", true, false, false, false, false),
        ("exact_off_no_local", false, false, false, false, false),
        ("exact_on_local_no_reach", true, true, false, false, false),
        ("exact_off_local_no_reach", false, true, false, false, false),
        ("exact_on_local_drainer", true, true, true, true, true),
        ("exact_off_local_drainer", false, true, true, true, true),
    ];
    variants
        .iter()
        .map(
            |(label, allow_exact, local_ctx, summary, target, drainer_target)| {
                let left_score = attribution_search_eval_with_flags(
                    left_game,
                    perspective,
                    config,
                    *allow_exact,
                    *local_ctx,
                    *summary,
                    *target,
                    *drainer_target,
                );
                let right_score = attribution_search_eval_with_flags(
                    right_game,
                    perspective,
                    config,
                    *allow_exact,
                    *local_ctx,
                    *summary,
                    *target,
                    *drainer_target,
                );
                format!(
                    "{}:{}_minus_{}={}({}-{})",
                    label,
                    left_label,
                    right_label,
                    left_score - right_score,
                    left_score,
                    right_score,
                )
            },
        )
        .collect::<Vec<_>>()
        .join(",")
}

fn attribution_worst_reply_state(
    state_after_move: &MonsGame,
    perspective: Color,
    config: AutomoveSearchConfig,
) -> Option<AttributionWorstReply> {
    let reply_limit = config.root_reply_risk_reply_limit.max(1).min(24);
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
    let duel_specs = vec![
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

    with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
        println!(
            "pro reliability duel trace probe: frontier={} shipping={} repeats={} games_per_repeat={} max_plies={} trace_limit={}",
            frontier_profile,
            shipping_profile,
            repeats,
            games,
            max_plies,
            trace_limit,
        );

        for duel in duel_specs {
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
    });
}

#[test]
#[ignore = "diagnostic: inspect white confirm pro ply11 reply-order utility and floor"]
fn white_confirm_pro_ply11_reply_order_probe() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 2 0 0 3 n11/n03y0xd0ms0xa0xe0xn03/n11/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn03Y0xn02/n07xxMn03/n05S0xn05/n03E0xA0xn03D0xn02",
        false,
    )
    .expect("valid white confirm pro ply11 fen");
    let perspective = game.active_color;
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
    let projections = MonsGameModel::turn_engine_reply_risk_projections(
        scored_roots.as_slice(),
        shortlist.as_slice(),
        perspective,
        config,
    );
    let approved_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l10,4;l9,3")
        .expect("approved root should exist");
    let shipping_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l7,8;l6,9")
        .expect("shipping root should exist");
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(shortlist.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let approved_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[approved_index],
        projections.get(&approved_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let shipping_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[shipping_index],
        projections.get(&shipping_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let approved_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[approved_index]);
    let shipping_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[shipping_index]);
    let approved_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[approved_index],
        perspective,
        config,
        approved_family,
    );
    let shipping_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[shipping_index],
        perspective,
        config,
        shipping_family,
    );
    let shipping_beats_approved = MonsGameModel::is_better_reply_risk_candidate(
        &game,
        shipping_index,
        shipping_snapshot,
        approved_index,
        approved_snapshot,
        projections.get(&shipping_index),
        projections.get(&approved_index),
        scored_roots.as_slice(),
        perspective,
        config,
        &mut std::collections::HashMap::new(),
    );
    println!(
        "WHITE_CONFIRM_PRO_PLY11_REPLY_ORDER shortlist={:?} approved={} shipping={} approved_snapshot={} shipping_snapshot={} approved_utility={} shipping_utility={} shipping_vs_approved={} approved_projection={:?} shipping_projection={:?}",
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        format_root_probe(scored_roots.get(approved_index)),
        format_root_probe(scored_roots.get(shipping_index)),
        format!(
            "win={} match_point={} floor={}",
            approved_snapshot.allows_immediate_opponent_win,
            approved_snapshot.opponent_reaches_match_point,
            approved_snapshot.worst_reply_score,
        ),
        format!(
            "win={} match_point={} floor={}",
            shipping_snapshot.allows_immediate_opponent_win,
            shipping_snapshot.opponent_reaches_match_point,
            shipping_snapshot.worst_reply_score,
        ),
        format_turn_engine_utility_probe(approved_utility),
        format_turn_engine_utility_probe(shipping_utility),
        shipping_beats_approved,
        projections.get(&approved_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        projections.get(&shipping_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
    );
}

#[test]
#[ignore = "diagnostic: inspect white confirm pro ply11 selector surface"]
fn white_confirm_pro_ply11_selector_surface_probe() {
    fn run_raw_search_variant<F>(game: &MonsGame, tweak: F) -> (String, &'static str, &'static str)
    where
        F: FnOnce(&mut AutomoveSearchConfig),
    {
        let selector = profile_selector_from_name("shipping_pro_search")
            .expect("shipping profile selector should exist");
        let mut config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            game,
            SmartAutomovePreference::Pro,
        );
        tweak(&mut config);

        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let inputs = select_inputs_with_runtime_fallback(selector, game, config);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        (
            Input::fen_from_array(&inputs),
            selector_diag.last_return_stage,
            selector_diag.selector_disable_reason,
        )
    }

    fn eval_breakdown_probe(
        game: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> String {
        let search_eval = MonsGameModel::evaluate_search_preferability(game, perspective, config);
        let breakdown = crate::models::scoring::evaluate_preferability_breakdown_with_weights(
            game,
            perspective,
            config.scoring_weights,
        );
        format!(
            "search_eval={} breakdown_total={} terms={:?} features={:?}",
            search_eval, breakdown.total, breakdown.terms, breakdown.features,
        )
    }

    fn worst_reply_details(
        root: &RootEvaluation,
        perspective: Color,
        config: AutomoveSearchConfig,
        reply_limit: usize,
    ) -> Vec<String> {
        let mut replies = MonsGameModel::enumerate_legal_transitions(
            &root.game,
            reply_limit.max(1),
            MonsGameModel::automove_start_input_options(config),
        )
        .into_iter()
        .map(|reply| {
            let reply_score = match reply.game.winner_color() {
                Some(winner) if winner == perspective => SMART_TERMINAL_SCORE / 2,
                Some(_) => -SMART_TERMINAL_SCORE / 2,
                None => {
                    MonsGameModel::evaluate_search_preferability(&reply.game, perspective, config)
                }
            };
            (
                reply_score,
                Input::fen_from_array(&reply.inputs),
                exact_opportunity_context_probe(&reply.game),
                eval_breakdown_probe(&reply.game, perspective, config),
            )
        })
        .collect::<Vec<_>>();
        replies.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        replies
            .into_iter()
            .take(4)
            .map(|(score, inputs, context, breakdown)| {
                format!(
                    "{} score={} context={} {}",
                    inputs, score, context, breakdown
                )
            })
            .collect()
    }

    let game = MonsGame::from_fen(
        "0 0 w 1 0 2 0 0 3 n11/n03y0xd0ms0xa0xe0xn03/n11/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn03Y0xn02/n07xxMn03/n05S0xn05/n03E0xA0xn03D0xn02",
        false,
    )
    .expect("valid white confirm pro ply11 fen");
    let perspective = game.active_color;
    let frontier_probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
    let shipping_probe =
        runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    let (config, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let (shipping_config, shipping_scored_roots, _, _) =
        profile_runtime_scored_roots_with_forced_engine_inputs(
            "shipping_pro_search",
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
    let projections = MonsGameModel::turn_engine_reply_risk_projections(
        scored_roots.as_slice(),
        shortlist.as_slice(),
        perspective,
        config,
    );
    let approved_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l10,4;l9,3")
        .expect("approved root should exist");
    let shipping_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l7,8;l6,9")
        .expect("shipping root should exist");
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(shortlist.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let all_indices = (0..scored_roots.len()).collect::<Vec<_>>();
    let mut pro_v1_no_guard_config = config;
    pro_v1_no_guard_config.enable_root_reply_risk_guard = false;
    pro_v1_no_guard_config.turn_engine_mode = TurnEngineMode::ProV1;
    let pro_v1_candidate_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            perspective,
            pro_v1_no_guard_config,
        );
    let pro_v1_full_pool_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            all_indices.as_slice(),
            perspective,
            pro_v1_no_guard_config,
        );
    let mut pro_v2_no_guard_config = config;
    pro_v2_no_guard_config.enable_root_reply_risk_guard = false;
    let pro_v2_candidate_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            perspective,
            pro_v2_no_guard_config,
        );
    let pro_v2_full_pool_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            all_indices.as_slice(),
            perspective,
            pro_v2_no_guard_config,
        );
    let raw_search_only_pro_v2 = run_raw_search_variant(&game, |variant| {
        variant.enable_turn_engine_selector = false;
        variant.enable_turn_head_rerank = true;
    });
    let raw_search_only_pro_v1 = run_raw_search_variant(&game, |variant| {
        variant.enable_turn_engine_selector = false;
        variant.enable_turn_head_rerank = true;
        variant.turn_engine_mode = TurnEngineMode::ProV1;
    });
    let raw_search_only_shipping_caps_pro_v2 = run_raw_search_variant(&game, |variant| {
        variant.enable_turn_engine_selector = false;
        variant.enable_turn_head_rerank = true;
        variant.turn_engine_seed_cap = shipping_config.turn_engine_seed_cap;
        variant.turn_engine_beam_width = shipping_config.turn_engine_beam_width;
        variant.turn_engine_per_node_family_cap = shipping_config.turn_engine_per_node_family_cap;
        variant.turn_engine_step_cap = shipping_config.turn_engine_step_cap;
    });
    let shipping_root_inputs = Input::array_from_fen("l7,8;l6,9");
    let approved_root_inputs = Input::array_from_fen("l10,4;l9,3");
    let shipping_rank_on_shipping = shipping_scored_roots
        .iter()
        .position(|root| root.inputs == shipping_root_inputs);
    let approved_rank_on_shipping = shipping_scored_roots
        .iter()
        .position(|root| root.inputs == approved_root_inputs);
    let target_details = [("approved", approved_index), ("shipping", shipping_index)]
        .into_iter()
        .map(|(label, index)| {
            let root = &scored_roots[index];
            let family = MonsGameModel::turn_engine_root_evaluation_family(root);
            let utility = MonsGameModel::turn_engine_selected_override_utility(
                &game,
                root,
                perspective,
                config,
                family,
            );
            let snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
                root,
                projections.get(&index),
                perspective,
                config,
                per_root_reply_limit,
            );
            format!(
                "{}={} index={} candidate_pos={:?} shortlist_pos={:?} family={:?} classes={:?} utility={} snapshot=win:{} match:{} floor:{} after_context={} after_eval={} worst_replies={:?} detail={}",
                label,
                Input::fen_from_array(&root.inputs),
                index,
                candidate_indices
                    .iter()
                    .position(|candidate_index| *candidate_index == index),
                shortlist
                    .iter()
                    .position(|shortlist_index| *shortlist_index == index),
                family,
                root.classes,
                format_turn_engine_utility_probe(utility),
                snapshot.allows_immediate_opponent_win,
                snapshot.opponent_reaches_match_point,
                snapshot.worst_reply_score,
                exact_opportunity_context_probe(&root.game),
                eval_breakdown_probe(&root.game, perspective, config),
                worst_reply_details(root, perspective, config, per_root_reply_limit),
                format_root_probe(Some(root)),
            )
        })
        .collect::<Vec<_>>();
    let shortlist_details = shortlist
        .iter()
        .map(|index| {
            format!(
                "{} => {}",
                Input::fen_from_array(&scored_roots[*index].inputs),
                format_root_probe(scored_roots.get(*index)),
            )
        })
        .collect::<Vec<_>>();
    let top_details = scored_roots
        .iter()
        .take(16)
        .map(|root| {
            format!(
                "{} => {} classes={:?}",
                Input::fen_from_array(&root.inputs),
                format_root_probe(Some(root)),
                root.classes,
            )
        })
        .collect::<Vec<_>>();

    println!(
        "WHITE_CONFIRM_PRO_PLY11_SELECTOR_SURFACE context={} frontier_probe={:?} frontier_advisor={:?} shipping_probe={:?} candidate_count={} candidate_head={:?} shortlist={:?} shortlist_details={:?} target_details={:?} top_details={:?} no_guard_selected={{pro_v1_candidate:{}, pro_v1_full_pool:{}, pro_v2_candidate:{}, pro_v2_full_pool:{}}} raw_search={{pro_v2:{:?}, pro_v1:{:?}, shipping_caps_pro_v2:{:?}}} shipping_root_positions={{shipping_rank_on_frontier:{:?}, approved_rank_on_frontier:{:?}, shipping_rank_on_shipping:{:?}, approved_rank_on_shipping:{:?}}}",
        exact_opportunity_context_probe(&game),
        frontier_probe,
        frontier_advisor,
        shipping_probe,
        candidate_indices.len(),
        candidate_indices
            .iter()
            .take(20)
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        shortlist_details,
        target_details,
        top_details,
        Input::fen_from_array(&pro_v1_candidate_selected),
        Input::fen_from_array(&pro_v1_full_pool_selected),
        Input::fen_from_array(&pro_v2_candidate_selected),
        Input::fen_from_array(&pro_v2_full_pool_selected),
        raw_search_only_pro_v2,
        raw_search_only_pro_v1,
        raw_search_only_shipping_caps_pro_v2,
        Some(shipping_index),
        Some(approved_index),
        shipping_rank_on_shipping,
        approved_rank_on_shipping,
    );
}

#[test]
#[ignore = "diagnostic: inspect late black fast reply-order utility and floor"]
fn black_late_fast_reply_order_probe() {
    let game = MonsGame::from_fen(
        "3 1 b 1 0 2 0 0 14 n11/n07a0xd0xxxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
        false,
    )
    .expect("valid black late fast trace fen");
    let perspective = game.active_color;
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
    let projections = MonsGameModel::turn_engine_reply_risk_projections(
        scored_roots.as_slice(),
        shortlist.as_slice(),
        perspective,
        config,
    );
    let approved_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l1,8;l1,9")
        .expect("approved root should exist");
    let shipping_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l1,8;l0,8")
        .expect("shipping root should exist");
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(shortlist.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let approved_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[approved_index],
        projections.get(&approved_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let shipping_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[shipping_index],
        projections.get(&shipping_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let approved_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[approved_index]);
    let shipping_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[shipping_index]);
    let approved_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[approved_index],
        perspective,
        config,
        approved_family,
    );
    let shipping_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[shipping_index],
        perspective,
        config,
        shipping_family,
    );
    let shipping_beats_approved = MonsGameModel::is_better_reply_risk_candidate(
        &game,
        shipping_index,
        shipping_snapshot,
        approved_index,
        approved_snapshot,
        projections.get(&shipping_index),
        projections.get(&approved_index),
        scored_roots.as_slice(),
        perspective,
        config,
        &mut std::collections::HashMap::new(),
    );

    println!(
        "BLACK_LATE_FAST_REPLY_ORDER shortlist={:?} approved={} shipping={} approved_snapshot={} shipping_snapshot={} approved_utility={} shipping_utility={} shipping_vs_approved={} approved_projection={:?} shipping_projection={:?}",
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        format_root_probe(scored_roots.get(approved_index)),
        format_root_probe(scored_roots.get(shipping_index)),
        format!(
            "win={} match_point={} floor={}",
            approved_snapshot.allows_immediate_opponent_win,
            approved_snapshot.opponent_reaches_match_point,
            approved_snapshot.worst_reply_score,
        ),
        format!(
            "win={} match_point={} floor={}",
            shipping_snapshot.allows_immediate_opponent_win,
            shipping_snapshot.opponent_reaches_match_point,
            shipping_snapshot.worst_reply_score,
        ),
        format_turn_engine_utility_probe(approved_utility),
        format_turn_engine_utility_probe(shipping_utility),
        shipping_beats_approved,
        projections.get(&approved_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        projections.get(&shipping_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
    );
}

#[test]
#[ignore = "diagnostic: inspect black recovery branch legacy-alignment guards"]
fn black_recovery_branch_legacy_alignment_probe() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        false,
    )
    .expect("valid black recovery branch fen");
    let perspective = game.active_color;
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
    let reply_risk_shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
        scored_roots.as_slice(),
        candidate_indices.as_slice(),
        config,
    );
    let approved_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l1,5;l3,3;l2,3")
        .expect("approved spirit root should exist");
    let legacy_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l6,0;l6,1")
        .expect("legacy mana root should exist");
    let approved = &scored_roots[approved_index];
    let legacy = &scored_roots[legacy_index];
    let exact_context =
        crate::models::automove_exact::exact_opportunity_context(&game, game.active_color);
    let approved_non_tactical = !approved.wins_immediately
        && !approved.attacks_opponent_drainer
        && !approved.scores_supermana_this_turn
        && !approved.scores_opponent_mana_this_turn
        && !approved.safe_supermana_pickup_now
        && !approved.safe_opponent_mana_pickup_now
        && !approved.mana_handoff_to_opponent
        && !approved.has_roundtrip;
    let legacy_non_tactical = !legacy.wins_immediately
        && !legacy.attacks_opponent_drainer
        && !legacy.scores_supermana_this_turn
        && !legacy.scores_opponent_mana_this_turn
        && !legacy.safe_supermana_pickup_now
        && !legacy.safe_opponent_mana_pickup_now
        && !legacy.mana_handoff_to_opponent
        && !legacy.has_roundtrip;
    let override_index = MonsGameModel::pro_v2_root_advisor_black_legacy_alignment_override(
        &game,
        scored_roots.as_slice(),
        candidate_indices.as_slice(),
        approved_index,
        legacy_index,
        config,
    );
    let mut legacy_selector_config = config;
    legacy_selector_config.enable_root_reply_risk_guard = false;
    legacy_selector_config.turn_engine_mode = TurnEngineMode::ProV1;
    let pro_v1_candidate_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            perspective,
            legacy_selector_config,
        );
    let (probe_legacy_selected, probe_legacy_full_pool_selected, _, _) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);
    let shortlist_root_details = reply_risk_shortlist
        .iter()
        .map(|index| {
            format!(
                "{} => {}",
                Input::fen_from_array(&scored_roots[*index].inputs),
                format_root_probe(scored_roots.get(*index)),
            )
        })
        .collect::<Vec<_>>();

    println!(
        "BLACK_RECOVERY_BRANCH_LEGACY_ALIGNMENT approved={} legacy={} candidate_contains_legacy={} shortlist={:?} shortlist_root_details={:?} approved_family={:?} legacy_family={:?} approved_plain_spirit={} approved_progress_surface={} approved_non_tactical={} legacy_non_tactical={} exact_window={} exact_deny={} exact_attack={} approved_vulnerable={} legacy_vulnerable={} legacy_score_ge_approved={} override={:?} pro_v1_candidate_selected={} probe_legacy_selected={} probe_legacy_full_pool_selected={}",
        format_root_probe(scored_roots.get(approved_index)),
        format_root_probe(scored_roots.get(legacy_index)),
        candidate_indices.contains(&legacy_index),
        reply_risk_shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        shortlist_root_details,
        MonsGameModel::turn_engine_root_evaluation_family(approved),
        MonsGameModel::turn_engine_root_evaluation_family(legacy),
        MonsGameModel::is_plain_spirit_development_root(approved),
        MonsGameModel::turn_engine_root_evaluation_has_progress_surface(approved),
        approved_non_tactical,
        legacy_non_tactical,
        exact_context.delta.same_turn_score_window_value,
        exact_context.delta.opponent_window_deny_gain,
        exact_context.delta.drainer_attack_available,
        approved.own_drainer_vulnerable,
        legacy.own_drainer_vulnerable,
        legacy.score >= approved.score,
        override_index.map(|index| Input::fen_from_array(&scored_roots[index].inputs)),
        Input::fen_from_array(&pro_v1_candidate_selected),
        probe_legacy_selected,
        probe_legacy_full_pool_selected,
    );
}

#[test]
#[ignore = "diagnostic: inspect black recovery branch selector ordering"]
fn black_recovery_branch_selector_ordering_probe() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        false,
    )
    .expect("valid black recovery branch fen");
    let perspective = game.active_color;
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
    let reply_risk_shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
        scored_roots.as_slice(),
        candidate_indices.as_slice(),
        config,
    );
    let guard_projections = MonsGameModel::turn_engine_reply_risk_projections(
        scored_roots.as_slice(),
        reply_risk_shortlist.as_slice(),
        perspective,
        config,
    );
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(reply_risk_shortlist.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / reply_risk_shortlist.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let frontier_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l1,5;l3,3;l2,3")
        .expect("frontier spirit root should exist");
    let shipping_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l6,0;l6,1")
        .expect("shipping mana root should exist");
    let score_leader_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l6,0;l7,0")
        .expect("score-leading mana sibling should exist");
    let mut legacy_selector_config = config;
    legacy_selector_config.enable_root_reply_risk_guard = false;
    legacy_selector_config.turn_engine_mode = TurnEngineMode::ProV1;
    let all_indices = (0..scored_roots.len()).collect::<Vec<_>>();
    let pro_v1_candidate_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            perspective,
            legacy_selector_config,
        );
    let pro_v1_candidate_index = scored_roots
        .iter()
        .position(|root| root.inputs == pro_v1_candidate_selected)
        .expect("candidate-only ProV1 selected root should exist");
    let pro_v1_full_pool_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            all_indices.as_slice(),
            perspective,
            legacy_selector_config,
        );
    let pro_v1_full_pool_index = scored_roots
        .iter()
        .position(|root| root.inputs == pro_v1_full_pool_selected)
        .expect("full-pool ProV1 selected root should exist");
    let mut pro_v2_no_guard_config = config;
    pro_v2_no_guard_config.enable_root_reply_risk_guard = false;
    let pro_v2_no_guard_candidate_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            perspective,
            pro_v2_no_guard_config,
        );
    let pro_v2_no_guard_full_pool_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            all_indices.as_slice(),
            perspective,
            pro_v2_no_guard_config,
        );
    let target_indices = [
        frontier_index,
        pro_v1_candidate_index,
        shipping_index,
        score_leader_index,
        pro_v1_full_pool_index,
    ]
    .into_iter()
    .fold(Vec::<usize>::new(), |mut indices, index| {
        if !indices.contains(&index) {
            indices.push(index);
        }
        indices
    });
    let target_projections = MonsGameModel::turn_engine_reply_risk_projections(
        scored_roots.as_slice(),
        target_indices.as_slice(),
        perspective,
        config,
    );
    let target_specs = [
        ("frontier", frontier_index),
        ("pro_v1_candidate", pro_v1_candidate_index),
        ("shipping", shipping_index),
        ("score_leader", score_leader_index),
        ("pro_v1_full_pool", pro_v1_full_pool_index),
    ];
    let target_details = target_specs
        .iter()
        .map(|(label, index)| {
            let root = &scored_roots[*index];
            let family = MonsGameModel::turn_engine_root_evaluation_family(root);
            let utility = MonsGameModel::turn_engine_selected_override_utility(
                &game,
                root,
                perspective,
                config,
                family,
            );
            let guard_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
                root,
                guard_projections.get(index),
                perspective,
                config,
                per_root_reply_limit,
            );
            let target_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
                root,
                target_projections.get(index),
                perspective,
                config,
                per_root_reply_limit,
            );
            format!(
                "{}={} index={} candidate_pos={:?} shortlist_pos={:?} family={:?} plain_spirit={} progress_surface={} utility={} guard_snapshot=win:{} match:{} floor:{} target_snapshot=win:{} match:{} floor:{} guard_projection={:?} target_projection={:?} followup={} details={}",
                label,
                Input::fen_from_array(&root.inputs),
                index,
                candidate_indices
                    .iter()
                    .position(|candidate_index| candidate_index == index),
                reply_risk_shortlist
                    .iter()
                    .position(|shortlist_index| shortlist_index == index),
                family,
                MonsGameModel::is_plain_spirit_development_root(root),
                MonsGameModel::turn_engine_root_evaluation_has_progress_surface(root),
                format_turn_engine_utility_probe(utility),
                guard_snapshot.allows_immediate_opponent_win,
                guard_snapshot.opponent_reaches_match_point,
                guard_snapshot.worst_reply_score,
                target_snapshot.allows_immediate_opponent_win,
                target_snapshot.opponent_reaches_match_point,
                target_snapshot.worst_reply_score,
                guard_projections.get(index).map(|projection| {
                    format!(
                        "{:?}/{:?}/{}",
                        projection.plan.head_family,
                        projection.plan.goal_family,
                        format_turn_engine_utility_probe(projection.plan.utility),
                    )
                }),
                target_projections.get(index).map(|projection| {
                    format!(
                        "{:?}/{:?}/{}",
                        projection.plan.head_family,
                        projection.plan.goal_family,
                        format_turn_engine_utility_probe(projection.plan.utility),
                    )
                }),
                MonsGameModel::pro_v2_spirit_followup_floor_score(&root.game, perspective, config),
                format_root_probe(scored_roots.get(*index)),
            )
        })
        .collect::<Vec<_>>();
    let pairwise = [
        ("shipping_vs_frontier", shipping_index, frontier_index),
        ("frontier_vs_shipping", frontier_index, shipping_index),
        (
            "shipping_vs_pro_v1_candidate",
            shipping_index,
            pro_v1_candidate_index,
        ),
        (
            "pro_v1_candidate_vs_shipping",
            pro_v1_candidate_index,
            shipping_index,
        ),
        (
            "score_leader_vs_shipping",
            score_leader_index,
            shipping_index,
        ),
        (
            "shipping_vs_score_leader",
            shipping_index,
            score_leader_index,
        ),
    ]
    .into_iter()
    .map(|(label, candidate_index, incumbent_index)| {
        let candidate = &scored_roots[candidate_index];
        let incumbent = &scored_roots[incumbent_index];
        let guard_candidate_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
            candidate,
            guard_projections.get(&candidate_index),
            perspective,
            config,
            per_root_reply_limit,
        );
        let guard_incumbent_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
            incumbent,
            guard_projections.get(&incumbent_index),
            perspective,
            config,
            per_root_reply_limit,
        );
        let target_candidate_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
            candidate,
            target_projections.get(&candidate_index),
            perspective,
            config,
            per_root_reply_limit,
        );
        let target_incumbent_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
            incumbent,
            target_projections.get(&incumbent_index),
            perspective,
            config,
            per_root_reply_limit,
        );
        let guard_better = MonsGameModel::is_better_reply_risk_candidate(
            &game,
            candidate_index,
            guard_candidate_snapshot,
            incumbent_index,
            guard_incumbent_snapshot,
            guard_projections.get(&candidate_index),
            guard_projections.get(&incumbent_index),
            scored_roots.as_slice(),
            perspective,
            config,
            &mut std::collections::HashMap::new(),
        );
        let target_better = MonsGameModel::is_better_reply_risk_candidate(
            &game,
            candidate_index,
            target_candidate_snapshot,
            incumbent_index,
            target_incumbent_snapshot,
            target_projections.get(&candidate_index),
            target_projections.get(&incumbent_index),
            scored_roots.as_slice(),
            perspective,
            config,
            &mut std::collections::HashMap::new(),
        );
        format!(
            "{} guard_better={} target_better={} guard_floors={}/{} target_floors={}/{}",
            label,
            guard_better,
            target_better,
            guard_candidate_snapshot.worst_reply_score,
            guard_incumbent_snapshot.worst_reply_score,
            target_candidate_snapshot.worst_reply_score,
            target_incumbent_snapshot.worst_reply_score,
        )
    })
    .collect::<Vec<_>>();
    let legacy_alignment_checks = [
        (
            "shipping_over_frontier",
            frontier_index,
            shipping_index,
            MonsGameModel::pro_v2_root_advisor_black_legacy_alignment_override(
                &game,
                scored_roots.as_slice(),
                candidate_indices.as_slice(),
                frontier_index,
                shipping_index,
                config,
            ),
        ),
        (
            "shipping_over_pro_v1_candidate",
            pro_v1_candidate_index,
            shipping_index,
            MonsGameModel::pro_v2_root_advisor_black_legacy_alignment_override(
                &game,
                scored_roots.as_slice(),
                candidate_indices.as_slice(),
                pro_v1_candidate_index,
                shipping_index,
                config,
            ),
        ),
        (
            "full_pool_legacy_over_frontier",
            frontier_index,
            pro_v1_full_pool_index,
            MonsGameModel::pro_v2_root_advisor_black_legacy_alignment_override(
                &game,
                scored_roots.as_slice(),
                candidate_indices.as_slice(),
                frontier_index,
                pro_v1_full_pool_index,
                config,
            ),
        ),
    ]
    .into_iter()
    .map(|(label, approved_index, legacy_index, override_index)| {
        let approved = &scored_roots[approved_index];
        let legacy = &scored_roots[legacy_index];
        format!(
            "{} approved={} legacy={} approved_family={:?} legacy_family={:?} approved_plain_spirit={} legacy_plain_spirit={} approved_progress_surface={} legacy_progress_surface={} approved_score={} legacy_score={} approved_rank={} legacy_rank={} override={:?}",
            label,
            Input::fen_from_array(&approved.inputs),
            Input::fen_from_array(&legacy.inputs),
            MonsGameModel::turn_engine_root_evaluation_family(approved),
            MonsGameModel::turn_engine_root_evaluation_family(legacy),
            MonsGameModel::is_plain_spirit_development_root(approved),
            MonsGameModel::is_plain_spirit_development_root(legacy),
            MonsGameModel::turn_engine_root_evaluation_has_progress_surface(approved),
            MonsGameModel::turn_engine_root_evaluation_has_progress_surface(legacy),
            approved.score,
            legacy.score,
            approved.root_rank,
            legacy.root_rank,
            override_index.map(|index| Input::fen_from_array(&scored_roots[index].inputs)),
        )
    })
    .collect::<Vec<_>>();
    let frontier_runtime_probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
    let shipping_runtime_probe =
        runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "BLACK_RECOVERY_BRANCH_SELECTOR_ORDERING context={} candidate_count={} candidate_head={:?} reply_risk_shortlist={:?} target_details={:?} pairwise={:?} legacy_alignment_checks={:?} pro_v1_candidate_selected={} pro_v1_full_pool_selected={} pro_v2_no_guard_candidate_selected={} pro_v2_no_guard_full_pool_selected={} frontier_runtime_probe={:?} frontier_advisor={:?} shipping_runtime_probe={:?}",
        exact_opportunity_context_probe(&game),
        candidate_indices.len(),
        candidate_indices
            .iter()
            .take(16)
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        reply_risk_shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        target_details,
        pairwise,
        legacy_alignment_checks,
        Input::fen_from_array(&pro_v1_candidate_selected),
        Input::fen_from_array(&pro_v1_full_pool_selected),
        Input::fen_from_array(&pro_v2_no_guard_candidate_selected),
        Input::fen_from_array(&pro_v2_no_guard_full_pool_selected),
        frontier_runtime_probe,
        frontier_advisor,
        shipping_runtime_probe,
    );
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
            config.root_reply_risk_reply_limit.max(1).min(24),
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
#[ignore = "diagnostic: inspect remaining black progress-vs-setup residue board"]
fn black_progress_vs_setup_residue_probe() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 0 0 0 6 n05d0xn05/n05s0xa0xe0xn03/n02xxmn04xxmn03/n07xxmn03/n03xxmn01xxmn05/n05xxUn04xxQ/n05xxMn05/n01y0xn01S0xE0xn01xxMxxMn03/n01xxMn09/n03A0xn03Y0xn03/n05D1xn05",
        false,
    )
    .expect("valid black residue fen");
    let perspective = game.active_color;
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
    let reply_risk_shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
        scored_roots.as_slice(),
        candidate_indices.as_slice(),
        config,
    );
    let approved_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l7,1;l9,3")
        .expect("approved root should exist");
    let shipping_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == "l1,5;l2,7;l1,8")
        .expect("shipping root should exist");
    let approved = &scored_roots[approved_index];
    let shipping = &scored_roots[shipping_index];
    let exact_context =
        crate::models::automove_exact::exact_opportunity_context(&game, game.active_color);
    let approved_non_tactical = !approved.wins_immediately
        && !approved.attacks_opponent_drainer
        && !approved.scores_supermana_this_turn
        && !approved.scores_opponent_mana_this_turn
        && !approved.safe_supermana_pickup_now
        && !approved.safe_opponent_mana_pickup_now
        && !approved.mana_handoff_to_opponent
        && !approved.has_roundtrip;
    let shipping_non_tactical = !shipping.wins_immediately
        && !shipping.attacks_opponent_drainer
        && !shipping.scores_supermana_this_turn
        && !shipping.scores_opponent_mana_this_turn
        && !shipping.safe_supermana_pickup_now
        && !shipping.safe_opponent_mana_pickup_now
        && !shipping.mana_handoff_to_opponent
        && !shipping.has_roundtrip;
    let mut legacy_selector_config = config;
    legacy_selector_config.enable_root_reply_risk_guard = false;
    legacy_selector_config.turn_engine_mode = TurnEngineMode::ProV1;
    let pro_v1_candidate_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            perspective,
            legacy_selector_config,
        );
    let (probe_legacy_selected, probe_legacy_full_pool_selected, _, _) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);
    let runtime_probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    let shipping_runtime_probe =
        runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    let frontier_top_root_details = scored_roots
        .iter()
        .take(8)
        .map(|root| {
            format!(
                "{} => {}",
                Input::fen_from_array(&root.inputs),
                format_root_probe(Some(root)),
            )
        })
        .collect::<Vec<_>>();
    let shortlist_root_details = reply_risk_shortlist
        .iter()
        .map(|index| {
            format!(
                "{} => {}",
                Input::fen_from_array(&scored_roots[*index].inputs),
                format_root_probe(scored_roots.get(*index)),
            )
        })
        .collect::<Vec<_>>();
    let spirit_setup_progress_candidates = candidate_indices
        .iter()
        .copied()
        .filter(|index| {
            let root = &scored_roots[*index];
            MonsGameModel::turn_engine_root_evaluation_family(root) == TurnPlanFamily::SpiritImpact
                && root.spirit_own_mana_setup_now
                && !root.spirit_same_turn_score_setup_now
                && MonsGameModel::turn_engine_root_evaluation_has_progress_surface(root)
                && !MonsGameModel::turn_engine_root_evaluation_is_unsafe(root)
        })
        .map(|index| {
            format!(
                "{} => {}",
                Input::fen_from_array(&scored_roots[index].inputs),
                format_root_probe(scored_roots.get(index)),
            )
        })
        .collect::<Vec<_>>();
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(candidate_indices.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / candidate_indices.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let spirit_setup_progress_candidate_metrics = candidate_indices
        .iter()
        .copied()
        .filter(|index| {
            let root = &scored_roots[*index];
            MonsGameModel::turn_engine_root_evaluation_family(root) == TurnPlanFamily::SpiritImpact
                && root.spirit_own_mana_setup_now
                && !root.spirit_same_turn_score_setup_now
                && MonsGameModel::turn_engine_root_evaluation_has_progress_surface(root)
                && !MonsGameModel::turn_engine_root_evaluation_is_unsafe(root)
        })
        .map(|index| {
            let root = &scored_roots[index];
            let utility = MonsGameModel::turn_engine_selected_override_utility(
                &game,
                root,
                perspective,
                config,
                TurnPlanFamily::SpiritImpact,
            );
            let snapshot = MonsGameModel::root_reply_risk_snapshot(
                &root.game,
                perspective,
                config,
                per_root_reply_limit,
            );
            let followup =
                MonsGameModel::pro_v2_spirit_followup_floor_score(&root.game, perspective, config);
            format!(
                "{} => utility={:?} worst_reply={} match_point={} immediate_win={} followup={} {}",
                Input::fen_from_array(&root.inputs),
                utility,
                snapshot.worst_reply_score,
                snapshot.opponent_reaches_match_point,
                snapshot.allows_immediate_opponent_win,
                followup,
                format_root_probe(Some(root)),
            )
        })
        .collect::<Vec<_>>();

    println!(
        "BLACK_PROGRESS_VS_SETUP_RESIDUE context={} approved={} shipping={} candidate_contains_shipping={} shortlist={:?} shortlist_root_details={:?} spirit_setup_progress_candidates={:?} spirit_setup_progress_candidate_metrics={:?} frontier_top_root_details={:?} approved_family={:?} shipping_family={:?} approved_plain_spirit={} approved_progress_surface={} shipping_progress_surface={} approved_non_tactical={} shipping_non_tactical={} exact_window={} exact_deny={} exact_attack={} approved_vulnerable={} shipping_vulnerable={} shipping_score_ge_approved={} pro_v1_candidate_selected={} probe_legacy_selected={} probe_legacy_full_pool_selected={} shipping_selected={} runtime_probe={:?} shipping_runtime_probe={:?}",
        exact_opportunity_context_probe(&game),
        format_root_probe(scored_roots.get(approved_index)),
        format_root_probe(scored_roots.get(shipping_index)),
        candidate_indices.contains(&shipping_index),
        reply_risk_shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        shortlist_root_details,
        spirit_setup_progress_candidates,
        spirit_setup_progress_candidate_metrics,
        frontier_top_root_details,
        MonsGameModel::turn_engine_root_evaluation_family(approved),
        MonsGameModel::turn_engine_root_evaluation_family(shipping),
        MonsGameModel::is_plain_spirit_development_root(approved),
        MonsGameModel::turn_engine_root_evaluation_has_progress_surface(approved),
        MonsGameModel::turn_engine_root_evaluation_has_progress_surface(shipping),
        approved_non_tactical,
        shipping_non_tactical,
        exact_context.delta.same_turn_score_window_value,
        exact_context.delta.opponent_window_deny_gain,
        exact_context.delta.drainer_attack_available,
        approved.own_drainer_vulnerable,
        shipping.own_drainer_vulnerable,
        shipping.score >= approved.score,
        Input::fen_from_array(&pro_v1_candidate_selected),
        probe_legacy_selected,
        probe_legacy_full_pool_selected,
        shipping_selected,
        runtime_probe,
        shipping_runtime_probe,
    );
}

fn log_black_confirm_fast_runtime_seam_probe(
    label: &str,
    board_fen: &str,
    frontier_move: &str,
    shipping_move: &str,
) {
    let game = MonsGame::from_fen(board_fen, false).expect("valid black confirm fast seam fen");
    let perspective = game.active_color;
    let frontier_probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    let shipping_runtime_probe =
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
    let projections = MonsGameModel::turn_engine_reply_risk_projections(
        scored_roots.as_slice(),
        shortlist.as_slice(),
        perspective,
        config,
    );
    let frontier_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == frontier_move)
        .expect("frontier move should exist");
    let frontier_pre_accept_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == frontier_probe.pre_accept_input_fen)
        .expect("frontier pre-accept root should exist");
    let shipping_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == shipping_move)
        .expect("shipping move should exist");
    let (probe_legacy_selected, probe_legacy_full_pool_selected, _, _) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);
    let mut production_legacy_config = config;
    production_legacy_config.enable_root_reply_risk_guard = false;
    production_legacy_config.turn_engine_mode = TurnEngineMode::ProV1;
    let production_legacy_selected =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            &game,
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            perspective,
            production_legacy_config,
        );
    let direct_legacy_alignment =
        MonsGameModel::pro_v2_root_advisor_black_legacy_alignment_override(
            &game,
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            frontier_index,
            shipping_index,
            config,
        )
        .map(|index| Input::fen_from_array(&scored_roots[index].inputs));
    let frontier = &scored_roots[frontier_index];
    let frontier_pre_accept = &scored_roots[frontier_pre_accept_index];
    let shipping = &scored_roots[shipping_index];
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(shortlist.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let frontier_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        frontier,
        projections.get(&frontier_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let frontier_pre_accept_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        frontier_pre_accept,
        projections.get(&frontier_pre_accept_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let shipping_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        shipping,
        projections.get(&shipping_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let frontier_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        frontier,
        perspective,
        config,
        MonsGameModel::turn_engine_root_evaluation_family(frontier),
    );
    let frontier_pre_accept_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        frontier_pre_accept,
        perspective,
        config,
        MonsGameModel::turn_engine_root_evaluation_family(frontier_pre_accept),
    );
    let shipping_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        shipping,
        perspective,
        config,
        MonsGameModel::turn_engine_root_evaluation_family(shipping),
    );
    let shipping_beats_frontier = MonsGameModel::is_better_reply_risk_candidate(
        &game,
        shipping_index,
        shipping_snapshot,
        frontier_index,
        frontier_snapshot,
        projections.get(&shipping_index),
        projections.get(&frontier_index),
        scored_roots.as_slice(),
        perspective,
        config,
        &mut std::collections::HashMap::new(),
    );
    let frontier_beats_shipping = MonsGameModel::is_better_reply_risk_candidate(
        &game,
        frontier_index,
        frontier_snapshot,
        shipping_index,
        shipping_snapshot,
        projections.get(&frontier_index),
        projections.get(&shipping_index),
        scored_roots.as_slice(),
        perspective,
        config,
        &mut std::collections::HashMap::new(),
    );
    let frontier_non_tactical = !frontier.wins_immediately
        && !frontier.attacks_opponent_drainer
        && !frontier.scores_supermana_this_turn
        && !frontier.scores_opponent_mana_this_turn
        && !frontier.safe_supermana_pickup_now
        && !frontier.safe_opponent_mana_pickup_now
        && !frontier.mana_handoff_to_opponent
        && !frontier.has_roundtrip;
    let frontier_pre_accept_non_tactical = !frontier_pre_accept.wins_immediately
        && !frontier_pre_accept.attacks_opponent_drainer
        && !frontier_pre_accept.scores_supermana_this_turn
        && !frontier_pre_accept.scores_opponent_mana_this_turn
        && !frontier_pre_accept.safe_supermana_pickup_now
        && !frontier_pre_accept.safe_opponent_mana_pickup_now
        && !frontier_pre_accept.mana_handoff_to_opponent
        && !frontier_pre_accept.has_roundtrip;
    let shipping_non_tactical = !shipping.wins_immediately
        && !shipping.attacks_opponent_drainer
        && !shipping.scores_supermana_this_turn
        && !shipping.scores_opponent_mana_this_turn
        && !shipping.safe_supermana_pickup_now
        && !shipping.safe_opponent_mana_pickup_now
        && !shipping.mana_handoff_to_opponent
        && !shipping.has_roundtrip;
    let shortlist_roots = shortlist
        .iter()
        .map(|index| {
            format!(
                "{} => {}",
                Input::fen_from_array(&scored_roots[*index].inputs),
                format_root_probe(scored_roots.get(*index)),
            )
        })
        .collect::<Vec<_>>();

    println!(
        "{} context={} frontier_move={} shipping_move={} shipping_selected={} frontier_probe={:?} shipping_runtime_probe={:?} advisor={:?} frontier={} frontier_pre_accept={} shipping={} frontier_snapshot={} frontier_pre_accept_snapshot={} shipping_snapshot={} frontier_utility={} frontier_pre_accept_utility={} shipping_utility={} shipping_vs_frontier={} frontier_vs_shipping={} frontier_projection={:?} frontier_pre_accept_projection={:?} shipping_projection={:?} frontier_family={:?} frontier_pre_accept_family={:?} shipping_family={:?} frontier_progress_surface={} frontier_pre_accept_progress_surface={} shipping_progress_surface={} frontier_plain_spirit={} frontier_pre_accept_plain_spirit={} shipping_plain_spirit={} frontier_non_tactical={} frontier_pre_accept_non_tactical={} shipping_non_tactical={} candidate_contains_shipping={} shortlist={:?} shortlist_roots={:?} legacy_candidate_selected={} legacy_full_pool_selected={} production_legacy_selected={} direct_legacy_alignment={:?}",
        label,
        exact_opportunity_context_probe(&game),
        frontier_move,
        shipping_move,
        shipping_selected,
        frontier_probe,
        shipping_runtime_probe,
        frontier_advisor,
        format_root_probe(scored_roots.get(frontier_index)),
        format_root_probe(scored_roots.get(frontier_pre_accept_index)),
        format_root_probe(scored_roots.get(shipping_index)),
        format!(
            "win={} match_point={} floor={}",
            frontier_snapshot.allows_immediate_opponent_win,
            frontier_snapshot.opponent_reaches_match_point,
            frontier_snapshot.worst_reply_score,
        ),
        format!(
            "win={} match_point={} floor={}",
            frontier_pre_accept_snapshot.allows_immediate_opponent_win,
            frontier_pre_accept_snapshot.opponent_reaches_match_point,
            frontier_pre_accept_snapshot.worst_reply_score,
        ),
        format!(
            "win={} match_point={} floor={}",
            shipping_snapshot.allows_immediate_opponent_win,
            shipping_snapshot.opponent_reaches_match_point,
            shipping_snapshot.worst_reply_score,
        ),
        format_turn_engine_utility_probe(frontier_utility),
        format_turn_engine_utility_probe(frontier_pre_accept_utility),
        format_turn_engine_utility_probe(shipping_utility),
        shipping_beats_frontier,
        frontier_beats_shipping,
        projections.get(&frontier_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        projections.get(&frontier_pre_accept_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        projections.get(&shipping_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        MonsGameModel::turn_engine_root_evaluation_family(frontier),
        MonsGameModel::turn_engine_root_evaluation_family(frontier_pre_accept),
        MonsGameModel::turn_engine_root_evaluation_family(shipping),
        MonsGameModel::turn_engine_root_evaluation_has_progress_surface(frontier),
        MonsGameModel::turn_engine_root_evaluation_has_progress_surface(frontier_pre_accept),
        MonsGameModel::turn_engine_root_evaluation_has_progress_surface(shipping),
        MonsGameModel::is_plain_spirit_development_root(frontier),
        MonsGameModel::is_plain_spirit_development_root(frontier_pre_accept),
        MonsGameModel::is_plain_spirit_development_root(shipping),
        frontier_non_tactical,
        frontier_pre_accept_non_tactical,
        shipping_non_tactical,
        candidate_indices.contains(&shipping_index),
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        shortlist_roots,
        probe_legacy_selected,
        probe_legacy_full_pool_selected,
        Input::fen_from_array(&production_legacy_selected),
        direct_legacy_alignment,
    );
}

#[test]
#[ignore = "diagnostic: inspect remaining late black confirm fast lane split seam"]
fn black_confirm_fast_lane_split_probe() {
    log_black_confirm_fast_runtime_seam_probe(
        "BLACK_CONFIRM_FAST_LANE_SPLIT",
        "1 1 b 1 0 0 0 0 8 d0xn10/n05s0xa0xe0xn03/n05xxmn06/n05xxmn01xxmn02/n05xxmn05/n05xxUn04xxQ/n02S0xn01xxMn06/n01y0xn04xxMxxMn03/n01E0xA0xn08/n01D0Mn09/n08Y0xn02",
        "l0,0;l1,1",
        "l7,1;l8,0",
    );
}

#[test]
#[ignore = "diagnostic: inspect later black pro lane split seam"]
fn black_pro_lane_split_probe() {
    let game = MonsGame::from_fen(
        "2 0 b 0 0 0 0 0 10 n09xxmn01/n06a0xn04/n05s0xd0xe0xn03/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n01y0xn03xxMn05/n11/n05S0xn01xxMn03/n03A0xn05xxMn01/D0xn02E0xn07",
        false,
    )
    .expect("valid later black pro lane split fen");
    let perspective = game.active_color;
    let frontier_probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
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
    let projections = MonsGameModel::turn_engine_reply_risk_projections(
        scored_roots.as_slice(),
        shortlist.as_slice(),
        perspective,
        config,
    );
    let frontier_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == frontier_probe.selected_input_fen)
        .expect("frontier root should exist");
    let frontier_pre_accept_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == frontier_probe.pre_accept_input_fen)
        .expect("frontier pre-accept root should exist");
    let frontier_head_index = frontier_probe.head_input_fen.as_ref().and_then(|inputs| {
        scored_roots
            .iter()
            .position(|root| Input::fen_from_array(&root.inputs) == *inputs)
    });
    let shipping_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == shipping_probe.selected_input_fen)
        .expect("shipping root should exist");
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(shortlist.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let frontier_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[frontier_index],
        projections.get(&frontier_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let frontier_pre_accept_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[frontier_pre_accept_index],
        projections.get(&frontier_pre_accept_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let frontier_head_snapshot = frontier_head_index.map(|index| {
        MonsGameModel::root_reply_risk_snapshot_with_projection(
            &scored_roots[index],
            projections.get(&index),
            perspective,
            config,
            per_root_reply_limit,
        )
    });
    let shipping_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[shipping_index],
        projections.get(&shipping_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let frontier_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[frontier_index]);
    let frontier_pre_accept_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[frontier_pre_accept_index]);
    let frontier_head_family = frontier_head_index
        .map(|index| MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[index]));
    let shipping_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[shipping_index]);
    let frontier_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[frontier_index],
        perspective,
        config,
        frontier_family,
    );
    let frontier_pre_accept_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[frontier_pre_accept_index],
        perspective,
        config,
        frontier_pre_accept_family,
    );
    let frontier_head_utility = frontier_head_index.map(|index| {
        MonsGameModel::turn_engine_selected_override_utility(
            &game,
            &scored_roots[index],
            perspective,
            config,
            frontier_head_family.expect("head family should exist"),
        )
    });
    let shipping_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[shipping_index],
        perspective,
        config,
        shipping_family,
    );
    let shipping_beats_frontier = MonsGameModel::is_better_reply_risk_candidate(
        &game,
        shipping_index,
        shipping_snapshot,
        frontier_index,
        frontier_snapshot,
        projections.get(&shipping_index),
        projections.get(&frontier_index),
        scored_roots.as_slice(),
        perspective,
        config,
        &mut std::collections::HashMap::new(),
    );
    let frontier_head_beats_frontier = frontier_head_index.map(|index| {
        MonsGameModel::is_better_reply_risk_candidate(
            &game,
            index,
            frontier_head_snapshot.expect("head snapshot should exist"),
            frontier_index,
            frontier_snapshot,
            projections.get(&index),
            projections.get(&frontier_index),
            scored_roots.as_slice(),
            perspective,
            config,
            &mut std::collections::HashMap::new(),
        )
    });

    println!(
        "BLACK_PRO_LANE_SPLIT context={} shortlist={:?} frontier_probe={:?} shipping_probe={:?} advisor={:?} frontier={} frontier_pre_accept={} frontier_head={} shipping={} frontier_snapshot={} frontier_pre_accept_snapshot={} frontier_head_snapshot={:?} shipping_snapshot={} frontier_utility={} frontier_pre_accept_utility={} frontier_head_utility={:?} shipping_utility={} shipping_vs_frontier={} frontier_head_vs_frontier={:?} frontier_projection={:?} frontier_pre_accept_projection={:?} frontier_head_projection={:?} shipping_projection={:?}",
        exact_opportunity_context_probe(&game),
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        frontier_probe,
        shipping_probe,
        frontier_advisor,
        format_root_probe(scored_roots.get(frontier_index)),
        format_root_probe(scored_roots.get(frontier_pre_accept_index)),
        frontier_head_index
            .and_then(|index| scored_roots.get(index))
            .map(|root| format_root_probe(Some(root)))
            .unwrap_or_else(|| "none".to_string()),
        format_root_probe(scored_roots.get(shipping_index)),
        format!(
            "win={} match_point={} floor={}",
            frontier_snapshot.allows_immediate_opponent_win,
            frontier_snapshot.opponent_reaches_match_point,
            frontier_snapshot.worst_reply_score,
        ),
        format!(
            "win={} match_point={} floor={}",
            frontier_pre_accept_snapshot.allows_immediate_opponent_win,
            frontier_pre_accept_snapshot.opponent_reaches_match_point,
            frontier_pre_accept_snapshot.worst_reply_score,
        ),
        frontier_head_snapshot.map(|snapshot| {
            format!(
                "win={} match_point={} floor={}",
                snapshot.allows_immediate_opponent_win,
                snapshot.opponent_reaches_match_point,
                snapshot.worst_reply_score,
            )
        }),
        format!(
            "win={} match_point={} floor={}",
            shipping_snapshot.allows_immediate_opponent_win,
            shipping_snapshot.opponent_reaches_match_point,
            shipping_snapshot.worst_reply_score,
        ),
        format_turn_engine_utility_probe(frontier_utility),
        format_turn_engine_utility_probe(frontier_pre_accept_utility),
        frontier_head_utility.map(format_turn_engine_utility_probe),
        format_turn_engine_utility_probe(shipping_utility),
        shipping_beats_frontier,
        frontier_head_beats_frontier,
        projections.get(&frontier_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        projections.get(&frontier_pre_accept_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        frontier_head_index.and_then(|index| {
            projections.get(&index).map(|projection| {
                format!(
                    "{:?}/{:?}/{}",
                    projection.plan.head_family,
                    projection.plan.goal_family,
                    format_turn_engine_utility_probe(projection.plan.utility),
                )
            })
        }),
        projections.get(&shipping_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
    );
}

#[test]
#[ignore = "diagnostic: compare remaining black shipping-disabled residual ordering under frontier and shipping configs"]
fn black_shipping_disabled_ordering_surface_probe() {
    #[derive(Clone, Copy)]
    struct ResidualCase {
        label: &'static str,
        board_fen: &'static str,
        frontier_root: &'static str,
        shipping_root: &'static str,
        extra_roots: &'static [&'static str],
    }

    fn snapshot_probe(
        root: &RootEvaluation,
        projection: Option<&TurnEngineRootProjection>,
        perspective: Color,
        config: AutomoveSearchConfig,
        reply_limit: usize,
    ) -> String {
        let snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
            root,
            projection,
            perspective,
            config,
            reply_limit,
        );
        format!(
            "win={} match_point={} floor={}",
            snapshot.allows_immediate_opponent_win,
            snapshot.opponent_reaches_match_point,
            snapshot.worst_reply_score,
        )
    }

    fn pairwise_probe(
        game: &MonsGame,
        scored_roots: &[RootEvaluation],
        projections: &std::collections::HashMap<usize, TurnEngineRootProjection>,
        perspective: Color,
        config: AutomoveSearchConfig,
        reply_limit: usize,
        challenger_index: usize,
        incumbent_index: usize,
    ) -> String {
        let challenger_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
            &scored_roots[challenger_index],
            projections.get(&challenger_index),
            perspective,
            config,
            reply_limit,
        );
        let incumbent_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
            &scored_roots[incumbent_index],
            projections.get(&incumbent_index),
            perspective,
            config,
            reply_limit,
        );
        let better = MonsGameModel::is_better_reply_risk_candidate(
            game,
            challenger_index,
            challenger_snapshot,
            incumbent_index,
            incumbent_snapshot,
            projections.get(&challenger_index),
            projections.get(&incumbent_index),
            scored_roots,
            perspective,
            config,
            &mut std::collections::HashMap::new(),
        );
        format!(
            "{}_vs_{}={} floors={}/{}",
            Input::fen_from_array(&scored_roots[challenger_index].inputs),
            Input::fen_from_array(&scored_roots[incumbent_index].inputs),
            better,
            challenger_snapshot.worst_reply_score,
            incumbent_snapshot.worst_reply_score,
        )
    }

    fn profile_surface(case: ResidualCase, profile: &str) -> String {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{} should have a valid fen", case.label));
        let perspective = game.active_color;
        let probe = runtime_decision_probe(profile, SmartAutomovePreference::Pro, &game);
        let (config, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            profile,
            SmartAutomovePreference::Pro,
            &game,
        );
        let candidate_indices = MonsGameModel::filtered_root_candidate_indices(
            &game,
            scored_roots.as_slice(),
            perspective,
            config,
        );
        let all_indices = (0..scored_roots.len()).collect::<Vec<_>>();
        let spirit_setup_indices = all_indices
            .iter()
            .copied()
            .filter(|index| {
                let root = &scored_roots[*index];
                root.spirit_own_mana_setup_now
                    && !root.own_drainer_vulnerable
                    && !root.mana_handoff_to_opponent
            })
            .collect::<Vec<_>>();
        let spirit_setup_filter_surface = if spirit_setup_indices.is_empty() {
            "spirit_setup_candidates=[]".to_string()
        } else {
            format!(
                "spirit_setup_candidates={:?} progress_competes={} followup_progress_competes={} risky_score_competes={} negative_deny_competes={} score_competes={} projection_competes={} risky_recovery_competes={}",
                spirit_setup_indices
                    .iter()
                    .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
                    .collect::<Vec<_>>(),
                MonsGameModel::safe_progress_competes_with_spirit_pref(
                    scored_roots.as_slice(),
                    all_indices.as_slice(),
                    config.turn_engine_mode,
                ),
                MonsGameModel::followup_progress_competes_with_spirit_pref(
                    &game,
                    scored_roots.as_slice(),
                    all_indices.as_slice(),
                    perspective,
                    config,
                ),
                MonsGameModel::risky_score_competes_with_spirit_pref(
                    scored_roots.as_slice(),
                    all_indices.as_slice(),
                    config.turn_engine_mode,
                ),
                MonsGameModel::negative_deny_competes_with_spirit_pref(
                    scored_roots.as_slice(),
                    all_indices.as_slice(),
                    perspective,
                    config,
                ),
                MonsGameModel::score_competes_with_spirit_pref(
                    scored_roots.as_slice(),
                    all_indices.as_slice(),
                    config.turn_engine_mode,
                ),
                MonsGameModel::projection_competes_with_spirit_pref(
                    scored_roots.as_slice(),
                    all_indices.as_slice(),
                    perspective,
                    config,
                ),
                MonsGameModel::risky_recovery_competes_with_spirit_pref(
                    &game,
                    scored_roots.as_slice(),
                    all_indices.as_slice(),
                    perspective,
                    config,
                ),
            )
        };
        let shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            config,
        );
        let projections = MonsGameModel::turn_engine_reply_risk_projections(
            scored_roots.as_slice(),
            shortlist.as_slice(),
            perspective,
            config,
        );
        let root_node_budget = ((config.max_visited_nodes
            * config.root_reply_risk_node_share_bp.max(0) as usize)
            / 10_000)
            .max(shortlist.len())
            .max(1);
        let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
            .max(1)
            .min(config.root_reply_risk_reply_limit.max(1));
        let reply_guard_selected = MonsGameModel::pick_root_move_with_reply_risk_guard(
            &game,
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            perspective,
            config,
        )
        .map(|index| Input::fen_from_array(&scored_roots[index].inputs));
        let final_pick = MonsGameModel::pick_root_move_with_exploration(
            &game,
            scored_roots.as_slice(),
            perspective,
            config,
        );
        let mut target_fens = vec![case.frontier_root, case.shipping_root];
        for extra in case.extra_roots {
            if !target_fens.contains(extra) {
                target_fens.push(extra);
            }
        }
        let target_indices = target_fens
            .iter()
            .map(|target| {
                (
                    *target,
                    scored_roots
                        .iter()
                        .position(|root| Input::fen_from_array(&root.inputs) == *target),
                )
            })
            .collect::<Vec<_>>();
        let target_details = target_indices
            .iter()
            .map(|(target, maybe_index)| {
                let Some(index) = maybe_index else {
                    return format!("{} => missing", target);
                };
                let root = &scored_roots[*index];
                let family = MonsGameModel::turn_engine_root_evaluation_family(root);
                let utility = MonsGameModel::turn_engine_selected_override_utility(
                    &game,
                    root,
                    perspective,
                    config,
                    family,
                );
                format!(
                    "{} => index={} candidate_pos={:?} shortlist_pos={:?} family={:?} utility={} snapshot={} projection={:?} {}",
                    target,
                    index,
                    candidate_indices.iter().position(|candidate| candidate == index),
                    shortlist.iter().position(|candidate| candidate == index),
                    family,
                    format_turn_engine_utility_probe(utility),
                    snapshot_probe(
                        root,
                        projections.get(index),
                        perspective,
                        config,
                        per_root_reply_limit,
                    ),
                    projections.get(index).map(|projection| {
                        format!(
                            "{:?}/{:?}/{}",
                            projection.plan.head_family,
                            projection.plan.goal_family,
                            format_turn_engine_utility_probe(projection.plan.utility),
                        )
                    }),
                    format_root_probe(Some(root)),
                )
            })
            .collect::<Vec<_>>();
        let mut pairwise = Vec::new();
        for (left_label, left_index) in target_indices.iter() {
            let Some(left_index) = *left_index else {
                continue;
            };
            for (right_label, right_index) in target_indices.iter() {
                if left_label == right_label {
                    continue;
                }
                let Some(right_index) = *right_index else {
                    continue;
                };
                pairwise.push(pairwise_probe(
                    &game,
                    scored_roots.as_slice(),
                    &projections,
                    perspective,
                    config,
                    per_root_reply_limit,
                    left_index,
                    right_index,
                ));
            }
        }
        let shortlist_details = shortlist
            .iter()
            .map(|index| {
                format!(
                    "{} => {}",
                    Input::fen_from_array(&scored_roots[*index].inputs),
                    format_root_probe(scored_roots.get(*index)),
                )
            })
            .collect::<Vec<_>>();

        format!(
            "profile={} config(selector={} head_rerank={} mode={:?} reply_guard={} margin={} shortlist_max={} reply_limit={} node_share_bp={}) probe={:?} final_pick={} reply_guard_selected={:?} candidate_count={} spirit_setup_filter_surface={} shortlist={:?} shortlist_details={:?} target_details={:?} pairwise={:?}",
            profile,
            config.enable_turn_engine_selector,
            config.enable_turn_head_rerank,
            config.turn_engine_mode,
            config.enable_root_reply_risk_guard,
            config.root_reply_risk_score_margin,
            config.root_reply_risk_shortlist_max,
            config.root_reply_risk_reply_limit,
            config.root_reply_risk_node_share_bp,
            probe,
            Input::fen_from_array(&final_pick),
            reply_guard_selected,
            candidate_indices.len(),
            spirit_setup_filter_surface,
            shortlist
                .iter()
                .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
                .collect::<Vec<_>>(),
            shortlist_details,
            target_details,
            pairwise,
        )
    }

    let cases = [
        ResidualCase {
            label: "black_recovery_branch",
            board_fen: "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
            frontier_root: "l1,5;l3,3;l2,3",
            shipping_root: "l6,0;l6,1",
            extra_roots: &["l1,5;l2,7;l1,8", "l6,0;l7,0"],
        },
        ResidualCase {
            label: "black_progress_vs_setup_residue",
            board_fen: "1 0 b 0 0 0 0 0 6 n05d0xn05/n05s0xa0xe0xn03/n02xxmn04xxmn03/n07xxmn03/n03xxmn01xxmn05/n05xxUn04xxQ/n05xxMn05/n01y0xn01S0xE0xn01xxMxxMn03/n01xxMn09/n03A0xn03Y0xn03/n05D1xn05",
            frontier_root: "l7,1;l9,3",
            shipping_root: "l1,5;l2,7;l1,8",
            extra_roots: &["l1,5;l3,7;l2,8"],
        },
    ];

    for case in cases {
        println!(
            "BLACK_SHIPPING_DISABLED_ORDERING label={} context={} frontier_surface={{ {} }} shipping_surface={{ {} }}",
            case.label,
            exact_opportunity_context_probe(
                &MonsGame::from_fen(case.board_fen, false)
                    .expect("case board should be valid")
            ),
            profile_surface(case, "frontier_pro_v2_guarded"),
            profile_surface(case, "shipping_pro_search"),
        );
    }
}

#[test]
#[ignore = "diagnostic: scope ProV2 progress competition predicates on black setup/progress boards"]
fn black_progress_competition_predicate_scope_probe() {
    #[derive(Clone, Copy)]
    struct ScopeCase {
        label: &'static str,
        board_fen: &'static str,
        expected_current: &'static str,
        shipping_root: &'static str,
    }

    fn competition_summary(
        game: &MonsGame,
        scored_roots: &[RootEvaluation],
        indices: &[usize],
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> String {
        format!(
            "progress_competes={} followup_progress_competes={} risky_score_competes={} negative_deny_competes={} score_competes={} projection_competes={} risky_recovery_competes={}",
            MonsGameModel::safe_progress_competes_with_spirit_pref(
                scored_roots,
                indices,
                config.turn_engine_mode,
            ),
            MonsGameModel::followup_progress_competes_with_spirit_pref(
                game,
                scored_roots,
                indices,
                perspective,
                config,
            ),
            MonsGameModel::risky_score_competes_with_spirit_pref(
                scored_roots,
                indices,
                config.turn_engine_mode,
            ),
            MonsGameModel::negative_deny_competes_with_spirit_pref(
                scored_roots,
                indices,
                perspective,
                config,
            ),
            MonsGameModel::score_competes_with_spirit_pref(
                scored_roots,
                indices,
                config.turn_engine_mode,
            ),
            MonsGameModel::projection_competes_with_spirit_pref(
                scored_roots,
                indices,
                perspective,
                config,
            ),
            MonsGameModel::risky_recovery_competes_with_spirit_pref(
                game,
                scored_roots,
                indices,
                perspective,
                config,
            ),
        )
    }

    fn root_set_details(scored_roots: &[RootEvaluation], indices: &[usize]) -> Vec<String> {
        indices
            .iter()
            .map(|index| {
                format!(
                    "{} => {}",
                    Input::fen_from_array(&scored_roots[*index].inputs),
                    format_root_probe(scored_roots.get(*index)),
                )
            })
            .collect()
    }

    fn surface_for_case(case: ScopeCase) -> String {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{} should have a valid fen", case.label));
        let perspective = game.active_color;
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
        let (frontier_config, frontier_roots, _, _) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "frontier_pro_v2_guarded",
                SmartAutomovePreference::Pro,
                &game,
            );
        let all_indices = (0..frontier_roots.len()).collect::<Vec<_>>();
        let filtered_indices = MonsGameModel::filtered_root_candidate_indices(
            &game,
            frontier_roots.as_slice(),
            perspective,
            frontier_config,
        );
        let shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
            frontier_roots.as_slice(),
            filtered_indices.as_slice(),
            frontier_config,
        );
        let mut prov1_filter_config = frontier_config;
        prov1_filter_config.turn_engine_mode = TurnEngineMode::ProV1;
        let prov1_filtered_indices = MonsGameModel::filtered_root_candidate_indices(
            &game,
            frontier_roots.as_slice(),
            perspective,
            prov1_filter_config,
        );
        let prov1_filter_pick =
            MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
                &game,
                frontier_roots.as_slice(),
                prov1_filtered_indices.as_slice(),
                perspective,
                prov1_filter_config,
            );
        let spirit_setup_indices = all_indices
            .iter()
            .copied()
            .filter(|index| {
                let root = &frontier_roots[*index];
                root.spirit_own_mana_setup_now
                    && !root.own_drainer_vulnerable
                    && !root.mana_handoff_to_opponent
            })
            .collect::<Vec<_>>();
        let safe_progress_indices = all_indices
            .iter()
            .copied()
            .filter(|index| {
                let root = &frontier_roots[*index];
                !MonsGameModel::turn_engine_root_evaluation_is_unsafe(root)
                    && MonsGameModel::turn_engine_root_evaluation_has_progress_surface(root)
                    && !root.spirit_development
                    && !root.spirit_same_turn_score_setup_now
                    && !root.spirit_own_mana_setup_now
            })
            .collect::<Vec<_>>();
        let expected_index = frontier_roots
            .iter()
            .position(|root| Input::fen_from_array(&root.inputs) == case.expected_current);
        let shipping_index = frontier_roots
            .iter()
            .position(|root| Input::fen_from_array(&root.inputs) == case.shipping_root);

        format!(
            "label={} context={} expected_current={} shipping_root={} frontier_selected={} shipping_selected={} frontier_competition_all=[{}] frontier_competition_filtered=[{}] prov1_filter_competition_all=[{}] filtered={:?} prov1_filtered={:?} shortlist={:?} prov1_filter_pick={} expected_details={} shipping_details={} spirit_setup_roots={:?} safe_progress_roots={:?} advisor={:?}",
            case.label,
            exact_opportunity_context_probe(&game),
            case.expected_current,
            case.shipping_root,
            frontier_probe.selected_input_fen,
            shipping_probe.selected_input_fen,
            competition_summary(
                &game,
                frontier_roots.as_slice(),
                all_indices.as_slice(),
                perspective,
                frontier_config,
            ),
            competition_summary(
                &game,
                frontier_roots.as_slice(),
                filtered_indices.as_slice(),
                perspective,
                frontier_config,
            ),
            competition_summary(
                &game,
                frontier_roots.as_slice(),
                all_indices.as_slice(),
                perspective,
                prov1_filter_config,
            ),
            filtered_indices
                .iter()
                .map(|index| Input::fen_from_array(&frontier_roots[*index].inputs))
                .collect::<Vec<_>>(),
            prov1_filtered_indices
                .iter()
                .map(|index| Input::fen_from_array(&frontier_roots[*index].inputs))
                .collect::<Vec<_>>(),
            shortlist
                .iter()
                .map(|index| Input::fen_from_array(&frontier_roots[*index].inputs))
                .collect::<Vec<_>>(),
            Input::fen_from_array(&prov1_filter_pick),
            format_root_probe(expected_index.and_then(|index| frontier_roots.get(index))),
            format_root_probe(shipping_index.and_then(|index| frontier_roots.get(index))),
            root_set_details(frontier_roots.as_slice(), spirit_setup_indices.as_slice()),
            root_set_details(frontier_roots.as_slice(), safe_progress_indices.as_slice()),
            frontier_advisor,
        )
    }

    let cases = [
        ScopeCase {
            label: "black_progress_vs_setup_residue",
            board_fen: "1 0 b 0 0 0 0 0 6 n05d0xn05/n05s0xa0xe0xn03/n02xxmn04xxmn03/n07xxmn03/n03xxmn01xxmn05/n05xxUn04xxQ/n05xxMn05/n01y0xn01S0xE0xn01xxMxxMn03/n01xxMn09/n03A0xn03Y0xn03/n05D1xn05",
            expected_current: "l7,1;l9,3",
            shipping_root: "l1,5;l2,7;l1,8",
        },
        ScopeCase {
            label: "black_late_setup_reply_risk",
            board_fen: "1 1 b 0 0 0 0 0 8 d0xn10/n05s0xa0xe0xxxmn02/n11/n07xxmn03/n03xxmn02xxmn04/n10xxQ/n02y0xn01D0UxxMn01xxMn03/n02xxMS0xn01A0xxxMn04/n06Y0xn04/n03E0xn07/n11",
            expected_current: "l1,5;l3,7;l2,8",
            shipping_root: "l1,5;l3,7;l2,8",
        },
        ScopeCase {
            label: "black_confirm_fast_setup",
            board_fen: "2 1 b 0 0 0 0 0 10 n05d0xn03xxmn01/n04a0xn02e0xn03/n05s0xn05/E0xn02xxmn03xxmn03/n05xxmn05/n05xxUn04xxQ/n05xxMn05/n03S0xn01Y0xxxMn04/n03y0xn04xxMn02/n03A0xn07/n05D1xn05",
            expected_current: "l2,5;l3,7;l2,8",
            shipping_root: "l2,5;l3,7;l2,8",
        },
    ];

    for case in cases {
        println!(
            "BLACK_PROGRESS_COMPETITION_SCOPE {}",
            surface_for_case(case)
        );
    }
}

#[test]
#[ignore = "diagnostic: quantify black progress-vs-setup shortlist economics"]
fn black_progress_setup_shortlist_economics_probe() {
    #[derive(Clone, Copy)]
    struct EconomicsCase {
        label: &'static str,
        board_fen: &'static str,
        frontier_root: &'static str,
        shipping_root: &'static str,
        reference_setup_roots: &'static [&'static str],
    }

    fn root_fens(scored_roots: &[RootEvaluation], indices: &[usize]) -> Vec<String> {
        indices
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect()
    }

    fn root_index(scored_roots: &[RootEvaluation], root_fen: &str) -> Option<usize> {
        scored_roots
            .iter()
            .position(|root| Input::fen_from_array(&root.inputs) == root_fen)
    }

    fn push_unique(indices: &mut Vec<usize>, index: Option<usize>) {
        if let Some(index) = index {
            if !indices.contains(&index) {
                indices.push(index);
            }
        }
    }

    fn root_economics(
        game: &MonsGame,
        scored_roots: &[RootEvaluation],
        index: usize,
        candidate_indices: &[usize],
        shortlist: &[usize],
        perspective: Color,
        config: AutomoveSearchConfig,
        best_score: i32,
        best_rank: usize,
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
        let reply_limit = config.root_reply_risk_reply_limit.max(1).min(24);
        let snapshot =
            MonsGameModel::root_reply_risk_snapshot(&root.game, perspective, config, reply_limit);
        let followup =
            MonsGameModel::pro_v2_spirit_followup_floor_score(&root.game, perspective, config);

        format!(
            "{} => score_gap={} rank={} rank_gap={} family={:?} in_candidate={} in_shortlist={} utility={:?} worst_reply={} match_point={} immediate_win={} followup={} setup_gain={} super_steps={} opp_steps={} {}",
            Input::fen_from_array(&root.inputs),
            best_score.saturating_sub(root.score),
            root.root_rank,
            root.root_rank.abs_diff(best_rank),
            family,
            candidate_indices.contains(&index),
            shortlist.contains(&index),
            utility,
            snapshot.worst_reply_score,
            snapshot.opponent_reaches_match_point,
            snapshot.allows_immediate_opponent_win,
            followup,
            root.spirit_setup_gain,
            root.safe_supermana_progress_steps,
            root.safe_opponent_mana_progress_steps,
            format_root_probe(Some(root)),
        )
    }

    fn margin_sweep(
        game: &MonsGame,
        scored_roots: &[RootEvaluation],
        perspective: Color,
        config: AutomoveSearchConfig,
        target_margins: &[i32],
    ) -> Vec<String> {
        let mut margins = target_margins.to_vec();
        margins.extend([
            config.root_reply_risk_score_margin,
            256,
            384,
            512,
            768,
            1024,
        ]);
        margins.sort();
        margins.dedup();

        margins
            .into_iter()
            .map(|margin| {
                let mut margin_config = config;
                margin_config.root_reply_risk_score_margin = margin;
                let candidate_indices = MonsGameModel::filtered_root_candidate_indices(
                    game,
                    scored_roots,
                    perspective,
                    margin_config,
                );
                let shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
                    scored_roots,
                    candidate_indices.as_slice(),
                    margin_config,
                );
                let guard_pick = MonsGameModel::pick_root_move_with_reply_risk_guard(
                    game,
                    scored_roots,
                    candidate_indices.as_slice(),
                    perspective,
                    margin_config,
                )
                .map(|index| Input::fen_from_array(&scored_roots[index].inputs))
                .unwrap_or_else(|| "none".to_string());
                let advisor_pick = MonsGameModel::pro_v2_root_advisor_select_root(
                    game,
                    scored_roots,
                    perspective,
                    margin_config,
                );

                format!(
                    "margin={} shortlist={:?} guard_pick={} advisor_pick={}",
                    margin,
                    root_fens(scored_roots, shortlist.as_slice()),
                    guard_pick,
                    Input::fen_from_array(&advisor_pick),
                )
            })
            .collect()
    }

    fn surface_for_case(case: EconomicsCase) -> String {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{} should have a valid fen", case.label));
        let perspective = game.active_color;
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
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
        let best_score = candidate_indices
            .iter()
            .map(|index| scored_roots[*index].score)
            .max()
            .unwrap_or(i32::MIN);
        let best_rank = candidate_indices
            .iter()
            .map(|index| scored_roots[*index].root_rank)
            .min()
            .unwrap_or(usize::MAX);
        let safe_progress_indices = candidate_indices
            .iter()
            .copied()
            .filter(|index| {
                let root = &scored_roots[*index];
                !MonsGameModel::turn_engine_root_evaluation_is_unsafe(root)
                    && MonsGameModel::turn_engine_root_evaluation_has_progress_surface(root)
                    && !root.spirit_development
                    && !root.spirit_same_turn_score_setup_now
                    && !root.spirit_own_mana_setup_now
            })
            .collect::<Vec<_>>();
        let setup_progress_indices = candidate_indices
            .iter()
            .copied()
            .filter(|index| {
                let root = &scored_roots[*index];
                MonsGameModel::turn_engine_root_evaluation_family(root)
                    == TurnPlanFamily::SpiritImpact
                    && root.spirit_own_mana_setup_now
                    && !root.spirit_same_turn_score_setup_now
                    && MonsGameModel::turn_engine_root_evaluation_has_progress_surface(root)
                    && !MonsGameModel::turn_engine_root_evaluation_is_unsafe(root)
            })
            .collect::<Vec<_>>();
        let frontier_index = root_index(scored_roots.as_slice(), case.frontier_root);
        let shipping_index = root_index(scored_roots.as_slice(), case.shipping_root);
        let mut detail_indices = Vec::new();
        push_unique(&mut detail_indices, frontier_index);
        push_unique(&mut detail_indices, shipping_index);
        for root_fen in case.reference_setup_roots {
            push_unique(
                &mut detail_indices,
                root_index(scored_roots.as_slice(), root_fen),
            );
        }
        for index in safe_progress_indices.iter().copied() {
            push_unique(&mut detail_indices, Some(index));
        }
        for index in setup_progress_indices.iter().copied().take(8) {
            push_unique(&mut detail_indices, Some(index));
        }
        detail_indices.sort_by(|left, right| {
            MonsGameModel::compare_ranked_scored_root_indices(
                scored_roots.as_slice(),
                *left,
                *right,
            )
        });

        let mut target_margins = Vec::new();
        for index in detail_indices.iter().copied() {
            target_margins.push(best_score.saturating_sub(scored_roots[index].score));
        }

        format!(
            "label={} context={} frontier_selected={} shipping_selected={} margin={} shortlist_max={} candidate_count={} shortlist={:?} best_score={} best_rank={} frontier_root={} shipping_root={} root_details={:?} margin_sweep={:?} advisor={:?}",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selected_input_fen,
            shipping_probe.selected_input_fen,
            config.root_reply_risk_score_margin,
            config.root_reply_risk_shortlist_max,
            candidate_indices.len(),
            root_fens(scored_roots.as_slice(), shortlist.as_slice()),
            best_score,
            best_rank,
            frontier_index
                .map(|index| root_economics(
                    &game,
                    scored_roots.as_slice(),
                    index,
                    candidate_indices.as_slice(),
                    shortlist.as_slice(),
                    perspective,
                    config,
                    best_score,
                    best_rank,
                ))
                .unwrap_or_else(|| "missing".to_string()),
            shipping_index
                .map(|index| root_economics(
                    &game,
                    scored_roots.as_slice(),
                    index,
                    candidate_indices.as_slice(),
                    shortlist.as_slice(),
                    perspective,
                    config,
                    best_score,
                    best_rank,
                ))
                .unwrap_or_else(|| "missing".to_string()),
            detail_indices
                .iter()
                .map(|index| root_economics(
                    &game,
                    scored_roots.as_slice(),
                    *index,
                    candidate_indices.as_slice(),
                    shortlist.as_slice(),
                    perspective,
                    config,
                    best_score,
                    best_rank,
                ))
                .collect::<Vec<_>>(),
            margin_sweep(
                &game,
                scored_roots.as_slice(),
                perspective,
                config,
                target_margins.as_slice(),
            ),
            frontier_advisor,
        )
    }

    let cases = [
        EconomicsCase {
            label: "black_progress_vs_setup_residue",
            board_fen: "1 0 b 0 0 0 0 0 6 n05d0xn05/n05s0xa0xe0xn03/n02xxmn04xxmn03/n07xxmn03/n03xxmn01xxmn05/n05xxUn04xxQ/n05xxMn05/n01y0xn01S0xE0xn01xxMxxMn03/n01xxMn09/n03A0xn03Y0xn03/n05D1xn05",
            frontier_root: "l7,1;l9,3",
            shipping_root: "l1,5;l2,7;l1,8",
            reference_setup_roots: &["l1,5;l3,7;l2,8", "l1,5;l2,7;l2,8"],
        },
        EconomicsCase {
            label: "black_late_setup_reply_risk",
            board_fen: "1 1 b 0 0 0 0 0 8 d0xn10/n05s0xa0xe0xxxmn02/n11/n07xxmn03/n03xxmn02xxmn04/n10xxQ/n02y0xn01D0UxxMn01xxMn03/n02xxMS0xn01A0xxxMn04/n06Y0xn04/n03E0xn07/n11",
            frontier_root: "l1,5;l3,7;l2,8",
            shipping_root: "l1,5;l3,7;l2,8",
            reference_setup_roots: &[],
        },
        EconomicsCase {
            label: "black_confirm_fast_setup",
            board_fen: "2 1 b 0 0 0 0 0 10 n05d0xn03xxmn01/n04a0xn02e0xn03/n05s0xn05/E0xn02xxmn03xxmn03/n05xxmn05/n05xxUn04xxQ/n05xxMn05/n03S0xn01Y0xxxMn04/n03y0xn04xxMn02/n03A0xn07/n05D1xn05",
            frontier_root: "l2,5;l3,7;l2,8",
            shipping_root: "l2,5;l3,7;l2,8",
            reference_setup_roots: &["l0,5;l1,5", "l2,5;l3,3;l2,2"],
        },
    ];

    for case in cases {
        println!(
            "BLACK_PROGRESS_SETUP_SHORTLIST_ECONOMICS {}",
            surface_for_case(case)
        );
    }
}

#[test]
#[ignore = "diagnostic: break down black progress-vs-setup reply floor scoring"]
fn black_progress_reply_floor_breakdown_probe() {
    use crate::models::scoring::evaluate_preferability_breakdown_with_weights;

    #[derive(Clone, Copy)]
    struct BreakdownCase {
        label: &'static str,
        board_fen: &'static str,
        roots: &'static [&'static str],
    }

    fn root_index(scored_roots: &[RootEvaluation], root_fen: &str) -> Option<usize> {
        scored_roots
            .iter()
            .position(|root| Input::fen_from_array(&root.inputs) == root_fen)
    }

    fn eval_breakdown_summary(
        game: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> String {
        let search_eval = MonsGameModel::evaluate_search_preferability(game, perspective, config);
        let breakdown = evaluate_preferability_breakdown_with_weights(
            game,
            perspective,
            config.scoring_weights,
        );
        format!(
            "search_eval={} breakdown_total={} terms={:?} features={:?}",
            search_eval, breakdown.total, breakdown.terms, breakdown.features,
        )
    }

    fn worst_reply_details(
        state_after_move: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
        reply_limit: usize,
    ) -> String {
        let replies = MonsGameModel::enumerate_legal_transitions(
            state_after_move,
            reply_limit.max(1),
            MonsGameModel::automove_start_input_options(config),
        );
        if replies.is_empty() {
            return "no_replies".to_string();
        }

        let mut scored_replies = replies
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
                (score, reply)
            })
            .collect::<Vec<_>>();
        scored_replies.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.inputs.cmp(&right.1.inputs))
        });

        scored_replies
            .iter()
            .take(4)
            .map(|(score, reply)| {
                format!(
                    "{} => score={} events={:?} {}",
                    Input::fen_from_array(&reply.inputs),
                    score,
                    reply.events,
                    eval_breakdown_summary(&reply.game, perspective, config),
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn root_breakdown(
        game: &MonsGame,
        scored_roots: &[RootEvaluation],
        root_fen: &str,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> String {
        let Some(index) = root_index(scored_roots, root_fen) else {
            return format!("{root_fen} => missing");
        };
        let root = &scored_roots[index];
        let family = MonsGameModel::turn_engine_root_evaluation_family(root);
        let utility = MonsGameModel::turn_engine_selected_override_utility(
            game,
            root,
            perspective,
            config,
            family,
        );
        let reply_limit = config.root_reply_risk_reply_limit.max(1).min(24);
        let snapshot =
            MonsGameModel::root_reply_risk_snapshot(&root.game, perspective, config, reply_limit);
        let followup =
            MonsGameModel::pro_v2_spirit_followup_floor_score(&root.game, perspective, config);

        format!(
            "{} => root={} family={:?} utility={:?} snapshot=[worst_reply={} match_point={} immediate_win={}] followup={} after_root_eval={{ {} }} worst_replies=[{}]",
            root_fen,
            format_root_probe(Some(root)),
            family,
            utility,
            snapshot.worst_reply_score,
            snapshot.opponent_reaches_match_point,
            snapshot.allows_immediate_opponent_win,
            followup,
            eval_breakdown_summary(&root.game, perspective, config),
            worst_reply_details(&root.game, perspective, config, reply_limit),
        )
    }

    fn surface_for_case(case: BreakdownCase) -> String {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{} should have a valid fen", case.label));
        let perspective = game.active_color;
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
        let (config, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );

        format!(
            "label={} context={} frontier_selected={} shipping_selected={} roots={:?} advisor={:?}",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selected_input_fen,
            shipping_probe.selected_input_fen,
            case.roots
                .iter()
                .map(|root_fen| root_breakdown(
                    &game,
                    scored_roots.as_slice(),
                    root_fen,
                    perspective,
                    config,
                ))
                .collect::<Vec<_>>(),
            frontier_advisor,
        )
    }

    let cases = [
        BreakdownCase {
            label: "black_progress_vs_setup_residue",
            board_fen: "1 0 b 0 0 0 0 0 6 n05d0xn05/n05s0xa0xe0xn03/n02xxmn04xxmn03/n07xxmn03/n03xxmn01xxmn05/n05xxUn04xxQ/n05xxMn05/n01y0xn01S0xE0xn01xxMxxMn03/n01xxMn09/n03A0xn03Y0xn03/n05D1xn05",
            roots: &["l7,1;l9,3", "l1,5;l2,7;l1,8", "l1,5;l3,7;l2,8"],
        },
        BreakdownCase {
            label: "black_confirm_fast_setup",
            board_fen: "2 1 b 0 0 0 0 0 10 n05d0xn03xxmn01/n04a0xn02e0xn03/n05s0xn05/E0xn02xxmn03xxmn03/n05xxmn05/n05xxUn04xxQ/n05xxMn05/n03S0xn01Y0xxxMn04/n03y0xn04xxMn02/n03A0xn07/n05D1xn05",
            roots: &["l2,5;l3,7;l2,8", "l0,5;l1,5", "l2,5;l3,3;l2,2"],
        },
    ];

    for case in cases {
        println!(
            "BLACK_PROGRESS_REPLY_FLOOR_BREAKDOWN {}",
            surface_for_case(case)
        );
    }
}

#[test]
#[ignore = "diagnostic: attribute black progress-vs-setup residual board-state scoring"]
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
        let reply_limit = config.root_reply_risk_reply_limit.max(1).min(24);
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

        format!(
            "label={} context={} frontier_selected={} shipping_selected={} comparisons={:?}",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selected_input_fen,
            shipping_probe.selected_input_fen,
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
#[ignore = "diagnostic: inspect remaining late black confirm fast setup seam"]
fn black_confirm_fast_setup_split_probe() {
    log_black_confirm_fast_runtime_seam_probe(
        "BLACK_CONFIRM_FAST_SETUP_SPLIT",
        "2 1 b 0 0 0 0 0 10 n05d0xn03xxmn01/n04a0xn02e0xn03/n05s0xn05/E0xn02xxmn03xxmn03/n05xxmn05/n05xxUn04xxQ/n05xxMn05/n03S0xn01Y0xxxMn04/n03y0xn04xxMn02/n03A0xn07/n05D1xn05",
        "l0,5;l1,5",
        "l2,5;l3,7;l2,8",
    );
}

#[test]
#[ignore = "diagnostic: inspect black normal confirm turn-six head split seam"]
fn black_confirm_normal_turn_six_head_split_probe() {
    log_black_confirm_fast_runtime_seam_probe(
        "BLACK_CONFIRM_NORMAL_TURN_SIX_HEAD_SPLIT",
        "0 1 b 0 0 0 0 0 6 n06a0xn03d0x/n06e0xn04/n01y0xn02s0xn06/n03xxmn07/n02xxmn02xxmn01xxmn03/E0xn09xxQ/n03xxMn02xxUn04/n03xxMxxMS0xY0xxxMn03/n05D0xn01xxMn03/n04A0xn06/n11",
        "l2,4;l0,6;l1,7",
        "l1,6;l2,6",
    );
}

#[test]
#[ignore = "diagnostic: inspect white fast ply10 reply-order utility and floor"]
fn white_fast_ply10_reply_order_probe() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05D0xn05/n03E0xn01A0xn01S0xY0xn02/n11",
        false,
    )
    .expect("valid white fast ply10 fen");
    let perspective = game.active_color;
    let frontier_probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
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
    let projections = MonsGameModel::turn_engine_reply_risk_projections(
        scored_roots.as_slice(),
        shortlist.as_slice(),
        perspective,
        config,
    );
    let frontier_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == frontier_probe.selected_input_fen)
        .expect("frontier root should exist");
    let shipping_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == shipping_probe.selected_input_fen)
        .expect("shipping root should exist");
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(shortlist.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let frontier_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[frontier_index],
        projections.get(&frontier_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let shipping_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[shipping_index],
        projections.get(&shipping_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let frontier_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[frontier_index]);
    let shipping_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[shipping_index]);
    let frontier_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[frontier_index],
        perspective,
        config,
        frontier_family,
    );
    let shipping_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[shipping_index],
        perspective,
        config,
        shipping_family,
    );
    let shipping_beats_frontier = MonsGameModel::is_better_reply_risk_candidate(
        &game,
        shipping_index,
        shipping_snapshot,
        frontier_index,
        frontier_snapshot,
        projections.get(&shipping_index),
        projections.get(&frontier_index),
        scored_roots.as_slice(),
        perspective,
        config,
        &mut std::collections::HashMap::new(),
    );

    println!(
        "WHITE_FAST_PLY10_REPLY_ORDER context={} shortlist={:?} frontier_probe={:?} shipping_probe={:?} advisor={:?} frontier={} shipping={} frontier_snapshot={} shipping_snapshot={} frontier_utility={} shipping_utility={} shipping_vs_frontier={} frontier_projection={:?} shipping_projection={:?}",
        exact_opportunity_context_probe(&game),
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        frontier_probe,
        shipping_probe,
        frontier_advisor,
        format_root_probe(scored_roots.get(frontier_index)),
        format_root_probe(scored_roots.get(shipping_index)),
        format!(
            "win={} match_point={} floor={}",
            frontier_snapshot.allows_immediate_opponent_win,
            frontier_snapshot.opponent_reaches_match_point,
            frontier_snapshot.worst_reply_score,
        ),
        format!(
            "win={} match_point={} floor={}",
            shipping_snapshot.allows_immediate_opponent_win,
            shipping_snapshot.opponent_reaches_match_point,
            shipping_snapshot.worst_reply_score,
        ),
        format_turn_engine_utility_probe(frontier_utility),
        format_turn_engine_utility_probe(shipping_utility),
        shipping_beats_frontier,
        projections.get(&frontier_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        projections.get(&shipping_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
    );
}

#[test]
#[ignore = "diagnostic: inspect late white fast hotspot utility and shortlist ordering"]
fn white_late_fast_hotspot_probe() {
    let game = MonsGame::from_fen(
        "1 1 w 0 0 1 0 0 9 n04s1xn06/n06a0xn04/n05e0xd0xn04/n03xxmxxmn02xxmn03/n05xxmn03Y0xn01/n05xxUn05/E0xn04xxMn01xxMn03/n01y0xn01xxMn03xxMn03/n05S0xn05/n05D0xn05/n04A1xn06",
        false,
    )
    .expect("valid late white fast hotspot fen");
    let perspective = game.active_color;
    let frontier_probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
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
    let projections = MonsGameModel::turn_engine_reply_risk_projections(
        scored_roots.as_slice(),
        shortlist.as_slice(),
        perspective,
        config,
    );
    let frontier_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == frontier_probe.selected_input_fen)
        .expect("frontier root should exist");
    let shipping_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == shipping_probe.selected_input_fen)
        .expect("shipping root should exist");
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(shortlist.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let frontier_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[frontier_index],
        projections.get(&frontier_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let shipping_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[shipping_index],
        projections.get(&shipping_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let frontier_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[frontier_index]);
    let shipping_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[shipping_index]);
    let frontier_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[frontier_index],
        perspective,
        config,
        frontier_family,
    );
    let shipping_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[shipping_index],
        perspective,
        config,
        shipping_family,
    );
    let shipping_beats_frontier = MonsGameModel::is_better_reply_risk_candidate(
        &game,
        shipping_index,
        shipping_snapshot,
        frontier_index,
        frontier_snapshot,
        projections.get(&shipping_index),
        projections.get(&frontier_index),
        scored_roots.as_slice(),
        perspective,
        config,
        &mut std::collections::HashMap::new(),
    );
    let top_root_details = scored_roots
        .iter()
        .take(8)
        .map(|root| {
            format!(
                "{}:{}",
                Input::fen_from_array(&root.inputs),
                format_root_probe(Some(root))
            )
        })
        .collect::<Vec<_>>();

    println!(
        "WHITE_LATE_FAST_HOTSPOT context={} shortlist={:?} frontier_probe={:?} shipping_probe={:?} advisor={:?} frontier={} shipping={} frontier_snapshot={} shipping_snapshot={} frontier_utility={} shipping_utility={} shipping_vs_frontier={} frontier_projection={:?} shipping_projection={:?} top_root_details={:?}",
        exact_opportunity_context_probe(&game),
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        frontier_probe,
        shipping_probe,
        frontier_advisor,
        format_root_probe(scored_roots.get(frontier_index)),
        format_root_probe(scored_roots.get(shipping_index)),
        format!(
            "win={} match_point={} floor={}",
            frontier_snapshot.allows_immediate_opponent_win,
            frontier_snapshot.opponent_reaches_match_point,
            frontier_snapshot.worst_reply_score,
        ),
        format!(
            "win={} match_point={} floor={}",
            shipping_snapshot.allows_immediate_opponent_win,
            shipping_snapshot.opponent_reaches_match_point,
            shipping_snapshot.worst_reply_score,
        ),
        format_turn_engine_utility_probe(frontier_utility),
        format_turn_engine_utility_probe(shipping_utility),
        shipping_beats_frontier,
        projections.get(&frontier_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        projections.get(&shipping_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        top_root_details,
    );
}

#[test]
#[ignore = "diagnostic: inspect rotated white normal head acceptance"]
fn white_normal_turn_five_head_acceptance_probe() {
    struct ProbeCase {
        label: &'static str,
        fen: &'static str,
    }

    let cases = [
        ProbeCase {
            label: "repeat1_opening1_ply24",
            fen: "0 0 w 1 0 0 0 0 5 n06a0xn04/n07d0me0xn02/n02y0xn01s0xn06/n04xxmn01xxmxxmn03/n03xxmn07/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn01xxMY0xxxMn04/n05D0xn05/n03E0xA0xS0xn05/n11",
        },
        ProbeCase {
            label: "repeat1_opening2_ply22",
            fen: "0 1 w 0 0 0 0 0 5 n06a0xn03d0x/n08e0xn02/n02y0xn01s0xn06/n04xxmn03xxmn02/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n03E0xA0xD0xn05/n05S0xn01Y0xn03/n11",
        },
        ProbeCase {
            label: "repeat1_opening2_ply24_head_over_pre_accept",
            fen: "0 1 w 1 0 1 0 0 5 n06a0xn03d0x/n08e0xn02/n02y0xn01s0xn06/n04xxmn03xxmn02/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03D0Mn02xxMn04/n03E0xA0xn06/n05S0xn01Y0xn03/n11",
        },
        ProbeCase {
            label: "fast_repeat1_opening1_ply24_head_over_pre_accept",
            fen: "0 0 w 0 0 2 0 0 5 n05d0xn05/n06a0xn04/n03xxmn01s0xn05/n02xxmn03e0xn01xxmn02/n03y0xn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n11/n02xxMn03S0xD0MY0xn02/n04A0xn06/n02E0xn08",
        },
        ProbeCase {
            label: "confirm_pro_repeat3_opening2_ply26_head_over_pre_accept",
            fen: "1 1 w 1 0 2 0 0 5 d0xn10/n05s0xn01e0xn03/n03y0xn01a0xn05/n03xxmn02xxmn04/n05xxmn01xxmn03/xxQn04xxUn04Y0x/n05xxMn01xxMn03/n03xxMn07/n03xxMn07/n05S0xn05/n03E0xA0xn05D0x",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.fen, false)
            .unwrap_or_else(|| panic!("{} fen should be valid", case.label));
        let perspective = game.active_color;
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
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
        let projections = MonsGameModel::turn_engine_reply_risk_projections(
            scored_roots.as_slice(),
            shortlist.as_slice(),
            perspective,
            config,
        );
        let frontier_index = scored_roots
            .iter()
            .position(|root| {
                Input::fen_from_array(&root.inputs) == frontier_probe.selected_input_fen
            })
            .expect("frontier root should exist");
        let pre_accept_index = scored_roots
            .iter()
            .position(|root| {
                Input::fen_from_array(&root.inputs) == frontier_probe.pre_accept_input_fen
            })
            .expect("pre-accept root should exist");
        let shipping_index = scored_roots
            .iter()
            .position(|root| {
                Input::fen_from_array(&root.inputs) == shipping_probe.selected_input_fen
            })
            .expect("shipping root should exist");
        let root_node_budget = ((config.max_visited_nodes
            * config.root_reply_risk_node_share_bp.max(0) as usize)
            / 10_000)
            .max(shortlist.len())
            .max(1);
        let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
            .max(1)
            .min(config.root_reply_risk_reply_limit.max(1));
        let frontier_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
            &scored_roots[frontier_index],
            projections.get(&frontier_index),
            perspective,
            config,
            per_root_reply_limit,
        );
        let pre_accept_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
            &scored_roots[pre_accept_index],
            projections.get(&pre_accept_index),
            perspective,
            config,
            per_root_reply_limit,
        );
        let shipping_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
            &scored_roots[shipping_index],
            projections.get(&shipping_index),
            perspective,
            config,
            per_root_reply_limit,
        );
        let frontier_family =
            MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[frontier_index]);
        let pre_accept_family =
            MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[pre_accept_index]);
        let shipping_family =
            MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[shipping_index]);
        let frontier_utility = MonsGameModel::turn_engine_selected_override_utility(
            &game,
            &scored_roots[frontier_index],
            perspective,
            config,
            frontier_family,
        );
        let pre_accept_utility = MonsGameModel::turn_engine_selected_override_utility(
            &game,
            &scored_roots[pre_accept_index],
            perspective,
            config,
            pre_accept_family,
        );
        let shipping_utility = MonsGameModel::turn_engine_selected_override_utility(
            &game,
            &scored_roots[shipping_index],
            perspective,
            config,
            shipping_family,
        );
        let shipping_beats_frontier = MonsGameModel::is_better_reply_risk_candidate(
            &game,
            shipping_index,
            shipping_snapshot,
            frontier_index,
            frontier_snapshot,
            projections.get(&shipping_index),
            projections.get(&frontier_index),
            scored_roots.as_slice(),
            perspective,
            config,
            &mut std::collections::HashMap::new(),
        );
        let pre_accept_beats_frontier = MonsGameModel::is_better_reply_risk_candidate(
            &game,
            pre_accept_index,
            pre_accept_snapshot,
            frontier_index,
            frontier_snapshot,
            projections.get(&pre_accept_index),
            projections.get(&frontier_index),
            scored_roots.as_slice(),
            perspective,
            config,
            &mut std::collections::HashMap::new(),
        );
        let top_root_details = scored_roots
            .iter()
            .take(8)
            .map(|root| {
                format!(
                    "{}:{}",
                    Input::fen_from_array(&root.inputs),
                    format_root_probe(Some(root))
                )
            })
            .collect::<Vec<_>>();

        println!(
            "WHITE_NORMAL_TURN_FIVE_HEAD label={} context={} shortlist={:?} frontier_probe={:?} shipping_probe={:?} advisor={:?} frontier={} pre_accept={} shipping={} frontier_snapshot={} pre_accept_snapshot={} shipping_snapshot={} frontier_utility={} pre_accept_utility={} shipping_utility={} pre_accept_vs_frontier={} shipping_vs_frontier={} frontier_projection={:?} pre_accept_projection={:?} shipping_projection={:?} top_root_details={:?}",
            case.label,
            exact_opportunity_context_probe(&game),
            shortlist
                .iter()
                .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
                .collect::<Vec<_>>(),
            frontier_probe,
            shipping_probe,
            frontier_advisor,
            format_root_probe(scored_roots.get(frontier_index)),
            format_root_probe(scored_roots.get(pre_accept_index)),
            format_root_probe(scored_roots.get(shipping_index)),
            format!(
                "win={} match_point={} floor={}",
                frontier_snapshot.allows_immediate_opponent_win,
                frontier_snapshot.opponent_reaches_match_point,
                frontier_snapshot.worst_reply_score,
            ),
            format!(
                "win={} match_point={} floor={}",
                pre_accept_snapshot.allows_immediate_opponent_win,
                pre_accept_snapshot.opponent_reaches_match_point,
                pre_accept_snapshot.worst_reply_score,
            ),
            format!(
                "win={} match_point={} floor={}",
                shipping_snapshot.allows_immediate_opponent_win,
                shipping_snapshot.opponent_reaches_match_point,
                shipping_snapshot.worst_reply_score,
            ),
            format_turn_engine_utility_probe(frontier_utility),
            format_turn_engine_utility_probe(pre_accept_utility),
            format_turn_engine_utility_probe(shipping_utility),
            pre_accept_beats_frontier,
            shipping_beats_frontier,
            projections.get(&frontier_index).map(|projection| {
                format!(
                    "{:?}/{:?}/{}",
                    projection.plan.head_family,
                    projection.plan.goal_family,
                    format_turn_engine_utility_probe(projection.plan.utility),
                )
            }),
            projections.get(&pre_accept_index).map(|projection| {
                format!(
                    "{:?}/{:?}/{}",
                    projection.plan.head_family,
                    projection.plan.goal_family,
                    format_turn_engine_utility_probe(projection.plan.utility),
                )
            }),
            projections.get(&shipping_index).map(|projection| {
                format!(
                    "{:?}/{:?}/{}",
                    projection.plan.head_family,
                    projection.plan.goal_family,
                    format_turn_engine_utility_probe(projection.plan.utility),
                )
            }),
            top_root_details,
        );
    }
}

#[test]
#[ignore = "diagnostic: compare static frontier vs shipping config on remaining white ordering boards"]
fn white_profile_config_ordering_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    let cases = [
        ProbeCase {
            label: "white_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "white_late_fast_hotspot",
            board_fen:
                "1 1 w 0 0 1 0 0 9 n04s1xn06/n06a0xn04/n05e0xd0xn04/n03xxmxxmn02xxmn03/n05xxmn03Y0xn01/n05xxUn05/E0xn04xxMn01xxMn03/n01y0xn01xxMn03xxMn03/n05S0xn05/n05D0xn05/n04A1xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let shipping_config =
            calibration_runtime_config("shipping_pro_search", &game, SmartAutomovePreference::Pro);
        let frontier_config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            &game,
            SmartAutomovePreference::Pro,
        );
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);

        println!(
            "WHITE_PROFILE_CONFIG_ORDERING label={} context={} same_scoring_weights={} shipping(depth={} nodes={} selector={} head_rerank={} mode={:?} reply_guard={} shortlist_max={} reply_limit={} node_share_bp={} low_budget={} mid_progress={} mid_tactical={} late_safe_mana={}) frontier(depth={} nodes={} selector={} head_rerank={} mode={:?} reply_guard={} shortlist_max={} reply_limit={} node_share_bp={} low_budget={} mid_progress={} mid_tactical={} late_safe_mana={}) shipping_stage={} frontier_stage={}",
            case.label,
            exact_opportunity_context_probe(&game),
            std::ptr::eq(
                shipping_config.scoring_weights,
                frontier_config.scoring_weights,
            ),
            shipping_config.depth,
            shipping_config.max_visited_nodes,
            shipping_config.enable_turn_engine_selector,
            shipping_config.enable_turn_head_rerank,
            shipping_config.turn_engine_mode,
            shipping_config.enable_root_reply_risk_guard,
            shipping_config.root_reply_risk_shortlist_max,
            shipping_config.root_reply_risk_reply_limit,
            shipping_config.root_reply_risk_node_share_bp,
            shipping_config.enable_turn_engine_low_budget_guard,
            shipping_config.enable_turn_engine_mid_turn_progress_guard,
            shipping_config.enable_turn_engine_mid_turn_tactical_guard,
            shipping_config.enable_turn_engine_late_safe_mana_root_preference,
            frontier_config.depth,
            frontier_config.max_visited_nodes,
            frontier_config.enable_turn_engine_selector,
            frontier_config.enable_turn_head_rerank,
            frontier_config.turn_engine_mode,
            frontier_config.enable_root_reply_risk_guard,
            frontier_config.root_reply_risk_shortlist_max,
            frontier_config.root_reply_risk_reply_limit,
            frontier_config.root_reply_risk_node_share_bp,
            frontier_config.enable_turn_engine_low_budget_guard,
            frontier_config.enable_turn_engine_mid_turn_progress_guard,
            frontier_config.enable_turn_engine_mid_turn_tactical_guard,
            frontier_config.enable_turn_engine_late_safe_mana_root_preference,
            shipping_probe.selector_last_stage,
            frontier_probe.selector_last_stage,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect rerank admissibility of remaining white shipping-order roots"]
fn white_ordering_rerank_semantics_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn root_moves_for_config(
        game: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> Vec<ScoredRootMove> {
        let mut root_moves = MonsGameModel::ranked_root_moves(game, perspective, config);
        if config.enable_turn_engine_root_injection {
            MonsGameModel::inject_turn_engine_root_candidates(
                game,
                perspective,
                config,
                &mut root_moves,
            );
        }
        root_moves
    }

    let cases = [
        ProbeCase {
            label: "white_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "white_late_fast_hotspot",
            board_fen:
                "1 1 w 0 0 1 0 0 9 n04s1xn06/n06a0xn04/n05e0xd0xn04/n03xxmxxmn02xxmn03/n05xxmn03Y0xn01/n05xxUn05/E0xn04xxMn01xxMn03/n01y0xn01xxMn03xxMn03/n05S0xn05/n05D0xn05/n04A1xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let perspective = game.active_color;
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
        let frontier_inputs = Input::array_from_fen(frontier_probe.selected_input_fen.as_str());
        let shipping_inputs = Input::array_from_fen(shipping_probe.selected_input_fen.as_str());
        let frontier_config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            &game,
            SmartAutomovePreference::Pro,
        );
        let shipping_config =
            calibration_runtime_config("shipping_pro_search", &game, SmartAutomovePreference::Pro);
        let frontier_root_moves = root_moves_for_config(&game, perspective, frontier_config);
        let shipping_root_moves = root_moves_for_config(&game, perspective, shipping_config);

        println!(
            "WHITE_ORDERING_RERANK label={} context={} shipping_stage={} frontier_stage={} shipping_head_rerank={} frontier_head_rerank={} shipping_should_rerank={} frontier_should_rerank={} shipping_root_rank_on_shipping={:?} shipping_root_rank_on_frontier={:?} frontier_root_rank_on_frontier={:?} shipping_accept_on_shipping={:?} shipping_accept_on_frontier={:?} shipping_allowed_on_shipping={} shipping_allowed_on_frontier={} shipping_conflict_on_shipping={} shipping_conflict_on_frontier={} frontier_accept_on_frontier={:?} frontier_allowed_on_frontier={} frontier_conflict_on_frontier={}",
            case.label,
            exact_opportunity_context_probe(&game),
            shipping_probe.selector_last_stage,
            frontier_probe.selector_last_stage,
            shipping_config.enable_turn_head_rerank,
            frontier_config.enable_turn_head_rerank,
            MonsGameModel::should_invoke_turn_head_rerank(shipping_root_moves.as_slice()),
            MonsGameModel::should_invoke_turn_head_rerank(frontier_root_moves.as_slice()),
            shipping_root_moves
                .iter()
                .position(|candidate| candidate.inputs.as_slice() == shipping_inputs.as_slice()),
            frontier_root_moves
                .iter()
                .position(|candidate| candidate.inputs.as_slice() == shipping_inputs.as_slice()),
            frontier_root_moves
                .iter()
                .position(|candidate| candidate.inputs.as_slice() == frontier_inputs.as_slice()),
            MonsGameModel::classify_turn_engine_rerank_override(
                shipping_root_moves.as_slice(),
                shipping_inputs.as_slice(),
            ),
            MonsGameModel::classify_turn_engine_rerank_override(
                frontier_root_moves.as_slice(),
                shipping_inputs.as_slice(),
            ),
            MonsGameModel::turn_engine_allowed_rerank_override_candidate(
                shipping_root_moves.as_slice(),
                shipping_inputs.as_slice(),
            ),
            MonsGameModel::turn_engine_allowed_rerank_override_candidate(
                frontier_root_moves.as_slice(),
                shipping_inputs.as_slice(),
            ),
            MonsGameModel::pro_v2_root_advisor_conflicts_with_choice(
                &game,
                perspective,
                shipping_config,
                shipping_root_moves.as_slice(),
                None,
                shipping_inputs.as_slice(),
            ),
            MonsGameModel::pro_v2_root_advisor_conflicts_with_choice(
                &game,
                perspective,
                frontier_config,
                frontier_root_moves.as_slice(),
                None,
                shipping_inputs.as_slice(),
            ),
            MonsGameModel::classify_turn_engine_rerank_override(
                frontier_root_moves.as_slice(),
                frontier_inputs.as_slice(),
            ),
            MonsGameModel::turn_engine_allowed_rerank_override_candidate(
                frontier_root_moves.as_slice(),
                frontier_inputs.as_slice(),
            ),
            MonsGameModel::pro_v2_root_advisor_conflicts_with_choice(
                &game,
                perspective,
                frontier_config,
                frontier_root_moves.as_slice(),
                None,
                frontier_inputs.as_slice(),
            ),
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect allowed-head rerank plans on white search-order family"]
fn white_search_order_allowed_head_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn root_moves_for_config(
        game: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> Vec<ScoredRootMove> {
        let mut root_moves = MonsGameModel::ranked_root_moves(game, perspective, config);
        if config.enable_turn_engine_root_injection {
            MonsGameModel::inject_turn_engine_root_candidates(
                game,
                perspective,
                config,
                &mut root_moves,
            );
        }
        root_moves
    }

    let cases = [
        ProbeCase {
            label: "white_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let perspective = game.active_color;
        let shipping_config =
            calibration_runtime_config("shipping_pro_search", &game, SmartAutomovePreference::Pro);
        let frontier_config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            &game,
            SmartAutomovePreference::Pro,
        );
        let shipping_rerank_config = calibration_turn_engine_rerank_config(shipping_config);
        let frontier_rerank_config = calibration_turn_engine_rerank_config(frontier_config);
        let shipping_root_moves = root_moves_for_config(&game, perspective, shipping_config);
        let frontier_root_moves = root_moves_for_config(&game, perspective, frontier_config);
        let shipping_allowed_heads = shipping_root_moves
            .iter()
            .map(|candidate| candidate.inputs.clone())
            .collect::<Vec<_>>();
        let frontier_allowed_heads = frontier_root_moves
            .iter()
            .map(|candidate| candidate.inputs.clone())
            .collect::<Vec<_>>();
        let shipping_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            shipping_rerank_config,
            shipping_allowed_heads.as_slice(),
        );
        let shipping_on_frontier_heads = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            shipping_rerank_config,
            frontier_allowed_heads.as_slice(),
        );
        let frontier_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            frontier_rerank_config,
            frontier_allowed_heads.as_slice(),
        );
        let shipping_ranked_plans =
            turn_engine_ranked_plan_digests_for_test(&game, perspective, shipping_rerank_config, 5);
        let frontier_ranked_plans =
            turn_engine_ranked_plan_digests_for_test(&game, perspective, frontier_rerank_config, 5);

        println!(
            "WHITE_ALLOWED_HEAD_RERANK label={} context={} shipping_allowed_best={:?} shipping_on_frontier_heads={:?} frontier_allowed_best={:?} shipping_ranked_plans={:?} frontier_ranked_plans={:?}",
            case.label,
            exact_opportunity_context_probe(&game),
            shipping_plan.as_ref().map(|plan| {
                format!(
                    "{}/{:?}/{:?}/{}",
                    Input::fen_from_array(plan.compiled_chunks.first().unwrap_or(&Vec::new())),
                    plan.head_family,
                    plan.goal_family,
                    format_turn_engine_utility_probe(plan.utility),
                )
            }),
            shipping_on_frontier_heads.as_ref().map(|plan| {
                format!(
                    "{}/{:?}/{:?}/{}",
                    Input::fen_from_array(plan.compiled_chunks.first().unwrap_or(&Vec::new())),
                    plan.head_family,
                    plan.goal_family,
                    format_turn_engine_utility_probe(plan.utility),
                )
            }),
            frontier_plan.as_ref().map(|plan| {
                format!(
                    "{}/{:?}/{:?}/{}",
                    Input::fen_from_array(plan.compiled_chunks.first().unwrap_or(&Vec::new())),
                    plan.head_family,
                    plan.goal_family,
                    format_turn_engine_utility_probe(plan.utility),
                )
            }),
            shipping_ranked_plans
                .iter()
                .map(|digest| {
                    format!(
                        "{}/{:?}/{:?}/{}",
                        digest.head_inputs_fen,
                        digest.head_family,
                        digest.goal_family,
                        format_turn_engine_utility_probe(digest.utility),
                    )
                })
                .collect::<Vec<_>>(),
            frontier_ranked_plans
                .iter()
                .map(|digest| {
                    format!(
                        "{}/{:?}/{:?}/{}",
                        digest.head_inputs_fen,
                        digest.head_family,
                        digest.goal_family,
                        format_turn_engine_utility_probe(digest.utility),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }
}

#[test]
#[ignore = "diagnostic: isolate rerank mode on white search-order family"]
fn white_search_order_rerank_mode_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn root_moves_for_config(
        game: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> Vec<ScoredRootMove> {
        let mut root_moves = MonsGameModel::ranked_root_moves(game, perspective, config);
        if config.enable_turn_engine_root_injection {
            MonsGameModel::inject_turn_engine_root_candidates(
                game,
                perspective,
                config,
                &mut root_moves,
            );
        }
        root_moves
    }

    let cases = [
        ProbeCase {
            label: "white_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let perspective = game.active_color;
        let shipping_config =
            calibration_runtime_config("shipping_pro_search", &game, SmartAutomovePreference::Pro);
        let frontier_config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            &game,
            SmartAutomovePreference::Pro,
        );
        let frontier_heads = root_moves_for_config(&game, perspective, frontier_config)
            .into_iter()
            .map(|candidate| candidate.inputs)
            .collect::<Vec<_>>();
        let frontier_rerank_pro_v2 = calibration_turn_engine_rerank_config(frontier_config);
        let frontier_rerank_pro_v1 =
            calibration_turn_engine_rerank_config_with_mode(frontier_config, TurnEngineMode::ProV1);
        let shipping_rerank_pro_v1 = calibration_turn_engine_rerank_config(shipping_config);
        let shipping_rerank_pro_v2 =
            calibration_turn_engine_rerank_config_with_mode(shipping_config, TurnEngineMode::ProV2);
        let frontier_plan_pro_v2 = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            frontier_rerank_pro_v2,
            frontier_heads.as_slice(),
        );
        let frontier_plan_pro_v1 = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            frontier_rerank_pro_v1,
            frontier_heads.as_slice(),
        );
        let shipping_plan_pro_v1 = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            shipping_rerank_pro_v1,
            frontier_heads.as_slice(),
        );
        let shipping_plan_pro_v2 = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            shipping_rerank_pro_v2,
            frontier_heads.as_slice(),
        );

        println!(
            "WHITE_RERANK_MODE label={} context={} frontier_pro_v2={:?} frontier_pro_v1={:?} shipping_pro_v1={:?} shipping_pro_v2={:?}",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_plan_pro_v2.as_ref().map(|plan| {
                format!(
                    "{}/{:?}/{:?}/{}",
                    Input::fen_from_array(plan.compiled_chunks.first().unwrap_or(&Vec::new())),
                    plan.head_family,
                    plan.goal_family,
                    format_turn_engine_utility_probe(plan.utility),
                )
            }),
            frontier_plan_pro_v1.as_ref().map(|plan| {
                format!(
                    "{}/{:?}/{:?}/{}",
                    Input::fen_from_array(plan.compiled_chunks.first().unwrap_or(&Vec::new())),
                    plan.head_family,
                    plan.goal_family,
                    format_turn_engine_utility_probe(plan.utility),
                )
            }),
            shipping_plan_pro_v1.as_ref().map(|plan| {
                format!(
                    "{}/{:?}/{:?}/{}",
                    Input::fen_from_array(plan.compiled_chunks.first().unwrap_or(&Vec::new())),
                    plan.head_family,
                    plan.goal_family,
                    format_turn_engine_utility_probe(plan.utility),
                )
            }),
            shipping_plan_pro_v2.as_ref().map(|plan| {
                format!(
                    "{}/{:?}/{:?}/{}",
                    Input::fen_from_array(plan.compiled_chunks.first().unwrap_or(&Vec::new())),
                    plan.head_family,
                    plan.goal_family,
                    format_turn_engine_utility_probe(plan.utility),
                )
            }),
        );
    }
}

#[test]
#[ignore = "diagnostic: isolate rerank budget structure on white search-order family"]
fn white_search_order_rerank_budget_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn root_moves_for_config(
        game: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> Vec<ScoredRootMove> {
        let mut root_moves = MonsGameModel::ranked_root_moves(game, perspective, config);
        if config.enable_turn_engine_root_injection {
            MonsGameModel::inject_turn_engine_root_candidates(
                game,
                perspective,
                config,
                &mut root_moves,
            );
        }
        root_moves
    }

    fn describe_plan(plan: Option<&TurnPlan>) -> Option<String> {
        plan.map(|plan| {
            format!(
                "{}/{:?}/{:?}/{}",
                Input::fen_from_array(plan.compiled_chunks.first().unwrap_or(&Vec::new())),
                plan.head_family,
                plan.goal_family,
                format_turn_engine_utility_probe(plan.utility),
            )
        })
    }

    let cases = [
        ProbeCase {
            label: "white_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let perspective = game.active_color;
        let shipping_config =
            calibration_runtime_config("shipping_pro_search", &game, SmartAutomovePreference::Pro);
        let frontier_config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            &game,
            SmartAutomovePreference::Pro,
        );
        let frontier_heads = root_moves_for_config(&game, perspective, frontier_config)
            .into_iter()
            .map(|candidate| candidate.inputs)
            .collect::<Vec<_>>();
        let frontier_base = calibration_turn_engine_rerank_config(frontier_config);
        let shipping_pro_v2 =
            calibration_turn_engine_rerank_config_with_mode(shipping_config, TurnEngineMode::ProV2);
        let mut frontier_own_caps = frontier_base;
        frontier_own_caps.own_seed_cap = shipping_pro_v2.own_seed_cap;
        frontier_own_caps.own_beam = shipping_pro_v2.own_beam;
        frontier_own_caps.per_node_family_cap = shipping_pro_v2.per_node_family_cap;
        frontier_own_caps.step_cap = shipping_pro_v2.step_cap;
        let mut frontier_reply_caps = frontier_base;
        frontier_reply_caps.opponent_seed_cap = shipping_pro_v2.opponent_seed_cap;
        frontier_reply_caps.opponent_beam = shipping_pro_v2.opponent_beam;
        frontier_reply_caps.reply_seed_cap = shipping_pro_v2.reply_seed_cap;
        frontier_reply_caps.reply_beam = shipping_pro_v2.reply_beam;
        let mut frontier_expansion_cap = frontier_base;
        frontier_expansion_cap.expansion_cap = shipping_pro_v2.expansion_cap;
        let mut frontier_all_shipping_caps = frontier_base;
        frontier_all_shipping_caps.own_seed_cap = shipping_pro_v2.own_seed_cap;
        frontier_all_shipping_caps.own_beam = shipping_pro_v2.own_beam;
        frontier_all_shipping_caps.per_node_family_cap = shipping_pro_v2.per_node_family_cap;
        frontier_all_shipping_caps.step_cap = shipping_pro_v2.step_cap;
        frontier_all_shipping_caps.opponent_seed_cap = shipping_pro_v2.opponent_seed_cap;
        frontier_all_shipping_caps.opponent_beam = shipping_pro_v2.opponent_beam;
        frontier_all_shipping_caps.reply_seed_cap = shipping_pro_v2.reply_seed_cap;
        frontier_all_shipping_caps.reply_beam = shipping_pro_v2.reply_beam;
        frontier_all_shipping_caps.expansion_cap = shipping_pro_v2.expansion_cap;

        let frontier_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            frontier_base,
            frontier_heads.as_slice(),
        );
        let own_caps_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            frontier_own_caps,
            frontier_heads.as_slice(),
        );
        let reply_caps_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            frontier_reply_caps,
            frontier_heads.as_slice(),
        );
        let expansion_cap_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            frontier_expansion_cap,
            frontier_heads.as_slice(),
        );
        let all_caps_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            frontier_all_shipping_caps,
            frontier_heads.as_slice(),
        );
        let shipping_pro_v2_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            shipping_pro_v2,
            frontier_heads.as_slice(),
        );

        println!(
            "WHITE_RERANK_BUDGET label={} context={} frontier_base={:?} frontier_own_caps={:?} frontier_reply_caps={:?} frontier_expansion_cap={:?} frontier_all_shipping_caps={:?} shipping_pro_v2={:?} caps(frontier={{own:{}:{}:{}:{}, reply:{}:{}:{}:{}, expansion:{}}}, shipping={{own:{}:{}:{}:{}, reply:{}:{}:{}:{}, expansion:{}}})",
            case.label,
            exact_opportunity_context_probe(&game),
            describe_plan(frontier_plan.as_ref()),
            describe_plan(own_caps_plan.as_ref()),
            describe_plan(reply_caps_plan.as_ref()),
            describe_plan(expansion_cap_plan.as_ref()),
            describe_plan(all_caps_plan.as_ref()),
            describe_plan(shipping_pro_v2_plan.as_ref()),
            frontier_base.own_seed_cap,
            frontier_base.own_beam,
            frontier_base.per_node_family_cap,
            frontier_base.step_cap,
            frontier_base.opponent_seed_cap,
            frontier_base.opponent_beam,
            frontier_base.reply_seed_cap,
            frontier_base.reply_beam,
            frontier_base.expansion_cap,
            shipping_pro_v2.own_seed_cap,
            shipping_pro_v2.own_beam,
            shipping_pro_v2.per_node_family_cap,
            shipping_pro_v2.step_cap,
            shipping_pro_v2.opponent_seed_cap,
            shipping_pro_v2.opponent_beam,
            shipping_pro_v2.reply_seed_cap,
            shipping_pro_v2.reply_beam,
            shipping_pro_v2.expansion_cap,
        );
    }
}

#[test]
#[ignore = "diagnostic: isolate rerank own-cap bundle on white search-order family"]
fn white_search_order_rerank_own_cap_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn root_moves_for_config(
        game: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> Vec<ScoredRootMove> {
        let mut root_moves = MonsGameModel::ranked_root_moves(game, perspective, config);
        if config.enable_turn_engine_root_injection {
            MonsGameModel::inject_turn_engine_root_candidates(
                game,
                perspective,
                config,
                &mut root_moves,
            );
        }
        root_moves
    }

    fn describe_plan(plan: Option<&TurnPlan>) -> Option<String> {
        plan.map(|plan| {
            format!(
                "{}/{:?}/{:?}/{}",
                Input::fen_from_array(plan.compiled_chunks.first().unwrap_or(&Vec::new())),
                plan.head_family,
                plan.goal_family,
                format_turn_engine_utility_probe(plan.utility),
            )
        })
    }

    let cases = [
        ProbeCase {
            label: "white_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let perspective = game.active_color;
        let shipping_config =
            calibration_runtime_config("shipping_pro_search", &game, SmartAutomovePreference::Pro);
        let frontier_config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            &game,
            SmartAutomovePreference::Pro,
        );
        let frontier_heads = root_moves_for_config(&game, perspective, frontier_config)
            .into_iter()
            .map(|candidate| candidate.inputs)
            .collect::<Vec<_>>();
        let frontier_base = calibration_turn_engine_rerank_config(frontier_config);
        let shipping_pro_v2 =
            calibration_turn_engine_rerank_config_with_mode(shipping_config, TurnEngineMode::ProV2);

        let mut seed_only = frontier_base;
        seed_only.own_seed_cap = shipping_pro_v2.own_seed_cap;
        let mut beam_only = frontier_base;
        beam_only.own_beam = shipping_pro_v2.own_beam;
        let mut family_only = frontier_base;
        family_only.per_node_family_cap = shipping_pro_v2.per_node_family_cap;
        let mut step_only = frontier_base;
        step_only.step_cap = shipping_pro_v2.step_cap;
        let mut seed_beam = frontier_base;
        seed_beam.own_seed_cap = shipping_pro_v2.own_seed_cap;
        seed_beam.own_beam = shipping_pro_v2.own_beam;
        let mut seed_beam_family = frontier_base;
        seed_beam_family.own_seed_cap = shipping_pro_v2.own_seed_cap;
        seed_beam_family.own_beam = shipping_pro_v2.own_beam;
        seed_beam_family.per_node_family_cap = shipping_pro_v2.per_node_family_cap;
        let mut full_own_caps = frontier_base;
        full_own_caps.own_seed_cap = shipping_pro_v2.own_seed_cap;
        full_own_caps.own_beam = shipping_pro_v2.own_beam;
        full_own_caps.per_node_family_cap = shipping_pro_v2.per_node_family_cap;
        full_own_caps.step_cap = shipping_pro_v2.step_cap;

        let frontier_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            frontier_base,
            frontier_heads.as_slice(),
        );
        let seed_only_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            seed_only,
            frontier_heads.as_slice(),
        );
        let beam_only_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            beam_only,
            frontier_heads.as_slice(),
        );
        let family_only_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            family_only,
            frontier_heads.as_slice(),
        );
        let step_only_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            step_only,
            frontier_heads.as_slice(),
        );
        let seed_beam_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            seed_beam,
            frontier_heads.as_slice(),
        );
        let seed_beam_family_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            seed_beam_family,
            frontier_heads.as_slice(),
        );
        let full_own_caps_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            full_own_caps,
            frontier_heads.as_slice(),
        );

        println!(
            "WHITE_RERANK_OWN_CAP label={} context={} frontier_base={:?} seed_only={:?} beam_only={:?} family_only={:?} step_only={:?} seed_beam={:?} seed_beam_family={:?} full_own_caps={:?} caps(frontier={}:{}:{}:{}, shipping={}:{}:{}:{})",
            case.label,
            exact_opportunity_context_probe(&game),
            describe_plan(frontier_plan.as_ref()),
            describe_plan(seed_only_plan.as_ref()),
            describe_plan(beam_only_plan.as_ref()),
            describe_plan(family_only_plan.as_ref()),
            describe_plan(step_only_plan.as_ref()),
            describe_plan(seed_beam_plan.as_ref()),
            describe_plan(seed_beam_family_plan.as_ref()),
            describe_plan(full_own_caps_plan.as_ref()),
            frontier_base.own_seed_cap,
            frontier_base.own_beam,
            frontier_base.per_node_family_cap,
            frontier_base.step_cap,
            shipping_pro_v2.own_seed_cap,
            shipping_pro_v2.own_beam,
            shipping_pro_v2.per_node_family_cap,
            shipping_pro_v2.step_cap,
        );
    }
}

#[test]
#[ignore = "diagnostic: measure rerank seed-vs-step scope on white ordering boards"]
fn white_search_order_seed_step_scope_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn root_moves_for_config(
        game: &MonsGame,
        perspective: Color,
        config: AutomoveSearchConfig,
    ) -> Vec<ScoredRootMove> {
        let mut root_moves = MonsGameModel::ranked_root_moves(game, perspective, config);
        if config.enable_turn_engine_root_injection {
            MonsGameModel::inject_turn_engine_root_candidates(
                game,
                perspective,
                config,
                &mut root_moves,
            );
        }
        root_moves
    }

    fn describe_plan(plan: Option<&TurnPlan>) -> Option<String> {
        plan.map(|plan| {
            format!(
                "{}/{:?}/{:?}/{}",
                Input::fen_from_array(plan.compiled_chunks.first().unwrap_or(&Vec::new())),
                plan.head_family,
                plan.goal_family,
                format_turn_engine_utility_probe(plan.utility),
            )
        })
    }

    let cases = [
        ProbeCase {
            label: "white_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
        ProbeCase {
            label: "white_late_fast_hotspot",
            board_fen:
                "1 1 w 0 0 1 0 0 9 n04s1xn06/n06a0xn04/n05e0xd0xn04/n03xxmxxmn02xxmn03/n05xxmn03Y0xn01/n05xxUn05/E0xn04xxMn01xxMn03/n01y0xn01xxMn03xxMn03/n05S0xn05/n05D0xn05/n04A1xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let perspective = game.active_color;
        let shipping_config =
            calibration_runtime_config("shipping_pro_search", &game, SmartAutomovePreference::Pro);
        let frontier_config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            &game,
            SmartAutomovePreference::Pro,
        );
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
        let frontier_heads = root_moves_for_config(&game, perspective, frontier_config)
            .into_iter()
            .map(|candidate| candidate.inputs)
            .collect::<Vec<_>>();
        let frontier_base = calibration_turn_engine_rerank_config(frontier_config);
        let shipping_rerank = calibration_turn_engine_rerank_config(shipping_config);
        let shipping_pro_v2 =
            calibration_turn_engine_rerank_config_with_mode(shipping_config, TurnEngineMode::ProV2);
        let mut seed_only = frontier_base;
        seed_only.own_seed_cap = shipping_pro_v2.own_seed_cap;
        let mut step_only = frontier_base;
        step_only.step_cap = shipping_pro_v2.step_cap;

        let frontier_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            frontier_base,
            frontier_heads.as_slice(),
        );
        let seed_only_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            seed_only,
            frontier_heads.as_slice(),
        );
        let step_only_plan = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            step_only,
            frontier_heads.as_slice(),
        );
        let shipping_on_frontier_heads = turn_engine_candidate_plan_from_allowed_heads(
            &game,
            perspective,
            shipping_rerank,
            frontier_heads.as_slice(),
        );

        println!(
            "WHITE_SEED_STEP_SCOPE label={} context={} frontier_stage={} shipping_stage={} frontier_base={:?} seed_only={:?} step_only={:?} shipping_on_frontier_heads={:?} caps(frontier={{seed:{},step:{}}}, shipping={{seed:{},step:{}}}, shipping_pro_v2={{seed:{},step:{}}})",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selector_last_stage,
            shipping_probe.selector_last_stage,
            describe_plan(frontier_plan.as_ref()),
            describe_plan(seed_only_plan.as_ref()),
            describe_plan(step_only_plan.as_ref()),
            describe_plan(shipping_on_frontier_heads.as_ref()),
            frontier_base.own_seed_cap,
            frontier_base.step_cap,
            shipping_rerank.own_seed_cap,
            shipping_rerank.step_cap,
            shipping_pro_v2.own_seed_cap,
            shipping_pro_v2.step_cap,
        );
    }
}

fn log_white_search_order_split_probe(probe_label: &'static str, board_fen: &'static str) {
    let game = MonsGame::from_fen(board_fen, false)
        .unwrap_or_else(|| panic!("valid white search-order split fen for {probe_label}"));
    let perspective = game.active_color;
    let frontier_probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
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
    let projections = MonsGameModel::turn_engine_reply_risk_projections(
        scored_roots.as_slice(),
        shortlist.as_slice(),
        perspective,
        config,
    );
    let frontier_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == frontier_probe.selected_input_fen)
        .expect("frontier root should exist");
    let frontier_pre_accept_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == frontier_probe.pre_accept_input_fen)
        .expect("frontier pre-accept root should exist");
    let shipping_index = scored_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == shipping_probe.selected_input_fen)
        .expect("shipping root should exist");
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(shortlist.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let frontier_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[frontier_index],
        projections.get(&frontier_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let shipping_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[shipping_index],
        projections.get(&shipping_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let frontier_pre_accept_snapshot = MonsGameModel::root_reply_risk_snapshot_with_projection(
        &scored_roots[frontier_pre_accept_index],
        projections.get(&frontier_pre_accept_index),
        perspective,
        config,
        per_root_reply_limit,
    );
    let frontier_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[frontier_index]);
    let shipping_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[shipping_index]);
    let frontier_pre_accept_family =
        MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[frontier_pre_accept_index]);
    let frontier_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[frontier_index],
        perspective,
        config,
        frontier_family,
    );
    let shipping_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[shipping_index],
        perspective,
        config,
        shipping_family,
    );
    let frontier_pre_accept_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        &scored_roots[frontier_pre_accept_index],
        perspective,
        config,
        frontier_pre_accept_family,
    );
    let shipping_beats_frontier = MonsGameModel::is_better_reply_risk_candidate(
        &game,
        shipping_index,
        shipping_snapshot,
        frontier_index,
        frontier_snapshot,
        projections.get(&shipping_index),
        projections.get(&frontier_index),
        scored_roots.as_slice(),
        perspective,
        config,
        &mut std::collections::HashMap::new(),
    );

    println!(
        "{} context={} shortlist={:?} frontier_probe={:?} shipping_probe={:?} advisor={:?} frontier={} frontier_pre_accept={} shipping={} frontier_snapshot={} frontier_pre_accept_snapshot={} shipping_snapshot={} frontier_utility={} frontier_pre_accept_utility={} shipping_utility={} shipping_vs_frontier={} frontier_projection={:?} frontier_pre_accept_projection={:?} shipping_projection={:?}",
        probe_label,
        exact_opportunity_context_probe(&game),
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        frontier_probe,
        shipping_probe,
        frontier_advisor,
        format_root_probe(scored_roots.get(frontier_index)),
        format_root_probe(scored_roots.get(frontier_pre_accept_index)),
        format_root_probe(scored_roots.get(shipping_index)),
        format!(
            "win={} match_point={} floor={}",
            frontier_snapshot.allows_immediate_opponent_win,
            frontier_snapshot.opponent_reaches_match_point,
            frontier_snapshot.worst_reply_score,
        ),
        format!(
            "win={} match_point={} floor={}",
            frontier_pre_accept_snapshot.allows_immediate_opponent_win,
            frontier_pre_accept_snapshot.opponent_reaches_match_point,
            frontier_pre_accept_snapshot.worst_reply_score,
        ),
        format!(
            "win={} match_point={} floor={}",
            shipping_snapshot.allows_immediate_opponent_win,
            shipping_snapshot.opponent_reaches_match_point,
            shipping_snapshot.worst_reply_score,
        ),
        format_turn_engine_utility_probe(frontier_utility),
        format_turn_engine_utility_probe(frontier_pre_accept_utility),
        format_turn_engine_utility_probe(shipping_utility),
        shipping_beats_frontier,
        projections.get(&frontier_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        projections.get(&frontier_pre_accept_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
        projections.get(&shipping_index).map(|projection| {
            format!(
                "{:?}/{:?}/{}",
                projection.plan.head_family,
                projection.plan.goal_family,
                format_turn_engine_utility_probe(projection.plan.utility),
            )
        }),
    );
}

#[test]
#[ignore = "diagnostic: inspect frontier shortlist gating on white search-order family"]
fn white_search_order_shortlist_gate_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    let cases = [
        ProbeCase {
            label: "white_fast_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
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
        let shipping_index = scored_roots
            .iter()
            .position(|root| {
                Input::fen_from_array(&root.inputs) == shipping_probe.selected_input_fen
            })
            .expect("shipping root should exist");
        let best_candidate_score = candidate_indices
            .iter()
            .map(|index| scored_roots[*index].score)
            .max()
            .unwrap_or(i32::MIN);
        let shipping_score = scored_roots[shipping_index].score;
        let shipping_in_candidates = candidate_indices.contains(&shipping_index);
        let shipping_in_shortlist = shortlist.contains(&shipping_index);
        let shipping_passes_margin = shipping_score
            .saturating_add(config.root_reply_risk_score_margin.max(0))
            >= best_candidate_score;
        let safe_progress_extension =
            MonsGameModel::pro_v2_safe_progress_sibling_shortlist_extension(
                scored_roots.as_slice(),
                candidate_indices.as_slice(),
                shortlist.as_slice(),
                config,
            );
        let shortlist_anchor = shortlist.first().copied();
        let shipping_same_progress_as_anchor = shortlist_anchor.is_some_and(|anchor_index| {
            MonsGameModel::is_same_non_tactical_progress_lane_root_pair(
                &scored_roots[shipping_index],
                &scored_roots[anchor_index],
            )
        });

        println!(
            "WHITE_SEARCH_ORDER_SHORTLIST_GATE label={} context={} frontier_stage={} shipping_stage={} frontier_selected={} shipping_selected={} shipping_in_candidates={} shipping_in_shortlist={} shipping_passes_margin={} shipping_score_gap={} shortlist_extension={:?} shipping_same_progress_as_anchor={} margin={} shortlist_max={} candidate_fens={:?} shortlist_details={:?} shipping_root={} anchor_root={} shipping_root_detail={} extension_root_detail={}",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selector_last_stage,
            shipping_probe.selector_last_stage,
            frontier_probe.selected_input_fen,
            shipping_probe.selected_input_fen,
            shipping_in_candidates,
            shipping_in_shortlist,
            shipping_passes_margin,
            best_candidate_score.saturating_sub(shipping_score),
            safe_progress_extension
                .map(|index| Input::fen_from_array(&scored_roots[index].inputs)),
            shipping_same_progress_as_anchor,
            config.root_reply_risk_score_margin.max(0),
            config.root_reply_risk_shortlist_max.max(1),
            candidate_indices
                .iter()
                .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
                .collect::<Vec<_>>(),
            shortlist
                .iter()
                .map(|index| {
                    format!(
                        "{}:{}",
                        Input::fen_from_array(&scored_roots[*index].inputs),
                        format_root_probe(Some(&scored_roots[*index]))
                    )
                })
                .collect::<Vec<_>>(),
            Input::fen_from_array(&scored_roots[shipping_index].inputs),
            shortlist_anchor
                .map(|index| Input::fen_from_array(&scored_roots[index].inputs))
                .unwrap_or_else(|| "none".to_string()),
            format_root_probe(Some(&scored_roots[shipping_index])),
            format_root_probe(safe_progress_extension.and_then(|index| scored_roots.get(index))),
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect selector-disable variants on white ordering family"]
fn white_search_order_selector_disable_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    #[derive(Debug)]
    struct VariantProbeResult {
        move_fen: String,
        stage: &'static str,
        disable_reason: &'static str,
        runtime_branch: &'static str,
    }

    fn run_frontier_variant(
        game: &MonsGame,
        tweak: impl FnOnce(&mut AutomoveSearchConfig),
    ) -> VariantProbeResult {
        let selector = profile_selector_from_name("frontier_pro_v2_guarded")
            .expect("frontier profile selector should exist");
        let mut config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            game,
            SmartAutomovePreference::Pro,
        );
        tweak(&mut config);

        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();
        clear_frontier_runtime_variant_branch();

        let inputs = select_inputs_with_runtime_fallback(selector, game, config);
        let move_fen = Input::fen_from_array(&inputs);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();

        VariantProbeResult {
            move_fen,
            stage: selector_diag.last_return_stage,
            disable_reason: selector_diag.selector_disable_reason,
            runtime_branch: frontier_runtime_variant_branch_snapshot(),
        }
    }

    let cases = [
        ProbeCase {
            label: "white_fast_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
        ProbeCase {
            label: "white_late_fast_hotspot",
            board_fen:
                "1 1 w 0 0 1 0 0 9 n04s1xn06/n06a0xn04/n05e0xd0xn04/n03xxmxxmn02xxmn03/n05xxmn03Y0xn01/n05xxUn05/E0xn04xxMn01xxMn03/n01y0xn01xxMn03xxMn03/n05S0xn05/n05D0xn05/n04A1xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
        let selector_off_pro_v2 = run_frontier_variant(&game, |config| {
            config.enable_turn_engine_selector = false;
            config.enable_turn_head_rerank = true;
        });
        let selector_off_shipping_own_caps_pro_v2 = run_frontier_variant(&game, |config| {
            config.enable_turn_engine_selector = false;
            config.enable_turn_head_rerank = true;
            let shipping = calibration_runtime_config(
                "shipping_pro_search",
                &game,
                SmartAutomovePreference::Pro,
            );
            config.turn_engine_seed_cap = shipping.turn_engine_seed_cap;
            config.turn_engine_beam_width = shipping.turn_engine_beam_width;
            config.turn_engine_per_node_family_cap = shipping.turn_engine_per_node_family_cap;
            config.turn_engine_step_cap = shipping.turn_engine_step_cap;
        });
        let selector_off_pro_v1 = run_frontier_variant(&game, |config| {
            config.enable_turn_engine_selector = false;
            config.enable_turn_head_rerank = true;
            config.turn_engine_mode = TurnEngineMode::ProV1;
        });

        println!(
            "WHITE_SEARCH_ORDER_SELECTOR_DISABLE label={} context={} frontier(selected={} stage={} disable_reason={} top={:?} selected_root=\"{}\") selector_off_pro_v2(move={} stage={} disable_reason={} branch={}) selector_off_shipping_own_caps_pro_v2(move={} stage={} disable_reason={} branch={}) selector_off_pro_v1(move={} stage={} disable_reason={} branch={}) shipping(selected={} stage={} top={:?} selected_root=\"{}\")",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selected_input_fen,
            frontier_probe.selector_last_stage,
            frontier_probe.selector_disable_reason,
            frontier_probe.top_root_fens,
            frontier_probe.selected_root,
            selector_off_pro_v2.move_fen,
            selector_off_pro_v2.stage,
            selector_off_pro_v2.disable_reason,
            selector_off_pro_v2.runtime_branch,
            selector_off_shipping_own_caps_pro_v2.move_fen,
            selector_off_shipping_own_caps_pro_v2.stage,
            selector_off_shipping_own_caps_pro_v2.disable_reason,
            selector_off_shipping_own_caps_pro_v2.runtime_branch,
            selector_off_pro_v1.move_fen,
            selector_off_pro_v1.stage,
            selector_off_pro_v1.disable_reason,
            selector_off_pro_v1.runtime_branch,
            shipping_probe.selected_input_fen,
            shipping_probe.selector_last_stage,
            shipping_probe.top_root_fens,
            shipping_probe.selected_root,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect raw wrapper-branch variants on white ordering family"]
fn white_search_order_wrapper_branch_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    #[derive(Debug)]
    struct VariantProbeResult {
        move_fen: String,
        stage: &'static str,
        disable_reason: &'static str,
    }

    fn run_raw_search_variant(
        game: &MonsGame,
        tweak: impl FnOnce(&mut AutomoveSearchConfig),
    ) -> VariantProbeResult {
        let selector = profile_selector_from_name("shipping_pro_search")
            .expect("shipping profile selector should exist");
        let mut config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            game,
            SmartAutomovePreference::Pro,
        );
        tweak(&mut config);

        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let inputs = select_inputs_with_runtime_fallback(selector, game, config);
        let move_fen = Input::fen_from_array(&inputs);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();

        VariantProbeResult {
            move_fen,
            stage: selector_diag.last_return_stage,
            disable_reason: selector_diag.selector_disable_reason,
        }
    }

    let cases = [
        ProbeCase {
            label: "white_fast_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
        ProbeCase {
            label: "white_late_fast_hotspot",
            board_fen:
                "1 1 w 0 0 1 0 0 9 n04s1xn06/n06a0xn04/n05e0xd0xn04/n03xxmxxmn02xxmn03/n05xxmn03Y0xn01/n05xxUn05/E0xn04xxMn01xxMn03/n01y0xn01xxMn03xxMn03/n05S0xn05/n05D0xn05/n04A1xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
        let raw_search_only_pro_v2 = run_raw_search_variant(&game, |config| {
            config.enable_turn_engine_selector = false;
            config.enable_turn_head_rerank = true;
        });
        let raw_search_only_shipping_own_caps_pro_v2 = run_raw_search_variant(&game, |config| {
            config.enable_turn_engine_selector = false;
            config.enable_turn_head_rerank = true;
            let shipping = calibration_runtime_config(
                "shipping_pro_search",
                &game,
                SmartAutomovePreference::Pro,
            );
            config.turn_engine_seed_cap = shipping.turn_engine_seed_cap;
            config.turn_engine_beam_width = shipping.turn_engine_beam_width;
            config.turn_engine_per_node_family_cap = shipping.turn_engine_per_node_family_cap;
            config.turn_engine_step_cap = shipping.turn_engine_step_cap;
        });
        let raw_search_only_pro_v1 = run_raw_search_variant(&game, |config| {
            config.enable_turn_engine_selector = false;
            config.enable_turn_head_rerank = true;
            config.turn_engine_mode = TurnEngineMode::ProV1;
        });

        println!(
            "WHITE_SEARCH_ORDER_WRAPPER_BRANCH label={} context={} frontier(selected={} stage={} disable_reason={}) raw_search_only_pro_v2(move={} stage={} disable_reason={} matches_shipping={}) raw_search_only_shipping_own_caps_pro_v2(move={} stage={} disable_reason={} matches_shipping={}) raw_search_only_pro_v1(move={} stage={} disable_reason={} matches_shipping={}) shipping(selected={} stage={})",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selected_input_fen,
            frontier_probe.selector_last_stage,
            frontier_probe.selector_disable_reason,
            raw_search_only_pro_v2.move_fen,
            raw_search_only_pro_v2.stage,
            raw_search_only_pro_v2.disable_reason,
            raw_search_only_pro_v2.move_fen == shipping_probe.selected_input_fen,
            raw_search_only_shipping_own_caps_pro_v2.move_fen,
            raw_search_only_shipping_own_caps_pro_v2.stage,
            raw_search_only_shipping_own_caps_pro_v2.disable_reason,
            raw_search_only_shipping_own_caps_pro_v2.move_fen == shipping_probe.selected_input_fen,
            raw_search_only_pro_v1.move_fen,
            raw_search_only_pro_v1.stage,
            raw_search_only_pro_v1.disable_reason,
            raw_search_only_pro_v1.move_fen == shipping_probe.selected_input_fen,
            shipping_probe.selected_input_fen,
            shipping_probe.selector_last_stage,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect raw search-only prov1 scope on white retained slice"]
fn white_search_order_raw_prov1_scope_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn raw_search_only_pro_v1_move(game: &MonsGame) -> (String, &'static str, &'static str) {
        let selector = profile_selector_from_name("shipping_pro_search")
            .expect("shipping profile selector should exist");
        let mut config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            game,
            SmartAutomovePreference::Pro,
        );
        config.enable_turn_engine_selector = false;
        config.enable_turn_head_rerank = true;
        config.turn_engine_mode = TurnEngineMode::ProV1;

        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let inputs = select_inputs_with_runtime_fallback(selector, game, config);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        (
            Input::fen_from_array(&inputs),
            selector_diag.last_return_stage,
            selector_diag.selector_disable_reason,
        )
    }

    let cases = [
        ProbeCase {
            label: "target_white_fast_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "target_white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
        ProbeCase {
            label: "target_white_late_fast_hotspot",
            board_fen:
                "1 1 w 0 0 1 0 0 9 n04s1xn06/n06a0xn04/n05e0xd0xn04/n03xxmxxmn02xxmn03/n05xxmn03Y0xn01/n05xxUn05/E0xn04xxMn01xxMn03/n01y0xn01xxMn03xxMn03/n05S0xn05/n05D0xn05/n04A1xn06",
        },
        ProbeCase {
            label: "guard_white_turn_three_mana_only_vulnerable",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn02Y0xn03",
        },
        ProbeCase {
            label: "guard_white_turn_three_mana_only_non_vulnerable",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n05S0xn05/n03E0xA0xD0xn02Y0xn02",
        },
        ProbeCase {
            label: "guard_white_early_engine_disabled_normal",
            board_fen:
                "0 0 w 0 0 0 0 0 5 n06a0xn04/n07d0me0xn02/n02y0xn01s0xn06/n04xxmn01xxmxxmn03/n03xxmn07/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn02Y0xxxMn04/n03xxMn01D0xn05/n03E0xA0xS0xn05/n11",
        },
        ProbeCase {
            label: "guard_white_post_search_duel_pro",
            board_fen:
                "1 1 w 1 0 0 0 0 5 n10d0x/n03y0xn03a0xn03/n01xxmn04s0xn01e0xn02/n04xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n06xxMn04/n02xxMn02S0xn05/n05A0xY0xn04/D0xn02E0xn07",
        },
        ProbeCase {
            label: "guard_white_confirm_pro_ply9",
            board_fen:
                "0 0 w 1 0 0 0 0 3 n11/n03y0xd0ms0xa0xe0xn03/n11/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn06/n07xxMn01Y0xn01/n05S0xn01D0xn03/n03E0xA0xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
        let (raw_move, raw_stage, raw_disable_reason) = raw_search_only_pro_v1_move(&game);

        println!(
            "WHITE_SEARCH_ORDER_RAW_PROV1_SCOPE label={} context={} frontier(selected={} stage={}) raw_search_only_pro_v1(move={} stage={} disable_reason={} matches_frontier={} matches_shipping={}) shipping(selected={} stage={})",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selected_input_fen,
            frontier_probe.selector_last_stage,
            raw_move,
            raw_stage,
            raw_disable_reason,
            raw_move == frontier_probe.selected_input_fen,
            raw_move == shipping_probe.selected_input_fen,
            shipping_probe.selected_input_fen,
            shipping_probe.selector_last_stage,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect white fast ply9 search-only split"]
fn white_fast_ply9_search_only_split_probe() {
    log_white_search_order_split_probe(
        "WHITE_FAST_PLY9_SEARCH_ONLY_SPLIT",
        "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
    );
}

#[test]
#[ignore = "diagnostic: inspect retained vulnerable white mana-only guard against search-order family"]
fn white_vulnerable_guard_search_order_probe() {
    log_white_search_order_split_probe(
        "WHITE_VULNERABLE_GUARD_SEARCH_ORDER",
        "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn02Y0xn03",
    );
}

#[test]
#[ignore = "diagnostic: inspect non-negative deny gate on raw white search-only prov1 scope"]
fn white_search_order_non_negative_deny_scope_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn raw_search_only_pro_v1_move(game: &MonsGame) -> (String, &'static str, &'static str) {
        let selector = profile_selector_from_name("shipping_pro_search")
            .expect("shipping profile selector should exist");
        let mut config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            game,
            SmartAutomovePreference::Pro,
        );
        config.enable_turn_engine_selector = false;
        config.enable_turn_head_rerank = true;
        config.turn_engine_mode = TurnEngineMode::ProV1;

        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let inputs = select_inputs_with_runtime_fallback(selector, game, config);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        (
            Input::fen_from_array(&inputs),
            selector_diag.last_return_stage,
            selector_diag.selector_disable_reason,
        )
    }

    let cases = [
        ProbeCase {
            label: "target_white_fast_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "target_white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
        ProbeCase {
            label: "target_white_late_fast_hotspot",
            board_fen:
                "1 1 w 0 0 1 0 0 9 n04s1xn06/n06a0xn04/n05e0xd0xn04/n03xxmxxmn02xxmn03/n05xxmn03Y0xn01/n05xxUn05/E0xn04xxMn01xxMn03/n01y0xn01xxMn03xxMn03/n05S0xn05/n05D0xn05/n04A1xn06",
        },
        ProbeCase {
            label: "guard_white_turn_three_mana_only_vulnerable",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn02Y0xn03",
        },
        ProbeCase {
            label: "guard_white_turn_three_mana_only_non_vulnerable",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n05S0xn05/n03E0xA0xD0xn02Y0xn02",
        },
        ProbeCase {
            label: "guard_white_early_engine_disabled_normal",
            board_fen:
                "0 0 w 0 0 0 0 0 5 n06a0xn04/n07d0me0xn02/n02y0xn01s0xn06/n04xxmn01xxmxxmn03/n03xxmn07/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn02Y0xxxMn04/n03xxMn01D0xn05/n03E0xA0xS0xn05/n11",
        },
        ProbeCase {
            label: "guard_white_post_search_duel_pro",
            board_fen:
                "1 1 w 1 0 0 0 0 5 n10d0x/n03y0xn03a0xn03/n01xxmn04s0xn01e0xn02/n04xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n06xxMn04/n02xxMn02S0xn05/n05A0xY0xn04/D0xn02E0xn07",
        },
        ProbeCase {
            label: "guard_white_confirm_pro_ply9",
            board_fen:
                "0 0 w 1 0 0 0 0 3 n11/n03y0xd0ms0xa0xe0xn03/n11/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn06/n07xxMn01Y0xn01/n05S0xn01D0xn03/n03E0xA0xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
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
        let frontier_index = scored_roots
            .iter()
            .position(|root| {
                Input::fen_from_array(&root.inputs) == frontier_probe.selected_input_fen
            })
            .expect("frontier selected root should exist");
        let frontier_family =
            MonsGameModel::turn_engine_root_evaluation_family(&scored_roots[frontier_index]);
        let frontier_utility = MonsGameModel::turn_engine_selected_override_utility(
            &game,
            &scored_roots[frontier_index],
            perspective,
            config,
            frontier_family,
        );
        let (raw_move, raw_stage, raw_disable_reason) = raw_search_only_pro_v1_move(&game);
        let gate_matches = frontier_probe.selector_last_stage == "engine_post_search"
            && frontier_utility.has_nonnegative_deny_gain()
            && raw_move != frontier_probe.selected_input_fen;

        println!(
            "WHITE_SEARCH_ORDER_NON_NEGATIVE_DENY_SCOPE label={} context={} frontier(selected={} stage={} utility={}) raw_search_only_pro_v1(move={} stage={} disable_reason={} matches_frontier={} matches_shipping={}) gate_matches={} shipping(selected={} stage={})",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selected_input_fen,
            frontier_probe.selector_last_stage,
            format_turn_engine_utility_probe(frontier_utility),
            raw_move,
            raw_stage,
            raw_disable_reason,
            raw_move == frontier_probe.selected_input_fen,
            raw_move == shipping_probe.selected_input_fen,
            gate_matches,
            shipping_probe.selected_input_fen,
            shipping_probe.selector_last_stage,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect raw white search-only shipping-own-caps prov2 scope"]
fn white_search_order_shipping_caps_scope_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn raw_search_only_shipping_own_caps_pro_v2_move(
        game: &MonsGame,
    ) -> (String, &'static str, &'static str) {
        let selector = profile_selector_from_name("shipping_pro_search")
            .expect("shipping profile selector should exist");
        let mut config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            game,
            SmartAutomovePreference::Pro,
        );
        config.enable_turn_engine_selector = false;
        config.enable_turn_head_rerank = true;
        let shipping =
            calibration_runtime_config("shipping_pro_search", game, SmartAutomovePreference::Pro);
        config.turn_engine_seed_cap = shipping.turn_engine_seed_cap;
        config.turn_engine_beam_width = shipping.turn_engine_beam_width;
        config.turn_engine_per_node_family_cap = shipping.turn_engine_per_node_family_cap;
        config.turn_engine_step_cap = shipping.turn_engine_step_cap;

        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let inputs = select_inputs_with_runtime_fallback(selector, game, config);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        (
            Input::fen_from_array(&inputs),
            selector_diag.last_return_stage,
            selector_diag.selector_disable_reason,
        )
    }

    let cases = [
        ProbeCase {
            label: "target_white_fast_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "target_white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
        ProbeCase {
            label: "target_white_late_fast_hotspot",
            board_fen:
                "1 1 w 0 0 1 0 0 9 n04s1xn06/n06a0xn04/n05e0xd0xn04/n03xxmxxmn02xxmn03/n05xxmn03Y0xn01/n05xxUn05/E0xn04xxMn01xxMn03/n01y0xn01xxMn03xxMn03/n05S0xn05/n05D0xn05/n04A1xn06",
        },
        ProbeCase {
            label: "guard_white_turn_three_mana_only_vulnerable",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn02Y0xn03",
        },
        ProbeCase {
            label: "guard_white_turn_three_mana_only_non_vulnerable",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n05S0xn05/n03E0xA0xD0xn02Y0xn02",
        },
        ProbeCase {
            label: "guard_white_early_engine_disabled_normal",
            board_fen:
                "0 0 w 0 0 0 0 0 5 n06a0xn04/n07d0me0xn02/n02y0xn01s0xn06/n04xxmn01xxmxxmn03/n03xxmn07/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn02Y0xxxMn04/n03xxMn01D0xn05/n03E0xA0xS0xn05/n11",
        },
        ProbeCase {
            label: "guard_white_post_search_duel_pro",
            board_fen:
                "1 1 w 1 0 0 0 0 5 n10d0x/n03y0xn03a0xn03/n01xxmn04s0xn01e0xn02/n04xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n06xxMn04/n02xxMn02S0xn05/n05A0xY0xn04/D0xn02E0xn07",
        },
        ProbeCase {
            label: "guard_white_confirm_pro_ply9",
            board_fen:
                "0 0 w 1 0 0 0 0 3 n11/n03y0xd0ms0xa0xe0xn03/n11/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn06/n07xxMn01Y0xn01/n05S0xn01D0xn03/n03E0xA0xn06",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
        let (raw_move, raw_stage, raw_disable_reason) =
            raw_search_only_shipping_own_caps_pro_v2_move(&game);

        println!(
            "WHITE_SEARCH_ORDER_SHIPPING_CAPS_SCOPE label={} context={} frontier(selected={} stage={}) raw_search_only_shipping_own_caps_pro_v2(move={} stage={} disable_reason={} matches_frontier={} matches_shipping={}) shipping(selected={} stage={})",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selected_input_fen,
            frontier_probe.selector_last_stage,
            raw_move,
            raw_stage,
            raw_disable_reason,
            raw_move == frontier_probe.selected_input_fen,
            raw_move == shipping_probe.selected_input_fen,
            shipping_probe.selected_input_fen,
            shipping_probe.selector_last_stage,
        );
    }
}

#[test]
#[ignore = "diagnostic: compare selected-rank vs root-rank on white negative-deny search-order boards"]
fn white_search_order_rank_surface_probe() {
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn raw_search_only_shipping_own_caps_snapshot(
        game: &MonsGame,
    ) -> (String, Option<usize>, usize, &'static str, &'static str) {
        let selector = profile_selector_from_name("shipping_pro_search")
            .expect("shipping profile selector should exist");
        let mut config = calibration_runtime_config(
            "frontier_pro_v2_guarded",
            game,
            SmartAutomovePreference::Pro,
        );
        config.enable_turn_engine_selector = false;
        config.enable_turn_head_rerank = true;
        let shipping =
            calibration_runtime_config("shipping_pro_search", game, SmartAutomovePreference::Pro);
        config.turn_engine_seed_cap = shipping.turn_engine_seed_cap;
        config.turn_engine_beam_width = shipping.turn_engine_beam_width;
        config.turn_engine_per_node_family_cap = shipping.turn_engine_per_node_family_cap;
        config.turn_engine_step_cap = shipping.turn_engine_step_cap;

        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let inputs = select_inputs_with_runtime_fallback(selector, game, config);
        let input_fen = Input::fen_from_array(&inputs);
        let scored_roots = scored_roots_for_runtime_config(game, config);
        let selected_rank = scored_roots.iter().position(|root| root.inputs == inputs);
        let root_rank = scored_roots
            .iter()
            .find(|root| root.inputs.as_slice() == inputs.as_slice())
            .map(|root| root.root_rank)
            .expect("raw shipping-own-caps move should exist in ranked roots");
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        (
            input_fen,
            selected_rank,
            root_rank,
            selector_diag.last_return_stage,
            selector_diag.selector_disable_reason,
        )
    }

    let cases = [
        ProbeCase {
            label: "target_white_fast_ply9_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        },
        ProbeCase {
            label: "target_white_normal_ply11_search_ordering",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        },
        ProbeCase {
            label: "guard_white_turn_three_mana_only_vulnerable",
            board_fen:
                "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn02Y0xn03",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let shipping_probe =
            runtime_decision_probe("shipping_pro_search", SmartAutomovePreference::Pro, &game);
        let shipping_runtime =
            calibration_runtime_config("shipping_pro_search", &game, SmartAutomovePreference::Pro);
        let shipping_inputs =
            automove_runtime_variants::select_shipping_search_inputs(&game, shipping_runtime);
        let shipping_roots =
            MonsGameModel::ranked_root_moves(&game, game.active_color, shipping_runtime);
        let shipping_root_rank = shipping_roots
            .iter()
            .find(|root| root.inputs.as_slice() == shipping_inputs.as_slice())
            .map(|root| root.root_rank)
            .expect("shipping selected move should exist in ranked roots");
        let (
            raw_caps_move,
            raw_caps_selected_rank,
            raw_caps_root_rank,
            raw_caps_stage,
            raw_caps_disable_reason,
        ) = raw_search_only_shipping_own_caps_snapshot(&game);

        println!(
            "WHITE_SEARCH_ORDER_RANK_SURFACE label={} context={} frontier(selected={} stage={}) shipping(selected={} selected_rank={:?} root_rank={}) raw_shipping_caps(move={} selected_rank={:?} root_rank={} stage={} disable_reason={})",
            case.label,
            exact_opportunity_context_probe(&game),
            frontier_probe.selected_input_fen,
            frontier_probe.selector_last_stage,
            shipping_probe.selected_input_fen,
            shipping_probe.selected_rank,
            shipping_root_rank,
            raw_caps_move,
            raw_caps_selected_rank,
            raw_caps_root_rank,
            raw_caps_stage,
            raw_caps_disable_reason,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect retained normal white ply11 search-only split"]
fn white_normal_ply11_search_only_split_probe() {
    log_white_search_order_split_probe(
        "WHITE_NORMAL_PLY11_SEARCH_ONLY_SPLIT",
        "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
    );
}

#[test]
#[ignore = "diagnostic: inspect white pro ply15 search-only split"]
fn white_pro_ply15_search_only_split_probe() {
    log_white_search_order_split_probe(
        "WHITE_PRO_PLY15_SEARCH_ONLY_SPLIT",
        "0 0 w 1 0 5 0 0 3 n11/n03y0xd0ms0xa0xe0xn03/n11/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04Y0x/n03xxMn01xxMn01xxMn03/n04xxMn06/n11/n05S0xn02D0Mn02/n03E0xA0xn06",
    );
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
    let duel_specs = vec![
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

    with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
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
    });
}

#[test]
#[ignore = "diagnostic: inspect ranked roots on the exact five-board retained pro-reliability non-win surface"]
fn smart_automove_pro_reliability_live_nonwin_root_probe() {
    #[derive(Clone, Copy)]
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
        shipping_mode: SmartAutomovePreference,
    }

    fn top_root_details(
        profile: &str,
        mode: SmartAutomovePreference,
        game: &MonsGame,
    ) -> Vec<String> {
        let (_, scored_roots, _, _) =
            profile_runtime_scored_roots_with_forced_engine_inputs(profile, mode, game);
        scored_roots
            .iter()
            .take(8)
            .map(|root| {
                format!(
                    "{}:{}",
                    Input::fen_from_array(&root.inputs),
                    format_root_probe(Some(root))
                )
            })
            .collect()
    }

    let cases = [
        ProbeCase {
            label: "vs_shipping_pro_opening_reply_white",
            board_fen: "1 0 w 1 0 1 0 0 5 n01d0xn09/n01xxmn04a0xe0xn03/n03y0xn01s0xn01xxmn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn02xxMn03/n05S0xn05/n04E0xA0xn01Y0xn03/n10D0x",
            shipping_mode: SmartAutomovePreference::Pro,
        },
        ProbeCase {
            label: "vs_shipping_pro_black_recovery_branch",
            board_fen: "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
            shipping_mode: SmartAutomovePreference::Pro,
        },
        ProbeCase {
            label: "vs_shipping_pro_white_split_trace",
            board_fen: "0 0 w 1 0 4 0 0 3 n03y0xn03e0xn03/n05a0xn05/n02xxmn01s0xn01d0xn04/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMn04/E0xn04S0xn05/n05D0xn05/n04A0xn03Y0xn02",
            shipping_mode: SmartAutomovePreference::Pro,
        },
        ProbeCase {
            label: "vs_shipping_normal_black_bridge_nonwin",
            board_fen: "0 0 w 0 0 2 0 0 3 n11/n03y0xn01s0xa0xe0xn03/n05d0xn05/n03xxmxxmn01xxmn04/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n01E0xn01xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xS0xn01Y0xn03/n04A0xn06",
            shipping_mode: SmartAutomovePreference::Normal,
        },
        ProbeCase {
            label: "vs_shipping_normal_white_head_acceptance",
            board_fen: "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
            shipping_mode: SmartAutomovePreference::Normal,
        },
    ];
    let frontier_profile = probe_frontier_profile_id();
    let shipping_profile = probe_shipping_profile_id();

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid live non-win board fen", case.label));
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();
        let frontier_probe = runtime_decision_probe(
            frontier_profile.as_str(),
            SmartAutomovePreference::Pro,
            &game,
        );
        let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
        let frontier_roots = top_root_details(
            frontier_profile.as_str(),
            SmartAutomovePreference::Pro,
            &game,
        );

        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();
        let shipping_probe =
            runtime_decision_probe(shipping_profile.as_str(), case.shipping_mode, &game);
        let shipping_roots = top_root_details(shipping_profile.as_str(), case.shipping_mode, &game);

        println!(
            "LIVE_NONWIN_ROOT label={} frontier_profile={} shipping_profile={} shipping_mode={:?} frontier_probe={:?} frontier_advisor={:?} frontier_roots={:?} shipping_probe={:?} shipping_roots={:?}",
            case.label,
            frontier_profile,
            shipping_profile,
            case.shipping_mode,
            frontier_probe,
            frontier_advisor,
            frontier_roots,
            shipping_probe,
            shipping_roots,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect white turn-three sibling surfaces on retained and duel boards"]
fn smart_automove_pro_white_turn_three_sibling_root_probe() {
    #[derive(Clone, Copy)]
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    fn top_root_details(game: &MonsGame) -> Vec<String> {
        let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            game,
        );
        scored_roots
            .iter()
            .take(10)
            .map(|root| {
                format!(
                    "{}:{}",
                    Input::fen_from_array(&root.inputs),
                    format_root_probe(Some(root))
                )
            })
            .collect()
    }

    let cases = [
        ProbeCase {
            label: "retained_normal_v92",
            board_fen: "0 0 w 1 0 4 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMY0xn03/n05S0xn05/n04A0xD0xn05/n02E0xn08",
        },
        ProbeCase {
            label: "retained_fast_v94",
            board_fen: "0 0 w 1 0 4 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn03Y0xn03/n03E0xn01S0xn05/n04A0xD0xn05",
        },
        ProbeCase {
            label: "duel_pro_new_turn_three",
            board_fen: "0 0 w 1 0 3 0 0 3 n03y0xn03e0xn03/n05a0xn05/n02xxmn01s0xn01d0xn04/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMn04/E0xn04S0xn05/n03A0xn01D0xn05/n08Y0xn02",
        },
        ProbeCase {
            label: "duel_pro_split_trace",
            board_fen: "0 0 w 1 0 4 0 0 3 n03y0xn03e0xn03/n05a0xn05/n02xxmn01s0xn01d0xn04/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMn04/E0xn04S0xn05/n05D0xn05/n04A0xn03Y0xn02",
        },
    ];

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .unwrap_or_else(|| panic!("{}: valid board fen", case.label));
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();
        let frontier_probe = runtime_decision_probe(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        );
        let frontier_advisor = pro_v2_root_advisor_decision_snapshot();
        let frontier_roots = top_root_details(&game);
        println!(
            "WHITE_T3_SIBLING label={} probe={:?} advisor={:?} roots={:?}",
            case.label, frontier_probe, frontier_advisor, frontier_roots,
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
        label: &'static str,
        game: MonsGame,
        mode: SmartAutomovePreference,
        opening_book_driven: bool,
        config_tweak: Option<fn(AutomoveSearchConfig) -> AutomoveSearchConfig>,
    }

    fn probe_case_from_fixture(label: &'static str, fixture: TriageFixture) -> ProbeCase {
        ProbeCase {
            label,
            game: fixture.game,
            mode: fixture.mode,
            opening_book_driven: fixture.opening_book_driven,
            config_tweak: fixture.config_tweak,
        }
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

    let cases = vec![
        probe_case_from_fixture(
            "primary_black_loss_opening_a_ply19",
            primary_pro_fixture_by_id("primary_black_loss_opening_a_ply19"),
        ),
        probe_case_from_fixture("human_win_pro_a", primary_pro_fixture_by_id("human_win_pro_a")),
        ProbeCase {
            label: "loss_opening_a",
            game: MonsGame::from_fen(
                "0 0 w 0 0 1 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n02E0xn01A0xD0xS0xY0xn03",
                false,
            )
            .expect("loss opening a fen should be valid"),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
        },
        ProbeCase {
            label: "loss_opening_b",
            game: MonsGame::from_fen(
                "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03",
                false,
            )
            .expect("loss opening b fen should be valid"),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
        },
        ProbeCase {
            label: "quiet_positional",
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
            opening_book_driven: false,
            config_tweak: None,
        },
    ];

    println!(
        "pro reliability hotspot probe: frontier={} shipping={} positions={}",
        frontier_profile,
        shipping_profile,
        cases.len()
    );
    for case in cases {
        with_env_override(
            "SMART_USE_WHITE_OPENING_BOOK",
            if case.opening_book_driven {
                "true"
            } else {
                "false"
            },
            || {
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
            },
        );
    }
}

#[test]
#[ignore = "diagnostic: trace the retained Fast hotspot opening and its first real divergence"]
fn fast_hotspot_trace_probe() {
    fn turn_digest(trace: &DuelTraceGame) -> Vec<String> {
        trace
            .profile_a_turns
            .iter()
            .take(6)
            .map(|turn| {
                format!(
                    "ply={} board={} move={}",
                    turn.ply, turn.board_fen, turn.move_fen
                )
            })
            .collect()
    }

    let opening_fen =
        "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03";
    let frontier_profile = reliability_frontier_profile_id();
    let shipping_profile = reliability_shipping_profile_id();

    with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
        println!(
            "fast hotspot trace probe: frontier={} shipping={} opening={}",
            frontier_profile, shipping_profile, opening_fen
        );

        for frontier_is_white in [true, false] {
            let frontier_trace = play_profile_duel_trace(
                frontier_profile.as_str(),
                shipping_profile.as_str(),
                SmartAutomovePreference::Fast,
                opening_fen,
                frontier_is_white,
                96,
            );
            let shipping_trace = play_profile_duel_trace(
                shipping_profile.as_str(),
                shipping_profile.as_str(),
                SmartAutomovePreference::Fast,
                opening_fen,
                frontier_is_white,
                96,
            );
            let divergence = first_duel_trace_divergence(&frontier_trace, &shipping_trace).map(
                |diff| {
                    let board = MonsGame::from_fen(diff.board_fen.as_str(), false)
                        .expect("divergence board fen should be valid");
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
                        diff.ply,
                        diff.board_fen,
                        diff.profile_a_move_fen,
                        diff.profile_b_move_fen,
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
                },
            );

            println!(
                "FAST_HOTSPOT frontier_is_white={} identical_trace={} frontier_result={} shipping_result={} frontier_final={} shipping_final={} frontier_profile_a_turns={} shipping_profile_a_turns={} {} frontier_turn_digest={:?} shipping_turn_digest={:?}",
                frontier_is_white,
                frontier_trace == shipping_trace,
                format_match_result(frontier_trace.result),
                format_match_result(shipping_trace.result),
                frontier_trace.final_fen,
                shipping_trace.final_fen,
                frontier_trace.profile_a_turns.len(),
                shipping_trace.profile_a_turns.len(),
                divergence.unwrap_or_else(|| "first_diff=none".to_string()),
                turn_digest(&frontier_trace),
                turn_digest(&shipping_trace),
            );
        }
    });
}

#[test]
#[ignore = "diagnostic: trace unified ProV2 root advisor decisions on retained footholds and duel boards"]
fn smart_automove_pro_root_advisor_trace_probe() {
    #[derive(Clone)]
    struct AdvisorTraceCase {
        label: &'static str,
        game: MonsGame,
        mode: SmartAutomovePreference,
        opening_book_driven: bool,
        expect_selected_matches_approved: bool,
    }

    fn case_from_fixture(id: &'static str) -> AdvisorTraceCase {
        let fixture = primary_pro_fixture_by_id(id);
        AdvisorTraceCase {
            label: id,
            game: fixture.game,
            mode: fixture.mode,
            opening_book_driven: fixture.opening_book_driven,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
            expect_selected_matches_approved: true,
        },
        AdvisorTraceCase {
            label: "duel_trace_fast_black_mana",
            game: primary_pro_fixture_by_id("primary_black_turn_four_action_mana_ply15").game,
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
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
            opening_book_driven: false,
            expect_selected_matches_approved: true,
        },
    ];

    for case in cases {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        with_env_override(
            "SMART_USE_WHITE_OPENING_BOOK",
            if case.opening_book_driven {
                "true"
            } else {
                "false"
            },
            || {
                let configured_runtime =
                    calibration_runtime_config("frontier_pro_v2_guarded", &case.game, case.mode);
                let selected =
                    MonsGameModel::smart_search_best_inputs(&case.game, configured_runtime);
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
            },
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
        with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
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
                let selected_root = scored_roots.iter().find(|root| {
                    Input::fen_from_array(&root.inputs) == snapshot.selected_input_fen
                });
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
        });
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
