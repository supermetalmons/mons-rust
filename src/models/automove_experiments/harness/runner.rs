use super::super::profiles::profile_selector_from_name;
use super::super::*;
use crate::models::automove_exact::clear_exact_state_analysis_cache;
use crate::models::automove_turn_engine::clear_turn_engine_plan_cache;
use crate::models::mons_game_model::clear_turn_engine_selector_diagnostics;
use std::collections::HashMap;

type OpeningFensCacheKey = (u64, usize);
type OpeningFens = Arc<Vec<String>>;
type OpeningFensCacheMap = HashMap<OpeningFensCacheKey, OpeningFens>;
type OpeningFensCache = Mutex<OpeningFensCacheMap>;

#[derive(Clone, Copy)]
pub(in super::super) struct CrossBudgetDuelConfig<'a> {
    pub profile_a: &'a str,
    pub mode_a: SmartAutomovePreference,
    pub profile_b: &'a str,
    pub mode_b: SmartAutomovePreference,
    pub seed_tag: &'a str,
    pub repeats: usize,
    pub games_per_repeat: usize,
    pub max_plies: usize,
    pub use_white_opening_book: bool,
}

pub(in super::super) fn select_inputs_with_runtime_fallback(
    selector: AutomoveSelector,
    game: &MonsGame,
    config: AutomoveSearchConfig,
) -> Vec<Input> {
    let inputs = selector(game, config);
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

pub(in super::super) fn run_budget_duel_series_with_timing(
    model_a: AutomoveModel,
    budget_a: SearchBudget,
    model_b: AutomoveModel,
    budget_b: SearchBudget,
    games: usize,
    seed: u64,
    max_plies: usize,
) -> TimedMatchupStats {
    let opening_fens = generate_opening_fens_cached(seed, games.max(1));
    let mut stats = TimedMatchupStats::default();
    for game_index in 0..games.max(1) {
        let a_is_white = game_index % 2 == 0;
        let (result, timing) = play_one_game_budget_duel_with_timing(
            model_a,
            budget_a,
            model_b,
            budget_b,
            a_is_white,
            opening_fens[game_index].as_str(),
            max_plies,
        );
        stats.record(result, timing);
    }
    stats
}

pub(in super::super) fn play_one_game_budget_duel_with_timing(
    model_a: AutomoveModel,
    budget_a: SearchBudget,
    model_b: AutomoveModel,
    budget_b: SearchBudget,
    a_is_white: bool,
    opening_fen: &str,
    max_plies: usize,
) -> (MatchResult, DuelTimingStats) {
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    clear_exact_state_analysis_cache();
    clear_turn_engine_plan_cache();
    clear_turn_engine_selector_diagnostics();
    let use_white_opening_book = env_bool("SMART_USE_WHITE_OPENING_BOOK").unwrap_or(false);
    let mut timing = DuelTimingStats::default();

    for _ in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return (match_result_from_winner(winner_color, a_is_white), timing);
        }

        let a_to_move = if a_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };
        let (actor_model, actor_budget) = if a_to_move {
            (model_a, budget_a)
        } else {
            (model_b, budget_b)
        };

        let config = actor_budget.runtime_config_for_game(&game);
        let start = Instant::now();
        let inputs = if use_white_opening_book {
            MonsGameModel::white_first_turn_opening_next_inputs(&game).unwrap_or_else(|| {
                select_inputs_with_runtime_fallback(actor_model.select_inputs, &game, config)
            })
        } else {
            select_inputs_with_runtime_fallback(actor_model.select_inputs, &game, config)
        };
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        if a_to_move {
            timing.record_profile_a_turn(elapsed_ms);
        } else {
            timing.record_profile_b_turn(elapsed_ms);
        }

        if inputs.is_empty() {
            return (
                if a_to_move {
                    MatchResult::ProfileBWin
                } else {
                    MatchResult::ProfileAWin
                },
                timing,
            );
        }

        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return (
                if a_to_move {
                    MatchResult::ProfileBWin
                } else {
                    MatchResult::ProfileAWin
                },
                timing,
            );
        }
    }

    (
        match adjudicate_non_terminal_game(&game) {
            Some(winner_color) => match_result_from_winner(winner_color, a_is_white),
            None => MatchResult::Draw,
        },
        timing,
    )
}

pub(in super::super) fn match_result_from_winner(
    winner_color: Color,
    profile_a_is_white: bool,
) -> MatchResult {
    if (profile_a_is_white && winner_color == Color::White)
        || (!profile_a_is_white && winner_color == Color::Black)
    {
        MatchResult::ProfileAWin
    } else {
        MatchResult::ProfileBWin
    }
}

pub(in super::super) fn adjudicate_non_terminal_game(game: &MonsGame) -> Option<Color> {
    if game.white_score > game.black_score {
        return Some(Color::White);
    }
    if game.black_score > game.white_score {
        return Some(Color::Black);
    }

    let white_eval =
        evaluate_preferability_with_weights(game, Color::White, &DEFAULT_SCORING_WEIGHTS);
    let black_eval =
        evaluate_preferability_with_weights(game, Color::Black, &DEFAULT_SCORING_WEIGHTS);
    let net_eval = white_eval - black_eval;
    if net_eval > 0 {
        Some(Color::White)
    } else if net_eval < 0 {
        Some(Color::Black)
    } else {
        None
    }
}

pub(in super::super) fn generate_opening_fens(seed: u64, count: usize) -> Vec<String> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut fens = Vec::with_capacity(count);

    while fens.len() < count {
        let mut game = MonsGame::new(false);
        let opening_plies = rng.gen_range(0..=OPENING_RANDOM_PLIES_MAX);
        let mut valid = true;

        for _ in 0..opening_plies {
            if game.winner_color().is_some() {
                break;
            }
            if !apply_seeded_random_move(&mut game, &mut rng) {
                valid = false;
                break;
            }
        }

        if valid && game.winner_color().is_none() {
            fens.push(game.fen());
        }
    }

    fens
}

pub(in super::super) fn opening_fens_cache() -> &'static OpeningFensCache {
    static CACHE: OnceLock<OpeningFensCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(in super::super) fn generate_opening_fens_cached(seed: u64, count: usize) -> Arc<Vec<String>> {
    let key = (seed, count);
    {
        let cache_guard = opening_fens_cache()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(cached) = cache_guard.get(&key) {
            return Arc::clone(cached);
        }
    }

    let generated = Arc::new(generate_opening_fens(seed, count));
    let mut cache_guard = opening_fens_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let entry = cache_guard
        .entry(key)
        .or_insert_with(|| Arc::clone(&generated));
    Arc::clone(entry)
}

pub(in super::super) fn apply_seeded_random_move(game: &mut MonsGame, rng: &mut StdRng) -> bool {
    let legal_inputs =
        MonsGameModel::enumerate_legal_inputs(game, 256, SuggestedStartInputOptions::default());
    if legal_inputs.is_empty() {
        return false;
    }
    let random_index = rng.gen_range(0..legal_inputs.len());
    matches!(
        game.process_input(legal_inputs[random_index].clone(), false, false),
        Output::Events(_)
    )
}

pub(in super::super) fn one_sided_binomial_p_value(successes: usize, trials: usize) -> f64 {
    if trials == 0 || successes == 0 {
        return 1.0;
    }
    if successes > trials {
        return 0.0;
    }

    let mut probability_mass = 2.0_f64.powi(-(trials as i32));
    let mut tail_probability = 0.0;
    for k in 0..=trials {
        if k >= successes {
            tail_probability += probability_mass;
        }
        if k < trials {
            probability_mass *= (trials - k) as f64 / (k + 1) as f64;
        }
    }

    tail_probability.min(1.0)
}

pub(in super::super) fn seed_for_pairing(candidate_id: &str, opponent_id: &str) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for byte in candidate_id.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash ^= b':' as u64;
    hash = hash.wrapping_mul(1099511628211);
    for byte in opponent_id.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

pub(in super::super) fn seed_for_budget_repeat_and_tag(
    budget: SearchBudget,
    repeat_index: usize,
    seed_tag: &str,
) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for byte in seed_tag.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash ^= b':' as u64;
    hash = hash.wrapping_mul(1099511628211);
    for byte in budget.key().bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash ^= (budget.depth as u64).wrapping_mul(0x9e3779b97f4a7c15);
    hash = hash.wrapping_mul(1099511628211);
    hash ^= (budget.max_nodes as u64).wrapping_mul(0x517cc1b727220a95);
    hash = hash.wrapping_mul(1099511628211);
    hash ^= (repeat_index as u64).wrapping_mul(0x94d049bb133111eb);
    hash = hash.wrapping_mul(1099511628211);
    hash
}

pub(in super::super) fn seed_for_budget_duel_repeat_and_tag(
    budget_a: SearchBudget,
    budget_b: SearchBudget,
    repeat_index: usize,
    seed_tag: &str,
) -> u64 {
    let mut hash = seed_for_budget_repeat_and_tag(budget_a, repeat_index, seed_tag);
    hash ^= b'|' as u64;
    hash = hash.wrapping_mul(1099511628211);
    for byte in budget_b.key().bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash ^= (budget_b.depth as u64).wrapping_mul(0x9e3779b97f4a7c15);
    hash = hash.wrapping_mul(1099511628211);
    hash ^= (budget_b.max_nodes as u64).wrapping_mul(0x517cc1b727220a95);
    hash = hash.wrapping_mul(1099511628211);
    hash
}

pub(in super::super) fn env_usize(name: &str) -> Option<usize> {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
}

pub(in super::super) fn env_bool(name: &str) -> Option<bool> {
    env::var(name).ok().and_then(|value| {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "1" | "true" | "yes" => Some(true),
            "0" | "false" | "no" => Some(false),
            _ => None,
        }
    })
}

pub(in super::super) fn stats_delta_confidence(stats: MatchupStats) -> (f64, f64) {
    (
        stats.win_rate_points() - 0.5,
        stats.confidence_better_than_even(),
    )
}

pub(in super::super) fn mirrored_profile_a_stats(
    ab: MatchupStats,
    ba: MatchupStats,
) -> MatchupStats {
    MatchupStats {
        wins: ab.wins + ba.losses,
        losses: ab.losses + ba.wins,
        draws: ab.draws + ba.draws,
    }
}

pub(in super::super) fn mirrored_profile_a_timing(
    ab: DuelTimingStats,
    ba: DuelTimingStats,
) -> DuelTimingStats {
    DuelTimingStats {
        profile_a_total_ms: ab.profile_a_total_ms + ba.profile_b_total_ms,
        profile_b_total_ms: ab.profile_b_total_ms + ba.profile_a_total_ms,
        profile_a_turns: ab.profile_a_turns + ba.profile_b_turns,
        profile_b_turns: ab.profile_b_turns + ba.profile_a_turns,
    }
}

pub(in super::super) fn run_cross_budget_duel_with_timing(
    config: CrossBudgetDuelConfig<'_>,
) -> TimedMatchupStats {
    let Some(selector_a) = profile_selector_from_name(config.profile_a) else {
        panic!(
            "unknown profile for cross-budget duel A: {}",
            config.profile_a
        );
    };
    let Some(selector_b) = profile_selector_from_name(config.profile_b) else {
        panic!(
            "unknown profile for cross-budget duel B: {}",
            config.profile_b
        );
    };

    let model_a = AutomoveModel {
        select_inputs: selector_a,
    };
    let model_b = AutomoveModel {
        select_inputs: selector_b,
    };
    let budget_a = SearchBudget::from_preference(config.mode_a);
    let budget_b = SearchBudget::from_preference(config.mode_b);

    let original_max_plies = env::var("SMART_POOL_MAX_PLIES").ok();
    let original_opening_book = env::var("SMART_USE_WHITE_OPENING_BOOK").ok();
    env::set_var("SMART_POOL_MAX_PLIES", config.max_plies.to_string());
    env::set_var(
        "SMART_USE_WHITE_OPENING_BOOK",
        if config.use_white_opening_book {
            "true"
        } else {
            "false"
        },
    );

    let mut aggregate = TimedMatchupStats::default();
    let progress = env_bool("SMART_DUEL_PROGRESS").unwrap_or(false);
    for repeat_index in 0..config.repeats.max(1) {
        let seed =
            seed_for_budget_duel_repeat_and_tag(budget_a, budget_b, repeat_index, config.seed_tag);
        let ab = run_budget_duel_series_with_timing(
            model_a,
            budget_a,
            model_b,
            budget_b,
            config.games_per_repeat.max(1),
            seed,
            config.max_plies,
        );
        let ba = run_budget_duel_series_with_timing(
            model_b,
            budget_b,
            model_a,
            budget_a,
            config.games_per_repeat.max(1),
            seed,
            config.max_plies,
        );
        aggregate
            .matchup
            .merge(mirrored_profile_a_stats(ab.matchup, ba.matchup));
        aggregate
            .timing
            .merge(mirrored_profile_a_timing(ab.timing, ba.timing));
        if progress {
            let (delta, confidence) = stats_delta_confidence(aggregate.matchup);
            println!(
                "cross-budget progress: {}({}) vs {}({}) seed={} repeat={}/{} games={} delta={:.4} confidence={:.3} profile_a_avg_ms={:.2} profile_b_avg_ms={:.2}",
                config.profile_a,
                config.mode_a.as_api_value(),
                config.profile_b,
                config.mode_b.as_api_value(),
                config.seed_tag,
                repeat_index + 1,
                config.repeats.max(1),
                aggregate.matchup.total_games(),
                delta,
                confidence,
                aggregate.timing.profile_a_avg_ms(),
                aggregate.timing.profile_b_avg_ms(),
            );
        }
    }

    if let Some(previous) = original_max_plies {
        env::set_var("SMART_POOL_MAX_PLIES", previous);
    } else {
        env::remove_var("SMART_POOL_MAX_PLIES");
    }
    if let Some(previous) = original_opening_book {
        env::set_var("SMART_USE_WHITE_OPENING_BOOK", previous);
    } else {
        env::remove_var("SMART_USE_WHITE_OPENING_BOOK");
    }

    aggregate
}

pub(in super::super) fn profile_speed_by_mode_ms(
    selector: AutomoveSelector,
    openings: &[String],
    budgets: &[SearchBudget],
) -> Vec<ModeSpeedStat> {
    let mut stats = Vec::with_capacity(budgets.len());
    for budget in budgets.iter().copied() {
        let start = Instant::now();
        for opening in openings {
            let game = MonsGame::from_fen(opening, false).expect("valid opening fen");
            let config = budget.runtime_config_for_game(&game);
            let _ = select_inputs_with_runtime_fallback(selector, &game, config);
        }
        stats.push(ModeSpeedStat {
            budget,
            avg_ms: start.elapsed().as_secs_f64() * 1000.0 / openings.len().max(1) as f64,
        });
    }
    stats
}

pub(in super::super) fn tactical_game_with_items(
    items: Vec<(Location, Item)>,
    active_color: Color,
    turn_number: i32,
) -> MonsGame {
    let mut game = MonsGame::new(false);
    let board_items = items.into_iter().collect::<HashMap<_, _>>();
    game.board = Board::new_with_items(board_items);
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

pub(in super::super) fn assert_tactical_guardrails(selector: AutomoveSelector, profile_name: &str) {
    let drainer_attack_game = tactical_game_with_items(
        vec![
            (
                Location::new(5, 5),
                Item::Mon {
                    mon: Mon::new(MonKind::Mystic, Color::White, 0),
                },
            ),
            (
                Location::new(10, 5),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                },
            ),
            (
                Location::new(7, 7),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                },
            ),
        ],
        Color::White,
        2,
    );
    let drainer_attack_config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
        .runtime_config_for_game(&drainer_attack_game);
    let drainer_attack_inputs =
        select_inputs_with_runtime_fallback(selector, &drainer_attack_game, drainer_attack_config);
    let (_, drainer_attack_events) = MonsGameModel::apply_inputs_for_search_with_events(
        &drainer_attack_game,
        &drainer_attack_inputs,
    )
    .expect("drainer attack move should be legal");
    assert!(
        MonsGameModel::events_include_opponent_drainer_fainted(
            &drainer_attack_events,
            Color::White
        ),
        "profile '{}' must take available same-turn drainer attack",
        profile_name
    );

    let bomb_drainer_attack_game = tactical_game_with_items(
        vec![
            (
                Location::new(10, 0),
                Item::Mon {
                    mon: Mon::new(MonKind::Angel, Color::White, 0),
                },
            ),
            (
                Location::new(10, 5),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                },
            ),
            (
                Location::new(8, 2),
                Item::Consumable {
                    consumable: Consumable::BombOrPotion,
                },
            ),
            (
                Location::new(5, 5),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                },
            ),
        ],
        Color::White,
        2,
    );
    let mut bomb_config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
        .runtime_config_for_game(&bomb_drainer_attack_game);
    bomb_config.root_enum_limit = 0;
    let bomb_inputs =
        select_inputs_with_runtime_fallback(selector, &bomb_drainer_attack_game, bomb_config);
    let (after_bomb_probe, bomb_events) =
        MonsGameModel::apply_inputs_for_search_with_events(&bomb_drainer_attack_game, &bomb_inputs)
            .expect("bomb drainer attack move should be legal");
    let bomb_attacks_now =
        MonsGameModel::events_include_opponent_drainer_fainted(&bomb_events, Color::White);
    let mut bomb_continuation_budget = SMART_FORCED_DRAINER_ATTACK_FALLBACK_NODE_BUDGET_FAST;
    let bomb_attacks_later = after_bomb_probe.active_color == Color::White
        && MonsGameModel::can_attack_opponent_drainer_before_turn_ends(
            &after_bomb_probe,
            Color::White,
            SMART_FORCED_DRAINER_ATTACK_FALLBACK_ENUM_LIMIT_FAST,
            SuggestedStartInputOptions::for_automove(),
            &mut bomb_continuation_budget,
            &mut U64HashSet::default(),
        );
    assert!(
        bomb_attacks_now || bomb_attacks_later,
        "profile '{}' must keep bomb-based drainer attack lines",
        profile_name
    );

    let mut winning_carrier_game = None;
    for location in [
        Location::new(9, 0),
        Location::new(9, 1),
        Location::new(9, 2),
        Location::new(8, 1),
    ] {
        let mut probe = tactical_game_with_items(
            vec![
                (
                    location,
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            3,
        );
        probe.white_score = Config::TARGET_SCORE - 2;
        let has_immediate_win = MonsGameModel::enumerate_legal_inputs(
            &probe,
            96,
            SuggestedStartInputOptions::for_automove(),
        )
        .into_iter()
        .any(|inputs| {
            let mut after = probe.clone_for_simulation();
            matches!(after.process_input(inputs, false, false), Output::Events(_))
                && after.winner_color() == Some(Color::White)
        });
        if has_immediate_win {
            winning_carrier_game = Some(probe);
            break;
        }
    }
    let winning_carrier_game =
        winning_carrier_game.expect("expected at least one immediate-winning carrier setup");
    let winning_config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
        .runtime_config_for_game(&winning_carrier_game);
    let winning_inputs =
        select_inputs_with_runtime_fallback(selector, &winning_carrier_game, winning_config);
    assert!(
        !winning_inputs.is_empty(),
        "profile '{}' should produce a move in immediate-win setup",
        profile_name
    );
    let mut winning_after = winning_carrier_game.clone_for_simulation();
    assert!(matches!(
        winning_after.process_input(winning_inputs, false, false),
        Output::Events(_)
    ));
    assert_eq!(
        winning_after.winner_color(),
        Some(Color::White),
        "profile '{}' should convert immediate winning carrier line",
        profile_name
    );

    let drainer_safety_game = tactical_game_with_items(
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
    );
    let drainer_safety_config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
        .runtime_config_for_game(&drainer_safety_game);
    let drainer_safety_roots =
        MonsGameModel::ranked_root_moves(&drainer_safety_game, Color::White, drainer_safety_config);
    if drainer_safety_roots
        .iter()
        .any(|root| !root.own_drainer_vulnerable)
    {
        let safety_inputs = select_inputs_with_runtime_fallback(
            selector,
            &drainer_safety_game,
            drainer_safety_config,
        );
        let (safety_after, _) = MonsGameModel::apply_inputs_for_search_with_events(
            &drainer_safety_game,
            &safety_inputs,
        )
        .expect("drainer safety move should be legal");
        assert!(
            !MonsGameModel::is_own_drainer_vulnerable_next_turn(&safety_after, Color::White, false),
            "profile '{}' left drainer vulnerable while safe alternatives existed",
            profile_name
        );
    }

    let spirit = Mon::new(MonKind::Spirit, Color::White, 0);
    let spirit_base = Board::new().base(spirit);
    let spirit_base_game = tactical_game_with_items(
        vec![
            (
                spirit_base,
                Item::Mon {
                    mon: Mon::new(MonKind::Spirit, Color::White, 0),
                },
            ),
            (
                Location::new(7, 7),
                Item::Mana {
                    mana: Mana::Regular(Color::White),
                },
            ),
        ],
        Color::White,
        2,
    );
    let spirit_config = SearchBudget::from_preference(SmartAutomovePreference::Normal)
        .runtime_config_for_game(&spirit_base_game);
    let spirit_inputs =
        select_inputs_with_runtime_fallback(selector, &spirit_base_game, spirit_config);
    let (spirit_after, _) =
        MonsGameModel::apply_inputs_for_search_with_events(&spirit_base_game, &spirit_inputs)
            .expect("spirit utility move should be legal");
    let spirit_still_on_base = spirit_after
        .board
        .item(spirit_base)
        .and_then(|item| item.mon())
        .map(|mon| mon.kind == MonKind::Spirit && mon.color == Color::White && !mon.is_fainted())
        .unwrap_or(false);
    assert!(
        !spirit_still_on_base,
        "profile '{}' should not keep awake spirit idle on base in utility setup",
        profile_name
    );

    let selected_move_classes = |game: &MonsGame,
                                 config: AutomoveSearchConfig,
                                 selected: &[Input]|
     -> Option<MoveClassFlags> {
        let selected_fen = Input::fen_from_array(selected);
        let mut class_config = config;
        class_config.enable_move_class_coverage = true;
        MonsGameModel::ranked_root_moves(game, game.active_color, class_config)
            .into_iter()
            .find(|root| Input::fen_from_array(&root.inputs) == selected_fen)
            .map(|root| root.classes)
            .or_else(|| {
                let own_drainer_vulnerable_before =
                    MonsGameModel::is_own_drainer_vulnerable_next_turn(
                        game,
                        game.active_color,
                        class_config.enable_enhanced_drainer_vulnerability,
                    );
                let (after, events) =
                    MonsGameModel::apply_inputs_for_search_with_events(game, selected)?;
                let own_drainer_vulnerable_after =
                    MonsGameModel::is_own_drainer_vulnerable_next_turn(
                        &after,
                        game.active_color,
                        class_config.enable_enhanced_drainer_vulnerability,
                    );
                Some(MonsGameModel::classify_move_classes(
                    game,
                    &after,
                    game.active_color,
                    &events,
                    own_drainer_vulnerable_before,
                    own_drainer_vulnerable_after,
                ))
            })
    };
    let carrier_progress_game = tactical_game_with_items(
        vec![
            (
                Location::new(10, 0),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                },
            ),
            (
                Location::new(10, 1),
                Item::Mon {
                    mon: Mon::new(MonKind::Angel, Color::White, 0),
                },
            ),
            (
                Location::new(9, 1),
                Item::Mon {
                    mon: Mon::new(MonKind::Mystic, Color::White, 0),
                },
            ),
            (
                Location::new(9, 0),
                Item::Mana {
                    mana: Mana::Regular(Color::White),
                },
            ),
        ],
        Color::White,
        2,
    );
    let scenario_pack = vec![
        (
            "drainer_attack",
            drainer_attack_game.clone_for_simulation(),
            MoveClass::DrainerAttack,
        ),
        (
            "immediate_score",
            winning_carrier_game.clone_for_simulation(),
            MoveClass::ImmediateScore,
        ),
        (
            "carrier_progress_probe",
            carrier_progress_game.clone_for_simulation(),
            MoveClass::CarrierProgress,
        ),
    ];
    for (scenario_id, game, expected_class) in scenario_pack {
        let config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
            .runtime_config_for_game(&game);
        let selected = select_inputs_with_runtime_fallback(selector, &game, config);
        let classes = selected_move_classes(&game, config, &selected).unwrap_or_default();
        let selected_fen = Input::fen_from_array(&selected);
        let score_before = MonsGameModel::score_for_color(&game, Color::White);
        let score_after = MonsGameModel::apply_inputs_for_search_with_events(&game, &selected)
            .map(|(after, _)| MonsGameModel::score_for_color(&after, Color::White))
            .unwrap_or(score_before);
        assert!(
            classes.has(expected_class),
            "profile '{}' failed scenario '{}' expected class {:?}, selected={}, classes={:?}, score_before={}, score_after={}",
            profile_name,
            scenario_id,
            expected_class,
            selected_fen,
            classes,
            score_before,
            score_after
        );
    }
}

pub(in super::super) fn assert_interview_policy_regressions(
    selector: AutomoveSelector,
    profile_name: &str,
) {
    let selected_root = |game: &MonsGame,
                         config: AutomoveSearchConfig,
                         selected: &[Input]|
     -> Option<ScoredRootMove> {
        let selected_fen = Input::fen_from_array(selected);
        MonsGameModel::ranked_root_moves(game, game.active_color, config)
            .into_iter()
            .find(|root| Input::fen_from_array(&root.inputs) == selected_fen)
    };

    let supermana_progress_game = tactical_game_with_items(
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
    );
    let supermana_config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
        .runtime_config_for_game(&supermana_progress_game);
    let supermana_roots =
        MonsGameModel::ranked_root_moves(&supermana_progress_game, Color::White, supermana_config);
    let safe_supermana_exists = supermana_roots.iter().any(|root| {
        root.supermana_progress
            && !root.own_drainer_vulnerable
            && !root.wins_immediately
            && !root.attacks_opponent_drainer
    });
    if safe_supermana_exists {
        let selected = select_inputs_with_runtime_fallback(
            selector,
            &supermana_progress_game,
            supermana_config,
        );
        let selected = selected_root(&supermana_progress_game, supermana_config, &selected)
            .expect("selected root should be in ranked list");
        assert!(
            selected.supermana_progress || selected.scores_supermana_this_turn,
            "profile '{}' should prefer safe supermana progress when available",
            profile_name
        );
    }

    let opponent_mana_progress_game = tactical_game_with_items(
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
    );
    let opponent_mana_config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
        .runtime_config_for_game(&opponent_mana_progress_game);
    let opponent_mana_roots = MonsGameModel::ranked_root_moves(
        &opponent_mana_progress_game,
        Color::White,
        opponent_mana_config,
    );
    let safe_opponent_mana_exists = opponent_mana_roots.iter().any(|root| {
        root.opponent_mana_progress
            && !root.own_drainer_vulnerable
            && !root.wins_immediately
            && !root.attacks_opponent_drainer
    });
    if safe_opponent_mana_exists {
        let selected = select_inputs_with_runtime_fallback(
            selector,
            &opponent_mana_progress_game,
            opponent_mana_config,
        );
        let selected = selected_root(
            &opponent_mana_progress_game,
            opponent_mana_config,
            &selected,
        )
        .expect("selected root should be in ranked list");
        assert!(
            selected.opponent_mana_progress || selected.scores_opponent_mana_this_turn,
            "profile '{}' should prefer safe opponent-mana progress when available",
            profile_name
        );
    }
}
