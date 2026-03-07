#[cfg(any(target_arch = "wasm32", test))]
use crate::models::scoring::{
    evaluate_preferability_with_weights, ScoringWeights, BALANCED_DISTANCE_SCORING_WEIGHTS,
    DEFAULT_SCORING_WEIGHTS, FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS,
    FINISHER_BALANCED_SOFT_SCORING_WEIGHTS, MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS,
    RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
    RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF,
    RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS,
    RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF,
    TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS, TACTICAL_BALANCED_SCORING_WEIGHTS,
};
use crate::*;

#[wasm_bindgen]
#[derive(Debug)]
pub struct MonsGameModel {
    game: MonsGame,
    #[cfg(any(target_arch = "wasm32", test))]
    pro_runtime_context_hint: std::cell::Cell<ProRuntimeContext>,
    #[cfg(target_arch = "wasm32")]
    smart_search_in_progress: std::rc::Rc<std::cell::Cell<bool>>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProRuntimeContext {
    Unknown,
    OpeningBookDriven,
    Independent,
}

impl Clone for MonsGameModel {
    fn clone(&self) -> Self {
        let cloned = Self::with_game(self.game.clone());
        #[cfg(any(target_arch = "wasm32", test))]
        cloned
            .pro_runtime_context_hint
            .set(self.pro_runtime_context_hint.get());
        cloned
    }
}

#[cfg(any(target_arch = "wasm32", test))]
const MIN_SMART_SEARCH_DEPTH: usize = 1;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_SMART_SEARCH_DEPTH: usize = 5;
#[cfg(any(target_arch = "wasm32", test))]
const MIN_SMART_MAX_VISITED_NODES: usize = 32;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_SMART_MAX_VISITED_NODES: usize = 180_000;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TERMINAL_SCORE: i32 = i32::MAX / 8;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_MAX_INPUT_CHAIN: usize = 8;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TRANSPOSITION_TABLE_MAX_ENTRIES: usize = 12_000;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NO_EFFECT_ROOT_PENALTY: i32 = 120;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NO_EFFECT_CHILD_PENALTY: i32 = 0;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_LOW_IMPACT_ROOT_PENALTY: i32 = 40;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_LOW_IMPACT_CHILD_PENALTY: i32 = 0;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_EFFICIENCY_SCORE_MARGIN: i32 = 2_500;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_BACKTRACK_PENALTY: i32 = 140;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_ASPIRATION_WINDOW: i32 = 1_600;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TT_BEST_CHILD_BONUS: i32 = 2_400;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_KILLER_MOVE_BONUS: i32 = 1_200;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TWO_PASS_ROOT_SCOUT_DEPTH: usize = 2;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TWO_PASS_ROOT_SCOUT_MIN_NODES: usize = 96;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TWO_PASS_ROOT_FOCUS_SCORE_MARGIN: i32 = 2_000;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TWO_PASS_ROOT_VOLATILITY_KEEP: usize = 2;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TWO_PASS_ROOT_VOLATILITY_MARGIN: i32 = 600;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_MANA_HANDOFF_PENALTY: i32 = 220;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_DRAINER_SAFETY_SCORE_MARGIN: i32 = 2_200;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NORMAL_ROOT_SAFETY_SHORTLIST_MAX: usize = 4;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NORMAL_ROOT_SAFETY_SCORE_MARGIN: i32 = 3_000;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NORMAL_ROOT_SAFETY_REPLY_LIMIT_MIN: usize = 12;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NORMAL_ROOT_SAFETY_REPLY_LIMIT_MAX: usize = 36;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NORMAL_ROOT_SAFETY_SCORE_RACE_TRIGGER: i32 = 3;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_SCORE_RACE_TRIGGER: i32 = 3;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_SCORE_MARGIN: i32 = 2_400;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_MAX_CANDIDATES: usize = 3;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_REPLY_LIMIT_MIN: usize = 8;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_REPLY_LIMIT_MAX: usize = 16;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_REPLY_RISK_SCORE_MARGIN: i32 = 140;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_REPLY_RISK_SHORTLIST_FAST: usize = 3;
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const SMART_ROOT_REPLY_RISK_SHORTLIST_NORMAL: usize = 5;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_REPLY_RISK_REPLY_LIMIT_FAST: usize = 8;
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const SMART_ROOT_REPLY_RISK_REPLY_LIMIT_NORMAL: usize = 12;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_FAST: i32 = 600;
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_NORMAL: i32 = 1_000;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_REPLY_RISK_WINNER_SPREAD_SKIP: i32 = SMART_TWO_PASS_ROOT_NARROW_SPREAD_FALLBACK;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TWO_PASS_ROOT_NARROW_SPREAD_FALLBACK: i32 = 700;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_MOVE_CLASS_ROOT_SCORE_MARGIN: i32 = 120;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_MOVE_CLASS_CHILD_SCORE_MARGIN: i32 = 110;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_ANTI_HELP_SCORE_MARGIN: i32 = 180;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_ANTI_HELP_REPLY_LIMIT_FAST: usize = 6;
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const SMART_ROOT_ANTI_HELP_REPLY_LIMIT_NORMAL: usize = 8;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_SELECTIVE_EXTENSION_NODE_SHARE_BP_NORMAL: i32 = 1_200;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_SPIRIT_DEVELOPMENT_SCORE_MARGIN: i32 = 700;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_INTERVIEW_SOFT_PRIORITY_SCORE_MARGIN: i32 = 120;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_INTERVIEW_SOFT_SUPERMANA_PROGRESS_BONUS: i32 = 240;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_INTERVIEW_SOFT_SUPERMANA_SCORE_BONUS: i32 = 420;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_INTERVIEW_SOFT_OPPONENT_MANA_PROGRESS_BONUS: i32 = 210;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_INTERVIEW_SOFT_OPPONENT_MANA_SCORE_BONUS: i32 = 360;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_INTERVIEW_SOFT_MANA_HANDOFF_PENALTY: i32 = 220;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_INTERVIEW_SOFT_ROUNDTRIP_PENALTY: i32 = 140;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_POTION_SPEND_NO_COMPENSATION_PENALTY_FAST: i32 = 340;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_POTION_SPEND_NO_COMPENSATION_PENALTY_NORMAL: i32 = 260;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_POTION_HOLD_SCORE_MARGIN: i32 = 180;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_SPIRIT_DEPLOY_EFFICIENCY_BONUS: i32 = 90;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_SPIRIT_ACTION_TARGET_DELTA_WEIGHT: i32 = 22;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_FORCED_DRAINER_ATTACK_FALLBACK_FAST_CANDIDATES: usize = 4;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_FORCED_DRAINER_ATTACK_FALLBACK_NORMAL_CANDIDATES: usize = 6;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_FORCED_DRAINER_ATTACK_FALLBACK_NODE_BUDGET_FAST: usize = 600;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_FORCED_DRAINER_ATTACK_FALLBACK_NODE_BUDGET_NORMAL: usize = 1_800;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_FORCED_DRAINER_ATTACK_FALLBACK_ENUM_LIMIT_FAST: usize = 220;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_FORCED_DRAINER_ATTACK_FALLBACK_ENUM_LIMIT_NORMAL: usize = 280;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_AUTOMOVE_FAST_DEPTH: i32 = 2;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_AUTOMOVE_FAST_MAX_VISITED_NODES: i32 = 480;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_AUTOMOVE_NORMAL_DEPTH: i32 = 3;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_AUTOMOVE_NORMAL_MAX_VISITED_NODES: i32 = 3800;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_AUTOMOVE_PRO_DEPTH: i32 = 4;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_AUTOMOVE_PRO_MAX_VISITED_NODES: i32 =
    SMART_AUTOMOVE_NORMAL_MAX_VISITED_NODES * 369 / 100;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_AUTOMOVE_ULTRA_DEPTH: i32 = 5;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_AUTOMOVE_ULTRA_MAX_VISITED_NODES: i32 = SMART_AUTOMOVE_PRO_MAX_VISITED_NODES * 10;

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
struct IdentityU64Hasher(u64);

#[cfg(any(target_arch = "wasm32", test))]
impl std::hash::Hasher for IdentityU64Hasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = 0xcbf29ce484222325u64;
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }
}

#[cfg(any(target_arch = "wasm32", test))]
type U64BuildHasher = std::hash::BuildHasherDefault<IdentityU64Hasher>;

#[cfg(any(target_arch = "wasm32", test))]
type U64HashMap<V> = std::collections::HashMap<u64, V, U64BuildHasher>;

#[cfg(any(target_arch = "wasm32", test))]
type U64HashSet = std::collections::HashSet<u64, U64BuildHasher>;
const WHITE_OPENING_BOOK: [[&str; 5]; 9] = [
    [
        "l10,3;l9,2",
        "l9,2;l8,1",
        "l8,1;l7,0",
        "l7,0;l6,0",
        "l6,0;l5,0;mp",
    ],
    [
        "l10,7;l9,8",
        "l9,8;l8,9",
        "l8,9;l7,10",
        "l7,10;l6,10",
        "l6,10;l5,10;mp",
    ],
    [
        "l10,4;l9,4",
        "l9,4;l8,4",
        "l8,4;l7,3",
        "l7,3;l6,4",
        "l6,4;l5,4",
    ],
    [
        "l10,5;l9,5",
        "l9,5;l8,5",
        "l10,6;l9,6",
        "l9,6;l8,6",
        "l8,6;l7,5",
    ],
    [
        "l10,5;l9,5",
        "l9,5;l8,5",
        "l10,6;l9,6",
        "l9,6;l8,6",
        "l10,4;l9,5",
    ],
    [
        "l10,5;l9,5",
        "l9,5;l8,5",
        "l8,5;l7,5",
        "l10,4;l9,5",
        "l9,5;l8,5",
    ],
    [
        "l10,6;l9,7",
        "l9,7;l8,6",
        "l8,6;l7,5",
        "l10,4;l9,4",
        "l9,4;l8,5",
    ],
    [
        "l10,5;l9,5",
        "l9,5;l8,5",
        "l10,3;l9,2",
        "l10,6;l9,6",
        "l9,6;l8,7",
    ],
    [
        "l10,3;l9,3",
        "l10,4;l9,4",
        "l10,5;l9,5",
        "l10,6;l9,6",
        "l10,7;l9,7",
    ],
];
static PARSED_WHITE_OPENING_BOOK: std::sync::LazyLock<Vec<Vec<Vec<Input>>>> =
    std::sync::LazyLock::new(|| {
        WHITE_OPENING_BOOK
            .iter()
            .map(|sequence| {
                sequence
                    .iter()
                    .map(|step| Input::array_from_fen(step))
                    .collect()
            })
            .collect()
    });

#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        use_legacy_formula: false,
        confirmed_score: 900,
        spirit_on_own_base_penalty: 260,
        score_race_path_progress: 86,
        opponent_score_race_path_progress: 184,
        immediate_score_window: 96,
        opponent_immediate_score_window: 245,
        angel_guarding_drainer: 120,
        supermana_race_control: 30,
        opponent_mana_denial: 24,
        drainer_holding_mana: 470,
        drainer_immediate_threat: -55,
        drainer_best_mana_path: 58,
        drainer_pickup_score_this_turn: 90,
        mana_carrier_score_this_turn: 150,
        drainer_close_to_mana: 360,
        spirit_action_utility: 86,
        ..BALANCED_DISTANCE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        use_legacy_formula: false,
        confirmed_score: 900,
        spirit_on_own_base_penalty: 260,
        score_race_path_progress: 94,
        opponent_score_race_path_progress: 220,
        immediate_score_window: 102,
        opponent_immediate_score_window: 310,
        angel_guarding_drainer: 180,
        supermana_race_control: 34,
        opponent_mana_denial: 30,
        drainer_holding_mana: 500,
        drainer_immediate_threat: -90,
        drainer_best_mana_path: 84,
        drainer_pickup_score_this_turn: 110,
        mana_carrier_score_this_turn: 180,
        drainer_close_to_mana: 390,
        spirit_action_utility: 90,
        ..TACTICAL_BALANCED_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        use_legacy_formula: false,
        confirmed_score: 890,
        spirit_on_own_base_penalty: 260,
        score_race_path_progress: 104,
        opponent_score_race_path_progress: 255,
        immediate_score_window: 114,
        opponent_immediate_score_window: 360,
        angel_guarding_drainer: 190,
        supermana_race_control: 40,
        opponent_mana_denial: 34,
        drainer_holding_mana: 520,
        drainer_immediate_threat: -120,
        drainer_best_mana_path: 96,
        drainer_pickup_score_this_turn: 130,
        mana_carrier_score_this_turn: 220,
        drainer_close_to_mana: 410,
        spirit_action_utility: 94,
        ..TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        use_legacy_formula: false,
        confirmed_score: 930,
        spirit_on_own_base_penalty: 260,
        score_race_path_progress: 170,
        opponent_score_race_path_progress: 170,
        immediate_score_window: 275,
        opponent_immediate_score_window: 235,
        supermana_race_control: 32,
        opponent_mana_denial: 28,
        drainer_holding_mana: 500,
        drainer_best_mana_path: 72,
        drainer_pickup_score_this_turn: 120,
        mana_carrier_score_this_turn: 240,
        drainer_close_to_mana: 375,
        angel_guarding_drainer: 170,
        spirit_action_utility: 88,
        ..FINISHER_BALANCED_SOFT_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        use_legacy_formula: false,
        confirmed_score: 940,
        spirit_on_own_base_penalty: 260,
        score_race_path_progress: 195,
        opponent_score_race_path_progress: 185,
        immediate_score_window: 330,
        opponent_immediate_score_window: 265,
        supermana_race_control: 36,
        opponent_mana_denial: 30,
        drainer_holding_mana: 520,
        drainer_best_mana_path: 84,
        drainer_pickup_score_this_turn: 140,
        mana_carrier_score_this_turn: 280,
        drainer_close_to_mana: 395,
        angel_guarding_drainer: 180,
        spirit_action_utility: 90,
        ..FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    };

#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        drainer_danger_boolean: -1200,
        mana_carrier_danger_boolean: -800,
        ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        drainer_danger_boolean: -1200,
        mana_carrier_danger_boolean: -800,
        ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_danger_boolean: -1200,
    mana_carrier_danger_boolean: -800,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_danger_boolean: -1200,
    mana_carrier_danger_boolean: -800,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_danger_boolean: -1200,
    mana_carrier_danger_boolean: -800,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_STRONG_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        drainer_danger_boolean: -1800,
        mana_carrier_danger_boolean: -1200,
        ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_STRONG_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        drainer_danger_boolean: -1800,
        mana_carrier_danger_boolean: -1200,
        ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_STRONG_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_danger_boolean: -1800,
    mana_carrier_danger_boolean: -1200,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_STRONG_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_danger_boolean: -1800,
    mana_carrier_danger_boolean: -1200,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_STRONG_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_danger_boolean: -1800,
    mana_carrier_danger_boolean: -1200,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_ATTACK_BONUS_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        opponent_drainer_attack_bonus: 400,
        ..RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_ATTACK_BONUS_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        opponent_drainer_attack_bonus: 400,
        ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_ATTACK_BONUS_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    opponent_drainer_attack_bonus: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_ATTACK_BONUS_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    opponent_drainer_attack_bonus: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_ATTACK_BONUS_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    opponent_drainer_attack_bonus: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_STRONG_ATTACK_BONUS_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    opponent_drainer_attack_bonus: 800,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_STRONG_ATTACK_BONUS_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    opponent_drainer_attack_bonus: 800,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_STRONG_ATTACK_BONUS_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    opponent_drainer_attack_bonus: 800,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_STRONG_ATTACK_BONUS_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    opponent_drainer_attack_bonus: 800,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_STRONG_ATTACK_BONUS_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    opponent_drainer_attack_bonus: 800,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        drainer_walk_threat_boolean: -600,
        mana_carrier_walk_threat_boolean: -400,
        ..RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        drainer_walk_threat_boolean: -600,
        mana_carrier_walk_threat_boolean: -400,
        ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -600,
    mana_carrier_walk_threat_boolean: -400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -600,
    mana_carrier_walk_threat_boolean: -400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -600,
    mana_carrier_walk_threat_boolean: -400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_LIGHT_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -200,
    mana_carrier_walk_threat_boolean: -100,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_LIGHT_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -200,
    mana_carrier_walk_threat_boolean: -100,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_LIGHT_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -200,
    mana_carrier_walk_threat_boolean: -100,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_LIGHT_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -200,
    mana_carrier_walk_threat_boolean: -100,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_LIGHT_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -200,
    mana_carrier_walk_threat_boolean: -100,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_MEDIUM_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -300,
    mana_carrier_walk_threat_boolean: -150,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_MEDIUM_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -300,
    mana_carrier_walk_threat_boolean: -150,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_MEDIUM_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -300,
    mana_carrier_walk_threat_boolean: -150,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_MEDIUM_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -300,
    mana_carrier_walk_threat_boolean: -150,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_MEDIUM_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -300,
    mana_carrier_walk_threat_boolean: -150,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_MODERATE_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -400,
    mana_carrier_walk_threat_boolean: -200,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_MODERATE_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -400,
    mana_carrier_walk_threat_boolean: -200,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_MODERATE_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -400,
    mana_carrier_walk_threat_boolean: -200,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_MODERATE_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -400,
    mana_carrier_walk_threat_boolean: -200,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_WALK_THREAT_MODERATE_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_walk_threat_boolean: -400,
    mana_carrier_walk_threat_boolean: -200,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_DRAINER_SHIELD_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        drainer_immediate_threat: -200,
        drainer_at_risk: -520,
        angel_guarding_drainer: 190,
        mana_carrier_at_risk: -320,
        drainer_danger_boolean: -1200,
        mana_carrier_danger_boolean: -800,
        drainer_walk_threat_boolean: -600,
        mana_carrier_walk_threat_boolean: -400,
        ..RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_DRAINER_SHIELD_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        drainer_immediate_threat: -260,
        drainer_at_risk: -560,
        angel_guarding_drainer: 240,
        mana_carrier_at_risk: -340,
        drainer_danger_boolean: -1200,
        mana_carrier_danger_boolean: -800,
        drainer_walk_threat_boolean: -600,
        mana_carrier_walk_threat_boolean: -400,
        ..RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_DRAINER_SHIELD_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_immediate_threat: -320,
    drainer_at_risk: -600,
    angel_guarding_drainer: 260,
    mana_carrier_at_risk: -360,
    drainer_danger_boolean: -1200,
    mana_carrier_danger_boolean: -800,
    drainer_walk_threat_boolean: -600,
    mana_carrier_walk_threat_boolean: -400,
    ..RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_DRAINER_SHIELD_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_immediate_threat: -80,
    drainer_at_risk: -480,
    angel_guarding_drainer: 210,
    mana_carrier_at_risk: -310,
    drainer_danger_boolean: -1200,
    mana_carrier_danger_boolean: -800,
    drainer_walk_threat_boolean: -600,
    mana_carrier_walk_threat_boolean: -400,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
#[allow(dead_code)]
const RUNTIME_NORMAL_DRAINER_SHIELD_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    drainer_immediate_threat: -100,
    drainer_at_risk: -500,
    angel_guarding_drainer: 220,
    mana_carrier_at_risk: -330,
    drainer_danger_boolean: -1200,
    mana_carrier_danger_boolean: -800,
    drainer_walk_threat_boolean: -600,
    mana_carrier_walk_threat_boolean: -400,
    ..RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_ATTACKER_PROXIMITY_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 200,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_ATTACKER_PROXIMITY_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 200,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_ATTACKER_PROXIMITY_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 200,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_ATTACKER_PROXIMITY_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 200,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_ATTACKER_PROXIMITY_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 200,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_STRONG_ATTACKER_PROXIMITY_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_STRONG_ATTACKER_PROXIMITY_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_STRONG_ATTACKER_PROXIMITY_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_STRONG_ATTACKER_PROXIMITY_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_STRONG_ATTACKER_PROXIMITY_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_COMBO_PROXIMITY_ATTACK_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 200,
    opponent_drainer_attack_bonus: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_COMBO_PROXIMITY_ATTACK_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 200,
    opponent_drainer_attack_bonus: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_COMBO_PROXIMITY_ATTACK_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 200,
    opponent_drainer_attack_bonus: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_COMBO_PROXIMITY_ATTACK_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 200,
    opponent_drainer_attack_bonus: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
};
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_COMBO_PROXIMITY_ATTACK_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS:
    ScoringWeights = ScoringWeights {
    attacker_close_to_opponent_drainer: 200,
    opponent_drainer_attack_bonus: 400,
    ..RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
};

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SmartAutomovePreference {
    Fast,
    Normal,
    Pro,
    Ultra,
}

#[cfg(any(target_arch = "wasm32", test))]
impl SmartAutomovePreference {
    fn from_api_value(value: &str) -> Option<Self> {
        let normalized = value.trim();
        if normalized.eq_ignore_ascii_case("fast") {
            Some(Self::Fast)
        } else if normalized.eq_ignore_ascii_case("normal") {
            Some(Self::Normal)
        } else if normalized.eq_ignore_ascii_case("pro") {
            Some(Self::Pro)
        } else if normalized.eq_ignore_ascii_case("ultra") {
            Some(Self::Ultra)
        } else {
            None
        }
    }

    fn as_api_value(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Normal => "normal",
            Self::Pro => "pro",
            Self::Ultra => "ultra",
        }
    }

    fn depth_and_max_nodes(self) -> (i32, i32) {
        match self {
            Self::Fast => (
                SMART_AUTOMOVE_FAST_DEPTH,
                SMART_AUTOMOVE_FAST_MAX_VISITED_NODES,
            ),
            Self::Normal => (
                SMART_AUTOMOVE_NORMAL_DEPTH,
                SMART_AUTOMOVE_NORMAL_MAX_VISITED_NODES,
            ),
            Self::Pro => (
                SMART_AUTOMOVE_PRO_DEPTH,
                SMART_AUTOMOVE_PRO_MAX_VISITED_NODES,
            ),
            Self::Ultra => (
                SMART_AUTOMOVE_ULTRA_DEPTH,
                SMART_AUTOMOVE_ULTRA_MAX_VISITED_NODES,
            ),
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy)]
struct SmartSearchConfig {
    depth: usize,
    max_visited_nodes: usize,
    root_enum_limit: usize,
    root_branch_limit: usize,
    node_enum_limit: usize,
    node_branch_limit: usize,
    scoring_weights: &'static ScoringWeights,
    enable_root_efficiency: bool,
    enable_event_ordering_bonus: bool,
    enable_backtrack_penalty: bool,
    enable_tt_best_child_ordering: bool,
    enable_root_aspiration: bool,
    enable_two_pass_root_allocation: bool,
    root_focus_k: usize,
    root_focus_budget_share_bp: i32,
    enable_selective_extensions: bool,
    enable_quiet_reductions: bool,
    max_extensions_per_path: usize,
    selective_extension_node_share_bp: i32,
    enable_root_mana_handoff_guard: bool,
    enable_forced_drainer_attack: bool,
    enable_forced_drainer_attack_fallback: bool,
    enable_targeted_drainer_attack_fallback: bool,
    enable_per_mon_drainer_attack_fallback: bool,
    enable_drainer_attack_priority_enum: bool,
    drainer_attack_priority_enum_boost: usize,
    enable_drainer_attack_minimax_selection: bool,
    enable_drainer_attack_full_pool: bool,
    enable_conditional_forced_drainer_attack: bool,
    conditional_forced_attack_score_margin: i32,
    enable_forced_tactical_prepass: bool,
    enable_root_drainer_safety_prefilter: bool,
    enable_root_spirit_development_pref: bool,
    enable_root_reply_risk_guard: bool,
    root_reply_risk_score_margin: i32,
    root_reply_risk_shortlist_max: usize,
    root_reply_risk_reply_limit: usize,
    root_reply_risk_node_share_bp: i32,
    enable_move_class_coverage: bool,
    enable_child_move_class_coverage: bool,
    enable_strict_tactical_class_coverage: bool,
    enable_strict_anti_help_filter: bool,
    root_anti_help_score_margin: i32,
    root_anti_help_reply_limit: usize,
    enable_two_pass_volatility_focus: bool,
    enable_normal_root_safety_rerank: bool,
    enable_normal_root_safety_deep_floor: bool,
    enable_interview_hard_spirit_deploy: bool,
    enable_interview_soft_root_priors: bool,
    enable_interview_deterministic_tiebreak: bool,
    enable_mana_start_mix_with_potion_actions: bool,
    enable_potion_progress_compensation: bool,
    prefer_clean_reply_risk_roots: bool,
    root_drainer_safety_score_margin: i32,
    root_mana_handoff_penalty: i32,
    root_backtrack_penalty: i32,
    root_efficiency_score_margin: i32,
    potion_spend_penalty_fast: i32,
    potion_spend_penalty_normal: i32,
    interview_soft_score_margin: i32,
    interview_soft_supermana_progress_bonus: i32,
    interview_soft_supermana_score_bonus: i32,
    interview_soft_opponent_mana_progress_bonus: i32,
    interview_soft_opponent_mana_score_bonus: i32,
    interview_soft_mana_handoff_penalty: i32,
    interview_soft_roundtrip_penalty: i32,
    enable_enhanced_drainer_vulnerability: bool,
    enable_supermana_prepass_exception: bool,
    enable_opponent_mana_prepass_exception: bool,
    enable_walk_threat_prefilter: bool,
    root_walk_threat_score_margin: i32,
    enable_killer_move_ordering: bool,
    enable_tt_depth_preferred_replacement: bool,
    enable_pvs: bool,
    quiet_reduction_depth_threshold: usize,
    #[allow(dead_code)]
    enable_iterative_deepening: bool,
    #[allow(dead_code)]
    iterative_deepening_depth_offset: usize,
    #[allow(dead_code)]
    iterative_deepening_alpha_margin: i32,
    enable_futility_pruning: bool,
    futility_margin: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
impl SmartSearchConfig {
    fn from_preference(preference: SmartAutomovePreference) -> Self {
        let (depth, max_visited_nodes) = preference.depth_and_max_nodes();
        let config = Self::from_budget(depth, max_visited_nodes).for_runtime();
        match preference {
            SmartAutomovePreference::Fast => {
                let mut tuned = Self::with_fast_wideroot_shape(config);
                tuned.enable_root_efficiency = true;
                tuned.enable_event_ordering_bonus = true;
                tuned.enable_backtrack_penalty = true;
                tuned.enable_tt_best_child_ordering = true;
                tuned.enable_root_aspiration = false;
                tuned.enable_two_pass_root_allocation = false;
                tuned.root_focus_k = 2;
                tuned.root_focus_budget_share_bp = 6_000;
                tuned.enable_selective_extensions = false;
                tuned.enable_quiet_reductions = true;
                tuned.max_extensions_per_path = 1;
                tuned.selective_extension_node_share_bp = 0;
                tuned.enable_root_mana_handoff_guard = true;
                tuned.enable_forced_drainer_attack = true;
                tuned.enable_forced_drainer_attack_fallback = true;
                tuned.enable_forced_tactical_prepass = true;
                tuned.enable_root_drainer_safety_prefilter = true;
                tuned.enable_root_spirit_development_pref = true;
                tuned.enable_root_reply_risk_guard = true;
                tuned.root_reply_risk_score_margin = 125;
                tuned.root_reply_risk_shortlist_max = 4;
                tuned.root_reply_risk_reply_limit = 10;
                tuned.root_reply_risk_node_share_bp = 650;
                tuned.enable_move_class_coverage = true;
                tuned.enable_child_move_class_coverage = true;
                tuned.enable_strict_tactical_class_coverage = true;
                tuned.enable_strict_anti_help_filter = true;
                tuned.root_anti_help_score_margin = 220;
                tuned.root_anti_help_reply_limit = SMART_ROOT_ANTI_HELP_REPLY_LIMIT_FAST;
                tuned.enable_two_pass_volatility_focus = false;
                tuned.enable_normal_root_safety_rerank = false;
                tuned.enable_normal_root_safety_deep_floor = false;
                tuned.enable_interview_hard_spirit_deploy = false;
                tuned.enable_interview_soft_root_priors = true;
                tuned.enable_interview_deterministic_tiebreak = false;
                tuned.enable_mana_start_mix_with_potion_actions = true;
                tuned.enable_potion_progress_compensation = true;
                tuned.prefer_clean_reply_risk_roots = false;
                tuned.root_drainer_safety_score_margin = SMART_ROOT_DRAINER_SAFETY_SCORE_MARGIN;
                tuned.root_mana_handoff_penalty = 300;
                tuned.root_backtrack_penalty = 220;
                tuned.root_efficiency_score_margin = 1_700;
                tuned.potion_spend_penalty_fast = 220;
                tuned.potion_spend_penalty_normal =
                    SMART_POTION_SPEND_NO_COMPENSATION_PENALTY_NORMAL;
                tuned.interview_soft_score_margin = 80;
                tuned.interview_soft_supermana_progress_bonus = 320;
                tuned.interview_soft_supermana_score_bonus = 600;
                tuned.interview_soft_opponent_mana_progress_bonus = 200;
                tuned.interview_soft_opponent_mana_score_bonus = 310;
                tuned.interview_soft_mana_handoff_penalty = 280;
                tuned.interview_soft_roundtrip_penalty = 220;
                tuned.enable_enhanced_drainer_vulnerability = true;
                tuned.enable_supermana_prepass_exception = true;
                tuned.scoring_weights = &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF;
                tuned
            }
            SmartAutomovePreference::Normal => {
                let mut tuned = Self::with_normal_deeper_shape(config);
                tuned.max_visited_nodes = (tuned.max_visited_nodes * 3 / 2)
                    .clamp(tuned.max_visited_nodes, MAX_SMART_MAX_VISITED_NODES);
                tuned.max_visited_nodes = (tuned.max_visited_nodes * 112 / 100)
                    .clamp(tuned.max_visited_nodes, MAX_SMART_MAX_VISITED_NODES);
                tuned.root_branch_limit = tuned.root_branch_limit.saturating_sub(1).clamp(12, 38);
                tuned.node_branch_limit = (tuned.node_branch_limit + 2).clamp(8, 18);
                tuned.root_enum_limit =
                    (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 240);
                tuned.node_enum_limit =
                    ((tuned.node_branch_limit + 2) * 6).clamp(tuned.node_branch_limit, 156);
                tuned.enable_root_efficiency = true;
                tuned.enable_event_ordering_bonus = false;
                tuned.enable_backtrack_penalty = true;
                tuned.enable_tt_best_child_ordering = true;
                tuned.enable_root_aspiration = false;
                tuned.enable_two_pass_root_allocation = true;
                tuned.root_focus_k = 3;
                tuned.root_focus_budget_share_bp = 7_000;
                tuned.enable_selective_extensions = true;
                tuned.enable_quiet_reductions = true;
                tuned.max_extensions_per_path = 1;
                tuned.selective_extension_node_share_bp =
                    SMART_SELECTIVE_EXTENSION_NODE_SHARE_BP_NORMAL;
                tuned.enable_root_mana_handoff_guard = true;
                tuned.enable_forced_drainer_attack = true;
                tuned.enable_forced_drainer_attack_fallback = true;
                tuned.enable_forced_tactical_prepass = true;
                tuned.enable_root_drainer_safety_prefilter = true;
                tuned.enable_root_spirit_development_pref = true;
                tuned.enable_root_reply_risk_guard = true;
                tuned.root_reply_risk_score_margin = 145;
                tuned.root_reply_risk_shortlist_max = 7;
                tuned.root_reply_risk_reply_limit = 16;
                tuned.root_reply_risk_node_share_bp = 1_350;
                tuned.enable_move_class_coverage = true;
                tuned.enable_child_move_class_coverage = true;
                tuned.enable_strict_tactical_class_coverage = true;
                tuned.enable_strict_anti_help_filter = true;
                tuned.root_anti_help_score_margin = 300;
                tuned.root_anti_help_reply_limit = 10;
                tuned.enable_two_pass_volatility_focus = true;
                tuned.enable_normal_root_safety_rerank = true;
                tuned.enable_normal_root_safety_deep_floor = true;
                tuned.enable_interview_hard_spirit_deploy = true;
                tuned.enable_interview_soft_root_priors = true;
                tuned.enable_interview_deterministic_tiebreak = false;
                tuned.enable_mana_start_mix_with_potion_actions = true;
                tuned.enable_potion_progress_compensation = true;
                tuned.prefer_clean_reply_risk_roots = true;
                tuned.root_drainer_safety_score_margin = 4_200;
                tuned.enable_enhanced_drainer_vulnerability = true;
                tuned.root_mana_handoff_penalty = 340;
                tuned.root_backtrack_penalty = 240;
                tuned.root_efficiency_score_margin = 1_400;
                tuned.selective_extension_node_share_bp = 1_250;
                tuned.potion_spend_penalty_fast = SMART_POTION_SPEND_NO_COMPENSATION_PENALTY_FAST;
                tuned.potion_spend_penalty_normal = 130;
                tuned.interview_soft_score_margin = 80;
                tuned.interview_soft_supermana_progress_bonus = 240;
                tuned.interview_soft_supermana_score_bonus = 300;
                tuned.interview_soft_opponent_mana_progress_bonus = 220;
                tuned.interview_soft_opponent_mana_score_bonus = 280;
                tuned.interview_soft_mana_handoff_penalty = 340;
                tuned.interview_soft_roundtrip_penalty = 260;
                tuned
            }
            SmartAutomovePreference::Pro => {
                let mut tuned = Self::with_normal_deeper_shape(config);
                tuned.max_visited_nodes = 9_800;
                tuned.root_branch_limit = tuned.root_branch_limit.clamp(14, 34);
                tuned.node_branch_limit = tuned.node_branch_limit.clamp(9, 15);
                tuned.root_enum_limit =
                    (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 204);
                tuned.node_enum_limit =
                    ((tuned.node_branch_limit + 2) * 6).clamp(tuned.node_branch_limit, 132);
                tuned.enable_root_efficiency = true;
                tuned.enable_event_ordering_bonus = false;
                tuned.enable_backtrack_penalty = true;
                tuned.enable_tt_best_child_ordering = true;
                tuned.enable_root_aspiration = false;
                tuned.enable_two_pass_root_allocation = true;
                tuned.root_focus_k = 3;
                tuned.root_focus_budget_share_bp = 7_000;
                tuned.enable_selective_extensions = true;
                tuned.enable_quiet_reductions = true;
                tuned.max_extensions_per_path = 1;
                tuned.selective_extension_node_share_bp = 1_500;
                tuned.enable_root_mana_handoff_guard = true;
                tuned.enable_forced_drainer_attack = true;
                tuned.enable_forced_drainer_attack_fallback = true;
                tuned.enable_forced_tactical_prepass = false;
                tuned.enable_root_drainer_safety_prefilter = true;
                tuned.enable_root_spirit_development_pref = true;
                tuned.enable_root_reply_risk_guard = true;
                tuned.root_reply_risk_score_margin = 165;
                tuned.root_reply_risk_shortlist_max = 9;
                tuned.root_reply_risk_reply_limit = 24;
                tuned.root_reply_risk_node_share_bp = 2_000;
                tuned.enable_move_class_coverage = true;
                tuned.enable_child_move_class_coverage = true;
                tuned.enable_strict_tactical_class_coverage = true;
                tuned.enable_strict_anti_help_filter = true;
                tuned.root_anti_help_score_margin = 300;
                tuned.root_anti_help_reply_limit = 10;
                tuned.enable_two_pass_volatility_focus = true;
                tuned.enable_normal_root_safety_rerank = true;
                tuned.enable_normal_root_safety_deep_floor = true;
                tuned.enable_interview_hard_spirit_deploy = true;
                tuned.enable_interview_soft_root_priors = true;
                tuned.enable_interview_deterministic_tiebreak = false;
                tuned.enable_mana_start_mix_with_potion_actions = true;
                tuned.enable_potion_progress_compensation = true;
                tuned.prefer_clean_reply_risk_roots = true;
                tuned.root_drainer_safety_score_margin = 4_800;
                tuned.enable_enhanced_drainer_vulnerability = true;
                tuned.root_mana_handoff_penalty = 340;
                tuned.root_backtrack_penalty = 240;
                tuned.root_efficiency_score_margin = 1_400;
                tuned.enable_futility_pruning = true;
                tuned.futility_margin = 2_300;
                tuned.quiet_reduction_depth_threshold = 2;
                tuned.potion_spend_penalty_fast = SMART_POTION_SPEND_NO_COMPENSATION_PENALTY_FAST;
                tuned.potion_spend_penalty_normal = 130;
                tuned.interview_soft_score_margin = 80;
                tuned.interview_soft_supermana_progress_bonus = 240;
                tuned.interview_soft_supermana_score_bonus = 300;
                tuned.interview_soft_opponent_mana_progress_bonus = 280;
                tuned.interview_soft_opponent_mana_score_bonus = 340;
                tuned.interview_soft_mana_handoff_penalty = 340;
                tuned.interview_soft_roundtrip_penalty = 260;
                tuned
            }
            SmartAutomovePreference::Ultra => {
                let mut tuned = Self::with_normal_deeper_shape(config);
                tuned.max_visited_nodes = SMART_AUTOMOVE_ULTRA_MAX_VISITED_NODES as usize;
                tuned.root_branch_limit = tuned.root_branch_limit.clamp(16, 48);
                tuned.node_branch_limit = tuned.node_branch_limit.clamp(10, 20);
                tuned.root_enum_limit =
                    (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 288);
                tuned.node_enum_limit =
                    ((tuned.node_branch_limit + 2) * 6).clamp(tuned.node_branch_limit, 168);
                tuned.enable_root_efficiency = true;
                tuned.enable_event_ordering_bonus = false;
                tuned.enable_backtrack_penalty = true;
                tuned.enable_tt_best_child_ordering = true;
                tuned.enable_root_aspiration = false;
                tuned.enable_two_pass_root_allocation = true;
                tuned.root_focus_k = 4;
                tuned.root_focus_budget_share_bp = 7_600;
                tuned.enable_selective_extensions = true;
                tuned.enable_quiet_reductions = true;
                tuned.max_extensions_per_path = 1;
                tuned.selective_extension_node_share_bp = 1_800;
                tuned.enable_root_mana_handoff_guard = true;
                tuned.enable_forced_drainer_attack = true;
                tuned.enable_forced_drainer_attack_fallback = true;
                tuned.enable_forced_tactical_prepass = false;
                tuned.enable_root_drainer_safety_prefilter = true;
                tuned.enable_root_spirit_development_pref = true;
                tuned.enable_root_reply_risk_guard = true;
                tuned.root_reply_risk_score_margin = 175;
                tuned.root_reply_risk_shortlist_max = 10;
                tuned.root_reply_risk_reply_limit = 30;
                tuned.root_reply_risk_node_share_bp = 2_400;
                tuned.enable_move_class_coverage = true;
                tuned.enable_child_move_class_coverage = true;
                tuned.enable_strict_tactical_class_coverage = true;
                tuned.enable_strict_anti_help_filter = true;
                tuned.root_anti_help_score_margin = 300;
                tuned.root_anti_help_reply_limit = 10;
                tuned.enable_two_pass_volatility_focus = true;
                tuned.enable_normal_root_safety_rerank = true;
                tuned.enable_normal_root_safety_deep_floor = true;
                tuned.enable_interview_hard_spirit_deploy = true;
                tuned.enable_interview_soft_root_priors = true;
                tuned.enable_interview_deterministic_tiebreak = false;
                tuned.enable_mana_start_mix_with_potion_actions = true;
                tuned.enable_potion_progress_compensation = true;
                tuned.prefer_clean_reply_risk_roots = true;
                tuned.root_drainer_safety_score_margin = 5_000;
                tuned.enable_enhanced_drainer_vulnerability = true;
                tuned.root_mana_handoff_penalty = 340;
                tuned.root_backtrack_penalty = 240;
                tuned.root_efficiency_score_margin = 1_400;
                tuned.enable_futility_pruning = true;
                tuned.futility_margin = 2_400;
                tuned.quiet_reduction_depth_threshold = 2;
                tuned.potion_spend_penalty_fast = SMART_POTION_SPEND_NO_COMPENSATION_PENALTY_FAST;
                tuned.potion_spend_penalty_normal = 130;
                tuned.interview_soft_score_margin = 80;
                tuned.interview_soft_supermana_progress_bonus = 240;
                tuned.interview_soft_supermana_score_bonus = 300;
                tuned.interview_soft_opponent_mana_progress_bonus = 320;
                tuned.interview_soft_opponent_mana_score_bonus = 380;
                tuned.interview_soft_mana_handoff_penalty = 340;
                tuned.interview_soft_roundtrip_penalty = 260;
                tuned
            }
        }
    }

    fn from_budget(depth: i32, max_visited_nodes: i32) -> Self {
        let depth =
            depth.clamp(MIN_SMART_SEARCH_DEPTH as i32, MAX_SMART_SEARCH_DEPTH as i32) as usize;
        let max_visited_nodes = max_visited_nodes.clamp(
            MIN_SMART_MAX_VISITED_NODES as i32,
            MAX_SMART_MAX_VISITED_NODES as i32,
        ) as usize;

        let root_branch_limit = (max_visited_nodes / 24).clamp(4, 28);
        let node_branch_limit = (max_visited_nodes / 40).clamp(4, 18);
        let root_enum_limit = (root_branch_limit * 5).clamp(root_branch_limit, 180);
        let node_enum_limit = (node_branch_limit * 3).clamp(node_branch_limit, 96);

        Self {
            depth,
            max_visited_nodes,
            root_enum_limit,
            root_branch_limit,
            node_enum_limit,
            node_branch_limit,
            scoring_weights: &DEFAULT_SCORING_WEIGHTS,
            enable_root_efficiency: false,
            enable_event_ordering_bonus: false,
            enable_backtrack_penalty: false,
            enable_tt_best_child_ordering: true,
            enable_root_aspiration: true,
            enable_two_pass_root_allocation: false,
            root_focus_k: 2,
            root_focus_budget_share_bp: 7_000,
            enable_selective_extensions: false,
            enable_quiet_reductions: false,
            max_extensions_per_path: 0,
            selective_extension_node_share_bp: 0,
            enable_root_mana_handoff_guard: false,
            enable_forced_drainer_attack: true,
            enable_forced_drainer_attack_fallback: true,
            enable_targeted_drainer_attack_fallback: false,
            enable_per_mon_drainer_attack_fallback: false,
            enable_drainer_attack_priority_enum: false,
            drainer_attack_priority_enum_boost: 0,
            enable_drainer_attack_minimax_selection: false,
            enable_drainer_attack_full_pool: false,
            enable_conditional_forced_drainer_attack: false,
            conditional_forced_attack_score_margin: 1,
            enable_forced_tactical_prepass: true,
            enable_root_drainer_safety_prefilter: true,
            enable_root_spirit_development_pref: true,
            enable_root_reply_risk_guard: false,
            root_reply_risk_score_margin: SMART_ROOT_REPLY_RISK_SCORE_MARGIN,
            root_reply_risk_shortlist_max: SMART_ROOT_REPLY_RISK_SHORTLIST_FAST,
            root_reply_risk_reply_limit: SMART_ROOT_REPLY_RISK_REPLY_LIMIT_FAST,
            root_reply_risk_node_share_bp: SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_FAST,
            enable_move_class_coverage: false,
            enable_child_move_class_coverage: false,
            enable_strict_tactical_class_coverage: false,
            enable_strict_anti_help_filter: false,
            root_anti_help_score_margin: SMART_ROOT_ANTI_HELP_SCORE_MARGIN,
            root_anti_help_reply_limit: SMART_ROOT_ANTI_HELP_REPLY_LIMIT_FAST,
            enable_two_pass_volatility_focus: false,
            enable_normal_root_safety_rerank: false,
            enable_normal_root_safety_deep_floor: false,
            enable_interview_hard_spirit_deploy: false,
            enable_interview_soft_root_priors: false,
            enable_interview_deterministic_tiebreak: false,
            enable_mana_start_mix_with_potion_actions: false,
            enable_potion_progress_compensation: false,
            prefer_clean_reply_risk_roots: false,
            root_drainer_safety_score_margin: SMART_ROOT_DRAINER_SAFETY_SCORE_MARGIN,
            root_mana_handoff_penalty: SMART_ROOT_MANA_HANDOFF_PENALTY,
            root_backtrack_penalty: SMART_ROOT_BACKTRACK_PENALTY,
            root_efficiency_score_margin: SMART_ROOT_EFFICIENCY_SCORE_MARGIN,
            potion_spend_penalty_fast: SMART_POTION_SPEND_NO_COMPENSATION_PENALTY_FAST,
            potion_spend_penalty_normal: SMART_POTION_SPEND_NO_COMPENSATION_PENALTY_NORMAL,
            interview_soft_score_margin: SMART_INTERVIEW_SOFT_PRIORITY_SCORE_MARGIN,
            interview_soft_supermana_progress_bonus: SMART_INTERVIEW_SOFT_SUPERMANA_PROGRESS_BONUS,
            interview_soft_supermana_score_bonus: SMART_INTERVIEW_SOFT_SUPERMANA_SCORE_BONUS,
            interview_soft_opponent_mana_progress_bonus:
                SMART_INTERVIEW_SOFT_OPPONENT_MANA_PROGRESS_BONUS,
            interview_soft_opponent_mana_score_bonus:
                SMART_INTERVIEW_SOFT_OPPONENT_MANA_SCORE_BONUS,
            interview_soft_mana_handoff_penalty: SMART_INTERVIEW_SOFT_MANA_HANDOFF_PENALTY,
            interview_soft_roundtrip_penalty: SMART_INTERVIEW_SOFT_ROUNDTRIP_PENALTY,
            enable_enhanced_drainer_vulnerability: false,
            enable_supermana_prepass_exception: false,
            enable_opponent_mana_prepass_exception: false,
            enable_walk_threat_prefilter: false,
            root_walk_threat_score_margin: 2000,
            enable_killer_move_ordering: false,
            enable_tt_depth_preferred_replacement: false,
            enable_pvs: false,
            quiet_reduction_depth_threshold: 3,
            enable_iterative_deepening: false,
            iterative_deepening_depth_offset: 2,
            iterative_deepening_alpha_margin: 0,
            enable_futility_pruning: false,
            futility_margin: 3000,
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn for_runtime(self) -> Self {
        let mut tuned = self;

        if tuned.depth >= 3 {
            tuned.root_branch_limit = (tuned.root_branch_limit + 10).clamp(6, 36);
            tuned.node_branch_limit = tuned.node_branch_limit.saturating_sub(11).clamp(6, 18);
            tuned.root_enum_limit =
                (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 220);
            tuned.node_enum_limit =
                (tuned.node_branch_limit * 4).clamp(tuned.node_branch_limit, 108);
            tuned.scoring_weights = &BALANCED_DISTANCE_SCORING_WEIGHTS;
        } else {
            tuned.scoring_weights = &MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS;
        }

        tuned
    }

    fn with_fast_wideroot_shape(self) -> Self {
        let mut tuned = self;
        tuned.root_branch_limit = (self.root_branch_limit + 8).clamp(8, 40);
        tuned.node_branch_limit = self.node_branch_limit.saturating_sub(2).clamp(6, 18);
        tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 240);
        tuned.node_enum_limit = (tuned.node_branch_limit * 4).clamp(tuned.node_branch_limit, 108);
        tuned
    }

    fn with_normal_deeper_shape(self) -> Self {
        let mut tuned = self;
        tuned.root_branch_limit = self.root_branch_limit.clamp(8, 36);
        tuned.node_branch_limit = (self.node_branch_limit + 3).clamp(9, 18);
        tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 220);
        tuned.node_enum_limit = (tuned.node_branch_limit * 6).clamp(tuned.node_branch_limit, 132);
        tuned
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone)]
struct ScoredRootMove {
    inputs: Vec<Input>,
    game: MonsGame,
    heuristic: i32,
    efficiency: i32,
    wins_immediately: bool,
    attacks_opponent_drainer: bool,
    own_drainer_vulnerable: bool,
    own_drainer_walk_vulnerable: bool,
    spirit_development: bool,
    keeps_awake_spirit_on_base: bool,
    mana_handoff_to_opponent: bool,
    has_roundtrip: bool,
    scores_supermana_this_turn: bool,
    scores_opponent_mana_this_turn: bool,
    safe_supermana_pickup_now: bool,
    safe_opponent_mana_pickup_now: bool,
    safe_supermana_progress_steps: i32,
    safe_opponent_mana_progress_steps: i32,
    score_path_best_steps: i32,
    same_turn_score_window_value: i32,
    spirit_same_turn_score_setup_now: bool,
    spirit_own_mana_setup_now: bool,
    supermana_progress: bool,
    opponent_mana_progress: bool,
    interview_soft_priority: i32,
    classes: MoveClassFlags,
}

#[cfg(any(target_arch = "wasm32", test))]
struct RootEvaluation {
    score: i32,
    efficiency: i32,
    inputs: Vec<Input>,
    game: MonsGame,
    wins_immediately: bool,
    attacks_opponent_drainer: bool,
    own_drainer_vulnerable: bool,
    own_drainer_walk_vulnerable: bool,
    spirit_development: bool,
    keeps_awake_spirit_on_base: bool,
    mana_handoff_to_opponent: bool,
    has_roundtrip: bool,
    scores_supermana_this_turn: bool,
    scores_opponent_mana_this_turn: bool,
    safe_supermana_pickup_now: bool,
    safe_opponent_mana_pickup_now: bool,
    safe_supermana_progress_steps: i32,
    safe_opponent_mana_progress_steps: i32,
    score_path_best_steps: i32,
    same_turn_score_window_value: i32,
    spirit_same_turn_score_setup_now: bool,
    spirit_own_mana_setup_now: bool,
    supermana_progress: bool,
    opponent_mana_progress: bool,
    interview_soft_priority: i32,
    classes: MoveClassFlags,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy)]
struct MoveEfficiencySnapshot {
    my_best_carrier_steps: i32,
    opponent_best_carrier_steps: i32,
    my_best_drainer_to_mana_steps: i32,
    opponent_best_drainer_to_mana_steps: i32,
    my_carrier_count: i32,
    opponent_carrier_count: i32,
    my_spirit_on_base: bool,
    opponent_spirit_on_base: bool,
    my_spirit_action_targets: i32,
    opponent_spirit_action_targets: i32,
    my_same_turn_score_value: i32,
    opponent_same_turn_score_value: i32,
    my_same_turn_opponent_mana_score_value: i32,
    opponent_same_turn_opponent_mana_score_value: i32,
    my_safe_supermana_progress: bool,
    opponent_safe_supermana_progress: bool,
    my_safe_opponent_mana_progress: bool,
    opponent_safe_opponent_mana_progress: bool,
    my_safe_supermana_progress_steps: i32,
    opponent_safe_supermana_progress_steps: i32,
    my_safe_opponent_mana_progress_steps: i32,
    opponent_safe_opponent_mana_progress_steps: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy)]
struct NormalRootSafetySnapshot {
    allows_immediate_opponent_win: bool,
    opponent_reaches_match_point: bool,
    opponent_max_score_gain: i32,
    my_score_gain: i32,
    worst_reply_score: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum MoveClass {
    ImmediateScore,
    DrainerAttack,
    DrainerSafetyRecover,
    CarrierProgress,
    Material,
    Quiet,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy, Debug, Default)]
struct MoveClassFlags {
    immediate_score: bool,
    drainer_attack: bool,
    drainer_safety_recover: bool,
    carrier_progress: bool,
    material: bool,
    quiet: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
impl MoveClassFlags {
    fn has(self, class: MoveClass) -> bool {
        match class {
            MoveClass::ImmediateScore => self.immediate_score,
            MoveClass::DrainerAttack => self.drainer_attack,
            MoveClass::DrainerSafetyRecover => self.drainer_safety_recover,
            MoveClass::CarrierProgress => self.carrier_progress,
            MoveClass::Material => self.material,
            MoveClass::Quiet => self.quiet,
        }
    }

    fn is_tactical_priority(self) -> bool {
        self.immediate_score || self.drainer_attack || self.drainer_safety_recover
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy)]
struct RootReplyRiskSnapshot {
    allows_immediate_opponent_win: bool,
    opponent_reaches_match_point: bool,
    worst_reply_score: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy)]
enum TranspositionBound {
    Exact,
    LowerBound,
    UpperBound,
}

#[cfg(any(target_arch = "wasm32", test))]
type KillerTable = [[u64; 2]; MAX_SMART_SEARCH_DEPTH + 2];

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy)]
struct TranspositionEntry {
    depth: usize,
    score: i32,
    bound: TranspositionBound,
    best_child_hash: u64,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone)]
struct RankedChildState {
    game: MonsGame,
    hash: u64,
    ordering_efficiency: i32,
    tactical_extension_trigger: bool,
    quiet_reduction_candidate: bool,
    classes: MoveClassFlags,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone)]
struct LegalInputTransition {
    inputs: Vec<Input>,
    game: MonsGame,
    events: Vec<Event>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AsyncSmartSearchPhase {
    PendingRootRanking,
    PendingFocusedCandidates,
    Scoring,
}

#[cfg(test)]
enum AsyncSmartSearchStart {
    Immediate(OutputModel),
    Pending(AsyncSmartSearchState),
}

#[cfg(test)]
struct AsyncSmartSearchState {
    game: MonsGame,
    perspective: Color,
    config: SmartSearchConfig,
    phase: AsyncSmartSearchPhase,
    root_moves: Vec<ScoredRootMove>,
    pending_output: Option<OutputModel>,
    next_index: usize,
    visited_nodes: usize,
    alpha: i32,
    extension_nodes_used: usize,
    extension_node_budget: usize,
    scored_roots: Vec<RootEvaluation>,
    transposition_table: U64HashMap<TranspositionEntry>,
    killer_table: KillerTable,
}

#[wasm_bindgen]
impl MonsGameModel {
    fn with_game(game: MonsGame) -> Self {
        Self {
            game,
            #[cfg(any(target_arch = "wasm32", test))]
            pro_runtime_context_hint: std::cell::Cell::new(ProRuntimeContext::Unknown),
            #[cfg(target_arch = "wasm32")]
            smart_search_in_progress: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
    }

    pub fn new() -> MonsGameModel {
        Self::with_game(MonsGame::new(true))
    }

    #[wasm_bindgen(js_name = newForSimulation)]
    pub fn new_for_simulation() -> MonsGameModel {
        let mut game = MonsGame::new(false);
        game.set_takeback_history_tracking(false);
        Self::with_game(game)
    }

    pub fn from_fen(fen: &str) -> Option<MonsGameModel> {
        if let Some(game) = MonsGame::from_fen(fen, true) {
            Some(Self::with_game(game))
        } else {
            return None;
        }
    }

    #[wasm_bindgen(js_name = fromFenForSimulation)]
    pub fn from_fen_for_simulation(fen: &str) -> Option<MonsGameModel> {
        MonsGame::from_fen(fen, false).map(|mut game| {
            game.set_takeback_history_tracking(false);
            Self::with_game(game)
        })
    }

    pub fn without_last_turn(&self, takeback_fens: Vec<String>) -> Option<MonsGameModel> {
        let mut verbose_tracking_entities = self.game.verbose_tracking_entities.clone();

        if verbose_tracking_entities.len() <= 1 {
            return None;
        }

        verbose_tracking_entities.pop();

        let fen = verbose_tracking_entities
            .last()
            .map(|e| e.fen.clone())
            .unwrap_or_else(|| self.game.fen());

        if let Some(mut new_game) = MonsGame::from_fen(fen.as_str(), true) {
            new_game.takeback_fens = takeback_fens;
            new_game.verbose_tracking_entities = verbose_tracking_entities;
            new_game.with_verbose_tracking = self.game.with_verbose_tracking;
            new_game.is_moves_verified = self.game.is_moves_verified;
            return Some(Self::with_game(new_game));
        }

        None
    }

    pub fn fen(&self) -> String {
        return self.game.fen();
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = smartAutomoveAsync)]
    pub fn smart_automove_async(&self, preference: &str) -> js_sys::Promise {
        let Some(preference) = SmartAutomovePreference::from_api_value(preference) else {
            let message = format!(
                "invalid smart automove mode; expected '{}', '{}', '{}', or '{}'",
                SmartAutomovePreference::Fast.as_api_value(),
                SmartAutomovePreference::Normal.as_api_value(),
                SmartAutomovePreference::Pro.as_api_value(),
                SmartAutomovePreference::Ultra.as_api_value(),
            );
            return js_sys::Promise::reject(&JsValue::from_str(message.as_str()));
        };

        if self.smart_search_in_progress.get() {
            return js_sys::Promise::reject(&JsValue::from_str(
                "smart automove already in progress",
            ));
        }

        self.smart_search_in_progress.set(true);
        let output = self.smart_automove_output(preference);
        self.smart_search_in_progress.set(false);
        js_sys::Promise::resolve(&JsValue::from(output))
    }

    pub fn automove(&mut self) -> OutputModel {
        return Self::automove_game(&mut self.game);
    }

    fn automove_game(game: &mut MonsGame) -> OutputModel {
        if let Some(opening_inputs) = Self::white_first_turn_opening_next_inputs(game) {
            let input_fen = Input::fen_from_array(&opening_inputs);
            let output = game.process_input(opening_inputs, false, false);
            if matches!(output, Output::Events(_)) {
                return OutputModel::new(output, input_fen.as_str());
            }
        }

        let automove_start_options = Some(SuggestedStartInputOptions::for_automove());
        let mut inputs = Vec::new();
        let mut output =
            game.process_input_with_start_options_slice(&[], false, false, automove_start_options);

        loop {
            match output {
                Output::InvalidInput => {
                    return OutputModel::new(Output::InvalidInput, "");
                }
                Output::LocationsToStartFrom(locations) => {
                    if locations.is_empty() {
                        return OutputModel::new(Output::InvalidInput, "");
                    }
                    let random_index = random_index(locations.len());
                    let location = locations[random_index];
                    inputs.push(Input::Location(location));
                    output = game.process_input_with_start_options_slice(
                        inputs.as_slice(),
                        false,
                        false,
                        automove_start_options,
                    );
                }
                Output::NextInputOptions(options) => {
                    if options.is_empty() {
                        return OutputModel::new(Output::InvalidInput, "");
                    }
                    let random_index = random_index(options.len());
                    let next_input = options[random_index].input;
                    inputs.push(next_input);
                    output = game.process_input_with_start_options_slice(
                        inputs.as_slice(),
                        false,
                        false,
                        automove_start_options,
                    );
                }
                Output::Events(events) => {
                    let input_fen = Input::fen_from_array(&inputs);
                    return OutputModel::new(Output::Events(events), input_fen.as_str());
                }
            }
        }
    }

    pub fn process_input(
        &mut self,
        locations: Vec<Location>,
        modifier: Option<Modifier>,
    ) -> OutputModel {
        let mut inputs: Vec<Input> = locations.into_iter().map(Input::Location).collect();
        if let Some(modifier) = modifier {
            inputs.push(Input::Modifier(modifier));
        }
        let input_fen = Input::fen_from_array(&inputs);
        let output = self.game.process_input(inputs, false, false);
        return OutputModel::new(output, input_fen.as_str());
    }

    pub fn can_takeback(&self, color: Color) -> bool {
        return self.game.can_takeback(color);
    }

    #[wasm_bindgen(js_name = setVerboseTracking)]
    pub fn set_verbose_tracking(&mut self, enabled: bool) {
        self.game.set_verbose_tracking(enabled);
    }

    #[wasm_bindgen(js_name = clearTracking)]
    pub fn clear_tracking(&mut self) {
        self.game.clear_tracking();
    }

    pub fn takeback(&mut self) -> OutputModel {
        let inputs: Vec<Input> = vec![Input::Takeback];
        let input_fen = Input::fen_from_array(&inputs);
        let output = self.game.process_input(inputs, false, false);
        return OutputModel::new(output, input_fen.as_str());
    }

    pub fn process_input_fen(&mut self, input_fen: &str) -> OutputModel {
        let inputs = Input::array_from_fen(input_fen);
        let output = self.game.process_input(inputs, false, false);
        return OutputModel::new(output, input_fen);
    }

    pub fn remove_item(&mut self, location: Location) {
        self.game.board.remove_item(location);
        self.game.invalidate_process_input_cache();
    }

    pub fn item(&self, at: Location) -> Option<ItemModel> {
        if let Some(item) = self.game.board.item(at) {
            return Some(ItemModel::new(item));
        } else {
            return None;
        }
    }

    pub fn square(&self, at: Location) -> SquareModel {
        let square = self.game.board.square(at);
        return SquareModel::new(&square);
    }

    pub fn is_later_than(&self, other_fen: &str) -> bool {
        if let Some(other_game) = MonsGame::from_fen(other_fen, false) {
            return self.game.is_later_than(&other_game);
        } else {
            return true;
        }
    }

    pub fn is_moves_verified(&self) -> bool {
        return self.game.is_moves_verified;
    }

    pub fn verify_moves(&mut self, flat_moves_string_w: &str, flat_moves_string_b: &str) -> bool {
        let moves_w: Vec<&str> = if flat_moves_string_w.is_empty() {
            Vec::new()
        } else {
            flat_moves_string_w.split("-").collect()
        };
        let moves_b: Vec<&str> = if flat_moves_string_b.is_empty() {
            Vec::new()
        } else {
            flat_moves_string_b.split("-").collect()
        };

        let with_verbose_tracking = self.game.with_verbose_tracking;
        let mut fresh_verification_game = MonsGame::new(with_verbose_tracking);

        let mut w_index = 0;
        let mut b_index = 0;

        while w_index < moves_w.len() || b_index < moves_b.len() {
            if fresh_verification_game.active_color == Color::White {
                if w_index >= moves_w.len() {
                    return false;
                }
                let inputs = Input::array_from_fen(moves_w[w_index]);
                _ = fresh_verification_game.process_input(inputs, false, false);
                w_index += 1;
            } else {
                if b_index >= moves_b.len() {
                    return false;
                }
                let inputs = Input::array_from_fen(moves_b[b_index]);
                _ = fresh_verification_game.process_input(inputs, false, false);
                b_index += 1;
            }
        }

        if fresh_verification_game.fen() == self.game.fen() {
            self.game.takeback_fens = fresh_verification_game.takeback_fens;
            if with_verbose_tracking {
                self.game.verbose_tracking_entities =
                    fresh_verification_game.verbose_tracking_entities;
            } else {
                self.game.verbose_tracking_entities.clear();
                self.game.verbose_tracking_entities.shrink_to_fit();
            }
            self.game.is_moves_verified = true;
            return true;
        } else {
            return false;
        }
    }

    pub fn active_color(&self) -> Color {
        return self.game.active_color;
    }

    pub fn winner_color(&self) -> Option<Color> {
        return self.game.winner_color();
    }

    pub fn black_score(&self) -> i32 {
        return self.game.black_score;
    }

    pub fn white_score(&self) -> i32 {
        return self.game.white_score;
    }

    pub fn turn_number(&self) -> i32 {
        return self.game.turn_number;
    }

    pub fn inactive_player_items_counters(&self) -> Vec<i32> {
        let player_potions_count = match self.game.active_color.other() {
            Color::White => self.game.white_potions_count,
            Color::Black => self.game.black_potions_count,
        };
        return [0, 0, 0, player_potions_count].to_vec();
    }

    pub fn takeback_fens(&self) -> Vec<String> {
        self.game.takeback_fens.clone()
    }

    pub fn available_move_kinds(&self) -> Vec<i32> {
        let map = self.game.available_move_kinds();
        return [
            map[&AvailableMoveKind::MonMove],
            map[&AvailableMoveKind::ManaMove],
            map[&AvailableMoveKind::Action],
            map[&AvailableMoveKind::Potion],
        ]
        .to_vec();
    }

    pub fn locations_with_content(&self) -> Vec<Location> {
        let mut locations: Vec<Location> = self
            .game
            .board
            .items
            .iter()
            .enumerate()
            .filter_map(|(idx, opt)| opt.as_ref().map(|_| Location::from_index(idx)))
            .collect();
        let mons_bases = self.game.board.all_mons_bases();
        locations.extend(mons_bases);
        locations.sort();
        locations.dedup();
        return locations;
    }

    pub fn verbose_tracking_entities(&self) -> Vec<VerboseTrackingEntityModel> {
        self.game
            .verbose_tracking_entities
            .iter()
            .map(|e| VerboseTrackingEntityModel::new(e))
            .collect()
    }
}

impl MonsGameModel {
    fn white_first_turn_opening_next_inputs(game: &MonsGame) -> Option<Vec<Input>> {
        if game.active_color != Color::White || !game.is_first_turn() {
            return None;
        }

        let parsed_book = &*PARSED_WHITE_OPENING_BOOK;
        let opening_step = game.mons_moves_count.max(0) as usize;
        if opening_step >= parsed_book[0].len() {
            return None;
        }

        let current_fen = game.fen();
        let mut viable_sequences = Vec::new();

        for (sequence_index, sequence) in parsed_book.iter().enumerate() {
            let mut simulated = MonsGame::new(false);
            let mut prefix_is_valid = true;
            for step_inputs in sequence.iter().take(opening_step) {
                if !matches!(
                    simulated.process_input(step_inputs.clone(), false, false),
                    Output::Events(_)
                ) {
                    prefix_is_valid = false;
                    break;
                }
            }

            if !prefix_is_valid || simulated.fen() != current_fen {
                continue;
            }

            let next_inputs = &sequence[opening_step];
            let mut probe = game.clone_for_simulation();
            if matches!(
                probe.process_input_slice(next_inputs.as_slice(), true, false),
                Output::Events(_)
            ) {
                viable_sequences.push(sequence_index);
            }
        }

        if viable_sequences.is_empty() {
            return None;
        }

        let chosen = viable_sequences[random_index(viable_sequences.len())];
        Some(parsed_book[chosen][opening_step].clone())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl MonsGameModel {
    fn mark_opening_book_driven_context(&self) {
        self.pro_runtime_context_hint
            .set(ProRuntimeContext::OpeningBookDriven);
    }

    fn smart_automove_output(&self, preference: SmartAutomovePreference) -> OutputModel {
        if let Some(opening_inputs) = Self::white_first_turn_opening_next_inputs(&self.game) {
            let mut game = self.game.clone_for_simulation();
            let input_fen = Input::fen_from_array(&opening_inputs);
            let output = game.process_input(opening_inputs, false, false);
            if matches!(output, Output::Events(_)) {
                self.mark_opening_book_driven_context();
                return OutputModel::new(output, input_fen.as_str());
            }
        }

        let config = self.runtime_config_for_preference(preference);
        let inputs = Self::smart_search_best_inputs(&self.game, config);
        if inputs.is_empty() {
            let mut game = self.game.clone_for_simulation();
            return Self::automove_game(&mut game);
        }

        let mut game = self.game.clone_for_simulation();
        let input_fen = Input::fen_from_array(&inputs);
        let output = game.process_input(inputs, false, false);
        OutputModel::new(output, input_fen.as_str())
    }

    #[cfg(test)]
    fn new_async_smart_search_state(
        game: MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
    ) -> AsyncSmartSearchState {
        AsyncSmartSearchState {
            game,
            perspective,
            config,
            phase: AsyncSmartSearchPhase::PendingRootRanking,
            root_moves: Vec::new(),
            pending_output: None,
            next_index: 0,
            visited_nodes: 0,
            alpha: i32::MIN,
            extension_nodes_used: 0,
            extension_node_budget: 0,
            scored_roots: Vec::new(),
            transposition_table: U64HashMap::default(),
            killer_table: [[0u64; 2]; MAX_SMART_SEARCH_DEPTH + 2],
        }
    }

    #[cfg(test)]
    fn begin_async_smart_search(
        &self,
        preference: SmartAutomovePreference,
    ) -> AsyncSmartSearchStart {
        if let Some(opening_inputs) = Self::white_first_turn_opening_next_inputs(&self.game) {
            let mut game = self.game.clone_for_simulation();
            let input_fen = Input::fen_from_array(&opening_inputs);
            let output = game.process_input(opening_inputs, false, false);
            if matches!(output, Output::Events(_)) {
                self.mark_opening_book_driven_context();
                return AsyncSmartSearchStart::Immediate(OutputModel::new(
                    output,
                    input_fen.as_str(),
                ));
            }
        }

        clear_exact_state_analysis_cache();
        let config = self.runtime_config_for_preference(preference);
        let perspective = self.game.active_color;
        let game = self.game.clone_for_simulation();
        AsyncSmartSearchStart::Pending(Self::new_async_smart_search_state(
            game,
            perspective,
            config,
        ))
    }

    fn runtime_config_for_preference(
        &self,
        preference: SmartAutomovePreference,
    ) -> SmartSearchConfig {
        let hinted = self.pro_runtime_context_hint.get();
        let (config, resolved_context) =
            Self::runtime_config_for_game_with_context(&self.game, preference, hinted);
        self.pro_runtime_context_hint.set(resolved_context);
        config
    }

    #[cfg(test)]
    fn smart_automove_output_via_async_loop(
        &self,
        preference: SmartAutomovePreference,
    ) -> OutputModel {
        let mut state = match self.begin_async_smart_search(preference) {
            AsyncSmartSearchStart::Immediate(output) => return output,
            AsyncSmartSearchStart::Pending(state) => state,
        };

        for _ in 0..1_000_000usize {
            if Self::advance_async_search(&mut state) {
                return Self::finalize_async_search(&mut state);
            }
        }

        panic!(
            "async smart search did not finish for {} within tick budget",
            preference.as_api_value()
        );
    }

    fn runtime_config_for_game_with_context(
        game: &MonsGame,
        preference: SmartAutomovePreference,
        hinted_context: ProRuntimeContext,
    ) -> (SmartSearchConfig, ProRuntimeContext) {
        let opening_black_reply = hinted_context == ProRuntimeContext::OpeningBookDriven
            || Self::detect_opening_book_context(game);
        let mut config = Self::with_runtime_scoring_weights(
            game,
            SmartSearchConfig::from_preference(preference),
        );
        let resolved_context = if Self::uses_deep_runtime_context(preference) {
            let resolved = Self::resolve_pro_runtime_context(game, hinted_context);
            config = Self::apply_deep_runtime_context_profile(game, config, preference, resolved);
            resolved
        } else {
            hinted_context
        };
        if opening_black_reply {
            config = match preference {
                SmartAutomovePreference::Fast => Self::apply_fast_opening_reply_profile(config),
                SmartAutomovePreference::Normal => {
                    Self::apply_normal_opening_reply_profile(config)
                }
                SmartAutomovePreference::Pro => Self::apply_pro_opening_reply_profile(config),
                SmartAutomovePreference::Ultra => {
                    Self::apply_ultra_opening_reply_profile(config)
                }
            };
        }
        (config, resolved_context)
    }

    fn uses_deep_runtime_context(preference: SmartAutomovePreference) -> bool {
        matches!(
            preference,
            SmartAutomovePreference::Pro | SmartAutomovePreference::Ultra
        )
    }

    fn apply_deep_runtime_context_profile(
        game: &MonsGame,
        config: SmartSearchConfig,
        preference: SmartAutomovePreference,
        context: ProRuntimeContext,
    ) -> SmartSearchConfig {
        match preference {
            SmartAutomovePreference::Pro => {
                Self::apply_pro_runtime_context_profile(game, config, context)
            }
            SmartAutomovePreference::Ultra => {
                Self::apply_ultra_runtime_context_profile(game, config, context)
            }
            SmartAutomovePreference::Fast | SmartAutomovePreference::Normal => config,
        }
    }

    fn resolve_pro_runtime_context(
        game: &MonsGame,
        hinted_context: ProRuntimeContext,
    ) -> ProRuntimeContext {
        if hinted_context == ProRuntimeContext::OpeningBookDriven {
            return ProRuntimeContext::OpeningBookDriven;
        }
        if Self::detect_opening_book_context(game) {
            ProRuntimeContext::OpeningBookDriven
        } else {
            ProRuntimeContext::Independent
        }
    }

    fn detect_opening_book_context(game: &MonsGame) -> bool {
        if game.turn_number != 2 || game.active_color != Color::Black {
            return false;
        }
        let current_fen = game.fen();
        for sequence in PARSED_WHITE_OPENING_BOOK.iter() {
            if sequence.is_empty() {
                continue;
            }
            let mut simulated = MonsGame::new(false);
            let mut valid = true;
            for step_inputs in sequence.iter() {
                if !matches!(
                    simulated.process_input(step_inputs.clone(), false, false),
                    Output::Events(_)
                ) {
                    valid = false;
                    break;
                }
                if simulated.turn_number == 2 && simulated.active_color == Color::Black {
                    break;
                }
            }
            if valid
                && simulated.turn_number == 2
                && simulated.active_color == Color::Black
                && simulated.fen() == current_fen
            {
                return true;
            }
        }
        false
    }

    fn apply_fast_opening_reply_profile(config: SmartSearchConfig) -> SmartSearchConfig {
        if config.depth > SMART_AUTOMOVE_FAST_DEPTH as usize {
            return config;
        }
        Self::apply_opening_reply_latency_profile(config, 2, 320, 18, 8, 108, 48)
    }

    fn apply_normal_opening_reply_profile(config: SmartSearchConfig) -> SmartSearchConfig {
        Self::apply_opening_reply_latency_profile(config, 2, 420, 20, 9, 120, 56)
    }

    fn apply_pro_opening_reply_profile(config: SmartSearchConfig) -> SmartSearchConfig {
        Self::apply_opening_reply_latency_profile(config, 3, 1_100, 20, 10, 132, 72)
    }

    fn apply_ultra_opening_reply_profile(config: SmartSearchConfig) -> SmartSearchConfig {
        Self::apply_opening_reply_latency_profile(config, 4, 2_400, 22, 11, 144, 84)
    }

    fn apply_opening_reply_latency_profile(
        mut config: SmartSearchConfig,
        depth: usize,
        max_visited_nodes: usize,
        root_branch_limit: usize,
        node_branch_limit: usize,
        root_enum_limit: usize,
        node_enum_limit: usize,
    ) -> SmartSearchConfig {
        config.depth = depth;
        config.max_visited_nodes = max_visited_nodes;
        config.root_branch_limit = root_branch_limit;
        config.node_branch_limit = node_branch_limit;
        config.root_enum_limit = root_enum_limit;
        config.node_enum_limit = node_enum_limit;
        config.enable_root_efficiency = false;
        config.enable_child_move_class_coverage = false;
        config.enable_two_pass_root_allocation = false;
        config.enable_root_aspiration = false;
        config.enable_selective_extensions = false;
        config.enable_normal_root_safety_deep_floor = false;
        config.root_reply_risk_reply_limit = config.root_reply_risk_reply_limit.min(8);
        config.root_reply_risk_node_share_bp = config.root_reply_risk_node_share_bp.min(560);
        config
    }

    fn apply_pro_runtime_context_profile(
        game: &MonsGame,
        config: SmartSearchConfig,
        context: ProRuntimeContext,
    ) -> SmartSearchConfig {
        match context {
            ProRuntimeContext::OpeningBookDriven => Self::apply_pro_confirmation_profile(config),
            ProRuntimeContext::Unknown | ProRuntimeContext::Independent => {
                Self::apply_pro_primary_profile(game, config)
            }
        }
    }

    fn apply_ultra_runtime_context_profile(
        game: &MonsGame,
        config: SmartSearchConfig,
        context: ProRuntimeContext,
    ) -> SmartSearchConfig {
        match context {
            ProRuntimeContext::OpeningBookDriven => Self::apply_ultra_confirmation_profile(config),
            ProRuntimeContext::Unknown | ProRuntimeContext::Independent => {
                Self::apply_ultra_primary_profile(game, config)
            }
        }
    }

    fn apply_pro_primary_profile(
        game: &MonsGame,
        mut config: SmartSearchConfig,
    ) -> SmartSearchConfig {
        if config.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
            return config;
        }
        config.max_visited_nodes = SMART_AUTOMOVE_PRO_MAX_VISITED_NODES as usize;
        config.enable_forced_tactical_prepass = false;
        config.root_branch_limit = config.root_branch_limit.clamp(14, 34);
        config.node_branch_limit = config.node_branch_limit.clamp(9, 15);
        config.root_enum_limit =
            (config.root_branch_limit * 6).clamp(config.root_branch_limit, 204);
        config.node_enum_limit =
            ((config.node_branch_limit + 2) * 6).clamp(config.node_branch_limit, 132);
        config.enable_futility_pruning = true;
        config.futility_margin = 2_300;
        config.enable_quiet_reductions = true;
        config.quiet_reduction_depth_threshold = 2;
        config.enable_root_reply_risk_guard = true;
        config.root_reply_risk_score_margin = 165;
        config.root_reply_risk_shortlist_max = 9;
        config.root_reply_risk_reply_limit = 24;
        config.root_reply_risk_node_share_bp = 2_000;
        config.enable_normal_root_safety_rerank = true;
        config.enable_normal_root_safety_deep_floor = true;
        config.root_drainer_safety_score_margin = 4_800;
        config.enable_selective_extensions = true;
        config.max_extensions_per_path = 1;
        config.selective_extension_node_share_bp = 1_500;
        config.scoring_weights =
            Self::runtime_phase_adaptive_attacker_proximity_scoring_weights(game, config.depth);
        config.interview_soft_opponent_mana_progress_bonus = 320;
        config.interview_soft_opponent_mana_score_bonus = 400;
        config
    }

    fn apply_pro_confirmation_profile(mut config: SmartSearchConfig) -> SmartSearchConfig {
        if config.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
            return config;
        }
        config.max_visited_nodes = SMART_AUTOMOVE_PRO_MAX_VISITED_NODES as usize;
        config.enable_forced_tactical_prepass = false;
        config.root_branch_limit = config.root_branch_limit.clamp(14, 34);
        config.node_branch_limit = config.node_branch_limit.clamp(9, 15);
        config.root_enum_limit =
            (config.root_branch_limit * 6).clamp(config.root_branch_limit, 204);
        config.node_enum_limit =
            ((config.node_branch_limit + 2) * 6).clamp(config.node_branch_limit, 132);
        config.enable_futility_pruning = true;
        config.futility_margin = 2_500;
        config.enable_quiet_reductions = true;
        config.quiet_reduction_depth_threshold = 2;
        config.enable_root_reply_risk_guard = true;
        config.root_reply_risk_score_margin = 155;
        config.root_reply_risk_shortlist_max = 7;
        config.root_reply_risk_reply_limit = 18;
        config.root_reply_risk_node_share_bp = 1_400;
        config.enable_normal_root_safety_rerank = true;
        config.enable_normal_root_safety_deep_floor = false;
        config.root_drainer_safety_score_margin = 4_300;
        config.enable_selective_extensions = true;
        config.max_extensions_per_path = 1;
        config.selective_extension_node_share_bp = 1_200;
        config
    }

    fn apply_ultra_primary_profile(
        game: &MonsGame,
        mut config: SmartSearchConfig,
    ) -> SmartSearchConfig {
        if config.depth < SMART_AUTOMOVE_ULTRA_DEPTH as usize {
            return config;
        }
        config.max_visited_nodes = SMART_AUTOMOVE_ULTRA_MAX_VISITED_NODES as usize;
        config.enable_forced_tactical_prepass = false;
        config.enable_two_pass_root_allocation = true;
        config.root_focus_k = 4;
        config.root_focus_budget_share_bp = 7_600;
        config.root_branch_limit = config.root_branch_limit.clamp(16, 48);
        config.node_branch_limit = config.node_branch_limit.clamp(10, 20);
        config.root_enum_limit =
            (config.root_branch_limit * 6).clamp(config.root_branch_limit, 288);
        config.node_enum_limit =
            ((config.node_branch_limit + 2) * 6).clamp(config.node_branch_limit, 168);
        config.enable_futility_pruning = true;
        config.futility_margin = 2_500;
        config.enable_quiet_reductions = true;
        config.quiet_reduction_depth_threshold = 2;
        config.enable_root_reply_risk_guard = true;
        config.root_reply_risk_score_margin = 175;
        config.root_reply_risk_shortlist_max = 10;
        config.root_reply_risk_reply_limit = 30;
        config.root_reply_risk_node_share_bp = 2_400;
        config.enable_normal_root_safety_rerank = true;
        config.enable_normal_root_safety_deep_floor = true;
        config.root_drainer_safety_score_margin = 5_000;
        config.enable_selective_extensions = true;
        config.max_extensions_per_path = 1;
        config.selective_extension_node_share_bp = 1_900;
        config.scoring_weights =
            Self::runtime_phase_adaptive_attacker_proximity_scoring_weights(game, config.depth);
        config.interview_soft_opponent_mana_progress_bonus = 330;
        config.interview_soft_opponent_mana_score_bonus = 390;
        config
    }

    fn apply_ultra_confirmation_profile(mut config: SmartSearchConfig) -> SmartSearchConfig {
        if config.depth < SMART_AUTOMOVE_ULTRA_DEPTH as usize {
            return config;
        }
        config.max_visited_nodes = SMART_AUTOMOVE_ULTRA_MAX_VISITED_NODES as usize;
        config.enable_forced_tactical_prepass = false;
        config.enable_two_pass_root_allocation = true;
        config.root_focus_k = 4;
        config.root_focus_budget_share_bp = 7_300;
        config.root_branch_limit = config.root_branch_limit.clamp(16, 46);
        config.node_branch_limit = config.node_branch_limit.clamp(10, 19);
        config.root_enum_limit =
            (config.root_branch_limit * 6).clamp(config.root_branch_limit, 276);
        config.node_enum_limit =
            ((config.node_branch_limit + 2) * 6).clamp(config.node_branch_limit, 162);
        config.enable_futility_pruning = true;
        config.futility_margin = 2_700;
        config.enable_quiet_reductions = true;
        config.quiet_reduction_depth_threshold = 2;
        config.enable_root_reply_risk_guard = true;
        config.root_reply_risk_score_margin = 165;
        config.root_reply_risk_shortlist_max = 8;
        config.root_reply_risk_reply_limit = 22;
        config.root_reply_risk_node_share_bp = 1_700;
        config.enable_normal_root_safety_rerank = true;
        config.enable_normal_root_safety_deep_floor = false;
        config.root_drainer_safety_score_margin = 4_600;
        config.enable_selective_extensions = true;
        config.max_extensions_per_path = 1;
        config.selective_extension_node_share_bp = 1_200;
        config
    }

    #[cfg(test)]
    fn pro_runtime_context_hint_for_tests(&self) -> ProRuntimeContext {
        self.pro_runtime_context_hint.get()
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn with_runtime_scoring_weights(
        game: &MonsGame,
        mut config: SmartSearchConfig,
    ) -> SmartSearchConfig {
        config.scoring_weights =
            if config.depth < 3 && config.enable_mana_start_mix_with_potion_actions {
                &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF
            } else {
                Self::runtime_phase_adaptive_walk_threat_medium_scoring_weights(game, config.depth)
            };
        if config.depth >= 3 {
            config.max_visited_nodes = (config.max_visited_nodes * 120) / 100;
        }
        config
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_scoring_weights(
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
            &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_boolean_drainer_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_BOOLEAN_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_BOOLEAN_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_BOOLEAN_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_attack_bonus_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_ATTACK_BONUS_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_ATTACK_BONUS_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_ATTACK_BONUS_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_ATTACK_BONUS_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_ATTACK_BONUS_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_strong_attack_bonus_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_STRONG_ATTACK_BONUS_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_STRONG_ATTACK_BONUS_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_STRONG_ATTACK_BONUS_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_STRONG_ATTACK_BONUS_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_STRONG_ATTACK_BONUS_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_drainer_shield_scoring_weights(
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
            &RUNTIME_NORMAL_DRAINER_SHIELD_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_DRAINER_SHIELD_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_DRAINER_SHIELD_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_DRAINER_SHIELD_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_DRAINER_SHIELD_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn with_drainer_shield_scoring_weights(
        game: &MonsGame,
        mut config: SmartSearchConfig,
    ) -> SmartSearchConfig {
        config.scoring_weights =
            if config.depth < 3 && config.enable_mana_start_mix_with_potion_actions {
                &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF
            } else {
                Self::runtime_phase_adaptive_walk_threat_scoring_weights(game, config.depth)
            };
        config
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_walk_threat_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_WALK_THREAT_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_WALK_THREAT_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_WALK_THREAT_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_WALK_THREAT_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_WALK_THREAT_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_walk_threat_light_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_WALK_THREAT_LIGHT_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_WALK_THREAT_LIGHT_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_WALK_THREAT_LIGHT_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_WALK_THREAT_LIGHT_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_WALK_THREAT_LIGHT_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_walk_threat_medium_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_WALK_THREAT_MEDIUM_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_WALK_THREAT_MEDIUM_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_WALK_THREAT_MEDIUM_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_WALK_THREAT_MEDIUM_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_WALK_THREAT_MEDIUM_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_walk_threat_moderate_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_WALK_THREAT_MODERATE_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_WALK_THREAT_MODERATE_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_WALK_THREAT_MODERATE_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_WALK_THREAT_MODERATE_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_WALK_THREAT_MODERATE_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_strong_drainer_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_STRONG_DRAINER_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_STRONG_DRAINER_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_STRONG_DRAINER_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_STRONG_DRAINER_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_STRONG_DRAINER_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_attacker_proximity_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_ATTACKER_PROXIMITY_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_ATTACKER_PROXIMITY_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_ATTACKER_PROXIMITY_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_ATTACKER_PROXIMITY_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_ATTACKER_PROXIMITY_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_strong_attacker_proximity_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_STRONG_ATTACKER_PROXIMITY_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_STRONG_ATTACKER_PROXIMITY_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_STRONG_ATTACKER_PROXIMITY_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_STRONG_ATTACKER_PROXIMITY_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_STRONG_ATTACKER_PROXIMITY_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn runtime_phase_adaptive_combo_proximity_attack_scoring_weights(
        game: &MonsGame,
        depth: usize,
    ) -> &'static ScoringWeights {
        if depth < 3 {
            return &RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS;
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
            &RUNTIME_NORMAL_COMBO_PROXIMITY_ATTACK_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 1 {
            &RUNTIME_NORMAL_COMBO_PROXIMITY_ATTACK_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS
        } else if my_distance_to_win <= 2 {
            &RUNTIME_NORMAL_COMBO_PROXIMITY_ATTACK_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS
        } else if opponent_distance_to_win <= 2 || score_gap <= -1 {
            &RUNTIME_NORMAL_COMBO_PROXIMITY_ATTACK_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS
        } else {
            &RUNTIME_NORMAL_COMBO_PROXIMITY_ATTACK_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS
        }
    }

    #[allow(dead_code)]
    fn build_scored_root_move(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        own_drainer_vulnerable_before: bool,
        inputs: &[Input],
    ) -> Option<ScoredRootMove> {
        let (simulated_game, events) = Self::apply_inputs_for_search_with_events(game, inputs)?;
        Self::build_scored_root_move_from_transition(
            game,
            perspective,
            config,
            own_drainer_vulnerable_before,
            inputs.to_vec(),
            simulated_game,
            events,
        )
    }

    fn build_scored_root_move_from_transition(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        own_drainer_vulnerable_before: bool,
        inputs: Vec<Input>,
        simulated_game: MonsGame,
        events: Vec<Event>,
    ) -> Option<ScoredRootMove> {
        let efficiency = if config.enable_root_efficiency {
            Self::move_efficiency_delta(
                game,
                &simulated_game,
                perspective,
                &events,
                true,
                config.enable_backtrack_penalty,
                config.enable_root_mana_handoff_guard,
                config.root_backtrack_penalty,
                config.root_mana_handoff_penalty,
            )
        } else {
            0
        };
        let heuristic = Self::score_state(
            &simulated_game,
            perspective,
            config.depth.saturating_sub(1),
            config.depth,
            config.scoring_weights,
        );
        let ordering_bonus = if config.enable_event_ordering_bonus {
            Self::ordering_event_bonus(game.active_color, perspective, &events)
        } else {
            0
        };
        let attacks_opponent_drainer =
            Self::events_include_opponent_drainer_fainted(&events, perspective);
        let wins_immediately = simulated_game.winner_color() == Some(perspective);
        let spirit_development =
            Self::has_spirit_development_turn(game, &simulated_game, perspective, &events);
        let keeps_awake_spirit_on_base = Self::has_awake_spirit_on_base(&game.board, perspective)
            && Self::has_awake_spirit_on_base(&simulated_game.board, perspective);
        let scores_supermana_this_turn = Self::events_score_supermana(&events);
        let scores_opponent_mana_this_turn = Self::events_score_opponent_mana(&events, perspective);
        let exact_turn = if simulated_game.active_color == perspective {
            if Self::root_transition_requires_active_turn_exact(&events, config) {
                exact_turn_summary(&simulated_game, perspective)
            } else {
                Self::approximate_active_turn_summary(&simulated_game, perspective)
            }
        } else {
            ExactTurnSummary {
                color: Some(perspective),
                ..ExactTurnSummary::default()
            }
        };
        let unknown_progress_steps = Config::BOARD_SIZE + 4;
        let safe_supermana_progress_steps = exact_turn
            .safe_supermana_progress_steps
            .unwrap_or(unknown_progress_steps);
        let safe_opponent_mana_progress_steps = exact_turn
            .safe_opponent_mana_progress_steps
            .unwrap_or(unknown_progress_steps);
        let unknown_score_path_steps = Config::BOARD_SIZE * 3;
        let same_turn_score_window_value = exact_turn.same_turn_score_window_value;
        let score_path_best_steps = exact_turn
            .score_path_best_steps
            .unwrap_or(unknown_score_path_steps);
        let spirit_same_turn_score_setup_now = Self::events_include_spirit_target_move(&events)
            && simulated_game.active_color == perspective
            && same_turn_score_window_value > 0;
        let safe_supermana_pickup_now = Self::events_pickup_supermana(&events)
            && Self::own_drainer_carries_specific_mana_safely(
                &simulated_game.board,
                perspective,
                Mana::Supermana,
            );
        let safe_opponent_mana_pickup_now = Self::events_pickup_opponent_mana(&events, perspective)
            && Self::own_drainer_carries_specific_mana_safely(
                &simulated_game.board,
                perspective,
                Mana::Regular(perspective.other()),
            );
        let spirit_own_mana_setup_now =
            Self::events_spirit_scoring_mana_setup(&events, &simulated_game.board, perspective);
        let supermana_progress = scores_supermana_this_turn
            || Self::events_pickup_supermana(&events)
            || Self::events_move_supermana_toward_color(&events, perspective)
            || Self::events_spirit_supermana_setup(&events, &simulated_game.board, perspective)
            || exact_turn.safe_supermana_progress
            || exact_turn.spirit_assisted_supermana_progress;
        let opponent_mana_progress = scores_opponent_mana_this_turn
            || Self::events_pickup_opponent_mana(&events, perspective)
            || Self::events_move_opponent_mana_toward_color(&events, perspective)
            || Self::events_spirit_opponent_mana_setup(&events, &simulated_game.board, perspective)
            || exact_turn.safe_opponent_mana_progress
            || exact_turn.spirit_assisted_opponent_mana_progress
            || exact_turn.spirit_assisted_denial;
        let spirit_assisted_score = exact_turn.spirit_assisted_score;
        let own_drainer_vulnerable = if config.enable_root_drainer_safety_prefilter {
            Self::is_own_drainer_vulnerable_next_turn(
                &simulated_game,
                perspective,
                config.enable_enhanced_drainer_vulnerability,
            )
        } else {
            false
        };
        let own_drainer_walk_vulnerable =
            if config.enable_walk_threat_prefilter && !own_drainer_vulnerable {
                Self::is_own_drainer_walk_vulnerable_next_turn(
                    &simulated_game,
                    perspective,
                    config.enable_enhanced_drainer_vulnerability,
                )
            } else {
                false
            };
        let score_before = Self::score_for_color(game, perspective);
        let score_after = Self::score_for_color(&simulated_game, perspective);
        let scored_two_or_more = score_after >= score_before.saturating_add(2);
        let spent_potion = Self::events_use_potion(&events);
        let potion_compensated = wins_immediately
            || attacks_opponent_drainer
            || scored_two_or_more
            || scores_supermana_this_turn
            || scores_opponent_mana_this_turn
            || spirit_assisted_score
            || (config.enable_potion_progress_compensation
                && !own_drainer_vulnerable
                && (supermana_progress || opponent_mana_progress));
        let potion_spend_penalty = if config.enable_mana_start_mix_with_potion_actions
            && spent_potion
            && !potion_compensated
        {
            if config.depth >= 3 {
                config.potion_spend_penalty_normal.max(0)
            } else {
                config.potion_spend_penalty_fast.max(0)
            }
        } else {
            0
        };
        let root_compensates_handoff = wins_immediately
            || attacks_opponent_drainer
            || scores_supermana_this_turn
            || scores_opponent_mana_this_turn
            || spirit_assisted_score;
        let mana_handoff_to_opponent = !root_compensates_handoff
            && Self::mana_handoff_penalty(
                &events,
                perspective,
                config.root_mana_handoff_penalty.max(0),
            ) > 0;
        let has_roundtrip = Self::has_roundtrip_mon_move(&events);
        let classes = if config.enable_move_class_coverage {
            Self::classify_move_classes(
                game,
                &simulated_game,
                perspective,
                &events,
                own_drainer_vulnerable_before,
                own_drainer_vulnerable,
            )
        } else {
            MoveClassFlags::default()
        };
        let interview_soft_priority = Self::interview_root_soft_priority(
            config,
            supermana_progress,
            opponent_mana_progress,
            safe_supermana_progress_steps,
            safe_opponent_mana_progress_steps,
            scores_supermana_this_turn,
            scores_opponent_mana_this_turn,
            own_drainer_vulnerable,
            mana_handoff_to_opponent,
            has_roundtrip,
        );

        let heuristic_with_policy = heuristic
            .saturating_add(ordering_bonus)
            .saturating_add(if config.enable_interview_soft_root_priors {
                interview_soft_priority
            } else {
                0
            })
            .saturating_sub(potion_spend_penalty);

        Some(ScoredRootMove {
            inputs,
            game: simulated_game,
            heuristic: heuristic_with_policy,
            efficiency,
            wins_immediately,
            attacks_opponent_drainer,
            own_drainer_vulnerable,
            own_drainer_walk_vulnerable,
            spirit_development,
            keeps_awake_spirit_on_base,
            mana_handoff_to_opponent,
            has_roundtrip,
            scores_supermana_this_turn,
            scores_opponent_mana_this_turn,
            safe_supermana_pickup_now,
            safe_opponent_mana_pickup_now,
            safe_supermana_progress_steps,
            safe_opponent_mana_progress_steps,
            score_path_best_steps,
            same_turn_score_window_value,
            spirit_same_turn_score_setup_now,
            spirit_own_mana_setup_now,
            supermana_progress,
            opponent_mana_progress,
            interview_soft_priority,
            classes,
        })
    }

    fn spirit_base_for_color(board: &Board, color: Color) -> Location {
        board.base(Mon::new(MonKind::Spirit, color, 0))
    }

    fn is_awake_spirit_item_for_color(item: &Item, color: Color) -> bool {
        item.mon()
            .map(|mon| mon.kind == MonKind::Spirit && mon.color == color && !mon.is_fainted())
            .unwrap_or(false)
    }

    fn has_awake_spirit_on_base(board: &Board, color: Color) -> bool {
        let base = Self::spirit_base_for_color(board, color);
        board
            .item(base)
            .map(|item| Self::is_awake_spirit_item_for_color(item, color))
            .unwrap_or(false)
    }

    fn has_awake_spirit_off_base(board: &Board, color: Color) -> bool {
        let base = Self::spirit_base_for_color(board, color);
        board.occupied().any(|(location, item)| {
            location != base && Self::is_awake_spirit_item_for_color(item, color)
        })
    }

    fn has_spirit_development_turn(
        before: &MonsGame,
        after: &MonsGame,
        perspective: Color,
        events: &[Event],
    ) -> bool {
        if events
            .iter()
            .any(|event| matches!(event, Event::SpiritTargetMove { .. }))
        {
            return true;
        }

        Self::has_awake_spirit_on_base(&before.board, perspective)
            && Self::has_awake_spirit_off_base(&after.board, perspective)
    }

    fn events_score_supermana(events: &[Event]) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::ManaScored {
                    mana: Mana::Supermana,
                    ..
                }
            )
        })
    }

    fn events_score_opponent_mana(events: &[Event], perspective: Color) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::ManaScored {
                    mana: Mana::Regular(owner),
                    ..
                } if *owner == perspective.other()
            )
        })
    }

    fn events_include_spirit_target_move(events: &[Event]) -> bool {
        events
            .iter()
            .any(|event| matches!(event, Event::SpiritTargetMove { .. }))
    }

    fn events_use_potion(events: &[Event]) -> bool {
        events
            .iter()
            .any(|event| matches!(event, Event::UsePotion { .. }))
    }

    fn events_pickup_supermana(events: &[Event]) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::PickupMana {
                    mana: Mana::Supermana,
                    ..
                }
            )
        })
    }

    fn events_pickup_opponent_mana(events: &[Event], perspective: Color) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::PickupMana {
                    mana: Mana::Regular(owner),
                    ..
                } if *owner == perspective.other()
            )
        })
    }

    fn mana_move_toward_color(from: Location, to: Location, color: Color) -> bool {
        let before = Self::distance_to_color_pool_steps_for_efficiency(from, color);
        let after = Self::distance_to_color_pool_steps_for_efficiency(to, color);
        after < before
    }

    fn events_move_supermana_toward_color(events: &[Event], color: Color) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::ManaMove {
                    mana: Mana::Supermana,
                    from,
                    to
                } if Self::mana_move_toward_color(*from, *to, color)
            )
        })
    }

    fn events_move_opponent_mana_toward_color(events: &[Event], perspective: Color) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::ManaMove {
                    mana: Mana::Regular(owner),
                    from,
                    to
                } if *owner == perspective.other() && Self::mana_move_toward_color(*from, *to, perspective)
            )
        })
    }

    fn events_spirit_move_own_mana_toward_color(events: &[Event], perspective: Color) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::SpiritTargetMove {
                    item: Item::Mana {
                        mana: Mana::Regular(owner),
                    },
                    from,
                    to,
                    ..
                } if *owner == perspective && Self::mana_move_toward_color(*from, *to, perspective)
            )
        })
    }

    fn events_spirit_move_opponent_mana_toward_color(events: &[Event], perspective: Color) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::SpiritTargetMove {
                    item: Item::Mana {
                        mana: Mana::Regular(owner),
                    },
                    from,
                    to,
                    ..
                } if *owner == perspective.other()
                    && Self::mana_move_toward_color(*from, *to, perspective)
            )
        })
    }

    fn events_spirit_move_opponent_mana_onto_safe_drainer(
        events: &[Event],
        board_after: &Board,
        perspective: Color,
    ) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::SpiritTargetMove {
                    item: Item::Mana {
                        mana: Mana::Regular(owner),
                    },
                    ..
                } if *owner == perspective.other()
            )
        }) && Self::own_drainer_carries_specific_mana_safely(
            board_after,
            perspective,
            Mana::Regular(perspective.other()),
        )
    }

    fn events_spirit_move_supermana_toward_color(events: &[Event], perspective: Color) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::SpiritTargetMove {
                    item: Item::Mana {
                        mana: Mana::Supermana,
                    },
                    from,
                    to,
                    ..
                } if Self::mana_move_toward_color(*from, *to, perspective)
            )
        })
    }

    fn events_spirit_move_supermana_onto_safe_drainer(
        events: &[Event],
        board_after: &Board,
        perspective: Color,
    ) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::SpiritTargetMove {
                    item: Item::Mana {
                        mana: Mana::Supermana,
                    },
                    ..
                }
            )
        }) && Self::own_drainer_carries_specific_mana_safely(
            board_after,
            perspective,
            Mana::Supermana,
        )
    }

    fn events_spirit_supermana_setup(
        events: &[Event],
        board_after: &Board,
        perspective: Color,
    ) -> bool {
        Self::events_spirit_move_supermana_toward_color(events, perspective)
            || Self::events_spirit_move_supermana_onto_safe_drainer(
                events,
                board_after,
                perspective,
            )
    }

    fn events_spirit_opponent_mana_setup(
        events: &[Event],
        board_after: &Board,
        perspective: Color,
    ) -> bool {
        Self::events_spirit_move_opponent_mana_toward_color(events, perspective)
            || Self::events_spirit_move_opponent_mana_onto_safe_drainer(
                events,
                board_after,
                perspective,
            )
    }

    fn events_spirit_scoring_mana_setup(
        events: &[Event],
        board_after: &Board,
        perspective: Color,
    ) -> bool {
        Self::events_spirit_move_own_mana_toward_color(events, perspective)
            || Self::events_spirit_supermana_setup(events, board_after, perspective)
            || Self::events_spirit_opponent_mana_setup(events, board_after, perspective)
    }

    fn interview_root_soft_priority(
        config: SmartSearchConfig,
        supermana_progress: bool,
        opponent_mana_progress: bool,
        safe_supermana_progress_steps: i32,
        safe_opponent_mana_progress_steps: i32,
        scores_supermana_this_turn: bool,
        scores_opponent_mana_this_turn: bool,
        own_drainer_vulnerable: bool,
        mana_handoff_to_opponent: bool,
        has_roundtrip: bool,
    ) -> i32 {
        if !config.enable_interview_soft_root_priors {
            return 0;
        }

        let mut score = 0i32;
        if scores_supermana_this_turn {
            score = score.saturating_add(config.interview_soft_supermana_score_bonus.max(0));
        } else if supermana_progress && !own_drainer_vulnerable {
            score = score
                .saturating_add(config.interview_soft_supermana_progress_bonus.max(0))
                .saturating_add(Self::root_progress_step_soft_bonus(
                    safe_supermana_progress_steps,
                    8,
                ));
        }

        if scores_opponent_mana_this_turn {
            score = score.saturating_add(config.interview_soft_opponent_mana_score_bonus.max(0));
        } else if opponent_mana_progress && !own_drainer_vulnerable {
            score = score
                .saturating_add(config.interview_soft_opponent_mana_progress_bonus.max(0))
                .saturating_add(Self::root_progress_step_soft_bonus(
                    safe_opponent_mana_progress_steps,
                    6,
                ));
        }

        if mana_handoff_to_opponent {
            score = score.saturating_sub(config.interview_soft_mana_handoff_penalty.max(0));
        }
        if has_roundtrip {
            score = score.saturating_sub(config.interview_soft_roundtrip_penalty.max(0));
        }

        score
    }

    fn root_progress_step_soft_bonus(steps: i32, per_step_bonus: i32) -> i32 {
        let unknown_steps = Config::BOARD_SIZE + 4;
        if steps >= unknown_steps || per_step_bonus <= 0 {
            return 0;
        }
        let clamped_steps = steps.clamp(0, Config::MONS_MOVES_PER_TURN);
        (Config::MONS_MOVES_PER_TURN - clamped_steps) * per_step_bonus
    }

    fn root_progress_steps_better(candidate_steps: i32, incumbent_steps: i32) -> bool {
        let unknown_steps = Config::BOARD_SIZE + 4;
        let candidate_known = candidate_steps < unknown_steps;
        let incumbent_known = incumbent_steps < unknown_steps;
        match (candidate_known, incumbent_known) {
            (true, true) => candidate_steps < incumbent_steps,
            (true, false) => true,
            _ => false,
        }
    }

    fn root_score_path_steps_better(candidate_steps: i32, incumbent_steps: i32) -> bool {
        let unknown_steps = Config::BOARD_SIZE * 3;
        let candidate_known = candidate_steps < unknown_steps;
        let incumbent_known = incumbent_steps < unknown_steps;
        match (candidate_known, incumbent_known) {
            (true, true) => candidate_steps < incumbent_steps,
            (true, false) => true,
            _ => false,
        }
    }

    fn forced_drainer_attack_fallback_candidates_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            SMART_FORCED_DRAINER_ATTACK_FALLBACK_NORMAL_CANDIDATES
        } else {
            SMART_FORCED_DRAINER_ATTACK_FALLBACK_FAST_CANDIDATES
        }
    }

    fn forced_drainer_attack_fallback_node_budget(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            SMART_FORCED_DRAINER_ATTACK_FALLBACK_NODE_BUDGET_NORMAL
        } else {
            SMART_FORCED_DRAINER_ATTACK_FALLBACK_NODE_BUDGET_FAST
        }
    }

    fn forced_drainer_attack_fallback_enum_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            SMART_FORCED_DRAINER_ATTACK_FALLBACK_ENUM_LIMIT_NORMAL
        } else {
            SMART_FORCED_DRAINER_ATTACK_FALLBACK_ENUM_LIMIT_FAST
        }
    }

    fn spirit_setup_fallback_candidates_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            8
        } else {
            4
        }
    }

    fn spirit_setup_fallback_enum_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            256
        } else {
            128
        }
    }

    fn safe_drainer_pickup_fallback_candidates_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            8
        } else {
            4
        }
    }

    fn spirit_score_fallback_candidates_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            8
        } else {
            4
        }
    }

    fn same_turn_score_window_fallback_candidates_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            8
        } else {
            4
        }
    }

    fn drainer_safety_fallback_candidates_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            8
        } else {
            4
        }
    }

    fn drainer_safety_fallback_enum_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            192
        } else {
            96
        }
    }

    fn child_exact_progress_fallback_candidates_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            6
        } else {
            4
        }
    }

    fn child_exact_progress_fallback_enum_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            192
        } else {
            96
        }
    }

    fn generic_root_fallback_enum_limit(config: SmartSearchConfig) -> usize {
        if config.depth >= 3 {
            24
        } else {
            12
        }
    }

    fn automove_start_input_options(config: SmartSearchConfig) -> SuggestedStartInputOptions {
        SuggestedStartInputOptions {
            include_mana_starts_with_potion_action: config
                .enable_mana_start_mix_with_potion_actions,
        }
    }

    fn opponent_awake_drainer_location(board: &Board, perspective: Color) -> Option<Location> {
        board.occupied().find_map(|(location, item)| {
            let mon = item.mon()?;
            if mon.color == perspective.other() && mon.kind == MonKind::Drainer && !mon.is_fainted()
            {
                Some(location)
            } else {
                None
            }
        })
    }

    fn min_steps_to_mystic_attack_source(from: Location, target: Location) -> i32 {
        target
            .reachable_by_mystic_action_ref()
            .iter()
            .map(|source| from.distance(source))
            .min()
            .unwrap_or(i32::MAX)
    }

    fn min_steps_to_demon_attack_source(from: Location, target: Location) -> i32 {
        target
            .reachable_by_demon_action_ref()
            .iter()
            .map(|source| from.distance(source))
            .min()
            .unwrap_or(i32::MAX)
    }

    fn find_potential_drainer_attacker_locations(
        game: &MonsGame,
        perspective: Color,
    ) -> Vec<Location> {
        let mut attackers = Vec::new();

        let Some(opponent_drainer_location) =
            Self::opponent_awake_drainer_location(&game.board, perspective)
        else {
            return attackers;
        };

        let remaining_mon_moves = (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0);
        if remaining_mon_moves <= 0 {
            return attackers;
        }

        let opponent_drainer_guarded = Self::is_location_guarded_by_angel(
            &game.board,
            perspective.other(),
            opponent_drainer_location,
        );
        let bomb_pickup_locations = game
            .board
            .occupied()
            .filter_map(|(location, item)| match item {
                Item::Consumable {
                    consumable: Consumable::BombOrPotion,
                } => Some(location),
                _ => None,
            })
            .collect::<Vec<_>>();

        for (location, item) in game.board.occupied() {
            let Some(mon) = item.mon() else {
                continue;
            };
            if mon.color != perspective || mon.is_fainted() {
                continue;
            }

            let has_bomb = matches!(
                item,
                Item::MonWithConsumable {
                    consumable: Consumable::Bomb,
                    ..
                }
            );
            if has_bomb && location.distance(&opponent_drainer_location) <= remaining_mon_moves + 3
            {
                attackers.push(location);
                continue;
            }

            if !opponent_drainer_guarded {
                let action_distance = match mon.kind {
                    MonKind::Mystic => {
                        Self::min_steps_to_mystic_attack_source(location, opponent_drainer_location)
                    }
                    MonKind::Demon => {
                        Self::min_steps_to_demon_attack_source(location, opponent_drainer_location)
                    }
                    _ => i32::MAX,
                };
                if action_distance <= remaining_mon_moves {
                    attackers.push(location);
                    continue;
                }
            }

            for bomb_location in &bomb_pickup_locations {
                let to_bomb = location.distance(bomb_location);
                if to_bomb > remaining_mon_moves {
                    continue;
                }
                let moves_after_pickup = remaining_mon_moves - to_bomb;
                if bomb_location.distance(&opponent_drainer_location) <= moves_after_pickup + 3 {
                    attackers.push(location);
                    break;
                }
            }
        }

        attackers
    }

    fn can_attempt_forced_drainer_attack_fallback(game: &MonsGame, perspective: Color) -> bool {
        if !game.player_can_move_mon() {
            return false;
        }
        !Self::find_potential_drainer_attacker_locations(game, perspective).is_empty()
    }

    fn find_awake_spirit_locations(game: &MonsGame, perspective: Color) -> Vec<Location> {
        game.board
            .occupied()
            .filter_map(|(location, item)| {
                let mon = item.mon()?;
                (mon.color == perspective && mon.kind == MonKind::Spirit && !mon.is_fainted())
                    .then_some(location)
            })
            .collect()
    }

    fn find_awake_drainer_locations(game: &MonsGame, perspective: Color) -> Vec<Location> {
        game.board
            .occupied()
            .filter_map(|(location, item)| {
                let mon = item.mon()?;
                (mon.color == perspective && mon.kind == MonKind::Drainer && !mon.is_fainted())
                    .then_some(location)
            })
            .collect()
    }

    fn find_awake_mon_locations(game: &MonsGame, perspective: Color) -> Vec<Location> {
        game.board
            .occupied()
            .filter_map(|(location, item)| {
                let mon = item.mon()?;
                (mon.color == perspective && !mon.is_fainted()).then_some(location)
            })
            .collect()
    }

    fn collect_targeted_spirit_setup_inputs(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        max_candidates: usize,
    ) -> Vec<LegalInputTransition> {
        let spirit_locations = Self::find_awake_spirit_locations(game, perspective);
        if spirit_locations.is_empty() || !game.player_can_use_action() {
            return Vec::new();
        }

        let start_options = Self::automove_start_input_options(config);
        let per_spirit_enum_limit = Self::spirit_setup_fallback_enum_limit(config);
        let mut setup_inputs = Vec::new();

        for spirit_loc in spirit_locations {
            if setup_inputs.len() >= max_candidates.max(1) {
                break;
            }

            let mut spirit_transitions = Vec::new();
            let mut partial_inputs = vec![Input::Location(spirit_loc)];
            let mut simulated_game = game.clone_for_simulation();
            Self::collect_legal_transitions(
                &mut simulated_game,
                &mut partial_inputs,
                &mut spirit_transitions,
                per_spirit_enum_limit,
                start_options,
            );
            spirit_transitions.sort_by(|a, b| a.inputs.cmp(&b.inputs));

            for transition in spirit_transitions {
                if setup_inputs.len() >= max_candidates.max(1) {
                    break;
                }
                if !Self::events_spirit_scoring_mana_setup(
                    &transition.events,
                    &transition.game.board,
                    perspective,
                ) {
                    continue;
                }
                setup_inputs.push(transition);
            }
        }

        setup_inputs
    }

    fn collect_targeted_spirit_score_inputs(
        game: &MonsGame,
        perspective: Color,
        max_candidates: usize,
        opponent_mana_only: bool,
    ) -> Vec<LegalInputTransition> {
        let spirit_locations = Self::find_awake_spirit_locations(game, perspective);
        if spirit_locations.is_empty() || !game.player_can_use_action() {
            return Vec::new();
        }

        let score_before = Self::score_for_color(game, perspective);
        let mut score_inputs = Vec::new();

        for spirit_loc in spirit_locations {
            if score_inputs.len() >= max_candidates.max(1) {
                break;
            }
            if matches!(game.board.square(spirit_loc), Square::MonBase { .. }) {
                continue;
            }

            for &target in spirit_loc.reachable_by_spirit_action_ref() {
                if score_inputs.len() >= max_candidates.max(1) {
                    break;
                }
                let Some(target_item) = game.board.item(target).copied() else {
                    continue;
                };
                if !Self::spirit_target_allowed_for_root_fallback(target_item) {
                    continue;
                }

                for &dest in target.nearby_locations_ref() {
                    if score_inputs.len() >= max_candidates.max(1) {
                        break;
                    }
                    if !Self::spirit_destination_allowed_for_root_fallback(
                        &game.board,
                        target_item,
                        dest,
                    ) {
                        continue;
                    }
                    let inputs = vec![
                        Input::Location(spirit_loc),
                        Input::Location(target),
                        Input::Location(dest),
                    ];
                    let Some((after_game, events)) =
                        Self::apply_inputs_for_search_with_events(game, &inputs)
                    else {
                        continue;
                    };
                    if !Self::events_include_spirit_target_move(&events) {
                        continue;
                    }
                    let after_same_turn_score_window_value = if after_game.active_color == perspective
                    {
                        exact_turn_summary(&after_game, perspective).same_turn_score_window_value
                    } else {
                        0
                    };
                    let scores_now = if opponent_mana_only {
                        Self::events_score_opponent_mana(&events, perspective)
                            || (after_game.active_color == perspective
                                && exact_turn_summary(&after_game, perspective)
                                    .safe_opponent_mana_progress)
                    } else {
                        Self::score_for_color(&after_game, perspective) > score_before
                            || after_same_turn_score_window_value > 0
                    };
                    if scores_now {
                        score_inputs.push(LegalInputTransition {
                            inputs,
                            game: after_game,
                            events,
                        });
                    }
                }
            }
        }

        score_inputs.sort_by(|a, b| a.inputs.cmp(&b.inputs));
        score_inputs
    }

    fn collect_targeted_same_turn_score_window_inputs(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        max_candidates: usize,
    ) -> Vec<LegalInputTransition> {
        if !game.player_can_move_mon() && !game.player_can_use_action() {
            return Vec::new();
        }

        let mut actor_locations = Self::find_awake_drainer_locations(game, perspective);
        actor_locations.extend(Self::find_awake_spirit_locations(game, perspective));
        if actor_locations.is_empty() {
            return Vec::new();
        }

        let start_options = Self::automove_start_input_options(config);
        let per_actor_enum_limit = Self::spirit_setup_fallback_enum_limit(config);
        let mut score_window_inputs = Vec::new();
        let mut seen_inputs = std::collections::HashSet::new();

        for actor_loc in actor_locations {
            let mut actor_transitions = Vec::new();
            let mut partial_inputs = vec![Input::Location(actor_loc)];
            let mut simulated_game = game.clone_for_simulation();
            Self::collect_legal_transitions(
                &mut simulated_game,
                &mut partial_inputs,
                &mut actor_transitions,
                per_actor_enum_limit,
                start_options,
            );
            actor_transitions.sort_by(|a, b| a.inputs.cmp(&b.inputs));

            for transition in actor_transitions {
                if transition.game.active_color != perspective {
                    continue;
                }
                if exact_turn_summary(&transition.game, perspective).same_turn_score_window_value
                    <= 0
                {
                    continue;
                }
                if seen_inputs.insert(transition.inputs.clone()) {
                    score_window_inputs.push(transition);
                }
            }
        }

        score_window_inputs
            .sort_by(|a, b| Self::compare_same_turn_score_window_transitions(a, b, perspective));
        if score_window_inputs.len() > max_candidates.max(1) {
            score_window_inputs.truncate(max_candidates.max(1));
        }
        score_window_inputs
    }

    fn compare_same_turn_score_window_transitions(
        a: &LegalInputTransition,
        b: &LegalInputTransition,
        perspective: Color,
    ) -> std::cmp::Ordering {
        let a_turn = exact_turn_summary(&a.game, perspective);
        let b_turn = exact_turn_summary(&b.game, perspective);

        b_turn
            .same_turn_score_window_value
            .cmp(&a_turn.same_turn_score_window_value)
            .then_with(|| {
                b_turn
                    .spirit_assisted_denial_value
                    .cmp(&a_turn.spirit_assisted_denial_value)
            })
            .then_with(|| {
                Self::events_include_spirit_target_move(&b.events)
                    .cmp(&Self::events_include_spirit_target_move(&a.events))
            })
            .then_with(|| a.inputs.cmp(&b.inputs))
    }

    fn collect_targeted_drainer_safety_inputs(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        max_candidates: usize,
        require_walk_safe: bool,
    ) -> Vec<LegalInputTransition> {
        let actor_locations = Self::find_awake_mon_locations(game, perspective);
        if actor_locations.is_empty() {
            return Vec::new();
        }

        let start_options = Self::automove_start_input_options(config);
        let per_actor_enum_limit = Self::drainer_safety_fallback_enum_limit(config);
        let mut safety_inputs = Vec::new();
        let mut seen_inputs = std::collections::HashSet::new();

        for actor_loc in actor_locations {
            if safety_inputs.len() >= max_candidates.max(1) {
                break;
            }

            let mut actor_transitions = Vec::new();
            let mut partial_inputs = vec![Input::Location(actor_loc)];
            let mut simulated_game = game.clone_for_simulation();
            Self::collect_legal_transitions(
                &mut simulated_game,
                &mut partial_inputs,
                &mut actor_transitions,
                per_actor_enum_limit,
                start_options,
            );
            actor_transitions.sort_by(|a, b| a.inputs.cmp(&b.inputs));

            for transition in actor_transitions {
                if safety_inputs.len() >= max_candidates.max(1) {
                    break;
                }
                if Self::is_own_drainer_vulnerable_next_turn(
                    &transition.game,
                    perspective,
                    config.enable_enhanced_drainer_vulnerability,
                ) {
                    continue;
                }
                if require_walk_safe
                    && Self::is_own_drainer_walk_vulnerable_next_turn(
                        &transition.game,
                        perspective,
                        config.enable_enhanced_drainer_vulnerability,
                    )
                {
                    continue;
                }
                if seen_inputs.insert(transition.inputs.clone()) {
                    safety_inputs.push(transition);
                }
            }
        }

        safety_inputs.sort_by(|a, b| a.inputs.cmp(&b.inputs));
        safety_inputs
    }

    fn transition_preserves_exact_progress(
        transition: &LegalInputTransition,
        perspective: Color,
        wanted_mana: Mana,
    ) -> bool {
        if matches!(wanted_mana, Mana::Supermana) {
            if Self::events_score_supermana(&transition.events)
                || Self::own_drainer_carries_specific_mana_safely(
                    &transition.game.board,
                    perspective,
                    Mana::Supermana,
                )
            {
                return true;
            }
            if transition.game.active_color == perspective {
                let exact_turn = exact_turn_summary(&transition.game, perspective);
                return exact_turn.safe_supermana_progress
                    || exact_turn.spirit_assisted_supermana_progress;
            }
            return false;
        }

        if wanted_mana == Mana::Regular(perspective.other()) {
            if Self::events_score_opponent_mana(&transition.events, perspective)
                || Self::own_drainer_carries_specific_mana_safely(
                    &transition.game.board,
                    perspective,
                    Mana::Regular(perspective.other()),
                )
            {
                return true;
            }
            if transition.game.active_color == perspective {
                let exact_turn = exact_turn_summary(&transition.game, perspective);
                return exact_turn.safe_opponent_mana_progress
                    || exact_turn.spirit_assisted_opponent_mana_progress
                    || exact_turn.spirit_assisted_denial;
            }
        }

        false
    }

    fn collect_targeted_exact_progress_inputs(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        max_candidates: usize,
        wanted_mana: Mana,
    ) -> Vec<LegalInputTransition> {
        if !game.player_can_move_mon() && !game.player_can_use_action() {
            return Vec::new();
        }

        let mut actor_locations = Self::find_awake_drainer_locations(game, perspective);
        actor_locations.extend(Self::find_awake_spirit_locations(game, perspective));
        if actor_locations.is_empty() {
            return Vec::new();
        }

        let start_options = Self::automove_start_input_options(config);
        let per_actor_enum_limit = Self::child_exact_progress_fallback_enum_limit(config);
        let mut progress_inputs = Vec::new();
        let mut seen_inputs = std::collections::HashSet::new();

        for actor_loc in actor_locations {
            if progress_inputs.len() >= max_candidates.max(1) {
                break;
            }

            let mut actor_transitions = Vec::new();
            let mut partial_inputs = vec![Input::Location(actor_loc)];
            let mut simulated_game = game.clone_for_simulation();
            Self::collect_legal_transitions(
                &mut simulated_game,
                &mut partial_inputs,
                &mut actor_transitions,
                per_actor_enum_limit,
                start_options,
            );
            actor_transitions.sort_by(|a, b| a.inputs.cmp(&b.inputs));

            for transition in actor_transitions {
                if progress_inputs.len() >= max_candidates.max(1) {
                    break;
                }
                if !Self::transition_preserves_exact_progress(&transition, perspective, wanted_mana)
                {
                    continue;
                }
                if seen_inputs.insert(transition.inputs.clone()) {
                    progress_inputs.push(transition);
                }
            }
        }

        progress_inputs.sort_by(|a, b| a.inputs.cmp(&b.inputs));
        progress_inputs
    }

    fn spirit_target_allowed_for_root_fallback(item: Item) -> bool {
        match item {
            Item::Mon { mon }
            | Item::MonWithMana { mon, .. }
            | Item::MonWithConsumable { mon, .. } => !mon.is_fainted(),
            Item::Mana { .. } | Item::Consumable { .. } => true,
        }
    }

    fn spirit_destination_allowed_for_root_fallback(
        board: &Board,
        target_item: Item,
        destination: Location,
    ) -> bool {
        let destination_item = board.item(destination).copied();
        let destination_square = board.square(destination);
        let target_mon = target_item.mon().copied();
        let target_mana = target_item.mana().copied();

        let valid_destination = match destination_item {
            Some(Item::Mon {
                mon: destination_mon,
            }) => match target_item {
                Item::Mon { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. } => {
                    false
                }
                Item::Mana { .. } => {
                    destination_mon.kind == MonKind::Drainer && !destination_mon.is_fainted()
                }
                Item::Consumable {
                    consumable: Consumable::BombOrPotion,
                } => true,
                Item::Consumable { .. } => false,
            },
            Some(Item::Mana { .. }) => {
                matches!(target_mon, Some(mon) if mon.kind == MonKind::Drainer && !mon.is_fainted())
            }
            Some(Item::MonWithMana { .. }) | Some(Item::MonWithConsumable { .. }) => {
                matches!(
                    target_item,
                    Item::Consumable {
                        consumable: Consumable::BombOrPotion,
                    }
                )
            }
            Some(Item::Consumable {
                consumable: Consumable::BombOrPotion,
            }) => matches!(
                target_item,
                Item::Mon { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. }
            ),
            Some(Item::Consumable { .. }) => false,
            None => true,
        };

        if !valid_destination {
            return false;
        }

        match destination_square {
            Square::Regular
            | Square::ConsumableBase
            | Square::ManaBase { .. }
            | Square::ManaPool { .. } => true,
            Square::SupermanaBase => {
                target_mana == Some(Mana::Supermana)
                    || (target_mana.is_none()
                        && matches!(target_mon.map(|mon| mon.kind), Some(MonKind::Drainer)))
            }
            Square::MonBase { kind, color } => {
                matches!(target_mon, Some(mon) if mon.kind == kind && mon.color == color)
                    && target_mana.is_none()
                    && target_item.consumable().is_none()
            }
        }
    }

    fn collect_targeted_safe_drainer_pickup_inputs(
        game: &MonsGame,
        perspective: Color,
        max_candidates: usize,
        wanted_mana: Mana,
    ) -> Vec<LegalInputTransition> {
        let drainer_locations = Self::find_awake_drainer_locations(game, perspective);
        if drainer_locations.is_empty() || !game.player_can_move_mon() {
            return Vec::new();
        }

        let mut pickup_inputs = Vec::new();
        let mut seen_inputs = std::collections::HashSet::new();

        for drainer_loc in drainer_locations {
            if pickup_inputs.len() >= max_candidates.max(1) {
                break;
            }

            let Some(path) =
                exact_secure_specific_mana_path_from(game, perspective, drainer_loc, wanted_mana)
            else {
                continue;
            };
            if path.is_empty() {
                continue;
            }

            let mut inputs = Vec::with_capacity(path.len() + 1);
            inputs.push(Input::Location(drainer_loc));
            inputs.extend(path.into_iter().map(Input::Location));
            let Some((after_game, events)) =
                Self::apply_inputs_for_search_with_events(game, &inputs)
            else {
                continue;
            };
            let picked_wanted_mana = match wanted_mana {
                Mana::Supermana => Self::events_pickup_supermana(&events),
                Mana::Regular(owner) if owner == perspective.other() => {
                    Self::events_pickup_opponent_mana(&events, perspective)
                }
                _ => false,
            };
            if picked_wanted_mana
                && (Self::own_drainer_carries_specific_mana_safely(
                    &after_game.board,
                    perspective,
                    wanted_mana,
                ) || match wanted_mana {
                    Mana::Supermana => Self::events_score_supermana(&events),
                    Mana::Regular(owner) if owner == perspective.other() => {
                        Self::events_score_opponent_mana(&events, perspective)
                    }
                    _ => false,
                })
                && seen_inputs.insert(inputs.clone())
            {
                pickup_inputs.push(LegalInputTransition {
                    inputs,
                    game: after_game,
                    events,
                });
            }
        }

        pickup_inputs.sort_by(|a, b| a.inputs.cmp(&b.inputs));
        pickup_inputs
    }

    fn collect_targeted_drainer_attack_inputs(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        max_candidates: usize,
    ) -> Vec<LegalInputTransition> {
        let attacker_locations = Self::find_potential_drainer_attacker_locations(game, perspective);
        if attacker_locations.is_empty() {
            return Vec::new();
        }
        let attacker_set: std::collections::HashSet<Location> =
            attacker_locations.into_iter().collect();

        let mut memo_true = U64HashSet::default();
        let mut attack_inputs = Vec::new();
        let base_budget = Self::forced_drainer_attack_fallback_node_budget(config);
        let mut continuation_budget = base_budget * 2;
        let base_enum = Self::forced_drainer_attack_fallback_enum_limit(config);
        let enum_limit = base_enum * 2;
        let start_options = Self::automove_start_input_options(config);

        let mut root_transitions =
            Self::enumerate_legal_transitions(game, usize::MAX, start_options);
        root_transitions.retain(|transition| {
            matches!(
                transition.inputs.first(),
                Some(Input::Location(loc)) if attacker_set.contains(loc)
            )
        });
        if root_transitions.len() > enum_limit {
            root_transitions.truncate(enum_limit);
        }

        for transition in root_transitions {
            if attack_inputs.len() >= max_candidates.max(1) {
                break;
            }

            if Self::events_include_opponent_drainer_fainted(&transition.events, perspective) {
                attack_inputs.push(transition);
                continue;
            }
            if transition.game.active_color != perspective {
                continue;
            }
            if Self::can_attack_opponent_drainer_before_turn_ends(
                &transition.game,
                perspective,
                enum_limit,
                start_options,
                &mut continuation_budget,
                &mut memo_true,
            ) {
                attack_inputs.push(transition);
            }
        }

        attack_inputs
    }

    fn collect_per_mon_drainer_attack_inputs(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        max_candidates: usize,
    ) -> Vec<LegalInputTransition> {
        let attacker_locations = Self::find_potential_drainer_attacker_locations(game, perspective);
        if attacker_locations.is_empty() {
            return Vec::new();
        }

        let mut memo_true = U64HashSet::default();
        let mut attack_inputs = Vec::new();
        let base_budget = Self::forced_drainer_attack_fallback_node_budget(config);
        let mut continuation_budget = base_budget * 2;
        let per_mon_enum_limit: usize = 200;
        let start_options = Self::automove_start_input_options(config);

        for &attacker_loc in &attacker_locations {
            if attack_inputs.len() >= max_candidates.max(1) {
                break;
            }

            let mut per_mon_transitions = Vec::new();
            let mut partial_inputs = vec![Input::Location(attacker_loc)];
            let mut simulated_game = game.clone_for_simulation();
            Self::collect_legal_transitions(
                &mut simulated_game,
                &mut partial_inputs,
                &mut per_mon_transitions,
                per_mon_enum_limit,
                start_options,
            );
            per_mon_transitions.sort_by(|a, b| a.inputs.cmp(&b.inputs));

            for transition in per_mon_transitions {
                if attack_inputs.len() >= max_candidates.max(1) {
                    break;
                }

                if Self::events_include_opponent_drainer_fainted(&transition.events, perspective) {
                    attack_inputs.push(transition);
                    continue;
                }
                if transition.game.active_color != perspective {
                    continue;
                }
                if continuation_budget == 0 {
                    continue;
                }
                if Self::can_attack_opponent_drainer_before_turn_ends(
                    &transition.game,
                    perspective,
                    per_mon_enum_limit,
                    start_options,
                    &mut continuation_budget,
                    &mut memo_true,
                ) {
                    attack_inputs.push(transition);
                }
            }
        }

        attack_inputs
    }

    fn collect_forced_drainer_attack_inputs(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        max_candidates: usize,
    ) -> Vec<LegalInputTransition> {
        let mut memo_true = U64HashSet::default();
        let mut attack_inputs = Vec::new();
        let mut continuation_budget = Self::forced_drainer_attack_fallback_node_budget(config);
        let enum_limit = Self::forced_drainer_attack_fallback_enum_limit(config);
        let start_options = Self::automove_start_input_options(config);
        let mut root_transitions =
            Self::enumerate_legal_transitions(game, usize::MAX, start_options);
        if root_transitions.len() > enum_limit {
            root_transitions.truncate(enum_limit);
        }
        for transition in root_transitions {
            if attack_inputs.len() >= max_candidates.max(1) {
                break;
            }

            if Self::events_include_opponent_drainer_fainted(&transition.events, perspective) {
                attack_inputs.push(transition);
                continue;
            }
            if transition.game.active_color != perspective {
                continue;
            }
            if Self::can_attack_opponent_drainer_before_turn_ends(
                &transition.game,
                perspective,
                enum_limit,
                start_options,
                &mut continuation_budget,
                &mut memo_true,
            ) {
                attack_inputs.push(transition);
            }
        }

        attack_inputs
    }

    fn can_attack_opponent_drainer_before_turn_ends(
        game: &MonsGame,
        perspective: Color,
        _enum_limit: usize,
        _start_options: SuggestedStartInputOptions,
        continuation_budget: &mut usize,
        memo_true: &mut U64HashSet,
    ) -> bool {
        if game.active_color != perspective || *continuation_budget == 0 {
            return false;
        }
        let state_hash = Self::search_state_hash(game);
        if memo_true.contains(&state_hash) {
            return true;
        }
        *continuation_budget = continuation_budget.saturating_sub(1);
        let can_attack = can_attack_opponent_drainer_this_turn(game, perspective);
        if can_attack {
            memo_true.insert(state_hash);
        }
        can_attack
    }

    fn ranked_root_moves(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
    ) -> Vec<ScoredRootMove> {
        let mut candidates = Vec::new();
        let own_drainer_vulnerable_before = if config.enable_move_class_coverage {
            Self::is_own_drainer_vulnerable_next_turn(
                game,
                perspective,
                config.enable_enhanced_drainer_vulnerability,
            )
        } else {
            false
        };

        let start_options = Self::automove_start_input_options(config);
        let effective_enum_limit = if config.enable_drainer_attack_priority_enum
            && config.drainer_attack_priority_enum_boost > 0
            && Self::can_attempt_forced_drainer_attack_fallback(game, perspective)
        {
            config.root_enum_limit + config.drainer_attack_priority_enum_boost
        } else {
            config.root_enum_limit
        };

        let mut root_transitions = if config.enable_drainer_attack_priority_enum {
            let attacker_locs = Self::find_potential_drainer_attacker_locations(game, perspective);
            if attacker_locs.is_empty() {
                Self::enumerate_legal_transitions(game, effective_enum_limit, start_options)
            } else {
                Self::enumerate_legal_transitions_with_priority(
                    game,
                    effective_enum_limit,
                    start_options,
                    &attacker_locs,
                )
            }
        } else {
            Self::enumerate_legal_transitions(game, effective_enum_limit, start_options)
        };

        let own_drainer_walk_vulnerable_before = if config.enable_move_class_coverage
            && config.enable_walk_threat_prefilter
            && !own_drainer_vulnerable_before
        {
            Self::is_own_drainer_walk_vulnerable_next_turn(
                game,
                perspective,
                config.enable_enhanced_drainer_vulnerability,
            )
        } else {
            false
        };

        if config.enable_root_drainer_safety_prefilter
            && own_drainer_vulnerable_before
            && !root_transitions.iter().any(|transition| {
                !Self::is_own_drainer_vulnerable_next_turn(
                    &transition.game,
                    perspective,
                    config.enable_enhanced_drainer_vulnerability,
                )
            })
        {
            let fallback_limit = Self::drainer_safety_fallback_candidates_limit(config);
            let fallback_inputs = Self::collect_targeted_drainer_safety_inputs(
                game,
                perspective,
                config,
                fallback_limit,
                false,
            );
            if !fallback_inputs.is_empty() {
                let mut seen_inputs = root_transitions
                    .iter()
                    .map(|transition| transition.inputs.clone())
                    .collect::<std::collections::HashSet<_>>();
                for transition in fallback_inputs {
                    if seen_inputs.insert(transition.inputs.clone()) {
                        root_transitions.push(transition);
                    }
                }
            }
        }
        if config.enable_walk_threat_prefilter
            && own_drainer_walk_vulnerable_before
            && !root_transitions.iter().any(|transition| {
                !Self::is_own_drainer_vulnerable_next_turn(
                    &transition.game,
                    perspective,
                    config.enable_enhanced_drainer_vulnerability,
                ) && !Self::is_own_drainer_walk_vulnerable_next_turn(
                    &transition.game,
                    perspective,
                    config.enable_enhanced_drainer_vulnerability,
                )
            })
        {
            let fallback_limit = Self::drainer_safety_fallback_candidates_limit(config);
            let fallback_inputs = Self::collect_targeted_drainer_safety_inputs(
                game,
                perspective,
                config,
                fallback_limit,
                true,
            );
            if !fallback_inputs.is_empty() {
                let mut seen_inputs = root_transitions
                    .iter()
                    .map(|transition| transition.inputs.clone())
                    .collect::<std::collections::HashSet<_>>();
                for transition in fallback_inputs {
                    if seen_inputs.insert(transition.inputs.clone()) {
                        root_transitions.push(transition);
                    }
                }
            }
        }

        let exact_turn_before = exact_turn_summary(game, perspective);
        let exact_spirit_setup_gain_before = exact_strategic_analysis(game)
            .color_summary(perspective)
            .spirit
            .next_turn_setup_gain;
        if exact_turn_before.safe_supermana_progress
            && !root_transitions.iter().any(|transition| {
                Self::events_pickup_supermana(&transition.events)
                    && Self::own_drainer_carries_specific_mana_safely(
                        &transition.game.board,
                        perspective,
                        Mana::Supermana,
                    )
            })
        {
            let fallback_limit = Self::safe_drainer_pickup_fallback_candidates_limit(config);
            let fallback_inputs = Self::collect_targeted_safe_drainer_pickup_inputs(
                game,
                perspective,
                fallback_limit,
                Mana::Supermana,
            );
            if !fallback_inputs.is_empty() {
                let mut seen_inputs = root_transitions
                    .iter()
                    .map(|transition| transition.inputs.clone())
                    .collect::<std::collections::HashSet<_>>();
                for transition in fallback_inputs {
                    if seen_inputs.insert(transition.inputs.clone()) {
                        root_transitions.push(transition);
                    }
                }
            }
        }
        if exact_turn_before.safe_opponent_mana_progress
            && !root_transitions.iter().any(|transition| {
                Self::events_pickup_opponent_mana(&transition.events, perspective)
                    && Self::own_drainer_carries_specific_mana_safely(
                        &transition.game.board,
                        perspective,
                        Mana::Regular(perspective.other()),
                    )
            })
        {
            let fallback_limit = Self::safe_drainer_pickup_fallback_candidates_limit(config);
            let fallback_inputs = Self::collect_targeted_safe_drainer_pickup_inputs(
                game,
                perspective,
                fallback_limit,
                Mana::Regular(perspective.other()),
            );
            if !fallback_inputs.is_empty() {
                let mut seen_inputs = root_transitions
                    .iter()
                    .map(|transition| transition.inputs.clone())
                    .collect::<std::collections::HashSet<_>>();
                for transition in fallback_inputs {
                    if seen_inputs.insert(transition.inputs.clone()) {
                        root_transitions.push(transition);
                    }
                }
            }
        }
        if exact_turn_before.spirit_assisted_score
            && !root_transitions.iter().any(|transition| {
                transition.game.active_color == perspective
                    && exact_turn_summary(&transition.game, perspective).same_turn_score_window_value
                        > 0
            })
        {
            let fallback_limit = Self::same_turn_score_window_fallback_candidates_limit(config);
            let fallback_inputs = Self::collect_targeted_same_turn_score_window_inputs(
                game,
                perspective,
                config,
                fallback_limit,
            );
            if !fallback_inputs.is_empty() {
                let mut seen_inputs = root_transitions
                    .iter()
                    .map(|transition| transition.inputs.clone())
                    .collect::<std::collections::HashSet<_>>();
                for transition in fallback_inputs {
                    if seen_inputs.insert(transition.inputs.clone()) {
                        root_transitions.push(transition);
                    }
                }
            }
        }
        if exact_turn_before.spirit_assisted_denial
            && !root_transitions.iter().any(|transition| {
                Self::events_include_spirit_target_move(&transition.events)
                    && Self::events_score_opponent_mana(&transition.events, perspective)
            })
        {
            let fallback_limit = Self::spirit_score_fallback_candidates_limit(config);
            let fallback_inputs =
                Self::collect_targeted_spirit_score_inputs(game, perspective, fallback_limit, true);
            if !fallback_inputs.is_empty() {
                let mut seen_inputs = root_transitions
                    .iter()
                    .map(|transition| transition.inputs.clone())
                    .collect::<std::collections::HashSet<_>>();
                for transition in fallback_inputs {
                    if seen_inputs.insert(transition.inputs.clone()) {
                        root_transitions.push(transition);
                    }
                }
            }
        }
        if (config.enable_interview_hard_spirit_deploy
            || config.enable_root_spirit_development_pref)
            && (Self::should_prefer_spirit_development(game, perspective)
                || exact_spirit_setup_gain_before > 0
                || exact_turn_before.spirit_assisted_supermana_progress
                || exact_turn_before.spirit_assisted_opponent_mana_progress)
            && !root_transitions.iter().any(|transition| {
                Self::events_spirit_scoring_mana_setup(
                    &transition.events,
                    &transition.game.board,
                    perspective,
                )
            })
        {
            let fallback_limit = Self::spirit_setup_fallback_candidates_limit(config);
            let fallback_inputs = Self::collect_targeted_spirit_setup_inputs(
                game,
                perspective,
                config,
                fallback_limit,
            );
            if !fallback_inputs.is_empty() {
                let mut seen_inputs = root_transitions
                    .iter()
                    .map(|transition| transition.inputs.clone())
                    .collect::<std::collections::HashSet<_>>();
                for transition in fallback_inputs {
                    if seen_inputs.insert(transition.inputs.clone()) {
                        root_transitions.push(transition);
                    }
                }
            }
        }

        for transition in root_transitions {
            if let Some(candidate) = Self::build_scored_root_move_from_transition(
                game,
                perspective,
                config,
                own_drainer_vulnerable_before,
                transition.inputs,
                transition.game,
                transition.events,
            ) {
                candidates.push(candidate);
            }
        }

        if candidates.is_empty() {
            let fallback_limit = Self::generic_root_fallback_enum_limit(config);
            let fallback_transitions =
                Self::enumerate_legal_transitions(game, fallback_limit, start_options);
            for transition in fallback_transitions {
                if let Some(candidate) = Self::build_scored_root_move_from_transition(
                    game,
                    perspective,
                    config,
                    own_drainer_vulnerable_before,
                    transition.inputs,
                    transition.game,
                    transition.events,
                ) {
                    candidates.push(candidate);
                }
            }
        }

        Self::sort_root_candidates_by_search_priority(candidates.as_mut_slice());
        let mut has_winning_candidate = candidates
            .iter()
            .any(|candidate| candidate.wins_immediately);
        let mut forced_turn_attack_inputs: Option<std::collections::HashSet<Vec<Input>>> = None;
        if config.enable_forced_drainer_attack
            && config.enable_forced_drainer_attack_fallback
            && !has_winning_candidate
            && !candidates
                .iter()
                .any(|candidate| candidate.attacks_opponent_drainer)
            && Self::can_attempt_forced_drainer_attack_fallback(game, perspective)
        {
            let fallback_limit = Self::forced_drainer_attack_fallback_candidates_limit(config);
            let fallback_inputs = if config.enable_per_mon_drainer_attack_fallback {
                Self::collect_per_mon_drainer_attack_inputs(
                    game,
                    perspective,
                    config,
                    fallback_limit,
                )
            } else if config.enable_targeted_drainer_attack_fallback {
                Self::collect_targeted_drainer_attack_inputs(
                    game,
                    perspective,
                    config,
                    fallback_limit,
                )
            } else {
                Self::collect_forced_drainer_attack_inputs(
                    game,
                    perspective,
                    config,
                    fallback_limit,
                )
            };

            if !fallback_inputs.is_empty() {
                let forced_inputs = fallback_inputs
                    .iter()
                    .map(|transition| transition.inputs.clone())
                    .collect::<std::collections::HashSet<_>>();
                let mut seen_inputs = candidates
                    .iter()
                    .map(|candidate| candidate.inputs.clone())
                    .collect::<std::collections::HashSet<_>>();

                for transition in fallback_inputs {
                    if !seen_inputs.insert(transition.inputs.clone()) {
                        continue;
                    }
                    if let Some(candidate) = Self::build_scored_root_move_from_transition(
                        game,
                        perspective,
                        config,
                        own_drainer_vulnerable_before,
                        transition.inputs,
                        transition.game,
                        transition.events,
                    ) {
                        candidates.push(candidate);
                    }
                }

                Self::sort_root_candidates_by_search_priority(candidates.as_mut_slice());
                has_winning_candidate = candidates
                    .iter()
                    .any(|candidate| candidate.wins_immediately);
                forced_turn_attack_inputs = Some(forced_inputs);
            }
        }

        let should_filter_to_attacks = if config.enable_drainer_attack_full_pool {
            false
        } else if config.enable_conditional_forced_drainer_attack {
            let (my_score, opponent_score) = if perspective == Color::White {
                (game.white_score, game.black_score)
            } else {
                (game.black_score, game.white_score)
            };
            my_score <= opponent_score + config.conditional_forced_attack_score_margin
        } else {
            true
        };

        if config.enable_forced_drainer_attack
            && should_filter_to_attacks
            && !has_winning_candidate
            && candidates
                .iter()
                .any(|candidate| candidate.attacks_opponent_drainer)
        {
            candidates.retain(|candidate| candidate.attacks_opponent_drainer);
        } else if config.enable_forced_drainer_attack && should_filter_to_attacks {
            if let Some(forced_inputs) = forced_turn_attack_inputs {
                candidates.retain(|candidate| forced_inputs.contains(&candidate.inputs));
            }
        }
        if candidates.len() > config.root_branch_limit {
            if config.enable_move_class_coverage {
                candidates = Self::truncate_root_candidates_with_class_coverage(
                    candidates,
                    config.root_branch_limit,
                    config.enable_strict_tactical_class_coverage,
                );
            } else {
                candidates.truncate(config.root_branch_limit);
            }
        }
        candidates
    }

    fn is_better_tactical_root_candidate(
        candidate: &ScoredRootMove,
        incumbent: &ScoredRootMove,
    ) -> bool {
        if candidate.wins_immediately != incumbent.wins_immediately {
            return candidate.wins_immediately;
        }
        if candidate.attacks_opponent_drainer != incumbent.attacks_opponent_drainer {
            return candidate.attacks_opponent_drainer;
        }
        if candidate.own_drainer_vulnerable != incumbent.own_drainer_vulnerable {
            return !candidate.own_drainer_vulnerable;
        }
        if candidate.classes.immediate_score != incumbent.classes.immediate_score {
            return candidate.classes.immediate_score;
        }
        if candidate.scores_supermana_this_turn != incumbent.scores_supermana_this_turn {
            return candidate.scores_supermana_this_turn;
        }
        if candidate.scores_opponent_mana_this_turn != incumbent.scores_opponent_mana_this_turn {
            return candidate.scores_opponent_mana_this_turn;
        }
        if candidate.safe_supermana_pickup_now != incumbent.safe_supermana_pickup_now {
            return candidate.safe_supermana_pickup_now;
        }
        if candidate.safe_opponent_mana_pickup_now != incumbent.safe_opponent_mana_pickup_now {
            return candidate.safe_opponent_mana_pickup_now;
        }
        if candidate.same_turn_score_window_value != incumbent.same_turn_score_window_value {
            return candidate.same_turn_score_window_value > incumbent.same_turn_score_window_value;
        }
        if candidate.spirit_same_turn_score_setup_now != incumbent.spirit_same_turn_score_setup_now
        {
            return candidate.spirit_same_turn_score_setup_now;
        }
        if candidate.spirit_own_mana_setup_now != incumbent.spirit_own_mana_setup_now {
            return candidate.spirit_own_mana_setup_now;
        }
        if candidate.spirit_own_mana_setup_now
            && incumbent.spirit_own_mana_setup_now
            && candidate.supermana_progress
            && incumbent.supermana_progress
            && candidate.safe_supermana_progress_steps != incumbent.safe_supermana_progress_steps
        {
            return Self::root_progress_steps_better(
                candidate.safe_supermana_progress_steps,
                incumbent.safe_supermana_progress_steps,
            );
        }
        if candidate.spirit_own_mana_setup_now
            && incumbent.spirit_own_mana_setup_now
            && candidate.opponent_mana_progress
            && incumbent.opponent_mana_progress
            && candidate.safe_opponent_mana_progress_steps
                != incumbent.safe_opponent_mana_progress_steps
        {
            return Self::root_progress_steps_better(
                candidate.safe_opponent_mana_progress_steps,
                incumbent.safe_opponent_mana_progress_steps,
            );
        }
        if candidate.spirit_own_mana_setup_now
            && incumbent.spirit_own_mana_setup_now
            && candidate.score_path_best_steps != incumbent.score_path_best_steps
        {
            return Self::root_score_path_steps_better(
                candidate.score_path_best_steps,
                incumbent.score_path_best_steps,
            );
        }
        if candidate.supermana_progress != incumbent.supermana_progress {
            return candidate.supermana_progress;
        }
        if candidate.supermana_progress
            && incumbent.supermana_progress
            && candidate.safe_supermana_progress_steps != incumbent.safe_supermana_progress_steps
        {
            return Self::root_progress_steps_better(
                candidate.safe_supermana_progress_steps,
                incumbent.safe_supermana_progress_steps,
            );
        }
        if candidate.opponent_mana_progress != incumbent.opponent_mana_progress {
            return candidate.opponent_mana_progress;
        }
        if candidate.opponent_mana_progress
            && incumbent.opponent_mana_progress
            && candidate.safe_opponent_mana_progress_steps
                != incumbent.safe_opponent_mana_progress_steps
        {
            return Self::root_progress_steps_better(
                candidate.safe_opponent_mana_progress_steps,
                incumbent.safe_opponent_mana_progress_steps,
            );
        }
        if candidate.mana_handoff_to_opponent != incumbent.mana_handoff_to_opponent {
            return !candidate.mana_handoff_to_opponent;
        }
        if candidate.has_roundtrip != incumbent.has_roundtrip {
            return !candidate.has_roundtrip;
        }
        if candidate.spirit_development != incumbent.spirit_development {
            return candidate.spirit_development;
        }
        if candidate.efficiency != incumbent.efficiency {
            return candidate.efficiency > incumbent.efficiency;
        }
        if candidate.heuristic != incumbent.heuristic {
            return candidate.heuristic > incumbent.heuristic;
        }
        false
    }

    fn compare_tactical_root_candidates(
        candidate: &ScoredRootMove,
        incumbent: &ScoredRootMove,
    ) -> std::cmp::Ordering {
        if Self::is_better_tactical_root_candidate(candidate, incumbent) {
            std::cmp::Ordering::Less
        } else if Self::is_better_tactical_root_candidate(incumbent, candidate) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }

    fn sort_root_candidates_by_search_priority(candidates: &mut [ScoredRootMove]) {
        candidates.sort_by(|a, b| {
            b.heuristic
                .cmp(&a.heuristic)
                .then_with(|| Self::compare_tactical_root_candidates(a, b))
                .then_with(|| a.inputs.cmp(&b.inputs))
        });
    }

    fn child_class_priority_score(classes: MoveClassFlags) -> i32 {
        let mut score = 0;
        if classes.immediate_score {
            score += 1_000;
        }
        if classes.drainer_attack {
            score += 700;
        }
        if classes.drainer_safety_recover {
            score += 500;
        }
        if classes.carrier_progress {
            score += 220;
        }
        if classes.material {
            score += 80;
        }
        score
    }

    fn compare_ranked_child_entries(
        a: &(i32, RankedChildState),
        b: &(i32, RankedChildState),
        maximizing: bool,
    ) -> std::cmp::Ordering {
        let heuristic_cmp = if maximizing {
            b.0.cmp(&a.0)
        } else {
            a.0.cmp(&b.0)
        };
        heuristic_cmp
            .then_with(|| b.1.ordering_efficiency.cmp(&a.1.ordering_efficiency))
            .then_with(|| {
                Self::child_class_priority_score(b.1.classes)
                    .cmp(&Self::child_class_priority_score(a.1.classes))
            })
            .then_with(|| b.1.hash.cmp(&a.1.hash))
    }

    fn is_child_search_priority_class(classes: MoveClassFlags) -> bool {
        classes.is_tactical_priority() || classes.carrier_progress
    }

    fn is_exact_child_continuation_candidate(
        ordering_efficiency: i32,
        classes: MoveClassFlags,
    ) -> bool {
        ordering_efficiency > 0 && !classes.material
    }

    fn is_child_search_priority_candidate(state: &RankedChildState) -> bool {
        Self::is_child_search_priority_class(state.classes)
            || Self::is_exact_child_continuation_candidate(state.ordering_efficiency, state.classes)
    }

    fn is_quiet_reduction_candidate(
        ordering_efficiency: i32,
        tactical_extension_trigger: bool,
        classes: MoveClassFlags,
    ) -> bool {
        !classes.material
            && ordering_efficiency <= 0
            && !tactical_extension_trigger
            && !classes.is_tactical_priority()
            && !classes.carrier_progress
    }

    fn is_selective_extension_candidate(
        tactical_extension_trigger: bool,
        ordering_efficiency: i32,
        classes: MoveClassFlags,
    ) -> bool {
        tactical_extension_trigger
            || (ordering_efficiency > 0 && !classes.quiet && !classes.material)
    }

    fn child_score_within_coverage_margin(
        score: i32,
        reference_score: i32,
        maximizing: bool,
    ) -> bool {
        let margin = SMART_MOVE_CLASS_CHILD_SCORE_MARGIN.max(0);
        if maximizing {
            score.saturating_add(margin) >= reference_score
        } else {
            score <= reference_score.saturating_add(margin)
        }
    }

    fn truncate_child_states_with_coverage(
        scored_states: Vec<(i32, RankedChildState)>,
        limit: usize,
        maximizing: bool,
        strict_guarantees: bool,
    ) -> Vec<(i32, RankedChildState)> {
        if scored_states.len() <= limit || limit == 0 {
            return scored_states;
        }

        let cutoff_score = scored_states[limit - 1].0;
        let preserve_index =
            scored_states
                .iter()
                .enumerate()
                .skip(limit)
                .find_map(|(index, (score, state))| {
                    if !Self::is_child_search_priority_candidate(state) {
                        return None;
                    }
                    if strict_guarantees
                        || Self::child_score_within_coverage_margin(
                            *score,
                            cutoff_score,
                            maximizing,
                        )
                    {
                        Some(index)
                    } else {
                        None
                    }
                });

        let Some(preserve_index) = preserve_index else {
            return scored_states.into_iter().take(limit).collect();
        };

        let mut selected = vec![false; scored_states.len()];
        selected[preserve_index] = true;
        let mut selected_count = 1usize;
        for index in 0..scored_states.len() {
            if selected_count >= limit {
                break;
            }
            if selected[index] {
                continue;
            }
            selected[index] = true;
            selected_count += 1;
        }

        scored_states
            .into_iter()
            .enumerate()
            .filter_map(|(index, entry)| selected[index].then_some(entry))
            .collect()
    }

    fn has_exact_frontier_tactical_potential(game: &MonsGame) -> bool {
        let active_color = game.active_color;
        let active_summary = exact_strategic_analysis(game).color_summary(active_color);
        let turn_summary = exact_turn_summary(game, active_color);

        active_summary.immediate_window.best_score > 0
            || turn_summary.can_attack_opponent_drainer
            || turn_summary.safe_supermana_progress
            || turn_summary.safe_opponent_mana_progress
            || turn_summary.spirit_assisted_supermana_progress
            || turn_summary.spirit_assisted_opponent_mana_progress
            || turn_summary.spirit_assisted_score
            || turn_summary.spirit_assisted_denial
    }

    fn compare_ranked_root_indices(
        root_moves: &[ScoredRootMove],
        a: (usize, i32),
        b: (usize, i32),
    ) -> std::cmp::Ordering {
        b.1.cmp(&a.1)
            .then_with(|| {
                Self::compare_tactical_root_candidates(&root_moves[a.0], &root_moves[b.0])
            })
            .then_with(|| a.0.cmp(&b.0))
    }

    fn root_scout_progress_bonus(candidate: &ScoredRootMove) -> i32 {
        let mut bonus = 0i32;

        if candidate.supermana_progress
            && !candidate.scores_supermana_this_turn
            && !candidate.safe_supermana_pickup_now
            && !candidate.spirit_same_turn_score_setup_now
            && !candidate.spirit_own_mana_setup_now
        {
            bonus = bonus
                .saturating_add(520)
                .saturating_add(Self::root_progress_step_soft_bonus(
                    candidate.safe_supermana_progress_steps,
                    48,
                ));
        }

        if candidate.opponent_mana_progress
            && !candidate.scores_opponent_mana_this_turn
            && !candidate.safe_opponent_mana_pickup_now
            && !candidate.spirit_same_turn_score_setup_now
            && !candidate.spirit_own_mana_setup_now
        {
            bonus = bonus
                .saturating_add(480)
                .saturating_add(Self::root_progress_step_soft_bonus(
                    candidate.safe_opponent_mana_progress_steps,
                    40,
                ));
        }

        bonus
    }

    fn root_focus_scout_score(candidate: &ScoredRootMove) -> i32 {
        candidate
            .heuristic
            .saturating_add(candidate.efficiency / 2)
            .saturating_add(Self::root_scout_progress_bonus(candidate))
    }

    fn reorder_root_moves_by_ranked_indices(
        root_moves: Vec<ScoredRootMove>,
        ranked_indices: &[(usize, i32)],
    ) -> Vec<ScoredRootMove> {
        let mut owned_root_moves = root_moves.into_iter().map(Some).collect::<Vec<_>>();
        let mut ordered_root_moves = Vec::with_capacity(owned_root_moves.len());
        for (index, _) in ranked_indices {
            if let Some(candidate) = owned_root_moves[*index].take() {
                ordered_root_moves.push(candidate);
            }
        }
        ordered_root_moves
    }

    fn sort_root_moves_by_ranked_scores(
        root_moves: Vec<ScoredRootMove>,
        scores: &[i32],
    ) -> Vec<ScoredRootMove> {
        debug_assert_eq!(root_moves.len(), scores.len());
        let mut ranked_indices = scores.iter().copied().enumerate().collect::<Vec<_>>();
        ranked_indices.sort_by(|a, b| Self::compare_ranked_root_indices(&root_moves, *a, *b));
        Self::reorder_root_moves_by_ranked_indices(root_moves, ranked_indices.as_slice())
    }

    fn compare_tactical_root_evaluations(
        candidate: &RootEvaluation,
        incumbent: &RootEvaluation,
    ) -> std::cmp::Ordering {
        if candidate.wins_immediately != incumbent.wins_immediately {
            return if candidate.wins_immediately {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.attacks_opponent_drainer != incumbent.attacks_opponent_drainer {
            return if candidate.attacks_opponent_drainer {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.own_drainer_vulnerable != incumbent.own_drainer_vulnerable {
            return if !candidate.own_drainer_vulnerable {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.classes.immediate_score != incumbent.classes.immediate_score {
            return if candidate.classes.immediate_score {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.scores_supermana_this_turn != incumbent.scores_supermana_this_turn {
            return if candidate.scores_supermana_this_turn {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.scores_opponent_mana_this_turn != incumbent.scores_opponent_mana_this_turn {
            return if candidate.scores_opponent_mana_this_turn {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.safe_supermana_pickup_now != incumbent.safe_supermana_pickup_now {
            return if candidate.safe_supermana_pickup_now {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.safe_opponent_mana_pickup_now != incumbent.safe_opponent_mana_pickup_now {
            return if candidate.safe_opponent_mana_pickup_now {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.same_turn_score_window_value != incumbent.same_turn_score_window_value {
            return incumbent
                .same_turn_score_window_value
                .cmp(&candidate.same_turn_score_window_value);
        }
        if candidate.spirit_same_turn_score_setup_now != incumbent.spirit_same_turn_score_setup_now
        {
            return if candidate.spirit_same_turn_score_setup_now {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.spirit_own_mana_setup_now != incumbent.spirit_own_mana_setup_now {
            return if candidate.spirit_own_mana_setup_now {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.spirit_own_mana_setup_now
            && incumbent.spirit_own_mana_setup_now
            && candidate.supermana_progress
            && incumbent.supermana_progress
            && candidate.safe_supermana_progress_steps != incumbent.safe_supermana_progress_steps
        {
            return if Self::root_progress_steps_better(
                candidate.safe_supermana_progress_steps,
                incumbent.safe_supermana_progress_steps,
            ) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.spirit_own_mana_setup_now
            && incumbent.spirit_own_mana_setup_now
            && candidate.opponent_mana_progress
            && incumbent.opponent_mana_progress
            && candidate.safe_opponent_mana_progress_steps
                != incumbent.safe_opponent_mana_progress_steps
        {
            return if Self::root_progress_steps_better(
                candidate.safe_opponent_mana_progress_steps,
                incumbent.safe_opponent_mana_progress_steps,
            ) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.spirit_own_mana_setup_now
            && incumbent.spirit_own_mana_setup_now
            && candidate.score_path_best_steps != incumbent.score_path_best_steps
        {
            return if Self::root_score_path_steps_better(
                candidate.score_path_best_steps,
                incumbent.score_path_best_steps,
            ) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.supermana_progress != incumbent.supermana_progress {
            return if candidate.supermana_progress {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.supermana_progress
            && incumbent.supermana_progress
            && candidate.safe_supermana_progress_steps != incumbent.safe_supermana_progress_steps
        {
            return if Self::root_progress_steps_better(
                candidate.safe_supermana_progress_steps,
                incumbent.safe_supermana_progress_steps,
            ) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.opponent_mana_progress != incumbent.opponent_mana_progress {
            return if candidate.opponent_mana_progress {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.opponent_mana_progress
            && incumbent.opponent_mana_progress
            && candidate.safe_opponent_mana_progress_steps
                != incumbent.safe_opponent_mana_progress_steps
        {
            return if Self::root_progress_steps_better(
                candidate.safe_opponent_mana_progress_steps,
                incumbent.safe_opponent_mana_progress_steps,
            ) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.mana_handoff_to_opponent != incumbent.mana_handoff_to_opponent {
            return if !candidate.mana_handoff_to_opponent {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.has_roundtrip != incumbent.has_roundtrip {
            return if !candidate.has_roundtrip {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.spirit_development != incumbent.spirit_development {
            return if candidate.spirit_development {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        if candidate.interview_soft_priority != incumbent.interview_soft_priority {
            return incumbent
                .interview_soft_priority
                .cmp(&candidate.interview_soft_priority);
        }
        if candidate.efficiency != incumbent.efficiency {
            return incumbent.efficiency.cmp(&candidate.efficiency);
        }
        std::cmp::Ordering::Equal
    }

    fn compare_ranked_scored_root_indices(
        scored_roots: &[RootEvaluation],
        a: usize,
        b: usize,
    ) -> std::cmp::Ordering {
        scored_roots[b]
            .score
            .cmp(&scored_roots[a].score)
            .then_with(|| {
                Self::compare_tactical_root_evaluations(&scored_roots[a], &scored_roots[b])
            })
            .then_with(|| a.cmp(&b))
    }

    fn best_tactical_root_index<F>(root_moves: &[ScoredRootMove], predicate: F) -> Option<usize>
    where
        F: Fn(&ScoredRootMove) -> bool,
    {
        let mut best_index = None;
        for (index, candidate) in root_moves.iter().enumerate() {
            if !predicate(candidate) {
                continue;
            }
            let is_better = match best_index {
                None => true,
                Some(incumbent_index) => {
                    Self::is_better_tactical_root_candidate(candidate, &root_moves[incumbent_index])
                }
            };
            if is_better {
                best_index = Some(index);
            }
        }
        best_index
    }

    fn forced_tactical_prepass_choice(
        game: &MonsGame,
        perspective: Color,
        root_moves: &[ScoredRootMove],
        config: SmartSearchConfig,
    ) -> Option<Vec<Input>> {
        if !config.enable_forced_tactical_prepass || root_moves.is_empty() {
            return None;
        }

        if let Some(index) =
            Self::best_tactical_root_index(root_moves, |candidate| candidate.wins_immediately)
        {
            return Some(root_moves[index].inputs.clone());
        }

        let has_supermana_scoring = config.enable_supermana_prepass_exception
            && root_moves.iter().any(|m| m.scores_supermana_this_turn);
        let has_safe_supermana_pickup = config.enable_supermana_prepass_exception
            && root_moves.iter().any(|m| {
                m.safe_supermana_pickup_now
                    && !m.own_drainer_vulnerable
                    && !m.mana_handoff_to_opponent
                    && !m.wins_immediately
                    && !m.attacks_opponent_drainer
            });
        let has_safe_opponent_mana_score = config.enable_opponent_mana_prepass_exception
            && root_moves.iter().any(|m| {
                m.scores_opponent_mana_this_turn
                    && !m.own_drainer_vulnerable
                    && !m.mana_handoff_to_opponent
                    && !m.wins_immediately
                    && !m.attacks_opponent_drainer
            });
        let has_safe_opponent_mana_pickup = config.enable_opponent_mana_prepass_exception
            && root_moves.iter().any(|m| {
                m.safe_opponent_mana_pickup_now
                    && !m.own_drainer_vulnerable
                    && !m.mana_handoff_to_opponent
                    && !m.wins_immediately
                    && !m.attacks_opponent_drainer
            });
        let has_tactical_prepass_exception = has_supermana_scoring
            || has_safe_supermana_pickup
            || has_safe_opponent_mana_score
            || has_safe_opponent_mana_pickup;

        if config.enable_supermana_prepass_exception {
            if let Some(index) = Self::best_tactical_root_index(root_moves, |candidate| {
                candidate.scores_supermana_this_turn
            }) {
                return Some(root_moves[index].inputs.clone());
            }
            if let Some(index) = Self::best_tactical_root_index(root_moves, |candidate| {
                candidate.safe_supermana_pickup_now
                    && !candidate.own_drainer_vulnerable
                    && !candidate.mana_handoff_to_opponent
                    && !candidate.wins_immediately
                    && !candidate.attacks_opponent_drainer
            }) {
                return Some(root_moves[index].inputs.clone());
            }
        }

        if config.enable_opponent_mana_prepass_exception {
            if let Some(index) = Self::best_tactical_root_index(root_moves, |candidate| {
                candidate.scores_opponent_mana_this_turn
                    && !candidate.own_drainer_vulnerable
                    && !candidate.mana_handoff_to_opponent
                    && !candidate.wins_immediately
                    && !candidate.attacks_opponent_drainer
            }) {
                return Some(root_moves[index].inputs.clone());
            }
            if let Some(index) = Self::best_tactical_root_index(root_moves, |candidate| {
                candidate.safe_opponent_mana_pickup_now
                    && !candidate.own_drainer_vulnerable
                    && !candidate.mana_handoff_to_opponent
                    && !candidate.wins_immediately
                    && !candidate.attacks_opponent_drainer
            }) {
                return Some(root_moves[index].inputs.clone());
            }
        }

        if config.enable_forced_drainer_attack
            && !has_tactical_prepass_exception
            && !config.enable_drainer_attack_minimax_selection
            && !config.enable_drainer_attack_full_pool
        {
            let prepass_attack = if config.enable_conditional_forced_drainer_attack {
                let (my_score, opponent_score) = if perspective == Color::White {
                    (game.white_score, game.black_score)
                } else {
                    (game.black_score, game.white_score)
                };
                my_score <= opponent_score + config.conditional_forced_attack_score_margin
            } else {
                true
            };
            if prepass_attack {
                if let Some(index) = Self::best_tactical_root_index(root_moves, |candidate| {
                    candidate.attacks_opponent_drainer
                }) {
                    return Some(root_moves[index].inputs.clone());
                }
            }
        }

        if config.enable_root_drainer_safety_prefilter
            && !has_tactical_prepass_exception
            && Self::is_own_drainer_vulnerable_next_turn(
                game,
                perspective,
                config.enable_enhanced_drainer_vulnerability,
            )
        {
            if let Some(index) = Self::best_tactical_root_index(root_moves, |candidate| {
                !candidate.own_drainer_vulnerable
            }) {
                return Some(root_moves[index].inputs.clone());
            }
        }

        if config.enable_walk_threat_prefilter
            && !has_tactical_prepass_exception
            && Self::is_own_drainer_walk_vulnerable_next_turn(
                game,
                perspective,
                config.enable_enhanced_drainer_vulnerability,
            )
        {
            if let Some(index) = Self::best_tactical_root_index(root_moves, |candidate| {
                !candidate.own_drainer_vulnerable && !candidate.own_drainer_walk_vulnerable
            }) {
                return Some(root_moves[index].inputs.clone());
            }
        }

        let opponent_score = if perspective == Color::White {
            game.black_score
        } else {
            game.white_score
        };
        let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
        if opponent_distance_to_win <= 1 {
            if let Some(index) = Self::best_tactical_root_index(root_moves, |candidate| {
                candidate.classes.immediate_score
            }) {
                return Some(root_moves[index].inputs.clone());
            }
        }

        None
    }

    fn truncate_root_candidates_with_class_coverage(
        candidates: Vec<ScoredRootMove>,
        limit: usize,
        strict_guarantees: bool,
    ) -> Vec<ScoredRootMove> {
        if candidates.len() <= limit {
            return candidates;
        }

        let mut selected_mask = vec![false; candidates.len()];
        let mut priority_indices = Vec::new();
        let best_heuristic = candidates[0].heuristic;
        let min_critical_heuristic =
            best_heuristic.saturating_sub(SMART_MOVE_CLASS_ROOT_SCORE_MARGIN.max(0));
        let mut mark_priority = |index: usize| {
            selected_mask[index] = true;
            if !priority_indices.contains(&index) {
                priority_indices.push(index);
            }
        };
        for class in [
            MoveClass::ImmediateScore,
            MoveClass::DrainerAttack,
            MoveClass::DrainerSafetyRecover,
        ] {
            let chosen = if strict_guarantees {
                candidates
                    .iter()
                    .enumerate()
                    .find(|(_, candidate)| candidate.classes.has(class))
            } else {
                candidates.iter().enumerate().find(|(_, candidate)| {
                    candidate.classes.has(class) && candidate.heuristic >= min_critical_heuristic
                })
            };
            if let Some((index, _)) = chosen {
                mark_priority(index);
            }
        }
        if let Some(index) = Self::best_tactical_root_index(candidates.as_slice(), |candidate| {
            candidate.same_turn_score_window_value > 0
                && (strict_guarantees || candidate.heuristic >= min_critical_heuristic)
        }) {
            mark_priority(index);
        }
        let unknown_progress_steps = Config::BOARD_SIZE + 4;
        for (index, candidate) in candidates.iter().enumerate() {
            let keep_direct_high_value = candidate.scores_supermana_this_turn
                || candidate.scores_opponent_mana_this_turn
                || candidate.safe_supermana_pickup_now
                || candidate.safe_opponent_mana_pickup_now
                || candidate.spirit_same_turn_score_setup_now
                || candidate.same_turn_score_window_value > 0
                || candidate.spirit_own_mana_setup_now;
            let keep_exact_progress = (candidate.supermana_progress
                && candidate.safe_supermana_progress_steps < unknown_progress_steps)
                || (candidate.opponent_mana_progress
                    && candidate.safe_opponent_mana_progress_steps < unknown_progress_steps);
            if !keep_direct_high_value && !keep_exact_progress {
                continue;
            }
            if strict_guarantees || candidate.heuristic >= min_critical_heuristic {
                selected_mask[index] = true;
            }
        }

        let mut owned_candidates = candidates.into_iter().map(Some).collect::<Vec<_>>();
        let mut shortlisted = Vec::with_capacity(limit.min(owned_candidates.len()));
        for index in priority_indices {
            if shortlisted.len() >= limit {
                break;
            }
            if let Some(candidate) = owned_candidates[index].take() {
                shortlisted.push(candidate);
            }
        }
        for index in 0..owned_candidates.len() {
            if shortlisted.len() >= limit {
                break;
            }
            if selected_mask[index] {
                if let Some(candidate) = owned_candidates[index].take() {
                    shortlisted.push(candidate);
                }
            }
        }
        for index in 0..owned_candidates.len() {
            if shortlisted.len() >= limit {
                break;
            }
            if selected_mask[index] {
                continue;
            }
            if let Some(candidate) = owned_candidates[index].take() {
                shortlisted.push(candidate);
            }
        }
        shortlisted
    }

    fn classify_move_classes(
        before: &MonsGame,
        after: &MonsGame,
        actor_color: Color,
        events: &[Event],
        own_drainer_vulnerable_before: bool,
        own_drainer_vulnerable_after: bool,
    ) -> MoveClassFlags {
        let immediate_score = events
            .iter()
            .any(|event| matches!(event, Event::ManaScored { .. }));
        let drainer_attack = Self::events_include_opponent_drainer_fainted(events, actor_color);
        let drainer_safety_recover = own_drainer_vulnerable_before && !own_drainer_vulnerable_after;
        let spirit_development =
            Self::has_spirit_development_turn(before, after, actor_color, events);
        let material = Self::has_material_event(events);
        let carrier_progress = Self::has_actor_carrier_progress(before, after, actor_color);
        let quiet = !immediate_score
            && !drainer_attack
            && !drainer_safety_recover
            && !spirit_development
            && !carrier_progress
            && !material;
        MoveClassFlags {
            immediate_score,
            drainer_attack,
            drainer_safety_recover,
            carrier_progress,
            material,
            quiet,
        }
    }

    fn has_actor_carrier_progress(before: &MonsGame, after: &MonsGame, actor_color: Color) -> bool {
        let before_snapshot = Self::move_efficiency_snapshot(before, actor_color, false);
        let after_snapshot = Self::move_efficiency_snapshot(after, actor_color, false);
        if after_snapshot.my_carrier_count > before_snapshot.my_carrier_count {
            return true;
        }
        let unknown_steps = Config::BOARD_SIZE + 4;
        after_snapshot.my_best_carrier_steps < before_snapshot.my_best_carrier_steps
            && after_snapshot.my_best_carrier_steps < unknown_steps
    }

    fn root_volatility_score(candidate: &ScoredRootMove) -> i32 {
        let mut score = 0i32;
        if candidate.wins_immediately {
            score += 5_000;
        }
        if candidate.attacks_opponent_drainer || candidate.classes.drainer_attack {
            score += 2_800;
        }
        if candidate.own_drainer_vulnerable {
            score += 2_200;
        }
        if candidate.classes.immediate_score {
            score += 1_700;
        }
        if candidate.classes.drainer_safety_recover {
            score += 1_500;
        }
        if candidate.mana_handoff_to_opponent {
            score += 900;
        }
        if candidate.has_roundtrip {
            score += 700;
        }
        if candidate.classes.material {
            score += 240;
        }
        if candidate.efficiency < 0 {
            score += (-candidate.efficiency).min(400);
        }
        score
    }

    fn enforce_tactical_child_top2(
        scored_states: &mut [(i32, RankedChildState)],
        maximizing: bool,
        strict_guarantees: bool,
    ) {
        if scored_states.len() < 3 {
            return;
        }
        let top_has_tactical = scored_states
            .iter()
            .take(2)
            .any(|(_, state)| Self::is_child_search_priority_candidate(state));
        if top_has_tactical {
            return;
        }

        let second_score = scored_states[1].0;
        let replacement_index =
            scored_states
                .iter()
                .enumerate()
                .skip(2)
                .find_map(|(index, (score, state))| {
                    if !Self::is_child_search_priority_candidate(state) {
                        return None;
                    }
                    if strict_guarantees {
                        Some(index)
                    } else {
                        let tactical_margin = SMART_MOVE_CLASS_CHILD_SCORE_MARGIN.max(0);
                        let close_enough = if maximizing {
                            score.saturating_add(tactical_margin) >= second_score
                        } else {
                            *score <= second_score.saturating_add(tactical_margin)
                        };
                        if close_enough {
                            Some(index)
                        } else {
                            None
                        }
                    }
                });
        let Some(replacement_index) = replacement_index else {
            return;
        };

        let swap_index = 1;
        scored_states.swap(swap_index, replacement_index);
    }

    #[cfg(any(target_arch = "wasm32", test))]
    fn smart_search_best_inputs(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
        Self::smart_search_best_inputs_internal(game, config, true)
    }

    #[cfg(test)]
    fn smart_search_best_inputs_legacy_no_transposition(
        game: &MonsGame,
        config: SmartSearchConfig,
    ) -> Vec<Input> {
        Self::smart_search_best_inputs_internal(game, config, false)
    }

    #[cfg(any(target_arch = "wasm32", test))]
    fn smart_search_best_inputs_internal(
        game: &MonsGame,
        config: SmartSearchConfig,
        use_transposition_table: bool,
    ) -> Vec<Input> {
        clear_exact_state_analysis_cache();
        let perspective = game.active_color;
        let root_moves = Self::ranked_root_moves(game, perspective, config);
        if root_moves.is_empty() {
            return Vec::new();
        }
        if let Some(forced_inputs) =
            Self::forced_tactical_prepass_choice(game, perspective, root_moves.as_slice(), config)
        {
            return forced_inputs;
        }

        let (mut root_moves, scout_visited_nodes) = Self::focused_root_candidates(
            game,
            perspective,
            root_moves,
            config,
            use_transposition_table,
        );
        if root_moves.is_empty() {
            return Vec::new();
        }

        let mut visited_nodes = scout_visited_nodes;
        let mut alpha = i32::MIN;
        let mut scored_roots = Vec::with_capacity(root_moves.len());
        let mut transposition_table = U64HashMap::default();
        let extension_node_budget =
            if config.enable_selective_extensions && config.selective_extension_node_share_bp > 0 {
                ((config.max_visited_nodes * config.selective_extension_node_share_bp as usize)
                    / 10_000)
                    .max(1)
            } else {
                0
            };
        let mut extension_nodes_used = 0usize;
        let mut killer_table: KillerTable = [[0u64; 2]; MAX_SMART_SEARCH_DEPTH + 2];

        // Iterative deepening: run a shallow pass to pre-populate TT and reorder roots
        if config.enable_iterative_deepening && config.depth >= 3 {
            let preliminary_depth = config
                .depth
                .saturating_sub(config.iterative_deepening_depth_offset)
                .max(1);
            let mut prelim_alpha = i32::MIN;
            let mut prelim_scores: Vec<i32> = Vec::with_capacity(root_moves.len());
            for candidate in root_moves.iter() {
                if visited_nodes >= config.max_visited_nodes {
                    prelim_scores.push(i32::MIN);
                    continue;
                }
                visited_nodes += 1;
                let prelim_score = Self::search_score(
                    &candidate.game,
                    perspective,
                    preliminary_depth,
                    prelim_alpha,
                    i32::MAX,
                    &mut visited_nodes,
                    config,
                    &mut transposition_table,
                    0,
                    &mut extension_nodes_used,
                    0,
                    use_transposition_table,
                    &mut killer_table,
                );
                prelim_scores.push(prelim_score);
                if prelim_score > prelim_alpha {
                    prelim_alpha = prelim_score;
                }
            }
            let best_prelim_score = prelim_scores.iter().copied().max().unwrap_or(i32::MIN);
            root_moves =
                Self::sort_root_moves_by_ranked_scores(root_moves, prelim_scores.as_slice());
            // Optionally initialize alpha from preliminary best score
            if config.iterative_deepening_alpha_margin > 0 {
                if best_prelim_score != i32::MIN {
                    alpha =
                        best_prelim_score.saturating_sub(config.iterative_deepening_alpha_margin);
                }
            }
        }

        for candidate in root_moves {
            if visited_nodes >= config.max_visited_nodes {
                break;
            }

            visited_nodes += 1;
            let candidate_score = Self::evaluate_root_candidate_score(
                &candidate,
                perspective,
                alpha,
                &mut visited_nodes,
                config,
                &mut transposition_table,
                &mut extension_nodes_used,
                extension_node_budget,
                use_transposition_table,
                &mut killer_table,
            );

            scored_roots.push(RootEvaluation {
                score: candidate_score,
                efficiency: candidate.efficiency,
                inputs: candidate.inputs,
                game: candidate.game,
                wins_immediately: candidate.wins_immediately,
                attacks_opponent_drainer: candidate.attacks_opponent_drainer,
                own_drainer_vulnerable: candidate.own_drainer_vulnerable,
                own_drainer_walk_vulnerable: candidate.own_drainer_walk_vulnerable,
                spirit_development: candidate.spirit_development,
                keeps_awake_spirit_on_base: candidate.keeps_awake_spirit_on_base,
                mana_handoff_to_opponent: candidate.mana_handoff_to_opponent,
                has_roundtrip: candidate.has_roundtrip,
                scores_supermana_this_turn: candidate.scores_supermana_this_turn,
                scores_opponent_mana_this_turn: candidate.scores_opponent_mana_this_turn,
                safe_supermana_pickup_now: candidate.safe_supermana_pickup_now,
                safe_opponent_mana_pickup_now: candidate.safe_opponent_mana_pickup_now,
                safe_supermana_progress_steps: candidate.safe_supermana_progress_steps,
                safe_opponent_mana_progress_steps: candidate.safe_opponent_mana_progress_steps,
                score_path_best_steps: candidate.score_path_best_steps,
                same_turn_score_window_value: candidate.same_turn_score_window_value,
                spirit_same_turn_score_setup_now: candidate.spirit_same_turn_score_setup_now,
                spirit_own_mana_setup_now: candidate.spirit_own_mana_setup_now,
                supermana_progress: candidate.supermana_progress,
                opponent_mana_progress: candidate.opponent_mana_progress,
                interview_soft_priority: candidate.interview_soft_priority,
                classes: candidate.classes,
            });

            if candidate_score > alpha {
                alpha = candidate_score;
            }
        }

        if scored_roots.is_empty() {
            Vec::new()
        } else {
            Self::pick_root_move_with_exploration(game, &scored_roots, perspective, config)
        }
    }

    fn evaluate_root_candidate_score(
        candidate: &ScoredRootMove,
        perspective: Color,
        alpha: i32,
        visited_nodes: &mut usize,
        config: SmartSearchConfig,
        transposition_table: &mut U64HashMap<TranspositionEntry>,
        extension_nodes_used: &mut usize,
        extension_node_budget: usize,
        use_transposition_table: bool,
        killer_table: &mut KillerTable,
    ) -> i32 {
        if config.depth <= 1 {
            return candidate.heuristic;
        }

        let mut alpha_bound = alpha;
        let mut beta_bound = i32::MAX;

        if config.enable_root_aspiration && alpha != i32::MIN {
            alpha_bound = alpha.saturating_sub(SMART_ROOT_ASPIRATION_WINDOW);
            beta_bound = alpha.saturating_add(SMART_ROOT_ASPIRATION_WINDOW);
        }

        let mut score = Self::search_score(
            &candidate.game,
            perspective,
            config.depth - 1,
            alpha_bound,
            beta_bound,
            visited_nodes,
            config,
            transposition_table,
            config.max_extensions_per_path,
            extension_nodes_used,
            extension_node_budget,
            use_transposition_table,
            killer_table,
        );

        if config.enable_root_aspiration
            && alpha != i32::MIN
            && visited_nodes.saturating_add(1) < config.max_visited_nodes
            && (score <= alpha_bound || score >= beta_bound)
        {
            score = Self::search_score(
                &candidate.game,
                perspective,
                config.depth - 1,
                alpha,
                i32::MAX,
                visited_nodes,
                config,
                transposition_table,
                config.max_extensions_per_path,
                extension_nodes_used,
                extension_node_budget,
                use_transposition_table,
                killer_table,
            );
        }

        score
    }

    fn focused_root_candidates(
        _game: &MonsGame,
        perspective: Color,
        root_moves: Vec<ScoredRootMove>,
        config: SmartSearchConfig,
        use_transposition_table: bool,
    ) -> (Vec<ScoredRootMove>, usize) {
        if !config.enable_two_pass_root_allocation
            || root_moves.len() <= config.root_focus_k.max(1)
            || config.depth <= 1
        {
            return (root_moves, 0);
        }

        let scout_depth = if config.enable_two_pass_volatility_focus || config.depth <= 3 {
            1
        } else {
            config.depth.min(SMART_TWO_PASS_ROOT_SCOUT_DEPTH).max(1)
        };
        let scout_share_bp = (10_000 - config.root_focus_budget_share_bp).clamp(500, 4_000);
        let scout_budget = if scout_depth <= 1 {
            root_moves.len()
        } else {
            ((config.max_visited_nodes * scout_share_bp as usize) / 10_000).clamp(
                SMART_TWO_PASS_ROOT_SCOUT_MIN_NODES,
                config.max_visited_nodes.saturating_sub(1),
            )
        };
        if scout_budget < root_moves.len() {
            return (root_moves, 0);
        }

        let mut scout_config = config;
        scout_config.depth = scout_depth;
        scout_config.max_visited_nodes = scout_budget;
        scout_config.enable_root_aspiration = false;
        scout_config.enable_selective_extensions = false;
        scout_config.enable_quiet_reductions = false;

        let mut scout_visited_nodes = 0usize;
        let mut scout_alpha = i32::MIN;
        let mut scout_transposition_table = U64HashMap::default();
        let mut scout_extension_nodes_used = 0usize;
        let mut scout_killer_table: KillerTable = [[0u64; 2]; MAX_SMART_SEARCH_DEPTH + 2];
        let mut scout_scores = vec![i32::MIN; root_moves.len()];
        let mut best_scout_score = i32::MIN;

        for (index, candidate) in root_moves.iter().enumerate() {
            if scout_depth > 1 && scout_visited_nodes >= scout_config.max_visited_nodes {
                break;
            }
            if scout_depth > 1 {
                scout_visited_nodes += 1;
            }
            let score = if scout_depth > 1 {
                Self::search_score(
                    &candidate.game,
                    perspective,
                    scout_depth - 1,
                    scout_alpha,
                    i32::MAX,
                    &mut scout_visited_nodes,
                    scout_config,
                    &mut scout_transposition_table,
                    0,
                    &mut scout_extension_nodes_used,
                    0,
                    use_transposition_table,
                    &mut scout_killer_table,
                )
            } else {
                Self::root_focus_scout_score(candidate)
            };
            scout_scores[index] = score;
            best_scout_score = best_scout_score.max(score);
            scout_alpha = scout_alpha.max(score);
        }

        let focus_k = config.root_focus_k.max(1).min(root_moves.len());
        let mut ranked_indices = (0..root_moves.len())
            .map(|index| {
                let score = if scout_scores[index] == i32::MIN {
                    Self::root_focus_scout_score(&root_moves[index])
                } else {
                    scout_scores[index]
                };
                (index, score)
            })
            .collect::<Vec<_>>();
        ranked_indices.sort_by(|a, b| Self::compare_ranked_root_indices(&root_moves, *a, *b));

        if ranked_indices.len() >= focus_k {
            let best_score = ranked_indices[0].1;
            let kth_score = ranked_indices[focus_k - 1].1;
            if best_score.saturating_sub(kth_score) <= SMART_TWO_PASS_ROOT_NARROW_SPREAD_FALLBACK {
                return (
                    Self::reorder_root_moves_by_ranked_indices(
                        root_moves,
                        ranked_indices.as_slice(),
                    ),
                    0,
                );
            }
        }

        let mut selected_mask = vec![false; root_moves.len()];
        for (index, _) in ranked_indices.iter().take(focus_k) {
            selected_mask[*index] = true;
        }
        for (index, score) in ranked_indices.iter().copied() {
            if score + SMART_TWO_PASS_ROOT_FOCUS_SCORE_MARGIN < best_scout_score {
                continue;
            }
            selected_mask[index] = true;
        }
        for (index, candidate) in root_moves.iter().enumerate() {
            if candidate.attacks_opponent_drainer {
                selected_mask[index] = true;
            }
            if candidate.scores_supermana_this_turn
                || candidate.scores_opponent_mana_this_turn
                || candidate.safe_supermana_pickup_now
                || candidate.safe_opponent_mana_pickup_now
                || candidate.spirit_same_turn_score_setup_now
                || candidate.same_turn_score_window_value > 0
                || candidate.spirit_own_mana_setup_now
            {
                selected_mask[index] = true;
            }
        }
        if config.enable_two_pass_volatility_focus {
            let mut volatility_ranked = (0..root_moves.len())
                .map(|index| {
                    let scout_score = if scout_scores[index] == i32::MIN {
                        Self::root_focus_scout_score(&root_moves[index])
                    } else {
                        scout_scores[index]
                    };
                    (
                        index,
                        Self::root_volatility_score(&root_moves[index]),
                        scout_score,
                    )
                })
                .filter(|(_, volatility, _)| *volatility > 0)
                .collect::<Vec<_>>();
            volatility_ranked.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));

            for (index, _, _) in volatility_ranked
                .iter()
                .take(SMART_TWO_PASS_ROOT_VOLATILITY_KEEP.max(1))
            {
                selected_mask[*index] = true;
            }

            if let Some(best_volatility) = volatility_ranked.first().map(|(_, value, _)| *value) {
                for (index, volatility, scout_score) in volatility_ranked {
                    if volatility + SMART_TWO_PASS_ROOT_VOLATILITY_MARGIN < best_volatility {
                        break;
                    }
                    if scout_score + SMART_TWO_PASS_ROOT_FOCUS_SCORE_MARGIN < best_scout_score {
                        continue;
                    }
                    selected_mask[index] = true;
                }
            }
        }

        if !selected_mask.iter().any(|is_selected| *is_selected) {
            return (root_moves, 0);
        }

        let mut focused_with_scores = selected_mask
            .iter()
            .enumerate()
            .filter_map(|(index, is_selected)| {
                if !*is_selected {
                    return None;
                }
                let score = if scout_scores[index] == i32::MIN {
                    Self::root_focus_scout_score(&root_moves[index])
                } else {
                    scout_scores[index]
                };
                Some((index, score))
            })
            .collect::<Vec<_>>();
        focused_with_scores.sort_by(|a, b| Self::compare_ranked_root_indices(&root_moves, *a, *b));

        let mut owned_root_moves = root_moves.into_iter().map(Some).collect::<Vec<_>>();
        let mut focused_root_moves = Vec::with_capacity(focused_with_scores.len());
        for (index, _) in focused_with_scores {
            if let Some(candidate) = owned_root_moves[index].take() {
                focused_root_moves.push(candidate);
            }
        }

        (
            focused_root_moves,
            scout_visited_nodes.min(config.max_visited_nodes),
        )
    }

    fn search_score(
        game: &MonsGame,
        perspective: Color,
        depth: usize,
        alpha: i32,
        beta: i32,
        visited_nodes: &mut usize,
        config: SmartSearchConfig,
        transposition_table: &mut U64HashMap<TranspositionEntry>,
        extensions_remaining: usize,
        extension_nodes_used: &mut usize,
        extension_node_budget: usize,
        use_transposition_table: bool,
        killer_table: &mut KillerTable,
    ) -> i32 {
        if let Some(terminal_score) = Self::terminal_score(game, perspective, depth, config.depth) {
            return terminal_score;
        }
        if *visited_nodes >= config.max_visited_nodes {
            return evaluate_preferability_with_weights(game, perspective, config.scoring_weights);
        }
        if depth == 0 {
            return evaluate_preferability_with_weights(game, perspective, config.scoring_weights);
        }

        let mut alpha = alpha;
        let mut beta = beta;
        let alpha_before = alpha;
        let beta_before = beta;
        let state_key = Self::search_state_hash(game);
        let mut preferred_child_hash = None;

        if use_transposition_table {
            if let Some(entry) = transposition_table.get(&state_key).copied() {
                if config.enable_tt_best_child_ordering && entry.best_child_hash != 0 {
                    preferred_child_hash = Some(entry.best_child_hash);
                }
                if entry.depth >= depth {
                    match entry.bound {
                        TranspositionBound::Exact => return entry.score,
                        TranspositionBound::LowerBound => {
                            alpha = alpha.max(entry.score);
                        }
                        TranspositionBound::UpperBound => {
                            beta = beta.min(entry.score);
                        }
                    }
                    if alpha >= beta {
                        return entry.score;
                    }
                }
            }
        }

        let maximizing = game.active_color == perspective;

        // Futility pruning: at frontier nodes (depth=1), skip expansion if static eval
        // is so far from the window that no child move could possibly improve it enough
        if config.enable_futility_pruning
            && depth == 1
            && !Self::has_exact_frontier_tactical_potential(game)
        {
            let static_eval =
                evaluate_preferability_with_weights(game, perspective, config.scoring_weights);
            if maximizing && static_eval + config.futility_margin < alpha {
                return static_eval;
            }
            if !maximizing && static_eval - config.futility_margin > beta {
                return static_eval;
            }
        }

        let current_killers = if config.enable_killer_move_ordering && depth < killer_table.len() {
            killer_table[depth]
        } else {
            [0u64; 2]
        };
        let mut children = Self::ranked_child_states(
            game,
            perspective,
            maximizing,
            preferred_child_hash,
            current_killers,
            config,
        );
        if children.is_empty() {
            return evaluate_preferability_with_weights(game, perspective, config.scoring_weights);
        }

        let mut stopped_by_budget = false;
        let mut best_child_hash = 0u64;
        let value = if maximizing {
            let mut value = i32::MIN;
            let mut first_child = true;
            for child in children.drain(..) {
                if *visited_nodes >= config.max_visited_nodes {
                    stopped_by_budget = true;
                    break;
                }

                let mut child_depth = depth.saturating_sub(1);
                let mut child_extensions_remaining = extensions_remaining;
                if config.enable_selective_extensions
                    && Self::is_selective_extension_candidate(
                        child.tactical_extension_trigger,
                        child.ordering_efficiency,
                        child.classes,
                    )
                    && child_extensions_remaining > 0
                    && (extension_node_budget == 0 || *extension_nodes_used < extension_node_budget)
                {
                    child_depth = depth;
                    child_extensions_remaining = child_extensions_remaining.saturating_sub(1);
                    if extension_node_budget > 0 {
                        *extension_nodes_used = extension_nodes_used.saturating_add(1);
                    }
                } else if config.enable_quiet_reductions
                    && child.quiet_reduction_candidate
                    && depth >= config.quiet_reduction_depth_threshold
                {
                    child_depth = depth.saturating_sub(2);
                }

                *visited_nodes += 1;
                let score = if config.enable_pvs && !first_child && alpha + 1 < beta {
                    // PVS: null-window probe
                    let probe = Self::search_score(
                        &child.game,
                        perspective,
                        child_depth,
                        alpha,
                        alpha + 1,
                        visited_nodes,
                        config,
                        transposition_table,
                        child_extensions_remaining,
                        extension_nodes_used,
                        extension_node_budget,
                        use_transposition_table,
                        killer_table,
                    );
                    if probe > alpha && probe < beta {
                        // Fail high: re-search with full window
                        Self::search_score(
                            &child.game,
                            perspective,
                            child_depth,
                            alpha,
                            beta,
                            visited_nodes,
                            config,
                            transposition_table,
                            child_extensions_remaining,
                            extension_nodes_used,
                            extension_node_budget,
                            use_transposition_table,
                            killer_table,
                        )
                    } else {
                        probe
                    }
                } else {
                    Self::search_score(
                        &child.game,
                        perspective,
                        child_depth,
                        alpha,
                        beta,
                        visited_nodes,
                        config,
                        transposition_table,
                        child_extensions_remaining,
                        extension_nodes_used,
                        extension_node_budget,
                        use_transposition_table,
                        killer_table,
                    )
                };
                first_child = false;
                if score > value {
                    value = score;
                    best_child_hash = child.hash;
                }
                alpha = alpha.max(value);
                if alpha >= beta {
                    // Store killer move on beta cutoff
                    if config.enable_killer_move_ordering
                        && child.hash != 0
                        && depth < killer_table.len()
                    {
                        if killer_table[depth][0] != child.hash {
                            killer_table[depth][1] = killer_table[depth][0];
                            killer_table[depth][0] = child.hash;
                        }
                    }
                    break;
                }
            }

            if value == i32::MIN {
                evaluate_preferability_with_weights(game, perspective, config.scoring_weights)
            } else {
                value
            }
        } else {
            let mut value = i32::MAX;
            let mut first_child = true;
            for child in children.drain(..) {
                if *visited_nodes >= config.max_visited_nodes {
                    stopped_by_budget = true;
                    break;
                }

                let mut child_depth = depth.saturating_sub(1);
                let mut child_extensions_remaining = extensions_remaining;
                if config.enable_selective_extensions
                    && Self::is_selective_extension_candidate(
                        child.tactical_extension_trigger,
                        child.ordering_efficiency,
                        child.classes,
                    )
                    && child_extensions_remaining > 0
                    && (extension_node_budget == 0 || *extension_nodes_used < extension_node_budget)
                {
                    child_depth = depth;
                    child_extensions_remaining = child_extensions_remaining.saturating_sub(1);
                    if extension_node_budget > 0 {
                        *extension_nodes_used = extension_nodes_used.saturating_add(1);
                    }
                } else if config.enable_quiet_reductions
                    && child.quiet_reduction_candidate
                    && depth >= config.quiet_reduction_depth_threshold
                {
                    child_depth = depth.saturating_sub(2);
                }

                *visited_nodes += 1;
                let score = if config.enable_pvs && !first_child && beta - 1 > alpha {
                    // PVS: null-window probe (minimizing)
                    let probe = Self::search_score(
                        &child.game,
                        perspective,
                        child_depth,
                        beta - 1,
                        beta,
                        visited_nodes,
                        config,
                        transposition_table,
                        child_extensions_remaining,
                        extension_nodes_used,
                        extension_node_budget,
                        use_transposition_table,
                        killer_table,
                    );
                    if probe < beta && probe > alpha {
                        // Fail low: re-search with full window
                        Self::search_score(
                            &child.game,
                            perspective,
                            child_depth,
                            alpha,
                            beta,
                            visited_nodes,
                            config,
                            transposition_table,
                            child_extensions_remaining,
                            extension_nodes_used,
                            extension_node_budget,
                            use_transposition_table,
                            killer_table,
                        )
                    } else {
                        probe
                    }
                } else {
                    Self::search_score(
                        &child.game,
                        perspective,
                        child_depth,
                        alpha,
                        beta,
                        visited_nodes,
                        config,
                        transposition_table,
                        child_extensions_remaining,
                        extension_nodes_used,
                        extension_node_budget,
                        use_transposition_table,
                        killer_table,
                    )
                };
                first_child = false;
                if score < value {
                    value = score;
                    best_child_hash = child.hash;
                }
                beta = beta.min(value);
                if beta <= alpha {
                    // Store killer move on alpha cutoff (minimizing side)
                    if config.enable_killer_move_ordering
                        && child.hash != 0
                        && depth < killer_table.len()
                    {
                        if killer_table[depth][0] != child.hash {
                            killer_table[depth][1] = killer_table[depth][0];
                            killer_table[depth][0] = child.hash;
                        }
                    }
                    break;
                }
            }

            if value == i32::MAX {
                evaluate_preferability_with_weights(game, perspective, config.scoring_weights)
            } else {
                value
            }
        };

        if use_transposition_table && !stopped_by_budget {
            let bound = if value <= alpha_before {
                TranspositionBound::UpperBound
            } else if value >= beta_before {
                TranspositionBound::LowerBound
            } else {
                TranspositionBound::Exact
            };

            let mut skip_tt_write = false;
            if transposition_table.len() >= SMART_TRANSPOSITION_TABLE_MAX_ENTRIES
                && !transposition_table.contains_key(&state_key)
            {
                if config.enable_tt_depth_preferred_replacement {
                    // Depth-preferred replacement: evict the shallowest entry
                    let mut shallowest_key = 0u64;
                    let mut shallowest_depth = usize::MAX;
                    for (&k, v) in transposition_table.iter() {
                        if v.depth < shallowest_depth {
                            shallowest_depth = v.depth;
                            shallowest_key = k;
                        }
                    }
                    if shallowest_depth < depth {
                        transposition_table.remove(&shallowest_key);
                    } else {
                        // No shallower entry exists; skip writing to respect capacity
                        skip_tt_write = true;
                    }
                } else {
                    transposition_table.clear();
                }
            }
            if !skip_tt_write {
                transposition_table.insert(
                    state_key,
                    TranspositionEntry {
                        depth,
                        score: value,
                        bound,
                        best_child_hash,
                    },
                );
            }
        }

        value
    }

    fn ranked_child_states(
        game: &MonsGame,
        perspective: Color,
        maximizing: bool,
        preferred_child_hash: Option<u64>,
        killer_hashes: [u64; 2],
        config: SmartSearchConfig,
    ) -> Vec<RankedChildState> {
        let mut scored_states: Vec<(i32, RankedChildState)> = Vec::new();
        let actor_color = game.active_color;
        let own_drainer_vulnerable_before = if config.enable_child_move_class_coverage {
            Self::is_own_drainer_vulnerable_next_turn(
                game,
                actor_color,
                config.enable_enhanced_drainer_vulnerability,
            )
        } else {
            false
        };
        let start_options = Self::automove_start_input_options(config);
        let mut child_transitions =
            Self::enumerate_legal_transitions(game, config.node_enum_limit, start_options);
        if config.enable_child_move_class_coverage {
            let exact_turn_before = exact_turn_summary(game, actor_color);
            let fallback_limit = Self::child_exact_progress_fallback_candidates_limit(config);
            let mut seen_inputs = child_transitions
                .iter()
                .map(|transition| transition.inputs.clone())
                .collect::<std::collections::HashSet<_>>();
            if (exact_turn_before.safe_supermana_progress
                || exact_turn_before.spirit_assisted_supermana_progress)
                && !child_transitions.iter().any(|transition| {
                    Self::transition_preserves_exact_progress(
                        transition,
                        actor_color,
                        Mana::Supermana,
                    )
                })
            {
                let fallback_inputs = Self::collect_targeted_exact_progress_inputs(
                    game,
                    actor_color,
                    config,
                    fallback_limit,
                    Mana::Supermana,
                );
                for transition in fallback_inputs {
                    if seen_inputs.insert(transition.inputs.clone()) {
                        child_transitions.push(transition);
                    }
                }
            }
            if (exact_turn_before.safe_opponent_mana_progress
                || exact_turn_before.spirit_assisted_opponent_mana_progress
                || exact_turn_before.spirit_assisted_denial)
                && !child_transitions.iter().any(|transition| {
                    Self::transition_preserves_exact_progress(
                        transition,
                        actor_color,
                        Mana::Regular(actor_color.other()),
                    )
                })
            {
                let fallback_inputs = Self::collect_targeted_exact_progress_inputs(
                    game,
                    actor_color,
                    config,
                    fallback_limit,
                    Mana::Regular(actor_color.other()),
                );
                for transition in fallback_inputs {
                    if seen_inputs.insert(transition.inputs.clone()) {
                        child_transitions.push(transition);
                    }
                }
            }
        }

        for transition in child_transitions {
            let simulated_game = transition.game;
            let events = transition.events;
            let child_hash = Self::search_state_hash(&simulated_game);
            let mut heuristic = Self::score_state(
                &simulated_game,
                perspective,
                0,
                config.depth,
                config.scoring_weights,
            );

            let ordering_efficiency = if config.enable_root_efficiency {
                Self::move_efficiency_delta(
                    game,
                    &simulated_game,
                    actor_color,
                    &events,
                    false,
                    false,
                    false,
                    config.root_backtrack_penalty,
                    config.root_mana_handoff_penalty,
                )
            } else {
                0
            };
            if config.enable_root_efficiency
                && SMART_NO_EFFECT_CHILD_PENALTY != 0
                && Self::is_no_effect_state_transition(game, &simulated_game)
            {
                heuristic -= SMART_NO_EFFECT_CHILD_PENALTY;
            }
            if config.enable_event_ordering_bonus {
                heuristic = heuristic.saturating_add(Self::ordering_event_bonus(
                    game.active_color,
                    perspective,
                    &events,
                ));
            }
            if config.enable_tt_best_child_ordering
                && preferred_child_hash.is_some()
                && preferred_child_hash == Some(child_hash)
            {
                heuristic = heuristic.saturating_add(SMART_TT_BEST_CHILD_BONUS);
            }
            if config.enable_killer_move_ordering
                && (killer_hashes[0] == child_hash || killer_hashes[1] == child_hash)
                && child_hash != 0
                && preferred_child_hash != Some(child_hash)
            {
                heuristic = heuristic.saturating_add(SMART_KILLER_MOVE_BONUS);
            }

            let own_drainer_vulnerable_after = if config.enable_child_move_class_coverage {
                Self::is_own_drainer_vulnerable_next_turn(
                    &simulated_game,
                    actor_color,
                    config.enable_enhanced_drainer_vulnerability,
                )
            } else {
                false
            };
            let own_drainer_vulnerability_transition =
                own_drainer_vulnerable_before != own_drainer_vulnerable_after;
            let tactical_extension_trigger = Self::events_include_any_drainer_fainted(&events)
                || own_drainer_vulnerability_transition
                || events
                    .iter()
                    .any(|event| matches!(event, Event::ManaScored { .. }));
            let classes = if config.enable_child_move_class_coverage {
                Self::classify_move_classes(
                    game,
                    &simulated_game,
                    actor_color,
                    &events,
                    own_drainer_vulnerable_before,
                    own_drainer_vulnerable_after,
                )
            } else {
                MoveClassFlags::default()
            };
            let quiet_reduction_candidate = Self::is_quiet_reduction_candidate(
                ordering_efficiency,
                tactical_extension_trigger,
                classes,
            );

            scored_states.push((
                heuristic,
                RankedChildState {
                    game: simulated_game,
                    hash: child_hash,
                    ordering_efficiency,
                    tactical_extension_trigger,
                    quiet_reduction_candidate,
                    classes,
                },
            ));
        }

        scored_states.sort_by(|a, b| Self::compare_ranked_child_entries(a, b, maximizing));

        if config.enable_child_move_class_coverage && scored_states.len() >= 3 {
            Self::enforce_tactical_child_top2(
                &mut scored_states,
                maximizing,
                config.enable_strict_tactical_class_coverage,
            );
        }

        if scored_states.len() > config.node_branch_limit {
            scored_states = Self::truncate_child_states_with_coverage(
                scored_states,
                config.node_branch_limit,
                maximizing,
                config.enable_strict_tactical_class_coverage,
            );
        }

        scored_states.into_iter().map(|(_, state)| state).collect()
    }

    fn enumerate_legal_transitions(
        game: &MonsGame,
        max_moves: usize,
        start_options: SuggestedStartInputOptions,
    ) -> Vec<LegalInputTransition> {
        let mut transitions = Vec::new();
        let mut partial_inputs = Vec::new();
        let mut simulated_game = game.clone_for_simulation();
        Self::collect_legal_transitions(
            &mut simulated_game,
            &mut partial_inputs,
            &mut transitions,
            max_moves,
            start_options,
        );
        transitions.sort_by(|a, b| a.inputs.cmp(&b.inputs));
        transitions
    }

    fn enumerate_legal_transitions_with_priority(
        game: &MonsGame,
        max_moves: usize,
        start_options: SuggestedStartInputOptions,
        priority_locations: &[Location],
    ) -> Vec<LegalInputTransition> {
        if priority_locations.is_empty() {
            return Self::enumerate_legal_transitions(game, max_moves, start_options);
        }

        let mut priority_mask = [false; BOARD_CELLS];
        for &location in priority_locations {
            priority_mask[location.index()] = true;
        }
        let priority_budget = (max_moves / 2).max(max_moves.saturating_sub(60));
        let remaining_budget = max_moves.saturating_sub(priority_budget);
        let all_transitions = Self::enumerate_legal_transitions(game, max_moves, start_options);
        let mut priority_transitions = Vec::new();
        let mut other_transitions = Vec::new();

        for transition in all_transitions {
            let is_priority = matches!(
                transition.inputs.first(),
                Some(Input::Location(loc)) if priority_mask[loc.index()]
            );
            if is_priority {
                if priority_transitions.len() < priority_budget {
                    priority_transitions.push(transition);
                }
            } else if remaining_budget > 0 && other_transitions.len() < remaining_budget {
                other_transitions.push(transition);
            }

            if priority_transitions.len() >= priority_budget
                && (remaining_budget == 0 || other_transitions.len() >= remaining_budget)
            {
                break;
            }
        }

        priority_transitions.extend(other_transitions);
        priority_transitions
    }

    fn collect_legal_transitions(
        game: &mut MonsGame,
        partial_inputs: &mut Vec<Input>,
        transitions: &mut Vec<LegalInputTransition>,
        max_moves: usize,
        start_options: SuggestedStartInputOptions,
    ) {
        if transitions.len() >= max_moves || partial_inputs.len() > SMART_MAX_INPUT_CHAIN {
            return;
        }

        match game.process_input_with_start_options_slice(
            partial_inputs.as_slice(),
            true,
            false,
            Some(start_options),
        ) {
            Output::InvalidInput => {}
            Output::Events(events) => {
                let mut after_game = game.clone_for_simulation();
                let applied_events = after_game.apply_and_add_resulting_events(events);
                transitions.push(LegalInputTransition {
                    inputs: partial_inputs.clone(),
                    game: after_game,
                    events: applied_events,
                });
            }
            Output::LocationsToStartFrom(locations) => {
                for location in locations {
                    if transitions.len() >= max_moves {
                        break;
                    }
                    partial_inputs.push(Input::Location(location));
                    Self::collect_legal_transitions(
                        game,
                        partial_inputs,
                        transitions,
                        max_moves,
                        start_options,
                    );
                    partial_inputs.pop();
                }
            }
            Output::NextInputOptions(options) => {
                for option in options {
                    if transitions.len() >= max_moves {
                        break;
                    }
                    partial_inputs.push(option.input);
                    Self::collect_legal_transitions(
                        game,
                        partial_inputs,
                        transitions,
                        max_moves,
                        start_options,
                    );
                    partial_inputs.pop();
                }
            }
        }
    }

    #[allow(dead_code)]
    fn enumerate_legal_inputs(
        game: &MonsGame,
        max_moves: usize,
        start_options: SuggestedStartInputOptions,
    ) -> Vec<Vec<Input>> {
        Self::enumerate_legal_transitions(game, max_moves, start_options)
            .into_iter()
            .map(|transition| transition.inputs)
            .collect()
    }

    #[allow(dead_code)]
    fn enumerate_legal_inputs_with_priority(
        game: &MonsGame,
        max_moves: usize,
        start_options: SuggestedStartInputOptions,
        priority_locations: &[Location],
    ) -> Vec<Vec<Input>> {
        Self::enumerate_legal_transitions_with_priority(
            game,
            max_moves,
            start_options,
            priority_locations,
        )
        .into_iter()
        .map(|transition| transition.inputs)
        .collect()
    }

    #[allow(dead_code)]
    fn collect_legal_inputs(
        game: &mut MonsGame,
        partial_inputs: &mut Vec<Input>,
        all_inputs: &mut Vec<Vec<Input>>,
        max_moves: usize,
        start_options: SuggestedStartInputOptions,
    ) {
        let mut transitions = Vec::new();
        Self::collect_legal_transitions(
            game,
            partial_inputs,
            &mut transitions,
            max_moves,
            start_options,
        );
        all_inputs.extend(transitions.into_iter().map(|transition| transition.inputs));
    }

    #[allow(dead_code)]
    fn apply_inputs_for_search(game: &MonsGame, inputs: &[Input]) -> Option<MonsGame> {
        Self::apply_inputs_for_search_with_events(game, inputs)
            .map(|(simulated_game, _)| simulated_game)
    }

    fn apply_inputs_for_search_with_events(
        game: &MonsGame,
        inputs: &[Input],
    ) -> Option<(MonsGame, Vec<Event>)> {
        let mut simulated_game = game.clone_for_simulation();
        match simulated_game.process_input_slice(inputs, false, false) {
            Output::Events(events) => Some((simulated_game, events)),
            _ => None,
        }
    }

    fn is_no_effect_turn_transition(
        game: &MonsGame,
        simulated_game: &MonsGame,
        events: &[Event],
    ) -> bool {
        if !Self::is_no_effect_state_transition(game, simulated_game) {
            return false;
        }
        !Self::has_material_event(events)
    }

    fn is_no_effect_state_transition(game: &MonsGame, simulated_game: &MonsGame) -> bool {
        game.board == simulated_game.board
            && game.white_score == simulated_game.white_score
            && game.black_score == simulated_game.black_score
            && game.white_potions_count == simulated_game.white_potions_count
            && game.black_potions_count == simulated_game.black_potions_count
    }

    fn has_material_event(events: &[Event]) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::ManaScored { .. }
                    | Event::PickupMana { .. }
                    | Event::MonFainted { .. }
                    | Event::UsePotion { .. }
                    | Event::PickupBomb { .. }
                    | Event::PickupPotion { .. }
                    | Event::BombAttack { .. }
                    | Event::BombExplosion { .. }
            )
        })
    }

    fn ordering_event_bonus(actor_color: Color, perspective: Color, events: &[Event]) -> i32 {
        let mut bonus = 0i32;
        for event in events {
            match event {
                Event::ManaScored { .. } => {
                    bonus += if actor_color == perspective {
                        780
                    } else {
                        -780
                    };
                }
                Event::PickupMana { .. } => {
                    bonus += if actor_color == perspective {
                        230
                    } else {
                        -230
                    };
                }
                Event::MonFainted { mon, .. } => {
                    bonus += if mon.color == perspective { -360 } else { 360 };
                }
                Event::UsePotion { .. } => {
                    bonus += if actor_color == perspective { -80 } else { 80 };
                }
                Event::PickupBomb { .. } | Event::PickupPotion { .. } => {
                    bonus += if actor_color == perspective { 45 } else { -45 };
                }
                Event::MonMove { .. }
                | Event::ManaMove { .. }
                | Event::MysticAction { .. }
                | Event::DemonAction { .. }
                | Event::DemonAdditionalStep { .. }
                | Event::SpiritTargetMove { .. }
                | Event::ManaDropped { .. }
                | Event::SupermanaBackToBase { .. }
                | Event::BombAttack { .. }
                | Event::MonAwake { .. }
                | Event::BombExplosion { .. }
                | Event::NextTurn { .. }
                | Event::GameOver { .. }
                | Event::Takeback => {}
            }
        }
        bonus
    }

    fn has_roundtrip_mon_move(events: &[Event]) -> bool {
        let mut seen_moves: Vec<(Location, Location, Color, MonKind)> =
            Vec::with_capacity(events.len().min(8));
        for event in events {
            let Event::MonMove { item, from, to } = event else {
                continue;
            };
            let Some(mon) = item.mon() else {
                continue;
            };
            let reverse = (*to, *from, mon.color, mon.kind);
            if seen_moves.iter().any(|seen| *seen == reverse) {
                return true;
            }
            seen_moves.push((*from, *to, mon.color, mon.kind));
        }
        false
    }

    fn root_transition_requires_active_turn_exact(
        events: &[Event],
        config: SmartSearchConfig,
    ) -> bool {
        let fast_mode = config.depth <= 2;
        events.iter().any(|event| match event {
            Event::MonMove { item, .. } => {
                matches!(item.mon().map(|mon| mon.kind), Some(MonKind::Spirit))
                    || (!fast_mode
                        && matches!(item.mon().map(|mon| mon.kind), Some(MonKind::Drainer)))
            }
            Event::ManaMove { .. }
            | Event::ManaScored { .. }
            | Event::PickupMana { .. }
            | Event::SpiritTargetMove { .. }
            | Event::MonFainted { .. }
            | Event::MysticAction { .. }
            | Event::DemonAction { .. }
            | Event::DemonAdditionalStep { .. }
            | Event::BombAttack { .. }
            | Event::BombExplosion { .. }
            | Event::ManaDropped { .. }
            | Event::SupermanaBackToBase { .. } => true,
            Event::UsePotion { .. }
            | Event::PickupBomb { .. }
            | Event::PickupPotion { .. }
            | Event::MonAwake { .. }
            | Event::NextTurn { .. }
            | Event::GameOver { .. }
            | Event::Takeback => false,
        })
    }

    fn approximate_active_turn_summary(game: &MonsGame, color: Color) -> ExactTurnSummary {
        let strategic = exact_strategic_analysis(game).color_summary(color);
        let safe_supermana_progress_steps =
            crate::models::automove_exact::exact_secure_specific_mana_steps_on_board(
                &game.board,
                color,
                Mana::Supermana,
                (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0),
            );
        let safe_opponent_mana_progress_steps =
            crate::models::automove_exact::exact_secure_specific_mana_steps_on_board(
                &game.board,
                color,
                Mana::Regular(color.other()),
                (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0),
            );
        ExactTurnSummary {
            color: Some(color),
            safe_supermana_progress: safe_supermana_progress_steps.is_some(),
            safe_supermana_progress_steps,
            safe_opponent_mana_progress: safe_opponent_mana_progress_steps.is_some(),
            safe_opponent_mana_progress_steps,
            same_turn_score_window_value: strategic.immediate_window.best_score,
            score_path_best_steps: strategic.score_path_window.best_steps,
            ..ExactTurnSummary::default()
        }
    }

    fn move_efficiency_delta(
        game: &MonsGame,
        simulated_game: &MonsGame,
        perspective: Color,
        events: &[Event],
        is_root: bool,
        apply_backtrack_penalty: bool,
        apply_root_mana_handoff_guard: bool,
        root_backtrack_penalty: i32,
        root_mana_handoff_penalty: i32,
    ) -> i32 {
        let before = Self::move_efficiency_snapshot(game, perspective, false);
        let after = Self::move_efficiency_snapshot(simulated_game, perspective, false);
        let unknown_steps = Config::BOARD_SIZE + 4;

        let mut delta = 0i32;
        delta += Self::step_progress_delta(
            before.my_best_carrier_steps,
            after.my_best_carrier_steps,
            90,
            130,
            unknown_steps,
        );
        delta -= Self::step_progress_delta(
            before.opponent_best_carrier_steps,
            after.opponent_best_carrier_steps,
            80,
            120,
            unknown_steps,
        );
        delta += Self::step_progress_delta(
            before.my_best_drainer_to_mana_steps,
            after.my_best_drainer_to_mana_steps,
            34,
            50,
            unknown_steps,
        );
        delta -= Self::step_progress_delta(
            before.opponent_best_drainer_to_mana_steps,
            after.opponent_best_drainer_to_mana_steps,
            30,
            44,
            unknown_steps,
        );
        delta += (after.my_carrier_count - before.my_carrier_count) * 55;
        delta -= (after.opponent_carrier_count - before.opponent_carrier_count) * 48;
        if before.my_spirit_on_base && !after.my_spirit_on_base {
            delta += SMART_SPIRIT_DEPLOY_EFFICIENCY_BONUS;
        }
        if !before.opponent_spirit_on_base && after.opponent_spirit_on_base {
            delta += SMART_SPIRIT_DEPLOY_EFFICIENCY_BONUS / 3;
        }
        delta += (after.my_spirit_action_targets - before.my_spirit_action_targets)
            * SMART_SPIRIT_ACTION_TARGET_DELTA_WEIGHT;
        delta -= (after.opponent_spirit_action_targets - before.opponent_spirit_action_targets)
            * (SMART_SPIRIT_ACTION_TARGET_DELTA_WEIGHT / 2);
        delta += (after.my_same_turn_score_value - before.my_same_turn_score_value) * 55;
        delta -=
            (after.opponent_same_turn_score_value - before.opponent_same_turn_score_value) * 45;
        delta += (after.my_same_turn_opponent_mana_score_value
            - before.my_same_turn_opponent_mana_score_value)
            * 90;
        delta -= (after.opponent_same_turn_opponent_mana_score_value
            - before.opponent_same_turn_opponent_mana_score_value)
            * 75;
        if !before.my_safe_supermana_progress && after.my_safe_supermana_progress {
            delta += 140;
        }
        if !before.opponent_safe_supermana_progress && after.opponent_safe_supermana_progress {
            delta -= 120;
        }
        if !before.my_safe_opponent_mana_progress && after.my_safe_opponent_mana_progress {
            delta += 120;
        }
        if !before.opponent_safe_opponent_mana_progress
            && after.opponent_safe_opponent_mana_progress
        {
            delta -= 110;
        }
        delta += Self::step_progress_delta(
            before.my_safe_supermana_progress_steps,
            after.my_safe_supermana_progress_steps,
            26,
            40,
            unknown_steps,
        );
        delta -= Self::step_progress_delta(
            before.opponent_safe_supermana_progress_steps,
            after.opponent_safe_supermana_progress_steps,
            22,
            36,
            unknown_steps,
        );
        delta += Self::step_progress_delta(
            before.my_safe_opponent_mana_progress_steps,
            after.my_safe_opponent_mana_progress_steps,
            22,
            34,
            unknown_steps,
        );
        delta -= Self::step_progress_delta(
            before.opponent_safe_opponent_mana_progress_steps,
            after.opponent_safe_opponent_mana_progress_steps,
            18,
            30,
            unknown_steps,
        );

        if is_root {
            let root_compensates_handoff = events
                .iter()
                .any(|event| matches!(event, Event::ManaScored { .. }))
                || Self::events_include_opponent_drainer_fainted(events, perspective);
            if apply_root_mana_handoff_guard && !root_compensates_handoff {
                delta -= Self::mana_handoff_penalty(events, perspective, root_mana_handoff_penalty);
            }

            if SMART_NO_EFFECT_ROOT_PENALTY != 0
                && Self::is_no_effect_turn_transition(game, simulated_game, events)
            {
                delta -= SMART_NO_EFFECT_ROOT_PENALTY;
            } else if SMART_LOW_IMPACT_ROOT_PENALTY != 0
                && !Self::has_material_event(events)
                && delta <= 0
            {
                delta -= SMART_LOW_IMPACT_ROOT_PENALTY;
            }
            if apply_backtrack_penalty
                && root_backtrack_penalty > 0
                && Self::has_roundtrip_mon_move(events)
            {
                delta -= root_backtrack_penalty;
            }
        } else if SMART_NO_EFFECT_CHILD_PENALTY != 0
            && Self::is_no_effect_state_transition(game, simulated_game)
        {
            delta -= SMART_NO_EFFECT_CHILD_PENALTY;
        } else if SMART_LOW_IMPACT_CHILD_PENALTY != 0
            && !Self::has_material_event(events)
            && delta < 0
        {
            delta -= SMART_LOW_IMPACT_CHILD_PENALTY;
        }

        delta
    }

    fn mana_handoff_penalty(events: &[Event], perspective: Color, per_step_penalty: i32) -> i32 {
        if per_step_penalty <= 0 {
            return 0;
        }
        let mut penalty = 0i32;
        let opponent = perspective.other();

        for event in events {
            let Event::ManaMove { mana, from, to } = event else {
                continue;
            };

            let my_before = Self::distance_to_color_pool_steps_for_efficiency(*from, perspective);
            let my_after = Self::distance_to_color_pool_steps_for_efficiency(*to, perspective);
            let opponent_before =
                Self::distance_to_color_pool_steps_for_efficiency(*from, opponent);
            let opponent_after = Self::distance_to_color_pool_steps_for_efficiency(*to, opponent);
            let moved_toward_opponent = (opponent_before - opponent_after).max(0);
            let moved_toward_me = (my_before - my_after).max(0);

            if moved_toward_opponent > moved_toward_me {
                penalty += (moved_toward_opponent - moved_toward_me)
                    * mana.score(opponent)
                    * per_step_penalty;
            }
        }

        penalty
    }

    fn move_efficiency_snapshot(
        game: &MonsGame,
        perspective: Color,
        include_tactical_exact: bool,
    ) -> MoveEfficiencySnapshot {
        let unknown_steps = Config::BOARD_SIZE + 4;
        let analysis = exact_strategic_analysis(game);
        let my_summary = analysis.color_summary(perspective);
        let opponent_summary = analysis.color_summary(perspective.other());
        let my_turn_summary = if include_tactical_exact && game.active_color == perspective {
            exact_turn_summary(game, perspective)
        } else {
            ExactTurnSummary {
                color: Some(perspective),
                same_turn_score_window_value: my_summary.immediate_window.best_score,
                ..ExactTurnSummary::default()
            }
        };
        let opponent_turn_summary =
            if include_tactical_exact && game.active_color == perspective.other() {
                exact_turn_summary(game, perspective.other())
            } else {
                ExactTurnSummary {
                    color: Some(perspective.other()),
                    same_turn_score_window_value: opponent_summary.immediate_window.best_score,
                    ..ExactTurnSummary::default()
                }
            };

        let mut snapshot = MoveEfficiencySnapshot {
            my_best_carrier_steps: my_summary.best_carrier_steps.unwrap_or(unknown_steps),
            opponent_best_carrier_steps: opponent_summary
                .best_carrier_steps
                .unwrap_or(unknown_steps),
            my_best_drainer_to_mana_steps: my_summary
                .best_drainer_to_mana_steps
                .unwrap_or(unknown_steps),
            opponent_best_drainer_to_mana_steps: opponent_summary
                .best_drainer_to_mana_steps
                .unwrap_or(unknown_steps),
            my_carrier_count: 0,
            opponent_carrier_count: 0,
            my_spirit_on_base: false,
            opponent_spirit_on_base: false,
            my_spirit_action_targets: my_summary.spirit.utility,
            opponent_spirit_action_targets: opponent_summary.spirit.utility,
            my_same_turn_score_value: if game.active_color == perspective {
                if include_tactical_exact {
                    my_turn_summary.spirit_assisted_score_value
                } else {
                    my_turn_summary.same_turn_score_window_value
                }
            } else {
                0
            },
            opponent_same_turn_score_value: if game.active_color == perspective.other() {
                if include_tactical_exact {
                    opponent_turn_summary.spirit_assisted_score_value
                } else {
                    opponent_turn_summary.same_turn_score_window_value
                }
            } else {
                0
            },
            my_same_turn_opponent_mana_score_value: if game.active_color == perspective {
                if include_tactical_exact {
                    my_turn_summary.spirit_assisted_denial_value
                } else {
                    0
                }
            } else {
                0
            },
            opponent_same_turn_opponent_mana_score_value: if game.active_color
                == perspective.other()
            {
                if include_tactical_exact {
                    opponent_turn_summary.spirit_assisted_denial_value
                } else {
                    0
                }
            } else {
                0
            },
            my_safe_supermana_progress: include_tactical_exact && my_turn_summary.safe_supermana_progress,
            opponent_safe_supermana_progress: include_tactical_exact
                && opponent_turn_summary.safe_supermana_progress,
            my_safe_opponent_mana_progress: include_tactical_exact
                && my_turn_summary.safe_opponent_mana_progress,
            opponent_safe_opponent_mana_progress: include_tactical_exact
                && opponent_turn_summary.safe_opponent_mana_progress,
            my_safe_supermana_progress_steps: my_turn_summary
                .safe_supermana_progress_steps
                .unwrap_or(unknown_steps),
            opponent_safe_supermana_progress_steps: opponent_turn_summary
                .safe_supermana_progress_steps
                .unwrap_or(unknown_steps),
            my_safe_opponent_mana_progress_steps: my_turn_summary
                .safe_opponent_mana_progress_steps
                .unwrap_or(unknown_steps),
            opponent_safe_opponent_mana_progress_steps: opponent_turn_summary
                .safe_opponent_mana_progress_steps
                .unwrap_or(unknown_steps),
        };
        let my_spirit_base = Self::spirit_base_for_color(&game.board, perspective);
        let opponent_spirit_base = Self::spirit_base_for_color(&game.board, perspective.other());

        for (location, item) in game.board.occupied() {
            match item {
                Item::MonWithMana { mon, .. } => {
                    if mon.is_fainted() {
                        continue;
                    }
                    let pool_steps =
                        Self::distance_to_any_pool_steps_for_efficiency(location).saturating_sub(1);
                    if mon.color == perspective {
                        snapshot.my_carrier_count += 1;
                        snapshot.my_best_carrier_steps =
                            snapshot.my_best_carrier_steps.min(pool_steps);
                    } else {
                        snapshot.opponent_carrier_count += 1;
                        snapshot.opponent_best_carrier_steps =
                            snapshot.opponent_best_carrier_steps.min(pool_steps);
                    }
                }
                Item::Mon { mon } | Item::MonWithConsumable { mon, .. } => {
                    if mon.is_fainted() {
                        continue;
                    }
                    if mon.kind == MonKind::Spirit {
                        if mon.color == perspective {
                            snapshot.my_spirit_on_base = location == my_spirit_base;
                        } else {
                            snapshot.opponent_spirit_on_base = location == opponent_spirit_base;
                        }
                    }
                }
                Item::Mana { .. } | Item::Consumable { .. } => {}
            }
        }

        snapshot
    }

    fn distance_to_any_pool_steps_for_efficiency(location: Location) -> i32 {
        let max_index = Config::MAX_LOCATION_INDEX;
        let i = location.i;
        let j = location.j;
        i32::max(i32::min(i, max_index - i), i32::min(j, max_index - j)) + 1
    }

    fn distance_to_color_pool_steps_for_efficiency(location: Location, color: Color) -> i32 {
        let max_index = Config::MAX_LOCATION_INDEX;
        let pool_row = if color == Color::White { max_index } else { 0 };
        let i = location.i;
        let j = location.j;
        i32::max((pool_row - i).abs(), i32::min(j, max_index - j)) + 1
    }

    fn step_progress_delta(
        before_steps: i32,
        after_steps: i32,
        forward_weight: i32,
        backward_weight: i32,
        unknown_steps: i32,
    ) -> i32 {
        let before_known = before_steps < unknown_steps;
        let after_known = after_steps < unknown_steps;
        match (before_known, after_known) {
            (true, true) => {
                let delta_steps = before_steps - after_steps;
                if delta_steps > 0 {
                    delta_steps * forward_weight
                } else if delta_steps < 0 {
                    delta_steps * backward_weight
                } else {
                    0
                }
            }
            (false, true) => forward_weight,
            (true, false) => -backward_weight,
            (false, false) => 0,
        }
    }

    fn score_state(
        game: &MonsGame,
        perspective: Color,
        depth: usize,
        search_depth: usize,
        scoring_weights: &'static ScoringWeights,
    ) -> i32 {
        if let Some(terminal_score) = Self::terminal_score(game, perspective, depth, search_depth) {
            terminal_score
        } else {
            evaluate_preferability_with_weights(game, perspective, scoring_weights)
        }
    }

    fn terminal_score(
        game: &MonsGame,
        perspective: Color,
        depth: usize,
        search_depth: usize,
    ) -> Option<i32> {
        game.winner_color().map(|winner| {
            let ply_count = (search_depth.saturating_sub(depth)) as i32;
            if winner == perspective {
                SMART_TERMINAL_SCORE - ply_count
            } else {
                -SMART_TERMINAL_SCORE + ply_count
            }
        })
    }

    pub(crate) fn search_state_hash(game: &MonsGame) -> u64 {
        let mut state = 0x6a09e667f3bcc909u64;
        for (idx, item) in game.board.items.iter().enumerate() {
            let Some(item) = item else { continue };
            let entry = ((idx as u64)
                .wrapping_add(1)
                .wrapping_mul(0x9e3779b185ebca87))
                ^ Self::search_hash_item(*item);
            state ^= Self::search_mix_u64(entry);
            state = state.rotate_left(17).wrapping_mul(0x94d049bb133111eb);
        }

        state ^= Self::search_mix_u64(game.white_score as i64 as u64 ^ 0x11);
        state ^= Self::search_mix_u64(game.black_score as i64 as u64 ^ 0x23);
        state ^= Self::search_mix_u64(Self::search_hash_color(game.active_color) ^ 0x35);
        state ^= Self::search_mix_u64(game.actions_used_count as i64 as u64 ^ 0x47);
        state ^= Self::search_mix_u64(game.mana_moves_count as i64 as u64 ^ 0x59);
        state ^= Self::search_mix_u64(game.mons_moves_count as i64 as u64 ^ 0x6b);
        state ^= Self::search_mix_u64(game.white_potions_count as i64 as u64 ^ 0x7d);
        state ^= Self::search_mix_u64(game.black_potions_count as i64 as u64 ^ 0x8f);
        state ^= Self::search_mix_u64(game.turn_number as i64 as u64 ^ 0xa1);
        Self::search_mix_u64(state)
    }

    #[inline]
    fn search_hash_item(item: Item) -> u64 {
        match item {
            Item::Mon { mon } => 0x100 | Self::search_hash_mon(mon),
            Item::Mana { mana } => 0x200 | Self::search_hash_mana(mana),
            Item::MonWithMana { mon, mana } => {
                0x300 | Self::search_hash_mon(mon) | (Self::search_hash_mana(mana) << 16)
            }
            Item::MonWithConsumable { mon, consumable } => {
                0x400
                    | Self::search_hash_mon(mon)
                    | (Self::search_hash_consumable(consumable) << 16)
            }
            Item::Consumable { consumable } => 0x500 | Self::search_hash_consumable(consumable),
        }
    }

    #[inline]
    fn search_hash_mon(mon: Mon) -> u64 {
        Self::search_hash_mon_kind(mon.kind)
            | (Self::search_hash_color(mon.color) << 4)
            | (((mon.cooldown as i64 as u64) & 0xff) << 8)
    }

    #[inline]
    fn search_hash_mon_kind(kind: MonKind) -> u64 {
        match kind {
            MonKind::Demon => 1,
            MonKind::Drainer => 2,
            MonKind::Angel => 3,
            MonKind::Spirit => 4,
            MonKind::Mystic => 5,
        }
    }

    #[inline]
    fn search_hash_color(color: Color) -> u64 {
        match color {
            Color::White => 1,
            Color::Black => 2,
        }
    }

    #[inline]
    fn search_hash_mana(mana: Mana) -> u64 {
        match mana {
            Mana::Regular(color) => 0x10 | Self::search_hash_color(color),
            Mana::Supermana => 0x20,
        }
    }

    #[inline]
    fn search_hash_consumable(consumable: Consumable) -> u64 {
        match consumable {
            Consumable::Potion => 1,
            Consumable::Bomb => 2,
            Consumable::BombOrPotion => 3,
        }
    }

    #[inline]
    fn search_mix_u64(mut value: u64) -> u64 {
        value = value.wrapping_add(0x9e3779b97f4a7c15);
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^ (value >> 31)
    }

    #[cfg(test)]
    fn advance_async_search(state: &mut AsyncSmartSearchState) -> bool {
        match state.phase {
            AsyncSmartSearchPhase::PendingRootRanking => {
                state.root_moves =
                    Self::ranked_root_moves(&state.game, state.perspective, state.config);
                state.phase = AsyncSmartSearchPhase::PendingFocusedCandidates;
                false
            }
            AsyncSmartSearchPhase::PendingFocusedCandidates => {
                if let Some(forced_inputs) = Self::forced_tactical_prepass_choice(
                    &state.game,
                    state.perspective,
                    state.root_moves.as_slice(),
                    state.config,
                ) {
                    let mut forced_game = state.game.clone_for_simulation();
                    let input_fen = Input::fen_from_array(&forced_inputs);
                    let output = forced_game.process_input(forced_inputs, false, false);
                    state.pending_output = Some(OutputModel::new(output, input_fen.as_str()));
                    return true;
                }

                let root_moves = std::mem::take(&mut state.root_moves);
                let (root_moves, scout_visited_nodes) = Self::focused_root_candidates(
                    &state.game,
                    state.perspective,
                    root_moves,
                    state.config,
                    true,
                );
                state.root_moves = root_moves;
                state.visited_nodes = scout_visited_nodes;
                state.extension_node_budget = if state.config.enable_selective_extensions
                    && state.config.selective_extension_node_share_bp > 0
                {
                    ((state.config.max_visited_nodes
                        * state.config.selective_extension_node_share_bp as usize)
                        / 10_000)
                        .max(1)
                } else {
                    0
                };
                state.phase = AsyncSmartSearchPhase::Scoring;
                state.next_index >= state.root_moves.len()
                    || state.visited_nodes >= state.config.max_visited_nodes
            }
            AsyncSmartSearchPhase::Scoring => {
                if state.next_index >= state.root_moves.len()
                    || state.visited_nodes >= state.config.max_visited_nodes
                {
                    return true;
                }

                let candidate = &state.root_moves[state.next_index];
                state.visited_nodes += 1;
                let candidate_score = Self::evaluate_root_candidate_score(
                    candidate,
                    state.perspective,
                    state.alpha,
                    &mut state.visited_nodes,
                    state.config,
                    &mut state.transposition_table,
                    &mut state.extension_nodes_used,
                    state.extension_node_budget,
                    true,
                    &mut state.killer_table,
                );

                state.scored_roots.push(RootEvaluation {
                    score: candidate_score,
                    efficiency: candidate.efficiency,
                    inputs: candidate.inputs.clone(),
                    game: candidate.game.clone(),
                    wins_immediately: candidate.wins_immediately,
                    attacks_opponent_drainer: candidate.attacks_opponent_drainer,
                    own_drainer_vulnerable: candidate.own_drainer_vulnerable,
                    own_drainer_walk_vulnerable: candidate.own_drainer_walk_vulnerable,
                    spirit_development: candidate.spirit_development,
                    keeps_awake_spirit_on_base: candidate.keeps_awake_spirit_on_base,
                    mana_handoff_to_opponent: candidate.mana_handoff_to_opponent,
                    has_roundtrip: candidate.has_roundtrip,
                    scores_supermana_this_turn: candidate.scores_supermana_this_turn,
                    scores_opponent_mana_this_turn: candidate.scores_opponent_mana_this_turn,
                    safe_supermana_pickup_now: candidate.safe_supermana_pickup_now,
                    safe_opponent_mana_pickup_now: candidate.safe_opponent_mana_pickup_now,
                    safe_supermana_progress_steps: candidate.safe_supermana_progress_steps,
                    safe_opponent_mana_progress_steps: candidate.safe_opponent_mana_progress_steps,
                    score_path_best_steps: candidate.score_path_best_steps,
                    same_turn_score_window_value: candidate.same_turn_score_window_value,
                    spirit_same_turn_score_setup_now: candidate.spirit_same_turn_score_setup_now,
                    spirit_own_mana_setup_now: candidate.spirit_own_mana_setup_now,
                    supermana_progress: candidate.supermana_progress,
                    opponent_mana_progress: candidate.opponent_mana_progress,
                    interview_soft_priority: candidate.interview_soft_priority,
                    classes: candidate.classes,
                });

                if candidate_score > state.alpha {
                    state.alpha = candidate_score;
                }

                state.next_index += 1;
                state.next_index >= state.root_moves.len()
                    || state.visited_nodes >= state.config.max_visited_nodes
            }
        }
    }

    #[cfg(test)]
    fn finalize_async_search(state: &mut AsyncSmartSearchState) -> OutputModel {
        if let Some(output) = state.pending_output.take() {
            return output;
        }
        if state.scored_roots.is_empty() {
            return Self::automove_game(&mut state.game);
        }

        let best_inputs = Self::pick_root_move_with_exploration(
            &state.game,
            &state.scored_roots,
            state.perspective,
            state.config,
        );
        let input_fen = Input::fen_from_array(&best_inputs);
        let output = state.game.process_input(best_inputs, false, false);
        OutputModel::new(output, input_fen.as_str())
    }

    fn events_include_opponent_drainer_fainted(events: &[Event], perspective: Color) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::MonFainted { mon, .. }
                    if mon.kind == MonKind::Drainer && mon.color == perspective.other()
            )
        })
    }

    fn events_include_any_drainer_fainted(events: &[Event]) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                Event::MonFainted { mon, .. } if mon.kind == MonKind::Drainer
            )
        })
    }

    pub(crate) fn is_location_guarded_by_angel(
        board: &Board,
        color: Color,
        location: Location,
    ) -> bool {
        board
            .find_awake_angel(color)
            .map_or(false, |angel_location| {
                angel_location.distance(&location) == 1
            })
    }

    #[inline]
    fn own_awake_drainer_location_and_fainted(
        board: &Board,
        perspective: Color,
    ) -> (Option<Location>, bool) {
        for (location, item) in board.occupied() {
            let Some(mon) = item.mon() else {
                continue;
            };
            if mon.color != perspective || mon.kind != MonKind::Drainer {
                continue;
            }
            if mon.is_fainted() {
                return (None, true);
            }
            return (Some(location), false);
        }
        (None, false)
    }

    fn own_drainer_carries_specific_mana_safely(
        board: &Board,
        perspective: Color,
        wanted_mana: Mana,
    ) -> bool {
        board.occupied().any(|(location, item)| {
            matches!(
                item,
                Item::MonWithMana { mon, mana }
                    if mon.color == perspective
                        && mon.kind == MonKind::Drainer
                        && !mon.is_fainted()
                        && *mana == wanted_mana
                        && crate::models::automove_exact::is_drainer_exactly_safe_next_turn_on_board(
                            board,
                            perspective,
                            location,
                        )
            )
        })
    }

    fn is_own_drainer_immediately_vulnerable_on_board(
        board: &Board,
        perspective: Color,
        opponent_to_move: bool,
        opponent_remaining_moves: i32,
        opponent_can_use_action: bool,
        _enhanced: bool,
    ) -> bool {
        let (own_drainer_location, own_drainer_fainted) =
            Self::own_awake_drainer_location_and_fainted(board, perspective);
        if own_drainer_fainted {
            return true;
        }
        let Some(own_drainer_location) = own_drainer_location else {
            return false;
        };

        if !opponent_to_move || !opponent_can_use_action {
            return false;
        }
        can_attack_target_on_board(
            board,
            perspective.other(),
            perspective,
            own_drainer_location,
            opponent_remaining_moves.max(0),
            opponent_can_use_action,
        )
    }

    #[allow(dead_code)]
    fn is_own_drainer_immediately_vulnerable(
        game: &MonsGame,
        perspective: Color,
        enhanced: bool,
    ) -> bool {
        Self::is_own_drainer_immediately_vulnerable_on_board(
            &game.board,
            perspective,
            game.active_color == perspective.other(),
            if game.active_color == perspective.other() {
                (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0)
            } else {
                0
            },
            game.player_can_use_action(),
            enhanced,
        )
    }

    fn is_own_drainer_vulnerable_next_turn(
        game: &MonsGame,
        perspective: Color,
        enhanced: bool,
    ) -> bool {
        Self::is_own_drainer_immediately_vulnerable_on_board(
            &game.board,
            perspective,
            true,
            Config::MONS_MOVES_PER_TURN,
            !game.is_first_turn(),
            enhanced,
        )
    }

    fn is_own_drainer_walk_vulnerable_next_turn(
        game: &MonsGame,
        perspective: Color,
        enhanced: bool,
    ) -> bool {
        let opponent = perspective.other();
        let opponent_can_use_action = !game.is_first_turn();
        if Self::is_own_drainer_immediately_vulnerable_on_board(
            &game.board,
            perspective,
            true,
            Config::MONS_MOVES_PER_TURN,
            opponent_can_use_action,
            enhanced,
        ) {
            return false;
        }
        let (drainer_location, own_drainer_fainted) =
            Self::own_awake_drainer_location_and_fainted(&game.board, perspective);
        if own_drainer_fainted {
            return false;
        }
        let Some(drainer_location) = drainer_location else {
            return false;
        };
        let drainer_angel_protected =
            Self::is_location_guarded_by_angel(&game.board, perspective, drainer_location);
        let _ = opponent;
        let _ = enhanced;
        is_drainer_under_walk_threat(
            &game.board,
            perspective,
            drainer_location,
            drainer_angel_protected,
        )
    }

    fn should_prefer_spirit_development(game: &MonsGame, perspective: Color) -> bool {
        !game.is_first_turn()
            && game.player_can_move_mon()
            && Self::has_awake_spirit_on_base(&game.board, perspective)
    }

    fn score_for_color(game: &MonsGame, color: Color) -> i32 {
        if color == Color::White {
            game.white_score
        } else {
            game.black_score
        }
    }

    fn potions_for_color(game: &MonsGame, color: Color) -> i32 {
        if color == Color::White {
            game.white_potions_count
        } else {
            game.black_potions_count
        }
    }

    fn should_prefer_potion_takeback_lines(game: &MonsGame, perspective: Color) -> bool {
        game.active_color == perspective
            && !game.is_first_turn()
            && !game.player_can_move_mon()
            && game.actions_used_count >= Config::ACTIONS_PER_TURN
            && game.player_can_move_mana()
            && Self::potions_for_color(game, perspective) > 0
    }

    fn root_spends_potion(
        game_before: &MonsGame,
        root: &RootEvaluation,
        perspective: Color,
    ) -> bool {
        Self::potions_for_color(&root.game, perspective)
            < Self::potions_for_color(game_before, perspective)
    }

    fn root_potion_spend_compensated(
        game_before: &MonsGame,
        root: &RootEvaluation,
        perspective: Color,
        config: SmartSearchConfig,
    ) -> bool {
        root.wins_immediately
            || root.attacks_opponent_drainer
            || Self::score_for_color(&root.game, perspective)
                >= Self::score_for_color(game_before, perspective).saturating_add(2)
            || root.scores_supermana_this_turn
            || root.scores_opponent_mana_this_turn
            || (config.enable_potion_progress_compensation
                && !root.own_drainer_vulnerable
                && (root.supermana_progress || root.opponent_mana_progress))
    }

    fn root_allows_immediate_opponent_win_quick(
        state_after_move: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
    ) -> bool {
        let reply_limit = config.root_anti_help_reply_limit.max(1);
        let snapshot =
            Self::root_reply_risk_snapshot(state_after_move, perspective, config, reply_limit);
        snapshot.allows_immediate_opponent_win
    }

    fn filtered_root_candidate_indices(
        game: &MonsGame,
        scored_roots: &[RootEvaluation],
        perspective: Color,
        config: SmartSearchConfig,
    ) -> Vec<usize> {
        if scored_roots.is_empty() {
            return Vec::new();
        }

        let mut candidate_indices = (0..scored_roots.len()).collect::<Vec<_>>();
        let mut forced_attack_applied = false;

        if candidate_indices
            .iter()
            .any(|index| scored_roots[*index].wins_immediately)
        {
            candidate_indices.retain(|index| scored_roots[*index].wins_immediately);
            return candidate_indices;
        }

        let should_filter_post_search = if config.enable_drainer_attack_full_pool {
            false
        } else if config.enable_conditional_forced_drainer_attack {
            let (my_score, opponent_score) = if perspective == Color::White {
                (game.white_score, game.black_score)
            } else {
                (game.black_score, game.white_score)
            };
            my_score <= opponent_score + config.conditional_forced_attack_score_margin
        } else {
            true
        };

        if config.enable_forced_drainer_attack
            && should_filter_post_search
            && candidate_indices
                .iter()
                .any(|index| scored_roots[*index].attacks_opponent_drainer)
        {
            candidate_indices.retain(|index| scored_roots[*index].attacks_opponent_drainer);
            forced_attack_applied = true;
        }

        if !forced_attack_applied
            && !candidate_indices
                .iter()
                .any(|index| scored_roots[*index].classes.immediate_score)
        {
            let best_score_window = candidate_indices
                .iter()
                .filter_map(|index| {
                    let root = &scored_roots[*index];
                    (!root.own_drainer_vulnerable && !root.mana_handoff_to_opponent)
                        .then_some(root.same_turn_score_window_value)
                })
                .max()
                .unwrap_or(0);
            if best_score_window > 0 {
                let window_indices = candidate_indices
                    .iter()
                    .copied()
                    .filter(|index| {
                        let root = &scored_roots[*index];
                        root.same_turn_score_window_value == best_score_window
                            && !root.own_drainer_vulnerable
                            && !root.mana_handoff_to_opponent
                    })
                    .collect::<Vec<_>>();
                if !window_indices.is_empty() {
                    candidate_indices = window_indices;
                }
            }
        }

        if !forced_attack_applied
            && candidate_indices
                .iter()
                .any(|index| scored_roots[*index].safe_supermana_pickup_now)
        {
            candidate_indices.retain(|index| {
                let root = &scored_roots[*index];
                root.scores_supermana_this_turn || root.safe_supermana_pickup_now
            });
        } else if !forced_attack_applied
            && candidate_indices
                .iter()
                .any(|index| scored_roots[*index].safe_opponent_mana_pickup_now)
        {
            candidate_indices.retain(|index| {
                let root = &scored_roots[*index];
                root.scores_opponent_mana_this_turn || root.safe_opponent_mana_pickup_now
            });
        }

        if !forced_attack_applied
            && !candidate_indices
                .iter()
                .any(|index| scored_roots[*index].classes.immediate_score)
        {
            let best_spirit_score_window = candidate_indices
                .iter()
                .filter_map(|index| {
                    let root = &scored_roots[*index];
                    (root.spirit_same_turn_score_setup_now
                        && !root.own_drainer_vulnerable
                        && !root.mana_handoff_to_opponent)
                        .then_some(root.same_turn_score_window_value)
                })
                .max()
                .unwrap_or(0);
            let best_non_spirit_score_window = candidate_indices
                .iter()
                .filter_map(|index| {
                    let root = &scored_roots[*index];
                    (!root.spirit_same_turn_score_setup_now
                        && !root.own_drainer_vulnerable
                        && !root.mana_handoff_to_opponent)
                        .then_some(root.same_turn_score_window_value)
                })
                .max()
                .unwrap_or(0);
            if best_spirit_score_window > best_non_spirit_score_window {
                candidate_indices.retain(|index| {
                    let root = &scored_roots[*index];
                    root.spirit_same_turn_score_setup_now
                        && root.same_turn_score_window_value == best_spirit_score_window
                        && !root.own_drainer_vulnerable
                        && !root.mana_handoff_to_opponent
                });
            }
        }

        if !forced_attack_applied
            && !candidate_indices
                .iter()
                .any(|index| scored_roots[*index].classes.immediate_score)
            && candidate_indices.iter().any(|index| {
                let root = &scored_roots[*index];
                root.same_turn_score_window_value > 0
                    && !root.own_drainer_vulnerable
                    && !root.mana_handoff_to_opponent
            })
        {
            let best_score_window = candidate_indices
                .iter()
                .map(|index| scored_roots[*index].same_turn_score_window_value)
                .max()
                .unwrap_or(0);
            if best_score_window > 0 {
                let window_indices = candidate_indices
                    .iter()
                    .copied()
                    .filter(|index| {
                        let root = &scored_roots[*index];
                        root.same_turn_score_window_value == best_score_window
                            && !root.own_drainer_vulnerable
                            && !root.mana_handoff_to_opponent
                    })
                    .collect::<Vec<_>>();
                if !window_indices.is_empty() {
                    candidate_indices = window_indices;
                }
            }
        }

        if !forced_attack_applied {
            let spirit_setup_indices = candidate_indices
                .iter()
                .copied()
                .filter(|index| {
                    let root = &scored_roots[*index];
                    root.spirit_own_mana_setup_now
                        && !root.own_drainer_vulnerable
                        && !root.mana_handoff_to_opponent
                })
                .collect::<Vec<_>>();
            if !spirit_setup_indices.is_empty() {
                let unknown_progress_steps = Config::BOARD_SIZE + 4;
                let spirit_supermana_indices = spirit_setup_indices
                    .iter()
                    .copied()
                    .filter(|index| scored_roots[*index].supermana_progress)
                    .collect::<Vec<_>>();
                if !spirit_supermana_indices.is_empty() {
                    let best_supermana_steps = spirit_supermana_indices
                        .iter()
                        .map(|index| scored_roots[*index].safe_supermana_progress_steps)
                        .filter(|steps| *steps < unknown_progress_steps)
                        .min();
                    candidate_indices = if let Some(best_steps) = best_supermana_steps {
                        spirit_supermana_indices
                            .into_iter()
                            .filter(|index| {
                                scored_roots[*index].safe_supermana_progress_steps == best_steps
                            })
                            .collect()
                    } else {
                        spirit_supermana_indices
                    };
                } else {
                    let spirit_opponent_indices = spirit_setup_indices
                        .iter()
                        .copied()
                        .filter(|index| scored_roots[*index].opponent_mana_progress)
                        .collect::<Vec<_>>();
                    if !spirit_opponent_indices.is_empty() {
                        let best_opponent_steps = spirit_opponent_indices
                            .iter()
                            .map(|index| scored_roots[*index].safe_opponent_mana_progress_steps)
                            .filter(|steps| *steps < unknown_progress_steps)
                            .min();
                        candidate_indices = if let Some(best_steps) = best_opponent_steps {
                            spirit_opponent_indices
                                .into_iter()
                                .filter(|index| {
                                    scored_roots[*index].safe_opponent_mana_progress_steps
                                        == best_steps
                                })
                                .collect()
                        } else {
                            spirit_opponent_indices
                        };
                    } else {
                        let unknown_score_path_steps = Config::BOARD_SIZE * 3;
                        let best_score_path_steps = spirit_setup_indices
                            .iter()
                            .map(|index| scored_roots[*index].score_path_best_steps)
                            .filter(|steps| *steps < unknown_score_path_steps)
                            .min();
                        candidate_indices = if let Some(best_steps) = best_score_path_steps {
                            spirit_setup_indices
                                .into_iter()
                                .filter(|index| {
                                    scored_roots[*index].score_path_best_steps == best_steps
                                })
                                .collect()
                        } else {
                            spirit_setup_indices
                        };
                    }
                }
            }
        }

        if config.enable_interview_hard_spirit_deploy
            && !forced_attack_applied
            && Self::should_prefer_spirit_development(game, perspective)
        {
            let has_safe_high_value_pickup = candidate_indices.iter().any(|index| {
                let root = &scored_roots[*index];
                root.scores_supermana_this_turn
                    || root.scores_opponent_mana_this_turn
                    || root.safe_supermana_pickup_now
                    || root.safe_opponent_mana_pickup_now
            });
            if has_safe_high_value_pickup {
                // Interview priority puts safe supermana/opponent-mana conversion above
                // default spirit deployment.
            } else {
                let spirit_setup_indices = candidate_indices
                    .iter()
                    .copied()
                    .filter(|index| {
                        let root = &scored_roots[*index];
                        root.spirit_own_mana_setup_now
                            && !root.own_drainer_vulnerable
                            && !root.mana_handoff_to_opponent
                    })
                    .collect::<Vec<_>>();
                if !spirit_setup_indices.is_empty() {
                    candidate_indices = spirit_setup_indices;
                } else {
                    let my_score_before = Self::score_for_color(game, perspective);
                    let spirit_ready_indices = candidate_indices
                        .iter()
                        .copied()
                        .filter(|index| !scored_roots[*index].keeps_awake_spirit_on_base)
                        .collect::<Vec<_>>();
                    if !spirit_ready_indices.is_empty() {
                        let safe_spirit_ready_indices = spirit_ready_indices
                            .iter()
                            .copied()
                            .filter(|index| {
                                let root = &scored_roots[*index];
                                !root.own_drainer_vulnerable && !root.mana_handoff_to_opponent
                            })
                            .collect::<Vec<_>>();
                        let preferred_spirit_indices = if !safe_spirit_ready_indices.is_empty() {
                            safe_spirit_ready_indices
                        } else {
                            spirit_ready_indices
                        };

                        let keeps_spirit_and_scores = candidate_indices.iter().any(|index| {
                            let root = &scored_roots[*index];
                            root.keeps_awake_spirit_on_base
                                && Self::score_for_color(&root.game, perspective) > my_score_before
                        });
                        let spirit_line_scores = preferred_spirit_indices.iter().any(|index| {
                            let root = &scored_roots[*index];
                            Self::score_for_color(&root.game, perspective) > my_score_before
                        });

                        if !keeps_spirit_and_scores || spirit_line_scores {
                            candidate_indices = preferred_spirit_indices;
                        }
                    }
                }
            }
        }

        if config.enable_root_drainer_safety_prefilter && !forced_attack_applied {
            let best_score = candidate_indices
                .iter()
                .map(|index| scored_roots[*index].score)
                .max()
                .unwrap_or(i32::MIN);
            let margin = config.root_drainer_safety_score_margin.max(0);
            let safer_indices = candidate_indices
                .iter()
                .copied()
                .filter(|index| {
                    let root = &scored_roots[*index];
                    !root.own_drainer_vulnerable && root.score + margin >= best_score
                })
                .collect::<Vec<_>>();
            if !safer_indices.is_empty() {
                if config.enable_walk_threat_prefilter {
                    let walk_margin = config.root_walk_threat_score_margin.max(0);
                    let walk_best = safer_indices
                        .iter()
                        .map(|index| scored_roots[*index].score)
                        .max()
                        .unwrap_or(i32::MIN);
                    let fully_safe_indices = safer_indices
                        .iter()
                        .copied()
                        .filter(|index| {
                            let root = &scored_roots[*index];
                            !root.own_drainer_walk_vulnerable
                                && root.score + walk_margin >= walk_best
                        })
                        .collect::<Vec<_>>();
                    if !fully_safe_indices.is_empty() {
                        candidate_indices = fully_safe_indices;
                    } else {
                        candidate_indices = safer_indices;
                    }
                } else {
                    candidate_indices = safer_indices;
                }
            }
        }

        if config.enable_root_spirit_development_pref
            && Self::should_prefer_spirit_development(game, perspective)
            && candidate_indices
                .iter()
                .any(|index| scored_roots[*index].spirit_development)
        {
            let has_safe_high_value_pickup = candidate_indices.iter().any(|index| {
                let root = &scored_roots[*index];
                root.scores_supermana_this_turn
                    || root.scores_opponent_mana_this_turn
                    || root.safe_supermana_pickup_now
                    || root.safe_opponent_mana_pickup_now
            });
            if has_safe_high_value_pickup {
                // Keep the safe high-value pickup candidates instead of forcing spirit deploy.
            } else {
                let best_score = candidate_indices
                    .iter()
                    .map(|index| scored_roots[*index].score)
                    .max()
                    .unwrap_or(i32::MIN);
                let margin = SMART_ROOT_SPIRIT_DEVELOPMENT_SCORE_MARGIN.max(0);
                let spirit_setup_indices = candidate_indices
                    .iter()
                    .copied()
                    .filter(|index| {
                        let root = &scored_roots[*index];
                        root.spirit_own_mana_setup_now && root.score + margin >= best_score
                    })
                    .collect::<Vec<_>>();
                if !spirit_setup_indices.is_empty() {
                    candidate_indices = spirit_setup_indices;
                } else {
                    let spirit_indices = candidate_indices
                        .iter()
                        .copied()
                        .filter(|index| {
                            let root = &scored_roots[*index];
                            root.spirit_development && root.score + margin >= best_score
                        })
                        .collect::<Vec<_>>();
                    if !spirit_indices.is_empty() {
                        candidate_indices = spirit_indices;
                    }
                }
            }
        }

        if config.enable_mana_start_mix_with_potion_actions
            && !forced_attack_applied
            && candidate_indices.len() > 1
            && Self::should_prefer_potion_takeback_lines(game, perspective)
        {
            let best_score = candidate_indices
                .iter()
                .map(|index| scored_roots[*index].score)
                .max()
                .unwrap_or(i32::MIN);
            let margin = SMART_ROOT_POTION_HOLD_SCORE_MARGIN.max(0);
            let near_best = candidate_indices
                .iter()
                .copied()
                .filter(|index| scored_roots[*index].score + margin >= best_score)
                .collect::<Vec<_>>();

            if near_best.len() > 1 {
                let mut quick_loss_cache = std::collections::HashMap::<usize, bool>::new();
                let has_non_potion_non_losing_alternative =
                    near_best.iter().copied().any(|index| {
                        let root = &scored_roots[index];
                        if Self::root_spends_potion(game, root, perspective) {
                            return false;
                        }
                        let allows_loss = *quick_loss_cache.entry(index).or_insert_with(|| {
                            Self::root_allows_immediate_opponent_win_quick(
                                &root.game,
                                perspective,
                                config,
                            )
                        });
                        !allows_loss
                    });

                if has_non_potion_non_losing_alternative {
                    let near_best_set = near_best
                        .iter()
                        .copied()
                        .collect::<std::collections::HashSet<_>>();
                    let strict_indices = candidate_indices
                        .iter()
                        .copied()
                        .filter(|index| {
                            let root = &scored_roots[*index];
                            if root.wins_immediately || !near_best_set.contains(index) {
                                return true;
                            }
                            !Self::root_spends_potion(game, root, perspective)
                                || Self::root_potion_spend_compensated(
                                    game,
                                    root,
                                    perspective,
                                    config,
                                )
                        })
                        .collect::<Vec<_>>();
                    if !strict_indices.is_empty() {
                        candidate_indices = strict_indices;
                    }
                }
            }
        }

        if config.enable_strict_anti_help_filter && candidate_indices.len() > 1 {
            let best_score = candidate_indices
                .iter()
                .map(|index| scored_roots[*index].score)
                .max()
                .unwrap_or(i32::MIN);
            let margin = config.root_anti_help_score_margin.max(0);
            let near_best = candidate_indices
                .iter()
                .copied()
                .filter(|index| scored_roots[*index].score + margin >= best_score)
                .collect::<Vec<_>>();
            if near_best.len() > 1 {
                let mut quick_loss_cache = std::collections::HashMap::<usize, bool>::new();
                let has_clean_non_losing_alternative = near_best.iter().copied().any(|index| {
                    let root = &scored_roots[index];
                    if root.mana_handoff_to_opponent || root.has_roundtrip {
                        return false;
                    }
                    let allows_loss = *quick_loss_cache.entry(index).or_insert_with(|| {
                        Self::root_allows_immediate_opponent_win_quick(
                            &root.game,
                            perspective,
                            config,
                        )
                    });
                    !allows_loss
                });

                if has_clean_non_losing_alternative {
                    let near_best_set = near_best
                        .iter()
                        .copied()
                        .collect::<std::collections::HashSet<_>>();
                    let mut strict_indices = candidate_indices
                        .iter()
                        .copied()
                        .filter(|index| {
                            let root = &scored_roots[*index];
                            if root.wins_immediately || !near_best_set.contains(index) {
                                return true;
                            }
                            !root.mana_handoff_to_opponent && !root.has_roundtrip
                        })
                        .collect::<Vec<_>>();
                    if !strict_indices.is_empty() {
                        candidate_indices = std::mem::take(&mut strict_indices);
                    }
                }
            }
        }

        candidate_indices
    }

    fn best_scored_root_index(
        scored_roots: &[RootEvaluation],
        candidate_indices: &[usize],
    ) -> usize {
        let mut best_index = candidate_indices.first().copied().unwrap_or(0);
        for index in candidate_indices.iter().copied() {
            if Self::compare_ranked_scored_root_indices(scored_roots, index, best_index)
                == std::cmp::Ordering::Less
            {
                best_index = index;
            }
        }
        best_index
    }

    fn pick_root_move_with_exploration(
        game: &MonsGame,
        scored_roots: &[RootEvaluation],
        perspective: Color,
        config: SmartSearchConfig,
    ) -> Vec<Input> {
        if scored_roots.is_empty() {
            return Vec::new();
        }

        let mut candidate_indices =
            Self::filtered_root_candidate_indices(game, scored_roots, perspective, config);
        if candidate_indices.is_empty() {
            candidate_indices = (0..scored_roots.len()).collect();
        }

        if config.enable_root_reply_risk_guard {
            if let Some(reply_guarded_index) = Self::pick_root_move_with_reply_risk_guard(
                game,
                scored_roots,
                candidate_indices.as_slice(),
                perspective,
                config,
            ) {
                return scored_roots[reply_guarded_index].inputs.clone();
            }
        }

        if !config.enable_root_efficiency {
            if config.enable_normal_root_safety_rerank
                && Self::should_apply_normal_root_safety(game, perspective)
            {
                return Self::pick_root_move_with_normal_safety(
                    game,
                    scored_roots,
                    candidate_indices.as_slice(),
                    perspective,
                    config,
                );
            }
            let best_index =
                Self::best_scored_root_index(scored_roots, candidate_indices.as_slice());
            return scored_roots[best_index].inputs.clone();
        }

        let best_score = candidate_indices
            .iter()
            .map(|index| scored_roots[*index].score)
            .max()
            .unwrap_or(i32::MIN);
        let score_margin = config.root_efficiency_score_margin.max(0);

        let mut best_index =
            Self::best_scored_root_index(scored_roots, candidate_indices.as_slice());
        let mut best_efficiency = i32::MIN;
        let mut best_shortlisted_score = i32::MIN;
        let prefer_spirit_development = config.enable_root_spirit_development_pref
            && Self::should_prefer_spirit_development(game, perspective);
        let mut best_spirit_development = scored_roots[best_index].spirit_development;
        let mut best_spirit_same_turn_score_setup =
            scored_roots[best_index].spirit_same_turn_score_setup_now;
        let mut best_spirit_own_mana_setup = scored_roots[best_index].spirit_own_mana_setup_now;
        let mut best_supermana_progress = scored_roots[best_index].supermana_progress;
        let mut best_opponent_mana_progress = scored_roots[best_index].opponent_mana_progress;
        let mut best_score_path_best_steps = scored_roots[best_index].score_path_best_steps;
        let mut best_safe_supermana_progress_steps =
            scored_roots[best_index].safe_supermana_progress_steps;
        let mut best_safe_opponent_mana_progress_steps =
            scored_roots[best_index].safe_opponent_mana_progress_steps;
        let mut best_same_turn_score_window_value =
            scored_roots[best_index].same_turn_score_window_value;
        let mut best_mana_handoff = scored_roots[best_index].mana_handoff_to_opponent;
        let mut best_has_roundtrip = scored_roots[best_index].has_roundtrip;
        let mut best_soft_priority = scored_roots[best_index].interview_soft_priority;

        for index in candidate_indices {
            let evaluation = &scored_roots[index];
            if evaluation.score + score_margin < best_score {
                continue;
            }
            let spirit_better = prefer_spirit_development
                && evaluation.spirit_development
                && !best_spirit_development;
            let equal_spirit_preference = !prefer_spirit_development
                || evaluation.spirit_development == best_spirit_development;
            let spirit_same_turn_score_setup_better =
                evaluation.spirit_same_turn_score_setup_now && !best_spirit_same_turn_score_setup;
            let equal_spirit_same_turn_score_setup =
                evaluation.spirit_same_turn_score_setup_now == best_spirit_same_turn_score_setup;
            let spirit_setup_better =
                evaluation.spirit_own_mana_setup_now && !best_spirit_own_mana_setup;
            let equal_spirit_setup =
                evaluation.spirit_own_mana_setup_now == best_spirit_own_mana_setup;
            let spirit_setup_supermana_progress_steps_better = evaluation.spirit_own_mana_setup_now
                && best_spirit_own_mana_setup
                && evaluation.supermana_progress
                && best_supermana_progress
                && Self::root_progress_steps_better(
                    evaluation.safe_supermana_progress_steps,
                    best_safe_supermana_progress_steps,
                );
            let equal_spirit_setup_supermana_progress_steps = !evaluation.spirit_own_mana_setup_now
                || !best_spirit_own_mana_setup
                || !evaluation.supermana_progress
                || !best_supermana_progress
                || evaluation.safe_supermana_progress_steps == best_safe_supermana_progress_steps;
            let spirit_setup_opponent_mana_progress_steps_better = evaluation
                .spirit_own_mana_setup_now
                && best_spirit_own_mana_setup
                && evaluation.opponent_mana_progress
                && best_opponent_mana_progress
                && Self::root_progress_steps_better(
                    evaluation.safe_opponent_mana_progress_steps,
                    best_safe_opponent_mana_progress_steps,
                );
            let equal_spirit_setup_opponent_mana_progress_steps = !evaluation
                .spirit_own_mana_setup_now
                || !best_spirit_own_mana_setup
                || !evaluation.opponent_mana_progress
                || !best_opponent_mana_progress
                || evaluation.safe_opponent_mana_progress_steps
                    == best_safe_opponent_mana_progress_steps;
            let spirit_setup_score_path_better = evaluation.spirit_own_mana_setup_now
                && best_spirit_own_mana_setup
                && Self::root_score_path_steps_better(
                    evaluation.score_path_best_steps,
                    best_score_path_best_steps,
                );
            let equal_spirit_setup_score_path = !evaluation.spirit_own_mana_setup_now
                || !best_spirit_own_mana_setup
                || evaluation.score_path_best_steps == best_score_path_best_steps;
            let supermana_progress_steps_better = Self::root_progress_steps_better(
                evaluation.safe_supermana_progress_steps,
                best_safe_supermana_progress_steps,
            );
            let equal_supermana_progress_steps =
                evaluation.safe_supermana_progress_steps == best_safe_supermana_progress_steps;
            let opponent_mana_progress_steps_better = Self::root_progress_steps_better(
                evaluation.safe_opponent_mana_progress_steps,
                best_safe_opponent_mana_progress_steps,
            );
            let equal_opponent_mana_progress_steps = evaluation.safe_opponent_mana_progress_steps
                == best_safe_opponent_mana_progress_steps;
            let same_turn_score_window_better =
                evaluation.same_turn_score_window_value > best_same_turn_score_window_value;
            let equal_same_turn_score_window =
                evaluation.same_turn_score_window_value == best_same_turn_score_window_value;
            let handoff_better = !evaluation.mana_handoff_to_opponent && best_mana_handoff;
            let equal_handoff = evaluation.mana_handoff_to_opponent == best_mana_handoff;
            let roundtrip_better = !evaluation.has_roundtrip && best_has_roundtrip;
            let equal_roundtrip = evaluation.has_roundtrip == best_has_roundtrip;
            let soft_better = config.enable_interview_soft_root_priors
                && evaluation.interview_soft_priority
                    > best_soft_priority.saturating_add(config.interview_soft_score_margin.max(0));
            let soft_equal_or_disabled = !config.enable_interview_soft_root_priors
                || evaluation
                    .interview_soft_priority
                    .saturating_add(config.interview_soft_score_margin.max(0))
                    >= best_soft_priority;
            let efficiency_or_score_better = evaluation.efficiency > best_efficiency
                || (evaluation.efficiency == best_efficiency
                    && evaluation.score > best_shortlisted_score);
            let tie_break_better = if soft_better {
                true
            } else if !soft_equal_or_disabled {
                false
            } else if same_turn_score_window_better {
                true
            } else if !equal_same_turn_score_window {
                false
            } else if spirit_same_turn_score_setup_better {
                true
            } else if !equal_spirit_same_turn_score_setup {
                false
            } else if spirit_setup_better {
                true
            } else if !equal_spirit_setup {
                false
            } else if spirit_setup_supermana_progress_steps_better {
                true
            } else if !equal_spirit_setup_supermana_progress_steps {
                false
            } else if spirit_setup_opponent_mana_progress_steps_better {
                true
            } else if !equal_spirit_setup_opponent_mana_progress_steps {
                false
            } else if spirit_setup_score_path_better {
                true
            } else if !equal_spirit_setup_score_path {
                false
            } else if supermana_progress_steps_better {
                true
            } else if !equal_supermana_progress_steps {
                false
            } else if opponent_mana_progress_steps_better {
                true
            } else if !equal_opponent_mana_progress_steps {
                false
            } else if handoff_better {
                true
            } else if !equal_handoff {
                false
            } else if roundtrip_better {
                true
            } else if !equal_roundtrip {
                false
            } else {
                efficiency_or_score_better
            };
            if spirit_better || (equal_spirit_preference && tie_break_better) {
                best_index = index;
                best_efficiency = evaluation.efficiency;
                best_shortlisted_score = evaluation.score;
                best_spirit_development = evaluation.spirit_development;
                best_spirit_same_turn_score_setup = evaluation.spirit_same_turn_score_setup_now;
                best_spirit_own_mana_setup = evaluation.spirit_own_mana_setup_now;
                best_supermana_progress = evaluation.supermana_progress;
                best_opponent_mana_progress = evaluation.opponent_mana_progress;
                best_score_path_best_steps = evaluation.score_path_best_steps;
                best_safe_supermana_progress_steps = evaluation.safe_supermana_progress_steps;
                best_safe_opponent_mana_progress_steps =
                    evaluation.safe_opponent_mana_progress_steps;
                best_same_turn_score_window_value = evaluation.same_turn_score_window_value;
                best_mana_handoff = evaluation.mana_handoff_to_opponent;
                best_has_roundtrip = evaluation.has_roundtrip;
                best_soft_priority = evaluation.interview_soft_priority;
            }
        }

        scored_roots[best_index].inputs.clone()
    }

    fn pick_root_move_with_reply_risk_guard(
        _game: &MonsGame,
        scored_roots: &[RootEvaluation],
        candidate_indices: &[usize],
        perspective: Color,
        config: SmartSearchConfig,
    ) -> Option<usize> {
        if candidate_indices.is_empty() {
            return None;
        }

        let best_score = candidate_indices
            .iter()
            .map(|index| scored_roots[*index].score)
            .max()
            .unwrap_or(i32::MIN);
        let worst_score = candidate_indices
            .iter()
            .map(|index| scored_roots[*index].score)
            .min()
            .unwrap_or(i32::MIN);
        let has_winning_candidate = candidate_indices
            .iter()
            .any(|index| scored_roots[*index].wins_immediately);
        if has_winning_candidate
            && best_score.saturating_sub(worst_score) > SMART_ROOT_REPLY_RISK_WINNER_SPREAD_SKIP
        {
            return None;
        }

        let score_margin = config.root_reply_risk_score_margin.max(0);
        let mut shortlist = candidate_indices
            .iter()
            .copied()
            .filter(|index| scored_roots[*index].score + score_margin >= best_score)
            .collect::<Vec<_>>();
        shortlist.sort_by(|a, b| Self::compare_ranked_scored_root_indices(scored_roots, *a, *b));
        if shortlist.is_empty() {
            return None;
        }
        if shortlist.len() > config.root_reply_risk_shortlist_max.max(1) {
            shortlist.truncate(config.root_reply_risk_shortlist_max.max(1));
        }

        let root_node_budget = ((config.max_visited_nodes
            * config.root_reply_risk_node_share_bp.max(0) as usize)
            / 10_000)
            .max(shortlist.len())
            .max(1);
        let per_root_reply_limit = (root_node_budget / shortlist.len().max(1))
            .max(1)
            .min(config.root_reply_risk_reply_limit.max(1));

        let mut best_index = shortlist[0];
        let mut best_snapshot = Self::root_reply_risk_snapshot(
            &scored_roots[best_index].game,
            perspective,
            config,
            per_root_reply_limit,
        );
        for index in shortlist.iter().copied().skip(1) {
            let snapshot = Self::root_reply_risk_snapshot(
                &scored_roots[index].game,
                perspective,
                config,
                per_root_reply_limit,
            );
            if Self::is_better_reply_risk_candidate(
                index,
                snapshot,
                best_index,
                best_snapshot,
                scored_roots,
                config,
            ) {
                best_index = index;
                best_snapshot = snapshot;
            }
        }
        Some(best_index)
    }

    fn root_reply_risk_snapshot(
        state_after_move: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        reply_limit: usize,
    ) -> RootReplyRiskSnapshot {
        if let Some(winner) = state_after_move.winner_color() {
            return if winner == perspective {
                RootReplyRiskSnapshot {
                    allows_immediate_opponent_win: false,
                    opponent_reaches_match_point: false,
                    worst_reply_score: SMART_TERMINAL_SCORE / 2,
                }
            } else {
                RootReplyRiskSnapshot {
                    allows_immediate_opponent_win: true,
                    opponent_reaches_match_point: true,
                    worst_reply_score: -SMART_TERMINAL_SCORE / 2,
                }
            };
        }

        if state_after_move.active_color == perspective {
            return RootReplyRiskSnapshot {
                allows_immediate_opponent_win: false,
                opponent_reaches_match_point: false,
                worst_reply_score: evaluate_preferability_with_weights(
                    state_after_move,
                    perspective,
                    config.scoring_weights,
                ),
            };
        }

        let replies = Self::enumerate_legal_transitions(
            state_after_move,
            reply_limit.max(1),
            Self::automove_start_input_options(config),
        );
        if replies.is_empty() {
            return RootReplyRiskSnapshot {
                allows_immediate_opponent_win: false,
                opponent_reaches_match_point: false,
                worst_reply_score: SMART_TERMINAL_SCORE / 4,
            };
        }

        let mut allows_immediate_opponent_win = false;
        let mut opponent_reaches_match_point = false;
        let mut worst_reply_score = i32::MAX;
        let mut evaluated_reply = false;

        for reply in replies {
            let after_reply = reply.game;
            evaluated_reply = true;

            let opponent_score_after = if perspective == Color::White {
                after_reply.black_score
            } else {
                after_reply.white_score
            };
            if Config::TARGET_SCORE - opponent_score_after <= 1 {
                opponent_reaches_match_point = true;
            }

            let reply_score = match after_reply.winner_color() {
                Some(winner) if winner == perspective => SMART_TERMINAL_SCORE / 2,
                Some(_) => {
                    allows_immediate_opponent_win = true;
                    opponent_reaches_match_point = true;
                    -SMART_TERMINAL_SCORE / 2
                }
                None => evaluate_preferability_with_weights(
                    &after_reply,
                    perspective,
                    config.scoring_weights,
                ),
            };
            worst_reply_score = worst_reply_score.min(reply_score);

            if allows_immediate_opponent_win {
                break;
            }
        }

        if !evaluated_reply || worst_reply_score == i32::MAX {
            worst_reply_score = evaluate_preferability_with_weights(
                state_after_move,
                perspective,
                config.scoring_weights,
            );
        }

        RootReplyRiskSnapshot {
            allows_immediate_opponent_win,
            opponent_reaches_match_point,
            worst_reply_score,
        }
    }

    fn is_better_reply_risk_candidate(
        candidate_index: usize,
        candidate_snapshot: RootReplyRiskSnapshot,
        incumbent_index: usize,
        incumbent_snapshot: RootReplyRiskSnapshot,
        scored_roots: &[RootEvaluation],
        config: SmartSearchConfig,
    ) -> bool {
        let candidate = &scored_roots[candidate_index];
        let incumbent = &scored_roots[incumbent_index];

        if candidate.wins_immediately != incumbent.wins_immediately {
            return candidate.wins_immediately;
        }
        if candidate.attacks_opponent_drainer != incumbent.attacks_opponent_drainer {
            return candidate.attacks_opponent_drainer;
        }
        if candidate_snapshot.allows_immediate_opponent_win
            != incumbent_snapshot.allows_immediate_opponent_win
        {
            return !candidate_snapshot.allows_immediate_opponent_win;
        }
        if candidate_snapshot.opponent_reaches_match_point
            != incumbent_snapshot.opponent_reaches_match_point
        {
            return !candidate_snapshot.opponent_reaches_match_point;
        }
        if config.enable_mana_start_mix_with_potion_actions && config.enable_move_class_coverage {
            let candidate_progress_adv =
                candidate.classes.carrier_progress && !incumbent.classes.carrier_progress;
            let incumbent_progress_adv =
                incumbent.classes.carrier_progress && !candidate.classes.carrier_progress;
            if candidate_progress_adv || incumbent_progress_adv {
                let no_tactical_priority = !candidate.classes.is_tactical_priority()
                    && !incumbent.classes.is_tactical_priority();
                let tight_reply_floor = (candidate_snapshot.worst_reply_score
                    - incumbent_snapshot.worst_reply_score)
                    .abs()
                    <= 80;
                if no_tactical_priority && tight_reply_floor {
                    return candidate_progress_adv;
                }
            }
        }
        if config.prefer_clean_reply_risk_roots {
            if candidate.mana_handoff_to_opponent != incumbent.mana_handoff_to_opponent {
                return !candidate.mana_handoff_to_opponent;
            }
            if candidate.has_roundtrip != incumbent.has_roundtrip {
                return !candidate.has_roundtrip;
            }
        }
        if config.enable_interview_deterministic_tiebreak
            && candidate.spirit_own_mana_setup_now != incumbent.spirit_own_mana_setup_now
        {
            return candidate.spirit_own_mana_setup_now;
        }
        if config.enable_interview_deterministic_tiebreak
            && candidate.spirit_own_mana_setup_now
            && incumbent.spirit_own_mana_setup_now
            && candidate.supermana_progress
            && incumbent.supermana_progress
            && candidate.safe_supermana_progress_steps != incumbent.safe_supermana_progress_steps
        {
            return Self::root_progress_steps_better(
                candidate.safe_supermana_progress_steps,
                incumbent.safe_supermana_progress_steps,
            );
        }
        if config.enable_interview_deterministic_tiebreak
            && candidate.spirit_own_mana_setup_now
            && incumbent.spirit_own_mana_setup_now
            && candidate.opponent_mana_progress
            && incumbent.opponent_mana_progress
            && candidate.safe_opponent_mana_progress_steps
                != incumbent.safe_opponent_mana_progress_steps
        {
            return Self::root_progress_steps_better(
                candidate.safe_opponent_mana_progress_steps,
                incumbent.safe_opponent_mana_progress_steps,
            );
        }
        if config.enable_interview_deterministic_tiebreak
            && candidate.spirit_own_mana_setup_now
            && incumbent.spirit_own_mana_setup_now
            && candidate.score_path_best_steps != incumbent.score_path_best_steps
        {
            return Self::root_score_path_steps_better(
                candidate.score_path_best_steps,
                incumbent.score_path_best_steps,
            );
        }
        if config.enable_interview_deterministic_tiebreak
            && candidate.spirit_development != incumbent.spirit_development
        {
            return candidate.spirit_development;
        }
        if config.enable_interview_soft_root_priors
            && candidate.interview_soft_priority != incumbent.interview_soft_priority
        {
            return candidate.interview_soft_priority > incumbent.interview_soft_priority;
        }
        if candidate_snapshot.worst_reply_score != incumbent_snapshot.worst_reply_score {
            return candidate_snapshot.worst_reply_score > incumbent_snapshot.worst_reply_score;
        }
        if candidate.score != incumbent.score {
            return candidate.score > incumbent.score;
        }
        if candidate.efficiency != incumbent.efficiency {
            return candidate.efficiency > incumbent.efficiency;
        }
        false
    }

    fn should_apply_normal_root_safety(game: &MonsGame, perspective: Color) -> bool {
        let (my_score, opponent_score) = if perspective == Color::White {
            (game.white_score, game.black_score)
        } else {
            (game.black_score, game.white_score)
        };
        let my_distance_to_win = Config::TARGET_SCORE - my_score;
        let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
        my_distance_to_win <= SMART_NORMAL_ROOT_SAFETY_SCORE_RACE_TRIGGER
            || opponent_distance_to_win <= SMART_NORMAL_ROOT_SAFETY_SCORE_RACE_TRIGGER
    }

    fn pick_root_move_with_normal_safety(
        game: &MonsGame,
        scored_roots: &[RootEvaluation],
        candidate_indices: &[usize],
        perspective: Color,
        config: SmartSearchConfig,
    ) -> Vec<Input> {
        if scored_roots.is_empty() {
            return Vec::new();
        }

        let eligible_indices = if candidate_indices.is_empty() {
            (0..scored_roots.len()).collect::<Vec<_>>()
        } else {
            candidate_indices.to_vec()
        };
        let best_score = eligible_indices
            .iter()
            .map(|index| scored_roots[*index].score)
            .max()
            .unwrap_or(i32::MIN);
        let score_margin = SMART_NORMAL_ROOT_SAFETY_SCORE_MARGIN.max(0);
        let reply_limit = config.node_enum_limit.clamp(
            SMART_NORMAL_ROOT_SAFETY_REPLY_LIMIT_MIN,
            SMART_NORMAL_ROOT_SAFETY_REPLY_LIMIT_MAX,
        );
        let my_score_before = if perspective == Color::White {
            game.white_score
        } else {
            game.black_score
        };
        let shortlist_limit = SMART_NORMAL_ROOT_SAFETY_SHORTLIST_MAX.max(1);

        let mut shortlist = eligible_indices
            .iter()
            .copied()
            .filter(|index| scored_roots[*index].score + score_margin >= best_score)
            .collect::<Vec<_>>();
        shortlist.sort_by(|a, b| Self::compare_ranked_scored_root_indices(scored_roots, *a, *b));
        if shortlist.len() > shortlist_limit {
            shortlist.truncate(shortlist_limit);
        }

        if shortlist.is_empty() {
            let best_index =
                Self::best_scored_root_index(scored_roots, eligible_indices.as_slice());
            return scored_roots[best_index].inputs.clone();
        }

        let best_scored_index = shortlist[0];
        if shortlist.len() < 2 {
            return scored_roots[best_scored_index].inputs.clone();
        }

        let shortlist_indices = shortlist;
        let start_options = Self::automove_start_input_options(config);

        let mut best_index = best_scored_index;
        let mut best_snapshot = Self::normal_root_safety_snapshot(
            &scored_roots[best_scored_index].game,
            perspective,
            my_score_before,
            config.scoring_weights,
            reply_limit,
            start_options,
        );

        for index in shortlist_indices.iter().copied().skip(1) {
            let evaluation = &scored_roots[index];
            let snapshot = Self::normal_root_safety_snapshot(
                &evaluation.game,
                perspective,
                my_score_before,
                config.scoring_weights,
                reply_limit,
                start_options,
            );
            if Self::is_better_normal_root_safety_candidate(
                snapshot,
                evaluation.score,
                best_snapshot,
                scored_roots[best_index].score,
            ) {
                best_index = index;
                best_snapshot = snapshot;
            }
        }

        let selected_index = if config.enable_normal_root_safety_deep_floor
            && Self::should_apply_normal_root_safety_deep_floor(game, perspective)
        {
            Self::pick_normal_root_with_deep_floor(
                scored_roots,
                shortlist_indices.as_slice(),
                best_index,
                perspective,
                config,
            )
        } else {
            best_index
        };

        scored_roots[selected_index].inputs.clone()
    }

    fn normal_root_safety_snapshot(
        state_after_move: &MonsGame,
        perspective: Color,
        my_score_before: i32,
        scoring_weights: &'static ScoringWeights,
        reply_limit: usize,
        start_options: SuggestedStartInputOptions,
    ) -> NormalRootSafetySnapshot {
        let my_score_after_move = if perspective == Color::White {
            state_after_move.white_score
        } else {
            state_after_move.black_score
        };
        let my_score_gain = (my_score_after_move - my_score_before).max(0);

        if let Some(winner) = state_after_move.winner_color() {
            return if winner == perspective {
                NormalRootSafetySnapshot {
                    allows_immediate_opponent_win: false,
                    opponent_reaches_match_point: false,
                    opponent_max_score_gain: 0,
                    my_score_gain,
                    worst_reply_score: SMART_TERMINAL_SCORE / 2,
                }
            } else {
                NormalRootSafetySnapshot {
                    allows_immediate_opponent_win: true,
                    opponent_reaches_match_point: true,
                    opponent_max_score_gain: Config::TARGET_SCORE,
                    my_score_gain,
                    worst_reply_score: -SMART_TERMINAL_SCORE / 2,
                }
            };
        }

        if state_after_move.active_color == perspective {
            return NormalRootSafetySnapshot {
                allows_immediate_opponent_win: false,
                opponent_reaches_match_point: false,
                opponent_max_score_gain: 0,
                my_score_gain,
                worst_reply_score: evaluate_preferability_with_weights(
                    state_after_move,
                    perspective,
                    scoring_weights,
                ),
            };
        }

        let opponent_score_before = if perspective == Color::White {
            state_after_move.black_score
        } else {
            state_after_move.white_score
        };
        let replies =
            Self::enumerate_legal_transitions(state_after_move, reply_limit.max(1), start_options);
        if replies.is_empty() {
            return NormalRootSafetySnapshot {
                allows_immediate_opponent_win: false,
                opponent_reaches_match_point: false,
                opponent_max_score_gain: 0,
                my_score_gain,
                worst_reply_score: SMART_TERMINAL_SCORE / 4,
            };
        }

        let mut allows_immediate_opponent_win = false;
        let mut opponent_reaches_match_point = false;
        let mut opponent_max_score_gain = 0i32;
        let mut worst_reply_score = i32::MAX;
        let mut evaluated_reply = false;

        for reply in replies {
            let after_reply = reply.game;
            evaluated_reply = true;

            let opponent_score_after = if perspective == Color::White {
                after_reply.black_score
            } else {
                after_reply.white_score
            };
            let score_gain = (opponent_score_after - opponent_score_before).max(0);
            opponent_max_score_gain = opponent_max_score_gain.max(score_gain);
            if Config::TARGET_SCORE - opponent_score_after <= 1 {
                opponent_reaches_match_point = true;
            }

            let reply_score = match after_reply.winner_color() {
                Some(winner) if winner == perspective => SMART_TERMINAL_SCORE / 2,
                Some(_) => {
                    allows_immediate_opponent_win = true;
                    opponent_reaches_match_point = true;
                    -SMART_TERMINAL_SCORE / 2
                }
                None => {
                    evaluate_preferability_with_weights(&after_reply, perspective, scoring_weights)
                }
            };
            worst_reply_score = worst_reply_score.min(reply_score);

            if allows_immediate_opponent_win {
                break;
            }
        }

        if !evaluated_reply || worst_reply_score == i32::MAX {
            worst_reply_score =
                evaluate_preferability_with_weights(state_after_move, perspective, scoring_weights);
        }

        NormalRootSafetySnapshot {
            allows_immediate_opponent_win,
            opponent_reaches_match_point,
            opponent_max_score_gain,
            my_score_gain,
            worst_reply_score,
        }
    }

    fn should_apply_normal_root_safety_deep_floor(game: &MonsGame, perspective: Color) -> bool {
        let (my_score, opponent_score) = if perspective == Color::White {
            (game.white_score, game.black_score)
        } else {
            (game.black_score, game.white_score)
        };
        let my_distance_to_win = Config::TARGET_SCORE - my_score;
        let opponent_distance_to_win = Config::TARGET_SCORE - opponent_score;
        my_distance_to_win <= SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_SCORE_RACE_TRIGGER
            || opponent_distance_to_win <= SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_SCORE_RACE_TRIGGER
    }

    fn pick_normal_root_with_deep_floor(
        scored_roots: &[RootEvaluation],
        shortlist_indices: &[usize],
        selected_index: usize,
        perspective: Color,
        config: SmartSearchConfig,
    ) -> usize {
        if shortlist_indices.len() < 2 {
            return selected_index;
        }

        let shortlist_best_score = shortlist_indices
            .iter()
            .map(|index| scored_roots[*index].score)
            .max()
            .unwrap_or(i32::MIN);
        let margin = SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_SCORE_MARGIN.max(0);
        let max_candidates = SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_MAX_CANDIDATES.max(1);
        let reply_limit = config.node_enum_limit.clamp(
            SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_REPLY_LIMIT_MIN,
            SMART_NORMAL_ROOT_SAFETY_DEEP_FLOOR_REPLY_LIMIT_MAX,
        );

        let mut candidate_indices = shortlist_indices
            .iter()
            .copied()
            .filter(|index| scored_roots[*index].score + margin >= shortlist_best_score)
            .collect::<Vec<_>>();
        if candidate_indices.len() > max_candidates {
            candidate_indices.truncate(max_candidates);
        }
        if !candidate_indices.contains(&selected_index) {
            candidate_indices.push(selected_index);
        }
        if candidate_indices.len() < 2 {
            return selected_index;
        }

        let mut best_index = selected_index;
        let mut best_floor_score = i32::MIN;
        for index in candidate_indices {
            let floor_score = Self::normal_root_safety_deep_floor_score(
                &scored_roots[index].game,
                perspective,
                config,
                reply_limit,
            );
            if floor_score > best_floor_score
                || (floor_score == best_floor_score
                    && Self::compare_ranked_scored_root_indices(scored_roots, index, best_index)
                        == std::cmp::Ordering::Less)
            {
                best_floor_score = floor_score;
                best_index = index;
            }
        }

        best_index
    }

    fn normal_root_safety_deep_floor_score(
        state_after_move: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
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
            return evaluate_preferability_with_weights(
                state_after_move,
                perspective,
                config.scoring_weights,
            );
        }

        let mut probe = config;
        probe.depth = 1;
        probe.max_visited_nodes = (config.max_visited_nodes / 18).clamp(110, 360);
        probe.root_branch_limit = config.node_branch_limit.clamp(5, 12);
        probe.node_branch_limit = config.node_branch_limit.saturating_sub(4).clamp(4, 10);
        probe.root_enum_limit = (probe.root_branch_limit * 3).clamp(probe.root_branch_limit, 48);
        probe.node_enum_limit = (probe.node_branch_limit * 3).clamp(probe.node_branch_limit, 36);

        let replies = Self::enumerate_legal_transitions(
            state_after_move,
            reply_limit.max(1),
            Self::automove_start_input_options(config),
        );
        if replies.is_empty() {
            return SMART_TERMINAL_SCORE / 4;
        }

        let mut worst = i32::MAX;
        for reply in replies {
            let after_reply = reply.game;
            let score = match after_reply.winner_color() {
                Some(winner) if winner == perspective => SMART_TERMINAL_SCORE / 2,
                Some(_) => -SMART_TERMINAL_SCORE / 2,
                None => {
                    let mut visited_nodes = 0usize;
                    let mut transposition_table = U64HashMap::<TranspositionEntry>::default();
                    let mut probe_extension_nodes_used = 0usize;
                    let mut probe_killer_table: KillerTable =
                        [[0u64; 2]; MAX_SMART_SEARCH_DEPTH + 2];
                    Self::search_score(
                        &after_reply,
                        perspective,
                        1,
                        i32::MIN,
                        i32::MAX,
                        &mut visited_nodes,
                        probe,
                        &mut transposition_table,
                        0,
                        &mut probe_extension_nodes_used,
                        0,
                        true,
                        &mut probe_killer_table,
                    )
                }
            };
            worst = worst.min(score);
        }

        if worst == i32::MAX {
            evaluate_preferability_with_weights(
                state_after_move,
                perspective,
                config.scoring_weights,
            )
        } else {
            worst
        }
    }

    fn is_better_normal_root_safety_candidate(
        candidate_snapshot: NormalRootSafetySnapshot,
        candidate_score: i32,
        incumbent_snapshot: NormalRootSafetySnapshot,
        incumbent_score: i32,
    ) -> bool {
        if candidate_snapshot.allows_immediate_opponent_win
            != incumbent_snapshot.allows_immediate_opponent_win
        {
            return !candidate_snapshot.allows_immediate_opponent_win;
        }
        if candidate_snapshot.opponent_reaches_match_point
            != incumbent_snapshot.opponent_reaches_match_point
        {
            return !candidate_snapshot.opponent_reaches_match_point;
        }
        if candidate_snapshot.opponent_max_score_gain != incumbent_snapshot.opponent_max_score_gain
        {
            return candidate_snapshot.opponent_max_score_gain
                < incumbent_snapshot.opponent_max_score_gain;
        }
        if candidate_snapshot.my_score_gain != incumbent_snapshot.my_score_gain {
            return candidate_snapshot.my_score_gain > incumbent_snapshot.my_score_gain;
        }
        if candidate_snapshot.worst_reply_score != incumbent_snapshot.worst_reply_score {
            return candidate_snapshot.worst_reply_score > incumbent_snapshot.worst_reply_score;
        }
        candidate_score > incumbent_score
    }
}

#[cfg(test)]
#[path = "automove_experiments/mod.rs"]
mod smart_automove_pool_tests;

#[cfg(test)]
mod opening_book_tests {
    use super::*;
    use std::collections::HashMap;
    use std::collections::HashSet;

    fn game_with_items(
        items: Vec<(Location, Item)>,
        active_color: Color,
        turn_number: i32,
    ) -> MonsGame {
        let mut game = MonsGame::new(false);
        game.board = Board::new_with_items(items.into_iter().collect::<HashMap<_, _>>());
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

    fn advance_opening_book_until_black_turn(game: &mut MonsGame) {
        let mut applied_steps = 0;
        while game.turn_number == 1 && applied_steps < 8 {
            let inputs = MonsGameModel::white_first_turn_opening_next_inputs(game)
                .expect("expected opening-book continuation during white first turn");
            assert!(matches!(
                game.process_input(inputs, false, false),
                Output::Events(_)
            ));
            applied_steps += 1;
        }
        assert_eq!(
            game.turn_number, 2,
            "opening book should finish white first turn"
        );
        assert_eq!(game.active_color, Color::Black);
    }

    fn apply_opening_book_sequence(game: &mut MonsGame, sequence_index: usize) {
        for inputs in PARSED_WHITE_OPENING_BOOK[sequence_index].iter() {
            assert!(matches!(
                game.process_input(inputs.clone(), false, false),
                Output::Events(_)
            ));
        }
        assert_eq!(
            game.turn_number, 2,
            "opening sequence should finish white first turn"
        );
        assert_eq!(game.active_color, Color::Black);
    }

    fn opening_black_reply_fixture(label: &'static str, sequence_index: usize) -> (&'static str, MonsGame) {
        let mut game = MonsGame::new(false);
        apply_opening_book_sequence(&mut game, sequence_index);
        (label, game)
    }

    fn opening_black_reply_fixtures() -> Vec<(&'static str, MonsGame)> {
        vec![
            opening_black_reply_fixture("left-route", 0),
            opening_black_reply_fixture("center-route", 2),
            opening_black_reply_fixture("right-route", 1),
        ]
    }

    fn clone_model_for_smart_automove_tests(model: &MonsGameModel) -> MonsGameModel {
        let cloned = MonsGameModel::with_game(model.game.clone_for_simulation());
        cloned
            .pro_runtime_context_hint
            .set(model.pro_runtime_context_hint.get());
        cloned
    }

    fn advance_smart_opening_book_until_black_turn(
        model: &mut MonsGameModel,
        preference: SmartAutomovePreference,
    ) {
        let mut applied_steps = 0;
        while model.game.turn_number == 1 && applied_steps < 8 {
            let output = model.smart_automove_output(preference);
            assert_eq!(
                output.kind,
                OutputModelKind::Events,
                "opening-book smart output should be events for {}",
                preference.as_api_value()
            );
            let applied = model.process_input_fen(output.input_fen().as_str());
            assert_eq!(
                applied.kind,
                OutputModelKind::Events,
                "opening-book smart input fen should apply cleanly for {}",
                preference.as_api_value()
            );
            applied_steps += 1;
        }
        assert_eq!(
            model.game.turn_number,
            2,
            "opening book should finish white first turn for {}",
            preference.as_api_value()
        );
        assert_eq!(model.game.active_color, Color::Black);
    }

    fn apply_smart_preference_step(
        model: &mut MonsGameModel,
        preference: SmartAutomovePreference,
    ) -> OutputModel {
        let output = model.smart_automove_output(preference);
        assert_eq!(
            output.kind,
            OutputModelKind::Events,
            "smart automove should produce events for {}",
            preference.as_api_value()
        );
        let input_fen = output.input_fen();
        let applied = model.process_input_fen(input_fen.as_str());
        assert_eq!(
            applied.kind,
            OutputModelKind::Events,
            "smart automove input fen should apply cleanly for {}",
            preference.as_api_value()
        );
        applied
    }

    fn play_smart_preference_until_end(
        preference: SmartAutomovePreference,
        max_plies: usize,
    ) -> Color {
        let mut model = MonsGameModel::new_for_simulation();
        let mut previous_fen = model.fen();

        for ply in 0..max_plies {
            let _ = apply_smart_preference_step(&mut model, preference);
            let next_fen = model.fen();
            assert_ne!(
                next_fen,
                previous_fen,
                "smart automove should advance the board for {} at ply {}",
                preference.as_api_value(),
                ply + 1
            );
            if let Some(winner) = model.winner_color() {
                return winner;
            }
            previous_fen = next_fen;
        }

        panic!(
            "smart automove for {} did not finish within {} plies",
            preference.as_api_value(),
            max_plies
        );
    }

    fn exhaustive_same_turn_reachable<F>(game: &MonsGame, color: Color, predicate: F) -> bool
    where
        F: Fn(&MonsGame, &[Event]) -> bool,
    {
        fn visit<F>(game: &MonsGame, color: Color, seen: &mut HashSet<u64>, predicate: &F) -> bool
        where
            F: Fn(&MonsGame, &[Event]) -> bool,
        {
            if game.active_color != color {
                return false;
            }

            let state_hash = MonsGameModel::search_state_hash(game);
            if !seen.insert(state_hash) {
                return false;
            }

            for transition in MonsGameModel::enumerate_legal_transitions(
                game,
                usize::MAX,
                SuggestedStartInputOptions::for_automove(),
            ) {
                if predicate(&transition.game, &transition.events) {
                    return true;
                }
                if transition.game.active_color == color
                    && visit(&transition.game, color, seen, predicate)
                {
                    return true;
                }
            }

            false
        }

        if predicate(game, &[]) {
            return true;
        }

        let mut seen = HashSet::new();
        visit(game, color, &mut seen, &predicate)
    }

    fn exhaustive_same_turn_reachable_with_spirit_history<F>(
        game: &MonsGame,
        color: Color,
        predicate: F,
    ) -> bool
    where
        F: Fn(&MonsGame, bool) -> bool,
    {
        fn visit<F>(
            game: &MonsGame,
            color: Color,
            spirit_used: bool,
            seen: &mut HashSet<(u64, bool)>,
            predicate: &F,
        ) -> bool
        where
            F: Fn(&MonsGame, bool) -> bool,
        {
            if game.active_color != color {
                return false;
            }

            let state_hash = MonsGameModel::search_state_hash(game);
            if !seen.insert((state_hash, spirit_used)) {
                return false;
            }

            for transition in MonsGameModel::enumerate_legal_transitions(
                game,
                usize::MAX,
                SuggestedStartInputOptions::for_automove(),
            ) {
                let used_spirit =
                    spirit_used || MonsGameModel::events_include_spirit_target_move(&transition.events);
                if predicate(&transition.game, used_spirit) {
                    return true;
                }
                if transition.game.active_color == color
                    && visit(&transition.game, color, used_spirit, seen, predicate)
                {
                    return true;
                }
            }

            false
        }

        if predicate(game, false) {
            return true;
        }

        let mut seen = HashSet::new();
        visit(game, color, false, &mut seen, &predicate)
    }

    fn drainer_carries_exact_safe_mana(board: &Board, color: Color, wanted_mana: Mana) -> bool {
        board.occupied().any(|(location, item)| {
            matches!(
                item,
                Item::MonWithMana { mon, mana }
                    if mon.color == color
                        && mon.kind == MonKind::Drainer
                        && !mon.is_fainted()
                        && *mana == wanted_mana
                        && crate::models::automove_exact::is_drainer_exactly_safe_next_turn_on_board(
                            board,
                            color,
                            location,
                        )
            )
        })
    }

    #[test]
    fn white_opening_book_selects_a_valid_first_move() {
        let game = MonsGame::new(false);
        let opening_inputs = MonsGameModel::white_first_turn_opening_next_inputs(&game)
            .expect("expected opening-book move on initial white turn");
        let opening_fen = Input::fen_from_array(&opening_inputs);
        let allowed = [
            "l10,3;l9,2",
            "l10,7;l9,8",
            "l10,4;l9,4",
            "l10,5;l9,5",
            "l10,6;l9,7",
            "l10,3;l9,3",
        ];
        assert!(allowed.contains(&opening_fen.as_str()));
    }

    #[test]
    fn white_opening_book_continues_unique_prefix() {
        let mut game = MonsGame::new(false);
        let first_inputs = Input::array_from_fen("l10,3;l9,2");
        assert!(matches!(
            game.process_input(first_inputs, false, false),
            Output::Events(_)
        ));

        let next_inputs = MonsGameModel::white_first_turn_opening_next_inputs(&game)
            .expect("expected follow-up opening move");
        assert_eq!(Input::fen_from_array(&next_inputs), "l9,2;l8,1");
    }

    #[test]
    fn white_opening_book_falls_back_when_position_diverged() {
        let mut game = MonsGame::new(false);
        let custom_inputs = Input::array_from_fen("l10,3;l9,4");
        assert!(matches!(
            game.process_input(custom_inputs, false, false),
            Output::Events(_)
        ));
        assert!(MonsGameModel::white_first_turn_opening_next_inputs(&game).is_none());
    }

    #[test]
    fn pro_and_ultra_modes_are_accepted_and_produce_legal_inputs() {
        assert_eq!(
            SmartAutomovePreference::from_api_value("pro"),
            Some(SmartAutomovePreference::Pro)
        );
        assert_eq!(SmartAutomovePreference::Pro.as_api_value(), "pro");
        assert_eq!(
            SmartAutomovePreference::from_api_value("ultra"),
            Some(SmartAutomovePreference::Ultra)
        );
        assert_eq!(
            SmartAutomovePreference::from_api_value("ULTRA"),
            Some(SmartAutomovePreference::Ultra)
        );
        assert_eq!(SmartAutomovePreference::Ultra.as_api_value(), "ultra");

        let game = MonsGame::new(false);
        for preference in [SmartAutomovePreference::Pro, SmartAutomovePreference::Ultra] {
            let mut config = SmartSearchConfig::from_preference(preference);
            config.depth = 1;
            config.max_visited_nodes = 16;
            config.root_branch_limit = 1;
            config.node_branch_limit = 1;
            config.root_enum_limit = 1;
            config.node_enum_limit = 1;
            let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
            assert!(
                !inputs.is_empty(),
                "{} mode should produce at least one input from initial position",
                preference.as_api_value()
            );
            assert!(
                MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs).is_some(),
                "{} mode selected inputs should be legal",
                preference.as_api_value()
            );
        }
    }

    #[test]
    #[ignore = "full pro/ultra opening searches can take over a minute in debug"]
    fn pro_and_ultra_modes_full_runtime_searches_produce_legal_inputs() {
        let game = MonsGame::new(false);
        for preference in [SmartAutomovePreference::Pro, SmartAutomovePreference::Ultra] {
            let config = SmartSearchConfig::from_preference(preference);
            let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
            assert!(
                !inputs.is_empty(),
                "{} mode should produce at least one input from initial position",
                preference.as_api_value()
            );
            assert!(
                MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs).is_some(),
                "{} mode selected inputs should be legal",
                preference.as_api_value()
            );
        }
    }

    #[test]
    fn opening_book_move_marks_pro_runtime_context_hint() {
        let model = MonsGameModel::new();
        assert_eq!(
            model.pro_runtime_context_hint_for_tests(),
            ProRuntimeContext::Unknown
        );
        model.mark_opening_book_driven_context();
        assert_eq!(
            model.pro_runtime_context_hint_for_tests(),
            ProRuntimeContext::OpeningBookDriven
        );
    }

    #[test]
    fn smart_automove_output_uses_opening_book_until_black_turn_for_all_modes() {
        for preference in [
            SmartAutomovePreference::Fast,
            SmartAutomovePreference::Normal,
            SmartAutomovePreference::Pro,
            SmartAutomovePreference::Ultra,
        ] {
            let mut model = MonsGameModel::new_for_simulation();
            advance_smart_opening_book_until_black_turn(&mut model, preference);
            assert_eq!(
                model.pro_runtime_context_hint_for_tests(),
                ProRuntimeContext::OpeningBookDriven
            );
        }
    }

    #[test]
    fn smart_automove_black_reply_opening_fixtures_are_legal_for_all_modes() {
        let (label, game) = opening_black_reply_fixture("center-route", 2);
        for preference in [
            SmartAutomovePreference::Fast,
            SmartAutomovePreference::Normal,
            SmartAutomovePreference::Pro,
            SmartAutomovePreference::Ultra,
        ] {
            let model = MonsGameModel::with_game(game.clone_for_simulation());
            model.mark_opening_book_driven_context();
            let mut config = model.runtime_config_for_preference(preference);
            config.depth = 1;
            config.max_visited_nodes = 8;
            config.root_branch_limit = 1;
            config.node_branch_limit = 1;
            config.root_enum_limit = 1;
            config.node_enum_limit = 1;

            let inputs = MonsGameModel::smart_search_best_inputs(&model.game, config);
            assert!(
                !inputs.is_empty(),
                "{} should produce at least one black reply on {}",
                preference.as_api_value(),
                label
            );
            assert!(
                MonsGameModel::apply_inputs_for_search_with_events(&model.game, &inputs).is_some(),
                "{} should stay legal on {}",
                preference.as_api_value(),
                label
            );
        }
    }

    #[test]
    fn active_turn_exact_tactical_queries_match_exhaustive_curated_fixtures() {
        let attack_game = game_with_items(
            vec![
                (
                    Location::new(5, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::White, 0),
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
        let attack_turn =
            crate::models::automove_exact::exact_turn_summary(&attack_game, Color::White);
        assert_eq!(
            attack_turn.can_attack_opponent_drainer,
            exhaustive_same_turn_reachable(&attack_game, Color::White, |_, events| {
                events.iter().any(|event| {
                    matches!(
                        event,
                        Event::MonFainted { mon, .. }
                            if mon.kind == MonKind::Drainer && mon.color == Color::Black
                    )
                })
            })
        );

        let supermana_game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
        let supermana_turn =
            crate::models::automove_exact::exact_turn_summary(&supermana_game, Color::White);
        assert_eq!(
            supermana_turn.safe_supermana_progress,
            exhaustive_same_turn_reachable(&supermana_game, Color::White, |reachable, events| {
                events.iter().any(|event| {
                    matches!(event, Event::ManaScored { mana, .. } if *mana == Mana::Supermana)
                }) || drainer_carries_exact_safe_mana(&reachable.board, Color::White, Mana::Supermana)
            })
        );

        let opponent_mana_game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
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
        let opponent_mana_turn =
            crate::models::automove_exact::exact_turn_summary(&opponent_mana_game, Color::White);
        assert_eq!(
            opponent_mana_turn.safe_opponent_mana_progress,
            exhaustive_same_turn_reachable(&opponent_mana_game, Color::White, |reachable, events| {
                events.iter().any(|event| {
                    matches!(
                        event,
                        Event::ManaScored { mana, .. }
                            if *mana == Mana::Regular(Color::Black)
                    )
                }) || drainer_carries_exact_safe_mana(
                    &reachable.board,
                    Color::White,
                    Mana::Regular(Color::Black),
                )
            })
        );

        let mut spirit_game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 1),
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
        spirit_game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 2;
        let spirit_turn =
            crate::models::automove_exact::exact_turn_summary(&spirit_game, Color::White);
        let exhaustive_spirit_score = exhaustive_same_turn_reachable_with_spirit_history(
            &spirit_game,
            Color::White,
            |reachable, spirit_used| spirit_used && reachable.white_score > 0,
        );
        let exhaustive_spirit_denial = exhaustive_same_turn_reachable_with_spirit_history(
            &spirit_game,
            Color::White,
            |reachable, spirit_used| {
                spirit_used
                    && (reachable.white_score
                        >= Mana::Regular(Color::Black).score(Color::White)
                        || drainer_carries_exact_safe_mana(
                            &reachable.board,
                            Color::White,
                            Mana::Regular(Color::Black),
                        ))
            },
        );
        assert_eq!(spirit_turn.spirit_assisted_score, exhaustive_spirit_score);
        assert_eq!(spirit_turn.spirit_assisted_denial, exhaustive_spirit_denial);
    }

    #[test]
    fn smart_automove_sync_matches_async_pipeline_on_sparse_tactical_position_for_all_modes() {
        let game = game_with_items(
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
        let base_model = MonsGameModel::with_game(game);
        base_model
            .pro_runtime_context_hint
            .set(ProRuntimeContext::Independent);

        for preference in [
            SmartAutomovePreference::Fast,
            SmartAutomovePreference::Normal,
            SmartAutomovePreference::Pro,
            SmartAutomovePreference::Ultra,
        ] {
            let sync_model = clone_model_for_smart_automove_tests(&base_model);
            let async_model = clone_model_for_smart_automove_tests(&base_model);

            let sync_output = sync_model.smart_automove_output(preference);
            let async_output = async_model.smart_automove_output_via_async_loop(preference);

            assert_eq!(sync_output.kind, OutputModelKind::Events);
            assert_eq!(async_output.kind, OutputModelKind::Events);
            assert_eq!(
                sync_output.input_fen(),
                async_output.input_fen(),
                "sync production helper should match test-only async pipeline for {}",
                preference.as_api_value()
            );
        }
    }

    #[test]
    #[ignore = "release-only opening reply latency guard for publish flow"]
    fn smart_automove_release_opening_black_reply_speed_gate() {
        use std::time::Instant;

        let fixtures = opening_black_reply_fixtures();
        let passes = 3usize;
        let limits_ms = [
            (SmartAutomovePreference::Fast, 250.0),
            (SmartAutomovePreference::Normal, 250.0),
            (SmartAutomovePreference::Pro, 500.0),
            (SmartAutomovePreference::Ultra, 1000.0),
        ];

        for (preference, limit_ms) in limits_ms {
            let warm_model = MonsGameModel::with_game(fixtures[0].1.clone_for_simulation());
            warm_model.mark_opening_book_driven_context();
            let warm_output = warm_model.smart_automove_output(preference);
            assert_eq!(warm_output.kind, OutputModelKind::Events);

            let mut pass_averages = Vec::with_capacity(passes);
            for _ in 0..passes {
                let mut total_ms = 0.0;
                for (_, fixture) in fixtures.iter() {
                    crate::models::automove_exact::clear_exact_state_analysis_cache();
                    let model = MonsGameModel::with_game(fixture.clone_for_simulation());
                    model.mark_opening_book_driven_context();
                    let start = Instant::now();
                    let output = model.smart_automove_output(preference);
                    total_ms += start.elapsed().as_secs_f64() * 1000.0;
                    assert_eq!(output.kind, OutputModelKind::Events);
                }
                pass_averages.push(total_ms / fixtures.len() as f64);
            }

            pass_averages.sort_by(|a, b| a.partial_cmp(b).expect("finite timings"));
            let median_avg_ms = pass_averages[passes / 2];
            println!(
                "release opening reply speed gate {}: median_avg_ms={:.2} limit_ms={:.2}",
                preference.as_api_value(),
                median_avg_ms,
                limit_ms
            );
            assert!(
                median_avg_ms <= limit_ms,
                "{} opening-black-reply median {:.2}ms exceeded limit {:.2}ms",
                preference.as_api_value(),
                median_avg_ms,
                limit_ms
            );
        }
    }

    #[test]
    #[ignore = "long-running end-to-end smart automove test for fast mode"]
    fn smart_automove_fast_till_end() {
        let _ = play_smart_preference_until_end(SmartAutomovePreference::Fast, 512);
    }

    #[test]
    #[ignore = "long-running end-to-end smart automove test for normal mode"]
    fn smart_automove_normal_till_end() {
        let _ = play_smart_preference_until_end(SmartAutomovePreference::Normal, 512);
    }

    #[test]
    #[ignore = "long-running end-to-end smart automove test for pro mode"]
    fn smart_automove_pro_till_end() {
        let _ = play_smart_preference_until_end(SmartAutomovePreference::Pro, 512);
    }

    #[test]
    #[ignore = "long-running end-to-end smart automove test for ultra mode"]
    fn smart_automove_ultra_till_end() {
        let _ = play_smart_preference_until_end(SmartAutomovePreference::Ultra, 512);
    }

    #[test]
    fn pro_runtime_context_resolver_handles_unknown_opening_and_independent() {
        let mut opening_game = MonsGame::new(false);
        advance_opening_book_until_black_turn(&mut opening_game);
        assert_eq!(
            MonsGameModel::resolve_pro_runtime_context(&opening_game, ProRuntimeContext::Unknown),
            ProRuntimeContext::OpeningBookDriven
        );
        assert_eq!(
            MonsGameModel::resolve_pro_runtime_context(
                &opening_game,
                ProRuntimeContext::OpeningBookDriven
            ),
            ProRuntimeContext::OpeningBookDriven
        );

        let mut independent_game = MonsGame::new(false);
        let diverged_inputs = Input::array_from_fen("l10,3;l9,4");
        assert!(matches!(
            independent_game.process_input(diverged_inputs, false, false),
            Output::Events(_)
        ));
        assert_eq!(
            MonsGameModel::resolve_pro_runtime_context(
                &independent_game,
                ProRuntimeContext::Unknown
            ),
            ProRuntimeContext::Independent
        );
    }

    #[test]
    fn pro_runtime_context_profile_applies_expected_tuning() {
        let base_game = MonsGame::new(false);
        let independent_model = MonsGameModel::with_game(base_game);
        let independent_config =
            independent_model.runtime_config_for_preference(SmartAutomovePreference::Pro);
        assert_eq!(
            independent_config.max_visited_nodes,
            SMART_AUTOMOVE_PRO_MAX_VISITED_NODES as usize
        );
        assert_eq!(independent_config.root_reply_risk_score_margin, 165);
        assert_eq!(independent_config.root_reply_risk_shortlist_max, 9);
        assert_eq!(independent_config.root_reply_risk_reply_limit, 24);
        assert_eq!(independent_config.root_reply_risk_node_share_bp, 2_000);
        assert!(independent_config.enable_normal_root_safety_deep_floor);
        assert_eq!(independent_config.root_drainer_safety_score_margin, 4_800);
        assert_eq!(independent_config.selective_extension_node_share_bp, 1_500);
        assert_eq!(
            independent_config.interview_soft_opponent_mana_progress_bonus,
            320
        );
        assert_eq!(
            independent_config.interview_soft_opponent_mana_score_bonus,
            400
        );

        let mut opening_game = MonsGame::new(false);
        advance_opening_book_until_black_turn(&mut opening_game);
        let opening_model = MonsGameModel::with_game(opening_game);
        let opening_config =
            opening_model.runtime_config_for_preference(SmartAutomovePreference::Pro);
        assert_eq!(opening_config.depth, 3);
        assert_eq!(opening_config.max_visited_nodes, 1_100);
        assert_eq!(opening_config.root_branch_limit, 20);
        assert_eq!(opening_config.node_branch_limit, 10);
        assert_eq!(opening_config.root_enum_limit, 132);
        assert_eq!(opening_config.node_enum_limit, 72);
        assert!(!opening_config.enable_root_efficiency);
        assert!(!opening_config.enable_child_move_class_coverage);
        assert!(!opening_config.enable_two_pass_root_allocation);
        assert!(!opening_config.enable_root_aspiration);
        assert!(!opening_config.enable_selective_extensions);
        assert_eq!(opening_config.root_reply_risk_score_margin, 155);
        assert_eq!(opening_config.root_reply_risk_shortlist_max, 7);
        assert_eq!(opening_config.root_reply_risk_reply_limit, 8);
        assert_eq!(opening_config.root_reply_risk_node_share_bp, 560);
        assert!(!opening_config.enable_normal_root_safety_deep_floor);
        assert_eq!(opening_config.root_drainer_safety_score_margin, 4_300);
        assert_eq!(opening_config.selective_extension_node_share_bp, 1_200);
    }

    #[test]
    fn ultra_runtime_context_profile_applies_expected_tuning() {
        let base_game = MonsGame::new(false);
        let independent_model = MonsGameModel::with_game(base_game);
        let independent_config =
            independent_model.runtime_config_for_preference(SmartAutomovePreference::Ultra);
        assert_eq!(
            independent_config.max_visited_nodes,
            SMART_AUTOMOVE_ULTRA_MAX_VISITED_NODES as usize
        );
        assert_eq!(independent_config.root_reply_risk_score_margin, 175);
        assert_eq!(independent_config.root_reply_risk_shortlist_max, 10);
        assert_eq!(independent_config.root_reply_risk_reply_limit, 30);
        assert_eq!(independent_config.root_reply_risk_node_share_bp, 2_400);
        assert!(independent_config.enable_normal_root_safety_deep_floor);
        assert_eq!(independent_config.root_drainer_safety_score_margin, 5_000);
        assert_eq!(independent_config.selective_extension_node_share_bp, 1_900);
        assert_eq!(
            independent_config.interview_soft_opponent_mana_progress_bonus,
            330
        );
        assert_eq!(
            independent_config.interview_soft_opponent_mana_score_bonus,
            390
        );

        let mut opening_game = MonsGame::new(false);
        advance_opening_book_until_black_turn(&mut opening_game);
        let opening_model = MonsGameModel::with_game(opening_game);
        let opening_config =
            opening_model.runtime_config_for_preference(SmartAutomovePreference::Ultra);
        assert_eq!(opening_config.depth, 4);
        assert_eq!(opening_config.max_visited_nodes, 2_400);
        assert_eq!(opening_config.root_branch_limit, 22);
        assert_eq!(opening_config.node_branch_limit, 11);
        assert_eq!(opening_config.root_enum_limit, 144);
        assert_eq!(opening_config.node_enum_limit, 84);
        assert!(!opening_config.enable_root_efficiency);
        assert!(!opening_config.enable_child_move_class_coverage);
        assert!(!opening_config.enable_two_pass_root_allocation);
        assert!(!opening_config.enable_root_aspiration);
        assert!(!opening_config.enable_selective_extensions);
        assert_eq!(opening_config.root_reply_risk_score_margin, 165);
        assert_eq!(opening_config.root_reply_risk_shortlist_max, 8);
        assert_eq!(opening_config.root_reply_risk_reply_limit, 8);
        assert_eq!(opening_config.root_reply_risk_node_share_bp, 560);
        assert!(!opening_config.enable_normal_root_safety_deep_floor);
        assert_eq!(opening_config.root_drainer_safety_score_margin, 4_600);
        assert_eq!(opening_config.selective_extension_node_share_bp, 1_200);
    }

    #[test]
    fn forced_drainer_attack_selects_a_drainer_fainting_turn_when_available() {
        let white_mystic = Mon::new(MonKind::Mystic, Color::White, 0);
        let white_drainer = Mon::new(MonKind::Drainer, Color::White, 0);
        let black_drainer = Mon::new(MonKind::Drainer, Color::Black, 0);
        let game = game_with_items(
            vec![
                (Location::new(5, 5), Item::Mon { mon: white_mystic }),
                (Location::new(10, 5), Item::Mon { mon: white_drainer }),
                (Location::new(7, 7), Item::Mon { mon: black_drainer }),
            ],
            Color::White,
            2,
        );

        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (_, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("expected selected inputs to be legal");
        assert!(MonsGameModel::events_include_opponent_drainer_fainted(
            &events,
            Color::White
        ));
    }

    #[test]
    fn forced_drainer_attack_with_bomb_survives_root_enumeration_cutoff() {
        let game = game_with_items(
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_enum_limit = 0;
        let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("expected selected bomb inputs to be legal");
        let attacks_now =
            MonsGameModel::events_include_opponent_drainer_fainted(&events, Color::White);
        let mut continuation_budget = SMART_FORCED_DRAINER_ATTACK_FALLBACK_NODE_BUDGET_FAST;
        let attacks_later_this_turn = after.active_color == Color::White
            && MonsGameModel::can_attack_opponent_drainer_before_turn_ends(
                &after,
                Color::White,
                SMART_FORCED_DRAINER_ATTACK_FALLBACK_ENUM_LIMIT_FAST,
                SuggestedStartInputOptions::for_automove(),
                &mut continuation_budget,
                &mut U64HashSet::default(),
            );
        assert!(
            attacks_now || attacks_later_this_turn,
            "selected line must keep same-turn drainer attack reachable"
        );
    }

    #[test]
    fn drainer_vulnerability_detection_respects_friendly_angel_guard() {
        let white_drainer = Mon::new(MonKind::Drainer, Color::White, 0);
        let white_angel = Mon::new(MonKind::Angel, Color::White, 0);
        let black_mystic = Mon::new(MonKind::Mystic, Color::Black, 0);

        let unguarded = game_with_items(
            vec![
                (Location::new(5, 5), Item::Mon { mon: white_drainer }),
                (Location::new(3, 3), Item::Mon { mon: black_mystic }),
            ],
            Color::Black,
            2,
        );
        assert!(MonsGameModel::is_own_drainer_immediately_vulnerable(
            &unguarded,
            Color::White,
            false,
        ));

        let guarded = game_with_items(
            vec![
                (Location::new(5, 5), Item::Mon { mon: white_drainer }),
                (Location::new(5, 4), Item::Mon { mon: white_angel }),
                (Location::new(3, 3), Item::Mon { mon: black_mystic }),
            ],
            Color::Black,
            2,
        );
        assert!(!MonsGameModel::is_own_drainer_immediately_vulnerable(
            &guarded,
            Color::White,
            false,
        ));
    }

    #[test]
    fn drainer_vulnerability_detects_multi_step_mystic_attack() {
        let game = game_with_items(
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
            2,
        );

        assert!(
            !crate::models::automove_exact::is_drainer_under_immediate_threat(
                &game.board,
                Color::White,
                Location::new(8, 5),
                false,
            ),
            "pure immediate geometry should not already see this multi-step attack"
        );
        assert!(
            !crate::models::automove_exact::is_drainer_under_walk_threat(
                &game.board,
                Color::White,
                Location::new(8, 5),
                false,
            ),
            "one-step walk geometry should not already see this two-step mystic attack"
        );
        assert!(MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            true,
        ));
    }

    #[test]
    fn selected_line_avoids_drainer_vulnerability_when_safe_root_exists() {
        let game = game_with_items(
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let roots = MonsGameModel::ranked_root_moves(&game, Color::White, config);
        assert!(
            roots.iter().any(|root| !root.own_drainer_vulnerable),
            "scenario should offer at least one safe root"
        );

        let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected drainer-safety inputs should be legal");
        assert!(
            !MonsGameModel::is_own_drainer_vulnerable_next_turn(
                &after,
                Color::White,
                config.enable_enhanced_drainer_vulnerability,
            ),
            "selected line should keep drainer safe when a safe root exists, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn selected_line_avoids_drainer_vulnerability_with_root_cutoff() {
        let game = game_with_items(
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let roots = MonsGameModel::ranked_root_moves(&game, Color::White, config);
        assert!(
            roots.iter().any(|root| !root.own_drainer_vulnerable),
            "scenario should offer at least one safe root"
        );

        config.root_enum_limit = 0;
        let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected drainer-safety inputs should be legal");
        assert!(
            !MonsGameModel::is_own_drainer_vulnerable_next_turn(
                &after,
                Color::White,
                config.enable_enhanced_drainer_vulnerability,
            ),
            "selected line should keep drainer safe under capped root enumeration when a safe root exists, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn spirit_development_is_preferred_when_drainer_progress_is_still_available() {
        let spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let spirit_base = Board::new().base(spirit);
        let game = game_with_items(
            vec![
                (spirit_base, Item::Mon { mon: spirit }),
                (
                    Location::new(9, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 5),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
                (
                    Location::new(1, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (after, _) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit development inputs should be legal");
        assert!(
            !MonsGameModel::has_awake_spirit_on_base(&after.board, Color::White),
            "selected move should deploy awake spirit off base when core progress remains available"
        );
    }

    #[test]
    fn spirit_scores_opponent_mana_when_immediately_available() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
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
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-score inputs should be legal");
        assert!(
            after.white_score >= game.white_score + 2,
            "selected line should score opponent mana immediately, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn spirit_scores_own_mana_when_immediately_available() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
                (
                    Location::new(9, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 5),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
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

        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-score inputs should be legal");
        assert!(
            after.white_score >= game.white_score + 1,
            "selected line should score own mana immediately, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn spirit_scores_own_mana_with_root_cutoff_when_immediately_available() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
                (
                    Location::new(9, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 5),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
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
        assert!(
            crate::models::automove_exact::exact_turn_summary(&game, Color::White)
                .spirit_assisted_score
        );

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_enum_limit = 0;
        let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-score inputs should be legal");
        assert!(
            after.white_score >= game.white_score + 1,
            "selected line should still score own mana immediately under capped root enumeration, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn safe_supermana_pickup_is_preferred_when_available() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let white_spirit_base = Board::new().base(white_spirit);
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (white_spirit_base, Item::Mon { mon: white_spirit }),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected supermana inputs should be legal");
        assert!(
            matches!(
                after.board.item(Location::new(5, 5)),
                Some(Item::MonWithMana {
                    mon,
                    mana: Mana::Supermana,
                }) if mon.color == Color::White && mon.kind == MonKind::Drainer
            ),
            "selected line should pick up safe supermana, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn safe_opponent_mana_pickup_is_preferred_when_available() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let white_spirit_base = Board::new().base(white_spirit);
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (white_spirit_base, Item::Mon { mon: white_spirit }),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected opponent-mana inputs should be legal");
        assert!(
            matches!(
                after.board.item(Location::new(5, 4)),
                Some(Item::MonWithMana {
                    mon,
                    mana: Mana::Regular(Color::Black),
                }) if mon.color == Color::White && mon.kind == MonKind::Drainer
            ),
            "selected line should pick up safe opponent mana, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn unsafe_multi_step_attack_supermana_pickup_is_not_flagged_safe() {
        let game = game_with_items(
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
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(4, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let candidate = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            MonsGameModel::is_own_drainer_vulnerable_next_turn(
                &game,
                Color::White,
                config.enable_enhanced_drainer_vulnerability,
            ),
            &[
                Input::Location(Location::new(9, 5)),
                Input::Location(Location::new(8, 5)),
            ],
        )
        .expect("supermana pickup inputs should build a scored root");

        assert!(
            candidate.own_drainer_vulnerable,
            "pickup square should be exactly vulnerable to the black mystic"
        );
        assert!(
            !candidate.safe_supermana_pickup_now,
            "unsafe exact-vulnerable supermana pickup must not be flagged safe"
        );
    }

    #[test]
    fn unsafe_multi_step_attack_opponent_mana_pickup_is_not_flagged_safe() {
        let game = game_with_items(
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
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(4, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let candidate = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            MonsGameModel::is_own_drainer_vulnerable_next_turn(
                &game,
                Color::White,
                config.enable_enhanced_drainer_vulnerability,
            ),
            &[
                Input::Location(Location::new(9, 5)),
                Input::Location(Location::new(8, 5)),
            ],
        )
        .expect("opponent-mana pickup inputs should build a scored root");

        assert!(
            candidate.own_drainer_vulnerable,
            "pickup square should be exactly vulnerable to the black mystic"
        );
        assert!(
            !candidate.safe_opponent_mana_pickup_now,
            "unsafe exact-vulnerable opponent-mana pickup must not be flagged safe"
        );
    }

    fn root_evaluation_for_test(candidate: &ScoredRootMove, score: i32) -> RootEvaluation {
        RootEvaluation {
            score,
            efficiency: candidate.efficiency,
            inputs: candidate.inputs.clone(),
            game: candidate.game.clone(),
            wins_immediately: candidate.wins_immediately,
            attacks_opponent_drainer: candidate.attacks_opponent_drainer,
            own_drainer_vulnerable: candidate.own_drainer_vulnerable,
            own_drainer_walk_vulnerable: candidate.own_drainer_walk_vulnerable,
            spirit_development: candidate.spirit_development,
            keeps_awake_spirit_on_base: candidate.keeps_awake_spirit_on_base,
            mana_handoff_to_opponent: candidate.mana_handoff_to_opponent,
            has_roundtrip: candidate.has_roundtrip,
            scores_supermana_this_turn: candidate.scores_supermana_this_turn,
            scores_opponent_mana_this_turn: candidate.scores_opponent_mana_this_turn,
            safe_supermana_pickup_now: candidate.safe_supermana_pickup_now,
            safe_opponent_mana_pickup_now: candidate.safe_opponent_mana_pickup_now,
            safe_supermana_progress_steps: candidate.safe_supermana_progress_steps,
            safe_opponent_mana_progress_steps: candidate.safe_opponent_mana_progress_steps,
            score_path_best_steps: candidate.score_path_best_steps,
            same_turn_score_window_value: candidate.same_turn_score_window_value,
            spirit_same_turn_score_setup_now: candidate.spirit_same_turn_score_setup_now,
            spirit_own_mana_setup_now: candidate.spirit_own_mana_setup_now,
            supermana_progress: candidate.supermana_progress,
            opponent_mana_progress: candidate.opponent_mana_progress,
            interview_soft_priority: candidate.interview_soft_priority,
            classes: candidate.classes,
        }
    }

    fn ranked_child_state_for_test(
        game: &MonsGame,
        hash: u64,
        ordering_efficiency: i32,
        classes: MoveClassFlags,
    ) -> RankedChildState {
        RankedChildState {
            game: game.clone_for_simulation(),
            hash,
            ordering_efficiency,
            tactical_extension_trigger: false,
            quiet_reduction_candidate: false,
            classes,
        }
    }

    #[test]
    fn same_turn_score_window_transition_sort_prefers_stronger_exact_window() {
        let weaker = LegalInputTransition {
            inputs: vec![Input::Location(Location::new(0, 0))],
            game: game_with_items(
                vec![
                    (
                        Location::new(9, 0),
                        Item::Mon {
                            mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        },
                    ),
                    (
                        Location::new(9, 1),
                        Item::Mana {
                            mana: Mana::Regular(Color::White),
                        },
                    ),
                ],
                Color::White,
                2,
            ),
            events: Vec::new(),
        };
        let stronger = LegalInputTransition {
            inputs: vec![Input::Location(Location::new(1, 1))],
            game: game_with_items(
                vec![
                    (
                        Location::new(9, 0),
                        Item::Mon {
                            mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        },
                    ),
                    (
                        Location::new(9, 1),
                        Item::Mana {
                            mana: Mana::Regular(Color::Black),
                        },
                    ),
                ],
                Color::White,
                2,
            ),
            events: Vec::new(),
        };

        let mut transitions = vec![weaker, stronger.clone()];
        transitions.sort_by(|a, b| {
            MonsGameModel::compare_same_turn_score_window_transitions(a, b, Color::White)
        });

        assert_eq!(transitions[0].inputs, stronger.inputs);
    }

    #[test]
    fn child_priority_sort_prefers_higher_exact_ordering_efficiency_when_heuristic_tied() {
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            1,
        );

        let quiet = MoveClassFlags {
            quiet: true,
            ..MoveClassFlags::default()
        };
        let mut scored_states = vec![
            (100, ranked_child_state_for_test(&game, 1, 0, quiet)),
            (100, ranked_child_state_for_test(&game, 2, 40, quiet)),
        ];

        scored_states.sort_by(|a, b| MonsGameModel::compare_ranked_child_entries(a, b, true));
        assert_eq!(scored_states[0].1.hash, 2);
    }

    #[test]
    fn child_priority_sort_prefers_tactical_class_when_efficiency_tied() {
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            1,
        );

        let quiet = MoveClassFlags {
            quiet: true,
            ..MoveClassFlags::default()
        };
        let tactical = MoveClassFlags {
            carrier_progress: true,
            ..MoveClassFlags::default()
        };
        let mut scored_states = vec![
            (100, ranked_child_state_for_test(&game, 1, 20, quiet)),
            (100, ranked_child_state_for_test(&game, 2, 20, tactical)),
        ];

        scored_states.sort_by(|a, b| MonsGameModel::compare_ranked_child_entries(a, b, true));
        assert_eq!(scored_states[0].1.hash, 2);
    }

    #[test]
    fn quiet_reduction_candidate_skips_carrier_progress_class() {
        let classes = MoveClassFlags {
            carrier_progress: true,
            ..MoveClassFlags::default()
        };

        assert!(
            !MonsGameModel::is_quiet_reduction_candidate(-10, false, classes),
            "carrier-progress child should stay at full depth even when ordering efficiency is non-positive"
        );
    }

    #[test]
    fn quiet_reduction_candidate_still_applies_to_true_quiet_child() {
        let classes = MoveClassFlags {
            quiet: true,
            ..MoveClassFlags::default()
        };

        assert!(
            MonsGameModel::is_quiet_reduction_candidate(0, false, classes),
            "true quiet child should remain eligible for quiet reduction"
        );
    }

    #[test]
    fn child_search_priority_candidate_includes_positive_exact_continuation() {
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            1,
        );

        let quiet = MoveClassFlags {
            quiet: true,
            ..MoveClassFlags::default()
        };
        let child = ranked_child_state_for_test(&game, 1, 20, quiet);
        assert!(MonsGameModel::is_child_search_priority_candidate(&child));
    }

    #[test]
    fn selective_extension_candidate_preserves_existing_tactical_trigger() {
        let classes = MoveClassFlags {
            quiet: true,
            ..MoveClassFlags::default()
        };

        assert!(
            MonsGameModel::is_selective_extension_candidate(true, -20, classes),
            "existing tactical trigger should still qualify for selective extension"
        );
    }

    #[test]
    fn selective_extension_candidate_includes_positive_carrier_progress() {
        let classes = MoveClassFlags {
            carrier_progress: true,
            ..MoveClassFlags::default()
        };

        assert!(
            MonsGameModel::is_selective_extension_candidate(false, 15, classes),
            "positive exact carrier progress should qualify for selective extension"
        );
    }

    #[test]
    fn selective_extension_candidate_skips_nonpositive_carrier_progress() {
        let classes = MoveClassFlags {
            carrier_progress: true,
            ..MoveClassFlags::default()
        };

        assert!(
            !MonsGameModel::is_selective_extension_candidate(false, 0, classes),
            "non-improving carrier progress should not consume selective extension budget"
        );
    }

    #[test]
    fn selective_extension_candidate_skips_positive_material_only_child() {
        let classes = MoveClassFlags {
            material: true,
            ..MoveClassFlags::default()
        };

        assert!(
            !MonsGameModel::is_selective_extension_candidate(false, 20, classes),
            "material-only child should not consume selective extension budget without a tactical trigger"
        );
    }

    #[test]
    fn frontier_tactical_potential_detects_safe_supermana_progress() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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

        assert!(
            MonsGameModel::has_exact_frontier_tactical_potential(&game),
            "same-turn exact safe supermana progress should block frontier futility pruning"
        );
    }

    #[test]
    fn frontier_tactical_potential_is_false_on_quiet_board() {
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            1,
        );

        assert!(
            !MonsGameModel::has_exact_frontier_tactical_potential(&game),
            "quiet frontier state should remain eligible for futility pruning"
        );
    }

    #[test]
    fn frontier_tactical_potential_detects_spirit_supermana_progress() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(10, 2),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(7, 3),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
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

        assert!(
            MonsGameModel::has_exact_frontier_tactical_potential(&game),
            "spirit-assisted exact supermana progress should block frontier futility pruning"
        );
    }

    #[test]
    fn root_focus_scout_score_rewards_exact_safe_supermana_progress() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut progress = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(7, 5)),
                Input::Location(Location::new(6, 5)),
            ],
        )
        .expect("drainer advance should build a scored root");
        progress.heuristic = 0;
        progress.efficiency = 0;
        assert!(progress.supermana_progress);
        assert!(!progress.safe_supermana_pickup_now);
        assert!(!progress.spirit_own_mana_setup_now);

        let mut filler = progress.clone();
        filler.inputs = vec![Input::Location(Location::new(0, 0))];
        filler.supermana_progress = false;
        filler.safe_supermana_progress_steps = Config::BOARD_SIZE + 4;
        filler.heuristic = 0;
        filler.efficiency = 0;

        assert!(
            MonsGameModel::root_focus_scout_score(&progress)
                > MonsGameModel::root_focus_scout_score(&filler),
            "exact safe supermana progress should improve scout fallback score"
        );
    }

    #[test]
    fn tactical_child_top2_promotes_close_exact_continuation_child() {
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            1,
        );

        let quiet = MoveClassFlags {
            quiet: true,
            ..MoveClassFlags::default()
        };
        let mut scored_states = vec![
            (1_000, ranked_child_state_for_test(&game, 1, 0, quiet)),
            (995, ranked_child_state_for_test(&game, 2, 0, quiet)),
            (986, ranked_child_state_for_test(&game, 3, 20, quiet)),
        ];

        MonsGameModel::enforce_tactical_child_top2(&mut scored_states, true, false);
        assert_eq!(scored_states[1].1.hash, 3);
    }

    #[test]
    fn truncate_child_states_keeps_exact_continuation_within_margin() {
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            1,
        );

        let quiet = MoveClassFlags {
            quiet: true,
            ..MoveClassFlags::default()
        };
        let truncated = MonsGameModel::truncate_child_states_with_coverage(
            vec![
                (1_000, ranked_child_state_for_test(&game, 1, 0, quiet)),
                (950, ranked_child_state_for_test(&game, 2, 20, quiet)),
            ],
            1,
            true,
            false,
        );
        assert_eq!(truncated.len(), 1);
        assert_eq!(truncated[0].1.hash, 2);
    }

    #[test]
    fn truncate_child_states_drops_distant_exact_continuation_outside_margin() {
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            1,
        );

        let quiet = MoveClassFlags {
            quiet: true,
            ..MoveClassFlags::default()
        };
        let truncated = MonsGameModel::truncate_child_states_with_coverage(
            vec![
                (1_000, ranked_child_state_for_test(&game, 1, 0, quiet)),
                (800, ranked_child_state_for_test(&game, 2, 20, quiet)),
            ],
            1,
            true,
            false,
        );
        assert_eq!(truncated.len(), 1);
        assert_eq!(truncated[0].1.hash, 1);
    }

    #[test]
    fn truncate_root_candidates_keeps_exact_safe_supermana_progress() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut progress = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(7, 5)),
                Input::Location(Location::new(6, 5)),
            ],
        )
        .expect("drainer advance should build a scored root");
        progress.heuristic = 100;
        progress.efficiency = 0;
        assert!(progress.supermana_progress);
        assert!(progress.safe_supermana_progress_steps < Config::BOARD_SIZE + 4);
        assert!(!progress.safe_supermana_pickup_now);

        let mut filler_a = progress.clone();
        filler_a.inputs = vec![Input::Location(Location::new(0, 0))];
        filler_a.supermana_progress = false;
        filler_a.safe_supermana_progress_steps = Config::BOARD_SIZE + 4;
        filler_a.heuristic = 2_000;

        let mut filler_b = filler_a.clone();
        filler_b.inputs = vec![Input::Location(Location::new(0, 1))];
        filler_b.heuristic = 1_950;

        progress.heuristic = 1_885;

        let truncated = MonsGameModel::truncate_root_candidates_with_class_coverage(
            vec![filler_a, filler_b, progress.clone()],
            2,
            false,
        );
        assert!(
            truncated
                .iter()
                .any(|candidate| candidate.inputs == progress.inputs),
            "exact safe supermana progress root should survive root branch truncation"
        );
    }

    #[test]
    fn truncate_root_candidates_keeps_exact_safe_opponent_mana_progress() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 6),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut progress = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(7, 6)),
                Input::Location(Location::new(6, 5)),
            ],
        )
        .expect("drainer advance should build a scored root");
        progress.heuristic = 1_885;
        progress.efficiency = 0;
        assert!(progress.opponent_mana_progress);
        assert!(progress.safe_opponent_mana_progress_steps < Config::BOARD_SIZE + 4);
        assert!(!progress.safe_opponent_mana_pickup_now);

        let mut filler_a = progress.clone();
        filler_a.inputs = vec![Input::Location(Location::new(0, 0))];
        filler_a.opponent_mana_progress = false;
        filler_a.safe_opponent_mana_progress_steps = Config::BOARD_SIZE + 4;
        filler_a.heuristic = 2_000;

        let mut filler_b = filler_a.clone();
        filler_b.inputs = vec![Input::Location(Location::new(0, 1))];
        filler_b.heuristic = 1_950;

        let truncated = MonsGameModel::truncate_root_candidates_with_class_coverage(
            vec![filler_a, filler_b, progress.clone()],
            2,
            false,
        );
        assert!(
            truncated
                .iter()
                .any(|candidate| candidate.inputs == progress.inputs),
            "exact safe opponent-mana progress root should survive root branch truncation"
        );
    }

    #[test]
    fn truncate_root_candidates_prefers_stronger_exact_same_turn_score_window() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut stronger = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(7, 5)),
                Input::Location(Location::new(6, 5)),
            ],
        )
        .expect("drainer advance should build a scored root");
        stronger.inputs = vec![Input::Location(Location::new(1, 1))];
        stronger.heuristic = 1_900;
        stronger.efficiency = 0;
        stronger.same_turn_score_window_value = 2;
        stronger.spirit_same_turn_score_setup_now = true;
        stronger.own_drainer_vulnerable = false;
        stronger.mana_handoff_to_opponent = false;

        let mut weaker = stronger.clone();
        weaker.inputs = vec![Input::Location(Location::new(0, 0))];
        weaker.heuristic = 2_000;
        weaker.same_turn_score_window_value = 1;
        weaker.spirit_same_turn_score_setup_now = false;

        let truncated = MonsGameModel::truncate_root_candidates_with_class_coverage(
            vec![weaker, stronger.clone()],
            1,
            false,
        );
        assert_eq!(truncated.len(), 1);
        assert_eq!(
            truncated[0].inputs, stronger.inputs,
            "stronger exact same-turn score window root should survive branch truncation even when a weaker window root has higher heuristic"
        );
    }

    #[test]
    fn tactical_child_top2_promotes_close_carrier_progress_child() {
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            1,
        );

        let quiet = MoveClassFlags {
            quiet: true,
            ..MoveClassFlags::default()
        };
        let carrier_progress = MoveClassFlags {
            carrier_progress: true,
            ..MoveClassFlags::default()
        };
        let mut scored_states = vec![
            (1_000, ranked_child_state_for_test(&game, 1, 0, quiet)),
            (995, ranked_child_state_for_test(&game, 2, 0, quiet)),
            (
                992,
                ranked_child_state_for_test(&game, 3, 20, carrier_progress),
            ),
        ];

        MonsGameModel::enforce_tactical_child_top2(&mut scored_states, true, false);
        assert_eq!(scored_states[1].1.hash, 3);
    }

    #[test]
    fn tactical_child_top2_keeps_existing_carrier_progress_in_front() {
        let game = game_with_items(
            vec![
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            1,
        );

        let quiet = MoveClassFlags {
            quiet: true,
            ..MoveClassFlags::default()
        };
        let carrier_progress = MoveClassFlags {
            carrier_progress: true,
            ..MoveClassFlags::default()
        };
        let mut scored_states = vec![
            (
                1_000,
                ranked_child_state_for_test(&game, 1, 20, carrier_progress),
            ),
            (995, ranked_child_state_for_test(&game, 2, 0, quiet)),
            (994, ranked_child_state_for_test(&game, 3, 0, quiet)),
        ];

        MonsGameModel::enforce_tactical_child_top2(&mut scored_states, true, false);
        assert_eq!(scored_states[0].1.hash, 1);
        assert_eq!(scored_states[1].1.hash, 2);
    }

    #[test]
    fn opponent_spirit_supermana_progress_child_is_not_quiet_reduced() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(6, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::Black, 0),
                    },
                ),
                (
                    Location::new(3, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
                (
                    Location::new(5, 8),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
            ],
            Color::Black,
            2,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.node_enum_limit = 256;
        config.node_branch_limit = 64;
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::Black,
            config.enable_enhanced_drainer_vulnerability,
        );
        let child = MonsGameModel::build_scored_root_move(
            &game,
            Color::Black,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(6, 10)),
                Input::Location(Location::new(5, 8)),
                Input::Location(Location::new(4, 9)),
            ],
        )
        .expect("mirrored opponent spirit supermana handoff inputs should build a scored root");
        let child_hash = MonsGameModel::search_state_hash(&child.game);

        let children =
            MonsGameModel::ranked_child_states(&game, Color::White, false, None, [0; 2], config);
        let ranked_child = children
            .iter()
            .find(|candidate| candidate.hash == child_hash)
            .expect("ranked child states should include mirrored opponent spirit setup");

        assert!(
            ranked_child.ordering_efficiency > 0,
            "dangerous opponent spirit setup should have positive actor-relative ordering efficiency"
        );
        assert!(
            !ranked_child.quiet_reduction_candidate,
            "dangerous opponent spirit setup should not be marked for quiet reduction"
        );
        assert!(
            MonsGameModel::is_selective_extension_candidate(
                ranked_child.tactical_extension_trigger,
                ranked_child.ordering_efficiency,
                ranked_child.classes,
            ),
            "dangerous opponent spirit setup should qualify for selective extension"
        );
    }

    #[test]
    fn ranked_child_states_surface_exact_supermana_progress_with_node_enum_cutoff() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(0, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Angel, Color::White, 0),
                    },
                ),
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        assert!(
            crate::models::automove_exact::exact_turn_summary(&game, Color::White)
                .safe_supermana_progress,
            "board should start with exact safe supermana progress"
        );
        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let limited_transitions = MonsGameModel::enumerate_legal_transitions(
            &game,
            1,
            SuggestedStartInputOptions::for_automove(),
        );
        assert!(
            !limited_transitions.iter().any(|transition| {
                MonsGameModel::transition_preserves_exact_progress(
                    transition,
                    Color::White,
                    Mana::Supermana,
                )
            }),
            "plain tiny child enumeration should miss exact supermana continuations on this board"
        );

        let mut cutoff_config = config;
        cutoff_config.node_enum_limit = 1;
        cutoff_config.node_branch_limit = 8;
        let children = MonsGameModel::ranked_child_states(
            &game,
            Color::White,
            true,
            None,
            [0; 2],
            cutoff_config,
        );

        assert!(
            children.iter().any(|candidate| {
                let game = &candidate.game;
                MonsGameModel::own_drainer_carries_specific_mana_safely(
                    &game.board,
                    Color::White,
                    Mana::Supermana,
                ) || (game.active_color == Color::White && {
                    let exact_turn =
                        crate::models::automove_exact::exact_turn_summary(game, Color::White);
                    exact_turn.safe_supermana_progress
                        || exact_turn.spirit_assisted_supermana_progress
                })
            }),
            "exact safe supermana continuation child should survive tiny child enumeration cutoff"
        );
    }

    #[test]
    fn ranked_child_states_surface_exact_opponent_mana_progress_with_node_enum_cutoff() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(0, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Angel, Color::White, 0),
                    },
                ),
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
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
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        assert!(
            crate::models::automove_exact::exact_turn_summary(&game, Color::White)
                .safe_opponent_mana_progress,
            "board should start with exact safe opponent-mana progress"
        );
        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let limited_transitions = MonsGameModel::enumerate_legal_transitions(
            &game,
            1,
            SuggestedStartInputOptions::for_automove(),
        );
        assert!(
            !limited_transitions.iter().any(|transition| {
                MonsGameModel::transition_preserves_exact_progress(
                    transition,
                    Color::White,
                    Mana::Regular(Color::Black),
                )
            }),
            "plain tiny child enumeration should miss exact opponent-mana continuations on this board"
        );

        let mut cutoff_config = config;
        cutoff_config.node_enum_limit = 1;
        cutoff_config.node_branch_limit = 8;
        let children = MonsGameModel::ranked_child_states(
            &game,
            Color::White,
            true,
            None,
            [0; 2],
            cutoff_config,
        );

        assert!(
            children.iter().any(|candidate| {
                let game = &candidate.game;
                MonsGameModel::own_drainer_carries_specific_mana_safely(
                    &game.board,
                    Color::White,
                    Mana::Regular(Color::Black),
                ) || (game.active_color == Color::White
                    && {
                        let exact_turn =
                            crate::models::automove_exact::exact_turn_summary(game, Color::White);
                        exact_turn.safe_opponent_mana_progress
                            || exact_turn.spirit_assisted_opponent_mana_progress
                            || exact_turn.spirit_assisted_denial
                    })
            }),
            "exact safe opponent-mana continuation child should survive tiny child enumeration cutoff"
        );
    }

    #[test]
    fn shorter_exact_safe_supermana_progress_gets_higher_root_priority() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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
            1,
        );

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(7, 5)),
                Input::Location(Location::new(6, 5)),
            ],
        )
        .expect("short supermana progress inputs should build a scored root");
        let mut long = short.clone();
        long.safe_supermana_progress_steps = 2;
        long.interview_soft_priority = MonsGameModel::interview_root_soft_priority(
            config,
            true,
            false,
            2,
            Config::BOARD_SIZE + 4,
            false,
            false,
            false,
            false,
            false,
        );

        assert!(short.supermana_progress);
        assert_eq!(short.safe_supermana_progress_steps, 1);
        assert_eq!(
            short.interview_soft_priority,
            MonsGameModel::interview_root_soft_priority(
                config,
                true,
                false,
                1,
                Config::BOARD_SIZE + 4,
                false,
                false,
                false,
                false,
                false,
            )
        );
        assert!(short.interview_soft_priority > long.interview_soft_priority);
        assert!(MonsGameModel::is_better_tactical_root_candidate(
            &short, &long
        ));
        let picked = MonsGameModel::pick_root_move_with_exploration(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            Color::White,
            config,
        );
        assert_eq!(picked, short.inputs);
    }

    #[test]
    fn shorter_exact_safe_opponent_mana_progress_gets_higher_root_priority() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
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
            1,
        );

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(7, 5)),
                Input::Location(Location::new(6, 4)),
            ],
        )
        .expect("short opponent mana progress inputs should build a scored root");
        let mut long = short.clone();
        long.safe_opponent_mana_progress_steps = 2;
        long.interview_soft_priority = MonsGameModel::interview_root_soft_priority(
            config,
            false,
            true,
            Config::BOARD_SIZE + 4,
            2,
            false,
            false,
            false,
            false,
            false,
        );

        assert!(short.opponent_mana_progress);
        assert_eq!(short.safe_opponent_mana_progress_steps, 1);
        assert_eq!(
            short.interview_soft_priority,
            MonsGameModel::interview_root_soft_priority(
                config,
                false,
                true,
                Config::BOARD_SIZE + 4,
                1,
                false,
                false,
                false,
                false,
                false,
            )
        );
        assert!(short.interview_soft_priority > long.interview_soft_priority);
        assert!(MonsGameModel::is_better_tactical_root_candidate(
            &short, &long
        ));
        let picked = MonsGameModel::pick_root_move_with_exploration(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            Color::White,
            config,
        );
        assert_eq!(picked, short.inputs);
    }

    #[test]
    fn ranked_root_moves_surface_safe_opponent_mana_pickup_when_available() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let white_spirit_base = Board::new().base(white_spirit);
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (white_spirit_base, Item::Mon { mon: white_spirit }),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let roots = MonsGameModel::ranked_root_moves(
            &game,
            Color::White,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );

        assert!(
            roots.iter().any(|root| root.safe_opponent_mana_pickup_now),
            "expected a surfaced safe opponent-mana pickup root, got inputs={:?}",
            roots
                .iter()
                .map(|root| root.inputs.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn ranked_root_moves_surface_safe_supermana_pickup_with_root_cutoff_when_available() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let white_spirit_base = Board::new().base(white_spirit);
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (white_spirit_base, Item::Mon { mon: white_spirit }),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_enum_limit = 1;
        let roots = MonsGameModel::ranked_root_moves(&game, Color::White, config);

        assert!(
            roots.iter().any(|root| root.safe_supermana_pickup_now),
            "expected a surfaced safe supermana pickup root under cutoff, got inputs={:?}",
            roots
                .iter()
                .map(|root| root.inputs.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn ranked_root_moves_surface_safe_opponent_mana_pickup_with_root_cutoff_when_available() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let white_spirit_base = Board::new().base(white_spirit);
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (white_spirit_base, Item::Mon { mon: white_spirit }),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_enum_limit = 1;
        let roots = MonsGameModel::ranked_root_moves(&game, Color::White, config);

        assert!(
            roots.iter().any(|root| root.safe_opponent_mana_pickup_now),
            "expected a surfaced safe opponent-mana pickup root under cutoff, got inputs={:?}",
            roots
                .iter()
                .map(|root| root.inputs.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn ranked_root_moves_surface_spirit_opponent_mana_score_when_available() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
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
                (
                    Location::new(10, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let roots = MonsGameModel::ranked_root_moves(
            &game,
            Color::White,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );

        assert!(
            roots.iter().any(|root| {
                MonsGameModel::apply_inputs_for_search_with_events(&game, &root.inputs)
                    .map(|(after, events)| {
                        MonsGameModel::events_include_spirit_target_move(&events)
                            && MonsGameModel::events_score_opponent_mana(&events, Color::White)
                            && after.white_score >= game.white_score + 2
                    })
                    .unwrap_or(false)
            }),
            "expected a surfaced spirit opponent-mana score root, got inputs={:?}",
            roots
                .iter()
                .map(|root| root.inputs.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn ranked_root_moves_surface_spirit_setup_into_same_turn_score_when_available() {
        let game = game_with_items(
            vec![
                (
                    Location::new(4, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(4, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 0),
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
        assert!(
            crate::models::automove_exact::exact_turn_summary(&game, Color::White)
                .spirit_assisted_score
        );
        let roots = MonsGameModel::ranked_root_moves(
            &game,
            Color::White,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );

        assert!(
            roots.iter().any(|root| {
                MonsGameModel::apply_inputs_for_search_with_events(&game, &root.inputs)
                    .map(|(after, _)| {
                        crate::models::automove_exact::exact_state_analysis(&after)
                            .color_summary(Color::White)
                            .immediate_window
                            .best_score
                            > 0
                    })
                    .unwrap_or(false)
            }),
            "expected a surfaced root that preserves the spirit-assisted same-turn score window, got inputs={:?}",
            roots
                .iter()
                .map(|root| root.inputs.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn spirit_setup_into_same_turn_score_is_preferred_when_available() {
        let game = game_with_items(
            vec![
                (
                    Location::new(4, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(4, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 0),
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
        assert!(
            crate::models::automove_exact::exact_turn_summary(&game, Color::White)
                .spirit_assisted_score
        );
        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-setup inputs should be legal");
        let after_best_score = crate::models::automove_exact::exact_state_analysis(&after)
            .color_summary(Color::White)
            .immediate_window
            .best_score;

        assert!(
            after_best_score > 0,
            "selected line should preserve the spirit-assisted same-turn scoring window, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn spirit_setup_into_same_turn_score_survives_root_enumeration_cutoff() {
        let game = game_with_items(
            vec![
                (
                    Location::new(4, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(4, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 0),
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
        assert!(
            crate::models::automove_exact::exact_turn_summary(&game, Color::White)
                .spirit_assisted_score
        );
        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_enum_limit = 0;
        let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-setup inputs should be legal");
        let after_best_score = crate::models::automove_exact::exact_state_analysis(&after)
            .color_summary(Color::White)
            .immediate_window
            .best_score;

        assert!(
            after_best_score > 0,
            "selected line should still preserve the spirit-assisted same-turn scoring window with capped root enumeration, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn selected_line_preserves_same_turn_supermana_threat_when_available() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let white_spirit_base = Board::new().base(white_spirit);
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (white_spirit_base, Item::Mon { mon: white_spirit }),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let exact_turn_before =
            crate::models::automove_exact::exact_turn_summary(&game, Color::White);
        assert!(
            exact_turn_before.safe_supermana_progress,
            "scenario should start with an exact same-turn supermana threat"
        );

        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected supermana-threat inputs should be legal");
        let exact_turn_after =
            crate::models::automove_exact::exact_turn_summary(&after, Color::White);
        assert!(
            exact_turn_after.safe_supermana_progress
                || matches!(
                    after.board.occupied().find(|(_, item)| matches!(
                        item,
                        Item::MonWithMana {
                            mon,
                            mana: Mana::Supermana,
                        } if mon.color == Color::White && mon.kind == MonKind::Drainer
                    )),
                    Some(_)
                ),
            "selected line should preserve the exact same-turn supermana threat, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn selected_line_preserves_same_turn_supermana_threat_with_root_cutoff() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let white_spirit_base = Board::new().base(white_spirit);
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (white_spirit_base, Item::Mon { mon: white_spirit }),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let exact_turn_before =
            crate::models::automove_exact::exact_turn_summary(&game, Color::White);
        assert!(
            exact_turn_before.safe_supermana_progress,
            "scenario should start with an exact same-turn supermana threat"
        );

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_enum_limit = 0;
        let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected supermana-threat inputs should be legal");
        let exact_turn_after =
            crate::models::automove_exact::exact_turn_summary(&after, Color::White);
        assert!(
            exact_turn_after.safe_supermana_progress
                || matches!(
                    after.board.occupied().find(|(_, item)| matches!(
                        item,
                        Item::MonWithMana {
                            mon,
                            mana: Mana::Supermana,
                        } if mon.color == Color::White && mon.kind == MonKind::Drainer
                    )),
                    Some(_)
                ),
            "selected line should preserve the exact same-turn supermana threat under capped root enumeration, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn spirit_converts_supermana_with_root_cutoff_when_direct_pickup_is_unsafe() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(10, 2),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(7, 3),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
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
        let exact_turn_before =
            crate::models::automove_exact::exact_turn_summary(&game, Color::White);
        assert!(
            !exact_turn_before.safe_supermana_progress,
            "direct safe supermana pickup should be unavailable before the spirit move"
        );

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_enum_limit = 0;
        let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-supermana inputs should be legal");
        let spirit_supermana_to_safe_drainer =
            events.iter().any(|event| {
                matches!(
                    event,
                    Event::SpiritTargetMove {
                        item: Item::Mana {
                            mana: Mana::Supermana,
                        },
                        to,
                        ..
                    } if *to == Location::new(10, 2)
                )
            }) && after.board.occupied().any(|(location, item)| {
                matches!(
                    item,
                    Item::MonWithMana {
                        mon,
                        mana: Mana::Supermana,
                    } if location == Location::new(10, 2)
                        && mon.color == Color::White
                        && mon.kind == MonKind::Drainer
                )
            });
        assert!(
            after.white_score >= game.white_score + 2 || spirit_supermana_to_safe_drainer,
            "selected line should convert the spirit supermana opportunity into either an immediate score or a safe drainer handoff under capped root enumeration, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn move_efficiency_rewards_faster_exact_safe_supermana_progress() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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

        let exact_turn_before =
            crate::models::automove_exact::exact_turn_summary(&game, Color::White);
        assert_eq!(exact_turn_before.safe_supermana_progress_steps, Some(1));

        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(
            &game,
            &[
                Input::Location(Location::new(6, 5)),
                Input::Location(Location::new(5, 5)),
            ],
        )
        .expect("shortening supermana path inputs should be legal");
        let exact_turn_after =
            crate::models::automove_exact::exact_turn_summary(&after, Color::White);
        assert_eq!(exact_turn_after.safe_supermana_progress_steps, Some(0));

        let delta = MonsGameModel::move_efficiency_delta(
            &game,
            &after,
            Color::White,
            &events,
            false,
            false,
            false,
            0,
            0,
        );
        assert!(delta > 0);
    }

    #[test]
    fn move_efficiency_rewards_faster_exact_safe_opponent_mana_progress() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
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

        let exact_turn_before =
            crate::models::automove_exact::exact_turn_summary(&game, Color::White);
        assert_eq!(exact_turn_before.safe_opponent_mana_progress_steps, Some(1));

        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(
            &game,
            &[
                Input::Location(Location::new(6, 5)),
                Input::Location(Location::new(5, 4)),
            ],
        )
        .expect("shortening opponent mana path inputs should be legal");
        let exact_turn_after =
            crate::models::automove_exact::exact_turn_summary(&after, Color::White);
        assert_eq!(exact_turn_after.safe_opponent_mana_progress_steps, Some(0));

        let delta = MonsGameModel::move_efficiency_delta(
            &game,
            &after,
            Color::White,
            &events,
            false,
            false,
            false,
            0,
            0,
        );
        assert!(delta > 0);
    }

    #[test]
    fn spirit_moves_own_mana_closer_to_pool_when_setup_is_strongest() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let game = game_with_items(
            vec![
                (Location::new(9, 7), Item::Mon { mon: white_spirit }),
                (
                    Location::new(9, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 8),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
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

        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (_, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-setup inputs should be legal");
        let spirit_moved_own_mana_closer = events.iter().any(|event| {
            let Event::SpiritTargetMove {
                item:
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                from,
                to,
                ..
            } = event
            else {
                return false;
            };
            let from_pool_steps = from
                .distance(&Location::new(10, 0))
                .min(from.distance(&Location::new(10, 10)));
            let to_pool_steps = to
                .distance(&Location::new(10, 0))
                .min(to.distance(&Location::new(10, 10)));
            to_pool_steps < from_pool_steps
        });
        assert!(
            spirit_moved_own_mana_closer,
            "selected line should use spirit to move own mana closer to a pool, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn spirit_moves_own_mana_closer_to_pool_with_root_cutoff_when_setup_is_strongest() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let game = game_with_items(
            vec![
                (Location::new(9, 7), Item::Mon { mon: white_spirit }),
                (
                    Location::new(9, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 8),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_enum_limit = 0;
        let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
        let (_, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-setup inputs should be legal");
        let spirit_moved_own_mana_closer = events.iter().any(|event| {
            let Event::SpiritTargetMove {
                item:
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                from,
                to,
                ..
            } = event
            else {
                return false;
            };
            let from_pool_steps = from
                .distance(&Location::new(10, 0))
                .min(from.distance(&Location::new(10, 10)));
            let to_pool_steps = to
                .distance(&Location::new(10, 0))
                .min(to.distance(&Location::new(10, 10)));
            to_pool_steps < from_pool_steps
        });
        assert!(
            spirit_moved_own_mana_closer,
            "selected line should still use spirit to move own mana closer to a pool under capped root enumeration, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn spirit_setup_with_shorter_exact_score_path_gets_higher_root_priority() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(6, 3),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 7),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
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
        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(6, 5)),
                Input::Location(Location::new(5, 7)),
                Input::Location(Location::new(6, 6)),
            ],
        )
        .expect("shorter spirit setup inputs should build a scored root");
        let long = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(6, 5)),
                Input::Location(Location::new(5, 7)),
                Input::Location(Location::new(6, 7)),
            ],
        )
        .expect("longer spirit setup inputs should build a scored root");

        assert!(short.spirit_own_mana_setup_now);
        assert!(long.spirit_own_mana_setup_now);
        assert!(!short.spirit_same_turn_score_setup_now);
        assert!(!long.spirit_same_turn_score_setup_now);
        assert_eq!(short.score_path_best_steps, 7);
        assert_eq!(long.score_path_best_steps, 8);
        assert!(MonsGameModel::is_better_tactical_root_candidate(
            &short, &long
        ));
        let picked = MonsGameModel::pick_root_move_with_exploration(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            Color::White,
            config,
        );
        assert_eq!(picked, short.inputs);
    }

    #[test]
    fn spirit_supermana_setup_prefers_shorter_exact_secure_continuation() {
        let mut game = game_with_items(
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
                        mana: Mana::Supermana,
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit supermana handoff inputs should build a scored root");
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_supermana_progress_steps = short.safe_supermana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);

        assert!(short.spirit_own_mana_setup_now);
        assert!(short.supermana_progress);
        assert!(short.safe_supermana_progress_steps < Config::BOARD_SIZE + 4);
        assert!(short.score_path_best_steps > 0);
        assert!(long.score_path_best_steps < short.score_path_best_steps);
        assert!(MonsGameModel::is_better_tactical_root_candidate(
            &short, &long
        ));
        let picked = MonsGameModel::pick_root_move_with_exploration(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            Color::White,
            config,
        );
        assert_eq!(picked, short.inputs);
    }

    #[test]
    fn filtered_root_candidates_prefer_spirit_supermana_exact_secure_continuation() {
        let mut game = game_with_items(
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
                        mana: Mana::Supermana,
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit supermana handoff inputs should build a scored root");
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_supermana_progress_steps = short.safe_supermana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);

        let filtered = MonsGameModel::filtered_root_candidate_indices(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            Color::White,
            config,
        );
        assert_eq!(filtered, vec![1]);
    }

    #[test]
    fn root_priority_sort_prefers_spirit_supermana_exact_continuation_when_heuristic_tied() {
        let mut game = game_with_items(
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
                        mana: Mana::Supermana,
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit supermana handoff inputs should build a scored root");
        short.heuristic = 100;
        short.efficiency = 0;
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_supermana_progress_steps = short.safe_supermana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);
        long.heuristic = 100;
        long.efficiency = 0;

        let short_inputs = short.inputs.clone();
        let long_inputs = long.inputs.clone();
        let mut candidates = vec![long, short];
        MonsGameModel::sort_root_candidates_by_search_priority(candidates.as_mut_slice());

        let short_pos = candidates
            .iter()
            .position(|candidate| candidate.inputs == short_inputs)
            .expect("short spirit root should remain in sorted roots");
        let long_pos = candidates
            .iter()
            .position(|candidate| candidate.inputs == long_inputs)
            .expect("long spirit root should remain in sorted roots");
        assert!(short_pos < long_pos);
    }

    #[test]
    fn ranked_score_sort_prefers_spirit_supermana_exact_continuation_when_score_tied() {
        let mut game = game_with_items(
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
                        mana: Mana::Supermana,
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit supermana handoff inputs should build a scored root");
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_supermana_progress_steps = short.safe_supermana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);

        let short_inputs = short.inputs.clone();
        let long_inputs = long.inputs.clone();
        let ordered =
            MonsGameModel::sort_root_moves_by_ranked_scores(vec![long, short], &[100, 100]);
        let short_pos = ordered
            .iter()
            .position(|candidate| candidate.inputs == short_inputs)
            .expect("short spirit root should remain in ranked-score order");
        let long_pos = ordered
            .iter()
            .position(|candidate| candidate.inputs == long_inputs)
            .expect("long spirit root should remain in ranked-score order");
        assert!(short_pos < long_pos);
    }

    #[test]
    fn reply_risk_shortlist_prefers_spirit_supermana_exact_continuation_when_score_tied() {
        let mut game = game_with_items(
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
                        mana: Mana::Supermana,
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_reply_risk_score_margin = 0;
        config.root_reply_risk_shortlist_max = 1;
        config.root_reply_risk_reply_limit = 1;
        config.root_reply_risk_node_share_bp = 1_000;
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit supermana handoff inputs should build a scored root");
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_supermana_progress_steps = short.safe_supermana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);

        let picked = MonsGameModel::pick_root_move_with_reply_risk_guard(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            &[0, 1],
            Color::White,
            config,
        );
        assert_eq!(picked, Some(1));
    }

    #[test]
    fn normal_root_safety_shortlist_prefers_spirit_supermana_exact_continuation_when_score_tied() {
        let mut game = game_with_items(
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
                        mana: Mana::Supermana,
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.enable_normal_root_safety_deep_floor = false;
        config.node_enum_limit = 1;
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit supermana handoff inputs should build a scored root");
        let mut long = short.clone();
        long.safe_supermana_progress_steps = short.safe_supermana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);

        let picked = MonsGameModel::pick_root_move_with_normal_safety(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            &[0, 1],
            Color::White,
            config,
        );
        assert_eq!(picked, short.inputs);
    }

    #[test]
    fn focused_root_candidates_prioritize_spirit_supermana_exact_continuation_when_scout_tied() {
        let mut game = game_with_items(
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
                        mana: Mana::Supermana,
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.depth = 2;
        config.root_focus_k = 2;
        config.enable_two_pass_root_allocation = true;
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit supermana handoff inputs should build a scored root");
        short.heuristic = 100;
        short.efficiency = 0;
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_supermana_progress_steps = short.safe_supermana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);
        long.heuristic = 100;
        long.efficiency = 0;

        let mut filler = short.clone();
        filler.inputs = vec![Input::Location(Location::new(0, 1))];
        filler.spirit_own_mana_setup_now = false;
        filler.supermana_progress = false;
        filler.score_path_best_steps = Config::BOARD_SIZE * 3;
        filler.safe_supermana_progress_steps = Config::BOARD_SIZE + 4;
        filler.heuristic = 2_000;
        filler.efficiency = 0;

        let mut low = short.clone();
        low.inputs = vec![Input::Location(Location::new(0, 2))];
        low.spirit_own_mana_setup_now = false;
        low.supermana_progress = false;
        low.score_path_best_steps = Config::BOARD_SIZE * 3;
        low.safe_supermana_progress_steps = Config::BOARD_SIZE + 4;
        low.heuristic = -1_000;
        low.efficiency = 0;

        let short_inputs = short.inputs.clone();
        let long_inputs = long.inputs.clone();
        let (focused, _) = MonsGameModel::focused_root_candidates(
            &game,
            Color::White,
            vec![long, filler, low, short],
            config,
            true,
        );
        let short_pos = focused
            .iter()
            .position(|candidate| candidate.inputs == short_inputs)
            .expect("short spirit root should survive focus");
        let long_pos = focused
            .iter()
            .position(|candidate| candidate.inputs == long_inputs)
            .expect("long spirit root should survive focus");
        assert!(short_pos < long_pos);
    }

    #[test]
    fn focused_root_candidates_keep_exact_safe_supermana_progress_with_scout_bonus() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.depth = 2;
        config.root_focus_k = 1;
        config.enable_two_pass_root_allocation = true;
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut progress = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(7, 5)),
                Input::Location(Location::new(6, 5)),
            ],
        )
        .expect("drainer advance should build a scored root");
        progress.heuristic = 0;
        progress.efficiency = 0;
        assert!(progress.supermana_progress);
        assert!(!progress.safe_supermana_pickup_now);
        assert!(!progress.spirit_own_mana_setup_now);

        let mut filler = progress.clone();
        filler.inputs = vec![Input::Location(Location::new(0, 0))];
        filler.supermana_progress = false;
        filler.safe_supermana_progress_steps = Config::BOARD_SIZE + 4;
        filler.score_path_best_steps = Config::BOARD_SIZE * 3;
        filler.heuristic = 2_500;
        filler.efficiency = 0;

        let progress_inputs = progress.inputs.clone();
        let (focused, _) = MonsGameModel::focused_root_candidates(
            &game,
            Color::White,
            vec![filler, progress],
            config,
            true,
        );

        assert!(
            focused.iter().any(|candidate| candidate.inputs == progress_inputs),
            "exact safe supermana progress root should survive scout focus despite lower raw heuristic"
        );
    }

    #[test]
    fn focused_root_candidates_prioritize_spirit_supermana_exact_continuation_on_narrow_spread_fallback(
    ) {
        let mut game = game_with_items(
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
                        mana: Mana::Supermana,
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.depth = 2;
        config.root_focus_k = 2;
        config.enable_two_pass_root_allocation = true;
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit supermana handoff inputs should build a scored root");
        short.heuristic = 100;
        short.efficiency = 0;
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_supermana_progress_steps = short.safe_supermana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);
        long.heuristic = 100;
        long.efficiency = 0;

        let mut filler = short.clone();
        filler.inputs = vec![Input::Location(Location::new(0, 1))];
        filler.spirit_own_mana_setup_now = false;
        filler.supermana_progress = false;
        filler.score_path_best_steps = Config::BOARD_SIZE * 3;
        filler.safe_supermana_progress_steps = Config::BOARD_SIZE + 4;
        filler.heuristic = 0;
        filler.efficiency = 0;

        let short_inputs = short.inputs.clone();
        let long_inputs = long.inputs.clone();
        let (focused, _) = MonsGameModel::focused_root_candidates(
            &game,
            Color::White,
            vec![long, filler, short],
            config,
            true,
        );
        assert_eq!(focused.len(), 3);
        let short_pos = focused
            .iter()
            .position(|candidate| candidate.inputs == short_inputs)
            .expect("short spirit root should survive narrow fallback");
        let long_pos = focused
            .iter()
            .position(|candidate| candidate.inputs == long_inputs)
            .expect("long spirit root should survive narrow fallback");
        assert!(short_pos < long_pos);
    }

    #[test]
    fn spirit_opponent_mana_setup_prefers_shorter_exact_secure_continuation() {
        let mut game = game_with_items(
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit opponent mana handoff inputs should build a scored root");
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_opponent_mana_progress_steps = short.safe_opponent_mana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);

        assert!(short.spirit_own_mana_setup_now);
        assert!(short.opponent_mana_progress);
        assert!(short.safe_opponent_mana_progress_steps < Config::BOARD_SIZE + 4);
        assert!(short.score_path_best_steps > 0);
        assert!(long.score_path_best_steps < short.score_path_best_steps);
        assert!(MonsGameModel::is_better_tactical_root_candidate(
            &short, &long
        ));
        let picked = MonsGameModel::pick_root_move_with_exploration(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            Color::White,
            config,
        );
        assert_eq!(picked, short.inputs);
    }

    #[test]
    fn filtered_root_candidates_prefer_spirit_opponent_exact_secure_continuation() {
        let mut game = game_with_items(
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit opponent mana handoff inputs should build a scored root");
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_opponent_mana_progress_steps = short.safe_opponent_mana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);

        let filtered = MonsGameModel::filtered_root_candidate_indices(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            Color::White,
            config,
        );
        assert_eq!(filtered, vec![1]);
    }

    #[test]
    fn root_priority_sort_prefers_spirit_opponent_exact_continuation_when_heuristic_tied() {
        let mut game = game_with_items(
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit opponent mana handoff inputs should build a scored root");
        short.heuristic = 100;
        short.efficiency = 0;
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_opponent_mana_progress_steps = short.safe_opponent_mana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);
        long.heuristic = 100;
        long.efficiency = 0;

        let short_inputs = short.inputs.clone();
        let long_inputs = long.inputs.clone();
        let mut candidates = vec![long, short];
        MonsGameModel::sort_root_candidates_by_search_priority(candidates.as_mut_slice());

        let short_pos = candidates
            .iter()
            .position(|candidate| candidate.inputs == short_inputs)
            .expect("short spirit root should remain in sorted roots");
        let long_pos = candidates
            .iter()
            .position(|candidate| candidate.inputs == long_inputs)
            .expect("long spirit root should remain in sorted roots");
        assert!(short_pos < long_pos);
    }

    #[test]
    fn ranked_score_sort_prefers_spirit_opponent_exact_continuation_when_score_tied() {
        let mut game = game_with_items(
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

        let config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit opponent mana handoff inputs should build a scored root");
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_opponent_mana_progress_steps = short.safe_opponent_mana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);

        let short_inputs = short.inputs.clone();
        let long_inputs = long.inputs.clone();
        let ordered =
            MonsGameModel::sort_root_moves_by_ranked_scores(vec![long, short], &[100, 100]);
        let short_pos = ordered
            .iter()
            .position(|candidate| candidate.inputs == short_inputs)
            .expect("short spirit root should remain in ranked-score order");
        let long_pos = ordered
            .iter()
            .position(|candidate| candidate.inputs == long_inputs)
            .expect("long spirit root should remain in ranked-score order");
        assert!(short_pos < long_pos);
    }

    #[test]
    fn reply_risk_shortlist_prefers_spirit_opponent_exact_continuation_when_score_tied() {
        let mut game = game_with_items(
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_reply_risk_score_margin = 0;
        config.root_reply_risk_shortlist_max = 1;
        config.root_reply_risk_reply_limit = 1;
        config.root_reply_risk_node_share_bp = 1_000;
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit opponent mana handoff inputs should build a scored root");
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_opponent_mana_progress_steps = short.safe_opponent_mana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);

        let picked = MonsGameModel::pick_root_move_with_reply_risk_guard(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            &[0, 1],
            Color::White,
            config,
        );
        assert_eq!(picked, Some(1));
    }

    #[test]
    fn normal_root_safety_shortlist_prefers_spirit_opponent_exact_continuation_when_score_tied() {
        let mut game = game_with_items(
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.enable_normal_root_safety_deep_floor = false;
        config.node_enum_limit = 1;
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit opponent mana handoff inputs should build a scored root");
        let mut long = short.clone();
        long.safe_opponent_mana_progress_steps = short.safe_opponent_mana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);

        let picked = MonsGameModel::pick_root_move_with_normal_safety(
            &game,
            &[
                root_evaluation_for_test(&long, 100),
                root_evaluation_for_test(&short, 100),
            ],
            &[0, 1],
            Color::White,
            config,
        );
        assert_eq!(picked, short.inputs);
    }

    #[test]
    fn focused_root_candidates_prioritize_spirit_opponent_exact_continuation_when_scout_tied() {
        let mut game = game_with_items(
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.depth = 2;
        config.root_focus_k = 2;
        config.enable_two_pass_root_allocation = true;
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit opponent mana handoff inputs should build a scored root");
        short.heuristic = 100;
        short.efficiency = 0;
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_opponent_mana_progress_steps = short.safe_opponent_mana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);
        long.heuristic = 100;
        long.efficiency = 0;

        let mut filler = short.clone();
        filler.inputs = vec![Input::Location(Location::new(0, 1))];
        filler.spirit_own_mana_setup_now = false;
        filler.opponent_mana_progress = false;
        filler.score_path_best_steps = Config::BOARD_SIZE * 3;
        filler.safe_opponent_mana_progress_steps = Config::BOARD_SIZE + 4;
        filler.heuristic = 2_000;
        filler.efficiency = 0;

        let mut low = short.clone();
        low.inputs = vec![Input::Location(Location::new(0, 2))];
        low.spirit_own_mana_setup_now = false;
        low.opponent_mana_progress = false;
        low.score_path_best_steps = Config::BOARD_SIZE * 3;
        low.safe_opponent_mana_progress_steps = Config::BOARD_SIZE + 4;
        low.heuristic = -1_000;
        low.efficiency = 0;

        let short_inputs = short.inputs.clone();
        let long_inputs = long.inputs.clone();
        let (focused, _) = MonsGameModel::focused_root_candidates(
            &game,
            Color::White,
            vec![long, filler, low, short],
            config,
            true,
        );
        let short_pos = focused
            .iter()
            .position(|candidate| candidate.inputs == short_inputs)
            .expect("short spirit root should survive focus");
        let long_pos = focused
            .iter()
            .position(|candidate| candidate.inputs == long_inputs)
            .expect("long spirit root should survive focus");
        assert!(short_pos < long_pos);
    }

    #[test]
    fn focused_root_candidates_prioritize_spirit_opponent_exact_continuation_on_narrow_spread_fallback(
    ) {
        let mut game = game_with_items(
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

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.depth = 2;
        config.root_focus_k = 2;
        config.enable_two_pass_root_allocation = true;
        let own_drainer_vulnerable_before = MonsGameModel::is_own_drainer_vulnerable_next_turn(
            &game,
            Color::White,
            config.enable_enhanced_drainer_vulnerability,
        );
        let mut short = MonsGameModel::build_scored_root_move(
            &game,
            Color::White,
            config,
            own_drainer_vulnerable_before,
            &[
                Input::Location(Location::new(4, 0)),
                Input::Location(Location::new(5, 2)),
                Input::Location(Location::new(6, 1)),
            ],
        )
        .expect("spirit opponent mana handoff inputs should build a scored root");
        short.heuristic = 100;
        short.efficiency = 0;
        let mut long = short.clone();
        long.inputs = vec![Input::Location(Location::new(0, 0))];
        long.safe_opponent_mana_progress_steps = short.safe_opponent_mana_progress_steps + 2;
        long.score_path_best_steps = short.score_path_best_steps.saturating_sub(1);
        long.heuristic = 100;
        long.efficiency = 0;

        let mut filler = short.clone();
        filler.inputs = vec![Input::Location(Location::new(0, 1))];
        filler.spirit_own_mana_setup_now = false;
        filler.opponent_mana_progress = false;
        filler.score_path_best_steps = Config::BOARD_SIZE * 3;
        filler.safe_opponent_mana_progress_steps = Config::BOARD_SIZE + 4;
        filler.heuristic = 0;
        filler.efficiency = 0;

        let short_inputs = short.inputs.clone();
        let long_inputs = long.inputs.clone();
        let (focused, _) = MonsGameModel::focused_root_candidates(
            &game,
            Color::White,
            vec![long, filler, short],
            config,
            true,
        );
        assert_eq!(focused.len(), 3);
        let short_pos = focused
            .iter()
            .position(|candidate| candidate.inputs == short_inputs)
            .expect("short spirit root should survive narrow fallback");
        let long_pos = focused
            .iter()
            .position(|candidate| candidate.inputs == long_inputs)
            .expect("long spirit root should survive narrow fallback");
        assert!(short_pos < long_pos);
    }

    #[test]
    fn spirit_moves_opponent_mana_closer_to_pool_when_setup_is_strongest() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let game = game_with_items(
            vec![
                (Location::new(9, 7), Item::Mon { mon: white_spirit }),
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
        );

        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (_, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-opponent-setup inputs should be legal");
        let spirit_moved_opponent_mana_closer = events.iter().any(|event| {
            let Event::SpiritTargetMove {
                item:
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                from,
                to,
                ..
            } = event
            else {
                return false;
            };
            let from_pool_steps = from
                .distance(&Location::new(10, 0))
                .min(from.distance(&Location::new(10, 10)));
            let to_pool_steps = to
                .distance(&Location::new(10, 0))
                .min(to.distance(&Location::new(10, 10)));
            to_pool_steps < from_pool_steps
        });
        assert!(
            spirit_moved_opponent_mana_closer,
            "selected line should use spirit to move opponent mana closer to a pool, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn spirit_moves_opponent_mana_closer_to_pool_with_root_cutoff_when_setup_is_strongest() {
        let white_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let game = game_with_items(
            vec![
                (Location::new(9, 7), Item::Mon { mon: white_spirit }),
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
        );

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_enum_limit = 0;
        let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
        let (_, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-opponent-setup inputs should be legal");
        let spirit_moved_opponent_mana_closer = events.iter().any(|event| {
            let Event::SpiritTargetMove {
                item:
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                from,
                to,
                ..
            } = event
            else {
                return false;
            };
            let from_pool_steps = from
                .distance(&Location::new(10, 0))
                .min(from.distance(&Location::new(10, 10)));
            let to_pool_steps = to
                .distance(&Location::new(10, 0))
                .min(to.distance(&Location::new(10, 10)));
            to_pool_steps < from_pool_steps
        });
        assert!(
            spirit_moved_opponent_mana_closer,
            "selected line should use spirit to move opponent mana closer to a pool under capped root enumeration, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn selected_line_preserves_same_turn_opponent_mana_threat_when_available() {
        let game = game_with_items(
            vec![
                (
                    Location::new(5, 3),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 3),
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
        let exact_turn_before =
            crate::models::automove_exact::exact_turn_summary(&game, Color::White);
        assert!(
            exact_turn_before.safe_opponent_mana_progress
                || exact_turn_before.spirit_assisted_denial,
            "scenario should start with an exact same-turn opponent-mana threat"
        );

        let inputs = MonsGameModel::smart_search_best_inputs(
            &game,
            SmartSearchConfig::from_preference(SmartAutomovePreference::Fast),
        );
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected spirit-setup inputs should be legal");
        let exact_turn_after =
            crate::models::automove_exact::exact_turn_summary(&after, Color::White);
        assert!(
            exact_turn_after.safe_opponent_mana_progress || exact_turn_after.spirit_assisted_denial,
            "selected line should preserve the exact same-turn opponent-mana threat, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn selected_line_preserves_same_turn_opponent_mana_threat_with_root_cutoff() {
        let game = game_with_items(
            vec![
                (
                    Location::new(5, 3),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 3),
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
        let exact_turn_before =
            crate::models::automove_exact::exact_turn_summary(&game, Color::White);
        assert!(
            exact_turn_before.safe_opponent_mana_progress
                || exact_turn_before.spirit_assisted_denial,
            "scenario should start with an exact same-turn opponent-mana threat"
        );

        let mut config = SmartSearchConfig::from_preference(SmartAutomovePreference::Fast);
        config.root_enum_limit = 0;
        let inputs = MonsGameModel::smart_search_best_inputs(&game, config);
        let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(&game, &inputs)
            .expect("selected opponent-mana-threat inputs should be legal");
        let exact_turn_after =
            crate::models::automove_exact::exact_turn_summary(&after, Color::White);
        assert!(
            exact_turn_after.safe_opponent_mana_progress || exact_turn_after.spirit_assisted_denial,
            "selected line should preserve the exact same-turn opponent-mana threat under capped root enumeration, inputs={:?}, events={:?}",
            inputs,
            events
        );
    }

    #[test]
    fn legal_transition_enumeration_matches_apply_helper() {
        let game = MonsGame::new(false);
        let transitions = MonsGameModel::enumerate_legal_transitions(
            &game,
            64,
            SuggestedStartInputOptions::for_automove(),
        );
        assert!(!transitions.is_empty());

        for transition in transitions.iter().take(16) {
            let (after, events) = MonsGameModel::apply_inputs_for_search_with_events(
                &game,
                transition.inputs.as_slice(),
            )
            .expect("enumerated transition input should remain legal");
            assert_eq!(after.fen(), transition.game.fen());
            assert_eq!(events, transition.events);
        }
    }

    #[test]
    fn exact_drainer_attack_oracle_matches_exhaustive_same_turn_search() {
        let game = game_with_items(
            vec![
                (
                    Location::new(5, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::White, 0),
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

        let exact = crate::models::automove_exact::exact_turn_summary(&game, Color::White)
            .can_attack_opponent_drainer;
        let exhaustive = exhaustive_same_turn_reachable(&game, Color::White, |_, events| {
            MonsGameModel::events_include_opponent_drainer_fainted(events, Color::White)
        });

        assert_eq!(exact, exhaustive);
    }

    #[test]
    fn exact_drainer_vulnerability_oracle_matches_exhaustive_next_turn_search() {
        let game = game_with_items(
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
            2,
        );

        let exact = MonsGameModel::is_own_drainer_vulnerable_next_turn(&game, Color::White, true);
        let mut opponent_turn = game.clone_for_simulation();
        opponent_turn.active_color = Color::Black;
        opponent_turn.actions_used_count = 0;
        opponent_turn.mons_moves_count = 0;
        let exhaustive =
            exhaustive_same_turn_reachable(&opponent_turn, Color::Black, |_, events| {
                MonsGameModel::events_include_opponent_drainer_fainted(events, Color::Black)
            });

        assert_eq!(exact, exhaustive);
    }

    #[test]
    fn exact_drainer_pickup_score_oracle_matches_exhaustive_same_turn_search() {
        let game = game_with_items(
            vec![
                (
                    Location::new(8, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let exact = crate::models::automove_exact::exact_state_analysis(&game)
            .white
            .best_drainer_pickup
            .map_or(false, |path| {
                path.total_moves <= Config::MONS_MOVES_PER_TURN
            });
        let exhaustive = exhaustive_same_turn_reachable(&game, Color::White, |state, _| {
            state.white_score > game.white_score
        });

        assert_eq!(exact, exhaustive);
    }

    #[test]
    fn exact_safe_supermana_progress_oracle_matches_reachable_state_search() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
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

        let exact = crate::models::automove_exact::exact_turn_summary(&game, Color::White)
            .safe_supermana_progress;
        let exhaustive = exhaustive_same_turn_reachable(&game, Color::White, |state, _| {
            drainer_carries_exact_safe_mana(&state.board, Color::White, Mana::Supermana)
        });

        assert_eq!(exact, exhaustive);
    }

    #[test]
    fn exact_spirit_supermana_progress_oracle_matches_exhaustive_same_turn_search() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 2),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 1),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(5, 3),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
            2,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        let exact = crate::models::automove_exact::exact_turn_summary(&game, Color::White)
            .spirit_assisted_supermana_progress;
        let exhaustive = exhaustive_same_turn_reachable(&game, Color::White, |state, _| {
            state.white_score > game.white_score
                || drainer_carries_exact_safe_mana(&state.board, Color::White, Mana::Supermana)
        });

        assert_eq!(exact, exhaustive);
    }

    #[test]
    fn exact_spirit_opponent_mana_progress_oracle_matches_exhaustive_same_turn_search() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 2),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(5, 3),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
            2,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        let exact = crate::models::automove_exact::exact_turn_summary(&game, Color::White)
            .spirit_assisted_opponent_mana_progress;
        let exhaustive = exhaustive_same_turn_reachable(&game, Color::White, |state, _| {
            state.white_score > game.white_score
                || drainer_carries_exact_safe_mana(
                    &state.board,
                    Color::White,
                    Mana::Regular(Color::Black),
                )
        });

        assert_eq!(exact, exhaustive);
    }

    #[test]
    fn exact_spirit_opponent_mana_score_oracle_matches_exhaustive_same_turn_search() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
            2,
        );

        let exact = crate::models::automove_exact::exact_state_analysis(&game)
            .white
            .spirit
            .same_turn_opponent_mana_score;
        let exhaustive = exhaustive_same_turn_reachable(&game, Color::White, |state, _| {
            state.white_score > game.white_score
        });

        assert_eq!(exact, exhaustive);
    }

    #[test]
    fn model_remove_item_invalidates_cached_start_suggestions() {
        let mut model = MonsGameModel::new_for_simulation();
        let initial_suggestions = match model.game.process_input(vec![], true, false) {
            Output::LocationsToStartFrom(locations) => locations,
            output => panic!("expected start locations, got {:?}", output),
        };
        let removed = initial_suggestions
            .iter()
            .copied()
            .find(|location| model.game.board.item(*location).is_some())
            .expect("expected removable starting piece");

        model.remove_item(removed);

        let updated_suggestions = match model.game.process_input(vec![], true, false) {
            Output::LocationsToStartFrom(locations) => locations,
            output => panic!("expected start locations after removal, got {:?}", output),
        };
        assert!(!updated_suggestions.contains(&removed));
    }
}

fn random_index(len: usize) -> usize {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(0..len)
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OutputModelKind {
    InvalidInput,
    LocationsToStartFrom,
    NextInputOptions,
    Events,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OutputModel {
    pub kind: OutputModelKind,
    locations: Vec<Location>,
    next_inputs: Vec<NextInputModel>,
    events: Vec<EventModel>,
    input_fen: String,
}

#[wasm_bindgen]
impl OutputModel {
    pub fn locations(&self) -> Vec<Location> {
        self.locations.clone()
    }

    pub fn next_inputs(&self) -> Vec<NextInputModel> {
        self.next_inputs.clone()
    }

    pub fn events(&self) -> Vec<EventModel> {
        self.events.clone()
    }

    pub fn input_fen(&self) -> String {
        self.input_fen.clone()
    }
}

impl OutputModel {
    fn new(output: Output, input_fen: &str) -> Self {
        match output {
            Output::InvalidInput => Self {
                kind: OutputModelKind::InvalidInput,
                locations: vec![],
                next_inputs: vec![],
                events: vec![],
                input_fen: input_fen.to_string(),
            },
            Output::LocationsToStartFrom(locations) => Self {
                kind: OutputModelKind::LocationsToStartFrom,
                locations,
                next_inputs: vec![],
                events: vec![],
                input_fen: input_fen.to_string(),
            },
            Output::NextInputOptions(next_inputs) => Self {
                kind: OutputModelKind::NextInputOptions,
                locations: vec![],
                next_inputs: next_inputs
                    .into_iter()
                    .map(|input| NextInputModel::new(&input))
                    .collect(),
                events: vec![],
                input_fen: input_fen.to_string(),
            },
            Output::Events(events) => Self {
                kind: OutputModelKind::Events,
                locations: vec![],
                next_inputs: vec![],
                events: events
                    .into_iter()
                    .map(|event| EventModel::new(&event))
                    .collect(),
                input_fen: input_fen.to_string(),
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SquareModel {
    pub kind: SquareModelKind,
    pub color: Option<Color>,
    pub mon_kind: Option<MonKind>,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SquareModelKind {
    Regular,
    ConsumableBase,
    SupermanaBase,
    ManaBase,
    ManaPool,
    MonBase,
}

impl SquareModel {
    fn new(item: &Square) -> Self {
        match item {
            Square::Regular => SquareModel {
                kind: SquareModelKind::Regular,
                color: None,
                mon_kind: None,
            },
            Square::ConsumableBase => SquareModel {
                kind: SquareModelKind::ConsumableBase,
                color: None,
                mon_kind: None,
            },
            Square::SupermanaBase => SquareModel {
                kind: SquareModelKind::SupermanaBase,
                color: None,
                mon_kind: None,
            },
            Square::ManaBase { color } => SquareModel {
                kind: SquareModelKind::ManaBase,
                color: Some(*color),
                mon_kind: None,
            },
            Square::ManaPool { color } => SquareModel {
                kind: SquareModelKind::ManaPool,
                color: Some(*color),
                mon_kind: None,
            },
            Square::MonBase { kind, color } => SquareModel {
                kind: SquareModelKind::MonBase,
                color: Some(*color),
                mon_kind: Some(*kind),
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemModelKind {
    Mon,
    Mana,
    MonWithMana,
    MonWithConsumable,
    Consumable,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ItemModel {
    pub kind: ItemModelKind,
    pub mon: Option<Mon>,
    pub mana: Option<ManaModel>,
    pub consumable: Option<Consumable>,
}

impl ItemModel {
    fn new(item: &Item) -> Self {
        let (kind, mon, mana, consumable) = match item {
            Item::Mon { mon } => (ItemModelKind::Mon, Some(*mon), None, None),
            Item::Mana { mana } => (ItemModelKind::Mana, None, Some(ManaModel::new(mana)), None),
            Item::MonWithMana { mon, mana } => (
                ItemModelKind::MonWithMana,
                Some(*mon),
                Some(ManaModel::new(mana)),
                None,
            ),
            Item::MonWithConsumable { mon, consumable } => (
                ItemModelKind::MonWithConsumable,
                Some(*mon),
                None,
                Some(*consumable),
            ),
            Item::Consumable { consumable } => {
                (ItemModelKind::Consumable, None, None, Some(*consumable))
            }
        };
        Self {
            kind,
            mon,
            mana,
            consumable,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ManaModel {
    pub kind: ManaKind,
    pub color: Color,
}

impl ManaModel {
    fn new(item: &Mana) -> Self {
        match item {
            Mana::Regular(color) => ManaModel {
                kind: ManaKind::Regular,
                color: *color,
            },
            Mana::Supermana => ManaModel {
                kind: ManaKind::Supermana,
                color: Color::White,
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ManaKind {
    Regular,
    Supermana,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NextInputModel {
    pub location: Option<Location>,
    pub modifier: Option<Modifier>,
    pub kind: NextInputKind,
    pub actor_mon_item: Option<ItemModel>,
}

impl NextInputModel {
    fn new(input: &NextInput) -> Self {
        Self {
            location: match input.input {
                Input::Location(loc) => Some(loc),
                _ => None,
            },
            modifier: match input.input {
                Input::Modifier(modifier) => Some(modifier),
                _ => None,
            },
            kind: input.kind,
            actor_mon_item: if input.actor_mon_item.is_some() {
                Some(ItemModel::new(&input.actor_mon_item.unwrap()))
            } else {
                None
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EventModelKind {
    MonMove,
    ManaMove,
    ManaScored,
    MysticAction,
    DemonAction,
    DemonAdditionalStep,
    SpiritTargetMove,
    PickupBomb,
    PickupPotion,
    PickupMana,
    MonFainted,
    ManaDropped,
    SupermanaBackToBase,
    BombAttack,
    MonAwake,
    BombExplosion,
    NextTurn,
    GameOver,
    Takeback,
    UsePotion,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EventModel {
    pub kind: EventModelKind,
    pub item: Option<ItemModel>,
    pub mon: Option<Mon>,
    pub mana: Option<ManaModel>,
    pub loc1: Option<Location>,
    pub loc2: Option<Location>,
    pub color: Option<Color>,
}

impl EventModel {
    fn new(event: &Event) -> Self {
        match event {
            Event::MonMove { item, from, to } => EventModel {
                kind: EventModelKind::MonMove,
                item: Some(ItemModel::new(item)),
                mon: None,
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::ManaMove { mana, from, to } => EventModel {
                kind: EventModelKind::ManaMove,
                item: None,
                mon: None,
                mana: Some(ManaModel::new(mana)),
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::ManaScored { mana, at } => EventModel {
                kind: EventModelKind::ManaScored,
                item: None,
                mon: None,
                mana: Some(ManaModel::new(mana)),
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::MysticAction { mystic, from, to } => EventModel {
                kind: EventModelKind::MysticAction,
                item: None,
                mon: Some(mystic.clone()),
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::DemonAction { demon, from, to } => EventModel {
                kind: EventModelKind::DemonAction,
                item: None,
                mon: Some(demon.clone()),
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::DemonAdditionalStep { demon, from, to } => EventModel {
                kind: EventModelKind::DemonAdditionalStep,
                item: None,
                mon: Some(demon.clone()),
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::SpiritTargetMove {
                item,
                from,
                to,
                by: _,
            } => EventModel {
                kind: EventModelKind::SpiritTargetMove,
                item: Some(ItemModel::new(item)),
                mon: None,
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::PickupBomb { by, at } => EventModel {
                kind: EventModelKind::PickupBomb,
                item: None,
                mon: Some(by.clone()),
                mana: None,
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::PickupPotion { by, at } => EventModel {
                kind: EventModelKind::PickupPotion,
                item: Some(ItemModel::new(by)),
                mon: None,
                mana: None,
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::PickupMana { mana, by, at } => EventModel {
                kind: EventModelKind::PickupMana,
                item: None,
                mon: Some(by.clone()),
                mana: Some(ManaModel::new(mana)),
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::MonFainted { mon, from, to } => EventModel {
                kind: EventModelKind::MonFainted,
                item: None,
                mon: Some(mon.clone()),
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::ManaDropped { mana, at } => EventModel {
                kind: EventModelKind::ManaDropped,
                item: None,
                mon: None,
                mana: Some(ManaModel::new(mana)),
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::SupermanaBackToBase { from, to } => EventModel {
                kind: EventModelKind::SupermanaBackToBase,
                item: None,
                mon: None,
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::BombAttack { by, from, to } => EventModel {
                kind: EventModelKind::BombAttack,
                item: None,
                mon: Some(by.clone()),
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::MonAwake { mon, at } => EventModel {
                kind: EventModelKind::MonAwake,
                item: None,
                mon: Some(mon.clone()),
                mana: None,
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::BombExplosion { at } => EventModel {
                kind: EventModelKind::BombExplosion,
                item: None,
                mon: None,
                mana: None,
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::NextTurn { color } => EventModel {
                kind: EventModelKind::NextTurn,
                item: None,
                mon: None,
                mana: None,
                loc1: None,
                loc2: None,
                color: Some(*color),
            },
            Event::GameOver { winner } => EventModel {
                kind: EventModelKind::GameOver,
                item: None,
                mon: None,
                mana: None,
                loc1: None,
                loc2: None,
                color: Some(*winner),
            },
            Event::Takeback => EventModel {
                kind: EventModelKind::Takeback,
                item: None,
                mon: None,
                mana: None,
                loc1: None,
                loc2: None,
                color: None,
            },
            Event::UsePotion { from, to } => EventModel {
                kind: EventModelKind::UsePotion,
                item: None,
                mon: None,
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VerboseTrackingEntityModel {
    fen: String,
    color: Color,
    events: Vec<Event>,
}

impl VerboseTrackingEntityModel {
    fn new(entity: &VerboseTrackingEntity) -> Self {
        Self {
            fen: entity.fen.clone(),
            color: entity.color,
            events: entity.events.clone(),
        }
    }
}

#[wasm_bindgen]
impl VerboseTrackingEntityModel {
    pub fn fen(&self) -> String {
        self.fen.clone()
    }
    pub fn color(&self) -> Color {
        self.color
    }
    pub fn events(&self) -> Vec<EventModel> {
        self.events.iter().map(|e| EventModel::new(e)).collect()
    }
    pub fn events_fen(&self) -> String {
        self.events
            .iter()
            .map(|e| e.fen())
            .collect::<Vec<_>>()
            .join(" ")
    }
}
