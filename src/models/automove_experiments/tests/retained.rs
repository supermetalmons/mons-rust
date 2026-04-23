use super::*;
use crate::models::mons_game_model::automove_runtime_variants::select_frontier_pro_v2_guarded_inputs;

#[test]
fn frontier_pro_v2_guarded_profile_prefers_safe_white_opening_turn_one_tail_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 2 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xS0xn05/n03E0xA0xn02Y0xn03",
        false,
    )
    .expect("white opening turn-one tail fen should be valid");
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l10,3;l9,3"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_three_move_opening_tail() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 3 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xn01S0xn04/n02E0xn01A0xn02Y0xn03",
        false,
    )
    .expect("white three-move opening tail fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l10,7;l9,7"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_keeps_shipping_search_path_on_engine_disabled_opening() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 2 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04E0xn01D0xn04/n04A0xn01S0xY0xn03",
        false,
    )
    .expect("engine-disabled opening search-path fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l9,6;l8,6"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_turn_two_mana_only_root() {
    let game = MonsGame::from_fen(
        "0 0 b 1 0 2 0 0 2 n03y0xn02a0xe0xn03/n05s0xd0xn04/n07xxmn03/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n04D0xn06/n02E0xA0xn01S0xn05/n07Y0xn03",
        false,
    )
    .expect("black turn-two mana-only loss fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l0,3;l1,3"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_turn_four_start_action_mana_root() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 0 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n05S0xn05/n04E0xA0xn05/n07Y0xn02D0x",
        false,
    )
    .expect("black turn-four action+mana loss fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l1,5;l3,3;l2,2"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_turn_three_full_resources_root() {
    let fixture = primary_pro_fixture_by_id("primary_white_mana_sibling_ply9");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &fixture.game
        ),
        "l5,0;l4,1"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_keeps_v30_white_turn_three_mana_only_vulnerable_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03xxMn02xxMn04/n04D0xn06/n04E0xn01S0xn04/n04A0xn02Y0xn03",
        false,
    )
    .expect("white turn-three mana-only vulnerable fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l8,4;l7,3"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_keeps_v30_white_turn_three_mana_only_non_vulnerable_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n05S0xn05/n03E0xA0xD0xn02Y0xn02",
        false,
    )
    .expect("white turn-three mana-only non-vulnerable fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l10,8;l9,7"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_v30_white_opening_spirit_sibling_pro_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n02E0xn01A0xD0xS0xY0xn03",
        false,
    )
    .expect("white opening spirit sibling pro fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game,
        ),
        "l10,6;l9,6"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_v30_white_turn_four_mana_sibling_normal_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 4 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMY0xn03/n05S0xn05/n04A0xD0xn05/n02E0xn08",
        false,
    )
    .expect("white turn-four mana sibling normal fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l7,7;l6,6"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_v30_white_turn_three_mana_sibling_pro_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 3 0 0 3 n03y0xn03e0xn03/n05a0xn05/n02xxmn01s0xn01d0xn04/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMxxMn01xxMn04/E0xn04S0xn05/n03A0xn01D0xn05/n08Y0xn02",
        false,
    )
    .expect("white turn-three mana sibling pro fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l9,3;l10,4"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_v30_white_turn_four_mana_sibling_fast_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 4 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn03Y0xn03/n03E0xn01S0xn05/n04A0xD0xn05",
        false,
    )
    .expect("white turn-four mana sibling fast fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l10,4;l9,4"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_v30_black_plain_spirit_sibling_full_reliability_pro_root(
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
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l0,4;l1,4"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_v30_black_late_progress_over_non_concrete_window_fast_root(
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
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l3,2;l4,1"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_rejects_v30_white_recovery_head_full_reliability_normal_root() {
    let game = MonsGame::from_fen(
        "1 0 w 0 0 0 0 0 7 n11/n06a0xn01e0xn02/n05d0mn05/n03xxmxxmn02xxmn03/n05xxmxxUn04/y0xn03xxMn01s0xn03xxQ/n06Y0xxxMn03/n03xxMn07/n05S0xxxMn04/n04A0xn06/D0xn01E0xn08",
        false,
    )
    .expect("full reliability normal white recovery-head fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l8,5;l7,3;l8,2"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_does_not_seed_cached_plain_spirit_continuation_when_head_is_rejected(
) {
    fn game_with_items(items: Vec<(Location, Item)>, active_color: Color) -> MonsGame {
        let mut game = MonsGame::new(false, GameVariant::Classic);
        game.replace_board_items(items);
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
        "frontier_pro_v2_guarded",
        &game,
        SmartAutomovePreference::Pro,
    );
    let first = select_frontier_pro_v2_guarded_inputs(&game, config);
    assert_eq!(Input::fen_from_array(&first), "l9,7;l7,8;l7,7");
    let after_first = MonsGameModel::apply_inputs_for_search(&game, first.as_slice())
        .expect("v30 first spirit-setup chunk should be legal");
    let after_config = calibration_runtime_config(
        "frontier_pro_v2_guarded",
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
fn frontier_pro_v2_guarded_prefers_safe_black_opening_a_ply19_root() {
    let fixture = primary_pro_fixture_by_id("primary_black_loss_opening_a_ply19");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen("frontier_pro_v2_guarded", fixture.mode, &fixture.game),
        "l2,5;l1,4"
    );
}

#[test]
fn frontier_pro_v2_guarded_prefers_safe_black_plain_spirit_followup_root() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 1 0 0 4 n05d0xa0xn04/n05s0xxxme0xn03/n11/n04xxmn06/n02y0xxxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n06S0xn04/n02E0xn01A0xn03Y0xn02/D0xn10",
        false,
    )
    .expect("valid black plain spirit followup fixture fen");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l4,2;l5,1"
    );
}

#[test]
fn frontier_pro_v2_guarded_prefers_concrete_white_spirit_followup_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 5 0 0 3 n05d2xa0xn04/n05s0xn01e0xn03/n03y0xn03xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn01S0xn01/xxQn04xxUn05/n03xxMn01xxMn01xxMn03/n04D0Mn01xxMn04/n11/n04A0xn06/n03E0xn03Y0xn03",
        false,
    )
    .expect("valid white spirit followup fixture fen");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l4,9;l4,7;l5,7"
    );
}

#[test]
fn frontier_pro_v2_guarded_prefers_searched_white_progress_tail_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 7 n11/n05d0xa0xn01e0xn02/n06s0xS0xxxmn02/n02xxmxxmxxmn06/n08xxmn02/y0xn04xxUn05/n05xxMn01xxMn03/n04xxMn01xxMn04/n01E0xxxMn08/n04A0xD0xY0xn04/n11",
        false,
    )
    .expect("valid white progress tail fixture fen");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l9,5;l8,4"
    );
}

#[test]
fn frontier_pro_v2_guarded_prefers_spirit_reentry_on_fast_flat_opening_ply37_root() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        false,
    )
    .expect("valid fast flat ply37 fen");

    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
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
fn frontier_pro_v2_guarded_rejects_late_black_plain_spirit_progress_head_without_concrete_gain() {
    let fixture = primary_pro_fixture_by_id("primary_black_late_accepted_head_ply4");
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
        profile_decision_move_fen("frontier_pro_v2_guarded", fixture.mode, &fixture.game),
        "l3,2;l4,1",
    );
}

#[test]
fn frontier_pro_v2_guarded_rejects_white_fast_deferred_recovery_progress_head_without_concrete_gain(
) {
    let fixture = primary_pro_fixture_by_id("primary_white_fast_accepted_head_ply13");
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
        profile_decision_move_fen("frontier_pro_v2_guarded", fixture.mode, &fixture.game),
        "l8,7;l7,8",
    );
}

#[test]
fn frontier_pro_v2_guarded_rejects_v30_white_vulnerable_progress_head_flat_nonwin_normal_root() {
    let game = MonsGame::from_fen(
        "1 0 w 1 0 1 0 0 5 n11/n05a0xn02e0xn02/n03y0xd0ms0xn05/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n11/n03xxMn01A0xn05/n01D0xn04Y0xS0xn03/n03E0xn07",
        false,
    )
    .expect("white vulnerable progress flat non-win fen should be valid");
    let (config, scored_roots, head_plan, forced_engine_inputs) =
        profile_runtime_scored_roots_with_forced_engine_inputs(
            "frontier_pro_v2_guarded",
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
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l9,7;l8,7",
    );
}

#[test]
fn frontier_pro_v2_guarded_accepts_v30_white_head_flat_nonwin_normal_root() {
    let game = MonsGame::from_fen(
        "1 0 w 0 0 1 0 0 9 n02a0xy1xn07/n01d0mn09/n02xxmn02s0xn01e0xn03/n03xxmn03xxmn03/E0xn03xxmn06/n05xxUn04xxQ/n05xxMn01xxMn03/n07S0xn03/n02xxMxxMn01A0xn05/n04D0xn01Y0xn04/n11",
        false,
    )
    .expect("white flat nonwin normal accepted-head fen should be valid");
    let (config, scored_roots, head_plan, _) =
        profile_runtime_scored_roots_with_forced_engine_inputs(
            "frontier_pro_v2_guarded",
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
    assert_eq!(
        Input::fen_from_array(&pre_accept_selected),
        "l7,7;l5,5;l5,6"
    );
    assert_eq!(Input::fen_from_array(head_inputs), "l7,7;l5,5;l6,4");
    assert!(accepted);
    assert_eq!(
        profile_decision_move_fen(
            "frontier_pro_v2_guarded",
            SmartAutomovePreference::Pro,
            &game
        ),
        "l7,7;l5,5;l6,4"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_flat_nonwin_normal_root() {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 6 n11/n05d0xa0xe0xn03/n05s0xxxmn04/n02xxmxxmy0xn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n01E0xn01xxMY0xn01S0xxxMn03/n04xxMn06/n05D0Mn02xxMn02/n05A0xn05/n11",
        false,
    )
    .expect("black flat nonwin normal fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let (config, scored_roots, head_plan, _) =
        profile_runtime_scored_roots_with_forced_engine_inputs(
            "frontier_pro_v2_guarded",
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
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    println!(
        "BLACK_NORMAL_FLAT_NONWIN_ADVISOR selected={} advisor={:?}",
        runtime_selected,
        pro_v2_root_advisor_decision_snapshot(),
    );
    assert_eq!(runtime_selected, "l2,5;l1,7;l2,7");
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_flat_nonwin_fast_root() {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 5 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/n05xxUn04xxQ/n02y0xxxMn01xxMn01xxMn03/n04xxMn06/n03E0xA0xn03xxMn02/n06S0xn04/n05D2xn03Y0xn01",
        false,
    )
    .expect("black flat nonwin fast fen should be valid");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    let (config, scored_roots, head_plan, _) =
        profile_runtime_scored_roots_with_forced_engine_inputs(
            "frontier_pro_v2_guarded",
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
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    println!(
        "BLACK_FAST_FLAT_NONWIN_ADVISOR selected={} advisor={:?}",
        runtime_selected,
        pro_v2_root_advisor_decision_snapshot(),
    );
    assert_eq!(runtime_selected, "l1,5;l3,3;l2,2");
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_late_head_duel_normal_root() {
    let game = primary_pro_fixture_by_id("primary_black_late_accepted_head_ply4").game;

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);

    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    println!(
        "BLACK_LATE_HEAD_DUEL_NORMAL shipping_selected={} context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} probe={:?} advisor={:?}",
        shipping_selected,
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
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_recovery_duel_fast_root() {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 3 0 0 4 n06a0xn04/n06d0xe0xn03/n04s0xn02xxmn03/n03xxmn07/n01y0xn01xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn03xxMn04/n03xxMD0xn06/n03A0xE0xn01S0xn04/n08Y0xn02",
        false,
    )
    .expect("black fast recovery duel fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);

    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    println!(
        "BLACK_RECOVERY_DUEL_FAST shipping_selected={} context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} probe={:?} advisor={:?}",
        shipping_selected,
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
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_spirit_bridge_duel_fast_root() {
    let game = MonsGame::from_fen(
        "1 1 b 0 0 3 1 0 8 n10d0x/n07a0xn03/n05s0xn05/n02xxmxxmy0xn02xxmn03/n05xxmn03e0xn01/E0xn09xxQ/n03xxMY0xxxMxxUxxMn03/n03S0xn07/n06D0Mn04/n05A0xn05/n11",
        false,
    )
    .expect("black fast spirit bridge duel fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);

    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    println!(
        "BLACK_SPIRIT_BRIDGE_DUEL_FAST shipping_selected={} context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} probe={:?} advisor={:?}",
        shipping_selected,
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
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_late_mana_sibling_duel_normal_root() {
    let game = MonsGame::from_fen(
        "2 1 w 0 0 0 0 0 11 d0xa0xn09/n01xxmn01y0xn07/n05s0xn02xxmn02/n03xxmn07/E0xn03xxmn01e0xn04/n10xxQ/n04xxUxxMn05/n07S0xxxMn02/n02xxMn02A0xn05/n06Y0xn04/D0xn10",
        false,
    )
    .expect("white late mana sibling duel normal fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);
    let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );

    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    let shipping_root = format_root_probe(
        scored_roots
            .iter()
            .find(|root| Input::fen_from_array(&root.inputs) == shipping_selected),
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
        "WHITE_LATE_MANA_SIBLING_DUEL_NORMAL shipping_selected={} shipping_root=\"{}\" context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} top_root_details={:?} probe={:?} advisor={:?}",
        shipping_selected,
        shipping_root,
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
fn frontier_pro_v2_guarded_uses_white_early_engine_disabled_fallback_on_normal_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 0 0 0 5 n06a0xn04/n07d0me0xn02/n02y0xn01s0xn06/n04xxmn01xxmxxmn03/n03xxmn07/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn02Y0xxxMn04/n03xxMn01D0xn05/n03E0xA0xS0xn05/n11",
        false,
    )
    .expect("white early engine-disabled normal fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "WHITE_EARLY_ENGINE_DISABLED_NORMAL shipping_selected={} context={} probe={:?}",
        shipping_selected,
        exact_opportunity_context_probe(&game),
        probe,
    );
    assert_eq!(shipping_selected, "l9,5;l8,3;l7,4");
    assert_eq!(probe.selected_input_fen, "l9,5;l8,3;l7,4");
    assert_eq!(probe.pre_accept_input_fen, "l8,5;l7,6");
    assert_eq!(
        probe.runtime_variant_branch,
        "white_early_engine_disabled_fallback"
    );
}

#[test]
fn frontier_pro_v2_guarded_uses_white_nonnegative_deny_search_only_fallback_on_fast_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn07/n04D0xS0xn01Y0xn03/n02E0xn01A0xn06",
        false,
    )
    .expect("white fast ply9 search-order fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "WHITE_NONNEGATIVE_DENY_SEARCH_ONLY_FAST shipping_selected={} context={} probe={:?}",
        shipping_selected,
        exact_opportunity_context_probe(&game),
        probe,
    );
    assert_eq!(shipping_selected, "l9,4;l8,5");
    assert_eq!(probe.selected_input_fen, "l9,4;l8,5");
    assert_eq!(probe.pre_accept_input_fen, "l9,4;l8,3");
    assert_eq!(
        probe.runtime_variant_branch,
        "white_nonnegative_deny_search_only_fallback"
    );
}

#[test]
fn frontier_pro_v2_guarded_uses_white_negative_deny_search_only_selected_rank_fallback_on_normal_root(
) {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 1 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn04/n03xxMn02Y0xn04/n04D0xS0xn05/n03E0xA0xn06",
        false,
    )
    .expect("white normal ply11 search-order fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "WHITE_NEGATIVE_DENY_SELECTED_RANK_NORMAL shipping_selected={} context={} probe={:?}",
        shipping_selected,
        exact_opportunity_context_probe(&game),
        probe,
    );
    assert_eq!(shipping_selected, "l9,4;l8,5");
    assert_eq!(probe.selected_input_fen, "l9,4;l8,5");
    assert_eq!(probe.pre_accept_input_fen, "l9,4;l8,3");
    assert_eq!(
        probe.runtime_variant_branch,
        "white_negative_deny_search_only_selected_rank_fallback"
    );
}

#[test]
fn frontier_pro_v2_guarded_rejects_white_turn_five_same_window_mana_head_normal_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 0 0 0 5 n06a0xn04/n07d0me0xn02/n02y0xn01s0xn06/n04xxmn01xxmxxmn03/n03xxmn07/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn01xxMY0xxxMn04/n05D0xn05/n03E0xA0xS0xn05/n11",
        false,
    )
    .expect("white normal turn-five same-window head fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "WHITE_TURN_FIVE_SAME_WINDOW_MANA_HEAD shipping_selected={} context={} probe={:?}",
        shipping_selected,
        exact_opportunity_context_probe(&game),
        probe,
    );
    assert_eq!(shipping_selected, "l8,5;l7,4");
    assert_eq!(probe.selected_input_fen, "l8,5;l7,4");
    assert_eq!(probe.pre_accept_input_fen, "l8,5;l7,4");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l8,5;l7,6"));
    assert!(!probe.head_accepted);
}

#[test]
fn frontier_pro_v2_guarded_rejects_white_turn_five_same_window_mana_head_action_normal_root() {
    let game = MonsGame::from_fen(
        "0 1 w 0 0 0 0 0 5 n06a0xn03d0x/n08e0xn02/n02y0xn01s0xn06/n04xxmn03xxmn02/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n03E0xA0xD0xn05/n05S0xn01Y0xn03/n11",
        false,
    )
    .expect("white normal turn-five same-window action head fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "WHITE_TURN_FIVE_SAME_WINDOW_MANA_ACTION_HEAD shipping_selected={} context={} probe={:?}",
        shipping_selected,
        exact_opportunity_context_probe(&game),
        probe,
    );
    assert_eq!(shipping_selected, "l8,5;l7,4");
    assert_eq!(probe.selected_input_fen, "l8,5;l7,4");
    assert_eq!(probe.pre_accept_input_fen, "l8,5;l7,4");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l8,5;l7,6"));
    assert!(!probe.head_accepted);
}

#[test]
fn frontier_pro_v2_guarded_rejects_white_turn_five_mid_turn_window_mana_head_normal_root() {
    let game = MonsGame::from_fen(
        "0 1 w 1 0 1 0 0 5 n06a0xn03d0x/n08e0xn02/n02y0xn01s0xn06/n04xxmn03xxmn02/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n03D0Mn02xxMn04/n03E0xA0xn06/n05S0xn01Y0xn03/n11",
        false,
    )
    .expect("white mid-turn window mana head normal fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "WHITE_TURN_FIVE_MID_TURN_WINDOW_MANA_HEAD shipping_selected={} context={} probe={:?}",
        shipping_selected,
        exact_opportunity_context_probe(&game),
        probe,
    );
    assert_eq!(shipping_selected, "l8,3;l7,4");
    assert_eq!(probe.selected_input_fen, "l8,3;l7,4");
    assert_eq!(probe.pre_accept_input_fen, "l8,3;l7,4");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l7,3;l8,2"));
    assert!(!probe.head_accepted);
}

#[test]
fn frontier_pro_v2_guarded_rejects_white_turn_five_spirit_setup_pre_accept_fast_root() {
    let game = MonsGame::from_fen(
        "0 0 w 0 0 2 0 0 5 n05d0xn05/n06a0xn04/n03xxmn01s0xn05/n02xxmn03e0xn01xxmn02/n03y0xn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n11/n02xxMn03S0xD0MY0xn02/n04A0xn06/n02E0xn08",
        false,
    )
    .expect("white turn-five spirit setup pre-accept fast fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "WHITE_TURN_FIVE_SPIRIT_SETUP_PRE_ACCEPT_FAST shipping_selected={} context={} probe={:?}",
        shipping_selected,
        exact_opportunity_context_probe(&game),
        probe,
    );
    assert_eq!(shipping_selected, "l8,6;l6,5;l6,4");
    assert_eq!(probe.selected_input_fen, "l8,6;l6,5;l6,4");
    assert_eq!(probe.pre_accept_input_fen, "l8,6;l6,5;l6,4");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l8,7;l9,8"));
    assert!(!probe.head_accepted);
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_post_search_duel_normal_root() {
    let game = MonsGame::from_fen(
        "0 1 b 0 0 0 0 0 8 n10d0x/n06a0xn04/n05s0xn01e0xn03/n02xxmxxmy0xn06/E0xn10/n04xxmxxUxxmn03xxQ/n03xxMY0xn01S0xxxMn03/n04D0Mn06/n04xxMA0xn05/n09xxMn01/n11",
        false,
    )
    .expect("black post-search duel normal fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);
    let (_, scored_roots, _, _) = profile_runtime_scored_roots_with_forced_engine_inputs(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );

    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    let shipping_root = format_root_probe(
        scored_roots
            .iter()
            .find(|root| Input::fen_from_array(&root.inputs) == shipping_selected),
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
        "BLACK_POST_SEARCH_DUEL_NORMAL shipping_selected={} shipping_root=\"{}\" context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} top_root_details={:?} probe={:?} advisor={:?}",
        shipping_selected,
        shipping_root,
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
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_bridge_nonwin_duel_fast_root() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 4 n06a0xn04/n05s0xd0xe0xn03/n07xxmn03/n03xxmn07/n01y0xn01xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n11/n02E0xA0xn01S0xn01Y0xn03/D0xn10",
        false,
    )
    .expect("black bridge non-win duel fast fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let (legacy_selected, legacy_full_pool_selected, legacy_candidates, legacy_full_pool) =
        pro_v2_legacy_selector_probe(&game, SmartAutomovePreference::Pro);

    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);
    println!(
        "BLACK_BRIDGE_NONWIN_DUEL_FAST shipping_selected={} context={} legacy_selected={} legacy_full_pool_selected={} legacy_candidates={:?} legacy_full_pool={:?} probe={:?} advisor={:?}",
        shipping_selected,
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
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_post_search_duel_pro_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_POST_SEARCH_DUEL_PRO",
        "1 1 w 1 0 0 0 0 5 n10d0x/n03y0xn03a0xn03/n01xxmn04s0xn01e0xn02/n04xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n06xxMn04/n02xxMn02S0xn05/n05A0xY0xn04/D0xn02E0xn07",
        "l9,6;l8,7",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_flat_nonwin_duel_pro_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_FLAT_NONWIN_DUEL_PRO",
        "0 0 w 0 0 1 0 0 3 n03y0xn03e0xn03/n05s0xa0xn01d0mn02/n11/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n01E0xn05Y0xn03/n04D0xn01S0xn04/n04A0xn06",
        "l8,7;l7,8",
    );
}

#[test]
fn frontier_pro_v2_guarded_rejects_black_post_search_spirit_reentry_duel_pro_root() {
    let game = MonsGame::from_fen(
        "1 0 b 0 0 0 0 0 6 n05d1xa0xn04/n05s0xn01e0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/n05xxMn05/n01y0xn01xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        false,
    )
    .expect("black post-search duel pro fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "BLACK_POST_SEARCH_DUEL_PRO shipping_selected={} probe={:?} advisor={:?}",
        shipping_selected,
        probe,
        pro_v2_root_advisor_decision_snapshot(),
    );
    assert_eq!(shipping_selected, "l0,6;l1,6");
    assert_eq!(probe.pre_accept_input_fen, "l0,6;l1,6");
    assert_eq!(probe.selected_input_fen, "l0,6;l1,6");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l1,5;l1,7;l0,7"));
    assert!(!probe.head_accepted);
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_head_nonwin_duel_pro_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_HEAD_NONWIN_DUEL_PRO",
        "0 0 b 0 0 2 0 0 2 n03y0xn01d0xn01e0xn03/n04s0xa0xn05/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05S0xn05/n03A0xn07/n02E0xn02D0xn02Y0xn02",
        "l1,4;l3,4;l3,3",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_head_black_fast_regression_reply_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_FAST_REGRESSION_REPLY",
        "0 0 b 0 0 0 0 0 2 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04E0xD0xS0xn04/n04A0xn04Y0xn01",
        "l0,4;l1,5",
    );
}

#[test]
fn frontier_pro_v2_guarded_rejects_black_followup_spirit_head_duel_pro_root() {
    let game = MonsGame::from_fen(
        "0 0 b 0 0 2 0 0 2 n03y0xn01d0xa0xn04/n04s0xn01e0xn04/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04A0xD0xn05/n03E0xn02S0xn02Y0xn01",
        false,
    )
    .expect("black followup spirit duel pro fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "BLACK_FOLLOWUP_SPIRIT_DUEL_PRO shipping_selected={} probe={:?} advisor={:?}",
        shipping_selected,
        probe,
        pro_v2_root_advisor_decision_snapshot(),
    );
    assert_eq!(shipping_selected, "l1,4;l3,4;l3,3");
    assert_eq!(probe.pre_accept_input_fen, "l1,4;l3,4;l3,3");
    assert_eq!(probe.selected_input_fen, "l1,4;l3,4;l3,3");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l1,4;l0,6;l1,7"));
    assert!(!probe.head_accepted);
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_mana_cluster_duel_pro_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_MANA_CLUSTER_DUEL_PRO",
        "2 1 w 0 0 0 0 0 7 n11/n01xxmn01y0xn03a0xd0mn02/n06s0xn01e0xn02/n04xxmn06/n05xxmn05/xxQn04xxUn04xxQ/n04xxMn02xxMn03/n06xxMn04/n05S0xn01Y0xn03/n05A0xn05/D0xn02E0xn07",
        "l8,5;l7,5",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_confirm_normal_ply49_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_CONFIRM_NORMAL_PLY49",
        "1 1 w 0 0 0 0 0 9 n11/n02y0xn01s0xn01a0xn04/n02xxmn04d0xn03/n06xxmn04/n04xxmn02xxmn03/xxQn04xxUn02Y0xn02/n04xxMn06/n05xxMn05/n02xxMn01S0xn03xxMn01e0x/n11/n02E0xn01A0xD0xn05",
        "l8,4;l8,2;l9,1",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_confirm_normal_ply26_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_CONFIRM_NORMAL_PLY26",
        "0 0 w 0 0 0 0 0 5 n05d1xn05/n06a0xn04/n02xxmn03s0xn04/n02y0xn01xxmn01xxmn04/n05xxmn01xxme0xn02/xxQn04xxUn05/n03xxMn01xxMn01xxMn03/n07xxMn03/n04xxMn06/n04E0xD0xS0xn04/n04A0xn02Y1xn03",
        "l9,6;l7,7;l7,8",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_confirm_normal_ply46_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_CONFIRM_NORMAL_PLY46",
        "1 1 b 0 0 0 0 0 8 E0xn02y0xn01d1xn05/n05s0xa0xe0xn03/n03xxmn03xxmn03/n11/n03xxmn03xxmn03/n05xxUn04xxQ/n03xxMxxMn02xxMn03/n11/n04A0xn01S0xn04/n05D0xxxMn01Y0xn02/n11",
        "l1,5;l2,3;l2,2",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_confirm_pro_ply23_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_CONFIRM_PRO_PLY23",
        "1 1 w 1 0 0 0 0 5 d0xn10/n05s0xa0xe0xn03/n03y0xn03xxmn03/n04xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn03xxMn03/n04xxMn06/n07xxMn03/n03A0xn01S0xn01Y0xn03/n03E0xn06D0x",
        "l10,3;l9,2",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_confirm_pro_ply9_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_CONFIRM_PRO_PLY9",
        "0 0 w 1 0 0 0 0 3 n11/n03y0xd0ms0xa0xe0xn03/n11/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn06/n07xxMn01Y0xn01/n05S0xn01D0xn03/n03E0xA0xn06",
        "l9,7;l10,8",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_confirm_pro_ply11_root() {
    let game = MonsGame::from_fen(
        "0 0 w 1 0 2 0 0 3 n11/n03y0xd0ms0xa0xe0xn03/n11/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn03Y0xn02/n07xxMn03/n05S0xn05/n03E0xA0xn03D0xn02",
        false,
    )
    .expect("white confirm pro ply11 fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "WHITE_CONFIRM_PRO_PLY11 shipping_selected={} context={} probe={:?}",
        shipping_selected,
        exact_opportunity_context_probe(&game),
        probe,
    );
    assert_eq!(shipping_selected, "l7,8;l6,9");
    assert_eq!(probe.selected_input_fen, "l7,8;l6,9");
    assert_eq!(probe.pre_accept_input_fen, "l10,4;l9,3");
    assert_eq!(
        probe.runtime_variant_branch,
        "white_confirm_prov1_search_only_tiebreak_fallback"
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_confirm_pro_ply15_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_CONFIRM_PRO_PLY15",
        "0 0 w 1 0 5 0 0 3 n11/n03y0xd0ms0xa0xe0xn03/n11/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04Y0x/n03xxMn01xxMn01xxMn03/n04xxMn06/n11/n05S0xn02D0Mn02/n03E0xA0xn06",
        "l6,3;l7,3",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_head_runtime_duel_pro_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_HEAD_RUNTIME_DUEL_PRO",
        "1 1 b 0 0 0 0 0 6 d0xn10/n05s0xa0xe0xn03/n03y0xn03xxmn03/n11/n04xxmxxmn01xxmn03/E0xn09xxQ/n05xxMxxUn04/n03xxMxxMn01S0xn04/n08xxMn02/n05A0xn05/n07Y0xn02D0x",
        "l1,5;l2,5",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_fast_ply10_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_FAST_PLY10",
        "0 0 w 0 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05D0xn05/n03E0xn01A0xn01S0xY0xn02/n11",
        "l9,7;l7,6;l7,7",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_engine_disabled_duel_fast_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_ENGINE_DISABLED_DUEL_FAST",
        "1 1 b 0 0 0 0 0 6 n06a0xn03d0x/n03y0xn01s0xn01e0xn03/n03xxmn07/n08xxmn02/n03xxmn01xxmn05/E0xn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n11/n04A0xD0MS0xn04/n08Y0xn02/n11",
        "l1,5;l2,3;l1,2",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_confirm_pro_ply16_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_CONFIRM_PRO_PLY16",
        "1 0 b 1 0 0 0 0 4 n11/n03y0xd0ms0xa0xe0xn03/n07xxmn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n04S0xn04Y0xn01/n11/n03E0xA0xn05D0x",
        "l1,6;l2,5",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_spirit_rerank_duel_pro_fast_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_SPIRIT_RERANK_DUEL_PRO_FAST",
        "2 0 b 0 0 0 0 0 8 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        "l1,5;l2,7;l1,8",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_shared_late_post_search_nonwin_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_SHARED_LATE_POST_SEARCH_NONWIN",
        "1 0 b 1 0 0 0 0 8 n05d0xn05/n05s0xa0xe0xxxmn02/n11/n02xxmxxmn03xxmn03/n05xxmn03Y0xn01/n05xxUn05/n05xxMn05/y0xn03S0xn06/n02xxMn04xxMxxMn02/n03D0xA0xn06/n03E1xn07",
        "l1,5;l2,5",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_early_post_search_nonwin_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_EARLY_POST_SEARCH_NONWIN",
        "1 0 b 1 0 0 0 0 6 n05d0xn03xxmn01/n03y0xn02a0xn04/n03xxmn01s0xn05/n02xxmn03e0xxxmn03/n05xxmn04Y0x/xxQn04xxUn05/n03xxMxxMn06/n06xxMxxMn03/n01E0xn03S0xn05/n03A0xn07/D0xn10",
        "l0,5;l1,4",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_turn_four_followup_nonwin_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_TURN_FOUR_FOLLOWUP_NONWIN",
        "0 0 b 1 0 1 0 0 4 n03y0xn03e0xn03/n05a0xn05/n02xxmn01s0xn02d0mn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/E0xn03xxMS0xn05/n05D0xn01Y0xn03/n04A0xn06",
        "l1,5;l1,6",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_late_post_search_nonwin_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_LATE_POST_SEARCH_NONWIN",
        "2 1 w 0 0 4 0 0 7 n11/n01xxmn01y0xn03a0xd0mn02/n06s0xn01e0xn02/n04xxmn06/n05xxmn05/xxQn04xxUn04Y0B/n04xxMn02xxMn03/n05S0xxxMn04/n11/n05A0xn05/D0xn02E0xn07",
        "l5,10;l4,10",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_harvest_followup_nonwin_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_HARVEST_FOLLOWUP_NONWIN",
        "0 0 w 0 0 2 0 0 3 n03y0xn03e0xn03/n05s0xa0xn01d0mn02/n11/n04xxmn02xxmn03/n03xxmn01xxmn05/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn01Y0xn02/n01E0xn09/n04D0xn01S0xn04/n04A0xn06",
        "l7,8;l6,9",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_white_late_cluster_nonwin_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "WHITE_LATE_CLUSTER_NONWIN",
        "1 1 w 0 0 0 0 0 5 d0xn10/n05s0xa0xe0xn03/n03y0xn03xxmn03/n11/n04xxmxxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn03xxMn02/n05S0xn05/n04E0xA0xn05/n07Y0xn02D0x",
        "l8,5;l6,3;l7,3",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_turn_ten_nonwin_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_TURN_TEN_NONWIN",
        "3 0 b 1 0 0 0 0 10 n09xxmn01/n05a0xn01e0xn03/n05s0xd0mn04/n02xxmxxmn07/n05xxmn02Y0xn02/n05xxUn05/y0xn04xxMn05/n03xxMn07/n04S0xn06/n02E0xn08/n04A0xn05D0x",
        "l2,5;l3,6",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_late_setup_reply_risk_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_LATE_SETUP_REPLY_RISK",
        "1 1 b 0 0 0 0 0 8 d0xn10/n05s0xa0xe0xxxmn02/n11/n07xxmn03/n03xxmn02xxmn04/n10xxQ/n02y0xn01D0UxxMn01xxMn03/n02xxMS0xn01A0xxxMn04/n06Y0xn04/n03E0xn07/n11",
        "l1,5;l3,7;l2,8",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_confirm_fast_setup_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_CONFIRM_FAST_SETUP_SPLIT",
        "2 1 b 0 0 0 0 0 10 n05d0xn03xxmn01/n04a0xn02e0xn03/n05s0xn05/E0xn02xxmn03xxmn03/n05xxmn05/n05xxUn04xxQ/n05xxMn05/n03S0xn01Y0xxxMn04/n03y0xn04xxMn02/n03A0xn07/n05D1xn05",
        "l2,5;l3,7;l2,8",
    );
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_late_fast_trace_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_LATE_FAST_TRACE",
        "3 1 b 1 0 2 0 0 14 n11/n07a0xd0xxxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
        "l1,8;l0,8",
    );
}

#[test]
fn frontier_pro_v2_guarded_keeps_recovery_on_black_late_fast_trace_root() {
    let game = MonsGame::from_fen(
        "3 1 b 0 0 0 0 0 14 n05d0xn05/n07a0xn01xxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
        false,
    )
    .expect("black late fast recovery trace fen should be valid");

    clear_turn_engine_selector_diagnostics();
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
        SmartAutomovePreference::Pro,
        &game,
    );
    let advisor = pro_v2_root_advisor_decision_snapshot();
    let shipping_selected =
        profile_decision_move_fen("shipping_pro_search", SmartAutomovePreference::Pro, &game);

    println!(
        "BLACK_LATE_FAST_RECOVERY_TRACE shipping_selected={} context={} probe={:?} advisor={:?}",
        shipping_selected,
        exact_opportunity_context_probe(&game),
        probe,
        advisor,
    );
    assert_eq!(shipping_selected, "l2,5;l0,5;l1,6");
    assert_eq!(probe.selected_input_fen, "l2,5;l0,5;l1,6");
    assert_eq!(probe.pre_accept_input_fen, "l2,5;l0,5;l1,6");
    assert_eq!(probe.head_input_fen.as_deref(), Some("l0,5;l1,6"));
    assert!(!probe.head_accepted);
}

#[test]
fn frontier_pro_v2_guarded_profile_prefers_shipping_black_late_fast_second_lane_nonwin_root() {
    assert_frontier_pro_v2_guarded_prefers_shipping_root_on_board(
        "BLACK_LATE_FAST_SECOND_LANE_NONWIN",
        "3 1 b 1 0 3 0 0 14 n08d0xn02/n07a0xn01xxmn01/n01xxmn03s0xn05/n03xxmn07/n05xxmn01e0xn01Y0xn01/n11/n04xxUn01S0xn04/n04xxMn06/n01y0xA0xn04xxMn03/n01D0xn09/n03E1xn07",
        "l0,8;l1,9",
    );
}

#[test]
fn frontier_pro_v2_guarded_avoids_vulnerable_safe_progress_on_black_opening_lane_nonwin_root() {
    let game = MonsGame::from_fen(
        "1 1 b 1 0 0 0 0 6 n03y0xn03e0xn02d0x/n01xxmn04a0xn04/n04s0xn06/n11/n03xxmn02xxmxxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n06xxMn04/E0xn04S0xn05/n01xxMn05Y0xn03/D0xn03A0xn06",
        false,
    )
    .expect("black opening lane nonwin fen should be valid");
    let probe = runtime_decision_probe(
        "frontier_pro_v2_guarded",
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
fn frontier_pro_v2_guarded_rejects_white_harvest_non_progress_window_injection() {
    let fixture = primary_pro_fixture_by_id("primary_white_harvest_loss_c_ply24");
    let config = calibration_runtime_config("frontier_pro_v2_guarded", &fixture.game, fixture.mode);
    let perspective = fixture.game.active_color;
    let mut root_moves = MonsGameModel::ranked_root_moves(&fixture.game, perspective, config);
    let engine_plan = turn_engine_candidate_plan(
        &fixture.game,
        perspective,
        MonsGameModel::turn_engine_config_for_game(&fixture.game, config),
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
        profile_decision_move_fen("frontier_pro_v2_guarded", fixture.mode, &fixture.game),
        "l7,2;l6,1",
    );
}
