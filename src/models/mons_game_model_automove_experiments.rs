use super::*;
use crate::models::scoring::{
    evaluate_preferability_breakdown_with_weights, FINISHER_BALANCED_SCORING_WEIGHTS,
    FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS, FINISHER_BALANCED_SOFT_SCORING_WEIGHTS,
    FINISHER_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS, FINISHER_MANA_RACE_LITE_SCORING_WEIGHTS,
    FINISHER_MANA_RACE_LITE_SOFT_AGGRESSIVE_SCORING_WEIGHTS,
    FINISHER_MANA_RACE_LITE_SOFT_SCORING_WEIGHTS,
    MANA_RACE_LITE_D2_TUNED_AGGRESSIVE_SCORING_WEIGHTS, MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS,
    MANA_RACE_LITE_SCORING_WEIGHTS, MANA_RACE_NEUTRAL_SCORING_WEIGHTS, MANA_RACE_SCORING_WEIGHTS,
    RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS, RUNTIME_FAST_DRAINER_PRIORITY_SCORING_WEIGHTS,
    RUNTIME_NORMAL_WINLOSS_SCORING_WEIGHTS, RUNTIME_RUSH_SCORING_WEIGHTS,
    TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS, TACTICAL_BALANCED_SCORING_WEIGHTS,
    TACTICAL_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS, TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;
use std::env;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

const POOL_SIZE: usize = 10;
const GAMES_PER_MATCHUP: usize = 100;
const MAX_GAME_PLIES: usize = 320;
const OPENING_RANDOM_PLIES_MAX: usize = 6;
const MIN_CONFIDENCE_TO_PROMOTE: f64 = 0.75;
const MIN_OPPONENTS_BEAT_TO_PROMOTE: usize = 7;
const LEGACY_NORMAL_MAX_VISITED_NODES: i32 = 2300;
const LEGACY_RUNTIME_FAST_MAX_VISITED_NODES: i32 = 420;
const LEGACY_RUNTIME_NORMAL_MAX_VISITED_NODES: i32 = 3450;
const SMART_BUDGET_CONVERSION_REGRESSION_TOLERANCE: f64 = 0.04;

#[derive(Debug, Clone, Copy)]
struct SearchBudget {
    label: &'static str,
    depth: i32,
    max_nodes: i32,
}

#[derive(Clone, Copy, Debug)]
struct BudgetConversionDiagnostic {
    fast_wins: usize,
    fast_losses: usize,
    draws: usize,
    fast_win_rate: f64,
    normal_edge: f64,
    confidence: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum EvalPhaseFamily {
    Fast,
    NormalBalanced,
    NormalTactical,
    NormalTacticalAggressive,
    NormalFinisher,
    NormalFinisherAggressive,
}

impl EvalPhaseFamily {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::NormalBalanced => "normal_balanced",
            Self::NormalTactical => "normal_tactical",
            Self::NormalTacticalAggressive => "normal_tactical_aggr",
            Self::NormalFinisher => "normal_finisher",
            Self::NormalFinisherAggressive => "normal_finisher_aggr",
        }
    }
}

#[derive(Clone)]
struct EvalTuningSample {
    mode: &'static str,
    phase_family: EvalPhaseFamily,
    fen: String,
    input: String,
    after_fen: String,
    after_game: MonsGame,
    perspective: Color,
    selected: bool,
    score: i32,
    best_score: i32,
    regret: i32,
    efficiency: i32,
    attack_drainer: bool,
    own_drainer_vulnerable: bool,
    mana_handoff: bool,
    roundtrip: bool,
    class_immediate_score: bool,
    class_drainer_attack: bool,
    class_drainer_safety_recover: bool,
    class_carrier_progress: bool,
    class_material: bool,
    class_quiet: bool,
    eval_total: i32,
    eval_confirmed_score: i32,
    eval_consumable_score: i32,
    eval_score_race_path: i32,
    eval_opp_score_race_path: i32,
    eval_score_race_multi: i32,
    eval_opp_score_race_multi: i32,
    eval_immediate_window: i32,
    eval_opp_immediate_window: i32,
    eval_immediate_multi: i32,
    eval_opp_immediate_multi: i32,
    eval_match_point: i32,
    eval_residual: i32,
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

    fn key(self) -> &'static str {
        self.label
    }

    fn runtime_config_for_game(self, game: &MonsGame) -> SmartSearchConfig {
        let config = if let Some(preference) = SmartAutomovePreference::from_api_value(self.label) {
            SmartSearchConfig::from_preference(preference)
        } else {
            SmartSearchConfig::from_budget(self.depth, self.max_nodes).for_runtime()
        };
        MonsGameModel::with_runtime_scoring_weights(game, config)
    }
}

fn client_budgets() -> [SearchBudget; 2] {
    [
        SearchBudget::from_preference(SmartAutomovePreference::Fast),
        SearchBudget::from_preference(SmartAutomovePreference::Normal),
    ]
}

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

const DRAINER_PRIORITY_FAST_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    drainer_at_risk: -520,
    drainer_close_to_mana: 370,
    drainer_holding_mana: 470,
    drainer_close_to_own_pool: 350,
    regular_mana_drainer_control: 24,
    mana_carrier_at_risk: -240,
    mana_carrier_guarded: 120,
    mana_carrier_one_step_from_pool: 310,
    supermana_carrier_one_step_from_pool_extra: 190,
    immediate_winning_carrier: 460,
    angel_guarding_drainer: 340,
    ..MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS
};

const DRAINER_PRIORITY_NORMAL_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    drainer_at_risk: -560,
    drainer_close_to_mana: 390,
    drainer_holding_mana: 500,
    drainer_close_to_own_pool: 380,
    regular_mana_drainer_control: 24,
    mana_carrier_at_risk: -280,
    mana_carrier_guarded: 150,
    mana_carrier_one_step_from_pool: 330,
    supermana_carrier_one_step_from_pool_extra: 220,
    immediate_winning_carrier: 640,
    angel_guarding_drainer: 360,
    ..TACTICAL_BALANCED_SCORING_WEIGHTS
};

const DRAINER_PRIORITY_NORMAL_AGGR_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    drainer_at_risk: -620,
    mana_carrier_at_risk: -320,
    mana_carrier_guarded: 165,
    mana_carrier_one_step_from_pool: 360,
    supermana_carrier_one_step_from_pool_extra: 240,
    immediate_winning_carrier: 820,
    spirit_close_to_enemy: 270,
    angel_guarding_drainer: 370,
    ..DRAINER_PRIORITY_NORMAL_SCORING_WEIGHTS
};

const BALANCED_DISTANCE_SPIRIT_BASE_STRICT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    spirit_on_own_base_penalty: 260,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

const TACTICAL_BALANCED_SPIRIT_BASE_STRICT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    spirit_on_own_base_penalty: 260,
    ..TACTICAL_BALANCED_SCORING_WEIGHTS
};

const TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_STRICT_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        spirit_on_own_base_penalty: 260,
        ..TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS
    };

const FINISHER_BALANCED_SOFT_SPIRIT_BASE_STRICT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    spirit_on_own_base_penalty: 260,
    ..FINISHER_BALANCED_SOFT_SCORING_WEIGHTS
};

const FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_STRICT_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        spirit_on_own_base_penalty: 260,
        ..FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    };

const BALANCED_DISTANCE_CONFIRMED_850_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    confirmed_score: 850,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

const TACTICAL_BALANCED_CONFIRMED_850_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    confirmed_score: 850,
    ..TACTICAL_BALANCED_SCORING_WEIGHTS
};

const TACTICAL_BALANCED_AGGRESSIVE_CONFIRMED_850_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    confirmed_score: 850,
    ..TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS
};

const FINISHER_BALANCED_SOFT_CONFIRMED_850_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    confirmed_score: 850,
    ..FINISHER_BALANCED_SOFT_SCORING_WEIGHTS
};

const FINISHER_BALANCED_SOFT_AGGRESSIVE_CONFIRMED_850_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        confirmed_score: 850,
        ..FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    };

const RUNTIME_PRE_HORIZON_FAST_CONTEXT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    score_race_path_progress: 0,
    opponent_score_race_path_progress: 0,
    immediate_score_window: 0,
    opponent_immediate_score_window: 0,
    spirit_action_utility: 0,
    ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
};

const RUNTIME_PRE_HORIZON_NORMAL_BALANCED_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    score_race_path_progress: 0,
    opponent_score_race_path_progress: 0,
    immediate_score_window: 0,
    opponent_immediate_score_window: 0,
    spirit_action_utility: 0,
    ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_PRE_HORIZON_NORMAL_TACTICAL_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    score_race_path_progress: 0,
    opponent_score_race_path_progress: 0,
    immediate_score_window: 0,
    opponent_immediate_score_window: 0,
    spirit_action_utility: 0,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_PRE_HORIZON_NORMAL_TACTICAL_AGGR_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    score_race_path_progress: 0,
    opponent_score_race_path_progress: 0,
    immediate_score_window: 0,
    opponent_immediate_score_window: 0,
    spirit_action_utility: 0,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_PRE_HORIZON_NORMAL_FINISHER_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    score_race_path_progress: 0,
    opponent_score_race_path_progress: 0,
    immediate_score_window: 0,
    opponent_immediate_score_window: 0,
    spirit_action_utility: 0,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_PRE_HORIZON_NORMAL_FINISHER_AGGR_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    score_race_path_progress: 0,
    opponent_score_race_path_progress: 0,
    immediate_score_window: 0,
    opponent_immediate_score_window: 0,
    spirit_action_utility: 0,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_PRE_EVAL_V2: ScoringWeights = ScoringWeights {
    use_legacy_formula: true,
    confirmed_score: 1000,
    drainer_best_mana_path: 220,
    drainer_pickup_score_this_turn: 180,
    mana_carrier_score_this_turn: 260,
    drainer_immediate_threat: -190,
    score_race_path_progress: 140,
    opponent_score_race_path_progress: 120,
    immediate_score_window: 210,
    opponent_immediate_score_window: 170,
    spirit_action_utility: 44,
    mana_carrier_at_risk: -250,
    mana_carrier_guarded: 130,
    mana_carrier_one_step_from_pool: 300,
    supermana_carrier_one_step_from_pool_extra: 190,
    ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
};

const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_PRE_MULTI_LANE: ScoringWeights =
    ScoringWeights {
        score_race_multi_path: 0,
        opponent_score_race_multi_path: 0,
        immediate_score_multi_window: 0,
        opponent_immediate_score_multi_window: 0,
        ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_PRE_EVAL_V2: ScoringWeights =
    ScoringWeights {
        use_legacy_formula: true,
        confirmed_score: 1000,
        score_race_path_progress: 72,
        opponent_score_race_path_progress: 132,
        immediate_score_window: 70,
        opponent_immediate_score_window: 170,
        spirit_action_utility: 58,
        ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_PRE_EVAL_V2: ScoringWeights =
    ScoringWeights {
        use_legacy_formula: true,
        confirmed_score: 1000,
        score_race_path_progress: 82,
        opponent_score_race_path_progress: 155,
        immediate_score_window: 82,
        opponent_immediate_score_window: 215,
        spirit_action_utility: 64,
        ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_PRE_EVAL_V2:
    ScoringWeights = ScoringWeights {
    use_legacy_formula: true,
    confirmed_score: 1000,
    score_race_path_progress: 92,
    opponent_score_race_path_progress: 180,
    immediate_score_window: 95,
    opponent_immediate_score_window: 250,
    spirit_action_utility: 70,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_PRE_EVAL_V2:
    ScoringWeights = ScoringWeights {
    use_legacy_formula: true,
    confirmed_score: 1000,
    score_race_path_progress: 140,
    opponent_score_race_path_progress: 130,
    immediate_score_window: 210,
    opponent_immediate_score_window: 175,
    spirit_action_utility: 58,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_PRE_EVAL_V2:
    ScoringWeights = ScoringWeights {
    use_legacy_formula: true,
    confirmed_score: 1000,
    score_race_path_progress: 160,
    opponent_score_race_path_progress: 145,
    immediate_score_window: 260,
    opponent_immediate_score_window: 195,
    spirit_action_utility: 62,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW: ScoringWeights =
    ScoringWeights {
        include_regular_mana_move_windows: true,
        ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW: ScoringWeights =
    ScoringWeights {
        include_regular_mana_move_windows: true,
        ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW:
    ScoringWeights = ScoringWeights {
    include_regular_mana_move_windows: true,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW:
    ScoringWeights = ScoringWeights {
    include_regular_mana_move_windows: true,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW:
    ScoringWeights = ScoringWeights {
    include_regular_mana_move_windows: true,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE:
    ScoringWeights = ScoringWeights {
    score_race_path_progress: 100,
    opponent_score_race_path_progress: 230,
    immediate_score_window: 110,
    opponent_immediate_score_window: 300,
    drainer_immediate_threat: -70,
    drainer_best_mana_path: 32,
    spirit_action_utility: 74,
    ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
};

const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE:
    ScoringWeights = ScoringWeights {
    score_race_path_progress: 108,
    opponent_score_race_path_progress: 260,
    immediate_score_window: 120,
    opponent_immediate_score_window: 370,
    drainer_immediate_threat: -115,
    drainer_best_mana_path: 52,
    spirit_action_utility: 78,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
};

const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE:
    ScoringWeights = ScoringWeights {
    score_race_path_progress: 118,
    opponent_score_race_path_progress: 300,
    immediate_score_window: 132,
    opponent_immediate_score_window: 420,
    drainer_immediate_threat: -145,
    drainer_best_mana_path: 66,
    spirit_action_utility: 84,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE:
    ScoringWeights = ScoringWeights {
    score_race_path_progress: 185,
    opponent_score_race_path_progress: 190,
    immediate_score_window: 300,
    opponent_immediate_score_window: 280,
    spirit_action_utility: 72,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE:
    ScoringWeights = ScoringWeights {
    score_race_path_progress: 210,
    opponent_score_race_path_progress: 205,
    immediate_score_window: 360,
    opponent_immediate_score_window: 320,
    spirit_action_utility: 76,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
};

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_PRESSURE:
    ScoringWeights = ScoringWeights {
    next_turn_window_scale_bp: 8_000,
    ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
};

const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_PRESSURE:
    ScoringWeights = ScoringWeights {
    next_turn_window_scale_bp: 8_000,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
};

const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_PRESSURE:
    ScoringWeights = ScoringWeights {
    next_turn_window_scale_bp: 8_000,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_PRESSURE:
    ScoringWeights = ScoringWeights {
    next_turn_window_scale_bp: 8_000,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_PRESSURE:
    ScoringWeights = ScoringWeights {
    next_turn_window_scale_bp: 8_000,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
};

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_NO_CONFIRMED_SQUARE:
    ScoringWeights = ScoringWeights {
    double_confirmed_score: false,
    ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_NO_CONFIRMED_SQUARE:
    ScoringWeights = ScoringWeights {
    double_confirmed_score: false,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_NO_CONFIRMED_SQUARE:
    ScoringWeights = ScoringWeights {
    double_confirmed_score: false,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_NO_CONFIRMED_SQUARE:
    ScoringWeights = ScoringWeights {
    double_confirmed_score: false,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_NO_CONFIRMED_SQUARE:
    ScoringWeights = ScoringWeights {
    double_confirmed_score: false,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MATCH_POINT_WINDOW:
    ScoringWeights = ScoringWeights {
    include_match_point_window: true,
    ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MATCH_POINT_WINDOW:
    ScoringWeights = ScoringWeights {
    include_match_point_window: true,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MATCH_POINT_WINDOW:
    ScoringWeights = ScoringWeights {
    include_match_point_window: true,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MATCH_POINT_WINDOW:
    ScoringWeights = ScoringWeights {
    include_match_point_window: true,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MATCH_POINT_WINDOW:
    ScoringWeights = ScoringWeights {
    include_match_point_window: true,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_GLOBAL_PRESSURE: ScoringWeights =
    ScoringWeights {
        include_regular_mana_move_windows: true,
        include_match_point_window: true,
        next_turn_window_scale_bp: 8_000,
        score_race_path_progress: 180,
        opponent_score_race_path_progress: 200,
        immediate_score_window: 280,
        opponent_immediate_score_window: 300,
        drainer_immediate_threat: -260,
        spirit_action_utility: 62,
        immediate_winning_carrier: 700,
        ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_GLOBAL_PRESSURE: ScoringWeights =
    ScoringWeights {
        include_match_point_window: true,
        next_turn_window_scale_bp: 8_000,
        ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE
    };

const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_GLOBAL_PRESSURE: ScoringWeights =
    ScoringWeights {
        include_match_point_window: true,
        next_turn_window_scale_bp: 8_000,
        ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE
    };

const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_GLOBAL_PRESSURE:
    ScoringWeights = ScoringWeights {
    include_match_point_window: true,
    next_turn_window_scale_bp: 8_000,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_GLOBAL_PRESSURE:
    ScoringWeights = ScoringWeights {
    include_match_point_window: true,
    next_turn_window_scale_bp: 8_000,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_GLOBAL_PRESSURE:
    ScoringWeights = ScoringWeights {
    include_match_point_window: true,
    next_turn_window_scale_bp: 8_000,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE
};

const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_MULTI_LANE_LIGHT: ScoringWeights =
    ScoringWeights {
        score_race_multi_path: 90,
        opponent_score_race_multi_path: 130,
        immediate_score_multi_window: 120,
        opponent_immediate_score_multi_window: 170,
        ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_LIGHT:
    ScoringWeights = ScoringWeights {
    score_race_multi_path: 80,
    opponent_score_race_multi_path: 150,
    immediate_score_multi_window: 95,
    opponent_immediate_score_multi_window: 170,
    ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_LIGHT:
    ScoringWeights = ScoringWeights {
    score_race_multi_path: 95,
    opponent_score_race_multi_path: 185,
    immediate_score_multi_window: 120,
    opponent_immediate_score_multi_window: 230,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_LIGHT:
    ScoringWeights = ScoringWeights {
    score_race_multi_path: 105,
    opponent_score_race_multi_path: 215,
    immediate_score_multi_window: 145,
    opponent_immediate_score_multi_window: 265,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_LIGHT:
    ScoringWeights = ScoringWeights {
    score_race_multi_path: 125,
    opponent_score_race_multi_path: 155,
    immediate_score_multi_window: 175,
    opponent_immediate_score_multi_window: 185,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_LIGHT:
    ScoringWeights = ScoringWeights {
    score_race_multi_path: 145,
    opponent_score_race_multi_path: 175,
    immediate_score_multi_window: 205,
    opponent_immediate_score_multi_window: 215,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_MULTI_LANE_AGGR: ScoringWeights =
    ScoringWeights {
        score_race_multi_path: 130,
        opponent_score_race_multi_path: 180,
        immediate_score_multi_window: 185,
        opponent_immediate_score_multi_window: 245,
        ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
    };

const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_MULTI_LANE_MIXED: ScoringWeights =
    ScoringWeights {
        score_race_multi_path: 120,
        opponent_score_race_multi_path: 170,
        immediate_score_multi_window: 170,
        opponent_immediate_score_multi_window: 230,
        ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_AGGR: ScoringWeights =
    ScoringWeights {
        score_race_multi_path: 120,
        opponent_score_race_multi_path: 220,
        immediate_score_multi_window: 150,
        opponent_immediate_score_multi_window: 270,
        ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_AGGR: ScoringWeights =
    ScoringWeights {
        score_race_multi_path: 145,
        opponent_score_race_multi_path: 270,
        immediate_score_multi_window: 190,
        opponent_immediate_score_multi_window: 360,
        ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_AGGR:
    ScoringWeights = ScoringWeights {
    score_race_multi_path: 170,
    opponent_score_race_multi_path: 315,
    immediate_score_multi_window: 230,
    opponent_immediate_score_multi_window: 420,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_AGGR:
    ScoringWeights = ScoringWeights {
    score_race_multi_path: 180,
    opponent_score_race_multi_path: 200,
    immediate_score_multi_window: 260,
    opponent_immediate_score_multi_window: 250,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_AGGR:
    ScoringWeights = ScoringWeights {
    score_race_multi_path: 220,
    opponent_score_race_multi_path: 240,
    immediate_score_multi_window: 310,
    opponent_immediate_score_multi_window: 290,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
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
    MonsGameModel::smart_search_best_inputs(
        game,
        MonsGameModel::with_runtime_scoring_weights(game, config),
    )
}

fn model_runtime_pre_efficiency_logic(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_root_efficiency = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_root_reply_floor(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    // Root reply-floor re-rank was removed from runtime search cleanup.
    model_current_best(game, config)
}

fn model_runtime_pre_event_ordering(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_event_ordering_bonus = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_backtrack_penalty(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_backtrack_penalty = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_tt_best_child_ordering(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_tt_best_child_ordering = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_root_aspiration(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_root_aspiration = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_two_pass_root_allocation(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_two_pass_root_allocation = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_selective_extensions(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_selective_extensions = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_quiet_reductions(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_quiet_reductions = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_root_mana_handoff_guard(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_root_mana_handoff_guard = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn runtime_pre_horizon_phase_weights(game: &MonsGame, depth: usize) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_PRE_HORIZON_FAST_CONTEXT_SCORING_WEIGHTS;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_PRE_HORIZON_NORMAL_FINISHER_AGGR_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_PRE_HORIZON_NORMAL_TACTICAL_AGGR_SCORING_WEIGHTS
    } else if my_distance_to_win <= 2 {
        &RUNTIME_PRE_HORIZON_NORMAL_FINISHER_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_PRE_HORIZON_NORMAL_TACTICAL_SCORING_WEIGHTS
    } else {
        &RUNTIME_PRE_HORIZON_NORMAL_BALANCED_SCORING_WEIGHTS
    }
}

fn runtime_phase_adaptive_scoring_weights_pre_eval_v2(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_PRE_EVAL_V2;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_PRE_EVAL_V2
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_PRE_EVAL_V2
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_PRE_EVAL_V2
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_PRE_EVAL_V2
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_PRE_EVAL_V2
    }
}

fn runtime_phase_adaptive_scoring_weights_mana_window(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW
    }
}

fn runtime_phase_adaptive_scoring_weights_mana_window_defense(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_DEFENSE
    }
}

fn runtime_phase_adaptive_scoring_weights_mana_window_pressure(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_PRESSURE
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_PRESSURE
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_PRESSURE
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_PRESSURE
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MANA_WINDOW_PRESSURE
    }
}

fn runtime_phase_adaptive_scoring_weights_no_confirmed_square(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_NO_CONFIRMED_SQUARE
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_NO_CONFIRMED_SQUARE
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_NO_CONFIRMED_SQUARE
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_NO_CONFIRMED_SQUARE
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_NO_CONFIRMED_SQUARE
    }
}

fn runtime_phase_adaptive_scoring_weights_match_point_window(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MATCH_POINT_WINDOW
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MATCH_POINT_WINDOW
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MATCH_POINT_WINDOW
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MATCH_POINT_WINDOW
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MATCH_POINT_WINDOW
    }
}

fn runtime_phase_adaptive_scoring_weights_global_pressure(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_GLOBAL_PRESSURE;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_GLOBAL_PRESSURE
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_GLOBAL_PRESSURE
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_GLOBAL_PRESSURE
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_GLOBAL_PRESSURE
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_GLOBAL_PRESSURE
    }
}

fn runtime_phase_adaptive_scoring_weights_multi_lane_light(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_MULTI_LANE_LIGHT;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_LIGHT
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_LIGHT
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_LIGHT
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_LIGHT
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_LIGHT
    }
}

fn runtime_phase_adaptive_scoring_weights_multi_lane_aggr(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_MULTI_LANE_AGGR;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_AGGR
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_AGGR
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_AGGR
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_AGGR
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_MULTI_LANE_AGGR
    }
}

fn runtime_phase_adaptive_scoring_weights_multi_lane_normal_only(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
    }

    runtime_phase_adaptive_scoring_weights_multi_lane_aggr(game, depth)
}

fn runtime_phase_adaptive_scoring_weights_multi_lane_mixed(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_MULTI_LANE_MIXED;
    }

    runtime_phase_adaptive_scoring_weights_multi_lane_aggr(game, depth)
}

fn env_tuned_eval_weights(base: ScoringWeights, fast_mode: bool) -> ScoringWeights {
    let mut tuned = base;
    let prefix = if fast_mode { "FAST" } else { "NORMAL" };

    let score_multi = env_i32(format!("SMART_EVAL_{}_SCORE_MULTI", prefix).as_str())
        .unwrap_or(tuned.score_race_multi_path);
    let opponent_score_multi = env_i32(format!("SMART_EVAL_{}_OPP_SCORE_MULTI", prefix).as_str())
        .unwrap_or(tuned.opponent_score_race_multi_path);
    let immediate_multi = env_i32(format!("SMART_EVAL_{}_IMM_MULTI", prefix).as_str())
        .unwrap_or(tuned.immediate_score_multi_window);
    let opponent_immediate_multi = env_i32(format!("SMART_EVAL_{}_OPP_IMM_MULTI", prefix).as_str())
        .unwrap_or(tuned.opponent_immediate_score_multi_window);
    let immediate_delta = env_i32(format!("SMART_EVAL_{}_IMM_DELTA", prefix).as_str()).unwrap_or(0);
    let opponent_immediate_delta =
        env_i32(format!("SMART_EVAL_{}_OPP_IMM_DELTA", prefix).as_str()).unwrap_or(0);
    let next_turn_bp = env_i32(format!("SMART_EVAL_{}_NEXT_TURN_BP", prefix).as_str())
        .unwrap_or(tuned.next_turn_window_scale_bp);

    tuned.score_race_multi_path = score_multi.max(0);
    tuned.opponent_score_race_multi_path = opponent_score_multi.max(0);
    tuned.immediate_score_multi_window = immediate_multi.max(0);
    tuned.opponent_immediate_score_multi_window = opponent_immediate_multi.max(0);
    tuned.immediate_score_window = (tuned.immediate_score_window + immediate_delta).max(0);
    tuned.opponent_immediate_score_window =
        (tuned.opponent_immediate_score_window + opponent_immediate_delta).max(0);
    tuned.next_turn_window_scale_bp = next_turn_bp.clamp(0, 20_000);
    tuned
}

fn runtime_eval_env_fast_weights() -> &'static ScoringWeights {
    static FAST: OnceLock<ScoringWeights> = OnceLock::new();
    FAST.get_or_init(|| env_tuned_eval_weights(RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS, true))
}

fn runtime_eval_env_normal_balanced_weights() -> &'static ScoringWeights {
    static W: OnceLock<ScoringWeights> = OnceLock::new();
    W.get_or_init(|| {
        env_tuned_eval_weights(
            RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS,
            false,
        )
    })
}

fn runtime_eval_env_normal_tactical_weights() -> &'static ScoringWeights {
    static W: OnceLock<ScoringWeights> = OnceLock::new();
    W.get_or_init(|| {
        env_tuned_eval_weights(
            RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS,
            false,
        )
    })
}

fn runtime_eval_env_normal_tactical_aggr_weights() -> &'static ScoringWeights {
    static W: OnceLock<ScoringWeights> = OnceLock::new();
    W.get_or_init(|| {
        env_tuned_eval_weights(
            RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS,
            false,
        )
    })
}

fn runtime_eval_env_normal_finisher_weights() -> &'static ScoringWeights {
    static W: OnceLock<ScoringWeights> = OnceLock::new();
    W.get_or_init(|| {
        env_tuned_eval_weights(
            RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS,
            false,
        )
    })
}

fn runtime_eval_env_normal_finisher_aggr_weights() -> &'static ScoringWeights {
    static W: OnceLock<ScoringWeights> = OnceLock::new();
    W.get_or_init(|| {
        env_tuned_eval_weights(
            RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS,
            false,
        )
    })
}

fn runtime_phase_adaptive_scoring_weights_env_multi(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return runtime_eval_env_fast_weights();
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        runtime_eval_env_normal_finisher_aggr_weights()
    } else if opponent_distance_to_win <= 1 {
        runtime_eval_env_normal_tactical_aggr_weights()
    } else if my_distance_to_win <= 2 {
        runtime_eval_env_normal_finisher_weights()
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        runtime_eval_env_normal_tactical_weights()
    } else {
        runtime_eval_env_normal_balanced_weights()
    }
}

fn model_runtime_pre_root_reply_risk_guard(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_root_reply_risk_guard = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_normal_reply_risk_cleanup(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth >= 3 {
        runtime.enable_root_reply_risk_guard = true;
        runtime.root_reply_risk_score_margin = SMART_ROOT_REPLY_RISK_SCORE_MARGIN;
        runtime.root_reply_risk_shortlist_max = SMART_ROOT_REPLY_RISK_SHORTLIST_NORMAL;
        runtime.root_reply_risk_reply_limit = SMART_ROOT_REPLY_RISK_REPLY_LIMIT_NORMAL;
        runtime.root_reply_risk_node_share_bp = SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_NORMAL;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_move_class_coverage(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_move_class_coverage = false;
    runtime.enable_child_move_class_coverage = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_horizon_eval(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights = runtime_pre_horizon_phase_weights(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_eval_formula_v2(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_pre_eval_v2(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_fast_multi_lane_eval(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth < 3 {
        runtime.scoring_weights = &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_PRE_MULTI_LANE;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_mana_window(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_mana_window(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_mana_window_defense(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_mana_window_defense(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_mana_window_pressure(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_mana_window_pressure(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_no_confirmed_square(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_no_confirmed_square(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_match_point_window(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_match_point_window(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_global_pressure(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_global_pressure(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_multi_lane_light(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_multi_lane_light(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_multi_lane_aggr(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_multi_lane_aggr(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_multi_lane_normal_only(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_multi_lane_normal_only(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_multi_lane_mixed(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights =
        runtime_phase_adaptive_scoring_weights_multi_lane_mixed(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_env_multi(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights = runtime_phase_adaptive_scoring_weights_env_multi(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_BOARD_V1: ScoringWeights = ScoringWeights {
    spirit_action_utility: 61,
    ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
};

const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_BOARD_V3: ScoringWeights = ScoringWeights {
    immediate_score_window: 240,
    opponent_immediate_score_window: 220,
    immediate_score_multi_window: 80,
    opponent_immediate_score_multi_window: 120,
    next_turn_window_scale_bp: 5000,
    include_regular_mana_move_windows: false,
    include_match_point_window: false,
    double_confirmed_score: true,
    spirit_action_utility: 56,
    drainer_immediate_threat: -220,
    ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
};

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_BOARD_V1: ScoringWeights =
    ScoringWeights {
        spirit_action_utility: 63,
        ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_BOARD_V3: ScoringWeights =
    ScoringWeights {
        immediate_score_window: 96,
        opponent_immediate_score_window: 245,
        next_turn_window_scale_bp: 5000,
        include_regular_mana_move_windows: true,
        include_match_point_window: false,
        double_confirmed_score: true,
        spirit_action_utility: 45,
        drainer_immediate_threat: -55,
        ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_BOARD_V3: ScoringWeights =
    ScoringWeights {
        immediate_score_window: 102,
        opponent_immediate_score_window: 310,
        ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
    };

const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_BOARD_V3: ScoringWeights =
    ScoringWeights {
        immediate_score_window: 275,
        opponent_immediate_score_window: 235,
        ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
    };

fn runtime_phase_adaptive_scoring_weights_board_v1(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_BOARD_V1;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_BOARD_V1
    }
}

fn model_runtime_eval_board_v1(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights = runtime_phase_adaptive_scoring_weights_board_v1(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_eval_board_v2(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_runtime_eval_multi_lane_aggr(game, config)
}

fn runtime_phase_adaptive_scoring_weights_board_v3(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    if depth < 3 {
        return &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_BOARD_V3;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
    } else if my_distance_to_win <= 2 {
        &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS_BOARD_V3
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS_BOARD_V3
    } else {
        &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS_BOARD_V3
    }
}

fn model_runtime_eval_board_v3(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights = runtime_phase_adaptive_scoring_weights_board_v3(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_normal_guarded_lookahead(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth >= 3 {
        runtime.enable_selective_extensions = false;
        runtime.enable_two_pass_root_allocation = false;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_search_upgrade_bundle(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_tt_best_child_ordering = false;
    runtime.enable_root_aspiration = false;
    runtime.enable_two_pass_root_allocation = false;
    runtime.enable_quiet_reductions = false;
    runtime.enable_selective_extensions = false;
    runtime.enable_root_mana_handoff_guard = false;
    runtime.enable_root_reply_risk_guard = false;
    runtime.enable_move_class_coverage = false;
    runtime.enable_child_move_class_coverage = false;
    runtime.scoring_weights = runtime_pre_horizon_phase_weights(game, runtime.depth);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_fast_efficiency_cleanup(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth < 3 {
        runtime.enable_root_efficiency = true;
        runtime.enable_backtrack_penalty = true;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_drainer_tactical_requirements(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_forced_drainer_attack = false;
    runtime.enable_forced_drainer_attack_fallback = false;
    runtime.enable_root_drainer_safety_prefilter = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_forced_drainer_attack_fallback(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_forced_drainer_attack_fallback = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_forced_tactical_prepass(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_forced_tactical_prepass = true;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_phase1_root_quality_bundle(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_forced_tactical_prepass = true;
    if runtime.depth < 3 {
        runtime.enable_move_class_coverage = false;
        runtime.enable_child_move_class_coverage = false;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_spirit_development_pref(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_root_spirit_development_pref = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_normal_root_safety(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_normal_root_safety_rerank = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_normal_root_safety_deep_floor(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_normal_root_safety_deep_floor = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_normal_spirit_base_penalty(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth >= 3 {
        runtime.scoring_weights = runtime_normal_phase_adaptive_weights(game);
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_normal_phase_deeper_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return model_current_best(game, config);
    }

    let mut runtime = SmartSearchConfig::from_budget(
        SMART_AUTOMOVE_NORMAL_DEPTH,
        SMART_AUTOMOVE_NORMAL_MAX_VISITED_NODES,
    )
    .for_runtime()
    .with_normal_deeper_shape();
    runtime.scoring_weights = &RUNTIME_RUSH_SCORING_WEIGHTS;
    runtime.enable_root_efficiency = false;
    runtime.enable_event_ordering_bonus = false;
    runtime.enable_backtrack_penalty = false;
    runtime.enable_normal_root_safety_rerank = true;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn candidate_model_runtime_normal_efficiency_reply_floor(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth >= 3 {
        runtime.enable_root_efficiency = true;
        runtime.enable_backtrack_penalty = true;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_normal_efficiency_reply_floor(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth >= 3 {
        runtime.enable_root_efficiency = false;
        runtime.enable_backtrack_penalty = false;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_root_upgrade_bundle(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.enable_event_ordering_bonus = false;
    runtime.enable_backtrack_penalty = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn candidate_model_runtime_normal_spirit_base_strict(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights = runtime_normal_phase_weights_spirit_base_strict(game);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn candidate_model_runtime_normal_reinvest_search(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth >= 3 {
        runtime.max_visited_nodes = (runtime.max_visited_nodes * 11 / 10)
            .clamp(runtime.max_visited_nodes, MAX_SMART_MAX_VISITED_NODES);
        runtime.enable_normal_root_safety_deep_floor = false;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn candidate_model_runtime_normal_confirmed_850(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights = runtime_normal_phase_weights_confirmed_850(game);
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_move_efficiency(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = if config.depth >= 3 {
        SmartSearchConfig::from_budget(
            SMART_AUTOMOVE_NORMAL_DEPTH,
            LEGACY_RUNTIME_NORMAL_MAX_VISITED_NODES,
        )
        .for_runtime()
        .with_normal_deeper_shape()
    } else {
        SmartSearchConfig::from_budget(
            SMART_AUTOMOVE_FAST_DEPTH,
            LEGACY_RUNTIME_FAST_MAX_VISITED_NODES,
        )
        .for_runtime()
        .with_fast_wideroot_shape()
    };
    runtime.scoring_weights = &RUNTIME_RUSH_SCORING_WEIGHTS;
    runtime.enable_root_efficiency = false;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_current_best_legacy_no_transposition(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs_legacy_no_transposition(
        game,
        MonsGameModel::with_runtime_scoring_weights(game, config),
    )
}

fn model_runtime_legacy_phase_adaptive(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut legacy =
        SmartSearchConfig::from_budget(config.depth as i32, config.max_visited_nodes as i32)
            .for_runtime();
    if legacy.depth < 3 {
        legacy = tuned_candidate_config_wideroot(legacy);
    }
    legacy.scoring_weights = legacy_runtime_phase_adaptive_scoring_weights(game, legacy.depth);
    MonsGameModel::smart_search_best_inputs(game, legacy)
}

fn model_runtime_pre_drainer_context(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime =
        SmartSearchConfig::from_budget(config.depth as i32, config.max_visited_nodes as i32)
            .for_runtime();
    if runtime.depth < 3 {
        runtime = runtime.with_fast_wideroot_shape();
        runtime.scoring_weights = phase_adaptive_scoring_v2_weights(game, false);
    } else {
        runtime.scoring_weights = &TACTICAL_BALANCED_SCORING_WEIGHTS;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_tactical_runtime(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = config;
    if runtime.depth >= 3 {
        runtime.scoring_weights =
            legacy_runtime_phase_adaptive_scoring_weights(game, runtime.depth);
    } else {
        runtime.scoring_weights = &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_winloss_weights(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = config;
    if runtime.depth >= 3 {
        runtime.scoring_weights = &TACTICAL_BALANCED_SCORING_WEIGHTS;
    } else {
        runtime.scoring_weights = &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS;
    }
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn model_runtime_pre_fast_drainer_priority(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        MonsGameModel::with_runtime_scoring_weights(game, config)
    } else {
        with_scoring_weights(config, &RUNTIME_RUSH_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn model_runtime_pre_normal_x15(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    if config.depth < 3 {
        return model_current_best(game, config);
    }

    let mut runtime = SmartSearchConfig::from_budget(
        SMART_AUTOMOVE_NORMAL_DEPTH,
        LEGACY_NORMAL_MAX_VISITED_NODES,
    )
    .for_runtime();
    runtime.scoring_weights = &TACTICAL_BALANCED_SCORING_WEIGHTS;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

fn legacy_runtime_phase_adaptive_scoring_weights(
    game: &MonsGame,
    depth: usize,
) -> &'static ScoringWeights {
    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if depth >= 3 {
        if my_distance_to_win <= 1 {
            &FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &FINISHER_BALANCED_SOFT_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &TACTICAL_BALANCED_SCORING_WEIGHTS
        } else {
            &BALANCED_DISTANCE_SCORING_WEIGHTS
        }
    } else if my_distance_to_win <= 1 {
        &FINISHER_MANA_RACE_LITE_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &TACTICAL_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS
    } else if my_distance_to_win <= 2 {
        &FINISHER_MANA_RACE_LITE_SOFT_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS
    } else if score_gap >= 2 {
        &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS
    } else {
        &MANA_RACE_LITE_SCORING_WEIGHTS
    }
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
    model_current_best(game, config)
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

fn candidate_model_runtime_d2_tuned_d3_tactical(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &TACTICAL_BALANCED_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_d2_tuned_d3_winloss(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &RUNTIME_NORMAL_WINLOSS_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_d2_tuned_d3_tactical_phase(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, normal_tactical_phase_weights(game))
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_d2_tuned_d3_tactical_aggr(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_d2_tuned_d3_finisher_soft_aggr(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_d2_tuned_d3_mana_neutral(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &MANA_RACE_NEUTRAL_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_d2_tuned_d3_adaptive_neutral(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, d3_adaptive_neutral_weights(game))
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_d2_tuned_d3_phase_adaptive(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, d3_phase_adaptive_weights(game))
    } else {
        with_scoring_weights(config, &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_fast_phase_normal_tactical(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, normal_tactical_phase_weights(game))
    } else {
        with_scoring_weights(config, phase_adaptive_scoring_v2_weights(game, false))
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

#[derive(Debug, Clone)]
struct GuardedRootEvaluation {
    score: i32,
    inputs: Vec<Input>,
    game: MonsGame,
}

fn candidate_model_runtime_root_safety_tiebreak(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let perspective = game.active_color;
    let mut scored_roots = search_scored_roots_with_states(game, config, perspective);
    if scored_roots.is_empty() {
        return Vec::new();
    }

    scored_roots.sort_by(|a, b| b.score.cmp(&a.score));
    let best_score = scored_roots[0].score;
    let score_margin = if config.depth >= 3 { 700 } else { 500 };
    let shortlist_limit = if config.depth >= 3 { 3 } else { 2 };

    let mut shortlist: Vec<GuardedRootEvaluation> = scored_roots
        .into_iter()
        .take_while(|root| root.score + score_margin >= best_score)
        .take(shortlist_limit)
        .collect();

    if shortlist.len() <= 1 {
        return shortlist.remove(0).inputs;
    }

    let reply_limit = if config.depth >= 3 {
        config.node_enum_limit.clamp(4, 8)
    } else {
        config.node_enum_limit.clamp(4, 6)
    };

    let mut best_index = 0usize;
    let mut best_floor = i32::MIN;
    let mut best_root_score = i32::MIN;
    for (index, root) in shortlist.iter().enumerate() {
        let reply_floor =
            root_reply_floor(&root.game, perspective, config.scoring_weights, reply_limit);
        if reply_floor > best_floor || (reply_floor == best_floor && root.score > best_root_score) {
            best_floor = reply_floor;
            best_root_score = root.score;
            best_index = index;
        }
    }

    shortlist[best_index].inputs.clone()
}

fn search_scored_roots_with_states(
    game: &MonsGame,
    config: SmartSearchConfig,
    perspective: Color,
) -> Vec<GuardedRootEvaluation> {
    let root_moves = MonsGameModel::ranked_root_moves(game, perspective, config);
    if root_moves.is_empty() {
        return Vec::new();
    }

    let mut visited_nodes = 0usize;
    let mut alpha = i32::MIN;
    let mut scored_roots = Vec::with_capacity(root_moves.len());
    let mut transposition_table = std::collections::HashMap::new();
    let extension_node_budget = if config.enable_selective_extensions
        && config.selective_extension_node_share_bp > 0
    {
        ((config.max_visited_nodes * config.selective_extension_node_share_bp as usize) / 10_000)
            .max(1)
    } else {
        0
    };
    let mut extension_nodes_used = 0usize;

    for candidate in root_moves {
        if visited_nodes >= config.max_visited_nodes {
            break;
        }

        visited_nodes += 1;
        let candidate_score = if config.depth > 1 {
            MonsGameModel::search_score(
                &candidate.game,
                perspective,
                config.depth - 1,
                alpha,
                i32::MAX,
                &mut visited_nodes,
                config,
                &mut transposition_table,
                config.max_extensions_per_path,
                &mut extension_nodes_used,
                extension_node_budget,
                true,
            )
        } else {
            candidate.heuristic
        };

        if candidate_score > alpha {
            alpha = candidate_score;
        }

        scored_roots.push(GuardedRootEvaluation {
            score: candidate_score,
            inputs: candidate.inputs,
            game: candidate.game,
        });
    }

    scored_roots
}

fn root_reply_floor(
    state_after_move: &MonsGame,
    perspective: Color,
    scoring_weights: &'static ScoringWeights,
    reply_limit: usize,
) -> i32 {
    if let Some(winner) = state_after_move.winner_color() {
        return if winner == perspective {
            SMART_TERMINAL_SCORE / 2
        } else {
            -SMART_TERMINAL_SCORE / 2
        };
    }

    if state_after_move.active_color == perspective {
        return evaluate_preferability_with_weights(state_after_move, perspective, scoring_weights);
    }

    let replies = MonsGameModel::enumerate_legal_inputs(state_after_move, reply_limit.max(1));
    if replies.is_empty() {
        return SMART_TERMINAL_SCORE / 4;
    }

    let mut worst_reply_score = i32::MAX;
    for reply in replies {
        let Some(after_reply) = MonsGameModel::apply_inputs_for_search(state_after_move, &reply)
        else {
            continue;
        };

        let score = match after_reply.winner_color() {
            Some(winner) if winner == perspective => SMART_TERMINAL_SCORE / 2,
            Some(_) => -SMART_TERMINAL_SCORE / 2,
            None => evaluate_preferability_with_weights(&after_reply, perspective, scoring_weights),
        };
        worst_reply_score = worst_reply_score.min(score);
    }

    if worst_reply_score == i32::MAX {
        evaluate_preferability_with_weights(state_after_move, perspective, scoring_weights)
    } else {
        worst_reply_score
    }
}

fn candidate_model_runtime_d2_tuned_normal_reply_guard(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_runtime_d2_tuned(game, config);
    }

    let perspective = game.active_color;
    let scoring_weights = &BALANCED_DISTANCE_SCORING_WEIGHTS;
    let mut root_candidates = Vec::new();
    push_unique_candidate(
        &mut root_candidates,
        candidate_model_runtime_d2_tuned(game, config),
    );
    push_unique_candidate(
        &mut root_candidates,
        candidate_model_runtime_d2_tuned_d3_tactical(game, config),
    );
    push_unique_candidate(
        &mut root_candidates,
        candidate_model_runtime_d2_tuned_d3_finisher_soft_aggr(game, config),
    );

    if root_candidates.is_empty() {
        return Vec::new();
    }

    let mut best_inputs = root_candidates[0].clone();
    let mut best_score = i32::MIN;

    for inputs in root_candidates {
        let Some(after_move) = MonsGameModel::apply_inputs_for_search(game, &inputs) else {
            continue;
        };

        let optimistic =
            evaluate_preferability_with_weights(&after_move, perspective, scoring_weights);
        let reply_floor = reply_guard_lite_floor(&after_move, perspective, config, scoring_weights);
        let combined_score = reply_floor.saturating_mul(3).saturating_add(optimistic);

        if combined_score > best_score {
            best_score = combined_score;
            best_inputs = inputs;
        }
    }

    if best_score == i32::MIN {
        candidate_model_runtime_d2_tuned(game, config)
    } else {
        best_inputs
    }
}

fn reply_guard_lite_floor(
    state_after_move: &MonsGame,
    perspective: Color,
    config: SmartSearchConfig,
    scoring_weights: &'static ScoringWeights,
) -> i32 {
    if let Some(winner) = state_after_move.winner_color() {
        return if winner == perspective {
            SMART_TERMINAL_SCORE / 2
        } else {
            -SMART_TERMINAL_SCORE / 2
        };
    }

    if state_after_move.active_color == perspective {
        return evaluate_preferability_with_weights(state_after_move, perspective, scoring_weights);
    }

    let reply_limit = config.node_enum_limit.clamp(6, 14);
    let replies = MonsGameModel::enumerate_legal_inputs(state_after_move, reply_limit);
    if replies.is_empty() {
        return SMART_TERMINAL_SCORE / 4;
    }

    let mut worst_reply_score = i32::MAX;
    for reply in replies {
        let Some(after_reply) = MonsGameModel::apply_inputs_for_search(state_after_move, &reply)
        else {
            continue;
        };

        let score = match after_reply.winner_color() {
            Some(winner) if winner == perspective => SMART_TERMINAL_SCORE / 2,
            Some(_) => -SMART_TERMINAL_SCORE / 2,
            None => evaluate_preferability_with_weights(&after_reply, perspective, scoring_weights),
        };
        worst_reply_score = worst_reply_score.min(score);
    }

    if worst_reply_score == i32::MAX {
        evaluate_preferability_with_weights(state_after_move, perspective, scoring_weights)
    } else {
        worst_reply_score
    }
}

fn d3_adaptive_neutral_weights(game: &MonsGame) -> &'static ScoringWeights {
    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;

    if my_score < opponent_score || opponent_distance_to_win <= 2 || my_distance_to_win <= 2 {
        &MANA_RACE_NEUTRAL_SCORING_WEIGHTS
    } else {
        &BALANCED_DISTANCE_SCORING_WEIGHTS
    }
}

fn d3_phase_adaptive_weights(game: &MonsGame) -> &'static ScoringWeights {
    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS
    } else if my_distance_to_win <= 2 {
        &FINISHER_BALANCED_SOFT_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &TACTICAL_BALANCED_SCORING_WEIGHTS
    } else {
        &BALANCED_DISTANCE_SCORING_WEIGHTS
    }
}

fn normal_tactical_phase_weights(game: &MonsGame) -> &'static ScoringWeights {
    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;

    if my_distance_to_win <= 1 {
        &FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS
    } else {
        &TACTICAL_BALANCED_SCORING_WEIGHTS
    }
}

fn candidate_model_phase_adaptive_d2(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &BALANCED_DISTANCE_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, phase_adaptive_d2_weights(game))
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn phase_adaptive_d2_weights(game: &MonsGame) -> &'static ScoringWeights {
    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;

    if my_distance_to_win <= 1 {
        &FINISHER_MANA_RACE_LITE_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &TACTICAL_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS
    } else if my_distance_to_win <= 2 || opponent_distance_to_win <= 2 {
        &FINISHER_MANA_RACE_LITE_SOFT_SCORING_WEIGHTS
    } else if opponent_score > my_score {
        &TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS
    } else {
        &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS
    }
}

fn candidate_model_phase_adaptive_scoring_v2(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let scoring_weights = phase_adaptive_scoring_v2_weights(game, config.depth >= 3);
    MonsGameModel::smart_search_best_inputs(game, with_scoring_weights(config, scoring_weights))
}

fn phase_adaptive_scoring_v2_weights(game: &MonsGame, deep_mode: bool) -> &'static ScoringWeights {
    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if deep_mode {
        if my_distance_to_win <= 1 {
            &FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &FINISHER_BALANCED_SOFT_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &TACTICAL_BALANCED_SCORING_WEIGHTS
        } else {
            &BALANCED_DISTANCE_SCORING_WEIGHTS
        }
    } else if my_distance_to_win <= 1 {
        &FINISHER_MANA_RACE_LITE_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &TACTICAL_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS
    } else if my_distance_to_win <= 2 {
        &FINISHER_MANA_RACE_LITE_SOFT_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS
    } else if score_gap >= 2 {
        &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS
    } else {
        &MANA_RACE_LITE_SCORING_WEIGHTS
    }
}

fn candidate_model_wideroot(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_wideroot(config))
}

fn candidate_model_runtime_fast_wideroot_normal_current(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        candidate_model_base(game, config)
    } else {
        MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_wideroot(config))
    }
}

fn candidate_model_runtime_fast_wideroot_lite_normal_current(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        candidate_model_base(game, config)
    } else {
        MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_wideroot_lite(config))
    }
}

fn candidate_model_runtime_drainer_priority(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &DRAINER_PRIORITY_NORMAL_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, &DRAINER_PRIORITY_FAST_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_drainer_priority_aggr(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        with_scoring_weights(config, &DRAINER_PRIORITY_NORMAL_AGGR_SCORING_WEIGHTS)
    } else {
        with_scoring_weights(config, &DRAINER_PRIORITY_FAST_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_drainer_tiebreak(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let perspective = game.active_color;
    let mut scored_roots = search_scored_roots_with_states(game, config, perspective);
    if scored_roots.is_empty() {
        return Vec::new();
    }

    scored_roots.sort_by(|a, b| b.score.cmp(&a.score));
    let best_root_score = scored_roots[0].score;
    let shortlist_limit = if config.depth >= 3 { 6 } else { 5 };
    let score_margin = if config.depth >= 3 { 120_000 } else { 90_000 };

    let mut best_inputs = scored_roots[0].inputs.clone();
    let mut best_drainer_score = i32::MIN;
    let mut best_root_tiebreak = i32::MIN;

    for root in scored_roots.iter().take(shortlist_limit) {
        if root.score + score_margin < best_root_score {
            break;
        }

        let drainer_score = drainer_priority_delta(&root.game, perspective);
        if drainer_score > best_drainer_score
            || (drainer_score == best_drainer_score && root.score > best_root_tiebreak)
        {
            best_drainer_score = drainer_score;
            best_root_tiebreak = root.score;
            best_inputs = root.inputs.clone();
        }
    }

    best_inputs
}

fn candidate_model_runtime_drainer_context(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        MonsGameModel::with_runtime_scoring_weights(game, config)
    } else {
        with_scoring_weights(config, &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_drainer_priority_fast_only(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = if config.depth >= 3 {
        MonsGameModel::with_runtime_scoring_weights(game, config)
    } else {
        with_scoring_weights(config, &RUNTIME_FAST_DRAINER_PRIORITY_SCORING_WEIGHTS)
    };
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_fast_wideroot_normal_tactical(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        candidate_model_runtime_d2_tuned_d3_tactical(game, config)
    } else {
        MonsGameModel::smart_search_best_inputs(game, tuned_candidate_config_wideroot(config))
    }
}

fn candidate_model_runtime_normal_x15_phase_deeper(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let tuned = tuned_candidate_config_runtime_normal_x15_phase_deeper(game, config);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_normal_x15_tactical(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let mut tuned = tuned_candidate_config_runtime_normal_x15_phase_deeper(game, config);
    tuned.scoring_weights = &TACTICAL_BALANCED_SCORING_WEIGHTS;
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_normal_x15_tactical_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let mut tuned =
        SmartSearchConfig::from_budget(config.depth as i32, config.max_visited_nodes as i32)
            .for_runtime();
    tuned.root_branch_limit = tuned.root_branch_limit.saturating_sub(4).clamp(8, 36);
    tuned.node_branch_limit = (tuned.node_branch_limit + 2).clamp(8, 18);
    tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 220);
    tuned.node_enum_limit = (tuned.node_branch_limit * 5).clamp(tuned.node_branch_limit, 120);
    tuned.scoring_weights = &TACTICAL_BALANCED_SCORING_WEIGHTS;
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn tuned_fast_reply_risk_guard_config(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth < 3 {
        runtime.enable_root_reply_risk_guard = true;
        runtime.root_reply_risk_score_margin = SMART_ROOT_REPLY_RISK_SCORE_MARGIN;
        runtime.root_reply_risk_shortlist_max = 2;
        runtime.root_reply_risk_reply_limit = 6;
        runtime.root_reply_risk_node_share_bp = 350;
        runtime.enable_child_move_class_coverage = true;
    }
    runtime
}

fn candidate_model_runtime_fast_reply_guard_normal_current(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = tuned_fast_reply_risk_guard_config(game, config);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_fast_reply_guard_normal_x15_tactical_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        return candidate_model_runtime_normal_x15_tactical_lite(game, config);
    }

    let tuned = tuned_fast_reply_risk_guard_config(game, config);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn tuned_fast_simplified_root_config(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth < 3 {
        runtime.enable_event_ordering_bonus = false;
        runtime.enable_backtrack_penalty = false;
        runtime.enable_move_class_coverage = false;
        runtime.enable_child_move_class_coverage = false;
        runtime.enable_root_spirit_development_pref = false;
    }
    runtime
}

fn candidate_model_runtime_fast_simplified_normal_current(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let tuned = tuned_fast_simplified_root_config(game, config);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_fast_simplified_normal_x15_tactical_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        return candidate_model_runtime_normal_x15_tactical_lite(game, config);
    }

    let tuned = tuned_fast_simplified_root_config(game, config);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_fast_no_event_ordering_normal_x15_tactical_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        return candidate_model_runtime_normal_x15_tactical_lite(game, config);
    }

    let mut tuned = MonsGameModel::with_runtime_scoring_weights(game, config);
    tuned.enable_event_ordering_bonus = false;
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn tuned_fast_eventless_reinvest_config(
    game: &MonsGame,
    config: SmartSearchConfig,
    budget_scale_bp: i32,
) -> SmartSearchConfig {
    let mut tuned = MonsGameModel::with_runtime_scoring_weights(game, config);
    if tuned.depth < 3 {
        tuned.enable_event_ordering_bonus = false;
        tuned.max_visited_nodes = ((tuned.max_visited_nodes * budget_scale_bp as usize) / 10_000)
            .clamp(tuned.max_visited_nodes, MAX_SMART_MAX_VISITED_NODES);
        let extra_root = if budget_scale_bp >= 11_500 { 2 } else { 1 };
        tuned.root_branch_limit = (tuned.root_branch_limit + extra_root).clamp(8, 44);
        tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 260);
    }
    tuned
}

fn candidate_model_runtime_fast_eventless_reinvest10_normal_x15_tactical_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        return candidate_model_runtime_normal_x15_tactical_lite(game, config);
    }

    let tuned = tuned_fast_eventless_reinvest_config(game, config, 11_000);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_fast_eventless_reinvest15_normal_x15_tactical_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        return candidate_model_runtime_normal_x15_tactical_lite(game, config);
    }

    let tuned = tuned_fast_eventless_reinvest_config(game, config, 11_500);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn tuned_fast_depth3_lite_config(max_nodes: usize, root_branch_limit: usize) -> SmartSearchConfig {
    let mut tuned = SmartSearchConfig::from_budget(3, max_nodes as i32).for_runtime();
    tuned.root_branch_limit = root_branch_limit.clamp(8, 16);
    tuned.node_branch_limit = 7;
    tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 96);
    tuned.node_enum_limit = (tuned.node_branch_limit * 4).clamp(tuned.node_branch_limit, 36);
    tuned.scoring_weights = &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
    tuned.enable_root_efficiency = true;
    tuned.enable_event_ordering_bonus = false;
    tuned.enable_backtrack_penalty = true;
    tuned.enable_tt_best_child_ordering = true;
    tuned.enable_root_aspiration = false;
    tuned.enable_two_pass_root_allocation = false;
    tuned.root_focus_k = 2;
    tuned.root_focus_budget_share_bp = 6_000;
    tuned.enable_selective_extensions = false;
    tuned.enable_quiet_reductions = true;
    tuned.max_extensions_per_path = 0;
    tuned.selective_extension_node_share_bp = 0;
    tuned.enable_root_mana_handoff_guard = false;
    tuned.enable_forced_drainer_attack = true;
    tuned.enable_forced_drainer_attack_fallback = true;
    tuned.enable_forced_tactical_prepass = true;
    tuned.enable_root_drainer_safety_prefilter = true;
    tuned.enable_root_spirit_development_pref = true;
    tuned.enable_root_reply_risk_guard = false;
    tuned.root_reply_risk_score_margin = SMART_ROOT_REPLY_RISK_SCORE_MARGIN;
    tuned.root_reply_risk_shortlist_max = SMART_ROOT_REPLY_RISK_SHORTLIST_FAST;
    tuned.root_reply_risk_reply_limit = SMART_ROOT_REPLY_RISK_REPLY_LIMIT_FAST;
    tuned.root_reply_risk_node_share_bp = SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_FAST;
    tuned.enable_move_class_coverage = true;
    tuned.enable_child_move_class_coverage = false;
    tuned.enable_strict_tactical_class_coverage = true;
    tuned.enable_strict_anti_help_filter = true;
    tuned.root_anti_help_score_margin = SMART_ROOT_ANTI_HELP_SCORE_MARGIN;
    tuned.root_anti_help_reply_limit = SMART_ROOT_ANTI_HELP_REPLY_LIMIT_FAST;
    tuned.enable_two_pass_volatility_focus = false;
    tuned.enable_normal_root_safety_rerank = false;
    tuned.enable_normal_root_safety_deep_floor = false;
    tuned
}

fn candidate_model_runtime_fast_d3_lite600_normal_x15_tactical_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        return candidate_model_runtime_normal_x15_tactical_lite(game, config);
    }

    let tuned = tuned_fast_depth3_lite_config(600, 12);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_fast_d3_lite700_normal_x15_tactical_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        return candidate_model_runtime_normal_x15_tactical_lite(game, config);
    }

    let tuned = tuned_fast_depth3_lite_config(700, 12);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_fast_env_tune_normal_x15_tactical_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth >= 3 {
        return candidate_model_runtime_normal_x15_tactical_lite(game, config);
    }

    let mut tuned = MonsGameModel::with_runtime_scoring_weights(game, config);
    let scale_bp = env_i32("SMART_FAST_TUNE_NODE_SCALE_BP")
        .unwrap_or(10_000)
        .clamp(7_000, 13_000);
    tuned.max_visited_nodes = ((tuned.max_visited_nodes * scale_bp as usize) / 10_000)
        .clamp(120, MAX_SMART_MAX_VISITED_NODES);

    let root_branch_delta = env_i32("SMART_FAST_TUNE_ROOT_BRANCH_DELTA")
        .unwrap_or(0)
        .clamp(-6, 6);
    let node_branch_delta = env_i32("SMART_FAST_TUNE_NODE_BRANCH_DELTA")
        .unwrap_or(0)
        .clamp(-6, 6);
    tuned.root_branch_limit = ((tuned.root_branch_limit as i32) + root_branch_delta)
        .clamp(6, 44) as usize;
    tuned.node_branch_limit = ((tuned.node_branch_limit as i32) + node_branch_delta)
        .clamp(4, 22) as usize;
    tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 260);
    tuned.node_enum_limit = (tuned.node_branch_limit * 4).clamp(tuned.node_branch_limit, 120);

    if let Some(value) = env_bool("SMART_FAST_TUNE_EVENT_ORDERING") {
        tuned.enable_event_ordering_bonus = value;
    }
    if let Some(value) = env_bool("SMART_FAST_TUNE_BACKTRACK") {
        tuned.enable_backtrack_penalty = value;
    }
    if let Some(value) = env_bool("SMART_FAST_TUNE_MOVE_CLASS") {
        tuned.enable_move_class_coverage = value;
    }
    if let Some(value) = env_bool("SMART_FAST_TUNE_CHILD_MOVE_CLASS") {
        tuned.enable_child_move_class_coverage = value;
    }
    if let Some(value) = env_bool("SMART_FAST_TUNE_SPIRIT_PREF") {
        tuned.enable_root_spirit_development_pref = value;
    }
    if let Some(value) = env_bool("SMART_FAST_TUNE_FORCED_PREPASS") {
        tuned.enable_forced_tactical_prepass = value;
    }
    if let Some(value) = env_bool("SMART_FAST_TUNE_REPLY_GUARD") {
        tuned.enable_root_reply_risk_guard = value;
    }
    if let Some(value) = env_usize("SMART_FAST_TUNE_REPLY_LIMIT") {
        tuned.root_reply_risk_reply_limit = value.clamp(2, 12);
    }
    if let Some(value) = env_usize("SMART_FAST_TUNE_REPLY_SHORTLIST") {
        tuned.root_reply_risk_shortlist_max = value.clamp(1, 6);
    }
    if let Some(value) = env_i32("SMART_FAST_TUNE_REPLY_MARGIN") {
        tuned.root_reply_risk_score_margin = value.clamp(50, 300);
    }
    if let Some(value) = env_i32("SMART_FAST_TUNE_REPLY_NODE_SHARE_BP") {
        tuned.root_reply_risk_node_share_bp = value.clamp(100, 1_200);
    }

    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_normal_x15_tactical_phase(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let mut tuned = tuned_candidate_config_runtime_normal_x15_phase_deeper(game, config);
    tuned.scoring_weights = normal_tactical_phase_weights(game);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_normal_x15_plain(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let mut tuned =
        SmartSearchConfig::from_budget(config.depth as i32, config.max_visited_nodes as i32)
            .for_runtime();
    tuned.scoring_weights = &TACTICAL_BALANCED_SCORING_WEIGHTS;
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_normal_x15_finisher_only(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let mut tuned =
        SmartSearchConfig::from_budget(config.depth as i32, config.max_visited_nodes as i32)
            .for_runtime();
    tuned.scoring_weights = runtime_normal_finisher_only_weights(game);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn candidate_model_runtime_normal_d4_selective(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let mut tuned =
        SmartSearchConfig::from_budget(config.depth as i32, config.max_visited_nodes as i32)
            .for_runtime();
    tuned.depth = 4;
    tuned.max_visited_nodes =
        (tuned.max_visited_nodes * 11 / 10).clamp(2600, MAX_SMART_MAX_VISITED_NODES);
    tuned.root_branch_limit = tuned.root_branch_limit.saturating_sub(11).clamp(8, 24);
    tuned.node_branch_limit = tuned.node_branch_limit.saturating_sub(6).clamp(5, 10);
    tuned.root_enum_limit = (tuned.root_branch_limit * 5).clamp(tuned.root_branch_limit, 150);
    tuned.node_enum_limit = (tuned.node_branch_limit * 3).clamp(tuned.node_branch_limit, 42);
    tuned.scoring_weights = &TACTICAL_BALANCED_SCORING_WEIGHTS;
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn runtime_normal_finisher_only_weights(game: &MonsGame) -> &'static ScoringWeights {
    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;

    if my_distance_to_win <= 1 {
        &FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS
    } else {
        &TACTICAL_BALANCED_SCORING_WEIGHTS
    }
}

fn candidate_model_runtime_normal_x15_guarded_root(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let tuned = tuned_candidate_config_runtime_normal_x15_phase_deeper(game, config);
    let perspective = game.active_color;
    let mut scored_roots = search_scored_roots_with_states(game, tuned, perspective);
    if scored_roots.is_empty() {
        return Vec::new();
    }

    scored_roots.sort_by(|a, b| b.score.cmp(&a.score));
    let best_score = scored_roots[0].score;
    let shortlist_limit = 4usize;
    let score_margin = 900i32;
    let reply_limit = tuned.node_enum_limit.clamp(6, 10);

    let mut best_inputs = scored_roots[0].inputs.clone();
    let mut best_combined_score = i64::MIN;
    for root in scored_roots.iter().take(shortlist_limit) {
        if root.score + score_margin < best_score {
            break;
        }

        let reply_floor =
            root_reply_floor(&root.game, perspective, tuned.scoring_weights, reply_limit);
        let drainer_bonus = drainer_priority_delta(&root.game, perspective) / 5;
        let combined_score = root.score as i64 + (reply_floor as i64) * 2 + drainer_bonus as i64;

        if combined_score > best_combined_score {
            best_combined_score = combined_score;
            best_inputs = root.inputs.clone();
        }
    }

    best_inputs
}

fn candidate_model_runtime_normal_x15_guarded_root_v2(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let tuned = tuned_candidate_config_runtime_normal_x15_phase_deeper(game, config);
    let perspective = game.active_color;
    let mut scored_roots = search_scored_roots_with_states(game, tuned, perspective);
    if scored_roots.is_empty() {
        return Vec::new();
    }

    scored_roots.sort_by(|a, b| b.score.cmp(&a.score));
    let best_score = scored_roots[0].score;
    let shortlist_limit = 3usize;
    let score_margin = 500i32;
    let reply_limit = tuned.node_enum_limit.clamp(4, 8);

    let mut best_inputs = scored_roots[0].inputs.clone();
    let mut best_combined_score = i64::MIN;
    for root in scored_roots.iter().take(shortlist_limit) {
        if root.score + score_margin < best_score {
            break;
        }

        let reply_floor =
            root_reply_floor(&root.game, perspective, tuned.scoring_weights, reply_limit);
        let combined_score = root.score as i64 + (reply_floor as i64) * 4;
        if combined_score > best_combined_score {
            best_combined_score = combined_score;
            best_inputs = root.inputs.clone();
        }
    }

    best_inputs
}

fn tuned_candidate_config_runtime_normal_x15_phase_deeper(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut tuned = config;
    tuned.max_visited_nodes = (config.max_visited_nodes * 3 / 2)
        .clamp(config.max_visited_nodes, MAX_SMART_MAX_VISITED_NODES);
    tuned.root_branch_limit = config.root_branch_limit.clamp(8, 36);
    tuned.node_branch_limit = (config.node_branch_limit + 3).clamp(9, 18);
    tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 220);
    tuned.node_enum_limit = (tuned.node_branch_limit * 6).clamp(tuned.node_branch_limit, 132);
    tuned.scoring_weights = runtime_normal_phase_adaptive_weights(game);
    tuned
}

fn tuned_candidate_config_runtime_normal_x15_phase_deeper_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut tuned = config;
    tuned.max_visited_nodes = (config.max_visited_nodes * 3 / 2)
        .clamp(config.max_visited_nodes, MAX_SMART_MAX_VISITED_NODES);
    tuned.root_branch_limit = config.root_branch_limit.saturating_sub(4).clamp(8, 36);
    tuned.node_branch_limit = (config.node_branch_limit + 2).clamp(8, 18);
    tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 220);
    tuned.node_enum_limit = (tuned.node_branch_limit * 5).clamp(tuned.node_branch_limit, 120);
    tuned.scoring_weights = runtime_normal_phase_adaptive_weights(game);
    tuned
}

fn candidate_model_runtime_normal_x15_phase_deeper_lite(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    if config.depth < 3 {
        return candidate_model_base(game, config);
    }

    let tuned = tuned_candidate_config_runtime_normal_x15_phase_deeper_lite(game, config);
    MonsGameModel::smart_search_best_inputs(game, tuned)
}

fn runtime_normal_phase_adaptive_weights(game: &MonsGame) -> &'static ScoringWeights {
    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS
    } else if my_distance_to_win <= 2 {
        &FINISHER_BALANCED_SOFT_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &TACTICAL_BALANCED_SCORING_WEIGHTS
    } else {
        &BALANCED_DISTANCE_SCORING_WEIGHTS
    }
}

fn runtime_normal_phase_weights_spirit_base_strict(game: &MonsGame) -> &'static ScoringWeights {
    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_STRICT_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_STRICT_SCORING_WEIGHTS
    } else if my_distance_to_win <= 2 {
        &FINISHER_BALANCED_SOFT_SPIRIT_BASE_STRICT_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &TACTICAL_BALANCED_SPIRIT_BASE_STRICT_SCORING_WEIGHTS
    } else {
        &BALANCED_DISTANCE_SPIRIT_BASE_STRICT_SCORING_WEIGHTS
    }
}

fn runtime_normal_phase_weights_confirmed_850(game: &MonsGame) -> &'static ScoringWeights {
    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        &FINISHER_BALANCED_SOFT_AGGRESSIVE_CONFIRMED_850_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 1 {
        &TACTICAL_BALANCED_AGGRESSIVE_CONFIRMED_850_SCORING_WEIGHTS
    } else if my_distance_to_win <= 2 {
        &FINISHER_BALANCED_SOFT_CONFIRMED_850_SCORING_WEIGHTS
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        &TACTICAL_BALANCED_CONFIRMED_850_SCORING_WEIGHTS
    } else {
        &BALANCED_DISTANCE_CONFIRMED_850_SCORING_WEIGHTS
    }
}

fn drainer_priority_delta(game: &MonsGame, perspective: Color) -> i32 {
    let mut delta = 0i32;
    for (&location, item) in &game.board.items {
        let (mon, carried_mana) = match item {
            Item::Mon { mon } => (mon, None),
            Item::MonWithMana { mon, mana } => (mon, Some(*mana)),
            Item::MonWithConsumable { mon, .. } => (mon, None),
            Item::Mana { .. } | Item::Consumable { .. } => continue,
        };

        if mon.kind != MonKind::Drainer {
            continue;
        }

        let ownership = if mon.color == perspective { 1 } else { -1 };
        if mon.is_fainted() {
            delta += ownership * (-520 - 70 * mon.cooldown);
            continue;
        }

        let nearest_mana = nearest_mana_distance_for_eval(game, location).max(1);
        let nearest_pool = distance_to_any_pool_for_eval(location).max(1);
        let nearest_threat =
            nearest_drainer_threat_distance_for_eval(game, mon.color, location).max(1);
        let angel_guarded = friendly_angel_adjacent_for_eval(game, mon.color, location);

        delta += ownership * 220;
        delta += ownership * (220 / nearest_mana);
        delta += ownership * (280 / nearest_pool);
        delta += ownership * nearest_threat.min(6) * 22;
        if angel_guarded {
            delta += ownership * 120;
        }

        if let Some(mana) = carried_mana {
            delta += ownership * 260;
            delta += ownership * (360 / nearest_pool);
            if nearest_pool <= 2 {
                delta += ownership * 320;
            }

            let current_score = if mon.color == Color::White {
                game.white_score
            } else {
                game.black_score
            };
            if current_score + mana.score(mon.color) >= Config::TARGET_SCORE {
                delta += ownership * 800;
            }
        }
    }
    delta
}

fn nearest_mana_distance_for_eval(game: &MonsGame, location: Location) -> i32 {
    let mut best = Config::BOARD_SIZE as i32;
    for (&item_location, item) in &game.board.items {
        if matches!(item, Item::Mana { .. }) {
            best = best.min(item_location.distance(&location) as i32 + 1);
        }
    }
    best.max(1)
}

fn nearest_drainer_threat_distance_for_eval(
    game: &MonsGame,
    drainer_color: Color,
    location: Location,
) -> i32 {
    let mut best = Config::BOARD_SIZE as i32;
    for (&item_location, item) in &game.board.items {
        let Some(mon) = item.mon() else {
            continue;
        };
        if mon.color == drainer_color || mon.is_fainted() {
            continue;
        }
        if mon.kind == MonKind::Mystic
            || mon.kind == MonKind::Demon
            || matches!(item, Item::MonWithConsumable { .. })
        {
            best = best.min(item_location.distance(&location) as i32 + 1);
        }
    }
    best.max(1)
}

fn friendly_angel_adjacent_for_eval(
    game: &MonsGame,
    drainer_color: Color,
    location: Location,
) -> bool {
    for (&item_location, item) in &game.board.items {
        let Some(mon) = item.mon() else {
            continue;
        };
        if mon.color == drainer_color
            && mon.kind == MonKind::Angel
            && !mon.is_fainted()
            && item_location.distance(&location) == 1
        {
            return true;
        }
    }
    false
}

fn distance_to_any_pool_for_eval(location: Location) -> i32 {
    let max_index = Config::MAX_LOCATION_INDEX as i32;
    let i = location.i as i32;
    let j = location.j as i32;
    i32::max(
        i32::min(i, (max_index - i).abs()),
        i32::min(j, (max_index - j).abs()),
    ) + 1
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

fn candidate_model_turn_reply_guard(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let perspective = game.active_color;
    let scoring_weights = if config.depth >= 3 {
        &BALANCED_DISTANCE_SCORING_WEIGHTS
    } else {
        &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS
    };

    let mut root_candidates = Vec::new();
    push_unique_candidate(
        &mut root_candidates,
        candidate_model_runtime_d2_tuned(game, config),
    );
    push_unique_candidate(&mut root_candidates, candidate_model_base(game, config));
    if config.depth >= 3 {
        push_unique_candidate(
            &mut root_candidates,
            candidate_model_weights_balanced(game, config),
        );
    } else {
        push_unique_candidate(
            &mut root_candidates,
            candidate_model_focus_light_with_tactical_d2_only(game, config),
        );
    }

    if root_candidates.is_empty() {
        return Vec::new();
    }

    let mut best_inputs = root_candidates[0].clone();
    let mut best_score = i32::MIN;

    for inputs in root_candidates {
        let Some(after_move) = MonsGameModel::apply_inputs_for_search(game, &inputs) else {
            continue;
        };

        let optimistic =
            evaluate_preferability_with_weights(&after_move, perspective, scoring_weights);
        let reply_floor = turn_reply_guard_floor(&after_move, perspective, config, scoring_weights);
        let combined_score = reply_floor.saturating_mul(4).saturating_add(optimistic);

        if combined_score > best_score {
            best_score = combined_score;
            best_inputs = inputs;
        }
    }

    if best_score == i32::MIN {
        candidate_model_runtime_d2_tuned(game, config)
    } else {
        best_inputs
    }
}

fn push_unique_candidate(candidates: &mut Vec<Vec<Input>>, inputs: Vec<Input>) {
    if inputs.is_empty() {
        return;
    }

    let new_key = Input::fen_from_array(&inputs);
    let already_present = candidates
        .iter()
        .any(|existing| Input::fen_from_array(existing) == new_key);
    if !already_present {
        candidates.push(inputs);
    }
}

fn turn_reply_guard_floor(
    state_after_move: &MonsGame,
    perspective: Color,
    config: SmartSearchConfig,
    scoring_weights: &'static ScoringWeights,
) -> i32 {
    if let Some(winner) = state_after_move.winner_color() {
        return if winner == perspective {
            SMART_TERMINAL_SCORE / 2
        } else {
            -SMART_TERMINAL_SCORE / 2
        };
    }

    let mut probe = state_after_move.clone_for_simulation();
    let rollout_steps = if config.depth >= 3 { 2 } else { 1 };

    for _ in 0..rollout_steps {
        if probe.active_color != perspective {
            break;
        }

        let rollout_inputs = MonsGameModel::smart_search_best_inputs(
            &probe,
            turn_reply_guard_rollout_config(config, scoring_weights),
        );

        if rollout_inputs.is_empty() {
            break;
        }

        if !matches!(
            probe.process_input(rollout_inputs, false, false),
            Output::Events(_)
        ) {
            break;
        }

        if probe.winner_color().is_some() {
            break;
        }
    }

    if let Some(winner) = probe.winner_color() {
        return if winner == perspective {
            SMART_TERMINAL_SCORE / 2
        } else {
            -SMART_TERMINAL_SCORE / 2
        };
    }

    if probe.active_color == perspective {
        return evaluate_preferability_with_weights(&probe, perspective, scoring_weights);
    }

    let reply_limit = if config.depth >= 3 {
        config.node_enum_limit.clamp(8, 28)
    } else {
        config.node_enum_limit.clamp(6, 18)
    };
    let replies = MonsGameModel::enumerate_legal_inputs(&probe, reply_limit);
    if replies.is_empty() {
        return SMART_TERMINAL_SCORE / 4;
    }

    let mut worst_reply_score = i32::MAX;
    for reply in replies {
        let Some(after_reply) = MonsGameModel::apply_inputs_for_search(&probe, &reply) else {
            continue;
        };

        let score = match after_reply.winner_color() {
            Some(winner) if winner == perspective => SMART_TERMINAL_SCORE / 2,
            Some(_) => -SMART_TERMINAL_SCORE / 2,
            None => evaluate_preferability_with_weights(&after_reply, perspective, scoring_weights),
        };
        worst_reply_score = worst_reply_score.min(score);
    }

    if worst_reply_score == i32::MAX {
        evaluate_preferability_with_weights(&probe, perspective, scoring_weights)
    } else {
        worst_reply_score
    }
}

fn turn_reply_guard_rollout_config(
    config: SmartSearchConfig,
    scoring_weights: &'static ScoringWeights,
) -> SmartSearchConfig {
    let mut tuned = config;
    tuned.depth = 1;
    tuned.max_visited_nodes = (config.max_visited_nodes / 4).clamp(96, 900);
    tuned.root_branch_limit = config.root_branch_limit.saturating_sub(4).clamp(4, 14);
    tuned.node_branch_limit = config.node_branch_limit.saturating_sub(6).clamp(4, 10);
    tuned.root_enum_limit = (tuned.root_branch_limit * 4).clamp(tuned.root_branch_limit, 48);
    tuned.node_enum_limit = (tuned.node_branch_limit * 3).clamp(tuned.node_branch_limit, 30);
    tuned.scoring_weights = scoring_weights;
    tuned
}

// Replace this when introducing a real contender.
fn candidate_model(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    match candidate_profile().as_str() {
        "base" => candidate_model_base(game, config),
        "runtime_current" => candidate_model_base(game, config),
        "runtime_pre_efficiency_logic" => model_runtime_pre_efficiency_logic(game, config),
        "runtime_pre_root_reply_floor" => model_runtime_pre_root_reply_floor(game, config),
        "runtime_pre_event_ordering" => model_runtime_pre_event_ordering(game, config),
        "runtime_pre_backtrack_penalty" => model_runtime_pre_backtrack_penalty(game, config),
        "runtime_pre_tt_best_child_ordering" => {
            model_runtime_pre_tt_best_child_ordering(game, config)
        }
        "runtime_pre_root_aspiration" => model_runtime_pre_root_aspiration(game, config),
        "runtime_pre_two_pass_root_allocation" => {
            model_runtime_pre_two_pass_root_allocation(game, config)
        }
        "runtime_pre_selective_extensions" => model_runtime_pre_selective_extensions(game, config),
        "runtime_pre_quiet_reductions" => model_runtime_pre_quiet_reductions(game, config),
        "runtime_pre_root_mana_handoff_guard" => {
            model_runtime_pre_root_mana_handoff_guard(game, config)
        }
        "runtime_pre_root_reply_risk_guard" => {
            model_runtime_pre_root_reply_risk_guard(game, config)
        }
        "runtime_pre_normal_reply_risk_cleanup" => {
            model_runtime_pre_normal_reply_risk_cleanup(game, config)
        }
        "runtime_pre_move_class_coverage" => model_runtime_pre_move_class_coverage(game, config),
        "runtime_pre_horizon_eval" => model_runtime_pre_horizon_eval(game, config),
        "runtime_pre_eval_formula_v2" => model_runtime_pre_eval_formula_v2(game, config),
        "runtime_pre_fast_multi_lane_eval" => model_runtime_pre_fast_multi_lane_eval(game, config),
        "runtime_eval_mana_window" => model_runtime_eval_mana_window(game, config),
        "runtime_eval_mana_window_defense" => model_runtime_eval_mana_window_defense(game, config),
        "runtime_eval_mana_window_pressure" => {
            model_runtime_eval_mana_window_pressure(game, config)
        }
        "runtime_eval_no_confirmed_square" => model_runtime_eval_no_confirmed_square(game, config),
        "runtime_eval_match_point_window" => model_runtime_eval_match_point_window(game, config),
        "runtime_eval_global_pressure" => model_runtime_eval_global_pressure(game, config),
        "runtime_eval_multi_lane_light" => model_runtime_eval_multi_lane_light(game, config),
        "runtime_eval_multi_lane_aggr" => model_runtime_eval_multi_lane_aggr(game, config),
        "runtime_eval_multi_lane_normal_only" => {
            model_runtime_eval_multi_lane_normal_only(game, config)
        }
        "runtime_eval_multi_lane_mixed" => model_runtime_eval_multi_lane_mixed(game, config),
        "runtime_eval_env_multi" => model_runtime_eval_env_multi(game, config),
        "runtime_eval_board_v1" => model_runtime_eval_board_v1(game, config),
        "runtime_eval_board_v2" => model_runtime_eval_board_v2(game, config),
        "runtime_eval_board_v3" => model_runtime_eval_board_v3(game, config),
        "runtime_pre_normal_guarded_lookahead" => {
            model_runtime_pre_normal_guarded_lookahead(game, config)
        }
        "runtime_pre_search_upgrade_bundle" => {
            model_runtime_pre_search_upgrade_bundle(game, config)
        }
        "runtime_pre_fast_efficiency_cleanup" => {
            model_runtime_pre_fast_efficiency_cleanup(game, config)
        }
        "runtime_pre_drainer_tactical_requirements" => {
            model_runtime_pre_drainer_tactical_requirements(game, config)
        }
        "runtime_pre_forced_drainer_attack_fallback" => {
            model_runtime_pre_forced_drainer_attack_fallback(game, config)
        }
        "runtime_pre_forced_tactical_prepass" => {
            model_runtime_pre_forced_tactical_prepass(game, config)
        }
        "runtime_pre_phase1_root_quality_bundle" => {
            model_runtime_pre_phase1_root_quality_bundle(game, config)
        }
        "runtime_pre_spirit_development_pref" => {
            model_runtime_pre_spirit_development_pref(game, config)
        }
        "runtime_pre_normal_phase_deeper_lite" => {
            model_runtime_pre_normal_phase_deeper_lite(game, config)
        }
        "runtime_pre_normal_spirit_base_penalty" => {
            model_runtime_pre_normal_spirit_base_penalty(game, config)
        }
        "runtime_normal_spirit_base_strict" => {
            candidate_model_runtime_normal_spirit_base_strict(game, config)
        }
        "runtime_normal_confirmed_850" => {
            candidate_model_runtime_normal_confirmed_850(game, config)
        }
        "runtime_normal_reinvest_search" => {
            candidate_model_runtime_normal_reinvest_search(game, config)
        }
        "runtime_normal_efficiency_reply_floor" => {
            candidate_model_runtime_normal_efficiency_reply_floor(game, config)
        }
        "runtime_pre_normal_efficiency_reply_floor" => {
            model_runtime_pre_normal_efficiency_reply_floor(game, config)
        }
        "runtime_pre_normal_root_safety_deep_floor" => {
            model_runtime_pre_normal_root_safety_deep_floor(game, config)
        }
        "runtime_pre_normal_root_safety" => model_runtime_pre_normal_root_safety(game, config),
        "runtime_pre_root_upgrade_bundle" => model_runtime_pre_root_upgrade_bundle(game, config),
        "runtime_pre_move_efficiency" => model_runtime_pre_move_efficiency(game, config),
        "runtime_pre_fast_drainer_priority" => {
            model_runtime_pre_fast_drainer_priority(game, config)
        }
        "runtime_pre_tactical_runtime" => model_runtime_pre_tactical_runtime(game, config),
        "runtime_pre_winloss_weights" => model_runtime_pre_winloss_weights(game, config),
        "runtime_pre_transposition" => model_current_best_legacy_no_transposition(game, config),
        "runtime_legacy_phase_adaptive" => model_runtime_legacy_phase_adaptive(game, config),
        "runtime_pre_drainer_context" => model_runtime_pre_drainer_context(game, config),
        "runtime_pre_normal_x15" => model_runtime_pre_normal_x15(game, config),
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
        "runtime_d2_tuned_d3_tactical" => {
            candidate_model_runtime_d2_tuned_d3_tactical(game, config)
        }
        "runtime_d2_tuned_d3_winloss" => candidate_model_runtime_d2_tuned_d3_winloss(game, config),
        "runtime_d2_tuned_d3_tactical_phase" => {
            candidate_model_runtime_d2_tuned_d3_tactical_phase(game, config)
        }
        "runtime_d2_tuned_d3_tactical_aggr" => {
            candidate_model_runtime_d2_tuned_d3_tactical_aggr(game, config)
        }
        "runtime_d2_tuned_d3_finisher_soft_aggr" => {
            candidate_model_runtime_d2_tuned_d3_finisher_soft_aggr(game, config)
        }
        "runtime_d2_tuned_d3_mana_neutral" => {
            candidate_model_runtime_d2_tuned_d3_mana_neutral(game, config)
        }
        "runtime_d2_tuned_d3_adaptive_neutral" => {
            candidate_model_runtime_d2_tuned_d3_adaptive_neutral(game, config)
        }
        "runtime_d2_tuned_d3_phase_adaptive" => {
            candidate_model_runtime_d2_tuned_d3_phase_adaptive(game, config)
        }
        "runtime_fast_phase_normal_tactical" => {
            candidate_model_runtime_fast_phase_normal_tactical(game, config)
        }
        "runtime_fast_wideroot_normal_current" => {
            candidate_model_runtime_fast_wideroot_normal_current(game, config)
        }
        "runtime_fast_wideroot_lite_normal_current" => {
            candidate_model_runtime_fast_wideroot_lite_normal_current(game, config)
        }
        "runtime_drainer_priority" => candidate_model_runtime_drainer_priority(game, config),
        "runtime_drainer_priority_aggr" => {
            candidate_model_runtime_drainer_priority_aggr(game, config)
        }
        "runtime_drainer_tiebreak" => candidate_model_runtime_drainer_tiebreak(game, config),
        "runtime_drainer_context" => candidate_model_runtime_drainer_context(game, config),
        "runtime_drainer_priority_fast_only" => {
            candidate_model_runtime_drainer_priority_fast_only(game, config)
        }
        "runtime_fast_wideroot_normal_tactical" => {
            candidate_model_runtime_fast_wideroot_normal_tactical(game, config)
        }
        "runtime_fast_reply_guard_normal_current" => {
            candidate_model_runtime_fast_reply_guard_normal_current(game, config)
        }
        "runtime_fast_reply_guard_normal_x15_tactical_lite" => {
            candidate_model_runtime_fast_reply_guard_normal_x15_tactical_lite(game, config)
        }
        "runtime_fast_simplified_normal_current" => {
            candidate_model_runtime_fast_simplified_normal_current(game, config)
        }
        "runtime_fast_simplified_normal_x15_tactical_lite" => {
            candidate_model_runtime_fast_simplified_normal_x15_tactical_lite(game, config)
        }
        "runtime_fast_no_event_ordering_normal_x15_tactical_lite" => {
            candidate_model_runtime_fast_no_event_ordering_normal_x15_tactical_lite(game, config)
        }
        "runtime_fast_eventless_reinvest10_normal_x15_tactical_lite" => {
            candidate_model_runtime_fast_eventless_reinvest10_normal_x15_tactical_lite(game, config)
        }
        "runtime_fast_eventless_reinvest15_normal_x15_tactical_lite" => {
            candidate_model_runtime_fast_eventless_reinvest15_normal_x15_tactical_lite(game, config)
        }
        "runtime_fast_d3_lite600_normal_x15_tactical_lite" => {
            candidate_model_runtime_fast_d3_lite600_normal_x15_tactical_lite(game, config)
        }
        "runtime_fast_d3_lite700_normal_x15_tactical_lite" => {
            candidate_model_runtime_fast_d3_lite700_normal_x15_tactical_lite(game, config)
        }
        "runtime_fast_env_tune_normal_x15_tactical_lite" => {
            candidate_model_runtime_fast_env_tune_normal_x15_tactical_lite(game, config)
        }
        "runtime_normal_x15_phase_deeper" => {
            candidate_model_runtime_normal_x15_phase_deeper(game, config)
        }
        "runtime_normal_x15_phase_deeper_lite" => {
            candidate_model_runtime_normal_x15_phase_deeper_lite(game, config)
        }
        "runtime_normal_x15_tactical" => candidate_model_runtime_normal_x15_tactical(game, config),
        "runtime_normal_x15_tactical_lite" => {
            candidate_model_runtime_normal_x15_tactical_lite(game, config)
        }
        "runtime_normal_x15_tactical_phase" => {
            candidate_model_runtime_normal_x15_tactical_phase(game, config)
        }
        "runtime_normal_x15_plain" => candidate_model_runtime_normal_x15_plain(game, config),
        "runtime_normal_x15_finisher_only" => {
            candidate_model_runtime_normal_x15_finisher_only(game, config)
        }
        "runtime_normal_d4_selective" => candidate_model_runtime_normal_d4_selective(game, config),
        "runtime_normal_x15_guarded_root" => {
            candidate_model_runtime_normal_x15_guarded_root(game, config)
        }
        "runtime_normal_x15_guarded_root_v2" => {
            candidate_model_runtime_normal_x15_guarded_root_v2(game, config)
        }
        "runtime_root_safety_tiebreak" => {
            candidate_model_runtime_root_safety_tiebreak(game, config)
        }
        "runtime_d2_tuned_normal_reply_guard" => {
            candidate_model_runtime_d2_tuned_normal_reply_guard(game, config)
        }
        "phase_adaptive_d2" => candidate_model_phase_adaptive_d2(game, config),
        "phase_adaptive_scoring_v2" => candidate_model_phase_adaptive_scoring_v2(game, config),
        "turn_reply_guard" => candidate_model_turn_reply_guard(game, config),
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

fn all_profile_variants() -> Vec<(&'static str, fn(&MonsGame, SmartSearchConfig) -> Vec<Input>)> {
    vec![
        ("base", candidate_model_base),
        ("runtime_current", candidate_model_base),
        (
            "runtime_pre_efficiency_logic",
            model_runtime_pre_efficiency_logic,
        ),
        (
            "runtime_pre_root_reply_floor",
            model_runtime_pre_root_reply_floor,
        ),
        (
            "runtime_pre_event_ordering",
            model_runtime_pre_event_ordering,
        ),
        (
            "runtime_pre_backtrack_penalty",
            model_runtime_pre_backtrack_penalty,
        ),
        (
            "runtime_pre_tt_best_child_ordering",
            model_runtime_pre_tt_best_child_ordering,
        ),
        (
            "runtime_pre_root_aspiration",
            model_runtime_pre_root_aspiration,
        ),
        (
            "runtime_pre_two_pass_root_allocation",
            model_runtime_pre_two_pass_root_allocation,
        ),
        (
            "runtime_pre_selective_extensions",
            model_runtime_pre_selective_extensions,
        ),
        (
            "runtime_pre_quiet_reductions",
            model_runtime_pre_quiet_reductions,
        ),
        (
            "runtime_pre_root_mana_handoff_guard",
            model_runtime_pre_root_mana_handoff_guard,
        ),
        (
            "runtime_pre_root_reply_risk_guard",
            model_runtime_pre_root_reply_risk_guard,
        ),
        (
            "runtime_pre_normal_reply_risk_cleanup",
            model_runtime_pre_normal_reply_risk_cleanup,
        ),
        (
            "runtime_pre_move_class_coverage",
            model_runtime_pre_move_class_coverage,
        ),
        ("runtime_pre_horizon_eval", model_runtime_pre_horizon_eval),
        (
            "runtime_pre_eval_formula_v2",
            model_runtime_pre_eval_formula_v2,
        ),
        (
            "runtime_pre_fast_multi_lane_eval",
            model_runtime_pre_fast_multi_lane_eval,
        ),
        ("runtime_eval_mana_window", model_runtime_eval_mana_window),
        (
            "runtime_eval_mana_window_defense",
            model_runtime_eval_mana_window_defense,
        ),
        (
            "runtime_eval_mana_window_pressure",
            model_runtime_eval_mana_window_pressure,
        ),
        (
            "runtime_eval_no_confirmed_square",
            model_runtime_eval_no_confirmed_square,
        ),
        (
            "runtime_eval_match_point_window",
            model_runtime_eval_match_point_window,
        ),
        (
            "runtime_eval_global_pressure",
            model_runtime_eval_global_pressure,
        ),
        (
            "runtime_eval_multi_lane_light",
            model_runtime_eval_multi_lane_light,
        ),
        (
            "runtime_eval_multi_lane_aggr",
            model_runtime_eval_multi_lane_aggr,
        ),
        (
            "runtime_eval_multi_lane_normal_only",
            model_runtime_eval_multi_lane_normal_only,
        ),
        (
            "runtime_eval_multi_lane_mixed",
            model_runtime_eval_multi_lane_mixed,
        ),
        ("runtime_eval_env_multi", model_runtime_eval_env_multi),
        ("runtime_eval_board_v1", model_runtime_eval_board_v1),
        ("runtime_eval_board_v2", model_runtime_eval_board_v2),
        ("runtime_eval_board_v3", model_runtime_eval_board_v3),
        (
            "runtime_pre_normal_guarded_lookahead",
            model_runtime_pre_normal_guarded_lookahead,
        ),
        (
            "runtime_pre_search_upgrade_bundle",
            model_runtime_pre_search_upgrade_bundle,
        ),
        (
            "runtime_pre_fast_efficiency_cleanup",
            model_runtime_pre_fast_efficiency_cleanup,
        ),
        (
            "runtime_pre_drainer_tactical_requirements",
            model_runtime_pre_drainer_tactical_requirements,
        ),
        (
            "runtime_pre_forced_drainer_attack_fallback",
            model_runtime_pre_forced_drainer_attack_fallback,
        ),
        (
            "runtime_pre_forced_tactical_prepass",
            model_runtime_pre_forced_tactical_prepass,
        ),
        (
            "runtime_pre_phase1_root_quality_bundle",
            model_runtime_pre_phase1_root_quality_bundle,
        ),
        (
            "runtime_pre_spirit_development_pref",
            model_runtime_pre_spirit_development_pref,
        ),
        (
            "runtime_pre_normal_phase_deeper_lite",
            model_runtime_pre_normal_phase_deeper_lite,
        ),
        (
            "runtime_pre_normal_spirit_base_penalty",
            model_runtime_pre_normal_spirit_base_penalty,
        ),
        (
            "runtime_normal_spirit_base_strict",
            candidate_model_runtime_normal_spirit_base_strict,
        ),
        (
            "runtime_normal_confirmed_850",
            candidate_model_runtime_normal_confirmed_850,
        ),
        (
            "runtime_normal_reinvest_search",
            candidate_model_runtime_normal_reinvest_search,
        ),
        (
            "runtime_normal_efficiency_reply_floor",
            candidate_model_runtime_normal_efficiency_reply_floor,
        ),
        (
            "runtime_pre_normal_efficiency_reply_floor",
            model_runtime_pre_normal_efficiency_reply_floor,
        ),
        (
            "runtime_pre_normal_root_safety_deep_floor",
            model_runtime_pre_normal_root_safety_deep_floor,
        ),
        (
            "runtime_pre_normal_root_safety",
            model_runtime_pre_normal_root_safety,
        ),
        (
            "runtime_pre_root_upgrade_bundle",
            model_runtime_pre_root_upgrade_bundle,
        ),
        (
            "runtime_pre_move_efficiency",
            model_runtime_pre_move_efficiency,
        ),
        (
            "runtime_pre_fast_drainer_priority",
            model_runtime_pre_fast_drainer_priority,
        ),
        (
            "runtime_pre_tactical_runtime",
            model_runtime_pre_tactical_runtime,
        ),
        (
            "runtime_pre_winloss_weights",
            model_runtime_pre_winloss_weights,
        ),
        (
            "runtime_pre_transposition",
            model_current_best_legacy_no_transposition,
        ),
        (
            "runtime_legacy_phase_adaptive",
            model_runtime_legacy_phase_adaptive,
        ),
        (
            "runtime_pre_drainer_context",
            model_runtime_pre_drainer_context,
        ),
        ("runtime_pre_normal_x15", model_runtime_pre_normal_x15),
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
        (
            "runtime_d2_tuned_d3_tactical",
            candidate_model_runtime_d2_tuned_d3_tactical,
        ),
        (
            "runtime_d2_tuned_d3_winloss",
            candidate_model_runtime_d2_tuned_d3_winloss,
        ),
        (
            "runtime_d2_tuned_d3_tactical_phase",
            candidate_model_runtime_d2_tuned_d3_tactical_phase,
        ),
        (
            "runtime_d2_tuned_d3_tactical_aggr",
            candidate_model_runtime_d2_tuned_d3_tactical_aggr,
        ),
        (
            "runtime_d2_tuned_d3_finisher_soft_aggr",
            candidate_model_runtime_d2_tuned_d3_finisher_soft_aggr,
        ),
        (
            "runtime_d2_tuned_d3_mana_neutral",
            candidate_model_runtime_d2_tuned_d3_mana_neutral,
        ),
        (
            "runtime_d2_tuned_d3_adaptive_neutral",
            candidate_model_runtime_d2_tuned_d3_adaptive_neutral,
        ),
        (
            "runtime_d2_tuned_d3_phase_adaptive",
            candidate_model_runtime_d2_tuned_d3_phase_adaptive,
        ),
        (
            "runtime_fast_phase_normal_tactical",
            candidate_model_runtime_fast_phase_normal_tactical,
        ),
        (
            "runtime_fast_wideroot_normal_current",
            candidate_model_runtime_fast_wideroot_normal_current,
        ),
        (
            "runtime_fast_wideroot_lite_normal_current",
            candidate_model_runtime_fast_wideroot_lite_normal_current,
        ),
        (
            "runtime_drainer_priority",
            candidate_model_runtime_drainer_priority,
        ),
        (
            "runtime_drainer_priority_aggr",
            candidate_model_runtime_drainer_priority_aggr,
        ),
        (
            "runtime_drainer_tiebreak",
            candidate_model_runtime_drainer_tiebreak,
        ),
        (
            "runtime_drainer_context",
            candidate_model_runtime_drainer_context,
        ),
        (
            "runtime_drainer_priority_fast_only",
            candidate_model_runtime_drainer_priority_fast_only,
        ),
        (
            "runtime_fast_wideroot_normal_tactical",
            candidate_model_runtime_fast_wideroot_normal_tactical,
        ),
        (
            "runtime_fast_reply_guard_normal_current",
            candidate_model_runtime_fast_reply_guard_normal_current,
        ),
        (
            "runtime_fast_reply_guard_normal_x15_tactical_lite",
            candidate_model_runtime_fast_reply_guard_normal_x15_tactical_lite,
        ),
        (
            "runtime_fast_simplified_normal_current",
            candidate_model_runtime_fast_simplified_normal_current,
        ),
        (
            "runtime_fast_simplified_normal_x15_tactical_lite",
            candidate_model_runtime_fast_simplified_normal_x15_tactical_lite,
        ),
        (
            "runtime_fast_no_event_ordering_normal_x15_tactical_lite",
            candidate_model_runtime_fast_no_event_ordering_normal_x15_tactical_lite,
        ),
        (
            "runtime_fast_eventless_reinvest10_normal_x15_tactical_lite",
            candidate_model_runtime_fast_eventless_reinvest10_normal_x15_tactical_lite,
        ),
        (
            "runtime_fast_eventless_reinvest15_normal_x15_tactical_lite",
            candidate_model_runtime_fast_eventless_reinvest15_normal_x15_tactical_lite,
        ),
        (
            "runtime_fast_d3_lite600_normal_x15_tactical_lite",
            candidate_model_runtime_fast_d3_lite600_normal_x15_tactical_lite,
        ),
        (
            "runtime_fast_d3_lite700_normal_x15_tactical_lite",
            candidate_model_runtime_fast_d3_lite700_normal_x15_tactical_lite,
        ),
        (
            "runtime_fast_env_tune_normal_x15_tactical_lite",
            candidate_model_runtime_fast_env_tune_normal_x15_tactical_lite,
        ),
        (
            "runtime_normal_x15_phase_deeper",
            candidate_model_runtime_normal_x15_phase_deeper,
        ),
        (
            "runtime_normal_x15_phase_deeper_lite",
            candidate_model_runtime_normal_x15_phase_deeper_lite,
        ),
        (
            "runtime_normal_x15_tactical",
            candidate_model_runtime_normal_x15_tactical,
        ),
        (
            "runtime_normal_x15_tactical_lite",
            candidate_model_runtime_normal_x15_tactical_lite,
        ),
        (
            "runtime_normal_x15_tactical_phase",
            candidate_model_runtime_normal_x15_tactical_phase,
        ),
        (
            "runtime_normal_x15_plain",
            candidate_model_runtime_normal_x15_plain,
        ),
        (
            "runtime_normal_x15_finisher_only",
            candidate_model_runtime_normal_x15_finisher_only,
        ),
        (
            "runtime_normal_d4_selective",
            candidate_model_runtime_normal_d4_selective,
        ),
        (
            "runtime_normal_x15_guarded_root",
            candidate_model_runtime_normal_x15_guarded_root,
        ),
        (
            "runtime_normal_x15_guarded_root_v2",
            candidate_model_runtime_normal_x15_guarded_root_v2,
        ),
        (
            "runtime_root_safety_tiebreak",
            candidate_model_runtime_root_safety_tiebreak,
        ),
        (
            "runtime_d2_tuned_normal_reply_guard",
            candidate_model_runtime_d2_tuned_normal_reply_guard,
        ),
        ("phase_adaptive_d2", candidate_model_phase_adaptive_d2),
        (
            "phase_adaptive_scoring_v2",
            candidate_model_phase_adaptive_scoring_v2,
        ),
        ("turn_reply_guard", candidate_model_turn_reply_guard),
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
    ]
}

fn profile_selector_from_name(
    profile_name: &str,
) -> Option<fn(&MonsGame, SmartSearchConfig) -> Vec<Input>> {
    all_profile_variants()
        .into_iter()
        .find(|(name, _)| *name == profile_name)
        .map(|(_, selector)| selector)
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

fn tuned_candidate_config_wideroot_lite(config: SmartSearchConfig) -> SmartSearchConfig {
    let mut tuned = config;
    tuned.root_branch_limit = (config.root_branch_limit + 5).clamp(8, 36);
    tuned.node_branch_limit = config.node_branch_limit.saturating_sub(1).clamp(6, 18);
    tuned.root_enum_limit = (tuned.root_branch_limit * 5).clamp(tuned.root_branch_limit, 210);
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
    let opening_fens = generate_opening_fens_cached(seed, games_per_matchup);
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
    let use_white_opening_book = env_bool("SMART_USE_WHITE_OPENING_BOOK").unwrap_or(false);

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

        let config = budget.runtime_config_for_game(&game);
        let inputs = if use_white_opening_book {
            MonsGameModel::white_first_turn_opening_next_inputs(&game)
                .unwrap_or_else(|| (actor_model.select_inputs)(&game, config))
        } else {
            (actor_model.select_inputs)(&game, config)
        };
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

fn run_budget_duel_series(
    model_a: AutomoveModel,
    budget_a: SearchBudget,
    model_b: AutomoveModel,
    budget_b: SearchBudget,
    games: usize,
    seed: u64,
    max_plies: usize,
) -> MatchupStats {
    let opening_fens = generate_opening_fens_cached(seed, games);
    let mut stats = MatchupStats::default();
    for game_index in 0..games {
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

fn run_budget_conversion_diagnostic(
    profile_name: &str,
    selector: fn(&MonsGame, SmartSearchConfig) -> Vec<Input>,
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
    for repeat_index in 0..repeats {
        let seed = seed_for_budget_duel_repeat_and_tag(
            fast_budget,
            normal_budget,
            repeat_index,
            format!("{}:{}", seed_tag, profile_name).as_str(),
        );
        let stats = run_budget_duel_series(
            model_fast,
            fast_budget,
            model_normal,
            normal_budget,
            games_per_repeat,
            seed,
            max_plies,
        );
        aggregate.merge(stats);
    }

    let fast_win_rate = aggregate.win_rate_points();
    BudgetConversionDiagnostic {
        fast_wins: aggregate.wins,
        fast_losses: aggregate.losses,
        draws: aggregate.draws,
        fast_win_rate,
        normal_edge: 0.5 - fast_win_rate,
        confidence: aggregate.confidence_better_than_even(),
    }
}

fn play_one_game_budget_duel(
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

    for _ply in 0..max_plies {
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
            MonsGameModel::white_first_turn_opening_next_inputs(&game)
                .unwrap_or_else(|| (actor_model.select_inputs)(&game, config))
        } else {
            (actor_model.select_inputs)(&game, config)
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

    let adjudicated_winner = adjudicate_non_terminal_game(&game);
    match adjudicated_winner {
        Some(winner_color) => match_result_from_winner(winner_color, a_is_white),
        None => MatchResult::Draw,
    }
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

fn opening_fens_cache() -> &'static Mutex<std::collections::HashMap<(u64, usize), Arc<Vec<String>>>>
{
    static CACHE: OnceLock<Mutex<std::collections::HashMap<(u64, usize), Arc<Vec<String>>>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn generate_opening_fens_cached(seed: u64, count: usize) -> Arc<Vec<String>> {
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

fn seed_for_pairing_budget_and_repeat(
    candidate_id: &str,
    opponent_id: &str,
    budget: SearchBudget,
    repeat_index: usize,
) -> u64 {
    let mut hash = seed_for_pairing_and_budget(candidate_id, opponent_id, budget);
    hash ^= (repeat_index as u64).wrapping_mul(0x94d049bb133111eb);
    hash = hash.wrapping_mul(1099511628211);
    hash
}

fn seed_for_budget_repeat_and_tag(
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

fn seed_for_budget_duel_repeat_and_tag(
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

fn seed_for_budget_duel_pairing_and_repeat(
    candidate_id: &str,
    opponent_id: &str,
    budget_a: SearchBudget,
    budget_b: SearchBudget,
    repeat_index: usize,
) -> u64 {
    let mut hash =
        seed_for_pairing_budget_and_repeat(candidate_id, opponent_id, budget_a, repeat_index);
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

fn env_usize(name: &str) -> Option<usize> {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
}

fn env_i32(name: &str) -> Option<i32> {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<i32>().ok())
}

fn env_bool(name: &str) -> Option<bool> {
    env::var(name).ok().and_then(|value| {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "1" | "true" | "yes" => Some(true),
            "0" | "false" | "no" => Some(false),
            _ => None,
        }
    })
}

fn env_automove_preference(name: &str) -> Option<SmartAutomovePreference> {
    env::var(name)
        .ok()
        .and_then(|value| SmartAutomovePreference::from_api_value(value.as_str()))
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
        label: "smoke_d1n96",
        depth: 1,
        max_nodes: 96,
    }];
    let pool = selected_pool_models();
    let evaluation = evaluate_candidate_against_pool(CANDIDATE_MODEL, &pool, 2, &quick_budgets);

    assert_eq!(evaluation.opponents.len(), pool.len());
    assert_eq!(evaluation.games_per_matchup, 2);
}

#[test]
#[ignore = "long-running tournament using production client modes"]
fn smart_automove_pool_candidate_promotion_with_client_budgets() {
    let games_per_matchup = env_usize("SMART_POOL_GAMES").unwrap_or(GAMES_PER_MATCHUP);
    let pool = selected_pool_models();
    let client_budgets = client_budgets();
    let evaluation =
        evaluate_candidate_against_pool(CANDIDATE_MODEL, &pool, games_per_matchup, &client_budgets);
    println!(
        "settings: profile={} games_per_matchup={} opponents={} modes={:?}",
        candidate_profile(),
        games_per_matchup,
        pool.len(),
        client_budgets
            .iter()
            .map(|budget| budget.key().to_string())
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
    let client_budgets = client_budgets();
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
    let variants = all_profile_variants()
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
            evaluate_candidate_against_pool(candidate, &pool, games_per_matchup, &client_budgets);
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

#[derive(Clone, Copy)]
struct ModeSpeedStat {
    budget: SearchBudget,
    avg_ms: f64,
}

fn profile_speed_by_mode_ms(
    selector: fn(&MonsGame, SmartSearchConfig) -> Vec<Input>,
    openings: &[String],
    budgets: &[SearchBudget],
) -> Vec<ModeSpeedStat> {
    if openings.is_empty() || budgets.is_empty() {
        return Vec::new();
    }

    let mut mode_stats = Vec::with_capacity(budgets.len());
    for budget in budgets.iter().copied() {
        let start = Instant::now();
        for opening in openings {
            let game = MonsGame::from_fen(opening, false).expect("valid opening fen");
            let config = budget.runtime_config_for_game(&game);
            let _inputs = selector(&game, config);
        }

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        mode_stats.push(ModeSpeedStat {
            budget,
            avg_ms: elapsed_ms / openings.len() as f64,
        });
    }

    mode_stats
}

fn average_mode_speed_ms(stats: &[ModeSpeedStat]) -> f64 {
    if stats.is_empty() {
        0.0
    } else {
        stats.iter().map(|stat| stat.avg_ms).sum::<f64>() / stats.len() as f64
    }
}

fn warmup_profile_speed_measurement(
    selectors: &[fn(&MonsGame, SmartSearchConfig) -> Vec<Input>],
    openings: &[String],
    budgets: &[SearchBudget],
) {
    if selectors.is_empty() || openings.is_empty() || budgets.is_empty() {
        return;
    }

    let warmup_positions = openings.len().min(2);
    for selector in selectors {
        for opening in openings.iter().take(warmup_positions) {
            let game = MonsGame::from_fen(opening, false).expect("valid opening fen");
            for budget in budgets.iter().copied() {
                let config = budget.runtime_config_for_game(&game);
                let _inputs = selector(&game, config);
            }
        }
    }
}

#[test]
#[ignore = "fast iterative profile pipeline with speed gating and quick strength ranking"]
fn smart_automove_pool_fast_pipeline() {
    let games_per_matchup = env_usize("SMART_FAST_GAMES").unwrap_or(2).max(1);
    let opponents = env_usize("SMART_FAST_OPPONENTS")
        .unwrap_or(2)
        .clamp(1, POOL_SIZE);
    let max_plies = env_usize("SMART_FAST_MAX_PLIES").unwrap_or(80).max(32);
    let speed_positions = env_usize("SMART_FAST_SPEED_POSITIONS").unwrap_or(6).max(1);
    let speed_ratio_limit = env::var("SMART_FAST_SPEED_RATIO_MAX")
        .ok()
        .and_then(|value| value.trim().parse::<f64>().ok())
        .unwrap_or(1.25)
        .max(1.0);
    let speed_ratio_mode_limit = env::var("SMART_FAST_SPEED_RATIO_MODE_MAX")
        .ok()
        .and_then(|value| value.trim().parse::<f64>().ok())
        .unwrap_or(speed_ratio_limit)
        .max(1.0);
    let use_client_modes = env_bool("SMART_FAST_USE_CLIENT_MODES")
        .or_else(|| env_bool("SMART_FAST_USE_CLIENT_BUDGETS"))
        .unwrap_or(true);
    let baseline_profile = env::var("SMART_FAST_BASELINE")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "base".to_string());

    let requested_profiles = env::var("SMART_FAST_PROFILES")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|name| name.trim().to_lowercase())
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            vec![
                "base".to_string(),
                "runtime_d2_tuned".to_string(),
                "runtime_d2_tuned_d3_phase_adaptive".to_string(),
            ]
        });

    let profiles_catalog = all_profile_variants();
    let mut profiles = Vec::new();
    for profile_name in requested_profiles {
        if let Some((name, selector)) = profiles_catalog
            .iter()
            .find(|(name, _)| *name == profile_name)
            .copied()
        {
            profiles.push((name.to_string(), selector));
        } else {
            println!(
                "fast pipeline: unknown profile '{}', skipping",
                profile_name
            );
        }
    }
    assert!(
        !profiles.is_empty(),
        "no valid profiles selected for fast pipeline"
    );

    let budgets: Vec<SearchBudget> = if use_client_modes {
        client_budgets().to_vec()
    } else {
        vec![SearchBudget::from_preference(SmartAutomovePreference::Fast)]
    };

    let original_max_plies = env::var("SMART_POOL_MAX_PLIES").ok();
    env::set_var("SMART_POOL_MAX_PLIES", max_plies.to_string());

    let speed_seed = seed_for_pairing("fast", "speed");
    let speed_openings = generate_opening_fens_cached(speed_seed, speed_positions);
    let baseline_selector = profiles
        .iter()
        .find(|(name, _)| name == &baseline_profile)
        .map(|(_, selector)| *selector)
        .or_else(|| profile_selector_from_name(baseline_profile.as_str()))
        .unwrap_or(candidate_model_base);
    let mut warmup_selectors = profiles
        .iter()
        .map(|(_, selector)| *selector)
        .collect::<Vec<_>>();
    if !warmup_selectors
        .iter()
        .any(|selector| std::ptr::fn_addr_eq(*selector, baseline_selector))
    {
        warmup_selectors.push(baseline_selector);
    }
    warmup_profile_speed_measurement(
        warmup_selectors.as_slice(),
        speed_openings.as_ref().as_slice(),
        budgets.as_slice(),
    );
    let baseline_speed_stats = profile_speed_by_mode_ms(
        baseline_selector,
        speed_openings.as_ref().as_slice(),
        budgets.as_slice(),
    );
    let baseline_speed_ms = average_mode_speed_ms(baseline_speed_stats.as_slice());
    let baseline_speed_by_mode: std::collections::HashMap<&'static str, f64> = baseline_speed_stats
        .iter()
        .map(|stat| (stat.budget.key(), stat.avg_ms))
        .collect();
    println!(
        "fast pipeline baseline: profile={} avg_ms={:.2} mode_ms={:?}",
        baseline_profile,
        baseline_speed_ms,
        baseline_speed_stats
            .iter()
            .map(|stat| format!("{}:{:.2}", stat.budget.key(), stat.avg_ms))
            .collect::<Vec<_>>()
    );

    let pool = selected_pool_models()
        .into_iter()
        .take(opponents)
        .collect::<Vec<_>>();

    let mut ranked: Vec<(String, f64, f64, f64, String, CandidateEvaluation)> = Vec::new();
    for (profile_name, selector) in profiles {
        let speed_stats = if profile_name == baseline_profile {
            baseline_speed_stats.clone()
        } else {
            profile_speed_by_mode_ms(
                selector,
                speed_openings.as_ref().as_slice(),
                budgets.as_slice(),
            )
        };
        let avg_speed_ms = average_mode_speed_ms(speed_stats.as_slice());
        let speed_ratio = if baseline_speed_ms > 0.0 {
            avg_speed_ms / baseline_speed_ms
        } else {
            1.0
        };
        let mut max_mode_speed_ratio: f64 = 1.0;
        let mode_ratio_summary = speed_stats
            .iter()
            .map(|stat| {
                let baseline_mode_ms = baseline_speed_by_mode
                    .get(stat.budget.key())
                    .copied()
                    .unwrap_or(baseline_speed_ms.max(0.001));
                let mode_ratio = if baseline_mode_ms > 0.0 {
                    stat.avg_ms / baseline_mode_ms
                } else {
                    1.0
                };
                max_mode_speed_ratio = max_mode_speed_ratio.max(mode_ratio);
                format!("{}:{:.2}", stat.budget.key(), mode_ratio)
            })
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "fast pipeline speed: profile={} avg_ms={:.2} ratio_vs_base={:.2} mode_ratios=[{}]",
            profile_name, avg_speed_ms, speed_ratio, mode_ratio_summary
        );

        if speed_ratio > speed_ratio_limit || max_mode_speed_ratio > speed_ratio_mode_limit {
            println!(
                "fast pipeline dropped: profile={} reason=avg_ratio {:.2} > {:.2} or mode_ratio {:.2} > {:.2}",
                profile_name, speed_ratio, speed_ratio_limit, max_mode_speed_ratio, speed_ratio_mode_limit
            );
            continue;
        }

        let candidate = AutomoveModel {
            id: "candidate",
            select_inputs: selector,
        };
        let evaluation = evaluate_candidate_against_pool(
            candidate,
            &pool,
            games_per_matchup,
            budgets.as_slice(),
        );
        println!(
            "fast pipeline strength: profile={} win_rate={:.3} confidence={:.3} beaten={}/{} promoted={}",
            profile_name,
            evaluation.aggregate_stats.win_rate_points(),
            evaluation.aggregate_confidence,
            evaluation.beaten_opponents,
            evaluation.opponents.len(),
            evaluation.promoted
        );

        ranked.push((
            profile_name,
            avg_speed_ms,
            speed_ratio,
            max_mode_speed_ratio,
            mode_ratio_summary,
            evaluation,
        ));
    }

    if let Some(previous) = original_max_plies {
        env::set_var("SMART_POOL_MAX_PLIES", previous);
    } else {
        env::remove_var("SMART_POOL_MAX_PLIES");
    }

    assert!(
        !ranked.is_empty(),
        "all profiles were dropped by speed gate; widen SMART_FAST_SPEED_RATIO_MAX"
    );

    ranked.sort_by(|a, b| {
        b.5.aggregate_stats
            .win_rate_points()
            .partial_cmp(&a.5.aggregate_stats.win_rate_points())
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                b.5.aggregate_confidence
                    .partial_cmp(&a.5.aggregate_confidence)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal))
            .then_with(|| a.3.partial_cmp(&b.3).unwrap_or(Ordering::Equal))
    });

    println!(
        "fast pipeline summary: baseline={} games_per_matchup={} opponents={} max_plies={} budgets={:?}",
        baseline_profile,
        games_per_matchup,
        pool.len(),
        max_plies,
        budgets.iter().map(|budget| budget.key()).collect::<Vec<_>>()
    );
    for (
        rank,
        (
            profile_name,
            avg_speed_ms,
            speed_ratio,
            max_mode_speed_ratio,
            mode_ratio_summary,
            evaluation,
        ),
    ) in ranked.iter().enumerate()
    {
        println!(
            "  rank={} profile={} win_rate={:.3} confidence={:.3} beaten={}/{} speed_ms={:.2} speed_ratio={:.2} max_mode_ratio={:.2} mode_ratios=[{}]",
            rank + 1,
            profile_name,
            evaluation.aggregate_stats.win_rate_points(),
            evaluation.aggregate_confidence,
            evaluation.beaten_opponents,
            evaluation.opponents.len(),
            avg_speed_ms,
            speed_ratio,
            max_mode_speed_ratio,
            mode_ratio_summary
        );
    }
}

#[test]
#[ignore = "diagnostic run for understanding tournament runtime/game lengths"]
fn smart_automove_pool_runtime_diagnostics() {
    let games = env_usize("SMART_DIAG_GAMES").unwrap_or(4).max(1);
    let budget = if let Some(preference) = env_automove_preference("SMART_DIAG_MODE") {
        SearchBudget::from_preference(preference)
    } else {
        SearchBudget {
            label: "custom",
            depth: env_usize("SMART_DIAG_DEPTH")
                .map(|value| value as i32)
                .unwrap_or(3),
            max_nodes: env_usize("SMART_DIAG_NODES")
                .map(|value| value as i32)
                .unwrap_or(2300),
        }
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
        "diag summary: games={} avg_plies={:.2} mode={} depth={} max_nodes={}",
        games,
        ply_sum as f64 / games as f64,
        budget.key(),
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
    let client_budgets = client_budgets();

    println!(
        "speed probe: profile={} positions={} modes={:?}",
        profile,
        positions,
        client_budgets
            .iter()
            .map(|budget| budget.key().to_string())
            .collect::<Vec<_>>()
    );

    for budget in client_budgets.iter().copied() {
        let start = Instant::now();
        let mut total_moves = 0usize;

        for opening in &openings {
            let game = MonsGame::from_fen(opening, false).expect("valid opening fen");
            let config = budget.runtime_config_for_game(&game);
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

#[test]
#[ignore = "head-to-head duel between two profile selectors across modes"]
fn smart_automove_pool_profile_duel() {
    let profile_a = env::var("SMART_DUEL_A")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "base".to_string());
    let profile_b = env::var("SMART_DUEL_B")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "runtime_d2_tuned".to_string());
    let games_per_mode = env_usize("SMART_DUEL_GAMES").unwrap_or(4).max(1);
    let repeats = env_usize("SMART_DUEL_REPEATS").unwrap_or(1).max(1);
    let max_plies = env_usize("SMART_DUEL_MAX_PLIES")
        .or_else(|| env_usize("SMART_POOL_MAX_PLIES"))
        .unwrap_or(96)
        .max(32);
    let duel_seed_tag = env::var("SMART_DUEL_SEED_TAG")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let use_client_modes = env_bool("SMART_DUEL_USE_CLIENT_MODES").unwrap_or(true);
    let explicit_mode = env_automove_preference("SMART_DUEL_MODE");

    let Some(selector_a) = profile_selector_from_name(profile_a.as_str()) else {
        panic!("unknown profile for SMART_DUEL_A: {}", profile_a);
    };
    let Some(selector_b) = profile_selector_from_name(profile_b.as_str()) else {
        panic!("unknown profile for SMART_DUEL_B: {}", profile_b);
    };

    let model_a = AutomoveModel {
        id: "duel_a",
        select_inputs: selector_a,
    };
    let model_b = AutomoveModel {
        id: "duel_b",
        select_inputs: selector_b,
    };

    let budgets = if let Some(mode) = explicit_mode {
        vec![SearchBudget::from_preference(mode)]
    } else if use_client_modes {
        client_budgets().to_vec()
    } else {
        vec![SearchBudget::from_preference(SmartAutomovePreference::Fast)]
    };

    let original_max_plies = env::var("SMART_POOL_MAX_PLIES").ok();
    env::set_var("SMART_POOL_MAX_PLIES", max_plies.to_string());

    let mut aggregate = MatchupStats::default();
    for budget in budgets.iter().copied() {
        let mut mode_stats = MatchupStats::default();
        for repeat_index in 0..repeats {
            let seed = if let Some(seed_tag) = duel_seed_tag.as_deref() {
                seed_for_budget_repeat_and_tag(budget, repeat_index, seed_tag)
            } else {
                seed_for_pairing_budget_and_repeat(model_a.id, model_b.id, budget, repeat_index)
            };
            let stats = run_matchup_series(model_a, model_b, games_per_mode, budget, seed);
            mode_stats.merge(stats);
        }
        aggregate.merge(mode_stats);
        println!(
            "duel mode {}: a={} b={} wins={} losses={} draws={} win_rate={:.3} confidence={:.3}",
            budget.key(),
            profile_a,
            profile_b,
            mode_stats.wins,
            mode_stats.losses,
            mode_stats.draws,
            mode_stats.win_rate_points(),
            mode_stats.confidence_better_than_even(),
        );
    }

    if let Some(previous) = original_max_plies {
        env::set_var("SMART_POOL_MAX_PLIES", previous);
    } else {
        env::remove_var("SMART_POOL_MAX_PLIES");
    }

    println!(
        "duel summary: modes={:?} repeats={} games_per_mode={} max_plies={} a={} b={} wins={} losses={} draws={} win_rate={:.3} confidence={:.3}",
        budgets.iter().map(|budget| budget.key()).collect::<Vec<_>>(),
        repeats,
        games_per_mode,
        max_plies,
        profile_a,
        profile_b,
        aggregate.wins,
        aggregate.losses,
        aggregate.draws,
        aggregate.win_rate_points(),
        aggregate.confidence_better_than_even(),
    );
}

#[test]
#[ignore = "head-to-head duel between fast and normal budgets"]
fn smart_automove_pool_budget_duel() {
    let profile_a = env::var("SMART_BUDGET_DUEL_A")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "runtime_current".to_string());
    let profile_b = env::var("SMART_BUDGET_DUEL_B")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| profile_a.clone());
    let mode_a = env_automove_preference("SMART_BUDGET_DUEL_A_MODE")
        .unwrap_or(SmartAutomovePreference::Fast);
    let mode_b = env_automove_preference("SMART_BUDGET_DUEL_B_MODE")
        .unwrap_or(SmartAutomovePreference::Normal);
    let games_per_repeat = env_usize("SMART_BUDGET_DUEL_GAMES").unwrap_or(4).max(1);
    let repeats = env_usize("SMART_BUDGET_DUEL_REPEATS").unwrap_or(1).max(1);
    let max_plies = env_usize("SMART_BUDGET_DUEL_MAX_PLIES")
        .or_else(|| env_usize("SMART_POOL_MAX_PLIES"))
        .unwrap_or(96)
        .max(32);
    let duel_seed_tag = env::var("SMART_BUDGET_DUEL_SEED_TAG")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let Some(selector_a) = profile_selector_from_name(profile_a.as_str()) else {
        panic!("unknown profile for SMART_BUDGET_DUEL_A: {}", profile_a);
    };
    let Some(selector_b) = profile_selector_from_name(profile_b.as_str()) else {
        panic!("unknown profile for SMART_BUDGET_DUEL_B: {}", profile_b);
    };

    let model_a = AutomoveModel {
        id: "budget_duel_a",
        select_inputs: selector_a,
    };
    let model_b = AutomoveModel {
        id: "budget_duel_b",
        select_inputs: selector_b,
    };
    let budget_a = SearchBudget::from_preference(mode_a);
    let budget_b = SearchBudget::from_preference(mode_b);

    let mut aggregate = MatchupStats::default();
    for repeat_index in 0..repeats {
        let seed = if let Some(seed_tag) = duel_seed_tag.as_deref() {
            seed_for_budget_duel_repeat_and_tag(budget_a, budget_b, repeat_index, seed_tag)
        } else {
            seed_for_budget_duel_pairing_and_repeat(
                model_a.id,
                model_b.id,
                budget_a,
                budget_b,
                repeat_index,
            )
        };
        let stats = run_budget_duel_series(
            model_a,
            budget_a,
            model_b,
            budget_b,
            games_per_repeat,
            seed,
            max_plies,
        );
        aggregate.merge(stats);
    }

    println!(
        "budget duel summary: a={}({}:{}/{}) b={}({}:{}/{}) repeats={} games_per_repeat={} max_plies={} wins={} losses={} draws={} win_rate={:.3} confidence={:.3}",
        profile_a,
        mode_a.as_api_value(),
        budget_a.depth,
        budget_a.max_nodes,
        profile_b,
        mode_b.as_api_value(),
        budget_b.depth,
        budget_b.max_nodes,
        repeats,
        games_per_repeat,
        max_plies,
        aggregate.wins,
        aggregate.losses,
        aggregate.draws,
        aggregate.win_rate_points(),
        aggregate.confidence_better_than_even(),
    );
}

fn mirrored_candidate_stats(ab: MatchupStats, ba: MatchupStats) -> MatchupStats {
    MatchupStats {
        wins: ab.wins + ba.losses,
        losses: ab.losses + ba.wins,
        draws: ab.draws + ba.draws,
    }
}

fn run_mirrored_duel_for_seed_tag(
    candidate: AutomoveModel,
    baseline: AutomoveModel,
    budgets: &[SearchBudget],
    seed_tag: &str,
    repeats: usize,
    games_per_mode: usize,
    max_plies: usize,
    use_white_opening_book: bool,
) -> Vec<(SearchBudget, MatchupStats)> {
    let original_max_plies = env::var("SMART_POOL_MAX_PLIES").ok();
    let original_opening_book = env::var("SMART_USE_WHITE_OPENING_BOOK").ok();
    env::set_var("SMART_POOL_MAX_PLIES", max_plies.to_string());
    env::set_var(
        "SMART_USE_WHITE_OPENING_BOOK",
        if use_white_opening_book {
            "true"
        } else {
            "false"
        },
    );

    let mut results = Vec::with_capacity(budgets.len());
    for budget in budgets.iter().copied() {
        let mut aggregate = MatchupStats::default();
        for repeat_index in 0..repeats {
            let seed = seed_for_budget_repeat_and_tag(budget, repeat_index, seed_tag);
            let ab = run_matchup_series(candidate, baseline, games_per_mode, budget, seed);
            let ba = run_matchup_series(baseline, candidate, games_per_mode, budget, seed);
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

fn merge_mode_stats(
    target: &mut std::collections::HashMap<&'static str, MatchupStats>,
    updates: &[(SearchBudget, MatchupStats)],
) {
    for (budget, stats) in updates {
        let entry = target.entry(budget.key()).or_default();
        entry.merge(*stats);
    }
}

fn max_achievable_delta(current: MatchupStats, remaining_games: usize) -> f64 {
    let total_games = current.total_games() + remaining_games;
    if total_games == 0 {
        return 0.0;
    }
    let max_wins = current.wins + remaining_games;
    let best_case_win_rate = (max_wins as f64 + 0.5 * current.draws as f64) / total_games as f64;
    best_case_win_rate - 0.5
}

fn run_pool_non_regression_check(
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

fn persist_ladder_artifacts(lines: &[String]) {
    let Some(path) = env::var("SMART_LADDER_ARTIFACT_PATH")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return;
    };

    let payload = lines.join("\n");
    if let Err(err) = std::fs::write(path.as_str(), payload.as_bytes()) {
        panic!(
            "failed writing SMART_LADDER_ARTIFACT_PATH '{}': {}",
            path, err
        );
    }
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn eval_phase_family_for_game(game: &MonsGame, budget: SearchBudget) -> EvalPhaseFamily {
    if budget.key() == "fast" {
        return EvalPhaseFamily::Fast;
    }

    let (my_score, opponent_score) = if game.active_color == Color::White {
        (game.white_score, game.black_score)
    } else {
        (game.black_score, game.white_score)
    };
    let my_distance_to_win = Config::TARGET_SCORE - my_score;
    let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
    let score_gap = my_score - opponent_score;

    if my_distance_to_win <= 1 {
        EvalPhaseFamily::NormalFinisherAggressive
    } else if opponent_distance_to_win <= 1 {
        EvalPhaseFamily::NormalTacticalAggressive
    } else if my_distance_to_win <= 2 {
        EvalPhaseFamily::NormalFinisher
    } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
        EvalPhaseFamily::NormalTactical
    } else {
        EvalPhaseFamily::NormalBalanced
    }
}

fn eval_phase_family_base_weights(family: EvalPhaseFamily) -> &'static ScoringWeights {
    match family {
        EvalPhaseFamily::Fast => &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS,
        EvalPhaseFamily::NormalBalanced => {
            &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
        EvalPhaseFamily::NormalTactical => {
            &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        }
        EvalPhaseFamily::NormalTacticalAggressive => {
            &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        }
        EvalPhaseFamily::NormalFinisher => {
            &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        }
        EvalPhaseFamily::NormalFinisherAggressive => {
            &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }
}

fn eval_tuning_sample_json_line(profile_name: &str, sample: &EvalTuningSample) -> String {
    format!(
        concat!(
            r#"{{"profile":"{}","mode":"{}","phase":"{}","fen":"{}","input":"{}","after_fen":"{}","selected":{},"score":{},"best_score":{},"regret":{},"efficiency":{},"attack_drainer":{},"own_drainer_vulnerable":{},"mana_handoff":{},"roundtrip":{},"class_immediate_score":{},"class_drainer_attack":{},"class_drainer_safety_recover":{},"class_carrier_progress":{},"class_material":{},"class_quiet":{},"#,
            r#""eval_total":{},"eval_confirmed_score":{},"eval_consumable_score":{},"eval_score_race_path":{},"eval_opp_score_race_path":{},"eval_score_race_multi":{},"eval_opp_score_race_multi":{},"eval_immediate_window":{},"eval_opp_immediate_window":{},"eval_immediate_multi":{},"eval_opp_immediate_multi":{},"eval_match_point":{},"eval_residual":{}}}"#
        ),
        json_escape(profile_name),
        sample.mode,
        sample.phase_family.as_str(),
        json_escape(sample.fen.as_str()),
        json_escape(sample.input.as_str()),
        json_escape(sample.after_fen.as_str()),
        if sample.selected { "true" } else { "false" },
        sample.score,
        sample.best_score,
        sample.regret,
        sample.efficiency,
        if sample.attack_drainer {
            "true"
        } else {
            "false"
        },
        if sample.own_drainer_vulnerable {
            "true"
        } else {
            "false"
        },
        if sample.mana_handoff { "true" } else { "false" },
        if sample.roundtrip { "true" } else { "false" },
        if sample.class_immediate_score {
            "true"
        } else {
            "false"
        },
        if sample.class_drainer_attack {
            "true"
        } else {
            "false"
        },
        if sample.class_drainer_safety_recover {
            "true"
        } else {
            "false"
        },
        if sample.class_carrier_progress {
            "true"
        } else {
            "false"
        },
        if sample.class_material {
            "true"
        } else {
            "false"
        },
        if sample.class_quiet { "true" } else { "false" },
        sample.eval_total,
        sample.eval_confirmed_score,
        sample.eval_consumable_score,
        sample.eval_score_race_path,
        sample.eval_opp_score_race_path,
        sample.eval_score_race_multi,
        sample.eval_opp_score_race_multi,
        sample.eval_immediate_window,
        sample.eval_opp_immediate_window,
        sample.eval_immediate_multi,
        sample.eval_opp_immediate_multi,
        sample.eval_match_point,
        sample.eval_residual,
    )
}

fn collect_eval_tuning_samples_for_budget(
    _profile_name: &str,
    selector: fn(&MonsGame, SmartSearchConfig) -> Vec<Input>,
    budget: SearchBudget,
    openings: &[String],
    root_limit: usize,
) -> Vec<EvalTuningSample> {
    let mut samples = Vec::new();
    for opening in openings {
        let game = MonsGame::from_fen(opening.as_str(), false).expect("valid opening fen");
        if game.winner_color().is_some() {
            continue;
        }

        let config = budget.runtime_config_for_game(&game);
        let selected_inputs = selector(&game, config);
        let selected_fen = Input::fen_from_array(&selected_inputs);
        let phase_family = eval_phase_family_for_game(&game, budget);
        let label_depth_boost = env_usize("SMART_TUNE_LABEL_DEPTH_BOOST").unwrap_or(0);
        let label_node_multiplier = env_usize("SMART_TUNE_LABEL_NODE_MULTIPLIER").unwrap_or(1);
        let mut label_config = config;
        if label_depth_boost > 0 {
            label_config.depth =
                (config.depth + label_depth_boost).clamp(1, MAX_SMART_SEARCH_DEPTH);
        }
        if label_node_multiplier > 1 {
            label_config.max_visited_nodes = (config.max_visited_nodes * label_node_multiplier)
                .clamp(config.max_visited_nodes, MAX_SMART_MAX_VISITED_NODES);
            label_config.root_branch_limit = (config.root_branch_limit + label_node_multiplier * 2)
                .clamp(config.root_branch_limit, 48);
            label_config.node_branch_limit = (config.node_branch_limit + label_node_multiplier)
                .clamp(config.node_branch_limit, 24);
            label_config.root_enum_limit =
                (label_config.root_branch_limit * 6).clamp(label_config.root_branch_limit, 320);
            label_config.node_enum_limit =
                (label_config.node_branch_limit * 5).clamp(label_config.node_branch_limit, 180);
        }

        let mut roots = MonsGameModel::ranked_root_moves(&game, game.active_color, config);
        if roots.is_empty() {
            continue;
        }
        roots.truncate(root_limit.max(1));

        let mut visited_nodes = 0usize;
        let mut alpha = i32::MIN;
        let mut transposition_table = std::collections::HashMap::new();
        let extension_node_budget =
            if config.enable_selective_extensions && config.selective_extension_node_share_bp > 0 {
                ((config.max_visited_nodes * config.selective_extension_node_share_bp as usize)
                    / 10_000)
                    .max(1)
            } else {
                0
            };
        let mut extension_nodes_used = 0usize;
        let mut scored_roots = Vec::new();

        for root in &roots {
            if visited_nodes >= config.max_visited_nodes {
                break;
            }
            visited_nodes += 1;
            let score = MonsGameModel::evaluate_root_candidate_score(
                root,
                game.active_color,
                alpha,
                &mut visited_nodes,
                label_config,
                &mut transposition_table,
                &mut extension_nodes_used,
                extension_node_budget,
                true,
            );
            alpha = alpha.max(score);
            let eval_breakdown = evaluate_preferability_breakdown_with_weights(
                &root.game,
                game.active_color,
                config.scoring_weights,
            );
            scored_roots.push((
                Input::fen_from_array(root.inputs.as_slice()),
                score,
                root.game.fen(),
                root.game.clone(),
                root.efficiency,
                root.attacks_opponent_drainer,
                root.own_drainer_vulnerable,
                root.mana_handoff_to_opponent,
                root.has_roundtrip,
                root.classes,
                eval_breakdown,
            ));
        }
        if scored_roots.is_empty() {
            continue;
        }

        let best_score = scored_roots
            .iter()
            .map(|(_, score, ..)| *score)
            .max()
            .unwrap_or(i32::MIN);
        for (
            input_fen,
            score,
            after_fen,
            after_game,
            efficiency,
            attacks_opponent_drainer,
            own_drainer_vulnerable,
            mana_handoff_to_opponent,
            has_roundtrip,
            classes,
            eval_breakdown,
        ) in scored_roots
        {
            let regret = best_score.saturating_sub(score);
            let selected = input_fen == selected_fen;
            samples.push(EvalTuningSample {
                mode: budget.key(),
                phase_family,
                fen: opening.clone(),
                input: input_fen,
                after_fen,
                after_game,
                perspective: game.active_color,
                selected,
                score,
                best_score,
                regret,
                efficiency,
                attack_drainer: attacks_opponent_drainer,
                own_drainer_vulnerable,
                mana_handoff: mana_handoff_to_opponent,
                roundtrip: has_roundtrip,
                class_immediate_score: classes.immediate_score,
                class_drainer_attack: classes.drainer_attack,
                class_drainer_safety_recover: classes.drainer_safety_recover,
                class_carrier_progress: classes.carrier_progress,
                class_material: classes.material,
                class_quiet: classes.quiet,
                eval_total: eval_breakdown.total,
                eval_confirmed_score: eval_breakdown.terms.confirmed_score,
                eval_consumable_score: eval_breakdown.terms.consumable_score,
                eval_score_race_path: eval_breakdown.terms.score_race_path_progress,
                eval_opp_score_race_path: eval_breakdown.terms.opponent_score_race_path_progress,
                eval_score_race_multi: eval_breakdown.terms.score_race_multi_path,
                eval_opp_score_race_multi: eval_breakdown.terms.opponent_score_race_multi_path,
                eval_immediate_window: eval_breakdown.terms.immediate_score_window,
                eval_opp_immediate_window: eval_breakdown.terms.opponent_immediate_score_window,
                eval_immediate_multi: eval_breakdown.terms.immediate_score_multi_window,
                eval_opp_immediate_multi: eval_breakdown
                    .terms
                    .opponent_immediate_score_multi_window,
                eval_match_point: eval_breakdown.terms.match_point_window,
                eval_residual: eval_breakdown.terms.residual_board_state,
            });
        }
    }

    samples
}

#[derive(Clone, Copy)]
struct EvalTuneBundle {
    fast: ScoringWeights,
    normal_balanced: ScoringWeights,
    normal_tactical: ScoringWeights,
    normal_tactical_aggressive: ScoringWeights,
    normal_finisher: ScoringWeights,
    normal_finisher_aggressive: ScoringWeights,
}

impl EvalTuneBundle {
    fn baseline() -> Self {
        Self {
            fast: RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS,
            normal_balanced: RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS,
            normal_tactical: RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS,
            normal_tactical_aggressive:
                RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS,
            normal_finisher: RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS,
            normal_finisher_aggressive:
                RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS,
        }
    }

    fn weights_for_family(self, family: EvalPhaseFamily) -> ScoringWeights {
        match family {
            EvalPhaseFamily::Fast => self.fast,
            EvalPhaseFamily::NormalBalanced => self.normal_balanced,
            EvalPhaseFamily::NormalTactical => self.normal_tactical,
            EvalPhaseFamily::NormalTacticalAggressive => self.normal_tactical_aggressive,
            EvalPhaseFamily::NormalFinisher => self.normal_finisher,
            EvalPhaseFamily::NormalFinisherAggressive => self.normal_finisher_aggressive,
        }
    }

    fn with_family_weights(mut self, family: EvalPhaseFamily, weights: ScoringWeights) -> Self {
        match family {
            EvalPhaseFamily::Fast => self.fast = weights,
            EvalPhaseFamily::NormalBalanced => self.normal_balanced = weights,
            EvalPhaseFamily::NormalTactical => self.normal_tactical = weights,
            EvalPhaseFamily::NormalTacticalAggressive => self.normal_tactical_aggressive = weights,
            EvalPhaseFamily::NormalFinisher => self.normal_finisher = weights,
            EvalPhaseFamily::NormalFinisherAggressive => self.normal_finisher_aggressive = weights,
        }
        self
    }
}

#[derive(Clone, Copy)]
enum EvalTuneField {
    ScoreRacePathProgress,
    OpponentScoreRacePathProgress,
    ScoreRaceMultiPath,
    OpponentScoreRaceMultiPath,
    ImmediateScoreWindow,
    OpponentImmediateScoreWindow,
    ImmediateScoreMultiWindow,
    OpponentImmediateScoreMultiWindow,
    SpiritActionUtility,
    DrainerImmediateThreat,
    DrainerBestManaPath,
    DrainerPickupScoreThisTurn,
    ManaCarrierScoreThisTurn,
    NextTurnWindowScaleBp,
    IncludeRegularManaMoveWindows,
    IncludeMatchPointWindow,
    DoubleConfirmedScore,
}

const EVAL_TUNE_FIELDS: [EvalTuneField; 17] = [
    EvalTuneField::ScoreRacePathProgress,
    EvalTuneField::OpponentScoreRacePathProgress,
    EvalTuneField::ScoreRaceMultiPath,
    EvalTuneField::OpponentScoreRaceMultiPath,
    EvalTuneField::ImmediateScoreWindow,
    EvalTuneField::OpponentImmediateScoreWindow,
    EvalTuneField::ImmediateScoreMultiWindow,
    EvalTuneField::OpponentImmediateScoreMultiWindow,
    EvalTuneField::SpiritActionUtility,
    EvalTuneField::DrainerImmediateThreat,
    EvalTuneField::DrainerBestManaPath,
    EvalTuneField::DrainerPickupScoreThisTurn,
    EvalTuneField::ManaCarrierScoreThisTurn,
    EvalTuneField::NextTurnWindowScaleBp,
    EvalTuneField::IncludeRegularManaMoveWindows,
    EvalTuneField::IncludeMatchPointWindow,
    EvalTuneField::DoubleConfirmedScore,
];

fn eval_tune_field_is_bool(field: EvalTuneField) -> bool {
    matches!(
        field,
        EvalTuneField::IncludeRegularManaMoveWindows
            | EvalTuneField::IncludeMatchPointWindow
            | EvalTuneField::DoubleConfirmedScore
    )
}

fn eval_tune_field_non_negative(field: EvalTuneField) -> bool {
    !matches!(field, EvalTuneField::DrainerImmediateThreat)
}

fn eval_tune_field_step(field: EvalTuneField, base: i32) -> i32 {
    if matches!(field, EvalTuneField::NextTurnWindowScaleBp) {
        return 250;
    }
    if base.abs() >= 150 {
        10
    } else {
        5
    }
}

fn eval_tune_get_numeric(weights: &ScoringWeights, field: EvalTuneField) -> i32 {
    match field {
        EvalTuneField::ScoreRacePathProgress => weights.score_race_path_progress,
        EvalTuneField::OpponentScoreRacePathProgress => weights.opponent_score_race_path_progress,
        EvalTuneField::ScoreRaceMultiPath => weights.score_race_multi_path,
        EvalTuneField::OpponentScoreRaceMultiPath => weights.opponent_score_race_multi_path,
        EvalTuneField::ImmediateScoreWindow => weights.immediate_score_window,
        EvalTuneField::OpponentImmediateScoreWindow => weights.opponent_immediate_score_window,
        EvalTuneField::ImmediateScoreMultiWindow => weights.immediate_score_multi_window,
        EvalTuneField::OpponentImmediateScoreMultiWindow => {
            weights.opponent_immediate_score_multi_window
        }
        EvalTuneField::SpiritActionUtility => weights.spirit_action_utility,
        EvalTuneField::DrainerImmediateThreat => weights.drainer_immediate_threat,
        EvalTuneField::DrainerBestManaPath => weights.drainer_best_mana_path,
        EvalTuneField::DrainerPickupScoreThisTurn => weights.drainer_pickup_score_this_turn,
        EvalTuneField::ManaCarrierScoreThisTurn => weights.mana_carrier_score_this_turn,
        EvalTuneField::NextTurnWindowScaleBp => weights.next_turn_window_scale_bp,
        EvalTuneField::IncludeRegularManaMoveWindows => {
            if weights.include_regular_mana_move_windows {
                1
            } else {
                0
            }
        }
        EvalTuneField::IncludeMatchPointWindow => {
            if weights.include_match_point_window {
                1
            } else {
                0
            }
        }
        EvalTuneField::DoubleConfirmedScore => {
            if weights.double_confirmed_score {
                1
            } else {
                0
            }
        }
    }
}

fn eval_tune_set_field(weights: &mut ScoringWeights, field: EvalTuneField, value: i32) {
    match field {
        EvalTuneField::ScoreRacePathProgress => weights.score_race_path_progress = value,
        EvalTuneField::OpponentScoreRacePathProgress => {
            weights.opponent_score_race_path_progress = value
        }
        EvalTuneField::ScoreRaceMultiPath => weights.score_race_multi_path = value,
        EvalTuneField::OpponentScoreRaceMultiPath => weights.opponent_score_race_multi_path = value,
        EvalTuneField::ImmediateScoreWindow => weights.immediate_score_window = value,
        EvalTuneField::OpponentImmediateScoreWindow => {
            weights.opponent_immediate_score_window = value
        }
        EvalTuneField::ImmediateScoreMultiWindow => weights.immediate_score_multi_window = value,
        EvalTuneField::OpponentImmediateScoreMultiWindow => {
            weights.opponent_immediate_score_multi_window = value
        }
        EvalTuneField::SpiritActionUtility => weights.spirit_action_utility = value,
        EvalTuneField::DrainerImmediateThreat => weights.drainer_immediate_threat = value,
        EvalTuneField::DrainerBestManaPath => weights.drainer_best_mana_path = value,
        EvalTuneField::DrainerPickupScoreThisTurn => weights.drainer_pickup_score_this_turn = value,
        EvalTuneField::ManaCarrierScoreThisTurn => weights.mana_carrier_score_this_turn = value,
        EvalTuneField::NextTurnWindowScaleBp => {
            weights.next_turn_window_scale_bp = value.clamp(0, 20_000)
        }
        EvalTuneField::IncludeRegularManaMoveWindows => {
            weights.include_regular_mana_move_windows = value != 0
        }
        EvalTuneField::IncludeMatchPointWindow => weights.include_match_point_window = value != 0,
        EvalTuneField::DoubleConfirmedScore => weights.double_confirmed_score = value != 0,
    }
}

fn eval_tune_group_indices(
    samples: &[EvalTuningSample],
    family: EvalPhaseFamily,
) -> Vec<Vec<usize>> {
    let mut grouped = std::collections::BTreeMap::<String, Vec<usize>>::new();
    for (index, sample) in samples.iter().enumerate() {
        if sample.phase_family != family {
            continue;
        }
        let key = format!("{}|{}", sample.mode, sample.fen);
        grouped.entry(key).or_default().push(index);
    }
    grouped.into_values().collect()
}

fn eval_tune_objective(
    samples: &[EvalTuningSample],
    groups: &[Vec<usize>],
    weights: &ScoringWeights,
    base: &ScoringWeights,
) -> f64 {
    if groups.is_empty() {
        return f64::NEG_INFINITY;
    }

    let mut loss = 0.0f64;
    for group in groups {
        let mut evals = Vec::with_capacity(group.len());
        for index in group {
            let sample = &samples[*index];
            let eval = evaluate_preferability_with_weights(
                &sample.after_game,
                sample.perspective,
                weights,
            );
            evals.push((*index, eval));
        }

        let mut best_index = evals[0].0;
        let mut best_eval = evals[0].1;
        let mut best_regret = samples[best_index].regret;
        for (index, eval) in &evals {
            let sample = &samples[*index];
            if *eval > best_eval || (*eval == best_eval && sample.regret < best_regret) {
                best_eval = *eval;
                best_regret = sample.regret;
                best_index = *index;
            }
        }

        let chosen = &samples[best_index];
        loss += chosen.regret as f64;

        let any_drainer_attack = group
            .iter()
            .any(|index| samples[*index].class_drainer_attack);
        let any_immediate = group
            .iter()
            .any(|index| samples[*index].class_immediate_score);
        let any_safety = group
            .iter()
            .any(|index| samples[*index].class_drainer_safety_recover);

        if any_drainer_attack && !chosen.class_drainer_attack {
            loss += 20.0;
        }
        if any_immediate && !chosen.class_immediate_score {
            loss += 14.0;
        }
        if any_safety && !chosen.class_drainer_safety_recover {
            loss += 8.0;
        }

        let has_non_handoff_alt = group.iter().any(|index| {
            let candidate = &samples[*index];
            !candidate.mana_handoff && candidate.regret <= chosen.regret
        });
        if chosen.mana_handoff && has_non_handoff_alt {
            loss += 5.0;
        }

        let has_non_roundtrip_alt = group.iter().any(|index| {
            let candidate = &samples[*index];
            !candidate.roundtrip && candidate.regret <= chosen.regret
        });
        if chosen.roundtrip && has_non_roundtrip_alt {
            loss += 3.0;
        }

        let has_safer_alt = group.iter().any(|index| {
            let candidate = &samples[*index];
            !candidate.own_drainer_vulnerable && candidate.regret <= chosen.regret + 1
        });
        if chosen.own_drainer_vulnerable && has_safer_alt {
            loss += 6.0;
        }

        for i in 0..evals.len() {
            for j in (i + 1)..evals.len() {
                let (index_i, eval_i) = evals[i];
                let (index_j, eval_j) = evals[j];
                let label_diff = samples[index_i].score - samples[index_j].score;
                if label_diff == 0 {
                    continue;
                }
                let eval_diff = eval_i - eval_j;
                if eval_diff == 0
                    || (label_diff > 0 && eval_diff < 0)
                    || (label_diff < 0 && eval_diff > 0)
                {
                    loss += 0.35;
                }
            }
        }
    }

    for field in EVAL_TUNE_FIELDS {
        let base_value = eval_tune_get_numeric(base, field);
        let value = eval_tune_get_numeric(weights, field);
        let delta = (value - base_value).abs() as f64;
        let denom = base_value.abs().max(5) as f64;
        if eval_tune_field_is_bool(field) {
            if value != base_value {
                loss += 0.20;
            }
        } else {
            loss += 0.01 * (delta / denom);
        }
    }

    -loss
}

fn tune_weights_for_family(
    train_samples: &[EvalTuningSample],
    train_groups: &[Vec<usize>],
    base: ScoringWeights,
    rotation: usize,
) -> ScoringWeights {
    let mut tuned = base;
    let mut best_score = eval_tune_objective(train_samples, train_groups, &tuned, &base);
    let mut fields = EVAL_TUNE_FIELDS.to_vec();
    let fields_len = fields.len().max(1);
    fields.rotate_left(rotation % fields_len);
    let full_grid = env_bool("SMART_TUNE_FULL_GRID").unwrap_or(false);

    for _ in 0..3 {
        let mut improved = false;
        for field in &fields {
            if eval_tune_field_is_bool(*field) {
                let current = eval_tune_get_numeric(&tuned, *field);
                let mut trial = tuned;
                eval_tune_set_field(&mut trial, *field, if current == 0 { 1 } else { 0 });
                let trial_score = eval_tune_objective(train_samples, train_groups, &trial, &base);
                if trial_score > best_score {
                    best_score = trial_score;
                    tuned = trial;
                    improved = true;
                }
                continue;
            }

            let current = eval_tune_get_numeric(&tuned, *field);
            let base_value = eval_tune_get_numeric(&base, *field);
            let step = eval_tune_field_step(*field, base_value).max(1);
            let span = ((base_value.abs() * 35) / 100).max(step);
            let mut min_value = base_value - span;
            let mut max_value = base_value + span;
            if eval_tune_field_non_negative(*field) {
                min_value = min_value.max(0);
            }
            if matches!(field, EvalTuneField::NextTurnWindowScaleBp) {
                min_value = min_value.clamp(0, 20_000);
                max_value = max_value.clamp(0, 20_000);
            }

            if full_grid {
                let mut candidate = min_value;
                while candidate <= max_value {
                    if candidate == current {
                        candidate += step;
                        continue;
                    }
                    let mut trial = tuned;
                    eval_tune_set_field(&mut trial, *field, candidate);
                    let trial_score =
                        eval_tune_objective(train_samples, train_groups, &trial, &base);
                    if trial_score > best_score {
                        best_score = trial_score;
                        tuned = trial;
                        improved = true;
                    }
                    candidate += step;
                }
            } else {
                let mut candidates = vec![
                    base_value,
                    current - 2 * step,
                    current - step,
                    current + step,
                    current + 2 * step,
                ];
                candidates.sort();
                candidates.dedup();
                for candidate in candidates {
                    let candidate = candidate.clamp(min_value, max_value);
                    if candidate == current {
                        continue;
                    }
                    let mut trial = tuned;
                    eval_tune_set_field(&mut trial, *field, candidate);
                    let trial_score =
                        eval_tune_objective(train_samples, train_groups, &trial, &base);
                    if trial_score > best_score {
                        best_score = trial_score;
                        tuned = trial;
                        improved = true;
                    }
                }
            }
        }
        if !improved {
            break;
        }
    }

    tuned
}

fn eval_tune_bundle_score(
    samples: &[EvalTuningSample],
    groups_by_family: &std::collections::HashMap<EvalPhaseFamily, Vec<Vec<usize>>>,
    bundle: EvalTuneBundle,
) -> f64 {
    let mut score = 0.0;
    for family in [
        EvalPhaseFamily::Fast,
        EvalPhaseFamily::NormalBalanced,
        EvalPhaseFamily::NormalTactical,
        EvalPhaseFamily::NormalTacticalAggressive,
        EvalPhaseFamily::NormalFinisher,
        EvalPhaseFamily::NormalFinisherAggressive,
    ] {
        let groups = groups_by_family
            .get(&family)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if groups.is_empty() {
            continue;
        }
        let weights = bundle.weights_for_family(family);
        let base = eval_phase_family_base_weights(family);
        score += eval_tune_objective(samples, groups, &weights, base);
    }
    score
}

fn write_eval_tune_manifest(
    path: &str,
    ranked: &[(usize, f64, f64, EvalTuneBundle)],
) -> std::io::Result<()> {
    let mut lines = Vec::new();
    lines.push("[".to_string());
    for (index, (_candidate_id, train_score, holdout_score, bundle)) in ranked.iter().enumerate() {
        let comma = if index + 1 == ranked.len() { "" } else { "," };
        lines.push(format!(
            concat!(
                r#"  {{"rank":{},"train_score":{:.6},"holdout_score":{:.6},"#,
                r#""fast_imm":{},"fast_opp_imm":{},"fast_multi":{},"fast_opp_multi":{},"#,
                r#""fast_next_turn_bp":{},"fast_include_regular":{},"fast_include_match_point":{},"fast_double_confirmed":{},"fast_spirit_utility":{},"fast_drainer_threat":{},"#,
                r#""normal_balanced_imm":{},"normal_balanced_opp_imm":{},"#,
                r#""normal_balanced_next_turn_bp":{},"normal_balanced_include_regular":{},"normal_balanced_include_match_point":{},"normal_balanced_double_confirmed":{},"normal_balanced_spirit_utility":{},"normal_balanced_drainer_threat":{},"#,
                r#""normal_tactical_imm":{},"normal_tactical_opp_imm":{},"#,
                r#""normal_finisher_imm":{},"normal_finisher_opp_imm":{}}}"#
            ),
            index + 1,
            *train_score,
            *holdout_score,
            bundle.fast.immediate_score_window,
            bundle.fast.opponent_immediate_score_window,
            bundle.fast.immediate_score_multi_window,
            bundle.fast.opponent_immediate_score_multi_window,
            bundle.fast.next_turn_window_scale_bp,
            if bundle.fast.include_regular_mana_move_windows {
                "true"
            } else {
                "false"
            },
            if bundle.fast.include_match_point_window {
                "true"
            } else {
                "false"
            },
            if bundle.fast.double_confirmed_score {
                "true"
            } else {
                "false"
            },
            bundle.fast.spirit_action_utility,
            bundle.fast.drainer_immediate_threat,
            bundle.normal_balanced.immediate_score_window,
            bundle.normal_balanced.opponent_immediate_score_window,
            bundle.normal_balanced.next_turn_window_scale_bp,
            if bundle.normal_balanced.include_regular_mana_move_windows {
                "true"
            } else {
                "false"
            },
            if bundle.normal_balanced.include_match_point_window {
                "true"
            } else {
                "false"
            },
            if bundle.normal_balanced.double_confirmed_score {
                "true"
            } else {
                "false"
            },
            bundle.normal_balanced.spirit_action_utility,
            bundle.normal_balanced.drainer_immediate_threat,
            bundle.normal_tactical.immediate_score_window,
            bundle.normal_tactical.opponent_immediate_score_window,
            bundle.normal_finisher.immediate_score_window,
            bundle.normal_finisher.opponent_immediate_score_window,
        ));
        if !comma.is_empty() {
            let last = lines.pop().unwrap_or_default();
            lines.push(format!("{last}{comma}"));
        }
    }
    lines.push("]".to_string());
    std::fs::write(path, lines.join("\n").as_bytes())
}

fn collect_eval_tuning_samples_for_seed_tags(
    profile_name: &str,
    selector: fn(&MonsGame, SmartSearchConfig) -> Vec<Input>,
    seed_tags: &[&str],
    positions_per_seed: usize,
    root_limit: usize,
) -> Vec<EvalTuningSample> {
    let mut all = Vec::new();
    for seed_tag in seed_tags {
        let openings_seed = seed_for_pairing(seed_tag, profile_name);
        let openings = generate_opening_fens_cached(openings_seed, positions_per_seed.max(8));
        all.extend(collect_eval_tuning_samples_for_budget(
            profile_name,
            selector,
            SearchBudget::from_preference(SmartAutomovePreference::Fast),
            openings.as_slice(),
            root_limit,
        ));
        all.extend(collect_eval_tuning_samples_for_budget(
            profile_name,
            selector,
            SearchBudget::from_preference(SmartAutomovePreference::Normal),
            openings.as_slice(),
            root_limit,
        ));
    }
    all
}

fn tactical_game_with_items(
    items: Vec<(Location, Item)>,
    active_color: Color,
    turn_number: i32,
) -> MonsGame {
    let mut game = MonsGame::new(false);
    let board_items = items
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();
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

fn assert_tactical_guardrails(
    selector: fn(&MonsGame, SmartSearchConfig) -> Vec<Input>,
    profile_name: &str,
) {
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
    let drainer_attack_inputs = selector(&drainer_attack_game, drainer_attack_config);
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
    let mut bomb_drainer_attack_config =
        SearchBudget::from_preference(SmartAutomovePreference::Fast)
            .runtime_config_for_game(&bomb_drainer_attack_game);
    bomb_drainer_attack_config.root_enum_limit = 0;
    let bomb_drainer_attack_inputs =
        selector(&bomb_drainer_attack_game, bomb_drainer_attack_config);
    let (after_bomb_probe, bomb_drainer_attack_events) =
        MonsGameModel::apply_inputs_for_search_with_events(
            &bomb_drainer_attack_game,
            &bomb_drainer_attack_inputs,
        )
        .expect("bomb drainer attack move should be legal");
    let bomb_attacks_now = MonsGameModel::events_include_opponent_drainer_fainted(
        &bomb_drainer_attack_events,
        Color::White,
    );
    let mut bomb_continuation_budget = SMART_FORCED_DRAINER_ATTACK_FALLBACK_NODE_BUDGET_FAST;
    let bomb_attacks_later_this_turn = after_bomb_probe.active_color == Color::White
        && MonsGameModel::can_attack_opponent_drainer_before_turn_ends(
            &after_bomb_probe,
            Color::White,
            SMART_FORCED_DRAINER_ATTACK_FALLBACK_ENUM_LIMIT_FAST,
            &mut bomb_continuation_budget,
            &mut std::collections::HashSet::new(),
        );
    assert!(
        bomb_attacks_now || bomb_attacks_later_this_turn,
        "profile '{}' must take bomb-based drainer attack even when root enum misses it",
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
            vec![(
                location,
                Item::MonWithMana {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    mana: Mana::Regular(Color::Black),
                },
            )],
            Color::White,
            3,
        );
        probe.white_score = Config::TARGET_SCORE - 2;
        let has_immediate_win = MonsGameModel::enumerate_legal_inputs(&probe, 96)
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

    let mut winning_carrier_game =
        winning_carrier_game.expect("expected at least one immediate-winning carrier setup");
    let winning_config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
        .runtime_config_for_game(&winning_carrier_game);
    let winning_carrier_initial_fen = winning_carrier_game.fen();
    let winning_inputs = selector(&winning_carrier_game, winning_config);
    assert!(
        !winning_inputs.is_empty(),
        "profile '{}' should produce a move in immediate-win setup",
        profile_name
    );
    assert!(matches!(
        winning_carrier_game.process_input(winning_inputs, false, false),
        Output::Events(_)
    ));
    assert_eq!(
        winning_carrier_game.winner_color(),
        Some(Color::White),
        "profile '{}' should convert immediate winning carrier line",
        profile_name
    );

    let random_openings = generate_opening_fens(seed_for_pairing("tactical", "roundtrip"), 12);
    for opening in random_openings {
        let game = MonsGame::from_fen(opening.as_str(), false).expect("valid opening fen");
        let config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
            .runtime_config_for_game(&game);
        let selected_inputs = selector(&game, config);
        let Some((_, selected_events)) =
            MonsGameModel::apply_inputs_for_search_with_events(&game, &selected_inputs)
        else {
            continue;
        };
        if !MonsGameModel::has_roundtrip_mon_move(&selected_events) {
            continue;
        }

        let root_moves = MonsGameModel::ranked_root_moves(&game, game.active_color, config);
        let mut has_better_non_roundtrip = false;
        for root in root_moves {
            let Some((_, events)) =
                MonsGameModel::apply_inputs_for_search_with_events(&game, &root.inputs)
            else {
                continue;
            };
            if MonsGameModel::has_roundtrip_mon_move(&events) {
                continue;
            }
            if root.efficiency > 0 || MonsGameModel::has_material_event(&events) {
                has_better_non_roundtrip = true;
                break;
            }
        }

        assert!(
            !has_better_non_roundtrip,
            "profile '{}' selected roundtrip line while better non-roundtrip progress existed",
            profile_name
        );
    }

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
        let safety_inputs = selector(&drainer_safety_game, drainer_safety_config);
        let (safety_after, _) = MonsGameModel::apply_inputs_for_search_with_events(
            &drainer_safety_game,
            &safety_inputs,
        )
        .expect("drainer safety move should be legal");
        assert!(
            !MonsGameModel::is_own_drainer_vulnerable_next_turn(&safety_after, Color::White),
            "profile '{}' left drainer vulnerable while safe alternatives existed",
            profile_name
        );
    }

    let mana_handoff_to_opponent = |events: &[Event], perspective: Color| {
        let opponent = perspective.other();
        events.iter().any(|event| {
            let Event::ManaMove { from, to, .. } = event else {
                return false;
            };
            let my_before =
                MonsGameModel::distance_to_color_pool_steps_for_efficiency(*from, perspective);
            let my_after =
                MonsGameModel::distance_to_color_pool_steps_for_efficiency(*to, perspective);
            let opponent_before =
                MonsGameModel::distance_to_color_pool_steps_for_efficiency(*from, opponent);
            let opponent_after =
                MonsGameModel::distance_to_color_pool_steps_for_efficiency(*to, opponent);
            let moved_toward_opponent = (opponent_before - opponent_after).max(0);
            let moved_toward_me = (my_before - my_after).max(0);
            moved_toward_opponent > moved_toward_me
        })
    };
    let handoff_openings = generate_opening_fens(seed_for_pairing("tactical", "handoff"), 12);
    for opening in handoff_openings {
        let game = MonsGame::from_fen(opening.as_str(), false).expect("valid opening fen");
        let config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
            .runtime_config_for_game(&game);
        let selected_inputs = selector(&game, config);
        let Some((_, selected_events)) =
            MonsGameModel::apply_inputs_for_search_with_events(&game, &selected_inputs)
        else {
            continue;
        };
        if !mana_handoff_to_opponent(&selected_events, game.active_color) {
            continue;
        }

        let root_moves = MonsGameModel::ranked_root_moves(&game, game.active_color, config);
        let mut has_better_non_handoff = false;
        for root in root_moves {
            let Some((_, root_events)) =
                MonsGameModel::apply_inputs_for_search_with_events(&game, &root.inputs)
            else {
                continue;
            };
            if mana_handoff_to_opponent(&root_events, game.active_color) {
                continue;
            }
            if root.efficiency >= 0 || MonsGameModel::has_material_event(&root_events) {
                has_better_non_handoff = true;
                break;
            }
        }

        assert!(
            !has_better_non_handoff,
            "profile '{}' moved mana toward opponent while safer non-handoff alternatives existed",
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
    let spirit_inputs = selector(&spirit_base_game, spirit_config);
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
    };
    let carrier_progress_game = tactical_game_with_items(
        vec![
            (
                Location::new(9, 5),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                },
            ),
            (
                Location::new(8, 5),
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
            drainer_attack_game.fen(),
            MoveClass::DrainerAttack,
        ),
        (
            "immediate_score",
            winning_carrier_initial_fen,
            MoveClass::ImmediateScore,
        ),
        (
            "carrier_progress_probe",
            carrier_progress_game.fen(),
            MoveClass::CarrierProgress,
        ),
    ];
    for (scenario_id, fen, expected_class) in scenario_pack {
        let game = MonsGame::from_fen(fen.as_str(), false).expect("valid scenario fen");
        let config = SearchBudget::from_preference(SmartAutomovePreference::Fast)
            .runtime_config_for_game(&game);
        let selected = selector(&game, config);
        let classes = selected_move_classes(&game, config, &selected).unwrap_or_default();
        assert!(
            classes.has(expected_class),
            "profile '{}' failed scenario '{}' expected class {:?} got immediate={} drainer_attack={} drainer_safety_recover={} carrier_progress={} material={} quiet={}",
            profile_name,
            scenario_id,
            expected_class,
            classes.immediate_score,
            classes.drainer_attack,
            classes.drainer_safety_recover,
            classes.carrier_progress,
            classes.material,
            classes.quiet
        );
    }
}

#[test]
#[ignore = "exports test-only move-regret samples for offline evaluation tuning"]
fn smart_automove_pool_export_eval_tuning_dataset() {
    let profile_name = env::var("SMART_TUNE_PROFILE")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "runtime_current".to_string());
    let Some(selector) = profile_selector_from_name(profile_name.as_str()) else {
        panic!("unknown profile for SMART_TUNE_PROFILE: {}", profile_name);
    };

    let positions = env_usize("SMART_TUNE_POSITIONS").unwrap_or(64).max(8);
    let root_limit = env_usize("SMART_TUNE_ROOT_LIMIT").unwrap_or(8).max(2);
    let seed_tag = env::var("SMART_TUNE_SEED_TAG")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "eval_tune_v1".to_string());
    let output_path = env::var("SMART_TUNE_OUTPUT_PATH")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "target/smart_eval_tuning_samples.jsonl".to_string());

    let openings_seed = seed_for_pairing(seed_tag.as_str(), profile_name.as_str());
    let openings = generate_opening_fens_cached(openings_seed, positions);
    let fast_samples = collect_eval_tuning_samples_for_budget(
        profile_name.as_str(),
        selector,
        SearchBudget::from_preference(SmartAutomovePreference::Fast),
        openings.as_slice(),
        root_limit,
    );
    let normal_samples = collect_eval_tuning_samples_for_budget(
        profile_name.as_str(),
        selector,
        SearchBudget::from_preference(SmartAutomovePreference::Normal),
        openings.as_slice(),
        root_limit,
    );
    let mut all_samples = Vec::with_capacity(fast_samples.len() + normal_samples.len());
    all_samples.extend(fast_samples);
    all_samples.extend(normal_samples);
    let serialized = all_samples
        .iter()
        .map(|sample| eval_tuning_sample_json_line(profile_name.as_str(), sample))
        .collect::<Vec<_>>();

    assert!(
        !all_samples.is_empty(),
        "eval tuning dataset produced no samples"
    );
    if let Err(error) = std::fs::write(output_path.as_str(), serialized.join("\n")) {
        panic!(
            "failed writing SMART_TUNE_OUTPUT_PATH '{}': {}",
            output_path, error
        );
    }

    println!(
        "eval tuning dataset: profile={} positions={} root_limit={} samples={} output={}",
        profile_name,
        positions,
        root_limit,
        all_samples.len(),
        output_path
    );
    println!(
        "next step: tune only small runtime feature subset and copy resulting constants into src/models/scoring.rs"
    );
}

#[test]
#[ignore = "exports default train/holdout evaluation datasets for board-score tuning"]
fn smart_automove_pool_export_eval_tuning_dataset_suite() {
    let profile_name = env::var("SMART_TUNE_PROFILE")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "runtime_current".to_string());
    let Some(selector) = profile_selector_from_name(profile_name.as_str()) else {
        panic!("unknown profile for SMART_TUNE_PROFILE: {}", profile_name);
    };

    let train_positions_per_seed = env_usize("SMART_TUNE_TRAIN_POSITIONS_PER_SEED")
        .unwrap_or(256)
        .max(16);
    let holdout_positions_per_seed = env_usize("SMART_TUNE_HOLDOUT_POSITIONS_PER_SEED")
        .unwrap_or(128)
        .max(16);
    let root_limit = env_usize("SMART_TUNE_ROOT_LIMIT").unwrap_or(12).max(2);

    let train_seed_tags = ["eval_train_v1", "eval_train_v2", "eval_train_v3"];
    let holdout_seed_tags = ["eval_holdout_v1", "eval_holdout_v2"];

    let train_samples = collect_eval_tuning_samples_for_seed_tags(
        profile_name.as_str(),
        selector,
        train_seed_tags.as_slice(),
        train_positions_per_seed,
        root_limit,
    );
    let holdout_samples = collect_eval_tuning_samples_for_seed_tags(
        profile_name.as_str(),
        selector,
        holdout_seed_tags.as_slice(),
        holdout_positions_per_seed,
        root_limit,
    );

    assert!(
        !train_samples.is_empty(),
        "train eval tuning suite produced no samples"
    );
    assert!(
        !holdout_samples.is_empty(),
        "holdout eval tuning suite produced no samples"
    );

    for seed_tag in &train_seed_tags {
        let seed = seed_for_pairing(seed_tag, profile_name.as_str());
        let openings = generate_opening_fens_cached(seed, train_positions_per_seed);
        let mut seed_samples = collect_eval_tuning_samples_for_budget(
            profile_name.as_str(),
            selector,
            SearchBudget::from_preference(SmartAutomovePreference::Fast),
            openings.as_slice(),
            root_limit,
        );
        seed_samples.extend(collect_eval_tuning_samples_for_budget(
            profile_name.as_str(),
            selector,
            SearchBudget::from_preference(SmartAutomovePreference::Normal),
            openings.as_slice(),
            root_limit,
        ));
        let lines = seed_samples
            .iter()
            .map(|sample| eval_tuning_sample_json_line(profile_name.as_str(), sample))
            .collect::<Vec<_>>();
        let output = format!("target/{}_{}.jsonl", seed_tag, "samples");
        std::fs::write(output.as_str(), lines.join("\n")).unwrap_or_else(|error| {
            panic!(
                "failed writing eval train seed artifact '{}': {}",
                output, error
            )
        });
    }
    for seed_tag in &holdout_seed_tags {
        let seed = seed_for_pairing(seed_tag, profile_name.as_str());
        let openings = generate_opening_fens_cached(seed, holdout_positions_per_seed);
        let mut seed_samples = collect_eval_tuning_samples_for_budget(
            profile_name.as_str(),
            selector,
            SearchBudget::from_preference(SmartAutomovePreference::Fast),
            openings.as_slice(),
            root_limit,
        );
        seed_samples.extend(collect_eval_tuning_samples_for_budget(
            profile_name.as_str(),
            selector,
            SearchBudget::from_preference(SmartAutomovePreference::Normal),
            openings.as_slice(),
            root_limit,
        ));
        let lines = seed_samples
            .iter()
            .map(|sample| eval_tuning_sample_json_line(profile_name.as_str(), sample))
            .collect::<Vec<_>>();
        let output = format!("target/{}_{}.jsonl", seed_tag, "samples");
        std::fs::write(output.as_str(), lines.join("\n")).unwrap_or_else(|error| {
            panic!(
                "failed writing eval holdout seed artifact '{}': {}",
                output, error
            )
        });
    }

    println!(
        "eval tuning suite: profile={} train_samples={} holdout_samples={} root_limit={}",
        profile_name,
        train_samples.len(),
        holdout_samples.len(),
        root_limit
    );
}

#[test]
#[ignore = "deterministic coordinate-descent tuning of runtime board evaluation weights"]
fn smart_automove_pool_tune_eval_weights_coordinate_descent() {
    let profile_name = env::var("SMART_TUNE_PROFILE")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "runtime_current".to_string());
    let Some(selector) = profile_selector_from_name(profile_name.as_str()) else {
        panic!("unknown profile for SMART_TUNE_PROFILE: {}", profile_name);
    };

    let train_positions_per_seed = env_usize("SMART_TUNE_TRAIN_POSITIONS_PER_SEED")
        .unwrap_or(256)
        .max(16);
    let holdout_positions_per_seed = env_usize("SMART_TUNE_HOLDOUT_POSITIONS_PER_SEED")
        .unwrap_or(128)
        .max(16);
    let root_limit = env_usize("SMART_TUNE_ROOT_LIMIT").unwrap_or(12).max(2);
    let top_k = env_usize("SMART_TUNE_TOP_K").unwrap_or(8).max(1);
    let manifest_output = env::var("SMART_TUNE_MANIFEST_OUTPUT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "target/eval_tune_ranked_candidates.json".to_string());

    let train_seed_tags = ["eval_train_v1", "eval_train_v2", "eval_train_v3"];
    let holdout_seed_tags = ["eval_holdout_v1", "eval_holdout_v2"];

    let train_samples = collect_eval_tuning_samples_for_seed_tags(
        profile_name.as_str(),
        selector,
        train_seed_tags.as_slice(),
        train_positions_per_seed,
        root_limit,
    );
    let holdout_samples = collect_eval_tuning_samples_for_seed_tags(
        profile_name.as_str(),
        selector,
        holdout_seed_tags.as_slice(),
        holdout_positions_per_seed,
        root_limit,
    );
    assert!(!train_samples.is_empty(), "no train samples collected");
    assert!(!holdout_samples.is_empty(), "no holdout samples collected");

    let families = [
        EvalPhaseFamily::Fast,
        EvalPhaseFamily::NormalBalanced,
        EvalPhaseFamily::NormalTactical,
        EvalPhaseFamily::NormalTacticalAggressive,
        EvalPhaseFamily::NormalFinisher,
        EvalPhaseFamily::NormalFinisherAggressive,
    ];

    let mut train_groups = std::collections::HashMap::<EvalPhaseFamily, Vec<Vec<usize>>>::new();
    let mut holdout_groups = std::collections::HashMap::<EvalPhaseFamily, Vec<Vec<usize>>>::new();
    for family in families {
        train_groups.insert(
            family,
            eval_tune_group_indices(train_samples.as_slice(), family),
        );
        holdout_groups.insert(
            family,
            eval_tune_group_indices(holdout_samples.as_slice(), family),
        );
    }

    let baseline_bundle = EvalTuneBundle::baseline();
    let baseline_train =
        eval_tune_bundle_score(train_samples.as_slice(), &train_groups, baseline_bundle);
    let baseline_holdout =
        eval_tune_bundle_score(holdout_samples.as_slice(), &holdout_groups, baseline_bundle);

    let mut ranked = Vec::<(usize, f64, f64, EvalTuneBundle)>::new();
    ranked.push((0, baseline_train, baseline_holdout, baseline_bundle));

    for candidate_index in 0..(top_k * 2).max(2) {
        let mut bundle = baseline_bundle;
        for (family_offset, family) in families.iter().enumerate() {
            let groups = train_groups.get(family).map(Vec::as_slice).unwrap_or(&[]);
            if groups.is_empty() {
                continue;
            }
            let base_weights = eval_phase_family_base_weights(*family);
            let tuned = tune_weights_for_family(
                train_samples.as_slice(),
                groups,
                *base_weights,
                candidate_index + family_offset,
            );
            bundle = bundle.with_family_weights(*family, tuned);
        }
        let train_score = eval_tune_bundle_score(train_samples.as_slice(), &train_groups, bundle);
        let holdout_score =
            eval_tune_bundle_score(holdout_samples.as_slice(), &holdout_groups, bundle);
        ranked.push((candidate_index + 1, train_score, holdout_score, bundle));
    }

    ranked.sort_by(|a, b| {
        b.2.partial_cmp(&a.2)
            .unwrap_or(Ordering::Equal)
            .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal))
    });
    if ranked.len() > top_k {
        ranked.truncate(top_k);
    }

    write_eval_tune_manifest(manifest_output.as_str(), ranked.as_slice())
        .unwrap_or_else(|error| panic!("failed writing eval tune manifest: {}", error));

    let best = ranked[0];
    println!(
        "eval tune ranked: profile={} train_samples={} holdout_samples={} baseline_train={:.4} baseline_holdout={:.4} best_train={:.4} best_holdout={:.4} output={}",
        profile_name,
        train_samples.len(),
        holdout_samples.len(),
        baseline_train,
        baseline_holdout,
        best.1,
        best.2,
        manifest_output
    );
}

#[test]
#[ignore = "tactical guardrail suite for runtime candidate quality"]
fn smart_automove_tactical_suite() {
    let runtime_selector = profile_selector_from_name("runtime_current")
        .expect("runtime_current selector should exist");
    assert_tactical_guardrails(runtime_selector, "runtime_current");
}

#[test]
#[ignore = "strict promotion gate with raised +12pp target and cpu caps"]
fn smart_automove_pool_promotion_gate_v2() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = env::var("SMART_GATE_BASELINE_PROFILE")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "runtime_current".to_string());
    let allow_self_baseline = env_bool("SMART_GATE_ALLOW_SELF_BASELINE").unwrap_or(false);
    if !allow_self_baseline {
        assert!(
            candidate_profile_name != baseline_profile_name,
            "candidate profile and baseline profile must differ for gate_v2"
        );
    }

    let candidate = AutomoveModel {
        id: "candidate",
        select_inputs: CANDIDATE_MODEL.select_inputs,
    };
    let baseline = AutomoveModel {
        id: "baseline",
        select_inputs: profile_selector_from_name(baseline_profile_name.as_str()).unwrap_or_else(
            || panic!("baseline selector '{}' should exist", baseline_profile_name),
        ),
    };
    println!(
        "promotion gate profiles: candidate={} baseline={}",
        candidate_profile_name, baseline_profile_name
    );
    let budgets = client_budgets().to_vec();

    let speed_positions = env_usize("SMART_GATE_SPEED_POSITIONS").unwrap_or(12).max(4);
    let speed_seed = seed_for_pairing("promotion_gate_v2", "speed");
    let speed_openings = generate_opening_fens_cached(speed_seed, speed_positions);
    let baseline_speed = profile_speed_by_mode_ms(
        baseline.select_inputs,
        speed_openings.as_slice(),
        budgets.as_slice(),
    );
    let candidate_speed = profile_speed_by_mode_ms(
        candidate.select_inputs,
        speed_openings.as_slice(),
        budgets.as_slice(),
    );
    let baseline_map = baseline_speed
        .iter()
        .map(|stat| (stat.budget.key(), stat.avg_ms))
        .collect::<std::collections::HashMap<_, _>>();
    let mut speed_ratios = std::collections::HashMap::new();
    for stat in &candidate_speed {
        let baseline_ms = baseline_map.get(stat.budget.key()).copied().unwrap_or(1.0);
        let ratio = if baseline_ms > 0.0 {
            stat.avg_ms / baseline_ms
        } else {
            1.0
        };
        speed_ratios.insert(stat.budget.key(), ratio);
        println!(
            "promotion gate speed: mode={} candidate_ms={:.2} baseline_ms={:.2} ratio={:.3}",
            stat.budget.key(),
            stat.avg_ms,
            baseline_ms,
            ratio
        );
    }
    assert!(
        speed_ratios.get("fast").copied().unwrap_or(1.0) <= 1.08,
        "fast cpu gate failed: ratio={:.3}",
        speed_ratios.get("fast").copied().unwrap_or(1.0)
    );
    assert!(
        speed_ratios.get("normal").copied().unwrap_or(1.0) <= 1.15,
        "normal cpu gate failed: ratio={:.3}",
        speed_ratios.get("normal").copied().unwrap_or(1.0)
    );

    let budget_duel_games = env_usize("SMART_GATE_BUDGET_DUEL_GAMES")
        .unwrap_or(3)
        .max(1);
    let budget_duel_repeats = env_usize("SMART_GATE_BUDGET_DUEL_REPEATS")
        .unwrap_or(4)
        .max(1);
    let budget_duel_max_plies = env_usize("SMART_GATE_BUDGET_DUEL_MAX_PLIES")
        .or_else(|| env_usize("SMART_POOL_MAX_PLIES"))
        .unwrap_or(56)
        .max(32);
    let budget_duel_seed_tag = env::var("SMART_GATE_BUDGET_DUEL_SEED_TAG")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "fast_normal_v1".to_string());
    let baseline_budget_conversion = run_budget_conversion_diagnostic(
        baseline_profile_name.as_str(),
        baseline.select_inputs,
        budget_duel_games,
        budget_duel_repeats,
        budget_duel_max_plies,
        budget_duel_seed_tag.as_str(),
    );
    let candidate_budget_conversion = run_budget_conversion_diagnostic(
        candidate_profile_name.as_str(),
        candidate.select_inputs,
        budget_duel_games,
        budget_duel_repeats,
        budget_duel_max_plies,
        budget_duel_seed_tag.as_str(),
    );
    let conversion_delta =
        candidate_budget_conversion.normal_edge - baseline_budget_conversion.normal_edge;
    println!(
        "promotion gate budget conversion baseline={} fast_wins={} fast_losses={} draws={} fast_wr={:.3} normal_edge={:.3} confidence={:.3}",
        baseline_profile_name,
        baseline_budget_conversion.fast_wins,
        baseline_budget_conversion.fast_losses,
        baseline_budget_conversion.draws,
        baseline_budget_conversion.fast_win_rate,
        baseline_budget_conversion.normal_edge,
        baseline_budget_conversion.confidence
    );
    println!(
        "promotion gate budget conversion candidate={} fast_wins={} fast_losses={} draws={} fast_wr={:.3} normal_edge={:.3} confidence={:.3} delta_vs_baseline={:.3}",
        candidate_profile_name,
        candidate_budget_conversion.fast_wins,
        candidate_budget_conversion.fast_losses,
        candidate_budget_conversion.draws,
        candidate_budget_conversion.fast_win_rate,
        candidate_budget_conversion.normal_edge,
        candidate_budget_conversion.confidence,
        conversion_delta
    );
    assert!(
        candidate_budget_conversion.normal_edge + SMART_BUDGET_CONVERSION_REGRESSION_TOLERANCE
            >= baseline_budget_conversion.normal_edge,
        "budget conversion regression: candidate normal_edge {:.3} baseline {:.3} tolerance {:.3}",
        candidate_budget_conversion.normal_edge,
        baseline_budget_conversion.normal_edge,
        SMART_BUDGET_CONVERSION_REGRESSION_TOLERANCE
    );

    let quick_results = run_mirrored_duel_for_seed_tag(
        candidate,
        baseline,
        budgets.as_slice(),
        "quick_v1",
        2,
        2,
        72,
        false,
    );
    let mut quick_aggregate = MatchupStats::default();
    for (_budget, stats) in &quick_results {
        quick_aggregate.merge(*stats);
    }
    let quick_delta = quick_aggregate.win_rate_points() - 0.5;
    println!(
        "promotion gate quick-screen: wins={} losses={} draws={} win_rate={:.3} delta={:.3} confidence={:.3}",
        quick_aggregate.wins,
        quick_aggregate.losses,
        quick_aggregate.draws,
        quick_aggregate.win_rate_points(),
        quick_delta,
        quick_aggregate.confidence_better_than_even()
    );
    assert!(
        quick_delta >= 0.04,
        "quick screen failed: delta={:.3} < 0.040",
        quick_delta
    );

    assert_tactical_guardrails(candidate.select_inputs, candidate_profile_name.as_str());

    let primary_games = env_usize("SMART_GATE_PRIMARY_GAMES").unwrap_or(4).max(2);
    let primary_repeats = env_usize("SMART_GATE_PRIMARY_REPEATS").unwrap_or(6).max(2);
    let primary_max_plies = env_usize("SMART_GATE_PRIMARY_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let primary_seed_tags = ["neutral_v1", "neutral_v2", "neutral_v3"];

    let mut primary_mode_stats = std::collections::HashMap::<&'static str, MatchupStats>::new();
    for seed_tag in primary_seed_tags {
        let mode_results = run_mirrored_duel_for_seed_tag(
            candidate,
            baseline,
            budgets.as_slice(),
            seed_tag,
            primary_repeats,
            primary_games,
            primary_max_plies,
            false,
        );
        merge_mode_stats(&mut primary_mode_stats, mode_results.as_slice());
    }
    let mut primary_aggregate = MatchupStats::default();
    for budget in &budgets {
        let stats = primary_mode_stats
            .get(budget.key())
            .copied()
            .unwrap_or_default();
        primary_aggregate.merge(stats);
        let mode_delta = stats.win_rate_points() - 0.5;
        println!(
            "promotion gate primary mode {}: wins={} losses={} draws={} win_rate={:.3} delta={:.3} confidence={:.3}",
            budget.key(),
            stats.wins,
            stats.losses,
            stats.draws,
            stats.win_rate_points(),
            mode_delta,
            stats.confidence_better_than_even(),
        );
        assert!(
            mode_delta >= 0.08,
            "primary mode {} failed delta gate: {:.3} < 0.080",
            budget.key(),
            mode_delta
        );
    }
    let primary_delta = primary_aggregate.win_rate_points() - 0.5;
    let primary_confidence = primary_aggregate.confidence_better_than_even();
    println!(
        "promotion gate primary aggregate: wins={} losses={} draws={} win_rate={:.3} delta={:.3} confidence={:.3}",
        primary_aggregate.wins,
        primary_aggregate.losses,
        primary_aggregate.draws,
        primary_aggregate.win_rate_points(),
        primary_delta,
        primary_confidence
    );
    assert!(
        primary_delta >= 0.12,
        "primary aggregate failed delta gate: {:.3} < 0.120",
        primary_delta
    );
    assert!(
        primary_confidence >= 0.90,
        "primary aggregate failed confidence gate: {:.3} < 0.900",
        primary_confidence
    );

    let confirm_games = env_usize("SMART_GATE_CONFIRM_GAMES").unwrap_or(4).max(2);
    let confirm_repeats = env_usize("SMART_GATE_CONFIRM_REPEATS").unwrap_or(6).max(2);
    let confirm_max_plies = env_usize("SMART_GATE_CONFIRM_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let confirm_results = run_mirrored_duel_for_seed_tag(
        candidate,
        baseline,
        budgets.as_slice(),
        "prod_open_v1",
        confirm_repeats,
        confirm_games,
        confirm_max_plies,
        true,
    );
    let mut confirm_aggregate = MatchupStats::default();
    for (_budget, stats) in &confirm_results {
        confirm_aggregate.merge(*stats);
    }
    let confirm_delta = confirm_aggregate.win_rate_points() - 0.5;
    let confirm_confidence = confirm_aggregate.confidence_better_than_even();
    println!(
        "promotion gate confirmation aggregate: wins={} losses={} draws={} win_rate={:.3} delta={:.3} confidence={:.3}",
        confirm_aggregate.wins,
        confirm_aggregate.losses,
        confirm_aggregate.draws,
        confirm_aggregate.win_rate_points(),
        confirm_delta,
        confirm_confidence
    );
    assert!(
        confirm_delta >= 0.05,
        "confirmation delta gate failed: {:.3} < 0.050",
        confirm_delta
    );
    assert!(
        confirm_confidence >= 0.75,
        "confirmation confidence gate failed: {:.3} < 0.750",
        confirm_confidence
    );

    let pool_games = env_usize("SMART_GATE_POOL_GAMES").unwrap_or(3).max(1);
    let (candidate_pool_eval, baseline_pool_eval, candidate_pool_wr, baseline_pool_wr) =
        run_pool_non_regression_check(candidate, baseline, budgets.as_slice(), pool_games);
    println!(
        "promotion gate pool check: candidate_beaten={}/{} baseline_beaten={}/{} candidate_wr={:.3} baseline_wr={:.3}",
        candidate_pool_eval.beaten_opponents,
        candidate_pool_eval.opponents.len(),
        baseline_pool_eval.beaten_opponents,
        baseline_pool_eval.opponents.len(),
        candidate_pool_wr,
        baseline_pool_wr
    );
    assert!(
        candidate_pool_eval.beaten_opponents >= baseline_pool_eval.beaten_opponents,
        "pool non-regression failed beaten opponents: candidate {} < baseline {}",
        candidate_pool_eval.beaten_opponents,
        baseline_pool_eval.beaten_opponents
    );
    assert!(
        candidate_pool_wr + 0.01 >= baseline_pool_wr,
        "pool non-regression failed aggregate win-rate: candidate {:.3} baseline {:.3}",
        candidate_pool_wr,
        baseline_pool_wr
    );
}

#[test]
#[ignore = "staged promotion ladder with early-stop pruning and artifact output"]
fn smart_automove_pool_promotion_ladder() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = env::var("SMART_GATE_BASELINE_PROFILE")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "runtime_current".to_string());
    let allow_self_baseline = env_bool("SMART_GATE_ALLOW_SELF_BASELINE").unwrap_or(false);
    if !allow_self_baseline {
        assert!(
            candidate_profile_name != baseline_profile_name,
            "candidate profile and baseline profile must differ for ladder gate"
        );
    }

    let candidate = AutomoveModel {
        id: "candidate",
        select_inputs: CANDIDATE_MODEL.select_inputs,
    };
    let baseline = AutomoveModel {
        id: "baseline",
        select_inputs: profile_selector_from_name(baseline_profile_name.as_str()).unwrap_or_else(
            || panic!("baseline selector '{}' should exist", baseline_profile_name),
        ),
    };
    let budgets = client_budgets().to_vec();
    let mut artifacts = Vec::<String>::new();

    assert_tactical_guardrails(candidate.select_inputs, candidate_profile_name.as_str());
    artifacts.push(format!(
        r#"{{"stage":"A_tactical","profile":"{}","status":"pass"}}"#,
        candidate_profile_name
    ));

    let speed_positions = env_usize("SMART_GATE_SPEED_POSITIONS").unwrap_or(12).max(4);
    let speed_seed = seed_for_pairing("promotion_ladder", "speed");
    let speed_openings = generate_opening_fens_cached(speed_seed, speed_positions);
    let baseline_speed = profile_speed_by_mode_ms(
        baseline.select_inputs,
        speed_openings.as_slice(),
        budgets.as_slice(),
    );
    let candidate_speed = profile_speed_by_mode_ms(
        candidate.select_inputs,
        speed_openings.as_slice(),
        budgets.as_slice(),
    );
    let baseline_map = baseline_speed
        .iter()
        .map(|stat| (stat.budget.key(), stat.avg_ms))
        .collect::<std::collections::HashMap<_, _>>();
    let mut speed_ratios = std::collections::HashMap::new();
    for stat in &candidate_speed {
        let baseline_ms = baseline_map.get(stat.budget.key()).copied().unwrap_or(1.0);
        let ratio = if baseline_ms > 0.0 {
            stat.avg_ms / baseline_ms
        } else {
            1.0
        };
        speed_ratios.insert(stat.budget.key(), ratio);
    }
    let fast_ratio = speed_ratios.get("fast").copied().unwrap_or(1.0);
    let normal_ratio = speed_ratios.get("normal").copied().unwrap_or(1.0);
    artifacts.push(format!(
        r#"{{"stage":"A_speed","fast_ratio":{:.5},"normal_ratio":{:.5}}}"#,
        fast_ratio, normal_ratio
    ));
    assert!(
        fast_ratio <= 1.08,
        "fast cpu gate failed: ratio={:.3}",
        fast_ratio
    );
    assert!(
        normal_ratio <= 1.15,
        "normal cpu gate failed: ratio={:.3}",
        normal_ratio
    );

    let budget_duel_games = env_usize("SMART_GATE_BUDGET_DUEL_GAMES")
        .unwrap_or(3)
        .max(1);
    let budget_duel_repeats = env_usize("SMART_GATE_BUDGET_DUEL_REPEATS")
        .unwrap_or(4)
        .max(1);
    let budget_duel_max_plies = env_usize("SMART_GATE_BUDGET_DUEL_MAX_PLIES")
        .or_else(|| env_usize("SMART_POOL_MAX_PLIES"))
        .unwrap_or(56)
        .max(32);
    let budget_duel_seed_tag = env::var("SMART_GATE_BUDGET_DUEL_SEED_TAG")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "fast_normal_v1".to_string());
    let baseline_budget_conversion = run_budget_conversion_diagnostic(
        baseline_profile_name.as_str(),
        baseline.select_inputs,
        budget_duel_games,
        budget_duel_repeats,
        budget_duel_max_plies,
        budget_duel_seed_tag.as_str(),
    );
    let candidate_budget_conversion = run_budget_conversion_diagnostic(
        candidate_profile_name.as_str(),
        candidate.select_inputs,
        budget_duel_games,
        budget_duel_repeats,
        budget_duel_max_plies,
        budget_duel_seed_tag.as_str(),
    );
    let conversion_delta =
        candidate_budget_conversion.normal_edge - baseline_budget_conversion.normal_edge;
    artifacts.push(format!(
        r#"{{"stage":"A_budget_conversion","baseline_fast_wr":{:.5},"baseline_normal_edge":{:.5},"candidate_fast_wr":{:.5},"candidate_normal_edge":{:.5},"delta":{:.5}}}"#,
        baseline_budget_conversion.fast_win_rate,
        baseline_budget_conversion.normal_edge,
        candidate_budget_conversion.fast_win_rate,
        candidate_budget_conversion.normal_edge,
        conversion_delta
    ));
    assert!(
        candidate_budget_conversion.normal_edge + SMART_BUDGET_CONVERSION_REGRESSION_TOLERANCE
            >= baseline_budget_conversion.normal_edge,
        "budget conversion regression: candidate normal_edge {:.3} baseline {:.3} tolerance {:.3}",
        candidate_budget_conversion.normal_edge,
        baseline_budget_conversion.normal_edge,
        SMART_BUDGET_CONVERSION_REGRESSION_TOLERANCE
    );

    let quick_results = run_mirrored_duel_for_seed_tag(
        candidate,
        baseline,
        budgets.as_slice(),
        "quick_v1",
        2,
        2,
        72,
        false,
    );
    let mut quick_aggregate = MatchupStats::default();
    for (_budget, stats) in &quick_results {
        quick_aggregate.merge(*stats);
    }
    let quick_delta = quick_aggregate.win_rate_points() - 0.5;
    artifacts.push(format!(
        r#"{{"stage":"B_quick","wins":{},"losses":{},"draws":{},"delta":{:.5},"confidence":{:.5}}}"#,
        quick_aggregate.wins,
        quick_aggregate.losses,
        quick_aggregate.draws,
        quick_delta,
        quick_aggregate.confidence_better_than_even()
    ));
    assert!(
        quick_delta >= 0.04,
        "quick keep-going gate failed: delta={:.3} < 0.040",
        quick_delta
    );

    let reduced_games = env_usize("SMART_LADDER_REDUCED_GAMES").unwrap_or(2).max(2);
    let reduced_repeats = env_usize("SMART_LADDER_REDUCED_REPEATS")
        .unwrap_or(2)
        .max(2);
    let reduced_max_plies = env_usize("SMART_LADDER_REDUCED_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let reduced_seed_tags = ["neutral_v1", "neutral_v2", "neutral_v3"];
    let mut reduced_mode_stats = std::collections::HashMap::<&'static str, MatchupStats>::new();
    for (seed_index, seed_tag) in reduced_seed_tags.iter().enumerate() {
        let mode_results = run_mirrored_duel_for_seed_tag(
            candidate,
            baseline,
            budgets.as_slice(),
            seed_tag,
            reduced_repeats,
            reduced_games,
            reduced_max_plies,
            false,
        );
        merge_mode_stats(&mut reduced_mode_stats, mode_results.as_slice());

        let remaining_seed_count = reduced_seed_tags.len().saturating_sub(seed_index + 1);
        let remaining_games_per_mode = remaining_seed_count * reduced_repeats * reduced_games * 2;

        let mut reduced_aggregate = MatchupStats::default();
        for budget in &budgets {
            reduced_aggregate.merge(
                reduced_mode_stats
                    .get(budget.key())
                    .copied()
                    .unwrap_or_default(),
            );
        }
        let remaining_total_games = remaining_games_per_mode * budgets.len();
        let best_case_aggregate_delta =
            max_achievable_delta(reduced_aggregate, remaining_total_games);
        assert!(
            best_case_aggregate_delta >= 0.06,
            "reduced gate early-stop: aggregate best-case delta {:.3} < 0.060",
            best_case_aggregate_delta
        );
        for budget in &budgets {
            let mode_stats = reduced_mode_stats
                .get(budget.key())
                .copied()
                .unwrap_or_default();
            let best_case_mode_delta = max_achievable_delta(mode_stats, remaining_games_per_mode);
            assert!(
                best_case_mode_delta >= 0.04,
                "reduced gate early-stop: mode {} best-case delta {:.3} < 0.040",
                budget.key(),
                best_case_mode_delta
            );
        }
    }
    let mut reduced_aggregate = MatchupStats::default();
    for budget in &budgets {
        let mode_stats = reduced_mode_stats
            .get(budget.key())
            .copied()
            .unwrap_or_default();
        let mode_delta = mode_stats.win_rate_points() - 0.5;
        assert!(
            mode_delta >= 0.04,
            "reduced stage mode {} failed: delta {:.3} < 0.040",
            budget.key(),
            mode_delta
        );
        reduced_aggregate.merge(mode_stats);
    }
    let reduced_delta = reduced_aggregate.win_rate_points() - 0.5;
    let reduced_confidence = reduced_aggregate.confidence_better_than_even();
    artifacts.push(format!(
        r#"{{"stage":"C_reduced","wins":{},"losses":{},"draws":{},"delta":{:.5},"confidence":{:.5}}}"#,
        reduced_aggregate.wins,
        reduced_aggregate.losses,
        reduced_aggregate.draws,
        reduced_delta,
        reduced_confidence
    ));
    assert!(
        reduced_delta >= 0.06,
        "reduced stage aggregate failed: delta {:.3} < 0.060",
        reduced_delta
    );
    assert!(
        reduced_confidence >= 0.60,
        "reduced stage confidence failed: {:.3} < 0.600",
        reduced_confidence
    );

    let primary_games = env_usize("SMART_GATE_PRIMARY_GAMES").unwrap_or(4).max(2);
    let primary_repeats = env_usize("SMART_GATE_PRIMARY_REPEATS").unwrap_or(6).max(2);
    let primary_max_plies = env_usize("SMART_GATE_PRIMARY_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let primary_seed_tags = ["neutral_v1", "neutral_v2", "neutral_v3"];
    let mut primary_mode_stats = std::collections::HashMap::<&'static str, MatchupStats>::new();
    for (seed_index, seed_tag) in primary_seed_tags.iter().enumerate() {
        let mode_results = run_mirrored_duel_for_seed_tag(
            candidate,
            baseline,
            budgets.as_slice(),
            seed_tag,
            primary_repeats,
            primary_games,
            primary_max_plies,
            false,
        );
        merge_mode_stats(&mut primary_mode_stats, mode_results.as_slice());

        let remaining_seed_count = primary_seed_tags.len().saturating_sub(seed_index + 1);
        let remaining_games_per_mode = remaining_seed_count * primary_repeats * primary_games * 2;

        let mut primary_aggregate = MatchupStats::default();
        for budget in &budgets {
            primary_aggregate.merge(
                primary_mode_stats
                    .get(budget.key())
                    .copied()
                    .unwrap_or_default(),
            );
        }
        let remaining_total_games = remaining_games_per_mode * budgets.len();
        let best_case_aggregate_delta =
            max_achievable_delta(primary_aggregate, remaining_total_games);
        assert!(
            best_case_aggregate_delta >= 0.12,
            "primary early-stop: aggregate best-case delta {:.3} < 0.120",
            best_case_aggregate_delta
        );
        for budget in &budgets {
            let mode_stats = primary_mode_stats
                .get(budget.key())
                .copied()
                .unwrap_or_default();
            let best_case_mode_delta = max_achievable_delta(mode_stats, remaining_games_per_mode);
            assert!(
                best_case_mode_delta >= 0.08,
                "primary early-stop: mode {} best-case delta {:.3} < 0.080",
                budget.key(),
                best_case_mode_delta
            );
        }
    }
    let mut primary_aggregate = MatchupStats::default();
    for budget in &budgets {
        let stats = primary_mode_stats
            .get(budget.key())
            .copied()
            .unwrap_or_default();
        primary_aggregate.merge(stats);
        let mode_delta = stats.win_rate_points() - 0.5;
        assert!(
            mode_delta >= 0.08,
            "primary mode {} failed delta gate: {:.3} < 0.080",
            budget.key(),
            mode_delta
        );
    }
    let primary_delta = primary_aggregate.win_rate_points() - 0.5;
    let primary_confidence = primary_aggregate.confidence_better_than_even();
    artifacts.push(format!(
        r#"{{"stage":"D_primary","wins":{},"losses":{},"draws":{},"delta":{:.5},"confidence":{:.5}}}"#,
        primary_aggregate.wins,
        primary_aggregate.losses,
        primary_aggregate.draws,
        primary_delta,
        primary_confidence
    ));
    assert!(
        primary_delta >= 0.12,
        "primary aggregate failed delta gate: {:.3} < 0.120",
        primary_delta
    );
    assert!(
        primary_confidence >= 0.90,
        "primary aggregate failed confidence gate: {:.3} < 0.900",
        primary_confidence
    );

    let confirm_games = env_usize("SMART_GATE_CONFIRM_GAMES").unwrap_or(4).max(2);
    let confirm_repeats = env_usize("SMART_GATE_CONFIRM_REPEATS").unwrap_or(6).max(2);
    let confirm_max_plies = env_usize("SMART_GATE_CONFIRM_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let confirm_results = run_mirrored_duel_for_seed_tag(
        candidate,
        baseline,
        budgets.as_slice(),
        "prod_open_v1",
        confirm_repeats,
        confirm_games,
        confirm_max_plies,
        true,
    );
    let mut confirm_aggregate = MatchupStats::default();
    for (_budget, stats) in &confirm_results {
        confirm_aggregate.merge(*stats);
    }
    let confirm_delta = confirm_aggregate.win_rate_points() - 0.5;
    let confirm_confidence = confirm_aggregate.confidence_better_than_even();
    artifacts.push(format!(
        r#"{{"stage":"D_confirm","wins":{},"losses":{},"draws":{},"delta":{:.5},"confidence":{:.5}}}"#,
        confirm_aggregate.wins,
        confirm_aggregate.losses,
        confirm_aggregate.draws,
        confirm_delta,
        confirm_confidence
    ));
    assert!(
        confirm_delta >= 0.05,
        "confirmation delta gate failed: {:.3} < 0.050",
        confirm_delta
    );
    assert!(
        confirm_confidence >= 0.75,
        "confirmation confidence gate failed: {:.3} < 0.750",
        confirm_confidence
    );

    let pool_games = env_usize("SMART_GATE_POOL_GAMES").unwrap_or(3).max(1);
    let (candidate_pool_eval, baseline_pool_eval, candidate_pool_wr, baseline_pool_wr) =
        run_pool_non_regression_check(candidate, baseline, budgets.as_slice(), pool_games);
    artifacts.push(format!(
        r#"{{"stage":"D_pool","candidate_beaten":{},"candidate_total":{},"baseline_beaten":{},"baseline_total":{},"candidate_wr":{:.5},"baseline_wr":{:.5}}}"#,
        candidate_pool_eval.beaten_opponents,
        candidate_pool_eval.opponents.len(),
        baseline_pool_eval.beaten_opponents,
        baseline_pool_eval.opponents.len(),
        candidate_pool_wr,
        baseline_pool_wr
    ));
    assert!(
        candidate_pool_eval.beaten_opponents >= baseline_pool_eval.beaten_opponents,
        "pool non-regression failed beaten opponents: candidate {} < baseline {}",
        candidate_pool_eval.beaten_opponents,
        baseline_pool_eval.beaten_opponents
    );
    assert!(
        candidate_pool_wr + 0.01 >= baseline_pool_wr,
        "pool non-regression failed aggregate win-rate: candidate {:.3} baseline {:.3}",
        candidate_pool_wr,
        baseline_pool_wr
    );

    persist_ladder_artifacts(artifacts.as_slice());
}
