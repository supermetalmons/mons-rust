use super::super::profiles::profile_selector_from_name;
use super::super::*;
use super::runner::{select_inputs_with_runtime_fallback, tactical_game_with_items};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum TriageSurface {
    OpeningReply,
    PrimaryPro,
}

impl TriageSurface {
    pub(in super::super) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "opening_reply" => Some(Self::OpeningReply),
            "primary_pro" => Some(Self::PrimaryPro),
            _ => None,
        }
    }

    pub(in super::super) fn as_str(self) -> &'static str {
        match self {
            Self::OpeningReply => "opening_reply",
            Self::PrimaryPro => "primary_pro",
        }
    }
}

#[derive(Clone)]
pub(in super::super) struct TriageFixture {
    pub id: &'static str,
    pub game: MonsGame,
    pub mode: SmartAutomovePreference,
    pub opening_book_driven: bool,
    pub config_tweak: Option<fn(AutomoveSearchConfig) -> AutomoveSearchConfig>,
    pub expected_selected_input_fen: Option<&'static str>,
}
fn supermana_progress_triage_game() -> MonsGame {
    tactical_game_with_items(
        vec![
            (
                Location::new(6, 5),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                },
            ),
            (
                Location::new(10, 0),
                Item::Mon {
                    mon: Mon::new(MonKind::Angel, Color::White, 0),
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
    )
}

fn opponent_mana_progress_triage_game() -> MonsGame {
    tactical_game_with_items(
        vec![
            (
                Location::new(8, 5),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                },
            ),
            (
                Location::new(10, 0),
                Item::Mon {
                    mon: Mon::new(MonKind::Angel, Color::White, 0),
                },
            ),
            (
                Location::new(7, 5),
                Item::Mana {
                    mana: Mana::Regular(Color::Black),
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
    )
}

fn drainer_safety_triage_game() -> MonsGame {
    tactical_game_with_items(
        vec![
            (
                Location::new(8, 5),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                },
            ),
            (
                Location::new(9, 4),
                Item::Mon {
                    mon: Mon::new(MonKind::Angel, Color::White, 0),
                },
            ),
            (
                Location::new(6, 7),
                Item::Mon {
                    mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                },
            ),
        ],
        Color::White,
        2,
    )
}

fn extension_sensitive_no_ext_a_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 1 0 4 0 0 6 n03y0xn07/n07e0xn03/n06d0xa0xn03/n03xxmn03xxmn03/n02s0xxxmxxmn02xxmn03/xxQn04xxUn01xxMn02xxQ/n05xxMn05/n02xxMn03xxMn03Y0x/n03xxMn03D0xn03/n05A0xn05/n03E0xn02S0xn04",
        false,
    )
    .expect("extension_sensitive_no_ext_a: valid fen")
}

fn extension_sensitive_more_ext_a_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 2 0 0 6 n05d0xn01e0xn03/n03y0xn07/n02s0xn02xxmn01a0xn03/n11/n03xxmxxmxxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMxxMn02xxMn03/n06xxMn04/n04E0xn06/n03D0xxxMA0xS0xn02Y0xn01/n11",
        false,
    )
    .expect("extension_sensitive_more_ext_a: valid fen")
}

fn extension_sensitive_no_ext_b_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 6 n11/n05s0xe0xn04/n03y0xn03a0xn03/n04xxmn01d0mn04/n03xxmn01xxmn01xxmn03/xxQn01D0Mn02xxUxxMn03xxQ/n01xxMn05xxMn03/n11/n07xxMn03/n04Y0xn02S0xn03/n03E0xA0xn06",
        false,
    )
    .expect("extension_sensitive_no_ext_b: valid fen")
}

fn extension_sensitive_more_ext_b_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 1 0 5 0 0 5 n04s0xd0xa0xn02e0xn01/n02y0xn08/n11/n04xxmxxmxxmn04/n04xxmn06/xxQn04xxUxxmxxMn02xxQ/n03xxMn01D0Mn05/n04xxMn01xxMn04/n02E0xA0xn07/n06S0xn03Y0x/n11",
        false,
    )
    .expect("extension_sensitive_more_ext_b: valid fen")
}

fn harvested_black_override_loss_a_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 1 0 4 0 0 2 n05d0xa0xn04/n05s0xn01e0xn03/n03y0xn03xxmn03/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05D0xn05/n04A0xn01S0xn01Y0xn02/n03E0xn07",
        false,
    )
    .expect("harvested_black_override_loss_a: valid fen")
}

fn harvested_black_override_loss_b_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 1 0 4 0 0 2 n05d0xa0xn04/n05s0xn01e0xn03/n03y0xn03xxmn03/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n06S0xn04/n03E0xD0xn03Y0xn02/n04A0xn06",
        false,
    )
    .expect("harvested_black_override_loss_b: valid fen")
}

fn harvested_black_late_kill_loss_a_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 0 0 4 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/n05xxUn04xxQ/n01y0Bn03xxMn01xxMn03/n03xxMn02xxMn04/n05S0xn05/n04A0xn03Y0xn02/D0xn02E0xn07",
        false,
    )
    .expect("harvested_black_late_kill_loss_a: valid fen")
}

fn harvested_black_late_kill_loss_b_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 0 0 4 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/n05xxUn04xxQ/n01y0Bn03xxMn01xxMn03/n03xxMn02xxMn04/n06S0xn04/n03E0xA0xn03Y0xn02/D0xn10",
        false,
    )
    .expect("harvested_black_late_kill_loss_b: valid fen")
}

fn harvested_white_score_route_win_a_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 3 n05d0xn05/n05s0xa0xe0xn03/n03y0xn03xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04D0Mn01xxMn04/n11/n04A0xn01S0xn01Y0xn02/n03E0xn07",
        false,
    )
    .expect("harvested_white_score_route_win_a: valid fen")
}

fn harvested_white_score_route_win_b_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03",
        false,
    )
    .expect("harvested_white_score_route_win_b: valid fen")
}

fn white_mana_sibling_ply9_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 0 0 3 0 0 3 n06a0xn04/n02y0xn01s0xd0xxxmn04/n06e0xn04/n04xxmn06/n03xxmn01xxmn01xxmn03/E0Bn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n07S0xn03/n04A0xD0xn01Y0xn03",
        false,
    )
    .expect("white_mana_sibling_ply9: valid fen")
}

fn white_safe_progress_rerank_ply27_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 1 0 0 0 0 5 n05d1xn05/n01xxmn04a0xe0xn03/n11/n03y0xxxmn01xxmn04/n03s0xn01xxmn01xxmn03/n02E0xn02xxUn04xxQ/n03xxMn01xxMn05/n06xxMn04/n03xxMn04xxMn02/n04D0xn01S0xY0xn03/n04A0xn06",
        false,
    )
    .expect("white_safe_progress_rerank_ply27: valid fen")
}

fn derived_triage_game_after_inputs(start_fen: &str, input_fen: &str, label: &str) -> MonsGame {
    let game = MonsGame::from_fen(start_fen, false)
        .unwrap_or_else(|| panic!("{}: valid start fen", label));
    let inputs = Input::array_from_fen(input_fen);
    let (after, _) = MonsGameModel::apply_inputs_for_search_with_events(&game, inputs.as_slice())
        .unwrap_or_else(|| panic!("{}: inputs apply cleanly", label));
    after
}

fn black_loss_opening_a_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n02E0xn01A0xD0xS0xY0xn03",
        false,
    )
    .expect("black_loss_opening_a: valid fen")
}

fn black_loss_opening_b_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03",
        false,
    )
    .expect("black_loss_opening_b: valid fen")
}

fn black_loss_opening_a_after_white_triage_game() -> MonsGame {
    derived_triage_game_after_inputs(
        "0 0 w 0 0 1 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n02E0xn01A0xD0xS0xY0xn03",
        "l10,6;l9,6",
        "black_loss_opening_a_after_white",
    )
}

fn black_loss_opening_b_after_white_triage_game() -> MonsGame {
    derived_triage_game_after_inputs(
        "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03",
        "l10,7;l9,8",
        "black_loss_opening_b_after_white",
    )
}

fn derived_triage_game_after_white_opening_turn(start_fen: &str, label: &str) -> MonsGame {
    let mut game = MonsGame::from_fen(start_fen, false)
        .unwrap_or_else(|| panic!("{}: valid start fen", label));
    let start_turn = game.turn_number;
    let mut applied_steps = 0usize;

    while game.active_color == Color::White && game.turn_number == start_turn && applied_steps < 12
    {
        let opening_inputs = MonsGameModel::white_first_turn_opening_next_inputs(&game)
            .or_else(|| {
                let selector = profile_selector_from_name("shipping_pro_search")
                    .unwrap_or_else(|| panic!("{}: shipping_pro_search selector available", label));
                let config = SearchBudget::from_preference(SmartAutomovePreference::Pro)
                    .runtime_config_for_game(&game);
                Some(select_inputs_with_runtime_fallback(selector, &game, config))
            })
            .filter(|inputs| !inputs.is_empty())
            .unwrap_or_else(|| panic!("{}: white opening turn available", label));
        let (after, _) =
            MonsGameModel::apply_inputs_for_search_with_events(&game, opening_inputs.as_slice())
                .unwrap_or_else(|| panic!("{}: white opening turn applies", label));
        game = after;
        applied_steps += 1;
    }

    assert_eq!(
        game.active_color,
        Color::Black,
        "{}: expected black to move after white opening turn",
        label
    );
    assert!(
        game.turn_number > start_turn,
        "{}: expected opening turn to advance turn number",
        label
    );
    game
}

fn black_loss_opening_a_black_turn_triage_game() -> MonsGame {
    derived_triage_game_after_white_opening_turn(
        "0 0 w 0 0 1 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n02E0xn01A0xD0xS0xY0xn03",
        "black_loss_opening_a_black_turn",
    )
}

fn black_loss_opening_b_black_turn_triage_game() -> MonsGame {
    derived_triage_game_after_white_opening_turn(
        "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03",
        "black_loss_opening_b_black_turn",
    )
}

fn black_reduced_gate_opening_1_black_turn_triage_game() -> MonsGame {
    derived_triage_game_after_white_opening_turn(
        "0 0 w 0 0 2 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xA0xn05/n03E0xn02S0xY0xn03",
        "black_reduced_gate_opening_1_black_turn",
    )
}

fn black_reliability_opening_0_ba_black_turn_triage_game() -> MonsGame {
    derived_triage_game_after_white_opening_turn(
        "0 0 w 0 0 3 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05D0xn05/n06S0xn04/n03E0xA0xn02Y0xn03",
        "black_reliability_opening_0_ba_black_turn",
    )
}

fn derived_live_triage_game_after_white_baseline_turn(start_fen: &str, label: &str) -> MonsGame {
    let selector = profile_selector_from_name("shipping_pro_search")
        .unwrap_or_else(|| panic!("{}: shipping_pro_search selector available", label));
    let mut game = MonsGame::from_fen(start_fen, false)
        .unwrap_or_else(|| panic!("{}: valid start fen", label));
    let start_turn = game.turn_number;
    let mut applied_steps = 0usize;

    while game.active_color == Color::White && game.turn_number == start_turn && applied_steps < 12
    {
        let config = SearchBudget::from_preference(SmartAutomovePreference::Pro)
            .runtime_config_for_game(&game);
        let inputs = select_inputs_with_runtime_fallback(selector, &game, config);
        assert!(
            !inputs.is_empty(),
            "{}: white baseline opening turn available",
            label
        );
        assert!(
            matches!(game.process_input(inputs, false, false), Output::Events(_)),
            "{}: white baseline opening turn applies",
            label
        );
        applied_steps += 1;
    }

    assert_eq!(
        game.active_color,
        Color::Black,
        "{}: expected black to move after white baseline opening turn",
        label
    );
    assert!(
        game.turn_number > start_turn,
        "{}: expected live opening turn to advance turn number",
        label
    );
    game
}

fn black_reliability_opening_0_ba_live_black_turn_triage_game() -> MonsGame {
    derived_live_triage_game_after_white_baseline_turn(
        "0 0 w 0 0 3 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05D0xn05/n06S0xn04/n03E0xA0xn02Y0xn03",
        "black_reliability_opening_0_ba_live_black_turn",
    )
}

fn black_reliability_opening_1_ab_live_black_turn_triage_game() -> MonsGame {
    derived_live_triage_game_after_white_baseline_turn(
        "0 0 w 0 0 2 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xA0xn05/n03E0xn02S0xY0xn03",
        "black_reliability_opening_1_ab_live_black_turn",
    )
}

fn black_loss_runtime_b_ply3_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 2 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n04D0xn06/n05A0xS0xn01Y0xn02/n03E0xn07",
        false,
    )
    .expect("black_loss_runtime_b_ply3: valid fen")
}

fn black_harvest_loss_a_ply2_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 2 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05D0xn05/n04A0xn01S0xn01Y0xn02/n03E0xn07",
        false,
    )
    .expect("black_harvest_loss_a_ply2: valid fen")
}

fn black_harvest_loss_b_ply3_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 2 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n06S0xn04/n03E0xD0xn03Y0xn02/n04A0xn06",
        false,
    )
    .expect("black_harvest_loss_b_ply3: valid fen")
}

fn black_negative_deny_ply4_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 1 0 0 2 n03y0xn01d0xa0xe0xn03/n05s0xn05/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n05D0xn05/n03E0xA0xS0xn05/n07Y0xn03",
        false,
    )
    .expect("black_negative_deny_ply4: valid fen")
}

fn black_late_accepted_head_ply4_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 0 0 1 0 0 4 n06a0xn04/n05s0xd0xe0xn03/n07xxmn03/n02y0xxxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn03xxMn03/n11/n03E0xA0xS0xn05/D0xn06Y0xn03",
        false,
    )
    .expect("black_late_accepted_head_ply4: valid fen")
}

fn black_turn_four_action_mana_ply15_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 1 0 0 4 n06a0xn04/n06d0xe0xn03/n03y0xs0xn02xxmn03/n03xxmn07/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn03xxMn04/n03xxMD0xn06/n03A0xE0xn01S0xn04/n08Y0xn02",
        false,
    )
    .expect("black_turn_four_action_mana_ply15: valid fen")
}

fn black_mana_bridge_ply20_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn07/n01y0xn01xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n02xxMn03xxMn04/n06S0xn04/n02E0xn01A0xn06/D0xn06Y0xn03",
        false,
    )
    .expect("black_mana_bridge_ply20: valid fen")
}

fn black_spirit_bridge_ply19_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 4 n05d0xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn07/n01y0xn01xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn01xxMn03/n03xxMn02xxMn04/n05S0xn05/n02E0xn01A0xn06/D0xn06Y0xn03",
        false,
    )
    .expect("black_spirit_bridge_ply19: valid fen")
}

fn black_reliability_opening_3_ply3_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 2 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n03E0xA0xD0xS0xn01Y0xn02/n11",
        false,
    )
    .expect("black_reliability_opening_3_ply3: valid fen")
}

fn black_gate_loss_a_ply4_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 2 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xA0xS0xn01Y0xn02/n02E0xn08",
        false,
    )
    .expect("black_gate_loss_a_ply4: valid fen")
}

fn black_gate_loss_a_ply5_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 1 0 1 0 0 2 n03y0xn01d0xa0xe0xn03/n05s0xn05/n07xxmn03/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n04D0xn06/n05A0xS0xn01Y0xn02/n03E0xn07",
        false,
    )
    .expect("black_gate_loss_a_ply5: valid fen")
}

fn black_gate_loss_b_ply3_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 2 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n04D0xn06/n05A0xS0xn01Y0xn02/n03E0xn07",
        false,
    )
    .expect("black_gate_loss_b_ply3: valid fen")
}

fn black_loss_opening_a_ply6_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 2 0 0 2 n05d0xa0xe0xn03/n02y0xn01s0xn06/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xA0xS0xn01Y0xn02/n02E0xn08",
        false,
    )
    .expect("black_loss_opening_a_ply6: valid fen")
}

fn black_loss_opening_a_ply3_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 0 0 4 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xn01S0xn01Y0xn02/n02E0xn01A0xn06",
        false,
    )
    .expect("black_loss_opening_a_ply3: valid fen")
}

fn black_loss_opening_b_ply5_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 2 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n02E0xn01D0xA0xS0xn01Y0xn02/n11",
        false,
    )
    .expect("black_loss_opening_b_ply5: valid fen")
}

fn black_loss_opening_a_ply7_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 1 0 2 0 0 2 n05d0xa0xe0xn03/n03y0xn01s0xn05/n07xxmn03/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n04D0xA0xS0xn01Y0xn02/n02E0xn08",
        false,
    )
    .expect("black_loss_opening_a_ply7: valid fen")
}

fn black_loss_opening_c_ply6_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 1 0 2 0 0 2 n05d0xa0xe0xn03/n03y0xn01s0xn05/n07xxmn03/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n04D0xn06/n05A0xS0xn01Y0xn02/n03E0xn07",
        false,
    )
    .expect("black_loss_opening_c_ply6: valid fen")
}

fn black_loss_opening_a_ply19_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 1 0 0 0 0 4 n07e0xn03/n03y0xn01s0xa0xn04/n05d0mn01xxmn03/n02xxmn08/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n11/n05S0xn01xxMn03/n05A0xn02Y0xn02/D0xn01E0xn08",
        false,
    )
    .expect("black_loss_opening_a_ply19: valid fen")
}

fn black_reduced_gate_opening_1_ply19_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 1 0 1 0 0 4 n11/n04d0xs0xa0xe0xn03/n03y0xn03xxmn03/n02xxmxxmn07/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n06xxMn01xxMn02/n11/n05A0xS0xn01Y0xn02/D0xn02E0xn07",
        false,
    )
    .expect("black_reduced_gate_opening_1_ply19: valid fen")
}

fn black_reduced_gate_opening_1_ply21_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 1 0 3 0 0 4 n11/n05s0xa0xe0xn03/n05d0xn01xxmn03/n02xxmxxmy0xn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n06xxMn01xxMn02/n11/n05A0xS0xn01Y0xn02/D0xn02E0xn07",
        false,
    )
    .expect("black_reduced_gate_opening_1_ply21: valid fen")
}

fn black_reduced_gate_opening_1_ply31_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 0 0 0 0 0 6 n11/n05s0xn01e0xxxmn02/n06a0xn04/n02xxmxxmy0xd0xn05/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n05xxMn05/n02xxMn03xxMn04/n02E0xn01A0xn01S0xn01xxMn02/n08Y0xn02/D0xn10",
        false,
    )
    .expect("black_reduced_gate_opening_1_ply31: valid fen")
}

fn black_loss_opening_c_ply17_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 0 0 0 0 0 4 n07e0xn03/n03y0xn01s0xa0xn04/n05d0xn01xxmn03/n02xxmn01xxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n06xxMn01xxMn02/n11/n05A0xS0xn01Y0xn02/D0xn02E0xn07",
        false,
    )
    .expect("black_loss_opening_c_ply17: valid fen")
}

fn white_harvest_loss_a_ply2_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 0 0 4 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n06S0xn04/n04D0xn03Y0xn02/n03E0xA0xn06",
        false,
    )
    .expect("white_harvest_loss_a_ply2: valid fen")
}

fn white_harvest_loss_b_ply10_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 0 0 0 0 0 3 n03y0xn07/n05s0xn01e0xn01d0mn01/n07a0xn03/n04xxmn06/n03xxmn01xxmn05/xxQn09xxQ/n05xxMxxUxxMn03/n02E0xxxMxxMn01xxMS0xn03/n11/n04D0xn03Y0xn02/n03E0xA0xn06",
        false,
    )
    .expect("white_harvest_loss_b_ply10: valid fen")
}

fn white_harvest_loss_c_ply24_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 1 w 0 0 0 0 0 5 n03y0xn07/n05s0xn01e0xn01d0mn01/n07a0xn03/n04xxmn06/n03xxmn01xxmn05/xxQn09xxQ/n05xxMxxUxxMn03/n02E0xxxMxxMn01xxMS0xn03/n05D0xn05/n05A0xn02Y0xn02/n11",
        false,
    )
    .expect("white_harvest_loss_c_ply24: valid fen")
}

fn white_harvest_loss_d_ply25_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 1 w 1 0 0 0 0 5 n03y0xn07/n05s0xn01e0xn01d0mn01/n07a0xn03/n04xxmn06/n03xxmn01xxmn05/xxQn09xxQ/n05xxMxxUxxMn03/n02E0xxxMxxMn01D0MS0xn03/n11/n05A0xn02Y0xn02/n11",
        false,
    )
    .expect("white_harvest_loss_d_ply25: valid fen")
}

fn white_fast_accepted_head_ply13_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 0 0 1 0 0 3 n07e0xn03/n03y0xn01s0xn01a0xn03/n06d0xxxmn03/n03xxmxxmn06/n05xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n01E0xn05Y0xn03/n04D0xn01S0xn04/n04A0xn06",
        false,
    )
    .expect("white_fast_accepted_head_ply13: valid fen")
}

fn white_fast_screen_opening_0_ply9_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 w 0 0 0 0 0 3 n06a0xn04/n03y0xn01d0xxxmn01e0xn02/n04s0xn06/n04xxmn06/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n06xxMn01xxMn02/n11/n05D0xS0xn01Y0xn02/n02E0xn01A0xn06",
        false,
    )
    .expect("white_fast_screen_opening_0_ply9: valid fen")
}

fn black_gate_loss_b_ply31_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 0 0 0 6 n11/n06a0xn01e0xn02/n02xxmn01s0xd0mn05/n02xxmn08/n02y0xn02xxmn01xxmn03/xxQn04xxUn04xxQ/n07xxMn03/n05xxMxxMn04/n02xxMn08/n03A0xE0xn01S0xn01Y0xn02/D0xn10",
        false,
    )
    .expect("black_gate_loss_b_ply31: valid fen")
}

fn human_win_pro_a_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 w 0 0 1 1 0 5 n11/n02xxmn02a0xn05/n02y0xn01s0xn01d0xxxmn03/n02xxmn04e0xn03/n05xxmn01xxmn03/E0xn09xxQ/n03xxMS0xxxMxxUxxMn03/n04xxMn06/n05D0xn01xxMn03/n11/n04A0xn02Y0xn03",
        false,
    )
    .expect("human_win_pro_a: valid fen")
}

fn human_win_pro_b_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "2 0 w 0 0 1 0 0 7 n11/n05a0xn02xxmn02/n02y0xn01s0xn01d0xn04/n02xxmn03xxmn04/n02S0xn04xxmn03/E0xn07e0xn02/n03xxMn01xxMxxUxxMn03/n04xxMn06/n11/n05A0xn02xxMn02/n05D1xn01Y0xn03",
        false,
    )
    .expect("human_win_pro_b: valid fen")
}

fn human_win_pro_c_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "2 1 w 0 0 0 0 0 9 n09xxmd0x/n06a0xn04/n02y0xn01s0xn06/n02xxmn08/n07xxmn03/E0xn07e0xn02/n03xxMn01xxMn01xxMn03/n04xxMS0xn01xxUn03/n05A0xn05/n11/n05D0xn01Y0xn01xxMn01",
        false,
    )
    .expect("human_win_pro_c: valid fen")
}

fn live_nonwin_opening_reply_white_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 w 1 0 1 0 0 5 n01d0xn09/n01xxmn04a0xe0xn03/n03y0xn01s0xn01xxmn03/n11/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn05/n04xxMn02xxMn03/n05S0xn05/n04E0xA0xn01Y0xn03/n10D0x",
        false,
    )
    .expect("live_nonwin_opening_reply_white: valid fen")
}

fn live_nonwin_black_vulnerable_spirit_reentry_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "1 0 b 0 0 2 0 0 6 n05d1xn05/n05s0xa0xe0xn03/n07xxmn03/n03xxmn03xxmn03/n03xxmn01xxmn03Y0xn01/n05xxUn05/y0xn04xxMn05/n03xxMn03xxMn03/n07xxMn03/n02E0xn02S0xn05/n04A1xD1xn05",
        false,
    )
    .expect("live_nonwin_black_vulnerable_spirit_reentry: valid fen")
}

fn apply_triage_opening_sequence(game: &mut MonsGame, sequence: &[&str; 5]) {
    for step in sequence {
        let inputs = Input::array_from_fen(step);
        assert!(matches!(
            game.process_input(inputs, false, false),
            Output::Events(_)
        ));
    }
    assert_eq!(game.turn_number, 2);
    assert_eq!(game.active_color, Color::Black);
}

fn opening_black_reply_triage_fixture(id: &'static str, sequence: &[&str; 5]) -> TriageFixture {
    let mut game = MonsGame::new(false, GameVariant::Classic);
    apply_triage_opening_sequence(&mut game, sequence);
    TriageFixture {
        id,
        game,
        mode: SmartAutomovePreference::Pro,
        opening_book_driven: true,
        config_tweak: None,
        expected_selected_input_fen: None,
    }
}

pub(in super::super) fn opening_reply_triage_fixtures() -> Vec<TriageFixture> {
    vec![
        opening_black_reply_triage_fixture(
            "opening_left_route",
            &[
                "l10,3;l9,2",
                "l9,2;l8,1",
                "l8,1;l7,0",
                "l7,0;l6,0",
                "l6,0;l5,0;mp",
            ],
        ),
        opening_black_reply_triage_fixture(
            "opening_center_route",
            &[
                "l10,4;l9,4",
                "l9,4;l8,4",
                "l8,4;l7,3",
                "l7,3;l6,4",
                "l6,4;l5,4",
            ],
        ),
        opening_black_reply_triage_fixture(
            "opening_right_route",
            &[
                "l10,7;l9,8",
                "l9,8;l8,9",
                "l8,9;l7,10",
                "l7,10;l6,10",
                "l6,10;l5,10;mp",
            ],
        ),
    ]
}

pub(in super::super) fn primary_pro_triage_fixtures() -> Vec<TriageFixture> {
    vec![
        TriageFixture {
            id: "primary_supermana_progress",
            game: supermana_progress_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_opponent_mana_progress",
            game: opponent_mana_progress_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_drainer_safety",
            game: drainer_safety_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_ext_sensitive_no_ext_a",
            game: extension_sensitive_no_ext_a_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l2,6;l3,7"),
        },
        TriageFixture {
            id: "primary_ext_sensitive_more_ext_a",
            game: extension_sensitive_more_ext_a_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_ext_sensitive_no_ext_b",
            game: extension_sensitive_no_ext_b_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l3,6;l2,6"),
        },
        TriageFixture {
            id: "primary_ext_sensitive_more_ext_b",
            game: extension_sensitive_more_ext_b_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_harvest_black_override_loss_a",
            game: harvested_black_override_loss_a_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,6;l1,6"),
        },
        TriageFixture {
            id: "primary_harvest_black_override_loss_b",
            game: harvested_black_override_loss_b_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,6;l1,6"),
        },
        TriageFixture {
            id: "primary_harvest_black_late_kill_loss_a",
            game: harvested_black_late_kill_loss_a_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l6,1;l7,2"),
        },
        TriageFixture {
            id: "primary_harvest_black_late_kill_loss_b",
            game: harvested_black_late_kill_loss_b_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l6,1;l7,2"),
        },
        TriageFixture {
            id: "primary_harvest_white_score_route_win_a",
            game: harvested_white_score_route_win_a_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l7,4;l8,3"),
        },
        TriageFixture {
            id: "primary_harvest_white_score_route_win_b",
            game: harvested_white_score_route_win_b_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l10,5;l9,4"),
        },
        TriageFixture {
            id: "primary_white_mana_sibling_ply9",
            game: white_mana_sibling_ply9_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l5,0;l5,1"),
        },
        TriageFixture {
            id: "primary_white_safe_progress_rerank_ply27",
            game: white_safe_progress_rerank_ply27_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l9,4;l8,3"),
        },
        TriageFixture {
            id: "primary_live_nonwin_opening_reply_white",
            game: live_nonwin_opening_reply_white_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_live_nonwin_black_vulnerable_spirit_reentry",
            game: live_nonwin_black_vulnerable_spirit_reentry_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_black_loss_opening_a",
            game: black_loss_opening_a_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l10,6;l9,6"),
        },
        TriageFixture {
            id: "primary_black_loss_opening_b",
            game: black_loss_opening_b_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_black_loss_opening_a_after_white",
            game: black_loss_opening_a_after_white_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_black_loss_opening_b_after_white",
            game: black_loss_opening_b_after_white_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_black_loss_opening_a_black_turn",
            game: black_loss_opening_a_black_turn_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,4;l1,5"),
        },
        TriageFixture {
            id: "primary_black_reliability_opening_0_ba_black_turn",
            game: black_reliability_opening_0_ba_black_turn_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,4;l1,5"),
        },
        TriageFixture {
            id: "primary_black_reliability_opening_0_ba_live_black_turn",
            game: black_reliability_opening_0_ba_live_black_turn_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,4;l1,5"),
        },
        TriageFixture {
            id: "primary_black_reliability_opening_1_ab_live_black_turn",
            game: black_reliability_opening_1_ab_live_black_turn_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_black_loss_opening_b_black_turn",
            game: black_loss_opening_b_black_turn_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,4;l1,4"),
        },
        TriageFixture {
            id: "primary_black_reduced_gate_opening_1_black_turn",
            game: black_reduced_gate_opening_1_black_turn_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,4;l1,5"),
        },
        TriageFixture {
            id: "primary_black_loss_runtime_b_ply3",
            game: black_loss_runtime_b_ply3_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_black_harvest_loss_a_ply2",
            game: black_harvest_loss_a_ply2_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,3;l1,2"),
        },
        TriageFixture {
            id: "primary_black_harvest_loss_b_ply3",
            game: black_harvest_loss_b_ply3_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,4;l1,5"),
        },
        TriageFixture {
            id: "primary_black_negative_deny_ply4",
            game: black_negative_deny_ply4_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,5;l1,6"),
        },
        TriageFixture {
            id: "primary_black_late_accepted_head_ply4",
            game: black_late_accepted_head_ply4_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l1,5;l1,7;l0,7"),
        },
        TriageFixture {
            id: "primary_black_turn_four_action_mana_ply15",
            game: black_turn_four_action_mana_ply15_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l1,6;l2,7"),
        },
        TriageFixture {
            id: "primary_black_mana_bridge_ply20",
            game: black_mana_bridge_ply20_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,5;l1,4"),
        },
        TriageFixture {
            id: "primary_black_spirit_bridge_ply19",
            game: black_spirit_bridge_ply19_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l1,5;l1,7;l0,7"),
        },
        TriageFixture {
            id: "primary_black_reliability_opening_3_ply3",
            game: black_reliability_opening_3_ply3_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_black_gate_loss_a_ply4",
            game: black_gate_loss_a_ply4_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,4;l1,5"),
        },
        TriageFixture {
            id: "primary_black_gate_loss_a_ply5",
            game: black_gate_loss_a_ply5_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,3;l1,3"),
        },
        TriageFixture {
            id: "primary_black_gate_loss_b_ply3",
            game: black_gate_loss_b_ply3_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,3;l1,2"),
        },
        TriageFixture {
            id: "primary_black_loss_opening_a_ply6",
            game: black_loss_opening_a_ply6_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,5;l1,6"),
        },
        TriageFixture {
            id: "primary_black_loss_opening_a_ply3",
            game: black_loss_opening_a_ply3_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l10,4;l9,5"),
        },
        TriageFixture {
            id: "primary_black_loss_opening_b_ply5",
            game: black_loss_opening_b_ply5_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,4;l1,5"),
        },
        TriageFixture {
            id: "primary_black_loss_opening_a_ply7",
            game: black_loss_opening_a_ply7_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,7;l1,7"),
        },
        TriageFixture {
            id: "primary_black_loss_opening_c_ply6",
            game: black_loss_opening_c_ply6_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,7;l1,7"),
        },
        TriageFixture {
            id: "primary_black_loss_opening_a_ply19",
            game: black_loss_opening_a_ply19_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l2,5;l1,4"),
        },
        TriageFixture {
            id: "primary_black_reduced_gate_opening_1_ply19",
            game: black_reduced_gate_opening_1_ply19_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l1,4;l2,5"),
        },
        TriageFixture {
            id: "primary_black_reduced_gate_opening_1_ply21",
            game: black_reduced_gate_opening_1_ply21_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l1,6;l2,6"),
        },
        TriageFixture {
            id: "primary_black_reduced_gate_opening_1_ply31",
            game: black_reduced_gate_opening_1_ply31_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l1,5;l3,5;l3,6"),
        },
        TriageFixture {
            id: "primary_black_loss_opening_c_ply17",
            game: black_loss_opening_c_ply17_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l1,5;l3,4;l2,5"),
        },
        TriageFixture {
            id: "primary_white_harvest_loss_a_ply2",
            game: white_harvest_loss_a_ply2_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_white_harvest_loss_b_ply10",
            game: white_harvest_loss_b_ply10_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_white_harvest_loss_c_ply24",
            game: white_harvest_loss_c_ply24_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_white_harvest_loss_d_ply25",
            game: white_harvest_loss_d_ply25_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_white_fast_accepted_head_ply13",
            game: white_fast_accepted_head_ply13_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l9,4;l8,4"),
        },
        TriageFixture {
            id: "primary_white_fast_screen_opening_0_ply9",
            game: white_fast_screen_opening_0_ply9_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "primary_black_gate_loss_b_ply31",
            game: black_gate_loss_b_ply31_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "human_win_pro_a",
            game: human_win_pro_a_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "human_win_pro_b",
            game: human_win_pro_b_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
        TriageFixture {
            id: "human_win_pro_c",
            game: human_win_pro_c_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
        },
    ]
}
