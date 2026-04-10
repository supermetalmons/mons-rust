use super::*;
use crate::models::scoring::{evaluate_preferability_with_weights, DEFAULT_SCORING_WEIGHTS};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::env;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

type AutomoveSelector = fn(&MonsGame, SmartSearchConfig) -> Vec<Input>;

const OPENING_RANDOM_PLIES_MAX: usize = 6;
pub(super) const SMART_PRO_RELIABILITY_WIN_RATE_MIN: f64 = 0.90;
pub(super) const SMART_PRO_RELIABILITY_CONFIDENCE_MIN: f64 = 0.99;
pub(super) const SMART_PRO_RELIABILITY_MOVE_AVG_MS_MAX: f64 = 700.0;
// Stronger pro candidates may also be cheaper than the current runtime; keep a
// floor that preserves a meaningful pro budget without blocking genuinely stronger
// but cheaper search configurations (e.g. breadth-over-depth wins).
pub(super) const SMART_PRO_CPU_RATIO_TARGET_MIN: f64 = 0.50;
pub(super) const SMART_PRO_CPU_RATIO_TARGET_MAX: f64 = 10.00;
pub(super) const SMART_STAGE1_CPU_RATIO_MAX_FAST: f64 = 1.30;
pub(super) const SMART_STAGE1_CPU_RATIO_MAX_NORMAL: f64 = 1.30;
pub(super) const SMART_STAGE1_CPU_RATIO_MAX_PRO: f64 = 1.30;
pub(super) const SMART_EXACT_LITE_CACHE_HIT_RATE_MIN: f64 = 0.20;

#[derive(Debug, Clone, Copy)]
struct SearchBudget {
    label: &'static str,
    depth: i32,
    max_nodes: i32,
}

impl SearchBudget {
    fn from_preference(preference: SmartAutomovePreference) -> Self {
        let (depth, max_nodes) = preference.depth_and_max_nodes();
        Self {
            label: preference.as_api_value(),
            depth,
            max_nodes,
        }
    }

    pub(super) fn key(self) -> &'static str {
        self.label
    }

    fn runtime_config_for_game(self, game: &MonsGame) -> SmartSearchConfig {
        if let Some(preference) = SmartAutomovePreference::from_api_value(self.label) {
            let hinted_context = if matches!(preference, SmartAutomovePreference::Pro)
                && env_bool("SMART_USE_WHITE_OPENING_BOOK").unwrap_or(false)
            {
                ProRuntimeContext::OpeningBookDriven
            } else {
                ProRuntimeContext::Unknown
            };
            MonsGameModel::runtime_config_for_game_with_context(game, preference, hinted_context).0
        } else {
            MonsGameModel::with_runtime_scoring_weights(
                game,
                SmartSearchConfig::from_budget(self.depth, self.max_nodes).for_runtime(),
            )
        }
    }
}

fn client_budgets() -> [SearchBudget; 2] {
    [
        SearchBudget::from_preference(SmartAutomovePreference::Fast),
        SearchBudget::from_preference(SmartAutomovePreference::Normal),
    ]
}

fn pro_budget() -> SearchBudget {
    SearchBudget::from_preference(SmartAutomovePreference::Pro)
}

#[derive(Clone, Copy)]
struct AutomoveModel {
    select_inputs: AutomoveSelector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
        (1.0_f64 - p_value).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct DuelTimingStats {
    candidate_total_ms: f64,
    baseline_total_ms: f64,
    candidate_turns: usize,
    baseline_turns: usize,
}

impl DuelTimingStats {
    fn record_candidate_turn(&mut self, elapsed_ms: f64) {
        self.candidate_total_ms += elapsed_ms;
        self.candidate_turns += 1;
    }

    fn record_baseline_turn(&mut self, elapsed_ms: f64) {
        self.baseline_total_ms += elapsed_ms;
        self.baseline_turns += 1;
    }

    fn merge(&mut self, other: DuelTimingStats) {
        self.candidate_total_ms += other.candidate_total_ms;
        self.baseline_total_ms += other.baseline_total_ms;
        self.candidate_turns += other.candidate_turns;
        self.baseline_turns += other.baseline_turns;
    }

    fn candidate_avg_ms(&self) -> f64 {
        self.candidate_total_ms / self.candidate_turns.max(1) as f64
    }

    fn baseline_avg_ms(&self) -> f64 {
        self.baseline_total_ms / self.baseline_turns.max(1) as f64
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TimedMatchupStats {
    matchup: MatchupStats,
    timing: DuelTimingStats,
}

impl TimedMatchupStats {
    fn record(&mut self, result: MatchResult, timing: DuelTimingStats) {
        self.matchup.record(result);
        self.timing.merge(timing);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchResult {
    CandidateWin,
    OpponentWin,
    Draw,
}

#[derive(Clone, Copy)]
struct ModeSpeedStat {
    budget: SearchBudget,
    avg_ms: f64,
}

mod harness;
mod profiles;
#[cfg(test)]
mod tests;

use self::harness::{env_bool, one_sided_binomial_p_value};
