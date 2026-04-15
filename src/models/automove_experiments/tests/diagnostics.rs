use super::*;

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

    with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
        println!(
            "pro reliability duel trace probe: frontier={} shipping={} repeats={} games_per_repeat={} max_plies={} trace_limit={}",
            frontier_profile,
            shipping_profile,
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
    });
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

    with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
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
    });
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
        config_tweak: Option<fn(AutomoveSearchConfig) -> AutomoveSearchConfig>,
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
        "pro reliability hotspot probe: frontier={} shipping={} positions={}",
        frontier_profile,
        shipping_profile,
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
            },
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
                let configured_runtime =
                    calibration_runtime_config("frontier_pro_v2_guarded", &case.game, case.mode);
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
#[ignore = "diagnostic: inspect retained pro-triage churn fixtures for frontier_pro_v2_guarded"]
fn smart_automove_pro_triage_retained_churn_probe() {
    let frontier_profile = "frontier_pro_v2_guarded";
    let shipping_profile = "shipping_pro_search";
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
        "retained churn probe: frontier={} shipping={} fixtures={}",
        frontier_profile,
        shipping_profile,
        fixture_ids.len()
    );
    for fixture_id in fixture_ids {
        let fixture = primary_pro_fixture_by_id(fixture_id);
        with_env_override("SMART_USE_WHITE_OPENING_BOOK", "false", || {
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
#[ignore = "diagnostic: inspect forced-turn-engine probe acceptance on retained churn fixtures"]
fn smart_automove_pro_forced_turn_engine_retained_churn_probe() {
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
