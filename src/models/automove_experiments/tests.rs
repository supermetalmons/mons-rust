use super::harness::*;
use super::profiles::*;
use super::*;
use crate::models::automove_exact::{
    clear_exact_query_diagnostics, clear_exact_state_analysis_cache,
    exact_query_diagnostics_snapshot,
};

fn stage1_cpu_budgets() -> Vec<SearchBudget> {
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
            "SMART_TRIAGE_SURFACE is required (expected one of: opening_reply, primary_pro, reply_risk, supermana, opponent_mana, spirit_setup, drainer_safety, cache_reuse)"
        )
    });
    TriageSurface::parse(value.as_str()).unwrap_or_else(|| {
        panic!(
            "unknown SMART_TRIAGE_SURFACE='{}' (expected one of: opening_reply, primary_pro, reply_risk, supermana, opponent_mana, spirit_setup, drainer_safety, cache_reuse)",
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
        items.into_iter()
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
    profile_runtime_config_for_name(profile_name, game, base)
        .unwrap_or_else(|| panic!("profile '{}' does not expose a runtime config", profile_name))
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
    let config =
        calibration_runtime_config(profile_name, &game, SmartAutomovePreference::Fast);
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

    let config =
        calibration_runtime_config(profile_name, &game, SmartAutomovePreference::Normal);
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
    let config =
        calibration_runtime_config(profile_name, &game, SmartAutomovePreference::Normal);
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
    let candidate = reply_risk_calibration_probe(
        "runtime_pre_fast_root_quality_v1_normal_conversion_v3",
    );
    let baseline = reply_risk_calibration_probe("runtime_release_safe_pre_exact");
    assert!(candidate > baseline);
}

#[test]
fn triage_calibration_probe_detects_opponent_mana_profile_delta() {
    let candidate = opponent_mana_calibration_probe(
        "runtime_pre_fast_root_quality_v1_normal_conversion_v3",
    );
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
    let shortlisted_roots = ranked_roots
        .iter()
        .take(shortlist_len)
        .collect::<Vec<_>>();
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
                    SearchBudget::from_preference(fixture.mode).runtime_config_for_game(
                        &fixture.game,
                    )
                });
            let resolved_config =
                profile_runtime_config_for_name(profile_name, &fixture.game, base_config)
                    .unwrap_or(base_config);
            let inputs =
                select_inputs_with_runtime_fallback(selector, &fixture.game, base_config);
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
                .position(|root| Input::fen_from_array(&root.inputs) == input_fen)
                .unwrap_or_else(|| {
                    panic!(
                        "triage fixture '{}' selected move missing from ranked roots in mode {}",
                        fixture.id,
                        fixture.mode.as_api_value()
                    )
                });
            TriageSignalSnapshot {
                selected_rank,
                selected_root: triage_root_digest_entry(&ranked_roots[selected_rank]),
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
                || candidate.safe_supermana_progress_steps
                    != baseline.safe_supermana_progress_steps
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
                || candidate.safe_opponent_mana_pickup_now
                    != baseline.safe_opponent_mana_pickup_now
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
        TriageSurface::OpeningReply | TriageSurface::PrimaryPro => false,
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
        TriageSurface::OpeningReply | TriageSurface::PrimaryPro => {
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
        let fixture_changed =
            triage_surface_signal_changed(surface, &candidate_snapshot, &baseline_snapshot);
        if fixture_changed {
            changed += 1;
        }
        println!(
            "triage surface={} fixture={} mode={} opening_book={} changed={} candidate_profile={} candidate={:?} baseline_profile={} baseline={:?}",
            surface.as_str(),
            fixture.id,
            fixture.mode.as_api_value(),
            fixture.opening_book_driven,
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
    let baseline_selector = profile_selector_from_name("runtime_current")
        .expect("runtime_current selector should exist for stage-1 cpu gate");
    let budgets = stage1_cpu_budgets();
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
    let openings = generate_opening_fens_cached(seed_for_pairing("triage_cache_reuse", "fixed"), positions);
    let speed_stats = profile_speed_by_mode_ms(selector, openings.as_slice(), budgets.as_slice());
    let avg_ms = if speed_stats.is_empty() {
        0.0
    } else {
        speed_stats.iter().map(|stat| stat.avg_ms).sum::<f64>() / speed_stats.len() as f64
    };

    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    let repeats = env_usize("SMART_TRIAGE_CACHE_REPEATS")
        .unwrap_or(2)
        .max(1);
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
    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();

    CacheReuseProbe {
        avg_ms,
        calls,
        hits,
        hit_rate: if calls == 0 {
            0.0
        } else {
            hits as f64 / calls as f64
        },
    }
}

fn cache_reuse_triage_passes(candidate: CacheReuseProbe, baseline: CacheReuseProbe) -> bool {
    let faster = candidate.avg_ms <= baseline.avg_ms * 0.97;
    let better_cache = candidate.calls > 0
        && baseline.calls > 0
        && candidate.hit_rate >= baseline.hit_rate + 0.05;
    faster || better_cache
}

fn assert_exact_lite_diagnostics_gate_if_enabled(
    candidate_profile_name: &str,
    candidate_selector: AutomoveSelector,
) {
    let budgets = stage1_cpu_budgets();
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
        || assert_exact_lite_diagnostics_gate_if_enabled(candidate_profile_name, candidate_selector),
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
            "runtime_eff_non_exact_v1",
            "runtime_eff_non_exact_v2",
            "runtime_eff_exact_lite_v1",
            "swift_2024_eval_reference",
            "swift_2024_style_reference",
            "runtime_pre_fast_root_quality_v1_normal_conversion_v3",
            "runtime_attacker_proximity_v1",
            "runtime_normal_history_heuristic_v1",
            "runtime_pro_history_heuristic_v1",
            "runtime_normal_quiescence_v1",
            "runtime_pro_quiescence_v1",
            "runtime_pro_quiescence_v2",
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
#[ignore = "strict stage-1 cpu non-regression gate against runtime_current"]
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

    assert_tactical_guardrails(CANDIDATE_MODEL.select_inputs, candidate_profile_name.as_str());
    assert_interview_policy_regressions(CANDIDATE_MODEL.select_inputs, candidate_profile_name.as_str());

    match surface {
        TriageSurface::OpeningReply | TriageSurface::PrimaryPro => {
            panic!(
                "surface '{}' requires pro-triage; use SMART_TRIAGE_SURFACE=opening_reply|primary_pro with ./scripts/run-automove-experiment.sh pro-triage",
                surface.as_str()
            );
        }
        TriageSurface::CacheReuse => {
            let candidate_probe =
                cache_reuse_triage_probe(candidate_profile_name.as_str(), CANDIDATE_MODEL.select_inputs);
            let baseline_probe =
                cache_reuse_triage_probe(baseline_profile_name.as_str(), baseline_selector);
            println!(
                "triage surface=cache_reuse candidate={} avg_ms={:.2} hit_rate={:.3} hits={} calls={} baseline={} avg_ms={:.2} hit_rate={:.3} hits={} calls={}",
                candidate_profile_name,
                candidate_probe.avg_ms,
                candidate_probe.hit_rate,
                candidate_probe.hits,
                candidate_probe.calls,
                baseline_profile_name,
                baseline_probe.avg_ms,
                baseline_probe.hit_rate,
                baseline_probe.hits,
                baseline_probe.calls
            );
            assert!(
                cache_reuse_triage_passes(candidate_probe, baseline_probe),
                "cache_reuse triage found no deterministic evidence change: candidate avg_ms={:.2} hit_rate={:.3}, baseline avg_ms={:.2} hit_rate={:.3}",
                candidate_probe.avg_ms,
                candidate_probe.hit_rate,
                baseline_probe.avg_ms,
                baseline_probe.hit_rate
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
                candidate_profile_name,
                candidate_pick,
                baseline_profile_name,
                baseline_pick
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
        "runtime_eff_non_exact_v2",
        "runtime_attacker_proximity_v1",
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
        let ranked = MonsGameModel::ranked_root_moves(
            &game,
            game.active_color,
            pro_config,
        );
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
            max_plies: 72,
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
            "fast" => Some(vec![SearchBudget::from_preference(SmartAutomovePreference::Fast)]),
            "normal" => Some(vec![SearchBudget::from_preference(SmartAutomovePreference::Normal)]),
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
        .unwrap_or(72)
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
        .unwrap_or(72)
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
                "l10,6;l9,7", "l9,7;l8,6", "l8,6;l7,5", "l10,4;l9,4", "l9,4;l8,5",
                "l0,4;l1,5", "l1,5;l0,7;l1,8", "l1,8;l2,9", "l2,9;l3,10", "l3,10;l4,10",
                "l4,10;l5,10;mp", "l3,6;l2,7", "l7,5;l5,5;l6,4", "l10,5;l9,4", "l9,4;l8,4",
                "l7,5;l8,6", "l10,3;l9,2", "l10,7;l9,6", "l6,3;l7,3", "l1,5;l2,4",
                "l1,5;l2,5", "l2,5;l3,5", "l3,5;l4,4", "l4,4;l6,4;l5,3", "l0,5;l1,5",
                "l1,5;l2,5", "l3,4;l2,3", "l8,6;l8,4;l7,5", "l7,5;l7,6", "l8,6;l7,5",
                "l9,2;l8,1", "l8,1;l7,0", "l7,0;l6,1", "l6,7;l7,7", "l0,3;l1,2",
                "l1,2;l2,1", "l2,1;l3,0", "l3,0;l4,0", "l4,0;l5,0;mb", "l5,0;l6,1",
                "l4,4;l2,3;l1,2", "l1,2;l0,1", "l7,5;l5,3;l6,3", "l9,6;l8,7",
                "l8,5;l8,6", "l7,6;l7,7", "l7,5;l8,4", "l8,7;l7,8", "l7,3;l8,2",
                "l4,4;l5,3", "l5,3;l6,2", "l6,2;l8,2;l9,1", "l6,2;l7,1",
                "l7,1;l9,1;l10,0", "l2,5;l3,4", "l3,4;l4,3", "l0,1;l0,0", "l7,8;l6,7",
                "l6,7;l5,6", "l5,6;l4,6", "l4,6;l3,5", "l3,5;l2,5", "l2,5;l4,3",
                "l7,6;l8,7", "l7,1;l6,3;l5,2", "l0,6;l1,5", "l1,5;l2,4", "l2,4;l3,3",
                "l3,3;l4,2", "l4,2;l5,1", "l4,3;l3,2", "l8,4;l6,5;l6,4", "l8,4;l8,5",
                "l2,5;l3,4", "l2,5;l3,4", "l3,4;l4,3", "l7,7;l8,7", "l4,3;l3,3",
                "l7,4;l8,4", "l7,1;l5,2;l4,1", "l0,5;l1,4", "l1,4;l2,3", "l2,3;l3,2",
                "l3,2;l4,1", "l5,1;l4,0", "l3,2;l2,2", "l8,5;l7,7;l8,8", "l3,3;l4,2",
                "l8,6;l7,7", "l8,7;l8,8", "l8,8;l9,9", "l9,9;l10,10", "l8,7;l8,8",
                "l4,1;l3,0", "l3,0;l2,0", "l2,0;l1,0", "l1,0;l0,0",
            ],
        },
        GameSpec {
            label: "game2_human_black",
            bot_color: Color::White,
            moves: vec![
                "l10,3;l9,2", "l9,2;l8,1", "l8,1;l7,0", "l7,0;l6,0", "l6,0;l5,0;mp",
                "l0,4;l1,5", "l1,5;l3,6;l2,7", "l0,5;l1,6", "l0,3;l1,3", "l0,7;l1,7",
                "l1,5;l2,4", "l3,4;l2,3", "l10,6;l9,5", "l9,5;l8,5", "l8,5;l7,5",
                "l7,5;l5,5;l6,6", "l10,5;l9,5", "l9,5;l8,5", "l7,6;l8,7",
                "l2,4;l4,3;l3,2", "l1,3;l2,2", "l0,6;l1,5", "l1,6;l2,6", "l1,7;l2,8",
                "l2,8;l3,7", "l2,3;l1,2", "l7,5;l6,4", "l6,4;l5,3", "l5,3;l4,2",
                "l4,2;l3,1", "l3,1;l1,2;l1,1", "l3,1;l1,1;l0,0", "l10,4;l9,5",
                "l8,7;l9,8", "l3,7;l4,8", "l4,8;l4,9", "l4,9;l5,10;mb", "l5,10;l4,9",
                "l4,9;l5,8", "l5,8;l8,5", "l2,4;l4,5;l3,6", "l2,7;l1,8", "l3,1;l4,2",
                "l4,2;l5,3", "l5,3;l6,4", "l6,4;l6,6;l7,7", "l6,4;l7,5", "l9,5;l8,5",
                "l9,8;l10,9", "l2,4;l3,6;l2,6", "l1,5;l1,6", "l2,6;l1,7", "l1,7;l0,8",
                "l0,8;l0,9", "l0,9;l0,10", "l1,8;l0,9", "l7,5;l7,7;l8,7",
                "l10,5;l9,6", "l9,6;l8,7", "l8,7;l8,8", "l8,8;l9,9", "l9,9;l10,10",
                "l10,9;l10,10",
            ],
        },
        GameSpec {
            label: "game3_human_white",
            bot_color: Color::Black,
            moves: vec![
                "l10,5;l9,5", "l9,5;l8,5", "l10,6;l9,6", "l9,6;l8,6", "l10,4;l9,5",
                "l0,4;l1,5", "l1,5;l0,7;l1,8", "l1,8;l2,9", "l2,9;l3,10", "l3,10;l4,10",
                "l4,10;l5,10;mp", "l3,6;l2,7", "l8,6;l7,4;l8,5", "l8,5;l8,4", "l8,4;l9,3",
                "l9,3;l9,2", "l9,2;l9,1", "l9,1;l10,0", "l6,3;l6,4", "l1,5;l2,5",
                "l2,5;l3,5", "l3,5;l5,5;l4,4", "l0,5;l1,5", "l1,5;l2,5", "l0,6;l1,5",
                "l3,4;l2,3", "l8,6;l6,7;l7,8", "l10,3;l9,2", "l10,7;l9,8", "l9,2;l8,1",
                "l8,1;l7,0", "l7,0;l6,0", "l7,8;l8,8", "l0,3;l1,2", "l1,2;l2,2",
                "l2,2;l3,2", "l3,2;l4,2", "l4,2;l6,0", "l1,5;l2,4", "l2,3;l1,2",
                "l8,6;l6,5;l6,6", "l9,5;l8,4", "l8,4;l8,3", "l8,3;l8,2", "l8,2;l9,1",
                "l9,1;l9,0", "l7,6;l7,7", "l3,5;l4,6", "l4,6;l5,7", "l5,7;l6,8",
                "l6,8;l8,8;l9,9", "l6,8;l7,9", "l7,9;l9,9;l10,10", "l2,5;l3,4",
                "l4,2;l5,3", "l1,2;l0,1", "l10,3;l9,4", "l8,6;l9,8;l10,8", "l9,4;l8,4",
                "l8,4;l7,4", "l7,4;l6,5", "l9,0;l9,1", "l7,7;l8,8", "l5,3;l5,2",
                "l5,2;l5,1", "l5,1;l5,0;mp", "l7,9;l7,10", "l7,10;l8,8;l9,9",
                "l7,10;l9,9;l10,10", "l0,1;l0,0",
            ],
        },
        GameSpec {
            label: "game4_human_black",
            bot_color: Color::White,
            moves: vec![
                "l10,4;l9,5", "l9,5;l8,5", "l8,5;l7,5", "l7,5;l6,4", "l6,4;l5,4",
                "l0,4;l1,5", "l1,5;l3,6;l2,7", "l0,5;l1,6", "l0,3;l1,3", "l0,7;l1,7",
                "l1,5;l2,4", "l4,3;l3,2", "l10,5;l9,5", "l9,5;l8,5", "l10,6;l9,7",
                "l9,7;l8,5;l7,5", "l7,5;l6,5", "l6,5;l5,5", "l7,6;l8,7",
                "l2,4;l1,6;l2,5", "l0,6;l1,6", "l2,4;l3,3", "l3,3;l4,3", "l1,3;l2,2",
                "l2,2;l3,1", "l2,7;l1,8", "l5,5;l6,6", "l6,6;l7,7", "l7,7;l8,8",
                "l8,8;l9,9", "l9,7;l9,9;l10,10", "l9,7;l8,8", "l8,7;l9,8",
                "l4,3;l3,1;l4,0", "l4,0;l5,0;mp", "l2,5;l2,4", "l4,3;l6,3;l5,2",
                "l1,6;l2,5", "l1,7;l2,6", "l2,6;l3,5", "l1,8;l0,9", "l8,8;l7,9",
                "l7,9;l9,8;l10,9", "l7,9;l6,10", "l6,10;l5,10;mp", "l5,10;l5,9",
                "l5,9;l5,8", "l10,9;l10,10", "l3,5;l4,6", "l4,6;l5,7", "l5,7;l6,8",
                "l6,8;l7,9", "l7,9;l8,10", "l8,10;l10,10", "l0,9;l0,10", "l10,7;l9,8",
                "l9,8;l8,8", "l8,8;l10,10", "l5,8;l5,7", "l5,7;l5,6", "l10,3;l9,2",
                "l5,2;l6,3", "l4,3;l2,4;l3,4", "l3,4;l2,3", "l2,3;l1,2", "l1,2;l0,1",
                "l0,1;l0,0", "l2,5;l2,4", "l3,2;l2,1", "l5,6;l4,6", "l4,6;l3,5",
                "l3,5;l3,4", "l3,4;l3,3", "l3,3;l2,1;l1,1", "l3,3;l1,1;l0,0",
            ],
        },
        GameSpec {
            label: "game5_human_white",
            bot_color: Color::Black,
            moves: vec![
                "l10,5;l9,5", "l9,5;l8,5", "l10,3;l9,2", "l10,6;l9,6", "l9,6;l8,7",
                "l0,4;l1,5", "l1,5;l2,5", "l2,5;l3,5", "l3,5;l5,5;l4,4", "l0,5;l1,5",
                "l1,5;l2,5", "l3,6;l2,7", "l8,7;l7,8", "l7,8;l6,9", "l6,9;l5,10;mb",
                "l5,10;l4,9", "l4,9;l4,8", "l4,8;l2,5", "l4,8;l2,7;l3,6", "l7,4;l8,5",
                "l3,5;l4,3;l3,2", "l0,3;l1,2", "l1,2;l2,1", "l2,1;l3,0", "l3,0;l4,0",
                "l4,0;l5,0;mp", "l3,2;l2,1", "l4,8;l3,6;l4,6", "l8,5;l9,4", "l9,4;l9,3",
                "l4,8;l5,7", "l5,7;l6,6", "l10,7;l9,6", "l6,3;l7,2", "l0,5;l1,5",
                "l0,5;l1,5", "l3,5;l1,5;l2,5", "l2,5;l3,4", "l3,4;l4,4", "l4,4;l3,3",
                "l3,3;l2,3", "l3,5;l2,3;l1,2", "l3,4;l2,3", "l6,6;l4,7;l5,8",
                "l9,6;l8,5", "l10,4;l9,4", "l8,5;l7,4", "l7,4;l6,4", "l9,2;l8,1",
                "l7,2;l8,2", "l1,2;l0,1", "l0,1;l0,0", "l3,5;l2,3;l1,2", "l0,0;l1,1",
                "l1,1;l2,2", "l2,2;l3,3", "l1,2;l0,1", "l6,4;l5,3", "l5,3;l4,2",
                "l4,2;l5,1", "l5,1;l3,3", "l9,4;l8,4", "l8,1;l7,0", "l8,2;l9,1",
                "l3,5;l4,4", "l4,4;l5,3", "l5,3;l6,2", "l6,2;l7,1", "l7,1;l9,1;l10,0",
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

                    let ranked = MonsGameModel::ranked_root_moves(
                        &game, current_color, config,
                    );
                    let top_h = ranked.first().map(|r| r.heuristic).unwrap_or(0);
                    let search_rank = ranked.iter().position(|r| {
                        Input::fen_from_array(&r.inputs) == search_best_fen
                    });
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
                        search_rank.map(|r| format!("#{}", r + 1)).unwrap_or("?".into()),
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

        println!("\n  Final Score: W={} B={} (bot={:?})", game.white_score, game.black_score, spec.bot_color);
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
        let top_h = ranked.first().map(|r| r.heuristic).unwrap_or(0);
        let search_rank = ranked
            .iter()
            .position(|r| Input::fen_from_array(&r.inputs) == search_best_fen);
        let gap_1_2 = if ranked.len() >= 2 {
            ranked[0].heuristic.saturating_sub(ranked[1].heuristic)
        } else {
            0
        };

        println!("\n--- fixture={} mode={} active={:?} ---", fixture.id, fixture.mode.as_api_value(), game.active_color);
        println!("  search_best={} rank=#{} gap_1v2={} root_count={}",
            search_best_fen,
            search_rank.map(|r| format!("{}", r + 1)).unwrap_or("?".into()),
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
    println!("\nTotal sensitive: {}/{}  close_and_sensitive(gap=0): {}", found, positions, close_and_sensitive);
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
    println!("\nTotal sensitive: {}/{}  close_and_sensitive(gap=0): {}", found, positions, close_and_sensitive);
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
#[ignore = "diagnostic: find close-decision positions in human-win games"]
fn smart_automove_human_win_close_decision_probe() {
    let max_gap = env_usize("SMART_PROBE_MAX_GAP").unwrap_or(50) as i32;
    let pro_budget = SearchBudget::from_preference(SmartAutomovePreference::Pro);

    let games_data: Vec<(&str, &[&str])> = vec![
        (
            "white",
            &[
                "l10,6;l9,7", "l9,7;l8,6", "l8,6;l7,5", "l10,4;l9,4", "l9,4;l8,5",
                "l0,4;l1,5", "l1,5;l0,7;l1,8", "l1,8;l2,9", "l2,9;l3,10", "l3,10;l4,10",
                "l4,10;l5,10;mp", "l3,6;l2,7", "l7,5;l5,5;l6,4", "l10,5;l9,4", "l9,4;l8,4",
                "l7,5;l8,6", "l10,3;l9,2", "l10,7;l9,6", "l6,3;l7,3", "l1,5;l2,4",
                "l1,5;l2,5", "l2,5;l3,5", "l3,5;l4,4", "l4,4;l6,4;l5,3", "l0,5;l1,5",
                "l1,5;l2,5", "l3,4;l2,3", "l8,6;l8,4;l7,5", "l7,5;l7,6", "l8,6;l7,5",
                "l9,2;l8,1", "l8,1;l7,0", "l7,0;l6,1", "l6,7;l7,7", "l0,3;l1,2",
                "l1,2;l2,1", "l2,1;l3,0", "l3,0;l4,0", "l4,0;l5,0;mb", "l5,0;l6,1",
                "l4,4;l2,3;l1,2", "l1,2;l0,1", "l7,5;l5,3;l6,3", "l9,6;l8,7",
                "l8,5;l8,6", "l7,6;l7,7", "l7,5;l8,4", "l8,7;l7,8", "l7,3;l8,2",
                "l4,4;l5,3", "l5,3;l6,2", "l6,2;l8,2;l9,1", "l6,2;l7,1",
                "l7,1;l9,1;l10,0", "l2,5;l3,4", "l3,4;l4,3", "l0,1;l0,0",
                "l7,8;l6,7", "l6,7;l5,6", "l5,6;l4,6", "l4,6;l3,5", "l3,5;l2,5",
                "l2,5;l4,3", "l7,6;l8,7", "l7,1;l6,3;l5,2", "l0,6;l1,5", "l1,5;l2,4",
                "l2,4;l3,3", "l3,3;l4,2", "l4,2;l5,1", "l4,3;l3,2", "l8,4;l6,5;l6,4",
                "l8,4;l8,5", "l2,5;l3,4", "l3,4;l4,3", "l7,7;l8,7", "l4,3;l3,3",
                "l7,4;l8,4", "l7,1;l5,2;l4,1", "l0,5;l1,4", "l1,4;l2,3", "l2,3;l3,2",
                "l3,2;l4,1", "l5,1;l4,0", "l3,2;l2,2", "l8,5;l7,7;l8,8", "l3,3;l4,2",
                "l8,6;l7,7", "l8,7;l8,8", "l8,8;l9,9", "l9,9;l10,10", "l8,7;l8,8",
                "l4,1;l3,0", "l3,0;l2,0", "l2,0;l1,0", "l1,0;l0,0",
            ],
        ),
        (
            "black",
            &[
                "l10,3;l9,2", "l9,2;l8,1", "l8,1;l7,0", "l7,0;l6,0", "l6,0;l5,0;mp",
                "l0,4;l1,5", "l1,5;l3,6;l2,7", "l0,5;l1,6", "l0,3;l1,3", "l0,7;l1,7",
                "l1,5;l2,4", "l3,4;l2,3", "l10,6;l9,5", "l9,5;l8,5", "l8,5;l7,5",
                "l7,5;l5,5;l6,6", "l10,5;l9,5", "l9,5;l8,5", "l7,6;l8,7",
                "l2,4;l4,3;l3,2", "l1,3;l2,2", "l0,6;l1,5", "l1,6;l2,6", "l1,7;l2,8",
                "l2,8;l3,7", "l2,3;l1,2", "l7,5;l6,4", "l6,4;l5,3", "l5,3;l4,2",
                "l4,2;l3,1", "l3,1;l1,2;l1,1", "l3,1;l1,1;l0,0", "l10,4;l9,5",
                "l8,7;l9,8", "l3,7;l4,8", "l4,8;l4,9", "l4,9;l5,10;mb", "l5,10;l4,9",
                "l4,9;l5,8", "l5,8;l8,5", "l2,4;l4,5;l3,6", "l2,7;l1,8",
                "l3,1;l4,2", "l4,2;l5,3", "l5,3;l6,4", "l6,4;l6,6;l7,7",
                "l6,4;l7,5", "l9,5;l8,5", "l9,8;l10,9", "l2,4;l3,6;l2,6",
                "l1,5;l1,6", "l2,6;l1,7", "l1,7;l0,8", "l0,8;l0,9", "l0,9;l0,10",
                "l1,8;l0,9", "l7,5;l7,7;l8,7", "l10,5;l9,6", "l9,6;l8,7",
                "l8,7;l8,8", "l8,8;l9,9", "l9,9;l10,10", "l10,9;l10,10",
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
                let ranked =
                    MonsGameModel::ranked_root_moves(&game, game.active_color, pro_config);
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
    println!("\nTotal close-decision bot positions (gap<={}): {}", max_gap, total_close);
}
