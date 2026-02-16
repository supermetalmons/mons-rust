use super::*;
use crate::models::scoring::{
    FINISHER_BALANCED_SCORING_WEIGHTS, FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS,
    FINISHER_BALANCED_SOFT_SCORING_WEIGHTS, FINISHER_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS,
    FINISHER_MANA_RACE_LITE_SCORING_WEIGHTS,
    FINISHER_MANA_RACE_LITE_SOFT_AGGRESSIVE_SCORING_WEIGHTS,
    FINISHER_MANA_RACE_LITE_SOFT_SCORING_WEIGHTS,
    MANA_RACE_LITE_D2_TUNED_AGGRESSIVE_SCORING_WEIGHTS, MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS,
    MANA_RACE_LITE_SCORING_WEIGHTS, MANA_RACE_NEUTRAL_SCORING_WEIGHTS, MANA_RACE_SCORING_WEIGHTS,
    TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS, TACTICAL_BALANCED_SCORING_WEIGHTS,
    TACTICAL_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS, TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;
use std::env;
use std::sync::OnceLock;
use std::time::Instant;

const POOL_SIZE: usize = 10;
const GAMES_PER_MATCHUP: usize = 100;
const MAX_GAME_PLIES: usize = 320;
const OPENING_RANDOM_PLIES_MAX: usize = 6;
const MIN_CONFIDENCE_TO_PROMOTE: f64 = 0.75;
const MIN_OPPONENTS_BEAT_TO_PROMOTE: usize = 7;

#[derive(Debug, Clone, Copy)]
struct SearchBudget {
    depth: i32,
    max_nodes: i32,
}

impl SearchBudget {
    fn key(self) -> String {
        format!("d{}n{}", self.depth, self.max_nodes)
    }
}

const CLIENT_BUDGETS: [SearchBudget; 2] = [
    SearchBudget {
        depth: 2,
        max_nodes: 420,
    },
    SearchBudget {
        depth: 3,
        max_nodes: 2300,
    },
];

const CANDIDATE_SCORING_WEIGHTS_GUARDED: ScoringWeights = ScoringWeights {
    drainer_at_risk: -520,
    drainer_close_to_own_pool: 300,
    drainer_close_to_supermana: 160,
    spirit_close_to_enemy: 200,
    angel_guarding_drainer: 360,
    angel_close_to_friendly_drainer: 260,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

const CANDIDATE_SCORING_WEIGHTS_RUSH: ScoringWeights = ScoringWeights {
    drainer_at_risk: -300,
    mana_close_to_same_pool: 600,
    drainer_close_to_mana: 420,
    drainer_close_to_own_pool: 360,
    drainer_close_to_supermana: 220,
    mon_close_to_center: 140,
    spirit_close_to_enemy: 180,
    angel_guarding_drainer: 200,
    angel_close_to_friendly_drainer: 120,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

#[derive(Clone, Copy)]
struct AutomoveModel {
    id: &'static str,
    select_inputs: fn(&MonsGame, SmartSearchConfig) -> Vec<Input>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchResult {
    CandidateWin,
    OpponentWin,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameTermination {
    Winner(Color),
    CandidateNoMove,
    OpponentNoMove,
    CandidateInvalidMove,
    OpponentInvalidMove,
    MaxPlyAdjudicated(Option<Color>),
}

#[derive(Default, Debug, Clone, Copy)]
struct MatchupStats {
    wins: usize,
    losses: usize,
    draws: usize,
}

impl MatchupStats {
    fn record(&mut self, result: MatchResult) {
        match result {
            MatchResult::CandidateWin => self.wins += 1,
            MatchResult::OpponentWin => self.losses += 1,
            MatchResult::Draw => self.draws += 1,
        }
    }

    fn merge(&mut self, other: MatchupStats) {
        self.wins += other.wins;
        self.losses += other.losses;
        self.draws += other.draws;
    }

    fn total_games(&self) -> usize {
        self.wins + self.losses + self.draws
    }

    fn decisive_games(&self) -> usize {
        self.wins + self.losses
    }

    fn win_rate_points(&self) -> f64 {
        let total = self.total_games();
        if total == 0 {
            0.5
        } else {
            (self.wins as f64 + 0.5 * self.draws as f64) / total as f64
        }
    }

    fn confidence_better_than_even(&self) -> f64 {
        let decisive_games = self.decisive_games();
        if decisive_games == 0 || self.wins <= self.losses {
            return 0.0;
        }
        let p_value = one_sided_binomial_p_value(self.wins, decisive_games);
        (1.0 - p_value).clamp(0.0, 1.0)
    }
}

#[derive(Debug)]
struct OpponentEvaluation {
    opponent_id: &'static str,
    stats: MatchupStats,
}

#[derive(Debug)]
struct CandidateEvaluation {
    games_per_matchup: usize,
    beaten_opponents: usize,
    aggregate_stats: MatchupStats,
    aggregate_confidence: f64,
    promoted: bool,
    removed_model_id: Option<&'static str>,
    opponents: Vec<OpponentEvaluation>,
    mode_results: Vec<ModeEvaluation>,
}

#[derive(Debug)]
struct ModeEvaluation {
    budget: SearchBudget,
    beaten_opponents: usize,
    aggregate_stats: MatchupStats,
    aggregate_confidence: f64,
    opponents: Vec<OpponentEvaluation>,
}

impl CandidateEvaluation {
    fn render_report(&self, candidate_id: &'static str) -> String {
        let mut lines = vec![format!(
                "candidate={} promoted={} beaten={}/{} aggregate_win_rate={:.3} aggregate_confidence={:.3}",
                candidate_id,
                self.promoted,
                self.beaten_opponents,
                self.opponents.len(),
                self.aggregate_stats.win_rate_points(),
                self.aggregate_confidence
            )];

        if let Some(removed_model_id) = self.removed_model_id {
            lines.push(format!(
                "pool update: add={} remove={}",
                candidate_id, removed_model_id
            ));
        }

        for mode in &self.mode_results {
            lines.push(format!(
                "mode {}: beaten={}/{} win_rate={:.3} confidence={:.3}",
                mode.budget.key(),
                mode.beaten_opponents,
                mode.opponents.len(),
                mode.aggregate_stats.win_rate_points(),
                mode.aggregate_confidence
            ));

            let mut ordered_mode = mode.opponents.iter().collect::<Vec<_>>();
            ordered_mode.sort_by(|a, b| {
                b.stats
                    .win_rate_points()
                    .partial_cmp(&a.stats.win_rate_points())
                    .unwrap_or(Ordering::Equal)
            });
            for entry in ordered_mode {
                lines.push(format!(
                    "  {} vs {}: wins={} losses={} draws={} win_rate={:.3} confidence={:.3}",
                    mode.budget.key(),
                    entry.opponent_id,
                    entry.stats.wins,
                    entry.stats.losses,
                    entry.stats.draws,
                    entry.stats.win_rate_points(),
                    entry.stats.confidence_better_than_even()
                ));
            }
        }

        let mut ordered = self.opponents.iter().collect::<Vec<_>>();
        ordered.sort_by(|a, b| {
            b.stats
                .win_rate_points()
                .partial_cmp(&a.stats.win_rate_points())
                .unwrap_or(Ordering::Equal)
        });
        for entry in ordered {
            lines.push(format!(
                "combined vs {}: wins={} losses={} draws={} win_rate={:.3} confidence={:.3}",
                entry.opponent_id,
                entry.stats.wins,
                entry.stats.losses,
                entry.stats.draws,
                entry.stats.win_rate_points(),
                entry.stats.confidence_better_than_even()
            ));
        }

        lines.join("\n")
    }
}

fn model_current_best(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, config)
}

fn pool_model_01(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn pool_model_02(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn pool_model_03(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn pool_model_04(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn pool_model_05(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn pool_model_06(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn pool_model_07(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn pool_model_08(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn pool_model_09(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn pool_model_10(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn candidate_model_base(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, config)
}

fn candidate_model_focus(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_focus(config))
}

fn candidate_model_focus_deep_only(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    if config.depth >= 3 {
        MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_focus(config))
    } else {
        MonsGameModel::smart_search_best_inputs(game, config)
    }
}

fn candidate_model_weights_balanced(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_weights_balanced(config))
}

fn candidate_model_weights_guarded(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        game,
        with_scoring_weights(config, &CANDIDATE_SCORING_WEIGHTS_GUARDED),
    )
}

fn candidate_model_weights_rush(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        game,
        with_scoring_weights(config, &CANDIDATE_SCORING_WEIGHTS_RUSH),
    )
}

fn candidate_model_weights_mana_race(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        game,
        with_scoring_weights(config, &MANA_RACE_SCORING_WEIGHTS),
    )
}

fn candidate_model_weights_mana_race_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        game,
        with_scoring_weights(config, &MANA_RACE_LITE_SCORING_WEIGHTS),
    )
}

fn candidate_model_weights_mana_race_neutral(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        game,
        with_scoring_weights(config, &MANA_RACE_NEUTRAL_SCORING_WEIGHTS),
    )
}

fn candidate_model_focus_with_mana_race_d2(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        tuned_candidate_config_focus(config)
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_focus_light_with_mana_race_d2(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        tuned_candidate_config_focus_light(config)
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_focus_light_with_mana_race_d2_tactical(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(
            tuned_candidate_config_focus_light(config),
            &TACTICAL_BALANCED_SCORING_WEIGHTS,
        )
    } else {
        with_scoring_weights(config, &TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_focus_light_with_mana_race_d2_tactical_aggressive(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(
            tuned_candidate_config_focus_light(config),
            &TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS,
        )
    } else {
        with_scoring_weights(config, &TACTICAL_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_focus_light_with_tactical_d2_only(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        tuned_candidate_config_focus_light(config)
    } else {
        with_scoring_weights(config, &TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_focus_light_with_tactical_d2_only_aggressive(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        tuned_candidate_config_focus_light(config)
    } else {
        with_scoring_weights(config, &TACTICAL_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_focus_light_with_finisher_d2(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(
            tuned_candidate_config_focus_light(config),
            &FINISHER_BALANCED_SCORING_WEIGHTS,
        )
    } else {
        with_scoring_weights(config, &FINISHER_MANA_RACE_LITE_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_focus_light_with_finisher_d2_aggressive(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(
            tuned_candidate_config_focus_light(config),
            &FINISHER_BALANCED_SCORING_WEIGHTS,
        )
    } else {
        with_scoring_weights(config, &FINISHER_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_finisher_soft(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &FINISHER_BALANCED_SOFT_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, &FINISHER_MANA_RACE_LITE_SOFT_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_finisher_soft_aggressive(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(
            config,
            &FINISHER_MANA_RACE_LITE_SOFT_AGGRESSIVE_SCORING_WEIGHTS,
        )
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_d2_tuned(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &BALANCED_DISTANCE_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_d2_tuned_aggressive(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &BALANCED_DISTANCE_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_D2_TUNED_AGGRESSIVE_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_wideroot(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_wideroot(config))
}

fn candidate_model_narrow(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_narrow(config))
}

fn candidate_model_hybrid(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_hybrid(config))
}

fn candidate_model_deeper(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_deeper(config))
}

fn candidate_model_hybrid_deeper(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    if config.depth >= 3 {
        MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_deeper(config))
    } else {
        MonsGameModel::smart_search_best_inputs(game, config)
    }
}

fn candidate_model_hybrid_deeper_fast(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    if config.depth >= 3 {
        MonsGameModel::smart_search_best_inputs(
            game,
            tuned_candidate_config_hybrid_deeper_fast(config),
        )
    } else {
        MonsGameModel::smart_search_best_inputs(game, config)
    }
}

// Replace this when introducing a real contender.
fn candidate_model(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    match candidate_profile().as_str() {
        "base" => candidate_model_base(game, config),
        "focus" => candidate_model_focus(game, config),
        "focus_deep_only" => candidate_model_focus_deep_only(game, config),
        "focus_mana_d2" => candidate_model_focus_with_mana_race_d2(game, config),
        "focus_light_mana_d2" => candidate_model_focus_light_with_mana_race_d2(game, config),
        "focus_light_mana_d2_tactical" => {
            candidate_model_focus_light_with_mana_race_d2_tactical(game, config)
        }
        "focus_light_mana_d2_tactical_aggr" => {
            candidate_model_focus_light_with_mana_race_d2_tactical_aggressive(game, config)
        }
        "focus_light_tactical_d2_only" => {
            candidate_model_focus_light_with_tactical_d2_only(game, config)
        }
        "focus_light_tactical_d2_only_aggr" => {
            candidate_model_focus_light_with_tactical_d2_only_aggressive(game, config)
        }
        "focus_light_finisher_d2" => candidate_model_focus_light_with_finisher_d2(game, config),
        "focus_light_finisher_d2_aggr" => {
            candidate_model_focus_light_with_finisher_d2_aggressive(game, config)
        }
        "runtime_finisher_soft" => candidate_model_runtime_finisher_soft(game, config),
        "runtime_finisher_soft_aggr" => {
            candidate_model_runtime_finisher_soft_aggressive(game, config)
        }
        "runtime_d2_tuned" => candidate_model_runtime_d2_tuned(game, config),
        "runtime_d2_tuned_aggr" => candidate_model_runtime_d2_tuned_aggressive(game, config),
        "weights_balanced" => candidate_model_weights_balanced(game, config),
        "weights_guarded" => candidate_model_weights_guarded(game, config),
        "weights_rush" => candidate_model_weights_rush(game, config),
        "weights_mana_race" => candidate_model_weights_mana_race(game, config),
        "weights_mana_race_lite" => candidate_model_weights_mana_race_lite(game, config),
        "weights_mana_race_neutral" => candidate_model_weights_mana_race_neutral(game, config),
        "wideroot" => candidate_model_wideroot(game, config),
        "narrow" => candidate_model_narrow(game, config),
        "hybrid" => candidate_model_hybrid(game, config),
        "hybrid_deeper" => candidate_model_hybrid_deeper(game, config),
        "hybrid_deeper_fast" => candidate_model_hybrid_deeper_fast(game, config),
        "deeper" => candidate_model_deeper(game, config),
        _ => candidate_model_weights_balanced(game, config),
    }
}

fn candidate_profile() -> &'static String {
    static PROFILE: OnceLock<String> = OnceLock::new();
    PROFILE.get_or_init(|| {
        env::var("SMART_CANDIDATE_PROFILE")
            .ok()
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "base".to_string())
    })
}

fn tuned_candidate_config_focus(config: SmartSearchConfig) -> SmartSearchConfig {
    let mut tuned = config;

    if config.depth >= 3 {
        tuned.root_branch_limit = (config.root_branch_limit + 4).clamp(6, 36);
        tuned.node_branch_limit = config.node_branch_limit.saturating_sub(4).clamp(6, 18);
    } else {
        tuned.root_branch_limit = (config.root_branch_limit + 2).clamp(6, 32);
        tuned.node_branch_limit = config.node_branch_limit.clamp(6, 16);
    }

    tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 220);
    tuned.node_enum_limit = (tuned.node_branch_limit * 4).clamp(tuned.node_branch_limit, 108);
    tuned.scoring_weights = &BALANCED_DISTANCE_SCORING_WEIGHTS;
    tuned
}

fn tuned_candidate_config_focus_light(config: SmartSearchConfig) -> SmartSearchConfig {
    let mut tuned = config;

    if config.depth >= 3 {
        tuned.root_branch_limit = (config.root_branch_limit + 2).clamp(6, 32);
        tuned.node_branch_limit = config.node_branch_limit.saturating_sub(3).clamp(6, 18);
    } else {
        tuned.root_branch_limit = (config.root_branch_limit + 1).clamp(6, 30);
        tuned.node_branch_limit = config.node_branch_limit.saturating_sub(1).clamp(6, 16);
    }

    tuned.root_enum_limit = (tuned.root_branch_limit * 5).clamp(tuned.root_branch_limit, 190);
    tuned.node_enum_limit = (tuned.node_branch_limit * 4).clamp(tuned.node_branch_limit, 100);
    tuned.scoring_weights = &BALANCED_DISTANCE_SCORING_WEIGHTS;
    tuned
}

fn tuned_candidate_config_weights_balanced(config: SmartSearchConfig) -> SmartSearchConfig {
    with_scoring_weights(config, &BALANCED_DISTANCE_SCORING_WEIGHTS)
}

fn tuned_candidate_config_wideroot(config: SmartSearchConfig) -> SmartSearchConfig {
    let mut tuned = config;
    tuned.root_branch_limit = (config.root_branch_limit + 8).clamp(8, 40);
    tuned.node_branch_limit = config.node_branch_limit.saturating_sub(2).clamp(6, 18);
    tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 240);
    tuned.node_enum_limit = (tuned.node_branch_limit * 4).clamp(tuned.node_branch_limit, 108);
    tuned
}

fn tuned_candidate_config_narrow(config: SmartSearchConfig) -> SmartSearchConfig {
    let mut tuned = config;
    tuned.root_branch_limit = config.root_branch_limit.saturating_sub(4).clamp(6, 28);
    tuned.node_branch_limit = config.node_branch_limit.saturating_sub(5).clamp(5, 14);
    tuned.root_enum_limit = (tuned.root_branch_limit * 5).clamp(tuned.root_branch_limit, 180);
    tuned.node_enum_limit = (tuned.node_branch_limit * 3).clamp(tuned.node_branch_limit, 84);
    tuned
}

fn tuned_candidate_config_deeper(config: SmartSearchConfig) -> SmartSearchConfig {
    let mut tuned = tuned_candidate_config_focus(config);
    tuned.depth = (tuned.depth + 1).clamp(MIN_SMART_SEARCH_DEPTH, MAX_SMART_SEARCH_DEPTH);
    tuned.scoring_weights = &BALANCED_DISTANCE_SCORING_WEIGHTS;
    tuned
}

fn tuned_candidate_config_hybrid(config: SmartSearchConfig) -> SmartSearchConfig {
    if config.depth >= 3 {
        tuned_candidate_config_narrow(config)
    } else {
        config
    }
}

fn tuned_candidate_config_hybrid_deeper_fast(config: SmartSearchConfig) -> SmartSearchConfig {
    if config.depth < 3 {
        return config;
    }

    let mut tuned = config;
    tuned.max_visited_nodes = (tuned.max_visited_nodes * 2 / 3).max(700);
    tuned.root_branch_limit = tuned.root_branch_limit.saturating_sub(8).clamp(8, 24);
    tuned.node_branch_limit = tuned.node_branch_limit.saturating_sub(10).clamp(6, 12);
    tuned.root_enum_limit = (tuned.root_branch_limit * 5).clamp(tuned.root_branch_limit, 160);
    tuned.node_enum_limit = (tuned.node_branch_limit * 3).clamp(tuned.node_branch_limit, 72);
    tuned.depth = (tuned.depth + 1).clamp(MIN_SMART_SEARCH_DEPTH, MAX_SMART_SEARCH_DEPTH);
    tuned.scoring_weights = &BALANCED_DISTANCE_SCORING_WEIGHTS;
    tuned
}

fn with_scoring_weights(
    mut config: SmartSearchConfig,
    scoring_weights: &'static ScoringWeights,
) -> SmartSearchConfig {
    config.scoring_weights = scoring_weights;
    config
}

const POOL_MODELS: [AutomoveModel; POOL_SIZE] = [
    AutomoveModel {
        id: "pool_01",
        select_inputs: pool_model_01,
    },
    AutomoveModel {
        id: "pool_02",
        select_inputs: pool_model_02,
    },
    AutomoveModel {
        id: "pool_03",
        select_inputs: pool_model_03,
    },
    AutomoveModel {
        id: "pool_04",
        select_inputs: pool_model_04,
    },
    AutomoveModel {
        id: "pool_05",
        select_inputs: pool_model_05,
    },
    AutomoveModel {
        id: "pool_06",
        select_inputs: pool_model_06,
    },
    AutomoveModel {
        id: "pool_07",
        select_inputs: pool_model_07,
    },
    AutomoveModel {
        id: "pool_08",
        select_inputs: pool_model_08,
    },
    AutomoveModel {
        id: "pool_09",
        select_inputs: pool_model_09,
    },
    AutomoveModel {
        id: "pool_10",
        select_inputs: pool_model_10,
    },
];

const CANDIDATE_MODEL: AutomoveModel = AutomoveModel {
    id: "candidate",
    select_inputs: candidate_model,
};

fn evaluate_candidate_against_pool(
    candidate: AutomoveModel,
    pool: &[AutomoveModel],
    games_per_matchup: usize,
    budgets: &[SearchBudget],
) -> CandidateEvaluation {
    assert!(!budgets.is_empty());
    assert!(!pool.is_empty());

    let min_required_beaten = MIN_OPPONENTS_BEAT_TO_PROMOTE.min(pool.len());
    let mut mode_results = Vec::with_capacity(budgets.len());
    let mut combined_by_opponent: std::collections::HashMap<&'static str, MatchupStats> =
        std::collections::HashMap::new();
    let mut aggregate_stats = MatchupStats::default();

    for budget in budgets.iter().copied() {
        let mode_result = run_mode_evaluation(candidate, pool, games_per_matchup, budget);
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
        .count();
    let aggregate_confidence = aggregate_stats.confidence_better_than_even();
    let strong_in_each_mode = mode_results.iter().all(|mode| {
        mode.beaten_opponents >= min_required_beaten
            && mode.aggregate_stats.win_rate_points() > 0.5
            && mode.aggregate_confidence >= MIN_CONFIDENCE_TO_PROMOTE
    });
    let promoted = beaten_opponents >= min_required_beaten
        && aggregate_stats.win_rate_points() > 0.5
        && aggregate_confidence >= MIN_CONFIDENCE_TO_PROMOTE
        && strong_in_each_mode;

    let removed_model_id = if promoted {
        opponents
            .iter()
            .max_by(|a, b| {
                a.stats
                    .win_rate_points()
                    .partial_cmp(&b.stats.win_rate_points())
                    .unwrap_or(Ordering::Equal)
            })
            .map(|entry| entry.opponent_id)
    } else {
        None
    };

    CandidateEvaluation {
        games_per_matchup,
        beaten_opponents,
        aggregate_stats,
        aggregate_confidence,
        promoted,
        removed_model_id,
        opponents,
        mode_results,
    }
}

fn run_mode_evaluation(
    candidate: AutomoveModel,
    pool: &[AutomoveModel],
    games_per_matchup: usize,
    budget: SearchBudget,
) -> ModeEvaluation {
    let mut opponents = Vec::with_capacity(pool.len());
    let mut aggregate_stats = MatchupStats::default();

    let mut handles = Vec::with_capacity(pool.len());
    for opponent in pool.iter().copied() {
        handles.push(std::thread::spawn(move || {
            let stats = run_matchup_series(
                candidate,
                opponent,
                games_per_matchup,
                budget,
                seed_for_pairing_and_budget(candidate.id, opponent.id, budget),
            );
            OpponentEvaluation {
                opponent_id: opponent.id,
                stats,
            }
        }));
    }

    for handle in handles {
        let entry = handle.join().expect("matchup worker thread panicked");
        aggregate_stats.merge(entry.stats);
        opponents.push(entry);
    }
    opponents.sort_by(|a, b| a.opponent_id.cmp(b.opponent_id));

    let beaten_opponents = opponents
        .iter()
        .filter(|entry| {
            entry.stats.win_rate_points() > 0.5
                && entry.stats.confidence_better_than_even() >= MIN_CONFIDENCE_TO_PROMOTE
        })
        .count();

    ModeEvaluation {
        budget,
        beaten_opponents,
        aggregate_stats,
        aggregate_confidence: aggregate_stats.confidence_better_than_even(),
        opponents,
    }
}

fn run_matchup_series(
    candidate: AutomoveModel,
    opponent: AutomoveModel,
    games_per_matchup: usize,
    budget: SearchBudget,
    seed: u64,
) -> MatchupStats {
    let opening_fens = generate_opening_fens(seed, games_per_matchup);
    let max_plies = env_usize("SMART_POOL_MAX_PLIES").unwrap_or(MAX_GAME_PLIES);
    let mut stats = MatchupStats::default();

    for game_index in 0..games_per_matchup {
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

fn play_one_game(
    candidate: AutomoveModel,
    opponent: AutomoveModel,
    budget: SearchBudget,
    candidate_is_white: bool,
    opening_fen: &str,
    max_plies: usize,
) -> MatchResult {
    play_one_game_with_diagnostics(
        candidate,
        opponent,
        budget,
        candidate_is_white,
        opening_fen,
        max_plies,
    )
    .0
}

fn play_one_game_with_diagnostics(
    candidate: AutomoveModel,
    opponent: AutomoveModel,
    budget: SearchBudget,
    candidate_is_white: bool,
    opening_fen: &str,
    max_plies: usize,
) -> (MatchResult, usize, GameTermination) {
    let mut game = MonsGame::from_fen(opening_fen, false).expect("valid opening fen");

    for ply in 0..max_plies {
        if let Some(winner_color) = game.winner_color() {
            return (
                match_result_from_winner(winner_color, candidate_is_white),
                ply,
                GameTermination::Winner(winner_color),
            );
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

        let config = SmartSearchConfig::from_budget(budget.depth, budget.max_nodes).for_runtime();
        let inputs = (actor_model.select_inputs)(&game, config);
        if inputs.is_empty() {
            let result = if candidate_to_move {
                MatchResult::OpponentWin
            } else {
                MatchResult::CandidateWin
            };
            let termination = if candidate_to_move {
                GameTermination::CandidateNoMove
            } else {
                GameTermination::OpponentNoMove
            };
            return (result, ply + 1, termination);
        }

        match game.process_input(inputs, false, false) {
            Output::Events(_) => {}
            _ => {
                let result = if candidate_to_move {
                    MatchResult::OpponentWin
                } else {
                    MatchResult::CandidateWin
                };
                let termination = if candidate_to_move {
                    GameTermination::CandidateInvalidMove
                } else {
                    GameTermination::OpponentInvalidMove
                };
                return (result, ply + 1, termination);
            }
        }
    }

    let adjudicated_winner = adjudicate_non_terminal_game(&game);
    let result = match adjudicated_winner {
        Some(winner_color) => match_result_from_winner(winner_color, candidate_is_white),
        None => MatchResult::Draw,
    };
    (
        result,
        max_plies,
        GameTermination::MaxPlyAdjudicated(adjudicated_winner),
    )
}

fn match_result_from_winner(winner_color: Color, candidate_is_white: bool) -> MatchResult {
    if (candidate_is_white && winner_color == Color::White)
        || (!candidate_is_white && winner_color == Color::Black)
    {
        MatchResult::CandidateWin
    } else {
        MatchResult::OpponentWin
    }
}

fn adjudicate_non_terminal_game(game: &MonsGame) -> Option<Color> {
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

fn generate_opening_fens(seed: u64, count: usize) -> Vec<String> {
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

fn apply_seeded_random_move(game: &mut MonsGame, rng: &mut StdRng) -> bool {
    let legal_inputs = MonsGameModel::enumerate_legal_inputs(game, 256);
    if legal_inputs.is_empty() {
        return false;
    }

    let random_index = rng.gen_range(0..legal_inputs.len());
    let chosen_inputs = legal_inputs[random_index].clone();
    matches!(
        game.process_input(chosen_inputs, false, false),
        Output::Events(_)
    )
}

fn one_sided_binomial_p_value(successes: usize, trials: usize) -> f64 {
    if trials == 0 {
        return 1.0;
    }
    if successes == 0 {
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

fn seed_for_pairing(candidate_id: &str, opponent_id: &str) -> u64 {
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

fn seed_for_pairing_and_budget(candidate_id: &str, opponent_id: &str, budget: SearchBudget) -> u64 {
    let mut hash = seed_for_pairing(candidate_id, opponent_id);
    hash ^= (budget.depth as u64).wrapping_mul(0x9e3779b97f4a7c15);
    hash = hash.wrapping_mul(1099511628211);
    hash ^= (budget.max_nodes as u64).wrapping_mul(0x517cc1b727220a95);
    hash = hash.wrapping_mul(1099511628211);
    hash
}

fn env_usize(name: &str) -> Option<usize> {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
}

fn selected_pool_models() -> Vec<AutomoveModel> {
    let requested = env_usize("SMART_POOL_OPPONENTS").unwrap_or(POOL_SIZE);
    let limit = requested.clamp(1, POOL_SIZE);
    POOL_MODELS.iter().copied().take(limit).collect()
}

#[test]
fn smart_automove_pool_keeps_ten_models() {
    assert_eq!(POOL_MODELS.len(), POOL_SIZE);
}

#[test]
fn smart_automove_pool_smoke_runs() {
    let quick_budgets = [SearchBudget {
        depth: 1,
        max_nodes: 96,
    }];
    let pool = selected_pool_models();
    let evaluation = evaluate_candidate_against_pool(CANDIDATE_MODEL, &pool, 2, &quick_budgets);

    assert_eq!(evaluation.opponents.len(), pool.len());
    assert_eq!(evaluation.games_per_matchup, 2);
}

#[test]
#[ignore = "long-running tournament using production client budgets"]
fn smart_automove_pool_candidate_promotion_with_client_budgets() {
    let games_per_matchup = env_usize("SMART_POOL_GAMES").unwrap_or(GAMES_PER_MATCHUP);
    let pool = selected_pool_models();
    let evaluation =
        evaluate_candidate_against_pool(CANDIDATE_MODEL, &pool, games_per_matchup, &CLIENT_BUDGETS);
    println!(
        "settings: profile={} games_per_matchup={} opponents={} budgets={:?}",
        candidate_profile(),
        games_per_matchup,
        pool.len(),
        CLIENT_BUDGETS
            .iter()
            .map(|budget| format!("d{}n{}", budget.depth, budget.max_nodes))
            .collect::<Vec<_>>()
    );
    println!("{}", evaluation.render_report(CANDIDATE_MODEL.id));

    assert_eq!(evaluation.opponents.len(), pool.len());
    if evaluation.promoted {
        assert!(evaluation.removed_model_id.is_some());
    } else {
        assert!(evaluation.removed_model_id.is_none());
    }
}

#[test]
#[ignore = "profile sweep to compare candidate variants on the same openings"]
fn smart_automove_pool_profile_sweep() {
    let games_per_matchup = env_usize("SMART_POOL_GAMES").unwrap_or(2);
    let pool = selected_pool_models();
    let requested_profiles = env::var("SMART_SWEEP_PROFILES")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|name| name.trim().to_lowercase())
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let all_variants: [(&str, fn(&MonsGame, SmartSearchConfig) -> Vec<Input>); 27] = [
        ("base", candidate_model_base),
        ("focus", candidate_model_focus),
        ("focus_deep_only", candidate_model_focus_deep_only),
        ("focus_mana_d2", candidate_model_focus_with_mana_race_d2),
        (
            "focus_light_mana_d2",
            candidate_model_focus_light_with_mana_race_d2,
        ),
        (
            "focus_light_mana_d2_tactical",
            candidate_model_focus_light_with_mana_race_d2_tactical,
        ),
        (
            "focus_light_mana_d2_tactical_aggr",
            candidate_model_focus_light_with_mana_race_d2_tactical_aggressive,
        ),
        (
            "focus_light_tactical_d2_only",
            candidate_model_focus_light_with_tactical_d2_only,
        ),
        (
            "focus_light_tactical_d2_only_aggr",
            candidate_model_focus_light_with_tactical_d2_only_aggressive,
        ),
        (
            "focus_light_finisher_d2",
            candidate_model_focus_light_with_finisher_d2,
        ),
        (
            "focus_light_finisher_d2_aggr",
            candidate_model_focus_light_with_finisher_d2_aggressive,
        ),
        (
            "runtime_finisher_soft",
            candidate_model_runtime_finisher_soft,
        ),
        (
            "runtime_finisher_soft_aggr",
            candidate_model_runtime_finisher_soft_aggressive,
        ),
        ("runtime_d2_tuned", candidate_model_runtime_d2_tuned),
        (
            "runtime_d2_tuned_aggr",
            candidate_model_runtime_d2_tuned_aggressive,
        ),
        ("weights_balanced", candidate_model_weights_balanced),
        ("weights_guarded", candidate_model_weights_guarded),
        ("weights_rush", candidate_model_weights_rush),
        ("weights_mana_race", candidate_model_weights_mana_race),
        (
            "weights_mana_race_lite",
            candidate_model_weights_mana_race_lite,
        ),
        (
            "weights_mana_race_neutral",
            candidate_model_weights_mana_race_neutral,
        ),
        ("wideroot", candidate_model_wideroot),
        ("narrow", candidate_model_narrow),
        ("hybrid", candidate_model_hybrid),
        ("hybrid_deeper", candidate_model_hybrid_deeper),
        ("hybrid_deeper_fast", candidate_model_hybrid_deeper_fast),
        ("deeper", candidate_model_deeper),
    ];
    let variants = all_variants
        .into_iter()
        .filter(|(name, _)| {
            requested_profiles.is_empty() || requested_profiles.contains(&name.to_string())
        })
        .collect::<Vec<_>>();

    for (name, selector) in variants {
        let candidate = AutomoveModel {
            id: "candidate",
            select_inputs: selector,
        };
        let evaluation =
            evaluate_candidate_against_pool(candidate, &pool, games_per_matchup, &CLIENT_BUDGETS);
        println!(
            "profile sweep: name={} games_per_matchup={} opponents={} max_plies={}",
            name,
            games_per_matchup,
            pool.len(),
            env_usize("SMART_POOL_MAX_PLIES").unwrap_or(MAX_GAME_PLIES),
        );
        println!("{}", evaluation.render_report(candidate.id));
    }
}

#[test]
#[ignore = "diagnostic run for understanding tournament runtime/game lengths"]
fn smart_automove_pool_runtime_diagnostics() {
    let games = env_usize("SMART_DIAG_GAMES").unwrap_or(4).max(1);
    let budget = SearchBudget {
        depth: env_usize("SMART_DIAG_DEPTH")
            .map(|value| value as i32)
            .unwrap_or(3),
        max_nodes: env_usize("SMART_DIAG_NODES")
            .map(|value| value as i32)
            .unwrap_or(2300),
    };

    let openings = generate_opening_fens(seed_for_pairing("diag", "diag"), games);
    let mut ply_sum = 0usize;

    for game_index in 0..games {
        let candidate_is_white = game_index % 2 == 0;
        let (result, plies, termination) = play_one_game_with_diagnostics(
            CANDIDATE_MODEL,
            POOL_MODELS[0],
            budget,
            candidate_is_white,
            openings[game_index].as_str(),
            env_usize("SMART_POOL_MAX_PLIES").unwrap_or(MAX_GAME_PLIES),
        );
        ply_sum += plies;
        println!(
            "diag game {}: result={:?} plies={} termination={:?}",
            game_index + 1,
            result,
            plies,
            termination
        );
    }

    println!(
        "diag summary: games={} avg_plies={:.2} budget=d{}n{}",
        games,
        ply_sum as f64 / games as f64,
        budget.depth,
        budget.max_nodes
    );
}

#[test]
#[ignore = "profile speed probe on fixed opening positions"]
fn smart_automove_pool_profile_speed_probe() {
    let positions = env_usize("SMART_SPEED_POSITIONS").unwrap_or(20).max(1);
    let openings = generate_opening_fens(seed_for_pairing("speed", "probe"), positions);
    let profile = candidate_profile().as_str().to_string();
    let selector = CANDIDATE_MODEL.select_inputs;

    println!(
        "speed probe: profile={} positions={} budgets={:?}",
        profile,
        positions,
        CLIENT_BUDGETS
            .iter()
            .map(|budget| format!("d{}n{}", budget.depth, budget.max_nodes))
            .collect::<Vec<_>>()
    );

    for budget in CLIENT_BUDGETS {
        let start = Instant::now();
        let mut total_moves = 0usize;

        for opening in &openings {
            let game = MonsGame::from_fen(opening, false).expect("valid opening fen");
            let config =
                SmartSearchConfig::from_budget(budget.depth, budget.max_nodes).for_runtime();
            let inputs = selector(&game, config);
            total_moves += inputs.len();
        }

        let elapsed = start.elapsed();
        let avg_ms = elapsed.as_secs_f64() * 1000.0 / positions as f64;
        println!(
                "speed probe mode {}: elapsed_ms={:.2} avg_ms_per_position={:.2} avg_inputs_per_move={:.2}",
                budget.key(),
                elapsed.as_secs_f64() * 1000.0,
                avg_ms,
                total_moves as f64 / positions as f64
            );
    }
}
