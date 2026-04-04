use super::harness::*;
use super::profiles::*;
use super::*;
use crate::models::automove_exact::{
    clear_exact_query_diagnostics, clear_exact_state_analysis_cache, exact_opportunity_context,
    exact_query_diagnostics_snapshot, exact_strategic_analysis,
};
use crate::models::automove_turn_engine::{
    clear_turn_engine_diagnostics, clear_turn_engine_plan_cache, turn_engine_best_plan_for_test,
    turn_engine_cached_step, turn_engine_diagnostics_snapshot, turn_engine_probe,
    turn_engine_ranked_plan_digests_for_test, TurnEngineConfig, TurnEngineProbeStatus,
    TurnPlanFamily,
};
use crate::models::automove_turn_planner::{
    clear_turn_planner_diagnostics, turn_planner_diagnostics_snapshot,
};
use crate::models::mons_game_model::{
    clear_turn_engine_selector_diagnostics, turn_engine_selector_diagnostics_snapshot,
};

fn stage1_cpu_budgets(profile_name: &str) -> Vec<SearchBudget> {
    if profile_name.starts_with("runtime_pro_") {
        return vec![pro_budget()];
    }

    let mut budgets = client_budgets().to_vec();
    if env_bool("SMART_STAGE1_INCLUDE_PRO").unwrap_or(false) {
        budgets.push(pro_budget());
    }
    budgets
}

fn stage1_cpu_ratio_limit(mode: &str) -> f64 {
    match mode {
        "fast" => SMART_STAGE1_CPU_RATIO_MAX_FAST,
        "normal" => SMART_STAGE1_CPU_RATIO_MAX_NORMAL,
        "pro" => SMART_STAGE1_CPU_RATIO_MAX_PRO,
        _ => SMART_STAGE1_CPU_RATIO_MAX_PRO,
    }
}

fn stage1_seed_tags() -> Vec<String> {
    let from_env = env::var("SMART_STAGE1_SEED_TAGS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !from_env.is_empty() {
        assert!(
            from_env.len() >= 3,
            "stage-1 cpu gate requires at least 3 seeds; got {}",
            from_env.len()
        );
        return from_env;
    }
    vec![
        "stage1_cpu_v1".to_string(),
        "stage1_cpu_v2".to_string(),
        "stage1_cpu_v3".to_string(),
    ]
}

fn stage1_cpu_measurement_repeats() -> usize {
    env_usize("SMART_STAGE1_MEASUREMENT_REPEATS")
        .unwrap_or(3)
        .max(1)
}

fn median_f64(values: &mut [f64]) -> f64 {
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn opening_reply_speed_probe_avg_ms(
    profile_name: &str,
    selector: AutomoveSelector,
    fixtures: &[TriageFixture],
) -> f64 {
    use std::time::Instant;

    with_env_override("SMART_USE_WHITE_OPENING_BOOK", "true", || {
        let mut total_ms = 0.0;
        for fixture in fixtures {
            clear_exact_state_analysis_cache();
            let base_config =
                SearchBudget::from_preference(fixture.mode).runtime_config_for_game(&fixture.game);
            let start = Instant::now();
            let inputs = select_inputs_with_runtime_fallback(selector, &fixture.game, base_config);
            total_ms += start.elapsed().as_secs_f64() * 1000.0;
            assert!(
                !inputs.is_empty(),
                "opening reply speed probe profile '{}' produced no legal move for fixture '{}'",
                profile_name,
                fixture.id
            );
        }
        total_ms / fixtures.len().max(1) as f64
    })
}

#[derive(Debug, Clone, Copy)]
struct ProReliabilityGateMetrics {
    win_rate: f64,
    confidence: f64,
    candidate_avg_ms: f64,
}

fn pro_reliability_duel_passes(metrics: ProReliabilityGateMetrics) -> bool {
    metrics.win_rate >= SMART_PRO_RELIABILITY_WIN_RATE_MIN
        && metrics.confidence >= SMART_PRO_RELIABILITY_CONFIDENCE_MIN
        && metrics.candidate_avg_ms <= SMART_PRO_RELIABILITY_MOVE_AVG_MS_MAX
}

fn pro_reliability_gate_passes(
    vs_current_pro: ProReliabilityGateMetrics,
    vs_current_normal: ProReliabilityGateMetrics,
) -> bool {
    pro_reliability_duel_passes(vs_current_pro) && pro_reliability_duel_passes(vs_current_normal)
}

fn assert_pro_reliability_duel_passes(label: &str, metrics: ProReliabilityGateMetrics) {
    assert!(
        metrics.win_rate >= SMART_PRO_RELIABILITY_WIN_RATE_MIN,
        "{} failed: win_rate {:.4} < {:.2}",
        label,
        metrics.win_rate,
        SMART_PRO_RELIABILITY_WIN_RATE_MIN
    );
    assert!(
        metrics.confidence >= SMART_PRO_RELIABILITY_CONFIDENCE_MIN,
        "{} confidence failed: {:.4} < {:.2}",
        label,
        metrics.confidence,
        SMART_PRO_RELIABILITY_CONFIDENCE_MIN
    );
    assert!(
        metrics.candidate_avg_ms <= SMART_PRO_RELIABILITY_MOVE_AVG_MS_MAX,
        "{} move time failed: candidate_avg_ms {:.2}ms > {:.2}ms",
        label,
        metrics.candidate_avg_ms,
        SMART_PRO_RELIABILITY_MOVE_AVG_MS_MAX
    );
}

#[test]
fn duel_timing_stats_merge_and_average_track_candidate_and_baseline_turns() {
    let mut first = DuelTimingStats::default();
    first.record_candidate_turn(120.0);
    first.record_candidate_turn(180.0);
    first.record_baseline_turn(80.0);

    let mut second = DuelTimingStats::default();
    second.record_candidate_turn(60.0);
    second.record_baseline_turn(20.0);
    second.record_baseline_turn(40.0);

    first.merge(second);

    assert_eq!(first.candidate_turns, 3);
    assert_eq!(first.baseline_turns, 3);
    assert!((first.candidate_total_ms - 360.0).abs() < 0.001);
    assert!((first.baseline_total_ms - 140.0).abs() < 0.001);
    assert!((first.candidate_avg_ms() - 120.0).abs() < 0.001);
    assert!((first.baseline_avg_ms() - 46.666_666_7).abs() < 0.001);
}

#[test]
fn pro_reliability_gate_passes_only_when_both_matchups_clear_win_confidence_and_move_time() {
    let passing = ProReliabilityGateMetrics {
        win_rate: 0.90,
        confidence: 0.99,
        candidate_avg_ms: 700.0,
    };
    assert!(pro_reliability_gate_passes(passing, passing));
    assert!(!pro_reliability_gate_passes(
        ProReliabilityGateMetrics {
            win_rate: 0.89,
            ..passing
        },
        passing
    ));
    assert!(!pro_reliability_gate_passes(
        passing,
        ProReliabilityGateMetrics {
            confidence: 0.98,
            ..passing
        }
    ));
    assert!(!pro_reliability_gate_passes(
        passing,
        ProReliabilityGateMetrics {
            candidate_avg_ms: 700.01,
            ..passing
        }
    ));
}

#[test]
fn progressive_stop_rejects_dead_even_first_screen_tier() {
    let budgets = client_budgets().to_vec();
    let mut mode_stats = std::collections::HashMap::<&'static str, MatchupStats>::new();
    let flat_stats = MatchupStats {
        wins: 4,
        losses: 4,
        draws: 0,
    };
    mode_stats.insert("fast", flat_stats);
    mode_stats.insert("normal", flat_stats);

    let stop = evaluate_progressive_stop(
        budgets.as_slice(),
        &mode_stats,
        0.0,
        2,
        &ProgressiveDuelConfig {
            first_tier_signal_games_per_seed: Some(2),
            first_tier_signal_aggregate_delta_min: 0.10,
            first_tier_signal_mode_delta_min: 0.125,
            first_tier_signal_mode_floor: 0.0,
            ..ProgressiveDuelConfig::default()
        },
    );

    assert_eq!(stop, Some(ProgressiveStopReason::EarlyReject));
}

#[test]
fn progressive_stop_rejects_weak_positive_first_screen_tier() {
    let budgets = client_budgets().to_vec();
    let mut mode_stats = std::collections::HashMap::<&'static str, MatchupStats>::new();
    mode_stats.insert(
        "fast",
        MatchupStats {
            wins: 5,
            losses: 3,
            draws: 0,
        },
    );
    mode_stats.insert(
        "normal",
        MatchupStats {
            wins: 4,
            losses: 4,
            draws: 0,
        },
    );

    let stop = evaluate_progressive_stop(
        budgets.as_slice(),
        &mode_stats,
        0.0625,
        2,
        &ProgressiveDuelConfig {
            first_tier_signal_games_per_seed: Some(2),
            first_tier_signal_aggregate_delta_min: 0.10,
            first_tier_signal_mode_delta_min: 0.125,
            first_tier_signal_mode_floor: 0.0,
            ..ProgressiveDuelConfig::default()
        },
    );

    assert_eq!(stop, Some(ProgressiveStopReason::EarlyReject));
}

#[test]
fn progressive_stop_keeps_strong_first_screen_tier_alive() {
    let budgets = client_budgets().to_vec();
    let mut mode_stats = std::collections::HashMap::<&'static str, MatchupStats>::new();
    let strong_stats = MatchupStats {
        wins: 5,
        losses: 3,
        draws: 0,
    };
    mode_stats.insert("fast", strong_stats);
    mode_stats.insert("normal", strong_stats);

    let stop = evaluate_progressive_stop(
        budgets.as_slice(),
        &mode_stats,
        0.125,
        2,
        &ProgressiveDuelConfig {
            first_tier_signal_games_per_seed: Some(2),
            first_tier_signal_aggregate_delta_min: 0.10,
            first_tier_signal_mode_delta_min: 0.125,
            first_tier_signal_mode_floor: 0.0,
            ..ProgressiveDuelConfig::default()
        },
    );

    assert_eq!(stop, None);
}

#[test]
fn progressive_stop_targeted_fast_accepts_fast_win_with_normal_non_regression() {
    let budgets = client_budgets().to_vec();
    let mut mode_stats = std::collections::HashMap::<&'static str, MatchupStats>::new();
    mode_stats.insert(
        "fast",
        MatchupStats {
            wins: 5,
            losses: 3,
            draws: 0,
        },
    );
    mode_stats.insert(
        "normal",
        MatchupStats {
            wins: 4,
            losses: 4,
            draws: 0,
        },
    );

    let stop = evaluate_progressive_stop(
        budgets.as_slice(),
        &mode_stats,
        0.0625,
        2,
        &ProgressiveDuelConfig {
            first_tier_signal_games_per_seed: Some(2),
            first_tier_signal_aggregate_delta_min: 0.0,
            first_tier_signal_mode_delta_min: 0.125,
            first_tier_target_confidence_min: 0.60,
            first_tier_signal_mode_floor: 0.0,
            promotion_target_mode: Some(PromotionTargetMode::Fast),
            ..ProgressiveDuelConfig::default()
        },
    );

    assert_eq!(stop, None);
}

#[test]
fn progressive_stop_targeted_fast_rejects_fast_win_with_normal_regression() {
    let budgets = client_budgets().to_vec();
    let mut mode_stats = std::collections::HashMap::<&'static str, MatchupStats>::new();
    mode_stats.insert(
        "fast",
        MatchupStats {
            wins: 5,
            losses: 3,
            draws: 0,
        },
    );
    mode_stats.insert(
        "normal",
        MatchupStats {
            wins: 3,
            losses: 5,
            draws: 0,
        },
    );

    let stop = evaluate_progressive_stop(
        budgets.as_slice(),
        &mode_stats,
        0.0,
        2,
        &ProgressiveDuelConfig {
            first_tier_signal_games_per_seed: Some(2),
            first_tier_signal_aggregate_delta_min: 0.0,
            first_tier_signal_mode_delta_min: 0.125,
            first_tier_target_confidence_min: 0.60,
            first_tier_signal_mode_floor: 0.0,
            promotion_target_mode: Some(PromotionTargetMode::Fast),
            ..ProgressiveDuelConfig::default()
        },
    );

    assert_eq!(stop, Some(ProgressiveStopReason::EarlyReject));
}

#[test]
fn progressive_stop_targeted_fast_promotes_without_off_target_improvement() {
    let budgets = client_budgets().to_vec();
    let mut mode_stats = std::collections::HashMap::<&'static str, MatchupStats>::new();
    mode_stats.insert(
        "fast",
        MatchupStats {
            wins: 45,
            losses: 19,
            draws: 0,
        },
    );
    mode_stats.insert(
        "normal",
        MatchupStats {
            wins: 31,
            losses: 33,
            draws: 0,
        },
    );

    let stop = evaluate_progressive_stop(
        budgets.as_slice(),
        &mode_stats,
        0.09375,
        32,
        &ProgressiveDuelConfig {
            promotion_target_mode: Some(PromotionTargetMode::Fast),
            ..ProgressiveDuelConfig::default()
        },
    );

    assert_eq!(stop, Some(ProgressiveStopReason::EarlyPromote));
}

#[test]
fn progressive_stop_targeted_fast_does_not_promote_off_target_winner() {
    let budgets = client_budgets().to_vec();
    let mut mode_stats = std::collections::HashMap::<&'static str, MatchupStats>::new();
    mode_stats.insert(
        "fast",
        MatchupStats {
            wins: 32,
            losses: 32,
            draws: 0,
        },
    );
    mode_stats.insert(
        "normal",
        MatchupStats {
            wins: 45,
            losses: 19,
            draws: 0,
        },
    );

    let stop = evaluate_progressive_stop(
        budgets.as_slice(),
        &mode_stats,
        0.1016,
        32,
        &ProgressiveDuelConfig {
            promotion_target_mode: Some(PromotionTargetMode::Fast),
            ..ProgressiveDuelConfig::default()
        },
    );

    assert_eq!(stop, Some(ProgressiveStopReason::MaxGamesReached));
}

fn triage_surface_from_env() -> TriageSurface {
    let value = env::var("SMART_TRIAGE_SURFACE").unwrap_or_else(|_| {
        panic!(
            "SMART_TRIAGE_SURFACE is required (expected one of: opening_reply, primary_pro, reply_risk, supermana, opponent_mana, normal_fast_gap, normal_release_seed_gap, normal_tiebreak, spirit_setup, drainer_safety, cache_reuse)"
        )
    });
    TriageSurface::parse(value.as_str()).unwrap_or_else(|| {
        panic!(
            "unknown SMART_TRIAGE_SURFACE='{}' (expected one of: opening_reply, primary_pro, reply_risk, supermana, opponent_mana, normal_fast_gap, normal_release_seed_gap, normal_tiebreak, spirit_setup, drainer_safety, cache_reuse)",
            value
        )
    })
}

fn generic_signal_triage_passes(target_changed: usize) -> bool {
    target_changed > 0
}

fn pro_signal_triage_passes(target_changed: usize, off_target_changed: usize) -> bool {
    target_changed > 0 && off_target_changed <= 1
}

fn triage_game_with_items(
    items: Vec<(Location, Item)>,
    active_color: Color,
    turn_number: i32,
) -> MonsGame {
    let mut game = MonsGame::new(false);
    game.board = Board::new_with_items(
        items
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>(),
    );
    game.active_color = active_color;
    game.turn_number = turn_number;
    game.actions_used_count = 0;
    game.mana_moves_count = 0;
    game.mons_moves_count = 0;
    game.white_score = 0;
    game.black_score = 0;
    game.white_potions_count = 0;
    game.black_potions_count = 0;
    game
}

fn triage_root_evaluation(candidate: &ScoredRootMove, score: i32) -> RootEvaluation {
    RootEvaluation {
        root_rank: 0,
        score,
        efficiency: candidate.efficiency,
        inputs: candidate.inputs.clone(),
        game: candidate.game.clone(),
        wins_immediately: candidate.wins_immediately,
        attacks_opponent_drainer: candidate.attacks_opponent_drainer,
        own_drainer_vulnerable: candidate.own_drainer_vulnerable,
        own_drainer_walk_vulnerable: candidate.own_drainer_walk_vulnerable,
        spirit_development: candidate.spirit_development,
        keeps_awake_spirit_on_base: candidate.keeps_awake_spirit_on_base,
        mana_handoff_to_opponent: candidate.mana_handoff_to_opponent,
        has_roundtrip: candidate.has_roundtrip,
        scores_supermana_this_turn: candidate.scores_supermana_this_turn,
        scores_opponent_mana_this_turn: candidate.scores_opponent_mana_this_turn,
        safe_supermana_pickup_now: candidate.safe_supermana_pickup_now,
        safe_opponent_mana_pickup_now: candidate.safe_opponent_mana_pickup_now,
        safe_supermana_progress_steps: candidate.safe_supermana_progress_steps,
        safe_opponent_mana_progress_steps: candidate.safe_opponent_mana_progress_steps,
        score_path_best_steps: candidate.score_path_best_steps,
        same_turn_score_window_value: candidate.same_turn_score_window_value,
        spirit_setup_gain: candidate.spirit_setup_gain,
        spirit_same_turn_score_setup_now: candidate.spirit_same_turn_score_setup_now,
        spirit_own_mana_setup_now: candidate.spirit_own_mana_setup_now,
        supermana_progress: candidate.supermana_progress,
        opponent_mana_progress: candidate.opponent_mana_progress,
        interview_soft_priority: candidate.interview_soft_priority,
        classes: candidate.classes,
    }
}

fn calibration_runtime_config(
    profile_name: &str,
    game: &MonsGame,
    mode: SmartAutomovePreference,
) -> SmartSearchConfig {
    let base = SearchBudget::from_preference(mode).runtime_config_for_game(game);
    profile_runtime_config_for_name(profile_name, game, base).unwrap_or_else(|| {
        panic!(
            "profile '{}' does not expose a runtime config",
            profile_name
        )
    })
}

fn probe_config_with_env_overrides(mut config: SmartSearchConfig) -> SmartSearchConfig {
    let env_i32 = |name: &str| {
        std::env::var(name)
            .ok()
            .and_then(|value| value.trim().parse::<i32>().ok())
    };
    let env_bool_value = |name: &str| env_bool(name);
    if env_bool("SMART_PROBE_FORCE_ENGINE_DISABLED").unwrap_or(false) {
        config.enable_turn_engine = false;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_TURN_PLANNER") {
        config.enable_turn_opportunity_planner = value;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_TURN_PLANNER_ROOT_INJECTION") {
        config.enable_turn_planner_intent_root_injection = value;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_SELECTED_FOLLOWUP_PROJECTION") {
        config.enable_turn_engine_selected_followup_projection = value;
    }
    if let Some(nodes) = env_usize("SMART_PROBE_FORCE_MAX_NODES") {
        config.max_visited_nodes = nodes.max(1);
    }
    if let Some(limit) = env_usize("SMART_PROBE_FORCE_ROOT_LIMIT") {
        config.root_branch_limit = limit.max(1);
        config.root_enum_limit = config.root_enum_limit.max(config.root_branch_limit);
    }
    if let Some(limit) = env_usize("SMART_PROBE_FORCE_ENUM_LIMIT") {
        config.root_enum_limit = limit.max(config.root_branch_limit.max(1));
    }
    if let Some(limit) = env_usize("SMART_PROBE_FORCE_NODE_LIMIT") {
        config.node_branch_limit = limit.max(1);
        config.node_enum_limit = config.node_enum_limit.max(config.node_branch_limit);
    }
    if let Some(limit) = env_usize("SMART_PROBE_FORCE_NODE_ENUM_LIMIT") {
        config.node_enum_limit = limit.max(config.node_branch_limit.max(1));
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_TWO_PASS_ROOT_ALLOCATION") {
        config.enable_two_pass_root_allocation = value;
    }
    if let Some(value) = env_usize("SMART_PROBE_FORCE_FOCUS_K") {
        config.root_focus_k = value.max(1);
    }
    if let Some(value) = env_i32("SMART_PROBE_FORCE_FOCUS_SHARE_BP") {
        config.root_focus_budget_share_bp = value.max(0);
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_EVENT_ORDERING") {
        config.enable_event_ordering_bonus = value;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_SELECTIVE_EXTENSIONS") {
        config.enable_selective_extensions = value;
    }
    if let Some(value) = env_i32("SMART_PROBE_FORCE_SELECTIVE_EXTENSION_SHARE_BP") {
        config.selective_extension_node_share_bp = value.max(0);
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_QUIET_REDUCTIONS") {
        config.enable_quiet_reductions = value;
    }
    if let Some(value) = env_usize("SMART_PROBE_FORCE_QUIET_REDUCTION_DEPTH") {
        config.quiet_reduction_depth_threshold = value.max(1);
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_VOLATILITY_FOCUS") {
        config.enable_two_pass_volatility_focus = value;
    }
    if let Some(limit) = env_usize("SMART_PROBE_FORCE_SHORTLIST_MAX") {
        config.root_reply_risk_shortlist_max = limit.max(1);
    }
    if let Some(limit) = env_usize("SMART_PROBE_FORCE_REPLY_LIMIT") {
        config.root_reply_risk_reply_limit = limit.max(1);
    }
    if let Some(share_bp) = env_i32("SMART_PROBE_FORCE_REPLY_SHARE_BP") {
        config.root_reply_risk_node_share_bp = share_bp.max(0);
    }
    if let Some(margin) = env_i32("SMART_PROBE_FORCE_REPLY_MARGIN") {
        config.root_reply_risk_score_margin = margin;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_CLEAN_REPLY") {
        config.prefer_clean_reply_risk_roots = value;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_HARD_SPIRIT_DEPLOY") {
        config.enable_interview_hard_spirit_deploy = value;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_SOFT_ROOT_PRIORS") {
        config.enable_interview_soft_root_priors = value;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_DETERMINISTIC_TIEBREAK") {
        config.enable_interview_deterministic_tiebreak = value;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_NORMAL_SAFETY_RERANK") {
        config.enable_normal_root_safety_rerank = value;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_NORMAL_SAFETY_DEEP_FLOOR") {
        config.enable_normal_root_safety_deep_floor = value;
    }
    if let Some(value) = env_i32("SMART_PROBE_FORCE_DRAINER_MARGIN") {
        config.root_drainer_safety_score_margin = value.max(0);
    }
    if let Some(value) = env_i32("SMART_PROBE_FORCE_EFFICIENCY_MARGIN") {
        config.root_efficiency_score_margin = value.max(0);
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_QUIESCENCE") {
        config.enable_quiescence_search = value;
    }
    if let Some(value) = env_usize("SMART_PROBE_FORCE_QUIESCENCE_BUDGET") {
        config.quiescence_node_budget = value;
    }
    if let Some(value) = env_bool_value("SMART_PROBE_FORCE_QUIESCENCE_TACTICAL_ONLY") {
        config.enable_quiescence_tactical_children_only = value;
    }
    if let Some(value) = env_usize("SMART_PROBE_FORCE_QUIESCENCE_ENUM_LIMIT") {
        config.quiescence_tactical_enum_limit = value;
    }
    config
}

fn calibration_turn_engine_config(config: SmartSearchConfig) -> TurnEngineConfig {
    TurnEngineConfig {
        mode: config.turn_engine_mode,
        own_seed_cap: config.turn_engine_seed_cap.max(1),
        own_beam: config.turn_engine_beam_width.max(1),
        per_node_family_cap: config.turn_engine_per_node_family_cap.max(1),
        step_cap: config.turn_engine_step_cap.max(1),
        opponent_seed_cap: config.turn_engine_opponent_seed_cap.max(1),
        opponent_beam: config.turn_engine_opponent_beam_width.max(1),
        reply_seed_cap: config.turn_engine_reply_seed_cap.max(1),
        reply_beam: config.turn_engine_reply_beam_width.max(1),
        expansion_cap: config.turn_engine_expansion_cap.max(1),
        enable_spirit_family: config.turn_engine_enable_spirit_family,
        scoring_weights: config.scoring_weights,
        allow_exact_static_evaluation: config.enable_static_exact_evaluation,
        enable_lazy_oracle_score_window_projection: config
            .enable_turn_engine_lazy_oracle_score_window_projection,
    }
}

fn reply_risk_calibration_probe(profile_name: &str) -> i32 {
    let white_drainer = Mon::new(MonKind::Drainer, Color::White, 0);
    let black_drainer = Mon::new(MonKind::Drainer, Color::Black, 0);
    let game = triage_game_with_items(
        vec![
            (Location::new(4, 0), Item::Mon { mon: white_drainer }),
            (Location::new(0, 5), Item::Mon { mon: black_drainer }),
        ],
        Color::White,
        2,
    );
    let config = calibration_runtime_config(profile_name, &game, SmartAutomovePreference::Fast);
    let events = vec![
        Event::MonMove {
            item: Item::Mon { mon: white_drainer },
            from: Location::new(4, 0),
            to: Location::new(5, 0),
        },
        Event::MonMove {
            item: Item::Mon { mon: white_drainer },
            from: Location::new(5, 0),
            to: Location::new(4, 0),
        },
    ];
    MonsGameModel::move_efficiency_delta(
        &game,
        &game,
        Color::White,
        events.as_slice(),
        true,
        true,
        false,
        false,
        false,
        config.root_backtrack_penalty,
        config.root_mana_handoff_penalty,
    )
}

fn opponent_mana_calibration_probe(profile_name: &str) -> usize {
    let mut game = triage_game_with_items(
        vec![
            (
                Location::new(4, 0),
                Item::Mon {
                    mon: Mon::new(MonKind::Spirit, Color::White, 0),
                },
            ),
            (
                Location::new(7, 0),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                },
            ),
            (
                Location::new(5, 2),
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
    );
    game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

    let config = calibration_runtime_config(profile_name, &game, SmartAutomovePreference::Normal);
    let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
        &game,
        Color::White,
        config.enable_enhanced_drainer_vulnerability,
    );
    let mut progress = MonsGameModel::build_scored_root_move(
        &game,
        Color::White,
        config,
        own_drainer_vulnerable_before,
        &[
            Input::Location(Location::new(4, 0)),
            Input::Location(Location::new(5, 2)),
            Input::Location(Location::new(6, 1)),
        ],
    )
    .expect("spirit opponent mana handoff inputs should build a scored root");
    progress.opponent_mana_progress = true;
    progress.safe_opponent_mana_progress_steps = 1;
    progress.mana_handoff_to_opponent = false;
    progress.has_roundtrip = false;

    let mut risky = progress.clone();
    risky.inputs = vec![Input::Location(Location::new(0, 0))];
    risky.opponent_mana_progress = false;
    risky.safe_opponent_mana_progress_steps = 6;
    risky.mana_handoff_to_opponent = true;
    risky.has_roundtrip = true;
    risky.interview_soft_priority = 0;

    MonsGameModel::pick_root_move_with_reply_risk_guard(
        &game,
        &[
            triage_root_evaluation(&risky, 200),
            triage_root_evaluation(&progress, 40),
        ],
        &[0, 1],
        Color::White,
        config,
    )
    .expect("reply-risk calibration probe should pick one of the synthetic roots")
}

fn supermana_calibration_probe(profile_name: &str) -> bool {
    let game = triage_game_with_items(
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
    );
    let config = calibration_runtime_config(profile_name, &game, SmartAutomovePreference::Normal);
    let (_, events) = MonsGameModel::apply_inputs_for_search_with_events(
        &game,
        &[
            Input::Location(Location::new(6, 5)),
            Input::Location(Location::new(5, 5)),
        ],
    )
    .expect("shortening supermana path inputs should be legal");
    MonsGameModel::should_use_root_exact_summary_for_transition(events.as_slice(), config)
}

#[test]
fn signal_triage_rejects_no_op_candidate() {
    assert!(!generic_signal_triage_passes(0));
}

#[test]
fn signal_triage_rejects_wrong_surface_candidate() {
    let target_changed = 0;
    let off_target_changed = 2;
    assert_eq!(off_target_changed, 2);
    assert!(!generic_signal_triage_passes(target_changed));
}

#[test]
fn signal_triage_accepts_target_surface_candidate() {
    assert!(generic_signal_triage_passes(1));
}

#[test]
fn pro_signal_triage_accepts_opening_reply_with_stable_primary() {
    assert!(pro_signal_triage_passes(2, 1));
}

#[test]
fn pro_signal_triage_accepts_primary_pro_with_stable_opening_reply() {
    assert!(pro_signal_triage_passes(1, 0));
}

#[test]
fn triage_calibration_probe_detects_reply_risk_profile_delta() {
    let candidate =
        reply_risk_calibration_probe("runtime_pre_fast_root_quality_v1_normal_conversion_v3");
    let baseline = reply_risk_calibration_probe("runtime_release_safe_pre_exact");
    assert!(candidate > baseline);
}

#[test]
fn triage_calibration_probe_detects_opponent_mana_profile_delta() {
    let candidate =
        opponent_mana_calibration_probe("runtime_pre_fast_root_quality_v1_normal_conversion_v3");
    let baseline = opponent_mana_calibration_probe("runtime_release_safe_pre_exact");
    assert_ne!(candidate, baseline);
}

#[test]
fn triage_calibration_probe_detects_supermana_profile_delta() {
    let candidate = supermana_calibration_probe("runtime_eff_exact_lite_v1");
    let baseline = supermana_calibration_probe("runtime_release_safe_pre_exact");
    assert!(candidate);
    assert!(!baseline);
}

fn maybe_run_runtime_preflight_checks(
    skip_runtime_preflight: bool,
    run_stage1: impl FnOnce(),
    run_exact: impl FnOnce(),
) {
    if skip_runtime_preflight {
        return;
    }
    run_stage1();
    run_exact();
}

#[test]
fn runtime_preflight_checks_run_when_not_skipped() {
    let stage1_calls = std::cell::Cell::new(0);
    let exact_calls = std::cell::Cell::new(0);

    maybe_run_runtime_preflight_checks(
        false,
        || stage1_calls.set(stage1_calls.get() + 1),
        || exact_calls.set(exact_calls.get() + 1),
    );

    assert_eq!(stage1_calls.get(), 1);
    assert_eq!(exact_calls.get(), 1);
}

#[test]
fn runtime_preflight_checks_are_skipped_when_requested() {
    let stage1_calls = std::cell::Cell::new(0);
    let exact_calls = std::cell::Cell::new(0);

    maybe_run_runtime_preflight_checks(
        true,
        || stage1_calls.set(stage1_calls.get() + 1),
        || exact_calls.set(exact_calls.get() + 1),
    );

    assert_eq!(stage1_calls.get(), 0);
    assert_eq!(exact_calls.get(), 0);
}

#[test]
fn reply_risk_triage_detects_shortlist_signal_when_selected_root_is_unchanged() {
    let risky_root = TriageRootDigestEntry {
        input_fen: "l4,0;l5,2;l6,1".to_string(),
        heuristic: 100,
        efficiency: 12,
        wins_immediately: false,
        attacks_opponent_drainer: false,
        own_drainer_vulnerable: false,
        own_drainer_walk_vulnerable: false,
        spirit_development: false,
        mana_handoff_to_opponent: true,
        has_roundtrip: true,
        scores_supermana_this_turn: false,
        scores_opponent_mana_this_turn: false,
        safe_supermana_pickup_now: false,
        safe_opponent_mana_pickup_now: false,
        safe_supermana_progress_steps: 15,
        safe_opponent_mana_progress_steps: 15,
        score_path_best_steps: 20,
        same_turn_score_window_value: 0,
        spirit_setup_gain: 0,
        spirit_same_turn_score_setup_now: false,
        spirit_own_mana_setup_now: false,
        supermana_progress: false,
        opponent_mana_progress: false,
        interview_soft_priority: 0,
    };
    let mut clean_root = risky_root.clone();
    clean_root.input_fen = "l7,0;l6,1".to_string();
    clean_root.mana_handoff_to_opponent = false;
    clean_root.has_roundtrip = false;

    let baseline = TriageSignalSnapshot {
        selected_rank: 0,
        selected_root: risky_root.clone(),
        top_root_count: 2,
        top_roots: vec![risky_root.clone(), clean_root.clone()],
        reply_risk_shortlist: Some(TriageReplyRiskShortlistDigest {
            preferred_root_input_fen: risky_root.input_fen.clone(),
            shortlisted_roots: vec![
                TriageReplyRiskShortlistEntry {
                    input_fen: risky_root.input_fen.clone(),
                    mana_handoff_to_opponent: true,
                    has_roundtrip: true,
                    own_drainer_vulnerable: false,
                    own_drainer_walk_vulnerable: false,
                    allows_immediate_opponent_win: false,
                    opponent_reaches_match_point: false,
                    worst_reply_score: 120,
                },
                TriageReplyRiskShortlistEntry {
                    input_fen: clean_root.input_fen.clone(),
                    mana_handoff_to_opponent: false,
                    has_roundtrip: false,
                    own_drainer_vulnerable: false,
                    own_drainer_walk_vulnerable: false,
                    allows_immediate_opponent_win: false,
                    opponent_reaches_match_point: false,
                    worst_reply_score: 120,
                },
            ],
        }),
    };
    let candidate = TriageSignalSnapshot {
        selected_rank: 0,
        selected_root: risky_root.clone(),
        top_root_count: 2,
        top_roots: vec![risky_root, clean_root.clone()],
        reply_risk_shortlist: Some(TriageReplyRiskShortlistDigest {
            preferred_root_input_fen: clean_root.input_fen.clone(),
            shortlisted_roots: vec![
                TriageReplyRiskShortlistEntry {
                    input_fen: "l4,0;l5,2;l6,1".to_string(),
                    mana_handoff_to_opponent: true,
                    has_roundtrip: true,
                    own_drainer_vulnerable: false,
                    own_drainer_walk_vulnerable: false,
                    allows_immediate_opponent_win: false,
                    opponent_reaches_match_point: false,
                    worst_reply_score: 120,
                },
                TriageReplyRiskShortlistEntry {
                    input_fen: clean_root.input_fen,
                    mana_handoff_to_opponent: false,
                    has_roundtrip: false,
                    own_drainer_vulnerable: false,
                    own_drainer_walk_vulnerable: false,
                    allows_immediate_opponent_win: false,
                    opponent_reaches_match_point: false,
                    worst_reply_score: 120,
                },
            ],
        }),
    };

    assert!(triage_surface_signal_changed(
        TriageSurface::ReplyRisk,
        &candidate,
        &baseline,
    ));
}

fn with_env_override<T>(name: &str, value: &str, f: impl FnOnce() -> T) -> T {
    let previous = env::var(name).ok();
    env::set_var(name, value);
    let result = f();
    if let Some(previous) = previous {
        env::set_var(name, previous);
    } else {
        env::remove_var(name);
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TriageRootDigestEntry {
    input_fen: String,
    heuristic: i32,
    efficiency: i32,
    wins_immediately: bool,
    attacks_opponent_drainer: bool,
    own_drainer_vulnerable: bool,
    own_drainer_walk_vulnerable: bool,
    spirit_development: bool,
    mana_handoff_to_opponent: bool,
    has_roundtrip: bool,
    scores_supermana_this_turn: bool,
    scores_opponent_mana_this_turn: bool,
    safe_supermana_pickup_now: bool,
    safe_opponent_mana_pickup_now: bool,
    safe_supermana_progress_steps: i32,
    safe_opponent_mana_progress_steps: i32,
    score_path_best_steps: i32,
    same_turn_score_window_value: i32,
    spirit_setup_gain: i32,
    spirit_same_turn_score_setup_now: bool,
    spirit_own_mana_setup_now: bool,
    supermana_progress: bool,
    opponent_mana_progress: bool,
    interview_soft_priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TriageReplyRiskShortlistEntry {
    input_fen: String,
    mana_handoff_to_opponent: bool,
    has_roundtrip: bool,
    own_drainer_vulnerable: bool,
    own_drainer_walk_vulnerable: bool,
    allows_immediate_opponent_win: bool,
    opponent_reaches_match_point: bool,
    worst_reply_score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TriageReplyRiskShortlistDigest {
    preferred_root_input_fen: String,
    shortlisted_roots: Vec<TriageReplyRiskShortlistEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TriageSignalSnapshot {
    selected_rank: usize,
    selected_root: TriageRootDigestEntry,
    top_root_count: usize,
    top_roots: Vec<TriageRootDigestEntry>,
    reply_risk_shortlist: Option<TriageReplyRiskShortlistDigest>,
}

const TRIAGE_TOP_ROOT_DIGEST_SIZE: usize = 3;
const TRIAGE_REPLY_RISK_SHORTLIST_DIGEST_SIZE: usize = 6;
const TRIAGE_HEURISTIC_DELTA_MIN: i32 = 20;
const TRIAGE_EFFICIENCY_DELTA_MIN: i32 = 8;
const TRIAGE_INTERVIEW_PRIORITY_DELTA_MIN: i32 = 16;
const TRIAGE_REPLY_RISK_WORST_REPLY_DELTA_MIN: i32 = 20;

fn triage_root_digest_entry(root: &ScoredRootMove) -> TriageRootDigestEntry {
    TriageRootDigestEntry {
        input_fen: Input::fen_from_array(&root.inputs),
        heuristic: root.heuristic,
        efficiency: root.efficiency,
        wins_immediately: root.wins_immediately,
        attacks_opponent_drainer: root.attacks_opponent_drainer,
        own_drainer_vulnerable: root.own_drainer_vulnerable,
        own_drainer_walk_vulnerable: root.own_drainer_walk_vulnerable,
        spirit_development: root.spirit_development,
        mana_handoff_to_opponent: root.mana_handoff_to_opponent,
        has_roundtrip: root.has_roundtrip,
        scores_supermana_this_turn: root.scores_supermana_this_turn,
        scores_opponent_mana_this_turn: root.scores_opponent_mana_this_turn,
        safe_supermana_pickup_now: root.safe_supermana_pickup_now,
        safe_opponent_mana_pickup_now: root.safe_opponent_mana_pickup_now,
        safe_supermana_progress_steps: root.safe_supermana_progress_steps,
        safe_opponent_mana_progress_steps: root.safe_opponent_mana_progress_steps,
        score_path_best_steps: root.score_path_best_steps,
        same_turn_score_window_value: root.same_turn_score_window_value,
        spirit_setup_gain: root.spirit_setup_gain,
        spirit_same_turn_score_setup_now: root.spirit_same_turn_score_setup_now,
        spirit_own_mana_setup_now: root.spirit_own_mana_setup_now,
        supermana_progress: root.supermana_progress,
        opponent_mana_progress: root.opponent_mana_progress,
        interview_soft_priority: root.interview_soft_priority,
    }
}

fn triage_meaningful_i32_delta(candidate: i32, baseline: i32, min_delta: i32) -> bool {
    candidate.abs_diff(baseline) >= min_delta.max(0) as u32
}

fn triage_reply_risk_shortlist_entry(
    root: &ScoredRootMove,
    snapshot: RootReplyRiskSnapshot,
) -> TriageReplyRiskShortlistEntry {
    TriageReplyRiskShortlistEntry {
        input_fen: Input::fen_from_array(&root.inputs),
        mana_handoff_to_opponent: root.mana_handoff_to_opponent,
        has_roundtrip: root.has_roundtrip,
        own_drainer_vulnerable: root.own_drainer_vulnerable,
        own_drainer_walk_vulnerable: root.own_drainer_walk_vulnerable,
        allows_immediate_opponent_win: snapshot.allows_immediate_opponent_win,
        opponent_reaches_match_point: snapshot.opponent_reaches_match_point,
        worst_reply_score: snapshot.worst_reply_score,
    }
}

fn triage_reply_risk_shortlist_digest(
    game: &MonsGame,
    ranked_roots: &[ScoredRootMove],
    config: SmartSearchConfig,
) -> Option<TriageReplyRiskShortlistDigest> {
    if !config.enable_root_reply_risk_guard || ranked_roots.is_empty() {
        return None;
    }

    let shortlist_len = config
        .root_reply_risk_shortlist_max
        .max(1)
        .min(ranked_roots.len())
        .min(TRIAGE_REPLY_RISK_SHORTLIST_DIGEST_SIZE);
    let shortlisted_roots = ranked_roots.iter().take(shortlist_len).collect::<Vec<_>>();
    let root_node_budget = ((config.max_visited_nodes
        * config.root_reply_risk_node_share_bp.max(0) as usize)
        / 10_000)
        .max(shortlisted_roots.len())
        .max(1);
    let per_root_reply_limit = (root_node_budget / shortlisted_roots.len().max(1))
        .max(1)
        .min(config.root_reply_risk_reply_limit.max(1));
    let digest_entries = shortlisted_roots
        .iter()
        .map(|root| {
            let snapshot = MonsGameModel::root_reply_risk_snapshot(
                &root.game,
                game.active_color,
                config,
                per_root_reply_limit,
            );
            triage_reply_risk_shortlist_entry(root, snapshot)
        })
        .collect::<Vec<_>>();
    let shortlisted_evaluations = shortlisted_roots
        .iter()
        .map(|root| triage_root_evaluation(root, 0))
        .collect::<Vec<_>>();
    let shortlist_indices = (0..shortlisted_evaluations.len()).collect::<Vec<_>>();
    let preferred_index = MonsGameModel::pick_root_move_with_reply_risk_guard(
        game,
        shortlisted_evaluations.as_slice(),
        shortlist_indices.as_slice(),
        game.active_color,
        config,
    )
    .unwrap_or(0);

    Some(TriageReplyRiskShortlistDigest {
        preferred_root_input_fen: digest_entries[preferred_index].input_fen.clone(),
        shortlisted_roots: digest_entries,
    })
}

fn triage_reply_risk_shortlist_digest_changed(
    candidate: Option<&TriageReplyRiskShortlistDigest>,
    baseline: Option<&TriageReplyRiskShortlistDigest>,
) -> bool {
    match (candidate, baseline) {
        (Some(candidate), Some(baseline)) => {
            candidate.preferred_root_input_fen != baseline.preferred_root_input_fen
                || candidate.shortlisted_roots.len() != baseline.shortlisted_roots.len()
                || candidate
                    .shortlisted_roots
                    .iter()
                    .zip(baseline.shortlisted_roots.iter())
                    .any(|(candidate_root, baseline_root)| {
                        candidate_root.input_fen != baseline_root.input_fen
                            || candidate_root.mana_handoff_to_opponent
                                != baseline_root.mana_handoff_to_opponent
                            || candidate_root.has_roundtrip != baseline_root.has_roundtrip
                            || candidate_root.own_drainer_vulnerable
                                != baseline_root.own_drainer_vulnerable
                            || candidate_root.own_drainer_walk_vulnerable
                                != baseline_root.own_drainer_walk_vulnerable
                            || candidate_root.allows_immediate_opponent_win
                                != baseline_root.allows_immediate_opponent_win
                            || candidate_root.opponent_reaches_match_point
                                != baseline_root.opponent_reaches_match_point
                            || triage_meaningful_i32_delta(
                                candidate_root.worst_reply_score,
                                baseline_root.worst_reply_score,
                                TRIAGE_REPLY_RISK_WORST_REPLY_DELTA_MIN,
                            )
                    })
        }
        (None, None) => false,
        _ => true,
    }
}

fn triage_fixture_snapshot(
    surface: TriageSurface,
    profile_name: &str,
    selector: AutomoveSelector,
    fixture: &TriageFixture,
) -> TriageSignalSnapshot {
    with_env_override(
        "SMART_USE_WHITE_OPENING_BOOK",
        if fixture.opening_book_driven {
            "true"
        } else {
            "false"
        },
        || {
            let base_config = fixture
                .config_tweak
                .map(|tweak| {
                    tweak(
                        SearchBudget::from_preference(fixture.mode)
                            .runtime_config_for_game(&fixture.game),
                    )
                })
                .unwrap_or_else(|| {
                    SearchBudget::from_preference(fixture.mode)
                        .runtime_config_for_game(&fixture.game)
                });
            let resolved_config =
                profile_runtime_config_for_name(profile_name, &fixture.game, base_config)
                    .unwrap_or(base_config);
            let inputs = select_inputs_with_runtime_fallback(selector, &fixture.game, base_config);
            assert!(
                !inputs.is_empty(),
                "triage fixture '{}' produced no legal move for mode {}",
                fixture.id,
                fixture.mode.as_api_value()
            );
            MonsGameModel::apply_inputs_for_search_with_events(&fixture.game, &inputs)
                .unwrap_or_else(|| {
                    panic!(
                        "triage fixture '{}' selected illegal move in mode {}",
                        fixture.id,
                        fixture.mode.as_api_value()
                    )
                });
            let input_fen = Input::fen_from_array(&inputs);
            let ranked_roots = MonsGameModel::ranked_root_moves(
                &fixture.game,
                fixture.game.active_color,
                resolved_config,
            );
            let selected_rank = ranked_roots
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == input_fen);
            let selected_root = if let Some(rank) = selected_rank {
                triage_root_digest_entry(&ranked_roots[rank])
            } else {
                let own_drainer_vulnerable_before =
                    MonsGameModel::is_own_drainer_vulnerable_next_turn(
                        &fixture.game,
                        fixture.game.active_color,
                        resolved_config.enable_enhanced_drainer_vulnerability,
                    );
                let selected = MonsGameModel::build_scored_root_move(
                    &fixture.game,
                    fixture.game.active_color,
                    resolved_config,
                    own_drainer_vulnerable_before,
                    &inputs,
                )
                .unwrap_or_else(|| {
                    panic!(
                        "triage fixture '{}' selected move missing from ranked roots and could not be materialized in mode {}",
                        fixture.id,
                        fixture.mode.as_api_value()
                    )
                });
                triage_root_digest_entry(&selected)
            };
            TriageSignalSnapshot {
                selected_rank: selected_rank.unwrap_or(ranked_roots.len()),
                selected_root,
                top_root_count: ranked_roots.len(),
                top_roots: ranked_roots
                    .iter()
                    .take(TRIAGE_TOP_ROOT_DIGEST_SIZE)
                    .map(triage_root_digest_entry)
                    .collect(),
                reply_risk_shortlist: match surface {
                    TriageSurface::ReplyRisk => triage_reply_risk_shortlist_digest(
                        &fixture.game,
                        ranked_roots.as_slice(),
                        resolved_config,
                    ),
                    _ => None,
                },
            }
        },
    )
}

fn triage_root_digest_changed(
    surface: TriageSurface,
    candidate: &TriageRootDigestEntry,
    baseline: &TriageRootDigestEntry,
) -> bool {
    if candidate.input_fen != baseline.input_fen {
        return true;
    }

    match surface {
        TriageSurface::ReplyRisk => {
            candidate.mana_handoff_to_opponent != baseline.mana_handoff_to_opponent
                || candidate.has_roundtrip != baseline.has_roundtrip
                || candidate.own_drainer_vulnerable != baseline.own_drainer_vulnerable
                || candidate.own_drainer_walk_vulnerable != baseline.own_drainer_walk_vulnerable
                || triage_meaningful_i32_delta(
                    candidate.heuristic,
                    baseline.heuristic,
                    TRIAGE_HEURISTIC_DELTA_MIN,
                )
                || triage_meaningful_i32_delta(
                    candidate.efficiency,
                    baseline.efficiency,
                    TRIAGE_EFFICIENCY_DELTA_MIN,
                )
                || triage_meaningful_i32_delta(
                    candidate.interview_soft_priority,
                    baseline.interview_soft_priority,
                    TRIAGE_INTERVIEW_PRIORITY_DELTA_MIN,
                )
        }
        TriageSurface::Supermana => {
            candidate.supermana_progress != baseline.supermana_progress
                || candidate.scores_supermana_this_turn != baseline.scores_supermana_this_turn
                || candidate.safe_supermana_pickup_now != baseline.safe_supermana_pickup_now
                || candidate.safe_supermana_progress_steps != baseline.safe_supermana_progress_steps
                || candidate.spirit_own_mana_setup_now != baseline.spirit_own_mana_setup_now
                || candidate.same_turn_score_window_value != baseline.same_turn_score_window_value
                || triage_meaningful_i32_delta(
                    candidate.heuristic,
                    baseline.heuristic,
                    TRIAGE_HEURISTIC_DELTA_MIN,
                )
                || triage_meaningful_i32_delta(
                    candidate.interview_soft_priority,
                    baseline.interview_soft_priority,
                    TRIAGE_INTERVIEW_PRIORITY_DELTA_MIN,
                )
        }
        TriageSurface::OpponentMana => {
            candidate.opponent_mana_progress != baseline.opponent_mana_progress
                || candidate.scores_opponent_mana_this_turn
                    != baseline.scores_opponent_mana_this_turn
                || candidate.safe_opponent_mana_pickup_now != baseline.safe_opponent_mana_pickup_now
                || candidate.safe_opponent_mana_progress_steps
                    != baseline.safe_opponent_mana_progress_steps
                || candidate.spirit_own_mana_setup_now != baseline.spirit_own_mana_setup_now
                || triage_meaningful_i32_delta(
                    candidate.heuristic,
                    baseline.heuristic,
                    TRIAGE_HEURISTIC_DELTA_MIN,
                )
                || triage_meaningful_i32_delta(
                    candidate.interview_soft_priority,
                    baseline.interview_soft_priority,
                    TRIAGE_INTERVIEW_PRIORITY_DELTA_MIN,
                )
        }
        TriageSurface::SpiritSetup => {
            candidate.spirit_development != baseline.spirit_development
                || candidate.spirit_own_mana_setup_now != baseline.spirit_own_mana_setup_now
                || candidate.spirit_same_turn_score_setup_now
                    != baseline.spirit_same_turn_score_setup_now
                || candidate.score_path_best_steps != baseline.score_path_best_steps
                || candidate.same_turn_score_window_value != baseline.same_turn_score_window_value
                || triage_meaningful_i32_delta(
                    candidate.heuristic,
                    baseline.heuristic,
                    TRIAGE_HEURISTIC_DELTA_MIN,
                )
        }
        TriageSurface::DrainerSafety => {
            candidate.own_drainer_vulnerable != baseline.own_drainer_vulnerable
                || candidate.own_drainer_walk_vulnerable != baseline.own_drainer_walk_vulnerable
                || candidate.mana_handoff_to_opponent != baseline.mana_handoff_to_opponent
                || candidate.has_roundtrip != baseline.has_roundtrip
                || candidate.attacks_opponent_drainer != baseline.attacks_opponent_drainer
        }
        TriageSurface::NormalFastGap
        | TriageSurface::NormalReleaseSeedGap
        | TriageSurface::NormalTiebreak
        | TriageSurface::OpeningReply
        | TriageSurface::PrimaryPro => false,
        TriageSurface::CacheReuse => false,
    }
}

fn triage_top_root_digest_changed(
    surface: TriageSurface,
    candidate: &[TriageRootDigestEntry],
    baseline: &[TriageRootDigestEntry],
) -> bool {
    candidate.len() != baseline.len()
        || candidate
            .iter()
            .zip(baseline.iter())
            .any(|(candidate_root, baseline_root)| {
                triage_root_digest_changed(surface, candidate_root, baseline_root)
            })
}

fn triage_surface_signal_changed(
    surface: TriageSurface,
    candidate: &TriageSignalSnapshot,
    baseline: &TriageSignalSnapshot,
) -> bool {
    match surface {
        TriageSurface::ReplyRisk => {
            candidate.selected_rank != baseline.selected_rank
                || candidate.top_root_count != baseline.top_root_count
                || triage_root_digest_changed(
                    surface,
                    &candidate.selected_root,
                    &baseline.selected_root,
                )
                || triage_top_root_digest_changed(
                    surface,
                    candidate.top_roots.as_slice(),
                    baseline.top_roots.as_slice(),
                )
                || triage_reply_risk_shortlist_digest_changed(
                    candidate.reply_risk_shortlist.as_ref(),
                    baseline.reply_risk_shortlist.as_ref(),
                )
        }
        TriageSurface::Supermana
        | TriageSurface::OpponentMana
        | TriageSurface::SpiritSetup
        | TriageSurface::DrainerSafety => {
            candidate.selected_rank != baseline.selected_rank
                || candidate.top_root_count != baseline.top_root_count
                || triage_root_digest_changed(
                    surface,
                    &candidate.selected_root,
                    &baseline.selected_root,
                )
                || triage_top_root_digest_changed(
                    surface,
                    candidate.top_roots.as_slice(),
                    baseline.top_roots.as_slice(),
                )
        }
        TriageSurface::NormalFastGap
        | TriageSurface::NormalReleaseSeedGap
        | TriageSurface::NormalTiebreak
        | TriageSurface::OpeningReply
        | TriageSurface::PrimaryPro => {
            candidate.selected_root.input_fen != baseline.selected_root.input_fen
                || candidate.selected_rank != baseline.selected_rank
                || triage_top_root_digest_changed(
                    surface,
                    candidate.top_roots.as_slice(),
                    baseline.top_roots.as_slice(),
                )
        }
        TriageSurface::CacheReuse => false,
    }
}

fn triage_surface_fixture_changed(
    surface: TriageSurface,
    fixture: &TriageFixture,
    candidate: &TriageSignalSnapshot,
    baseline: &TriageSignalSnapshot,
) -> bool {
    match surface {
        TriageSurface::NormalFastGap => {
            let expected = fixture.expected_selected_input_fen.unwrap_or_else(|| {
                panic!(
                    "triage surface '{}' fixture '{}' is missing expected_selected_input_fen",
                    surface.as_str(),
                    fixture.id
                )
            });
            candidate.selected_root.input_fen == expected
                && baseline.selected_root.input_fen != expected
        }
        TriageSurface::NormalReleaseSeedGap => {
            let expected = fixture.expected_selected_input_fen.unwrap_or_else(|| {
                panic!(
                    "triage surface '{}' fixture '{}' is missing expected_selected_input_fen",
                    surface.as_str(),
                    fixture.id
                )
            });
            candidate.selected_root.input_fen == expected
                && baseline.selected_root.input_fen != expected
        }
        TriageSurface::PrimaryPro => {
            if let Some(expected) = fixture.expected_selected_input_fen {
                candidate.selected_root.input_fen == expected
                    && baseline.selected_root.input_fen != expected
            } else {
                triage_surface_signal_changed(surface, candidate, baseline)
            }
        }
        _ => triage_surface_signal_changed(surface, candidate, baseline),
    }
}

fn compare_triage_fixture_pack(
    surface: TriageSurface,
    candidate_profile: &str,
    candidate_selector: AutomoveSelector,
    baseline_profile: &str,
    baseline_selector: AutomoveSelector,
    fixtures: &[TriageFixture],
) -> usize {
    let mut changed = 0;
    for fixture in fixtures {
        let candidate_snapshot =
            triage_fixture_snapshot(surface, candidate_profile, candidate_selector, fixture);
        let baseline_snapshot =
            triage_fixture_snapshot(surface, baseline_profile, baseline_selector, fixture);
        let fixture_changed = triage_surface_fixture_changed(
            surface,
            fixture,
            &candidate_snapshot,
            &baseline_snapshot,
        );
        if fixture_changed {
            changed += 1;
        }
        println!(
            "triage surface={} fixture={} mode={} opening_book={} expected={:?} changed={} candidate_profile={} candidate={:?} baseline_profile={} baseline={:?}",
            surface.as_str(),
            fixture.id,
            fixture.mode.as_api_value(),
            fixture.opening_book_driven,
            fixture.expected_selected_input_fen,
            fixture_changed,
            candidate_profile,
            candidate_snapshot,
            baseline_profile,
            baseline_snapshot
        );
    }
    println!(
        "triage surface={} summary candidate={} baseline={} changed={}/{}",
        surface.as_str(),
        candidate_profile,
        baseline_profile,
        changed,
        fixtures.len()
    );
    changed
}

fn assert_stage1_cpu_non_regression(
    candidate_profile_name: &str,
    candidate_selector: AutomoveSelector,
) {
    let advisory_only = candidate_profile_name.starts_with("runtime_pro_")
        && env_bool("SMART_STAGE1_CPU_ADVISORY").unwrap_or(false);
    let baseline_selector = profile_selector_from_name("runtime_current")
        .expect("runtime_current selector should exist for stage-1 cpu gate");
    let budgets = stage1_cpu_budgets(candidate_profile_name);
    let repeats = stage1_cpu_measurement_repeats();
    let speed_positions = env_usize("SMART_STAGE1_SPEED_POSITIONS")
        .unwrap_or(16)
        .max(12);

    for seed_tag in stage1_seed_tags() {
        let speed_seed = seed_for_pairing(
            "stage1_cpu_gate",
            format!("{}:{}", candidate_profile_name, seed_tag).as_str(),
        );
        let speed_openings = generate_opening_fens_cached(speed_seed, speed_positions);
        let mut ratio_samples = std::collections::HashMap::<&'static str, Vec<f64>>::new();

        for _ in 0..repeats {
            let baseline_speed = profile_speed_by_mode_ms(
                baseline_selector,
                speed_openings.as_slice(),
                budgets.as_slice(),
            );
            let candidate_speed = profile_speed_by_mode_ms(
                candidate_selector,
                speed_openings.as_slice(),
                budgets.as_slice(),
            );
            let baseline_map = baseline_speed
                .iter()
                .map(|stat| (stat.budget.key(), stat.avg_ms))
                .collect::<std::collections::HashMap<_, _>>();

            for stat in candidate_speed {
                let baseline_ms = baseline_map
                    .get(stat.budget.key())
                    .copied()
                    .unwrap_or(1.0)
                    .max(0.001);
                let ratio = stat.avg_ms / baseline_ms;
                ratio_samples
                    .entry(stat.budget.key())
                    .or_default()
                    .push(ratio);
            }
        }

        for budget in &budgets {
            let mode = budget.key();
            let mut samples = ratio_samples.remove(mode).unwrap_or_default();
            assert_eq!(
                samples.len(),
                repeats,
                "stage-1 cpu gate expected {} samples for mode {}",
                repeats,
                mode
            );
            let ratio = median_f64(samples.as_mut_slice());
            let ratio_limit = stage1_cpu_ratio_limit(mode);
            println!(
                "stage-1 cpu seed={} mode={} candidate={} ratio={:.3} limit={:.3} samples={:?}",
                seed_tag, mode, candidate_profile_name, ratio, ratio_limit, samples
            );
            if advisory_only && ratio > ratio_limit {
                println!(
                    "stage-1 cpu advisory: seed={} mode={} candidate={} ratio={:.3} > {:.3}; continuing because SMART_STAGE1_CPU_ADVISORY=true for a Pro candidate",
                    seed_tag,
                    mode,
                    candidate_profile_name,
                    ratio,
                    ratio_limit,
                );
            } else {
                assert!(
                    ratio <= ratio_limit,
                    "stage-1 cpu gate failed for seed={} mode={} candidate={} baseline=runtime_current median_ratio={:.3} > {:.3} samples={:?}",
                    seed_tag,
                    mode,
                    candidate_profile_name,
                    ratio,
                    ratio_limit,
                    samples
                );
            }
        }
    }
}

fn profile_speed_with_turn_engine_diagnostics(
    selector: AutomoveSelector,
    openings: &[String],
    budgets: &[SearchBudget],
) -> Vec<(
    ModeSpeedStat,
    crate::models::automove_turn_engine::TurnEngineDiagnostics,
    crate::models::mons_game_model::TurnEngineSelectorDiagnostics,
)> {
    let mut stats = Vec::with_capacity(budgets.len());
    for budget in budgets.iter().copied() {
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();
        let start = std::time::Instant::now();
        for opening in openings {
            let game = MonsGame::from_fen(opening, false).expect("valid opening fen");
            let config = budget.runtime_config_for_game(&game);
            let _ = select_inputs_with_runtime_fallback(selector, &game, config);
        }
        stats.push((
            ModeSpeedStat {
                budget,
                avg_ms: start.elapsed().as_secs_f64() * 1000.0 / openings.len().max(1) as f64,
            },
            turn_engine_diagnostics_snapshot(),
            turn_engine_selector_diagnostics_snapshot(),
        ));
    }
    stats
}

#[test]
#[ignore = "diagnostic: stage1 cpu probe for pro turn engine callsites"]
fn smart_automove_pro_turn_engine_stage1_cpu_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".to_string());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".to_string());
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate profile '{}' not found", candidate_profile));
    let baseline_selector = profile_selector_from_name(baseline_profile.as_str())
        .unwrap_or_else(|| panic!("baseline profile '{}' not found", baseline_profile));
    let positions = env_usize("SMART_STAGE1_SPEED_POSITIONS")
        .unwrap_or(16)
        .max(12);
    let budgets = stage1_cpu_budgets(candidate_profile.as_str());

    for seed_tag in stage1_seed_tags() {
        let speed_seed = seed_for_pairing(
            "stage1_cpu_gate",
            format!("{}:{}", candidate_profile, seed_tag).as_str(),
        );
        let openings = generate_opening_fens_cached(speed_seed, positions);
        let baseline =
            profile_speed_by_mode_ms(baseline_selector, openings.as_slice(), budgets.as_slice());
        let candidate = profile_speed_with_turn_engine_diagnostics(
            candidate_selector,
            openings.as_slice(),
            budgets.as_slice(),
        );
        for ((candidate_stat, engine_diag, selector_diag), baseline_stat) in
            candidate.into_iter().zip(baseline.into_iter())
        {
            let ratio = candidate_stat.avg_ms / baseline_stat.avg_ms.max(0.001);
            println!(
                "stage1 cpu probe seed={} mode={} candidate={} baseline={} candidate_ms={:.3} baseline_ms={:.3} ratio={:.3} selector(head={}/{} selected={}/{}/{} spirit={}/{}/{}/{} reply={}/{}/{}/{} followup_floor={}) engine(cache={}/{} accepted={} reply_calls={} seeds=[score:{} deny:{} kill:{} super:{} opp:{} safety:{} spirit:{} tempo:{}] compile_attempts={} compile_failures={} fallback_no_plan={} fallback_budget={})",
                seed_tag,
                candidate_stat.budget.key(),
                candidate_profile,
                baseline_profile,
                candidate_stat.avg_ms,
                baseline_stat.avg_ms,
                ratio,
                selector_diag.head_plan_hits,
                selector_diag.head_plan_calls,
                selector_diag.selected_override_calls,
                selector_diag.selected_override_plan_hits,
                selector_diag.selected_override_plan_calls,
                selector_diag.spirit_projection_passes,
                selector_diag.spirit_projection_root_count,
                selector_diag.spirit_projection_plan_hits,
                selector_diag.spirit_projection_plan_calls,
                selector_diag.reply_projection_passes,
                selector_diag.reply_projection_root_count,
                selector_diag.reply_projection_plan_hits,
                selector_diag.reply_projection_plan_calls,
                selector_diag.followup_floor_calls,
                engine_diag.cache_hits,
                engine_diag.cache_misses,
                engine_diag.accepted_plans,
                engine_diag.reply_search_calls,
                engine_diag.seed_immediate_score,
                engine_diag.seed_deny_window,
                engine_diag.seed_drainer_kill,
                engine_diag.seed_safe_supermana_progress,
                engine_diag.seed_safe_opponent_mana_progress,
                engine_diag.seed_safety_recovery,
                engine_diag.seed_spirit_impact,
                engine_diag.seed_mana_tempo,
                engine_diag.compile_attempts,
                engine_diag.compile_failures,
                engine_diag.fallback_no_plan,
                engine_diag.fallback_budget_exceeded,
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: per-opening stage1 cpu probe for pro turn engine"]
fn smart_automove_pro_turn_engine_stage1_cpu_opening_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".to_string());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".to_string());
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate profile '{}' not found", candidate_profile));
    let baseline_selector = profile_selector_from_name(baseline_profile.as_str())
        .unwrap_or_else(|| panic!("baseline profile '{}' not found", baseline_profile));
    let seed_tag =
        env::var("SMART_STAGE1_OPENING_PROBE_SEED").unwrap_or_else(|_| "stage1_cpu_v2".to_string());
    let positions = env_usize("SMART_STAGE1_SPEED_POSITIONS")
        .unwrap_or(12)
        .max(1);
    let budgets = stage1_cpu_budgets(candidate_profile.as_str());
    let budget = *budgets
        .first()
        .expect("stage1 opening probe requires at least one budget");
    let speed_seed = seed_for_pairing(
        "stage1_cpu_gate",
        format!("{}:{}", candidate_profile, seed_tag).as_str(),
    );
    let openings = generate_opening_fens_cached(speed_seed, positions);

    for (index, opening) in openings.iter().enumerate() {
        let game = MonsGame::from_fen(opening, false).expect("valid opening fen");
        let base_runtime = budget.runtime_config_for_game(&game);
        let candidate_runtime =
            profile_runtime_config_for_name(candidate_profile.as_str(), &game, base_runtime)
                .unwrap_or(base_runtime);
        let perspective = game.active_color;
        let context = exact_opportunity_context(&game, perspective);
        let strategic = exact_strategic_analysis(&game).color_summary(perspective);

        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();
        let baseline_start = std::time::Instant::now();
        let baseline_inputs =
            select_inputs_with_runtime_fallback(baseline_selector, &game, base_runtime);
        let baseline_ms = baseline_start.elapsed().as_secs_f64() * 1000.0;

        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();
        let candidate_start = std::time::Instant::now();
        let candidate_inputs =
            select_inputs_with_runtime_fallback(candidate_selector, &game, base_runtime);
        let candidate_ms = candidate_start.elapsed().as_secs_f64() * 1000.0;
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let engine_diag = turn_engine_diagnostics_snapshot();
        if selector_diag.head_plan_calls == 0 {
            continue;
        }

        let ratio = candidate_ms / baseline_ms.max(0.001);
        let plan_digests = turn_engine_ranked_plan_digests_for_test(
            &game,
            perspective,
            calibration_turn_engine_config(candidate_runtime),
            3,
        );
        let best_plan = turn_engine_best_plan_for_test(
            &game,
            perspective,
            calibration_turn_engine_config(candidate_runtime),
        )
        .map(|plan| {
            let head = plan
                .compiled_chunks
                .first()
                .map(|chunk| Input::fen_from_array(chunk.as_slice()))
                .unwrap_or_default();
            format!(
                "{}:{:?}->{:?}/chunks={}",
                head,
                plan.head_family,
                plan.goal_family,
                plan.compiled_chunks.len()
            )
        })
        .unwrap_or_else(|| "none".to_string());

        println!(
            "stage1 opening probe seed={} idx={} ratio={:.3} baseline_ms={:.3} candidate_ms={:.3} fen={} active={:?} turn={} mon_moves={} can_action={} can_mana={} eligible={} selected={} baseline_selected={} head_calls={} head_hits={} selected_override={}/{}/{} spirit_projection={}/{}/{} reply_projection={}/{}/{} followup_floor={} accepted={} compile_attempts={} opp_win_now={} score_window={} deny_gain={} drainer_attack={} drainer_safety={} spirit_gain={} spirit_setup_gain={} best_plan={} top_plans={:?}",
            seed_tag,
            index,
            ratio,
            baseline_ms,
            candidate_ms,
            opening,
            game.active_color,
            game.turn_number,
            game.mons_moves_count,
            game.player_can_use_action(),
            game.player_can_move_mana(),
            crate::models::automove_turn_engine::pro_v2_turn_engine_eligible(&game),
            Input::fen_from_array(candidate_inputs.as_slice()),
            Input::fen_from_array(baseline_inputs.as_slice()),
            selector_diag.head_plan_calls,
            selector_diag.head_plan_hits,
            selector_diag.selected_override_calls,
            selector_diag.selected_override_plan_calls,
            selector_diag.selected_override_plan_hits,
            selector_diag.spirit_projection_passes,
            selector_diag.spirit_projection_root_count,
            selector_diag.spirit_projection_plan_calls,
            selector_diag.reply_projection_passes,
            selector_diag.reply_projection_root_count,
            selector_diag.reply_projection_plan_calls,
            selector_diag.followup_floor_calls,
            engine_diag.accepted_plans,
            engine_diag.compile_attempts,
            context.opponent_can_win_immediately,
            context.delta.same_turn_score_window_value,
            context.delta.opponent_window_deny_gain,
            context.delta.drainer_attack_available,
            context.delta.drainer_safety,
            context.delta.spirit_gain,
            strategic.spirit.next_turn_setup_gain,
            best_plan,
            plan_digests,
        );
    }
}

fn exact_lite_cache_totals() -> (usize, usize) {
    let diagnostics = exact_query_diagnostics_snapshot();
    let calls = diagnostics.exact_spirit_summary_calls as usize
        + diagnostics.tactical_spirit_summary_calls as usize
        + diagnostics.exact_followup_summary_calls as usize
        + diagnostics.exact_secure_mana_calls as usize
        + diagnostics.pickup_path_calls as usize;
    let hits = diagnostics.exact_spirit_summary_cache_hits as usize
        + diagnostics.tactical_spirit_summary_cache_hits as usize
        + diagnostics.exact_followup_summary_cache_hits as usize
        + diagnostics.exact_secure_mana_cache_hits as usize
        + diagnostics.pickup_path_cache_hits as usize;
    (calls, hits)
}

#[derive(Debug, Clone, Copy)]
struct CacheReuseProbe {
    avg_ms: f64,
    calls: usize,
    hits: usize,
    hit_rate: f64,
    intent_generation_calls: usize,
    intent_generation_hits: usize,
    compile_fallbacks: usize,
    injected_root_attempts: usize,
    injected_root_accepts: usize,
    injected_root_candidates_seen: usize,
    injected_root_duplicates: usize,
    injected_root_reject_build: usize,
    injected_root_reject_emergency_guard: usize,
    injected_root_reject_emergency_introduced_loss: usize,
    injected_root_reject_emergency_no_crisis_signal: usize,
    injected_root_reject_emergency_mana_handoff: usize,
    injected_root_reject_emergency_drainer_unsafe: usize,
    injected_root_reject_top_wins: usize,
    injected_root_reject_candidate_unsafe: usize,
    injected_root_reject_no_tactical_signal: usize,
    injected_root_reject_heuristic_gap: usize,
}

fn env_f64(name: &str) -> Option<f64> {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<f64>().ok())
}

fn cache_reuse_triage_probe(_profile_name: &str, selector: AutomoveSelector) -> CacheReuseProbe {
    let budgets = client_budgets().to_vec();
    let positions = env_usize("SMART_TRIAGE_SPEED_POSITIONS")
        .unwrap_or(6)
        .max(2);
    let openings =
        generate_opening_fens_cached(seed_for_pairing("triage_cache_reuse", "fixed"), positions);
    let speed_stats = profile_speed_by_mode_ms(selector, openings.as_slice(), budgets.as_slice());
    let avg_ms = if speed_stats.is_empty() {
        0.0
    } else {
        speed_stats.iter().map(|stat| stat.avg_ms).sum::<f64>() / speed_stats.len() as f64
    };

    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_planner_diagnostics();
    let repeats = env_usize("SMART_TRIAGE_CACHE_REPEATS").unwrap_or(2).max(1);
    for _ in 0..repeats {
        for opening in openings.iter() {
            let game = MonsGame::from_fen(opening, false).expect("valid cache triage opening");
            for budget in budgets.iter().copied() {
                let config = budget.runtime_config_for_game(&game);
                let _ = select_inputs_with_runtime_fallback(selector, &game, config);
            }
        }
    }
    let (calls, hits) = exact_lite_cache_totals();
    let planner_diag = turn_planner_diagnostics_snapshot();
    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_planner_diagnostics();

    CacheReuseProbe {
        avg_ms,
        calls,
        hits,
        hit_rate: if calls == 0 {
            0.0
        } else {
            hits as f64 / calls as f64
        },
        intent_generation_calls: planner_diag.intent_generation_calls,
        intent_generation_hits: planner_diag.intent_generation_hits,
        compile_fallbacks: planner_diag.compile_fallbacks,
        injected_root_attempts: planner_diag.injected_root_attempts,
        injected_root_accepts: planner_diag.injected_root_accepts,
        injected_root_candidates_seen: planner_diag.injected_root_candidates_seen,
        injected_root_duplicates: planner_diag.injected_root_duplicates,
        injected_root_reject_build: planner_diag.injected_root_reject_build,
        injected_root_reject_emergency_guard: planner_diag.injected_root_reject_emergency_guard,
        injected_root_reject_emergency_introduced_loss: planner_diag
            .injected_root_reject_emergency_introduced_loss,
        injected_root_reject_emergency_no_crisis_signal: planner_diag
            .injected_root_reject_emergency_no_crisis_signal,
        injected_root_reject_emergency_mana_handoff: planner_diag
            .injected_root_reject_emergency_mana_handoff,
        injected_root_reject_emergency_drainer_unsafe: planner_diag
            .injected_root_reject_emergency_drainer_unsafe,
        injected_root_reject_top_wins: planner_diag.injected_root_reject_top_wins,
        injected_root_reject_candidate_unsafe: planner_diag.injected_root_reject_candidate_unsafe,
        injected_root_reject_no_tactical_signal: planner_diag
            .injected_root_reject_no_tactical_signal,
        injected_root_reject_heuristic_gap: planner_diag.injected_root_reject_heuristic_gap,
    }
}

fn cache_reuse_triage_passes(candidate: CacheReuseProbe, baseline: CacheReuseProbe) -> bool {
    let faster = candidate.avg_ms <= baseline.avg_ms * 0.97;
    let better_cache =
        candidate.calls > 0 && baseline.calls > 0 && candidate.hit_rate >= baseline.hit_rate + 0.05;
    let candidate_accept_rate = if candidate.injected_root_attempts == 0 {
        0.0
    } else {
        candidate.injected_root_accepts as f64 / candidate.injected_root_attempts as f64
    };
    let baseline_accept_rate = if baseline.injected_root_attempts == 0 {
        0.0
    } else {
        baseline.injected_root_accepts as f64 / baseline.injected_root_attempts as f64
    };
    let stronger_planner_signal = candidate.intent_generation_calls > 0
        && candidate.intent_generation_hits >= baseline.intent_generation_hits
        && candidate.compile_fallbacks <= baseline.compile_fallbacks.saturating_add(8)
        && candidate_accept_rate >= baseline_accept_rate;
    faster || better_cache || stronger_planner_signal
}

fn assert_exact_lite_diagnostics_gate_if_enabled(
    candidate_profile_name: &str,
    candidate_selector: AutomoveSelector,
) {
    let budgets = stage1_cpu_budgets(candidate_profile_name);
    let positions = env_usize("SMART_EXACT_LITE_DIAGNOSTIC_POSITIONS")
        .unwrap_or(8)
        .max(1);
    let openings = generate_opening_fens_cached(
        seed_for_pairing("exact_lite_diag", candidate_profile_name),
        positions,
    );
    let cache_repeats = env_usize("SMART_EXACT_LITE_CACHE_REPEATS")
        .unwrap_or(2)
        .max(2);
    let min_cache_calls = env_usize("SMART_EXACT_LITE_CACHE_MIN_CALLS")
        .unwrap_or(12)
        .max(1);
    let min_cache_hit_rate = env_f64("SMART_EXACT_LITE_CACHE_HIT_RATE_MIN")
        .unwrap_or(SMART_EXACT_LITE_CACHE_HIT_RATE_MIN)
        .clamp(0.0, 1.0);

    let mut any_exact_lite_budget = false;
    for budget in budgets.iter().copied() {
        for opening in openings.iter() {
            let game = MonsGame::from_fen(opening, false).expect("valid opening fen");
            let config = budget.runtime_config_for_game(&game);
            let Some(limits) = profile_exact_lite_budgets(candidate_profile_name, &game, config)
            else {
                continue;
            };
            any_exact_lite_budget = true;
            clear_exact_state_analysis_cache();
            clear_exact_query_diagnostics();
            let _ = select_inputs_with_runtime_fallback(candidate_selector, &game, config);
            let diagnostics = exact_query_diagnostics_snapshot();
            let root_calls = diagnostics.exact_turn_summary_builds as usize;
            let static_calls = (diagnostics.passive_strategic_summary_builds as usize).div_ceil(2);

            assert!(
                root_calls <= limits.root_call_budget,
                "exact-lite root budget exceeded for profile={} mode={} opening={} calls={} budget={}",
                candidate_profile_name,
                budget.key(),
                opening,
                root_calls,
                limits.root_call_budget
            );
            assert!(
                static_calls <= limits.static_call_budget,
                "exact-lite static budget exceeded for profile={} mode={} opening={} calls={} budget={}",
                candidate_profile_name,
                budget.key(),
                opening,
                static_calls,
                limits.static_call_budget
            );
        }
    }

    if !any_exact_lite_budget {
        return;
    }

    for budget in budgets.iter().copied() {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        let mut budget_uses_exact_lite = false;
        for _ in 0..cache_repeats {
            for opening in openings.iter() {
                let game = MonsGame::from_fen(opening, false).expect("valid opening fen");
                let config = budget.runtime_config_for_game(&game);
                if profile_exact_lite_budgets(candidate_profile_name, &game, config).is_none() {
                    continue;
                }
                budget_uses_exact_lite = true;
                let _ = select_inputs_with_runtime_fallback(candidate_selector, &game, config);
            }
        }

        if !budget_uses_exact_lite {
            continue;
        }
        let (cache_calls, cache_hits) = exact_lite_cache_totals();
        if cache_calls < min_cache_calls {
            continue;
        }
        let cache_hit_rate = cache_hits as f64 / cache_calls as f64;
        assert!(
            cache_hit_rate >= min_cache_hit_rate,
            "exact-lite cache-hit gate failed for profile={} mode={} rate={:.3} < {:.3} (hits={}, calls={})",
            candidate_profile_name,
            budget.key(),
            cache_hit_rate,
            min_cache_hit_rate,
            cache_hits,
            cache_calls
        );
    }
    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
}

fn mode_compare_seed_tags() -> Vec<String> {
    let from_env = env::var("SMART_MODE_COMPARE_SEED_TAGS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !from_env.is_empty() {
        return from_env;
    }
    vec![
        "mode_cmp_v1".to_string(),
        "mode_cmp_v2".to_string(),
        "mode_cmp_v3".to_string(),
        "mode_cmp_v4".to_string(),
        "mode_cmp_v5".to_string(),
    ]
}

fn mode_compare_modes() -> Vec<SmartAutomovePreference> {
    let from_env = env::var("SMART_MODE_COMPARE_MODES")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().to_ascii_lowercase())
                .filter_map(|item| match item.as_str() {
                    "fast" => Some(SmartAutomovePreference::Fast),
                    "normal" => Some(SmartAutomovePreference::Normal),
                    "pro" => Some(SmartAutomovePreference::Pro),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !from_env.is_empty() {
        return from_env;
    }
    vec![
        SmartAutomovePreference::Fast,
        SmartAutomovePreference::Normal,
        SmartAutomovePreference::Pro,
    ]
}

fn compare_focus_mode_from_env(
    name: &str,
    fallback: SmartAutomovePreference,
) -> SmartAutomovePreference {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .and_then(|value| match value.as_str() {
            "fast" => Some(SmartAutomovePreference::Fast),
            "normal" => Some(SmartAutomovePreference::Normal),
            "pro" => Some(SmartAutomovePreference::Pro),
            _ => None,
        })
        .unwrap_or(fallback)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LossProbeConfigDigest {
    max_visited_nodes: usize,
    root_branch_limit: usize,
    node_branch_limit: usize,
    root_focus_k: usize,
    root_focus_budget_share_bp: i32,
    enable_event_ordering_bonus: bool,
    enable_two_pass_root_allocation: bool,
    enable_two_pass_volatility_focus: bool,
    enable_quiet_reductions: bool,
    quiet_reduction_depth_threshold: usize,
    enable_normal_root_safety_rerank: bool,
    enable_normal_root_safety_deep_floor: bool,
    enable_interview_hard_spirit_deploy: bool,
    prefer_clean_reply_risk_roots: bool,
    root_reply_risk_shortlist_max: usize,
    root_reply_risk_reply_limit: usize,
    root_reply_risk_node_share_bp: i32,
    root_efficiency_score_margin: i32,
    root_drainer_safety_score_margin: i32,
}

#[derive(Debug, Clone)]
struct LossProbeTurnEngineDecision {
    status: TurnEngineProbeStatus,
    cached_move_fen: Option<String>,
    cached_rank: Option<usize>,
    candidate_family: Option<TurnPlanFamily>,
    candidate_move_fen: Option<String>,
    candidate_rank: Option<usize>,
    chunk_count: usize,
    root_search_selected_move_fen: Option<String>,
    root_search_selected_rank: Option<usize>,
    accepted_after_search: Option<bool>,
    selected_utility: Option<TurnEngineUtility>,
    candidate_utility: Option<TurnEngineUtility>,
}

struct LossProbeDecision {
    inputs: Vec<Input>,
    move_fen: String,
    selected_rank: Option<usize>,
    selected_root: Option<TriageRootDigestEntry>,
    top_roots: Vec<TriageRootDigestEntry>,
    turn_engine: Option<LossProbeTurnEngineDecision>,
    selector_last_stage: &'static str,
    config: LossProbeConfigDigest,
}

struct LossProbeTrace {
    ply: usize,
    fen: String,
    normal: LossProbeDecision,
    fast: LossProbeDecision,
    pro: LossProbeDecision,
}

struct ReliabilityLossProbeTrace {
    ply: usize,
    fen: String,
    candidate: LossProbeDecision,
    baseline: LossProbeDecision,
}

fn loss_probe_seed_tags() -> Vec<String> {
    let from_env = env::var("SMART_PROBE_SEED_TAGS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !from_env.is_empty() {
        return from_env;
    }
    vec![
        "normal_fast_probe_v1".to_string(),
        "normal_fast_probe_v2".to_string(),
        "normal_fast_probe_v3".to_string(),
    ]
}

fn loss_probe_runtime_config(
    profile_name: &str,
    game: &MonsGame,
    mode: SmartAutomovePreference,
) -> SmartSearchConfig {
    let base = SearchBudget::from_preference(mode).runtime_config_for_game(game);
    profile_runtime_config_for_name(profile_name, game, base).unwrap_or_else(|| {
        panic!(
            "profile '{}' does not expose a runtime config for mode '{}'",
            profile_name,
            mode.as_api_value()
        )
    })
}

fn loss_probe_turn_engine_decision_with_options(
    game: &MonsGame,
    config: SmartSearchConfig,
    ranked_roots: &[ScoredRootMove],
    include_acceptance: bool,
) -> Option<LossProbeTurnEngineDecision> {
    if !config.enable_turn_engine {
        return None;
    }

    let probe = turn_engine_probe(
        game,
        game.active_color,
        config.turn_engine_mode,
        calibration_turn_engine_config(config),
    );

    let cached_rank = probe.cached_step.as_ref().and_then(|inputs| {
        ranked_roots
            .iter()
            .position(|root| root.inputs.as_slice() == inputs.as_slice())
    });
    let cached_move_fen = probe
        .cached_step
        .as_ref()
        .map(|inputs| Input::fen_from_array(inputs));
    let candidate_rank = probe.candidate_chunk.as_ref().and_then(|inputs| {
        ranked_roots
            .iter()
            .position(|root| root.inputs.as_slice() == inputs.as_slice())
    });
    let candidate_move_fen = probe
        .candidate_chunk
        .as_ref()
        .map(|inputs| Input::fen_from_array(inputs));
    let acceptance = if include_acceptance {
        MonsGameModel::turn_engine_acceptance_probe_for_test(game, config)
    } else {
        None
    };

    Some(LossProbeTurnEngineDecision {
        status: probe.status,
        cached_move_fen,
        cached_rank,
        candidate_family: probe.candidate_family,
        candidate_move_fen,
        candidate_rank,
        chunk_count: probe.chunk_count,
        root_search_selected_move_fen: acceptance
            .as_ref()
            .map(|probe| Input::fen_from_array(&probe.selected_inputs)),
        root_search_selected_rank: acceptance.as_ref().map(|probe| probe.selected_index),
        accepted_after_search: acceptance.as_ref().map(|probe| probe.accepted),
        selected_utility: acceptance.as_ref().map(|probe| probe.selected_utility),
        candidate_utility: acceptance.as_ref().map(|probe| probe.candidate_utility),
    })
}

fn loss_probe_decision_with_options(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
    include_acceptance: bool,
) -> LossProbeDecision {
    let selector = profile_selector_from_name(profile_name)
        .unwrap_or_else(|| panic!("profile '{}' not found for loss probe", profile_name));
    let config = loss_probe_runtime_config(profile_name, game, mode);
    clear_turn_engine_selector_diagnostics();
    let inputs = select_inputs_with_runtime_fallback(selector, game, config);
    let selector_last_stage = turn_engine_selector_diagnostics_snapshot().last_return_stage;
    let move_fen = Input::fen_from_array(&inputs);
    let ranked_roots = MonsGameModel::ranked_root_moves(game, game.active_color, config);
    let selected_rank = ranked_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == move_fen);
    let selected_root = if let Some(rank) = selected_rank {
        Some(triage_root_digest_entry(&ranked_roots[rank]))
    } else {
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            game,
            game.active_color,
            config.enable_enhanced_drainer_vulnerability,
        );
        MonsGameModel::build_scored_root_move(
            game,
            game.active_color,
            config,
            own_drainer_vulnerable_before,
            &inputs,
        )
        .map(|selected| triage_root_digest_entry(&selected))
    };
    let top_roots = ranked_roots
        .iter()
        .take(TRIAGE_TOP_ROOT_DIGEST_SIZE)
        .map(triage_root_digest_entry)
        .collect::<Vec<_>>();
    let turn_engine = loss_probe_turn_engine_decision_with_options(
        game,
        config,
        ranked_roots.as_slice(),
        include_acceptance,
    );

    LossProbeDecision {
        inputs,
        move_fen,
        selected_rank,
        selected_root,
        top_roots,
        turn_engine,
        selector_last_stage,
        config: LossProbeConfigDigest {
            max_visited_nodes: config.max_visited_nodes,
            root_branch_limit: config.root_branch_limit,
            node_branch_limit: config.node_branch_limit,
            root_focus_k: config.root_focus_k,
            root_focus_budget_share_bp: config.root_focus_budget_share_bp,
            enable_event_ordering_bonus: config.enable_event_ordering_bonus,
            enable_two_pass_root_allocation: config.enable_two_pass_root_allocation,
            enable_two_pass_volatility_focus: config.enable_two_pass_volatility_focus,
            enable_quiet_reductions: config.enable_quiet_reductions,
            quiet_reduction_depth_threshold: config.quiet_reduction_depth_threshold,
            enable_normal_root_safety_rerank: config.enable_normal_root_safety_rerank,
            enable_normal_root_safety_deep_floor: config.enable_normal_root_safety_deep_floor,
            enable_interview_hard_spirit_deploy: config.enable_interview_hard_spirit_deploy,
            prefer_clean_reply_risk_roots: config.prefer_clean_reply_risk_roots,
            root_reply_risk_shortlist_max: config.root_reply_risk_shortlist_max,
            root_reply_risk_reply_limit: config.root_reply_risk_reply_limit,
            root_reply_risk_node_share_bp: config.root_reply_risk_node_share_bp,
            root_efficiency_score_margin: config.root_efficiency_score_margin,
            root_drainer_safety_score_margin: config.root_drainer_safety_score_margin,
        },
    }
}

fn loss_probe_direct_runtime_decision_with_options(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
    include_acceptance: bool,
) -> LossProbeDecision {
    let config = loss_probe_runtime_config(profile_name, game, mode);
    clear_turn_engine_selector_diagnostics();
    let inputs = MonsGameModel::smart_search_best_inputs(game, config);
    let selector_last_stage = turn_engine_selector_diagnostics_snapshot().last_return_stage;
    let move_fen = Input::fen_from_array(&inputs);
    let ranked_roots = MonsGameModel::ranked_root_moves(game, game.active_color, config);
    let selected_rank = ranked_roots
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == move_fen);
    let selected_root = if let Some(rank) = selected_rank {
        Some(triage_root_digest_entry(&ranked_roots[rank]))
    } else {
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            game,
            game.active_color,
            config.enable_enhanced_drainer_vulnerability,
        );
        MonsGameModel::build_scored_root_move(
            game,
            game.active_color,
            config,
            own_drainer_vulnerable_before,
            &inputs,
        )
        .map(|selected| triage_root_digest_entry(&selected))
    };
    let top_roots = ranked_roots
        .iter()
        .take(TRIAGE_TOP_ROOT_DIGEST_SIZE)
        .map(triage_root_digest_entry)
        .collect::<Vec<_>>();
    let turn_engine = loss_probe_turn_engine_decision_with_options(
        game,
        config,
        ranked_roots.as_slice(),
        include_acceptance,
    );

    LossProbeDecision {
        inputs,
        move_fen,
        selected_rank,
        selected_root,
        top_roots,
        turn_engine,
        selector_last_stage,
        config: LossProbeConfigDigest {
            max_visited_nodes: config.max_visited_nodes,
            root_branch_limit: config.root_branch_limit,
            node_branch_limit: config.node_branch_limit,
            root_focus_k: config.root_focus_k,
            root_focus_budget_share_bp: config.root_focus_budget_share_bp,
            enable_event_ordering_bonus: config.enable_event_ordering_bonus,
            enable_two_pass_root_allocation: config.enable_two_pass_root_allocation,
            enable_two_pass_volatility_focus: config.enable_two_pass_volatility_focus,
            enable_quiet_reductions: config.enable_quiet_reductions,
            quiet_reduction_depth_threshold: config.quiet_reduction_depth_threshold,
            enable_normal_root_safety_rerank: config.enable_normal_root_safety_rerank,
            enable_normal_root_safety_deep_floor: config.enable_normal_root_safety_deep_floor,
            enable_interview_hard_spirit_deploy: config.enable_interview_hard_spirit_deploy,
            prefer_clean_reply_risk_roots: config.prefer_clean_reply_risk_roots,
            root_reply_risk_shortlist_max: config.root_reply_risk_shortlist_max,
            root_reply_risk_reply_limit: config.root_reply_risk_reply_limit,
            root_reply_risk_node_share_bp: config.root_reply_risk_node_share_bp,
            root_efficiency_score_margin: config.root_efficiency_score_margin,
            root_drainer_safety_score_margin: config.root_drainer_safety_score_margin,
        },
    }
}

fn loss_probe_decision(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
) -> LossProbeDecision {
    loss_probe_decision_with_options(
        profile_name,
        mode,
        game,
        env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(false),
    )
}

fn print_loss_probe_decision(label: &str, decision: &LossProbeDecision) {
    eprintln!(
        "{} move={} rank={:?} cfg(nodes={} root={} node={} focus={}/{} event_order={} two_pass={} vol_focus={} quiet_red={} q_depth={} safety_rerank={} deep_floor={} hard_spirit={} clean_reply={} reply={}/{} reply_share_bp={} eff_margin={} drainer_margin={})",
        label,
        decision.move_fen,
        decision.selected_rank,
        decision.config.max_visited_nodes,
        decision.config.root_branch_limit,
        decision.config.node_branch_limit,
        decision.config.root_focus_k,
        decision.config.root_focus_budget_share_bp,
        decision.config.enable_event_ordering_bonus,
        decision.config.enable_two_pass_root_allocation,
        decision.config.enable_two_pass_volatility_focus,
        decision.config.enable_quiet_reductions,
        decision.config.quiet_reduction_depth_threshold,
        decision.config.enable_normal_root_safety_rerank,
        decision.config.enable_normal_root_safety_deep_floor,
        decision.config.enable_interview_hard_spirit_deploy,
        decision.config.prefer_clean_reply_risk_roots,
        decision.config.root_reply_risk_shortlist_max,
        decision.config.root_reply_risk_reply_limit,
        decision.config.root_reply_risk_node_share_bp,
        decision.config.root_efficiency_score_margin,
        decision.config.root_drainer_safety_score_margin,
    );
    eprintln!("{} selector_stage={}", label, decision.selector_last_stage);
    eprintln!("{} selected_root={:?}", label, decision.selected_root);
    if let Some(engine) = decision.turn_engine.as_ref() {
        eprintln!(
            "{} turn_engine=status={:?} cached_move={:?}@{:?} candidate_family={:?} candidate_move={:?}@{:?} root_search_selected={:?}@{:?} chunk_count={} selected_matches_candidate={} accepted_after_search={:?} selected_utility={:?} candidate_utility={:?}",
            label,
            engine.status,
            engine.cached_move_fen,
            engine.cached_rank,
            engine.candidate_family,
            engine.candidate_move_fen,
            engine.candidate_rank,
            engine.root_search_selected_move_fen,
            engine.root_search_selected_rank,
            engine.chunk_count,
            engine.candidate_move_fen.as_ref() == Some(&decision.move_fen),
            engine.accepted_after_search,
            engine.selected_utility,
            engine.candidate_utility,
        );
    }
    eprintln!("{} top_roots={:?}", label, decision.top_roots);
}

fn print_turn_engine_acceptance_probe(
    label: &str,
    probe: Option<&crate::models::mons_game_model::TurnEngineAcceptanceProbe>,
) {
    let Some(probe) = probe else {
        println!("  {}=None", label);
        return;
    };
    println!(
        "  {} selected={}@{} score={} candidate={}@{} score={} chunk_count={} accepted={} selected_utility={:?} candidate_utility={:?} selected_progress_surface={} candidate_progress_surface={} selected_spirit_phase={} progress_better={} pickup_upgrade={} near_tie_progress={} large_safe_search_lead={} primary_axes_better={} allow_non_concrete_white_progress_head={} white_plain_spirit_progress_override={} strategic_override_axes_better={} allow_safe_progress_fallback={} safe_non_progress_fallback={} engine_progress_override={}",
        label,
        Input::fen_from_array(&probe.selected_inputs),
        probe.selected_index,
        probe.selected_score,
        Input::fen_from_array(&probe.candidate_inputs),
        probe.candidate_index,
        probe.candidate_score,
        probe.chunk_count,
        probe.accepted,
        probe.selected_utility,
        probe.candidate_utility,
        probe.selected_progress_surface,
        probe.candidate_progress_surface,
        probe.selected_spirit_phase,
        probe.progress_better,
        probe.pickup_upgrade,
        probe.near_tie_progress,
        probe.large_safe_search_lead,
        probe.primary_axes_better,
        probe.allow_non_concrete_white_progress_head,
        probe.white_plain_spirit_progress_override,
        probe.strategic_override_axes_better,
        probe.allow_safe_progress_fallback,
        probe.safe_non_progress_fallback,
        probe.engine_progress_override,
    );
    println!(
        "  {} scored_roots={:?}",
        label,
        probe
            .scored_root_debug_summaries
            .iter()
            .take(8)
            .collect::<Vec<_>>()
    );
    println!(
        "  {} filtered_roots={:?}",
        label, probe.filtered_root_move_fens
    );
    println!(
        "  {} reply_shortlist={:?} reply_selected={:?}",
        label, probe.reply_guard_shortlist_move_fens, probe.reply_guard_selected_move_fen
    );
}

fn replay_normal_vs_fast_loss_probe_game(
    normal_profile: &str,
    fast_profile: &str,
    pro_profile: &str,
    opening_fen: &str,
    normal_is_white: bool,
    max_plies: usize,
    use_white_opening_book: bool,
    trace_limit: usize,
) -> (MatchResult, Vec<LossProbeTrace>) {
    let fast_selector = profile_selector_from_name(fast_profile)
        .unwrap_or_else(|| panic!("profile '{}' not found for fast side", fast_profile));
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    let mut traces = Vec::new();

    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return (
                match_result_from_winner(winner_color, normal_is_white),
                traces,
            );
        }

        let normal_to_move = if normal_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };

        let inputs = if use_white_opening_book {
            MonsGameModel::white_first_turn_opening_next_inputs(&game).unwrap_or_else(|| {
                if normal_to_move {
                    let normal =
                        loss_probe_decision(normal_profile, SmartAutomovePreference::Normal, &game);
                    let fast =
                        loss_probe_decision(fast_profile, SmartAutomovePreference::Fast, &game);
                    let pro = loss_probe_decision(pro_profile, SmartAutomovePreference::Pro, &game);
                    let normal_inputs = normal.inputs.clone();
                    if normal.move_fen != fast.move_fen && traces.len() < trace_limit {
                        traces.push(LossProbeTrace {
                            ply,
                            fen: game.fen(),
                            normal,
                            fast,
                            pro,
                        });
                    }
                    normal_inputs
                } else {
                    let config = loss_probe_runtime_config(
                        fast_profile,
                        &game,
                        SmartAutomovePreference::Fast,
                    );
                    select_inputs_with_runtime_fallback(fast_selector, &game, config)
                }
            })
        } else if normal_to_move {
            let normal =
                loss_probe_decision(normal_profile, SmartAutomovePreference::Normal, &game);
            let fast = loss_probe_decision(fast_profile, SmartAutomovePreference::Fast, &game);
            let pro = loss_probe_decision(pro_profile, SmartAutomovePreference::Pro, &game);
            let normal_inputs = normal.inputs.clone();
            if normal.move_fen != fast.move_fen && traces.len() < trace_limit {
                traces.push(LossProbeTrace {
                    ply,
                    fen: game.fen(),
                    normal,
                    fast,
                    pro,
                });
            }
            normal_inputs
        } else {
            let config =
                loss_probe_runtime_config(fast_profile, &game, SmartAutomovePreference::Fast);
            select_inputs_with_runtime_fallback(fast_selector, &game, config)
        };

        if inputs.is_empty() {
            return if normal_to_move {
                (MatchResult::OpponentWin, traces)
            } else {
                (MatchResult::CandidateWin, traces)
            };
        }

        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return if normal_to_move {
                (MatchResult::OpponentWin, traces)
            } else {
                (MatchResult::CandidateWin, traces)
            };
        }
    }

    (
        match adjudicate_non_terminal_game(&game) {
            Some(winner_color) => match_result_from_winner(winner_color, normal_is_white),
            None => MatchResult::Draw,
        },
        traces,
    )
}

fn replay_pro_reliability_loss_probe_game_with_options(
    candidate_profile: &str,
    baseline_profile: &str,
    opening_fen: &str,
    candidate_is_white: bool,
    max_plies: usize,
    trace_limit: usize,
    include_acceptance: bool,
) -> (MatchResult, Vec<ReliabilityLossProbeTrace>) {
    let baseline_selector = profile_selector_from_name(baseline_profile).unwrap_or_else(|| {
        panic!(
            "profile '{}' not found for reliability probe baseline side",
            baseline_profile
        )
    });
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    let mut traces = Vec::new();

    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return (
                match_result_from_winner(winner_color, candidate_is_white),
                traces,
            );
        }

        let candidate_to_move = if candidate_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };

        let inputs = if candidate_to_move {
            let candidate = loss_probe_decision_with_options(
                candidate_profile,
                SmartAutomovePreference::Pro,
                &game,
                include_acceptance,
            );
            let baseline = loss_probe_decision_with_options(
                baseline_profile,
                SmartAutomovePreference::Pro,
                &game,
                include_acceptance,
            );
            let candidate_inputs = candidate.inputs.clone();
            if candidate.move_fen != baseline.move_fen && traces.len() < trace_limit {
                traces.push(ReliabilityLossProbeTrace {
                    ply,
                    fen: game.fen(),
                    candidate,
                    baseline,
                });
            }
            candidate_inputs
        } else {
            let config =
                loss_probe_runtime_config(baseline_profile, &game, SmartAutomovePreference::Pro);
            select_inputs_with_runtime_fallback(baseline_selector, &game, config)
        };

        if inputs.is_empty() {
            return if candidate_to_move {
                (MatchResult::OpponentWin, traces)
            } else {
                (MatchResult::CandidateWin, traces)
            };
        }

        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return if candidate_to_move {
                (MatchResult::OpponentWin, traces)
            } else {
                (MatchResult::CandidateWin, traces)
            };
        }
    }

    (
        match adjudicate_non_terminal_game(&game) {
            Some(winner_color) => match_result_from_winner(winner_color, candidate_is_white),
            None => MatchResult::Draw,
        },
        traces,
    )
}

fn replay_cross_budget_loss_probe_game_with_options(
    candidate_profile: &str,
    candidate_mode: SmartAutomovePreference,
    baseline_profile: &str,
    baseline_mode: SmartAutomovePreference,
    opening_fen: &str,
    candidate_is_white: bool,
    max_plies: usize,
    trace_limit: usize,
    include_acceptance: bool,
) -> (MatchResult, Vec<ReliabilityLossProbeTrace>) {
    let baseline_selector = profile_selector_from_name(baseline_profile).unwrap_or_else(|| {
        panic!(
            "profile '{}' not found for cross-budget probe baseline side",
            baseline_profile
        )
    });
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    let mut traces = Vec::new();
    let use_white_opening_book = env_bool("SMART_PROBE_USE_WHITE_OPENING_BOOK").unwrap_or(false);

    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return (
                match_result_from_winner(winner_color, candidate_is_white),
                traces,
            );
        }

        let candidate_to_move = if candidate_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };

        let inputs = if use_white_opening_book {
            MonsGameModel::white_first_turn_opening_next_inputs(&game).unwrap_or_else(|| {
                if candidate_to_move {
                    let candidate = loss_probe_decision_with_options(
                        candidate_profile,
                        candidate_mode,
                        &game,
                        include_acceptance,
                    );
                    let baseline = loss_probe_decision_with_options(
                        baseline_profile,
                        baseline_mode,
                        &game,
                        include_acceptance,
                    );
                    let candidate_inputs = candidate.inputs.clone();
                    if candidate.move_fen != baseline.move_fen && traces.len() < trace_limit {
                        traces.push(ReliabilityLossProbeTrace {
                            ply,
                            fen: game.fen(),
                            candidate,
                            baseline,
                        });
                    }
                    candidate_inputs
                } else {
                    let config = loss_probe_runtime_config(baseline_profile, &game, baseline_mode);
                    select_inputs_with_runtime_fallback(baseline_selector, &game, config)
                }
            })
        } else if candidate_to_move {
            let candidate = loss_probe_decision_with_options(
                candidate_profile,
                candidate_mode,
                &game,
                include_acceptance,
            );
            let baseline = loss_probe_decision_with_options(
                baseline_profile,
                baseline_mode,
                &game,
                include_acceptance,
            );
            let candidate_inputs = candidate.inputs.clone();
            if candidate.move_fen != baseline.move_fen && traces.len() < trace_limit {
                traces.push(ReliabilityLossProbeTrace {
                    ply,
                    fen: game.fen(),
                    candidate,
                    baseline,
                });
            }
            candidate_inputs
        } else {
            let config = loss_probe_runtime_config(baseline_profile, &game, baseline_mode);
            select_inputs_with_runtime_fallback(baseline_selector, &game, config)
        };

        if inputs.is_empty() {
            return if candidate_to_move {
                (MatchResult::OpponentWin, traces)
            } else {
                (MatchResult::CandidateWin, traces)
            };
        }

        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return if candidate_to_move {
                (MatchResult::OpponentWin, traces)
            } else {
                (MatchResult::CandidateWin, traces)
            };
        }
    }

    (
        match adjudicate_non_terminal_game(&game) {
            Some(winner_color) => match_result_from_winner(winner_color, candidate_is_white),
            None => MatchResult::Draw,
        },
        traces,
    )
}

fn replay_pro_reliability_loss_probe_game(
    candidate_profile: &str,
    baseline_profile: &str,
    opening_fen: &str,
    candidate_is_white: bool,
    max_plies: usize,
    trace_limit: usize,
) -> (MatchResult, Vec<ReliabilityLossProbeTrace>) {
    replay_pro_reliability_loss_probe_game_with_options(
        candidate_profile,
        baseline_profile,
        opening_fen,
        candidate_is_white,
        max_plies,
        trace_limit,
        env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(false),
    )
}

fn run_smart_automove_pro_fast_screen_loss_probe(
    baseline_preference: SmartAutomovePreference,
    default_seed_tag: &str,
    label: &str,
) {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_release_safe_pre_exact".into());
    let repeats = env_usize("SMART_PRO_FAST_SCREEN_REPEATS")
        .unwrap_or(2)
        .max(1);
    let games_per_repeat = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(84)
        .max(56);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(3).max(1);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let seed_tag =
        env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG").unwrap_or_else(|| default_seed_tag.into());
    let budget_a = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let budget_b = SearchBudget::from_preference(baseline_preference);
    let mut aggregate = MatchupStats::default();
    let mut losses = 0usize;
    let mut losses_with_disagreement = 0usize;
    let mut disagreements_logged = 0usize;

    eprintln!(
        "{} config: candidate_profile={} baseline_profile={} seed_tag={} repeats={} games_per_repeat={} max_plies={} trace_limit={} include_acceptance={}",
        label,
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeats,
        games_per_repeat,
        max_plies,
        trace_limit,
        include_acceptance,
    );

    for repeat_index in 0..repeats {
        let seed = seed_for_budget_duel_repeat_and_tag(
            budget_a,
            budget_b,
            repeat_index,
            seed_tag.as_str(),
        );
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in
                [("ab", candidate_white_ab), ("ba", !candidate_white_ab)]
            {
                let (result, traces) = replay_cross_budget_loss_probe_game_with_options(
                    candidate_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    baseline_profile.as_str(),
                    baseline_preference,
                    opening_fen.as_str(),
                    candidate_is_white,
                    max_plies,
                    trace_limit,
                    include_acceptance,
                );
                aggregate.record(result);

                if result != MatchResult::OpponentWin {
                    continue;
                }

                losses += 1;
                if !traces.is_empty() {
                    losses_with_disagreement += 1;
                }

                eprintln!(
                    "PRO_FAST_SCREEN_LOSS game={} repeat={} opening_index={} mirror={} candidate_is_white={} seed={} opening={}",
                    losses,
                    repeat_index,
                    opening_index,
                    mirror,
                    candidate_is_white,
                    seed,
                    opening_fen
                );
                for trace in &traces {
                    disagreements_logged += 1;
                    eprintln!("  TRACE ply={} fen={}", trace.ply, trace.fen);
                    print_loss_probe_decision("    candidate", &trace.candidate);
                    print_loss_probe_decision("    baseline", &trace.baseline);
                }
            }
        }
    }

    eprintln!(
        "{} summary: total_games={} wins={} losses={} draws={} win_rate={:.4} confidence={:.4} losses_with_disagreement={} disagreements_logged={}",
        label,
        aggregate.total_games(),
        aggregate.wins,
        aggregate.losses,
        aggregate.draws,
        aggregate.win_rate_points(),
        aggregate.confidence_better_than_even(),
        losses_with_disagreement,
        disagreements_logged,
    );
}

#[test]
#[ignore = "diagnostic: replay pro fast-screen vs normal losses and print candidate-vs-baseline divergences"]
fn smart_automove_pro_fast_screen_loss_probe_vs_normal() {
    run_smart_automove_pro_fast_screen_loss_probe(
        SmartAutomovePreference::Normal,
        "pro_fast_screen_vs_normal_v1",
        "pro fast-screen loss probe",
    );
}

#[test]
#[ignore = "diagnostic: replay pro fast-screen vs pro losses and print candidate-vs-baseline divergences"]
fn smart_automove_pro_fast_screen_loss_probe_vs_pro() {
    run_smart_automove_pro_fast_screen_loss_probe(
        SmartAutomovePreference::Pro,
        "pro_fast_screen_vs_pro_v1",
        "pro fast-screen loss probe vs pro",
    );
}

#[test]
#[ignore = "diagnostic: replay the same pro openings against current Pro and current Normal and print shared candidate losses"]
fn smart_automove_pro_fast_screen_shared_loss_probe_vs_current() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let repeats = env_usize("SMART_PRO_FAST_SCREEN_REPEATS")
        .unwrap_or(2)
        .max(1);
    let games_per_repeat = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(84)
        .max(56);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(3).max(1);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let seed_tag = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "pro_fast_screen_shared_vs_current_v1".to_string());
    let budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let mut total_games = 0usize;
    let mut shared_losses = 0usize;
    let mut pro_only_losses = 0usize;
    let mut normal_only_losses = 0usize;

    eprintln!(
        "pro fast-screen shared-loss probe config: candidate_profile={} baseline_profile={} seed_tag={} repeats={} games_per_repeat={} max_plies={} trace_limit={} include_acceptance={}",
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeats,
        games_per_repeat,
        max_plies,
        trace_limit,
        include_acceptance,
    );

    for repeat_index in 0..repeats {
        let seed =
            seed_for_budget_duel_repeat_and_tag(budget, budget, repeat_index, seed_tag.as_str());
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in
                [("ab", candidate_white_ab), ("ba", !candidate_white_ab)]
            {
                total_games += 1;
                let (vs_pro_result, vs_pro_traces) = replay_cross_budget_loss_probe_game_with_options(
                    candidate_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    baseline_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    opening_fen.as_str(),
                    candidate_is_white,
                    max_plies,
                    trace_limit,
                    include_acceptance,
                );
                let (vs_normal_result, vs_normal_traces) =
                    replay_cross_budget_loss_probe_game_with_options(
                        candidate_profile.as_str(),
                        SmartAutomovePreference::Pro,
                        baseline_profile.as_str(),
                        SmartAutomovePreference::Normal,
                        opening_fen.as_str(),
                        candidate_is_white,
                        max_plies,
                        trace_limit,
                        include_acceptance,
                    );
                let lost_vs_pro = vs_pro_result == MatchResult::OpponentWin;
                let lost_vs_normal = vs_normal_result == MatchResult::OpponentWin;

                match (lost_vs_pro, lost_vs_normal) {
                    (true, true) => {
                        shared_losses += 1;
                        eprintln!(
                            "PRO_FAST_SCREEN_SHARED_LOSS game={} repeat={} opening_index={} mirror={} candidate_is_white={} seed={} opening={}",
                            shared_losses,
                            repeat_index,
                            opening_index,
                            mirror,
                            candidate_is_white,
                            seed,
                            opening_fen
                        );
                        for trace in &vs_pro_traces {
                            eprintln!("  VS_PRO_TRACE ply={} fen={}", trace.ply, trace.fen);
                            print_loss_probe_decision("    candidate", &trace.candidate);
                            print_loss_probe_decision("    baseline", &trace.baseline);
                        }
                        for trace in &vs_normal_traces {
                            eprintln!("  VS_NORMAL_TRACE ply={} fen={}", trace.ply, trace.fen);
                            print_loss_probe_decision("    candidate", &trace.candidate);
                            print_loss_probe_decision("    baseline", &trace.baseline);
                        }
                    }
                    (true, false) => {
                        pro_only_losses += 1;
                    }
                    (false, true) => {
                        normal_only_losses += 1;
                    }
                    (false, false) => {}
                }
            }
        }
    }

    eprintln!(
        "pro fast-screen shared-loss probe summary: total_games={} shared_losses={} pro_only_losses={} normal_only_losses={}",
        total_games,
        shared_losses,
        pro_only_losses,
        normal_only_losses,
    );
}

#[test]
#[ignore = "diagnostic: classify same-opening shared losses into wrapper-owned vs direct-runtime internal exacts"]
fn smart_automove_pro_fast_screen_shared_loss_direct_runtime_probe_vs_current() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let repeats = env_usize("SMART_PRO_FAST_SCREEN_REPEATS")
        .unwrap_or(2)
        .max(1);
    let games_per_repeat = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(84)
        .max(56);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(3).max(1);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let shared_loss_limit = env_usize("SMART_PROBE_SHARED_LOSS_LIMIT");
    let repeat_filter = env_usize("SMART_PRO_FAST_SCREEN_REPEAT_INDEX");
    let opening_filter = env_usize("SMART_PRO_FAST_SCREEN_OPENING_INDEX");
    let mirror_filter = env::var("SMART_PRO_FAST_SCREEN_MIRROR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let seed_tag = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "pro_fast_screen_shared_vs_current_v1".to_string());
    let budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let mut total_games = 0usize;
    let mut shared_losses = 0usize;
    let mut shared_losses_with_only_wrapper_owned_exacts = 0usize;
    let mut shared_losses_with_internal_profile_exacts = 0usize;
    let mut total_shared_exacts = 0usize;
    let mut wrapper_owned_exacts = 0usize;
    let mut profile_internal_exacts = 0usize;
    let mut direct_matches_pro = 0usize;
    let mut direct_matches_normal = 0usize;
    let mut direct_matches_both = 0usize;
    let mut direct_matches_neither = 0usize;

    eprintln!(
        "pro fast-screen shared-loss direct-runtime probe config: candidate_profile={} baseline_profile={} seed_tag={} repeats={} games_per_repeat={} max_plies={} trace_limit={} include_acceptance={}",
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeats,
        games_per_repeat,
        max_plies,
        trace_limit,
        include_acceptance,
    );

    'repeat_loop: for repeat_index in 0..repeats {
        if repeat_filter.is_some_and(|expected| expected != repeat_index) {
            continue;
        }
        let seed =
            seed_for_budget_duel_repeat_and_tag(budget, budget, repeat_index, seed_tag.as_str());
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            if opening_filter.is_some_and(|expected| expected != opening_index) {
                continue;
            }
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in
                [("ab", candidate_white_ab), ("ba", !candidate_white_ab)]
            {
                if mirror_filter
                    .as_deref()
                    .is_some_and(|expected| expected != mirror)
                {
                    continue;
                }
                total_games += 1;
                let (vs_pro_result, vs_pro_traces) = replay_cross_budget_loss_probe_game_with_options(
                    candidate_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    baseline_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    opening_fen.as_str(),
                    candidate_is_white,
                    max_plies,
                    trace_limit,
                    include_acceptance,
                );
                let (vs_normal_result, vs_normal_traces) =
                    replay_cross_budget_loss_probe_game_with_options(
                        candidate_profile.as_str(),
                        SmartAutomovePreference::Pro,
                        baseline_profile.as_str(),
                        SmartAutomovePreference::Normal,
                        opening_fen.as_str(),
                        candidate_is_white,
                        max_plies,
                        trace_limit,
                        include_acceptance,
                    );
                let lost_vs_pro = vs_pro_result == MatchResult::OpponentWin;
                let lost_vs_normal = vs_normal_result == MatchResult::OpponentWin;
                if !(lost_vs_pro && lost_vs_normal) {
                    continue;
                }

                shared_losses += 1;
                eprintln!(
                    "PRO_FAST_SCREEN_SHARED_DIRECT_RUNTIME game={} repeat={} opening_index={} mirror={} candidate_is_white={} seed={} opening={}",
                    shared_losses,
                    repeat_index,
                    opening_index,
                    mirror,
                    candidate_is_white,
                    seed,
                    opening_fen
                );

                let mut per_fen =
                    std::collections::BTreeMap::<String, (Option<(String, String)>, Option<(String, String)>)>::new();
                for trace in &vs_pro_traces {
                    per_fen.insert(
                        trace.fen.clone(),
                        (
                            Some((trace.candidate.move_fen.clone(), trace.baseline.move_fen.clone())),
                            None,
                        ),
                    );
                }
                for trace in &vs_normal_traces {
                    per_fen
                        .entry(trace.fen.clone())
                        .and_modify(|entry| {
                            entry.1 = Some((
                                trace.candidate.move_fen.clone(),
                                trace.baseline.move_fen.clone(),
                            ));
                        })
                        .or_insert((
                            None,
                            Some((trace.candidate.move_fen.clone(), trace.baseline.move_fen.clone())),
                        ));
                }

                let mut opening_has_internal_profile_exact = false;
                for (fen, (vs_pro_moves, vs_normal_moves)) in per_fen {
                    total_shared_exacts += 1;
                    let profile_move = vs_pro_moves
                        .as_ref()
                        .map(|(candidate, _)| candidate.as_str())
                        .or_else(|| {
                            vs_normal_moves
                                .as_ref()
                                .map(|(candidate, _)| candidate.as_str())
                        })
                        .expect("shared exact should include at least one candidate move");
                    let baseline_pro_move =
                        vs_pro_moves.as_ref().map(|(_, baseline)| baseline.as_str());
                    let baseline_normal_move = vs_normal_moves
                        .as_ref()
                        .map(|(_, baseline)| baseline.as_str());
                    let game = MonsGame::from_fen(fen.as_str(), false)
                        .expect("shared-loss exact fen should be valid");
                    let direct = loss_probe_direct_runtime_decision_with_options(
                        candidate_profile.as_str(),
                        SmartAutomovePreference::Pro,
                        &game,
                        include_acceptance,
                    );
                    let wrapper_owned = direct.move_fen != profile_move;
                    if wrapper_owned {
                        wrapper_owned_exacts += 1;
                    } else {
                        profile_internal_exacts += 1;
                        opening_has_internal_profile_exact = true;
                    }

                    let direct_match_label = match (baseline_pro_move, baseline_normal_move) {
                        (Some(pro_move), Some(normal_move))
                            if direct.move_fen == pro_move && direct.move_fen == normal_move =>
                        {
                            direct_matches_both += 1;
                            "both"
                        }
                        (Some(pro_move), _) if direct.move_fen == pro_move => {
                            direct_matches_pro += 1;
                            "pro"
                        }
                        (_, Some(normal_move)) if direct.move_fen == normal_move => {
                            direct_matches_normal += 1;
                            "normal"
                        }
                        _ => {
                            direct_matches_neither += 1;
                            "neither"
                        }
                    };

                    eprintln!(
                        "  EXACT fen={} profile_move={} direct_move={} wrapper_owned={} direct_matches={} baseline_pro={:?} baseline_normal={:?}",
                        fen,
                        profile_move,
                        direct.move_fen,
                        wrapper_owned,
                        direct_match_label,
                        baseline_pro_move,
                        baseline_normal_move,
                    );
                }

                if opening_has_internal_profile_exact {
                    shared_losses_with_internal_profile_exacts += 1;
                } else {
                    shared_losses_with_only_wrapper_owned_exacts += 1;
                }

                if shared_loss_limit.is_some_and(|limit| shared_losses >= limit.max(1)) {
                    break 'repeat_loop;
                }
            }
        }
    }

    eprintln!(
        "pro fast-screen shared-loss direct-runtime probe summary: total_games={} shared_losses={} shared_losses_with_only_wrapper_owned_exacts={} shared_losses_with_internal_profile_exacts={} total_shared_exacts={} wrapper_owned_exacts={} profile_internal_exacts={} direct_matches_pro={} direct_matches_normal={} direct_matches_both={} direct_matches_neither={}",
        total_games,
        shared_losses,
        shared_losses_with_only_wrapper_owned_exacts,
        shared_losses_with_internal_profile_exacts,
        total_shared_exacts,
        wrapper_owned_exacts,
        profile_internal_exacts,
        direct_matches_pro,
        direct_matches_normal,
        direct_matches_both,
        direct_matches_neither,
    );
}

#[test]
#[ignore = "diagnostic: find exact shared baseline targets across current Pro and current Normal on shared-loss openings"]
fn smart_automove_pro_fast_screen_shared_exact_probe_vs_current() {
    #[derive(Clone)]
    struct SharedExactSide {
        profile_move: String,
        baseline_move: String,
        profile_stage: &'static str,
        turn_status: String,
        candidate_family: String,
    }

    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let repeats = env_usize("SMART_PRO_FAST_SCREEN_REPEATS")
        .unwrap_or(2)
        .max(1);
    let games_per_repeat = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(84)
        .max(56);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(3).max(1);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let shared_exact_limit = env_usize("SMART_PROBE_SHARED_EXACT_LIMIT");
    let repeat_filter = env_usize("SMART_PRO_FAST_SCREEN_REPEAT_INDEX");
    let opening_filter = env_usize("SMART_PRO_FAST_SCREEN_OPENING_INDEX");
    let mirror_filter = env::var("SMART_PRO_FAST_SCREEN_MIRROR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let seed_tag = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "pro_fast_screen_shared_vs_current_v1".to_string());
    let budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let mut total_games = 0usize;
    let mut shared_loss_games = 0usize;
    let mut shared_exact_hits = 0usize;
    let mut shared_exact_internal_hits = 0usize;
    let mut direct_matches_shared_baseline = 0usize;
    let mut fen_hit_counts = std::collections::BTreeMap::<String, usize>::new();
    let mut stage_counts = std::collections::BTreeMap::<String, usize>::new();

    eprintln!(
        "pro fast-screen shared-exact probe config: candidate_profile={} baseline_profile={} seed_tag={} repeats={} games_per_repeat={} max_plies={} trace_limit={} include_acceptance={}",
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeats,
        games_per_repeat,
        max_plies,
        trace_limit,
        include_acceptance,
    );

    'repeat_loop: for repeat_index in 0..repeats {
        if repeat_filter.is_some_and(|expected| expected != repeat_index) {
            continue;
        }
        let seed =
            seed_for_budget_duel_repeat_and_tag(budget, budget, repeat_index, seed_tag.as_str());
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            if opening_filter.is_some_and(|expected| expected != opening_index) {
                continue;
            }
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in
                [("ab", candidate_white_ab), ("ba", !candidate_white_ab)]
            {
                if mirror_filter
                    .as_deref()
                    .is_some_and(|expected| expected != mirror)
                {
                    continue;
                }
                total_games += 1;
                let (vs_pro_result, vs_pro_traces) = replay_cross_budget_loss_probe_game_with_options(
                    candidate_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    baseline_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    opening_fen.as_str(),
                    candidate_is_white,
                    max_plies,
                    trace_limit,
                    include_acceptance,
                );
                let (vs_normal_result, vs_normal_traces) =
                    replay_cross_budget_loss_probe_game_with_options(
                        candidate_profile.as_str(),
                        SmartAutomovePreference::Pro,
                        baseline_profile.as_str(),
                        SmartAutomovePreference::Normal,
                        opening_fen.as_str(),
                        candidate_is_white,
                        max_plies,
                        trace_limit,
                        include_acceptance,
                    );
                let lost_vs_pro = vs_pro_result == MatchResult::OpponentWin;
                let lost_vs_normal = vs_normal_result == MatchResult::OpponentWin;
                if !(lost_vs_pro && lost_vs_normal) {
                    continue;
                }

                shared_loss_games += 1;
                let mut per_fen = std::collections::BTreeMap::<
                    String,
                    (Option<SharedExactSide>, Option<SharedExactSide>),
                >::new();

                for trace in &vs_pro_traces {
                    let turn_status = trace
                        .candidate
                        .turn_engine
                        .as_ref()
                        .map(|engine| format!("{:?}", engine.status))
                        .unwrap_or_else(|| "None".to_string());
                    let candidate_family = trace
                        .candidate
                        .turn_engine
                        .as_ref()
                        .and_then(|engine| engine.candidate_family)
                        .map(|family| format!("{:?}", family))
                        .unwrap_or_else(|| "None".to_string());
                    per_fen.insert(
                        trace.fen.clone(),
                        (
                            Some(SharedExactSide {
                                profile_move: trace.candidate.move_fen.clone(),
                                baseline_move: trace.baseline.move_fen.clone(),
                                profile_stage: trace.candidate.selector_last_stage,
                                turn_status,
                                candidate_family,
                            }),
                            None,
                        ),
                    );
                }
                for trace in &vs_normal_traces {
                    let turn_status = trace
                        .candidate
                        .turn_engine
                        .as_ref()
                        .map(|engine| format!("{:?}", engine.status))
                        .unwrap_or_else(|| "None".to_string());
                    let candidate_family = trace
                        .candidate
                        .turn_engine
                        .as_ref()
                        .and_then(|engine| engine.candidate_family)
                        .map(|family| format!("{:?}", family))
                        .unwrap_or_else(|| "None".to_string());
                    per_fen
                        .entry(trace.fen.clone())
                        .and_modify(|entry| {
                            entry.1 = Some(SharedExactSide {
                                profile_move: trace.candidate.move_fen.clone(),
                                baseline_move: trace.baseline.move_fen.clone(),
                                profile_stage: trace.candidate.selector_last_stage,
                                turn_status: turn_status.clone(),
                                candidate_family: candidate_family.clone(),
                            });
                        })
                        .or_insert((
                            None,
                            Some(SharedExactSide {
                                profile_move: trace.candidate.move_fen.clone(),
                                baseline_move: trace.baseline.move_fen.clone(),
                                profile_stage: trace.candidate.selector_last_stage,
                                turn_status,
                                candidate_family,
                            }),
                        ));
                }

                for (fen, (pro_side, normal_side)) in per_fen {
                    let (Some(pro_side), Some(normal_side)) = (pro_side, normal_side) else {
                        continue;
                    };
                    if pro_side.baseline_move != normal_side.baseline_move {
                        continue;
                    }
                    if pro_side.profile_move != normal_side.profile_move {
                        eprintln!(
                            "SHARED_EXACT_CANDIDATE_MISMATCH repeat={} opening_index={} mirror={} fen={} profile_pro={} profile_normal={} baseline_shared={}",
                            repeat_index,
                            opening_index,
                            mirror,
                            fen,
                            pro_side.profile_move,
                            normal_side.profile_move,
                            pro_side.baseline_move,
                        );
                        continue;
                    }
                    if pro_side.profile_move == pro_side.baseline_move {
                        continue;
                    }

                    shared_exact_hits += 1;
                    *fen_hit_counts.entry(fen.clone()).or_default() += 1;

                    let game = MonsGame::from_fen(fen.as_str(), false)
                        .expect("shared exact fen should be valid");
                    let direct = loss_probe_direct_runtime_decision_with_options(
                        candidate_profile.as_str(),
                        SmartAutomovePreference::Pro,
                        &game,
                        include_acceptance,
                    );
                    let wrapper_owned = direct.move_fen != pro_side.profile_move;
                    if !wrapper_owned {
                        shared_exact_internal_hits += 1;
                    }
                    if direct.move_fen == pro_side.baseline_move {
                        direct_matches_shared_baseline += 1;
                    }

                    let stage_key = format!(
                        "profile_stage={} direct_stage={} wrapper_owned={} turn_status={} candidate_family={}",
                        pro_side.profile_stage,
                        direct.selector_last_stage,
                        wrapper_owned,
                        pro_side.turn_status,
                        pro_side.candidate_family,
                    );
                    *stage_counts.entry(stage_key.clone()).or_default() += 1;

                    eprintln!(
                        "SHARED_EXACT hit={} repeat={} opening_index={} mirror={} fen={} profile_move={} baseline_move={} direct_move={} wrapper_owned={} profile_stage={} direct_stage={} turn_status={} candidate_family={}",
                        shared_exact_hits,
                        repeat_index,
                        opening_index,
                        mirror,
                        fen,
                        pro_side.profile_move,
                        pro_side.baseline_move,
                        direct.move_fen,
                        wrapper_owned,
                        pro_side.profile_stage,
                        direct.selector_last_stage,
                        pro_side.turn_status,
                        pro_side.candidate_family,
                    );

                    if shared_exact_limit
                        .is_some_and(|limit| shared_exact_hits >= limit.max(1))
                    {
                        break 'repeat_loop;
                    }
                }
            }
        }
    }

    let repeated_exact_fens = fen_hit_counts
        .values()
        .filter(|&&hits| hits > 1)
        .count();

    eprintln!(
        "pro fast-screen shared-exact probe summary: total_games={} shared_loss_games={} shared_exact_hits={} shared_exact_internal_hits={} direct_matches_shared_baseline={} unique_shared_exact_fens={} repeated_shared_exact_fens={}",
        total_games,
        shared_loss_games,
        shared_exact_hits,
        shared_exact_internal_hits,
        direct_matches_shared_baseline,
        fen_hit_counts.len(),
        repeated_exact_fens,
    );
    for (stage_key, count) in stage_counts {
        eprintln!("  SHARED_EXACT_SURFACE count={} {}", count, stage_key);
    }
}

#[test]
#[ignore = "diagnostic: find cross-opening exact FENs where current Pro and current Normal agree against candidate"]
fn smart_automove_pro_fast_screen_cross_opening_shared_exact_probe_vs_current() {
    #[derive(Clone)]
    struct SharedExactSurfaceEntry {
        profile_move: String,
        baseline_move: String,
        profile_stage: &'static str,
        turn_status: String,
        candidate_family: String,
        repeat_index: usize,
        opening_index: usize,
        mirror: &'static str,
    }

    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let repeats = env_usize("SMART_PRO_FAST_SCREEN_REPEATS")
        .unwrap_or(2)
        .max(1);
    let games_per_repeat = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(84)
        .max(56);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(3).max(1);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let shared_exact_limit = env_usize("SMART_PROBE_SHARED_EXACT_LIMIT");
    let seed_tag_pro = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG_PRO")
        .or_else(|| env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG"))
        .unwrap_or_else(|| "pro_fast_screen_vs_pro_v1".to_string());
    let seed_tag_normal = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG_NORMAL")
        .or_else(|| env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG"))
        .unwrap_or_else(|| "pro_fast_screen_vs_normal_v1".to_string());
    let budget_pro = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let budget_normal = SearchBudget::from_preference(SmartAutomovePreference::Normal);
    let mut total_vs_pro_games = 0usize;
    let mut total_vs_normal_games = 0usize;
    let mut vs_pro_loss_games = 0usize;
    let mut vs_normal_loss_games = 0usize;
    let mut pro_entries =
        std::collections::BTreeMap::<String, Vec<SharedExactSurfaceEntry>>::new();
    let mut normal_entries =
        std::collections::BTreeMap::<String, Vec<SharedExactSurfaceEntry>>::new();

    eprintln!(
        "pro fast-screen cross-opening shared-exact probe config: candidate_profile={} baseline_profile={} seed_tag_pro={} seed_tag_normal={} repeats={} games_per_repeat={} max_plies={} trace_limit={} include_acceptance={}",
        candidate_profile,
        baseline_profile,
        seed_tag_pro,
        seed_tag_normal,
        repeats,
        games_per_repeat,
        max_plies,
        trace_limit,
        include_acceptance,
    );

    for repeat_index in 0..repeats {
        let seed = seed_for_budget_duel_repeat_and_tag(
            budget_pro,
            budget_pro,
            repeat_index,
            seed_tag_pro.as_str(),
        );
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in [("ab", candidate_white_ab), ("ba", !candidate_white_ab)] {
                total_vs_pro_games += 1;
                let (vs_pro_result, vs_pro_traces) = replay_cross_budget_loss_probe_game_with_options(
                    candidate_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    baseline_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    opening_fen.as_str(),
                    candidate_is_white,
                    max_plies,
                    trace_limit,
                    include_acceptance,
                );
                if vs_pro_result == MatchResult::OpponentWin {
                    vs_pro_loss_games += 1;
                    for trace in &vs_pro_traces {
                        let turn_status = trace
                            .candidate
                            .turn_engine
                            .as_ref()
                            .map(|engine| format!("{:?}", engine.status))
                            .unwrap_or_else(|| "None".to_string());
                        let candidate_family = trace
                            .candidate
                            .turn_engine
                            .as_ref()
                            .and_then(|engine| engine.candidate_family)
                            .map(|family| format!("{:?}", family))
                            .unwrap_or_else(|| "None".to_string());
                        pro_entries
                            .entry(trace.fen.clone())
                            .or_default()
                            .push(SharedExactSurfaceEntry {
                                profile_move: trace.candidate.move_fen.clone(),
                                baseline_move: trace.baseline.move_fen.clone(),
                                profile_stage: trace.candidate.selector_last_stage,
                                turn_status,
                                candidate_family,
                                repeat_index,
                                opening_index,
                                mirror,
                            });
                    }
                }
            }
        }
    }

    for repeat_index in 0..repeats {
        let seed = seed_for_budget_duel_repeat_and_tag(
            budget_pro,
            budget_normal,
            repeat_index,
            seed_tag_normal.as_str(),
        );
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in [("ab", candidate_white_ab), ("ba", !candidate_white_ab)] {
                total_vs_normal_games += 1;
                let (vs_normal_result, vs_normal_traces) =
                    replay_cross_budget_loss_probe_game_with_options(
                        candidate_profile.as_str(),
                        SmartAutomovePreference::Pro,
                        baseline_profile.as_str(),
                        SmartAutomovePreference::Normal,
                        opening_fen.as_str(),
                        candidate_is_white,
                        max_plies,
                        trace_limit,
                        include_acceptance,
                    );
                if vs_normal_result == MatchResult::OpponentWin {
                    vs_normal_loss_games += 1;
                    for trace in &vs_normal_traces {
                        let turn_status = trace
                            .candidate
                            .turn_engine
                            .as_ref()
                            .map(|engine| format!("{:?}", engine.status))
                            .unwrap_or_else(|| "None".to_string());
                        let candidate_family = trace
                            .candidate
                            .turn_engine
                            .as_ref()
                            .and_then(|engine| engine.candidate_family)
                            .map(|family| format!("{:?}", family))
                            .unwrap_or_else(|| "None".to_string());
                        normal_entries
                            .entry(trace.fen.clone())
                            .or_default()
                            .push(SharedExactSurfaceEntry {
                                profile_move: trace.candidate.move_fen.clone(),
                                baseline_move: trace.baseline.move_fen.clone(),
                                profile_stage: trace.candidate.selector_last_stage,
                                turn_status,
                                candidate_family,
                                repeat_index,
                                opening_index,
                                mirror,
                            });
                    }
                }
            }
        }
    }

    let mut shared_exact_hits = 0usize;
    let mut shared_exact_internal_hits = 0usize;
    let mut direct_matches_shared_baseline = 0usize;
    let mut repeated_shared_exact_fens = 0usize;
    let mut stage_counts = std::collections::BTreeMap::<String, usize>::new();

    for (fen, pro_sides) in &pro_entries {
        let Some(normal_sides) = normal_entries.get(fen) else {
            continue;
        };
        let mut hit_this_fen = 0usize;
        for pro_side in pro_sides {
            for normal_side in normal_sides {
                if pro_side.baseline_move != normal_side.baseline_move {
                    continue;
                }
                if pro_side.profile_move != normal_side.profile_move {
                    continue;
                }
                if pro_side.profile_move == pro_side.baseline_move {
                    continue;
                }

                shared_exact_hits += 1;
                hit_this_fen += 1;
                let game = MonsGame::from_fen(fen.as_str(), false)
                    .expect("cross-opening shared exact fen should be valid");
                let direct = loss_probe_direct_runtime_decision_with_options(
                    candidate_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    &game,
                    include_acceptance,
                );
                let wrapper_owned = direct.move_fen != pro_side.profile_move;
                if !wrapper_owned {
                    shared_exact_internal_hits += 1;
                }
                if direct.move_fen == pro_side.baseline_move {
                    direct_matches_shared_baseline += 1;
                }

                let stage_key = format!(
                    "profile_stage={} direct_stage={} wrapper_owned={} turn_status={} candidate_family={}",
                    pro_side.profile_stage,
                    direct.selector_last_stage,
                    wrapper_owned,
                    pro_side.turn_status,
                    pro_side.candidate_family,
                );
                *stage_counts.entry(stage_key.clone()).or_default() += 1;

                eprintln!(
                    "CROSS_OPENING_SHARED_EXACT hit={} fen={} profile_move={} baseline_move={} direct_move={} wrapper_owned={} profile_stage={} direct_stage={} turn_status={} candidate_family={} pro_loc=({},{},{}) normal_loc=({},{},{})",
                    shared_exact_hits,
                    fen,
                    pro_side.profile_move,
                    pro_side.baseline_move,
                    direct.move_fen,
                    wrapper_owned,
                    pro_side.profile_stage,
                    direct.selector_last_stage,
                    pro_side.turn_status,
                    pro_side.candidate_family,
                    pro_side.repeat_index,
                    pro_side.opening_index,
                    pro_side.mirror,
                    normal_side.repeat_index,
                    normal_side.opening_index,
                    normal_side.mirror,
                );

                if shared_exact_limit
                    .is_some_and(|limit| shared_exact_hits >= limit.max(1))
                {
                    break;
                }
            }
            if shared_exact_limit
                .is_some_and(|limit| shared_exact_hits >= limit.max(1))
            {
                break;
            }
        }
        if hit_this_fen > 1 {
            repeated_shared_exact_fens += 1;
        }
        if shared_exact_limit
            .is_some_and(|limit| shared_exact_hits >= limit.max(1))
        {
            break;
        }
    }

    eprintln!(
        "pro fast-screen cross-opening shared-exact probe summary: total_vs_pro_games={} total_vs_normal_games={} vs_pro_loss_games={} vs_normal_loss_games={} shared_exact_hits={} shared_exact_internal_hits={} direct_matches_shared_baseline={} repeated_shared_exact_fens={}",
        total_vs_pro_games,
        total_vs_normal_games,
        vs_pro_loss_games,
        vs_normal_loss_games,
        shared_exact_hits,
        shared_exact_internal_hits,
        direct_matches_shared_baseline,
        repeated_shared_exact_fens,
    );
    for (stage_key, count) in stage_counts {
        eprintln!("  CROSS_OPENING_SHARED_EXACT_SURFACE count={} {}", count, stage_key);
    }
}

#[test]
#[ignore = "diagnostic: aggregate shared selector/planner families across retained vs-pro and vs-normal duel losses"]
fn smart_automove_pro_fast_screen_shared_family_probe_vs_current() {
    #[derive(Default)]
    struct SharedFamilySideStats {
        exacts: usize,
        internal_exacts: usize,
        wrapper_owned_exacts: usize,
        direct_matches_baseline: usize,
        head_selected_true: usize,
        head_selected_false: usize,
        head_selected_none: usize,
        accepted_true: usize,
        accepted_false: usize,
        accepted_none: usize,
        unique_fens: std::collections::BTreeSet<String>,
        samples: Vec<String>,
    }

    #[derive(Default)]
    struct SharedFamilySurfaceStats {
        vs_pro: SharedFamilySideStats,
        vs_normal: SharedFamilySideStats,
    }

    fn build_shared_family_trace_stats(
        candidate_profile: &str,
        trace: &ReliabilityLossProbeTrace,
        repeat_index: usize,
        opening_index: usize,
        mirror: &str,
        include_acceptance: bool,
        surface_key_mode: &str,
    ) -> (String, SharedFamilySideStats) {
        let turn_status = trace
            .candidate
            .turn_engine
            .as_ref()
            .map(|engine| format!("{:?}", engine.status))
            .unwrap_or_else(|| "None".to_string());
        let candidate_family = trace
            .candidate
            .turn_engine
            .as_ref()
            .and_then(|engine| engine.candidate_family)
            .map(|family| format!("{:?}", family))
            .unwrap_or_else(|| "None".to_string());
        let head_selected = trace
            .candidate
            .turn_engine
            .as_ref()
            .and_then(|engine| engine.candidate_move_fen.as_ref())
            .map(|move_fen| move_fen == &trace.candidate.move_fen);
        let accepted_after_search = trace
            .candidate
            .turn_engine
            .as_ref()
            .and_then(|engine| engine.accepted_after_search);
        let game = MonsGame::from_fen(trace.fen.as_str(), false)
            .expect("shared family probe fen should be valid");
        let direct = loss_probe_direct_runtime_decision_with_options(
            candidate_profile,
            SmartAutomovePreference::Pro,
            &game,
            include_acceptance,
        );
        let wrapper_owned = direct.move_fen != trace.candidate.move_fen;
        let move_len = trace.candidate.inputs.len();

        let mut stats = SharedFamilySideStats {
            exacts: 1,
            ..SharedFamilySideStats::default()
        };
        stats.unique_fens.insert(trace.fen.clone());
        if wrapper_owned {
            stats.wrapper_owned_exacts = 1;
        } else {
            stats.internal_exacts = 1;
        }
        if direct.move_fen == trace.baseline.move_fen {
            stats.direct_matches_baseline = 1;
        }
        match head_selected {
            Some(true) => stats.head_selected_true = 1,
            Some(false) => stats.head_selected_false = 1,
            None => stats.head_selected_none = 1,
        }
        match accepted_after_search {
            Some(true) => stats.accepted_true = 1,
            Some(false) => stats.accepted_false = 1,
            None => stats.accepted_none = 1,
        }
        stats.samples.push(format!(
            "repeat={} opening_index={} mirror={} ply={} fen={} profile_move={} baseline_move={} direct_move={} wrapper_owned={} direct_stage={} head_selected={:?} accepted_after_search={:?}",
            repeat_index,
            opening_index,
            mirror,
            trace.ply,
            trace.fen,
            trace.candidate.move_fen,
            trace.baseline.move_fen,
            direct.move_fen,
            wrapper_owned,
            direct.selector_last_stage,
            head_selected,
            accepted_after_search,
        ));

        let surface_key = match surface_key_mode {
            "aligned" => format!(
                "profile_stage={} direct_stage={} turn_status={} candidate_family={} active_color={:?} turn={} mons_moves={} can_action={} can_move_mana={} move_len={} wrapper_owned={} head_selected={:?} accepted_after_search={:?}",
                trace.candidate.selector_last_stage,
                direct.selector_last_stage,
                turn_status,
                candidate_family,
                game.active_color,
                game.turn_number,
                game.mons_moves_count,
                game.player_can_use_action(),
                game.player_can_move_mana(),
                move_len,
                wrapper_owned,
                head_selected,
                accepted_after_search,
            ),
            _ => format!(
                "profile_stage={} turn_status={} candidate_family={}",
                trace.candidate.selector_last_stage,
                turn_status,
                candidate_family,
            ),
        };

        (surface_key, stats)
    }

    fn merge_shared_family_side_stats(
        dest: &mut SharedFamilySideStats,
        src: SharedFamilySideStats,
        sample_limit: usize,
    ) {
        dest.exacts += src.exacts;
        dest.internal_exacts += src.internal_exacts;
        dest.wrapper_owned_exacts += src.wrapper_owned_exacts;
        dest.direct_matches_baseline += src.direct_matches_baseline;
        dest.head_selected_true += src.head_selected_true;
        dest.head_selected_false += src.head_selected_false;
        dest.head_selected_none += src.head_selected_none;
        dest.accepted_true += src.accepted_true;
        dest.accepted_false += src.accepted_false;
        dest.accepted_none += src.accepted_none;
        dest.unique_fens.extend(src.unique_fens);
        for sample in src.samples {
            if dest.samples.len() >= sample_limit {
                break;
            }
            dest.samples.push(sample);
        }
    }

    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let repeats = env_usize("SMART_PRO_FAST_SCREEN_REPEATS")
        .unwrap_or(2)
        .max(1);
    let games_per_repeat = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(84)
        .max(56);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(3).max(1);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let sample_limit = env_usize("SMART_PROBE_SHARED_SURFACE_SAMPLE_LIMIT")
        .unwrap_or(2)
        .max(1);
    let surface_key_mode = env::var("SMART_PROBE_SHARED_FAMILY_KEY_MODE")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "family".to_string());
    let seed_tag_pro = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG_PRO")
        .or_else(|| env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG"))
        .unwrap_or_else(|| "pro_fast_screen_vs_pro_v1".to_string());
    let seed_tag_normal = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG_NORMAL")
        .or_else(|| env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG"))
        .unwrap_or_else(|| "pro_fast_screen_vs_normal_v1".to_string());
    let budget_pro = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let budget_normal = SearchBudget::from_preference(SmartAutomovePreference::Normal);
    let mut total_vs_pro_games = 0usize;
    let mut total_vs_normal_games = 0usize;
    let mut vs_pro_loss_games = 0usize;
    let mut vs_normal_loss_games = 0usize;
    let mut surface_stats =
        std::collections::BTreeMap::<String, SharedFamilySurfaceStats>::new();

    eprintln!(
        "pro fast-screen shared-family probe config: candidate_profile={} baseline_profile={} seed_tag_pro={} seed_tag_normal={} repeats={} games_per_repeat={} max_plies={} trace_limit={} include_acceptance={} sample_limit={} key_mode={}",
        candidate_profile,
        baseline_profile,
        seed_tag_pro,
        seed_tag_normal,
        repeats,
        games_per_repeat,
        max_plies,
        trace_limit,
        include_acceptance,
        sample_limit,
        surface_key_mode,
    );

    for repeat_index in 0..repeats {
        let seed = seed_for_budget_duel_repeat_and_tag(
            budget_pro,
            budget_pro,
            repeat_index,
            seed_tag_pro.as_str(),
        );
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in [("ab", candidate_white_ab), ("ba", !candidate_white_ab)] {
                total_vs_pro_games += 1;
                let (result, traces) = replay_cross_budget_loss_probe_game_with_options(
                    candidate_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    baseline_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    opening_fen.as_str(),
                    candidate_is_white,
                    max_plies,
                    trace_limit,
                    include_acceptance,
                );
                if result != MatchResult::OpponentWin {
                    continue;
                }
                vs_pro_loss_games += 1;
                for trace in &traces {
                    let (surface_key, side_stats) = build_shared_family_trace_stats(
                        candidate_profile.as_str(),
                        trace,
                        repeat_index,
                        opening_index,
                        mirror,
                        include_acceptance,
                        surface_key_mode.as_str(),
                    );
                    let entry = surface_stats.entry(surface_key).or_default();
                    merge_shared_family_side_stats(&mut entry.vs_pro, side_stats, sample_limit);
                }
            }
        }
    }

    for repeat_index in 0..repeats {
        let seed = seed_for_budget_duel_repeat_and_tag(
            budget_pro,
            budget_normal,
            repeat_index,
            seed_tag_normal.as_str(),
        );
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in [("ab", candidate_white_ab), ("ba", !candidate_white_ab)] {
                total_vs_normal_games += 1;
                let (result, traces) = replay_cross_budget_loss_probe_game_with_options(
                    candidate_profile.as_str(),
                    SmartAutomovePreference::Pro,
                    baseline_profile.as_str(),
                    SmartAutomovePreference::Normal,
                    opening_fen.as_str(),
                    candidate_is_white,
                    max_plies,
                    trace_limit,
                    include_acceptance,
                );
                if result != MatchResult::OpponentWin {
                    continue;
                }
                vs_normal_loss_games += 1;
                for trace in &traces {
                    let (surface_key, side_stats) = build_shared_family_trace_stats(
                        candidate_profile.as_str(),
                        trace,
                        repeat_index,
                        opening_index,
                        mirror,
                        include_acceptance,
                        surface_key_mode.as_str(),
                    );
                    let entry = surface_stats.entry(surface_key).or_default();
                    merge_shared_family_side_stats(&mut entry.vs_normal, side_stats, sample_limit);
                }
            }
        }
    }

    let mut shared_surfaces = 0usize;
    let mut shared_surfaces_with_internal_on_both = 0usize;
    let mut shared_surfaces_wrapper_only = 0usize;
    let mut shared_surface_exact_hits = 0usize;
    let mut shared_surface_internal_hits = 0usize;

    for (surface_key, stats) in &surface_stats {
        if stats.vs_pro.exacts == 0 || stats.vs_normal.exacts == 0 {
            continue;
        }
        shared_surfaces += 1;
        shared_surface_exact_hits += stats.vs_pro.exacts + stats.vs_normal.exacts;
        shared_surface_internal_hits +=
            stats.vs_pro.internal_exacts + stats.vs_normal.internal_exacts;
        if stats.vs_pro.internal_exacts > 0 && stats.vs_normal.internal_exacts > 0 {
            shared_surfaces_with_internal_on_both += 1;
        }
        if stats.vs_pro.internal_exacts == 0 && stats.vs_normal.internal_exacts == 0 {
            shared_surfaces_wrapper_only += 1;
        }

        eprintln!(
            "SHARED_FAMILY count_pro={} count_normal={} internal_pro={} internal_normal={} wrapper_pro={} wrapper_normal={} direct_matches_baseline_pro={} direct_matches_baseline_normal={} unique_fens_pro={} unique_fens_normal={} head_selected_pro=[t:{} f:{} n:{}] head_selected_normal=[t:{} f:{} n:{}] accepted_pro=[t:{} f:{} n:{}] accepted_normal=[t:{} f:{} n:{}] {}",
            stats.vs_pro.exacts,
            stats.vs_normal.exacts,
            stats.vs_pro.internal_exacts,
            stats.vs_normal.internal_exacts,
            stats.vs_pro.wrapper_owned_exacts,
            stats.vs_normal.wrapper_owned_exacts,
            stats.vs_pro.direct_matches_baseline,
            stats.vs_normal.direct_matches_baseline,
            stats.vs_pro.unique_fens.len(),
            stats.vs_normal.unique_fens.len(),
            stats.vs_pro.head_selected_true,
            stats.vs_pro.head_selected_false,
            stats.vs_pro.head_selected_none,
            stats.vs_normal.head_selected_true,
            stats.vs_normal.head_selected_false,
            stats.vs_normal.head_selected_none,
            stats.vs_pro.accepted_true,
            stats.vs_pro.accepted_false,
            stats.vs_pro.accepted_none,
            stats.vs_normal.accepted_true,
            stats.vs_normal.accepted_false,
            stats.vs_normal.accepted_none,
            surface_key,
        );
        for sample in &stats.vs_pro.samples {
            eprintln!("  SHARED_FAMILY_PRO_SAMPLE {}", sample);
        }
        for sample in &stats.vs_normal.samples {
            eprintln!("  SHARED_FAMILY_NORMAL_SAMPLE {}", sample);
        }
    }

    eprintln!(
        "pro fast-screen shared-family probe summary: key_mode={} total_vs_pro_games={} total_vs_normal_games={} vs_pro_loss_games={} vs_normal_loss_games={} shared_surfaces={} shared_surfaces_with_internal_on_both={} shared_surfaces_wrapper_only={} shared_surface_exact_hits={} shared_surface_internal_hits={}",
        surface_key_mode,
        total_vs_pro_games,
        total_vs_normal_games,
        vs_pro_loss_games,
        vs_normal_loss_games,
        shared_surfaces,
        shared_surfaces_with_internal_on_both,
        shared_surfaces_wrapper_only,
        shared_surface_exact_hits,
        shared_surface_internal_hits,
    );
}

#[test]
#[ignore = "diagnostic: inspect one pro fast-screen opening against the normal baseline"]
fn smart_automove_pro_fast_screen_opening_probe_vs_normal() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_release_safe_pre_exact".into());
    let repeat_index = env_usize("SMART_PRO_FAST_SCREEN_REPEAT_INDEX").unwrap_or(0);
    let opening_index = env_usize("SMART_PRO_FAST_SCREEN_OPENING_INDEX").unwrap_or(0);
    let games_per_repeat = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(84)
        .max(56);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(8).max(1);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let mirror_filter = env::var("SMART_PRO_FAST_SCREEN_MIRROR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let seed_tag = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "pro_fast_screen_vs_normal_v1".to_string());
    let budget_a = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let budget_b = SearchBudget::from_preference(SmartAutomovePreference::Normal);
    let seed =
        seed_for_budget_duel_repeat_and_tag(budget_a, budget_b, repeat_index, seed_tag.as_str());
    let opening_fens = generate_opening_fens_cached(seed, games_per_repeat.max(opening_index + 1));
    let opening_fen = opening_fens
        .get(opening_index)
        .unwrap_or_else(|| panic!("opening index {} unavailable", opening_index))
        .clone();
    let candidate_white_ab = opening_index % 2 == 0;

    eprintln!(
        "pro fast-screen opening probe config: candidate_profile={} baseline_profile={} seed_tag={} repeat_index={} opening_index={} games_per_repeat={} max_plies={} trace_limit={} include_acceptance={} mirror_filter={:?} opening={}",
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeat_index,
        opening_index,
        games_per_repeat,
        max_plies,
        trace_limit,
        include_acceptance,
        mirror_filter,
        opening_fen,
    );

    for (mirror, candidate_is_white) in [("ab", candidate_white_ab), ("ba", !candidate_white_ab)] {
        if mirror_filter
            .as_deref()
            .is_some_and(|expected| expected != mirror)
        {
            continue;
        }

        let (result, traces) = replay_cross_budget_loss_probe_game_with_options(
            candidate_profile.as_str(),
            SmartAutomovePreference::Pro,
            baseline_profile.as_str(),
            SmartAutomovePreference::Normal,
            opening_fen.as_str(),
            candidate_is_white,
            max_plies,
            trace_limit,
            include_acceptance,
        );
        let outcome = match result {
            MatchResult::CandidateWin => "W",
            MatchResult::OpponentWin => "L",
            MatchResult::Draw => "D",
        };
        eprintln!(
            "PRO_FAST_SCREEN_OPENING outcome={} repeat={} opening_index={} mirror={} candidate_is_white={} seed={} opening={}",
            outcome,
            repeat_index,
            opening_index,
            mirror,
            candidate_is_white,
            seed,
            opening_fen
        );
        for trace in traces {
            let engine = trace.candidate.turn_engine.as_ref();
            let direct = loss_probe_direct_runtime_decision_with_options(
                candidate_profile.as_str(),
                SmartAutomovePreference::Pro,
                &MonsGame::from_fen(trace.fen.as_str(), false)
                    .expect("fast-screen opening trace fen should be valid"),
                include_acceptance,
            );
            eprintln!(
                "PRO_FAST_SCREEN_OPENING_TRACE opening_index={} mirror={} candidate_is_white={} ply={} fen={} candidate_move={} direct_move={} baseline_move={} wrapper_owned={} direct_stage={} engine_head={:?} root_search_selected={:?} accepted_after_search={:?} selected_utility={:?} candidate_utility={:?}",
                opening_index,
                mirror,
                candidate_is_white,
                trace.ply,
                trace.fen,
                trace.candidate.move_fen,
                direct.move_fen,
                trace.baseline.move_fen,
                trace.candidate.move_fen != direct.move_fen,
                direct.selector_last_stage,
                engine.and_then(|engine| engine.candidate_move_fen.as_ref()),
                engine.and_then(|engine| engine.root_search_selected_move_fen.as_ref()),
                engine.and_then(|engine| engine.accepted_after_search),
                engine.and_then(|engine| engine.selected_utility),
                engine.and_then(|engine| engine.candidate_utility),
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: print every ply on one pro fast-screen opening against the normal baseline"]
fn smart_automove_pro_fast_screen_opening_full_trace_vs_normal() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_release_safe_pre_exact".into());
    let repeat_index = env_usize("SMART_PRO_FAST_SCREEN_REPEAT_INDEX").unwrap_or(0);
    let opening_index = env_usize("SMART_PRO_FAST_SCREEN_OPENING_INDEX").unwrap_or(0);
    let games_per_repeat = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(84)
        .max(56);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let mirror = env::var("SMART_PRO_FAST_SCREEN_MIRROR").unwrap_or_else(|_| "ab".into());
    let seed_tag = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "pro_fast_screen_vs_normal_v1".to_string());
    let budget_a = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let budget_b = SearchBudget::from_preference(SmartAutomovePreference::Normal);
    let seed =
        seed_for_budget_duel_repeat_and_tag(budget_a, budget_b, repeat_index, seed_tag.as_str());
    let opening_fens = generate_opening_fens_cached(seed, games_per_repeat.max(opening_index + 1));
    let opening_fen = opening_fens
        .get(opening_index)
        .unwrap_or_else(|| panic!("opening index {} unavailable", opening_index))
        .clone();
    let candidate_white_ab = opening_index % 2 == 0;
    let candidate_is_white = match mirror.as_str() {
        "ab" => candidate_white_ab,
        "ba" => !candidate_white_ab,
        _ => panic!("unsupported mirror '{}'", mirror),
    };
    let baseline_selector =
        profile_selector_from_name(baseline_profile.as_str()).unwrap_or_else(|| {
            panic!(
                "profile '{}' not found for fast-screen probe baseline side",
                baseline_profile
            )
        });
    let mut game = MonsGame::from_fen(opening_fen.as_str(), false).expect("valid opening fen");

    eprintln!(
        "PRO_FAST_SCREEN_FULL_TRACE_CONFIG candidate_profile={} baseline_profile={} seed_tag={} repeat_index={} opening_index={} mirror={} candidate_is_white={} max_plies={} opening={}",
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeat_index,
        opening_index,
        mirror,
        candidate_is_white,
        max_plies,
        opening_fen
    );

    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            eprintln!(
                "PRO_FAST_SCREEN_FULL_TRACE_END ply={} winner={:?} fen={}",
                ply,
                winner_color,
                game.fen()
            );
            return;
        }

        let candidate_to_move = if candidate_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };

        if candidate_to_move {
            let candidate = loss_probe_decision_with_options(
                candidate_profile.as_str(),
                SmartAutomovePreference::Pro,
                &game,
                include_acceptance,
            );
            let baseline = loss_probe_decision_with_options(
                baseline_profile.as_str(),
                SmartAutomovePreference::Normal,
                &game,
                include_acceptance,
            );
            let engine = candidate.turn_engine.as_ref();
            eprintln!(
                "PRO_FAST_SCREEN_FULL_TRACE ply={} side=candidate active={:?} fen={} candidate_move={} baseline_move={} engine_status={:?} cached_move={:?} engine_head={:?} root_search_selected={:?} accepted_after_search={:?}",
                ply,
                game.active_color,
                game.fen(),
                candidate.move_fen,
                baseline.move_fen,
                engine.map(|engine| engine.status),
                engine.and_then(|engine| engine.cached_move_fen.as_ref()),
                engine.and_then(|engine| engine.candidate_move_fen.as_ref()),
                engine.and_then(|engine| engine.root_search_selected_move_fen.as_ref()),
                engine.and_then(|engine| engine.accepted_after_search),
            );
            let inputs = candidate.inputs.clone();
            if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
                eprintln!(
                    "PRO_FAST_SCREEN_FULL_TRACE_END ply={} invalid_candidate_move",
                    ply
                );
                return;
            }
        } else {
            let config = loss_probe_runtime_config(
                baseline_profile.as_str(),
                &game,
                SmartAutomovePreference::Normal,
            );
            let inputs = select_inputs_with_runtime_fallback(baseline_selector, &game, config);
            eprintln!(
                "PRO_FAST_SCREEN_FULL_TRACE ply={} side=baseline active={:?} fen={} baseline_move={}",
                ply,
                game.active_color,
                game.fen(),
                Input::fen_from_array(&inputs)
            );
            if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
                eprintln!(
                    "PRO_FAST_SCREEN_FULL_TRACE_END ply={} invalid_baseline_move",
                    ply
                );
                return;
            }
        }
    }

    eprintln!(
        "PRO_FAST_SCREEN_FULL_TRACE_END ply={} adjudicated={:?} fen={}",
        max_plies,
        adjudicate_non_terminal_game(&game),
        game.fen()
    );
}

#[test]
#[ignore = "diagnostic: print every ply on one pro primary opening against the configured baseline mode"]
fn smart_automove_pro_primary_opening_full_trace() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_release_safe_pre_exact".into());
    let baseline_mode = match env::var("SMART_PRO_PRIMARY_BASELINE_MODE")
        .unwrap_or_else(|_| "normal".into())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "fast" => SmartAutomovePreference::Fast,
        _ => SmartAutomovePreference::Normal,
    };
    let repeat_index = env_usize("SMART_PRO_PRIMARY_REPEAT_INDEX").unwrap_or(0);
    let opening_index = env_usize("SMART_PRO_PRIMARY_OPENING_INDEX").unwrap_or(0);
    let games_per_repeat = env_usize("SMART_PRO_PRIMARY_GAMES").unwrap_or(1).max(1);
    let max_plies = env_usize("SMART_PRO_PRIMARY_MAX_PLIES")
        .unwrap_or(56)
        .max(56);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let mirror = env::var("SMART_PRO_PRIMARY_MIRROR").unwrap_or_else(|_| "ab".into());
    let seed_tag = env_profile_name("SMART_PRO_PRIMARY_SEED_TAG")
        .unwrap_or_else(|| "pro_primary_vs_normal:neutral_v1".to_string());
    let budget_a = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let budget_b = SearchBudget::from_preference(baseline_mode);
    let seed =
        seed_for_budget_duel_repeat_and_tag(budget_a, budget_b, repeat_index, seed_tag.as_str());
    let opening_fens = generate_opening_fens_cached(seed, games_per_repeat.max(opening_index + 1));
    let opening_fen = opening_fens
        .get(opening_index)
        .unwrap_or_else(|| panic!("opening index {} unavailable", opening_index))
        .clone();
    let candidate_white_ab = opening_index % 2 == 0;
    let candidate_is_white = match mirror.as_str() {
        "ab" => candidate_white_ab,
        "ba" => !candidate_white_ab,
        _ => panic!("unsupported mirror '{}'", mirror),
    };
    let baseline_selector =
        profile_selector_from_name(baseline_profile.as_str()).unwrap_or_else(|| {
            panic!(
                "profile '{}' not found for primary probe baseline side",
                baseline_profile
            )
        });
    let mut game = MonsGame::from_fen(opening_fen.as_str(), false).expect("valid opening fen");

    eprintln!(
        "PRO_PRIMARY_FULL_TRACE_CONFIG candidate_profile={} baseline_profile={} baseline_mode={:?} seed_tag={} repeat_index={} opening_index={} mirror={} candidate_is_white={} max_plies={} opening={}",
        candidate_profile,
        baseline_profile,
        baseline_mode,
        seed_tag,
        repeat_index,
        opening_index,
        mirror,
        candidate_is_white,
        max_plies,
        opening_fen
    );

    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            eprintln!(
                "PRO_PRIMARY_FULL_TRACE_END ply={} winner={:?} fen={}",
                ply,
                winner_color,
                game.fen()
            );
            return;
        }

        let candidate_to_move = if candidate_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };

        if candidate_to_move {
            let candidate = loss_probe_decision_with_options(
                candidate_profile.as_str(),
                SmartAutomovePreference::Pro,
                &game,
                include_acceptance,
            );
            let baseline = loss_probe_decision_with_options(
                baseline_profile.as_str(),
                baseline_mode,
                &game,
                include_acceptance,
            );
            let engine = candidate.turn_engine.as_ref();
            eprintln!(
                "PRO_PRIMARY_FULL_TRACE ply={} side=candidate active={:?} fen={} candidate_move={} baseline_move={} engine_status={:?} cached_move={:?} engine_head={:?} root_search_selected={:?} accepted_after_search={:?}",
                ply,
                game.active_color,
                game.fen(),
                candidate.move_fen,
                baseline.move_fen,
                engine.map(|engine| engine.status),
                engine.and_then(|engine| engine.cached_move_fen.as_ref()),
                engine.and_then(|engine| engine.candidate_move_fen.as_ref()),
                engine.and_then(|engine| engine.root_search_selected_move_fen.as_ref()),
                engine.and_then(|engine| engine.accepted_after_search),
            );
            let inputs = candidate.inputs.clone();
            if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
                eprintln!(
                    "PRO_PRIMARY_FULL_TRACE_END ply={} invalid_candidate_move",
                    ply
                );
                return;
            }
        } else {
            let config = loss_probe_runtime_config(baseline_profile.as_str(), &game, baseline_mode);
            let inputs = select_inputs_with_runtime_fallback(baseline_selector, &game, config);
            eprintln!(
                "PRO_PRIMARY_FULL_TRACE ply={} side=baseline active={:?} fen={} baseline_move={}",
                ply,
                game.active_color,
                game.fen(),
                Input::fen_from_array(&inputs)
            );
            if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
                eprintln!(
                    "PRO_PRIMARY_FULL_TRACE_END ply={} invalid_baseline_move",
                    ply
                );
                return;
            }
        }
    }

    eprintln!(
        "PRO_PRIMARY_FULL_TRACE_END ply={} adjudicated={:?} fen={}",
        max_plies,
        adjudicate_non_terminal_game(&game),
        game.fen()
    );
}

#[test]
#[ignore = "diagnostic: replay one pro primary opening and override one candidate ply while keeping live cache state"]
fn smart_automove_pro_primary_opening_sequence_override_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_release_safe_pre_exact".into());
    let baseline_mode = match env::var("SMART_PRO_PRIMARY_BASELINE_MODE")
        .unwrap_or_else(|_| "normal".into())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "fast" => SmartAutomovePreference::Fast,
        _ => SmartAutomovePreference::Normal,
    };
    let repeat_index = env_usize("SMART_PRO_PRIMARY_REPEAT_INDEX").unwrap_or(0);
    let opening_index = env_usize("SMART_PRO_PRIMARY_OPENING_INDEX").unwrap_or(0);
    let games_per_repeat = env_usize("SMART_PRO_PRIMARY_GAMES").unwrap_or(1).max(1);
    let max_plies = env_usize("SMART_PRO_PRIMARY_MAX_PLIES")
        .unwrap_or(56)
        .max(1);
    let mirror = env::var("SMART_PRO_PRIMARY_MIRROR").unwrap_or_else(|_| "ab".into());
    let seed_tag = env_profile_name("SMART_PRO_PRIMARY_SEED_TAG")
        .unwrap_or_else(|| "pro_primary_vs_normal:neutral_v1".to_string());
    let forced_ply = env_usize("SMART_PROBE_OVERRIDE_PLY")
        .expect("SMART_PROBE_OVERRIDE_PLY is required for the sequence override probe");
    let forced_move_fen = env::var("SMART_PROBE_FORCED_MOVE_FEN")
        .expect("SMART_PROBE_FORCED_MOVE_FEN is required for the sequence override probe");
    let forced_inputs = Input::array_from_fen(forced_move_fen.as_str());
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let budget_a = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let budget_b = SearchBudget::from_preference(baseline_mode);
    let seed =
        seed_for_budget_duel_repeat_and_tag(budget_a, budget_b, repeat_index, seed_tag.as_str());
    let opening_fens = generate_opening_fens_cached(seed, games_per_repeat.max(opening_index + 1));
    let opening_fen = opening_fens
        .get(opening_index)
        .unwrap_or_else(|| panic!("opening index {} unavailable", opening_index))
        .clone();
    let candidate_white_ab = opening_index % 2 == 0;
    let candidate_is_white = match mirror.as_str() {
        "ab" => candidate_white_ab,
        "ba" => !candidate_white_ab,
        _ => panic!("unsupported mirror '{}'", mirror),
    };
    let baseline_selector =
        profile_selector_from_name(baseline_profile.as_str()).unwrap_or_else(|| {
            panic!(
                "profile '{}' not found for primary override probe baseline side",
                baseline_profile
            )
        });
    clear_turn_engine_plan_cache();
    let mut game = MonsGame::from_fen(opening_fen.as_str(), false).expect("valid opening fen");

    eprintln!(
        "PRO_PRIMARY_OVERRIDE_CONFIG candidate_profile={} baseline_profile={} baseline_mode={:?} seed_tag={} repeat_index={} opening_index={} mirror={} candidate_is_white={} forced_ply={} forced_move={} max_plies={} opening={}",
        candidate_profile,
        baseline_profile,
        baseline_mode,
        seed_tag,
        repeat_index,
        opening_index,
        mirror,
        candidate_is_white,
        forced_ply,
        forced_move_fen,
        max_plies,
        opening_fen
    );

    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            eprintln!(
                "PRO_PRIMARY_OVERRIDE_END ply={} winner={:?} fen={}",
                ply,
                winner_color,
                game.fen()
            );
            return;
        }

        let candidate_to_move = if candidate_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };

        if candidate_to_move {
            let candidate = loss_probe_decision_with_options(
                candidate_profile.as_str(),
                SmartAutomovePreference::Pro,
                &game,
                include_acceptance,
            );
            let baseline = loss_probe_decision_with_options(
                baseline_profile.as_str(),
                baseline_mode,
                &game,
                include_acceptance,
            );
            let using_forced_move = ply == forced_ply;
            let inputs = if using_forced_move {
                forced_inputs.clone()
            } else {
                candidate.inputs.clone()
            };
            eprintln!(
                "PRO_PRIMARY_OVERRIDE ply={} active={:?} fen={} candidate_move={} baseline_move={} using_forced_move={} applied_move={}",
                ply,
                game.active_color,
                game.fen(),
                candidate.move_fen,
                baseline.move_fen,
                using_forced_move,
                Input::fen_from_array(&inputs),
            );
            if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
                eprintln!(
                    "PRO_PRIMARY_OVERRIDE_END ply={} invalid_candidate_move",
                    ply
                );
                return;
            }
        } else {
            let config = loss_probe_runtime_config(baseline_profile.as_str(), &game, baseline_mode);
            let inputs = select_inputs_with_runtime_fallback(baseline_selector, &game, config);
            if !matches!(
                game.process_input(inputs.clone(), false, false),
                Output::Events(_)
            ) {
                eprintln!("PRO_PRIMARY_OVERRIDE_END ply={} invalid_baseline_move", ply);
                return;
            }
        }
    }

    eprintln!(
        "PRO_PRIMARY_OVERRIDE_END ply={} adjudicated={:?} fen={}",
        max_plies,
        adjudicate_non_terminal_game(&game),
        game.fen()
    );
}

#[test]
#[ignore = "diagnostic: inspect one pro primary opening against the configured baseline mode"]
fn smart_automove_pro_primary_opening_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_release_safe_pre_exact".into());
    let baseline_mode = match env::var("SMART_PRO_PRIMARY_BASELINE_MODE")
        .unwrap_or_else(|_| "fast".into())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "normal" => SmartAutomovePreference::Normal,
        _ => SmartAutomovePreference::Fast,
    };
    let repeat_index = env_usize("SMART_PRO_PRIMARY_REPEAT_INDEX").unwrap_or(0);
    let opening_index = env_usize("SMART_PRO_PRIMARY_OPENING_INDEX").unwrap_or(0);
    let games_per_repeat = env_usize("SMART_PRO_PRIMARY_GAMES").unwrap_or(1).max(1);
    let max_plies = env_usize("SMART_PRO_PRIMARY_MAX_PLIES")
        .unwrap_or(56)
        .max(56);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(8).max(1);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let mirror_filter = env::var("SMART_PRO_PRIMARY_MIRROR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let seed_tag = env_profile_name("SMART_PRO_PRIMARY_SEED_TAG")
        .unwrap_or_else(|| "pro_primary_vs_fast:neutral_v3".to_string());
    let budget_a = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let budget_b = SearchBudget::from_preference(baseline_mode);
    let seed =
        seed_for_budget_duel_repeat_and_tag(budget_a, budget_b, repeat_index, seed_tag.as_str());
    let opening_fens = generate_opening_fens_cached(seed, games_per_repeat.max(opening_index + 1));
    let opening_fen = opening_fens
        .get(opening_index)
        .unwrap_or_else(|| panic!("opening index {} unavailable", opening_index))
        .clone();
    let candidate_white_ab = opening_index % 2 == 0;

    eprintln!(
        "pro primary opening probe config: candidate_profile={} baseline_profile={} baseline_mode={:?} seed_tag={} repeat_index={} opening_index={} games_per_repeat={} max_plies={} trace_limit={} include_acceptance={} mirror_filter={:?} opening={}",
        candidate_profile,
        baseline_profile,
        baseline_mode,
        seed_tag,
        repeat_index,
        opening_index,
        games_per_repeat,
        max_plies,
        trace_limit,
        include_acceptance,
        mirror_filter,
        opening_fen,
    );

    for (mirror, candidate_is_white) in [("ab", candidate_white_ab), ("ba", !candidate_white_ab)] {
        if mirror_filter
            .as_deref()
            .is_some_and(|expected| expected != mirror)
        {
            continue;
        }

        let (result, traces) = replay_cross_budget_loss_probe_game_with_options(
            candidate_profile.as_str(),
            SmartAutomovePreference::Pro,
            baseline_profile.as_str(),
            baseline_mode,
            opening_fen.as_str(),
            candidate_is_white,
            max_plies,
            trace_limit,
            include_acceptance,
        );
        let outcome = match result {
            MatchResult::CandidateWin => "W",
            MatchResult::OpponentWin => "L",
            MatchResult::Draw => "D",
        };
        eprintln!(
            "PRO_PRIMARY_OPENING outcome={} repeat={} opening_index={} mirror={} candidate_is_white={} seed={} seed_tag={} opening={}",
            outcome,
            repeat_index,
            opening_index,
            mirror,
            candidate_is_white,
            seed,
            seed_tag,
            opening_fen
        );
        for trace in traces {
            let engine = trace.candidate.turn_engine.as_ref();
            eprintln!(
                "PRO_PRIMARY_OPENING_TRACE opening_index={} mirror={} candidate_is_white={} ply={} fen={} candidate_move={} baseline_move={} candidate_stage={} baseline_stage={} engine_head={:?} root_search_selected={:?} accepted_after_search={:?} selected_utility={:?} candidate_utility={:?}",
                opening_index,
                mirror,
                candidate_is_white,
                trace.ply,
                trace.fen,
                trace.candidate.move_fen,
                trace.baseline.move_fen,
                trace.candidate.selector_last_stage,
                trace.baseline.selector_last_stage,
                engine.and_then(|engine| engine.candidate_move_fen.as_ref()),
                engine.and_then(|engine| engine.root_search_selected_move_fen.as_ref()),
                engine.and_then(|engine| engine.accepted_after_search),
                engine.and_then(|engine| engine.selected_utility),
                engine.and_then(|engine| engine.candidate_utility),
            );
        }
    }
}

fn replay_pro_reliability_from_game(
    candidate_profile: &str,
    baseline_profile: &str,
    start_game: &MonsGame,
    candidate_is_white: bool,
    max_plies: usize,
) -> MatchResult {
    let baseline_selector = profile_selector_from_name(baseline_profile).unwrap_or_else(|| {
        panic!(
            "profile '{}' not found for reliability probe baseline side",
            baseline_profile
        )
    });
    let candidate_selector = profile_selector_from_name(candidate_profile).unwrap_or_else(|| {
        panic!(
            "profile '{}' not found for reliability probe candidate side",
            candidate_profile
        )
    });
    let mut game = start_game.clone_for_simulation();

    for _ in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return match_result_from_winner(winner_color, candidate_is_white);
        }

        let candidate_to_move = if candidate_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };
        let (profile, selector) = if candidate_to_move {
            (candidate_profile, candidate_selector)
        } else {
            (baseline_profile, baseline_selector)
        };
        let config = loss_probe_runtime_config(profile, &game, SmartAutomovePreference::Pro);
        let inputs = select_inputs_with_runtime_fallback(selector, &game, config);

        if inputs.is_empty() {
            return if candidate_to_move {
                MatchResult::OpponentWin
            } else {
                MatchResult::CandidateWin
            };
        }

        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return if candidate_to_move {
                MatchResult::OpponentWin
            } else {
                MatchResult::CandidateWin
            };
        }
    }

    match adjudicate_non_terminal_game(&game) {
        Some(winner_color) => match_result_from_winner(winner_color, candidate_is_white),
        None => MatchResult::Draw,
    }
}

#[test]
#[ignore = "diagnostic: inspect direct pro-vs-current engine activity on every candidate ply"]
fn smart_automove_pro_direct_activity_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let repeats = env_usize("SMART_PRO_RELIABILITY_REPEATS")
        .unwrap_or(1)
        .max(1);
    let games_per_repeat = env_usize("SMART_PRO_RELIABILITY_GAMES").unwrap_or(1).max(1);
    let max_plies = env_usize("SMART_PRO_RELIABILITY_MAX_PLIES")
        .unwrap_or(80)
        .max(1);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(6).max(1);
    let seed_tag = env_profile_name("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_engine_direct_activity_v1".to_string());

    let baseline_selector =
        profile_selector_from_name(baseline_profile.as_str()).unwrap_or_else(|| {
            panic!(
                "profile '{}' not found for direct activity probe baseline side",
                baseline_profile
            )
        });
    let budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let mut aggregate = MatchupStats::default();
    let mut candidate_turns = 0usize;
    let mut disagreements = 0usize;
    let mut disagreements_logged = 0usize;
    let mut engine_planned = 0usize;
    let mut engine_cached_only = 0usize;
    let mut engine_no_plan = 0usize;
    let mut engine_budget = 0usize;
    let mut engine_selected_matches = 0usize;
    let mut engine_rank0 = 0usize;
    let mut engine_missing_from_roots = 0usize;
    let mut engine_score = 0usize;
    let mut engine_deny = 0usize;
    let mut engine_kill = 0usize;
    let mut engine_progress = 0usize;
    let mut engine_safety = 0usize;
    let mut engine_spirit = 0usize;
    let mut engine_tempo = 0usize;

    eprintln!(
        "pro direct activity config: candidate_profile={} baseline_profile={} seed_tag={} repeats={} games_per_repeat={} max_plies={} trace_limit={}",
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeats,
        games_per_repeat,
        max_plies,
        trace_limit
    );

    for repeat_index in 0..repeats {
        let seed =
            seed_for_budget_duel_repeat_and_tag(budget, budget, repeat_index, seed_tag.as_str());
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in
                [("ab", candidate_white_ab), ("ba", !candidate_white_ab)]
            {
                let mut game =
                    MonsGame::from_fen(opening_fen.as_str(), false).expect("valid opening fen");
                let mut game_recorded = false;
                for ply in 0..max_plies {
                    if let Some(winner_color) = game.winner_color() {
                        aggregate
                            .record(match_result_from_winner(winner_color, candidate_is_white));
                        game_recorded = true;
                        break;
                    }

                    let candidate_to_move = if candidate_is_white {
                        game.active_color == Color::White
                    } else {
                        game.active_color == Color::Black
                    };

                    let inputs = if candidate_to_move {
                        let candidate = loss_probe_decision(
                            candidate_profile.as_str(),
                            SmartAutomovePreference::Pro,
                            &game,
                        );
                        let baseline = loss_probe_decision(
                            baseline_profile.as_str(),
                            SmartAutomovePreference::Pro,
                            &game,
                        );
                        candidate_turns += 1;
                        if let Some(engine) = candidate.turn_engine.as_ref() {
                            match engine.status {
                                TurnEngineProbeStatus::Planned => engine_planned += 1,
                                TurnEngineProbeStatus::CachedOnly => engine_cached_only += 1,
                                TurnEngineProbeStatus::NoPlan => engine_no_plan += 1,
                                TurnEngineProbeStatus::BudgetExceeded => engine_budget += 1,
                                TurnEngineProbeStatus::InactiveColor => {}
                            }
                            if engine.candidate_move_fen.as_ref() == Some(&candidate.move_fen) {
                                engine_selected_matches += 1;
                            }
                            if engine.candidate_rank == Some(0) {
                                engine_rank0 += 1;
                            }
                            if engine.candidate_move_fen.is_some()
                                && engine.candidate_rank.is_none()
                            {
                                engine_missing_from_roots += 1;
                            }
                            match engine.candidate_family {
                                Some(TurnPlanFamily::ImmediateScore) => engine_score += 1,
                                Some(TurnPlanFamily::DenyOpponentWindow) => engine_deny += 1,
                                Some(TurnPlanFamily::DrainerKill) => engine_kill += 1,
                                Some(TurnPlanFamily::SafeSupermanaProgress)
                                | Some(TurnPlanFamily::SafeOpponentManaProgress) => {
                                    engine_progress += 1
                                }
                                Some(TurnPlanFamily::DrainerSafetyRecovery) => engine_safety += 1,
                                Some(TurnPlanFamily::SpiritImpact) => engine_spirit += 1,
                                Some(TurnPlanFamily::ManaTempo) => engine_tempo += 1,
                                None => {}
                            }
                        }
                        if candidate.move_fen != baseline.move_fen {
                            disagreements += 1;
                            if disagreements_logged < trace_limit {
                                disagreements_logged += 1;
                                eprintln!(
                                    "PRO_ACTIVITY disagreement={} repeat={} opening_index={} mirror={} candidate_is_white={} ply={} seed={} opening={}",
                                    disagreements_logged,
                                    repeat_index,
                                    opening_index,
                                    mirror,
                                    candidate_is_white,
                                    ply,
                                    seed,
                                    opening_fen
                                );
                                print_loss_probe_decision("    candidate", &candidate);
                                print_loss_probe_decision("    baseline", &baseline);
                            }
                        }
                        candidate.inputs.clone()
                    } else {
                        let config = loss_probe_runtime_config(
                            baseline_profile.as_str(),
                            &game,
                            SmartAutomovePreference::Pro,
                        );
                        select_inputs_with_runtime_fallback(baseline_selector, &game, config)
                    };

                    if inputs.is_empty() {
                        aggregate.record(if candidate_to_move {
                            MatchResult::OpponentWin
                        } else {
                            MatchResult::CandidateWin
                        });
                        game_recorded = true;
                        break;
                    }

                    if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
                        aggregate.record(if candidate_to_move {
                            MatchResult::OpponentWin
                        } else {
                            MatchResult::CandidateWin
                        });
                        game_recorded = true;
                        break;
                    }
                }

                if !game_recorded {
                    aggregate.record(match adjudicate_non_terminal_game(&game) {
                        Some(winner_color) => {
                            match_result_from_winner(winner_color, candidate_is_white)
                        }
                        None => MatchResult::Draw,
                    });
                }
            }
        }
    }

    eprintln!(
        "pro direct activity summary: total_games={} wins={} losses={} draws={} win_rate={:.4} confidence={:.4} candidate_turns={} disagreements={} engine_status=[planned:{} cached_only:{} no_plan:{} budget:{}] engine_head=[matches_selected:{} rank0:{} missing_from_roots:{}] engine_families=[score:{} deny:{} kill:{} progress:{} safety:{} spirit:{} tempo:{}]",
        aggregate.total_games(),
        aggregate.wins,
        aggregate.losses,
        aggregate.draws,
        aggregate.win_rate_points(),
        aggregate.confidence_better_than_even(),
        candidate_turns,
        disagreements,
        engine_planned,
        engine_cached_only,
        engine_no_plan,
        engine_budget,
        engine_selected_matches,
        engine_rank0,
        engine_missing_from_roots,
        engine_score,
        engine_deny,
        engine_kill,
        engine_progress,
        engine_safety,
        engine_spirit,
        engine_tempo,
    );

    assert!(
        aggregate.total_games() > 0,
        "direct activity probe ran no games"
    );
}

#[test]
#[ignore = "diagnostic: force first black chunks on isolated pro loss states and compare outcomes"]
fn smart_automove_pro_black_turn_forced_root_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let fixture_filter = env::var("SMART_PROBE_FIXTURES")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            vec![
                "primary_black_loss_opening_a_black_turn".to_string(),
                "primary_black_loss_opening_b_black_turn".to_string(),
            ]
        });
    let max_plies = env_usize("SMART_PROBE_MAX_PLIES").unwrap_or(24).max(1);
    let top_n = env_usize("SMART_PROBE_TOP_ROOTS").unwrap_or(6).max(1);
    let fixtures = primary_pro_triage_fixtures();

    for fixture in fixtures.iter().filter(|fixture| {
        fixture_filter.is_empty() || fixture_filter.iter().any(|id| id == fixture.id)
    }) {
        let config = loss_probe_runtime_config(
            candidate_profile.as_str(),
            &fixture.game,
            SmartAutomovePreference::Pro,
        );
        let ranked_roots =
            MonsGameModel::ranked_root_moves(&fixture.game, fixture.game.active_color, config);
        let selection = loss_probe_decision(
            candidate_profile.as_str(),
            SmartAutomovePreference::Pro,
            &fixture.game,
        );
        let baseline = loss_probe_decision(
            baseline_profile.as_str(),
            SmartAutomovePreference::Pro,
            &fixture.game,
        );
        let engine_probe = turn_engine_probe(
            &fixture.game,
            fixture.game.active_color,
            config.turn_engine_mode,
            calibration_turn_engine_config(config),
        );
        let mut candidates = ranked_roots
            .iter()
            .take(top_n)
            .map(|root| root.inputs.clone())
            .collect::<Vec<_>>();
        if let Some(engine_inputs) = engine_probe.candidate_chunk.clone() {
            if !candidates.iter().any(|inputs| *inputs == engine_inputs) {
                candidates.push(engine_inputs);
            }
        }

        println!(
            "\nforced-root fixture={} active={:?} fen={}",
            fixture.id,
            fixture.game.active_color,
            fixture.game.fen()
        );
        print_loss_probe_decision("  candidate", &selection);
        print_loss_probe_decision("  baseline", &baseline);
        println!("  engine_probe={:?}", engine_probe);

        for forced_inputs in candidates {
            let forced_fen = Input::fen_from_array(&forced_inputs);
            let root_digest = ranked_roots
                .iter()
                .find(|root| root.inputs == forced_inputs)
                .map(triage_root_digest_entry);
            let Some((after, _)) =
                MonsGameModel::apply_inputs_for_search_with_events(&fixture.game, &forced_inputs)
            else {
                println!("  forced={} outcome=invalid", forced_fen);
                continue;
            };
            let outcome = match replay_pro_reliability_from_game(
                candidate_profile.as_str(),
                baseline_profile.as_str(),
                &after,
                false,
                max_plies.saturating_sub(1),
            ) {
                MatchResult::CandidateWin => "W",
                MatchResult::OpponentWin => "L",
                MatchResult::Draw => "D",
            };
            let continuation_config = loss_probe_runtime_config(
                candidate_profile.as_str(),
                &after,
                SmartAutomovePreference::Pro,
            );
            let continuation_probe = turn_engine_probe(
                &after,
                after.active_color,
                continuation_config.turn_engine_mode,
                calibration_turn_engine_config(continuation_config),
            );
            let continuation_plan = turn_engine_best_plan_for_test(
                &after,
                after.active_color,
                calibration_turn_engine_config(continuation_config),
            )
            .map(|plan| {
                (
                    plan.head_family,
                    plan.goal_family,
                    plan.utility,
                    plan.head_utility,
                    plan.compiled_chunks
                        .first()
                        .map(|inputs| Input::fen_from_array(inputs))
                        .unwrap_or_default(),
                    plan.compiled_chunks.len(),
                )
            });
            println!(
                "  forced={} outcome={} next_active={:?} next_turn={} root={:?} continuation_probe={:?} continuation_plan={:?}",
                forced_fen,
                outcome,
                after.active_color,
                after.turn_number,
                root_digest,
                continuation_probe,
                continuation_plan,
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: replay Normal losses vs Fast and print same-position move disagreements"]
fn smart_automove_normal_vs_fast_loss_probe() {
    let normal_profile =
        env_profile_name("SMART_PROBE_NORMAL_PROFILE").unwrap_or_else(|| "runtime_current".into());
    let fast_profile =
        env_profile_name("SMART_PROBE_FAST_PROFILE").unwrap_or_else(|| normal_profile.clone());
    let pro_profile =
        env_profile_name("SMART_PROBE_PRO_PROFILE").unwrap_or_else(|| normal_profile.clone());
    let repeats = env_usize("SMART_PROBE_REPEATS").unwrap_or(1).max(1);
    let games_per_repeat = env_usize("SMART_PROBE_GAMES").unwrap_or(4).max(1);
    let max_plies = env_usize("SMART_PROBE_MAX_PLIES").unwrap_or(80).max(40);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(3).max(1);
    let loss_limit = env_usize("SMART_PROBE_LOSS_LIMIT").unwrap_or(6).max(1);
    let use_white_opening_book = env_bool("SMART_PROBE_USE_WHITE_OPENING_BOOK").unwrap_or(false);
    let seed_tags = loss_probe_seed_tags();
    let normal_budget = SearchBudget::from_preference(SmartAutomovePreference::Normal);
    let fast_budget = SearchBudget::from_preference(SmartAutomovePreference::Fast);

    let mut total_games = 0usize;
    let mut normal_losses = 0usize;
    let mut losses_with_disagreement = 0usize;
    let mut disagreements_logged = 0usize;
    let mut pro_matches_fast = 0usize;
    let mut pro_matches_normal = 0usize;
    let mut pro_matches_neither = 0usize;
    let mut fast_safer_supermana = 0usize;
    let mut fast_safer_opponent_mana = 0usize;
    let mut fast_avoids_handoff = 0usize;
    let mut fast_avoids_roundtrip = 0usize;
    let mut fast_avoids_drainer_vulnerability = 0usize;
    let mut fast_more_attack = 0usize;
    let mut fast_more_spirit = 0usize;

    eprintln!(
        "normal-fast loss probe config: normal_profile={} fast_profile={} pro_profile={} seeds={} repeats={} games_per_repeat={} max_plies={} trace_limit={} loss_limit={} use_white_opening_book={}",
        normal_profile,
        fast_profile,
        pro_profile,
        seed_tags.len(),
        repeats,
        games_per_repeat,
        max_plies,
        trace_limit,
        loss_limit,
        use_white_opening_book
    );

    'seed_loop: for seed_tag in &seed_tags {
        for repeat_index in 0..repeats {
            let seed = seed_for_budget_duel_repeat_and_tag(
                normal_budget,
                fast_budget,
                repeat_index,
                seed_tag.as_str(),
            );
            let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);
            for opening_fen in opening_fens.iter() {
                for normal_is_white in [true, false] {
                    total_games += 1;
                    let (result, traces) = replay_normal_vs_fast_loss_probe_game(
                        normal_profile.as_str(),
                        fast_profile.as_str(),
                        pro_profile.as_str(),
                        opening_fen.as_str(),
                        normal_is_white,
                        max_plies,
                        use_white_opening_book,
                        trace_limit,
                    );
                    if result != MatchResult::OpponentWin {
                        continue;
                    }

                    normal_losses += 1;
                    if !traces.is_empty() {
                        losses_with_disagreement += 1;
                    }

                    eprintln!(
                        "NORMAL_LOSS game={} seed_tag={} repeat={} normal_is_white={} opening={}",
                        normal_losses, seed_tag, repeat_index, normal_is_white, opening_fen
                    );
                    for trace in &traces {
                        disagreements_logged += 1;
                        match (
                            trace.pro.move_fen == trace.fast.move_fen,
                            trace.pro.move_fen == trace.normal.move_fen,
                        ) {
                            (true, false) => pro_matches_fast += 1,
                            (false, true) => pro_matches_normal += 1,
                            _ => pro_matches_neither += 1,
                        }
                        if let (Some(normal_root), Some(fast_root)) = (
                            trace.normal.selected_root.as_ref(),
                            trace.fast.selected_root.as_ref(),
                        ) {
                            if fast_root.safe_supermana_progress_steps
                                < normal_root.safe_supermana_progress_steps
                            {
                                fast_safer_supermana += 1;
                            }
                            if fast_root.safe_opponent_mana_progress_steps
                                < normal_root.safe_opponent_mana_progress_steps
                            {
                                fast_safer_opponent_mana += 1;
                            }
                            if normal_root.mana_handoff_to_opponent
                                && !fast_root.mana_handoff_to_opponent
                            {
                                fast_avoids_handoff += 1;
                            }
                            if normal_root.has_roundtrip && !fast_root.has_roundtrip {
                                fast_avoids_roundtrip += 1;
                            }
                            if normal_root.own_drainer_vulnerable
                                && !fast_root.own_drainer_vulnerable
                            {
                                fast_avoids_drainer_vulnerability += 1;
                            }
                            if fast_root.attacks_opponent_drainer
                                && !normal_root.attacks_opponent_drainer
                            {
                                fast_more_attack += 1;
                            }
                            if fast_root.spirit_development && !normal_root.spirit_development {
                                fast_more_spirit += 1;
                            }
                        }

                        eprintln!("  TRACE ply={} fen={}", trace.ply, trace.fen);
                        print_loss_probe_decision("    normal", &trace.normal);
                        print_loss_probe_decision("    fast", &trace.fast);
                        print_loss_probe_decision("    pro", &trace.pro);
                    }

                    if normal_losses >= loss_limit {
                        break 'seed_loop;
                    }
                }
            }
        }
    }

    eprintln!(
        "normal-fast loss probe summary: total_games={} normal_losses={} losses_with_disagreement={} disagreements_logged={} pro_matches_fast={} pro_matches_normal={} pro_matches_neither={} fast_safer_supermana={} fast_safer_opponent_mana={} fast_avoids_handoff={} fast_avoids_roundtrip={} fast_avoids_drainer_vulnerability={} fast_more_attack={} fast_more_spirit={}",
        total_games,
        normal_losses,
        losses_with_disagreement,
        disagreements_logged,
        pro_matches_fast,
        pro_matches_normal,
        pro_matches_neither,
        fast_safer_supermana,
        fast_safer_opponent_mana,
        fast_avoids_handoff,
        fast_avoids_roundtrip,
        fast_avoids_drainer_vulnerability,
        fast_more_attack,
        fast_more_spirit,
    );

    assert!(
        normal_losses > 0,
        "loss probe found no normal losses; increase SMART_PROBE_GAMES/SMART_PROBE_REPEATS"
    );
}

#[test]
#[ignore = "diagnostic: replay pro reliability smoke losses and print candidate-vs-baseline divergences"]
fn smart_automove_pro_reliability_loss_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let repeats = env_usize("SMART_PRO_RELIABILITY_REPEATS")
        .unwrap_or(2)
        .max(1);
    let games_per_repeat = env_usize("SMART_PRO_RELIABILITY_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_RELIABILITY_MAX_PLIES")
        .unwrap_or(12)
        .max(1);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(3).max(1);
    let seed_tag = env_profile_name("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());

    let budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let mut aggregate = MatchupStats::default();
    let mut losses = 0usize;
    let mut losses_with_disagreement = 0usize;
    let mut disagreements_logged = 0usize;
    let mut baseline_safer_supermana = 0usize;
    let mut baseline_safer_opponent_mana = 0usize;
    let mut baseline_avoids_handoff = 0usize;
    let mut baseline_avoids_roundtrip = 0usize;
    let mut baseline_avoids_drainer_vulnerability = 0usize;
    let mut candidate_engine_planned = 0usize;
    let mut candidate_engine_cached_only = 0usize;
    let mut candidate_engine_no_plan = 0usize;
    let mut candidate_engine_budget = 0usize;
    let mut candidate_engine_head_matches_selected = 0usize;
    let mut candidate_engine_head_rank0 = 0usize;
    let mut candidate_engine_head_missing_from_roots = 0usize;
    let mut candidate_engine_immediate_score = 0usize;
    let mut candidate_engine_deny = 0usize;
    let mut candidate_engine_kill = 0usize;
    let mut candidate_engine_progress = 0usize;
    let mut candidate_engine_safety = 0usize;
    let mut candidate_engine_spirit = 0usize;
    let mut candidate_engine_tempo = 0usize;

    eprintln!(
        "pro reliability loss probe config: candidate_profile={} baseline_profile={} seed_tag={} repeats={} games_per_repeat={} max_plies={} trace_limit={}",
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeats,
        games_per_repeat,
        max_plies,
        trace_limit
    );

    for repeat_index in 0..repeats {
        let seed =
            seed_for_budget_duel_repeat_and_tag(budget, budget, repeat_index, seed_tag.as_str());
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in
                [("ab", candidate_white_ab), ("ba", !candidate_white_ab)]
            {
                let (result, traces) = replay_pro_reliability_loss_probe_game(
                    candidate_profile.as_str(),
                    baseline_profile.as_str(),
                    opening_fen.as_str(),
                    candidate_is_white,
                    max_plies,
                    trace_limit,
                );
                aggregate.record(result);

                if result != MatchResult::OpponentWin {
                    continue;
                }

                losses += 1;
                if !traces.is_empty() {
                    losses_with_disagreement += 1;
                }

                eprintln!(
                    "PRO_LOSS game={} repeat={} opening_index={} mirror={} candidate_is_white={} seed={} opening={}",
                    losses,
                    repeat_index,
                    opening_index,
                    mirror,
                    candidate_is_white,
                    seed,
                    opening_fen
                );
                for trace in &traces {
                    disagreements_logged += 1;
                    if let Some(engine) = trace.candidate.turn_engine.as_ref() {
                        match engine.status {
                            TurnEngineProbeStatus::Planned => candidate_engine_planned += 1,
                            TurnEngineProbeStatus::CachedOnly => candidate_engine_cached_only += 1,
                            TurnEngineProbeStatus::NoPlan => candidate_engine_no_plan += 1,
                            TurnEngineProbeStatus::BudgetExceeded => candidate_engine_budget += 1,
                            TurnEngineProbeStatus::InactiveColor => {}
                        }
                        if engine.candidate_move_fen.as_ref() == Some(&trace.candidate.move_fen) {
                            candidate_engine_head_matches_selected += 1;
                        }
                        if engine.candidate_rank == Some(0) {
                            candidate_engine_head_rank0 += 1;
                        }
                        if engine.candidate_move_fen.is_some() && engine.candidate_rank.is_none() {
                            candidate_engine_head_missing_from_roots += 1;
                        }
                        match engine.candidate_family {
                            Some(TurnPlanFamily::ImmediateScore) => {
                                candidate_engine_immediate_score += 1
                            }
                            Some(TurnPlanFamily::DenyOpponentWindow) => candidate_engine_deny += 1,
                            Some(TurnPlanFamily::DrainerKill) => candidate_engine_kill += 1,
                            Some(TurnPlanFamily::SafeSupermanaProgress)
                            | Some(TurnPlanFamily::SafeOpponentManaProgress) => {
                                candidate_engine_progress += 1
                            }
                            Some(TurnPlanFamily::DrainerSafetyRecovery) => {
                                candidate_engine_safety += 1
                            }
                            Some(TurnPlanFamily::SpiritImpact) => candidate_engine_spirit += 1,
                            Some(TurnPlanFamily::ManaTempo) => candidate_engine_tempo += 1,
                            None => {}
                        }
                    }
                    if let (Some(candidate_root), Some(baseline_root)) = (
                        trace.candidate.selected_root.as_ref(),
                        trace.baseline.selected_root.as_ref(),
                    ) {
                        if baseline_root.safe_supermana_progress_steps
                            < candidate_root.safe_supermana_progress_steps
                        {
                            baseline_safer_supermana += 1;
                        }
                        if baseline_root.safe_opponent_mana_progress_steps
                            < candidate_root.safe_opponent_mana_progress_steps
                        {
                            baseline_safer_opponent_mana += 1;
                        }
                        if candidate_root.mana_handoff_to_opponent
                            && !baseline_root.mana_handoff_to_opponent
                        {
                            baseline_avoids_handoff += 1;
                        }
                        if candidate_root.has_roundtrip && !baseline_root.has_roundtrip {
                            baseline_avoids_roundtrip += 1;
                        }
                        if candidate_root.own_drainer_vulnerable
                            && !baseline_root.own_drainer_vulnerable
                        {
                            baseline_avoids_drainer_vulnerability += 1;
                        }
                    }
                    eprintln!("  TRACE ply={} fen={}", trace.ply, trace.fen);
                    print_loss_probe_decision("    candidate", &trace.candidate);
                    print_loss_probe_decision("    baseline", &trace.baseline);
                }
            }
        }
    }

    eprintln!(
        "pro reliability loss probe summary: total_games={} wins={} losses={} draws={} win_rate={:.4} confidence={:.4} losses_with_disagreement={} disagreements_logged={} baseline_safer_supermana={} baseline_safer_opponent_mana={} baseline_avoids_handoff={} baseline_avoids_roundtrip={} baseline_avoids_drainer_vulnerability={} candidate_engine_status=[planned:{} cached_only:{} no_plan:{} budget:{}] candidate_engine_head=[matches_selected:{} rank0:{} missing_from_roots:{}] candidate_engine_families=[score:{} deny:{} kill:{} progress:{} safety:{} spirit:{} tempo:{}]",
        aggregate.total_games(),
        aggregate.wins,
        aggregate.losses,
        aggregate.draws,
        aggregate.win_rate_points(),
        aggregate.confidence_better_than_even(),
        losses_with_disagreement,
        disagreements_logged,
        baseline_safer_supermana,
        baseline_safer_opponent_mana,
        baseline_avoids_handoff,
        baseline_avoids_roundtrip,
        baseline_avoids_drainer_vulnerability,
        candidate_engine_planned,
        candidate_engine_cached_only,
        candidate_engine_no_plan,
        candidate_engine_budget,
        candidate_engine_head_matches_selected,
        candidate_engine_head_rank0,
        candidate_engine_head_missing_from_roots,
        candidate_engine_immediate_score,
        candidate_engine_deny,
        candidate_engine_kill,
        candidate_engine_progress,
        candidate_engine_safety,
        candidate_engine_spirit,
        candidate_engine_tempo,
    );

    assert!(
        losses > 0,
        "reliability loss probe found no candidate losses"
    );
}

#[test]
#[ignore = "diagnostic: harvest pro reliability disagreements with engine utilities and acceptance"]
fn smart_automove_pro_reliability_disagreement_harvest() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let repeats = env_usize("SMART_PRO_RELIABILITY_REPEATS")
        .unwrap_or(1)
        .max(1);
    let games_per_repeat = env_usize("SMART_PRO_RELIABILITY_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_RELIABILITY_MAX_PLIES")
        .unwrap_or(32)
        .max(1);
    let disagreement_limit = env_usize("SMART_PROBE_DISAGREEMENT_LIMIT")
        .unwrap_or(64)
        .max(1);
    let seed_tag = env_profile_name("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_engine_direct_activity_v1".to_string());
    let budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);

    let mut total_games = 0usize;
    let mut total_disagreements = 0usize;
    let mut logged_disagreements = 0usize;

    eprintln!(
        "pro reliability disagreement harvest config: candidate_profile={} baseline_profile={} seed_tag={} repeats={} games_per_repeat={} max_plies={} disagreement_limit={}",
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeats,
        games_per_repeat,
        max_plies,
        disagreement_limit,
    );

    for repeat_index in 0..repeats {
        let seed =
            seed_for_budget_duel_repeat_and_tag(budget, budget, repeat_index, seed_tag.as_str());
        let opening_fens = generate_opening_fens_cached(seed, games_per_repeat);

        for (opening_index, opening_fen) in opening_fens.iter().enumerate() {
            let candidate_white_ab = opening_index % 2 == 0;
            for (mirror, candidate_is_white) in
                [("ab", candidate_white_ab), ("ba", !candidate_white_ab)]
            {
                total_games += 1;
                let (result, traces) = replay_pro_reliability_loss_probe_game_with_options(
                    candidate_profile.as_str(),
                    baseline_profile.as_str(),
                    opening_fen.as_str(),
                    candidate_is_white,
                    max_plies,
                    max_plies,
                    true,
                );
                let outcome = match result {
                    MatchResult::CandidateWin => "W",
                    MatchResult::OpponentWin => "L",
                    MatchResult::Draw => "D",
                };

                for trace in traces {
                    total_disagreements += 1;
                    if logged_disagreements >= disagreement_limit {
                        continue;
                    }
                    logged_disagreements += 1;
                    let engine = trace.candidate.turn_engine.as_ref();
                    eprintln!(
                        "PRO_HARVEST disagreement={} repeat={} opening_index={} mirror={} candidate_is_white={} seed={} ply={} outcome={} opening={} fen={} candidate_move={} baseline_move={} engine_head={:?} root_search_selected={:?} accepted_after_search={:?} selected_utility={:?} candidate_utility={:?}",
                        logged_disagreements,
                        repeat_index,
                        opening_index,
                        mirror,
                        candidate_is_white,
                        seed,
                        trace.ply,
                        outcome,
                        opening_fen,
                        trace.fen,
                        trace.candidate.move_fen,
                        trace.baseline.move_fen,
                        engine.and_then(|engine| engine.candidate_move_fen.as_ref()),
                        engine.and_then(|engine| engine.root_search_selected_move_fen.as_ref()),
                        engine.and_then(|engine| engine.accepted_after_search),
                        engine.and_then(|engine| engine.selected_utility),
                        engine.and_then(|engine| engine.candidate_utility),
                    );
                }
            }
        }
    }

    eprintln!(
        "pro reliability disagreement harvest summary: total_games={} total_disagreements={} logged_disagreements={}",
        total_games,
        total_disagreements,
        logged_disagreements,
    );

    assert!(
        total_disagreements > 0,
        "disagreement harvest found no candidate-vs-baseline disagreements"
    );
}

#[test]
#[ignore = "diagnostic: replay a single pro reliability opening index with optional mirror filter"]
fn smart_automove_pro_reliability_opening_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let repeat_index = env_usize("SMART_PRO_RELIABILITY_REPEAT_INDEX").unwrap_or(0);
    let opening_index = env_usize("SMART_PRO_RELIABILITY_OPENING_INDEX").unwrap_or(0);
    let games_per_repeat = env_usize("SMART_PRO_RELIABILITY_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_RELIABILITY_MAX_PLIES")
        .unwrap_or(32)
        .max(1);
    let trace_limit = env_usize("SMART_PROBE_TRACE_LIMIT").unwrap_or(8).max(1);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let mirror_filter = env::var("SMART_PRO_RELIABILITY_MIRROR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let seed_tag = env_profile_name("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());
    let budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let seed = seed_for_budget_duel_repeat_and_tag(budget, budget, repeat_index, seed_tag.as_str());
    let opening_fens = generate_opening_fens_cached(seed, games_per_repeat.max(opening_index + 1));
    let opening_fen = opening_fens
        .get(opening_index)
        .unwrap_or_else(|| panic!("opening index {} unavailable", opening_index))
        .clone();
    let candidate_white_ab = opening_index % 2 == 0;

    eprintln!(
        "pro reliability opening probe config: candidate_profile={} baseline_profile={} seed_tag={} repeat_index={} opening_index={} games_per_repeat={} max_plies={} trace_limit={} include_acceptance={} mirror_filter={:?} opening={}",
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeat_index,
        opening_index,
        games_per_repeat,
        max_plies,
        trace_limit,
        include_acceptance,
        mirror_filter,
        opening_fen,
    );

    for (mirror, candidate_is_white) in [("ab", candidate_white_ab), ("ba", !candidate_white_ab)] {
        if mirror_filter
            .as_deref()
            .is_some_and(|expected| expected != mirror)
        {
            continue;
        }

        let (result, traces) = replay_pro_reliability_loss_probe_game_with_options(
            candidate_profile.as_str(),
            baseline_profile.as_str(),
            opening_fen.as_str(),
            candidate_is_white,
            max_plies,
            trace_limit,
            include_acceptance,
        );
        let outcome = match result {
            MatchResult::CandidateWin => "W",
            MatchResult::OpponentWin => "L",
            MatchResult::Draw => "D",
        };
        eprintln!(
            "PRO_OPENING outcome={} repeat={} opening_index={} mirror={} candidate_is_white={} seed={} opening={}",
            outcome,
            repeat_index,
            opening_index,
            mirror,
            candidate_is_white,
            seed,
            opening_fen
        );
        for trace in traces {
            let engine = trace.candidate.turn_engine.as_ref();
            eprintln!(
                "PRO_OPENING_TRACE opening_index={} mirror={} candidate_is_white={} ply={} fen={} candidate_move={} baseline_move={} engine_head={:?} root_search_selected={:?} accepted_after_search={:?} selected_utility={:?} candidate_utility={:?}",
                opening_index,
                mirror,
                candidate_is_white,
                trace.ply,
                trace.fen,
                trace.candidate.move_fen,
                trace.baseline.move_fen,
                engine.and_then(|engine| engine.candidate_move_fen.as_ref()),
                engine.and_then(|engine| engine.root_search_selected_move_fen.as_ref()),
                engine.and_then(|engine| engine.accepted_after_search),
                engine.and_then(|engine| engine.selected_utility),
                engine.and_then(|engine| engine.candidate_utility),
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: print every ply on one pro reliability opening"]
fn smart_automove_pro_reliability_opening_full_trace() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let repeat_index = env_usize("SMART_PRO_RELIABILITY_REPEAT_INDEX").unwrap_or(0);
    let opening_index = env_usize("SMART_PRO_RELIABILITY_OPENING_INDEX").unwrap_or(0);
    let games_per_repeat = env_usize("SMART_PRO_RELIABILITY_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_RELIABILITY_MAX_PLIES")
        .unwrap_or(32)
        .max(1);
    let include_acceptance = env_bool("SMART_PROBE_INCLUDE_ACCEPTANCE").unwrap_or(true);
    let mirror = env::var("SMART_PRO_RELIABILITY_MIRROR").unwrap_or_else(|_| "ab".into());
    let seed_tag = env_profile_name("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());
    let budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let seed = seed_for_budget_duel_repeat_and_tag(budget, budget, repeat_index, seed_tag.as_str());
    let opening_fens = generate_opening_fens_cached(seed, games_per_repeat.max(opening_index + 1));
    let opening_fen = opening_fens
        .get(opening_index)
        .unwrap_or_else(|| panic!("opening index {} unavailable", opening_index))
        .clone();
    let candidate_white_ab = opening_index % 2 == 0;
    let candidate_is_white = match mirror.as_str() {
        "ab" => candidate_white_ab,
        "ba" => !candidate_white_ab,
        _ => panic!("unsupported mirror '{}'", mirror),
    };
    let baseline_selector =
        profile_selector_from_name(baseline_profile.as_str()).unwrap_or_else(|| {
            panic!(
                "profile '{}' not found for reliability probe baseline side",
                baseline_profile
            )
        });
    let mut game = MonsGame::from_fen(opening_fen.as_str(), false).expect("valid opening fen");

    eprintln!(
        "PRO_FULL_TRACE_CONFIG candidate_profile={} baseline_profile={} seed_tag={} repeat_index={} opening_index={} mirror={} candidate_is_white={} max_plies={} opening={}",
        candidate_profile,
        baseline_profile,
        seed_tag,
        repeat_index,
        opening_index,
        mirror,
        candidate_is_white,
        max_plies,
        opening_fen
    );

    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            eprintln!(
                "PRO_FULL_TRACE_END ply={} winner={:?} fen={}",
                ply,
                winner_color,
                game.fen()
            );
            return;
        }

        let candidate_to_move = if candidate_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };

        if candidate_to_move {
            let candidate = loss_probe_decision_with_options(
                candidate_profile.as_str(),
                SmartAutomovePreference::Pro,
                &game,
                include_acceptance,
            );
            let baseline = loss_probe_decision_with_options(
                baseline_profile.as_str(),
                SmartAutomovePreference::Pro,
                &game,
                include_acceptance,
            );
            let engine = candidate.turn_engine.as_ref();
            eprintln!(
                "PRO_FULL_TRACE ply={} side=candidate active={:?} fen={} candidate_move={} baseline_move={} engine_status={:?} cached_move={:?} engine_head={:?} root_search_selected={:?} accepted_after_search={:?}",
                ply,
                game.active_color,
                game.fen(),
                candidate.move_fen,
                baseline.move_fen,
                engine.map(|engine| engine.status),
                engine.and_then(|engine| engine.cached_move_fen.as_ref()),
                engine.and_then(|engine| engine.candidate_move_fen.as_ref()),
                engine.and_then(|engine| engine.root_search_selected_move_fen.as_ref()),
                engine.and_then(|engine| engine.accepted_after_search),
            );
            let inputs = candidate.inputs.clone();
            if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
                eprintln!("PRO_FULL_TRACE_END ply={} invalid_candidate_move", ply);
                return;
            }
        } else {
            let config = loss_probe_runtime_config(
                baseline_profile.as_str(),
                &game,
                SmartAutomovePreference::Pro,
            );
            let inputs = select_inputs_with_runtime_fallback(baseline_selector, &game, config);
            eprintln!(
                "PRO_FULL_TRACE ply={} side=baseline active={:?} fen={} move={}",
                ply,
                game.active_color,
                game.fen(),
                Input::fen_from_array(&inputs),
            );
            if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
                eprintln!("PRO_FULL_TRACE_END ply={} invalid_baseline_move", ply);
                return;
            }
        }
    }

    eprintln!(
        "PRO_FULL_TRACE_END ply={} adjudication={:?} fen={}",
        max_plies,
        adjudicate_non_terminal_game(&game),
        game.fen()
    );
}

#[test]
#[ignore = "diagnostic: probe pro confirmation lane deltas with opening-book enabled"]
fn smart_automove_pro_confirmation_lane_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_release_safe_pre_exact".into());
    let repeats = env_usize("SMART_PRO_CONFIRM_PROBE_REPEATS")
        .unwrap_or(2)
        .max(1);
    let games = env_usize("SMART_PRO_CONFIRM_PROBE_GAMES")
        .unwrap_or(2)
        .max(1);
    let max_plies = env_usize("SMART_PRO_CONFIRM_PROBE_MAX_PLIES")
        .unwrap_or(56)
        .max(24);

    let vs_normal = run_cross_budget_duel(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Normal,
        seed_tag: "pro_confirm_vs_normal_v1",
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: true,
    });
    let vs_fast = run_cross_budget_duel(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Fast,
        seed_tag: "pro_confirm_vs_fast_v1",
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: true,
    });
    let (vn_delta, vn_conf) = stats_delta_confidence(vs_normal);
    let (vf_delta, vf_conf) = stats_delta_confidence(vs_fast);
    println!(
        "pro confirm probe candidate={} baseline={} repeats={} games={} max_plies={} vs_normal_delta={:.4} vs_normal_conf={:.4} vs_fast_delta={:.4} vs_fast_conf={:.4}",
        candidate_profile,
        baseline_profile,
        repeats,
        games,
        max_plies,
        vn_delta,
        vn_conf,
        vf_delta,
        vf_conf
    );
}

#[test]
#[ignore = "diagnostic: probe pro planner diagnostics counters on duel lanes"]
fn smart_automove_pro_planner_activity_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_release_safe_pre_exact".into());
    let repeats = env_usize("SMART_PRO_ACTIVITY_REPEATS").unwrap_or(1).max(1);
    let games = env_usize("SMART_PRO_ACTIVITY_GAMES").unwrap_or(1).max(1);
    let max_plies = env_usize("SMART_PRO_ACTIVITY_MAX_PLIES")
        .unwrap_or(56)
        .max(24);
    let seed_tag_vs_normal = env_profile_name("SMART_PRO_ACTIVITY_VS_NORMAL_SEED_TAG")
        .unwrap_or_else(|| "pro_activity_vs_normal_v1".to_string());
    let seed_tag_vs_fast = env_profile_name("SMART_PRO_ACTIVITY_VS_FAST_SEED_TAG")
        .unwrap_or_else(|| "pro_activity_vs_fast_v1".to_string());

    clear_turn_planner_diagnostics();
    let vs_normal = run_cross_budget_duel(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Normal,
        seed_tag: seed_tag_vs_normal.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let diag_vs_normal = turn_planner_diagnostics_snapshot();

    clear_turn_planner_diagnostics();
    let vs_fast = run_cross_budget_duel(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Fast,
        seed_tag: seed_tag_vs_fast.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let diag_vs_fast = turn_planner_diagnostics_snapshot();
    clear_turn_planner_diagnostics();

    let (vn_delta, vn_conf) = stats_delta_confidence(vs_normal);
    let (vf_delta, vf_conf) = stats_delta_confidence(vs_fast);
    println!(
        "pro planner activity candidate={} baseline={} repeats={} games={} max_plies={} vs_normal_delta={:.4} vs_normal_conf={:.4} vs_fast_delta={:.4} vs_fast_conf={:.4} normal(intent_calls={} intent_hits={} compile_fallbacks={} compile_fallbacks_path={} compile_fallbacks_spirit={} compile_fallbacks_attack={} injected_candidates={} injected_duplicates={} injected_attempts={} injected_accepts={} reject_build={} reject_emergency={} reject_emerg_loss={} reject_emerg_no_signal={} reject_emerg_handoff={} reject_emerg_drainer={} reject_top_wins={} reject_unsafe={} reject_no_tactical={} reject_gap={} planner_no_plan={} planner_attempts={} planner_accepts={} planner_rejects={} planner_rej_not_in_root={} planner_rej_missing_top={} planner_rej_top_not_tactical_or_unsafe={} planner_rej_top_wins={} planner_rej_candidate_unsafe={} planner_rej_progress_gate={} planner_rej_tactical_gate={} planner_rej_safety_progress_gate={} route_model_tactical={} route_drainer_score={} route_drainer_kill={} route_spirit_impact={} route_drainer_safety={} route_mana_move={} route_tactical_deny={} route_fallback={} expansions={}) fast(intent_calls={} intent_hits={} compile_fallbacks={} compile_fallbacks_path={} compile_fallbacks_spirit={} compile_fallbacks_attack={} injected_candidates={} injected_duplicates={} injected_attempts={} injected_accepts={} reject_build={} reject_emergency={} reject_emerg_loss={} reject_emerg_no_signal={} reject_emerg_handoff={} reject_emerg_drainer={} reject_top_wins={} reject_unsafe={} reject_no_tactical={} reject_gap={} planner_no_plan={} planner_attempts={} planner_accepts={} planner_rejects={} planner_rej_not_in_root={} planner_rej_missing_top={} planner_rej_top_not_tactical_or_unsafe={} planner_rej_top_wins={} planner_rej_candidate_unsafe={} planner_rej_progress_gate={} planner_rej_tactical_gate={} planner_rej_safety_progress_gate={} route_model_tactical={} route_drainer_score={} route_drainer_kill={} route_spirit_impact={} route_drainer_safety={} route_mana_move={} route_tactical_deny={} route_fallback={} expansions={})",
        candidate_profile,
        baseline_profile,
        repeats,
        games,
        max_plies,
        vn_delta,
        vn_conf,
        vf_delta,
        vf_conf,
        diag_vs_normal.intent_generation_calls,
        diag_vs_normal.intent_generation_hits,
        diag_vs_normal.compile_fallbacks,
        diag_vs_normal.compile_fallbacks_path,
        diag_vs_normal.compile_fallbacks_spirit,
        diag_vs_normal.compile_fallbacks_attack,
        diag_vs_normal.injected_root_candidates_seen,
        diag_vs_normal.injected_root_duplicates,
        diag_vs_normal.injected_root_attempts,
        diag_vs_normal.injected_root_accepts,
        diag_vs_normal.injected_root_reject_build,
        diag_vs_normal.injected_root_reject_emergency_guard,
        diag_vs_normal.injected_root_reject_emergency_introduced_loss,
        diag_vs_normal.injected_root_reject_emergency_no_crisis_signal,
        diag_vs_normal.injected_root_reject_emergency_mana_handoff,
        diag_vs_normal.injected_root_reject_emergency_drainer_unsafe,
        diag_vs_normal.injected_root_reject_top_wins,
        diag_vs_normal.injected_root_reject_candidate_unsafe,
        diag_vs_normal.injected_root_reject_no_tactical_signal,
        diag_vs_normal.injected_root_reject_heuristic_gap,
        diag_vs_normal.planner_choice_no_plan,
        diag_vs_normal.planner_choice_attempts,
        diag_vs_normal.planner_choice_accepts,
        diag_vs_normal.planner_choice_rejects,
        diag_vs_normal.planner_choice_reject_not_in_root,
        diag_vs_normal.planner_choice_reject_missing_top,
        diag_vs_normal.planner_choice_reject_top_not_tactical_or_unsafe,
        diag_vs_normal.planner_choice_reject_top_wins,
        diag_vs_normal.planner_choice_reject_candidate_unsafe,
        diag_vs_normal.planner_choice_reject_progress_gate,
        diag_vs_normal.planner_choice_reject_tactical_gate,
        diag_vs_normal.planner_choice_reject_safety_progress_gate,
        diag_vs_normal.route_model_tactical,
        diag_vs_normal.route_drainer_score,
        diag_vs_normal.route_drainer_kill,
        diag_vs_normal.route_spirit_impact,
        diag_vs_normal.route_drainer_safety,
        diag_vs_normal.route_mana_move,
        diag_vs_normal.route_tactical_deny,
        diag_vs_normal.route_fallback,
        diag_vs_normal.expansions,
        diag_vs_fast.intent_generation_calls,
        diag_vs_fast.intent_generation_hits,
        diag_vs_fast.compile_fallbacks,
        diag_vs_fast.compile_fallbacks_path,
        diag_vs_fast.compile_fallbacks_spirit,
        diag_vs_fast.compile_fallbacks_attack,
        diag_vs_fast.injected_root_candidates_seen,
        diag_vs_fast.injected_root_duplicates,
        diag_vs_fast.injected_root_attempts,
        diag_vs_fast.injected_root_accepts,
        diag_vs_fast.injected_root_reject_build,
        diag_vs_fast.injected_root_reject_emergency_guard,
        diag_vs_fast.injected_root_reject_emergency_introduced_loss,
        diag_vs_fast.injected_root_reject_emergency_no_crisis_signal,
        diag_vs_fast.injected_root_reject_emergency_mana_handoff,
        diag_vs_fast.injected_root_reject_emergency_drainer_unsafe,
        diag_vs_fast.injected_root_reject_top_wins,
        diag_vs_fast.injected_root_reject_candidate_unsafe,
        diag_vs_fast.injected_root_reject_no_tactical_signal,
        diag_vs_fast.injected_root_reject_heuristic_gap,
        diag_vs_fast.planner_choice_no_plan,
        diag_vs_fast.planner_choice_attempts,
        diag_vs_fast.planner_choice_accepts,
        diag_vs_fast.planner_choice_rejects,
        diag_vs_fast.planner_choice_reject_not_in_root,
        diag_vs_fast.planner_choice_reject_missing_top,
        diag_vs_fast.planner_choice_reject_top_not_tactical_or_unsafe,
        diag_vs_fast.planner_choice_reject_top_wins,
        diag_vs_fast.planner_choice_reject_candidate_unsafe,
        diag_vs_fast.planner_choice_reject_progress_gate,
        diag_vs_fast.planner_choice_reject_tactical_gate,
        diag_vs_fast.planner_choice_reject_safety_progress_gate,
        diag_vs_fast.route_model_tactical,
        diag_vs_fast.route_drainer_score,
        diag_vs_fast.route_drainer_kill,
        diag_vs_fast.route_spirit_impact,
        diag_vs_fast.route_drainer_safety,
        diag_vs_fast.route_mana_move,
        diag_vs_fast.route_tactical_deny,
        diag_vs_fast.route_fallback,
        diag_vs_fast.expansions
    );
    println!(
        "pro planner no-plan reasons normal(inactive_gate={} empty_plans={} build_no_plan={} budget_exceeded={} no_best_plan={} empty_best_plan={}) fast(inactive_gate={} empty_plans={} build_no_plan={} budget_exceeded={} no_best_plan={} empty_best_plan={})",
        diag_vs_normal.planner_no_plan_inactive_gate,
        diag_vs_normal.planner_no_plan_empty_plans,
        diag_vs_normal.planner_no_plan_build_no_plan,
        diag_vs_normal.planner_no_plan_budget_exceeded,
        diag_vs_normal.planner_no_plan_no_best_plan,
        diag_vs_normal.planner_no_plan_empty_best_plan,
        diag_vs_fast.planner_no_plan_inactive_gate,
        diag_vs_fast.planner_no_plan_empty_plans,
        diag_vs_fast.planner_no_plan_build_no_plan,
        diag_vs_fast.planner_no_plan_budget_exceeded,
        diag_vs_fast.planner_no_plan_no_best_plan,
        diag_vs_fast.planner_no_plan_empty_best_plan,
    );
}

#[test]
#[ignore = "diagnostic: probe pro turn engine diagnostics counters on duel lanes"]
fn smart_automove_pro_turn_engine_activity_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_release_safe_pre_exact".into());
    let repeats = env_usize("SMART_PRO_ACTIVITY_REPEATS").unwrap_or(1).max(1);
    let games = env_usize("SMART_PRO_ACTIVITY_GAMES").unwrap_or(1).max(1);
    let max_plies = env_usize("SMART_PRO_ACTIVITY_MAX_PLIES")
        .unwrap_or(56)
        .max(24);
    let seed_tag_vs_normal = env_profile_name("SMART_PRO_ACTIVITY_VS_NORMAL_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_engine_vs_normal_v1".to_string());
    let seed_tag_vs_fast = env_profile_name("SMART_PRO_ACTIVITY_VS_FAST_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_engine_vs_fast_v1".to_string());

    clear_turn_engine_diagnostics();
    let vs_normal = run_cross_budget_duel(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Normal,
        seed_tag: seed_tag_vs_normal.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let diag_vs_normal = turn_engine_diagnostics_snapshot();

    clear_turn_engine_diagnostics();
    let vs_fast = run_cross_budget_duel(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Fast,
        seed_tag: seed_tag_vs_fast.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let diag_vs_fast = turn_engine_diagnostics_snapshot();
    clear_turn_engine_diagnostics();

    let (vn_delta, vn_conf) = stats_delta_confidence(vs_normal);
    let (vf_delta, vf_conf) = stats_delta_confidence(vs_fast);
    println!(
        "pro turn engine activity candidate={} baseline={} repeats={} games={} max_plies={} vs_normal_delta={:.4} vs_normal_conf={:.4} vs_fast_delta={:.4} vs_fast_conf={:.4} normal(cache_hits={} cache_misses={} seed_immediate={} seed_deny={} seed_kill={} seed_supermana={} seed_opponent={} seed_safety={} seed_spirit={} seed_mana={} accepted={} accepted_families=[score:{} deny:{} kill:{} super:{} opp:{} safety:{} spirit:{} tempo:{}] compile_attempts={} compile_failures={} compile_fail_limit={} compile_mismatch={} compile_actions=[walk:{}/{} attack:{}/{} spirit:{}/{} bomb:{}/{} mana:{}/{} score:{}/{} retreat:{}/{}] reply_calls={} fallback_no_plan={} fallback_budget={}) fast(cache_hits={} cache_misses={} seed_immediate={} seed_deny={} seed_kill={} seed_supermana={} seed_opponent={} seed_safety={} seed_spirit={} seed_mana={} accepted={} accepted_families=[score:{} deny:{} kill:{} super:{} opp:{} safety:{} spirit:{} tempo:{}] compile_attempts={} compile_failures={} compile_fail_limit={} compile_mismatch={} compile_actions=[walk:{}/{} attack:{}/{} spirit:{}/{} bomb:{}/{} mana:{}/{} score:{}/{} retreat:{}/{}] reply_calls={} fallback_no_plan={} fallback_budget={})",
        candidate_profile,
        baseline_profile,
        repeats,
        games,
        max_plies,
        vn_delta,
        vn_conf,
        vf_delta,
        vf_conf,
        diag_vs_normal.cache_hits,
        diag_vs_normal.cache_misses,
        diag_vs_normal.seed_immediate_score,
        diag_vs_normal.seed_deny_window,
        diag_vs_normal.seed_drainer_kill,
        diag_vs_normal.seed_safe_supermana_progress,
        diag_vs_normal.seed_safe_opponent_mana_progress,
        diag_vs_normal.seed_safety_recovery,
        diag_vs_normal.seed_spirit_impact,
        diag_vs_normal.seed_mana_tempo,
        diag_vs_normal.accepted_plans,
        diag_vs_normal.accepted_immediate_score,
        diag_vs_normal.accepted_deny_window,
        diag_vs_normal.accepted_drainer_kill,
        diag_vs_normal.accepted_safe_supermana_progress,
        diag_vs_normal.accepted_safe_opponent_mana_progress,
        diag_vs_normal.accepted_safety_recovery,
        diag_vs_normal.accepted_spirit_impact,
        diag_vs_normal.accepted_mana_tempo,
        diag_vs_normal.compile_attempts,
        diag_vs_normal.compile_failures,
        diag_vs_normal.compile_failures_at_limit,
        diag_vs_normal.compile_state_mismatches,
        diag_vs_normal.compile_walk_attempts,
        diag_vs_normal.compile_walk_failures,
        diag_vs_normal.compile_attack_attempts,
        diag_vs_normal.compile_attack_failures,
        diag_vs_normal.compile_spirit_shift_attempts,
        diag_vs_normal.compile_spirit_shift_failures,
        diag_vs_normal.compile_bomb_attempts,
        diag_vs_normal.compile_bomb_failures,
        diag_vs_normal.compile_move_mana_attempts,
        diag_vs_normal.compile_move_mana_failures,
        diag_vs_normal.compile_score_attempts,
        diag_vs_normal.compile_score_failures,
        diag_vs_normal.compile_retreat_attempts,
        diag_vs_normal.compile_retreat_failures,
        diag_vs_normal.reply_search_calls,
        diag_vs_normal.fallback_no_plan,
        diag_vs_normal.fallback_budget_exceeded,
        diag_vs_fast.cache_hits,
        diag_vs_fast.cache_misses,
        diag_vs_fast.seed_immediate_score,
        diag_vs_fast.seed_deny_window,
        diag_vs_fast.seed_drainer_kill,
        diag_vs_fast.seed_safe_supermana_progress,
        diag_vs_fast.seed_safe_opponent_mana_progress,
        diag_vs_fast.seed_safety_recovery,
        diag_vs_fast.seed_spirit_impact,
        diag_vs_fast.seed_mana_tempo,
        diag_vs_fast.accepted_plans,
        diag_vs_fast.accepted_immediate_score,
        diag_vs_fast.accepted_deny_window,
        diag_vs_fast.accepted_drainer_kill,
        diag_vs_fast.accepted_safe_supermana_progress,
        diag_vs_fast.accepted_safe_opponent_mana_progress,
        diag_vs_fast.accepted_safety_recovery,
        diag_vs_fast.accepted_spirit_impact,
        diag_vs_fast.accepted_mana_tempo,
        diag_vs_fast.compile_attempts,
        diag_vs_fast.compile_failures,
        diag_vs_fast.compile_failures_at_limit,
        diag_vs_fast.compile_state_mismatches,
        diag_vs_fast.compile_walk_attempts,
        diag_vs_fast.compile_walk_failures,
        diag_vs_fast.compile_attack_attempts,
        diag_vs_fast.compile_attack_failures,
        diag_vs_fast.compile_spirit_shift_attempts,
        diag_vs_fast.compile_spirit_shift_failures,
        diag_vs_fast.compile_bomb_attempts,
        diag_vs_fast.compile_bomb_failures,
        diag_vs_fast.compile_move_mana_attempts,
        diag_vs_fast.compile_move_mana_failures,
        diag_vs_fast.compile_score_attempts,
        diag_vs_fast.compile_score_failures,
        diag_vs_fast.compile_retreat_attempts,
        diag_vs_fast.compile_retreat_failures,
        diag_vs_fast.reply_search_calls,
        diag_vs_fast.fallback_no_plan,
        diag_vs_fast.fallback_budget_exceeded,
    );
}

#[test]
fn runtime_pro_turn_engine_profile_uses_engine_on_immediate_score_fixture() {
    fn game_with_items(items: Vec<(Location, Item)>, active_color: Color) -> MonsGame {
        let mut game = MonsGame::new(false);
        game.board = Board::new_with_items(items.into_iter().collect());
        game.active_color = active_color;
        game.actions_used_count = 0;
        game.mana_moves_count = 0;
        game.mons_moves_count = 0;
        game.turn_number = 2;
        game.white_score = 0;
        game.black_score = 0;
        game.white_potions_count = 0;
        game.black_potions_count = 0;
        game
    }

    let game = game_with_items(
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
    );
    clear_turn_engine_plan_cache();
    let config = calibration_runtime_config(
        "runtime_pro_turn_engine_v1",
        &game,
        SmartAutomovePreference::Pro,
    );
    let inputs = model_runtime_pro_turn_engine_v1(&game, config);
    let probe = turn_engine_probe(
        &game,
        Color::White,
        config.turn_engine_mode,
        calibration_turn_engine_config(config),
    );

    assert!(
        !inputs.is_empty(),
        "turn engine profile should return legal inputs"
    );
    assert!(
        config.enable_turn_engine,
        "turn engine profile should enable the engine on Pro fixtures"
    );
    assert!(
        probe.status == TurnEngineProbeStatus::Planned && probe.candidate_chunk.is_some(),
        "turn engine probe should materialize a concrete plan on the immediate-score fixture: {:?}",
        probe
    );

    let (after, _) = MonsGameModel::apply_inputs_for_search_with_events(&game, inputs.as_slice())
        .expect("turn engine inputs should apply cleanly");
    assert!(
        after.white_score > game.white_score,
        "turn engine profile should convert the immediate score fixture"
    );
}

#[test]
fn runtime_pro_turn_engine_profile_caches_spirit_setup_continuation() {
    fn game_with_items(items: Vec<(Location, Item)>, active_color: Color) -> MonsGame {
        let mut game = MonsGame::new(false);
        game.board = Board::new_with_items(items.into_iter().collect());
        game.active_color = active_color;
        game.actions_used_count = 0;
        game.mana_moves_count = 0;
        game.mons_moves_count = 0;
        game.turn_number = 2;
        game.white_score = 0;
        game.black_score = 0;
        game.white_potions_count = 0;
        game.black_potions_count = 0;
        game
    }

    let game = game_with_items(
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
    );
    clear_turn_engine_plan_cache();
    let config = calibration_runtime_config(
        "runtime_pro_turn_engine_v1",
        &game,
        SmartAutomovePreference::Pro,
    );
    let engine_config = calibration_turn_engine_config(config);
    let first = model_runtime_pro_turn_engine_v1(&game, config);
    let after_first = MonsGameModel::apply_inputs_for_search(&game, first.as_slice())
        .expect("first spirit-setup chunk should be legal");
    let cached = turn_engine_cached_step(&after_first, engine_config);

    assert_eq!(
        first,
        vec![
            Input::Location(Location::new(9, 7)),
            Input::Location(Location::new(7, 8)),
            Input::Location(Location::new(7, 7)),
        ]
    );
    assert!(
        cached.is_some(),
        "spirit setup should cache the continuation after the selected first chunk"
    );
}

#[test]
fn runtime_pro_turn_engine_profile_selects_spirit_setup_opening() {
    fn game_with_items(items: Vec<(Location, Item)>, active_color: Color) -> MonsGame {
        let mut game = MonsGame::new(false);
        game.board = Board::new_with_items(items.into_iter().collect());
        game.active_color = active_color;
        game.actions_used_count = 0;
        game.mana_moves_count = 0;
        game.mons_moves_count = 0;
        game.turn_number = 2;
        game.white_score = 0;
        game.black_score = 0;
        game.white_potions_count = 0;
        game.black_potions_count = 0;
        game
    }

    let game = game_with_items(
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
    );
    clear_turn_engine_plan_cache();
    let config = calibration_runtime_config(
        "runtime_pro_turn_engine_v1",
        &game,
        SmartAutomovePreference::Pro,
    );
    let first = model_runtime_pro_turn_engine_v1(&game, config);

    assert_eq!(
        first,
        vec![
            Input::Location(Location::new(9, 7)),
            Input::Location(Location::new(7, 8)),
            Input::Location(Location::new(7, 7)),
        ]
    );
}

#[test]
#[ignore = "diagnostic: inspect arbitrary selector state via SMART_PROBE_FEN"]
fn smart_automove_pro_turn_engine_selector_probe() {
    let fen = std::env::var("SMART_PROBE_FEN").expect("SMART_PROBE_FEN should be set");
    let profile = std::env::var("SMART_PROBE_PROFILE")
        .unwrap_or_else(|_| "runtime_pro_turn_engine_v30".to_string());
    let preference = match std::env::var("SMART_PROBE_MODE")
        .unwrap_or_else(|_| "pro".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "fast" => SmartAutomovePreference::Fast,
        "normal" => SmartAutomovePreference::Normal,
        _ => SmartAutomovePreference::Pro,
    };
    let game = MonsGame::from_fen(fen.as_str(), false).expect("SMART_PROBE_FEN should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    clear_turn_engine_selector_diagnostics();
    clear_turn_planner_diagnostics();
    let decision = loss_probe_decision_with_options(profile.as_str(), preference, &game, true);
    println!("probe_profile={profile} probe_mode={preference:?}");
    println!(
        "turn_state turn={} mons_moves={} can_action={} can_move_mana={}",
        game.turn_number,
        game.mons_moves_count,
        game.player_can_use_action(),
        game.player_can_move_mana(),
    );
    println!(
        "selector_diag={:?}",
        turn_engine_selector_diagnostics_snapshot()
    );
    println!("planner_diag={:?}", turn_planner_diagnostics_snapshot());
    print_loss_probe_decision("decision", &decision);
    let config =
        probe_config_with_env_overrides(calibration_runtime_config(profile.as_str(), &game, preference));
    clear_turn_engine_selector_diagnostics();
    clear_turn_planner_diagnostics();
    let direct_runtime_inputs = MonsGameModel::smart_search_best_inputs(&game, config);
    println!(
        "direct_runtime_move={}",
        Input::fen_from_array(&direct_runtime_inputs)
    );
    println!(
        "direct_selector_diag={:?}",
        turn_engine_selector_diagnostics_snapshot()
    );
    println!("direct_planner_diag={:?}", turn_planner_diagnostics_snapshot());
    if let Some(selection_probe) = MonsGameModel::root_selection_probe_for_test(&game, config) {
        println!("root_selection_probe={:?}", selection_probe);
    } else {
        println!("root_selection_probe none");
    }
    if let Some(search_probe) = MonsGameModel::root_search_probe_for_test(&game, config) {
        println!("root_search_probe={:?}", search_probe);
    } else {
        println!("root_search_probe none");
    }
    if let Some(acceptance) = MonsGameModel::turn_engine_acceptance_probe_for_test(&game, config) {
        print_turn_engine_acceptance_probe("acceptance", Some(&acceptance));
    } else {
        println!("acceptance none");
    }
}

#[test]
#[ignore = "diagnostic: inspect arbitrary selector state via SMART_PROBE_FEN using loss-probe runtime config"]
fn smart_automove_pro_turn_engine_loss_probe_selector_probe() {
    let fen = std::env::var("SMART_PROBE_FEN").expect("SMART_PROBE_FEN should be set");
    let profile = std::env::var("SMART_PROBE_PROFILE")
        .unwrap_or_else(|_| "runtime_pro_turn_engine_v30".to_string());
    let preference = match std::env::var("SMART_PROBE_MODE")
        .unwrap_or_else(|_| "pro".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "fast" => SmartAutomovePreference::Fast,
        "normal" => SmartAutomovePreference::Normal,
        _ => SmartAutomovePreference::Pro,
    };
    let game = MonsGame::from_fen(fen.as_str(), false).expect("SMART_PROBE_FEN should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    clear_turn_engine_selector_diagnostics();
    clear_turn_planner_diagnostics();
    let decision = loss_probe_decision_with_options(profile.as_str(), preference, &game, true);
    println!("probe_profile={profile} probe_mode={preference:?}");
    println!(
        "turn_state turn={} mons_moves={} can_action={} can_move_mana={}",
        game.turn_number,
        game.mons_moves_count,
        game.player_can_use_action(),
        game.player_can_move_mana(),
    );
    println!(
        "selector_diag={:?}",
        turn_engine_selector_diagnostics_snapshot()
    );
    println!("planner_diag={:?}", turn_planner_diagnostics_snapshot());
    print_loss_probe_decision("decision", &decision);
    let config =
        probe_config_with_env_overrides(loss_probe_runtime_config(profile.as_str(), &game, preference));
    clear_turn_engine_selector_diagnostics();
    clear_turn_planner_diagnostics();
    let direct_runtime_inputs = MonsGameModel::smart_search_best_inputs(&game, config);
    println!(
        "direct_runtime_move={}",
        Input::fen_from_array(&direct_runtime_inputs)
    );
    println!(
        "direct_selector_diag={:?}",
        turn_engine_selector_diagnostics_snapshot()
    );
    println!("direct_planner_diag={:?}", turn_planner_diagnostics_snapshot());
    if let Some(selection_probe) = MonsGameModel::root_selection_probe_for_test(&game, config) {
        println!("root_selection_probe={:?}", selection_probe);
    } else {
        println!("root_selection_probe none");
    }
    if let Some(search_probe) = MonsGameModel::root_search_probe_for_test(&game, config) {
        println!("root_search_probe={:?}", search_probe);
    } else {
        println!("root_search_probe none");
    }
    if let Some(acceptance) = MonsGameModel::turn_engine_acceptance_probe_for_test(&game, config) {
        print_turn_engine_acceptance_probe("acceptance", Some(&acceptance));
    } else {
        println!("acceptance none");
    }
}

#[test]
#[ignore = "diagnostic: inspect raw ranked root moves via SMART_PROBE_FEN using loss-probe runtime config"]
fn smart_automove_ranked_root_moves_probe() {
    let fen = std::env::var("SMART_PROBE_FEN").expect("SMART_PROBE_FEN should be set");
    let profile = std::env::var("SMART_PROBE_PROFILE")
        .unwrap_or_else(|_| "runtime_pro_turn_engine_v30".to_string());
    let preference = match std::env::var("SMART_PROBE_MODE")
        .unwrap_or_else(|_| "pro".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "fast" => SmartAutomovePreference::Fast,
        "normal" => SmartAutomovePreference::Normal,
        _ => SmartAutomovePreference::Pro,
    };
    let game = MonsGame::from_fen(fen.as_str(), false).expect("SMART_PROBE_FEN should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let config =
        probe_config_with_env_overrides(loss_probe_runtime_config(profile.as_str(), &game, preference));
    let ranked_roots = MonsGameModel::ranked_root_moves(&game, game.active_color, config);

    println!("probe_profile={profile} probe_mode={preference:?}");
    println!(
        "turn_state turn={} mons_moves={} can_action={} can_move_mana={}",
        game.turn_number,
        game.mons_moves_count,
        game.player_can_use_action(),
        game.player_can_move_mana(),
    );
    println!(
        "config root={} enum={} node={} two_pass={} focus={}/{}",
        config.root_branch_limit,
        config.root_enum_limit,
        config.node_branch_limit,
        config.enable_two_pass_root_allocation,
        config.root_focus_k,
        config.root_focus_budget_share_bp,
    );
    println!("ranked_root_count={}", ranked_roots.len());
    for (rank, root) in ranked_roots.iter().enumerate() {
        println!(
            "rank={} fen={} heuristic={} eff={} vuln={} walk_vuln={} handoff={} roundtrip={} supermana_progress={} opponent_progress={} score_window={} spirit_setup={} spirit_dev={}",
            rank,
            Input::fen_from_array(&root.inputs),
            root.heuristic,
            root.efficiency,
            root.own_drainer_vulnerable,
            root.own_drainer_walk_vulnerable,
            root.mana_handoff_to_opponent,
            root.has_roundtrip,
            root.supermana_progress,
            root.opponent_mana_progress,
            root.same_turn_score_window_value,
            root.spirit_own_mana_setup_now || root.spirit_same_turn_score_setup_now,
            root.spirit_development,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect ranked child ordering after applying SMART_PROBE_ROOT_INPUT_FEN"]
fn smart_automove_ranked_child_states_probe() {
    let fen = std::env::var("SMART_PROBE_FEN").expect("SMART_PROBE_FEN should be set");
    let root_input_fen =
        std::env::var("SMART_PROBE_ROOT_INPUT_FEN").expect("SMART_PROBE_ROOT_INPUT_FEN should be set");
    let profile = std::env::var("SMART_PROBE_PROFILE")
        .unwrap_or_else(|_| "runtime_pro_turn_engine_v30".to_string());
    let preference = match std::env::var("SMART_PROBE_MODE")
        .unwrap_or_else(|_| "pro".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "fast" => SmartAutomovePreference::Fast,
        "normal" => SmartAutomovePreference::Normal,
        _ => SmartAutomovePreference::Pro,
    };
    let game = MonsGame::from_fen(fen.as_str(), false).expect("SMART_PROBE_FEN should be valid");
    let perspective = game.active_color;
    let config =
        probe_config_with_env_overrides(loss_probe_runtime_config(profile.as_str(), &game, preference));
    let root_inputs = Input::array_from_fen(root_input_fen.as_str());
    let (after_root, _) = MonsGameModel::apply_inputs_for_search_with_events(&game, &root_inputs)
        .expect("SMART_PROBE_ROOT_INPUT_FEN should apply cleanly");
    let maximizing = after_root.active_color == perspective;
    let probe = MonsGameModel::ranked_child_ordering_probe_for_test(
        &after_root,
        perspective,
        maximizing,
        config,
    );

    println!("probe_profile={profile} probe_mode={preference:?}");
    println!("root_input_fen={root_input_fen}");
    println!(
        "after_root turn={} mons_moves={} active={:?} maximizing={}",
        after_root.turn_number,
        after_root.mons_moves_count,
        after_root.active_color,
        maximizing,
    );
    println!("ranked_child_count={}", probe.len());
    for (rank, child) in probe.iter().enumerate() {
        println!(
            "rank={} fen={} heuristic={} eff={} ext={} tactical={} quiet_red={} quiet={} material={} carrier_progress={}",
            rank,
            child.input_fen,
            child.heuristic,
            child.ordering_efficiency,
            child.selective_extension_candidate,
            child.tactical_extension_trigger,
            child.quiet_reduction_candidate,
            child.quiet,
            child.material,
            child.carrier_progress,
        );
    }
}

#[test]
#[ignore = "diagnostic: bounded selector/exact hotspot probe for pro reliability corpus"]
fn smart_automove_pro_reliability_hotspot_probe() {
    use std::collections::HashMap;
    use std::time::Instant;

    #[derive(Clone)]
    struct ProbeCase {
        label: &'static str,
        game: MonsGame,
        mode: SmartAutomovePreference,
        opening_book_driven: bool,
        config_tweak: Option<fn(SmartSearchConfig) -> SmartSearchConfig>,
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
        let mut game = MonsGame::new(false);
        game.board = Board::new_with_items(items.into_iter().collect::<HashMap<_, _>>());
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

    let profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".to_string());
    let selector = profile_selector_from_name(profile.as_str())
        .unwrap_or_else(|| panic!("profile '{}' not found", profile));

    let cases = vec![
        probe_case_from_fixture(
            "primary_spirit_setup",
            primary_pro_fixture_by_id("primary_spirit_setup"),
        ),
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
        ProbeCase {
            label: "drainer_risk",
            game: game_with_items(
                vec![
                    (
                        Location::new(7, 5),
                        Item::Mon {
                            mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        },
                    ),
                    (
                        Location::new(5, 3),
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
            ),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
        },
        ProbeCase {
            label: "exact_progress",
            game: game_with_items(
                vec![
                    (
                        Location::new(7, 5),
                        Item::Mon {
                            mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        },
                    ),
                    (
                        Location::new(6, 5),
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
            ),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
        },
        ProbeCase {
            label: "spirit_development",
            game: game_with_items(
                vec![
                    (
                        Location::new(10, 5),
                        Item::Mon {
                            mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        },
                    ),
                    (
                        Location::new(10, 6),
                        Item::Mon {
                            mon: Mon::new(MonKind::Spirit, Color::White, 0),
                        },
                    ),
                    (
                        Location::new(1, 5),
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
        "pro reliability hotspot probe: profile={} positions={}",
        profile,
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
                clear_exact_state_analysis_cache();
                clear_exact_query_diagnostics();
                clear_turn_engine_plan_cache();
                clear_turn_engine_diagnostics();
                clear_turn_engine_selector_diagnostics();

                let base_config = case
                    .config_tweak
                    .map(|tweak| {
                        tweak(
                            SearchBudget::from_preference(case.mode)
                                .runtime_config_for_game(&case.game),
                        )
                    })
                    .unwrap_or_else(|| {
                        SearchBudget::from_preference(case.mode).runtime_config_for_game(&case.game)
                    });
                let start = Instant::now();
                let inputs = select_inputs_with_runtime_fallback(selector, &case.game, base_config);
                let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                let selector_diag = turn_engine_selector_diagnostics_snapshot();
                let engine_diag = turn_engine_diagnostics_snapshot();
                let exact_diag = exact_query_diagnostics_snapshot();

                println!(
                    "HOTSPOT label={} move={} ms={:.2} selector(child_calls={} children={} fully_scored={} shortlist={} full_pass={} move_eff_builds={} move_eff_hits={} prefer_builds={} prefer_hits={} head_calls={} head_hits={} last_stage={}) exact(attack_summary_builds={} attack_calls={} attack_hits={} threat_calls={} payload_calls={} tactical_spirit_calls={} tactical_spirit_hits={} immediate_window_queries={} tactical_window_calls={} secure_mana_calls={} secure_mana_hits={} pickup_calls={} pickup_hits={}) engine(cache_hits={} cache_misses={} accepted={} reply_calls={}) fen={}",
                    case.label,
                    Input::fen_from_array(&inputs),
                    elapsed_ms,
                    selector_diag.ranked_child_states_calls,
                    selector_diag.ranked_child_states_children_enumerated,
                    selector_diag.ranked_child_states_children_fully_scored,
                    selector_diag.child_ordering_shortlist_children,
                    selector_diag.child_ordering_full_pass_children,
                    selector_diag.move_efficiency_snapshot_builds,
                    selector_diag.move_efficiency_snapshot_cache_hits,
                    selector_diag.search_preferability_builds,
                    selector_diag.search_preferability_cache_hits,
                    selector_diag.head_plan_calls,
                    selector_diag.head_plan_hits,
                    selector_diag.last_return_stage,
                    exact_diag.attack_reach_summary_builds,
                    exact_diag.attack_reach_calls,
                    exact_diag.attack_reach_cache_hits,
                    exact_diag.drainer_immediate_threat_calls,
                    exact_diag.actor_payload_after_move_calls,
                    exact_diag.tactical_spirit_summary_calls,
                    exact_diag.tactical_spirit_summary_cache_hits,
                    exact_diag.immediate_tactical_window_queries,
                    exact_diag.tactical_spirit_after_window_calls,
                    exact_diag.exact_secure_mana_calls,
                    exact_diag.exact_secure_mana_cache_hits,
                    exact_diag.pickup_path_calls,
                    exact_diag.pickup_path_cache_hits,
                    engine_diag.cache_hits,
                    engine_diag.cache_misses,
                    engine_diag.accepted_plans,
                    engine_diag.reply_search_calls,
                    case.game.fen(),
                );
                assert!(
                    !inputs.is_empty(),
                    "hotspot probe profile '{}' produced no legal move for '{}'",
                    profile,
                    case.label
                );
            },
        );
    }
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_safe_white_fast_screen_turn_one_tail_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 2 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xS0xn05/n03E0xA0xn02Y0xn03",
        false,
    )
    .expect("fast-screen turn-one white tail fen should be valid");
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    assert_eq!(
        decision.move_fen, "l10,3;l9,3",
        "v30 should route the traced white turn-one opening tail blocker to the shared current line, got {}",
        decision.move_fen
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_three_move_opening_tail() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 3 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xn01S0xn04/n02E0xn01A0xn02Y0xn03",
        false,
    )
    .expect("white three-move opening tail fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    assert_eq!(decision.move_fen, "l10,7;l9,7");
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_safe_white_fast_screen_turn_three_start_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 0 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n04D0xn06/n02E0xA0xn01S0xn05/n07Y0xn03",
        false,
    )
    .expect("fast-screen turn-three white start fen should be valid");
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    assert_eq!(
        decision.move_fen, "l8,4;l8,5",
        "v30 should route the traced white turn-three start fast-screen blocker to the shared fast/current line, got {}",
        decision.move_fen
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_turn_three_start_action_mana_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 0 0 0 3 n03y0xn03e0xn03/n05s0xa0xn01d0mn02/n11/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n07Y0xn03/n04D0xn01S0xn04/n02E0xn01A0xn06",
        false,
    )
    .expect("white turn-three start action+mana loss fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    assert_eq!(decision.move_fen, "l9,4;l8,4");
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_turn_three_start_action_mana_variant() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 0 0 0 3 n03y0xn03e0xn03/n05s0xa0xn01d0mn02/n11/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n01E0xn09/n04D0xn01S0xn01Y0xn02/n04A0xn06",
        false,
    )
    .expect("white turn-three variant loss fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    assert_eq!(decision.move_fen, "l9,8;l8,7");
}

#[test]
fn runtime_pro_turn_engine_v30_profile_keeps_current_planner_on_engine_disabled_opening() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 2 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04E0xn01D0xn04/n04A0xn01S0xY0xn03",
        false,
    )
    .expect("engine-disabled opening planner fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    assert_eq!(decision.move_fen, "l9,6;l8,6");
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_turn_two_start_root() {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 2 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n01E0xn09/n04D0xn01S0xn01Y0xn02/n04A0xn06",
        false,
    )
    .expect("black turn-two opening start loss fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    assert_eq!(decision.move_fen, "l0,4;l1,5");
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_turn_two_mana_only_root() {
    let game = MonsGame::from_fen(
        "0 0 b 1 0 2 0 0 2 n03y0xn02a0xe0xn03/n05s0xd0xn04/n07xxmn03/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n04D0xn06/n02E0xA0xn01S0xn05/n07Y0xn03",
        false,
    )
    .expect("black turn-two mana-only loss fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    assert_eq!(decision.move_fen, "l0,3;l1,3");
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_turn_four_start_action_mana_root() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 0 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n05S0xn05/n04E0xA0xn05/n07Y0xn02D0x",
        false,
    )
    .expect("black turn-four action+mana loss fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    assert_eq!(decision.move_fen, "l1,5;l3,3;l2,2");
}

#[test]
fn runtime_pro_turn_engine_v30_profile_resumes_cached_spirit_setup_continuation() {
    fn game_with_items(items: Vec<(Location, Item)>, active_color: Color) -> MonsGame {
        let mut game = MonsGame::new(false);
        game.board = Board::new_with_items(items.into_iter().collect());
        game.active_color = active_color;
        game.actions_used_count = 0;
        game.mana_moves_count = 0;
        game.mons_moves_count = 0;
        game.turn_number = 2;
        game.white_score = 0;
        game.black_score = 0;
        game.white_potions_count = 0;
        game.black_potions_count = 0;
        game
    }

    let game = game_with_items(
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
    );
    clear_turn_engine_plan_cache();

    let config = calibration_runtime_config(
        "runtime_pro_turn_engine_v30",
        &game,
        SmartAutomovePreference::Pro,
    );
    let first = model_runtime_pro_turn_engine_v30(&game, config);
    let after_first = MonsGameModel::apply_inputs_for_search(&game, first.as_slice())
        .expect("v30 first spirit-setup chunk should be legal");
    let after_config = calibration_runtime_config(
        "runtime_pro_turn_engine_v30",
        &after_first,
        SmartAutomovePreference::Pro,
    );
    let cached =
        turn_engine_cached_step(&after_first, calibration_turn_engine_config(after_config))
            .expect("v30 should seed a cached continuation after the first spirit-setup chunk");
    let resumed = model_runtime_pro_turn_engine_v30(&after_first, after_config);

    assert_eq!(
        resumed, cached,
        "v30 should resume the cached continuation on the post-chunk live state"
    );
}

fn primary_pro_fixture_by_id(id: &str) -> TriageFixture {
    primary_pro_triage_fixtures()
        .into_iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("primary_pro fixture '{}' not found", id))
}

#[test]
fn runtime_pro_turn_engine_v30_accepts_primary_spirit_setup_macro_head() {
    let fixture = primary_pro_fixture_by_id("primary_spirit_setup");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let config =
        loss_probe_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let probe = MonsGameModel::turn_engine_acceptance_probe_for_test(&fixture.game, config)
        .expect("v30 spirit setup fixture should produce an acceptance probe");

    assert!(
        probe.accepted,
        "spirit setup head should stay accepted: {:?}",
        probe
    );
    assert_eq!(probe.candidate_family, TurnPlanFamily::SpiritImpact);
    assert!(
        probe.chunk_count >= 4,
        "spirit setup should remain a whole-turn bundle"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_black_opening_a_ply19_macro_head() {
    let fixture = primary_pro_fixture_by_id("primary_black_loss_opening_a_ply19");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let config =
        loss_probe_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let probe = MonsGameModel::turn_engine_acceptance_probe_for_test(&fixture.game, config)
        .expect("v30 black opening fixture should produce an acceptance probe");

    assert_eq!(Input::fen_from_array(&probe.candidate_inputs), "l2,5;l2,6");
    assert!(
        !probe.accepted,
        "v30 should reject the unsafe black-opening macro head: {:?}",
        probe
    );
}

#[test]
fn runtime_pro_turn_engine_v30_prefers_safe_black_opening_a_ply19_root() {
    let fixture = primary_pro_fixture_by_id("primary_black_loss_opening_a_ply19");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game);

    assert_eq!(decision.move_fen, "l2,5;l1,4");
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_black_plain_spirit_followup_macro_head() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 1 0 0 4 n05d0xa0xn04/n05s0xxxme0xn03/n11/n04xxmn06/n02y0xxxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n06S0xn04/n02E0xn01A0xn03Y0xn02/D0xn10",
        false,
    )
    .expect("valid black plain spirit followup fixture fen");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let config = loss_probe_runtime_config(
        "runtime_pro_turn_engine_v30",
        &game,
        SmartAutomovePreference::Pro,
    );
    let probe = MonsGameModel::turn_engine_acceptance_probe_for_test(&game, config)
        .expect("v30 black plain spirit followup fixture should produce an acceptance probe");

    assert_eq!(Input::fen_from_array(&probe.candidate_inputs), "l1,5;l1,7;l0,7");
    assert!(
        !probe.accepted,
        "v30 should reject the black plain spirit followup macro head when the searched safe root projects a comparable followup: {:?}",
        probe
    );
}

#[test]
fn runtime_pro_turn_engine_v30_prefers_safe_black_plain_spirit_followup_root() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 1 0 0 4 n05d0xa0xn04/n05s0xxxme0xn03/n11/n04xxmn06/n02y0xxxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n06S0xn04/n02E0xn01A0xn03Y0xn02/D0xn10",
        false,
    )
    .expect("valid black plain spirit followup fixture fen");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision =
        loss_probe_decision("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game);

    assert_eq!(decision.move_fen, "l4,2;l5,1");
}

#[test]
fn runtime_pro_turn_engine_v30_reply_guard_prefers_concrete_white_spirit_followup_setup() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 5 0 0 3 n05d2xa0xn04/n05s0xn01e0xn03/n03y0xn03xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn01S0xn01/xxQn04xxUn05/n03xxMn01xxMn01xxMn03/n04D0Mn01xxMn04/n11/n04A0xn06/n03E0xn03Y0xn03",
        false,
    )
    .expect("valid white spirit followup fixture fen");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let config = loss_probe_runtime_config(
        "runtime_pro_turn_engine_v30",
        &game,
        SmartAutomovePreference::Pro,
    );
    let probe = MonsGameModel::root_selection_probe_for_test(&game, config)
        .expect("v30 white followup fixture should produce a root selection probe");

    assert_eq!(
        probe.reply_guard_selected_move_fen.as_deref(),
        Some("l4,9;l4,7;l5,7"),
    );
    assert_eq!(probe.final_selected_move_fen, "l4,9;l4,7;l5,7");
}

#[test]
fn runtime_pro_turn_engine_v30_prefers_concrete_white_spirit_followup_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 5 0 0 3 n05d2xa0xn04/n05s0xn01e0xn03/n03y0xn03xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn01S0xn01/xxQn04xxUn05/n03xxMn01xxMn01xxMn03/n04D0Mn01xxMn04/n11/n04A0xn06/n03E0xn03Y0xn03",
        false,
    )
    .expect("valid white spirit followup fixture fen");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision = loss_probe_decision("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game);

    assert_eq!(decision.move_fen, "l4,9;l4,7;l5,7");
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_white_progress_tail_macro_head() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 7 n11/n05d0xa0xn01e0xn02/n06s0xS0xxxmn02/n02xxmxxmxxmn06/n08xxmn02/y0xn04xxUn05/n05xxMn01xxMn03/n04xxMn01xxMn04/n01E0xxxMn08/n04A0xD0xY0xn04/n11",
        false,
    )
    .expect("valid white progress tail fixture fen");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let config = loss_probe_runtime_config(
        "runtime_pro_turn_engine_v30",
        &game,
        SmartAutomovePreference::Pro,
    );
    let probe = MonsGameModel::turn_engine_acceptance_probe_for_test(&game, config)
        .expect("v30 white progress tail fixture should produce an acceptance probe");

    assert_eq!(Input::fen_from_array(&probe.candidate_inputs), "l9,5;l8,5");
    assert!(
        !probe.accepted,
        "v30 should reject the tied white progress-tail macro head: {:?}",
        probe
    );
}

#[test]
fn runtime_pro_turn_engine_v30_prefers_searched_white_progress_tail_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 7 n11/n05d0xa0xn01e0xn02/n06s0xS0xxxmn02/n02xxmxxmxxmn06/n08xxmn02/y0xn04xxUn05/n05xxMn01xxMn03/n04xxMn01xxMn04/n01E0xxxMn08/n04A0xD0xY0xn04/n11",
        false,
    )
    .expect("valid white progress tail fixture fen");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let decision =
        loss_probe_decision("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game);

    assert_eq!(decision.move_fen, "l9,5;l8,4");
}

#[test]
fn runtime_pro_turn_engine_v30_surfaces_multi_chunk_human_win_plan() {
    let fixture = primary_pro_fixture_by_id("human_win_pro_a");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let config =
        loss_probe_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let digests = turn_engine_ranked_plan_digests_for_test(
        &fixture.game,
        fixture.game.active_color,
        calibration_turn_engine_config(config),
        3,
    );

    assert!(
        !digests.is_empty(),
        "v30 should surface at least one whole-turn plan on human_win_pro_a"
    );
    assert!(
        digests[0].chunk_count >= 4,
        "human_win_pro_a should plan a multi-chunk turn, got {:?}",
        digests[0]
    );
    assert_eq!(digests[0].goal_family, TurnPlanFamily::ImmediateScore);
}

#[test]
fn runtime_pro_turn_engine_v30_accepts_human_win_macro_head() {
    let fixture = primary_pro_fixture_by_id("human_win_pro_a");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let config =
        loss_probe_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let probe = MonsGameModel::turn_engine_acceptance_probe_for_test(&fixture.game, config)
        .expect("v30 human win fixture should now inject and score the macro head");

    assert_eq!(Input::fen_from_array(&probe.candidate_inputs), "l8,5;l7,4");
    assert!(
        probe.accepted,
        "v30 human win macro head should survive selector acceptance: {:?}",
        probe
    );
    assert!(
        probe.chunk_count >= 4,
        "human win should stay a whole-turn bundle"
    );
}

#[test]
#[ignore = "diagnostic: dominance probe for runtime_pro_turn_engine_v30 against runtime_current on curated opportunity fixtures"]
fn runtime_pro_turn_engine_v30_curated_dominance_probe() {
    let fixture_ids = [
        "primary_spirit_setup",
        "primary_black_loss_opening_a_ply19",
        "human_win_pro_a",
    ];
    let mut wins = 0usize;
    let mut total = 0usize;

    for fixture_id in fixture_ids {
        let fixture = primary_pro_fixture_by_id(fixture_id);
        let candidate_is_white = fixture.game.active_color == Color::White;
        let (result, traces) = replay_pro_reliability_loss_probe_game_with_options(
            "runtime_pro_turn_engine_v30",
            "runtime_current",
            fixture.game.fen().as_str(),
            candidate_is_white,
            56,
            2,
            true,
        );
        total += 1;
        if result == MatchResult::CandidateWin {
            wins += 1;
        }
        println!(
            "dominance fixture={} result={:?} traces_logged={}",
            fixture_id,
            result,
            traces.len()
        );
    }

    let win_rate = wins as f64 / total.max(1) as f64;
    assert!(
        win_rate > 0.90,
        "v30 should beat runtime_current on the curated opportunity pack (>90%), got {:.3}",
        win_rate
    );
}

#[test]
#[ignore = "diagnostic: inspect candidate selection on the fast primary neutral v3 ply10 blocker"]
fn smart_automove_pro_primary_fast_neutral_v3_ply10_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let game = MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn01Y0xn02/n11/n03A0xD0xS0xn05/n03E0xn07",
        false,
    )
    .expect("primary fast neutral v3 ply10 fen should be valid");
    let exact = crate::models::automove_exact::exact_opportunity_context(&game, game.active_color);
    eprintln!(
        "state turn={} mons_moves={} can_action={} can_mana={} score_window={} deny_gain={} drainer_attack={} drainer_safety={} safe_super={:?} safe_opp={:?}",
        game.turn_number,
        game.mons_moves_count,
        game.player_can_use_action(),
        game.player_can_move_mana(),
        exact.delta.same_turn_score_window_value,
        exact.delta.opponent_window_deny_gain,
        exact.delta.drainer_attack_available,
        exact.delta.drainer_safety,
        exact.delta.safe_supermana_progress_steps,
        exact.delta.safe_opponent_mana_progress_steps,
    );
    let decision = loss_probe_decision_with_options(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        &game,
        true,
    );
    print_loss_probe_decision("candidate", &decision);
}

#[test]
#[ignore = "diagnostic: inspect candidate selection on the fast primary neutral v3 ply14 blocker"]
fn smart_automove_pro_primary_fast_neutral_v3_ply14_probe() {
    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".into());
    let game = MonsGame::from_fen(
        "0 0 w 1 0 4 1 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04Y0x/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn01D0xn05/n03A0xn01S0xn05/n03E0xn07",
        false,
    )
    .expect("primary fast neutral v3 ply14 fen should be valid");
    let exact = crate::models::automove_exact::exact_opportunity_context(&game, game.active_color);
    eprintln!(
        "state turn={} mons_moves={} can_action={} can_mana={} score_window={} deny_gain={} drainer_attack={} drainer_safety={} safe_super={:?} safe_opp={:?}",
        game.turn_number,
        game.mons_moves_count,
        game.player_can_use_action(),
        game.player_can_move_mana(),
        exact.delta.same_turn_score_window_value,
        exact.delta.opponent_window_deny_gain,
        exact.delta.drainer_attack_available,
        exact.delta.drainer_safety,
        exact.delta.safe_supermana_progress_steps,
        exact.delta.safe_opponent_mana_progress_steps,
    );
    let decision = loss_probe_decision_with_options(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        &game,
        true,
    );
    print_loss_probe_decision("candidate", &decision);
}

#[test]
fn runtime_pro_turn_engine_profile_selects_ext_sensitive_spirit_head() {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 2 0 0 6 n05d0xn01e0xn03/n03y0xn07/n02s0xn02xxmn01a0xn03/n11/n03xxmxxmxxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMxxMn02xxMn03/n06xxMn04/n04E0xn06/n03D0xxxMA0xS0xn02Y0xn01/n11",
        false,
    )
    .expect("ext-sensitive spirit fixture fen should be valid");
    let config = calibration_runtime_config(
        "runtime_pro_turn_engine_v1",
        &game,
        SmartAutomovePreference::Pro,
    );
    let first = model_runtime_pro_turn_engine_v1(&game, config);

    let move_fen = Input::fen_from_array(&first);
    assert!(
        matches!(move_fen.as_str(), "l2,2;l4,3;l4,2" | "l2,2;l4,3;l3,2"),
        "ext-sensitive spirit head should stay on the concrete spirit lane, got {}",
        move_fen
    );
}

#[test]
#[ignore = "diagnostic: probe pro pool non-regression lanes without ladder asserts"]
fn smart_automove_pro_pool_lane_probe() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let pool_games = env_usize("SMART_PRO_GATE_POOL_GAMES").unwrap_or(1).max(1);
    let pool_max_plies = env_usize("SMART_PRO_GATE_POOL_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let pro_vs_normal_tag = env::var("SMART_PRO_POOL_PROBE_CANDIDATE_NORMAL_TAG")
        .unwrap_or_else(|_| "pro_pool_vs_normal".to_string());
    let baseline_vs_normal_tag = env::var("SMART_PRO_POOL_PROBE_BASELINE_NORMAL_TAG")
        .unwrap_or_else(|_| "baseline_pool_vs_normal".to_string());
    let pro_vs_fast_tag = env::var("SMART_PRO_POOL_PROBE_CANDIDATE_FAST_TAG")
        .unwrap_or_else(|_| "pro_pool_vs_fast".to_string());
    let baseline_vs_fast_tag = env::var("SMART_PRO_POOL_PROBE_BASELINE_FAST_TAG")
        .unwrap_or_else(|_| "baseline_pool_vs_fast".to_string());

    let candidate_pool_vs_normal = run_profile_vs_pool_cross_budget(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        SmartAutomovePreference::Normal,
        pool_games,
        pool_max_plies,
        pro_vs_normal_tag.as_str(),
    );
    let baseline_pool_vs_normal = run_profile_vs_pool_cross_budget(
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        SmartAutomovePreference::Normal,
        pool_games,
        pool_max_plies,
        baseline_vs_normal_tag.as_str(),
    );
    let candidate_pool_vs_fast = run_profile_vs_pool_cross_budget(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        SmartAutomovePreference::Fast,
        pool_games,
        pool_max_plies,
        pro_vs_fast_tag.as_str(),
    );
    let baseline_pool_vs_fast = run_profile_vs_pool_cross_budget(
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        SmartAutomovePreference::Fast,
        pool_games,
        pool_max_plies,
        baseline_vs_fast_tag.as_str(),
    );

    let candidate_vs_normal = stats_delta_confidence(candidate_pool_vs_normal).0;
    let baseline_vs_normal = stats_delta_confidence(baseline_pool_vs_normal).0;
    let candidate_vs_fast = stats_delta_confidence(candidate_pool_vs_fast).0;
    let baseline_vs_fast = stats_delta_confidence(baseline_pool_vs_fast).0;

    println!(
        "pro pool probe candidate={} baseline={} games={} max_plies={} candidate_vs_normal={:.4} baseline_vs_normal={:.4} margin_vs_normal={:.4} candidate_vs_fast={:.4} baseline_vs_fast={:.4} margin_vs_fast={:.4}",
        candidate_profile,
        baseline_profile,
        pool_games,
        pool_max_plies,
        candidate_vs_normal,
        baseline_vs_normal,
        candidate_vs_normal - baseline_vs_normal,
        candidate_vs_fast,
        baseline_vs_fast,
        candidate_vs_fast - baseline_vs_fast,
    );

    if env_bool("SMART_PRO_POOL_PROBE_BREAKDOWN").unwrap_or(false) {
        let Some(candidate_selector) = profile_selector_from_name(candidate_profile.as_str())
        else {
            panic!(
                "candidate selector '{}' should exist for pool probe breakdown",
                candidate_profile
            );
        };
        let Some(baseline_selector) = profile_selector_from_name(baseline_profile.as_str()) else {
            panic!(
                "baseline selector '{}' should exist for pool probe breakdown",
                baseline_profile
            );
        };
        let candidate_model = AutomoveModel {
            id: "pro_pool_probe_candidate",
            select_inputs: candidate_selector,
        };
        let baseline_model = AutomoveModel {
            id: "pro_pool_probe_baseline",
            select_inputs: baseline_selector,
        };
        let pro_budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);
        let normal_budget = SearchBudget::from_preference(SmartAutomovePreference::Normal);
        let opponents = selected_pool_models();

        let original_opening_book = env::var("SMART_USE_WHITE_OPENING_BOOK").ok();
        env::set_var("SMART_USE_WHITE_OPENING_BOOK", "false");
        for (opponent_index, opponent) in opponents.iter().copied().enumerate() {
            let candidate_seed = seed_for_budget_duel_repeat_and_tag(
                pro_budget,
                normal_budget,
                opponent_index,
                format!("{}:pool_{}", pro_vs_normal_tag, opponent_index).as_str(),
            );
            let baseline_seed = seed_for_budget_duel_repeat_and_tag(
                normal_budget,
                normal_budget,
                opponent_index,
                format!("{}:pool_{}", baseline_vs_normal_tag, opponent_index).as_str(),
            );

            let candidate_ab = run_budget_duel_series(
                candidate_model,
                pro_budget,
                opponent,
                normal_budget,
                pool_games,
                candidate_seed,
                pool_max_plies,
            );
            let candidate_ba = run_budget_duel_series(
                opponent,
                normal_budget,
                candidate_model,
                pro_budget,
                pool_games,
                candidate_seed,
                pool_max_plies,
            );
            let baseline_ab = run_budget_duel_series(
                baseline_model,
                normal_budget,
                opponent,
                normal_budget,
                pool_games,
                baseline_seed,
                pool_max_plies,
            );
            let baseline_ba = run_budget_duel_series(
                opponent,
                normal_budget,
                baseline_model,
                normal_budget,
                pool_games,
                baseline_seed,
                pool_max_plies,
            );

            let candidate_stats = mirrored_candidate_stats(candidate_ab, candidate_ba);
            let baseline_stats = mirrored_candidate_stats(baseline_ab, baseline_ba);
            let candidate_delta = candidate_stats.win_rate_points() - 0.5;
            let baseline_delta = baseline_stats.win_rate_points() - 0.5;
            println!(
                "pro pool probe breakdown opponent={} candidate_delta={:.4} baseline_delta={:.4} margin={:.4} candidate_stats={:?} baseline_stats={:?}",
                opponent.id,
                candidate_delta,
                baseline_delta,
                candidate_delta - baseline_delta,
                candidate_stats,
                baseline_stats,
            );
        }
        if let Some(previous) = original_opening_book {
            env::set_var("SMART_USE_WHITE_OPENING_BOOK", previous);
        } else {
            env::remove_var("SMART_USE_WHITE_OPENING_BOOK");
        }
    }
}

#[test]
fn smart_automove_pool_profile_registry_resolves_retained_profiles() {
    for profile_id in retained_profile_ids() {
        assert!(
            profile_selector_from_name(profile_id).is_some(),
            "retained profile '{}' should resolve",
            profile_id
        );
    }
}

fn assert_runtime_preflight_if_required(
    candidate_profile_name: &str,
    candidate_selector: AutomoveSelector,
) {
    let skip_runtime_preflight = env_bool("SMART_SKIP_RUNTIME_PREFLIGHT").unwrap_or(false);
    if skip_runtime_preflight {
        println!(
            "runtime preflight skipped for duel stage candidate={}",
            candidate_profile_name
        );
    }
    maybe_run_runtime_preflight_checks(
        skip_runtime_preflight,
        || assert_stage1_cpu_non_regression(candidate_profile_name, candidate_selector),
        || {
            assert_exact_lite_diagnostics_gate_if_enabled(
                candidate_profile_name,
                candidate_selector,
            )
        },
    );
}

#[test]
fn smart_automove_pool_retained_profile_ids_match_active_registry() {
    assert_eq!(
        retained_profile_ids(),
        vec![
            "base",
            "runtime_current",
            "runtime_release_safe_pre_exact",
            "runtime_eff_exact_lite_v1",
            "swift_2024_eval_reference",
            "swift_2024_style_reference",
            "runtime_pre_fast_root_quality_v1_normal_conversion_v3",
            "runtime_normal_from_fast_reference_v1",
            "runtime_pro_turn_engine_v1",
            "runtime_pro_turn_engine_v30",
        ]
    );
}

#[test]
fn smart_automove_pool_curated_pool_profiles_are_unique_and_resolvable() {
    let pool = selected_pool_models();
    assert_eq!(pool.len(), CURATED_POOL_SIZE);

    for model in &pool {
        assert!(
            retained_profile_ids().contains(&model.id),
            "curated pool model '{}' should come from retained registry",
            model.id
        );
    }

    for (index, left) in pool.iter().enumerate() {
        for right in pool.iter().skip(index + 1) {
            assert_ne!(left.id, right.id, "curated pool ids must be unique");
            assert!(
                !std::ptr::fn_addr_eq(left.select_inputs, right.select_inputs),
                "curated pool selectors must be unique: {} and {}",
                left.id,
                right.id
            );
        }
    }
}

#[test]
fn smart_automove_pool_smoke_runs() {
    let probe_model = AutomoveModel {
        id: "smoke_probe_candidate",
        select_inputs: model_first_legal_automove,
    };
    let quick_budgets = [SearchBudget {
        label: "smoke_probe",
        depth: 1,
        max_nodes: 1,
    }];
    let pool = vec![AutomoveModel {
        id: "smoke_probe_pool",
        select_inputs: model_first_legal_automove,
    }];

    let evaluation =
        evaluate_candidate_against_pool_with_max_plies(probe_model, &pool, 1, &quick_budgets, 2);
    assert_eq!(evaluation.opponents.len(), pool.len());
    assert_eq!(evaluation.games_per_matchup, 1);
}

#[test]
#[ignore = "profile speed probe on fixed opening positions"]
fn smart_automove_pool_profile_speed_probe() {
    let positions = env_usize("SMART_SPEED_POSITIONS").unwrap_or(20).max(1);
    let openings = generate_opening_fens(seed_for_pairing("speed", "probe"), positions);
    let profile = candidate_profile().as_str().to_string();
    let selector = CANDIDATE_MODEL.select_inputs;
    let client_budgets = client_budgets();

    println!(
        "speed probe: profile={} positions={} modes={:?}",
        profile,
        positions,
        client_budgets
            .iter()
            .map(|budget| budget.key().to_string())
            .collect::<Vec<_>>()
    );

    for stat in profile_speed_by_mode_ms(selector, openings.as_slice(), &client_budgets) {
        println!(
            "speed probe mode {}: avg_ms_per_position={:.2}",
            stat.budget.key(),
            stat.avg_ms
        );
    }
}

#[test]
#[ignore = "diagnostic: compare pro opening-reply latency on fixed fixtures"]
fn smart_automove_pool_opening_reply_speed_probe() {
    let compare_profile_name = env_profile_name("SMART_OPENING_SPEED_COMPARE_PROFILE")
        .or_else(|| Some(pro_candidate_profile_name()))
        .unwrap_or_else(|| "runtime_current".to_string());
    let baseline_profile_name = env_profile_name("SMART_OPENING_SPEED_BASELINE_PROFILE")
        .or_else(|| Some(pro_baseline_profile_name()))
        .unwrap_or_else(|| "runtime_current".to_string());
    let compare_selector = profile_selector_from_name(compare_profile_name.as_str())
        .unwrap_or_else(|| panic!("compare profile '{}' not found", compare_profile_name));
    let baseline_selector = profile_selector_from_name(baseline_profile_name.as_str())
        .unwrap_or_else(|| panic!("baseline profile '{}' not found", baseline_profile_name));
    let passes = env_usize("SMART_OPENING_SPEED_PASSES").unwrap_or(5).max(1);
    let fixtures = opening_reply_triage_fixtures();

    let _ = opening_reply_speed_probe_avg_ms(
        compare_profile_name.as_str(),
        compare_selector,
        fixtures.as_slice(),
    );
    let _ = opening_reply_speed_probe_avg_ms(
        baseline_profile_name.as_str(),
        baseline_selector,
        fixtures.as_slice(),
    );
    let mut compare_pass_averages = Vec::with_capacity(passes);
    let mut baseline_pass_averages = Vec::with_capacity(passes);
    for pass_index in 0..passes {
        if pass_index.is_multiple_of(2) {
            compare_pass_averages.push(opening_reply_speed_probe_avg_ms(
                compare_profile_name.as_str(),
                compare_selector,
                fixtures.as_slice(),
            ));
            baseline_pass_averages.push(opening_reply_speed_probe_avg_ms(
                baseline_profile_name.as_str(),
                baseline_selector,
                fixtures.as_slice(),
            ));
        } else {
            baseline_pass_averages.push(opening_reply_speed_probe_avg_ms(
                baseline_profile_name.as_str(),
                baseline_selector,
                fixtures.as_slice(),
            ));
            compare_pass_averages.push(opening_reply_speed_probe_avg_ms(
                compare_profile_name.as_str(),
                compare_selector,
                fixtures.as_slice(),
            ));
        }
    }
    let compare_median = median_f64(compare_pass_averages.as_mut_slice());
    let baseline_median = median_f64(baseline_pass_averages.as_mut_slice());
    let ratio = compare_median / baseline_median.max(0.001);

    println!(
        "opening reply speed probe compare={} pass_avg_ms={:?} median_avg_ms={:.2}",
        compare_profile_name, compare_pass_averages, compare_median
    );
    println!(
        "opening reply speed probe baseline={} pass_avg_ms={:?} median_avg_ms={:.2}",
        baseline_profile_name, baseline_pass_averages, baseline_median
    );
    println!(
        "opening reply speed probe delta compare={} baseline={} delta_ms={:.2} ratio={:.3}",
        compare_profile_name,
        baseline_profile_name,
        compare_median - baseline_median,
        ratio
    );

    if let Some(min_ratio) = env_f64("SMART_OPENING_SPEED_MIN_RATIO") {
        assert!(
            ratio >= min_ratio,
            "opening reply speed probe ratio {:.3} below required minimum {:.3}",
            ratio,
            min_ratio
        );
    }
    if let Some(max_ratio) = env_f64("SMART_OPENING_SPEED_MAX_RATIO") {
        assert!(
            ratio <= max_ratio,
            "opening reply speed probe ratio {:.3} above required maximum {:.3}",
            ratio,
            max_ratio
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect per-opening ladder speed for the pro candidate"]
fn smart_automove_pool_pro_ladder_speed_opening_probe() {
    use std::time::Instant;

    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate profile '{}' not found", candidate_profile));
    let baseline_selector = profile_selector_from_name(baseline_profile.as_str())
        .unwrap_or_else(|| panic!("baseline profile '{}' not found", baseline_profile));
    let speed_positions = env_usize("SMART_PRO_GATE_SPEED_POSITIONS")
        .unwrap_or(12)
        .max(4);
    let speed_seed = seed_for_pairing("pro_promotion_ladder", "speed");
    let openings = generate_opening_fens_cached(speed_seed, speed_positions);

    for (index, opening) in openings.iter().enumerate() {
        let game = MonsGame::from_fen(opening, false).expect("valid speed opening fen");

        clear_exact_state_analysis_cache();
        clear_turn_engine_plan_cache();
        clear_turn_engine_selector_diagnostics();
        let candidate_base = pro_budget().runtime_config_for_game(&game);
        let candidate_runtime =
            profile_runtime_config_for_name(candidate_profile.as_str(), &game, candidate_base)
                .unwrap_or(candidate_base);
        let candidate_start = Instant::now();
        let candidate_inputs =
            select_inputs_with_runtime_fallback(candidate_selector, &game, candidate_base);
        let candidate_ms = candidate_start.elapsed().as_secs_f64() * 1000.0;
        let candidate_diag = turn_engine_selector_diagnostics_snapshot();

        clear_exact_state_analysis_cache();
        clear_turn_engine_plan_cache();
        clear_turn_engine_selector_diagnostics();
        let baseline_base = SearchBudget::from_preference(SmartAutomovePreference::Normal)
            .runtime_config_for_game(&game);
        let baseline_runtime =
            profile_runtime_config_for_name(baseline_profile.as_str(), &game, baseline_base)
                .unwrap_or(baseline_base);
        let baseline_start = Instant::now();
        let baseline_inputs =
            select_inputs_with_runtime_fallback(baseline_selector, &game, baseline_base);
        let baseline_ms = baseline_start.elapsed().as_secs_f64() * 1000.0;

        println!(
            "PRO_LADDER_SPEED opening={} candidate_ms={:.2} baseline_ms={:.2} ratio={:.3} candidate_move={} baseline_move={} candidate_turn_engine={} candidate_mode={:?} candidate_last_stage={} candidate_head_calls={} candidate_head_hits={} candidate_fen={}",
            index,
            candidate_ms,
            baseline_ms,
            candidate_ms / baseline_ms.max(0.001),
            Input::fen_from_array(candidate_inputs.as_slice()),
            Input::fen_from_array(baseline_inputs.as_slice()),
            candidate_runtime.enable_turn_engine,
            candidate_runtime.turn_engine_mode,
            candidate_diag.last_return_stage,
            candidate_diag.head_plan_calls,
            candidate_diag.head_plan_hits,
            opening
        );
        println!(
            "PRO_LADDER_SPEED_BASE opening={} baseline_turn_engine={} baseline_mode={:?}",
            index, baseline_runtime.enable_turn_engine, baseline_runtime.turn_engine_mode
        );
    }
}

#[test]
#[ignore = "diagnostic: compare candidate vs baseline pool deltas per mode/opponent"]
fn smart_automove_pool_pool_regression_diagnostic() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();
    let candidate = AutomoveModel {
        id: "candidate",
        select_inputs: CANDIDATE_MODEL.select_inputs,
    };
    let baseline = AutomoveModel {
        id: "baseline",
        select_inputs: profile_selector_from_name(baseline_profile_name.as_str())
            .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile_name)),
    };
    let budgets = client_budgets().to_vec();
    let pool_games = env_usize("SMART_GATE_POOL_GAMES").unwrap_or(3).max(1);
    let (candidate_eval, baseline_eval, candidate_wr, baseline_wr) =
        run_pool_non_regression_check(candidate, baseline, budgets.as_slice(), pool_games);

    println!(
        "pool regression diagnostic: candidate={} baseline={} games={} candidate_wr={:.3} baseline_wr={:.3} delta={:+.3}",
        candidate_profile_name,
        baseline_profile_name,
        pool_games,
        candidate_wr,
        baseline_wr,
        candidate_wr - baseline_wr
    );
    println!(
        "candidate beaten={}/{} | baseline beaten={}/{}",
        candidate_eval.beaten_opponents,
        candidate_eval.opponents.len(),
        baseline_eval.beaten_opponents,
        baseline_eval.opponents.len()
    );

    for budget in budgets {
        let Some(candidate_mode) = candidate_eval
            .mode_results
            .iter()
            .find(|mode| mode.budget.key() == budget.key())
        else {
            continue;
        };
        let Some(baseline_mode) = baseline_eval
            .mode_results
            .iter()
            .find(|mode| mode.budget.key() == budget.key())
        else {
            continue;
        };
        println!(
            "mode {}: candidate_wr={:.3} baseline_wr={:.3} delta={:+.3}",
            budget.key(),
            candidate_mode.aggregate_stats.win_rate_points(),
            baseline_mode.aggregate_stats.win_rate_points(),
            candidate_mode.aggregate_stats.win_rate_points()
                - baseline_mode.aggregate_stats.win_rate_points(),
        );
    }
}

#[test]
#[ignore = "tactical guardrail suite for runtime candidate quality"]
fn smart_automove_tactical_suite() {
    let runtime_selector = profile_selector_from_name("runtime_current")
        .expect("runtime_current selector should exist");
    assert_tactical_guardrails(runtime_selector, "runtime_current");
}

#[test]
#[ignore = "tactical guardrail suite for selected candidate profile"]
fn smart_automove_tactical_candidate_profile() {
    let profile_name = candidate_profile().as_str().to_string();
    assert_tactical_guardrails(CANDIDATE_MODEL.select_inputs, profile_name.as_str());
    assert_interview_policy_regressions(CANDIDATE_MODEL.select_inputs, profile_name.as_str());
}

#[test]
#[ignore = "stage-1 cpu gate against runtime_current; advisory-only for Pro candidates when enabled"]
fn smart_automove_pool_stage1_cpu_non_regression_gate() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    assert_stage1_cpu_non_regression(
        candidate_profile_name.as_str(),
        CANDIDATE_MODEL.select_inputs,
    );
}

#[test]
#[ignore = "exact-lite diagnostics gate for per-move budgets and cache efficiency"]
fn smart_automove_pool_exact_lite_diagnostics_gate() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    assert_exact_lite_diagnostics_gate_if_enabled(
        candidate_profile_name.as_str(),
        CANDIDATE_MODEL.select_inputs,
    );
}

#[test]
#[ignore = "deterministic fixture-first triage for fast/normal candidate surfaces"]
fn smart_automove_pool_signal_triage() {
    let surface = triage_surface_from_env();
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();
    let baseline_selector = profile_selector_from_name(baseline_profile_name.as_str())
        .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile_name));

    assert_tactical_guardrails(
        CANDIDATE_MODEL.select_inputs,
        candidate_profile_name.as_str(),
    );
    assert_interview_policy_regressions(
        CANDIDATE_MODEL.select_inputs,
        candidate_profile_name.as_str(),
    );

    match surface {
        TriageSurface::OpeningReply | TriageSurface::PrimaryPro => {
            panic!(
                "surface '{}' requires pro-triage; use SMART_TRIAGE_SURFACE=opening_reply|primary_pro with ./scripts/run-automove-experiment.sh pro-triage",
                surface.as_str()
            );
        }
        TriageSurface::CacheReuse => {
            let candidate_probe = cache_reuse_triage_probe(
                candidate_profile_name.as_str(),
                CANDIDATE_MODEL.select_inputs,
            );
            let baseline_probe =
                cache_reuse_triage_probe(baseline_profile_name.as_str(), baseline_selector);
            println!(
                "triage surface=cache_reuse candidate={} avg_ms={:.2} hit_rate={:.3} hits={} calls={} intent_calls={} intent_hits={} compile_fallbacks={} injected_candidates={} injected_duplicates={} injected_attempts={} injected_accepts={} reject_build={} reject_emergency={} reject_emerg_loss={} reject_emerg_no_signal={} reject_emerg_handoff={} reject_emerg_drainer={} reject_top_wins={} reject_unsafe={} reject_no_tactical={} reject_gap={} baseline={} avg_ms={:.2} hit_rate={:.3} hits={} calls={} intent_calls={} intent_hits={} compile_fallbacks={} injected_candidates={} injected_duplicates={} injected_attempts={} injected_accepts={} reject_build={} reject_emergency={} reject_emerg_loss={} reject_emerg_no_signal={} reject_emerg_handoff={} reject_emerg_drainer={} reject_top_wins={} reject_unsafe={} reject_no_tactical={} reject_gap={}",
                candidate_profile_name,
                candidate_probe.avg_ms,
                candidate_probe.hit_rate,
                candidate_probe.hits,
                candidate_probe.calls,
                candidate_probe.intent_generation_calls,
                candidate_probe.intent_generation_hits,
                candidate_probe.compile_fallbacks,
                candidate_probe.injected_root_candidates_seen,
                candidate_probe.injected_root_duplicates,
                candidate_probe.injected_root_attempts,
                candidate_probe.injected_root_accepts,
                candidate_probe.injected_root_reject_build,
                candidate_probe.injected_root_reject_emergency_guard,
                candidate_probe.injected_root_reject_emergency_introduced_loss,
                candidate_probe.injected_root_reject_emergency_no_crisis_signal,
                candidate_probe.injected_root_reject_emergency_mana_handoff,
                candidate_probe.injected_root_reject_emergency_drainer_unsafe,
                candidate_probe.injected_root_reject_top_wins,
                candidate_probe.injected_root_reject_candidate_unsafe,
                candidate_probe.injected_root_reject_no_tactical_signal,
                candidate_probe.injected_root_reject_heuristic_gap,
                baseline_profile_name,
                baseline_probe.avg_ms,
                baseline_probe.hit_rate,
                baseline_probe.hits,
                baseline_probe.calls
                ,
                baseline_probe.intent_generation_calls,
                baseline_probe.intent_generation_hits,
                baseline_probe.compile_fallbacks,
                baseline_probe.injected_root_candidates_seen,
                baseline_probe.injected_root_duplicates,
                baseline_probe.injected_root_attempts,
                baseline_probe.injected_root_accepts,
                baseline_probe.injected_root_reject_build,
                baseline_probe.injected_root_reject_emergency_guard,
                baseline_probe.injected_root_reject_emergency_introduced_loss,
                baseline_probe.injected_root_reject_emergency_no_crisis_signal,
                baseline_probe.injected_root_reject_emergency_mana_handoff,
                baseline_probe.injected_root_reject_emergency_drainer_unsafe,
                baseline_probe.injected_root_reject_top_wins,
                baseline_probe.injected_root_reject_candidate_unsafe,
                baseline_probe.injected_root_reject_no_tactical_signal,
                baseline_probe.injected_root_reject_heuristic_gap
            );
            assert!(
                cache_reuse_triage_passes(candidate_probe, baseline_probe),
                "cache_reuse triage found no deterministic evidence change: candidate avg_ms={:.2} hit_rate={:.3} intent_calls={} intent_hits={} compile_fallbacks={} injected_candidates={} injected_duplicates={} injected_attempts={} injected_accepts={} reject_build={} reject_emergency={} reject_emerg_loss={} reject_emerg_no_signal={} reject_emerg_handoff={} reject_emerg_drainer={} reject_top_wins={} reject_unsafe={} reject_no_tactical={} reject_gap={}, baseline avg_ms={:.2} hit_rate={:.3} intent_calls={} intent_hits={} compile_fallbacks={} injected_candidates={} injected_duplicates={} injected_attempts={} injected_accepts={} reject_build={} reject_emergency={} reject_emerg_loss={} reject_emerg_no_signal={} reject_emerg_handoff={} reject_emerg_drainer={} reject_top_wins={} reject_unsafe={} reject_no_tactical={} reject_gap={}",
                candidate_probe.avg_ms,
                candidate_probe.hit_rate,
                candidate_probe.intent_generation_calls,
                candidate_probe.intent_generation_hits,
                candidate_probe.compile_fallbacks,
                candidate_probe.injected_root_candidates_seen,
                candidate_probe.injected_root_duplicates,
                candidate_probe.injected_root_attempts,
                candidate_probe.injected_root_accepts,
                candidate_probe.injected_root_reject_build,
                candidate_probe.injected_root_reject_emergency_guard,
                candidate_probe.injected_root_reject_emergency_introduced_loss,
                candidate_probe.injected_root_reject_emergency_no_crisis_signal,
                candidate_probe.injected_root_reject_emergency_mana_handoff,
                candidate_probe.injected_root_reject_emergency_drainer_unsafe,
                candidate_probe.injected_root_reject_top_wins,
                candidate_probe.injected_root_reject_candidate_unsafe,
                candidate_probe.injected_root_reject_no_tactical_signal,
                candidate_probe.injected_root_reject_heuristic_gap,
                baseline_probe.avg_ms,
                baseline_probe.hit_rate,
                baseline_probe.intent_generation_calls,
                baseline_probe.intent_generation_hits,
                baseline_probe.compile_fallbacks,
                baseline_probe.injected_root_candidates_seen,
                baseline_probe.injected_root_duplicates,
                baseline_probe.injected_root_attempts,
                baseline_probe.injected_root_accepts,
                baseline_probe.injected_root_reject_build,
                baseline_probe.injected_root_reject_emergency_guard,
                baseline_probe.injected_root_reject_emergency_introduced_loss,
                baseline_probe.injected_root_reject_emergency_no_crisis_signal,
                baseline_probe.injected_root_reject_emergency_mana_handoff,
                baseline_probe.injected_root_reject_emergency_drainer_unsafe,
                baseline_probe.injected_root_reject_top_wins,
                baseline_probe.injected_root_reject_candidate_unsafe,
                baseline_probe.injected_root_reject_no_tactical_signal,
                baseline_probe.injected_root_reject_heuristic_gap
            );
        }
        _ => {
            let fixtures = generic_triage_surface_fixtures(surface);
            assert!(
                !fixtures.is_empty(),
                "surface '{}' has no generic triage fixtures; add the fixture first",
                surface.as_str()
            );
            let target_changed = compare_triage_fixture_pack(
                surface,
                candidate_profile_name.as_str(),
                CANDIDATE_MODEL.select_inputs,
                baseline_profile_name.as_str(),
                baseline_selector,
                fixtures.as_slice(),
            );
            assert!(
                generic_signal_triage_passes(target_changed),
                "triage surface '{}' showed no deterministic evidence change vs baseline",
                surface.as_str()
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: validate normal_fast_gap fixtures against current fast-vs-normal gap"]
fn smart_automove_normal_fast_gap_surface_probe() {
    let fixtures = generic_triage_surface_fixtures(TriageSurface::NormalFastGap);
    let normal_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let fast_profile =
        env_profile_name("SMART_PROBE_FAST_PROFILE").unwrap_or_else(|| "runtime_current".into());

    assert!(
        !fixtures.is_empty(),
        "normal_fast_gap surface should have deterministic fixtures"
    );

    for fixture in fixtures {
        let expected = fixture.expected_selected_input_fen.unwrap_or_else(|| {
            panic!(
                "normal_fast_gap fixture '{}' must declare expected_selected_input_fen",
                fixture.id
            )
        });
        let normal = loss_probe_decision(normal_profile.as_str(), fixture.mode, &fixture.game);
        let fast = loss_probe_decision(
            fast_profile.as_str(),
            SmartAutomovePreference::Fast,
            &fixture.game,
        );
        println!(
            "normal_fast_gap fixture={} expected={} normal_move={} fast_move={} fen={}",
            fixture.id,
            expected,
            normal.move_fen,
            fast.move_fen,
            fixture.game.fen()
        );
        assert_eq!(
            fast.move_fen, expected,
            "normal_fast_gap fixture '{}' no longer matches current Fast",
            fixture.id
        );
        assert_ne!(
            normal.move_fen, expected,
            "normal_fast_gap fixture '{}' no longer separates current Normal from current Fast",
            fixture.id
        );
    }
}

#[test]
#[ignore = "diagnostic: validate normal_release_seed_gap fixtures against fast-derived reference vs release baseline"]
fn smart_automove_normal_release_seed_gap_surface_probe() {
    let fixtures = generic_triage_surface_fixtures(TriageSurface::NormalReleaseSeedGap);
    const REFERENCE_PROFILE: &str = "runtime_normal_from_fast_reference_v1";
    const BASELINE_PROFILE: &str = "runtime_release_safe_pre_exact";

    assert!(
        fixtures.len() >= 6,
        "normal_release_seed_gap surface should have at least 6 deterministic fixtures"
    );

    for fixture in fixtures {
        let expected = fixture.expected_selected_input_fen.unwrap_or_else(|| {
            panic!(
                "normal_release_seed_gap fixture '{}' must declare expected_selected_input_fen",
                fixture.id
            )
        });
        let reference = loss_probe_decision(REFERENCE_PROFILE, fixture.mode, &fixture.game);
        let baseline = loss_probe_decision(BASELINE_PROFILE, fixture.mode, &fixture.game);
        println!(
            "normal_release_seed_gap fixture={} expected={} reference_move={} baseline_move={} fen={}",
            fixture.id,
            expected,
            reference.move_fen,
            baseline.move_fen,
            fixture.game.fen()
        );
        assert_eq!(
            reference.move_fen, expected,
            "normal_release_seed_gap fixture '{}' no longer matches fast-derived reference",
            fixture.id
        );
        assert_ne!(
            baseline.move_fen, expected,
            "normal_release_seed_gap fixture '{}' no longer separates release baseline",
            fixture.id
        );
    }
}

#[test]
#[ignore = "retained-profile calibration for triage surfaces"]
fn smart_automove_pool_surface_calibration() {
    let surface = triage_surface_from_env();
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();

    match surface {
        TriageSurface::ReplyRisk => {
            let candidate_probe = reply_risk_calibration_probe(candidate_profile_name.as_str());
            let baseline_probe = reply_risk_calibration_probe(baseline_profile_name.as_str());
            println!(
                "triage-calibrate surface=reply_risk candidate={} probe={} baseline={} probe={} delta={}",
                candidate_profile_name,
                candidate_probe,
                baseline_profile_name,
                baseline_probe,
                candidate_probe - baseline_probe
            );
            assert!(
                candidate_probe.abs_diff(baseline_probe) >= 20,
                "reply_risk calibration found no meaningful profile delta: candidate={} baseline={}",
                candidate_probe,
                baseline_probe
            );
        }
        TriageSurface::OpponentMana => {
            let candidate_pick = opponent_mana_calibration_probe(candidate_profile_name.as_str());
            let baseline_pick = opponent_mana_calibration_probe(baseline_profile_name.as_str());
            println!(
                "triage-calibrate surface=opponent_mana candidate={} pick={} baseline={} pick={}",
                candidate_profile_name, candidate_pick, baseline_profile_name, baseline_pick
            );
            assert_ne!(
                candidate_pick, baseline_pick,
                "opponent_mana calibration found no profile delta: candidate_pick={} baseline_pick={}",
                candidate_pick, baseline_pick
            );
        }
        TriageSurface::Supermana => {
            let candidate_probe = supermana_calibration_probe(candidate_profile_name.as_str());
            let baseline_probe = supermana_calibration_probe(baseline_profile_name.as_str());
            println!(
                "triage-calibrate surface=supermana candidate={} exact_lite={} baseline={} exact_lite={}",
                candidate_profile_name,
                candidate_probe,
                baseline_profile_name,
                baseline_probe
            );
            assert_ne!(
                candidate_probe, baseline_probe,
                "supermana calibration found no profile delta: candidate_exact_lite={} baseline_exact_lite={}",
                candidate_probe, baseline_probe
            );
        }
        _ => {
            panic!(
                "triage-calibrate only supports SMART_TRIAGE_SURFACE=reply_risk|opponent_mana|supermana; got '{}'",
                surface.as_str()
            );
        }
    }
}

#[test]
#[ignore = "deterministic fixture-first triage for pro opening_reply and primary_pro surfaces"]
fn smart_automove_pool_pro_signal_triage() {
    let surface = triage_surface_from_env();
    let candidate_profile_name = pro_candidate_profile_name();
    let baseline_profile_name = pro_baseline_profile_name();
    let candidate_selector = profile_selector_from_name(candidate_profile_name.as_str())
        .unwrap_or_else(|| panic!("candidate '{}' not found", candidate_profile_name));
    let baseline_selector = profile_selector_from_name(baseline_profile_name.as_str())
        .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile_name));

    assert_tactical_guardrails(candidate_selector, candidate_profile_name.as_str());
    assert_interview_policy_regressions(candidate_selector, candidate_profile_name.as_str());

    let opening_changed = compare_triage_fixture_pack(
        TriageSurface::OpeningReply,
        candidate_profile_name.as_str(),
        candidate_selector,
        baseline_profile_name.as_str(),
        baseline_selector,
        opening_reply_triage_fixtures().as_slice(),
    );
    let primary_changed = compare_triage_fixture_pack(
        TriageSurface::PrimaryPro,
        candidate_profile_name.as_str(),
        candidate_selector,
        baseline_profile_name.as_str(),
        baseline_selector,
        primary_pro_triage_fixtures().as_slice(),
    );

    let (target_changed, off_target_changed) = match surface {
        TriageSurface::OpeningReply => (opening_changed, primary_changed),
        TriageSurface::PrimaryPro => (primary_changed, opening_changed),
        _ => {
            panic!(
                "pro-triage only supports SMART_TRIAGE_SURFACE=opening_reply or primary_pro; got '{}'",
                surface.as_str()
            );
        }
    };

    println!(
        "pro triage surface={} target_changed={} off_target_changed={}",
        surface.as_str(),
        target_changed,
        off_target_changed
    );
    assert!(
        pro_signal_triage_passes(target_changed, off_target_changed),
        "pro triage failed for surface='{}': target_changed={} off_target_changed={} (expected at least one target change and at most one off-target change)",
        surface.as_str(),
        target_changed,
        off_target_changed
    );
}

#[test]
#[ignore = "diagnostic: probe mid-game positions for Pro fixture expansion"]
fn smart_automove_pro_fixture_position_probe() {
    use rand::prelude::*;
    let pro_budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);
    let positions = env_usize("SMART_PROBE_POSITIONS").unwrap_or(20);
    let plies_per_position = env_usize("SMART_PROBE_PLIES").unwrap_or(12);
    let seed = env_usize("SMART_PROBE_SEED").unwrap_or(42) as u64;
    let openings = generate_opening_fens_cached(seed, positions);

    let profile_names: Vec<&str> = vec![
        "runtime_current",
        "runtime_release_safe_pre_exact",
        "runtime_pro_turn_engine_v30",
    ];

    println!(
        "probing {} positions x {} plies, seed={}, profiles={:?}",
        positions, plies_per_position, seed, profile_names
    );

    for (pos_idx, fen) in openings.iter().enumerate() {
        let base_game = MonsGame::from_fen(fen.as_str(), false);
        let Some(base_game) = base_game else { continue };
        let mut game = base_game.clone_for_simulation();
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(pos_idx as u64));
        for _ in 0..plies_per_position {
            if !apply_seeded_random_move(&mut game, &mut rng) {
                break;
            }
        }
        if game.winner_color().is_some() {
            continue;
        }

        let pro_config = pro_budget.runtime_config_for_game(&game);
        let ranked = MonsGameModel::ranked_root_moves(&game, game.active_color, pro_config);
        if ranked.len() < 2 {
            continue;
        }

        let h_gap = ranked[0].heuristic.saturating_sub(ranked[1].heuristic);

        let mut profile_moves: Vec<(&str, String)> = Vec::new();
        for &pname in &profile_names {
            let sel = profile_selector_from_name(pname)
                .unwrap_or_else(|| panic!("profile '{}' not found", pname));
            let inputs = select_inputs_with_runtime_fallback(sel, &game, pro_config);
            let fen_str = Input::fen_from_array(&inputs);
            profile_moves.push((pname, fen_str));
        }

        let base_move = &profile_moves[0].1;
        let any_different = profile_moves.iter().skip(1).any(|(_, m)| m != base_move);
        let base_rank = ranked
            .iter()
            .position(|r| Input::fen_from_array(&r.inputs) == *base_move)
            .unwrap_or(usize::MAX);

        println!(
            "pos={} h_gap={} base_rank={} root_count={} any_diff={} moves={:?}{}",
            pos_idx,
            h_gap,
            base_rank,
            ranked.len(),
            any_different,
            profile_moves
                .iter()
                .map(|(name, fen)| format!("{}={}", name, fen))
                .collect::<Vec<_>>(),
            if any_different {
                format!(" game_fen={}", game.fen())
            } else {
                String::new()
            }
        );
    }
}

#[test]
#[ignore = "diagnostic: comprehensive mode-vs-mode W/L/D comparison"]
fn smart_automove_pool_mode_comparison_report() {
    let focus_profile =
        env_profile_name("SMART_MODE_COMPARE_PROFILE").unwrap_or_else(|| "runtime_current".into());
    let baseline_profile = env_profile_name("SMART_MODE_COMPARE_BASELINE_PROFILE")
        .unwrap_or_else(|| focus_profile.clone());
    let repeats = env_usize("SMART_MODE_COMPARE_REPEATS").unwrap_or(4).max(1);
    let games_per_repeat = env_usize("SMART_MODE_COMPARE_GAMES").unwrap_or(8).max(1);
    let max_plies = env_usize("SMART_MODE_COMPARE_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let use_white_opening_book =
        env_bool("SMART_MODE_COMPARE_USE_WHITE_OPENING_BOOK").unwrap_or(false);
    let seed_tags = mode_compare_seed_tags();
    let modes = mode_compare_modes();
    let focus_mode = compare_focus_mode_from_env(
        "SMART_MODE_COMPARE_FOCUS_MODE",
        SmartAutomovePreference::Pro,
    );
    let compare_tag = env::var("SMART_MODE_COMPARE_TAG")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| focus_mode.as_api_value().to_string());

    eprintln!(
        "{} mode comparison config: focus_mode={} profile={} baseline_profile={} seeds={} repeats={} games_per_repeat={} max_plies={} use_white_opening_book={}",
        compare_tag,
        focus_mode.as_api_value(),
        focus_profile,
        baseline_profile,
        seed_tags.len(),
        repeats,
        games_per_repeat,
        max_plies,
        use_white_opening_book
    );

    for mode in modes {
        let mut aggregate = MatchupStats::default();
        for seed_tag in &seed_tags {
            let tagged_seed = format!(
                "{}_compare:{}:{}",
                compare_tag,
                mode.as_api_value(),
                seed_tag
            );
            aggregate.merge(run_cross_budget_duel(CrossBudgetDuelConfig {
                profile_a: focus_profile.as_str(),
                mode_a: focus_mode,
                profile_b: baseline_profile.as_str(),
                mode_b: mode,
                seed_tag: tagged_seed.as_str(),
                repeats,
                games_per_repeat,
                max_plies,
                use_white_opening_book,
            }));
            eprintln!(
                "{}-vs-{} seed={} cumulative_games={} W={} L={} D={} win_rate={:.4}",
                compare_tag,
                mode.as_api_value(),
                seed_tag,
                aggregate.total_games(),
                aggregate.wins,
                aggregate.losses,
                aggregate.draws,
                aggregate.win_rate_points()
            );
        }

        let (delta, confidence) = stats_delta_confidence(aggregate);
        eprintln!(
            "{}-vs-{}: games={} W={} L={} D={} win_rate={:.4} delta={:+.4} confidence={:.3}",
            compare_tag,
            mode.as_api_value(),
            aggregate.total_games(),
            aggregate.wins,
            aggregate.losses,
            aggregate.draws,
            aggregate.win_rate_points(),
            delta,
            confidence
        );
    }
}

#[test]
#[ignore = "quick reject-oriented screen that requires meaningful tier-0 signal"]
fn smart_automove_pool_fast_screen() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();
    let allow_self_baseline = env_bool("SMART_GATE_ALLOW_SELF_BASELINE").unwrap_or(false);
    if !allow_self_baseline {
        assert!(
            candidate_profile_name != baseline_profile_name,
            "candidate and baseline must differ (set SMART_GATE_ALLOW_SELF_BASELINE=true to override)"
        );
    }
    assert_runtime_preflight_if_required(
        candidate_profile_name.as_str(),
        CANDIDATE_MODEL.select_inputs,
    );

    let candidate = AutomoveModel {
        id: "candidate",
        select_inputs: CANDIDATE_MODEL.select_inputs,
    };
    let baseline = AutomoveModel {
        id: "baseline",
        select_inputs: profile_selector_from_name(baseline_profile_name.as_str())
            .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile_name)),
    };
    let budgets = client_budgets().to_vec();
    let config = ProgressiveDuelConfig::from_env_with_defaults("screen");
    let targeted_first_tier = config.promotion_target_mode.is_some();
    let target_only_budgets = env_bool("SMART_TARGET_ONLY_BUDGETS").unwrap_or(false);
    let duel_budgets: Vec<SearchBudget> = if target_only_budgets && targeted_first_tier {
        if let Some(target_mode) = config.promotion_target_mode {
            budgets
                .iter()
                .filter(|b| b.key() == target_mode.key())
                .copied()
                .collect()
        } else {
            budgets.clone()
        }
    } else {
        budgets.clone()
    };
    let max_games_per_seed = if targeted_first_tier {
        config.max_games_per_seed.clamp(4, 16)
    } else {
        config.max_games_per_seed.clamp(4, 8)
    };
    let artifact_path = env::var("SMART_LADDER_ARTIFACT_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_progressive_artifact_path(&candidate_profile_name));
    let result = run_progressive_duel(
        candidate,
        baseline,
        duel_budgets.as_slice(),
        &ProgressiveDuelConfig {
            initial_games: config.initial_games.max(2),
            max_games_per_seed,
            seed_tags: vec!["neutral_v1"],
            max_plies: 84,
            early_exit_delta_floor: -0.10,
            first_tier_signal_games_per_seed: Some(config.initial_games.max(2)),
            first_tier_signal_aggregate_delta_min: if targeted_first_tier { 0.0 } else { 0.10 },
            first_tier_signal_mode_delta_min: 0.125,
            first_tier_target_confidence_min: if targeted_first_tier { 0.60 } else { 0.0 },
            first_tier_signal_mode_floor: 0.0,
            ..config
        },
        Some(artifact_path.as_str()),
    );

    println!(
        "fast screen: {} vs {} | games={} delta={:.4} confidence={:.3} stop={:?}",
        candidate_profile_name,
        baseline_profile_name,
        result.total_games,
        result.final_delta,
        result.final_confidence,
        result.stop_reason
    );

    match result.stop_reason {
        ProgressiveStopReason::EarlyReject
        | ProgressiveStopReason::MathematicalReject
        | ProgressiveStopReason::FadingSignal => {
            panic!("fast screen rejected candidate");
        }
        ProgressiveStopReason::EarlyPromote => {}
        ProgressiveStopReason::MaxGamesReached => {
            assert!(
                result.final_delta >= 0.0,
                "fast screen: negative delta at max games"
            );
        }
    }
}

#[test]
#[ignore = "progressive evaluation: geometric doubling, 2→4→8→16→32 games"]
fn smart_automove_pool_progressive_duel() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();
    let allow_self_baseline = env_bool("SMART_GATE_ALLOW_SELF_BASELINE").unwrap_or(false);
    if !allow_self_baseline {
        assert!(
            candidate_profile_name != baseline_profile_name,
            "candidate and baseline must differ"
        );
    }
    assert_runtime_preflight_if_required(
        candidate_profile_name.as_str(),
        CANDIDATE_MODEL.select_inputs,
    );

    let candidate = AutomoveModel {
        id: "candidate",
        select_inputs: CANDIDATE_MODEL.select_inputs,
    };
    let baseline = AutomoveModel {
        id: "baseline",
        select_inputs: profile_selector_from_name(baseline_profile_name.as_str())
            .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile_name)),
    };
    let budgets = client_budgets().to_vec();
    let config = if env_bool("SMART_PROGRESSIVE_PRIMARY").unwrap_or(false) {
        ProgressiveDuelConfig::from_env_with_defaults("primary")
    } else {
        ProgressiveDuelConfig::from_env_with_defaults("duel")
    };
    let artifact_path = env::var("SMART_LADDER_ARTIFACT_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_progressive_artifact_path(&candidate_profile_name));
    let result = run_progressive_duel(
        candidate,
        baseline,
        budgets.as_slice(),
        &config,
        Some(artifact_path.as_str()),
    );

    println!(
        "progressive duel: {} vs {} | total_games={} delta={:.4} confidence={:.3} stop={:?}",
        candidate_profile_name,
        baseline_profile_name,
        result.total_games,
        result.final_delta,
        result.final_confidence,
        result.stop_reason
    );

    match result.stop_reason {
        ProgressiveStopReason::EarlyReject
        | ProgressiveStopReason::MathematicalReject
        | ProgressiveStopReason::FadingSignal => {
            panic!(
                "progressive duel rejected: {:?} at {} games, δ={:.4}",
                result.stop_reason, result.total_games, result.final_delta
            );
        }
        ProgressiveStopReason::EarlyPromote | ProgressiveStopReason::MaxGamesReached => {
            assert!(
                result.final_delta >= 0.0,
                "progressive duel failed aggregate non-regression: delta {:.4} < 0.0",
                result.final_delta
            );
        }
    }
}

#[test]
#[ignore = "staged promotion ladder with early-stop pruning and artifact output"]
fn smart_automove_pool_promotion_ladder() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();
    let allow_self_baseline = env_bool("SMART_GATE_ALLOW_SELF_BASELINE").unwrap_or(false);
    if !allow_self_baseline {
        assert!(
            candidate_profile_name != baseline_profile_name,
            "candidate profile and baseline profile must differ for ladder gate"
        );
    }
    assert_runtime_preflight_if_required(
        candidate_profile_name.as_str(),
        CANDIDATE_MODEL.select_inputs,
    );

    let candidate = AutomoveModel {
        id: "candidate",
        select_inputs: CANDIDATE_MODEL.select_inputs,
    };
    let baseline = AutomoveModel {
        id: "baseline",
        select_inputs: profile_selector_from_name(baseline_profile_name.as_str()).unwrap_or_else(
            || panic!("baseline selector '{}' should exist", baseline_profile_name),
        ),
    };
    let budgets = client_budgets().to_vec();
    let mut artifacts = Vec::<String>::new();

    assert_tactical_guardrails(candidate.select_inputs, candidate_profile_name.as_str());
    assert_interview_policy_regressions(candidate.select_inputs, candidate_profile_name.as_str());
    assert_tactical_guardrails(baseline.select_inputs, baseline_profile_name.as_str());
    assert_interview_policy_regressions(baseline.select_inputs, baseline_profile_name.as_str());
    artifacts.push(format!(
        r#"{{"stage":"A_tactical","profile":"{}","status":"pass"}}"#,
        candidate_profile_name
    ));

    let speed_positions = env_usize("SMART_GATE_SPEED_POSITIONS").unwrap_or(12).max(4);
    let speed_seed = seed_for_pairing("promotion_ladder", "speed");
    let speed_openings = generate_opening_fens_cached(speed_seed, speed_positions);
    let baseline_speed = profile_speed_by_mode_ms(
        baseline.select_inputs,
        speed_openings.as_slice(),
        budgets.as_slice(),
    );
    let candidate_speed = profile_speed_by_mode_ms(
        candidate.select_inputs,
        speed_openings.as_slice(),
        budgets.as_slice(),
    );
    let baseline_map = baseline_speed
        .iter()
        .map(|stat| (stat.budget.key(), stat.avg_ms))
        .collect::<std::collections::HashMap<_, _>>();
    let mut speed_ratios = std::collections::HashMap::new();
    for stat in &candidate_speed {
        let baseline_ms = baseline_map.get(stat.budget.key()).copied().unwrap_or(1.0);
        speed_ratios.insert(
            stat.budget.key(),
            if baseline_ms > 0.0 {
                stat.avg_ms / baseline_ms
            } else {
                1.0
            },
        );
    }
    let fast_ratio = speed_ratios.get("fast").copied().unwrap_or(1.0);
    let normal_ratio = speed_ratios.get("normal").copied().unwrap_or(1.0);
    artifacts.push(format!(
        r#"{{"stage":"A_speed","fast_ratio":{:.5},"normal_ratio":{:.5}}}"#,
        fast_ratio, normal_ratio
    ));
    assert!(
        fast_ratio <= 1.30,
        "fast cpu gate failed: ratio={:.3}",
        fast_ratio
    );
    assert!(
        normal_ratio <= 1.30,
        "normal cpu gate failed: ratio={:.3}",
        normal_ratio
    );

    let budget_duel_games = env_usize("SMART_GATE_BUDGET_DUEL_GAMES")
        .unwrap_or(3)
        .max(1);
    let budget_duel_repeats = env_usize("SMART_GATE_BUDGET_DUEL_REPEATS")
        .unwrap_or(4)
        .max(1);
    let budget_duel_max_plies = env_usize("SMART_GATE_BUDGET_DUEL_MAX_PLIES")
        .or_else(|| env_usize("SMART_POOL_MAX_PLIES"))
        .unwrap_or(56)
        .max(32);
    let budget_duel_seed_tag = env_profile_name("SMART_GATE_BUDGET_DUEL_SEED_TAG")
        .unwrap_or_else(|| "fast_normal_v1".to_string());
    let baseline_budget_conversion = run_budget_conversion_diagnostic(
        baseline_profile_name.as_str(),
        baseline.select_inputs,
        budget_duel_games,
        budget_duel_repeats,
        budget_duel_max_plies,
        budget_duel_seed_tag.as_str(),
    );
    let candidate_budget_conversion = run_budget_conversion_diagnostic(
        candidate_profile_name.as_str(),
        candidate.select_inputs,
        budget_duel_games,
        budget_duel_repeats,
        budget_duel_max_plies,
        budget_duel_seed_tag.as_str(),
    );
    let conversion_delta =
        candidate_budget_conversion.normal_edge - baseline_budget_conversion.normal_edge;
    artifacts.push(format!(
        r#"{{"stage":"A_budget_conversion","baseline_fast_wr":{:.5},"baseline_normal_edge":{:.5},"candidate_fast_wr":{:.5},"candidate_normal_edge":{:.5},"delta":{:.5}}}"#,
        baseline_budget_conversion.fast_win_rate,
        baseline_budget_conversion.normal_edge,
        candidate_budget_conversion.fast_win_rate,
        candidate_budget_conversion.normal_edge,
        conversion_delta
    ));
    if candidate_budget_conversion.normal_edge + SMART_BUDGET_CONVERSION_REGRESSION_TOLERANCE
        < baseline_budget_conversion.normal_edge
    {
        println!(
            "promotion gate budget conversion NOTE: candidate normal_edge {:.3} < baseline {:.3}",
            candidate_budget_conversion.normal_edge, baseline_budget_conversion.normal_edge
        );
    }

    let progressive_config = ProgressiveDuelConfig::from_env_with_defaults("ladder");
    let progressive_artifact = default_progressive_artifact_path(candidate_profile_name.as_str());
    let progressive_result = run_progressive_duel(
        candidate,
        baseline,
        budgets.as_slice(),
        &progressive_config,
        Some(progressive_artifact.as_str()),
    );
    match progressive_result.stop_reason {
        ProgressiveStopReason::EarlyReject => {
            artifacts.push(format!(
                r#"{{"stage":"B_progressive","status":"early_reject","total_games":{},"delta":{:.5},"confidence":{:.5}}}"#,
                progressive_result.total_games,
                progressive_result.final_delta,
                progressive_result.final_confidence
            ));
            persist_ladder_artifacts(artifacts.as_slice());
            panic!(
                "progressive duel early reject: delta={:.3} after {} games",
                progressive_result.final_delta, progressive_result.total_games
            );
        }
        ProgressiveStopReason::MathematicalReject | ProgressiveStopReason::FadingSignal => {
            artifacts.push(format!(
                r#"{{"stage":"B_progressive","status":"{:?}","total_games":{},"delta":{:.5},"confidence":{:.5}}}"#,
                progressive_result.stop_reason,
                progressive_result.total_games,
                progressive_result.final_delta,
                progressive_result.final_confidence
            ));
            persist_ladder_artifacts(artifacts.as_slice());
            panic!(
                "progressive duel {:?}: delta={:.3} after {} games",
                progressive_result.stop_reason,
                progressive_result.final_delta,
                progressive_result.total_games
            );
        }
        ProgressiveStopReason::EarlyPromote | ProgressiveStopReason::MaxGamesReached => {
            let mut any_mode_improved = false;
            for budget in &budgets {
                let stats = progressive_result
                    .final_mode_stats
                    .get(budget.key())
                    .copied()
                    .unwrap_or_default();
                let mode_delta = stats.win_rate_points() - 0.5;
                let non_regression_floor = progressive_config
                    .mode_non_regression_delta
                    .get(budget.key())
                    .copied()
                    .unwrap_or(-0.03);
                assert!(
                    mode_delta >= non_regression_floor,
                    "progressive mode {} non-regression failed: delta {:.3} < {:.3}",
                    budget.key(),
                    mode_delta,
                    non_regression_floor
                );
                let improvement_delta = progressive_config
                    .mode_improvement_delta
                    .get(budget.key())
                    .copied()
                    .unwrap_or(0.02);
                let improvement_confidence = progressive_config
                    .mode_improvement_confidence
                    .get(budget.key())
                    .copied()
                    .unwrap_or(0.60);
                let mode_confidence = stats.confidence_better_than_even();
                if mode_delta >= improvement_delta && mode_confidence >= improvement_confidence {
                    any_mode_improved = true;
                }
            }
            assert!(
                any_mode_improved,
                "progressive duel: no mode showed sufficient improvement after {} games",
                progressive_result.total_games
            );
            assert!(
                progressive_result.final_delta >= 0.0,
                "progressive aggregate non-regression failed: delta {:.3} < 0.0",
                progressive_result.final_delta
            );
            artifacts.push(format!(
                r#"{{"stage":"B_progressive","status":"ok","total_games":{},"tiers":{},"delta":{:.5},"confidence":{:.5}}}"#,
                progressive_result.total_games,
                progressive_result.tiers.len(),
                progressive_result.final_delta,
                progressive_result.final_confidence
            ));
        }
    }

    let confirm_games = env_usize("SMART_GATE_CONFIRM_GAMES").unwrap_or(4).max(2);
    let confirm_repeats = env_usize("SMART_GATE_CONFIRM_REPEATS").unwrap_or(6).max(2);
    let confirm_max_plies = env_usize("SMART_GATE_CONFIRM_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let confirm_results = run_mirrored_duel_for_seed_tag(MirroredDuelSeedConfig {
        candidate,
        baseline,
        budgets: budgets.as_slice(),
        seed_tag: "prod_open_v1",
        repeats: confirm_repeats,
        games_per_mode: confirm_games,
        max_plies: confirm_max_plies,
        use_white_opening_book: true,
    });
    let mut confirm_aggregate = MatchupStats::default();
    for (_, stats) in &confirm_results {
        confirm_aggregate.merge(*stats);
    }
    let confirm_delta = confirm_aggregate.win_rate_points() - 0.5;
    let confirm_confidence = confirm_aggregate.confidence_better_than_even();
    artifacts.push(format!(
        r#"{{"stage":"D_confirm","wins":{},"losses":{},"draws":{},"delta":{:.5},"confidence":{:.5}}}"#,
        confirm_aggregate.wins,
        confirm_aggregate.losses,
        confirm_aggregate.draws,
        confirm_delta,
        confirm_confidence
    ));

    let pool_games = env_usize("SMART_GATE_POOL_GAMES").unwrap_or(3).max(1);
    let pool_budgets: Vec<SearchBudget> = std::env::var("SMART_PROMOTION_TARGET_MODE")
        .ok()
        .map(|v| v.trim().to_lowercase())
        .and_then(|v| match v.as_str() {
            "fast" => Some(vec![SearchBudget::from_preference(
                SmartAutomovePreference::Fast,
            )]),
            "normal" => Some(vec![SearchBudget::from_preference(
                SmartAutomovePreference::Normal,
            )]),
            _ => None,
        })
        .unwrap_or_else(|| budgets.to_vec());
    let (candidate_pool_eval, baseline_pool_eval, candidate_pool_wr, baseline_pool_wr) =
        run_pool_non_regression_check(candidate, baseline, pool_budgets.as_slice(), pool_games);
    artifacts.push(format!(
        r#"{{"stage":"D_pool","candidate_beaten":{},"candidate_total":{},"baseline_beaten":{},"baseline_total":{},"candidate_wr":{:.5},"baseline_wr":{:.5}}}"#,
        candidate_pool_eval.beaten_opponents,
        candidate_pool_eval.opponents.len(),
        baseline_pool_eval.beaten_opponents,
        baseline_pool_eval.opponents.len(),
        candidate_pool_wr,
        baseline_pool_wr
    ));
    assert!(
        candidate_pool_eval.beaten_opponents >= baseline_pool_eval.beaten_opponents,
        "pool non-regression failed beaten opponents: candidate {} < baseline {}",
        candidate_pool_eval.beaten_opponents,
        baseline_pool_eval.beaten_opponents
    );
    assert!(
        candidate_pool_wr + 0.01 >= baseline_pool_wr,
        "pool non-regression failed aggregate win-rate: candidate {:.3} baseline {:.3}",
        candidate_pool_wr,
        baseline_pool_wr
    );

    persist_ladder_artifacts(artifacts.as_slice());
}

#[test]
#[ignore = "reliability gate: retained pro profile vs runtime_current pro and normal at pro budget with move-time cap"]
fn smart_automove_pool_pro_reliability_gate() {
    let candidate_profile = env_profile_name("SMART_PRO_RELIABILITY_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".to_string());
    let baseline_profile = env_profile_name("SMART_PRO_RELIABILITY_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".to_string());
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate '{}' not found", candidate_profile));
    let baseline_selector = profile_selector_from_name(baseline_profile.as_str())
        .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile));

    let skip_guardrails = env_bool("SMART_PRO_RELIABILITY_SKIP_GUARDRAILS").unwrap_or(false);
    if skip_guardrails {
        println!("pro reliability gate: guardrails skipped by SMART_PRO_RELIABILITY_SKIP_GUARDRAILS=true");
    } else {
        assert_runtime_preflight_if_required(candidate_profile.as_str(), candidate_selector);
        assert_tactical_guardrails(candidate_selector, candidate_profile.as_str());
        assert_tactical_guardrails(baseline_selector, baseline_profile.as_str());
    }

    let repeats = env_usize("SMART_PRO_RELIABILITY_REPEATS")
        .unwrap_or(3)
        .max(1);
    let games = env_usize("SMART_PRO_RELIABILITY_GAMES").unwrap_or(2).max(1);
    let max_plies_floor = if skip_guardrails { 8 } else { 56 };
    let max_plies = env_usize("SMART_PRO_RELIABILITY_MAX_PLIES")
        .unwrap_or(96)
        .max(max_plies_floor);
    let seed_tag = env_profile_name("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());
    let normal_seed_tag = format!("{}_vs_normal", seed_tag);

    let pro_stats = run_cross_budget_duel_with_timing(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Pro,
        seed_tag: seed_tag.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let normal_stats = run_cross_budget_duel_with_timing(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Normal,
        seed_tag: normal_seed_tag.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });

    let pro_total_games = pro_stats.matchup.total_games();
    let pro_metrics = ProReliabilityGateMetrics {
        win_rate: pro_stats.matchup.win_rate_points(),
        confidence: pro_stats.matchup.confidence_better_than_even(),
        candidate_avg_ms: pro_stats.timing.candidate_avg_ms(),
    };
    println!(
        "pro reliability gate vs current pro: candidate={} baseline={} total_games={} win_rate={:.4} confidence={:.4} candidate_avg_ms={:.2} baseline_avg_ms={:.2} candidate_turns={} baseline_turns={}",
        candidate_profile,
        baseline_profile,
        pro_total_games,
        pro_metrics.win_rate,
        pro_metrics.confidence,
        pro_metrics.candidate_avg_ms,
        pro_stats.timing.baseline_avg_ms(),
        pro_stats.timing.candidate_turns,
        pro_stats.timing.baseline_turns
    );

    let normal_total_games = normal_stats.matchup.total_games();
    let normal_metrics = ProReliabilityGateMetrics {
        win_rate: normal_stats.matchup.win_rate_points(),
        confidence: normal_stats.matchup.confidence_better_than_even(),
        candidate_avg_ms: normal_stats.timing.candidate_avg_ms(),
    };
    println!(
        "pro reliability gate vs current normal: candidate={} baseline={} total_games={} win_rate={:.4} confidence={:.4} candidate_avg_ms={:.2} baseline_avg_ms={:.2} candidate_turns={} baseline_turns={}",
        candidate_profile,
        baseline_profile,
        normal_total_games,
        normal_metrics.win_rate,
        normal_metrics.confidence,
        normal_metrics.candidate_avg_ms,
        normal_stats.timing.baseline_avg_ms(),
        normal_stats.timing.candidate_turns,
        normal_stats.timing.baseline_turns
    );

    let expected_games = repeats.saturating_mul(games).saturating_mul(2);
    assert_eq!(
        pro_total_games, expected_games,
        "pro reliability gate vs current pro expected {} mirrored games but ran {}",
        expected_games, pro_total_games
    );
    assert!(
        normal_total_games == expected_games,
        "pro reliability gate vs current normal expected {} mirrored games but ran {}",
        expected_games,
        normal_total_games
    );
    assert!(
        pro_reliability_gate_passes(pro_metrics, normal_metrics),
        "pro reliability gate failed overall: vs_current_pro [win_rate {:.4} confidence {:.4} candidate_avg_ms {:.2}ms] vs_current_normal [win_rate {:.4} confidence {:.4} candidate_avg_ms {:.2}ms] (required each duel to satisfy win_rate >= {:.2}, confidence >= {:.2}, candidate_avg_ms <= {:.2}ms)",
        pro_metrics.win_rate,
        pro_metrics.confidence,
        pro_metrics.candidate_avg_ms,
        normal_metrics.win_rate,
        normal_metrics.confidence,
        normal_metrics.candidate_avg_ms,
        SMART_PRO_RELIABILITY_WIN_RATE_MIN,
        SMART_PRO_RELIABILITY_CONFIDENCE_MIN,
        SMART_PRO_RELIABILITY_MOVE_AVG_MS_MAX
    );
    assert_pro_reliability_duel_passes("pro reliability gate vs current pro", pro_metrics);
    assert_pro_reliability_duel_passes("pro reliability gate vs current normal", normal_metrics);
}

#[test]
#[ignore = "pro fast screen against runtime normal baseline"]
fn smart_automove_pool_pro_fast_screen_vs_normal() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate '{}' not found", candidate_profile));
    let seed_tag = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "pro_fast_screen_vs_normal_v1".to_string());
    let repeats = env_usize("SMART_PRO_FAST_SCREEN_REPEATS")
        .unwrap_or(2)
        .max(1);
    let games = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(84)
        .max(56);

    assert_runtime_preflight_if_required(candidate_profile.as_str(), candidate_selector);

    let stats = run_cross_budget_duel(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Normal,
        seed_tag: seed_tag.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let (delta, confidence) = stats_delta_confidence(stats);
    println!(
        "pro fast screen vs normal: profile={} baseline={} delta={:.4} confidence={:.3}",
        candidate_profile, baseline_profile, delta, confidence
    );
    assert!(
        delta >= SMART_PRO_FAST_SCREEN_DELTA_MIN,
        "pro fast screen vs normal failed: delta {:.4} < {:.4}",
        delta,
        SMART_PRO_FAST_SCREEN_DELTA_MIN
    );
}

#[test]
#[ignore = "pro fast screen against runtime fast baseline"]
fn smart_automove_pool_pro_fast_screen_vs_fast() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate '{}' not found", candidate_profile));
    let seed_tag = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "pro_fast_screen_vs_fast_v1".to_string());
    let repeats = env_usize("SMART_PRO_FAST_SCREEN_REPEATS")
        .unwrap_or(2)
        .max(1);
    let games = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(84)
        .max(56);

    assert_runtime_preflight_if_required(candidate_profile.as_str(), candidate_selector);

    let stats = run_cross_budget_duel(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Fast,
        seed_tag: seed_tag.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let (delta, confidence) = stats_delta_confidence(stats);
    println!(
        "pro fast screen vs fast: profile={} baseline={} delta={:.4} confidence={:.3}",
        candidate_profile, baseline_profile, delta, confidence
    );
    assert!(
        delta >= SMART_PRO_FAST_SCREEN_DELTA_MIN,
        "pro fast screen vs fast failed: delta {:.4} < {:.4}",
        delta,
        SMART_PRO_FAST_SCREEN_DELTA_MIN
    );
}

#[test]
#[ignore = "pro progressive duel against runtime normal baseline"]
fn smart_automove_pool_pro_progressive_vs_normal() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate '{}' not found", candidate_profile));
    assert_runtime_preflight_if_required(candidate_profile.as_str(), candidate_selector);
    let (stats, _) = run_pro_progressive_matchup(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        "pro_progressive_vs_normal",
    );
    let (delta, confidence) = stats_delta_confidence(stats);
    println!(
        "pro progressive vs normal: profile={} baseline={} delta={:.4} confidence={:.3}",
        candidate_profile, baseline_profile, delta, confidence
    );
    assert!(
        delta >= 0.0,
        "pro progressive vs normal failed: delta {:.4} < 0.0",
        delta
    );
}

#[test]
#[ignore = "pro progressive duel against runtime fast baseline"]
fn smart_automove_pool_pro_progressive_vs_fast() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate '{}' not found", candidate_profile));
    assert_runtime_preflight_if_required(candidate_profile.as_str(), candidate_selector);
    let (stats, _) = run_pro_progressive_matchup(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        "pro_progressive_vs_fast",
    );
    let (delta, confidence) = stats_delta_confidence(stats);
    println!(
        "pro progressive vs fast: profile={} baseline={} delta={:.4} confidence={:.3}",
        candidate_profile, baseline_profile, delta, confidence
    );
    assert!(
        delta >= 0.0,
        "pro progressive vs fast failed: delta {:.4} < 0.0",
        delta
    );
}

#[test]
#[ignore = "strict pro promotion ladder against fast and normal baselines"]
fn smart_automove_pool_pro_primary_stage() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();

    let primary_games = env_usize("SMART_PRO_GATE_PRIMARY_GAMES")
        .unwrap_or(6)
        .max(1);
    let primary_repeats = env_usize("SMART_PRO_GATE_PRIMARY_REPEATS")
        .unwrap_or(6)
        .max(1);
    let primary_max_plies = env_usize("SMART_PRO_GATE_PRIMARY_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let primary_seed_tags = ["neutral_v1", "neutral_v2", "neutral_v3"];

    let vs_normal_stats = run_pro_matchup_across_seeds(ProMatchupAcrossSeedsConfig {
        candidate_profile: candidate_profile.as_str(),
        baseline_profile: baseline_profile.as_str(),
        baseline_mode: SmartAutomovePreference::Normal,
        seed_tag_prefix: "pro_primary_vs_normal",
        seed_tags: &primary_seed_tags,
        repeats: primary_repeats,
        games_per_seed: primary_games,
        max_plies: primary_max_plies,
        use_white_opening_book: false,
    });
    let vs_fast_stats = run_pro_matchup_across_seeds(ProMatchupAcrossSeedsConfig {
        candidate_profile: candidate_profile.as_str(),
        baseline_profile: baseline_profile.as_str(),
        baseline_mode: SmartAutomovePreference::Fast,
        seed_tag_prefix: "pro_primary_vs_fast",
        seed_tags: &primary_seed_tags,
        repeats: primary_repeats,
        games_per_seed: primary_games,
        max_plies: primary_max_plies,
        use_white_opening_book: false,
    });
    let (vs_normal_delta, vs_normal_confidence) = stats_delta_confidence(vs_normal_stats);
    let (vs_fast_delta, vs_fast_confidence) = stats_delta_confidence(vs_fast_stats);
    println!(
        "pro primary summary: profile={} baseline={} vs_normal delta={:.4} confidence={:.3} vs_fast delta={:.4} confidence={:.3}",
        candidate_profile,
        baseline_profile,
        vs_normal_delta,
        vs_normal_confidence,
        vs_fast_delta,
        vs_fast_confidence
    );
}

#[test]
#[ignore = "strict pro promotion ladder against fast and normal baselines"]
fn smart_automove_pool_pro_promotion_ladder() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate selector '{}' should exist", candidate_profile));
    let baseline_selector = profile_selector_from_name(baseline_profile.as_str())
        .unwrap_or_else(|| panic!("baseline selector '{}' should exist", baseline_profile));
    assert_runtime_preflight_if_required(candidate_profile.as_str(), candidate_selector);

    assert_tactical_guardrails(candidate_selector, candidate_profile.as_str());
    assert_interview_policy_regressions(candidate_selector, candidate_profile.as_str());
    assert_tactical_guardrails(baseline_selector, baseline_profile.as_str());
    assert_interview_policy_regressions(baseline_selector, baseline_profile.as_str());

    let speed_positions = env_usize("SMART_PRO_GATE_SPEED_POSITIONS")
        .unwrap_or(12)
        .max(4);
    let speed_seed = seed_for_pairing("pro_promotion_ladder", "speed");
    let speed_openings = generate_opening_fens_cached(speed_seed, speed_positions);
    let pro_ms = profile_speed_by_mode_ms(
        candidate_selector,
        speed_openings.as_slice(),
        &[pro_budget()],
    )
    .first()
    .map(|stat| stat.avg_ms)
    .unwrap_or(0.0);
    let normal_ms = profile_speed_by_mode_ms(
        baseline_selector,
        speed_openings.as_slice(),
        &[SearchBudget::from_preference(
            SmartAutomovePreference::Normal,
        )],
    )
    .first()
    .map(|stat| stat.avg_ms)
    .unwrap_or(1.0)
    .max(0.001);
    let speed_ratio = pro_ms / normal_ms;
    println!(
        "pro speed gate: candidate={} baseline={} pro_ms={:.2} normal_ms={:.2} ratio={:.3} target=[{:.3}, {:.3}]",
        candidate_profile,
        baseline_profile,
        pro_ms,
        normal_ms,
        speed_ratio,
        SMART_PRO_CPU_RATIO_TARGET_MIN,
        SMART_PRO_CPU_RATIO_TARGET_MAX
    );
    assert!(
        speed_ratio >= SMART_PRO_CPU_RATIO_TARGET_MIN,
        "pro cpu gate below target: ratio {:.3} < {:.3}",
        speed_ratio,
        SMART_PRO_CPU_RATIO_TARGET_MIN
    );
    assert!(
        speed_ratio <= SMART_PRO_CPU_RATIO_TARGET_MAX,
        "pro cpu gate above hard cap: ratio {:.3} > {:.3}",
        speed_ratio,
        SMART_PRO_CPU_RATIO_TARGET_MAX
    );

    let primary_games = env_usize("SMART_PRO_GATE_PRIMARY_GAMES")
        .unwrap_or(6)
        .max(2);
    let primary_repeats = env_usize("SMART_PRO_GATE_PRIMARY_REPEATS")
        .unwrap_or(6)
        .max(2);
    let primary_max_plies = env_usize("SMART_PRO_GATE_PRIMARY_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let primary_seed_tags = ["neutral_v1", "neutral_v2", "neutral_v3"];

    let vs_normal_stats = run_pro_matchup_across_seeds(ProMatchupAcrossSeedsConfig {
        candidate_profile: candidate_profile.as_str(),
        baseline_profile: baseline_profile.as_str(),
        baseline_mode: SmartAutomovePreference::Normal,
        seed_tag_prefix: "pro_primary_vs_normal",
        seed_tags: &primary_seed_tags,
        repeats: primary_repeats,
        games_per_seed: primary_games,
        max_plies: primary_max_plies,
        use_white_opening_book: false,
    });
    let vs_fast_stats = run_pro_matchup_across_seeds(ProMatchupAcrossSeedsConfig {
        candidate_profile: candidate_profile.as_str(),
        baseline_profile: baseline_profile.as_str(),
        baseline_mode: SmartAutomovePreference::Fast,
        seed_tag_prefix: "pro_primary_vs_fast",
        seed_tags: &primary_seed_tags,
        repeats: primary_repeats,
        games_per_seed: primary_games,
        max_plies: primary_max_plies,
        use_white_opening_book: false,
    });
    let (vs_normal_delta, vs_normal_confidence) = stats_delta_confidence(vs_normal_stats);
    let (vs_fast_delta, vs_fast_confidence) = stats_delta_confidence(vs_fast_stats);
    println!(
        "pro primary summary: vs_normal delta={:.4} confidence={:.3} vs_fast delta={:.4} confidence={:.3}",
        vs_normal_delta,
        vs_normal_confidence,
        vs_fast_delta,
        vs_fast_confidence
    );
    assert!(
        vs_normal_delta >= SMART_PRO_PRIMARY_IMPROVEMENT_DELTA_MIN_VS_NORMAL,
        "pro primary vs normal failed: delta {:.4} < {:.4}",
        vs_normal_delta,
        SMART_PRO_PRIMARY_IMPROVEMENT_DELTA_MIN_VS_NORMAL
    );
    assert!(
        vs_normal_confidence >= SMART_PRO_PRIMARY_IMPROVEMENT_CONFIDENCE_MIN,
        "pro primary vs normal confidence failed: {:.3} < {:.3}",
        vs_normal_confidence,
        SMART_PRO_PRIMARY_IMPROVEMENT_CONFIDENCE_MIN
    );
    assert!(
        vs_fast_delta >= SMART_PRO_PRIMARY_IMPROVEMENT_DELTA_MIN_VS_FAST,
        "pro primary vs fast failed: delta {:.4} < {:.4}",
        vs_fast_delta,
        SMART_PRO_PRIMARY_IMPROVEMENT_DELTA_MIN_VS_FAST
    );
    assert!(
        vs_fast_confidence >= SMART_PRO_PRIMARY_IMPROVEMENT_CONFIDENCE_MIN,
        "pro primary vs fast confidence failed: {:.3} < {:.3}",
        vs_fast_confidence,
        SMART_PRO_PRIMARY_IMPROVEMENT_CONFIDENCE_MIN
    );

    let confirm_games = env_usize("SMART_PRO_GATE_CONFIRM_GAMES")
        .unwrap_or(4)
        .max(2);
    let confirm_repeats = env_usize("SMART_PRO_GATE_CONFIRM_REPEATS")
        .unwrap_or(4)
        .max(2);
    let confirm_max_plies = env_usize("SMART_PRO_GATE_CONFIRM_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let confirm_vs_normal = run_cross_budget_duel(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Normal,
        seed_tag: "pro_confirm_vs_normal_v1",
        repeats: confirm_repeats,
        games_per_repeat: confirm_games,
        max_plies: confirm_max_plies,
        use_white_opening_book: true,
    });
    let confirm_vs_fast = run_cross_budget_duel(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Fast,
        seed_tag: "pro_confirm_vs_fast_v1",
        repeats: confirm_repeats,
        games_per_repeat: confirm_games,
        max_plies: confirm_max_plies,
        use_white_opening_book: true,
    });
    let confirm_tolerance: f64 = std::env::var("SMART_PRO_GATE_CONFIRM_TOLERANCE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(-0.10);
    let (cn_delta, cn_conf) = stats_delta_confidence(confirm_vs_normal);
    let (cf_delta, cf_conf) = stats_delta_confidence(confirm_vs_fast);
    println!(
        "pro confirmation vs_normal delta={:.4} confidence={:.4}  vs_fast delta={:.4} confidence={:.4}  tolerance={:.2}",
        cn_delta, cn_conf, cf_delta, cf_conf, confirm_tolerance
    );
    assert!(
        cn_delta >= confirm_tolerance,
        "pro confirmation vs normal failed non-regression: delta={cn_delta:.4} < tolerance={confirm_tolerance:.2}"
    );
    assert!(
        cf_delta >= confirm_tolerance,
        "pro confirmation vs fast failed non-regression: delta={cf_delta:.4} < tolerance={confirm_tolerance:.2}"
    );

    let pool_games = env_usize("SMART_PRO_GATE_POOL_GAMES").unwrap_or(1).max(1);
    let pool_max_plies = env_usize("SMART_PRO_GATE_POOL_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let candidate_pool_vs_normal = run_profile_vs_pool_cross_budget(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        SmartAutomovePreference::Normal,
        pool_games,
        pool_max_plies,
        "pro_pool_vs_normal",
    );
    let baseline_pool_vs_normal = run_profile_vs_pool_cross_budget(
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        SmartAutomovePreference::Normal,
        pool_games,
        pool_max_plies,
        "baseline_pool_vs_normal",
    );
    let candidate_pool_vs_fast = run_profile_vs_pool_cross_budget(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        SmartAutomovePreference::Fast,
        pool_games,
        pool_max_plies,
        "pro_pool_vs_fast",
    );
    let baseline_pool_vs_fast = run_profile_vs_pool_cross_budget(
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        SmartAutomovePreference::Fast,
        pool_games,
        pool_max_plies,
        "baseline_pool_vs_fast",
    );
    assert!(
        stats_delta_confidence(candidate_pool_vs_normal).0 + 0.01
            >= stats_delta_confidence(baseline_pool_vs_normal).0,
        "pro pool non-regression vs normal-opponents failed"
    );
    println!(
        "pro pool summary: candidate_vs_normal={:.4} baseline_vs_normal={:.4} candidate_vs_fast={:.4} baseline_vs_fast={:.4}",
        stats_delta_confidence(candidate_pool_vs_normal).0,
        stats_delta_confidence(baseline_pool_vs_normal).0,
        stats_delta_confidence(candidate_pool_vs_fast).0,
        stats_delta_confidence(baseline_pool_vs_fast).0
    );
    assert!(
        stats_delta_confidence(candidate_pool_vs_fast).0 + 0.01
            >= stats_delta_confidence(baseline_pool_vs_fast).0,
        "pro pool non-regression vs fast-opponents failed"
    );
}

#[test]
#[ignore = "diagnostic: replay human-wins-vs-pro games and analyze bot mistakes"]
fn smart_automove_human_wins_diagnostic() {
    struct GameSpec {
        label: &'static str,
        bot_color: Color,
        moves: Vec<&'static str>,
    }

    let games = vec![
        GameSpec {
            label: "game1_human_white",
            bot_color: Color::Black,
            moves: vec![
                "l10,6;l9,7",
                "l9,7;l8,6",
                "l8,6;l7,5",
                "l10,4;l9,4",
                "l9,4;l8,5",
                "l0,4;l1,5",
                "l1,5;l0,7;l1,8",
                "l1,8;l2,9",
                "l2,9;l3,10",
                "l3,10;l4,10",
                "l4,10;l5,10;mp",
                "l3,6;l2,7",
                "l7,5;l5,5;l6,4",
                "l10,5;l9,4",
                "l9,4;l8,4",
                "l7,5;l8,6",
                "l10,3;l9,2",
                "l10,7;l9,6",
                "l6,3;l7,3",
                "l1,5;l2,4",
                "l1,5;l2,5",
                "l2,5;l3,5",
                "l3,5;l4,4",
                "l4,4;l6,4;l5,3",
                "l0,5;l1,5",
                "l1,5;l2,5",
                "l3,4;l2,3",
                "l8,6;l8,4;l7,5",
                "l7,5;l7,6",
                "l8,6;l7,5",
                "l9,2;l8,1",
                "l8,1;l7,0",
                "l7,0;l6,1",
                "l6,7;l7,7",
                "l0,3;l1,2",
                "l1,2;l2,1",
                "l2,1;l3,0",
                "l3,0;l4,0",
                "l4,0;l5,0;mb",
                "l5,0;l6,1",
                "l4,4;l2,3;l1,2",
                "l1,2;l0,1",
                "l7,5;l5,3;l6,3",
                "l9,6;l8,7",
                "l8,5;l8,6",
                "l7,6;l7,7",
                "l7,5;l8,4",
                "l8,7;l7,8",
                "l7,3;l8,2",
                "l4,4;l5,3",
                "l5,3;l6,2",
                "l6,2;l8,2;l9,1",
                "l6,2;l7,1",
                "l7,1;l9,1;l10,0",
                "l2,5;l3,4",
                "l3,4;l4,3",
                "l0,1;l0,0",
                "l7,8;l6,7",
                "l6,7;l5,6",
                "l5,6;l4,6",
                "l4,6;l3,5",
                "l3,5;l2,5",
                "l2,5;l4,3",
                "l7,6;l8,7",
                "l7,1;l6,3;l5,2",
                "l0,6;l1,5",
                "l1,5;l2,4",
                "l2,4;l3,3",
                "l3,3;l4,2",
                "l4,2;l5,1",
                "l4,3;l3,2",
                "l8,4;l6,5;l6,4",
                "l8,4;l8,5",
                "l2,5;l3,4",
                "l2,5;l3,4",
                "l3,4;l4,3",
                "l7,7;l8,7",
                "l4,3;l3,3",
                "l7,4;l8,4",
                "l7,1;l5,2;l4,1",
                "l0,5;l1,4",
                "l1,4;l2,3",
                "l2,3;l3,2",
                "l3,2;l4,1",
                "l5,1;l4,0",
                "l3,2;l2,2",
                "l8,5;l7,7;l8,8",
                "l3,3;l4,2",
                "l8,6;l7,7",
                "l8,7;l8,8",
                "l8,8;l9,9",
                "l9,9;l10,10",
                "l8,7;l8,8",
                "l4,1;l3,0",
                "l3,0;l2,0",
                "l2,0;l1,0",
                "l1,0;l0,0",
            ],
        },
        GameSpec {
            label: "game2_human_black",
            bot_color: Color::White,
            moves: vec![
                "l10,3;l9,2",
                "l9,2;l8,1",
                "l8,1;l7,0",
                "l7,0;l6,0",
                "l6,0;l5,0;mp",
                "l0,4;l1,5",
                "l1,5;l3,6;l2,7",
                "l0,5;l1,6",
                "l0,3;l1,3",
                "l0,7;l1,7",
                "l1,5;l2,4",
                "l3,4;l2,3",
                "l10,6;l9,5",
                "l9,5;l8,5",
                "l8,5;l7,5",
                "l7,5;l5,5;l6,6",
                "l10,5;l9,5",
                "l9,5;l8,5",
                "l7,6;l8,7",
                "l2,4;l4,3;l3,2",
                "l1,3;l2,2",
                "l0,6;l1,5",
                "l1,6;l2,6",
                "l1,7;l2,8",
                "l2,8;l3,7",
                "l2,3;l1,2",
                "l7,5;l6,4",
                "l6,4;l5,3",
                "l5,3;l4,2",
                "l4,2;l3,1",
                "l3,1;l1,2;l1,1",
                "l3,1;l1,1;l0,0",
                "l10,4;l9,5",
                "l8,7;l9,8",
                "l3,7;l4,8",
                "l4,8;l4,9",
                "l4,9;l5,10;mb",
                "l5,10;l4,9",
                "l4,9;l5,8",
                "l5,8;l8,5",
                "l2,4;l4,5;l3,6",
                "l2,7;l1,8",
                "l3,1;l4,2",
                "l4,2;l5,3",
                "l5,3;l6,4",
                "l6,4;l6,6;l7,7",
                "l6,4;l7,5",
                "l9,5;l8,5",
                "l9,8;l10,9",
                "l2,4;l3,6;l2,6",
                "l1,5;l1,6",
                "l2,6;l1,7",
                "l1,7;l0,8",
                "l0,8;l0,9",
                "l0,9;l0,10",
                "l1,8;l0,9",
                "l7,5;l7,7;l8,7",
                "l10,5;l9,6",
                "l9,6;l8,7",
                "l8,7;l8,8",
                "l8,8;l9,9",
                "l9,9;l10,10",
                "l10,9;l10,10",
            ],
        },
        GameSpec {
            label: "game3_human_white",
            bot_color: Color::Black,
            moves: vec![
                "l10,5;l9,5",
                "l9,5;l8,5",
                "l10,6;l9,6",
                "l9,6;l8,6",
                "l10,4;l9,5",
                "l0,4;l1,5",
                "l1,5;l0,7;l1,8",
                "l1,8;l2,9",
                "l2,9;l3,10",
                "l3,10;l4,10",
                "l4,10;l5,10;mp",
                "l3,6;l2,7",
                "l8,6;l7,4;l8,5",
                "l8,5;l8,4",
                "l8,4;l9,3",
                "l9,3;l9,2",
                "l9,2;l9,1",
                "l9,1;l10,0",
                "l6,3;l6,4",
                "l1,5;l2,5",
                "l2,5;l3,5",
                "l3,5;l5,5;l4,4",
                "l0,5;l1,5",
                "l1,5;l2,5",
                "l0,6;l1,5",
                "l3,4;l2,3",
                "l8,6;l6,7;l7,8",
                "l10,3;l9,2",
                "l10,7;l9,8",
                "l9,2;l8,1",
                "l8,1;l7,0",
                "l7,0;l6,0",
                "l7,8;l8,8",
                "l0,3;l1,2",
                "l1,2;l2,2",
                "l2,2;l3,2",
                "l3,2;l4,2",
                "l4,2;l6,0",
                "l1,5;l2,4",
                "l2,3;l1,2",
                "l8,6;l6,5;l6,6",
                "l9,5;l8,4",
                "l8,4;l8,3",
                "l8,3;l8,2",
                "l8,2;l9,1",
                "l9,1;l9,0",
                "l7,6;l7,7",
                "l3,5;l4,6",
                "l4,6;l5,7",
                "l5,7;l6,8",
                "l6,8;l8,8;l9,9",
                "l6,8;l7,9",
                "l7,9;l9,9;l10,10",
                "l2,5;l3,4",
                "l4,2;l5,3",
                "l1,2;l0,1",
                "l10,3;l9,4",
                "l8,6;l9,8;l10,8",
                "l9,4;l8,4",
                "l8,4;l7,4",
                "l7,4;l6,5",
                "l9,0;l9,1",
                "l7,7;l8,8",
                "l5,3;l5,2",
                "l5,2;l5,1",
                "l5,1;l5,0;mp",
                "l7,9;l7,10",
                "l7,10;l8,8;l9,9",
                "l7,10;l9,9;l10,10",
                "l0,1;l0,0",
            ],
        },
        GameSpec {
            label: "game4_human_black",
            bot_color: Color::White,
            moves: vec![
                "l10,4;l9,5",
                "l9,5;l8,5",
                "l8,5;l7,5",
                "l7,5;l6,4",
                "l6,4;l5,4",
                "l0,4;l1,5",
                "l1,5;l3,6;l2,7",
                "l0,5;l1,6",
                "l0,3;l1,3",
                "l0,7;l1,7",
                "l1,5;l2,4",
                "l4,3;l3,2",
                "l10,5;l9,5",
                "l9,5;l8,5",
                "l10,6;l9,7",
                "l9,7;l8,5;l7,5",
                "l7,5;l6,5",
                "l6,5;l5,5",
                "l7,6;l8,7",
                "l2,4;l1,6;l2,5",
                "l0,6;l1,6",
                "l2,4;l3,3",
                "l3,3;l4,3",
                "l1,3;l2,2",
                "l2,2;l3,1",
                "l2,7;l1,8",
                "l5,5;l6,6",
                "l6,6;l7,7",
                "l7,7;l8,8",
                "l8,8;l9,9",
                "l9,7;l9,9;l10,10",
                "l9,7;l8,8",
                "l8,7;l9,8",
                "l4,3;l3,1;l4,0",
                "l4,0;l5,0;mp",
                "l2,5;l2,4",
                "l4,3;l6,3;l5,2",
                "l1,6;l2,5",
                "l1,7;l2,6",
                "l2,6;l3,5",
                "l1,8;l0,9",
                "l8,8;l7,9",
                "l7,9;l9,8;l10,9",
                "l7,9;l6,10",
                "l6,10;l5,10;mp",
                "l5,10;l5,9",
                "l5,9;l5,8",
                "l10,9;l10,10",
                "l3,5;l4,6",
                "l4,6;l5,7",
                "l5,7;l6,8",
                "l6,8;l7,9",
                "l7,9;l8,10",
                "l8,10;l10,10",
                "l0,9;l0,10",
                "l10,7;l9,8",
                "l9,8;l8,8",
                "l8,8;l10,10",
                "l5,8;l5,7",
                "l5,7;l5,6",
                "l10,3;l9,2",
                "l5,2;l6,3",
                "l4,3;l2,4;l3,4",
                "l3,4;l2,3",
                "l2,3;l1,2",
                "l1,2;l0,1",
                "l0,1;l0,0",
                "l2,5;l2,4",
                "l3,2;l2,1",
                "l5,6;l4,6",
                "l4,6;l3,5",
                "l3,5;l3,4",
                "l3,4;l3,3",
                "l3,3;l2,1;l1,1",
                "l3,3;l1,1;l0,0",
            ],
        },
        GameSpec {
            label: "game5_human_white",
            bot_color: Color::Black,
            moves: vec![
                "l10,5;l9,5",
                "l9,5;l8,5",
                "l10,3;l9,2",
                "l10,6;l9,6",
                "l9,6;l8,7",
                "l0,4;l1,5",
                "l1,5;l2,5",
                "l2,5;l3,5",
                "l3,5;l5,5;l4,4",
                "l0,5;l1,5",
                "l1,5;l2,5",
                "l3,6;l2,7",
                "l8,7;l7,8",
                "l7,8;l6,9",
                "l6,9;l5,10;mb",
                "l5,10;l4,9",
                "l4,9;l4,8",
                "l4,8;l2,5",
                "l4,8;l2,7;l3,6",
                "l7,4;l8,5",
                "l3,5;l4,3;l3,2",
                "l0,3;l1,2",
                "l1,2;l2,1",
                "l2,1;l3,0",
                "l3,0;l4,0",
                "l4,0;l5,0;mp",
                "l3,2;l2,1",
                "l4,8;l3,6;l4,6",
                "l8,5;l9,4",
                "l9,4;l9,3",
                "l4,8;l5,7",
                "l5,7;l6,6",
                "l10,7;l9,6",
                "l6,3;l7,2",
                "l0,5;l1,5",
                "l0,5;l1,5",
                "l3,5;l1,5;l2,5",
                "l2,5;l3,4",
                "l3,4;l4,4",
                "l4,4;l3,3",
                "l3,3;l2,3",
                "l3,5;l2,3;l1,2",
                "l3,4;l2,3",
                "l6,6;l4,7;l5,8",
                "l9,6;l8,5",
                "l10,4;l9,4",
                "l8,5;l7,4",
                "l7,4;l6,4",
                "l9,2;l8,1",
                "l7,2;l8,2",
                "l1,2;l0,1",
                "l0,1;l0,0",
                "l3,5;l2,3;l1,2",
                "l0,0;l1,1",
                "l1,1;l2,2",
                "l2,2;l3,3",
                "l1,2;l0,1",
                "l6,4;l5,3",
                "l5,3;l4,2",
                "l4,2;l5,1",
                "l5,1;l3,3",
                "l9,4;l8,4",
                "l8,1;l7,0",
                "l8,2;l9,1",
                "l3,5;l4,4",
                "l4,4;l5,3",
                "l5,3;l6,2",
                "l6,2;l7,1",
                "l7,1;l9,1;l10,0",
                "l0,1;l0,0",
            ],
        },
    ];

    let budget = pro_budget();

    for spec in &games {
        println!("\n{}", "=".repeat(70));
        println!("GAME: {} (bot={:?})", spec.label, spec.bot_color);
        println!("{}", "=".repeat(70));

        let mut game = MonsGame::new(false);
        let mut move_idx = 0;
        let mut turn_number = 0;
        let mut bot_turn_divergences = Vec::<String>::new();

        // At each turn start (actions_used == 0), run the search and record what the bot
        // would play as a complete turn. Then replay actions to advance the game.
        let mut last_turn_analyzed: Option<(Color, usize)> = None;

        for move_fen in &spec.moves {
            move_idx += 1;
            let current_color = game.active_color;
            let actions_used = game.actions_used_count;

            // Analyze at turn start (first action of a new turn)
            if actions_used == 0 {
                let is_new_turn = last_turn_analyzed
                    .map(|(c, t)| c != current_color || t != game.turn_number as usize)
                    .unwrap_or(true);
                if is_new_turn {
                    last_turn_analyzed = Some((current_color, game.turn_number as usize));
                    turn_number += 1;
                    let is_bot_turn = current_color == spec.bot_color;
                    let position_fen = game.fen();

                    let config = budget.runtime_config_for_game(&game);
                    let search_best = MonsGameModel::smart_search_best_inputs(&game, config);
                    let search_best_fen = Input::fen_from_array(&search_best);

                    let ranked = MonsGameModel::ranked_root_moves(&game, current_color, config);
                    let top_h = ranked.first().map(|r| r.heuristic).unwrap_or(0);
                    let search_rank = ranked
                        .iter()
                        .position(|r| Input::fen_from_array(&r.inputs) == search_best_fen);
                    let search_h = search_rank
                        .and_then(|i| ranked.get(i))
                        .map(|r| r.heuristic)
                        .unwrap_or(0);

                    let label = if is_bot_turn { "BOT" } else { "HUMAN" };
                    println!(
                        "\n  Turn {} ({:?} {}, move_idx={}, fen={})",
                        turn_number, current_color, label, move_idx, position_fen,
                    );
                    println!(
                        "    search_best={} rank={} h={} | top_h={}",
                        search_best_fen,
                        search_rank
                            .map(|r| format!("#{}", r + 1))
                            .unwrap_or("?".into()),
                        search_h,
                        top_h,
                    );
                    // Print top-5 ranked moves with key flags
                    for (i, root) in ranked.iter().take(5).enumerate() {
                        let root_fen = Input::fen_from_array(&root.inputs);
                        println!(
                            "    #{}: {} h={} eff={} wins={} atk_drn={} vuln={} spirit={} handoff={} roundtrip={} sup={} opp={} intv={}",
                            i + 1, root_fen, root.heuristic, root.efficiency,
                            root.wins_immediately, root.attacks_opponent_drainer,
                            root.own_drainer_vulnerable, root.spirit_development,
                            root.mana_handoff_to_opponent, root.has_roundtrip,
                            root.supermana_progress, root.opponent_mana_progress,
                            root.interview_soft_priority,
                        );
                    }

                    if is_bot_turn {
                        // Check: does the first action of the played turn match the
                        // first action of the search best?
                        let search_first_action = search_best_fen
                            .split(';')
                            .take(2)
                            .collect::<Vec<_>>()
                            .join(";");
                        let played_first = move_fen.to_string();
                        // Also find which ranked move starts with the played first action
                        let played_move_rank = ranked.iter().position(|r| {
                            let r_fen = Input::fen_from_array(&r.inputs);
                            r_fen.starts_with(&played_first)
                        });
                        let played_h = played_move_rank
                            .and_then(|i| ranked.get(i))
                            .map(|r| r.heuristic)
                            .unwrap_or(0);
                        let gap = top_h - played_h;

                        if !search_first_action.starts_with(&played_first) || gap > 100 {
                            let msg = format!(
                                "  DIVERGENCE turn {} (move {}): played_start={} best_match_rank={} h={} gap={} | search_start={}",
                                turn_number, move_idx, played_first,
                                played_move_rank.map(|r| format!("#{}", r + 1)).unwrap_or("?".into()),
                                played_h, gap, search_first_action,
                            );
                            bot_turn_divergences.push(msg);
                        }
                    }
                }
            }

            let inputs = Input::array_from_fen(move_fen);
            let output = game.process_input(inputs, false, false);
            if !matches!(output, Output::Events(_)) {
                continue;
            }
        }

        println!(
            "\n  Final Score: W={} B={} (bot={:?})",
            game.white_score, game.black_score, spec.bot_color
        );
        println!("  Bot turn divergences: {}", bot_turn_divergences.len());
        for msg in &bot_turn_divergences {
            println!("    {}", msg);
        }
    }
}

#[test]
#[ignore = "diagnostic: probe primary_pro triage fixture gaps"]
fn smart_automove_pro_fixture_gap_probe() {
    let budget = pro_budget();
    let fixtures = primary_pro_triage_fixtures();

    for fixture in &fixtures {
        let game = &fixture.game;
        let config = budget.runtime_config_for_game(game);
        let search_best = MonsGameModel::smart_search_best_inputs(game, config);
        let search_best_fen = Input::fen_from_array(&search_best);

        let ranked = MonsGameModel::ranked_root_moves(game, game.active_color, config);
        let _top_h = ranked.first().map(|r| r.heuristic).unwrap_or(0);
        let search_rank = ranked
            .iter()
            .position(|r| Input::fen_from_array(&r.inputs) == search_best_fen);
        let gap_1_2 = if ranked.len() >= 2 {
            ranked[0].heuristic.saturating_sub(ranked[1].heuristic)
        } else {
            0
        };

        println!(
            "\n--- fixture={} mode={} active={:?} ---",
            fixture.id,
            fixture.mode.as_api_value(),
            game.active_color
        );
        println!(
            "  search_best={} rank=#{} gap_1v2={} root_count={}",
            search_best_fen,
            search_rank
                .map(|r| format!("{}", r + 1))
                .unwrap_or("?".into()),
            gap_1_2,
            ranked.len(),
        );
        for (i, r) in ranked.iter().take(8).enumerate() {
            println!("    #{}: {} h={} eff={} wins={} atk_drn={} vuln={} spirit={} sup={} opp={} intv={}",
                i + 1,
                Input::fen_from_array(&r.inputs),
                r.heuristic,
                r.efficiency,
                r.wins_immediately,
                r.attacks_opponent_drainer,
                r.own_drainer_vulnerable,
                r.spirit_development,
                r.supermana_progress,
                r.opponent_mana_progress,
                r.interview_soft_priority,
            );
        }
        println!("  fen={}", game.fen());
    }
}

#[test]
#[ignore = "diagnostic: inspect turn-engine decisions on named primary_pro fixtures"]
fn smart_automove_pro_turn_engine_fixture_probe() {
    let profile = env_profile_name("SMART_PROBE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".to_string());
    let fixture_filter = env::var("SMART_PROBE_FIXTURES")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let fixtures = primary_pro_triage_fixtures();

    for fixture in fixtures.iter().filter(|fixture| {
        fixture_filter.is_empty() || fixture_filter.iter().any(|id| id == fixture.id)
    }) {
        clear_turn_engine_diagnostics();
        let decision = loss_probe_decision(profile.as_str(), fixture.mode, &fixture.game);
        let diagnostics = turn_engine_diagnostics_snapshot();
        let config = loss_probe_runtime_config(profile.as_str(), &fixture.game, fixture.mode);
        clear_turn_engine_diagnostics();
        let direct_probe = turn_engine_probe(
            &fixture.game,
            fixture.game.active_color,
            config.turn_engine_mode,
            calibration_turn_engine_config(config),
        );
        let ranked_plan_digests = turn_engine_ranked_plan_digests_for_test(
            &fixture.game,
            fixture.game.active_color,
            calibration_turn_engine_config(config),
            5,
        );
        let direct_diagnostics = turn_engine_diagnostics_snapshot();
        let acceptance_probe =
            MonsGameModel::turn_engine_acceptance_probe_for_test(&fixture.game, config);
        println!(
            "\nfixture={} mode={} active={:?} fen={}",
            fixture.id,
            fixture.mode.as_api_value(),
            fixture.game.active_color,
            fixture.game.fen()
        );
        print_loss_probe_decision("  candidate", &decision);
        println!("  turn_engine_diagnostics={:?}", diagnostics);
        println!("  direct_turn_engine_probe={:?}", direct_probe);
        println!(
            "  direct_turn_engine_ranked_plans={:?}",
            ranked_plan_digests
        );
        println!("  direct_turn_engine_diagnostics={:?}", direct_diagnostics);
        print_turn_engine_acceptance_probe(
            "turn_engine_acceptance_probe",
            acceptance_probe.as_ref(),
        );
    }
}

#[test]
#[ignore = "diagnostic: find Pro positions sensitive to config perturbations"]
fn smart_automove_pro_config_sensitivity_probe() {
    use rand::prelude::*;
    let positions = env_usize("SMART_PROBE_POSITIONS").unwrap_or(100);
    let plies_per_position = env_usize("SMART_PROBE_PLIES").unwrap_or(20);
    let seed = env_usize("SMART_PROBE_SEED").unwrap_or(42) as u64;
    let openings = generate_opening_fens_cached(seed, positions);
    let pro_budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);

    let mut found = 0;

    for (pos_idx, fen) in openings.iter().enumerate() {
        let base_game = MonsGame::from_fen(fen.as_str(), false);
        let Some(base_game) = base_game else { continue };
        let mut game = base_game.clone_for_simulation();
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(pos_idx as u64));
        for _ in 0..plies_per_position {
            if !apply_seeded_random_move(&mut game, &mut rng) {
                break;
            }
        }
        if game.winner_color().is_some() {
            continue;
        }

        let base_config = pro_budget.runtime_config_for_game(&game);
        let base_inputs = MonsGameModel::smart_search_best_inputs(&game, base_config);
        let base_fen = Input::fen_from_array(&base_inputs);

        // Perturbation 1: wider reply-risk shortlist
        let mut p1 = base_config;
        p1.root_reply_risk_shortlist_max = base_config.root_reply_risk_shortlist_max + 4;
        p1.root_reply_risk_reply_limit = base_config.root_reply_risk_reply_limit + 8;

        // Perturbation 2: tighter futility margin
        let mut p2 = base_config;
        p2.futility_margin = 1_800;

        // Perturbation 3: no selective extensions
        let mut p3 = base_config;
        p3.enable_selective_extensions = false;

        // Perturbation 4: wider futility margin
        let mut p4 = base_config;
        p4.futility_margin = 3_000;

        // Perturbation 5: tighter reply-risk
        let mut p5 = base_config;
        p5.root_reply_risk_shortlist_max = (base_config.root_reply_risk_shortlist_max).max(2) - 2;
        p5.root_reply_risk_reply_limit = (base_config.root_reply_risk_reply_limit).max(4) - 4;

        // Perturbation 6: more extensions
        let mut p6 = base_config;
        p6.max_extensions_per_path = 2;
        p6.selective_extension_node_share_bp = 2_500;

        let perturbations = [
            ("wider_reply", p1),
            ("tight_futility", p2),
            ("no_extensions", p3),
            ("wide_futility", p4),
            ("tight_reply", p5),
            ("more_extensions", p6),
        ];

        let mut sensitive_to: Vec<&str> = Vec::new();
        for (name, pconfig) in &perturbations {
            let p_inputs = MonsGameModel::smart_search_best_inputs(&game, *pconfig);
            let p_fen = Input::fen_from_array(&p_inputs);
            if p_fen != base_fen {
                sensitive_to.push(name);
            }
        }

        if !sensitive_to.is_empty() {
            found += 1;
            let ranked = MonsGameModel::ranked_root_moves(&game, game.active_color, base_config);
            let h_gap = if ranked.len() >= 2 {
                ranked[0].heuristic.saturating_sub(ranked[1].heuristic)
            } else {
                0
            };
            let base_rank = ranked
                .iter()
                .position(|r| Input::fen_from_array(&r.inputs) == base_fen)
                .unwrap_or(usize::MAX);
            println!(
                "SENSITIVE pos={} seed={} h_gap={} base_rank={} root_count={} sensitive_to={:?} game_fen={}",
                pos_idx, seed, h_gap, base_rank, ranked.len(), sensitive_to, game.fen()
            );
        }
    }
    println!("\nTotal sensitive positions: {}/{}", found, positions);
}

#[test]
#[ignore = "diagnostic: find Normal positions sensitive to config perturbations"]
fn smart_automove_normal_config_sensitivity_probe() {
    use rand::prelude::*;
    let positions = env_usize("SMART_PROBE_POSITIONS").unwrap_or(300);
    let plies_per_position = env_usize("SMART_PROBE_PLIES").unwrap_or(20);
    let seed = env_usize("SMART_PROBE_SEED").unwrap_or(42) as u64;
    let max_h_gap = env_usize("SMART_PROBE_MAX_GAP").unwrap_or(50) as i32;
    let openings = generate_opening_fens_cached(seed, positions);
    let normal_budget = SearchBudget::from_preference(SmartAutomovePreference::Normal);

    let mut found = 0;
    let mut close_and_sensitive = 0;

    for (pos_idx, fen) in openings.iter().enumerate() {
        let base_game = MonsGame::from_fen(fen.as_str(), false);
        let Some(base_game) = base_game else { continue };
        let mut game = base_game.clone_for_simulation();
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(pos_idx as u64));
        for _ in 0..plies_per_position {
            if !apply_seeded_random_move(&mut game, &mut rng) {
                break;
            }
        }
        if game.winner_color().is_some() {
            continue;
        }

        let base_config = normal_budget.runtime_config_for_game(&game);
        let ranked = MonsGameModel::ranked_root_moves(&game, game.active_color, base_config);
        if ranked.len() < 2 {
            continue;
        }
        let h_gap = ranked[0].heuristic.saturating_sub(ranked[1].heuristic);
        if h_gap > max_h_gap {
            continue;
        }

        let base_inputs = MonsGameModel::smart_search_best_inputs(&game, base_config);
        let base_fen = Input::fen_from_array(&base_inputs);

        // Perturbation 1: tighter efficiency margin
        let mut p1 = base_config;
        p1.root_efficiency_score_margin = 1_100;

        // Perturbation 2: wider efficiency margin
        let mut p2 = base_config;
        p2.root_efficiency_score_margin = 1_800;

        // Perturbation 3: no selective extensions
        let mut p3 = base_config;
        p3.enable_selective_extensions = false;

        // Perturbation 4: wider reply-risk
        let mut p4 = base_config;
        p4.root_reply_risk_shortlist_max += 4;
        p4.root_reply_risk_reply_limit += 8;

        // Perturbation 5: tighter reply-risk
        let mut p5 = base_config;
        p5.root_reply_risk_shortlist_max = base_config.root_reply_risk_shortlist_max.max(3) - 3;
        p5.root_reply_risk_reply_limit = base_config.root_reply_risk_reply_limit.max(6) - 6;

        // Perturbation 6: disable safety rerank
        let mut p6 = base_config;
        p6.enable_normal_root_safety_rerank = false;
        p6.enable_normal_root_safety_deep_floor = false;

        // Perturbation 7: stronger backtrack penalty
        let mut p7 = base_config;
        p7.root_backtrack_penalty = 400;

        // Perturbation 8: boosted interview-soft priors
        let mut p8 = base_config;
        p8.interview_soft_supermana_progress_bonus = 400;
        p8.interview_soft_supermana_score_bonus = 550;
        p8.interview_soft_opponent_mana_progress_bonus = 350;
        p8.interview_soft_opponent_mana_score_bonus = 400;

        // Perturbation 9: enable quiet reductions (off in Normal, on in Pro)
        let mut p9 = base_config;
        p9.enable_quiet_reductions = true;

        // Perturbation 10: more nodes (+25%)
        let mut p10 = base_config;
        p10.max_visited_nodes = (base_config.max_visited_nodes * 125) / 100;

        let perturbations = [
            ("tight_efficiency", p1),
            ("wide_efficiency", p2),
            ("no_extensions", p3),
            ("wider_reply", p4),
            ("tight_reply", p5),
            ("no_safety_rerank", p6),
            ("strong_backtrack", p7),
            ("boost_interview", p8),
            ("quiet_reductions", p9),
            ("more_nodes", p10),
        ];

        let mut sensitive_to: Vec<&str> = Vec::new();
        for (name, pconfig) in &perturbations {
            let p_inputs = MonsGameModel::smart_search_best_inputs(&game, *pconfig);
            let p_fen = Input::fen_from_array(&p_inputs);
            if p_fen != base_fen {
                sensitive_to.push(name);
            }
        }

        if !sensitive_to.is_empty() {
            found += 1;
            let base_rank = ranked
                .iter()
                .position(|r| Input::fen_from_array(&r.inputs) == base_fen)
                .unwrap_or(usize::MAX);
            if h_gap == 0 {
                close_and_sensitive += 1;
            }
            println!(
                "SENSITIVE pos={} seed={} h_gap={} base_rank={} root_count={} sensitive_to={:?} game_fen={}",
                pos_idx, seed, h_gap, base_rank, ranked.len(), sensitive_to, game.fen()
            );
        }
    }
    println!(
        "\nTotal sensitive: {}/{}  close_and_sensitive(gap=0): {}",
        found, positions, close_and_sensitive
    );
}

#[test]
#[ignore = "diagnostic: find Fast close-decision positions sensitive to config changes"]
fn smart_automove_fast_config_sensitivity_probe() {
    use rand::prelude::*;
    let positions = env_usize("SMART_PROBE_POSITIONS").unwrap_or(300);
    let plies_per_position = env_usize("SMART_PROBE_PLIES").unwrap_or(20);
    let seed = env_usize("SMART_PROBE_SEED").unwrap_or(42) as u64;
    let max_h_gap = env_usize("SMART_PROBE_MAX_GAP").unwrap_or(50) as i32;
    let openings = generate_opening_fens_cached(seed, positions);
    let fast_budget = SearchBudget::from_preference(SmartAutomovePreference::Fast);

    let mut found = 0;
    let mut close_and_sensitive = 0;

    for (pos_idx, fen) in openings.iter().enumerate() {
        let base_game = MonsGame::from_fen(fen.as_str(), false);
        let Some(base_game) = base_game else { continue };
        let mut game = base_game.clone_for_simulation();
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(pos_idx as u64));
        for _ in 0..plies_per_position {
            if !apply_seeded_random_move(&mut game, &mut rng) {
                break;
            }
        }
        if game.winner_color().is_some() {
            continue;
        }

        let base_config = fast_budget.runtime_config_for_game(&game);
        let ranked = MonsGameModel::ranked_root_moves(&game, game.active_color, base_config);
        if ranked.len() < 2 {
            continue;
        }
        let h_gap = ranked[0].heuristic.saturating_sub(ranked[1].heuristic);
        if h_gap > max_h_gap {
            continue;
        }

        let base_inputs = MonsGameModel::smart_search_best_inputs(&game, base_config);
        let base_fen = Input::fen_from_array(&base_inputs);

        // Perturbation 1: tighter efficiency margin
        let mut p1 = base_config;
        p1.root_efficiency_score_margin = 1_300;

        // Perturbation 2: wider efficiency margin
        let mut p2 = base_config;
        p2.root_efficiency_score_margin = 2_100;

        // Perturbation 3: disable event ordering bonus (ON in Fast, OFF in Normal)
        let mut p3 = base_config;
        p3.enable_event_ordering_bonus = false;

        // Perturbation 4: enable two-pass root allocation (OFF in Fast, ON in Normal)
        let mut p4 = base_config;
        p4.enable_two_pass_root_allocation = true;
        p4.enable_two_pass_volatility_focus = true;

        // Perturbation 5: enable safety rerank (OFF in Fast, ON in Normal)
        let mut p5 = base_config;
        p5.enable_normal_root_safety_rerank = true;
        p5.enable_normal_root_safety_deep_floor = true;

        // Perturbation 6: enable interview hard spirit deploy (OFF in Fast, ON in Normal)
        let mut p6 = base_config;
        p6.enable_interview_hard_spirit_deploy = true;

        // Perturbation 7: disable quiet reductions (ON in Fast, OFF in Normal)
        let mut p7 = base_config;
        p7.enable_quiet_reductions = false;

        // Perturbation 8: stronger backtrack penalty
        let mut p8 = base_config;
        p8.root_backtrack_penalty = 350;

        // Perturbation 9: boosted interview-soft priors
        let mut p9 = base_config;
        p9.interview_soft_supermana_progress_bonus = 500;
        p9.interview_soft_supermana_score_bonus = 800;
        p9.interview_soft_opponent_mana_progress_bonus = 350;
        p9.interview_soft_opponent_mana_score_bonus = 450;

        // Perturbation 10: more nodes (+30%)
        let mut p10 = base_config;
        p10.max_visited_nodes = (base_config.max_visited_nodes * 130) / 100;

        // Perturbation 11: prefer clean reply-risk roots (OFF in Fast, ON in Normal)
        let mut p11 = base_config;
        p11.prefer_clean_reply_risk_roots = true;

        // Perturbation 12: wider reply-risk (closer to Normal)
        let mut p12 = base_config;
        p12.root_reply_risk_shortlist_max = 7;
        p12.root_reply_risk_reply_limit = 16;
        p12.root_reply_risk_node_share_bp = 1_350;

        let perturbations = [
            ("tight_efficiency", p1),
            ("wide_efficiency", p2),
            ("no_event_ordering", p3),
            ("two_pass", p4),
            ("safety_rerank", p5),
            ("spirit_deploy", p6),
            ("no_quiet_reductions", p7),
            ("strong_backtrack", p8),
            ("boost_interview", p9),
            ("more_nodes", p10),
            ("clean_reply_pref", p11),
            ("wider_reply_risk", p12),
        ];

        let mut sensitive_to: Vec<&str> = Vec::new();
        for (name, pconfig) in &perturbations {
            let p_inputs = MonsGameModel::smart_search_best_inputs(&game, *pconfig);
            let p_fen = Input::fen_from_array(&p_inputs);
            if p_fen != base_fen {
                sensitive_to.push(name);
            }
        }

        if !sensitive_to.is_empty() {
            found += 1;
            let base_rank = ranked
                .iter()
                .position(|r| Input::fen_from_array(&r.inputs) == base_fen)
                .unwrap_or(usize::MAX);
            if h_gap == 0 {
                close_and_sensitive += 1;
            }
            println!(
                "SENSITIVE pos={} seed={} h_gap={} base_rank={} root_count={} sensitive_to={:?} game_fen={}",
                pos_idx, seed, h_gap, base_rank, ranked.len(), sensitive_to, game.fen()
            );
        }
    }
    println!(
        "\nTotal sensitive: {}/{}  close_and_sensitive(gap=0): {}",
        found, positions, close_and_sensitive
    );
}

#[test]
#[ignore = "diagnostic: find Normal positions where depth-3 disagrees with depth-4"]
fn smart_automove_normal_depth_disagreement_probe() {
    use rand::prelude::*;
    let positions = env_usize("SMART_PROBE_POSITIONS").unwrap_or(200);
    let plies_per_position = env_usize("SMART_PROBE_PLIES").unwrap_or(20);
    let seed = env_usize("SMART_PROBE_SEED").unwrap_or(42) as u64;
    let max_normal_gap = env_usize("SMART_PROBE_MAX_GAP").unwrap_or(100) as i32;
    let openings = generate_opening_fens_cached(seed, positions);
    let normal_budget = SearchBudget::from_preference(SmartAutomovePreference::Normal);
    let pro_budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);

    let mut disagreements = 0;
    let mut close_disagreements = 0;

    for (pos_idx, fen) in openings.iter().enumerate() {
        let base_game = MonsGame::from_fen(fen.as_str(), false);
        let Some(base_game) = base_game else { continue };
        let mut game = base_game.clone_for_simulation();
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(pos_idx as u64));
        for _ in 0..plies_per_position {
            if !apply_seeded_random_move(&mut game, &mut rng) {
                break;
            }
        }
        if game.winner_color().is_some() {
            continue;
        }

        let normal_config = normal_budget.runtime_config_for_game(&game);
        let pro_config = pro_budget.runtime_config_for_game(&game);

        let normal_best = MonsGameModel::smart_search_best_inputs(&game, normal_config);
        let normal_fen = Input::fen_from_array(&normal_best);

        let pro_best = MonsGameModel::smart_search_best_inputs(&game, pro_config);
        let pro_fen = Input::fen_from_array(&pro_best);

        if normal_fen == pro_fen {
            continue;
        }

        disagreements += 1;
        let ranked = MonsGameModel::ranked_root_moves(&game, game.active_color, normal_config);
        let normal_gap = if ranked.len() >= 2 {
            ranked[0].heuristic.saturating_sub(ranked[1].heuristic)
        } else {
            0
        };

        if normal_gap <= max_normal_gap {
            close_disagreements += 1;
            let normal_rank = ranked
                .iter()
                .position(|r| Input::fen_from_array(&r.inputs) == normal_fen)
                .unwrap_or(usize::MAX);
            let pro_in_normal_rank = ranked
                .iter()
                .position(|r| Input::fen_from_array(&r.inputs) == pro_fen)
                .unwrap_or(usize::MAX);
            println!(
                "DISAGREE pos={} seed={} normal_gap={} normal_rank={} pro_rank_in_normal={} roots={} normal_move={} pro_move={} fen={}",
                pos_idx, seed, normal_gap, normal_rank, pro_in_normal_rank, ranked.len(), normal_fen, pro_fen, game.fen()
            );
        }
    }
    println!(
        "\nTotal disagreements: {}/{}  close_disagreements(gap<={}): {}",
        disagreements, positions, max_normal_gap, close_disagreements
    );
}

#[test]
#[ignore = "diagnostic: find same-position disagreements between two profiles in one mode"]
fn smart_automove_profile_mode_disagreement_probe() {
    use rand::prelude::*;

    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_current".into());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_release_safe_pre_exact".into());
    let mode = compare_focus_mode_from_env("SMART_PROBE_MODE", SmartAutomovePreference::Normal);
    let positions = env_usize("SMART_PROBE_POSITIONS").unwrap_or(200).max(1);
    let plies_per_position = env_usize("SMART_PROBE_PLIES").unwrap_or(20);
    let seed = env_usize("SMART_PROBE_SEED").unwrap_or(42) as u64;
    let limit = env_usize("SMART_PROBE_LIMIT").unwrap_or(8).max(1);
    let openings = generate_opening_fens_cached(seed, positions);

    let mut disagreements = 0usize;
    let mut candidate_supermana_edges = 0usize;
    let mut candidate_opponent_mana_edges = 0usize;

    println!(
        "profile disagreement probe: candidate={} baseline={} mode={} positions={} plies={} seed={} limit={}",
        candidate_profile,
        baseline_profile,
        mode.as_api_value(),
        positions,
        plies_per_position,
        seed,
        limit
    );

    for (pos_idx, fen) in openings.iter().enumerate() {
        let base_game = MonsGame::from_fen(fen.as_str(), false);
        let Some(base_game) = base_game else { continue };
        let mut game = base_game.clone_for_simulation();
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(pos_idx as u64));
        for _ in 0..plies_per_position {
            if !apply_seeded_random_move(&mut game, &mut rng) {
                break;
            }
        }
        if game.winner_color().is_some() {
            continue;
        }

        let candidate = loss_probe_decision(candidate_profile.as_str(), mode, &game);
        let baseline = loss_probe_decision(baseline_profile.as_str(), mode, &game);
        if candidate.move_fen == baseline.move_fen {
            continue;
        }

        disagreements += 1;
        if let (Some(candidate_root), Some(baseline_root)) = (
            candidate.selected_root.as_ref(),
            baseline.selected_root.as_ref(),
        ) {
            if candidate_root.safe_supermana_progress_steps
                < baseline_root.safe_supermana_progress_steps
            {
                candidate_supermana_edges += 1;
            }
            if candidate_root.safe_opponent_mana_progress_steps
                < baseline_root.safe_opponent_mana_progress_steps
            {
                candidate_opponent_mana_edges += 1;
            }
        }

        println!(
            "DISAGREE pos={} mode={} candidate_move={} baseline_move={} fen={}",
            pos_idx,
            mode.as_api_value(),
            candidate.move_fen,
            baseline.move_fen,
            game.fen()
        );
        print_loss_probe_decision("  candidate", &candidate);
        print_loss_probe_decision("  baseline", &baseline);

        if disagreements >= limit {
            break;
        }
    }

    println!(
        "\nTotal disagreements: {}  candidate_supermana_edges={}  candidate_opponent_mana_edges={}",
        disagreements, candidate_supermana_edges, candidate_opponent_mana_edges
    );
}

#[test]
#[ignore = "diagnostic: find close-decision positions in human-win games"]
fn smart_automove_human_win_close_decision_probe() {
    let max_gap = env_usize("SMART_PROBE_MAX_GAP").unwrap_or(50) as i32;
    let pro_budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);

    let games_data: Vec<(&str, &[&str])> = vec![
        (
            "white",
            &[
                "l10,6;l9,7",
                "l9,7;l8,6",
                "l8,6;l7,5",
                "l10,4;l9,4",
                "l9,4;l8,5",
                "l0,4;l1,5",
                "l1,5;l0,7;l1,8",
                "l1,8;l2,9",
                "l2,9;l3,10",
                "l3,10;l4,10",
                "l4,10;l5,10;mp",
                "l3,6;l2,7",
                "l7,5;l5,5;l6,4",
                "l10,5;l9,4",
                "l9,4;l8,4",
                "l7,5;l8,6",
                "l10,3;l9,2",
                "l10,7;l9,6",
                "l6,3;l7,3",
                "l1,5;l2,4",
                "l1,5;l2,5",
                "l2,5;l3,5",
                "l3,5;l4,4",
                "l4,4;l6,4;l5,3",
                "l0,5;l1,5",
                "l1,5;l2,5",
                "l3,4;l2,3",
                "l8,6;l8,4;l7,5",
                "l7,5;l7,6",
                "l8,6;l7,5",
                "l9,2;l8,1",
                "l8,1;l7,0",
                "l7,0;l6,1",
                "l6,7;l7,7",
                "l0,3;l1,2",
                "l1,2;l2,1",
                "l2,1;l3,0",
                "l3,0;l4,0",
                "l4,0;l5,0;mb",
                "l5,0;l6,1",
                "l4,4;l2,3;l1,2",
                "l1,2;l0,1",
                "l7,5;l5,3;l6,3",
                "l9,6;l8,7",
                "l8,5;l8,6",
                "l7,6;l7,7",
                "l7,5;l8,4",
                "l8,7;l7,8",
                "l7,3;l8,2",
                "l4,4;l5,3",
                "l5,3;l6,2",
                "l6,2;l8,2;l9,1",
                "l6,2;l7,1",
                "l7,1;l9,1;l10,0",
                "l2,5;l3,4",
                "l3,4;l4,3",
                "l0,1;l0,0",
                "l7,8;l6,7",
                "l6,7;l5,6",
                "l5,6;l4,6",
                "l4,6;l3,5",
                "l3,5;l2,5",
                "l2,5;l4,3",
                "l7,6;l8,7",
                "l7,1;l6,3;l5,2",
                "l0,6;l1,5",
                "l1,5;l2,4",
                "l2,4;l3,3",
                "l3,3;l4,2",
                "l4,2;l5,1",
                "l4,3;l3,2",
                "l8,4;l6,5;l6,4",
                "l8,4;l8,5",
                "l2,5;l3,4",
                "l3,4;l4,3",
                "l7,7;l8,7",
                "l4,3;l3,3",
                "l7,4;l8,4",
                "l7,1;l5,2;l4,1",
                "l0,5;l1,4",
                "l1,4;l2,3",
                "l2,3;l3,2",
                "l3,2;l4,1",
                "l5,1;l4,0",
                "l3,2;l2,2",
                "l8,5;l7,7;l8,8",
                "l3,3;l4,2",
                "l8,6;l7,7",
                "l8,7;l8,8",
                "l8,8;l9,9",
                "l9,9;l10,10",
                "l8,7;l8,8",
                "l4,1;l3,0",
                "l3,0;l2,0",
                "l2,0;l1,0",
                "l1,0;l0,0",
            ],
        ),
        (
            "black",
            &[
                "l10,3;l9,2",
                "l9,2;l8,1",
                "l8,1;l7,0",
                "l7,0;l6,0",
                "l6,0;l5,0;mp",
                "l0,4;l1,5",
                "l1,5;l3,6;l2,7",
                "l0,5;l1,6",
                "l0,3;l1,3",
                "l0,7;l1,7",
                "l1,5;l2,4",
                "l3,4;l2,3",
                "l10,6;l9,5",
                "l9,5;l8,5",
                "l8,5;l7,5",
                "l7,5;l5,5;l6,6",
                "l10,5;l9,5",
                "l9,5;l8,5",
                "l7,6;l8,7",
                "l2,4;l4,3;l3,2",
                "l1,3;l2,2",
                "l0,6;l1,5",
                "l1,6;l2,6",
                "l1,7;l2,8",
                "l2,8;l3,7",
                "l2,3;l1,2",
                "l7,5;l6,4",
                "l6,4;l5,3",
                "l5,3;l4,2",
                "l4,2;l3,1",
                "l3,1;l1,2;l1,1",
                "l3,1;l1,1;l0,0",
                "l10,4;l9,5",
                "l8,7;l9,8",
                "l3,7;l4,8",
                "l4,8;l4,9",
                "l4,9;l5,10;mb",
                "l5,10;l4,9",
                "l4,9;l5,8",
                "l5,8;l8,5",
                "l2,4;l4,5;l3,6",
                "l2,7;l1,8",
                "l3,1;l4,2",
                "l4,2;l5,3",
                "l5,3;l6,4",
                "l6,4;l6,6;l7,7",
                "l6,4;l7,5",
                "l9,5;l8,5",
                "l9,8;l10,9",
                "l2,4;l3,6;l2,6",
                "l1,5;l1,6",
                "l2,6;l1,7",
                "l1,7;l0,8",
                "l0,8;l0,9",
                "l0,9;l0,10",
                "l1,8;l0,9",
                "l7,5;l7,7;l8,7",
                "l10,5;l9,6",
                "l9,6;l8,7",
                "l8,7;l8,8",
                "l8,8;l9,9",
                "l9,9;l10,10",
                "l10,9;l10,10",
            ],
        ),
    ];

    let mut total_close = 0;
    for (game_idx, (human_color, moves)) in games_data.iter().enumerate() {
        let bot_color = if *human_color == "white" {
            Color::Black
        } else {
            Color::White
        };
        let mut game = MonsGame::new(false);
        for (move_idx, move_fen) in moves.iter().enumerate() {
            if game.winner_color().is_some() {
                break;
            }
            let is_bot_turn = game.active_color == bot_color;
            if is_bot_turn {
                let pro_config = pro_budget.runtime_config_for_game(&game);
                let ranked = MonsGameModel::ranked_root_moves(&game, game.active_color, pro_config);
                if ranked.len() >= 2 {
                    let gap = ranked[0].heuristic.saturating_sub(ranked[1].heuristic);
                    if gap <= max_gap {
                        total_close += 1;
                        let selected = Input::fen_from_array(&ranked[0].inputs);
                        println!(
                            "CLOSE game={} move={} gap={} roots={} selected={} fen={}",
                            game_idx + 1,
                            move_idx,
                            gap,
                            ranked.len(),
                            selected,
                            game.fen()
                        );
                    }
                }
            }
            let inputs = Input::array_from_fen(move_fen);
            let _ = game.process_input(inputs, false, false);
        }
    }
    println!(
        "\nTotal close-decision bot positions (gap<={}): {}",
        max_gap, total_close
    );
}
