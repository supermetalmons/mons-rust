use super::harness::*;
use super::profiles::*;
use super::*;
use crate::models::automove_exact::{
    clear_exact_query_diagnostics, clear_exact_state_analysis_cache,
    exact_query_diagnostics_snapshot, ExactQueryDiagnostics,
};
use crate::models::automove_turn_engine::{
    clear_turn_engine_diagnostics, clear_turn_engine_plan_cache, turn_engine_cached_step,
    turn_engine_candidate_plan, turn_engine_diagnostics_snapshot, TurnEngineConfig,
    TurnEngineDiagnostics,
};
use crate::models::automove_turn_planner::clear_turn_opportunity_plan_cache;
use crate::models::mons_game_model::{
    clear_turn_engine_selector_diagnostics, pro_v2_root_advisor_decision_snapshot,
    turn_engine_selector_diagnostics_snapshot, TurnEngineSelectorDiagnostics,
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
    profile_runtime_config_for_name(profile_name, game, base).unwrap_or_else(|| {
        panic!(
            "profile '{}' does not expose a runtime config",
            profile_name
        )
    })
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
    clear_turn_engine_selector_diagnostics();
    profile_runtime_inputs(profile_name, mode, game)
}

fn profile_decision_move_fen(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
) -> String {
    Input::fen_from_array(&profile_decision_inputs(profile_name, mode, game))
}

fn profile_runtime_inputs(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
) -> Vec<Input> {
    let selector = profile_selector_from_name(profile_name)
        .unwrap_or_else(|| panic!("profile '{}' not found", profile_name));
    let config = calibration_runtime_config(profile_name, game, mode);
    select_inputs_with_runtime_fallback(selector, game, config)
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
    let extension_node_budget = if config.enable_selective_extensions
        && config.selective_extension_node_share_bp > 0
    {
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

fn format_root_advisor_entry_probe(
    entry: &crate::models::mons_game_model::ProV2RootAdvisorEntry,
) -> String {
    format!(
        "{}:{:?}:{:?}:rank{}",
        Input::fen_from_array(&entry.inputs),
        entry.family,
        entry.reason,
        entry.root_rank,
    )
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeDecisionProbe {
    selected_input_fen: String,
    selected_rank: Option<usize>,
    pre_accept_input_fen: String,
    pre_accept_rank: Option<usize>,
    top_root_fens: Vec<String>,
    selector_last_stage: &'static str,
    selector_head_calls: usize,
    selector_head_hits: usize,
    head_input_fen: Option<String>,
    head_rank: Option<usize>,
    head_accepted: bool,
    selected_root: String,
    head_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DuelTraceTurn {
    ply: usize,
    board_fen: String,
    move_fen: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DuelTraceGame {
    result: MatchResult,
    final_fen: String,
    candidate_turns: Vec<DuelTraceTurn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FirstDivergence {
    ply: usize,
    board_fen: String,
    candidate_move_fen: String,
    baseline_move_fen: String,
}

fn runtime_decision_probe(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
) -> RuntimeDecisionProbe {
    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_opportunity_plan_cache();
    clear_turn_engine_plan_cache();
    clear_turn_engine_diagnostics();
    clear_turn_engine_selector_diagnostics();

    let selected = profile_runtime_inputs(profile_name, mode, game);
    let selected_input_fen = Input::fen_from_array(&selected);
    let selector_diag = turn_engine_selector_diagnostics_snapshot();

    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_opportunity_plan_cache();
    clear_turn_engine_plan_cache();
    clear_turn_engine_diagnostics();
    clear_turn_engine_selector_diagnostics();

    let (config, scored_roots, head_plan, _) =
        profile_runtime_scored_roots_with_forced_engine_inputs(profile_name, mode, game);
    let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
        game,
        scored_roots.as_slice(),
        game.active_color,
        config,
    );
    let pre_accept_input_fen = Input::fen_from_array(&pre_accept_selected);
    let selected_rank = scored_roots.iter().position(|root| root.inputs == selected);
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
    let head_accepted = head_plan.as_ref().is_some_and(|plan| {
        MonsGameModel::accept_turn_engine_head_after_search(
            game,
            game.active_color,
            config,
            scored_roots.as_slice(),
            pre_accept_selected.as_slice(),
            plan,
        )
    });
    let selected_root = format_root_probe(scored_roots.iter().find(|root| root.inputs == selected));
    let head_root = format_root_probe(head_rank.and_then(|index| scored_roots.get(index)));

    RuntimeDecisionProbe {
        selected_input_fen,
        selected_rank,
        pre_accept_input_fen,
        pre_accept_rank,
        top_root_fens: scored_roots
            .iter()
            .take(TRIAGE_TOP_ROOT_DIGEST_SIZE)
            .map(|root| Input::fen_from_array(&root.inputs))
            .collect(),
        selector_last_stage: selector_diag.last_return_stage,
        selector_head_calls: selector_diag.head_plan_calls,
        selector_head_hits: selector_diag.head_plan_hits,
        head_input_fen: head_plan
            .as_ref()
            .and_then(|plan| plan.compiled_chunks.first())
            .map(|chunk| Input::fen_from_array(chunk)),
        head_rank,
        head_accepted,
        selected_root,
        head_root,
    }
}

fn exact_opportunity_context_probe(game: &MonsGame) -> String {
    let context = crate::models::automove_exact::exact_opportunity_context(game, game.active_color);
    format!(
        "turn={} mons_moves={} can_action={} can_mana={} opp_win={} window={} deny={} attack={} super_steps={:?} opp_steps={:?} drainer_safety={}",
        game.turn_number,
        game.mons_moves_count,
        game.player_can_use_action(),
        game.player_can_move_mana(),
        context.opponent_can_win_immediately,
        context.delta.same_turn_score_window_value,
        context.delta.opponent_window_deny_gain,
        context.delta.drainer_attack_available,
        context.delta.safe_supermana_progress_steps,
        context.delta.safe_opponent_mana_progress_steps,
        context.delta.drainer_safety,
    )
}

fn pro_v2_legacy_selector_probe(
    game: &MonsGame,
    mode: SmartAutomovePreference,
) -> (String, String, Vec<String>, Vec<String>) {
    let (mut config, scored_roots, _, _) =
        profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            mode,
            game,
        );
    let candidate_indices = MonsGameModel::filtered_root_candidate_indices(
        game,
        scored_roots.as_slice(),
        game.active_color,
        config,
    );
    let all_indices = (0..scored_roots.len()).collect::<Vec<_>>();
    config.turn_engine_mode = TurnEngineMode::ProV1;
    let selected_from_candidates =
        MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
            game,
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            game.active_color,
            config,
        );
    let selected_from_full_pool = MonsGameModel::pick_root_move_with_exploration_from_candidate_indices(
        game,
        scored_roots.as_slice(),
        all_indices.as_slice(),
        game.active_color,
        config,
    );
    (
        Input::fen_from_array(&selected_from_candidates),
        Input::fen_from_array(&selected_from_full_pool),
        candidate_indices
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect(),
        all_indices
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect(),
        )
}

fn assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
    label: &str,
    fen: &str,
    expected_selected: &str,
) {
    let game = MonsGame::from_fen(fen, false).expect("probe fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);
    let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let (_, current_scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );

    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );
    let current_root = format_root_probe(
        current_scored_roots
            .iter()
            .find(|root| Input::fen_from_array(&root.inputs) == current_selected),
    );
    let candidate_expected_root = format_root_probe(
        scored_roots
            .iter()
            .find(|root| Input::fen_from_array(&root.inputs) == expected_selected),
    );
    let candidate_top_root_details = scored_roots
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
    let current_top_root_details = current_scored_roots
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
        "{} current_selected={} current_root=\"{}\" candidate_expected_root=\"{}\" context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} candidate_top_root_details={:?} current_top_root_details={:?} probe={:?} advisor={:?}",
        label,
        current_selected,
        current_root,
        candidate_expected_root,
        exact_opportunity_context_probe(&game),
        legacy_selected,
        legacy_full_pool_selected,
        legacy_candidates,
        legacy_full_pool,
        candidate_top_root_details,
        current_top_root_details,
        probe,
        advisor
    );
    assert_eq!(current_selected, expected_selected);
    assert_eq!(probe.selected_input_fen, expected_selected);
}

fn profile_duel_turn_inputs(
    game: &MonsGame,
    candidate_profile: &str,
    opponent_profile: &str,
    opponent_mode: SmartAutomovePreference,
    candidate_is_white: bool,
) -> Result<(bool, Vec<Input>), MatchResult> {
    if let Some(winner_color) = game.winner_color() {
        return Err(match_result_from_winner(winner_color, candidate_is_white));
    }

    let candidate_to_move = if candidate_is_white {
        game.active_color == Color::White
    } else {
        game.active_color == Color::Black
    };
    let (profile_name, mode) = if candidate_to_move {
        (candidate_profile, SmartAutomovePreference::Pro)
    } else {
        (opponent_profile, opponent_mode)
    };
    Ok((candidate_to_move, profile_runtime_inputs(profile_name, mode, game)))
}

fn play_profile_duel_trace(
    candidate_profile: &str,
    opponent_profile: &str,
    opponent_mode: SmartAutomovePreference,
    opening_fen: &str,
    candidate_is_white: bool,
    max_plies: usize,
) -> DuelTraceGame {
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_opportunity_plan_cache();
    clear_turn_engine_plan_cache();
    clear_turn_engine_diagnostics();
    clear_turn_engine_selector_diagnostics();

    let mut candidate_turns = Vec::new();
    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return DuelTraceGame {
                result: match_result_from_winner(winner_color, candidate_is_white),
                final_fen: game.fen(),
                candidate_turns,
            };
        }

        let board_fen = game.fen();
        let (candidate_to_move, inputs) = match profile_duel_turn_inputs(
            &game,
            candidate_profile,
            opponent_profile,
            opponent_mode,
            candidate_is_white,
        ) {
            Ok(turn) => turn,
            Err(result) => {
                return DuelTraceGame {
                    result,
                    final_fen: game.fen(),
                    candidate_turns,
                };
            }
        };
        if candidate_to_move {
            candidate_turns.push(DuelTraceTurn {
                ply,
                board_fen,
                move_fen: Input::fen_from_array(&inputs),
            });
        }

        if inputs.is_empty() {
            return DuelTraceGame {
                result: if candidate_to_move {
                    MatchResult::OpponentWin
                } else {
                    MatchResult::CandidateWin
                },
                final_fen: game.fen(),
                candidate_turns,
            };
        }
        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return DuelTraceGame {
                result: if candidate_to_move {
                    MatchResult::OpponentWin
                } else {
                    MatchResult::CandidateWin
                },
                final_fen: game.fen(),
                candidate_turns,
            };
        }
    }

    DuelTraceGame {
        result: match adjudicate_non_terminal_game(&game) {
            Some(winner_color) => match_result_from_winner(winner_color, candidate_is_white),
            None => MatchResult::Draw,
        },
        final_fen: game.fen(),
        candidate_turns,
    }
}

fn play_profile_duel_trace_with_forced_candidate_first_move(
    candidate_profile: &str,
    opponent_profile: &str,
    opponent_mode: SmartAutomovePreference,
    opening_fen: &str,
    candidate_is_white: bool,
    max_plies: usize,
    forced_candidate_first_move: &[Input],
) -> DuelTraceGame {
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_opportunity_plan_cache();
    clear_turn_engine_plan_cache();
    clear_turn_engine_diagnostics();
    clear_turn_engine_selector_diagnostics();

    let mut candidate_turns = Vec::new();
    let mut forced_first_move_used = false;
    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return DuelTraceGame {
                result: match_result_from_winner(winner_color, candidate_is_white),
                final_fen: game.fen(),
                candidate_turns,
            };
        }

        let board_fen = game.fen();
        let (candidate_to_move, mut inputs) = match profile_duel_turn_inputs(
            &game,
            candidate_profile,
            opponent_profile,
            opponent_mode,
            candidate_is_white,
        ) {
            Ok(turn) => turn,
            Err(result) => {
                return DuelTraceGame {
                    result,
                    final_fen: game.fen(),
                    candidate_turns,
                };
            }
        };
        if candidate_to_move && !forced_first_move_used {
            inputs = forced_candidate_first_move.to_vec();
            forced_first_move_used = true;
        }
        if candidate_to_move {
            candidate_turns.push(DuelTraceTurn {
                ply,
                board_fen,
                move_fen: Input::fen_from_array(&inputs),
            });
        }

        if inputs.is_empty() {
            return DuelTraceGame {
                result: if candidate_to_move {
                    MatchResult::OpponentWin
                } else {
                    MatchResult::CandidateWin
                },
                final_fen: game.fen(),
                candidate_turns,
            };
        }
        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return DuelTraceGame {
                result: if candidate_to_move {
                    MatchResult::OpponentWin
                } else {
                    MatchResult::CandidateWin
                },
                final_fen: game.fen(),
                candidate_turns,
            };
        }
    }

    DuelTraceGame {
        result: match adjudicate_non_terminal_game(&game) {
            Some(winner_color) => match_result_from_winner(winner_color, candidate_is_white),
            None => MatchResult::Draw,
        },
        final_fen: game.fen(),
        candidate_turns,
    }
}

fn play_profile_duel_trace_with_forced_candidate_move_on_board(
    candidate_profile: &str,
    opponent_profile: &str,
    opponent_mode: SmartAutomovePreference,
    opening_fen: &str,
    candidate_is_white: bool,
    max_plies: usize,
    target_board_fen: &str,
    forced_candidate_move: &[Input],
) -> DuelTraceGame {
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_opportunity_plan_cache();
    clear_turn_engine_plan_cache();
    clear_turn_engine_diagnostics();
    clear_turn_engine_selector_diagnostics();

    let mut candidate_turns = Vec::new();
    let mut forced_move_used = false;
    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return DuelTraceGame {
                result: match_result_from_winner(winner_color, candidate_is_white),
                final_fen: game.fen(),
                candidate_turns,
            };
        }

        let board_fen = game.fen();
        let (candidate_to_move, mut inputs) = match profile_duel_turn_inputs(
            &game,
            candidate_profile,
            opponent_profile,
            opponent_mode,
            candidate_is_white,
        ) {
            Ok(turn) => turn,
            Err(result) => {
                return DuelTraceGame {
                    result,
                    final_fen: game.fen(),
                    candidate_turns,
                };
            }
        };
        if candidate_to_move
            && !forced_move_used
            && board_fen == target_board_fen
        {
            inputs = forced_candidate_move.to_vec();
            forced_move_used = true;
        }
        if candidate_to_move {
            candidate_turns.push(DuelTraceTurn {
                ply,
                board_fen,
                move_fen: Input::fen_from_array(&inputs),
            });
        }

        if inputs.is_empty() {
            return DuelTraceGame {
                result: if candidate_to_move {
                    MatchResult::OpponentWin
                } else {
                    MatchResult::CandidateWin
                },
                final_fen: game.fen(),
                candidate_turns,
            };
        }
        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return DuelTraceGame {
                result: if candidate_to_move {
                    MatchResult::OpponentWin
                } else {
                    MatchResult::CandidateWin
                },
                final_fen: game.fen(),
                candidate_turns,
            };
        }
    }

    DuelTraceGame {
        result: match adjudicate_non_terminal_game(&game) {
            Some(winner_color) => match_result_from_winner(winner_color, candidate_is_white),
            None => MatchResult::Draw,
        },
        final_fen: game.fen(),
        candidate_turns,
    }
}

fn first_duel_trace_divergence(
    candidate: &DuelTraceGame,
    baseline: &DuelTraceGame,
) -> Option<FirstDivergence> {
    candidate
        .candidate_turns
        .iter()
        .zip(baseline.candidate_turns.iter())
        .find_map(|(candidate_turn, baseline_turn)| {
            if candidate_turn.board_fen == baseline_turn.board_fen
                && candidate_turn.move_fen != baseline_turn.move_fen
            {
                Some(FirstDivergence {
                    ply: candidate_turn.ply,
                    board_fen: candidate_turn.board_fen.clone(),
                    candidate_move_fen: candidate_turn.move_fen.clone(),
                    baseline_move_fen: baseline_turn.move_fen.clone(),
                })
            } else {
                None
            }
        })
}

fn match_result_points(result: MatchResult) -> i32 {
    match result {
        MatchResult::CandidateWin => 2,
        MatchResult::Draw => 1,
        MatchResult::OpponentWin => 0,
    }
}

fn format_match_result(result: MatchResult) -> &'static str {
    match result {
        MatchResult::CandidateWin => "win",
        MatchResult::OpponentWin => "loss",
        MatchResult::Draw => "draw",
    }
}

fn profile_runtime_scored_roots_with_forced_engine_inputs(
    profile_name: &str,
    mode: SmartAutomovePreference,
    game: &MonsGame,
) -> (
    SmartSearchConfig,
    Vec<RootEvaluation>,
    Option<TurnPlan>,
    Option<Vec<Input>>,
) {
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
    let advisor_decision = MonsGameModel::pro_v2_root_advisor_presearch(
        game,
        perspective,
        config,
        &mut root_moves,
        engine_plan.as_ref(),
    );
    let advisor_priority_inputs = advisor_decision
        .as_ref()
        .map(MonsGameModel::pro_v2_root_advisor_priority_inputs)
        .unwrap_or_default();
    let forced_engine_inputs = advisor_decision.as_ref().and_then(|decision| {
        decision
            .injected_root
            .as_ref()
            .filter(|root| root.admitted)
            .map(|root| root.inputs.clone())
    });
    let (root_moves, scout_visited_nodes) =
        MonsGameModel::focused_root_candidates_with_priority_inputs(
            game,
            perspective,
            root_moves,
            config,
            true,
            (!advisor_priority_inputs.is_empty()).then_some(advisor_priority_inputs.as_slice()),
            None,
        );
    let mut visited_nodes = scout_visited_nodes;
    let mut alpha = i32::MIN;
    let mut scored_roots = Vec::with_capacity(root_moves.len());
    let mut transposition_table = U64HashMap::default();
    let extension_node_budget = if config.enable_selective_extensions
        && config.selective_extension_node_share_bp > 0
    {
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
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
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
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
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
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
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
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
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
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l1,5;l3,3;l2,2"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_turn_three_full_resources_root() {
    let fixture = primary_pro_fixture_by_id("primary_white_mana_sibling_ply9");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &fixture.game
        ),
        "l5,0;l4,1"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_keeps_v30_white_turn_three_mana_only_vulnerable_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn02Y0xn03",
        false,
    )
    .expect("white turn-three mana-only vulnerable fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l8,4;l7,3"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_keeps_v30_white_turn_three_mana_only_non_vulnerable_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n05S0xn05/n03E0xA0xD0xn02Y0xn02",
        false,
    )
    .expect("white turn-three mana-only non-vulnerable fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l10,8;l9,7"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_v30_white_opening_spirit_sibling_pro_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n02E0xn01A0xD0xS0xY0xn03",
        false,
    )
    .expect("white opening spirit sibling pro fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    ),
        "l10,6;l9,6"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_v30_white_turn_four_mana_sibling_normal_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 4 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMY0xn03/n05S0xn05/n04A0xD0xn05/n02E0xn08",
        false,
    )
    .expect("white turn-four mana sibling normal fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l7,7;l6,6"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_v30_white_turn_four_mana_sibling_fast_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 4 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn03Y0xn03/n03E0xn01S0xn05/n04A0xD0xn05",
        false,
    )
    .expect("white turn-four mana sibling fast fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l10,4;l9,4"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_v30_black_plain_spirit_sibling_full_reliability_pro_root(
) {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 1 0 0 2 n03y0xs0xd0xn01e0xn03/n05a0xn05/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05S0xn05/n03A0xn07/n02E0xn02D0xn02Y0xn02",
        false,
    )
    .expect("full reliability pro black plain spirit sibling fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l0,4;l1,4"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_v30_black_late_progress_over_non_concrete_window_fast_root(
) {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 1 0 0 4 n06a0xn04/n05s0xd0xe0xn03/n07xxmn03/n02y0xxxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n11/n02E0xA0xn01S0xn01Y0xn03/D0xn10",
        false,
    )
    .expect("late black fast non-concrete window fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l3,2;l4,1"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_rejects_v30_white_recovery_head_full_reliability_normal_root(
) {
    let game = MonsGame::from_fen(
        "1 0 w 0 0 0 0 0 7 n11/n06a0xn01e0xn02/n05d0mn05/n03xxmxxmn02xxmn03/n05xxmxxUn04/y0xn03xxMn01s0xn03xxQ/n06Y0xxxMn03/n03xxMn07/n05S0xxxMn04/n04A0xn06/D0xn01E0xn08",
        false,
    )
    .expect("full reliability normal white recovery-head fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l8,5;l7,3;l8,2"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_does_not_seed_cached_plain_spirit_continuation_when_head_is_rejected(
) {
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
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
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
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
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
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
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
#[ignore = "diagnostic: replay exact pro-reliability duel seeds against runtime_current and log first regression divergence"]
fn smart_automove_pro_reliability_duel_trace_probe() {
    use std::collections::BTreeMap;

    #[derive(Clone)]
    struct DuelSpec {
        label: &'static str,
        opponent_mode: SmartAutomovePreference,
        seed_tag: String,
    }

    let candidate_profile = env_profile_name("SMART_PRO_RELIABILITY_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".to_string());
    let baseline_profile = env_profile_name("SMART_PRO_RELIABILITY_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".to_string());
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
    let seed_tag = env_profile_name("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());
    let duel_specs = vec![
        DuelSpec {
            label: "vs_current_pro",
            opponent_mode: SmartAutomovePreference::Pro,
            seed_tag: seed_tag.clone(),
        },
        DuelSpec {
            label: "vs_current_normal",
            opponent_mode: SmartAutomovePreference::Normal,
            seed_tag: format!("{}_vs_normal", seed_tag),
        },
        DuelSpec {
            label: "vs_current_fast",
            opponent_mode: SmartAutomovePreference::Fast,
            seed_tag: format!("{}_vs_fast", seed_tag),
        },
    ];

    with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
        println!(
            "pro reliability duel trace probe: candidate={} baseline={} repeats={} games_per_repeat={} max_plies={} trace_limit={}",
            candidate_profile,
            baseline_profile,
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
                    for candidate_is_white in [true, false] {
                        total_games += 1;
                        let candidate_trace = play_profile_duel_trace(
                            candidate_profile.as_str(),
                            baseline_profile.as_str(),
                            duel.opponent_mode,
                            opening_fen.as_str(),
                            candidate_is_white,
                            max_plies,
                        );
                        let baseline_trace = play_profile_duel_trace(
                            baseline_profile.as_str(),
                            baseline_profile.as_str(),
                            duel.opponent_mode,
                            opening_fen.as_str(),
                            candidate_is_white,
                            max_plies,
                        );
                        let delta = match_result_points(candidate_trace.result)
                            - match_result_points(baseline_trace.result);
                        if delta < 0 {
                            regressions += 1;
                            let first_divergence =
                                first_duel_trace_divergence(&candidate_trace, &baseline_trace);
                            if let Some(divergence) = first_divergence.as_ref() {
                                *move_pair_counts
                                    .entry((
                                        divergence.candidate_move_fen.clone(),
                                        divergence.baseline_move_fen.clone(),
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
                                    let candidate_probe = runtime_decision_probe(
                                        candidate_profile.as_str(),
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    let baseline_probe = runtime_decision_probe(
                                        baseline_profile.as_str(),
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    format!(
                                        "first_diff_ply={} board={} candidate_move={} baseline_move={} candidate(selected={} rank={:?} pre_accept={} pre_rank={:?} stage={} head={:?} head_rank={:?} accepted={} top={:?} selected_root=\"{}\" head_root=\"{}\") baseline(selected={} rank={:?} pre_accept={} pre_rank={:?} stage={} head={:?} head_rank={:?} accepted={} top={:?} selected_root=\"{}\" head_root=\"{}\")",
                                        divergence.ply,
                                        divergence.board_fen,
                                        divergence.candidate_move_fen,
                                        divergence.baseline_move_fen,
                                        candidate_probe.selected_input_fen,
                                        candidate_probe.selected_rank,
                                        candidate_probe.pre_accept_input_fen,
                                        candidate_probe.pre_accept_rank,
                                        candidate_probe.selector_last_stage,
                                        candidate_probe.head_input_fen,
                                        candidate_probe.head_rank,
                                        candidate_probe.head_accepted,
                                        candidate_probe.top_root_fens,
                                        candidate_probe.selected_root,
                                        candidate_probe.head_root,
                                        baseline_probe.selected_input_fen,
                                        baseline_probe.selected_rank,
                                        baseline_probe.pre_accept_input_fen,
                                        baseline_probe.pre_accept_rank,
                                        baseline_probe.selector_last_stage,
                                        baseline_probe.head_input_fen,
                                        baseline_probe.head_rank,
                                        baseline_probe.head_accepted,
                                        baseline_probe.top_root_fens,
                                        baseline_probe.selected_root,
                                        baseline_probe.head_root,
                                    )
                                });

                                println!(
                                    "PRO_RELIABILITY_TRACE duel={} repeat={} opening_index={} candidate_is_white={} opening={} candidate_result={} baseline_result={} candidate_final={} baseline_final={} {}",
                                    duel.label,
                                    repeat_index,
                                    game_index,
                                    candidate_is_white,
                                    opening_fen,
                                    format_match_result(candidate_trace.result),
                                    format_match_result(baseline_trace.result),
                                    candidate_trace.final_fen,
                                    baseline_trace.final_fen,
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
#[ignore = "diagnostic: replay exact pro-reliability duel seeds and log candidate non-win openings"]
fn smart_automove_pro_reliability_nonwin_trace_probe() {
    let candidate_profile = env_profile_name("SMART_PRO_RELIABILITY_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".to_string());
    let baseline_profile = env_profile_name("SMART_PRO_RELIABILITY_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".to_string());
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
    let seed_tag = env_profile_name("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());
    let duel_filter = env::var("SMART_PRO_RELIABILITY_DUEL_FILTER").ok();
    let duel_specs = vec![
        (
            "vs_current_pro",
            SmartAutomovePreference::Pro,
            seed_tag.clone(),
        ),
        (
            "vs_current_normal",
            SmartAutomovePreference::Normal,
            format!("{}_vs_normal", seed_tag),
        ),
        (
            "vs_current_fast",
            SmartAutomovePreference::Fast,
            format!("{}_vs_fast", seed_tag),
        ),
    ];

    with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
        println!(
            "pro reliability non-win trace probe: candidate={} baseline={} repeats={} games_per_repeat={} max_plies={} trace_limit={} duel_filter={:?}",
            candidate_profile,
            baseline_profile,
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
                    for candidate_is_white in [true, false] {
                        let candidate_trace = play_profile_duel_trace(
                            candidate_profile.as_str(),
                            baseline_profile.as_str(),
                            opponent_mode,
                            opening_fen.as_str(),
                            candidate_is_white,
                            max_plies,
                        );
                        if !matches!(candidate_trace.result, MatchResult::CandidateWin) {
                            nonwins += 1;
                            if printed < trace_limit {
                                let baseline_trace = play_profile_duel_trace(
                                    baseline_profile.as_str(),
                                    baseline_profile.as_str(),
                                    opponent_mode,
                                    opening_fen.as_str(),
                                    candidate_is_white,
                                    max_plies,
                                );
                                let detail = first_duel_trace_divergence(
                                    &candidate_trace,
                                    &baseline_trace,
                                )
                                .map(|divergence| {
                                    let board = MonsGame::from_fen(
                                        divergence.board_fen.as_str(),
                                        false,
                                    )
                                    .expect("trace board fen should be valid");
                                    let candidate_probe = runtime_decision_probe(
                                        candidate_profile.as_str(),
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    let baseline_probe = runtime_decision_probe(
                                        baseline_profile.as_str(),
                                        SmartAutomovePreference::Pro,
                                        &board,
                                    );
                                    format!(
                                        "first_diff_ply={} board={} candidate_move={} baseline_move={} candidate(selected={} pre_accept={} stage={} head={:?} accepted={} top={:?}) baseline(selected={} pre_accept={} stage={} head={:?} accepted={} top={:?})",
                                        divergence.ply,
                                        divergence.board_fen,
                                        divergence.candidate_move_fen,
                                        divergence.baseline_move_fen,
                                        candidate_probe.selected_input_fen,
                                        candidate_probe.pre_accept_input_fen,
                                        candidate_probe.selector_last_stage,
                                        candidate_probe.head_input_fen,
                                        candidate_probe.head_accepted,
                                        candidate_probe.top_root_fens,
                                        baseline_probe.selected_input_fen,
                                        baseline_probe.pre_accept_input_fen,
                                        baseline_probe.selector_last_stage,
                                        baseline_probe.head_input_fen,
                                        baseline_probe.head_accepted,
                                        baseline_probe.top_root_fens,
                                    )
                                })
                                .unwrap_or_else(|| "first_diff=none".to_string());

                                println!(
                                    "PRO_RELIABILITY_NONWIN duel={} repeat={} opening_index={} candidate_is_white={} opening={} candidate_result={} baseline_result={} candidate_final={} baseline_final={} {}",
                                    duel_label,
                                    repeat_index,
                                    game_index,
                                    candidate_is_white,
                                    opening_fen,
                                    format_match_result(candidate_trace.result),
                                    format_match_result(baseline_trace.result),
                                    candidate_trace.final_fen,
                                    baseline_trace.final_fen,
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
#[ignore = "diagnostic: inspect advisor approval on confirm-sampled normal non-win boards"]
fn smart_automove_pro_reliability_confirm_normal_board_probe() {
    fn print_case(label: &str, fen: &str) {
        let game = MonsGame::from_fen(fen, false).expect("confirm normal probe fen should be valid");
        clear_turn_engine_selector_diagnostics();
        clear_turn_engine_plan_cache();
        clear_exact_state_analysis_cache();
        let probe = runtime_decision_probe(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let (_, scored_roots, head_plan, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let advisor = pro_v2_root_advisor_decision_snapshot();
        let current_selected = profile_decision_move_fen(
            "runtime_current",
            SmartAutomovePreference::Pro,
            &game,
        );
        let current_root = format_root_probe(
            scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == current_selected),
        );
        let pre_accept_root = format_root_probe(
            scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == probe.pre_accept_input_fen),
        );
        let head_root = probe.head_input_fen.as_ref().map_or_else(
            || "none".to_string(),
            |head| {
                format_root_probe(
                    scored_roots
                        .iter()
                        .find(|root| Input::fen_from_array(&root.inputs) == *head),
                )
            },
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
        let head_plan_probe = head_plan.as_ref().map_or_else(
            || "none".to_string(),
            |plan| {
                format!(
                    "head={:?} goal={:?} utility={:?} head_utility={:?} first_chunk={:?}",
                    plan.head_family,
                    plan.goal_family,
                    plan.utility,
                    plan.head_utility,
                    plan.compiled_chunks
                        .first()
                        .map(|chunk| Input::fen_from_array(chunk)),
                )
            },
        );
        println!(
            "CONFIRM_NORMAL_BOARD label={} current_selected={} current_root=\"{}\" pre_accept_root=\"{}\" head_root=\"{}\" head_plan={} context={} top_root_details={:?} probe={:?} advisor={:?}",
            label,
            current_selected,
            current_root,
            pre_accept_root,
            head_root,
            head_plan_probe,
            exact_opportunity_context_probe(&game),
            top_root_details,
            probe,
            advisor,
        );
    }

    for (label, fen) in [
        (
            "confirm_normal_white_v0_opening3_ply10",
            "0 0 w 1 0 3 0 0 3 n03y0xn03e0xn03/n06a0xn04/n02xxms0xn01d0xn05/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMY0xn02/n02xxMn01xxMn01xxMn04/n04S0xn06/n02E0xn01A0xn06/n05D0xn05",
        ),
        (
            "confirm_normal_black_v0_opening3_ply32",
            "1 0 b 0 0 1 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n02y0xn01xxmxxmn05/n03xxmn03xxmn03/xxQn04xxUn04xxQ/n03xxMxxMn02xxMn03/n06xxMn04/n06S0xn04/n04E0xA0xn05/D0xn06Y0xn03",
        ),
        (
            "confirm_normal_white_v3_opening0_ply13",
            "0 0 w 0 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05D0xn05/n04E0xS0xn05/n02A0xn04Y0xn03",
        ),
        (
            "confirm_normal_white_v3_opening1_ply12",
            "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn02xxMn03/n05D0xn05/n03A0xn01S0xY0xn04/n03E0xn07",
        ),
    ] {
        print_case(label, fen);
    }
}

#[test]
#[ignore = "diagnostic: inspect advisor approval on confirm normal non-win replay boards"]
fn smart_automove_pro_reliability_confirm_normal_nonwin_board_probe() {
    fn print_case(label: &str, fen: &str) {
        let game = MonsGame::from_fen(fen, false).expect("confirm normal replay fen should be valid");
        clear_turn_engine_selector_diagnostics();
        clear_turn_engine_plan_cache();
        clear_exact_state_analysis_cache();
        let probe = runtime_decision_probe(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let (_, scored_roots, head_plan, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let advisor = pro_v2_root_advisor_decision_snapshot();
        let current_selected = profile_decision_move_fen(
            "runtime_current",
            SmartAutomovePreference::Pro,
            &game,
        );
        let current_root = format_root_probe(
            scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == current_selected),
        );
        let pre_accept_root = format_root_probe(
            scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == probe.pre_accept_input_fen),
        );
        let head_root = probe.head_input_fen.as_ref().map_or_else(
            || "none".to_string(),
            |head| {
                format_root_probe(
                    scored_roots
                        .iter()
                        .find(|root| Input::fen_from_array(&root.inputs) == *head),
                )
            },
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
        let head_plan_probe = head_plan.as_ref().map_or_else(
            || "none".to_string(),
            |plan| {
                format!(
                    "head={:?} goal={:?} utility={:?} head_utility={:?} first_chunk={:?}",
                    plan.head_family,
                    plan.goal_family,
                    plan.utility,
                    plan.head_utility,
                    plan.compiled_chunks
                        .first()
                        .map(|chunk| Input::fen_from_array(chunk)),
                )
            },
        );
        println!(
            "CONFIRM_NORMAL_NONWIN_BOARD label={} current_selected={} current_root=\"{}\" pre_accept_root=\"{}\" head_root=\"{}\" head_plan={} context={} top_root_details={:?} probe={:?} advisor={:?}",
            label,
            current_selected,
            current_root,
            pre_accept_root,
            head_root,
            head_plan_probe,
            exact_opportunity_context_probe(&game),
            top_root_details,
            probe,
            advisor,
        );
    }

    for (label, fen) in [
        (
            "confirm_normal_nonwin_white_opening3_ply49",
            "1 1 w 0 0 0 0 0 9 n11/n02y0xn01s0xn01a0xn04/n02xxmn04d0xn03/n06xxmn04/n04xxmn02xxmn03/xxQn04xxUn02Y0xn02/n04xxMn06/n05xxMn05/n02xxMn01S0xn03xxMn01e0x/n11/n02E0xn01A0xD0xn05",
        ),
        (
            "confirm_normal_nonwin_black_opening3_ply32_head",
            "0 1 b 1 0 3 0 0 6 n10d0x/n06a0xn04/n04s0xn01e0xn04/n02y0xxxmn07/n03xxmn02xxmxxmn03/n10xxQ/n03xxMn02xxUn04/n03xxMxxMS0xY0xxxMn03/n05D0xn01xxMn03/n04A0xn06/n03E2xn07",
        ),
        (
            "confirm_normal_nonwin_white_opening2_ply14",
            "0 0 w 1 0 5 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn02xxMn03/n03E0xA0xD0xn05/n05S0xn01Y0xn03/n11",
        ),
        (
            "confirm_normal_nonwin_white_opening2_ply11",
            "0 0 w 1 0 0 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n06S0xn04/n04E0xD0xn01Y0xn03/n04A0xn06",
        ),
        (
            "confirm_normal_nonwin_black_opening2_ply46",
            "1 1 b 0 0 0 0 0 8 E0xn02y0xn01d1xn05/n05s0xa0xe0xn03/n03xxmn03xxmn03/n11/n03xxmn03xxmn03/n05xxUn04xxQ/n03xxMxxMn02xxMn03/n11/n04A0xn01S0xn04/n05D0xxxMn01Y0xn02/n11",
        ),
        (
            "confirm_normal_nonwin_white_opening0_ply13_bridge_head",
            "0 0 w 0 0 2 0 0 3 n11/n03y0xn01s0xa0xe0xn03/n05d0xn05/n03xxmxxmn01xxmn04/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn01Y0xn01/n04xxMn01xxMn04/n11/n04E0xD0xS0xn04/n04A0xn06",
        ),
        (
            "confirm_normal_nonwin_white_opening1_ply16",
            "0 0 w 1 0 5 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMY0xn01xxMn03/n05D0xn05/n03E0xA0xS0xn05/n11",
        ),
        (
            "confirm_normal_nonwin_black_opening1_ply75",
            "2 1 b 0 0 0 0 0 12 n04s1xd0xn05/n02xxmn03a0xn04/n06xxmn04/n08xxmn02/n02E0xn02xxme0xn04/n10Y0x/n04xxUxxMn05/n05xxMn05/n04D0xn03xxMn02/n02y0xA0xn07/n06S1xn04",
        ),
        (
            "confirm_normal_nonwin_white_opening3_ply11",
            "0 0 w 0 0 1 0 0 3 n07e0xn03/n02y0xn02s0xn01a0xn03/n06d0xxxmn03/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05D0xn05/n03E0xn01S0xn05/n04A0xn02Y0xn03",
        ),
    ] {
        print_case(label, fen);
    }
}

#[test]
#[ignore = "diagnostic: force top-root alternatives on current flat non-win boards"]
fn smart_automove_pro_flat_nonwin_alternative_root_probe() {
    #[derive(Clone)]
    struct ForcedRootCase {
        label: &'static str,
        board_fen: &'static str,
        opponent_mode: SmartAutomovePreference,
        candidate_is_white: bool,
        forced_moves: &'static [&'static str],
    }

    let cases = [
        ForcedRootCase {
            label: "vs_current_pro_loss_opening_a",
            board_fen:
                "0 0 w 0 0 1 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n02E0xn01A0xD0xS0xY0xn03",
            opponent_mode: SmartAutomovePreference::Pro,
            candidate_is_white: true,
            forced_moves: &[
                "l10,5;l9,4",
                "l10,6;l9,5",
                "l10,6;l9,6",
                "l10,6;l9,7",
                "l10,5;l9,5",
            ],
        },
        ForcedRootCase {
            label: "vs_current_pro_flat_white_nonwin_v1",
            board_fen:
                "0 0 w 0 0 1 0 0 3 n03y0xn03e0xn03/n05s0xa0xn01d0mn02/n11/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n01E0xn05Y0xn03/n04D0xn01S0xn04/n04A0xn06",
            opponent_mode: SmartAutomovePreference::Pro,
            candidate_is_white: true,
            forced_moves: &["l9,6;l7,4;l8,3", "l8,7;l7,8"],
        },
        ForcedRootCase {
            label: "vs_current_normal_flat_white_nonwin_v1",
            board_fen:
                "1 0 w 1 0 1 0 0 5 n11/n05a0xn02e0xn02/n03y0xd0ms0xn05/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n11/n03xxMn01A0xn05/n01D0xn04Y0xS0xn03/n03E0xn07",
            opponent_mode: SmartAutomovePreference::Normal,
            candidate_is_white: true,
            forced_moves: &["l9,1;l8,2", "l9,7;l8,7", "l9,1;l9,2", "l9,7;l8,6"],
        },
        ForcedRootCase {
            label: "vs_current_fast_flat_black_nonwin_v1",
            board_fen:
                "0 0 b 0 0 5 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/n05xxUn04xxQ/n02y0xxxMn01xxMn01xxMn03/n04xxMn06/n03E0xA0xn03xxMn02/n06S0xn04/n05D2xn03Y0xn01",
            opponent_mode: SmartAutomovePreference::Fast,
            candidate_is_white: false,
            forced_moves: &[
                "l1,5;l3,3;l2,2",
                "l1,5;l2,7;l1,8",
                "l1,5;l2,7;l2,8",
                "l1,5;l1,7;l0,7",
            ],
        },
    ];

    let candidate_profile = "runtime_pro_turn_engine_v30";
    let baseline_profile = "runtime_current";
    let max_plies = 96usize;

    with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
        println!(
            "flat non-win alternative root probe: candidate={} baseline={} cases={}",
            candidate_profile,
            baseline_profile,
            cases.len()
        );
        for case in cases {
            for forced_move in case.forced_moves {
                let trace = play_profile_duel_trace_with_forced_candidate_first_move(
                    candidate_profile,
                    baseline_profile,
                    case.opponent_mode,
                    case.board_fen,
                    case.candidate_is_white,
                    max_plies,
                    Input::array_from_fen(forced_move).as_slice(),
                );
                println!(
                    "FLAT_NONWIN_ALT_ROOT label={} opponent_mode={:?} candidate_is_white={} forced_move={} result={} final={}",
                    case.label,
                    case.opponent_mode,
                    case.candidate_is_white,
                    forced_move,
                    format_match_result(trace.result),
                    trace.final_fen,
                );
            }
        }
    });
}

#[test]
#[ignore = "diagnostic: print candidate turn trace for the sampled flat fast opening loss"]
fn smart_automove_pro_fast_flat_opening_loss_trace_probe() {
    let opening_fen =
        "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03";
    let trace = play_profile_duel_trace(
        "runtime_pro_turn_engine_v30",
        "runtime_current",
        SmartAutomovePreference::Fast,
        opening_fen,
        false,
        96,
    );

    println!(
        "FAST_FLAT_OPENING_LOSS_TRACE result={} final={} candidate_turns={}",
        format_match_result(trace.result),
        trace.final_fen,
        trace.candidate_turns.len(),
    );
    for turn in trace.candidate_turns {
        println!(
            "FAST_FLAT_OPENING_LOSS_TURN ply={} board={} move={}",
            turn.ply, turn.board_fen, turn.move_fen,
        );
    }
}

#[test]
#[ignore = "diagnostic: force top candidate roots on sampled fast flat-loss trace boards"]
fn smart_automove_pro_fast_flat_trace_top_root_probe() {
    #[derive(Clone)]
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
        candidate_is_white: bool,
        top_n: usize,
    }

    let cases = [
        ProbeCase {
            label: "fast_flat_trace_ply83",
            board_fen:
                "2 1 b 1 0 5 0 0 12 n05d1xn02xxmn02/n07a0xn03/n01xxmn03s0xn02Y0xn02/n03xxmn07/n05xxmn01e0xn03/n05xxUn05/n05xxMn05/n05S0xn05/n01y0xA0xn04xxMn03/n01D0Mn09/n03E2xn07",
            candidate_is_white: false,
            top_n: 6,
        },
        ProbeCase {
            label: "fast_flat_trace_ply91",
            board_fen:
                "3 1 b 0 0 0 0 0 14 n05d0xn05/n07a0xn01xxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
            candidate_is_white: false,
            top_n: 6,
        },
        ProbeCase {
            label: "fast_flat_trace_ply94",
            board_fen:
                "3 1 b 1 0 2 0 0 14 n11/n07a0xd0xxxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
            candidate_is_white: false,
            top_n: 6,
        },
        ProbeCase {
            label: "fast_flat_trace_ply95",
            board_fen:
                "3 1 b 1 0 3 0 0 14 n08d0xn02/n07a0xn01xxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
            candidate_is_white: false,
            top_n: 6,
        },
        ProbeCase {
            label: "fast_flat_trace_ply95_window_followup",
            board_fen:
                "3 1 b 1 0 3 0 0 14 n11/n07a0xn01d0mn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
            candidate_is_white: false,
            top_n: 6,
        },
    ];

    println!(
        "fast flat trace top-root probe: candidate={} baseline={} cases={}",
        "runtime_pro_turn_engine_v30",
        "runtime_current",
        cases.len(),
    );

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false).expect("valid fast flat trace probe fen");
        let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        for root in scored_roots.iter().take(case.top_n) {
            let forced_move = Input::fen_from_array(&root.inputs);
            let trace = play_profile_duel_trace_with_forced_candidate_first_move(
                "runtime_pro_turn_engine_v30",
                "runtime_current",
                SmartAutomovePreference::Fast,
                case.board_fen,
                case.candidate_is_white,
                96,
                root.inputs.as_slice(),
            );
            println!(
                "FAST_FLAT_TRACE_TOP_ROOT label={} forced_move={} rank={} root=\"{}\" result={} final={}",
                case.label,
                forced_move,
                root.root_rank,
                format_root_probe(Some(root)),
                format_match_result(trace.result),
                trace.final_fen,
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: force warm opening alternatives on current fast flat-loss followup boards"]
fn smart_automove_pro_fast_flat_opening_forced_followup_probe() {
    #[derive(Clone)]
    struct ProbeCase {
        label: &'static str,
        target_board_fen: &'static str,
        top_n: usize,
    }

    let opening_fen =
        "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03";
    let cases = [
        ProbeCase {
            label: "fast_flat_opening_force_ply49",
            target_board_fen:
                "1 0 b 0 0 0 0 0 8 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
            top_n: 6,
        },
        ProbeCase {
            label: "fast_flat_opening_force_ply63",
            target_board_fen:
                "1 0 b 0 0 0 0 0 10 n09xxmn01/n06a0xn04/n05s0xd0xe0xn03/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n05S0xn05/n02A0xn04xxMn03/n01D0Mn07xxMn01/n03E0xn07",
            top_n: 6,
        },
        ProbeCase {
            label: "fast_flat_opening_force_ply83",
            target_board_fen:
                "2 1 b 1 0 5 0 0 12 n05d1xn02xxmn02/n07a0xn03/n01xxmn03s0xn02Y0xn02/n03xxmn07/n05xxmn01e0xn03/n05xxUn05/n05xxMn05/n05S0xn05/n01y0xA0xn04xxMn03/n01D0Mn09/n03E2xn07",
            top_n: 6,
        },
        ProbeCase {
            label: "fast_flat_opening_force_ply91",
            target_board_fen:
                "3 1 b 0 0 0 0 0 14 n05d0xn05/n07a0xn01xxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
            top_n: 6,
        },
        ProbeCase {
            label: "fast_flat_opening_force_ply94",
            target_board_fen:
                "3 1 b 1 0 2 0 0 14 n11/n07a0xd0xxxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
            top_n: 6,
        },
        ProbeCase {
            label: "fast_flat_opening_force_ply95_window_followup",
            target_board_fen:
                "3 1 b 1 0 3 0 0 14 n11/n07a0xn01d0mn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
            top_n: 6,
        },
    ];

    println!(
        "fast flat opening forced-followup probe: candidate={} baseline={} cases={}",
        "runtime_pro_turn_engine_v30",
        "runtime_current",
        cases.len(),
    );

    for case in cases {
        let game = MonsGame::from_fen(case.target_board_fen, false)
            .expect("valid fast flat followup probe fen");
        let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        for root in scored_roots.iter().take(case.top_n) {
            let forced_move = Input::fen_from_array(&root.inputs);
            let trace = play_profile_duel_trace_with_forced_candidate_move_on_board(
                "runtime_pro_turn_engine_v30",
                "runtime_current",
                SmartAutomovePreference::Fast,
                opening_fen,
                false,
                96,
                case.target_board_fen,
                root.inputs.as_slice(),
            );
            println!(
                "FAST_FLAT_OPENING_FORCED_FOLLOWUP label={} forced_move={} rank={} root=\"{}\" result={} final={}",
                case.label,
                forced_move,
                root.root_rank,
                format_root_probe(Some(root)),
                format_match_result(trace.result),
                trace.final_fen,
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: search the sampled fast flat opening trace for a warm rescue board"]
fn smart_automove_pro_fast_flat_opening_warm_rescue_probe() {
    let opening_fen =
        "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03";
    let start_ply = env_usize("SMART_FAST_WARM_RESCUE_START_PLY").unwrap_or(63);
    let top_n = env_usize("SMART_FAST_WARM_RESCUE_TOP_N").unwrap_or(3).max(1);
    let trace = play_profile_duel_trace(
        "runtime_pro_turn_engine_v30",
        "runtime_current",
        SmartAutomovePreference::Fast,
        opening_fen,
        false,
        96,
    );

    println!(
        "fast flat opening warm-rescue probe: result={} final={} candidate_turns={} start_ply={} top_n={}",
        format_match_result(trace.result),
        trace.final_fen,
        trace.candidate_turns.len(),
        start_ply,
        top_n,
    );

    let mut winning_lines = 0usize;
    for turn in trace
        .candidate_turns
        .iter()
        .filter(|turn| turn.ply >= start_ply)
    {
        let game = MonsGame::from_fen(turn.board_fen.as_str(), false)
            .expect("warm rescue probe board fen should be valid");
        let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        for root in scored_roots.iter().take(top_n) {
            let forced_move = Input::fen_from_array(&root.inputs);
            let forced_trace = play_profile_duel_trace_with_forced_candidate_move_on_board(
                "runtime_pro_turn_engine_v30",
                "runtime_current",
                SmartAutomovePreference::Fast,
                opening_fen,
                false,
                96,
                turn.board_fen.as_str(),
                root.inputs.as_slice(),
            );
            let is_win = matches!(forced_trace.result, MatchResult::CandidateWin);
            if is_win {
                winning_lines += 1;
            }
            println!(
                "FAST_FLAT_OPENING_WARM_RESCUE ply={} board={} forced_move={} rank={} root=\"{}\" result={} final={}",
                turn.ply,
                turn.board_fen,
                forced_move,
                root.root_rank,
                format_root_probe(Some(root)),
                format_match_result(forced_trace.result),
                forced_trace.final_fen,
            );
        }
    }

    println!(
        "FAST_FLAT_OPENING_WARM_RESCUE_SUMMARY winning_lines={} start_ply={} top_n={}",
        winning_lines,
        start_ply,
        top_n,
    );
}

#[test]
#[ignore = "diagnostic: force rejected turn-engine heads on the sampled fast flat opening"]
fn smart_automove_pro_fast_flat_opening_forced_head_probe() {
    #[derive(Clone)]
    struct ProbeCase {
        label: &'static str,
        target_board_fen: &'static str,
        forced_move: &'static str,
    }

    let opening_fen =
        "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03";
    let cases = [
        ProbeCase {
            label: "fast_flat_opening_head_ply35_spirit",
            target_board_fen:
                "1 0 b 0 0 0 0 0 6 n05d1xa0xn04/n05s0xn01e0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/n05xxMn05/n01y0xn01xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
            forced_move: "l1,5;l1,7;l0,7",
        },
        ProbeCase {
            label: "fast_flat_opening_head_ply41_mana",
            target_board_fen:
                "1 0 b 1 0 5 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn02xxMn03xxMn03/n07xxMn03/n05S0xn05/n03E2xA1xD1xn05",
            forced_move: "l3,7;l2,8",
        },
        ProbeCase {
            label: "fast_flat_opening_head_ply49_safe_progress",
            target_board_fen:
                "1 0 b 0 0 0 0 0 8 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
            forced_move: "l0,5;l1,4",
        },
    ];

    println!(
        "fast flat opening forced-head probe: candidate={} baseline={} cases={}",
        "runtime_pro_turn_engine_v30",
        "runtime_current",
        cases.len(),
    );

    for case in cases {
        let trace = play_profile_duel_trace_with_forced_candidate_move_on_board(
            "runtime_pro_turn_engine_v30",
            "runtime_current",
            SmartAutomovePreference::Fast,
            opening_fen,
            false,
            96,
            case.target_board_fen,
            Input::array_from_fen(case.forced_move).as_slice(),
        );
        println!(
            "FAST_FLAT_OPENING_FORCED_HEAD label={} forced_move={} result={} final={}",
            case.label,
            case.forced_move,
            format_match_result(trace.result),
            trace.final_fen,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect early black runtime selection on the sampled fast flat opening loss"]
fn smart_automove_pro_fast_flat_opening_early_runtime_probe() {
    #[derive(Clone)]
    struct ProbeCase {
        label: &'static str,
        board_fen: &'static str,
    }

    let cases = [
        ProbeCase {
            label: "fast_flat_opening_ply35",
            board_fen:
                "1 0 b 0 0 0 0 0 6 n05d1xa0xn04/n05s0xn01e0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/n05xxMn05/n01y0xn01xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        },
        ProbeCase {
            label: "fast_flat_opening_ply37",
            board_fen:
                "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        },
        ProbeCase {
            label: "fast_flat_opening_ply39",
            board_fen:
                "1 0 b 0 0 4 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        },
        ProbeCase {
            label: "fast_flat_opening_ply41",
            board_fen:
                "1 0 b 1 0 5 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn02xxMn03xxMn03/n07xxMn03/n05S0xn05/n03E2xA1xD1xn05",
        },
        ProbeCase {
            label: "fast_flat_opening_ply49",
            board_fen:
                "1 0 b 0 0 0 0 0 8 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        },
        ProbeCase {
            label: "fast_flat_opening_ply50",
            board_fen:
                "1 0 b 1 0 0 0 0 8 n05d0xn05/n05s0xa0xe0xxxmn02/n11/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        },
        ProbeCase {
            label: "fast_flat_opening_ply51",
            board_fen:
                "1 0 b 1 0 1 0 0 8 n05d0xn05/n06a0xe0xxxmn02/n05s0xn05/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        },
        ProbeCase {
            label: "fast_flat_opening_ply52",
            board_fen:
                "1 0 b 1 0 2 0 0 8 n11/n05d0xa0xe0xxxmn02/n05s0xn05/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        },
        ProbeCase {
            label: "fast_flat_opening_ply53",
            board_fen:
                "1 0 b 1 0 3 0 0 8 n11/n06a0xe0xxxmn02/n05s0xd0xn04/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        },
        ProbeCase {
            label: "fast_flat_opening_ply54",
            board_fen:
                "1 0 b 1 0 4 0 0 8 n11/n06a0xe0xxxmn02/n05s0xd0xn04/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n04S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        },
        ProbeCase {
            label: "fast_flat_opening_ply55",
            board_fen:
                "1 0 b 1 0 5 0 0 8 n11/n06a0xn01xxmn02/n05s0xd0xe0xn03/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n04S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        },
        ProbeCase {
            label: "fast_flat_opening_ply63",
            board_fen:
                "1 0 b 0 0 0 0 0 10 n09xxmn01/n06a0xn04/n05s0xd0xe0xn03/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n05S0xn05/n02A0xn04xxMn03/n01D0Mn07xxMn01/n03E0xn07",
        },
    ];

    println!(
        "fast flat opening early runtime probe: candidate={} baseline={} cases={}",
        "runtime_pro_turn_engine_v30",
        "runtime_current",
        cases.len(),
    );

    for case in cases {
        let game = MonsGame::from_fen(case.board_fen, false)
            .expect("valid fast flat early runtime probe fen");
        let candidate_probe = runtime_decision_probe(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let advisor = pro_v2_root_advisor_decision_snapshot();
        let (_, candidate_scored_roots, head_plan, forced_engine_inputs) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "runtime_pro_turn_engine_v30",
                SmartAutomovePreference::Pro,
                &game,
            );
        let current_probe =
            runtime_decision_probe("runtime_current", SmartAutomovePreference::Pro, &game);
        let ordered_shortlist = advisor
            .as_ref()
            .map(|decision| {
                decision
                    .ordered_shortlist
                    .iter()
                    .map(format_root_advisor_entry_probe)
                    .collect::<Vec<_>>()
                    .join(" | ")
            })
            .unwrap_or_else(|| "none".to_string());
        let preserved = advisor
            .as_ref()
            .map(|decision| {
                decision
                    .preserved_family_representatives
                    .iter()
                    .map(format_root_advisor_entry_probe)
                    .collect::<Vec<_>>()
                    .join(" | ")
            })
            .unwrap_or_else(|| "none".to_string());
        let approved = advisor
            .as_ref()
            .and_then(|decision| decision.approved_root.as_ref())
            .map(format_root_advisor_entry_probe)
            .unwrap_or_else(|| "none".to_string());
        let injected = advisor
            .as_ref()
            .and_then(|decision| decision.injected_root.as_ref())
            .map_or_else(
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
        let current_selected_root = format_root_probe(
            candidate_scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == current_probe.selected_input_fen),
        );
        println!(
            "FAST_FLAT_EARLY_RUNTIME label={} candidate(selected={} rank={:?} pre_accept={} pre_rank={:?} stage={} head={:?} head_rank={:?} accepted={} forced={:?} selected_root=\"{}\" head_root=\"{}\" top={:?}) current(selected={} rank={:?} pre_accept={} pre_rank={:?} stage={} head={:?} head_rank={:?} accepted={} selected_root=\"{}\" top={:?}) advisor(approved={} injected={} shortlist=[{}] preserved=[{}]) fen={}",
            case.label,
            candidate_probe.selected_input_fen,
            candidate_probe.selected_rank,
            candidate_probe.pre_accept_input_fen,
            candidate_probe.pre_accept_rank,
            candidate_probe.selector_last_stage,
            candidate_probe.head_input_fen,
            candidate_probe.head_rank,
            candidate_probe.head_accepted,
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs.as_slice())),
            candidate_probe.selected_root,
            candidate_probe.head_root,
            candidate_probe.top_root_fens,
            current_probe.selected_input_fen,
            current_probe.selected_rank,
            current_probe.pre_accept_input_fen,
            current_probe.pre_accept_rank,
            current_probe.selector_last_stage,
            current_probe.head_input_fen,
            current_probe.head_rank,
            current_probe.head_accepted,
            current_selected_root,
            current_probe.top_root_fens,
            approved,
            injected,
            ordered_shortlist,
            preserved,
            case.board_fen,
        );
        if let Some(plan) = head_plan.as_ref() {
            println!(
                "FAST_FLAT_EARLY_RUNTIME_HEAD label={} head_family={:?} chunks={:?}",
                case.label,
                plan.head_family,
                plan.compiled_chunks,
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: inspect injected spirit head admission on fast flat opening ply35"]
fn smart_automove_pro_fast_flat_opening_ply35_injected_head_gate_probe() {
    fn format_root_move_probe(root: &ScoredRootMove) -> String {
        format!(
            "fen={} rank={} heuristic={} family={:?} win={} attack={} window={} same_turn_setup={} own_setup={} spirit={} supermana_progress={} opp_progress={} pickup_super={} pickup_opp={} vulnerable={} handoff={} roundtrip={}",
            Input::fen_from_array(&root.inputs),
            root.root_rank,
            root.heuristic,
            MonsGameModel::turn_engine_root_move_family(root),
            root.wins_immediately,
            root.attacks_opponent_drainer,
            root.same_turn_score_window_value,
            root.spirit_same_turn_score_setup_now,
            root.spirit_own_mana_setup_now,
            root.spirit_development,
            root.supermana_progress,
            root.opponent_mana_progress,
            root.safe_supermana_pickup_now,
            root.safe_opponent_mana_pickup_now,
            root.own_drainer_vulnerable,
            root.mana_handoff_to_opponent,
            root.has_roundtrip,
        )
    }

    let game = MonsGame::from_fen(
        "1 0 b 0 0 0 0 0 6 n05d1xa0xn04/n05s0xn01e0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/n05xxMn05/n01y0xn01xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        false,
    )
    .expect("valid fast flat ply35 fen");
    let perspective = game.active_color;
    let (config, _, head_plan, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let head_plan = head_plan.expect("fast flat ply35 should retain a head plan");
    let candidate_inputs = head_plan
        .compiled_chunks
        .first()
        .expect("head plan should include a first chunk")
        .clone();
    let root_moves = MonsGameModel::ranked_root_moves(&game, perspective, config);
    let own_drainer_vulnerable_before = if config.enable_move_class_coverage {
        MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        )
    } else {
        false
    };
    let candidate = MonsGameModel::build_scored_root_move(
        &game,
        perspective,
        config,
        own_drainer_vulnerable_before,
        candidate_inputs.as_slice(),
    );
    let override_allowed = candidate.as_ref().is_some_and(|candidate| {
        MonsGameModel::pro_v2_black_turn_six_plain_spirit_head_injection_override(
            &game,
            config,
            root_moves.as_slice(),
            candidate,
            &head_plan,
        )
    });
    let mut injected_root_moves = root_moves.clone();
    let injected_inputs = MonsGameModel::inject_turn_engine_root_candidate(
        &game,
        perspective,
        config,
        &mut injected_root_moves,
        &head_plan,
    );
    let injected_rank = injected_root_moves
        .iter()
        .position(|root| root.inputs == candidate_inputs);
    let top = root_moves
        .first()
        .expect("ranked root list should not be empty");
    println!(
        "FAST_FLAT_PLY35_INJECT head_family={:?} goal_family={:?} forced_head={} candidate_built={} override_allowed={} injected={:?} injected_rank={:?} top_root=\"{}\" candidate_root=\"{}\" root_top={:?} injected_top={:?}",
        head_plan.head_family,
        head_plan.goal_family,
        Input::fen_from_array(&candidate_inputs),
        candidate.is_some(),
        override_allowed,
        injected_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs.as_slice())),
        injected_rank,
        format_root_move_probe(top),
        candidate
            .as_ref()
            .map(format_root_move_probe)
            .unwrap_or_else(|| "none".to_string()),
        root_moves
            .iter()
            .take(8)
            .map(format_root_move_probe)
            .collect::<Vec<_>>(),
        injected_root_moves
            .iter()
            .take(8)
            .map(format_root_move_probe)
            .collect::<Vec<_>>(),
    );
    if let Some(candidate) = candidate.as_ref() {
        let top_utility = MonsGameModel::turn_engine_scored_root_utility(
            &game,
            top,
            perspective,
            config,
            MonsGameModel::turn_engine_root_move_family(top),
        );
        let candidate_utility = MonsGameModel::turn_engine_scored_root_utility(
            &game,
            candidate,
            perspective,
            config,
            MonsGameModel::turn_engine_root_move_family(candidate),
        );
        println!(
            "FAST_FLAT_PLY35_INJECT_UTILITY plan_vs_top={:?} head_vs_top={:?} candidate_vs_top={:?} plan_utility={:?} head_utility={:?} top_utility={:?} candidate_utility={:?}",
            crate::models::automove_turn_engine::compare_utility_primary_axes(
                head_plan.utility,
                top_utility,
            ),
            crate::models::automove_turn_engine::compare_utility_primary_axes(
                head_plan.head_utility,
                top_utility,
            ),
            crate::models::automove_turn_engine::compare_utility_primary_axes(
                candidate_utility,
                top_utility,
            ),
            head_plan.utility,
            head_plan.head_utility,
            top_utility,
            candidate_utility,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect post-search head acceptance on fast flat opening ply35"]
fn smart_automove_pro_fast_flat_opening_ply35_accept_trace_probe() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 0 0 0 6 n05d1xa0xn04/n05s0xn01e0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/n05xxMn05/n01y0xn01xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        false,
    )
    .expect("valid fast flat ply35 fen");
    let perspective = game.active_color;
    let (config, scored_roots, head_plan, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let head_plan = head_plan.expect("fast flat ply35 should retain a head plan");
    let selected_inputs = MonsGameModel::pick_root_move_with_exploration(
        &game,
        scored_roots.as_slice(),
        perspective,
        config,
    );
    let accepted = MonsGameModel::accept_turn_engine_head_after_search(
        &game,
        perspective,
        config,
        scored_roots.as_slice(),
        &selected_inputs,
        &head_plan,
    );
    let candidate_inputs = head_plan
        .compiled_chunks
        .first()
        .expect("head plan should include a first chunk");
    let candidate_index = scored_roots
        .iter()
        .position(|root| root.inputs == *candidate_inputs)
        .expect("candidate should be present in scored roots");
    let selected_index = scored_roots
        .iter()
        .position(|root| root.inputs == selected_inputs)
        .expect("selected should be present in scored roots");
    let candidate = &scored_roots[candidate_index];
    let selected = &scored_roots[selected_index];
    let candidate_family = MonsGameModel::turn_engine_root_evaluation_family(candidate);
    let selected_family = MonsGameModel::turn_engine_root_evaluation_family(selected);
    let candidate_unsafe = MonsGameModel::turn_engine_root_evaluation_is_unsafe(candidate);
    let selected_unsafe = MonsGameModel::turn_engine_root_evaluation_is_unsafe(selected);
    let candidate_progress_surface =
        MonsGameModel::turn_engine_root_evaluation_has_progress_surface(candidate);
    let selected_progress_surface =
        MonsGameModel::turn_engine_root_evaluation_has_progress_surface(selected);
    let candidate_spirit_tactical = candidate.spirit_same_turn_score_setup_now
        || candidate.same_turn_score_window_value > 0
        || candidate.attacks_opponent_drainer
        || candidate.scores_supermana_this_turn
        || candidate.scores_opponent_mana_this_turn
        || candidate.safe_supermana_pickup_now
        || candidate.safe_opponent_mana_pickup_now;
    let score_gap = selected.score.saturating_sub(candidate.score);
    let candidate_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        candidate,
        perspective,
        config,
        candidate_family,
    );
    let selected_utility = MonsGameModel::turn_engine_selected_override_utility(
        &game,
        selected,
        perspective,
        config,
        selected_family,
    );
    let black_turn_six_route_change_plain_spirit_head_override =
        matches!(config.turn_engine_mode, TurnEngineMode::ProV2)
            && game.active_color == Color::Black
            && game.turn_number == 6
            && game.mons_moves_count == 0
            && game.player_can_use_action()
            && game.player_can_move_mana()
            && matches!(head_plan.head_family, TurnPlanFamily::SpiritImpact)
            && matches!(head_plan.goal_family, TurnPlanFamily::SpiritImpact)
            && candidate_index <= 1
            && candidate_unsafe == selected_unsafe
            && candidate.spirit_development
            && !candidate.spirit_same_turn_score_setup_now
            && !candidate.spirit_own_mana_setup_now
            && !candidate_progress_surface
            && !candidate_spirit_tactical
            && matches!(selected_family, TurnPlanFamily::ManaTempo)
            && selected_unsafe
            && !selected.spirit_development
            && !selected.spirit_same_turn_score_setup_now
            && !selected.spirit_own_mana_setup_now
            && !selected_progress_surface
            && !selected.wins_immediately
            && !selected.attacks_opponent_drainer
            && !selected.scores_supermana_this_turn
            && !selected.scores_opponent_mana_this_turn
            && !selected.safe_supermana_pickup_now
            && !selected.safe_opponent_mana_pickup_now
            && selected.same_turn_score_window_value == 0
            && candidate.inputs.first() != selected.inputs.first()
            && candidate.own_drainer_vulnerable == selected.own_drainer_vulnerable
            && candidate.own_drainer_walk_vulnerable == selected.own_drainer_walk_vulnerable
            && candidate.mana_handoff_to_opponent == selected.mana_handoff_to_opponent
            && candidate.has_roundtrip == selected.has_roundtrip
            && score_gap <= 1024;
    let plan_dominates_selected =
        crate::models::automove_turn_engine::compare_utility_primary_axes(
            head_plan.utility,
            selected_utility,
        ) == std::cmp::Ordering::Greater
            && (head_plan
                .utility
                .strictly_dominates_override_axes(selected_utility)
                || head_plan
                    .head_utility
                    .strictly_dominates_override_axes(selected_utility));
    let candidate_dominates_selected =
        crate::models::automove_turn_engine::compare_utility_primary_axes(
            candidate_utility,
            selected_utility,
        ) == std::cmp::Ordering::Greater
            && candidate_utility.strictly_dominates_override_axes(selected_utility);
    println!(
        "FAST_FLAT_PLY35_ACCEPT accepted={} selected={} selected_rank={} candidate={} candidate_rank={} head_family={:?} goal_family={:?} candidate_family={:?} selected_family={:?} score_gap={} candidate_unsafe={} selected_unsafe={} candidate_progress={} selected_progress={} candidate_tactical={} selected_vulnerable={} candidate_vulnerable={} selected_walk_vulnerable={} candidate_walk_vulnerable={} same_handoff={} same_roundtrip={} same_first_input={} plan_vs_selected={:?} head_vs_selected={:?} candidate_vs_selected={:?} plan_dominates_selected={} candidate_dominates_selected={} route_change_override={} plan_utility={:?} head_utility={:?} selected_utility={:?} candidate_utility={:?}",
        accepted,
        Input::fen_from_array(&selected.inputs),
        selected_index,
        Input::fen_from_array(&candidate.inputs),
        candidate_index,
        head_plan.head_family,
        head_plan.goal_family,
        candidate_family,
        selected_family,
        score_gap,
        candidate_unsafe,
        selected_unsafe,
        candidate_progress_surface,
        selected_progress_surface,
        candidate_spirit_tactical,
        selected.own_drainer_vulnerable,
        candidate.own_drainer_vulnerable,
        selected.own_drainer_walk_vulnerable,
        candidate.own_drainer_walk_vulnerable,
        candidate.mana_handoff_to_opponent == selected.mana_handoff_to_opponent,
        candidate.has_roundtrip == selected.has_roundtrip,
        candidate.inputs.first() == selected.inputs.first(),
        crate::models::automove_turn_engine::compare_utility_primary_axes(
            head_plan.utility,
            selected_utility,
        ),
        crate::models::automove_turn_engine::compare_utility_primary_axes(
            head_plan.head_utility,
            selected_utility,
        ),
        crate::models::automove_turn_engine::compare_utility_primary_axes(
            candidate_utility,
            selected_utility,
        ),
        plan_dominates_selected,
        candidate_dominates_selected,
        black_turn_six_route_change_plain_spirit_head_override,
        head_plan.utility,
        head_plan.head_utility,
        selected_utility,
        candidate_utility,
    );
}

#[test]
fn runtime_pro_turn_engine_v30_prefers_spirit_reentry_on_fast_flat_opening_ply37_root() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        false,
    )
    .expect("valid fast flat ply37 fen");

    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot()
        .expect("advisor snapshot should exist for fast flat ply37");

    assert_eq!(probe.selected_input_fen, "l1,5;l3,3;l2,3");
    assert_eq!(probe.pre_accept_input_fen, "l1,5;l3,3;l2,3");
    assert_eq!(
        advisor
            .approved_root
            .as_ref()
            .map(|entry| Input::fen_from_array(&entry.inputs)),
        Some("l1,5;l3,3;l2,3".to_string())
    );
}

#[test]
#[ignore = "diagnostic: bounded selector/exact hotspot probe for pro reliability corpus"]
fn smart_automove_pro_reliability_hotspot_probe() {
    use std::collections::{BTreeMap, HashMap};
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
        candidate: &BTreeMap<&'static str, u64>,
        baseline: &BTreeMap<&'static str, u64>,
    ) -> String {
        candidate
            .iter()
            .map(|(label, candidate_value)| {
                let baseline_value = baseline.get(label).copied().unwrap_or_default();
                let delta = *candidate_value as i64 - baseline_value as i64;
                format!("{label}={candidate_value}/{baseline_value}({delta:+})")
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    let candidate_profile = env_profile_name("SMART_PROBE_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".to_string());
    let baseline_profile = env_profile_name("SMART_PROBE_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".to_string());

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
        "pro reliability hotspot probe: candidate={} baseline={} positions={}",
        candidate_profile,
        baseline_profile,
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
                let candidate = run_probe_for_profile(candidate_profile.as_str(), &case);
                let baseline = run_probe_for_profile(baseline_profile.as_str(), &case);

                println!(
                    "HOTSPOT label={} changed={} candidate_move={} baseline_move={} ms={:.2}/{:.2} selector(last_stage={}/{}) exact={} engine={} fen={}",
                    case.label,
                    candidate.move_fen != baseline.move_fen,
                    candidate.move_fen,
                    baseline.move_fen,
                    candidate.elapsed_ms,
                    baseline.elapsed_ms,
                    candidate.selector_diag.last_return_stage,
                    baseline.selector_diag.last_return_stage,
                    format_metric_delta(&exact_metrics(&candidate), &exact_metrics(&baseline)),
                    format_metric_delta(&engine_metrics(&candidate), &engine_metrics(&baseline)),
                    case.game.fen(),
                );
                println!(
                    "HOTSPOT_SELECTOR label={} {}",
                    case.label,
                    format_metric_delta(
                        &selector_metrics(&candidate),
                        &selector_metrics(&baseline)
                    ),
                );
            },
        );
    }
}

#[test]
#[ignore = "diagnostic: trace surviving pro confirm head-acceptance seams"]
fn smart_automove_pro_head_acceptance_trace_probe() {
    fn print_case(label: &str, fen: &str) {
        let game = MonsGame::from_fen(fen, false).expect("head acceptance probe fen should be valid");
        let perspective = game.active_color;
        clear_turn_engine_selector_diagnostics();
        clear_turn_engine_plan_cache();
        clear_exact_state_analysis_cache();
        let runtime_probe = runtime_decision_probe(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let (config, scored_roots, head_plan, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let Some(head_plan) = head_plan else {
            println!(
                "PRO_HEAD_ACCEPT_TRACE label={} current_selected={} probe={:?} advisor={:?} note=no_head_plan",
                label,
                profile_decision_move_fen("runtime_current", SmartAutomovePreference::Pro, &game),
                runtime_probe,
                pro_v2_root_advisor_decision_snapshot(),
            );
            return;
        };
        let pre_accept_selected = scored_roots
            .iter()
            .find(|root| Input::fen_from_array(&root.inputs) == runtime_probe.pre_accept_input_fen)
            .map(|root| root.inputs.clone())
            .unwrap_or_else(|| {
                MonsGameModel::pick_root_move_with_exploration(
                    &game,
                    scored_roots.as_slice(),
                    perspective,
                    config,
                )
            });
        let accepted = MonsGameModel::accept_turn_engine_head_after_search(
            &game,
            perspective,
            config,
            scored_roots.as_slice(),
            pre_accept_selected.as_slice(),
            &head_plan,
        );
        let candidate_inputs = runtime_probe
            .head_input_fen
            .as_ref()
            .and_then(|head| {
                scored_roots
                    .iter()
                    .find(|root| Input::fen_from_array(&root.inputs) == *head)
                    .map(|root| root.inputs.clone())
            })
            .or_else(|| head_plan.compiled_chunks.first().cloned());
        let Some(candidate_inputs) = candidate_inputs else {
            println!(
                "PRO_HEAD_ACCEPT_TRACE label={} current_selected={} probe={:?} advisor={:?} note=no_candidate_inputs",
                label,
                profile_decision_move_fen("runtime_current", SmartAutomovePreference::Pro, &game),
                runtime_probe,
                pro_v2_root_advisor_decision_snapshot(),
            );
            return;
        };
        let Some(candidate_index) = scored_roots
            .iter()
            .position(|root| root.inputs == candidate_inputs)
        else {
            println!(
                "PRO_HEAD_ACCEPT_TRACE label={} current_selected={} probe={:?} advisor={:?} note=candidate_not_in_roots candidate_inputs={}",
                label,
                profile_decision_move_fen("runtime_current", SmartAutomovePreference::Pro, &game),
                runtime_probe,
                pro_v2_root_advisor_decision_snapshot(),
                Input::fen_from_array(&candidate_inputs),
            );
            return;
        };
        let selected_index = scored_roots
            .iter()
            .position(|root| root.inputs == pre_accept_selected)
            .expect("selected should be present in scored roots");
        let candidate = &scored_roots[candidate_index];
        let selected = &scored_roots[selected_index];
        let candidate_family = MonsGameModel::turn_engine_root_evaluation_family(candidate);
        let selected_family = MonsGameModel::turn_engine_root_evaluation_family(selected);
        let candidate_unsafe = MonsGameModel::turn_engine_root_evaluation_is_unsafe(candidate);
        let selected_unsafe = MonsGameModel::turn_engine_root_evaluation_is_unsafe(selected);
        let candidate_progress_surface =
            MonsGameModel::turn_engine_root_evaluation_has_progress_surface(candidate);
        let selected_progress_surface =
            MonsGameModel::turn_engine_root_evaluation_has_progress_surface(selected);
        let candidate_spirit_tactical = candidate.spirit_same_turn_score_setup_now
            || candidate.same_turn_score_window_value > 0
            || candidate.attacks_opponent_drainer
            || candidate.scores_supermana_this_turn
            || candidate.scores_opponent_mana_this_turn
            || candidate.safe_supermana_pickup_now
            || candidate.safe_opponent_mana_pickup_now;
        let score_gap = selected.score.saturating_sub(candidate.score);
        let candidate_utility = MonsGameModel::turn_engine_selected_override_utility(
            &game,
            candidate,
            perspective,
            config,
            candidate_family,
        );
        let selected_utility = MonsGameModel::turn_engine_selected_override_utility(
            &game,
            selected,
            perspective,
            config,
            selected_family,
        );
        let black_plain_spirit_followup_head_override =
            matches!(config.turn_engine_mode, TurnEngineMode::ProV2)
                && matches!(head_plan.head_family, TurnPlanFamily::SpiritImpact)
                && MonsGameModel::is_pro_v2_black_plain_spirit_followup_setup_pair(
                    &game, candidate, selected, config,
                )
                && candidate.score > selected.score;
        let black_turn_six_route_change_plain_spirit_head_override =
            matches!(config.turn_engine_mode, TurnEngineMode::ProV2)
                && game.active_color == Color::Black
                && game.turn_number == 6
                && game.mons_moves_count == 0
                && game.player_can_use_action()
                && game.player_can_move_mana()
                && matches!(head_plan.head_family, TurnPlanFamily::SpiritImpact)
                && matches!(head_plan.goal_family, TurnPlanFamily::SpiritImpact)
                && candidate_index <= 1
                && candidate_unsafe == selected_unsafe
                && candidate.spirit_development
                && !candidate.spirit_same_turn_score_setup_now
                && !candidate.spirit_own_mana_setup_now
                && !candidate_progress_surface
                && !candidate_spirit_tactical
                && matches!(selected_family, TurnPlanFamily::ManaTempo)
                && selected_unsafe
                && !selected.spirit_development
                && !selected.spirit_same_turn_score_setup_now
                && !selected.spirit_own_mana_setup_now
                && !selected_progress_surface
                && !selected.wins_immediately
                && !selected.attacks_opponent_drainer
                && !selected.scores_supermana_this_turn
                && !selected.scores_opponent_mana_this_turn
                && !selected.safe_supermana_pickup_now
                && !selected.safe_opponent_mana_pickup_now
                && selected.same_turn_score_window_value == 0
                && candidate.inputs.first() != selected.inputs.first()
                && candidate.own_drainer_vulnerable == selected.own_drainer_vulnerable
                && candidate.own_drainer_walk_vulnerable == selected.own_drainer_walk_vulnerable
                && candidate.mana_handoff_to_opponent == selected.mana_handoff_to_opponent
                && candidate.has_roundtrip == selected.has_roundtrip
                && score_gap <= 1024;
        let white_spirit_setup_gain_head_override =
            matches!(config.turn_engine_mode, TurnEngineMode::ProV2)
                && game.active_color == Color::White
                && matches!(head_plan.head_family, TurnPlanFamily::SpiritImpact)
                && !selected_unsafe
                && !candidate_unsafe
                && candidate.spirit_own_mana_setup_now
                && !candidate.spirit_same_turn_score_setup_now
                && !selected.spirit_own_mana_setup_now
                && !selected.spirit_same_turn_score_setup_now
                && selected.spirit_development
                && candidate.spirit_development
                && candidate.spirit_setup_gain
                    >= selected.spirit_setup_gain.saturating_add(32)
                && candidate.safe_supermana_progress_steps
                    <= selected.safe_supermana_progress_steps
                && candidate.safe_opponent_mana_progress_steps
                    <= selected.safe_opponent_mana_progress_steps
                && candidate.own_drainer_vulnerable == selected.own_drainer_vulnerable
                && candidate.mana_handoff_to_opponent == selected.mana_handoff_to_opponent
                && candidate.has_roundtrip == selected.has_roundtrip
                && score_gap <= 96;
        let plan_dominates_selected =
            crate::models::automove_turn_engine::compare_utility_primary_axes(
                head_plan.utility,
                selected_utility,
            ) == std::cmp::Ordering::Greater
                && (head_plan
                    .utility
                    .strictly_dominates_override_axes(selected_utility)
                    || head_plan
                        .head_utility
                        .strictly_dominates_override_axes(selected_utility));
        let candidate_dominates_selected =
            crate::models::automove_turn_engine::compare_utility_primary_axes(
                candidate_utility,
                selected_utility,
            ) == std::cmp::Ordering::Greater
                && candidate_utility.strictly_dominates_override_axes(selected_utility);
        println!(
            "PRO_HEAD_ACCEPT_TRACE label={} current_selected={} accepted={} selected={} selected_rank={} candidate={} candidate_rank={} head_family={:?} goal_family={:?} candidate_family={:?} selected_family={:?} score_gap={} candidate_unsafe={} selected_unsafe={} candidate_progress={} selected_progress={} candidate_spirit_tactical={} same_first_input={} same_handoff={} same_roundtrip={} plan_vs_selected={:?} head_vs_selected={:?} candidate_vs_selected={:?} plan_dominates_selected={} candidate_dominates_selected={} black_followup_override={} black_turn6_override={} white_setup_override={} plan_utility={:?} head_utility={:?} selected_utility={:?} candidate_utility={:?} advisor={:?}",
            label,
            profile_decision_move_fen("runtime_current", SmartAutomovePreference::Pro, &game),
            accepted,
            Input::fen_from_array(&selected.inputs),
            selected_index,
            Input::fen_from_array(&candidate.inputs),
            candidate_index,
            head_plan.head_family,
            head_plan.goal_family,
            candidate_family,
            selected_family,
            score_gap,
            candidate_unsafe,
            selected_unsafe,
            candidate_progress_surface,
            selected_progress_surface,
            candidate_spirit_tactical,
            candidate.inputs.first() == selected.inputs.first(),
            candidate.mana_handoff_to_opponent == selected.mana_handoff_to_opponent,
            candidate.has_roundtrip == selected.has_roundtrip,
            crate::models::automove_turn_engine::compare_utility_primary_axes(
                head_plan.utility,
                selected_utility,
            ),
            crate::models::automove_turn_engine::compare_utility_primary_axes(
                head_plan.head_utility,
                selected_utility,
            ),
            crate::models::automove_turn_engine::compare_utility_primary_axes(
                candidate_utility,
                selected_utility,
            ),
            plan_dominates_selected,
            candidate_dominates_selected,
            black_plain_spirit_followup_head_override,
            black_turn_six_route_change_plain_spirit_head_override,
            white_spirit_setup_gain_head_override,
            head_plan.utility,
            head_plan.head_utility,
            selected_utility,
            candidate_utility,
            pro_v2_root_advisor_decision_snapshot(),
        );
    }

    for (label, fen) in [
        (
            "pro_nonwin_black_post_search_repeat2_opening0",
            "1 0 b 0 0 0 0 0 6 n05d1xa0xn04/n05s0xn01e0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/n05xxMn05/n01y0xn01xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        ),
        (
            "pro_nonwin_white_cluster_repeat2_opening2",
            "1 1 w 1 0 0 0 0 5 d0xn10/n05s0xa0xe0xn03/n03y0xn03xxmn03/n11/n04xxmxxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n05S0xn05/n04E0xA0xn05/n07Y0xn02D0x",
        ),
        (
            "pro_nonwin_black_head_repeat2_opening3",
            "0 0 b 0 0 2 0 0 2 n03y0xn01d0xa0xn04/n04s0xn01e0xn04/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04A0xD0xn05/n03E0xn02S0xn02Y0xn01",
        ),
        (
            "pro_nonwin_black_turn6_repeat3_opening0",
            "1 0 b 0 0 2 0 0 6 n11/n01xxmn04a0xn04/n04s0xn01d0xxxmn03/n02y0xn01xxmn06/n05xxmn01xxmn03/xxQn09e0x/n03xxMn01xxMxxUn01xxMn02/n05S0xn01Y0xn03/n02xxMn08/n04A0xn06/n02E0xn07D0x",
        ),
        (
            "pro_nonwin_black_turn4_repeat3_opening2",
            "1 0 b 1 0 0 0 0 4 n11/n03y0xd0ms0xa0xe0xn03/n07xxmn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n04S0xn04Y0xn01/n11/n03E0xA0xn05D0x",
        ),
    ] {
        print_case(label, fen);
    }
}

#[test]
#[ignore = "diagnostic: inspect remaining black pro confirm advisor seams"]
fn smart_automove_pro_remaining_black_nonwin_board_probe() {
    fn print_case(label: &str, fen: &str) {
        let game = MonsGame::from_fen(fen, false).expect("remaining black nonwin fen should be valid");
        clear_turn_engine_selector_diagnostics();
        clear_turn_engine_plan_cache();
        clear_exact_state_analysis_cache();
        let probe = runtime_decision_probe(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let advisor = pro_v2_root_advisor_decision_snapshot();
        let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let (_, current_scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_current",
            SmartAutomovePreference::Pro,
            &game,
        );
        let current_selected = profile_decision_move_fen(
            "runtime_current",
            SmartAutomovePreference::Pro,
            &game,
        );
        let current_root = format_root_probe(
            current_scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == current_selected),
        );
        let candidate_top_root_details = scored_roots
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
        let current_top_root_details = current_scored_roots
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
            "PRO_BLACK_NONWIN_BOARD label={} current_selected={} current_root=\"{}\" context={} candidate_top_root_details={:?} current_top_root_details={:?} probe={:?} advisor={:?}",
            label,
            current_selected,
            current_root,
            exact_opportunity_context_probe(&game),
            candidate_top_root_details,
            current_top_root_details,
            probe,
            advisor,
        );
    }

    for (label, fen) in [
        (
            "pro_nonwin_black_repeat2_opening0_ply36",
            "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        ),
        (
            "pro_nonwin_black_repeat2_opening3_ply14",
            "0 0 b 0 0 1 0 0 4 n06a0xn04/n04s0xd0xn05/n01y0xn04xxme0xn03/n03xxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n11/n04A0xn01S0xn02D0Mn01/n03E0xn05Y0xn01",
        ),
        (
            "pro_nonwin_black_repeat3_opening0_ply31",
            "1 0 b 0 0 2 0 0 6 n11/n01xxmn04a0xn04/n04s0xn01d0xxxmn03/n02y0xn01xxmn06/n05xxmn01xxmn03/xxQn09e0x/n03xxMn01xxMxxUn01xxMn02/n05S0xn01Y0xn03/n02xxMn08/n04A0xn06/n02E0xn07D0x",
        ),
        (
            "pro_nonwin_black_repeat3_opening2_ply16",
            "1 0 b 1 0 0 0 0 4 n11/n03y0xd0ms0xa0xe0xn03/n07xxmn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n04S0xn04Y0xn01/n11/n03E0xA0xn05D0x",
        ),
    ] {
        print_case(label, fen);
    }
}

#[test]
#[ignore = "diagnostic: inspect remaining confirm-vs-pro advisor seams"]
fn smart_automove_pro_confirm_pro_nonwin_board_probe() {
    fn print_case(label: &str, fen: &str) {
        let game = MonsGame::from_fen(fen, false).expect("confirm-vs-pro nonwin fen should be valid");

        clear_turn_engine_selector_diagnostics();
        let probe = runtime_decision_probe(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let advisor = pro_v2_root_advisor_decision_snapshot();
        let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
            pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);
        let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
        let (_, current_scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_current",
            SmartAutomovePreference::Pro,
            &game,
        );

        let current_selected = profile_decision_move_fen(
            "runtime_current",
            SmartAutomovePreference::Pro,
            &game,
        );
        let current_root = format_root_probe(
            current_scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == current_selected),
        );
        let candidate_top_root_details = scored_roots
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
        let current_top_root_details = current_scored_roots
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
            "PRO_CONFIRM_NONWIN_BOARD label={} current_selected={} current_root=\"{}\" context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} candidate_top_root_details={:?} current_top_root_details={:?} probe={:?} advisor={:?}",
            label,
            current_selected,
            current_root,
            exact_opportunity_context_probe(&game),
            legacy_selected,
            legacy_full_pool_selected,
            legacy_candidates,
            legacy_full_pool,
            candidate_top_root_details,
            current_top_root_details,
            probe,
            advisor,
        );
    }

    for (label, fen) in [
        (
            "confirm_pro_black_repeat2_opening0_ply36",
            "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        ),
        (
            "confirm_pro_white_repeat2_opening2_ply23",
            "1 1 w 1 0 0 0 0 5 d0xn10/n05s0xa0xe0xn03/n03y0xn03xxmn03/n04xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn03xxMn03/n04xxMn06/n07xxMn03/n03A0xn01S0xn01Y0xn03/n03E0xn06D0x",
        ),
        (
            "confirm_pro_black_repeat2_opening3_ply14",
            "0 0 b 0 0 1 0 0 4 n06a0xn04/n04s0xd0xn05/n01y0xn04xxme0xn03/n03xxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n11/n04A0xn01S0xn02D0Mn01/n03E0xn05Y0xn01",
        ),
        (
            "confirm_pro_black_repeat3_opening0_ply31",
            "1 0 b 0 0 2 0 0 6 n11/n01xxmn04a0xn04/n04s0xn01d0xxxmn03/n02y0xn01xxmn06/n05xxmn01xxmn03/xxQn09e0x/n03xxMn01xxMxxUn01xxMn02/n05S0xn01Y0xn03/n02xxMn08/n04A0xn06/n02E0xn07D0x",
        ),
        (
            "confirm_pro_black_repeat3_opening2_ply16",
            "1 0 b 1 0 0 0 0 4 n11/n03y0xd0ms0xa0xe0xn03/n07xxmn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n04S0xn04Y0xn01/n11/n03E0xA0xn05D0x",
        ),
    ] {
        print_case(label, fen);
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
        case_from_fixture("primary_spirit_setup"),
        case_from_fixture("primary_pvs_sensitive_search"),
        case_from_fixture("primary_black_reliability_opening_3_ply4"),
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
                let configured_runtime = calibration_runtime_config(
                    "runtime_pro_turn_engine_v30",
                    &case.game,
                    case.mode,
                );
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
#[ignore = "diagnostic: inspect retained pro-triage churn fixtures for runtime_pro_turn_engine_v30"]
fn smart_automove_pro_triage_retained_churn_probe() {
    let candidate_profile = "runtime_pro_turn_engine_v30";
    let baseline_profile = "runtime_current";
    let fixture_ids = [
        "primary_spirit_setup",
        "primary_pvs_sensitive_search",
        "primary_black_reliability_opening_3_ply4",
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
#[ignore = "diagnostic: inspect runtime-faithful forced-engine acceptance on retained churn fixtures"]
fn smart_automove_pro_runtime_faithful_retained_churn_probe() {
    let fixture_ids = [
        "primary_spirit_setup",
        "primary_pvs_sensitive_search",
        "primary_black_reliability_opening_3_ply4",
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
#[ignore = "diagnostic: inspect selector competition gates on primary_black_negative_deny_ply4"]
fn smart_automove_pro_black_negative_deny_selector_probe() {
    let fixture = primary_pro_fixture_by_id("primary_black_negative_deny_ply4");
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

    println!(
        "BLACK_NEGATIVE_DENY_SELECTOR selected={} baseline_selected={} filtered_len={} progress_competes={} followup_progress_competes={} risky_score_competes={} negative_deny_competes={} score_competes={} projection_competes={} risky_recovery_competes={} final_progress_reentry={:?}",
        Input::fen_from_array(&selected),
        Input::fen_from_array(&baseline_selected),
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

    let interesting = [
        "l0,5;l1,6",
        "l1,5;l3,4;l2,3",
        "l1,5;l3,6;l2,7",
        "l1,5;l0,3;l1,3",
        "l1,5;l0,7;l1,7",
    ];
    let mut followup_scores = std::collections::HashMap::new();
    for target in interesting {
        let rank = scored_roots
            .iter()
            .position(|root| Input::fen_from_array(&root.inputs) == target);
        println!("target={} rank={:?}", target, rank);
        if let Some(index) = rank {
            let root = &scored_roots[index];
            let projection = projections.get(&index).map(|plan| {
                (
                    plan.plan.head_family,
                    plan.plan.goal_family,
                    plan.plan.utility,
                    plan.plan.head_utility,
                )
            });
            let followup_floor = *followup_scores.entry(index).or_insert_with(|| {
                MonsGameModel::pro_v2_spirit_followup_floor_score(&root.game, perspective, config)
            });
            println!(
                "  fen={} score={} eff={} root_rank={} spirit={} plain_spirit={} setup_now={} own_setup={} vuln={} walk_vuln={} handoff={} roundtrip={} supermana_progress={} opponent_progress={} filtered={} projected={} projection={:?} followup_floor={} root=\"{}\"",
                target,
                root.score,
                root.efficiency,
                root.root_rank,
                root.spirit_development,
                MonsGameModel::is_plain_spirit_development_root(root),
                root.spirit_same_turn_score_setup_now,
                root.spirit_own_mana_setup_now,
                root.own_drainer_vulnerable,
                root.own_drainer_walk_vulnerable,
                root.mana_handoff_to_opponent,
                root.has_roundtrip,
                root.supermana_progress,
                root.opponent_mana_progress,
                filtered.contains(&index),
                projections.contains_key(&index),
                projection,
                followup_floor,
                format_root_probe(Some(root)),
            );
        }
    }
}

#[test]
#[ignore = "diagnostic: inspect forced engine shortlist seam on primary_white_harvest_loss_c_ply24"]
fn smart_automove_pro_white_harvest_forced_root_probe() {
    let fixture = primary_pro_fixture_by_id("primary_white_harvest_loss_c_ply24");
    let config =
        calibration_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
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
        let (config, scored_roots) =
            profile_scored_roots(profile_name, fixture.mode, &fixture.game);
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
#[ignore = "diagnostic: inspect later black accepted-head family on retained and traced duel boards"]
fn smart_automove_pro_black_late_accepted_head_probe() {
    fn run_probe(label: &str, game: &MonsGame, targets: &[&str]) {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let (config, scored_roots, head_plan, forced_engine_inputs) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "runtime_pro_turn_engine_v30",
                SmartAutomovePreference::Pro,
                game,
            );
        let perspective = game.active_color;
        let drainer_vulnerable = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        );
        let drainer_walk_vulnerable = MonsGameModel::is_own_drainer_walk_vulnerable_next_turn(
            game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        );
        let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
            game,
            scored_roots.as_slice(),
            perspective,
            config,
        );
        let selected = profile_decision_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            game,
        );
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let baseline_selected =
            profile_decision_inputs("runtime_current", SmartAutomovePreference::Pro, game);
        let head_plan =
            head_plan.expect("later black accepted-head probe should retain a head plan");
        let head_inputs = head_plan
            .compiled_chunks
            .first()
            .expect("head plan should include a first chunk");
        let accepted = MonsGameModel::accept_turn_engine_head_after_search(
            game,
            perspective,
            config,
            scored_roots.as_slice(),
            pre_accept_selected.as_slice(),
            &head_plan,
        );
        let pre_accept_root = scored_roots
            .iter()
            .find(|root| root.inputs == pre_accept_selected)
            .expect("pre-accept selected root should be present");
        let head_root = scored_roots
            .iter()
            .find(|root| root.inputs.as_slice() == head_inputs.as_slice())
            .expect("head root should be present");
        let baseline_root = scored_roots
            .iter()
            .find(|root| root.inputs == baseline_selected)
            .expect("baseline selected root should be present");
        let pre_accept_family = MonsGameModel::turn_engine_root_evaluation_family(pre_accept_root);
        let baseline_family = MonsGameModel::turn_engine_root_evaluation_family(baseline_root);
        let pre_accept_utility = MonsGameModel::turn_engine_root_plan_utility(
            game,
            pre_accept_root,
            perspective,
            config,
            pre_accept_family,
        );
        let baseline_utility = MonsGameModel::turn_engine_root_plan_utility(
            game,
            baseline_root,
            perspective,
            config,
            baseline_family,
        );

        println!(
            "BLACK_LATE_ACCEPTED_HEAD label={} selected={} pre_accept={} baseline_selected={} head={} accepted={} forced_inputs={:?} stage={} drainer_vulnerable={} drainer_walk_vulnerable={} head_family={:?} goal_family={:?} plan_utility={:?} head_utility={:?} pre_accept_utility={:?} baseline_utility={:?} fen={}",
            label,
            Input::fen_from_array(&selected),
            Input::fen_from_array(&pre_accept_selected),
            Input::fen_from_array(&baseline_selected),
            Input::fen_from_array(head_inputs),
            accepted,
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            selector_diag.last_return_stage,
            drainer_vulnerable,
            drainer_walk_vulnerable,
            head_plan.head_family,
            head_plan.goal_family,
            head_plan.utility,
            head_plan.head_utility,
            pre_accept_utility,
            baseline_utility,
            game.fen(),
        );
        println!(
            "BLACK_LATE_ACCEPTED_HEAD_ROOT label={} pre_accept=\"{}\" baseline=\"{}\" head=\"{}\"",
            label,
            format_root_probe(Some(pre_accept_root)),
            format_root_probe(Some(baseline_root)),
            format_root_probe(Some(head_root)),
        );
        for target in targets {
            let rank = scored_roots
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            println!(
                "BLACK_LATE_ACCEPTED_HEAD_TARGET label={} target={} rank={:?}",
                label, target, rank
            );
        }
    }

    let retained_fixture = primary_pro_fixture_by_id("primary_black_late_accepted_head_ply4");
    let traced_game = MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn07/n01y0xn01xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n05S0xn05/n04E0xA0xn05/D0xn06Y0xn03",
        false,
    )
    .expect("valid traced later black accepted-head fen");

    for (label, game, targets) in [
        (
            "primary_black_late_accepted_head_ply4",
            &retained_fixture.game,
            &["l1,5;l1,7;l0,7", "l3,2;l4,1"][..],
        ),
        (
            "traced_normal_duel_v5",
            &traced_game,
            &["l1,5;l1,7;l0,7", "l4,1;l5,0;mb"][..],
        ),
    ] {
        run_probe(label, game, targets);
    }
}

#[test]
#[ignore = "diagnostic: inspect repeated white fast accepted head on primary_white_fast_accepted_head_ply13"]
fn smart_automove_pro_white_fast_accepted_head_probe() {
    let fixture = primary_pro_fixture_by_id("primary_white_fast_accepted_head_ply13");
    let (config, scored_roots, head_plan, forced_engine_inputs) =
        profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            fixture.mode,
            &fixture.game,
        );
    let perspective = fixture.game.active_color;
    let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
        &fixture.game,
        scored_roots.as_slice(),
        perspective,
        config,
    );
    let selected =
        profile_decision_inputs("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game);
    let baseline_selected = profile_decision_inputs("runtime_current", fixture.mode, &fixture.game);
    let head_plan = head_plan.expect("white fast accepted-head fixture should retain a head plan");
    let head_inputs = head_plan
        .compiled_chunks
        .first()
        .expect("head plan should include a first chunk");
    let accepted = MonsGameModel::accept_turn_engine_head_after_search(
        &fixture.game,
        perspective,
        config,
        scored_roots.as_slice(),
        pre_accept_selected.as_slice(),
        &head_plan,
    );
    let pre_accept_root = scored_roots
        .iter()
        .find(|root| root.inputs == pre_accept_selected)
        .expect("pre-accept selected root should be present");
    let head_root = scored_roots
        .iter()
        .find(|root| root.inputs.as_slice() == head_inputs.as_slice())
        .expect("head root should be present");
    let baseline_root = scored_roots
        .iter()
        .find(|root| root.inputs == baseline_selected)
        .expect("baseline selected root should be present");
    let pre_accept_family = MonsGameModel::turn_engine_root_evaluation_family(pre_accept_root);
    let baseline_family = MonsGameModel::turn_engine_root_evaluation_family(baseline_root);
    let pre_accept_utility = MonsGameModel::turn_engine_root_plan_utility(
        &fixture.game,
        pre_accept_root,
        perspective,
        config,
        pre_accept_family,
    );
    let baseline_utility = MonsGameModel::turn_engine_root_plan_utility(
        &fixture.game,
        baseline_root,
        perspective,
        config,
        baseline_family,
    );

    println!(
        "WHITE_FAST_ACCEPTED_HEAD selected={} pre_accept={} baseline_selected={} head={} accepted={} forced_inputs={:?} head_family={:?} goal_family={:?} plan_utility={:?} head_utility={:?} pre_accept_utility={:?} baseline_utility={:?}",
        Input::fen_from_array(&selected),
        Input::fen_from_array(&pre_accept_selected),
        Input::fen_from_array(&baseline_selected),
        Input::fen_from_array(head_inputs),
        accepted,
        forced_engine_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs)),
        head_plan.head_family,
        head_plan.goal_family,
        head_plan.utility,
        head_plan.head_utility,
        pre_accept_utility,
        baseline_utility,
    );
    println!(
        "WHITE_FAST_ACCEPTED_HEAD_ROOT pre_accept=\"{}\" baseline=\"{}\" head=\"{}\"",
        format_root_probe(Some(pre_accept_root)),
        format_root_probe(Some(baseline_root)),
        format_root_probe(Some(head_root)),
    );
    for target in ["l9,4;l8,4", "l8,7;l7,8"] {
        let rank = scored_roots
            .iter()
            .position(|root| Input::fen_from_array(&root.inputs) == target);
        println!("target={} rank={:?}", target, rank);
    }
}

#[test]
#[ignore = "diagnostic: inspect black turn-four one-move action+mana forced progress-head seam"]
fn smart_automove_pro_black_turn_four_action_mana_probe() {
    let fixture = primary_pro_fixture_by_id("primary_black_turn_four_action_mana_ply15");
    clear_exact_state_analysis_cache();
    clear_exact_query_diagnostics();
    clear_turn_engine_plan_cache();
    clear_turn_engine_diagnostics();
    clear_turn_engine_selector_diagnostics();

    let base_runtime =
        SearchBudget::from_preference(fixture.mode).runtime_config_for_game(&fixture.game);
    let configured_runtime =
        calibration_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let guarded_inputs = model_runtime_pro_turn_engine_v30(&fixture.game, base_runtime);
    let selected_inputs =
        profile_decision_inputs("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game);
    let selector_diag = turn_engine_selector_diagnostics_snapshot();
    let direct_configured_inputs =
        MonsGameModel::smart_search_best_inputs(&fixture.game, configured_runtime);
    let current_inputs = model_current_best(
        &fixture.game,
        SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(&fixture.game),
    );
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
    let guarded_root = scored_roots
        .iter()
        .find(|root| root.inputs == guarded_inputs);
    let pre_accept_root = scored_roots
        .iter()
        .find(|root| root.inputs == pre_accept_selected);
    let current_root = scored_roots
        .iter()
        .find(|root| root.inputs == current_inputs);
    let head_root = head_rank.and_then(|index| scored_roots.get(index));
    let pre_accept_family = pre_accept_root.map(MonsGameModel::turn_engine_root_evaluation_family);
    let current_family = current_root.map(MonsGameModel::turn_engine_root_evaluation_family);
    let pre_accept_utility = pre_accept_root.map(|root| {
        MonsGameModel::turn_engine_root_plan_utility(
            &fixture.game,
            root,
            fixture.game.active_color,
            config,
            pre_accept_family.expect("pre_accept family should exist"),
        )
    });
    let current_utility = current_root.map(|root| {
        MonsGameModel::turn_engine_root_plan_utility(
            &fixture.game,
            root,
            fixture.game.active_color,
            config,
            current_family.expect("current family should exist"),
        )
    });

    println!(
        "BLACK_TURN_FOUR_ACTION_MANA guarded={} selected={} configured={} current={} pre_accept={} forced_inputs={:?} head={:?} accepted={} stage={} turn={} mons_moves={} action={} mana={} head_family={:?} goal_family={:?} plan_utility={:?} head_utility={:?} pre_accept_utility={:?} current_utility={:?} guarded_root=\"{}\" pre_accept_root=\"{}\" current_root=\"{}\" head_root=\"{}\"",
        Input::fen_from_array(&guarded_inputs),
        Input::fen_from_array(&selected_inputs),
        Input::fen_from_array(&direct_configured_inputs),
        Input::fen_from_array(&current_inputs),
        Input::fen_from_array(&pre_accept_selected),
        forced_engine_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs)),
        head_plan
            .as_ref()
            .and_then(|plan| plan.compiled_chunks.first())
            .map(|chunk| Input::fen_from_array(chunk)),
        accepted,
        selector_diag.last_return_stage,
        fixture.game.turn_number,
        fixture.game.mons_moves_count,
        fixture.game.player_can_use_action(),
        fixture.game.player_can_move_mana(),
        head_plan.as_ref().map(|plan| plan.head_family),
        head_plan.as_ref().map(|plan| plan.goal_family),
        head_plan.as_ref().map(|plan| plan.utility),
        head_plan.as_ref().map(|plan| plan.head_utility),
        pre_accept_utility,
        current_utility,
        format_root_probe(guarded_root),
        format_root_probe(pre_accept_root),
        format_root_probe(current_root),
        format_root_probe(head_root),
    );
}

#[test]
#[ignore = "diagnostic: compare retained black forced-root families at injection stage"]
fn smart_automove_pro_black_forced_root_probe() {
    fn run_probe(label: &str, game: &MonsGame, mode: SmartAutomovePreference, targets: &[&str]) {
        let config = calibration_runtime_config("runtime_pro_turn_engine_v30", game, mode);
        let perspective = game.active_color;
        let root_moves = MonsGameModel::ranked_root_moves(game, perspective, config);
        let engine_plan = turn_engine_candidate_plan(
            game,
            perspective,
            MonsGameModel::turn_engine_search_config_for_game(game, config),
        )
        .expect("black forced-root fixture should materialize a turn-engine plan");
        let forced_chunk = engine_plan
            .compiled_chunks
            .first()
            .cloned()
            .expect("engine plan should have a first chunk");
        let existing_forced_rank = root_moves
            .iter()
            .position(|root| root.inputs == forced_chunk);

        let mut injected_root_moves = root_moves.clone();
        let forced_engine_inputs = MonsGameModel::inject_turn_engine_root_candidate(
            game,
            perspective,
            config,
            &mut injected_root_moves,
            &engine_plan,
        );
        let injected_forced_rank = injected_root_moves
            .iter()
            .position(|root| root.inputs == forced_chunk);
        let (focused_root_moves, _) = MonsGameModel::focused_root_candidates_with_forced_inputs(
            game,
            perspective,
            injected_root_moves.clone(),
            config,
            true,
            forced_engine_inputs.as_deref(),
        );
        let focused_forced_rank = focused_root_moves
            .iter()
            .position(|root| root.inputs == forced_chunk);

        println!(
            "BLACK_FORCED_ROOT label={} forced={} existing_forced_rank={:?} injected_forced_rank={:?} focused_forced_rank={:?} forced_inputs={:?} head_family={:?} goal_family={:?} plan_utility={:?} head_utility={:?}",
            label,
            Input::fen_from_array(&forced_chunk),
            existing_forced_rank,
            injected_forced_rank,
            focused_forced_rank,
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            engine_plan.head_family,
            engine_plan.goal_family,
            engine_plan.utility,
            engine_plan.head_utility,
        );

        for target in targets {
            let raw_rank = root_moves
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            let injected_rank = injected_root_moves
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            let focused_rank = focused_root_moves
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            println!(
                "BLACK_FORCED_ROOT_TARGET label={} target={} raw_rank={:?} injected_rank={:?} focused_rank={:?}",
                label, target, raw_rank, injected_rank, focused_rank,
            );
        }

        for (rank, root) in root_moves.iter().enumerate().take(8) {
            println!(
                "BLACK_FORCED_ROOT_RAW label={} rank={} fen={} forced_match={} root=\"{}\"",
                label,
                rank,
                Input::fen_from_array(&root.inputs),
                root.inputs == forced_chunk,
                format_scored_root_move_probe(Some(root)),
            );
        }
        for (rank, root) in focused_root_moves.iter().enumerate().take(8) {
            println!(
                "BLACK_FORCED_ROOT_FOCUSED label={} rank={} fen={} forced_match={} root=\"{}\"",
                label,
                rank,
                Input::fen_from_array(&root.inputs),
                root.inputs == forced_chunk,
                format_scored_root_move_probe(Some(root)),
            );
        }
    }

    let action_mana_fixture =
        primary_pro_fixture_by_id("primary_black_turn_four_action_mana_ply15");
    let late_head_fixture = primary_pro_fixture_by_id("primary_black_late_accepted_head_ply4");
    let mana_bridge_fixture = primary_pro_fixture_by_id("primary_black_mana_bridge_ply20");
    let spirit_bridge_fixture = primary_pro_fixture_by_id("primary_black_spirit_bridge_ply19");
    let traced_fast_game = MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn07/n01y0xn01xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn03xxMn04/n05S0xn05/n03A0xn04Y0xn02/D0xn02E0xn07",
        false,
    )
    .expect("valid traced fast black forced-root fen");
    let traced_fast_nonwin_game = MonsGame::from_fen(
        "1 0 b 0 0 1 0 0 4 n06a0xn04/n05s0xd0xe0xn03/n07xxmn03/n02y0xxxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n11/n02E0xA0xn01S0xn01Y0xn03/D0xn10",
        false,
    )
    .expect("valid traced fast non-win black forced-root fen");

    run_probe(
        "primary_black_turn_four_action_mana_ply15",
        &action_mana_fixture.game,
        action_mana_fixture.mode,
        &["l1,6;l2,7", "l2,3;l3,2"],
    );
    run_probe(
        "primary_black_late_accepted_head_ply4",
        &late_head_fixture.game,
        late_head_fixture.mode,
        &["l1,5;l1,7;l0,7", "l3,2;l4,1"],
    );
    run_probe(
        "primary_black_mana_bridge_ply20",
        &mana_bridge_fixture.game,
        mana_bridge_fixture.mode,
        &["l0,5;l1,4", "l4,1;l5,0;mb"],
    );
    run_probe(
        "primary_black_spirit_bridge_ply19",
        &spirit_bridge_fixture.game,
        spirit_bridge_fixture.mode,
        &["l1,5;l1,7;l0,7", "l4,1;l5,0;mb"],
    );
    run_probe(
        "traced_fast_duel_v7",
        &traced_fast_game,
        SmartAutomovePreference::Pro,
        &["l0,5;l1,4", "l4,1;l5,0;mb"],
    );
    run_probe(
        "traced_fast_nonwin_v1",
        &traced_fast_nonwin_game,
        SmartAutomovePreference::Pro,
        &["l1,6;l2,7", "l3,2;l4,1"],
    );
}

#[test]
#[ignore = "diagnostic: compare retained and traced black forced-engine seams at runtime-faithful selection stage"]
fn smart_automove_pro_black_forced_runtime_probe() {
    fn run_probe(label: &str, game: &MonsGame, mode: SmartAutomovePreference, targets: &[&str]) {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let base_runtime = SearchBudget::from_preference(mode).runtime_config_for_game(game);
        let guarded_inputs = model_runtime_pro_turn_engine_v30(game, base_runtime);
        let selected = profile_decision_inputs("runtime_pro_turn_engine_v30", mode, game);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let baseline_selected = profile_decision_inputs("runtime_current", mode, game);
        let configured_runtime =
            calibration_runtime_config("runtime_pro_turn_engine_v30", game, mode);
        let configured_selected = MonsGameModel::smart_search_best_inputs(game, configured_runtime);
        let (config, scored_roots, head_plan, forced_engine_inputs) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "runtime_pro_turn_engine_v30",
                mode,
                game,
            );
        let perspective = game.active_color;
        let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
            game,
            scored_roots.as_slice(),
            perspective,
            config,
        );
        let accepted = head_plan.as_ref().is_some_and(|plan| {
            MonsGameModel::accept_turn_engine_head_after_search(
                game,
                perspective,
                config,
                scored_roots.as_slice(),
                pre_accept_selected.as_slice(),
                plan,
            )
        });
        let selected_root = scored_roots.iter().find(|root| root.inputs == selected);
        let pre_accept_root = scored_roots
            .iter()
            .find(|root| root.inputs == pre_accept_selected);
        let baseline_root = scored_roots
            .iter()
            .find(|root| root.inputs == baseline_selected);
        let head_root = head_plan.as_ref().and_then(|plan| {
            plan.compiled_chunks.first().and_then(|chunk| {
                scored_roots
                    .iter()
                    .find(|root| root.inputs.as_slice() == chunk.as_slice())
            })
        });
        let selected_utility = selected_root.map(|root| {
            let family = MonsGameModel::turn_engine_root_evaluation_family(root);
            MonsGameModel::turn_engine_root_plan_utility(game, root, perspective, config, family)
        });
        let baseline_utility = baseline_root.map(|root| {
            let family = MonsGameModel::turn_engine_root_evaluation_family(root);
            MonsGameModel::turn_engine_root_plan_utility(game, root, perspective, config, family)
        });
        let reply_limit = config.node_enum_limit.clamp(
            SMART_NORMAL_ROOT_SAFETY_REPLY_LIMIT_MIN,
            SMART_NORMAL_ROOT_SAFETY_REPLY_LIMIT_MAX,
        );
        let my_score_before = MonsGameModel::score_for_color(game, perspective);
        let start_options = MonsGameModel::automove_start_input_options(config);
        let selected_normal_snapshot = selected_root.map(|root| {
            MonsGameModel::normal_root_safety_snapshot(
                &root.game,
                perspective,
                my_score_before,
                config,
                reply_limit,
                start_options,
            )
        });
        let baseline_normal_snapshot = baseline_root.map(|root| {
            MonsGameModel::normal_root_safety_snapshot(
                &root.game,
                perspective,
                my_score_before,
                config,
                reply_limit,
                start_options,
            )
        });
        let head_normal_snapshot = head_root.map(|root| {
            MonsGameModel::normal_root_safety_snapshot(
                &root.game,
                perspective,
                my_score_before,
                config,
                reply_limit,
                start_options,
            )
        });

        println!(
            "BLACK_FORCED_RUNTIME label={} guarded={} selected={} configured={} baseline_selected={} pre_accept={} forced_inputs={:?} stage={} head={:?} accepted={} head_family={:?} goal_family={:?} plan_utility={:?} head_utility={:?} selected_utility={:?} baseline_utility={:?} normal_safety(selected=\"{}\" baseline=\"{}\" head=\"{}\") fen={}",
            label,
            Input::fen_from_array(&guarded_inputs),
            Input::fen_from_array(&selected),
            Input::fen_from_array(&configured_selected),
            Input::fen_from_array(&baseline_selected),
            Input::fen_from_array(&pre_accept_selected),
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            selector_diag.last_return_stage,
            head_plan
                .as_ref()
                .and_then(|plan| plan.compiled_chunks.first())
                .map(|chunk| Input::fen_from_array(chunk)),
            accepted,
            head_plan.as_ref().map(|plan| plan.head_family),
            head_plan.as_ref().map(|plan| plan.goal_family),
            head_plan.as_ref().map(|plan| plan.utility),
            head_plan.as_ref().map(|plan| plan.head_utility),
            selected_utility,
            baseline_utility,
            format_normal_safety_probe(selected_normal_snapshot),
            format_normal_safety_probe(baseline_normal_snapshot),
            format_normal_safety_probe(head_normal_snapshot),
            game.fen(),
        );
        println!(
            "BLACK_FORCED_RUNTIME_ROOTS label={} selected=\"{}\" pre_accept=\"{}\" baseline=\"{}\" head=\"{}\"",
            label,
            format_root_probe(selected_root),
            format_root_probe(pre_accept_root),
            format_root_probe(baseline_root),
            format_root_probe(head_root),
        );
        for target in targets {
            let rank = scored_roots
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            println!(
                "BLACK_FORCED_RUNTIME_TARGET label={} target={} rank={:?}",
                label, target, rank
            );
        }
    }

    let action_mana_fixture =
        primary_pro_fixture_by_id("primary_black_turn_four_action_mana_ply15");
    let late_head_fixture = primary_pro_fixture_by_id("primary_black_late_accepted_head_ply4");
    let mana_bridge_fixture = primary_pro_fixture_by_id("primary_black_mana_bridge_ply20");
    let spirit_bridge_fixture = primary_pro_fixture_by_id("primary_black_spirit_bridge_ply19");
    let traced_fast_v10_game = MonsGame::from_fen(
        "1 0 b 0 0 1 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n02y0xn01xxmn06/n03xxmn01xxmn02xxmn02/xxQn04xxUn04xxQ/n03xxMn03xxMn03/n04xxMn01xxMn04/n11/n04A0xn01S0xn04/D0xn02E0xn04Y0xn02",
        false,
    )
    .expect("valid traced fast v10 black mana rerank fen");
    let traced_fast_nonwin_v1_game = MonsGame::from_fen(
        "1 0 b 0 0 1 0 0 4 n06a0xn04/n05s0xd0xe0xn03/n07xxmn03/n02y0xxxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n11/n02E0xA0xn01S0xn01Y0xn03/D0xn10",
        false,
    )
    .expect("valid traced fast non-win v1 black mana rerank fen");
    let traced_normal_v12_mana_game = MonsGame::from_fen(
        "0 0 b 1 0 0 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmxxmn07/n05xxmn01xxmn03/E0xn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn06/n05D0xn02xxMn02/n04A0xn01S0xn04/n08Y0xn02",
        false,
    )
    .expect("valid traced normal v12 black mana rerank fen");
    let traced_normal_v13_safety_game = MonsGame::from_fen(
        "0 0 b 0 0 2 0 0 4 n06a0xn04/n06d0xe0xn03/n04s0xn02xxmn03/n02y0xxxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMxxMn01Y0xxxMn04/n04D0xn06/n02E0xn01A0xn01S0xn04/n11",
        false,
    )
    .expect("valid traced normal v13 black drainer-safety rerank fen");
    let traced_pro_v12_game = MonsGame::from_fen(
        "1 0 b 0 0 1 0 0 4 n05d0xa0xn04/n06xxme0xn03/n05s0xn05/n02y0xn01xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n06xxMxxMn03/n11/n07S0xn03/D0xn01A0xE0xn04Y0xn02",
        false,
    )
    .expect("valid traced pro v12 black spirit head fen");
    let traced_normal_v20_spirit_bridge_game = MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn07/n01y0xn01xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n05S0xn05/n04E0xA0xn05/D0xn06Y0xn03",
        false,
    )
    .expect("valid traced normal v20 black spirit bridge fen");
    let traced_normal_v24_mana_game = MonsGame::from_fen(
        "0 0 b 0 0 2 0 0 4 n05d0xn05/n05s0xa0xxxme0xn02/n11/n02y0xn01xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn01xxMn04/n04D0xS0xn02xxMn02/n03A0xE0xn02Y0xn03/n11",
        false,
    )
    .expect("valid traced normal v24 black mana rerank fen");
    let traced_normal_v26_progress_game = MonsGame::from_fen(
        "1 0 b 1 0 0 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n02xxmy0xn03xxmn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn02xxMn03/n11/n04E0xn01S0xn04/n04A0xn02Y0xn02D0x",
        false,
    )
    .expect("valid traced normal v26 black progress rerank fen");
    let traced_fast_v69_mana_game = MonsGame::from_fen(
        "1 0 b 1 0 2 0 0 4 n06a0xn04/n05s0xd0xe0xn03/n03y0xn03xxmn03/n02xxmxxmn07/n05xxmn01xxmn03/xxQn04xxUn04xxQ/E0xn04xxMn01xxMn03/n02xxMn01xxMn01xxMn04/n05D0xS0xn04/n04A0xn01Y0xn04/n11",
        false,
    )
    .expect("valid traced fast v69 black mana rerank fen");

    run_probe(
        "primary_black_turn_four_action_mana_ply15",
        &action_mana_fixture.game,
        action_mana_fixture.mode,
        &["l1,6;l2,7", "l2,3;l3,2"],
    );
    run_probe(
        "primary_black_late_accepted_head_ply4",
        &late_head_fixture.game,
        late_head_fixture.mode,
        &["l1,5;l1,7;l0,7", "l3,2;l4,1"],
    );
    run_probe(
        "primary_black_mana_bridge_ply20",
        &mana_bridge_fixture.game,
        mana_bridge_fixture.mode,
        &["l0,5;l1,4", "l4,1;l5,0;mb"],
    );
    run_probe(
        "traced_fast_duel_v10",
        &traced_fast_v10_game,
        SmartAutomovePreference::Pro,
        &["l1,5;l1,4", "l3,2;l4,1"],
    );
    run_probe(
        "traced_fast_nonwin_v1",
        &traced_fast_nonwin_v1_game,
        SmartAutomovePreference::Pro,
        &["l1,6;l2,7", "l3,2;l4,1"],
    );
    run_probe(
        "traced_normal_duel_v12_mana",
        &traced_normal_v12_mana_game,
        SmartAutomovePreference::Pro,
        &["l1,5;l2,5", "l1,6;l0,6"],
    );
    run_probe(
        "traced_normal_duel_v13_safety",
        &traced_normal_v13_safety_game,
        SmartAutomovePreference::Pro,
        &["l1,6;l1,5", "l3,2;l4,1"],
    );
    run_probe(
        "primary_black_spirit_bridge_ply19",
        &spirit_bridge_fixture.game,
        spirit_bridge_fixture.mode,
        &["l1,5;l1,7;l0,7", "l4,1;l5,0;mb"],
    );
    run_probe(
        "traced_normal_duel_v20_spirit_bridge",
        &traced_normal_v20_spirit_bridge_game,
        SmartAutomovePreference::Pro,
        &["l1,5;l1,7;l0,7", "l4,1;l5,0;mb"],
    );
    run_probe(
        "traced_normal_duel_v24_mana",
        &traced_normal_v24_mana_game,
        SmartAutomovePreference::Pro,
        &["l1,5;l2,5", "l3,2;l4,1"],
    );
    run_probe(
        "traced_normal_duel_v26_progress",
        &traced_normal_v26_progress_game,
        SmartAutomovePreference::Pro,
        &["l1,5;l2,4", "l0,5;l1,4", "l1,5;l2,5"],
    );
    run_probe(
        "traced_fast_duel_v69_mana",
        &traced_fast_v69_mana_game,
        SmartAutomovePreference::Pro,
        &["l1,6;l2,7", "l2,3;l3,4"],
    );
    run_probe(
        "traced_pro_duel_v12",
        &traced_pro_v12_game,
        SmartAutomovePreference::Pro,
        &["l2,5;l0,5;l1,6", "l3,2;l4,1"],
    );
}

#[test]
#[ignore = "diagnostic: compare traced black spirit sibling board against early black opening fixtures"]
fn smart_automove_pro_black_spirit_sibling_probe() {
    fn run_probe(label: &str, game: &MonsGame, mode: SmartAutomovePreference, targets: &[&str]) {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let (config, scored_roots, head_plan, forced_engine_inputs) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "runtime_pro_turn_engine_v30",
                mode,
                game,
            );
        let perspective = game.active_color;
        let selected = profile_decision_inputs("runtime_pro_turn_engine_v30", mode, game);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let baseline_selected = profile_decision_inputs("runtime_current", mode, game);
        let mut candidate_indices =
            MonsGameModel::filtered_root_candidate_indices(game, scored_roots.as_slice(), perspective, config);
        if candidate_indices.is_empty() {
            candidate_indices = (0..scored_roots.len()).collect();
        }
        let reply_shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
            scored_roots.as_slice(),
            candidate_indices.as_slice(),
            config,
        );
        let root_node_budget = ((config.max_visited_nodes
            * config.root_reply_risk_node_share_bp.max(0) as usize)
            / 10_000)
            .max(reply_shortlist.len())
            .max(1);
        let per_root_reply_limit = (root_node_budget / reply_shortlist.len().max(1))
            .max(1)
            .min(config.root_reply_risk_reply_limit.max(1));
        let reply_guarded = MonsGameModel::pick_root_move_with_reply_risk_guard_from_shortlist(
            game,
            scored_roots.as_slice(),
            reply_shortlist.as_slice(),
            Some(candidate_indices.as_slice()),
            perspective,
            config,
        )
        .map(|index| scored_roots[index].inputs.clone());
        let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
            game,
            scored_roots.as_slice(),
            perspective,
            config,
        );
        let selected_root = scored_roots.iter().find(|root| root.inputs == selected);
        let pre_accept_root = scored_roots
            .iter()
            .find(|root| root.inputs == pre_accept_selected);
        let baseline_root = scored_roots
            .iter()
            .find(|root| root.inputs == baseline_selected);
        let head_root = head_plan.as_ref().and_then(|plan| {
            plan.compiled_chunks.first().and_then(|chunk| {
                scored_roots
                    .iter()
                    .find(|root| root.inputs.as_slice() == chunk.as_slice())
            })
        });
        let accepted = head_plan.as_ref().is_some_and(|plan| {
            MonsGameModel::accept_turn_engine_head_after_search(
                game,
                perspective,
                config,
                scored_roots.as_slice(),
                pre_accept_selected.as_slice(),
                plan,
            )
        });

        println!(
            "BLACK_SPIRIT_SIBLING label={} selected={} pre_accept={} baseline_selected={} forced_inputs={:?} stage={} accepted={} head={:?} reply_guarded={:?} head_family={:?} goal_family={:?} fen={}",
            label,
            Input::fen_from_array(&selected),
            Input::fen_from_array(&pre_accept_selected),
            Input::fen_from_array(&baseline_selected),
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            selector_diag.last_return_stage,
            accepted,
            head_plan
                .as_ref()
                .and_then(|plan| plan.compiled_chunks.first())
                .map(|chunk| Input::fen_from_array(chunk)),
            reply_guarded
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            head_plan.as_ref().map(|plan| plan.head_family),
            head_plan.as_ref().map(|plan| plan.goal_family),
            game.fen(),
        );
        println!(
            "BLACK_SPIRIT_SIBLING_ROOTS label={} selected=\"{}\" pre_accept=\"{}\" baseline=\"{}\" head=\"{}\"",
            label,
            format_root_probe(selected_root),
            format_root_probe(pre_accept_root),
            format_root_probe(baseline_root),
            format_root_probe(head_root),
        );
        for target in targets {
            let rank = scored_roots
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            let target_summary = rank.map(|index| {
                let root = &scored_roots[index];
                let snapshot = MonsGameModel::root_reply_risk_snapshot(
                    &root.game,
                    perspective,
                    config,
                    per_root_reply_limit,
                );
                let followup =
                    MonsGameModel::pro_v2_spirit_followup_floor_score(&root.game, perspective, config);
                format!(
                    "score={} family={:?} worst_reply={} followup={}",
                    root.score,
                    MonsGameModel::turn_engine_root_evaluation_family(root),
                    snapshot.worst_reply_score,
                    followup,
                )
            });
            println!(
                "BLACK_SPIRIT_SIBLING_TARGET label={} target={} rank={:?} summary={:?}",
                label, target, rank, target_summary
            );
        }
    }

    let traced_pro_v12_game = MonsGame::from_fen(
        "0 0 b 0 0 1 0 0 2 n04s0xd0xa0xe0xn03/n04y0xn06/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n04A0xn03S0xn02/n11/n03E0xn01D0xn02Y0xn02",
        false,
    )
    .expect("valid traced pro v12 black spirit sibling fen");
    let traced_pro_v14_game = MonsGame::from_fen(
        "0 0 b 0 0 1 0 0 2 n04s0xd0xa0xe0xn03/n02y0xn08/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n06A0xn04/n06Y0xn04/n03E0xn01D0xS0xn04",
        false,
    )
    .expect("valid traced pro v14 black spirit sibling fen");
    let traced_full_reliability_pro_game = MonsGame::from_fen(
        "0 0 b 0 0 1 0 0 2 n03y0xs0xd0xn01e0xn03/n05a0xn05/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05S0xn05/n03A0xn07/n02E0xn02D0xn02Y0xn02",
        false,
    )
    .expect("valid traced full reliability pro black spirit sibling fen");
    let traced_normal_v82_plain_followup_game = MonsGame::from_fen(
        "0 0 b 0 0 1 0 0 4 n03y0xn01d1xa0xe0xn03/n04s0xn06/n04xxmn06/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn05/n05xxMn01xxMn03/n03xxMxxMn01xxMn01Y0xn02/n03E0xn07/n05S0xn05/n04A0xD1xn05",
        false,
    )
    .expect("valid traced normal v82 black plain followup fen");
    let opening_a_fixture = primary_pro_fixture_by_id("primary_black_loss_opening_a_black_turn");
    let opening_b_fixture = primary_pro_fixture_by_id("primary_black_loss_opening_b_black_turn");
    let reliability_ba_fixture =
        primary_pro_fixture_by_id("primary_black_reliability_opening_0_ba_black_turn");
    let reliability_live_fixture =
        primary_pro_fixture_by_id("primary_black_reliability_opening_0_ba_live_black_turn");
    let gate_fixture = primary_pro_fixture_by_id("primary_black_gate_loss_a_ply4");

    for (label, game) in [
        ("traced_pro_duel_v12", &traced_pro_v12_game),
        ("traced_pro_duel_v14", &traced_pro_v14_game),
        ("traced_full_reliability_pro", &traced_full_reliability_pro_game),
        (
            "traced_normal_duel_v82_plain_followup",
            &traced_normal_v82_plain_followup_game,
        ),
        (
            "primary_black_loss_opening_a_black_turn",
            &opening_a_fixture.game,
        ),
        (
            "primary_black_loss_opening_b_black_turn",
            &opening_b_fixture.game,
        ),
        (
            "primary_black_reliability_opening_0_ba_black_turn",
            &reliability_ba_fixture.game,
        ),
        (
            "primary_black_reliability_opening_0_ba_live_black_turn",
            &reliability_live_fixture.game,
        ),
        ("primary_black_gate_loss_a_ply4", &gate_fixture.game),
    ] {
        run_probe(
            label,
            game,
            SmartAutomovePreference::Pro,
            &[
                "l0,4;l1,3",
                "l0,4;l1,4",
                "l0,4;l1,5",
                "l1,4;l2,5",
                "l1,4;l3,6;l2,6",
                "l1,4;l3,6;l2,7",
                "l1,4;l0,6;l1,6",
            ],
        );
    }
}

#[test]
#[ignore = "diagnostic: compare traced white safe-progress rerank against retained and opening white surfaces"]
fn smart_automove_pro_white_safe_progress_probe() {
    fn run_probe(label: &str, game: &MonsGame, mode: SmartAutomovePreference, targets: &[&str]) {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let (config, scored_roots, head_plan, forced_engine_inputs) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "runtime_pro_turn_engine_v30",
                mode,
                game,
            );
        let perspective = game.active_color;
        let drainer_vulnerable = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        );
        let drainer_walk_vulnerable = MonsGameModel::is_own_drainer_walk_vulnerable_next_turn(
            game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        );
        let selected = profile_decision_inputs("runtime_pro_turn_engine_v30", mode, game);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let baseline_selected = profile_decision_inputs("runtime_current", mode, game);
        let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
            game,
            scored_roots.as_slice(),
            perspective,
            config,
        );
        let selected_root = scored_roots.iter().find(|root| root.inputs == selected);
        let pre_accept_root = scored_roots
            .iter()
            .find(|root| root.inputs == pre_accept_selected);
        let baseline_root = scored_roots
            .iter()
            .find(|root| root.inputs == baseline_selected);
        let head_root = head_plan.as_ref().and_then(|plan| {
            plan.compiled_chunks.first().and_then(|chunk| {
                scored_roots
                    .iter()
                    .find(|root| root.inputs.as_slice() == chunk.as_slice())
            })
        });
        let accepted = head_plan.as_ref().is_some_and(|plan| {
            MonsGameModel::accept_turn_engine_head_after_search(
                game,
                perspective,
                config,
                scored_roots.as_slice(),
                pre_accept_selected.as_slice(),
                plan,
            )
        });

        println!(
            "WHITE_SAFE_PROGRESS label={} selected={} pre_accept={} baseline_selected={} forced_inputs={:?} stage={} accepted={} drainer_vulnerable={} drainer_walk_vulnerable={} head={:?} head_family={:?} goal_family={:?} fen={}",
            label,
            Input::fen_from_array(&selected),
            Input::fen_from_array(&pre_accept_selected),
            Input::fen_from_array(&baseline_selected),
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            selector_diag.last_return_stage,
            accepted,
            drainer_vulnerable,
            drainer_walk_vulnerable,
            head_plan
                .as_ref()
                .and_then(|plan| plan.compiled_chunks.first())
                .map(|chunk| Input::fen_from_array(chunk)),
            head_plan.as_ref().map(|plan| plan.head_family),
            head_plan.as_ref().map(|plan| plan.goal_family),
            game.fen(),
        );
        println!(
            "WHITE_SAFE_PROGRESS_ROOTS label={} selected=\"{}\" pre_accept=\"{}\" baseline=\"{}\" head=\"{}\"",
            label,
            format_root_probe(selected_root),
            format_root_probe(pre_accept_root),
            format_root_probe(baseline_root),
            format_root_probe(head_root),
        );
        for target in targets {
            let rank = scored_roots
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            println!(
                "WHITE_SAFE_PROGRESS_TARGET label={} target={} rank={:?}",
                label, target, rank
            );
        }
    }

    let traced_normal_v12_game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n06a0xn04/n02y0xn01s0xn01d0xe0xn03/n07xxmn03/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn02xxMn03/n11/n04A0xD0xS0xn04/n03E0xn03Y0xn03",
        false,
    )
    .expect("valid traced normal v12 white safe-progress fen");
    let traced_normal_v26_game = MonsGame::from_fen(
        "1 1 w 0 0 0 0 0 7 n11/n06a0xn04/n04y0xd0xe0xn04/n02s0xn01xxmn01xxmn04/n01E0xn02xxUxxmn01xxmn03/n10xxQ/n05xxMn01xxMn03/n06xxMn04/n02xxMn08/n05S0xn01Y0xn03/D0xn03A0xn06",
        false,
    )
    .expect("valid traced normal v26 white safe-progress fen");
    let retained_safe_fixture =
        primary_pro_fixture_by_id("primary_white_safe_progress_rerank_ply27");
    let retained_fast_screen_fixture =
        primary_pro_fixture_by_id("primary_white_fast_screen_opening_0_ply9");

    for (label, game, opening_book_driven, targets) in [
        (
            "traced_normal_duel_v12",
            &traced_normal_v12_game,
            false,
            &["l9,5;l8,5", "l10,7;l9,8"][..],
        ),
        (
            "traced_normal_duel_v26",
            &traced_normal_v26_game,
            false,
            &["l9,5;l8,5", "l9,5;l7,6;l7,7", "l10,0;l9,0"][..],
        ),
        (
            "primary_white_safe_progress_rerank_ply27",
            &retained_safe_fixture.game,
            retained_safe_fixture.opening_book_driven,
            &["l9,5;l8,5", "l10,7;l9,8"][..],
        ),
        (
            "primary_white_fast_screen_opening_0_ply9",
            &retained_fast_screen_fixture.game,
            retained_fast_screen_fixture.opening_book_driven,
            &["l9,5;l8,5", "l10,7;l9,8"][..],
        ),
    ] {
        println!(
            "WHITE_SAFE_PROGRESS_META label={} opening_book_driven={}",
            label, opening_book_driven
        );
        run_probe(label, game, SmartAutomovePreference::Pro, targets);
    }
}

#[test]
#[ignore = "diagnostic: compare traced white engine-disabled seam against nearby retained white surfaces"]
fn smart_automove_pro_white_engine_disabled_runtime_probe() {
    fn run_probe(label: &str, game: &MonsGame, mode: SmartAutomovePreference, targets: &[&str]) {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let (config, scored_roots, head_plan, forced_engine_inputs) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "runtime_pro_turn_engine_v30",
                mode,
                game,
            );
        let perspective = game.active_color;
        let selected = profile_decision_inputs("runtime_pro_turn_engine_v30", mode, game);
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let baseline_selected = profile_decision_inputs("runtime_current", mode, game);
        let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
            game,
            scored_roots.as_slice(),
            perspective,
            config,
        );
        let selected_root = scored_roots.iter().find(|root| root.inputs == selected);
        let pre_accept_root = scored_roots
            .iter()
            .find(|root| root.inputs == pre_accept_selected);
        let baseline_root = scored_roots
            .iter()
            .find(|root| root.inputs == baseline_selected);
        let head_root = head_plan.as_ref().and_then(|plan| {
            plan.compiled_chunks.first().and_then(|chunk| {
                scored_roots
                    .iter()
                    .find(|root| root.inputs.as_slice() == chunk.as_slice())
            })
        });
        let accepted = head_plan.as_ref().is_some_and(|plan| {
            MonsGameModel::accept_turn_engine_head_after_search(
                game,
                perspective,
                config,
                scored_roots.as_slice(),
                pre_accept_selected.as_slice(),
                plan,
            )
        });

        println!(
            "WHITE_ENGINE_DISABLED_RUNTIME label={} selected={} pre_accept={} baseline_selected={} forced_inputs={:?} stage={} accepted={} head={:?} head_family={:?} goal_family={:?} fen={}",
            label,
            Input::fen_from_array(&selected),
            Input::fen_from_array(&pre_accept_selected),
            Input::fen_from_array(&baseline_selected),
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            selector_diag.last_return_stage,
            accepted,
            head_plan
                .as_ref()
                .and_then(|plan| plan.compiled_chunks.first())
                .map(|chunk| Input::fen_from_array(chunk)),
            head_plan.as_ref().map(|plan| plan.head_family),
            head_plan.as_ref().map(|plan| plan.goal_family),
            game.fen(),
        );
        println!(
            "WHITE_ENGINE_DISABLED_RUNTIME_ROOTS label={} selected=\"{}\" pre_accept=\"{}\" baseline=\"{}\" head=\"{}\"",
            label,
            format_root_probe(selected_root),
            format_root_probe(pre_accept_root),
            format_root_probe(baseline_root),
            format_root_probe(head_root),
        );
        for target in targets {
            let rank = scored_roots
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            println!(
                "WHITE_ENGINE_DISABLED_RUNTIME_TARGET label={} target={} rank={:?}",
                label, target, rank
            );
        }
    }

    let traced_normal_v24_game = MonsGame::from_fen(
        "0 0 w 1 0 2 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn02xxMn03/n04S0xD0xn05/n05A0xY0xn04/n02E0xn08",
        false,
    )
    .expect("valid traced normal v24 white engine-disabled fen");
    let traced_normal_v83_game = MonsGame::from_fen(
        "0 0 w 1 0 2 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMn04/n05S0xY0xn04/n05D0xn05/n02E0xn01A0xn06",
        false,
    )
    .expect("valid traced normal v83 white engine-disabled fen");
    let traced_fast_v91_game = MonsGame::from_fen(
        "0 0 w 1 0 2 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMY0xn01xxMn03/n11/n04E0xD0xS0xn04/n04A0xn06",
        false,
    )
    .expect("valid traced fast v91 white engine-disabled fen");
    let traced_normal_v92_game = MonsGame::from_fen(
        "0 0 w 1 0 4 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMY0xn03/n05S0xn05/n04A0xD0xn05/n02E0xn08",
        false,
    )
    .expect("valid traced normal v92 white engine-disabled fen");
    let traced_fast_v92_game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n05S0xn05/n03E0xA0xD0xn02Y0xn02",
        false,
    )
    .expect("valid traced fast v92 white engine-disabled fen");
    let spirit_setup_fixture = primary_pro_fixture_by_id("primary_spirit_setup");
    let engine_disabled_opening_game = MonsGame::from_fen(
        "0 0 w 0 0 2 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04E0xn01D0xn04/n04A0xn01S0xY0xn03",
        false,
    )
    .expect("valid white engine-disabled opening fen");
    let loss_opening_a_game = MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n02E0xn01A0xD0xS0xY0xn03",
        false,
    )
    .expect("valid white engine-disabled loss opening a fen");

    run_probe(
        "traced_normal_duel_v24",
        &traced_normal_v24_game,
        SmartAutomovePreference::Pro,
        &["l9,5;l8,6", "l9,6;l8,6", "l8,5;l7,4"],
    );
    run_probe(
        "traced_normal_duel_v83",
        &traced_normal_v83_game,
        SmartAutomovePreference::Pro,
        &["l10,4;l9,4", "l8,6;l7,7", "l9,5;l8,4"],
    );
    run_probe(
        "traced_fast_duel_v91",
        &traced_fast_v91_game,
        SmartAutomovePreference::Pro,
        &["l9,5;l8,5", "l7,5;l6,4", "l9,4;l8,5"],
    );
    run_probe(
        "traced_normal_duel_v92",
        &traced_normal_v92_game,
        SmartAutomovePreference::Pro,
        &["l10,2;l9,1", "l7,7;l6,6", "l9,5;l8,4"],
    );
    run_probe(
        "traced_fast_duel_v92",
        &traced_fast_v92_game,
        SmartAutomovePreference::Pro,
        &["l10,4;l9,3", "l10,8;l9,7", "l9,5;l8,5"],
    );
    run_probe(
        "primary_spirit_setup",
        &spirit_setup_fixture.game,
        spirit_setup_fixture.mode,
        &["l9,7;l7,8;l7,7", "l9,5;l8,6", "l9,7;l7,8;l8,7"],
    );
    run_probe(
        "engine_disabled_opening",
        &engine_disabled_opening_game,
        SmartAutomovePreference::Pro,
        &["l9,6;l8,6", "l9,5;l8,6", "l8,5;l7,4"],
    );
    run_probe(
        "loss_opening_a",
        &loss_opening_a_game,
        SmartAutomovePreference::Pro,
        &["l10,6;l9,5", "l10,6;l9,6", "l10,5;l9,4"],
    );
}

#[test]
#[ignore = "diagnostic: inspect white forced-prepass families on traced duel boards"]
fn smart_automove_pro_white_fast_forced_prepass_probe() {
    fn run_probe(label: &str, game: &MonsGame, targets: &[&str]) {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let (config, scored_roots, head_plan, forced_engine_inputs) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "runtime_pro_turn_engine_v30",
                SmartAutomovePreference::Pro,
                game,
            );
        let perspective = game.active_color;
        let drainer_vulnerable = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        );
        let drainer_walk_vulnerable = MonsGameModel::is_own_drainer_walk_vulnerable_next_turn(
            game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        );
        let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
            game,
            scored_roots.as_slice(),
            perspective,
            config,
        );
        let selected = profile_decision_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            game,
        );
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let baseline_selected =
            profile_decision_inputs("runtime_current", SmartAutomovePreference::Pro, game);
        let selected_root = scored_roots.iter().find(|root| root.inputs == selected);
        let pre_accept_root = scored_roots
            .iter()
            .find(|root| root.inputs == pre_accept_selected);
        let baseline_root = scored_roots
            .iter()
            .find(|root| root.inputs == baseline_selected);
        let head_root = head_plan.as_ref().and_then(|plan| {
            plan.compiled_chunks.first().and_then(|chunk| {
                scored_roots
                    .iter()
                    .find(|root| root.inputs.as_slice() == chunk.as_slice())
            })
        });
        let accepted = head_plan.as_ref().is_some_and(|plan| {
            MonsGameModel::accept_turn_engine_head_after_search(
                game,
                perspective,
                config,
                scored_roots.as_slice(),
                pre_accept_selected.as_slice(),
                plan,
            )
        });

        println!(
            "WHITE_FAST_FORCED_PREPASS label={} selected={} pre_accept={} baseline_selected={} forced_inputs={:?} stage={} accepted={} drainer_vulnerable={} drainer_walk_vulnerable={} head={:?} fen={}",
            label,
            Input::fen_from_array(&selected),
            Input::fen_from_array(&pre_accept_selected),
            Input::fen_from_array(&baseline_selected),
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            selector_diag.last_return_stage,
            accepted,
            drainer_vulnerable,
            drainer_walk_vulnerable,
            head_plan
                .as_ref()
                .and_then(|plan| plan.compiled_chunks.first())
                .map(|chunk| Input::fen_from_array(chunk)),
            game.fen(),
        );
        println!(
            "WHITE_FAST_FORCED_PREPASS_ROOTS label={} selected=\"{}\" pre_accept=\"{}\" baseline=\"{}\" head=\"{}\"",
            label,
            format_root_probe(selected_root),
            format_root_probe(pre_accept_root),
            format_root_probe(baseline_root),
            format_root_probe(head_root),
        );
        for target in targets {
            let rank = scored_roots
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            println!(
                "WHITE_FAST_FORCED_PREPASS_TARGET label={} target={} rank={:?}",
                label, target, rank
            );
        }
        for (rank, root) in scored_roots.iter().enumerate().take(8) {
            println!(
                "WHITE_FAST_FORCED_PREPASS_TOP label={} rank={} fen={} root=\"{}\"",
                label,
                rank,
                Input::fen_from_array(&root.inputs),
                format_root_probe(Some(root)),
            );
        }
    }

    let traced_game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn01Y0xn03",
        false,
    )
    .expect("valid traced white fast forced-prepass fen");
    let traced_normal_v5_game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n06a0xn04/n02y0xn03d0xe0xn03/n04s0xn02xxmn03/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxME0xn06/n04D0xn01S0xn04/n04A0xn02Y0xn03",
        false,
    )
    .expect("valid traced white normal forced-prepass fen");
    let traced_pro_v15_game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n04D0Mn01A0xn04/n06S0xn04/n03E0xn03Y0xn03",
        false,
    )
    .expect("valid traced white pro v15 forced-prepass fen");
    let traced_normal_v18_game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n07e0xn03/n02y0xn02s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn06/n04S0xn02xxMn03/n03E0xn02D0xn04/n04A0xn02Y0xn03",
        false,
    )
    .expect("valid traced white normal v18 forced-prepass fen");
    let traced_normal_v28_game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n07e0xn03/n02y0xn02s0xn01a0xn03/n06d0xxxmn03/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n04D0Mn06/n03E0xA0xS0xn05/n07Y0xn03",
        false,
    )
    .expect("valid traced white normal v28 forced-prepass fen");
    let traced_normal_v93_game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn02Y0xn03",
        false,
    )
    .expect("valid traced white normal v93 forced-prepass fen");
    let retained_fixture = primary_pro_fixture_by_id("primary_white_fast_screen_opening_0_ply9");

    for (label, game, targets) in [
        (
            "traced_fast_duel",
            &traced_game,
            &["l8,4;l8,5", "l8,4;l7,3", "l8,4;l8,3"][..],
        ),
        (
            "traced_normal_duel_v5",
            &traced_normal_v5_game,
            &["l9,4;l8,5", "l9,4;l8,3"][..],
        ),
        (
            "traced_pro_duel_v15",
            &traced_pro_v15_game,
            &["l8,4;l7,3", "l8,4;l9,3", "l8,4;l8,3"][..],
        ),
        (
            "traced_normal_duel_v18",
            &traced_normal_v18_game,
            &["l9,6;l8,5", "l9,6;l8,7", "l9,6;l8,6"][..],
        ),
        (
            "traced_normal_duel_v28",
            &traced_normal_v28_game,
            &["l8,4;l8,5", "l8,4;l8,3", "l8,4;l7,3"][..],
        ),
        (
            "traced_normal_duel_v93",
            &traced_normal_v93_game,
            &["l8,4;l8,5", "l8,4;l7,3", "l8,4;l7,5"][..],
        ),
        (
            "primary_white_fast_screen_opening_0_ply9",
            &retained_fixture.game,
            &["l8,4;l8,5", "l8,4;l7,3", "l8,4;l8,3"][..],
        ),
    ] {
        run_probe(label, game, targets);
    }
}

#[test]
#[ignore = "diagnostic: isolate cache-driven move divergence on traced white normal duel board"]
fn smart_automove_pro_white_cache_divergence_probe() {
    #[derive(Clone, Copy)]
    struct CacheResetCase {
        label: &'static str,
        clear: fn(),
    }

    fn clear_none() {}

    fn clear_exact_only() {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
    }

    fn clear_planner_only() {
        clear_turn_opportunity_plan_cache();
    }

    fn clear_engine_only() {
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
    }

    fn clear_selector_only() {
        clear_turn_engine_selector_diagnostics();
    }

    fn clear_all_runtime_caches() {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_opportunity_plan_cache();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();
    }

    fn replay_to_target(
        opening_fen: &str,
        target_board_fen: &str,
        candidate_profile: &str,
        opponent_profile: &str,
        opponent_mode: SmartAutomovePreference,
        candidate_is_white: bool,
        max_plies: usize,
    ) -> MonsGame {
        let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
        clear_all_runtime_caches();

        for _ in 0..max_plies {
            if game.fen() == target_board_fen {
                return game;
            }
            let (candidate_to_move, inputs) = profile_duel_turn_inputs(
                &game,
                candidate_profile,
                opponent_profile,
                opponent_mode,
                candidate_is_white,
            )
            .unwrap_or_else(|result| {
                panic!(
                    "duel reached terminal result {:?} before target board {}",
                    result, target_board_fen
                )
            });
            assert!(
                !inputs.is_empty(),
                "duel produced no legal move before target board {}; candidate_to_move={}",
                target_board_fen,
                candidate_to_move
            );
            assert!(
                matches!(game.process_input(inputs, false, false), Output::Events(_)),
                "duel produced illegal move before target board {}",
                target_board_fen
            );
        }

        panic!(
            "target board {} not reached within {} plies from {}",
            target_board_fen, max_plies, opening_fen
        );
    }

    let opening_fen =
        "0 0 w 0 0 3 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04E0xn06/n04A0xD0xS0xY0xn03";
    let target_board_fen =
        "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn02Y0xn03";

    for case in [
        CacheResetCase {
            label: "warm",
            clear: clear_none,
        },
        CacheResetCase {
            label: "exact_only",
            clear: clear_exact_only,
        },
        CacheResetCase {
            label: "planner_only",
            clear: clear_planner_only,
        },
        CacheResetCase {
            label: "engine_only",
            clear: clear_engine_only,
        },
        CacheResetCase {
            label: "selector_only",
            clear: clear_selector_only,
        },
        CacheResetCase {
            label: "all",
            clear: clear_all_runtime_caches,
        },
    ] {
        let game = replay_to_target(
            opening_fen,
            target_board_fen,
            "runtime_pro_turn_engine_v30",
            "runtime_current",
            SmartAutomovePreference::Normal,
            true,
            24,
        );
        (case.clear)();
        let selected = Input::fen_from_array(&profile_runtime_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        ));
        println!(
            "WHITE_CACHE_DIVERGENCE case={} selected={}",
            case.label,
            selected,
        );
    }
}

#[test]
#[ignore = "diagnostic: inspect white score-route accepted-head family on traced duel boards"]
fn smart_automove_pro_white_score_route_probe() {
    fn run_probe(label: &str, game: &MonsGame, targets: &[&str]) {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let (config, scored_roots, head_plan, forced_engine_inputs) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "runtime_pro_turn_engine_v30",
                SmartAutomovePreference::Pro,
                game,
            );
        let perspective = game.active_color;
        let drainer_vulnerable = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        );
        let drainer_walk_vulnerable = MonsGameModel::is_own_drainer_walk_vulnerable_next_turn(
            game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        );
        let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
            game,
            scored_roots.as_slice(),
            perspective,
            config,
        );
        let selected = profile_decision_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            game,
        );
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let baseline_selected =
            profile_decision_inputs("runtime_current", SmartAutomovePreference::Pro, game);
        let selected_root = scored_roots.iter().find(|root| root.inputs == selected);
        let pre_accept_root = scored_roots
            .iter()
            .find(|root| root.inputs == pre_accept_selected);
        let baseline_root = scored_roots
            .iter()
            .find(|root| root.inputs == baseline_selected);
        let head_root = head_plan.as_ref().and_then(|plan| {
            plan.compiled_chunks.first().and_then(|chunk| {
                scored_roots
                    .iter()
                    .find(|root| root.inputs.as_slice() == chunk.as_slice())
            })
        });
        let accepted = head_plan.as_ref().is_some_and(|plan| {
            MonsGameModel::accept_turn_engine_head_after_search(
                game,
                perspective,
                config,
                scored_roots.as_slice(),
                pre_accept_selected.as_slice(),
                plan,
            )
        });
        let pre_accept_utility = pre_accept_root.map(|root| {
            let family = MonsGameModel::turn_engine_root_evaluation_family(root);
            MonsGameModel::turn_engine_root_plan_utility(game, root, perspective, config, family)
        });
        let baseline_utility = baseline_root.map(|root| {
            let family = MonsGameModel::turn_engine_root_evaluation_family(root);
            MonsGameModel::turn_engine_root_plan_utility(game, root, perspective, config, family)
        });

        println!(
            "WHITE_SCORE_ROUTE label={} selected={} pre_accept={} baseline_selected={} forced_inputs={:?} stage={} accepted={} drainer_vulnerable={} drainer_walk_vulnerable={} head={:?} head_family={:?} goal_family={:?} plan_utility={:?} head_utility={:?} pre_accept_utility={:?} baseline_utility={:?} fen={}",
            label,
            Input::fen_from_array(&selected),
            Input::fen_from_array(&pre_accept_selected),
            Input::fen_from_array(&baseline_selected),
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            selector_diag.last_return_stage,
            accepted,
            drainer_vulnerable,
            drainer_walk_vulnerable,
            head_plan
                .as_ref()
                .and_then(|plan| plan.compiled_chunks.first())
                .map(|chunk| Input::fen_from_array(chunk)),
            head_plan.as_ref().map(|plan| plan.head_family),
            head_plan.as_ref().map(|plan| plan.goal_family),
            head_plan.as_ref().map(|plan| plan.utility),
            head_plan.as_ref().map(|plan| plan.head_utility),
            pre_accept_utility,
            baseline_utility,
            game.fen(),
        );
        println!(
            "WHITE_SCORE_ROUTE_ROOTS label={} selected=\"{}\" pre_accept=\"{}\" baseline=\"{}\" head=\"{}\"",
            label,
            format_root_probe(selected_root),
            format_root_probe(pre_accept_root),
            format_root_probe(baseline_root),
            format_root_probe(head_root),
        );
        for target in targets {
            let rank = scored_roots
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            println!(
                "WHITE_SCORE_ROUTE_TARGET label={} target={} rank={:?}",
                label, target, rank
            );
        }
        for (rank, root) in scored_roots.iter().enumerate().take(8) {
            println!(
                "WHITE_SCORE_ROUTE_TOP label={} rank={} fen={} root=\"{}\"",
                label,
                rank,
                Input::fen_from_array(&root.inputs),
                format_root_probe(Some(root)),
            );
        }
    }

    let traced_game = MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04D0Mn01xxMn04/n11/n07S0xn01Y0xn01/n03E0xA0xn06",
        false,
    )
    .expect("valid traced white score-route fen");
    let traced_v6_game = MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 3 n11/n02y0xn01s0xa0xd0xn01e0xn02/n07xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n04D0xn06/n04E0xA0xS0xn04/n07Y0xn03",
        false,
    )
    .expect("valid traced white score-route v6 fen");
    let traced_v15_normal_game = MonsGame::from_fen(
        "0 0 w 0 0 0 0 1 5 n05d1xn05/n05a0xn01e0xn03/n04xxmn02xxmn03/n06s0xn04/n03xxmn01xxmn01xxmn01S0xn01/y0xn04xxUn05/n05xxMn01xxMn03/n02xxMn01xxMn01xxMn04/n01E0xn09/n11/n04A0xD0xn03Y0xn01",
        false,
    )
    .expect("valid traced white score-route v15 normal fen");
    let traced_v73_fast_game = MonsGame::from_fen(
        "0 0 w 0 0 0 0 0 5 n05d1xn01e0xn03/n05s0xn01a0xxxmn02/n11/n03xxmxxmn06/n05xxmn01xxmn01Y0xn01/n05xxUn05/n01y0xn01xxMn01xxMn05/n03xxMn02xxMxxMn03/n11/n05S0xn05/n03E1xA0xD1xn05",
        false,
    )
    .expect("valid traced white score-route v73 fast fen");
    let traced_full_reliability_normal_game = MonsGame::from_fen(
        "1 0 w 0 0 0 0 0 7 n11/n06a0xn01e0xn02/n05d0mn05/n03xxmxxmn02xxmn03/n05xxmxxUn04/y0xn03xxMn01s0xn03xxQ/n06Y0xxxMn03/n03xxMn07/n05S0xxxMn04/n04A0xn06/D0xn01E0xn08",
        false,
    )
    .expect("valid traced full reliability normal white score-route fen");
    let traced_flat_pro_nonwin_game = MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 3 n03y0xn03e0xn03/n05s0xa0xn01d0mn02/n11/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n01E0xn05Y0xn03/n04D0xn01S0xn04/n04A0xn06",
        false,
    )
    .expect("valid traced flat pro non-win white score-route fen");
    let retained_fixture = primary_pro_fixture_by_id("primary_harvest_white_score_route_win_a");
    let retained_b_fixture = primary_pro_fixture_by_id("primary_harvest_white_score_route_win_b");
    let retained_v10_fixture =
        primary_pro_fixture_by_id("primary_white_safe_progress_rerank_ply27");

    for (label, game, targets) in [
        (
            "traced_pro_duel_v5",
            &traced_game,
            &["l7,4;l8,3", "l9,7;l7,6;l8,7"][..],
        ),
        (
            "traced_pro_duel_v6",
            &traced_v6_game,
            &["l9,6;l8,4;l7,4", "l9,6;l7,4;l7,3"][..],
        ),
        (
            "primary_white_safe_progress_rerank_ply27",
            &retained_v10_fixture.game,
            &["l9,4;l8,3", "l5,2;l4,1"][..],
        ),
        (
            "traced_normal_duel_v15",
            &traced_v15_normal_game,
            &["l10,5;l9,4", "l4,9;l4,7;l5,7"][..],
        ),
        (
            "traced_fast_duel_v73",
            &traced_v73_fast_game,
            &[
                "l10,4;l9,3",
                "l9,5;l7,6;l8,7",
                "l10,4;l9,4",
                "l9,5;l7,4;l8,3",
            ][..],
        ),
        (
            "traced_full_reliability_normal",
            &traced_full_reliability_normal_game,
            &[
                "l10,0;l9,1",
                "l8,5;l7,3;l8,2",
                "l9,4;l8,3",
                "l9,4;l9,3",
                "l10,2;l9,3",
            ][..],
        ),
        (
            "traced_flat_pro_nonwin_v1",
            &traced_flat_pro_nonwin_game,
            &["l9,6;l7,4;l8,3", "l8,7;l7,8"][..],
        ),
        (
            "primary_harvest_white_score_route_win_a",
            &retained_fixture.game,
            &["l9,6;l7,4;l8,3", "l9,6;l7,6;l7,7"][..],
        ),
        (
            "primary_harvest_white_score_route_win_b",
            &retained_b_fixture.game,
            &["l10,5;l9,4", "l4,9;l4,7;l5,7"][..],
        ),
    ] {
        run_probe(label, game, targets);
    }
}

#[test]
#[ignore = "diagnostic: inspect white mana sibling family on retained and traced duel boards"]
fn smart_automove_pro_white_mana_sibling_probe() {
    fn run_probe(label: &str, game: &MonsGame, targets: &[&str]) {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();
        clear_turn_engine_plan_cache();
        clear_turn_engine_diagnostics();
        clear_turn_engine_selector_diagnostics();

        let (config, scored_roots, head_plan, forced_engine_inputs) =
            profile_runtime_scored_roots_with_forced_engine_inputs(
                "runtime_pro_turn_engine_v30",
                SmartAutomovePreference::Pro,
                game,
            );
        let perspective = game.active_color;
        let drainer_vulnerable = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        );
        let drainer_walk_vulnerable = MonsGameModel::is_own_drainer_walk_vulnerable_next_turn(
            game,
            perspective,
            config.enable_enhanced_drainer_vulnerability,
        );
        let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
            game,
            scored_roots.as_slice(),
            perspective,
            config,
        );
        let selected = profile_decision_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            game,
        );
        let selector_diag = turn_engine_selector_diagnostics_snapshot();
        let baseline_selected =
            profile_decision_inputs("runtime_current", SmartAutomovePreference::Pro, game);
        let selected_root = scored_roots.iter().find(|root| root.inputs == selected);
        let pre_accept_root = scored_roots
            .iter()
            .find(|root| root.inputs == pre_accept_selected);
        let baseline_root = scored_roots
            .iter()
            .find(|root| root.inputs == baseline_selected);
        let head_root = head_plan.as_ref().and_then(|plan| {
            plan.compiled_chunks.first().and_then(|chunk| {
                scored_roots
                    .iter()
                    .find(|root| root.inputs.as_slice() == chunk.as_slice())
            })
        });
        let accepted = head_plan.as_ref().is_some_and(|plan| {
            MonsGameModel::accept_turn_engine_head_after_search(
                game,
                perspective,
                config,
                scored_roots.as_slice(),
                pre_accept_selected.as_slice(),
                plan,
            )
        });
        let pre_accept_utility = pre_accept_root.map(|root| {
            let family = MonsGameModel::turn_engine_root_evaluation_family(root);
            MonsGameModel::turn_engine_root_plan_utility(game, root, perspective, config, family)
        });
        let baseline_utility = baseline_root.map(|root| {
            let family = MonsGameModel::turn_engine_root_evaluation_family(root);
            MonsGameModel::turn_engine_root_plan_utility(game, root, perspective, config, family)
        });

        println!(
            "WHITE_MANA_SIBLING label={} selected={} pre_accept={} baseline_selected={} forced_inputs={:?} stage={} accepted={} drainer_vulnerable={} drainer_walk_vulnerable={} head={:?} head_family={:?} goal_family={:?} plan_utility={:?} head_utility={:?} pre_accept_utility={:?} baseline_utility={:?} fen={}",
            label,
            Input::fen_from_array(&selected),
            Input::fen_from_array(&pre_accept_selected),
            Input::fen_from_array(&baseline_selected),
            forced_engine_inputs
                .as_ref()
                .map(|inputs| Input::fen_from_array(inputs)),
            selector_diag.last_return_stage,
            accepted,
            drainer_vulnerable,
            drainer_walk_vulnerable,
            head_plan
                .as_ref()
                .and_then(|plan| plan.compiled_chunks.first())
                .map(|chunk| Input::fen_from_array(chunk)),
            head_plan.as_ref().map(|plan| plan.head_family),
            head_plan.as_ref().map(|plan| plan.goal_family),
            head_plan.as_ref().map(|plan| plan.utility),
            head_plan.as_ref().map(|plan| plan.head_utility),
            pre_accept_utility,
            baseline_utility,
            game.fen(),
        );
        println!(
            "WHITE_MANA_SIBLING_ROOTS label={} selected=\"{}\" pre_accept=\"{}\" baseline=\"{}\" head=\"{}\"",
            label,
            format_root_probe(selected_root),
            format_root_probe(pre_accept_root),
            format_root_probe(baseline_root),
            format_root_probe(head_root),
        );
        for target in targets {
            let rank = scored_roots
                .iter()
                .position(|root| Input::fen_from_array(&root.inputs) == *target);
            println!(
                "WHITE_MANA_SIBLING_TARGET label={} target={} rank={:?}",
                label, target, rank
            );
        }
        for (rank, root) in scored_roots.iter().enumerate().take(8) {
            println!(
                "WHITE_MANA_SIBLING_TOP label={} rank={} fen={} root=\"{}\"",
                label,
                rank,
                Input::fen_from_array(&root.inputs),
                format_root_probe(Some(root)),
            );
        }
    }

    let retained_fixture = primary_pro_fixture_by_id("primary_white_mana_sibling_ply9");
    let traced_normal_game = MonsGame::from_fen(
        "0 0 w 0 0 3 0 0 3 n07e0xn03/n03s0xn01a0xn05/n01y0xn03d0xn05/n03xxmxxmn01xxmn04/n05xxmn01xxmn03/E0Bn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n06D0xY0xn03/n04A0xn01S0xn04",
        false,
    )
    .expect("valid traced white mana sibling normal fen");
    let traced_normal_v92_game = MonsGame::from_fen(
        "0 0 w 1 0 4 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMY0xn03/n05S0xn05/n04A0xD0xn05/n02E0xn08",
        false,
    )
    .expect("valid traced white mana sibling normal v92 fen");
    let traced_fast_v92_game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n05S0xn05/n03E0xA0xD0xn02Y0xn02",
        false,
    )
    .expect("valid traced white mana sibling fast v92 fen");
    let traced_fast_v94_game = MonsGame::from_fen(
        "0 0 w 1 0 4 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn03Y0xn03/n03E0xn01S0xn05/n04A0xD0xn05",
        false,
    )
    .expect("valid traced white mana sibling fast v94 fen");
    let traced_normal_nonwin_game = MonsGame::from_fen(
        "1 0 w 1 0 1 0 0 5 n11/n05a0xn02e0xn02/n03y0xd0ms0xn05/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n11/n03xxMn01A0xn05/n01D0xn04Y0xS0xn03/n03E0xn07",
        false,
    )
    .expect("valid traced normal non-win white mana sibling fen");

    for (label, game, targets) in [
        (
            "primary_white_mana_sibling_ply9",
            &retained_fixture.game,
            &["l5,0;l5,1", "l5,0;l4,1"][..],
        ),
        (
            "traced_normal_duel_v6",
            &traced_normal_game,
            &["l5,0;l6,1", "l5,0;l4,1"][..],
        ),
        (
            "traced_normal_duel_v92",
            &traced_normal_v92_game,
            &["l10,2;l9,1", "l7,7;l6,6", "l9,5;l8,4"][..],
        ),
        (
            "traced_fast_duel_v92",
            &traced_fast_v92_game,
            &["l10,4;l9,3", "l10,8;l9,7", "l9,5;l8,5"][..],
        ),
        (
            "traced_fast_duel_v94",
            &traced_fast_v94_game,
            &["l9,5;l8,4", "l10,4;l9,4", "l8,7;l7,7"][..],
        ),
        (
            "traced_normal_flat_nonwin_v1",
            &traced_normal_nonwin_game,
            &["l9,1;l8,2", "l9,7;l8,7"][..],
        ),
    ] {
        run_probe(label, game, targets);
    }
}

#[test]
#[ignore = "diagnostic: inspect reply-risk shortlist arbitration on the fast black non-win mana board"]
fn smart_automove_pro_black_fast_nonwin_reply_guard_probe() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 1 0 0 4 n06a0xn04/n05s0xd0xe0xn03/n07xxmn03/n02y0xxxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n11/n02E0xA0xn01S0xn01Y0xn03/D0xn10",
        false,
    )
    .expect("valid fast black non-win reply-guard fen");
    let (config, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let candidate_indices = MonsGameModel::filtered_root_candidate_indices(
        &game,
        scored_roots.as_slice(),
        game.active_color,
        config,
    );
    let shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
        scored_roots.as_slice(),
        candidate_indices.as_slice(),
        config,
    );
    let reply_guarded = MonsGameModel::pick_root_move_with_reply_risk_guard_from_shortlist(
        &game,
        scored_roots.as_slice(),
        shortlist.as_slice(),
        Some(candidate_indices.as_slice()),
        game.active_color,
        config,
    );
    println!(
        "BLACK_FAST_NONWIN_REPLY_GUARD shortlist_competes={} candidate_indices={:?} shortlist={:?} reply_guarded={:?}",
        MonsGameModel::reply_risk_shortlist_has_black_mana_window_progress_competition(
            &game,
            scored_roots.as_slice(),
            shortlist.as_slice(),
            config,
        ),
        candidate_indices
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        reply_guarded.map(|index| Input::fen_from_array(&scored_roots[index].inputs)),
    );
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_late_black_plain_spirit_progress_head_without_concrete_gain()
{
    let fixture = primary_pro_fixture_by_id("primary_black_late_accepted_head_ply4");
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
    let head_plan = head_plan.expect("late black fixture should retain a head plan");
    let head_inputs = head_plan
        .compiled_chunks
        .first()
        .expect("head plan should include a first chunk");
    let pre_accept_root = scored_roots
        .iter()
        .find(|root| root.inputs == pre_accept_selected)
        .expect("pre-accept selected root should be present");
    let head_root = scored_roots
        .iter()
        .find(|root| root.inputs.as_slice() == head_inputs.as_slice())
        .expect("head root should be present");
    let pre_accept_family = MonsGameModel::turn_engine_root_evaluation_family(pre_accept_root);
    let pre_accept_utility = MonsGameModel::turn_engine_root_plan_utility(
        &fixture.game,
        pre_accept_root,
        fixture.game.active_color,
        config,
        pre_accept_family,
    );
    let accepted = MonsGameModel::accept_turn_engine_head_after_search(
        &fixture.game,
        fixture.game.active_color,
        config,
        scored_roots.as_slice(),
        pre_accept_selected.as_slice(),
        &head_plan,
    );

    assert_eq!(
        forced_engine_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs)),
        Some("l1,5;l1,7;l0,7".to_string()),
    );
    assert_eq!(Input::fen_from_array(&pre_accept_selected), "l3,2;l4,1");
    assert_eq!(Input::fen_from_array(head_inputs), "l1,5;l1,7;l0,7");
    assert!(pre_accept_root.score > head_root.score);
    assert!(!pre_accept_root.spirit_development);
    assert!(head_root.spirit_development);
    assert!(head_root.supermana_progress);
    assert!(!pre_accept_root.supermana_progress);
    assert!(!head_root.spirit_same_turn_score_setup_now);
    assert!(!head_root.spirit_own_mana_setup_now);
    assert!(!head_plan
        .utility
        .improves_non_score_override_axes(pre_accept_utility));
    assert!(
        !accepted,
        "a late black plain spirit progress head should not override a stronger safe non-spirit root without concrete setup, tactical, or primary-axis gain: selected_utility={:?} head_utility={:?} plan_utility={:?}",
        pre_accept_utility,
        head_plan.head_utility,
        head_plan.utility,
    );
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game),
        "l3,2;l4,1",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_white_fast_deferred_recovery_progress_head_without_concrete_gain(
) {
    let fixture = primary_pro_fixture_by_id("primary_white_fast_accepted_head_ply13");
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
    let head_plan = head_plan.expect("white fast fixture should retain a head plan");
    let head_inputs = head_plan
        .compiled_chunks
        .first()
        .expect("head plan should include a first chunk");
    let pre_accept_root = scored_roots
        .iter()
        .find(|root| root.inputs == pre_accept_selected)
        .expect("pre-accept selected root should be present");
    let head_root = scored_roots
        .iter()
        .find(|root| root.inputs.as_slice() == head_inputs.as_slice())
        .expect("head root should be present");
    let accepted = MonsGameModel::accept_turn_engine_head_after_search(
        &fixture.game,
        fixture.game.active_color,
        config,
        scored_roots.as_slice(),
        pre_accept_selected.as_slice(),
        &head_plan,
    );

    assert_eq!(
        forced_engine_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs)),
        Some("l9,4;l8,4".to_string()),
    );
    assert_eq!(Input::fen_from_array(&pre_accept_selected), "l8,7;l7,8");
    assert_eq!(Input::fen_from_array(head_inputs), "l9,4;l8,4");
    assert!(pre_accept_root.own_drainer_vulnerable);
    assert!(head_root.own_drainer_vulnerable);
    assert!(!pre_accept_root.supermana_progress);
    assert!(head_root.supermana_progress);
    assert!(!head_root.classes.drainer_safety_recover);
    assert_eq!(head_plan.goal_family, TurnPlanFamily::DrainerSafetyRecovery);
    assert!(
        !accepted,
        "a deferred unsafe progress head should not override an unsafe non-progress root when the head itself brings no concrete immediate recovery: head_utility={:?} plan_utility={:?}",
        head_plan.head_utility,
        head_plan.utility,
    );
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game),
        "l8,7;l7,8",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_v30_white_vulnerable_progress_head_flat_nonwin_normal_root(
) {
    let game = MonsGame::from_fen(
        "1 0 w 1 0 1 0 0 5 n11/n05a0xn02e0xn02/n03y0xd0ms0xn05/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n11/n03xxMn01A0xn05/n01D0xn04Y0xS0xn03/n03E0xn07",
        false,
    )
    .expect("white vulnerable progress flat non-win fen should be valid");
    let (config, scored_roots, head_plan, forced_engine_inputs) =
        profile_runtime_scored_roots_with_forced_engine_inputs(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game,
        );
    let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
        &game,
        scored_roots.as_slice(),
        game.active_color,
        config,
    );
    let head_plan = head_plan.expect("white vulnerable progress board should retain a head plan");
    let head_inputs = head_plan
        .compiled_chunks
        .first()
        .expect("head plan should include a first chunk");
    let pre_accept_root = scored_roots
        .iter()
        .find(|root| root.inputs == pre_accept_selected)
        .expect("pre-accept selected root should be present");
    let head_root = scored_roots
        .iter()
        .find(|root| root.inputs.as_slice() == head_inputs.as_slice())
        .expect("head root should be present");
    let accepted = MonsGameModel::accept_turn_engine_head_after_search(
        &game,
        game.active_color,
        config,
        scored_roots.as_slice(),
        pre_accept_selected.as_slice(),
        &head_plan,
    );

    assert_eq!(
        forced_engine_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs)),
        Some("l9,1;l8,2".to_string()),
    );
    assert_eq!(Input::fen_from_array(&pre_accept_selected), "l9,7;l8,7");
    assert_eq!(Input::fen_from_array(head_inputs), "l9,1;l8,2");
    assert!(pre_accept_root.own_drainer_vulnerable);
    assert!(head_root.own_drainer_vulnerable);
    assert!(!pre_accept_root.supermana_progress);
    assert!(head_root.supermana_progress);
    assert!(!accepted);
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", SmartAutomovePreference::Pro, &game),
        "l9,7;l8,7",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_accepts_v30_white_head_flat_nonwin_normal_root() {
    let game = MonsGame::from_fen(
        "1 0 w 0 0 1 0 0 9 n02a0xy1xn07/n01d0mn09/n02xxmn02s0xn01e0xn03/n03xxmn03xxmn03/E0xn03xxmn06/n05xxUn04xxQ/n05xxMn01xxMn03/n07S0xn03/n02xxMxxMn01A0xn05/n04D0xn01Y0xn04/n11",
        false,
    )
    .expect("white flat nonwin normal accepted-head fen should be valid");
    let (config, scored_roots, head_plan, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
        &game,
        scored_roots.as_slice(),
        game.active_color,
        config,
    );
    let head_plan = head_plan.expect("white flat nonwin normal board should keep a head plan");
    let head_inputs = head_plan
        .compiled_chunks
        .first()
        .expect("head plan should include a first chunk");
    let accepted = MonsGameModel::accept_turn_engine_head_after_search(
        &game,
        game.active_color,
        config,
        scored_roots.as_slice(),
        pre_accept_selected.as_slice(),
        &head_plan,
    );
    println!(
        "WHITE_HEAD_FLAT_NONWIN pre_accept={} head={} accepted={} pre_accept_root=\"{}\" head_root=\"{}\"",
        Input::fen_from_array(&pre_accept_selected),
        Input::fen_from_array(head_inputs),
        accepted,
        format_root_probe(scored_roots.iter().find(|root| root.inputs == pre_accept_selected)),
        format_root_probe(
            scored_roots
                .iter()
                .find(|root| root.inputs.as_slice() == head_inputs.as_slice())
        ),
    );
    assert_eq!(Input::fen_from_array(&pre_accept_selected), "l7,7;l5,5;l5,6");
    assert_eq!(Input::fen_from_array(head_inputs), "l7,7;l5,5;l6,4");
    assert!(accepted);
    assert_eq!(
        profile_decision_move_fen(
            "runtime_pro_turn_engine_v30",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l7,7;l5,5;l6,4"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_flat_nonwin_normal_root() {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 6 n11/n05d0xa0xe0xn03/n05s0xxxmn04/n02xxmxxmy0xn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n01E0xn01xxMY0xn01S0xxxMn03/n04xxMn06/n05D0Mn02xxMn02/n05A0xn05/n11",
        false,
    )
    .expect("black flat nonwin normal fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let (config, scored_roots, head_plan, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
        &game,
        scored_roots.as_slice(),
        game.active_color,
        config,
    );
    let candidate_indices = MonsGameModel::filtered_root_candidate_indices(
        &game,
        scored_roots.as_slice(),
        game.active_color,
        config,
    );
    let shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
        scored_roots.as_slice(),
        candidate_indices.as_slice(),
        config,
    );
    let head_inputs = head_plan
        .as_ref()
        .and_then(|plan| plan.compiled_chunks.first())
        .cloned();
    let head_root = head_inputs
        .as_ref()
        .and_then(|inputs| scored_roots.iter().find(|root| root.inputs == *inputs));
    println!(
        "BLACK_NORMAL_FLAT_NONWIN pre_accept={} head={:?} candidate_indices={:?} shortlist={:?} pre_accept_root=\"{}\" head_root=\"{}\" baseline_root=\"{}\"",
        Input::fen_from_array(&pre_accept_selected),
        head_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs)),
        candidate_indices
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        format_root_probe(scored_roots.iter().find(|root| root.inputs == pre_accept_selected)),
        format_root_probe(head_root),
        format_root_probe(
            scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == "l2,5;l1,7;l2,7")
        ),
    );
    clear_turn_engine_selector_diagnostics();
    let runtime_selected = profile_decision_move_fen(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    println!(
        "BLACK_NORMAL_FLAT_NONWIN_ADVISOR selected={} advisor={:?}",
        runtime_selected,
        pro_v2_root_advisor_decision_snapshot(),
    );
    assert_eq!(
        runtime_selected,
        "l2,5;l1,7;l2,7"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_flat_nonwin_fast_root() {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 5 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/n05xxUn04xxQ/n02y0xxxMn01xxMn01xxMn03/n04xxMn06/n03E0xA0xn03xxMn02/n06S0xn04/n05D2xn03Y0xn01",
        false,
    )
    .expect("black flat nonwin fast fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let (config, scored_roots, head_plan, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let pre_accept_selected = MonsGameModel::pick_root_move_with_exploration(
        &game,
        scored_roots.as_slice(),
        game.active_color,
        config,
    );
    let candidate_indices = MonsGameModel::filtered_root_candidate_indices(
        &game,
        scored_roots.as_slice(),
        game.active_color,
        config,
    );
    let shortlist = MonsGameModel::reply_risk_guard_shortlist_indices(
        scored_roots.as_slice(),
        candidate_indices.as_slice(),
        config,
    );
    let head_inputs = head_plan
        .as_ref()
        .and_then(|plan| plan.compiled_chunks.first())
        .cloned();
    println!(
        "BLACK_FAST_FLAT_NONWIN pre_accept={} head={:?} candidate_indices={:?} shortlist={:?} pre_accept_root=\"{}\" head_root=\"{}\" baseline_root=\"{}\"",
        Input::fen_from_array(&pre_accept_selected),
        head_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs)),
        candidate_indices
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        shortlist
            .iter()
            .map(|index| Input::fen_from_array(&scored_roots[*index].inputs))
            .collect::<Vec<_>>(),
        format_root_probe(scored_roots.iter().find(|root| root.inputs == pre_accept_selected)),
        format_root_probe(
            head_inputs
                .as_ref()
                .and_then(|inputs| scored_roots.iter().find(|root| root.inputs == *inputs))
        ),
        format_root_probe(
            scored_roots
                .iter()
                .find(|root| Input::fen_from_array(&root.inputs) == "l1,5;l3,3;l2,2")
        ),
    );
    clear_turn_engine_selector_diagnostics();
    let runtime_selected = profile_decision_move_fen(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    println!(
        "BLACK_FAST_FLAT_NONWIN_ADVISOR selected={} advisor={:?}",
        runtime_selected,
        pro_v2_root_advisor_decision_snapshot(),
    );
    assert_eq!(
        runtime_selected,
        "l1,5;l3,3;l2,2"
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_late_head_duel_normal_root() {
    let game = primary_pro_fixture_by_id("primary_black_late_accepted_head_ply4").game;

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);

    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );
    println!(
        "BLACK_LATE_HEAD_DUEL_NORMAL current_selected={} context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} probe={:?} advisor={:?}",
        current_selected,
        exact_opportunity_context_probe(&game),
        legacy_selected,
        legacy_full_pool_selected,
        legacy_candidates,
        legacy_full_pool,
        probe,
        advisor
    );
    assert_eq!(probe.selected_input_fen, "l3,2;l4,1");
    assert_eq!(probe.pre_accept_input_fen, "l3,2;l4,1");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l1,5;l1,7;l0,7"));
    assert!(!probe.head_accepted);
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_recovery_duel_fast_root() {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 3 0 0 4 n06a0xn04/n06d0xe0xn03/n04s0xn02xxmn03/n03xxmn07/n01y0xn01xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn03xxMn04/n03xxMD0xn06/n03A0xE0xn01S0xn04/n08Y0xn02",
        false,
    )
    .expect("black fast recovery duel fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);

    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );
    println!(
        "BLACK_RECOVERY_DUEL_FAST current_selected={} context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} probe={:?} advisor={:?}",
        current_selected,
        exact_opportunity_context_probe(&game),
        legacy_selected,
        legacy_full_pool_selected,
        legacy_candidates,
        legacy_full_pool,
        probe,
        advisor
    );
    assert_eq!(probe.selected_input_fen, "l4,1;l5,0;mb");
    assert_eq!(probe.pre_accept_input_fen, "l1,6;l0,5");
    assert_eq!(probe.selector_last_stage, "engine_disabled");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l1,6;l0,5"));
    assert!(probe.head_accepted);
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_spirit_bridge_duel_fast_root() {
    let game = MonsGame::from_fen(
        "1 1 b 0 0 3 1 0 8 n10d0x/n07a0xn03/n05s0xn05/n02xxmxxmy0xn02xxmn03/n05xxmn03e0xn01/E0xn09xxQ/n03xxMY0xxxMxxUxxMn03/n03S0xn07/n06D0Mn04/n05A0xn05/n11",
        false,
    )
    .expect("black fast spirit bridge duel fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);

    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );
    println!(
        "BLACK_SPIRIT_BRIDGE_DUEL_FAST current_selected={} context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} probe={:?} advisor={:?}",
        current_selected,
        exact_opportunity_context_probe(&game),
        legacy_selected,
        legacy_full_pool_selected,
        legacy_candidates,
        legacy_full_pool,
        probe,
        advisor
    );
    assert_eq!(probe.selected_input_fen, "l4,9;l5,10;mb");
    assert_eq!(probe.pre_accept_input_fen, "l2,5;l1,7;l0,8");
    assert_eq!(probe.selector_last_stage, "engine_disabled");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l2,5;l1,7;l0,8"));
    assert!(probe.head_accepted);
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_late_mana_sibling_duel_normal_root() {
    let game = MonsGame::from_fen(
        "2 1 w 0 0 0 0 0 11 d0xa0xn09/n01xxmn01y0xn07/n05s0xn02xxmn02/n03xxmn07/E0xn03xxmn01e0xn04/n10xxQ/n04xxUxxMn05/n07S0xxxMn02/n02xxMn02A0xn05/n06Y0xn04/D0xn10",
        false,
    )
    .expect("white late mana sibling duel normal fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);
    let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );
    let current_root = format_root_probe(
        scored_roots
            .iter()
            .find(|root| Input::fen_from_array(&root.inputs) == current_selected),
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
        "WHITE_LATE_MANA_SIBLING_DUEL_NORMAL current_selected={} current_root=\"{}\" context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} top_root_details={:?} probe={:?} advisor={:?}",
        current_selected,
        current_root,
        exact_opportunity_context_probe(&game),
        legacy_selected,
        legacy_full_pool_selected,
        legacy_candidates,
        legacy_full_pool,
        top_root_details,
        probe,
        advisor
    );
    assert_eq!(probe.selected_input_fen, "l7,7;l6,5;l6,6");
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_post_search_duel_normal_root() {
    let game = MonsGame::from_fen(
        "0 1 b 0 0 0 0 0 8 n10d0x/n06a0xn04/n05s0xn01e0xn03/n02xxmxxmy0xn06/E0xn10/n04xxmxxUxxmn03xxQ/n03xxMY0xn01S0xxxMn03/n04D0Mn06/n04xxMA0xn05/n09xxMn01/n11",
        false,
    )
    .expect("black post-search duel normal fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);
    let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );

    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );
    let current_root = format_root_probe(
        scored_roots
            .iter()
            .find(|root| Input::fen_from_array(&root.inputs) == current_selected),
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
        "BLACK_POST_SEARCH_DUEL_NORMAL current_selected={} current_root=\"{}\" context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} top_root_details={:?} probe={:?} advisor={:?}",
        current_selected,
        current_root,
        exact_opportunity_context_probe(&game),
        legacy_selected,
        legacy_full_pool_selected,
        legacy_candidates,
        legacy_full_pool,
        top_root_details,
        probe,
        advisor
    );
    assert_eq!(probe.selected_input_fen, "l2,5;l2,7;l3,7");
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_bridge_nonwin_duel_fast_root() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 4 n06a0xn04/n05s0xd0xe0xn03/n07xxmn03/n03xxmn07/n01y0xn01xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n11/n02E0xA0xn01S0xn01Y0xn03/D0xn10",
        false,
    )
    .expect("black bridge non-win duel fast fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);

    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );
    println!(
        "BLACK_BRIDGE_NONWIN_DUEL_FAST current_selected={} context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} probe={:?} advisor={:?}",
        current_selected,
        exact_opportunity_context_probe(&game),
        legacy_selected,
        legacy_full_pool_selected,
        legacy_candidates,
        legacy_full_pool,
        probe,
        advisor
    );
    assert_eq!(probe.selected_input_fen, "l4,1;l5,0;mb");
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_post_search_duel_pro_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "WHITE_POST_SEARCH_DUEL_PRO",
        "1 1 w 1 0 0 0 0 5 n10d0x/n03y0xn03a0xn03/n01xxmn04s0xn01e0xn02/n04xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n06xxMn04/n02xxMn02S0xn05/n05A0xY0xn04/D0xn02E0xn07",
        "l9,6;l8,7",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_flat_nonwin_duel_pro_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "WHITE_FLAT_NONWIN_DUEL_PRO",
        "0 0 w 0 0 1 0 0 3 n03y0xn03e0xn03/n05s0xa0xn01d0mn02/n11/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n01E0xn05Y0xn03/n04D0xn01S0xn04/n04A0xn06",
        "l8,7;l7,8",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_black_post_search_spirit_reentry_duel_pro_root() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 0 0 0 6 n05d1xa0xn04/n05s0xn01e0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/n05xxMn05/n01y0xn01xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        false,
    )
    .expect("black post-search duel pro fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );

    println!(
        "BLACK_POST_SEARCH_DUEL_PRO current_selected={} probe={:?} advisor={:?}",
        current_selected,
        probe,
        pro_v2_root_advisor_decision_snapshot(),
    );
    assert_eq!(current_selected, "l0,6;l1,6");
    assert_eq!(probe.pre_accept_input_fen, "l0,6;l1,6");
    assert_eq!(probe.selected_input_fen, "l0,6;l1,6");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l1,5;l1,7;l0,7"));
    assert!(!probe.head_accepted);
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_head_nonwin_duel_pro_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_HEAD_NONWIN_DUEL_PRO",
        "0 0 b 0 0 2 0 0 2 n03y0xn01d0xn01e0xn03/n04s0xa0xn05/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05S0xn05/n03A0xn07/n02E0xn02D0xn02Y0xn02",
        "l1,4;l3,4;l3,3",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_black_followup_spirit_head_duel_pro_root() {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 2 0 0 2 n03y0xn01d0xa0xn04/n04s0xn01e0xn04/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04A0xD0xn05/n03E0xn02S0xn02Y0xn01",
        false,
    )
    .expect("black followup spirit duel pro fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );

    println!(
        "BLACK_FOLLOWUP_SPIRIT_DUEL_PRO current_selected={} probe={:?} advisor={:?}",
        current_selected,
        probe,
        pro_v2_root_advisor_decision_snapshot(),
    );
    assert_eq!(current_selected, "l1,4;l3,4;l3,3");
    assert_eq!(probe.pre_accept_input_fen, "l1,4;l3,4;l3,3");
    assert_eq!(probe.selected_input_fen, "l1,4;l3,4;l3,3");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l1,4;l0,6;l1,7"));
    assert!(!probe.head_accepted);
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_mana_cluster_duel_pro_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "WHITE_MANA_CLUSTER_DUEL_PRO",
        "2 1 w 0 0 0 0 0 7 n11/n01xxmn01y0xn03a0xd0mn02/n06s0xn01e0xn02/n04xxmn06/n05xxmn05/xxQn04xxUn04xxQ/n04xxMn02xxMn03/n06xxMn04/n05S0xn01Y0xn03/n05A0xn05/D0xn02E0xn07",
        "l8,5;l7,5",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_confirm_normal_ply49_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "WHITE_CONFIRM_NORMAL_PLY49",
        "1 1 w 0 0 0 0 0 9 n11/n02y0xn01s0xn01a0xn04/n02xxmn04d0xn03/n06xxmn04/n04xxmn02xxmn03/xxQn04xxUn02Y0xn02/n04xxMn06/n05xxMn05/n02xxMn01S0xn03xxMn01e0x/n11/n02E0xn01A0xD0xn05",
        "l8,4;l8,2;l9,1",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_confirm_normal_ply26_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "WHITE_CONFIRM_NORMAL_PLY26",
        "0 0 w 0 0 0 0 0 5 n05d1xn05/n06a0xn04/n02xxmn03s0xn04/n02y0xn01xxmn01xxmn04/n05xxmn01xxme0xn02/xxQn04xxUn05/n03xxMn01xxMn01xxMn03/n07xxMn03/n04xxMn06/n04E0xD0xS0xn04/n04A0xn02Y1xn03",
        "l9,6;l7,7;l7,8",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_confirm_normal_ply46_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_CONFIRM_NORMAL_PLY46",
        "1 1 b 0 0 0 0 0 8 E0xn02y0xn01d1xn05/n05s0xa0xe0xn03/n03xxmn03xxmn03/n11/n03xxmn03xxmn03/n05xxUn04xxQ/n03xxMxxMn02xxMn03/n11/n04A0xn01S0xn04/n05D0xxxMn01Y0xn02/n11",
        "l1,5;l2,3;l2,2",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_confirm_pro_ply23_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "WHITE_CONFIRM_PRO_PLY23",
        "1 1 w 1 0 0 0 0 5 d0xn10/n05s0xa0xe0xn03/n03y0xn03xxmn03/n04xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn03xxMn03/n04xxMn06/n07xxMn03/n03A0xn01S0xn01Y0xn03/n03E0xn06D0x",
        "l10,3;l9,2",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_head_runtime_duel_pro_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_HEAD_RUNTIME_DUEL_PRO",
        "1 1 b 0 0 0 0 0 6 d0xn10/n05s0xa0xe0xn03/n03y0xn03xxmn03/n11/n04xxmxxmn01xxmn03/E0xn09xxQ/n05xxMxxUn04/n03xxMxxMn01S0xn04/n08xxMn02/n05A0xn05/n07Y0xn02D0x",
        "l1,5;l2,5",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_engine_disabled_duel_fast_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_ENGINE_DISABLED_DUEL_FAST",
        "1 1 b 0 0 0 0 0 6 n06a0xn03d0x/n03y0xn01s0xn01e0xn03/n03xxmn07/n08xxmn02/n03xxmn01xxmn05/E0xn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n11/n04A0xD0MS0xn04/n08Y0xn02/n11",
        "l1,5;l2,3;l1,2",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_confirm_pro_ply16_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_CONFIRM_PRO_PLY16",
        "1 0 b 1 0 0 0 0 4 n11/n03y0xd0ms0xa0xe0xn03/n07xxmn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n04S0xn04Y0xn01/n11/n03E0xA0xn05D0x",
        "l1,6;l2,5",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_spirit_rerank_duel_pro_fast_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_SPIRIT_RERANK_DUEL_PRO_FAST",
        "2 0 b 0 0 0 0 0 8 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        "l1,5;l2,7;l1,8",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_shared_late_post_search_nonwin_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_SHARED_LATE_POST_SEARCH_NONWIN",
        "1 0 b 1 0 0 0 0 8 n05d0xn05/n05s0xa0xe0xxxmn02/n11/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        "l1,5;l2,5",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_early_post_search_nonwin_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_EARLY_POST_SEARCH_NONWIN",
        "1 0 b 1 0 0 0 0 6 n05d0xn03xxmn01/n03y0xn02a0xn04/n03xxmn01s0xn05/n02xxmn03e0xxxmn03/n05xxmn04Y0x/xxQn04xxUn05/n03xxMxxMn06/n06xxMxxMn03/n01E0xn03S0xn05/n03A0xn07/D0xn10",
        "l0,5;l1,4",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_turn_four_followup_nonwin_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_TURN_FOUR_FOLLOWUP_NONWIN",
        "0 0 b 1 0 1 0 0 4 n03y0xn03e0xn03/n05a0xn05/n02xxmn01s0xn02d0mn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/E0xn03xxMS0xn05/n05D0xn01Y0xn03/n04A0xn06",
        "l1,5;l1,6",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_late_post_search_nonwin_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "WHITE_LATE_POST_SEARCH_NONWIN",
        "2 1 w 0 0 4 0 0 7 n11/n01xxmn01y0xn03a0xd0mn02/n06s0xn01e0xn02/n04xxmn06/n05xxmn05/xxQn04xxUn04Y0B/n04xxMn02xxMn03/n05S0xxxMn04/n11/n05A0xn05/D0xn02E0xn07",
        "l5,10;l4,10",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_harvest_followup_nonwin_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "WHITE_HARVEST_FOLLOWUP_NONWIN",
        "0 0 w 0 0 2 0 0 3 n03y0xn03e0xn03/n05s0xa0xn01d0mn02/n11/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn01Y0xn02/n01E0xn09/n04D0xn01S0xn04/n04A0xn06",
        "l7,8;l6,9",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_white_late_cluster_nonwin_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "WHITE_LATE_CLUSTER_NONWIN",
        "1 1 w 0 0 0 0 0 5 d0xn10/n05s0xa0xe0xn03/n03y0xn03xxmn03/n11/n04xxmxxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n05S0xn05/n04E0xA0xn05/n07Y0xn02D0x",
        "l8,5;l6,3;l7,3",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_turn_ten_nonwin_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_TURN_TEN_NONWIN",
        "3 0 b 1 0 0 0 0 10 n09xxmn01/n05a0xn01e0xn03/n05s0xd0mn04/n02xxmxxmn07/n05xxmn02Y0xn02/n05xxUn05/y0xn04xxMn05/n03xxMn07/n04S0xn06/n02E0xn08/n04A0xn05D0x",
        "l2,5;l3,6",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_prefers_windowed_mana_on_black_late_fast_trace_root() {
    let game = MonsGame::from_fen(
        "3 1 b 1 0 2 0 0 14 n11/n07a0xd0xxxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
        false,
    )
    .expect("black late fast mana lane trace fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );

    println!(
        "BLACK_LATE_FAST_MANA_LANE_TRACE current_selected={} context={} probe={:?} advisor={:?}",
        current_selected,
        exact_opportunity_context_probe(&game),
        probe,
        advisor,
    );
    assert_eq!(current_selected, "l1,8;l1,9");
    assert_eq!(probe.selected_input_fen, "l1,8;l1,9");
    assert_eq!(probe.pre_accept_input_fen, "l1,8;l1,9");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l1,8;l1,9"));
    assert!(probe.head_accepted);
}

#[test]
fn runtime_pro_turn_engine_v30_keeps_recovery_on_black_late_fast_trace_root() {
    let game = MonsGame::from_fen(
        "3 1 b 0 0 0 0 0 14 n05d0xn05/n07a0xn01xxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
        false,
    )
    .expect("black late fast recovery trace fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let current_selected = profile_decision_move_fen(
        "runtime_current",
        SmartAutomovePreference::Pro,
        &game,
    );

    println!(
        "BLACK_LATE_FAST_RECOVERY_TRACE current_selected={} context={} probe={:?} advisor={:?}",
        current_selected,
        exact_opportunity_context_probe(&game),
        probe,
        advisor,
    );
    assert_eq!(current_selected, "l2,5;l0,5;l1,6");
    assert_eq!(probe.selected_input_fen, "l2,5;l0,5;l1,6");
    assert_eq!(probe.pre_accept_input_fen, "l2,5;l0,5;l1,6");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l0,5;l1,6"));
    assert!(!probe.head_accepted);
}

#[test]
fn runtime_pro_turn_engine_v30_profile_prefers_current_black_late_fast_second_lane_nonwin_root() {
    assert_runtime_pro_turn_engine_v30_prefers_current_root_on_board(
        "BLACK_LATE_FAST_SECOND_LANE_NONWIN",
        "3 1 b 1 0 3 0 0 14 n08d0xn02/n07a0xn01xxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
        "l0,8;l1,9",
    );
}

#[test]
fn runtime_pro_turn_engine_v30_avoids_vulnerable_safe_progress_on_black_opening_lane_nonwin_root() {
    let game = MonsGame::from_fen(
        "1 1 b 1 0 0 0 0 6 n03y0xn03e0xn02d0x/n01xxmn04a0xn04/n04s0xn06/n11/n03xxmn02xxmxxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n06xxMn04/E0xn04S0xn05/n01xxMn05Y0xn03/D0xn03A0xn06",
        false,
    )
    .expect("black opening lane nonwin fen should be valid");
    let probe = runtime_decision_probe(
        "runtime_pro_turn_engine_v30",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot()
        .expect("advisor decision should be present on black opening lane nonwin board");

    assert_ne!(probe.selected_input_fen, "l0,10;l1,9");
    assert!(matches!(
        probe.selected_input_fen.as_str(),
        "l0,3;l1,2" | "l0,3;l0,2"
    ));
    assert_eq!(probe.selected_input_fen, probe.pre_accept_input_fen);
    assert!(!probe.head_accepted);
    assert!(
        advisor
            .approved_root
            .as_ref()
            .is_some_and(|root| root.family == TurnPlanFamily::ManaTempo),
        "advisor should stay on a safe mana sibling once the vulnerable safe-progress reentry is removed: {:?}",
        advisor,
    );
}

#[test]
fn runtime_pro_turn_engine_v30_rejects_white_harvest_non_progress_window_injection() {
    let fixture = primary_pro_fixture_by_id("primary_white_harvest_loss_c_ply24");
    let config =
        calibration_runtime_config("runtime_pro_turn_engine_v30", &fixture.game, fixture.mode);
    let perspective = fixture.game.active_color;
    let mut root_moves = MonsGameModel::ranked_root_moves(&fixture.game, perspective, config);
    let engine_plan = turn_engine_candidate_plan(
        &fixture.game,
        perspective,
        MonsGameModel::turn_engine_search_config_for_game(&fixture.game, config),
    )
    .expect("white harvest fixture should materialize a turn-engine plan");

    assert_eq!(
        engine_plan.head_family,
        TurnPlanFamily::SafeOpponentManaProgress
    );
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
    assert_eq!(
        Input::fen_from_array(&pre_accept_selected),
        "l9,7;l7,8;l7,7"
    );
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
fn runtime_pro_turn_engine_v30_rejects_lower_scored_pvs_progress_head_without_material_override() {
    let fixture = primary_pro_fixture_by_id("primary_pvs_sensitive_search");
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
    let head_plan = head_plan.expect("pvs fixture should retain a head plan");
    let head_inputs = head_plan
        .compiled_chunks
        .first()
        .expect("head plan should include a first chunk");
    let pre_accept_root = scored_roots
        .iter()
        .find(|root| root.inputs == pre_accept_selected)
        .expect("pre-accept selected root should be present");
    let head_root = scored_roots
        .iter()
        .find(|root| root.inputs.as_slice() == head_inputs.as_slice())
        .expect("head root should be present");
    let pre_accept_family = MonsGameModel::turn_engine_root_evaluation_family(pre_accept_root);
    let pre_accept_utility = MonsGameModel::turn_engine_root_plan_utility(
        &fixture.game,
        pre_accept_root,
        fixture.game.active_color,
        config,
        pre_accept_family,
    );
    let accepted = MonsGameModel::accept_turn_engine_head_after_search(
        &fixture.game,
        fixture.game.active_color,
        config,
        scored_roots.as_slice(),
        pre_accept_selected.as_slice(),
        &head_plan,
    );

    assert_eq!(
        forced_engine_inputs
            .as_ref()
            .map(|inputs| Input::fen_from_array(inputs)),
        Some("l0,5;l1,5".to_string()),
    );
    assert_eq!(Input::fen_from_array(&pre_accept_selected), "l0,6;l1,6");
    assert_eq!(Input::fen_from_array(head_inputs), "l0,5;l1,5");
    assert!(pre_accept_root.score > head_root.score);
    assert!(pre_accept_root.own_drainer_vulnerable);
    assert!(head_root.own_drainer_vulnerable);
    assert!(head_root.supermana_progress);
    assert!(!pre_accept_root.supermana_progress);
    assert!(!head_plan
        .utility
        .improves_non_score_override_axes(pre_accept_utility));
    assert!(!head_plan
        .utility
        .has_score_delta_force(pre_accept_utility, 220));
    assert!(
        !accepted,
        "a lower-scored unsafe progress head should not override the selected PVS root without a material primary-axis or safety gain: selected_utility={:?} head_utility={:?} plan_utility={:?}",
        pre_accept_utility,
        head_plan.head_utility,
        head_plan.utility,
    );
    assert_eq!(
        profile_decision_move_fen("runtime_pro_turn_engine_v30", fixture.mode, &fixture.game),
        "l0,6;l1,6",
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
