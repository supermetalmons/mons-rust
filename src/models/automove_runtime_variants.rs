#![cfg(any(target_arch = "wasm32", test))]

use super::*;

pub(crate) const SHIPPING_PRO_SEARCH_PROFILE_ID: &str = "shipping_pro_search";
pub(crate) const FRONTIER_PRO_V2_GUARDED_PROFILE_ID: &str = "frontier_pro_v2_guarded";

#[cfg(test)]
thread_local! {
    static FRONTIER_RUNTIME_VARIANT_BRANCH: std::cell::RefCell<&'static str> =
        const { std::cell::RefCell::new("unset") };
}

#[cfg(test)]
pub(crate) fn clear_frontier_runtime_variant_branch() {
    FRONTIER_RUNTIME_VARIANT_BRANCH.with(|branch| *branch.borrow_mut() = "unset");
}

#[cfg(test)]
pub(crate) fn frontier_runtime_variant_branch_snapshot() -> &'static str {
    FRONTIER_RUNTIME_VARIANT_BRANCH.with(|branch| *branch.borrow())
}

#[cfg(test)]
fn set_frontier_runtime_variant_branch(branch: &'static str) {
    FRONTIER_RUNTIME_VARIANT_BRANCH.with(|last_branch| *last_branch.borrow_mut() = branch);
}

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

fn shipping_search_config_for_game(
    game: &MonsGame,
    preference: SmartAutomovePreference,
) -> AutomoveSearchConfig {
    let hinted_context = if game.variant().supports_opening_book()
        && matches!(preference, SmartAutomovePreference::Pro)
        && opening_book_enabled()
    {
        ShippingProContext::OpeningBookDriven
    } else {
        ShippingProContext::Unknown
    };
    MonsGameModel::shipping_search_config_for_game_with_context(game, preference, hinted_context).0
}

fn select_shipping_search_inputs_internal(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
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

fn select_search_inputs_with_fresh_frontier_cache(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    if config.enable_turn_engine_selector {
        crate::models::automove_turn_engine::clear_turn_engine_plan_cache();
        crate::models::automove_turn_engine::clear_turn_engine_diagnostics();
    }
    select_shipping_search_inputs_internal(game, config)
}

pub(crate) fn select_shipping_search_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    select_shipping_search_inputs_internal(game, config)
}

pub(crate) fn select_shipping_pro_search_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    select_shipping_search_inputs(game, config)
}

pub(crate) fn apply_frontier_pro_v2_guarded_config(
    config: AutomoveSearchConfig,
) -> AutomoveSearchConfig {
    let mut runtime = config;
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize
        && runtime.enable_normal_root_safety_deep_floor
    {
        runtime.enable_turn_head_rerank = false;
        runtime.enable_turn_engine_selector = true;
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

fn select_opening_book_fallback_inputs(game: &MonsGame) -> Option<Vec<Input>> {
    if !game.variant().supports_opening_book() || !opening_book_enabled() {
        return None;
    }

    let opening_runtime = shipping_search_config_for_game(game, SmartAutomovePreference::Normal);
    Some(select_shipping_search_inputs(
        game,
        MonsGameModel::with_pre_exact_runtime_policy(opening_runtime),
    ))
}

fn select_early_white_fallback_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Option<Vec<Input>> {
    let early_white_turn_start = game.active_color == Color::White
        && game.turn_number <= 3
        && !game.player_can_use_action()
        && !game.player_can_move_mana()
        && matches!(game.mons_moves_count, 0 | 3);
    let white_turn_one_late_opening_tail = game.active_color == Color::White
        && game.turn_number == 1
        && game.mons_moves_count == 2
        && !game.player_can_use_action()
        && !game.player_can_move_mana();
    let white_turn_three_turn_start_action_mana = game.active_color == Color::White
        && game.turn_number == 3
        && game.mons_moves_count == 0
        && game.player_can_use_action()
        && game.player_can_move_mana();
    let white_turn_three_mid_turn_full_resources = game.active_color == Color::White
        && game.turn_number == 3
        && game.mons_moves_count >= 3
        && game.player_can_use_action()
        && game.player_can_move_mana();

    if early_white_turn_start
        || white_turn_one_late_opening_tail
        || white_turn_three_turn_start_action_mana
        || white_turn_three_mid_turn_full_resources
    {
        let shipping_runtime = shipping_search_config_for_game(game, SmartAutomovePreference::Pro);
        return Some(select_shipping_search_inputs(game, shipping_runtime));
    }

    let white_turn_three_mana_only = game.active_color == Color::White
        && game.turn_number == 3
        && game.mons_moves_count == 1
        && !game.player_can_use_action()
        && game.player_can_move_mana();
    let white_turn_three_mid_turn = game.active_color == Color::White
        && game.turn_number == 3
        && game.mons_moves_count > 0
        && !white_turn_three_mana_only
        && (game.player_can_use_action() || game.player_can_move_mana());
    if !white_turn_three_mid_turn {
        return None;
    }

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
    if !drainer_vulnerable && !drainer_walk_vulnerable {
        return None;
    }

    let fast_runtime = shipping_search_config_for_game(game, SmartAutomovePreference::Fast);
    Some(select_shipping_search_inputs(
        game,
        MonsGameModel::with_pre_exact_runtime_policy(fast_runtime),
    ))
}

fn select_score_window_tactical_fallback_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Option<Vec<Input>> {
    let white_turn_three_mid_turn_scoring_action_mana = game.active_color == Color::White
        && game.turn_number == 3
        && matches!(game.mons_moves_count, 1 | 2)
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if !white_turn_three_mid_turn_scoring_action_mana {
        return None;
    }

    let context = crate::models::automove_exact::exact_opportunity_context(game, game.active_color);
    if context.delta.same_turn_score_window_value <= 0 {
        return None;
    }

    Some(select_search_inputs_with_fresh_frontier_cache(
        game,
        apply_frontier_pro_v2_guarded_config(config),
    ))
}

fn select_late_black_search_fallback_inputs(
    game: &MonsGame,
    frontier_inputs: &[Input],
) -> Option<Vec<Input>> {
    let black_turn_two_turn_start_action_mana = game.active_color == Color::Black
        && game.turn_number == 2
        && game.mons_moves_count == 0
        && game.player_can_use_action()
        && game.player_can_move_mana();
    let black_turn_two_mana_only = game.active_color == Color::Black
        && game.turn_number == 2
        && game.mons_moves_count > 0
        && !game.player_can_use_action()
        && game.player_can_move_mana();
    let black_turn_four_turn_start_action_mana = game.active_color == Color::Black
        && game.turn_number == 4
        && game.mons_moves_count == 0
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if black_turn_two_turn_start_action_mana
        || black_turn_two_mana_only
        || black_turn_four_turn_start_action_mana
    {
        let shipping_runtime = shipping_search_config_for_game(game, SmartAutomovePreference::Pro);
        return Some(select_shipping_search_inputs(game, shipping_runtime));
    }

    if frontier_inputs.is_empty() {
        return None;
    }

    let shipping_runtime = shipping_search_config_for_game(game, SmartAutomovePreference::Pro);
    let shipping_inputs = select_shipping_search_inputs(game, shipping_runtime);
    let shipping_fen = Input::fen_from_array(&shipping_inputs);

    let black_turn_four_bridge_shipping_fallback = game.active_color == Color::Black
        && game.turn_number == 4
        && game.mons_moves_count == 2
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if black_turn_four_bridge_shipping_fallback
        && !shipping_inputs.is_empty()
        && shipping_inputs != frontier_inputs
        && shipping_inputs.len() == 3
        && shipping_fen.ends_with(";mb")
    {
        return Some(shipping_inputs);
    }

    let black_mid_turn_action_mana_shipping_fallback = game.active_color == Color::Black
        && game.turn_number >= 4
        && game.mons_moves_count >= 3
        && game.player_can_use_action()
        && game.player_can_move_mana();
    if black_mid_turn_action_mana_shipping_fallback
        && !shipping_inputs.is_empty()
        && shipping_inputs != frontier_inputs
    {
        return Some(shipping_inputs);
    }

    None
}

fn execute_frontier_candidate_inputs(game: &MonsGame, config: AutomoveSearchConfig) -> Vec<Input> {
    select_search_inputs_with_fresh_frontier_cache(
        game,
        apply_frontier_pro_v2_guarded_config(config),
    )
}

pub(crate) fn select_frontier_pro_v2_guarded_inputs(
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    if let Some(inputs) = select_opening_book_fallback_inputs(game) {
        #[cfg(test)]
        set_frontier_runtime_variant_branch("opening_book_fallback");
        return inputs;
    }
    if let Some(inputs) = select_early_white_fallback_inputs(game, config) {
        #[cfg(test)]
        set_frontier_runtime_variant_branch("early_white_fallback");
        return inputs;
    }
    if let Some(inputs) = select_score_window_tactical_fallback_inputs(game, config) {
        #[cfg(test)]
        set_frontier_runtime_variant_branch("score_window_tactical_fallback");
        return inputs;
    }

    let frontier_inputs = execute_frontier_candidate_inputs(game, config);
    if let Some(inputs) = select_late_black_search_fallback_inputs(game, frontier_inputs.as_slice())
    {
        #[cfg(test)]
        set_frontier_runtime_variant_branch("late_black_shipping_fallback");
        return inputs;
    }
    #[cfg(test)]
    set_frontier_runtime_variant_branch("frontier_execute");
    frontier_inputs
}

pub(crate) fn turn_engine_config_from_search_config(
    config: AutomoveSearchConfig,
) -> TurnEngineConfig {
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
