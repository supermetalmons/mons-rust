use super::harness::env_usize;
use super::*;

const DEFAULT_PROMOTION_BASELINE_PROFILE: &str = "runtime_release_safe_pre_exact";
const PROFILE_RUNTIME_EFF_EXACT_LITE_V1: &str = "runtime_eff_exact_lite_v1";
const PROFILE_RUNTIME_NORMAL_FROM_FAST_REFERENCE_V1: &str = "runtime_normal_from_fast_reference_v1";
const PROFILE_RUNTIME_PRE_FAST_ROOT_QUALITY_V1_NORMAL_CONVERSION_V3: &str =
    "runtime_pre_fast_root_quality_v1_normal_conversion_v3";
const PROFILE_SWIFT_2024_EVAL_REFERENCE: &str = "swift_2024_eval_reference";
const PROFILE_SWIFT_2024_STYLE_REFERENCE: &str = "swift_2024_style_reference";
const PROFILE_RUNTIME_PRO_TURN_ENGINE_V1: &str = "runtime_pro_turn_engine_v1";
const PROFILE_RUNTIME_PRO_TURN_ENGINE_V30: &str = "runtime_pro_turn_engine_v30";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ExactLiteBudgets {
    pub root_call_budget: usize,
    pub static_call_budget: usize,
}

#[derive(Clone, Copy)]
struct AutomoveProfile {
    id: &'static str,
    selector: AutomoveSelector,
}

const RETAINED_PROFILES: [AutomoveProfile; 10] = [
    AutomoveProfile {
        id: "base",
        selector: model_base_profile,
    },
    AutomoveProfile {
        id: "runtime_current",
        selector: model_runtime_current_profile,
    },
    AutomoveProfile {
        id: "runtime_release_safe_pre_exact",
        selector: model_runtime_release_safe_pre_exact,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_EFF_EXACT_LITE_V1,
        selector: model_runtime_eff_exact_lite_v1,
    },
    AutomoveProfile {
        id: PROFILE_SWIFT_2024_EVAL_REFERENCE,
        selector: model_swift_2024_eval_reference,
    },
    AutomoveProfile {
        id: PROFILE_SWIFT_2024_STYLE_REFERENCE,
        selector: model_swift_2024_style_reference,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_PRE_FAST_ROOT_QUALITY_V1_NORMAL_CONVERSION_V3,
        selector: model_runtime_pre_fast_root_quality_v1_normal_conversion_v3,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_NORMAL_FROM_FAST_REFERENCE_V1,
        selector: model_runtime_normal_from_fast_reference_v1,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_PRO_TURN_ENGINE_V1,
        selector: model_runtime_pro_turn_engine_v1,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_PRO_TURN_ENGINE_V30,
        selector: model_runtime_pro_turn_engine_v30,
    },
];

const CURATED_POOL_PROFILE_IDS: [&str; CURATED_POOL_SIZE] = [
    "runtime_current",
    "runtime_release_safe_pre_exact",
    PROFILE_SWIFT_2024_EVAL_REFERENCE,
    PROFILE_SWIFT_2024_STYLE_REFERENCE,
    PROFILE_RUNTIME_PRE_FAST_ROOT_QUALITY_V1_NORMAL_CONVERSION_V3,
];

pub(super) const CANDIDATE_MODEL: AutomoveModel = AutomoveModel {
    id: "candidate",
    select_inputs: candidate_model,
};

fn opening_book_enabled() -> bool {
    std::env::var("SMART_USE_WHITE_OPENING_BOOK")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
        .unwrap_or(false)
}

fn runtime_selector_inputs(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let inputs = MonsGameModel::smart_search_best_inputs(game, config);
    if !inputs.is_empty() {
        return inputs;
    }

    let mut simulated = game.clone_for_simulation();
    let output = MonsGameModel::automove_game(&mut simulated);
    if output.kind == OutputModelKind::Events {
        Input::array_from_fen(output.input_fen().as_str())
    } else {
        Vec::new()
    }
}

pub(super) fn model_current_best(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    runtime_selector_inputs(game, config)
}

pub(super) fn model_base_profile(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

pub(super) fn model_runtime_current_profile(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    model_current_best(game, config)
}

fn configure_runtime_release_safe_pre_exact(config: SmartSearchConfig) -> SmartSearchConfig {
    MonsGameModel::with_pre_exact_runtime_policy(config)
}

pub(super) fn model_runtime_release_safe_pre_exact(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    runtime_selector_inputs(game, configure_runtime_release_safe_pre_exact(config))
}

fn configure_runtime_eff_exact_lite_base(config: SmartSearchConfig) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth < 3 {
        runtime.root_reply_risk_score_margin = 130;
        runtime.root_reply_risk_shortlist_max = 4;
        runtime.root_reply_risk_reply_limit = 9;
        runtime.root_reply_risk_node_share_bp = 650;
        runtime.root_drainer_safety_score_margin = SMART_ROOT_DRAINER_SAFETY_SCORE_MARGIN;
        runtime.root_efficiency_score_margin = 1_700;
        runtime.enable_interview_soft_root_priors = true;
        runtime.enable_interview_deterministic_tiebreak = false;
        runtime.scoring_weights = &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF;
    } else {
        runtime.enable_interview_deterministic_tiebreak = false;
    }
    MonsGameModel::with_pre_exact_runtime_policy(runtime)
}

fn configure_runtime_eff_exact_lite_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = configure_runtime_eff_exact_lite_base(config);
    runtime.enable_root_exact_tactics = false;
    runtime.enable_child_exact_tactics = false;
    runtime.enable_static_exact_evaluation = false;
    runtime.enable_exact_lite_progress_checks = true;
    runtime.enable_exact_lite_spirit_window_checks = true;
    if runtime.depth < 3 {
        runtime.exact_lite_root_call_budget = 0;
        runtime.exact_lite_static_call_budget = 0;
    } else if runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.exact_lite_root_call_budget = 1;
        runtime.exact_lite_static_call_budget = 1;
    } else {
        runtime.exact_lite_root_call_budget = 2;
        runtime.exact_lite_static_call_budget = 1;
    }
    runtime
}

pub(super) fn model_runtime_eff_exact_lite_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, configure_runtime_eff_exact_lite_v1(game, config))
}

fn configure_runtime_pre_fast_root_quality_v1_normal_conversion_v3(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth < 3 {
        runtime.root_reply_risk_score_margin = SMART_ROOT_REPLY_RISK_SCORE_MARGIN;
        runtime.root_reply_risk_shortlist_max = SMART_ROOT_REPLY_RISK_SHORTLIST_FAST;
        runtime.root_reply_risk_reply_limit = SMART_ROOT_REPLY_RISK_REPLY_LIMIT_FAST;
        runtime.root_reply_risk_node_share_bp = SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_FAST;
        runtime.root_mana_handoff_penalty = 260;
        runtime.root_backtrack_penalty = 180;
        runtime.root_efficiency_score_margin = 1_900;
        runtime.root_anti_help_score_margin = 220;
    } else {
        runtime.root_reply_risk_score_margin = 170;
        runtime.root_reply_risk_shortlist_max = 6;
        runtime.root_reply_risk_reply_limit = 14;
        runtime.root_reply_risk_node_share_bp = 1_150;
        runtime.root_drainer_safety_score_margin = 4_000;
        runtime.selective_extension_node_share_bp = SMART_SELECTIVE_EXTENSION_NODE_SHARE_BP_NORMAL;
        runtime.root_efficiency_score_margin = 1_400;
    }
    runtime
}

pub(super) fn model_runtime_pre_fast_root_quality_v1_normal_conversion_v3(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        game,
        configure_runtime_pre_fast_root_quality_v1_normal_conversion_v3(game, config),
    )
}

fn configure_runtime_normal_fast_policy_block(mut runtime: SmartSearchConfig) -> SmartSearchConfig {
    runtime.root_branch_limit = (runtime.root_branch_limit + 5).clamp(12, 40);
    runtime.node_branch_limit = runtime.node_branch_limit.saturating_sub(2).clamp(8, 18);
    runtime.root_enum_limit = (runtime.root_branch_limit * 6).clamp(runtime.root_branch_limit, 240);
    runtime.node_enum_limit = (runtime.node_branch_limit * 4).clamp(runtime.node_branch_limit, 108);
    runtime.enable_event_ordering_bonus = true;
    runtime.enable_two_pass_root_allocation = false;
    runtime.root_focus_k = 2;
    runtime.root_focus_budget_share_bp = 6_000;
    runtime.enable_selective_extensions = false;
    runtime.selective_extension_node_share_bp = 0;
    runtime.enable_quiet_reductions = true;
    runtime.root_reply_risk_score_margin = 125;
    runtime.root_reply_risk_shortlist_max = 4;
    runtime.root_reply_risk_reply_limit = 10;
    runtime.root_reply_risk_node_share_bp = 650;
    runtime.root_anti_help_score_margin = 220;
    runtime.root_anti_help_reply_limit = SMART_ROOT_ANTI_HELP_REPLY_LIMIT_FAST;
    runtime.enable_two_pass_volatility_focus = false;
    runtime.enable_normal_root_safety_rerank = false;
    runtime.enable_normal_root_safety_deep_floor = false;
    runtime.enable_interview_hard_spirit_deploy = false;
    runtime.enable_interview_soft_root_priors = true;
    runtime.enable_interview_deterministic_tiebreak = false;
    runtime.prefer_clean_reply_risk_roots = false;
    runtime.root_drainer_safety_score_margin = SMART_ROOT_DRAINER_SAFETY_SCORE_MARGIN;
    runtime.root_mana_handoff_penalty = 300;
    runtime.root_backtrack_penalty = 220;
    runtime.root_efficiency_score_margin = 1_700;
    runtime.potion_spend_penalty_fast = 220;
    runtime.potion_spend_penalty_normal = SMART_POTION_SPEND_NO_COMPENSATION_PENALTY_NORMAL;
    runtime.interview_soft_score_margin = 80;
    runtime.interview_soft_supermana_progress_bonus = 320;
    runtime.interview_soft_supermana_score_bonus = 600;
    runtime.interview_soft_opponent_mana_progress_bonus = 200;
    runtime.interview_soft_opponent_mana_score_bonus = 310;
    runtime.interview_soft_mana_handoff_penalty = 280;
    runtime.interview_soft_roundtrip_penalty = 220;
    runtime.enable_enhanced_drainer_vulnerability = true;
    runtime.enable_supermana_prepass_exception = true;
    runtime.scoring_weights = &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF;
    runtime
}

fn configure_runtime_normal_from_fast_reference_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= 3 && runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime = configure_runtime_normal_fast_policy_block(runtime);
    } else {
        runtime = MonsGameModel::with_runtime_scoring_weights(game, runtime);
    }
    runtime
}

pub(super) fn model_runtime_normal_from_fast_reference_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        game,
        configure_runtime_normal_from_fast_reference_v1(game, config),
    )
}

fn configure_runtime_pro_turn_engine_v1(config: SmartSearchConfig) -> SmartSearchConfig {
    let mut runtime = config;
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize
        && runtime.enable_normal_root_safety_deep_floor
    {
        runtime.enable_turn_engine = true;
        runtime.turn_engine_mode = TurnEngineMode::ProV1;
        runtime.turn_engine_seed_cap = 16;
        runtime.turn_engine_beam_width = 6;
        runtime.turn_engine_per_node_family_cap = 4;
        runtime.turn_engine_step_cap = 7;
        runtime.turn_engine_opponent_seed_cap = 8;
        runtime.turn_engine_opponent_beam_width = 3;
        runtime.turn_engine_reply_seed_cap = 4;
        runtime.turn_engine_reply_beam_width = 2;
        runtime.turn_engine_expansion_cap = 192;
        runtime.turn_engine_enable_spirit_family = true;
    }
    runtime
}

fn configure_runtime_pro_turn_engine_v30(config: SmartSearchConfig) -> SmartSearchConfig {
    let mut runtime = config;
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize
        && runtime.enable_normal_root_safety_deep_floor
    {
        runtime.enable_turn_opportunity_planner = false;
        runtime.enable_turn_engine = true;
        runtime.turn_engine_mode = TurnEngineMode::ProV2;
        runtime.turn_engine_seed_cap = 14;
        runtime.turn_engine_beam_width = 5;
        runtime.turn_engine_per_node_family_cap = 4;
        runtime.turn_engine_step_cap = 6;
        runtime.turn_engine_opponent_seed_cap = 6;
        runtime.turn_engine_opponent_beam_width = 2;
        runtime.turn_engine_reply_seed_cap = 3;
        runtime.turn_engine_reply_beam_width = 1;
        runtime.turn_engine_expansion_cap = 176;
        runtime.turn_engine_enable_spirit_family = true;
        runtime.root_reply_risk_reply_limit = runtime.root_reply_risk_reply_limit.min(24);
        runtime.root_reply_risk_node_share_bp = runtime.root_reply_risk_node_share_bp.min(2_000);
        runtime.enable_turn_engine_low_budget_guard = true;
        runtime.enable_turn_engine_mid_turn_tactical_guard = true;
        runtime.enable_turn_engine_late_safe_mana_root_preference = true;
    }
    runtime
}

pub(super) fn model_runtime_pro_turn_engine_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if opening_book_enabled() {
        let opening_runtime = SearchBudget::from_preference(SmartAutomovePreference::Normal)
            .runtime_config_for_game(game);
        return runtime_selector_inputs(
            game,
            configure_runtime_release_safe_pre_exact(opening_runtime),
        );
    }

    runtime_selector_inputs(game, configure_runtime_pro_turn_engine_v1(config))
}

fn runtime_pro_turn_engine_v30_guarded_inputs(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if opening_book_enabled() {
        let opening_runtime = SearchBudget::from_preference(SmartAutomovePreference::Normal)
            .runtime_config_for_game(game);
        return runtime_selector_inputs(
            game,
            configure_runtime_release_safe_pre_exact(opening_runtime),
        );
    }

    let early_white_turn_start = game.active_color == Color::White
        && game.turn_number <= 3
        && !game.player_can_use_action()
        && !game.player_can_move_mana()
        && matches!(game.mons_moves_count, 0 | 3);
    if early_white_turn_start {
        let pro_runtime = SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(game);
        return model_current_best(game, pro_runtime);
    }

    let white_turn_one_late_opening_tail = game.active_color == Color::White
        && game.turn_number == 1
        && game.mons_moves_count == 2
        && !game.player_can_use_action()
        && !game.player_can_move_mana();
    if white_turn_one_late_opening_tail {
        let pro_runtime = SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(game);
        return model_current_best(game, pro_runtime);
    }

    let white_turn_three_turn_start_action_mana = game.active_color == Color::White
        && game.turn_number == 3
        && game.mons_moves_count == 0
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if white_turn_three_turn_start_action_mana {
        let pro_runtime = SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(game);
        return model_current_best(game, pro_runtime);
    }

    let white_turn_three_mana_only = game.active_color == Color::White
        && game.turn_number == 3
        && game.mons_moves_count == 1
        && !game.player_can_use_action()
        && game.player_can_move_mana();
    if white_turn_three_mana_only {
        let normal_runtime = SearchBudget::from_preference(SmartAutomovePreference::Normal)
            .runtime_config_for_game(game);
        return runtime_selector_inputs(
            game,
            configure_runtime_release_safe_pre_exact(normal_runtime),
        );
    }

    let white_turn_three_mid_turn_full_resources = game.active_color == Color::White
        && game.turn_number == 3
        && game.mons_moves_count >= 5
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if white_turn_three_mid_turn_full_resources {
        let pro_runtime = SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(game);
        return model_current_best(game, pro_runtime);
    }

    let white_turn_three_mid_turn = game.active_color == Color::White
        && game.turn_number == 3
        && game.mons_moves_count > 0
        && (game.player_can_use_action() || game.player_can_move_mana());
    if white_turn_three_mid_turn {
        let fast_runtime = SearchBudget::from_preference(SmartAutomovePreference::Fast)
            .runtime_config_for_game(game);
        return runtime_selector_inputs(
            game,
            configure_runtime_release_safe_pre_exact(fast_runtime),
        );
    }

    let black_turn_two_turn_start_action_mana = game.active_color == Color::Black
        && game.turn_number == 2
        && game.mons_moves_count == 0
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if black_turn_two_turn_start_action_mana {
        let pro_runtime = SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(game);
        return model_current_best(game, pro_runtime);
    }

    let black_turn_two_mana_only = game.active_color == Color::Black
        && game.turn_number == 2
        && game.mons_moves_count > 0
        && !game.player_can_use_action()
        && game.player_can_move_mana();
    if black_turn_two_mana_only {
        let pro_runtime = SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(game);
        return model_current_best(game, pro_runtime);
    }

    let black_turn_four_turn_start_action_mana = game.active_color == Color::Black
        && game.turn_number == 4
        && game.mons_moves_count == 0
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if black_turn_four_turn_start_action_mana {
        let pro_runtime = SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(game);
        return model_current_best(game, pro_runtime);
    }

    runtime_selector_inputs(game, configure_runtime_pro_turn_engine_v30(config))
}

pub(super) fn model_runtime_pro_turn_engine_v30(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    runtime_pro_turn_engine_v30_guarded_inputs(game, config)
}

fn configure_runtime_swift_2024_eval_reference(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights = &SWIFT_2024_REFERENCE_SCORING_WEIGHTS;
    runtime
}

pub(super) fn model_swift_2024_eval_reference(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, configure_runtime_swift_2024_eval_reference(game, config))
}

fn configure_runtime_swift_2024_style_reference(
    _game: &MonsGame,
    _config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut swift_style =
        SmartSearchConfig::from_budget(3, LEGACY_NORMAL_MAX_VISITED_NODES).for_runtime();
    swift_style.scoring_weights = &SWIFT_2024_REFERENCE_SCORING_WEIGHTS;
    swift_style.enable_root_efficiency = false;
    swift_style.enable_event_ordering_bonus = false;
    swift_style.enable_backtrack_penalty = false;
    swift_style.enable_tt_best_child_ordering = false;
    swift_style.enable_two_pass_root_allocation = false;
    swift_style.enable_selective_extensions = false;
    swift_style.enable_quiet_reductions = false;
    swift_style.enable_root_mana_handoff_guard = false;
    swift_style.enable_forced_tactical_prepass = false;
    swift_style.enable_root_drainer_safety_prefilter = false;
    swift_style.enable_root_reply_risk_guard = false;
    swift_style.enable_move_class_coverage = false;
    swift_style.enable_child_move_class_coverage = false;
    swift_style.enable_strict_tactical_class_coverage = false;
    swift_style.enable_strict_anti_help_filter = false;
    swift_style.enable_interview_hard_spirit_deploy = false;
    swift_style.enable_interview_soft_root_priors = false;
    swift_style.enable_interview_deterministic_tiebreak = false;
    swift_style.enable_mana_start_mix_with_potion_actions = false;
    swift_style.enable_potion_progress_compensation = false;
    swift_style.enable_enhanced_drainer_vulnerability = false;
    swift_style.enable_supermana_prepass_exception = false;
    swift_style.enable_opponent_mana_prepass_exception = false;
    swift_style.enable_walk_threat_prefilter = false;
    swift_style.enable_killer_move_ordering = false;
    swift_style.enable_tt_depth_preferred_replacement = false;
    swift_style.enable_pvs = false;
    swift_style
}

pub(super) fn model_swift_2024_style_reference(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let swift_style = configure_runtime_swift_2024_style_reference(game, config);
    MonsGameModel::smart_search_best_inputs_legacy_no_transposition(game, swift_style)
}

pub(super) fn profile_runtime_config_for_name(
    profile_name: &str,
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Option<SmartSearchConfig> {
    let resolved = match profile_name {
        "base" | "runtime_current" => config,
        "runtime_release_safe_pre_exact" => configure_runtime_release_safe_pre_exact(config),
        PROFILE_RUNTIME_EFF_EXACT_LITE_V1 => configure_runtime_eff_exact_lite_v1(game, config),
        PROFILE_SWIFT_2024_EVAL_REFERENCE => {
            configure_runtime_swift_2024_eval_reference(game, config)
        }
        PROFILE_SWIFT_2024_STYLE_REFERENCE => {
            configure_runtime_swift_2024_style_reference(game, config)
        }
        PROFILE_RUNTIME_PRE_FAST_ROOT_QUALITY_V1_NORMAL_CONVERSION_V3 => {
            configure_runtime_pre_fast_root_quality_v1_normal_conversion_v3(game, config)
        }
        PROFILE_RUNTIME_NORMAL_FROM_FAST_REFERENCE_V1 => {
            configure_runtime_normal_from_fast_reference_v1(game, config)
        }
        PROFILE_RUNTIME_PRO_TURN_ENGINE_V1 => configure_runtime_pro_turn_engine_v1(config),
        PROFILE_RUNTIME_PRO_TURN_ENGINE_V30 => configure_runtime_pro_turn_engine_v30(config),
        _ => return None,
    };
    Some(resolved)
}

pub(super) fn profile_exact_lite_budgets(
    profile_name: &str,
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Option<ExactLiteBudgets> {
    let runtime = match profile_name {
        PROFILE_RUNTIME_EFF_EXACT_LITE_V1 => configure_runtime_eff_exact_lite_v1(game, config),
        _ => return None,
    };
    if !(runtime.enable_exact_lite_progress_checks
        || runtime.enable_exact_lite_spirit_window_checks)
    {
        return None;
    }
    Some(ExactLiteBudgets {
        root_call_budget: runtime.exact_lite_root_call_budget,
        static_call_budget: runtime.exact_lite_static_call_budget,
    })
}

pub(super) fn candidate_model(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let selector =
        profile_selector_from_name(candidate_profile().as_str()).unwrap_or(model_base_profile);
    selector(game, config)
}

pub(super) fn model_first_legal_automove(
    game: &MonsGame,
    _config: SmartSearchConfig,
) -> Vec<Input> {
    let mut simulated = game.clone_for_simulation();
    let automove_start_options = Some(SuggestedStartInputOptions::for_automove());
    let mut inputs = Vec::new();
    let mut output =
        simulated.process_input_with_start_options(vec![], false, false, automove_start_options);

    loop {
        match output {
            Output::InvalidInput => return Vec::new(),
            Output::LocationsToStartFrom(locations) => {
                let Some(location) = locations.first().copied() else {
                    return Vec::new();
                };
                inputs.push(Input::Location(location));
                output = simulated.process_input_with_start_options(
                    inputs.clone(),
                    false,
                    false,
                    automove_start_options,
                );
            }
            Output::NextInputOptions(options) => {
                let Some(next_input) = options.first() else {
                    return Vec::new();
                };
                inputs.push(next_input.input);
                output = simulated.process_input_with_start_options(
                    inputs.clone(),
                    false,
                    false,
                    automove_start_options,
                );
            }
            Output::Events(_) => return inputs,
        }
    }
}

fn retained_profiles() -> &'static [AutomoveProfile] {
    &RETAINED_PROFILES
}

pub(super) fn retained_profile_ids() -> Vec<&'static str> {
    retained_profiles()
        .iter()
        .map(|profile| profile.id)
        .collect()
}

pub(super) fn curated_pool_profile_ids() -> &'static [&'static str] {
    &CURATED_POOL_PROFILE_IDS
}

pub(super) fn profile_selector_from_name(profile_name: &str) -> Option<AutomoveSelector> {
    retained_profiles()
        .iter()
        .find(|profile| profile.id == profile_name)
        .map(|profile| profile.selector)
}

pub(super) fn selected_pool_models() -> Vec<AutomoveModel> {
    let requested = env_usize("SMART_POOL_OPPONENTS").unwrap_or(CURATED_POOL_PROFILE_IDS.len());
    let limit = requested.clamp(1, CURATED_POOL_PROFILE_IDS.len());
    curated_pool_profile_ids()
        .iter()
        .take(limit)
        .map(|profile_id| AutomoveModel {
            id: profile_id,
            select_inputs: profile_selector_from_name(profile_id)
                .unwrap_or_else(|| panic!("curated pool profile '{}' must resolve", profile_id)),
        })
        .collect()
}

pub(super) fn candidate_profile() -> &'static String {
    static PROFILE: OnceLock<String> = OnceLock::new();
    PROFILE.get_or_init(|| {
        env::var("SMART_CANDIDATE_PROFILE")
            .ok()
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "base".to_string())
    })
}

pub(super) fn env_profile_name(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
}

pub(super) fn gate_baseline_profile_name() -> String {
    env_profile_name("SMART_GATE_BASELINE_PROFILE")
        .unwrap_or_else(|| DEFAULT_PROMOTION_BASELINE_PROFILE.to_string())
}

pub(super) fn pro_candidate_profile_name() -> String {
    env_profile_name("SMART_PRO_CANDIDATE_PROFILE")
        .or_else(|| env_profile_name("SMART_CANDIDATE_PROFILE"))
        .unwrap_or_else(|| "runtime_current".to_string())
}

pub(super) fn pro_baseline_profile_name() -> String {
    env_profile_name("SMART_PRO_BASELINE_PROFILE")
        .or_else(|| Some(gate_baseline_profile_name()))
        .unwrap_or_else(|| DEFAULT_PROMOTION_BASELINE_PROFILE.to_string())
}
