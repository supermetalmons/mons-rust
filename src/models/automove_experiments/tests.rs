use super::harness::*;
use super::profiles::*;
use super::*;
use crate::models::automove_exact::{
    clear_exact_query_diagnostics, clear_exact_state_analysis_cache,
    exact_query_diagnostics_snapshot,
};
use crate::models::automove_turn_engine::{
    clear_turn_engine_diagnostics, clear_turn_engine_plan_cache, turn_engine_cached_step,
    turn_engine_candidate_plan, turn_engine_diagnostics_snapshot, TurnEngineConfig,
};
use crate::models::mons_game_model::{
    clear_turn_engine_selector_diagnostics, turn_engine_selector_diagnostics_snapshot,
};
use std::env;

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

fn env_f64(name: &str) -> Option<f64> {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<f64>().ok())
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

fn calibration_runtime_config(
    profile_name: &str,
    game: &MonsGame,
    mode: SmartAutomovePreference,
) -> SmartSearchConfig {
    let base = SearchBudget::from_preference(mode).runtime_config_for_game(game);
    profile_runtime_config_for_name(profile_name, game, base)
        .unwrap_or_else(|| panic!("profile '{}' does not expose a runtime config", profile_name))
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

fn profile_decision_inputs(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
) -> Vec<Input> {
    let selector = profile_selector_from_name(profile_name)
        .unwrap_or_else(|| panic!("profile '{}' not found", profile_name));
    let config = calibration_runtime_config(profile_name, game, mode);
    clear_turn_engine_selector_diagnostics();
    select_inputs_with_runtime_fallback(selector, game, config)
}

fn profile_decision_move_fen(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
) -> String {
    Input::fen_from_array(&profile_decision_inputs(profile_name, mode, game))
}

fn primary_pro_fixture_by_id(id: &str) -> TriageFixture {
    primary_pro_triage_fixtures()
        .into_iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("primary_pro fixture '{}' not found", id))
}

fn profile_scored_roots(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
) -> (SmartSearchConfig, Vec<RootEvaluation>) {
    let config = calibration_runtime_config(profile_name, game, mode);
    let perspective = game.active_color;
    let root_moves = MonsGameModel::ranked_root_moves(game, perspective, config);
    let (root_moves, scout_visited_nodes) =
        MonsGameModel::focused_root_candidates(game, perspective, root_moves, config, true);
    let mut visited_nodes = scout_visited_nodes;
    let mut alpha = i32::MIN;
    let mut scored_roots = Vec::with_capacity(root_moves.len());
    let mut transposition_table = U64HashMap::default();
    let extension_node_budget =
        if config.enable_selective_extensions && config.selective_extension_node_share_bp > 0 {
            ((config.max_visited_nodes * config.selective_extension_node_share_bp as usize) / 10_000)
                .max(1)
        } else {
            0
        };
    let mut extension_nodes_used = 0usize;
    let mut killer_table: KillerTable = [[0u64; 2]; MAX_SMART_SEARCH_DEPTH + 2];
    let mut history_table: HistoryTable = HistoryTable::default();
    let mut quiescence_nodes_used = 0usize;

    for candidate in root_moves {
        if visited_nodes >= config.max_visited_nodes {
            break;
        }
        visited_nodes += 1;
        let candidate_score = MonsGameModel::evaluate_root_candidate_score(
            &candidate,
            perspective,
            alpha,
            &mut visited_nodes,
            config,
            &mut transposition_table,
            &mut extension_nodes_used,
            extension_node_budget,
            true,
            &mut killer_table,
            &mut history_table,
            &mut quiescence_nodes_used,
        );
        if candidate_score > alpha {
            alpha = candidate_score;
        }
        scored_roots.push(RootEvaluation {
            root_rank: candidate.root_rank,
            score: candidate_score,
            efficiency: candidate.efficiency,
            inputs: candidate.inputs,
            game: candidate.game,
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
        });
    }

    (config, scored_roots)
}

fn format_root_probe(root: Option<&RootEvaluation>) -> String {
    root.map(|root| {
        format!(
            "score={} rank={} family={:?} win={} attack={} window={} same_turn_setup={} own_setup={} spirit={} supermana_progress={} super_steps={} opponent_progress={} opp_steps={} score_path_steps={} setup_gain={} pickup_super={} pickup_opp={} vulnerable={} handoff={} roundtrip={}",
            root.score,
            root.root_rank,
            MonsGameModel::turn_engine_root_evaluation_family(root),
            root.wins_immediately,
            root.attacks_opponent_drainer,
            root.same_turn_score_window_value,
            root.spirit_same_turn_score_setup_now,
            root.spirit_own_mana_setup_now,
            root.spirit_development,
            root.supermana_progress,
            root.safe_supermana_progress_steps,
            root.opponent_mana_progress,
            root.safe_opponent_mana_progress_steps,
            root.score_path_best_steps,
            root.spirit_setup_gain,
            root.safe_supermana_pickup_now,
            root.safe_opponent_mana_pickup_now,
            root.own_drainer_vulnerable,
            root.mana_handoff_to_opponent,
            root.has_roundtrip,
        )
    })
    .unwrap_or_else(|| "none".to_string())
}

fn format_normal_safety_probe(snapshot: Option<NormalRootSafetySnapshot>) -> String {
    snapshot
        .map(|snapshot| {
            format!(
                "imm_loss={} match_point={} opp_gain={} my_gain={} worst_reply={}",
                snapshot.allows_immediate_opponent_win,
                snapshot.opponent_reaches_match_point,
                snapshot.opponent_max_score_gain,
                snapshot.my_score_gain,
                snapshot.worst_reply_score,
            )
        })
        .unwrap_or_else(|| "none".to_string())
}

fn format_scored_root_move_probe(root: Option<&ScoredRootMove>) -> String {
    root.map(|root| {
        format!(
            "root_rank={} eff={} win={} attack={} window={} same_turn_setup={} own_setup={} spirit={} supermana_progress={} super_steps={} opponent_progress={} opp_steps={} score_path_steps={} setup_gain={} pickup_super={} pickup_opp={} vulnerable={} handoff={} roundtrip={}",
            root.root_rank,
            root.efficiency,
            root.wins_immediately,
            root.attacks_opponent_drainer,
            root.same_turn_score_window_value,
            root.spirit_same_turn_score_setup_now,
            root.spirit_own_mana_setup_now,
            root.spirit_development,
            root.supermana_progress,
            root.safe_supermana_progress_steps,
            root.opponent_mana_progress,
            root.safe_opponent_mana_progress_steps,
            root.score_path_best_steps,
            root.spirit_setup_gain,
            root.safe_supermana_pickup_now,
            root.safe_opponent_mana_pickup_now,
            root.own_drainer_vulnerable,
            root.mana_handoff_to_opponent,
            root.has_roundtrip,
        )
    })
    .unwrap_or_else(|| "none".to_string())
}

fn profile_runtime_scored_roots_with_forced_engine_inputs(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
) -> (SmartSearchConfig, Vec<RootEvaluation>, Option<TurnPlan>, Option<Vec<Input>>) {
    let config = calibration_runtime_config(profile_name, game, mode);
    let perspective = game.active_color;
    let mut root_moves = MonsGameModel::ranked_root_moves(game, perspective, config);
    let engine_plan = if config.enable_turn_engine {
        turn_engine_candidate_plan(
            game,
            perspective,
            MonsGameModel::turn_engine_search_config_for_game(game, config),
        )
    } else {
        None
    };
    let forced_engine_inputs = engine_plan.as_ref().and_then(|plan| {
        MonsGameModel::inject_turn_engine_root_candidate(
            game,
            perspective,
            config,
            &mut root_moves,
            plan,
        )
    });
    let (root_moves, scout_visited_nodes) = MonsGameModel::focused_root_candidates_with_forced_inputs(
        game,
        perspective,
        root_moves,
        config,
        true,
        forced_engine_inputs.as_deref(),
    );
    let mut visited_nodes = scout_visited_nodes;
    let mut alpha = i32::MIN;
    let mut scored_roots = Vec::with_capacity(root_moves.len());
    let mut transposition_table = U64HashMap::default();
    let extension_node_budget =
        if config.enable_selective_extensions && config.selective_extension_node_share_bp > 0 {
            ((config.max_visited_nodes * config.selective_extension_node_share_bp as usize) / 10_000)
                .max(1)
        } else {
            0
        };
    let mut extension_nodes_used = 0usize;
    let mut killer_table: KillerTable = [[0u64; 2]; MAX_SMART_SEARCH_DEPTH + 2];
    let mut history_table: HistoryTable = HistoryTable::default();
    let mut quiescence_nodes_used = 0usize;

    for candidate in root_moves {
        if visited_nodes >= config.max_visited_nodes {
            break;
        }
        visited_nodes += 1;
        let candidate_score = MonsGameModel::evaluate_root_candidate_score(
            &candidate,
            perspective,
            alpha,
            &mut visited_nodes,
            config,
            &mut transposition_table,
            &mut extension_nodes_used,
            extension_node_budget,
            true,
            &mut killer_table,
            &mut history_table,
            &mut quiescence_nodes_used,
        );
        if candidate_score > alpha {
            alpha = candidate_score;
        }
        scored_roots.push(RootEvaluation {
            root_rank: candidate.root_rank,
            score: candidate_score,
            efficiency: candidate.efficiency,
            inputs: candidate.inputs,
            game: candidate.game,
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
        });
    }

    (config, scored_roots, engine_plan, forced_engine_inputs)
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
    vs_current_fast: ProReliabilityGateMetrics,
) -> bool {
    pro_reliability_duel_passes(vs_current_pro)
        && pro_reliability_duel_passes(vs_current_normal)
        && pro_reliability_duel_passes(vs_current_fast)
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

fn triage_surface_from_env() -> TriageSurface {
    let value = env::var("SMART_TRIAGE_SURFACE").unwrap_or_else(|_| {
        panic!(
            "SMART_TRIAGE_SURFACE is required (expected one of: opening_reply, primary_pro, reply_risk, supermana, opponent_mana)"
        )
    });
    TriageSurface::parse(value.as_str()).unwrap_or_else(|| {
        panic!(
            "unknown SMART_TRIAGE_SURFACE='{}' (expected one of: opening_reply, primary_pro, reply_risk, supermana, opponent_mana)",
            value
        )
    })
}

fn pro_signal_triage_passes(target_changed: usize, off_target_changed: usize) -> bool {
    target_changed > 0 && off_target_changed <= 1
}

const TRIAGE_TOP_ROOT_DIGEST_SIZE: usize = 5;

fn triage_game_with_items(
    items: Vec<(Location, Item)>,
    active_color: Color,
    turn_number: i32,
) -> MonsGame {
    let mut game = MonsGame::new(false);
    game.board = Board::new_with_items(items.into_iter().collect());
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProTriageSnapshot {
    selected_rank: usize,
    selected_input_fen: String,
    top_root_fens: Vec<String>,
}

fn pro_triage_fixture_snapshot(
    profile_name: &str,
    selector: AutomoveSelector,
    fixture: &TriageFixture,
) -> ProTriageSnapshot {
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
                .position(|root| Input::fen_from_array(&root.inputs) == input_fen)
                .unwrap_or(ranked_roots.len());

            ProTriageSnapshot {
                selected_rank,
                selected_input_fen: input_fen,
                top_root_fens: ranked_roots
                    .iter()
                    .take(TRIAGE_TOP_ROOT_DIGEST_SIZE)
                    .map(|root| Input::fen_from_array(&root.inputs))
                    .collect(),
            }
        },
    )
}

fn pro_triage_surface_signal_changed(
    candidate: &ProTriageSnapshot,
    baseline: &ProTriageSnapshot,
) -> bool {
    candidate.selected_input_fen != baseline.selected_input_fen
        || candidate.selected_rank != baseline.selected_rank
        || candidate.top_root_fens != baseline.top_root_fens
}

fn pro_triage_fixture_changed(
    surface: TriageSurface,
    fixture: &TriageFixture,
    candidate: &ProTriageSnapshot,
    baseline: &ProTriageSnapshot,
) -> bool {
    match surface {
        TriageSurface::PrimaryPro => {
            if let Some(expected) = fixture.expected_selected_input_fen {
                candidate.selected_input_fen == expected && baseline.selected_input_fen != expected
            } else {
                pro_triage_surface_signal_changed(candidate, baseline)
            }
        }
        TriageSurface::OpeningReply => pro_triage_surface_signal_changed(candidate, baseline),
        _ => panic!(
            "unsupported retained pro-triage surface '{}'",
            surface.as_str()
        ),
    }
}

fn compare_pro_triage_fixture_pack(
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
            pro_triage_fixture_snapshot(candidate_profile, candidate_selector, fixture);
        let baseline_snapshot =
            pro_triage_fixture_snapshot(baseline_profile, baseline_selector, fixture);
        let fixture_changed =
            pro_triage_fixture_changed(surface, fixture, &candidate_snapshot, &baseline_snapshot);
        if fixture_changed {
            changed += 1;
        }
        println!(
            "pro triage surface={} fixture={} mode={} opening_book={} expected={:?} changed={} candidate_profile={} candidate={:?} baseline_profile={} baseline={:?}",
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
        "pro triage surface={} summary candidate={} baseline={} changed={}/{}",
        surface.as_str(),
        candidate_profile,
        baseline_profile,
        changed,
        fixtures.len()
    );
    changed
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
                    ratio_limit
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
fn pro_reliability_gate_passes_only_when_all_matchups_clear_win_confidence_and_move_time() {
    let passing = ProReliabilityGateMetrics {
        win_rate: 0.90,
        confidence: 0.99,
        candidate_avg_ms: 700.0,
    };
    assert!(pro_reliability_gate_passes(passing, passing, passing));
    assert!(!pro_reliability_gate_passes(
        ProReliabilityGateMetrics {
            win_rate: 0.89,
            ..passing
        },
        passing,
        passing
    ));
    assert!(!pro_reliability_gate_passes(
        passing,
        ProReliabilityGateMetrics {
            confidence: 0.98,
            ..passing
        },
        passing
    ));
    assert!(!pro_reliability_gate_passes(
        passing,
        passing,
        ProReliabilityGateMetrics {
            candidate_avg_ms: 700.01,
            ..passing
        }
    ));
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
fn pro_signal_triage_accepts_target_change_with_bounded_off_target_churn() {
    assert!(pro_signal_triage_passes(2, 1));
    assert!(pro_signal_triage_passes(1, 0));
    assert!(!pro_signal_triage_passes(0, 0));
    assert!(!pro_signal_triage_passes(1, 2));
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
fn runtime_pro_turn_engine_v30_profile_prefers_safe_white_opening_turn_one_tail_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 2 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xS0xn05/n03E0xA0xn02Y0xn03",
        false,
    )
    .expect("white opening turn-one tail fen should be valid");
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game),
        "l10,3;l9,3"
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
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game),
        "l10,7;l9,7"
    );
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
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game),
        "l9,6;l8,6"
    );
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
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game),
        "l0,3;l1,3"
    );
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
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game),
        "l1,5;l3,3;l2,2"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_does_not_seed_cached_plain_spirit_continuation_when_head_is_rejected() {
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
    assert_eq!(Input::fen_from_array(&first), "l9,7;l7,8;l7,7");
    let after_first = MonsGameModel::apply_inputs_for_search(&game, first.as_slice())
        .expect("v30 first spirit-setup chunk should be legal");
    let after_config = calibration_runtime_config(
        "runtime_pro_turn_engine_v30",
        &after_first,
        SmartAutomovePreference::Pro,
    );
    assert!(
        turn_engine_cached_step(&after_first, calibration_turn_engine_config(after_config))
            .is_none(),
        "v30 should not seed a cached continuation when the plain spirit head is rejected"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_prefers_safe_black_opening_a_ply19_root() {
    let fixture = primary_pro_fixture_by_id("primary_black_loss_opening_a_ply19");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game),
        "l2,5;l1,4"
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
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game),
        "l4,2;l5,1"
    );
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
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game),
        "l4,9;l4,7;l5,7"
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
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game),
        "l9,5;l8,4"
    );
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

    let opening_changed = compare_pro_triage_fixture_pack(
        TriageSurface::OpeningReply,
        candidate_profile_name.as_str(),
        candidate_selector,
        baseline_profile_name.as_str(),
        baseline_selector,
        opening_reply_triage_fixtures().as_slice(),
    );
    let primary_changed = compare_pro_triage_fixture_pack(
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
#[ignore = "reliability gate: retained pro profile vs runtime_current pro, normal, and fast at pro budget with move-time cap"]
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
        println!(
            "pro reliability gate: guardrails skipped by SMART_PRO_RELIABILITY_SKIP_GUARDRAILS=true"
        );
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
    let fast_seed_tag = format!("{}_vs_fast", seed_tag);

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
    let fast_stats = run_cross_budget_duel_with_timing(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Fast,
        seed_tag: fast_seed_tag.as_str(),
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

    let fast_total_games = fast_stats.matchup.total_games();
    let fast_metrics = ProReliabilityGateMetrics {
        win_rate: fast_stats.matchup.win_rate_points(),
        confidence: fast_stats.matchup.confidence_better_than_even(),
        candidate_avg_ms: fast_stats.timing.candidate_avg_ms(),
    };
    println!(
        "pro reliability gate vs current fast: candidate={} baseline={} total_games={} win_rate={:.4} confidence={:.4} candidate_avg_ms={:.2} baseline_avg_ms={:.2} candidate_turns={} baseline_turns={}",
        candidate_profile,
        baseline_profile,
        fast_total_games,
        fast_metrics.win_rate,
        fast_metrics.confidence,
        fast_metrics.candidate_avg_ms,
        fast_stats.timing.baseline_avg_ms(),
        fast_stats.timing.candidate_turns,
        fast_stats.timing.baseline_turns
    );

    let expected_games = repeats.saturating_mul(games).saturating_mul(2);
    assert_eq!(
        pro_total_games, expected_games,
        "pro reliability gate vs current pro expected {} mirrored games but ran {}",
        expected_games, pro_total_games
    );
    assert_eq!(
        normal_total_games, expected_games,
        "pro reliability gate vs current normal expected {} mirrored games but ran {}",
        expected_games, normal_total_games
    );
    assert_eq!(
        fast_total_games, expected_games,
        "pro reliability gate vs current fast expected {} mirrored games but ran {}",
        expected_games, fast_total_games
    );
    assert!(
        pro_reliability_gate_passes(pro_metrics, normal_metrics, fast_metrics),
        "pro reliability gate failed overall: vs_current_pro [win_rate {:.4} confidence {:.4} candidate_avg_ms {:.2}ms] vs_current_normal [win_rate {:.4} confidence {:.4} candidate_avg_ms {:.2}ms] vs_current_fast [win_rate {:.4} confidence {:.4} candidate_avg_ms {:.2}ms] (required each duel to satisfy win_rate >= {:.2}, confidence >= {:.2}, candidate_avg_ms <= {:.2}ms)",
        pro_metrics.win_rate,
        pro_metrics.confidence,
        pro_metrics.candidate_avg_ms,
        normal_metrics.win_rate,
        normal_metrics.confidence,
        normal_metrics.candidate_avg_ms,
        fast_metrics.win_rate,
        fast_metrics.confidence,
        fast_metrics.candidate_avg_ms,
        SMART_PRO_RELIABILITY_WIN_RATE_MIN,
        SMART_PRO_RELIABILITY_CONFIDENCE_MIN,
        SMART_PRO_RELIABILITY_MOVE_AVG_MS_MAX
    );
    assert_pro_reliability_duel_passes("pro reliability gate vs current pro", pro_metrics);
    assert_pro_reliability_duel_passes("pro reliability gate vs current normal", normal_metrics);
    assert_pro_reliability_duel_passes("pro reliability gate vs current fast", fast_metrics);
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
#[ignore = "diagnostic: inspect retained pro-triage churn fixtures for runtime_pro_turn_engine_v30"]
fn smart_automove_pro_triage_retained_churn_probe() {
    let candidate_profile = "runtime_pro_turn_engine_v30";
    let baseline_profile = "runtime_current";
    let fixture_ids = [
        "primary_spirit_setup",
        "primary_pvs_sensitive_search",
        "primary_black_reliability_opening_3_ply4",
        "primary_white_harvest_loss_c_ply24",
        "human_win_pro_c",
    ];

    println!(
        "retained churn probe: candidate={} baseline={} fixtures={}",
        candidate_profile,
        baseline_profile,
        fixture_ids.len()
    );
    for fixture_id in fixture_ids {
        let fixture = primary_pro_fixture_by_id(fixture_id);
        with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
            for profile_name in [candidate_profile, baseline_profile] {
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
                let pre_accept_selected =
                    MonsGameModel::pick_root_move_with_exploration(
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
                let head_plan = if config.enable_turn_engine {
                    turn_engine_candidate_plan(
                        &fixture.game,
                        fixture.game.active_color,
                        MonsGameModel::turn_engine_search_config_for_game(&fixture.game, config),
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
                let my_score_before = MonsGameModel::score_for_color(&fixture.game, fixture.game.active_color);
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
#[ignore = "diagnostic: inspect runtime-faithful forced-engine acceptance on retained churn fixtures"]
fn smart_automove_pro_runtime_faithful_retained_churn_probe() {
    let fixture_ids = [
        "primary_spirit_setup",
        "primary_pvs_sensitive_search",
        "primary_black_reliability_opening_3_ply4",
        "primary_white_harvest_loss_c_ply24",
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
                "runtime_pro_turn_engine_v30",
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
            "RUNTIME_FAITHFUL fixture={} forced_inputs={:?} pre_accept_rank={:?} pre_accept={} head_rank={:?} head={:?} accepted={} selected_root=\"{}\" head_root=\"{}\" normal_safety(selected=\"{}\" head=\"{}\")",
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

#[test]
#[ignore = "diagnostic: inspect selector competition gates on human_win_pro_c"]
fn smart_automove_pro_human_win_pro_c_selector_probe() {
    let fixture = primary_pro_fixture_by_id("human_win_pro_c");
    let (config, scored_roots) =
        profile_scored_roots("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game);
    let perspective = fixture.game.active_color;
    let filtered = MonsGameModel::filtered_root_candidate_indices(
        &fixture.game,
        scored_roots.as_slice(),
        perspective,
        config,
    );
    let projections = MonsGameModel::turn_engine_spirit_root_projections(
        scored_roots.as_slice(),
        filtered.as_slice(),
        perspective,
        config,
    );
    let progress_competes = MonsGameModel::safe_progress_competes_with_spirit_pref(
        scored_roots.as_slice(),
        filtered.as_slice(),
        config.turn_engine_mode,
    );
    let followup_progress_competes = MonsGameModel::followup_progress_competes_with_spirit_pref(
        &fixture.game,
        scored_roots.as_slice(),
        filtered.as_slice(),
        perspective,
        config,
    );
    let risky_score_competes = MonsGameModel::risky_score_competes_with_spirit_pref(
        scored_roots.as_slice(),
        filtered.as_slice(),
        config.turn_engine_mode,
    );
    let negative_deny_competes = MonsGameModel::negative_deny_competes_with_spirit_pref(
        scored_roots.as_slice(),
        filtered.as_slice(),
        perspective,
        config,
    );
    let score_competes = MonsGameModel::score_competes_with_spirit_pref(
        scored_roots.as_slice(),
        filtered.as_slice(),
        config.turn_engine_mode,
    );
    let projection_competes = MonsGameModel::projection_competes_with_spirit_pref(
        scored_roots.as_slice(),
        filtered.as_slice(),
        perspective,
        config,
    );
    let risky_recovery_competes = MonsGameModel::risky_recovery_competes_with_spirit_pref(
        &fixture.game,
        scored_roots.as_slice(),
        filtered.as_slice(),
        perspective,
        config,
    );
    let final_progress_reentry = MonsGameModel::pro_v2_plain_spirit_cluster_progress_reentry(
        &fixture.game,
        scored_roots.as_slice(),
        filtered.as_slice(),
        perspective,
        config,
    );
    let selected = MonsGameModel::pick_root_move_with_exploration(
        &fixture.game,
        scored_roots.as_slice(),
        perspective,
        config,
    );
    let baseline_selected = profile_decision_inputs("runtime_current", fixture.mode, &fixture.game);
    let selected_fen = Input::fen_from_array(&selected);
    let baseline_selected_fen = Input::fen_from_array(&baseline_selected);

    println!(
        "HUMAN_WIN_PRO_C_SELECTOR selected={} baseline_selected={} filtered_len={} progress_competes={} followup_progress_competes={} risky_score_competes={} negative_deny_competes={} score_competes={} projection_competes={} risky_recovery_competes={} final_progress_reentry={:?}",
        selected_fen,
        baseline_selected_fen,
        filtered.len(),
        progress_competes,
        followup_progress_competes,
        risky_score_competes,
        negative_deny_competes,
        score_competes,
        projection_competes,
        risky_recovery_competes,
        final_progress_reentry.map(|index| Input::fen_from_array(&scored_roots[index].inputs)),
    );

    let mut followup_scores = std::collections::HashMap::new();
    for (rank, root) in scored_roots.iter().enumerate().take(18) {
        let fen = Input::fen_from_array(&root.inputs);
        let projection = projections.get(&rank).map(|plan| {
            (
                plan.plan.head_family,
                plan.plan.goal_family,
                plan.plan.utility,
                plan.plan.head_utility,
            )
        });
        let followup_floor = *followup_scores.entry(rank).or_insert_with(|| {
            MonsGameModel::pro_v2_spirit_followup_floor_score(&root.game, perspective, config)
        });
        println!(
            "HUMAN_WIN_PRO_C_ROOT rank={} fen={} filtered={} projected={} projection={:?} followup_floor={} root=\"{}\"",
            rank,
            fen,
            filtered.contains(&rank),
            projections.contains_key(&rank),
            projection,
            followup_floor,
            format_root_probe(Some(root)),
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect forced engine shortlist seam on primary_white_harvest_loss_c_ply24"]
fn smart_automove_pro_white_harvest_forced_root_probe() {
    let fixture = primary_pro_fixture_by_id("primary_white_harvest_loss_c_ply24");
    let config = calibration_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let perspective = fixture.game.active_color;
    let root_moves = MonsGameModel::ranked_root_moves(&fixture.game, perspective, config);
    let root_target = "l7,2;l6,1";

    let root_target_rank = root_moves
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == root_target);
    let engine_plan = turn_engine_candidate_plan(
        &fixture.game,
        perspective,
        MonsGameModel::turn_engine_search_config_for_game(&fixture.game, config),
    )
    .expect("white harvest fixture should materialize a turn-engine plan");
    let forced_chunk = engine_plan
        .compiled_chunks
        .first()
        .cloned()
        .expect("engine plan should have a first chunk");

    let mut injected_root_moves = root_moves.clone();
    let forced_engine_inputs = MonsGameModel::inject_turn_engine_root_candidate(
        &fixture.game,
        perspective,
        config,
        &mut injected_root_moves,
        &engine_plan,
    );
    let injected_target_rank = injected_root_moves
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == root_target);
    let injected_forced_rank = injected_root_moves
        .iter()
        .position(|root| root.inputs == forced_chunk);
    let (focused_root_moves, _) = MonsGameModel::focused_root_candidates_with_forced_inputs(
        &fixture.game,
        perspective,
        injected_root_moves.clone(),
        config,
        true,
        forced_engine_inputs.as_deref(),
    );
    let focused_target_rank = focused_root_moves
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == root_target);
    let focused_forced_rank = focused_root_moves
        .iter()
        .position(|root| root.inputs == forced_chunk);

    println!(
        "WHITE_HARVEST_FORCED_ROOT raw_target_rank={:?} injected_target_rank={:?} injected_forced_rank={:?} focused_target_rank={:?} focused_forced_rank={:?} forced_inputs={:?} head_family={:?} goal_family={:?}",
        root_target_rank,
        injected_target_rank,
        injected_forced_rank,
        focused_target_rank,
        focused_forced_rank,
        forced_engine_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs)),
        engine_plan.head_family,
        engine_plan.goal_family,
    );

    for (rank, root) in root_moves.iter().enumerate().take(10) {
        println!(
            "WHITE_HARVEST_RAW rank={} fen={} root=\"{}\"",
            rank,
            Input::fen_from_array(&root.inputs),
            format_scored_root_move_probe(Some(root)),
        );
    }
    for (rank, root) in focused_root_moves.iter().enumerate().take(10) {
        println!(
            "WHITE_HARVEST_FOCUSED rank={} fen={} forced_match={} root=\"{}\"",
            rank,
            Input::fen_from_array(&root.inputs),
            root.inputs == forced_chunk,
            format_scored_root_move_probe(Some(root)),
        );
    }

    assert!(
        forced_engine_inputs.is_none(),
        "white harvest forced-root probe should reflect the retained rejection of the non-progress window head",
    );
    assert_eq!(focused_forced_rank, None);
}

#[test]
#[ignore = "diagnostic: inspect forced engine shortlist seam on primary_spirit_setup"]
fn smart_automove_pro_spirit_setup_forced_root_probe() {
    let fixture = primary_pro_fixture_by_id("primary_spirit_setup");
    let config =
        calibration_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let perspective = fixture.game.active_color;
    let root_moves = MonsGameModel::ranked_root_moves(&fixture.game, perspective, config);
    let baseline_target = "l9,7;l7,8;l7,7";
    let safe_progress_target = "l9,5;l8,6";

    let baseline_target_rank = root_moves
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == baseline_target);
    let safe_progress_target_rank = root_moves
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == safe_progress_target);
    let engine_plan = turn_engine_candidate_plan(
        &fixture.game,
        perspective,
        MonsGameModel::turn_engine_search_config_for_game(&fixture.game, config),
    )
    .expect("spirit setup fixture should materialize a turn-engine plan");
    let forced_chunk = engine_plan
        .compiled_chunks
        .first()
        .cloned()
        .expect("engine plan should have a first chunk");

    let mut injected_root_moves = root_moves.clone();
    let forced_engine_inputs = MonsGameModel::inject_turn_engine_root_candidate(
        &fixture.game,
        perspective,
        config,
        &mut injected_root_moves,
        &engine_plan,
    );
    let injected_baseline_rank = injected_root_moves
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == baseline_target);
    let injected_safe_progress_rank = injected_root_moves
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == safe_progress_target);
    let injected_forced_rank = injected_root_moves
        .iter()
        .position(|root| root.inputs == forced_chunk);
    let (focused_root_moves, _) = MonsGameModel::focused_root_candidates_with_forced_inputs(
        &fixture.game,
        perspective,
        injected_root_moves.clone(),
        config,
        true,
        forced_engine_inputs.as_deref(),
    );
    let focused_baseline_rank = focused_root_moves
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == baseline_target);
    let focused_safe_progress_rank = focused_root_moves
        .iter()
        .position(|root| Input::fen_from_array(&root.inputs) == safe_progress_target);
    let focused_forced_rank = focused_root_moves
        .iter()
        .position(|root| root.inputs == forced_chunk);

    println!(
        "SPIRIT_SETUP_FORCED_ROOT raw_baseline_rank={:?} raw_safe_progress_rank={:?} injected_baseline_rank={:?} injected_safe_progress_rank={:?} injected_forced_rank={:?} focused_baseline_rank={:?} focused_safe_progress_rank={:?} focused_forced_rank={:?} forced_inputs={:?} head_family={:?} goal_family={:?}",
        baseline_target_rank,
        safe_progress_target_rank,
        injected_baseline_rank,
        injected_safe_progress_rank,
        injected_forced_rank,
        focused_baseline_rank,
        focused_safe_progress_rank,
        focused_forced_rank,
        forced_engine_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs)),
        engine_plan.head_family,
        engine_plan.goal_family,
    );

    for (rank, root) in root_moves.iter().enumerate().take(8) {
        println!(
            "SPIRIT_SETUP_RAW rank={} fen={} baseline_match={} safe_progress_match={} forced_match={} root=\"{}\"",
            rank,
            Input::fen_from_array(&root.inputs),
            Input::fen_from_array(&root.inputs) == baseline_target,
            Input::fen_from_array(&root.inputs) == safe_progress_target,
            root.inputs == forced_chunk,
            format_scored_root_move_probe(Some(root)),
        );
    }
    for (rank, root) in focused_root_moves.iter().enumerate().take(8) {
        println!(
            "SPIRIT_SETUP_FOCUSED rank={} fen={} baseline_match={} safe_progress_match={} forced_match={} root=\"{}\"",
            rank,
            Input::fen_from_array(&root.inputs),
            Input::fen_from_array(&root.inputs) == baseline_target,
            Input::fen_from_array(&root.inputs) == safe_progress_target,
            root.inputs == forced_chunk,
            format_scored_root_move_probe(Some(root)),
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect full selector path on primary_black_reliability_opening_3_ply4"]
fn smart_automove_pro_black_reliability_opening_3_selector_probe() {
    let fixture = primary_pro_fixture_by_id("primary_black_reliability_opening_3_ply4");
    let candidate_profile = "runtime_pro_turn_engine_v30";
    let baseline_profile = "runtime_current";
    let pro_runtime = SearchBudget::from_preference(SmartAutomovePreference::Pro)
        .runtime_config_for_game(&fixture.game);
    let configured_runtime =
        calibration_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let mut low_budget_disabled_runtime = configured_runtime;
    low_budget_disabled_runtime.enable_turn_engine_low_budget_guard = false;
    let guarded_inputs = model_runtime_pro_turn_engine_v30(&fixture.game, pro_runtime);
    let direct_configured_inputs =
        MonsGameModel::smart_search_best_inputs(&fixture.game, configured_runtime);
    let low_budget_disabled_inputs =
        MonsGameModel::smart_search_best_inputs(&fixture.game, low_budget_disabled_runtime);
    let plain_current_best_inputs =
        MonsGameModel::smart_search_best_inputs(&fixture.game, pro_runtime);

    println!(
        "BLACK_RELIABILITY_GUARDS turn={} mons_moves={} can_action={} can_mana={} guarded={} direct_configured={} low_budget_disabled={} plain_current_best={} black_turn_two_turn_start_action_mana={} black_turn_two_mana_only={} black_turn_four_turn_start_action_mana={}",
        fixture.game.turn_number,
        fixture.game.mons_moves_count,
        fixture.game.player_can_use_action(),
        fixture.game.player_can_move_mana(),
        Input::fen_from_array(&guarded_inputs),
        Input::fen_from_array(&direct_configured_inputs),
        Input::fen_from_array(&low_budget_disabled_inputs),
        Input::fen_from_array(&plain_current_best_inputs),
        fixture.game.active_color == Color::Black
            && fixture.game.turn_number == 2
            && fixture.game.mons_moves_count == 0
            && fixture.game.player_can_use_action()
            && fixture.game.player_can_move_mana(),
        fixture.game.active_color == Color::Black
            && fixture.game.turn_number == 2
            && fixture.game.mons_moves_count > 0
            && !fixture.game.player_can_use_action()
            && fixture.game.player_can_move_mana(),
        fixture.game.active_color == Color::Black
            && fixture.game.turn_number == 4
            && fixture.game.mons_moves_count == 0
            && fixture.game.player_can_use_action()
            && fixture.game.player_can_move_mana(),
    );

    for profile_name in [candidate_profile, baseline_profile] {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let selected = profile_decision_inputs(profile_name, fixture.mode, &fixture.game);
        let selected_fen = Input::fen_from_array(&selected);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let engine_diag = turn_engine_diagnostics_snapshot();
        let exact_diag = exact_query_diagnostics_snapshot();
        let (config, scored_roots) = profile_scored_roots(profile_name, fixture.mode, &fixture.game);
        let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
            &fixture.game,
            scored_roots.as_slice(),
            fixture.game.active_color,
            config,
        );

        println!(
            "BLACK_RELIABILITY_SELECTOR profile={} selected={} pre_accept={} selector(last_stage={} head_calls={} head_hits={} child_calls={} children={} shortlist={} full_pass={} prefer_builds={} prefer_hits={}) engine(accepted={} cache_hits={} cache_misses={} reply_calls={}) exact(tactical_spirit_calls={} tactical_spirit_hits={} secure_mana_calls={} secure_mana_hits={} pickup_calls={} pickup_hits={})",
            profile_name,
            selected_fen,
            Input::fen_from_array(&pre_accept_selected),
            selector_diag.last_return_stage,
            selector_diag.head_plan_calls,
            selector_diag.head_plan_hits,
            selector_diag.ranked_child_states_calls,
            selector_diag.ranked_child_states_children_enumerated,
            selector_diag.child_ordering_shortlist_children,
            selector_diag.child_ordering_full_pass_children,
            selector_diag.search_preferability_builds,
            selector_diag.search_preferability_cache_hits,
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
        );

        for (rank, root) in scored_roots.iter().enumerate().take(12) {
            println!(
                "BLACK_RELIABILITY_ROOT profile={} rank={} fen={} selected_match={} pre_accept_match={} root=\"{}\"",
                profile_name,
                rank,
                Input::fen_from_array(&root.inputs),
                root.inputs == selected,
                root.inputs == pre_accept_selected,
                format_root_probe(Some(root)),
            );
        }
    }
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_white_harvest_non_progress_window_injection() {
    let fixture = primary_pro_fixture_by_id("primary_white_harvest_loss_c_ply24");
    let config = calibration_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let perspective = fixture.game.active_color;
    let mut root_moves = MonsGameModel::ranked_root_moves(&fixture.game, perspective, config);
    let engine_plan = turn_engine_candidate_plan(
        &fixture.game,
        perspective,
        MonsGameModel::turn_engine_search_config_for_game(&fixture.game, config),
    )
    .expect("white harvest fixture should materialize a turn-engine plan");

    assert_eq!(engine_plan.head_family, TurnPlanFamily::SafeOpponentManaProgress);
    assert_eq!(
        Input::fen_from_array(
            engine_plan
                .compiled_chunks
                .first()
                .expect("plan should include a first chunk"),
        ),
        "l8,5;l7,4",
    );
    assert!(
        MonsGameModel::inject_turn_engine_root_candidate(
            &fixture.game,
            perspective,
            config,
            &mut root_moves,
            &engine_plan,
        )
        .is_none(),
        "a non-progress score-window first chunk should not be forced ahead of a concrete progress cluster",
    );
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game),
        "l7,2;l6,1",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_weaker_plain_spirit_head_on_primary_spirit_setup() {
    let fixture = primary_pro_fixture_by_id("primary_spirit_setup");
    let (config, scored_roots, head_plan, forced_engine_inputs) =
        profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            fixture.mode,
            &fixture.game,
        );
    let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
        &fixture.game,
        scored_roots.as_slice(),
        fixture.game.active_color,
        config,
    );
    let head_plan = head_plan.expect("spirit setup fixture should retain a head plan");

    assert_eq!(forced_engine_inputs, None);
    assert_eq!(Input::fen_from_array(&pre_accept_selected), "l9,7;l7,8;l7,7");
    assert_eq!(
        Input::fen_from_array(
            head_plan
                .compiled_chunks
                .first()
                .expect("head plan should include a first chunk"),
        ),
        "l9,7;l7,8;l8,7",
    );
    assert!(
        !MonsGameModel::accept_turn_engine_head_after_search(
            &fixture.game,
            fixture.game.active_color,
            config,
            scored_roots.as_slice(),
            pre_accept_selected.as_slice(),
            &head_plan,
        ),
        "a weaker plain spirit sibling should not override the stronger selected spirit root",
    );
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game),
        "l9,7;l7,8;l7,7",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_skips_black_turn_two_low_budget_clamp_with_full_resources() {
    let fixture = primary_pro_fixture_by_id("primary_black_reliability_opening_3_ply4");
    let configured_runtime =
        calibration_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let mut low_budget_disabled_runtime = configured_runtime;
    low_budget_disabled_runtime.enable_turn_engine_low_budget_guard = false;

    assert_eq!(
        Input::fen_from_array(&MonsGameModel::smart_search_best_inputs(
            &fixture.game,
            configured_runtime,
        )),
        "l1,3;l3,4;l3,3",
    );
    assert_eq!(
        Input::fen_from_array(&MonsGameModel::smart_search_best_inputs(
            &fixture.game,
            low_budget_disabled_runtime,
        )),
        "l1,3;l3,4;l3,3",
    );
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game),
        "l1,3;l3,4;l3,3",
    );
}
