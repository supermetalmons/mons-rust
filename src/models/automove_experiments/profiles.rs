use super::*;

const DEFAULT_PROMOTION_BASELINE_PROFILE: &str = "runtime_current";
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

const RETAINED_PROFILES: [AutomoveProfile; 2] = [
    AutomoveProfile {
        id: "runtime_current",
        selector: model_runtime_current_profile,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_PRO_TURN_ENGINE_V30,
        selector: model_runtime_pro_turn_engine_v30,
    },
];

pub(super) const CANDIDATE_MODEL: AutomoveModel = AutomoveModel {
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

fn runtime_selector_inputs_with_fresh_turn_engine_cache(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.enable_turn_engine {
        crate::models::automove_turn_engine::clear_turn_engine_plan_cache();
        crate::models::automove_turn_engine::clear_turn_engine_diagnostics();
    }
    runtime_selector_inputs(game, config)
}

fn model_current_best(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    runtime_selector_inputs(game, config)
}

pub(super) fn model_runtime_current_profile(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    model_runtime_pro_turn_engine_v30(game, config)
}

fn configure_runtime_release_safe_pre_exact(config: SmartSearchConfig) -> SmartSearchConfig {
    MonsGameModel::with_pre_exact_runtime_policy(config)
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

    let white_turn_three_mid_turn_scoring_action_mana = game.active_color == Color::White
        && game.turn_number == 3
        && matches!(game.mons_moves_count, 1 | 2)
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if white_turn_three_mid_turn_scoring_action_mana {
        let context =
            crate::models::automove_exact::exact_opportunity_context(game, game.active_color);
        if context.delta.same_turn_score_window_value > 0 {
            return runtime_selector_inputs_with_fresh_turn_engine_cache(
                game,
                configure_runtime_pro_turn_engine_v30(config),
            );
        }
    }

    let white_turn_three_mid_turn_full_resources = game.active_color == Color::White
        && game.turn_number == 3
        && game.mons_moves_count >= 3
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
        && !white_turn_three_mana_only
        && (game.player_can_use_action() || game.player_can_move_mana());
    if white_turn_three_mid_turn {
        let drainer_vulnerable = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            game,
            game.active_color,
            config.enable_enhanced_drainer_vulnerability,
        );
        let drainer_walk_vulnerable = MonsGameModel::is_own_drainer_walk_vulnerable_next_turn(
            game,
            game.active_color,
            config.enable_enhanced_drainer_vulnerability,
        );
        if drainer_vulnerable || drainer_walk_vulnerable {
            let fast_runtime = SearchBudget::from_preference(SmartAutomovePreference::Fast)
                .runtime_config_for_game(game);
            return runtime_selector_inputs(
                game,
                configure_runtime_release_safe_pre_exact(fast_runtime),
            );
        }
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

    let candidate_config = configure_runtime_pro_turn_engine_v30(config);
    let candidate_inputs =
        runtime_selector_inputs_with_fresh_turn_engine_cache(game, candidate_config);
    let black_turn_four_bridge_current_fallback = game.active_color == Color::Black
        && game.turn_number == 4
        && game.mons_moves_count == 2
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if black_turn_four_bridge_current_fallback && !candidate_inputs.is_empty() {
        let pro_runtime = SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(game);
        let current_inputs = model_current_best(game, pro_runtime);
        let current_fen = Input::fen_from_array(&current_inputs);
        if !current_inputs.is_empty()
            && current_inputs != candidate_inputs
            && current_inputs.len() == 3
            && current_fen.ends_with(";mb")
        {
            return current_inputs;
        }
    }

    let black_mid_turn_action_mana_current_fallback = game.active_color == Color::Black
        && game.turn_number >= 4
        && game.mons_moves_count >= 3
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if black_mid_turn_action_mana_current_fallback && !candidate_inputs.is_empty() {
        let pro_runtime = SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(game);
        let current_inputs = model_current_best(game, pro_runtime);
        if !current_inputs.is_empty() && current_inputs != candidate_inputs {
            return current_inputs;
        }
    }

    candidate_inputs
}

pub(super) fn model_runtime_pro_turn_engine_v30(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    runtime_pro_turn_engine_v30_guarded_inputs(game, config)
}

pub(super) fn profile_runtime_config_for_name(
    profile_name: &str,
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Option<SmartSearchConfig> {
    let resolved = match profile_name {
        "runtime_current" => config,
        PROFILE_RUNTIME_PRO_TURN_ENGINE_V30 => configure_runtime_pro_turn_engine_v30(config),
        _ => return None,
    };
    Some(resolved)
}

pub(super) fn profile_exact_lite_budgets(
    _profile_name: &str,
    _game: &MonsGame,
    _config: SmartSearchConfig,
) -> Option<ExactLiteBudgets> {
    None
}

pub(super) fn candidate_model(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let selector = profile_selector_from_name(candidate_profile().as_str())
        .unwrap_or(model_runtime_current_profile);
    selector(game, config)
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

pub(super) fn profile_selector_from_name(profile_name: &str) -> Option<AutomoveSelector> {
    retained_profiles()
        .iter()
        .find(|profile| profile.id == profile_name)
        .map(|profile| profile.selector)
}

pub(super) fn candidate_profile() -> &'static String {
    static PROFILE: OnceLock<String> = OnceLock::new();
    PROFILE.get_or_init(|| {
        env::var("SMART_CANDIDATE_PROFILE")
            .ok()
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "runtime_current".to_string())
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
