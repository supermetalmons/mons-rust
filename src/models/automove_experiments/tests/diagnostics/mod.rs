use super::*;
use crate::models::mons_game_model::automove_runtime_variants::{
    apply_frontier_pro_v2_guarded_config, select_frontier_pro_v2_guarded_inputs,
    select_shipping_search_inputs,
};
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

fn select_sweep_frontier_pro_v2_raw_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    select_sweep_frontier_config_inputs(game, apply_frontier_pro_v2_guarded_config(config))
}

fn select_sweep_frontier_pro_v2_head_rerank_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    let mut runtime = apply_frontier_pro_v2_guarded_config(config);
    runtime.enable_turn_head_rerank = true;
    select_sweep_frontier_config_inputs(game, runtime)
}

fn select_sweep_frontier_pro_v2_no_spirit_family_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    let mut runtime = apply_frontier_pro_v2_guarded_config(config);
    runtime.turn_engine_enable_spirit_family = false;
    select_sweep_frontier_config_inputs(game, runtime)
}

fn select_sweep_frontier_pro_v2_no_mid_tactical_guard_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    let mut runtime = apply_frontier_pro_v2_guarded_config(config);
    runtime.enable_turn_engine_mid_turn_tactical_guard = false;
    select_sweep_frontier_config_inputs(game, runtime)
}

fn select_sweep_frontier_pro_v2_expansion_224_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    let mut runtime = apply_frontier_pro_v2_guarded_config(config);
    runtime.turn_engine_expansion_cap = 224;
    select_sweep_frontier_config_inputs(game, runtime)
}

fn select_sweep_frontier_pro_v2_no_low_budget_guard_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    let mut runtime = apply_frontier_pro_v2_guarded_config(config);
    runtime.enable_turn_engine_low_budget_guard = false;
    select_sweep_frontier_config_inputs(game, runtime)
}

fn pro_profile_sweep_candidates() -> Vec<ProProfileSweepCandidate> {
    vec![
        ProProfileSweepCandidate {
            id: "shipping_pro_search_control",
            selector: select_sweep_shipping_pro_search_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_guarded",
            selector: select_frontier_pro_v2_guarded_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_raw",
            selector: select_sweep_frontier_pro_v2_raw_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_head_rerank",
            selector: select_sweep_frontier_pro_v2_head_rerank_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_no_spirit_family",
            selector: select_sweep_frontier_pro_v2_no_spirit_family_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_no_mid_tactical_guard",
            selector: select_sweep_frontier_pro_v2_no_mid_tactical_guard_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_expansion_224",
            selector: select_sweep_frontier_pro_v2_expansion_224_inputs,
        },
        ProProfileSweepCandidate {
            id: "frontier_pro_v2_no_low_budget_guard",
            selector: select_sweep_frontier_pro_v2_no_low_budget_guard_inputs,
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
    let duel_specs = vec![
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
