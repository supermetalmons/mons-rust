use super::profiles::profile_selector_from_name;
use super::*;
use crate::models::automove_exact::clear_exact_state_analysis_cache;
use crate::models::automove_turn_engine::clear_turn_engine_plan_cache;
use crate::models::mons_game_model::clear_turn_engine_selector_diagnostics;
use std::collections::HashMap;

type OpeningFensCacheKey = (u64, usize);
type OpeningFens = Arc<Vec<String>>;
type OpeningFensCacheMap = HashMap<OpeningFensCacheKey, OpeningFens>;
type OpeningFensCache = Mutex<OpeningFensCacheMap>;

#[derive(Clone, Copy)]
pub(super) struct CrossBudgetDuelConfig<'a> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TriageSurface {
    OpeningReply,
    PrimaryPro,
    ReplyRisk,
    Supermana,
    OpponentMana,
}

impl TriageSurface {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "opening_reply" => Some(Self::OpeningReply),
            "primary_pro" => Some(Self::PrimaryPro),
            "reply_risk" => Some(Self::ReplyRisk),
            "supermana" => Some(Self::Supermana),
            "opponent_mana" => Some(Self::OpponentMana),
            _ => None,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::OpeningReply => "opening_reply",
            Self::PrimaryPro => "primary_pro",
            Self::ReplyRisk => "reply_risk",
            Self::Supermana => "supermana",
            Self::OpponentMana => "opponent_mana",
        }
    }
}

#[derive(Clone)]
pub(super) struct TriageFixture {
    pub id: &'static str,
    pub game: MonsGame,
    pub mode: SmartAutomovePreference,
    pub opening_book_driven: bool,
    pub config_tweak: Option<fn(SmartSearchConfig) -> SmartSearchConfig>,
    pub expected_selected_input_fen: Option<&'static str>,
}

pub(super) fn select_inputs_with_runtime_fallback(
    selector: AutomoveSelector,
    game: &MonsGame,
    config: SmartSearchConfig,
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

pub(super) fn evaluate_candidate_against_pool_with_max_plies(
    candidate: AutomoveModel,
    pool: &[AutomoveModel],
    games_per_matchup: usize,
    budgets: &[SearchBudget],
    max_plies: usize,
) -> CandidateEvaluation {
    assert!(!budgets.is_empty());
    assert!(!pool.is_empty());

    let mut combined_by_opponent = HashMap::<&'static str, MatchupStats>::new();

    for budget in budgets.iter().copied() {
        let mode_result = run_mode_evaluation_with_max_plies(
            candidate,
            pool,
            games_per_matchup,
            budget,
            max_plies,
        );
        for entry in &mode_result.opponents {
            combined_by_opponent
                .entry(entry.opponent_id)
                .or_default()
                .merge(entry.stats);
        }
    }

    let mut opponents = combined_by_opponent
        .into_iter()
        .map(|(opponent_id, stats)| OpponentEvaluation { opponent_id, stats })
        .collect::<Vec<_>>();
    opponents.sort_by(|a, b| a.opponent_id.cmp(b.opponent_id));

    CandidateEvaluation {
        games_per_matchup,
        opponents,
    }
}

pub(super) fn run_mode_evaluation_with_max_plies(
    candidate: AutomoveModel,
    pool: &[AutomoveModel],
    games_per_matchup: usize,
    budget: SearchBudget,
    max_plies: usize,
) -> ModeEvaluation {
    let mut opponents = Vec::with_capacity(pool.len());

    for opponent in pool.iter().copied() {
        let stats = run_matchup_series_with_max_plies(
            candidate,
            opponent,
            games_per_matchup,
            budget,
            seed_for_pairing_and_budget(candidate.id, opponent.id, budget),
            max_plies,
        );
        opponents.push(OpponentEvaluation {
            opponent_id: opponent.id,
            stats,
        });
    }
    opponents.sort_by(|a, b| a.opponent_id.cmp(b.opponent_id));

    ModeEvaluation {
        opponents,
    }
}

pub(super) fn run_matchup_series_with_max_plies(
    candidate: AutomoveModel,
    opponent: AutomoveModel,
    games_per_matchup: usize,
    budget: SearchBudget,
    seed: u64,
    max_plies: usize,
) -> MatchupStats {
    let opening_fens = generate_opening_fens_cached(seed, games_per_matchup.max(1));
    let mut stats = MatchupStats::default();
    for game_index in 0..games_per_matchup.max(1) {
        let candidate_is_white = game_index % 2 == 0;
        let result = play_one_game(
            candidate,
            opponent,
            budget,
            candidate_is_white,
            opening_fens[game_index].as_str(),
            max_plies,
        );
        stats.record(result);
    }
    stats
}

pub(super) fn play_one_game(
    candidate: AutomoveModel,
    opponent: AutomoveModel,
    budget: SearchBudget,
    candidate_is_white: bool,
    opening_fen: &str,
    max_plies: usize,
) -> MatchResult {
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    clear_exact_state_analysis_cache();
    clear_turn_opportunity_plan_cache();
    clear_turn_engine_plan_cache();
    clear_turn_engine_selector_diagnostics();
    let use_white_opening_book = env_bool("SMART_USE_WHITE_OPENING_BOOK").unwrap_or(false);

    for _ in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return match_result_from_winner(winner_color, candidate_is_white);
        }

        let candidate_to_move = if candidate_is_white {
            game.active_color == Color::White
        } else {
            game.active_color == Color::Black
        };
        let actor_model = if candidate_to_move {
            candidate
        } else {
            opponent
        };

        let config = budget.runtime_config_for_game(&game);
        let inputs = if use_white_opening_book {
            MonsGameModel::white_first_turn_opening_next_inputs(&game).unwrap_or_else(|| {
                select_inputs_with_runtime_fallback(actor_model.select_inputs, &game, config)
            })
        } else {
            select_inputs_with_runtime_fallback(actor_model.select_inputs, &game, config)
        };
        if inputs.is_empty() {
            return if candidate_to_move {
                MatchResult::OpponentWin
            } else {
                MatchResult::CandidateWin
            };
        }

        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return if candidate_to_move {
                MatchResult::OpponentWin
            } else {
                MatchResult::CandidateWin
            };
        }
    }

    match adjudicate_non_terminal_game(&game) {
        Some(winner_color) => match_result_from_winner(winner_color, candidate_is_white),
        None => MatchResult::Draw,
    }
}

pub(super) fn run_budget_duel_series_with_timing(
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

pub(super) fn play_one_game_budget_duel_with_timing(
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
    clear_turn_opportunity_plan_cache();
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
            timing.record_candidate_turn(elapsed_ms);
        } else {
            timing.record_baseline_turn(elapsed_ms);
        }

        if inputs.is_empty() {
            return (
                if a_to_move {
                    MatchResult::OpponentWin
                } else {
                    MatchResult::CandidateWin
                },
                timing,
            );
        }

        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return (
                if a_to_move {
                    MatchResult::OpponentWin
                } else {
                    MatchResult::CandidateWin
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

pub(super) fn match_result_from_winner(
    winner_color: Color,
    candidate_is_white: bool,
) -> MatchResult {
    if (candidate_is_white && winner_color == Color::White)
        || (!candidate_is_white && winner_color == Color::Black)
    {
        MatchResult::CandidateWin
    } else {
        MatchResult::OpponentWin
    }
}

pub(super) fn adjudicate_non_terminal_game(game: &MonsGame) -> Option<Color> {
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

pub(super) fn generate_opening_fens(seed: u64, count: usize) -> Vec<String> {
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

pub(super) fn opening_fens_cache() -> &'static OpeningFensCache {
    static CACHE: OnceLock<OpeningFensCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn generate_opening_fens_cached(seed: u64, count: usize) -> Arc<Vec<String>> {
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

pub(super) fn apply_seeded_random_move(game: &mut MonsGame, rng: &mut StdRng) -> bool {
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

pub(super) fn one_sided_binomial_p_value(successes: usize, trials: usize) -> f64 {
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

pub(super) fn seed_for_pairing(candidate_id: &str, opponent_id: &str) -> u64 {
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

pub(super) fn seed_for_pairing_and_budget(
    candidate_id: &str,
    opponent_id: &str,
    budget: SearchBudget,
) -> u64 {
    let mut hash = seed_for_pairing(candidate_id, opponent_id);
    for byte in budget.key().bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash ^= (budget.depth as u64).wrapping_mul(0x9e3779b97f4a7c15);
    hash = hash.wrapping_mul(1099511628211);
    hash ^= (budget.max_nodes as u64).wrapping_mul(0x517cc1b727220a95);
    hash = hash.wrapping_mul(1099511628211);
    hash
}

pub(super) fn seed_for_budget_repeat_and_tag(
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

pub(super) fn seed_for_budget_duel_repeat_and_tag(
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

pub(super) fn env_usize(name: &str) -> Option<usize> {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
}

pub(super) fn env_bool(name: &str) -> Option<bool> {
    env::var(name).ok().and_then(|value| {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "1" | "true" | "yes" => Some(true),
            "0" | "false" | "no" => Some(false),
            _ => None,
        }
    })
}

pub(super) fn stats_delta_confidence(stats: MatchupStats) -> (f64, f64) {
    (
        stats.win_rate_points() - 0.5,
        stats.confidence_better_than_even(),
    )
}

pub(super) fn mirrored_candidate_stats(ab: MatchupStats, ba: MatchupStats) -> MatchupStats {
    MatchupStats {
        wins: ab.wins + ba.losses,
        losses: ab.losses + ba.wins,
        draws: ab.draws + ba.draws,
    }
}

pub(super) fn mirrored_candidate_timing(
    ab: DuelTimingStats,
    ba: DuelTimingStats,
) -> DuelTimingStats {
    DuelTimingStats {
        candidate_total_ms: ab.candidate_total_ms + ba.baseline_total_ms,
        baseline_total_ms: ab.baseline_total_ms + ba.candidate_total_ms,
        candidate_turns: ab.candidate_turns + ba.baseline_turns,
        baseline_turns: ab.baseline_turns + ba.candidate_turns,
    }
}


pub(super) fn run_cross_budget_duel_with_timing(
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
        id: "cross_budget_a",
        select_inputs: selector_a,
    };
    let model_b = AutomoveModel {
        id: "cross_budget_b",
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
        aggregate.matchup.merge(mirrored_candidate_stats(ab.matchup, ba.matchup));
        aggregate
            .timing
            .merge(mirrored_candidate_timing(ab.timing, ba.timing));
        if progress {
            let (delta, confidence) = stats_delta_confidence(aggregate.matchup);
            println!(
                "cross-budget progress: {}({}) vs {}({}) seed={} repeat={}/{} games={} delta={:.4} confidence={:.3} candidate_avg_ms={:.2} baseline_avg_ms={:.2}",
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
                aggregate.timing.candidate_avg_ms(),
                aggregate.timing.baseline_avg_ms(),
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

pub(super) fn profile_speed_by_mode_ms(
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

pub(super) fn tactical_game_with_items(
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

pub(super) fn assert_tactical_guardrails(selector: AutomoveSelector, profile_name: &str) {
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
                                 config: SmartSearchConfig,
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

pub(super) fn assert_interview_policy_regressions(selector: AutomoveSelector, profile_name: &str) {
    let selected_root = |game: &MonsGame,
                         config: SmartSearchConfig,
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

fn pvs_sensitive_search_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 1 0 0 0 0 4 n05d0xa0xn04/n02xxmn01s0xn03e0xn02/n02y0xn08/n06xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMxxMxxMn01xxMn03/n11/n01E0xn03D0xxxMS0xn03/n04A0xn01Y0xn04/n11",
        false,
    )
    .expect("pvs_sensitive_search_triage_game: valid fen")
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
                let selector = profile_selector_from_name("runtime_current")
                    .unwrap_or_else(|| panic!("{}: runtime_current selector available", label));
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
    let selector = profile_selector_from_name("runtime_current")
        .unwrap_or_else(|| panic!("{}: runtime_current selector available", label));
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

fn black_reliability_opening_3_ply4_triage_game() -> MonsGame {
    MonsGame::from_fen(
        "0 0 b 0 0 1 0 0 2 n03y0xn01d0xa0xe0xn03/n03s0xn07/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n03E0xA0xD0xS0xn01Y0xn02/n11",
        false,
    )
    .expect("black_reliability_opening_3_ply4: valid fen")
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

fn spirit_setup_triage_game() -> MonsGame {
    tactical_game_with_items(
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
        2,
    )
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
    let mut game = MonsGame::new(false);
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

pub(super) fn opening_reply_triage_fixtures() -> Vec<TriageFixture> {
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

pub(super) fn primary_pro_triage_fixtures() -> Vec<TriageFixture> {
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
            id: "primary_spirit_setup",
            game: spirit_setup_triage_game(),
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
            id: "primary_pvs_sensitive_search",
            game: pvs_sensitive_search_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: Some("l0,5;l1,5"),
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
            id: "primary_black_reliability_opening_3_ply4",
            game: black_reliability_opening_3_ply4_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
            config_tweak: None,
            expected_selected_input_fen: None,
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
