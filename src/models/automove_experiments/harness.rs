use super::profiles::{profile_selector_from_name, selected_pool_models};
use super::*;
use std::collections::HashMap;

type OpeningFensCacheKey = (u64, usize);
type OpeningFens = Arc<Vec<String>>;
type OpeningFensCacheMap = HashMap<OpeningFensCacheKey, OpeningFens>;
type OpeningFensCache = Mutex<OpeningFensCacheMap>;

#[derive(Clone, Copy)]
pub(super) struct ProMatchupAcrossSeedsConfig<'a> {
    pub candidate_profile: &'a str,
    pub baseline_profile: &'a str,
    pub baseline_mode: SmartAutomovePreference,
    pub seed_tag_prefix: &'a str,
    pub seed_tags: &'a [&'a str],
    pub repeats: usize,
    pub games_per_seed: usize,
    pub max_plies: usize,
    pub use_white_opening_book: bool,
}

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

#[derive(Clone, Copy)]
pub(super) struct MirroredDuelSeedConfig<'a> {
    pub candidate: AutomoveModel,
    pub baseline: AutomoveModel,
    pub budgets: &'a [SearchBudget],
    pub seed_tag: &'a str,
    pub repeats: usize,
    pub games_per_mode: usize,
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
    SpiritSetup,
    DrainerSafety,
    CacheReuse,
}

impl TriageSurface {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "opening_reply" => Some(Self::OpeningReply),
            "primary_pro" => Some(Self::PrimaryPro),
            "reply_risk" => Some(Self::ReplyRisk),
            "supermana" => Some(Self::Supermana),
            "opponent_mana" => Some(Self::OpponentMana),
            "spirit_setup" => Some(Self::SpiritSetup),
            "drainer_safety" => Some(Self::DrainerSafety),
            "cache_reuse" => Some(Self::CacheReuse),
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
            Self::SpiritSetup => "spirit_setup",
            Self::DrainerSafety => "drainer_safety",
            Self::CacheReuse => "cache_reuse",
        }
    }
}

#[derive(Clone)]
pub(super) struct TriageFixture {
    pub id: &'static str,
    pub game: MonsGame,
    pub mode: SmartAutomovePreference,
    pub opening_book_driven: bool,
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

pub(super) fn evaluate_candidate_against_pool(
    candidate: AutomoveModel,
    pool: &[AutomoveModel],
    games_per_matchup: usize,
    budgets: &[SearchBudget],
) -> CandidateEvaluation {
    evaluate_candidate_against_pool_with_max_plies(
        candidate,
        pool,
        games_per_matchup,
        budgets,
        env_usize("SMART_POOL_MAX_PLIES").unwrap_or(MAX_GAME_PLIES),
    )
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

    let min_required_beaten = MIN_OPPONENTS_BEAT_TO_PROMOTE.min(pool.len());
    let mut mode_results = Vec::with_capacity(budgets.len());
    let mut combined_by_opponent = HashMap::<&'static str, MatchupStats>::new();
    let mut aggregate_stats = MatchupStats::default();

    for budget in budgets.iter().copied() {
        let mode_result = run_mode_evaluation_with_max_plies(
            candidate,
            pool,
            games_per_matchup,
            budget,
            max_plies,
        );
        aggregate_stats.merge(mode_result.aggregate_stats);
        for entry in &mode_result.opponents {
            combined_by_opponent
                .entry(entry.opponent_id)
                .or_default()
                .merge(entry.stats);
        }
        mode_results.push(mode_result);
    }

    let mut opponents = combined_by_opponent
        .into_iter()
        .map(|(opponent_id, stats)| OpponentEvaluation { opponent_id, stats })
        .collect::<Vec<_>>();
    opponents.sort_by(|a, b| a.opponent_id.cmp(b.opponent_id));

    let beaten_opponents = opponents
        .iter()
        .filter(|entry| {
            entry.stats.win_rate_points() > 0.5
                && entry.stats.confidence_better_than_even() >= MIN_CONFIDENCE_TO_PROMOTE
        })
        .count()
        .min(min_required_beaten.max(1));

    CandidateEvaluation {
        games_per_matchup,
        beaten_opponents,
        aggregate_stats,
        opponents,
        mode_results,
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
    let mut aggregate_stats = MatchupStats::default();

    for opponent in pool.iter().copied() {
        let stats = run_matchup_series_with_max_plies(
            candidate,
            opponent,
            games_per_matchup,
            budget,
            seed_for_pairing_and_budget(candidate.id, opponent.id, budget),
            max_plies,
        );
        aggregate_stats.merge(stats);
        opponents.push(OpponentEvaluation {
            opponent_id: opponent.id,
            stats,
        });
    }
    opponents.sort_by(|a, b| a.opponent_id.cmp(b.opponent_id));

    ModeEvaluation {
        budget,
        aggregate_stats,
        opponents,
    }
}

pub(super) fn run_matchup_series(
    candidate: AutomoveModel,
    opponent: AutomoveModel,
    games_per_matchup: usize,
    budget: SearchBudget,
    seed: u64,
) -> MatchupStats {
    run_matchup_series_with_max_plies(
        candidate,
        opponent,
        games_per_matchup,
        budget,
        seed,
        env_usize("SMART_POOL_MAX_PLIES").unwrap_or(MAX_GAME_PLIES),
    )
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

pub(super) fn run_budget_duel_series(
    model_a: AutomoveModel,
    budget_a: SearchBudget,
    model_b: AutomoveModel,
    budget_b: SearchBudget,
    games: usize,
    seed: u64,
    max_plies: usize,
) -> MatchupStats {
    let opening_fens = generate_opening_fens_cached(seed, games.max(1));
    let mut stats = MatchupStats::default();
    for game_index in 0..games.max(1) {
        let a_is_white = game_index % 2 == 0;
        let result = play_one_game_budget_duel(
            model_a,
            budget_a,
            model_b,
            budget_b,
            a_is_white,
            opening_fens[game_index].as_str(),
            max_plies,
        );
        stats.record(result);
    }
    stats
}

pub(super) fn play_one_game_budget_duel(
    model_a: AutomoveModel,
    budget_a: SearchBudget,
    model_b: AutomoveModel,
    budget_b: SearchBudget,
    a_is_white: bool,
    opening_fen: &str,
    max_plies: usize,
) -> MatchResult {
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");
    let use_white_opening_book = env_bool("SMART_USE_WHITE_OPENING_BOOK").unwrap_or(false);

    for _ in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return match_result_from_winner(winner_color, a_is_white);
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
        let inputs = if use_white_opening_book {
            MonsGameModel::white_first_turn_opening_next_inputs(&game).unwrap_or_else(|| {
                select_inputs_with_runtime_fallback(actor_model.select_inputs, &game, config)
            })
        } else {
            select_inputs_with_runtime_fallback(actor_model.select_inputs, &game, config)
        };
        if inputs.is_empty() {
            return if a_to_move {
                MatchResult::OpponentWin
            } else {
                MatchResult::CandidateWin
            };
        }

        if !matches!(game.process_input(inputs, false, false), Output::Events(_)) {
            return if a_to_move {
                MatchResult::OpponentWin
            } else {
                MatchResult::CandidateWin
            };
        }
    }

    match adjudicate_non_terminal_game(&game) {
        Some(winner_color) => match_result_from_winner(winner_color, a_is_white),
        None => MatchResult::Draw,
    }
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

pub(super) fn run_pro_matchup_across_seeds(
    config: ProMatchupAcrossSeedsConfig<'_>,
) -> MatchupStats {
    let mut aggregate = MatchupStats::default();
    for seed_tag in config.seed_tags {
        let tagged = format!("{}:{}", config.seed_tag_prefix, seed_tag);
        aggregate.merge(run_cross_budget_duel(CrossBudgetDuelConfig {
            profile_a: config.candidate_profile,
            mode_a: SmartAutomovePreference::Pro,
            profile_b: config.baseline_profile,
            mode_b: config.baseline_mode,
            seed_tag: tagged.as_str(),
            repeats: config.repeats,
            games_per_repeat: config.games_per_seed,
            max_plies: config.max_plies,
            use_white_opening_book: config.use_white_opening_book,
        }));
    }
    aggregate
}

pub(super) fn run_pro_progressive_matchup(
    candidate_profile: &str,
    baseline_profile: &str,
    baseline_mode: SmartAutomovePreference,
    stage_tag: &str,
) -> (MatchupStats, bool) {
    let initial_games = env_usize("SMART_PRO_PROGRESSIVE_INITIAL_GAMES")
        .unwrap_or(2)
        .max(1);
    let max_games = env_usize("SMART_PRO_PROGRESSIVE_MAX_GAMES")
        .unwrap_or(32)
        .max(initial_games);
    let repeats = env_usize("SMART_PRO_PROGRESSIVE_REPEATS")
        .unwrap_or(2)
        .max(1);
    let max_plies = env_usize("SMART_PRO_PROGRESSIVE_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let seed_tags = ["neutral_v1", "neutral_v2", "neutral_v3"];

    let mut cumulative = MatchupStats::default();
    let mut games_per_seed = initial_games;
    let mut meaningful_lift = false;

    loop {
        for seed_tag in seed_tags {
            let tagged_seed = format!("{}:{}:{}", stage_tag, seed_tag, games_per_seed);
            cumulative.merge(run_cross_budget_duel(CrossBudgetDuelConfig {
                profile_a: candidate_profile,
                mode_a: SmartAutomovePreference::Pro,
                profile_b: baseline_profile,
                mode_b: baseline_mode,
                seed_tag: tagged_seed.as_str(),
                repeats,
                games_per_repeat: games_per_seed,
                max_plies,
                use_white_opening_book: false,
            }));
        }

        let (delta, confidence) = stats_delta_confidence(cumulative);
        println!(
            "pro progressive {} vs {}({}): games/seed={} cumulative={} delta={:.4} confidence={:.3}",
            candidate_profile,
            baseline_profile,
            baseline_mode.as_api_value(),
            games_per_seed,
            cumulative.total_games(),
            delta,
            confidence
        );

        if delta >= SMART_PRO_PROGRESSIVE_MEANINGFUL_DELTA_MIN
            && confidence >= SMART_PRO_PROGRESSIVE_MEANINGFUL_CONFIDENCE_MIN
        {
            meaningful_lift = true;
        }
        if delta < -0.05 || games_per_seed >= max_games {
            break;
        }
        games_per_seed = (games_per_seed * 2).min(max_games);
    }

    (cumulative, meaningful_lift)
}

pub(super) fn mirrored_candidate_stats(ab: MatchupStats, ba: MatchupStats) -> MatchupStats {
    MatchupStats {
        wins: ab.wins + ba.losses,
        losses: ab.losses + ba.wins,
        draws: ab.draws + ba.draws,
    }
}

pub(super) fn run_cross_budget_duel(config: CrossBudgetDuelConfig<'_>) -> MatchupStats {
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

    let mut aggregate = MatchupStats::default();
    for repeat_index in 0..config.repeats.max(1) {
        let seed =
            seed_for_budget_duel_repeat_and_tag(budget_a, budget_b, repeat_index, config.seed_tag);
        let ab = run_budget_duel_series(
            model_a,
            budget_a,
            model_b,
            budget_b,
            config.games_per_repeat.max(1),
            seed,
            config.max_plies,
        );
        let ba = run_budget_duel_series(
            model_b,
            budget_b,
            model_a,
            budget_a,
            config.games_per_repeat.max(1),
            seed,
            config.max_plies,
        );
        aggregate.merge(mirrored_candidate_stats(ab, ba));
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

pub(super) fn run_profile_vs_pool_cross_budget(
    profile_name: &str,
    profile_mode: SmartAutomovePreference,
    opponent_mode: SmartAutomovePreference,
    games_per_opponent: usize,
    max_plies: usize,
    seed_tag: &str,
) -> MatchupStats {
    let Some(selector) = profile_selector_from_name(profile_name) else {
        panic!(
            "unknown profile for pool cross-budget duel: {}",
            profile_name
        );
    };
    let profile_model = AutomoveModel {
        id: "pool_candidate",
        select_inputs: selector,
    };
    let profile_budget = SearchBudget::from_preference(profile_mode);
    let opponent_budget = SearchBudget::from_preference(opponent_mode);
    let opponents = selected_pool_models();

    let original_max_plies = env::var("SMART_POOL_MAX_PLIES").ok();
    let original_opening_book = env::var("SMART_USE_WHITE_OPENING_BOOK").ok();
    env::set_var("SMART_POOL_MAX_PLIES", max_plies.to_string());
    env::set_var("SMART_USE_WHITE_OPENING_BOOK", "false");

    let mut aggregate = MatchupStats::default();
    for (opponent_index, opponent) in opponents.iter().copied().enumerate() {
        let duel_tag = format!("{}:pool_{}", seed_tag, opponent_index);
        let seed = seed_for_budget_duel_repeat_and_tag(
            profile_budget,
            opponent_budget,
            opponent_index,
            duel_tag.as_str(),
        );
        let ab = run_budget_duel_series(
            profile_model,
            profile_budget,
            opponent,
            opponent_budget,
            games_per_opponent.max(1),
            seed,
            max_plies,
        );
        let ba = run_budget_duel_series(
            opponent,
            opponent_budget,
            profile_model,
            profile_budget,
            games_per_opponent.max(1),
            seed,
            max_plies,
        );
        aggregate.merge(mirrored_candidate_stats(ab, ba));
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

pub(super) fn run_pool_non_regression_check(
    candidate: AutomoveModel,
    baseline: AutomoveModel,
    budgets: &[SearchBudget],
    games_per_matchup: usize,
) -> (CandidateEvaluation, CandidateEvaluation, f64, f64) {
    let pool = selected_pool_models();
    let candidate_eval =
        evaluate_candidate_against_pool(candidate, &pool, games_per_matchup, budgets);
    let baseline_eval =
        evaluate_candidate_against_pool(baseline, &pool, games_per_matchup, budgets);
    let candidate_wr = candidate_eval.aggregate_stats.win_rate_points();
    let baseline_wr = baseline_eval.aggregate_stats.win_rate_points();
    (candidate_eval, baseline_eval, candidate_wr, baseline_wr)
}

pub(super) fn run_budget_conversion_diagnostic(
    profile_name: &str,
    selector: AutomoveSelector,
    games_per_repeat: usize,
    repeats: usize,
    max_plies: usize,
    seed_tag: &str,
) -> BudgetConversionDiagnostic {
    let fast_budget = SearchBudget::from_preference(SmartAutomovePreference::Fast);
    let normal_budget = SearchBudget::from_preference(SmartAutomovePreference::Normal);
    let model_fast = AutomoveModel {
        id: "budget_conversion_fast",
        select_inputs: selector,
    };
    let model_normal = AutomoveModel {
        id: "budget_conversion_normal",
        select_inputs: selector,
    };

    let mut aggregate = MatchupStats::default();
    for repeat_index in 0..repeats.max(1) {
        let seed = seed_for_budget_duel_repeat_and_tag(
            fast_budget,
            normal_budget,
            repeat_index,
            format!("{}:{}", seed_tag, profile_name).as_str(),
        );
        aggregate.merge(run_budget_duel_series(
            model_fast,
            fast_budget,
            model_normal,
            normal_budget,
            games_per_repeat.max(1),
            seed,
            max_plies,
        ));
    }

    let fast_win_rate = aggregate.win_rate_points();
    BudgetConversionDiagnostic {
        fast_win_rate,
        normal_edge: 0.5 - fast_win_rate,
    }
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

pub(super) fn run_mirrored_duel_for_seed_tag(
    config: MirroredDuelSeedConfig<'_>,
) -> Vec<(SearchBudget, MatchupStats)> {
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

    let mut results = Vec::with_capacity(config.budgets.len());
    for budget in config.budgets.iter().copied() {
        let mut aggregate = MatchupStats::default();
        for repeat_index in 0..config.repeats.max(1) {
            let seed = seed_for_budget_repeat_and_tag(budget, repeat_index, config.seed_tag);
            let ab = run_matchup_series(
                config.candidate,
                config.baseline,
                config.games_per_mode.max(1),
                budget,
                seed,
            );
            let ba = run_matchup_series(
                config.baseline,
                config.candidate,
                config.games_per_mode.max(1),
                budget,
                seed,
            );
            aggregate.merge(mirrored_candidate_stats(ab, ba));
        }
        results.push((budget, aggregate));
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

    results
}

pub(super) fn merge_mode_stats(
    target: &mut HashMap<&'static str, MatchupStats>,
    updates: &[(SearchBudget, MatchupStats)],
) {
    for (budget, stats) in updates {
        target.entry(budget.key()).or_default().merge(*stats);
    }
}

pub(super) fn max_achievable_delta(current: MatchupStats, remaining_games: usize) -> f64 {
    let total_games = current.total_games() + remaining_games;
    if total_games == 0 {
        return 0.0;
    }
    let max_wins = current.wins + remaining_games;
    let best_case_win_rate = (max_wins as f64 + 0.5 * current.draws as f64) / total_games as f64;
    best_case_win_rate - 0.5
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProgressiveStopReason {
    EarlyReject,
    MathematicalReject,
    EarlyPromote,
    MaxGamesReached,
}

#[derive(Debug, Clone)]
pub(super) struct ProgressiveTierResult {
    pub(super) tier: usize,
    pub(super) cumulative_games: usize,
    pub(super) mode_stats: HashMap<&'static str, MatchupStats>,
    pub(super) aggregate_delta: f64,
    pub(super) aggregate_confidence: f64,
    pub(super) stop_reason: Option<ProgressiveStopReason>,
}

#[derive(Debug, Clone)]
pub(super) struct ProgressiveDuelResult {
    pub(super) tiers: Vec<ProgressiveTierResult>,
    pub(super) final_mode_stats: HashMap<&'static str, MatchupStats>,
    pub(super) final_delta: f64,
    pub(super) final_confidence: f64,
    pub(super) total_games: usize,
    pub(super) stop_reason: ProgressiveStopReason,
}

pub(super) struct ProgressiveDuelConfig {
    pub(super) initial_games: usize,
    pub(super) max_games_per_seed: usize,
    pub(super) growth_factor: usize,
    pub(super) seed_tags: Vec<&'static str>,
    pub(super) max_plies: usize,
    pub(super) early_exit_delta_floor: f64,
    pub(super) first_tier_signal_games_per_seed: Option<usize>,
    pub(super) first_tier_signal_aggregate_delta_min: f64,
    pub(super) first_tier_signal_mode_delta_min: f64,
    pub(super) first_tier_signal_mode_floor: f64,
    pub(super) mode_improvement_delta: HashMap<&'static str, f64>,
    pub(super) mode_improvement_confidence: HashMap<&'static str, f64>,
    pub(super) mode_non_regression_delta: HashMap<&'static str, f64>,
    pub(super) repeats_per_game: usize,
    pub(super) use_white_opening_book: bool,
}

impl Default for ProgressiveDuelConfig {
    fn default() -> Self {
        let mut mode_improvement_delta = HashMap::new();
        mode_improvement_delta.insert("fast", SMART_REDUCED_IMPROVEMENT_DELTA_MIN_FAST);
        mode_improvement_delta.insert("normal", SMART_REDUCED_IMPROVEMENT_DELTA_MIN_NORMAL);

        let mut mode_improvement_confidence = HashMap::new();
        mode_improvement_confidence.insert("fast", SMART_REDUCED_IMPROVEMENT_CONFIDENCE_MIN);
        mode_improvement_confidence.insert("normal", SMART_REDUCED_IMPROVEMENT_CONFIDENCE_MIN);

        let mut mode_non_regression_delta = HashMap::new();
        mode_non_regression_delta.insert("fast", SMART_REDUCED_NON_REGRESSION_DELTA_MIN);
        mode_non_regression_delta.insert("normal", SMART_REDUCED_NON_REGRESSION_DELTA_MIN);

        Self {
            initial_games: 2,
            max_games_per_seed: 32,
            growth_factor: 2,
            seed_tags: vec!["neutral_v1", "neutral_v2", "neutral_v3"],
            max_plies: 80,
            early_exit_delta_floor: -0.08,
            first_tier_signal_games_per_seed: None,
            first_tier_signal_aggregate_delta_min: 0.0,
            first_tier_signal_mode_delta_min: 0.0,
            first_tier_signal_mode_floor: -1.0,
            mode_improvement_delta,
            mode_improvement_confidence,
            mode_non_regression_delta,
            repeats_per_game: 2,
            use_white_opening_book: false,
        }
    }
}

impl ProgressiveDuelConfig {
    pub(super) fn from_env_with_defaults(stage: &str) -> Self {
        let mut config = Self::default();
        let prefix = format!("SMART_PROGRESSIVE_{}", stage.to_uppercase());
        if let Some(value) = env_usize(&format!("{}_INITIAL_GAMES", prefix)) {
            config.initial_games = value.max(1);
        }
        if let Some(value) = env_usize(&format!("{}_MAX_GAMES", prefix)) {
            config.max_games_per_seed = value.max(config.initial_games);
        }
        if let Some(value) = env_usize(&format!("{}_REPEATS", prefix)) {
            config.repeats_per_game = value.max(1);
        }
        if let Some(value) = env_usize(&format!("{}_MAX_PLIES", prefix)) {
            config.max_plies = value.max(32);
        }
        config
    }
}

pub(super) fn run_progressive_duel(
    candidate: AutomoveModel,
    baseline: AutomoveModel,
    budgets: &[SearchBudget],
    config: &ProgressiveDuelConfig,
    artifact_path: Option<&str>,
) -> ProgressiveDuelResult {
    let mut cumulative_mode_stats = HashMap::<&'static str, MatchupStats>::new();
    let mut tiers = Vec::new();
    let mut games_per_seed = config.initial_games;
    let mut tier_index = 0usize;

    loop {
        for seed_tag in &config.seed_tags {
            let tier_results = run_mirrored_duel_for_seed_tag(MirroredDuelSeedConfig {
                candidate,
                baseline,
                budgets,
                seed_tag,
                repeats: config.repeats_per_game,
                games_per_mode: games_per_seed,
                max_plies: config.max_plies,
                use_white_opening_book: config.use_white_opening_book,
            });
            merge_mode_stats(&mut cumulative_mode_stats, tier_results.as_slice());
        }

        let mut aggregate = MatchupStats::default();
        for budget in budgets {
            aggregate.merge(
                cumulative_mode_stats
                    .get(budget.key())
                    .copied()
                    .unwrap_or_default(),
            );
        }
        let total_games = aggregate.total_games();
        let aggregate_delta = aggregate.win_rate_points() - 0.5;
        let aggregate_confidence = aggregate.confidence_better_than_even();
        let stop_reason = evaluate_progressive_stop(
            budgets,
            &cumulative_mode_stats,
            aggregate_delta,
            games_per_seed,
            config,
        );

        println!(
            "progressive tier {} | games/seed={} | total={} | δ={:.4} | conf={:.3}",
            tier_index, games_per_seed, total_games, aggregate_delta, aggregate_confidence
        );
        for budget in budgets {
            if let Some(mode_stats) = cumulative_mode_stats.get(budget.key()) {
                println!(
                    "  mode {} | {}W-{}L-{}D | δ={:.4} | conf={:.3}",
                    budget.key(),
                    mode_stats.wins,
                    mode_stats.losses,
                    mode_stats.draws,
                    mode_stats.win_rate_points() - 0.5,
                    mode_stats.confidence_better_than_even(),
                );
            }
        }

        tiers.push(ProgressiveTierResult {
            tier: tier_index,
            cumulative_games: total_games,
            mode_stats: cumulative_mode_stats.clone(),
            aggregate_delta,
            aggregate_confidence,
            stop_reason,
        });

        if let Some(path) = artifact_path {
            flush_progressive_artifact(path, &tiers, budgets);
        }

        if let Some(reason) = stop_reason {
            return ProgressiveDuelResult {
                tiers,
                final_mode_stats: cumulative_mode_stats,
                final_delta: aggregate_delta,
                final_confidence: aggregate_confidence,
                total_games,
                stop_reason: reason,
            };
        }

        games_per_seed = (games_per_seed * config.growth_factor).min(config.max_games_per_seed);
        tier_index += 1;
    }
}

pub(super) fn evaluate_progressive_stop(
    budgets: &[SearchBudget],
    mode_stats: &HashMap<&'static str, MatchupStats>,
    aggregate_delta: f64,
    current_games_per_seed: usize,
    config: &ProgressiveDuelConfig,
) -> Option<ProgressiveStopReason> {
    if aggregate_delta < config.early_exit_delta_floor {
        return Some(ProgressiveStopReason::EarlyReject);
    }

    if let Some(first_tier_signal_games_per_seed) = config.first_tier_signal_games_per_seed {
        if current_games_per_seed == first_tier_signal_games_per_seed {
            let mut any_mode_showed_signal = false;
            let mut all_modes_cleared_floor = true;
            for budget in budgets {
                let stats = mode_stats.get(budget.key()).copied().unwrap_or_default();
                let mode_delta = stats.win_rate_points() - 0.5;
                if mode_delta >= config.first_tier_signal_mode_delta_min {
                    any_mode_showed_signal = true;
                }
                if mode_delta < config.first_tier_signal_mode_floor {
                    all_modes_cleared_floor = false;
                }
            }

            if aggregate_delta < config.first_tier_signal_aggregate_delta_min
                || !any_mode_showed_signal
                || !all_modes_cleared_floor
            {
                return Some(ProgressiveStopReason::EarlyReject);
            }
        }
    }

    let next_games_per_seed =
        (current_games_per_seed * config.growth_factor).min(config.max_games_per_seed);
    let at_max = current_games_per_seed >= config.max_games_per_seed;

    if at_max {
        let mut any_mode_improved = false;
        let mut non_regression_ok = true;
        for budget in budgets {
            let stats = mode_stats.get(budget.key()).copied().unwrap_or_default();
            let mode_delta = stats.win_rate_points() - 0.5;
            let mode_confidence = stats.confidence_better_than_even();
            let required_delta = config
                .mode_improvement_delta
                .get(budget.key())
                .copied()
                .unwrap_or(0.02);
            let required_confidence = config
                .mode_improvement_confidence
                .get(budget.key())
                .copied()
                .unwrap_or(0.60);
            let floor = config
                .mode_non_regression_delta
                .get(budget.key())
                .copied()
                .unwrap_or(-0.03);

            if mode_delta >= required_delta && mode_confidence >= required_confidence {
                any_mode_improved = true;
            }
            if mode_delta < floor {
                non_regression_ok = false;
            }
        }

        if any_mode_improved && non_regression_ok && aggregate_delta >= 0.0 {
            return Some(ProgressiveStopReason::EarlyPromote);
        }
        return Some(ProgressiveStopReason::MaxGamesReached);
    }

    let remaining_seed_factor = config.seed_tags.len() * config.repeats_per_game * 2;
    let mut remaining_per_mode = 0usize;
    let mut next = next_games_per_seed;
    while next <= config.max_games_per_seed {
        remaining_per_mode += next * remaining_seed_factor;
        if next >= config.max_games_per_seed {
            break;
        }
        next = (next * config.growth_factor).min(config.max_games_per_seed);
    }

    let any_mode_can_improve = budgets.iter().any(|budget| {
        let stats = mode_stats.get(budget.key()).copied().unwrap_or_default();
        let best_case = max_achievable_delta(stats, remaining_per_mode);
        let required_delta = config
            .mode_improvement_delta
            .get(budget.key())
            .copied()
            .unwrap_or(0.02);
        best_case >= required_delta
    });
    if !any_mode_can_improve {
        return Some(ProgressiveStopReason::MathematicalReject);
    }

    let mut any_mode_improved = false;
    let mut all_modes_non_regressed = true;
    for budget in budgets {
        let stats = mode_stats.get(budget.key()).copied().unwrap_or_default();
        let mode_delta = stats.win_rate_points() - 0.5;
        let mode_confidence = stats.confidence_better_than_even();
        let required_delta = config
            .mode_improvement_delta
            .get(budget.key())
            .copied()
            .unwrap_or(0.02);
        let required_confidence = config
            .mode_improvement_confidence
            .get(budget.key())
            .copied()
            .unwrap_or(0.60);
        let floor = config
            .mode_non_regression_delta
            .get(budget.key())
            .copied()
            .unwrap_or(-0.03);

        if mode_delta >= required_delta && mode_confidence >= required_confidence {
            any_mode_improved = true;
        }
        if mode_delta < floor {
            all_modes_non_regressed = false;
        }
    }

    let total_games: usize = mode_stats.values().map(|stats| stats.total_games()).sum();
    let min_games_for_early_promote =
        config.initial_games * config.growth_factor * remaining_seed_factor * budgets.len();
    if any_mode_improved
        && all_modes_non_regressed
        && aggregate_delta >= 0.0
        && total_games >= min_games_for_early_promote
    {
        return Some(ProgressiveStopReason::EarlyPromote);
    }

    None
}

pub(super) fn flush_progressive_artifact(
    path: &str,
    tiers: &[ProgressiveTierResult],
    budgets: &[SearchBudget],
) {
    let mut lines = Vec::with_capacity(tiers.len());
    for tier in tiers {
        let mut mode_parts = Vec::new();
        for budget in budgets {
            if let Some(stats) = tier.mode_stats.get(budget.key()) {
                mode_parts.push(format!(
                    r#""{}":{{"wins":{},"losses":{},"draws":{},"delta":{:.5},"confidence":{:.5}}}"#,
                    budget.key(),
                    stats.wins,
                    stats.losses,
                    stats.draws,
                    stats.win_rate_points() - 0.5,
                    stats.confidence_better_than_even()
                ));
            }
        }
        let stop = match tier.stop_reason {
            Some(ProgressiveStopReason::EarlyReject) => "\"early_reject\"",
            Some(ProgressiveStopReason::MathematicalReject) => "\"math_reject\"",
            Some(ProgressiveStopReason::EarlyPromote) => "\"early_promote\"",
            Some(ProgressiveStopReason::MaxGamesReached) => "\"max_games\"",
            None => "null",
        };
        lines.push(format!(
            r#"{{"tier":{},"cumulative_games":{},"aggregate_delta":{:.5},"aggregate_confidence":{:.5},"stop":{},"modes":{{{}}}}}"#,
            tier.tier,
            tier.cumulative_games,
            tier.aggregate_delta,
            tier.aggregate_confidence,
            stop,
            mode_parts.join(",")
        ));
    }
    let payload = lines.join("\n") + "\n";
    if let Err(error) = std::fs::write(path, payload.as_bytes()) {
        println!(
            "WARNING: failed writing progressive artifact to '{}': {}",
            path, error
        );
    }
}

pub(super) fn default_progressive_artifact_path(profile: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let log_dir = env::var("SMART_EXPERIMENT_LOG_DIR")
        .unwrap_or_else(|_| "target/experiment-runs".to_string());
    let _ = std::fs::create_dir_all(&log_dir);
    format!("{}/progressive_{}_{}.jsonl", log_dir, profile, timestamp)
}

pub(super) fn persist_ladder_artifacts(lines: &[String]) {
    let Some(path) = env::var("SMART_LADDER_ARTIFACT_PATH")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let payload = lines.join("\n");
    if let Err(error) = std::fs::write(path.as_str(), payload.as_bytes()) {
        panic!(
            "failed writing SMART_LADDER_ARTIFACT_PATH '{}': {}",
            path, error
        );
    }
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

fn reply_risk_triage_game() -> MonsGame {
    let mut game = tactical_game_with_items(
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
    game
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

fn duplicate_fixture_across_client_modes(id: &'static str, game: MonsGame) -> Vec<TriageFixture> {
    vec![
        TriageFixture {
            id,
            game: game.clone_for_simulation(),
            mode: SmartAutomovePreference::Fast,
            opening_book_driven: false,
        },
        TriageFixture {
            id,
            game,
            mode: SmartAutomovePreference::Normal,
            opening_book_driven: false,
        },
    ]
}

pub(super) fn generic_triage_surface_fixtures(surface: TriageSurface) -> Vec<TriageFixture> {
    match surface {
        TriageSurface::ReplyRisk => {
            duplicate_fixture_across_client_modes("reply_risk_handoff", reply_risk_triage_game())
        }
        TriageSurface::Supermana => duplicate_fixture_across_client_modes(
            "safe_supermana_progress",
            supermana_progress_triage_game(),
        ),
        TriageSurface::OpponentMana => duplicate_fixture_across_client_modes(
            "safe_opponent_mana_progress",
            opponent_mana_progress_triage_game(),
        ),
        TriageSurface::SpiritSetup => duplicate_fixture_across_client_modes(
            "spirit_setup_progress",
            spirit_setup_triage_game(),
        ),
        TriageSurface::DrainerSafety => duplicate_fixture_across_client_modes(
            "drainer_safety_filter",
            drainer_safety_triage_game(),
        ),
        TriageSurface::OpeningReply
        | TriageSurface::PrimaryPro
        | TriageSurface::CacheReuse => Vec::new(),
    }
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

fn opening_black_reply_triage_fixture(
    id: &'static str,
    sequence: &[&str; 5],
) -> TriageFixture {
    let mut game = MonsGame::new(false);
    apply_triage_opening_sequence(&mut game, sequence);
    TriageFixture {
        id,
        game,
        mode: SmartAutomovePreference::Pro,
        opening_book_driven: true,
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
        },
        TriageFixture {
            id: "primary_opponent_mana_progress",
            game: opponent_mana_progress_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
        },
        TriageFixture {
            id: "primary_spirit_setup",
            game: spirit_setup_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
        },
        TriageFixture {
            id: "primary_drainer_safety",
            game: drainer_safety_triage_game(),
            mode: SmartAutomovePreference::Pro,
            opening_book_driven: false,
        },
    ]
}
