use crate::*;

const PROTECTED_HIGH_VALUE_CARRIER_SAFE_DANGER_MIN: i32 = 3;
const PROTECTED_HIGH_VALUE_CARRIER_SUPERMANA_SCALE_BP: i32 = 2_500;
const PROTECTED_HIGH_VALUE_CARRIER_OPPONENT_MANA_SCALE_BP: i32 = 2_500;
const PROTECTED_HIGH_VALUE_CARRIER_VIRTUAL_SCORE_BP_MAX: i32 = 9_200;
const PROTECTED_HIGH_VALUE_CARRIER_OPPONENT_SCORE_MARGIN: i32 = 2;

#[derive(Debug, Clone, Copy)]
pub struct ScoringWeights {
    pub use_legacy_formula: bool,
    pub include_regular_mana_move_windows: bool,
    pub include_match_point_window: bool,
    pub next_turn_window_scale_bp: i32,
    pub double_confirmed_score: bool,
    pub confirmed_score: i32,
    pub fainted_mon: i32,
    pub fainted_drainer: i32,
    pub fainted_cooldown_step: i32,
    pub drainer_at_risk: i32,
    pub mana_close_to_same_pool: i32,
    pub mon_with_mana_close_to_any_pool: i32,
    pub extra_for_supermana: i32,
    pub extra_for_opponents_mana: i32,
    pub drainer_close_to_mana: i32,
    pub drainer_holding_mana: i32,
    pub drainer_close_to_own_pool: i32,
    pub drainer_close_to_supermana: i32,
    pub mon_close_to_center: i32,
    pub spirit_close_to_enemy: i32,
    pub spirit_on_own_base_penalty: i32,
    pub angel_guarding_drainer: i32,
    pub angel_close_to_friendly_drainer: i32,
    pub has_consumable: i32,
    pub active_mon: i32,
    pub regular_mana_to_owner_pool: i32,
    pub regular_mana_drainer_control: i32,
    pub supermana_drainer_control: i32,
    pub supermana_race_control: i32,
    pub opponent_mana_denial: i32,
    pub mana_carrier_at_risk: i32,
    pub mana_carrier_guarded: i32,
    pub mana_carrier_one_step_from_pool: i32,
    pub supermana_carrier_one_step_from_pool_extra: i32,
    pub immediate_winning_carrier: i32,
    pub drainer_best_mana_path: i32,
    pub drainer_pickup_score_this_turn: i32,
    pub mana_carrier_score_this_turn: i32,
    pub drainer_immediate_threat: i32,
    pub score_race_path_progress: i32,
    pub opponent_score_race_path_progress: i32,
    pub score_race_multi_path: i32,
    pub opponent_score_race_multi_path: i32,
    pub immediate_score_window: i32,
    pub opponent_immediate_score_window: i32,
    pub immediate_score_multi_window: i32,
    pub opponent_immediate_score_multi_window: i32,
    pub spirit_action_utility: i32,
    pub drainer_danger_boolean: i32,
    pub mana_carrier_danger_boolean: i32,
    pub drainer_walk_threat_boolean: i32,
    pub mana_carrier_walk_threat_boolean: i32,
    pub opponent_drainer_attack_bonus: i32,
    pub attacker_close_to_opponent_drainer: i32,
}

pub const DEFAULT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    use_legacy_formula: true,
    include_regular_mana_move_windows: false,
    include_match_point_window: false,
    next_turn_window_scale_bp: 5_000,
    double_confirmed_score: true,
    confirmed_score: 1000,
    fainted_mon: -500,
    fainted_drainer: -800,
    fainted_cooldown_step: 0,
    drainer_at_risk: -350,
    mana_close_to_same_pool: 500,
    mon_with_mana_close_to_any_pool: 800,
    extra_for_supermana: 120,
    extra_for_opponents_mana: 100,
    drainer_close_to_mana: 300,
    drainer_holding_mana: 350,
    drainer_close_to_own_pool: 180,
    drainer_close_to_supermana: 120,
    mon_close_to_center: 210,
    spirit_close_to_enemy: 160,
    spirit_on_own_base_penalty: 180,
    angel_guarding_drainer: 180,
    angel_close_to_friendly_drainer: 120,
    has_consumable: 110,
    active_mon: 50,
    regular_mana_to_owner_pool: 0,
    regular_mana_drainer_control: 0,
    supermana_drainer_control: 0,
    supermana_race_control: 0,
    opponent_mana_denial: 0,
    mana_carrier_at_risk: 0,
    mana_carrier_guarded: 0,
    mana_carrier_one_step_from_pool: 0,
    supermana_carrier_one_step_from_pool_extra: 0,
    immediate_winning_carrier: 0,
    drainer_best_mana_path: 0,
    drainer_pickup_score_this_turn: 0,
    mana_carrier_score_this_turn: 0,
    drainer_immediate_threat: 0,
    score_race_path_progress: 0,
    opponent_score_race_path_progress: 0,
    score_race_multi_path: 0,
    opponent_score_race_multi_path: 0,
    immediate_score_window: 0,
    opponent_immediate_score_window: 0,
    immediate_score_multi_window: 0,
    opponent_immediate_score_multi_window: 0,
    spirit_action_utility: 0,
    drainer_danger_boolean: 0,
    mana_carrier_danger_boolean: 0,
    drainer_walk_threat_boolean: 0,
    mana_carrier_walk_threat_boolean: 0,
    opponent_drainer_attack_bonus: 0,
    attacker_close_to_opponent_drainer: 0,
};

pub const SWIFT_2024_REFERENCE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    use_legacy_formula: true,
    include_regular_mana_move_windows: false,
    include_match_point_window: false,
    next_turn_window_scale_bp: 5_000,
    double_confirmed_score: true,
    confirmed_score: 1000,
    fainted_mon: -500,
    fainted_drainer: -800,
    fainted_cooldown_step: 0,
    drainer_at_risk: -350,
    mana_close_to_same_pool: 500,
    mon_with_mana_close_to_any_pool: 800,
    extra_for_supermana: 120,
    extra_for_opponents_mana: 100,
    drainer_close_to_mana: 300,
    drainer_holding_mana: 350,
    drainer_close_to_own_pool: 180,
    drainer_close_to_supermana: 120,
    mon_close_to_center: 210,
    spirit_close_to_enemy: 160,
    spirit_on_own_base_penalty: 180,
    angel_guarding_drainer: 180,
    angel_close_to_friendly_drainer: 120,
    has_consumable: 110,
    active_mon: 50,
    regular_mana_to_owner_pool: 0,
    regular_mana_drainer_control: 0,
    supermana_drainer_control: 0,
    supermana_race_control: 0,
    opponent_mana_denial: 0,
    mana_carrier_at_risk: 0,
    mana_carrier_guarded: 0,
    mana_carrier_one_step_from_pool: 0,
    supermana_carrier_one_step_from_pool_extra: 0,
    immediate_winning_carrier: 0,
    drainer_best_mana_path: 0,
    drainer_pickup_score_this_turn: 0,
    mana_carrier_score_this_turn: 0,
    drainer_immediate_threat: 0,
    score_race_path_progress: 0,
    opponent_score_race_path_progress: 0,
    score_race_multi_path: 0,
    opponent_score_race_multi_path: 0,
    immediate_score_window: 0,
    opponent_immediate_score_window: 0,
    immediate_score_multi_window: 0,
    opponent_immediate_score_multi_window: 0,
    spirit_action_utility: 0,
    drainer_danger_boolean: 0,
    mana_carrier_danger_boolean: 0,
    drainer_walk_threat_boolean: 0,
    mana_carrier_walk_threat_boolean: 0,
    opponent_drainer_attack_bonus: 0,
    attacker_close_to_opponent_drainer: 0,
};

pub const BALANCED_DISTANCE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    use_legacy_formula: true,
    include_regular_mana_move_windows: false,
    include_match_point_window: false,
    next_turn_window_scale_bp: 5_000,
    double_confirmed_score: true,
    confirmed_score: 1000,
    fainted_mon: -520,
    fainted_drainer: -900,
    fainted_cooldown_step: -80,
    drainer_at_risk: -420,
    mana_close_to_same_pool: 520,
    mon_with_mana_close_to_any_pool: 820,
    extra_for_supermana: 130,
    extra_for_opponents_mana: 120,
    drainer_close_to_mana: 330,
    drainer_holding_mana: 370,
    drainer_close_to_own_pool: 280,
    drainer_close_to_supermana: 180,
    mon_close_to_center: 180,
    spirit_close_to_enemy: 220,
    spirit_on_own_base_penalty: 180,
    angel_guarding_drainer: 280,
    angel_close_to_friendly_drainer: 180,
    has_consumable: 105,
    active_mon: 45,
    regular_mana_to_owner_pool: 0,
    regular_mana_drainer_control: 0,
    supermana_drainer_control: 0,
    supermana_race_control: 0,
    opponent_mana_denial: 0,
    mana_carrier_at_risk: 0,
    mana_carrier_guarded: 0,
    mana_carrier_one_step_from_pool: 160,
    supermana_carrier_one_step_from_pool_extra: 80,
    immediate_winning_carrier: 0,
    drainer_best_mana_path: 0,
    drainer_pickup_score_this_turn: 0,
    mana_carrier_score_this_turn: 0,
    drainer_immediate_threat: 0,
    score_race_path_progress: 0,
    opponent_score_race_path_progress: 0,
    score_race_multi_path: 0,
    opponent_score_race_multi_path: 0,
    immediate_score_window: 0,
    opponent_immediate_score_window: 0,
    immediate_score_multi_window: 0,
    opponent_immediate_score_multi_window: 0,
    spirit_action_utility: 0,
    drainer_danger_boolean: 0,
    mana_carrier_danger_boolean: 0,
    drainer_walk_threat_boolean: 0,
    mana_carrier_walk_threat_boolean: 0,
    opponent_drainer_attack_bonus: 0,
    attacker_close_to_opponent_drainer: 0,
};

#[cfg(test)]
pub const MANA_RACE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 260,
    regular_mana_drainer_control: 26,
    supermana_drainer_control: 44,
    mana_carrier_at_risk: -260,
    mana_carrier_guarded: 140,
    drainer_close_to_own_pool: 300,
    drainer_close_to_supermana: 220,
    drainer_holding_mana: 380,
    spirit_close_to_enemy: 210,
    angel_guarding_drainer: 290,
    angel_close_to_friendly_drainer: 190,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const MANA_RACE_LITE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 150,
    regular_mana_drainer_control: 15,
    supermana_drainer_control: 26,
    mana_carrier_at_risk: -150,
    mana_carrier_guarded: 70,
    drainer_close_to_own_pool: 290,
    drainer_close_to_supermana: 200,
    angel_guarding_drainer: 290,
    mana_close_to_same_pool: 420,
    fainted_cooldown_step: -70,
    mana_carrier_one_step_from_pool: 220,
    supermana_carrier_one_step_from_pool_extra: 120,
    immediate_winning_carrier: 0,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const MANA_RACE_LITE_TACTICAL_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 170,
    regular_mana_drainer_control: 18,
    supermana_drainer_control: 30,
    fainted_cooldown_step: -110,
    mana_carrier_at_risk: -220,
    mana_carrier_guarded: 120,
    mana_carrier_one_step_from_pool: 270,
    supermana_carrier_one_step_from_pool_extra: 170,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const FINISHER_BALANCED_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_one_step_from_pool: 260,
    supermana_carrier_one_step_from_pool_extra: 140,
    immediate_winning_carrier: 850,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const FINISHER_MANA_RACE_LITE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_one_step_from_pool: 300,
    supermana_carrier_one_step_from_pool_extra: 180,
    immediate_winning_carrier: 980,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const FINISHER_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_at_risk: -190,
    mana_carrier_guarded: 90,
    mana_carrier_one_step_from_pool: 340,
    supermana_carrier_one_step_from_pool_extra: 220,
    immediate_winning_carrier: 1200,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

pub const FINISHER_BALANCED_SOFT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_one_step_from_pool: 220,
    supermana_carrier_one_step_from_pool_extra: 110,
    immediate_winning_carrier: 360,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_one_step_from_pool: 250,
    supermana_carrier_one_step_from_pool_extra: 130,
    immediate_winning_carrier: 540,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const FINISHER_MANA_RACE_LITE_SOFT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_one_step_from_pool: 250,
    supermana_carrier_one_step_from_pool_extra: 140,
    immediate_winning_carrier: 420,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

pub const FINISHER_MANA_RACE_LITE_SOFT_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        mana_carrier_one_step_from_pool: 280,
        supermana_carrier_one_step_from_pool_extra: 165,
        immediate_winning_carrier: 620,
        ..MANA_RACE_LITE_SCORING_WEIGHTS
    };

pub const MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 170,
    regular_mana_drainer_control: 18,
    mana_close_to_same_pool: 380,
    drainer_close_to_own_pool: 320,
    mana_carrier_at_risk: -210,
    mana_carrier_guarded: 95,
    mana_carrier_one_step_from_pool: 260,
    supermana_carrier_one_step_from_pool_extra: 150,
    immediate_winning_carrier: 300,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const MANA_RACE_LITE_D2_TUNED_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 190,
    regular_mana_drainer_control: 22,
    mana_close_to_same_pool: 360,
    drainer_close_to_own_pool: 340,
    mana_carrier_at_risk: -250,
    mana_carrier_guarded: 110,
    mana_carrier_one_step_from_pool: 300,
    supermana_carrier_one_step_from_pool_extra: 180,
    immediate_winning_carrier: 420,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const MANA_RACE_NEUTRAL_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 220,
    regular_mana_drainer_control: 18,
    supermana_drainer_control: 34,
    mana_carrier_at_risk: -180,
    mana_carrier_guarded: 90,
    mana_close_to_same_pool: 300,
    drainer_close_to_own_pool: 300,
    drainer_close_to_supermana: 210,
    fainted_cooldown_step: -90,
    mana_carrier_one_step_from_pool: 200,
    supermana_carrier_one_step_from_pool_extra: 100,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const TACTICAL_BALANCED_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    fainted_cooldown_step: -120,
    spirit_close_to_enemy: 230,
    angel_guarding_drainer: 300,
    mana_carrier_at_risk: -200,
    mana_carrier_guarded: 110,
    mana_carrier_one_step_from_pool: 240,
    supermana_carrier_one_step_from_pool_extra: 150,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS: ScoringWeights =
    MANA_RACE_LITE_TACTICAL_SCORING_WEIGHTS;

pub const TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    fainted_cooldown_step: -160,
    mana_carrier_at_risk: -260,
    mana_carrier_guarded: 140,
    mana_carrier_one_step_from_pool: 320,
    supermana_carrier_one_step_from_pool_extra: 220,
    spirit_close_to_enemy: 250,
    angel_guarding_drainer: 320,
    ..TACTICAL_BALANCED_SCORING_WEIGHTS
};

pub const TACTICAL_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    fainted_cooldown_step: -150,
    mana_carrier_at_risk: -290,
    mana_carrier_guarded: 160,
    mana_carrier_one_step_from_pool: 360,
    supermana_carrier_one_step_from_pool_extra: 240,
    regular_mana_to_owner_pool: 200,
    regular_mana_drainer_control: 24,
    supermana_drainer_control: 36,
    ..TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS
};

pub const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    use_legacy_formula: false,
    confirmed_score: 920,
    drainer_best_mana_path: 250,
    drainer_pickup_score_this_turn: 210,
    mana_carrier_score_this_turn: 290,
    drainer_immediate_threat: -220,
    score_race_path_progress: 165,
    opponent_score_race_path_progress: 150,
    score_race_multi_path: 60,
    opponent_score_race_multi_path: 90,
    immediate_score_window: 240,
    opponent_immediate_score_window: 220,
    immediate_score_multi_window: 80,
    opponent_immediate_score_multi_window: 120,
    spirit_action_utility: 56,
    drainer_close_to_mana: 360,
    drainer_holding_mana: 430,
    mana_carrier_at_risk: -285,
    mana_carrier_guarded: 145,
    mana_carrier_one_step_from_pool: 320,
    supermana_carrier_one_step_from_pool_extra: 210,
    immediate_winning_carrier: 520,
    ..MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS
};

pub const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF: ScoringWeights =
    ScoringWeights {
        has_consumable: 320,
        spirit_action_utility: 72,
        ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
    };

pub const RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    drainer_danger_boolean: -400,
    mana_carrier_danger_boolean: -300,
    supermana_race_control: 30,
    ..RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS
};

pub const RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF: ScoringWeights =
    ScoringWeights {
        has_consumable: 320,
        spirit_action_utility: 72,
        ..RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS
    };

pub const RUNTIME_RUSH_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    drainer_at_risk: -300,
    mana_close_to_same_pool: 600,
    drainer_close_to_mana: 420,
    drainer_close_to_own_pool: 360,
    drainer_close_to_supermana: 220,
    mon_close_to_center: 140,
    spirit_close_to_enemy: 180,
    angel_guarding_drainer: 200,
    angel_close_to_friendly_drainer: 120,
    score_race_path_progress: 85,
    opponent_score_race_path_progress: 130,
    immediate_score_window: 95,
    opponent_immediate_score_window: 190,
    spirit_action_utility: 55,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const RUNTIME_NORMAL_WINLOSS_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    confirmed_score: 900,
    fainted_cooldown_step: -130,
    spirit_close_to_enemy: 235,
    angel_guarding_drainer: 310,
    regular_mana_to_owner_pool: 70,
    regular_mana_drainer_control: 8,
    supermana_drainer_control: 16,
    mana_carrier_at_risk: -230,
    mana_carrier_guarded: 125,
    mana_carrier_one_step_from_pool: 300,
    supermana_carrier_one_step_from_pool_extra: 185,
    immediate_winning_carrier: 520,
    drainer_best_mana_path: 70,
    drainer_pickup_score_this_turn: 55,
    mana_carrier_score_this_turn: 150,
    drainer_immediate_threat: -100,
    ..TACTICAL_BALANCED_SCORING_WEIGHTS
};

#[cfg(test)]
pub const RUNTIME_FAST_DRAINER_PRIORITY_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
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

#[derive(Debug, Clone, Copy, Default)]
pub struct EvalFeatureSnapshot {
    pub active_color_is_perspective: bool,
    pub remaining_mon_moves_for_active: i32,
    pub my_score_path_best_steps: i32,
    pub my_score_path_multi_pressure: i32,
    pub opponent_score_path_best_steps: i32,
    pub opponent_score_path_multi_pressure: i32,
    pub my_immediate_best_score: i32,
    pub my_immediate_multi_pressure: i32,
    pub opponent_immediate_best_score: i32,
    pub opponent_immediate_multi_pressure: i32,
    pub include_regular_mana_move_windows: bool,
    pub include_match_point_window: bool,
    pub next_turn_window_scale_bp: i32,
    pub double_confirmed_score: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EvalTermContributions {
    pub confirmed_score: i32,
    pub consumable_score: i32,
    pub score_race_path_progress: i32,
    pub opponent_score_race_path_progress: i32,
    pub score_race_multi_path: i32,
    pub opponent_score_race_multi_path: i32,
    pub immediate_score_window: i32,
    pub opponent_immediate_score_window: i32,
    pub immediate_score_multi_window: i32,
    pub opponent_immediate_score_multi_window: i32,
    pub match_point_window: i32,
    pub residual_board_state: i32,
}

impl EvalTermContributions {
    pub fn sum(self) -> i32 {
        self.confirmed_score
            + self.consumable_score
            + self.score_race_path_progress
            + self.opponent_score_race_path_progress
            + self.score_race_multi_path
            + self.opponent_score_race_multi_path
            + self.immediate_score_window
            + self.opponent_immediate_score_window
            + self.immediate_score_multi_window
            + self.opponent_immediate_score_multi_window
            + self.match_point_window
            + self.residual_board_state
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EvalBreakdown {
    pub total: i32,
    pub terms: EvalTermContributions,
    pub features: EvalFeatureSnapshot,
}

pub fn evaluate_preferability(game: &MonsGame, color: Color) -> i32 {
    evaluate_preferability_with_weights(game, color, &DEFAULT_SCORING_WEIGHTS)
}

pub fn evaluate_preferability_breakdown(game: &MonsGame, color: Color) -> EvalBreakdown {
    evaluate_preferability_breakdown_with_weights(game, color, &DEFAULT_SCORING_WEIGHTS)
}

pub fn evaluate_preferability_breakdown_with_weights(
    game: &MonsGame,
    color: Color,
    weights: &ScoringWeights,
) -> EvalBreakdown {
    let use_legacy_formula = weights.use_legacy_formula;
    let include_regular_mana_move_windows =
        weights.include_regular_mana_move_windows && !use_legacy_formula;
    let include_match_point_window = weights.include_match_point_window && !use_legacy_formula;
    let next_turn_window_scale_bp = weights.next_turn_window_scale_bp.clamp(0, 20_000);
    let remaining_mon_moves_for_active =
        (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0);
    let offense_scale_bp = 10_000;
    let defense_scale_bp = 10_000;

    let total = evaluate_preferability_with_weights(game, color, weights);

    let score_diff = if color == Color::White {
        game.white_score - game.black_score
    } else {
        game.black_score - game.white_score
    };
    let potion_diff = if color == Color::White {
        game.white_potions_count - game.black_potions_count
    } else {
        game.black_potions_count - game.white_potions_count
    };

    let mut terms = EvalTermContributions {
        confirmed_score: score_diff * weights.confirmed_score,
        consumable_score: potion_diff * weights.has_consumable,
        ..EvalTermContributions::default()
    };

    if weights.double_confirmed_score {
        terms.confirmed_score *= weights.confirmed_score;
        terms.consumable_score *= weights.confirmed_score;
    }

    let my_score_now = if color == Color::White {
        game.white_score
    } else {
        game.black_score
    };
    let opponent_score_now = if color == Color::White {
        game.black_score
    } else {
        game.white_score
    };

    let mana_snapshot = ManaPathSnapshot::from_board(&game.board);
    let my_score_path_window = if use_legacy_formula {
        score_path_window_to_any_pool_with_snapshot(
            &game.board,
            &mana_snapshot,
            color,
            false,
            include_regular_mana_move_windows,
        )
    } else {
        exact_score_path_window_for_game(game, &mana_snapshot, color, include_regular_mana_move_windows)
    };
    let opponent_score_path_window = if use_legacy_formula {
        score_path_window_to_any_pool_with_snapshot(
            &game.board,
            &mana_snapshot,
            color.other(),
            false,
            include_regular_mana_move_windows,
        )
    } else {
        exact_score_path_window_for_game(
            game,
            &mana_snapshot,
            color.other(),
            include_regular_mana_move_windows,
        )
    };

    if let Some(steps) = my_score_path_window.best_steps {
        terms.score_race_path_progress = scale_by_bp(
            weights.score_race_path_progress / steps.max(1),
            offense_scale_bp,
        );
        if !use_legacy_formula {
            terms.score_race_multi_path = scale_by_bp(
                weights.score_race_multi_path * my_score_path_window.multi_pressure / 100,
                offense_scale_bp,
            );
        }
    }
    if let Some(steps) = opponent_score_path_window.best_steps {
        terms.opponent_score_race_path_progress = -scale_by_bp(
            weights.opponent_score_race_path_progress / steps.max(1),
            defense_scale_bp,
        );
        if !use_legacy_formula {
            terms.opponent_score_race_multi_path = -scale_by_bp(
                weights.opponent_score_race_multi_path * opponent_score_path_window.multi_pressure
                    / 100,
                defense_scale_bp,
            );
        }
    }

    let mut my_immediate_window = ImmediateScoreWindow::default();
    let mut opponent_immediate_window = ImmediateScoreWindow::default();

    if game.active_color == color {
        my_immediate_window = if use_legacy_formula {
            immediate_score_window_summary_with_snapshot(
                &game.board,
                &mana_snapshot,
                color,
                remaining_mon_moves_for_active,
                false,
                include_regular_mana_move_windows,
                include_regular_mana_move_windows && game.player_can_move_mana(),
            )
        } else {
            exact_immediate_score_window_for_game(
                game,
                &mana_snapshot,
                color,
                include_regular_mana_move_windows && game.player_can_move_mana(),
            )
        };
        terms.immediate_score_window = scale_by_bp(
            weights.immediate_score_window * my_immediate_window.best_score,
            offense_scale_bp,
        );
        if !use_legacy_formula {
            terms.immediate_score_multi_window = scale_by_bp(
                weights.immediate_score_multi_window * my_immediate_window.multi_pressure / 100,
                offense_scale_bp,
            );

            opponent_immediate_window = if use_legacy_formula {
                immediate_score_window_summary_with_snapshot(
                    &game.board,
                    &mana_snapshot,
                    color.other(),
                    Config::MONS_MOVES_PER_TURN,
                    true,
                    include_regular_mana_move_windows,
                    include_regular_mana_move_windows,
                )
            } else {
                exact_immediate_score_window_for_game(
                    game,
                    &mana_snapshot,
                    color.other(),
                    include_regular_mana_move_windows,
                )
            };
            terms.opponent_immediate_score_window = -scale_by_bp(
                (weights.opponent_immediate_score_window
                    * opponent_immediate_window.best_score
                    * next_turn_window_scale_bp)
                    / 10_000,
                defense_scale_bp,
            );
            terms.opponent_immediate_score_multi_window = -scale_by_bp(
                (weights.opponent_immediate_score_multi_window
                    * opponent_immediate_window.multi_pressure
                    * next_turn_window_scale_bp)
                    / 1_000_000,
                defense_scale_bp,
            );

            if include_match_point_window {
                if my_score_now + my_immediate_window.best_score >= Config::TARGET_SCORE {
                    terms.match_point_window += weights.immediate_winning_carrier;
                }
                if opponent_score_now + opponent_immediate_window.best_score >= Config::TARGET_SCORE
                {
                    terms.match_point_window -= weights.immediate_winning_carrier;
                }
            }
        }
    } else {
        opponent_immediate_window = if use_legacy_formula {
            immediate_score_window_summary_with_snapshot(
                &game.board,
                &mana_snapshot,
                color.other(),
                remaining_mon_moves_for_active,
                false,
                include_regular_mana_move_windows,
                include_regular_mana_move_windows && game.player_can_move_mana(),
            )
        } else {
            exact_immediate_score_window_for_game(
                game,
                &mana_snapshot,
                color.other(),
                include_regular_mana_move_windows && game.player_can_move_mana(),
            )
        };
        terms.opponent_immediate_score_window = -scale_by_bp(
            weights.opponent_immediate_score_window * opponent_immediate_window.best_score,
            defense_scale_bp,
        );
        if !use_legacy_formula {
            terms.opponent_immediate_score_multi_window = -scale_by_bp(
                weights.opponent_immediate_score_multi_window
                    * opponent_immediate_window.multi_pressure
                    / 100,
                defense_scale_bp,
            );
            my_immediate_window = if use_legacy_formula {
                immediate_score_window_summary_with_snapshot(
                    &game.board,
                    &mana_snapshot,
                    color,
                    Config::MONS_MOVES_PER_TURN,
                    true,
                    include_regular_mana_move_windows,
                    include_regular_mana_move_windows,
                )
            } else {
                exact_immediate_score_window_for_game(
                    game,
                    &mana_snapshot,
                    color,
                    include_regular_mana_move_windows,
                )
            };
            terms.immediate_score_window = scale_by_bp(
                (weights.immediate_score_window
                    * my_immediate_window.best_score
                    * next_turn_window_scale_bp)
                    / 10_000,
                offense_scale_bp,
            );
            terms.immediate_score_multi_window = scale_by_bp(
                (weights.immediate_score_multi_window
                    * my_immediate_window.multi_pressure
                    * next_turn_window_scale_bp)
                    / 1_000_000,
                offense_scale_bp,
            );

            if include_match_point_window {
                if opponent_score_now + opponent_immediate_window.best_score >= Config::TARGET_SCORE
                {
                    terms.match_point_window -= weights.immediate_winning_carrier;
                }
                if my_score_now + my_immediate_window.best_score >= Config::TARGET_SCORE {
                    terms.match_point_window += weights.immediate_winning_carrier;
                }
            }
        }
    }

    let known_total = terms.confirmed_score
        + terms.consumable_score
        + terms.score_race_path_progress
        + terms.opponent_score_race_path_progress
        + terms.score_race_multi_path
        + terms.opponent_score_race_multi_path
        + terms.immediate_score_window
        + terms.opponent_immediate_score_window
        + terms.immediate_score_multi_window
        + terms.opponent_immediate_score_multi_window
        + terms.match_point_window;
    terms.residual_board_state = total.saturating_sub(known_total);

    EvalBreakdown {
        total,
        terms,
        features: EvalFeatureSnapshot {
            active_color_is_perspective: game.active_color == color,
            remaining_mon_moves_for_active,
            my_score_path_best_steps: my_score_path_window.best_steps.unwrap_or(-1),
            my_score_path_multi_pressure: my_score_path_window.multi_pressure,
            opponent_score_path_best_steps: opponent_score_path_window.best_steps.unwrap_or(-1),
            opponent_score_path_multi_pressure: opponent_score_path_window.multi_pressure,
            my_immediate_best_score: my_immediate_window.best_score,
            my_immediate_multi_pressure: my_immediate_window.multi_pressure,
            opponent_immediate_best_score: opponent_immediate_window.best_score,
            opponent_immediate_multi_pressure: opponent_immediate_window.multi_pressure,
            include_regular_mana_move_windows,
            include_match_point_window,
            next_turn_window_scale_bp,
            double_confirmed_score: weights.double_confirmed_score,
        },
    }
}

pub fn evaluate_preferability_with_weights(
    game: &MonsGame,
    color: Color,
    weights: &ScoringWeights,
) -> i32 {
    let use_legacy_formula = weights.use_legacy_formula;
    let include_regular_mana_move_windows =
        weights.include_regular_mana_move_windows && !use_legacy_formula;
    let include_match_point_window = weights.include_match_point_window && !use_legacy_formula;
    let next_turn_window_scale_bp = weights.next_turn_window_scale_bp.clamp(0, 20_000);
    let supermana_base = game.board.supermana_base();
    let remaining_mon_moves_for_active =
        (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0);
    let mana_snapshot = ManaPathSnapshot::from_board(&game.board);
    let exact_analysis = exact_state_analysis(game);

    let mons_bases = Config::mons_bases_ref();
    let my_score_now = if color == Color::White {
        game.white_score
    } else {
        game.black_score
    };
    let opponent_score_now = if color == Color::White {
        game.black_score
    } else {
        game.white_score
    };
    let offense_scale_bp = 10_000;
    let defense_scale_bp = 10_000;

    let mut score = match color {
        Color::White => {
            (game.white_score - game.black_score) * weights.confirmed_score
                + (game.white_potions_count - game.black_potions_count) * weights.has_consumable
        }
        Color::Black => {
            (game.black_score - game.white_score) * weights.confirmed_score
                + (game.black_potions_count - game.white_potions_count) * weights.has_consumable
        }
    };

    if weights.double_confirmed_score {
        score *= weights.confirmed_score;
    }

    for (location, item) in game.board.occupied() {
        match item {
            Item::Mon { mon } => {
                let my_mon_multiplier = if mon.color == color { 1 } else { -1 };
                let is_drainer = mon.kind == MonKind::Drainer;

                if mon.is_fainted() {
                    score += my_mon_multiplier
                        * (if is_drainer {
                            weights.fainted_drainer
                        } else {
                            weights.fainted_mon
                        });
                    score += my_mon_multiplier * weights.fainted_cooldown_step * mon.cooldown;
                } else if is_drainer {
                    let (danger, min_mana, angel_nearby) =
                        drainer_distances(&game.board, mon.color, location, use_legacy_formula);
                    score += my_mon_multiplier * weights.drainer_close_to_mana / min_mana;
                    score += my_mon_multiplier * weights.drainer_close_to_own_pool
                        / distance(location, Destination::ClosestPool(mon.color));
                    score += my_mon_multiplier * weights.drainer_close_to_supermana
                        / distance_to_location(location, supermana_base);
                    if !angel_nearby {
                        score += my_mon_multiplier * weights.drainer_at_risk / danger;
                    } else {
                        score += my_mon_multiplier * weights.angel_guarding_drainer;
                    }

                    if let Some(path) = if use_legacy_formula {
                        best_drainer_pickup_path_with_snapshot(&mana_snapshot, mon.color, location)
                            .map(|(path_steps, mana_value)| (path_steps, path_steps + 1, mana_value))
                    } else {
                        exact_analysis
                            .color_summary(mon.color)
                            .best_drainer_pickup
                            .map(|path| (path.path_steps, path.total_moves, path.mana_value))
                    } {
                        let (path_steps, total_moves, mana_value) = path;
                        score += my_mon_multiplier * weights.drainer_best_mana_path * mana_value
                            / (path_steps + 1);
                        if mon.color == game.active_color && total_moves <= remaining_mon_moves_for_active {
                            score += my_mon_multiplier
                                * weights.drainer_pickup_score_this_turn
                                * mana_value;
                        }
                    }

                    let (action_threats, bomb_threats) = drainer_immediate_threats(
                        &game.board,
                        mon.color,
                        location,
                        use_legacy_formula,
                    );
                    let immediate_threats = if angel_nearby {
                        bomb_threats
                    } else {
                        action_threats + bomb_threats
                    };
                    if immediate_threats > 0 {
                        score += my_mon_multiplier
                            * weights.drainer_immediate_threat
                            * immediate_threats;
                    }

                    let evaluate_drainer_danger = weights.drainer_danger_boolean != 0
                        || weights.drainer_walk_threat_boolean != 0;
                    let drainer_under_danger_threat = evaluate_drainer_danger
                        && is_drainer_under_danger_threat(
                            &game.board,
                            mon.color,
                            location,
                            angel_nearby,
                            use_legacy_formula,
                        );
                    if weights.drainer_danger_boolean != 0 && drainer_under_danger_threat {
                        score += my_mon_multiplier * weights.drainer_danger_boolean;
                        if my_mon_multiplier == -1 {
                            score += weights.opponent_drainer_attack_bonus;
                        }
                    }

                    if weights.drainer_walk_threat_boolean != 0
                        && !drainer_under_danger_threat
                        && is_drainer_under_walk_threat(
                            &game.board,
                            mon.color,
                            location,
                            angel_nearby,
                        )
                    {
                        score += my_mon_multiplier * weights.drainer_walk_threat_boolean;
                    }
                } else if mon.kind == MonKind::Spirit {
                    let enemy_distance =
                        nearest_enemy_mon_distance(&game.board, mon.color, location);
                    score += my_mon_multiplier * weights.spirit_close_to_enemy / enemy_distance;
                    score -= my_mon_multiplier
                        * spirit_on_own_base_penalty(
                            &game.board,
                            *mon,
                            location,
                            weights.spirit_on_own_base_penalty,
                        );
                    let spirit_utility_cap = if use_legacy_formula { 4 } else { 6 };
                    let spirit_utility = if use_legacy_formula {
                        spirit_action_utility(&game.board, mon.color, location, true)
                    } else {
                        exact_analysis.color_summary(mon.color).spirit.utility
                    }
                    .min(spirit_utility_cap);
                    score += my_mon_multiplier * weights.spirit_action_utility * spirit_utility;
                } else if mon.kind == MonKind::Angel {
                    let friendly_drainer_distance =
                        nearest_friendly_drainer_distance(&game.board, mon.color, location);
                    score += my_mon_multiplier * weights.angel_close_to_friendly_drainer
                        / friendly_drainer_distance;
                } else if mon.kind != MonKind::Angel {
                    score += my_mon_multiplier * weights.mon_close_to_center
                        / distance(location, Destination::Center);
                }

                if weights.attacker_close_to_opponent_drainer != 0
                    && !mon.is_fainted()
                    && (mon.kind == MonKind::Demon || mon.kind == MonKind::Mystic)
                {
                    let opp_drainer_dist =
                        nearest_friendly_drainer_distance(&game.board, mon.color.other(), location);
                    score += my_mon_multiplier * weights.attacker_close_to_opponent_drainer
                        / opp_drainer_dist;
                }

                if !mons_bases.contains(&location) {
                    score += my_mon_multiplier * weights.active_mon;
                }
            }
            Item::MonWithConsumable { mon, .. } => {
                let my_mon_multiplier = if mon.color == color { 1 } else { -1 };
                let is_drainer = mon.kind == MonKind::Drainer;
                score += my_mon_multiplier * weights.has_consumable;

                if is_drainer {
                    let (danger, min_mana, angel_nearby) =
                        drainer_distances(&game.board, mon.color, location, use_legacy_formula);
                    score += my_mon_multiplier * weights.drainer_close_to_mana / min_mana;
                    score += my_mon_multiplier * weights.drainer_close_to_own_pool
                        / distance(location, Destination::ClosestPool(mon.color));
                    score += my_mon_multiplier * weights.drainer_close_to_supermana
                        / distance_to_location(location, supermana_base);
                    if !angel_nearby {
                        score += my_mon_multiplier * weights.drainer_at_risk / danger;
                    } else {
                        score += my_mon_multiplier * weights.angel_guarding_drainer;
                    }

                    let (action_threats, bomb_threats) = drainer_immediate_threats(
                        &game.board,
                        mon.color,
                        location,
                        use_legacy_formula,
                    );
                    let immediate_threats = if angel_nearby {
                        bomb_threats
                    } else {
                        action_threats + bomb_threats
                    };
                    if immediate_threats > 0 {
                        score += my_mon_multiplier
                            * weights.drainer_immediate_threat
                            * immediate_threats;
                    }

                    let evaluate_drainer_danger = weights.drainer_danger_boolean != 0
                        || weights.drainer_walk_threat_boolean != 0;
                    let drainer_under_danger_threat = evaluate_drainer_danger
                        && is_drainer_under_danger_threat(
                            &game.board,
                            mon.color,
                            location,
                            angel_nearby,
                            use_legacy_formula,
                        );
                    if weights.drainer_danger_boolean != 0 && drainer_under_danger_threat {
                        score += my_mon_multiplier * weights.drainer_danger_boolean;
                        if my_mon_multiplier == -1 {
                            score += weights.opponent_drainer_attack_bonus;
                        }
                    }

                    if weights.drainer_walk_threat_boolean != 0
                        && !drainer_under_danger_threat
                        && is_drainer_under_walk_threat(
                            &game.board,
                            mon.color,
                            location,
                            angel_nearby,
                        )
                    {
                        score += my_mon_multiplier * weights.drainer_walk_threat_boolean;
                    }
                    if !use_legacy_formula {
                        if let Some(path) = exact_analysis
                            .color_summary(mon.color)
                            .best_drainer_pickup
                            .map(|path| (path.path_steps, path.total_moves, path.mana_value))
                        {
                            let (path_steps, total_moves, mana_value) = path;
                            score +=
                                my_mon_multiplier * weights.drainer_best_mana_path * mana_value
                                    / (path_steps + 1);
                            if mon.color == game.active_color
                                && total_moves <= remaining_mon_moves_for_active
                            {
                                score += my_mon_multiplier
                                    * weights.drainer_pickup_score_this_turn
                                    * mana_value;
                            }
                        }
                    }
                } else if mon.kind == MonKind::Spirit {
                    let enemy_distance =
                        nearest_enemy_mon_distance(&game.board, mon.color, location);
                    score += my_mon_multiplier * weights.spirit_close_to_enemy / enemy_distance;
                    score -= my_mon_multiplier
                        * spirit_on_own_base_penalty(
                            &game.board,
                            *mon,
                            location,
                            weights.spirit_on_own_base_penalty,
                        );
                    let spirit_utility_cap = if use_legacy_formula { 4 } else { 6 };
                    let spirit_utility = if use_legacy_formula {
                        spirit_action_utility(&game.board, mon.color, location, true)
                    } else {
                        exact_analysis.color_summary(mon.color).spirit.utility
                    }
                    .min(spirit_utility_cap);
                    score += my_mon_multiplier * weights.spirit_action_utility * spirit_utility;
                } else if mon.kind == MonKind::Angel {
                    let friendly_drainer_distance =
                        nearest_friendly_drainer_distance(&game.board, mon.color, location);
                    score += my_mon_multiplier * weights.angel_close_to_friendly_drainer
                        / friendly_drainer_distance;
                } else if mon.kind != MonKind::Angel {
                    score += my_mon_multiplier * weights.mon_close_to_center
                        / distance(location, Destination::Center);
                }

                if weights.attacker_close_to_opponent_drainer != 0 && !mon.is_fainted() {
                    let is_attacker = mon.kind == MonKind::Demon
                        || mon.kind == MonKind::Mystic
                        || matches!(
                            item,
                            Item::MonWithConsumable {
                                consumable: Consumable::Bomb,
                                ..
                            }
                        );
                    if is_attacker {
                        let opp_drainer_dist = nearest_friendly_drainer_distance(
                            &game.board,
                            mon.color.other(),
                            location,
                        );
                        score += my_mon_multiplier * weights.attacker_close_to_opponent_drainer
                            / opp_drainer_dist;
                    }
                }

                if !use_legacy_formula && !mons_bases.contains(&location) {
                    score += my_mon_multiplier * weights.active_mon;
                }
            }
            Item::Mana { mana } => {
                score += weights.mana_close_to_same_pool
                    / distance(location, Destination::ClosestPool(color));
                let mana_bonus = match mana {
                    Mana::Regular(mana_color) => {
                        let owner_multiplier = if *mana_color == color { 1 } else { -1 };
                        let owner_pool_distance =
                            distance(location, Destination::ClosestPool(*mana_color));
                        let owner_drainer_distance =
                            nearest_friendly_drainer_distance(&game.board, *mana_color, location);
                        let enemy_drainer_distance = nearest_friendly_drainer_distance(
                            &game.board,
                            mana_color.other(),
                            location,
                        );
                        let drainer_control =
                            (enemy_drainer_distance - owner_drainer_distance).clamp(-4, 4);
                        let mut regular_bonus = owner_multiplier
                            * (weights.regular_mana_to_owner_pool / owner_pool_distance
                                + weights.regular_mana_drainer_control * drainer_control);
                        if !use_legacy_formula && *mana_color == color.other() {
                            regular_bonus += weights.opponent_mana_denial * (-drainer_control);
                        }
                        regular_bonus
                    }
                    Mana::Supermana => {
                        let my_drainer_distance =
                            nearest_friendly_drainer_distance(&game.board, color, location);
                        let enemy_drainer_distance =
                            nearest_friendly_drainer_distance(&game.board, color.other(), location);
                        let drainer_control =
                            (enemy_drainer_distance - my_drainer_distance).clamp(-4, 4);
                        weights.supermana_drainer_control * drainer_control
                            + if use_legacy_formula {
                                0
                            } else {
                                weights.supermana_race_control * drainer_control
                            }
                    }
                };
                score += mana_bonus;
            }
            Item::MonWithMana { mon, mana } => {
                let my_mon_multiplier = if mon.color == color { 1 } else { -1 };
                let nearest_pool_distance = distance(location, Destination::AnyClosestPool);
                let mana_extra = match mana {
                    Mana::Regular(mana_color) => {
                        if *mana_color == color {
                            0
                        } else {
                            weights.extra_for_opponents_mana
                        }
                    }
                    Mana::Supermana => weights.extra_for_supermana,
                };

                score += my_mon_multiplier * weights.drainer_holding_mana;
                score += my_mon_multiplier * (weights.mon_with_mana_close_to_any_pool + mana_extra)
                    / nearest_pool_distance;

                if nearest_pool_distance <= 2 {
                    let immediate_bonus = match mana {
                        Mana::Supermana => {
                            weights.mana_carrier_one_step_from_pool
                                + weights.supermana_carrier_one_step_from_pool_extra
                        }
                        Mana::Regular(_) => weights.mana_carrier_one_step_from_pool,
                    };
                    score += my_mon_multiplier * immediate_bonus;

                    let carrier_score = if mon.color == Color::White {
                        game.white_score
                    } else {
                        game.black_score
                    };
                    let score_if_scored_now = carrier_score + mana.score(mon.color);
                    if score_if_scored_now >= Config::TARGET_SCORE {
                        score += my_mon_multiplier * weights.immediate_winning_carrier;
                    }
                }

                let (danger, _, angel_nearby) =
                    drainer_distances(&game.board, mon.color, location, use_legacy_formula);
                score += my_mon_multiplier * weights.mana_carrier_at_risk / danger;
                if angel_nearby {
                    score += my_mon_multiplier * weights.mana_carrier_guarded;
                }

                if !use_legacy_formula && mon.kind == MonKind::Drainer {
                    let carries_high_value_mana = matches!(mana, Mana::Supermana)
                        || matches!(mana, Mana::Regular(owner) if *owner != mon.color);
                    if carries_high_value_mana {
                        let virtual_score_bp = match mana {
                            Mana::Supermana => weights
                                .supermana_race_control
                                .saturating_mul(PROTECTED_HIGH_VALUE_CARRIER_SUPERMANA_SCALE_BP),
                            Mana::Regular(owner) if *owner != mon.color => {
                                weights.opponent_mana_denial.saturating_mul(
                                    PROTECTED_HIGH_VALUE_CARRIER_OPPONENT_MANA_SCALE_BP,
                                )
                            }
                            Mana::Regular(_) => 0,
                        }
                        .clamp(0, PROTECTED_HIGH_VALUE_CARRIER_VIRTUAL_SCORE_BP_MAX);
                        let carrier_opponent_score = if mon.color == Color::White {
                            game.black_score
                        } else {
                            game.white_score
                        };
                        let opponent_score_limit = (Config::TARGET_SCORE
                            - PROTECTED_HIGH_VALUE_CARRIER_OPPONENT_SCORE_MARGIN)
                            .max(0);
                        let protected =
                            angel_nearby || danger >= PROTECTED_HIGH_VALUE_CARRIER_SAFE_DANGER_MIN;
                        if virtual_score_bp > 0
                            && protected
                            && carrier_opponent_score <= opponent_score_limit
                        {
                            let virtual_two_point_score = weights.confirmed_score.saturating_mul(2);
                            let virtual_bonus =
                                scale_by_bp(virtual_two_point_score, virtual_score_bp);
                            score += my_mon_multiplier * virtual_bonus;
                        }
                    }
                }

                if mon.color == game.active_color {
                    let pool_steps = nearest_pool_distance - 1;
                    if pool_steps <= remaining_mon_moves_for_active {
                        score += my_mon_multiplier * weights.mana_carrier_score_this_turn;
                    }
                }

                if mon.kind == MonKind::Drainer {
                    score += my_mon_multiplier * weights.drainer_close_to_own_pool
                        / distance(location, Destination::ClosestPool(mon.color));

                    let (action_threats, bomb_threats) = drainer_immediate_threats(
                        &game.board,
                        mon.color,
                        location,
                        use_legacy_formula,
                    );
                    let immediate_threats = if angel_nearby {
                        bomb_threats
                    } else {
                        action_threats + bomb_threats
                    };
                    if immediate_threats > 0 {
                        score += my_mon_multiplier
                            * weights.drainer_immediate_threat
                            * immediate_threats;
                    }

                    let evaluate_carrier_danger = weights.mana_carrier_danger_boolean != 0
                        || weights.mana_carrier_walk_threat_boolean != 0;
                    let drainer_under_danger_threat = evaluate_carrier_danger
                        && is_drainer_under_danger_threat(
                            &game.board,
                            mon.color,
                            location,
                            angel_nearby,
                            use_legacy_formula,
                        );
                    if weights.mana_carrier_danger_boolean != 0 && drainer_under_danger_threat {
                        score += my_mon_multiplier * weights.mana_carrier_danger_boolean;
                        if my_mon_multiplier == -1 {
                            score += weights.opponent_drainer_attack_bonus;
                        }
                    }

                    if weights.mana_carrier_walk_threat_boolean != 0
                        && !drainer_under_danger_threat
                        && is_drainer_under_walk_threat(
                            &game.board,
                            mon.color,
                            location,
                            angel_nearby,
                        )
                    {
                        score += my_mon_multiplier * weights.mana_carrier_walk_threat_boolean;
                    }
                } else if mon.kind == MonKind::Spirit {
                    score -= my_mon_multiplier
                        * spirit_on_own_base_penalty(
                            &game.board,
                            *mon,
                            location,
                            weights.spirit_on_own_base_penalty,
                        );
                    let spirit_utility_cap = if use_legacy_formula { 4 } else { 6 };
                    let spirit_utility = if use_legacy_formula {
                        spirit_action_utility(&game.board, mon.color, location, true)
                    } else {
                        exact_analysis.color_summary(mon.color).spirit.utility
                    }
                    .min(spirit_utility_cap);
                    score += my_mon_multiplier * weights.spirit_action_utility * spirit_utility;
                }

                if !use_legacy_formula && !mons_bases.contains(&location) {
                    score += my_mon_multiplier * weights.active_mon;
                }
            }
            Item::Consumable { .. } => {}
        }
    }

    let my_score_path_window = if use_legacy_formula {
        score_path_window_to_any_pool_with_snapshot(
            &game.board,
            &mana_snapshot,
            color,
            false,
            include_regular_mana_move_windows,
        )
    } else {
        exact_score_path_window_for_game(game, &mana_snapshot, color, include_regular_mana_move_windows)
    };
    let opponent_score_path_window = if use_legacy_formula {
        score_path_window_to_any_pool_with_snapshot(
            &game.board,
            &mana_snapshot,
            color.other(),
            false,
            include_regular_mana_move_windows,
        )
    } else {
        exact_score_path_window_for_game(
            game,
            &mana_snapshot,
            color.other(),
            include_regular_mana_move_windows,
        )
    };
    if let Some(steps) = my_score_path_window.best_steps {
        score += scale_by_bp(
            weights.score_race_path_progress / steps.max(1),
            offense_scale_bp,
        );
        if !use_legacy_formula {
            score += scale_by_bp(
                weights.score_race_multi_path * my_score_path_window.multi_pressure / 100,
                offense_scale_bp,
            );
        }
    }
    if let Some(steps) = opponent_score_path_window.best_steps {
        score -= scale_by_bp(
            weights.opponent_score_race_path_progress / steps.max(1),
            defense_scale_bp,
        );
        if !use_legacy_formula {
            score -= scale_by_bp(
                weights.opponent_score_race_multi_path * opponent_score_path_window.multi_pressure
                    / 100,
                defense_scale_bp,
            );
        }
    }

    if game.active_color == color {
        let immediate_window = if use_legacy_formula {
            immediate_score_window_summary_with_snapshot(
                &game.board,
                &mana_snapshot,
                color,
                remaining_mon_moves_for_active,
                false,
                include_regular_mana_move_windows,
                include_regular_mana_move_windows && game.player_can_move_mana(),
            )
        } else {
            exact_immediate_score_window_for_game(
                game,
                &mana_snapshot,
                color,
                include_regular_mana_move_windows && game.player_can_move_mana(),
            )
        };
        score += scale_by_bp(
            weights.immediate_score_window * immediate_window.best_score,
            offense_scale_bp,
        );
        if !use_legacy_formula {
            score += scale_by_bp(
                weights.immediate_score_multi_window * immediate_window.multi_pressure / 100,
                offense_scale_bp,
            );

            let opponent_next_turn_window = if use_legacy_formula {
                immediate_score_window_summary_with_snapshot(
                    &game.board,
                    &mana_snapshot,
                    color.other(),
                    Config::MONS_MOVES_PER_TURN,
                    true,
                    include_regular_mana_move_windows,
                    include_regular_mana_move_windows,
                )
            } else {
                exact_immediate_score_window_for_game(
                    game,
                    &mana_snapshot,
                    color.other(),
                    include_regular_mana_move_windows,
                )
            };
            score -= scale_by_bp(
                (weights.opponent_immediate_score_window
                    * opponent_next_turn_window.best_score
                    * next_turn_window_scale_bp)
                    / 10_000,
                defense_scale_bp,
            );
            score -= scale_by_bp(
                (weights.opponent_immediate_score_multi_window
                    * opponent_next_turn_window.multi_pressure
                    * next_turn_window_scale_bp)
                    / 1_000_000,
                defense_scale_bp,
            );
            if include_match_point_window {
                if my_score_now + immediate_window.best_score >= Config::TARGET_SCORE {
                    score += weights.immediate_winning_carrier;
                }
                if opponent_score_now + opponent_next_turn_window.best_score >= Config::TARGET_SCORE
                {
                    score -= weights.immediate_winning_carrier;
                }
            }
        }
    } else {
        let opponent_immediate_window = if use_legacy_formula {
            immediate_score_window_summary_with_snapshot(
                &game.board,
                &mana_snapshot,
                color.other(),
                remaining_mon_moves_for_active,
                false,
                include_regular_mana_move_windows,
                include_regular_mana_move_windows && game.player_can_move_mana(),
            )
        } else {
            exact_immediate_score_window_for_game(
                game,
                &mana_snapshot,
                color.other(),
                include_regular_mana_move_windows && game.player_can_move_mana(),
            )
        };
        score -= scale_by_bp(
            weights.opponent_immediate_score_window * opponent_immediate_window.best_score,
            defense_scale_bp,
        );
        if !use_legacy_formula {
            score -= scale_by_bp(
                weights.opponent_immediate_score_multi_window
                    * opponent_immediate_window.multi_pressure
                    / 100,
                defense_scale_bp,
            );

            let my_next_turn_window = if use_legacy_formula {
                immediate_score_window_summary_with_snapshot(
                    &game.board,
                    &mana_snapshot,
                    color,
                    Config::MONS_MOVES_PER_TURN,
                    true,
                    include_regular_mana_move_windows,
                    include_regular_mana_move_windows,
                )
            } else {
                exact_immediate_score_window_for_game(
                    game,
                    &mana_snapshot,
                    color,
                    include_regular_mana_move_windows,
                )
            };
            score += scale_by_bp(
                (weights.immediate_score_window
                    * my_next_turn_window.best_score
                    * next_turn_window_scale_bp)
                    / 10_000,
                offense_scale_bp,
            );
            score += scale_by_bp(
                (weights.immediate_score_multi_window
                    * my_next_turn_window.multi_pressure
                    * next_turn_window_scale_bp)
                    / 1_000_000,
                offense_scale_bp,
            );
            if include_match_point_window {
                if opponent_score_now + opponent_immediate_window.best_score >= Config::TARGET_SCORE
                {
                    score -= weights.immediate_winning_carrier;
                }
                if my_score_now + my_next_turn_window.best_score >= Config::TARGET_SCORE {
                    score += weights.immediate_winning_carrier;
                }
            }
        }
    }

    score
}

fn scale_by_bp(value: i32, basis_points: i32) -> i32 {
    ((value as i64 * basis_points as i64) / 10_000) as i32
}

fn spirit_on_own_base_penalty(board: &Board, mon: Mon, location: Location, penalty: i32) -> i32 {
    if mon.kind == MonKind::Spirit && !mon.is_fainted() && location == board.base(mon) {
        penalty
    } else {
        0
    }
}

fn exact_score_path_window_for_game(
    game: &MonsGame,
    mana_snapshot: &ManaPathSnapshot,
    color: Color,
    include_regular_mana_move_windows: bool,
) -> ScorePathWindow {
    let exact = exact_state_analysis(game).color_summary(color).score_path_window;
    let mut best_steps = exact.best_steps;
    if include_regular_mana_move_windows {
        for candidate in &mana_snapshot.candidates {
            if candidate.mana == Mana::Regular(color) {
                let candidate_steps = candidate.score_steps + 1;
                best_steps = Some(best_steps.map_or(candidate_steps, |best| best.min(candidate_steps)));
            }
        }
    }
    ScorePathWindow {
        best_steps,
        multi_pressure: exact.multi_pressure,
    }
}

fn exact_immediate_score_window_for_game(
    game: &MonsGame,
    mana_snapshot: &ManaPathSnapshot,
    color: Color,
    allow_mana_move: bool,
) -> ImmediateScoreWindow {
    let exact = exact_state_analysis(game).color_summary(color).immediate_window;
    let mut best_score = exact.best_score;
    if allow_mana_move {
        best_score = best_score.max(best_regular_mana_move_score_window_with_snapshot(
            mana_snapshot,
            color,
        ));
    }
    ImmediateScoreWindow {
        best_score,
        multi_pressure: exact.multi_pressure,
    }
}

fn spirit_action_utility(
    board: &Board,
    spirit_color: Color,
    location: Location,
    use_legacy_formula: bool,
) -> i32 {
    if use_legacy_formula {
        return location
            .reachable_by_spirit_action_ref()
            .iter()
            .copied()
            .filter(|target| {
                let Some(item) = board.item(*target) else {
                    return false;
                };
                match item {
                    Item::Mon { mon }
                    | Item::MonWithMana { mon, .. }
                    | Item::MonWithConsumable { mon, .. } => !mon.is_fainted(),
                    Item::Mana { .. } | Item::Consumable { .. } => true,
                }
            })
            .count() as i32;
    }

    let utility = board
        .occupied()
        .find_map(|(occupied, item)| {
            let mon = item.mon()?;
            (occupied == location
                && mon.kind == MonKind::Spirit
                && mon.color == spirit_color
                && !mon.is_fainted())
            .then_some(())
        })
        .map(|_| {
            let mut game = MonsGame::new(false);
            game.board = board.clone();
            game.active_color = spirit_color;
            game.turn_number = 2;
            exact_state_analysis(&game).color_summary(spirit_color).spirit.utility
        })
        .unwrap_or(0);
    utility.max(
        location
            .reachable_by_spirit_action_ref()
            .iter()
            .filter(|target| board.item(**target).is_some())
            .count() as i32,
    )
}

#[derive(Clone, Copy, Default)]
struct ScorePathWindow {
    best_steps: Option<i32>,
    multi_pressure: i32,
}

#[derive(Clone, Copy, Default)]
struct ImmediateScoreWindow {
    best_score: i32,
    multi_pressure: i32,
}

#[derive(Clone, Copy)]
struct ManaPathCandidate {
    location: Location,
    score_steps: i32,
    mana: Mana,
}

#[derive(Default)]
struct ManaPathSnapshot {
    candidates: Vec<ManaPathCandidate>,
    regular_mana_move_scores: [i32; 2],
}

impl ManaPathSnapshot {
    fn from_board(board: &Board) -> Self {
        let mut snapshot = Self {
            candidates: Vec::with_capacity(16),
            regular_mana_move_scores: [0; 2],
        };
        for (location, item) in board.occupied() {
            let Item::Mana { mana } = item else {
                continue;
            };
            let score_steps = distance(location, Destination::AnyClosestPool) - 1;
            snapshot.candidates.push(ManaPathCandidate {
                location,
                score_steps,
                mana: *mana,
            });
            if score_steps <= 1 {
                if *mana == Mana::Regular(Color::White) {
                    snapshot.regular_mana_move_scores[color_slot(Color::White)] = mana.score(Color::White);
                } else if *mana == Mana::Regular(Color::Black) {
                    snapshot.regular_mana_move_scores[color_slot(Color::Black)] = mana.score(Color::Black);
                }
            }
        }
        snapshot
    }

    #[inline]
    fn regular_mana_move_score(&self, color: Color) -> i32 {
        self.regular_mana_move_scores[color_slot(color)]
    }
}

#[inline]
fn color_slot(color: Color) -> usize {
    if color == Color::White {
        0
    } else {
        1
    }
}

#[allow(dead_code)]
fn score_path_window_to_any_pool(
    board: &Board,
    color: Color,
    include_drainer_pickups: bool,
    include_regular_mana_move_windows: bool,
) -> ScorePathWindow {
    let mana_snapshot = ManaPathSnapshot::from_board(board);
    score_path_window_to_any_pool_with_snapshot(
        board,
        &mana_snapshot,
        color,
        include_drainer_pickups,
        include_regular_mana_move_windows,
    )
}

fn score_path_window_to_any_pool_with_snapshot(
    board: &Board,
    mana_snapshot: &ManaPathSnapshot,
    color: Color,
    include_drainer_pickups: bool,
    include_regular_mana_move_windows: bool,
) -> ScorePathWindow {
    let mut top_steps = [i32::MAX; 3];

    for (location, item) in board.occupied() {
        let Item::MonWithMana { mon, .. } = item else {
            continue;
        };
        if mon.color != color || mon.is_fainted() {
            continue;
        }
        insert_lowest_step(
            &mut top_steps,
            distance(location, Destination::AnyClosestPool),
        );
    }

    if include_drainer_pickups {
        for (location, item) in board.occupied() {
            let Some(mon) = item.mon() else {
                continue;
            };
            if mon.color != color || mon.kind != MonKind::Drainer || mon.is_fainted() {
                continue;
            }
            if let Some((path_steps, _)) =
                best_drainer_pickup_path_with_snapshot(mana_snapshot, color, location)
            {
                insert_lowest_step(&mut top_steps, path_steps + 1);
            }
        }
    }

    if include_regular_mana_move_windows {
        for candidate in &mana_snapshot.candidates {
            if candidate.mana == Mana::Regular(color) {
                insert_lowest_step(&mut top_steps, candidate.score_steps + 1);
            }
        }
    }

    let best_steps = (top_steps[0] != i32::MAX).then_some(top_steps[0]);
    let mut multi_pressure = 0i32;
    if top_steps[1] != i32::MAX {
        multi_pressure += 70 / top_steps[1].max(1);
    }
    if top_steps[2] != i32::MAX {
        multi_pressure += 40 / top_steps[2].max(1);
    }

    ScorePathWindow {
        best_steps,
        multi_pressure,
    }
}

#[allow(dead_code)]
fn immediate_score_window_summary(
    board: &Board,
    color: Color,
    remaining_mon_moves: i32,
    include_drainer_pickups: bool,
    include_regular_mana_move_windows: bool,
    allow_mana_move: bool,
) -> ImmediateScoreWindow {
    let mana_snapshot = ManaPathSnapshot::from_board(board);
    immediate_score_window_summary_with_snapshot(
        board,
        &mana_snapshot,
        color,
        remaining_mon_moves,
        include_drainer_pickups,
        include_regular_mana_move_windows,
        allow_mana_move,
    )
}

fn immediate_score_window_summary_with_snapshot(
    board: &Board,
    mana_snapshot: &ManaPathSnapshot,
    color: Color,
    remaining_mon_moves: i32,
    include_drainer_pickups: bool,
    include_regular_mana_move_windows: bool,
    allow_mana_move: bool,
) -> ImmediateScoreWindow {
    if remaining_mon_moves <= 0 {
        return ImmediateScoreWindow::default();
    }

    let mut top_scores = [0i32; 3];

    for (location, item) in board.occupied() {
        let Item::MonWithMana { mon, mana } = item else {
            continue;
        };
        if mon.color != color || mon.is_fainted() {
            continue;
        }
        let pool_steps = distance(location, Destination::AnyClosestPool) - 1;
        if pool_steps <= remaining_mon_moves {
            insert_top_score(&mut top_scores, mana.score(color));
        }
    }

    if include_drainer_pickups {
        for (location, item) in board.occupied() {
            let Some(mon) = item.mon() else {
                continue;
            };
            if mon.color != color || mon.kind != MonKind::Drainer || mon.is_fainted() {
                continue;
            }
            let mut best_pickup_score = 0;
            for candidate in &mana_snapshot.candidates {
                let pickup_steps = location.distance(&candidate.location) as i32;
                if pickup_steps + candidate.score_steps <= remaining_mon_moves {
                    best_pickup_score = best_pickup_score.max(candidate.mana.score(color));
                }
            }
            if best_pickup_score > 0 {
                insert_top_score(&mut top_scores, best_pickup_score);
            }
        }
    }

    if include_regular_mana_move_windows && allow_mana_move {
        let mana_move_immediate =
            best_regular_mana_move_score_window_with_snapshot(mana_snapshot, color);
        if mana_move_immediate > 0 {
            insert_top_score(&mut top_scores, mana_move_immediate);
        }
    }

    ImmediateScoreWindow {
        best_score: top_scores[0],
        multi_pressure: top_scores[1] * 70 + top_scores[2] * 35,
    }
}

#[allow(dead_code)]
fn best_regular_mana_move_score_window(board: &Board, color: Color) -> i32 {
    let mana_snapshot = ManaPathSnapshot::from_board(board);
    best_regular_mana_move_score_window_with_snapshot(&mana_snapshot, color)
}

fn best_regular_mana_move_score_window_with_snapshot(
    mana_snapshot: &ManaPathSnapshot,
    color: Color,
) -> i32 {
    mana_snapshot.regular_mana_move_score(color)
}

fn insert_lowest_step(top_steps: &mut [i32; 3], step: i32) {
    if step >= top_steps[2] {
        return;
    }

    if step < top_steps[0] {
        top_steps[2] = top_steps[1];
        top_steps[1] = top_steps[0];
        top_steps[0] = step;
    } else if step < top_steps[1] {
        top_steps[2] = top_steps[1];
        top_steps[1] = step;
    } else {
        top_steps[2] = step;
    }
}

fn insert_top_score(top_scores: &mut [i32; 3], score: i32) {
    if score <= top_scores[2] {
        return;
    }

    if score > top_scores[0] {
        top_scores[2] = top_scores[1];
        top_scores[1] = top_scores[0];
        top_scores[0] = score;
    } else if score > top_scores[1] {
        top_scores[2] = top_scores[1];
        top_scores[1] = score;
    } else {
        top_scores[2] = score;
    }
}

enum Destination {
    Center,
    AnyClosestPool,
    ClosestPool(Color),
}

fn drainer_distances(
    board: &Board,
    color: Color,
    location: Location,
    use_legacy_formula: bool,
) -> (i32, i32, bool) {
    let mut min_mana = Config::BOARD_SIZE as i32;
    let mut min_danger = Config::BOARD_SIZE as i32;
    let mut angel_nearby = false;

    for (item_location, item) in board.occupied() {
        match item {
            Item::Mana { .. } => {
                let delta = item_location.distance(&location) as i32;
                if delta < min_mana {
                    min_mana = delta;
                }
            }
            Item::Mon { mon }
            | Item::MonWithMana { mon, .. }
            | Item::MonWithConsumable { mon, .. } => {
                if use_legacy_formula && matches!(item, Item::MonWithMana { .. }) {
                    continue;
                }
                if mon.color != color && !mon.is_fainted() {
                    let mut delta = None;
                    if use_legacy_formula {
                        if mon.kind == MonKind::Mystic
                            || mon.kind == MonKind::Demon
                            || matches!(item, Item::MonWithConsumable { .. })
                        {
                            delta = Some(item_location.distance(&location) as i32);
                        }
                    } else {
                        if mon.kind == MonKind::Mystic || mon.kind == MonKind::Demon {
                            delta = Some(item_location.distance(&location) as i32);
                        }
                        if matches!(
                            item,
                            Item::MonWithConsumable {
                                consumable: Consumable::Bomb,
                                ..
                            }
                        ) {
                            let bomb_delta = (item_location.distance(&location) as i32 - 2).max(1);
                            delta = Some(delta.map_or(bomb_delta, |base| base.min(bomb_delta)));
                        }
                    }
                    if let Some(delta) = delta {
                        if delta < min_danger {
                            min_danger = delta;
                        }
                    }
                } else if mon.color == color
                    && !mon.is_fainted()
                    && mon.kind == MonKind::Angel
                    && item_location.distance(&location) == 1
                {
                    angel_nearby = true;
                }
            }
            Item::Consumable { .. } => {
                if use_legacy_formula {
                    let delta = item_location.distance(&location) as i32;
                    if delta < min_danger {
                        min_danger = delta;
                    }
                }
            }
        }
    }

    if use_legacy_formula {
        (min_danger, min_mana, angel_nearby)
    } else {
        (min_danger.max(1), min_mana.max(1), angel_nearby)
    }
}

#[allow(dead_code)]
fn best_drainer_pickup_path(board: &Board, color: Color, from: Location) -> Option<(i32, i32)> {
    let mana_snapshot = ManaPathSnapshot::from_board(board);
    best_drainer_pickup_path_with_snapshot(&mana_snapshot, color, from)
}

fn best_drainer_pickup_path_with_snapshot(
    mana_snapshot: &ManaPathSnapshot,
    color: Color,
    from: Location,
) -> Option<(i32, i32)> {
    let mut best: Option<(i32, i32)> = None;
    for candidate in &mana_snapshot.candidates {
        let pickup_steps = from.distance(&candidate.location) as i32;
        let score_steps = candidate.score_steps;
        let total_steps = pickup_steps + score_steps;
        let mana_value = candidate.mana.score(color);

        let replace = match best {
            None => true,
            Some((best_steps, best_mana_value)) => {
                let total_metric = total_steps * 3 - mana_value;
                let best_metric = best_steps * 3 - best_mana_value;
                total_metric < best_metric
                    || (total_metric == best_metric && mana_value > best_mana_value)
            }
        };
        if replace {
            best = Some((total_steps, mana_value));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_with_items(items: Vec<(Location, Item)>, active_color: Color) -> MonsGame {
        let mut game = MonsGame::new(false);
        game.board = Board::new_with_items(items.into_iter().collect());
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

    fn exact_danger_only_weights() -> ScoringWeights {
        ScoringWeights {
            use_legacy_formula: false,
            include_regular_mana_move_windows: false,
            include_match_point_window: false,
            next_turn_window_scale_bp: 0,
            double_confirmed_score: false,
            confirmed_score: 0,
            fainted_mon: 0,
            fainted_drainer: 0,
            fainted_cooldown_step: 0,
            drainer_at_risk: 0,
            mana_close_to_same_pool: 0,
            mon_with_mana_close_to_any_pool: 0,
            extra_for_supermana: 0,
            extra_for_opponents_mana: 0,
            drainer_close_to_mana: 0,
            drainer_holding_mana: 0,
            drainer_close_to_own_pool: 0,
            drainer_close_to_supermana: 0,
            mon_close_to_center: 0,
            spirit_close_to_enemy: 0,
            spirit_on_own_base_penalty: 0,
            angel_guarding_drainer: 0,
            angel_close_to_friendly_drainer: 0,
            has_consumable: 0,
            active_mon: 0,
            regular_mana_to_owner_pool: 0,
            regular_mana_drainer_control: 0,
            supermana_drainer_control: 0,
            supermana_race_control: 0,
            opponent_mana_denial: 0,
            mana_carrier_at_risk: 0,
            mana_carrier_guarded: 0,
            mana_carrier_one_step_from_pool: 0,
            supermana_carrier_one_step_from_pool_extra: 0,
            immediate_winning_carrier: 0,
            drainer_best_mana_path: 0,
            drainer_pickup_score_this_turn: 0,
            mana_carrier_score_this_turn: 0,
            drainer_immediate_threat: 0,
            score_race_path_progress: 0,
            opponent_score_race_path_progress: 0,
            score_race_multi_path: 0,
            opponent_score_race_multi_path: 0,
            immediate_score_window: 0,
            opponent_immediate_score_window: 0,
            immediate_score_multi_window: 0,
            opponent_immediate_score_multi_window: 0,
            spirit_action_utility: 0,
            drainer_danger_boolean: 0,
            mana_carrier_danger_boolean: 0,
            drainer_walk_threat_boolean: 0,
            mana_carrier_walk_threat_boolean: 0,
            opponent_drainer_attack_bonus: 0,
            attacker_close_to_opponent_drainer: 0,
        }
    }

    fn swapped_color(color: Color) -> Color {
        color.other()
    }

    fn mirror_location(location: Location) -> Location {
        Location::new(Config::MAX_LOCATION_INDEX - location.i, location.j)
    }

    fn mirror_item(item: &Item) -> Item {
        match item {
            Item::Mon { mon } => Item::Mon {
                mon: Mon::new(mon.kind, swapped_color(mon.color), mon.cooldown),
            },
            Item::MonWithMana { mon, mana } => Item::MonWithMana {
                mon: Mon::new(mon.kind, swapped_color(mon.color), mon.cooldown),
                mana: match mana {
                    Mana::Regular(color) => Mana::Regular(swapped_color(*color)),
                    Mana::Supermana => Mana::Supermana,
                },
            },
            Item::MonWithConsumable { mon, consumable } => Item::MonWithConsumable {
                mon: Mon::new(mon.kind, swapped_color(mon.color), mon.cooldown),
                consumable: *consumable,
            },
            Item::Mana { mana } => Item::Mana {
                mana: match mana {
                    Mana::Regular(color) => Mana::Regular(swapped_color(*color)),
                    Mana::Supermana => Mana::Supermana,
                },
            },
            Item::Consumable { consumable } => Item::Consumable {
                consumable: *consumable,
            },
        }
    }

    fn mirrored_game_with_swapped_colors(game: &MonsGame) -> MonsGame {
        let mirrored_items = game
            .board
            .occupied()
            .map(|(location, item)| (mirror_location(location), mirror_item(item)))
            .collect::<std::collections::HashMap<_, _>>();
        let mut mirrored = MonsGame::new(false);
        mirrored.board = Board::new_with_items(mirrored_items);
        mirrored.active_color = swapped_color(game.active_color);
        mirrored.actions_used_count = game.actions_used_count;
        mirrored.mana_moves_count = game.mana_moves_count;
        mirrored.mons_moves_count = game.mons_moves_count;
        mirrored.turn_number = game.turn_number;
        mirrored.white_score = game.black_score;
        mirrored.black_score = game.white_score;
        mirrored.white_potions_count = game.black_potions_count;
        mirrored.black_potions_count = game.white_potions_count;
        mirrored
    }

    #[test]
    fn swift_2024_reference_weights_match_historical_values() {
        let weights = SWIFT_2024_REFERENCE_SCORING_WEIGHTS;
        assert!(weights.use_legacy_formula);
        assert!(weights.double_confirmed_score);
        assert_eq!(weights.confirmed_score, 1000);
        assert_eq!(weights.fainted_mon, -500);
        assert_eq!(weights.fainted_drainer, -800);
        assert_eq!(weights.drainer_at_risk, -350);
        assert_eq!(weights.mana_close_to_same_pool, 500);
        assert_eq!(weights.mon_with_mana_close_to_any_pool, 800);
        assert_eq!(weights.extra_for_supermana, 120);
        assert_eq!(weights.extra_for_opponents_mana, 100);
        assert_eq!(weights.drainer_close_to_mana, 300);
        assert_eq!(weights.drainer_holding_mana, 350);
        assert_eq!(weights.mon_close_to_center, 210);
        assert_eq!(weights.has_consumable, 110);
        assert_eq!(weights.active_mon, 50);
    }

    #[test]
    fn spirit_on_own_base_penalty_applies_for_awake_spirit_on_base() {
        let spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let board = Board::new();
        let base = board.base(spirit);
        assert_eq!(
            spirit_on_own_base_penalty(
                &board,
                spirit,
                base,
                DEFAULT_SCORING_WEIGHTS.spirit_on_own_base_penalty
            ),
            DEFAULT_SCORING_WEIGHTS.spirit_on_own_base_penalty
        );
    }

    #[test]
    fn spirit_on_own_base_penalty_is_zero_off_base_or_fainted() {
        let board = Board::new();

        let awake_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let awake_base = board.base(awake_spirit);
        let off_base = if awake_base.i == 0 {
            Location::new(awake_base.i + 1, awake_base.j)
        } else {
            Location::new(awake_base.i - 1, awake_base.j)
        };
        assert_eq!(
            spirit_on_own_base_penalty(
                &board,
                awake_spirit,
                off_base,
                DEFAULT_SCORING_WEIGHTS.spirit_on_own_base_penalty
            ),
            0
        );

        let fainted_spirit = Mon::new(MonKind::Spirit, Color::White, 1);
        let fainted_base = board.base(fainted_spirit);
        assert_eq!(
            spirit_on_own_base_penalty(
                &board,
                fainted_spirit,
                fainted_base,
                DEFAULT_SCORING_WEIGHTS.spirit_on_own_base_penalty
            ),
            0
        );
    }

    #[test]
    fn immediate_score_window_detects_carrier_scoring_this_turn() {
        let board = Board::new_with_items(
            vec![(
                Location::new(9, 0),
                Item::MonWithMana {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    mana: Mana::Regular(Color::Black),
                },
            )]
            .into_iter()
            .collect(),
        );
        let window = immediate_score_window_summary(&board, Color::White, 3, true, true, true);
        assert_eq!(
            window.best_score,
            Mana::Regular(Color::Black).score(Color::White)
        );
    }

    #[test]
    fn regular_mana_move_window_requires_allow_mana_move() {
        let board = Board::new_with_items(
            vec![(
                Location::new(9, 0),
                Item::Mana {
                    mana: Mana::Regular(Color::White),
                },
            )]
            .into_iter()
            .collect(),
        );
        let disallowed = immediate_score_window_summary(&board, Color::White, 3, true, true, false);
        let allowed = immediate_score_window_summary(&board, Color::White, 3, true, true, true);
        assert_eq!(disallowed.best_score, 0);
        assert!(allowed.best_score > 0);
    }

    #[test]
    fn opponent_next_turn_window_penalizes_preferability() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 6),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(1, 0),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
        );
        let mut zero_threat = RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
        zero_threat.opponent_immediate_score_window = 0;
        zero_threat.opponent_immediate_score_multi_window = 0;
        let mut with_threat = zero_threat;
        with_threat.opponent_immediate_score_window = 400;
        with_threat.opponent_immediate_score_multi_window = 120;

        let score_zero = evaluate_preferability_with_weights(&game, Color::White, &zero_threat);
        let score_threat = evaluate_preferability_with_weights(&game, Color::White, &with_threat);
        assert!(
            score_threat < score_zero,
            "opponent immediate threat should lower preferability"
        );
    }

    #[test]
    fn exact_multi_step_drainer_danger_penalty_reduces_preferability() {
        let threatened = game_with_items(
            vec![
                (
                    Location::new(8, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(4, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        let safe = game_with_items(
            vec![(
                Location::new(8, 5),
                Item::Mon {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                },
            )],
            Color::White,
        );

        assert!(!crate::models::automove_exact::is_drainer_under_immediate_threat(
            &threatened.board,
            Color::White,
            Location::new(8, 5),
            false,
        ));
        assert!(!crate::models::automove_exact::is_drainer_under_walk_threat(
            &threatened.board,
            Color::White,
            Location::new(8, 5),
            false,
        ));

        let mut weights = exact_danger_only_weights();
        weights.drainer_danger_boolean = -500;

        let threatened_score =
            evaluate_preferability_with_weights(&threatened, Color::White, &weights);
        let safe_score = evaluate_preferability_with_weights(&safe, Color::White, &weights);

        assert!(
            threatened_score < safe_score,
            "exact multi-step drainer attack should reduce preferability (threatened={}, safe={})",
            threatened_score,
            safe_score
        );
    }

    #[test]
    fn exact_multi_step_mana_carrier_danger_penalty_reduces_preferability() {
        let threatened = game_with_items(
            vec![
                (
                    Location::new(8, 5),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(4, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        let safe = game_with_items(
            vec![(
                Location::new(8, 5),
                Item::MonWithMana {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    mana: Mana::Supermana,
                },
            )],
            Color::White,
        );

        assert!(!crate::models::automove_exact::is_drainer_under_immediate_threat(
            &threatened.board,
            Color::White,
            Location::new(8, 5),
            false,
        ));
        assert!(!crate::models::automove_exact::is_drainer_under_walk_threat(
            &threatened.board,
            Color::White,
            Location::new(8, 5),
            false,
        ));

        let mut weights = exact_danger_only_weights();
        weights.mana_carrier_danger_boolean = -700;

        let threatened_score =
            evaluate_preferability_with_weights(&threatened, Color::White, &weights);
        let safe_score = evaluate_preferability_with_weights(&safe, Color::White, &weights);

        assert!(
            threatened_score < safe_score,
            "exact multi-step carrier attack should reduce preferability (threatened={}, safe={})",
            threatened_score,
            safe_score
        );
    }

    #[test]
    fn match_point_window_applies_immediate_winning_bonus() {
        let mut game = game_with_items(
            vec![(
                Location::new(9, 0),
                Item::MonWithMana {
                    mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    mana: Mana::Supermana,
                },
            )],
            Color::White,
        );
        game.white_score = Config::TARGET_SCORE - 1;

        let mut without_match_point = RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
        without_match_point.include_match_point_window = false;
        without_match_point.immediate_score_window = 0;
        without_match_point.opponent_immediate_score_window = 0;
        without_match_point.immediate_score_multi_window = 0;
        without_match_point.opponent_immediate_score_multi_window = 0;
        without_match_point.immediate_winning_carrier = 520;

        let mut with_match_point = without_match_point;
        with_match_point.include_match_point_window = true;

        let score_without =
            evaluate_preferability_with_weights(&game, Color::White, &without_match_point);
        let score_with =
            evaluate_preferability_with_weights(&game, Color::White, &with_match_point);
        assert_eq!(
            score_with - score_without,
            with_match_point.immediate_winning_carrier
        );
    }

    #[test]
    fn spirit_off_base_is_preferred_when_penalty_is_enabled() {
        let base = Board::new().base(Mon::new(MonKind::Spirit, Color::White, 0));
        let off_base = if base.i > 0 {
            Location::new(base.i - 1, base.j)
        } else {
            Location::new(base.i + 1, base.j)
        };
        let on_base_game = game_with_items(
            vec![(
                base,
                Item::Mon {
                    mon: Mon::new(MonKind::Spirit, Color::White, 0),
                },
            )],
            Color::White,
        );
        let off_base_game = game_with_items(
            vec![(
                off_base,
                Item::Mon {
                    mon: Mon::new(MonKind::Spirit, Color::White, 0),
                },
            )],
            Color::White,
        );

        let mut weights = RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
        weights.spirit_on_own_base_penalty = 400;
        weights.spirit_action_utility = 0;
        let on_base_score =
            evaluate_preferability_with_weights(&on_base_game, Color::White, &weights);
        let off_base_score =
            evaluate_preferability_with_weights(&off_base_game, Color::White, &weights);
        assert!(off_base_score > on_base_score);
    }

    #[test]
    fn protected_supermana_carrier_gets_less_virtual_credit_when_opponent_is_high() {
        let carrier_location = Location::new(7, 5);
        let guard_location = Location::new(7, 4);
        let enemy_drainer_location = Location::new(0, 5);
        let mut carrier_game = game_with_items(
            vec![
                (
                    carrier_location,
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Supermana,
                    },
                ),
                (
                    guard_location,
                    Item::Mon {
                        mon: Mon::new(MonKind::Angel, Color::White, 0),
                    },
                ),
                (
                    enemy_drainer_location,
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        let mut no_mana_game = game_with_items(
            vec![
                (
                    carrier_location,
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    guard_location,
                    Item::Mon {
                        mon: Mon::new(MonKind::Angel, Color::White, 0),
                    },
                ),
                (
                    enemy_drainer_location,
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );

        let opponent_not_high = (Config::TARGET_SCORE - 3).max(0);
        let opponent_high = (Config::TARGET_SCORE - 1).max(0);

        let mut weights = RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
        weights.include_match_point_window = false;
        weights.immediate_score_window = 0;
        weights.opponent_immediate_score_window = 0;
        weights.immediate_score_multi_window = 0;
        weights.opponent_immediate_score_multi_window = 0;
        weights.mana_carrier_score_this_turn = 0;
        weights.supermana_race_control = 3;

        carrier_game.black_score = opponent_not_high;
        no_mana_game.black_score = opponent_not_high;
        let low_boost = evaluate_preferability_with_weights(&carrier_game, Color::White, &weights)
            - evaluate_preferability_with_weights(&no_mana_game, Color::White, &weights);

        carrier_game.black_score = opponent_high;
        no_mana_game.black_score = opponent_high;
        let high_boost = evaluate_preferability_with_weights(&carrier_game, Color::White, &weights)
            - evaluate_preferability_with_weights(&no_mana_game, Color::White, &weights);

        assert!(
            low_boost > high_boost + weights.confirmed_score,
            "protected supermana carrier should get strong extra credit when opponent score is not high (low_boost={}, high_boost={})",
            low_boost,
            high_boost
        );
    }

    #[test]
    fn protected_opponent_mana_carrier_gets_less_virtual_credit_when_opponent_is_high() {
        let carrier_location = Location::new(7, 5);
        let guard_location = Location::new(7, 4);
        let enemy_drainer_location = Location::new(0, 5);
        let mut carrier_game = game_with_items(
            vec![
                (
                    carrier_location,
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    guard_location,
                    Item::Mon {
                        mon: Mon::new(MonKind::Angel, Color::White, 0),
                    },
                ),
                (
                    enemy_drainer_location,
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        let mut no_mana_game = game_with_items(
            vec![
                (
                    carrier_location,
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    guard_location,
                    Item::Mon {
                        mon: Mon::new(MonKind::Angel, Color::White, 0),
                    },
                ),
                (
                    enemy_drainer_location,
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );

        let opponent_not_high = (Config::TARGET_SCORE - 3).max(0);
        let opponent_high = (Config::TARGET_SCORE - 1).max(0);

        let mut weights = RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS;
        weights.include_match_point_window = false;
        weights.immediate_score_window = 0;
        weights.opponent_immediate_score_window = 0;
        weights.immediate_score_multi_window = 0;
        weights.opponent_immediate_score_multi_window = 0;
        weights.mana_carrier_score_this_turn = 0;
        weights.opponent_mana_denial = 3;

        carrier_game.black_score = opponent_not_high;
        no_mana_game.black_score = opponent_not_high;
        let low_boost = evaluate_preferability_with_weights(&carrier_game, Color::White, &weights)
            - evaluate_preferability_with_weights(&no_mana_game, Color::White, &weights);

        carrier_game.black_score = opponent_high;
        no_mana_game.black_score = opponent_high;
        let high_boost = evaluate_preferability_with_weights(&carrier_game, Color::White, &weights)
            - evaluate_preferability_with_weights(&no_mana_game, Color::White, &weights);

        assert!(
            low_boost > high_boost + weights.confirmed_score,
            "protected opponent-mana carrier should get strong extra credit when opponent score is not high (low_boost={}, high_boost={})",
            low_boost,
            high_boost
        );
    }

    #[test]
    fn eval_breakdown_sum_matches_total() {
        let game = game_with_items(
            vec![
                (
                    Location::new(9, 0),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(1, 0),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                        mana: Mana::Regular(Color::White),
                    },
                ),
                (
                    Location::new(10, 6),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(0, 6),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );

        let breakdown = evaluate_preferability_breakdown_with_weights(
            &game,
            Color::White,
            &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS,
        );
        assert_eq!(breakdown.total, breakdown.terms.sum());
    }

    #[test]
    fn mirrored_board_breakdown_is_symmetric_between_colors() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(9, 0),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(8, 6),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(1, 0),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                        mana: Mana::Regular(Color::White),
                    },
                ),
                (
                    Location::new(2, 4),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.white_score = 2;
        game.black_score = 1;
        game.white_potions_count = 1;
        let mirrored = mirrored_game_with_swapped_colors(&game);

        let original = evaluate_preferability_breakdown_with_weights(
            &game,
            Color::White,
            &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS,
        );
        let mirrored_eval = evaluate_preferability_breakdown_with_weights(
            &mirrored,
            Color::Black,
            &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS,
        );
        assert_eq!(original.total, mirrored_eval.total);
    }
}

fn drainer_immediate_threats(
    board: &Board,
    color: Color,
    location: Location,
    _use_legacy_formula: bool,
) -> (i32, i32) {
    crate::models::automove_exact::drainer_immediate_threats(board, color, location)
}

fn is_drainer_under_walk_threat(
    board: &Board,
    color: Color,
    location: Location,
    angel_nearby: bool,
) -> bool {
    crate::models::automove_exact::is_drainer_under_walk_threat(
        board,
        color,
        location,
        angel_nearby,
    )
}

fn is_drainer_under_danger_threat(
    board: &Board,
    color: Color,
    location: Location,
    angel_nearby: bool,
    use_legacy_formula: bool,
) -> bool {
    if use_legacy_formula {
        return is_drainer_under_immediate_threat(board, color, location, angel_nearby);
    }

    crate::models::automove_exact::can_attack_target_on_board(
        board,
        color.other(),
        color,
        location,
        Config::MONS_MOVES_PER_TURN,
        true,
    )
}

fn is_drainer_under_immediate_threat(
    board: &Board,
    color: Color,
    location: Location,
    angel_nearby: bool,
) -> bool {
    crate::models::automove_exact::is_drainer_under_immediate_threat(
        board,
        color,
        location,
        angel_nearby,
    )
}

fn nearest_enemy_mon_distance(board: &Board, color: Color, location: Location) -> i32 {
    let mut best = Config::BOARD_SIZE as i32;
    for (item_location, item) in board.occupied() {
        if let Some(mon) = item.mon() {
            if mon.color != color && !mon.is_fainted() {
                let delta = item_location.distance(&location) as i32;
                if delta < best {
                    best = delta;
                }
            }
        }
    }
    best.max(1)
}

fn nearest_friendly_drainer_distance(board: &Board, color: Color, location: Location) -> i32 {
    let mut best = Config::BOARD_SIZE as i32;
    for (item_location, item) in board.occupied() {
        if let Some(mon) = item.mon() {
            if mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted() {
                let delta = item_location.distance(&location) as i32;
                if delta < best {
                    best = delta;
                }
            }
        }
    }
    best.max(1)
}

fn distance_to_location(location: Location, destination: Location) -> i32 {
    location.distance(&destination) as i32 + 1
}

fn distance(location: Location, destination: Destination) -> i32 {
    let distance = match destination {
        Destination::Center => {
            // Once within 1 step from center, extra centralization is not rewarded further.
            (Config::BOARD_CENTER_INDEX as i32 - location.i as i32)
                .abs()
                .max(1)
        }
        Destination::AnyClosestPool => {
            let max_index = Config::MAX_LOCATION_INDEX as i32;
            let i = location.i as i32;
            let j = location.j as i32;
            i32::max(
                i32::min(i, (max_index - i).abs()),
                i32::min(j, (max_index - j).abs()),
            )
        }
        Destination::ClosestPool(color) => {
            let pool_row = if color == Color::White {
                Config::MAX_LOCATION_INDEX as i32
            } else {
                0
            };
            let i = location.i as i32;
            let j = location.j as i32;
            i32::max(
                (pool_row - i).abs(),
                i32::min(j, (Config::MAX_LOCATION_INDEX as i32 - j).abs()),
            )
        }
    };
    distance + 1
}
