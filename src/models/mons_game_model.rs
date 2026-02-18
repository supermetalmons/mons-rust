#[cfg(any(target_arch = "wasm32", test))]
use crate::models::scoring::{
    evaluate_preferability_with_weights, ScoringWeights, BALANCED_DISTANCE_SCORING_WEIGHTS,
    DEFAULT_SCORING_WEIGHTS, FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS,
    FINISHER_BALANCED_SOFT_SCORING_WEIGHTS, MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS,
    RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS, TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS,
    TACTICAL_BALANCED_SCORING_WEIGHTS,
};
use crate::*;

#[wasm_bindgen]
#[derive(Debug)]
pub struct MonsGameModel {
    game: MonsGame,
    #[cfg(target_arch = "wasm32")]
    smart_search_in_progress: std::rc::Rc<std::cell::Cell<bool>>,
}

impl Clone for MonsGameModel {
    fn clone(&self) -> Self {
        Self::with_game(self.game.clone())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
const MIN_SMART_SEARCH_DEPTH: usize = 1;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_SMART_SEARCH_DEPTH: usize = 4;
#[cfg(any(target_arch = "wasm32", test))]
const MIN_SMART_MAX_VISITED_NODES: usize = 32;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_SMART_MAX_VISITED_NODES: usize = 20_000;
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
const SMART_TWO_PASS_ROOT_SCOUT_DEPTH: usize = 2;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TWO_PASS_ROOT_SCOUT_MIN_NODES: usize = 96;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TWO_PASS_ROOT_FOCUS_SCORE_MARGIN: i32 = 2_000;
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
const SMART_ROOT_REPLY_RISK_SHORTLIST_NORMAL: usize = 5;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_REPLY_RISK_REPLY_LIMIT_FAST: usize = 8;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_REPLY_RISK_REPLY_LIMIT_NORMAL: usize = 12;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_FAST: i32 = 600;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_NORMAL: i32 = 1_200;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_TWO_PASS_ROOT_NARROW_SPREAD_FALLBACK: i32 = 700;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_MOVE_CLASS_ROOT_SCORE_MARGIN: i32 = 120;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_MOVE_CLASS_CHILD_SCORE_MARGIN: i32 = 110;
#[cfg(any(target_arch = "wasm32", test))]
const SMART_ROOT_SPIRIT_DEVELOPMENT_SCORE_MARGIN: i32 = 700;
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

#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_BALANCED_DISTANCE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        spirit_on_own_base_penalty: 260,
        score_race_path_progress: 72,
        opponent_score_race_path_progress: 132,
        immediate_score_window: 70,
        opponent_immediate_score_window: 170,
        spirit_action_utility: 58,
        ..BALANCED_DISTANCE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_TACTICAL_BALANCED_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        spirit_on_own_base_penalty: 260,
        score_race_path_progress: 82,
        opponent_score_race_path_progress: 155,
        immediate_score_window: 82,
        opponent_immediate_score_window: 215,
        spirit_action_utility: 64,
        ..TACTICAL_BALANCED_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_TACTICAL_BALANCED_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        spirit_on_own_base_penalty: 260,
        score_race_path_progress: 92,
        opponent_score_race_path_progress: 180,
        immediate_score_window: 95,
        opponent_immediate_score_window: 250,
        spirit_action_utility: 70,
        ..TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        spirit_on_own_base_penalty: 260,
        score_race_path_progress: 140,
        opponent_score_race_path_progress: 130,
        immediate_score_window: 210,
        opponent_immediate_score_window: 175,
        spirit_action_utility: 58,
        ..FINISHER_BALANCED_SOFT_SCORING_WEIGHTS
    };
#[cfg(any(target_arch = "wasm32", test))]
const RUNTIME_NORMAL_FINISHER_BALANCED_SOFT_AGGRESSIVE_SPIRIT_BASE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        spirit_on_own_base_penalty: 260,
        score_race_path_progress: 160,
        opponent_score_race_path_progress: 145,
        immediate_score_window: 260,
        opponent_immediate_score_window: 195,
        spirit_action_utility: 62,
        ..FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS
    };

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SmartAutomovePreference {
    Fast,
    Normal,
}

#[cfg(any(target_arch = "wasm32", test))]
impl SmartAutomovePreference {
    fn from_api_value(value: &str) -> Option<Self> {
        let normalized = value.trim();
        if normalized.eq_ignore_ascii_case("fast") {
            Some(Self::Fast)
        } else if normalized.eq_ignore_ascii_case("normal") {
            Some(Self::Normal)
        } else {
            None
        }
    }

    fn as_api_value(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Normal => "normal",
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
    enable_root_mana_handoff_guard: bool,
    enable_forced_drainer_attack: bool,
    enable_forced_drainer_attack_fallback: bool,
    enable_root_drainer_safety_prefilter: bool,
    enable_root_spirit_development_pref: bool,
    enable_root_reply_risk_guard: bool,
    root_reply_risk_score_margin: i32,
    root_reply_risk_shortlist_max: usize,
    root_reply_risk_reply_limit: usize,
    root_reply_risk_node_share_bp: i32,
    enable_move_class_coverage: bool,
    enable_normal_root_safety_rerank: bool,
    enable_normal_root_safety_deep_floor: bool,
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
                tuned.enable_root_mana_handoff_guard = false;
                tuned.enable_forced_drainer_attack = true;
                tuned.enable_forced_drainer_attack_fallback = true;
                tuned.enable_root_drainer_safety_prefilter = true;
                tuned.enable_root_spirit_development_pref = true;
                tuned.enable_root_reply_risk_guard = true;
                tuned.root_reply_risk_score_margin = SMART_ROOT_REPLY_RISK_SCORE_MARGIN;
                tuned.root_reply_risk_shortlist_max = SMART_ROOT_REPLY_RISK_SHORTLIST_FAST;
                tuned.root_reply_risk_reply_limit = SMART_ROOT_REPLY_RISK_REPLY_LIMIT_FAST;
                tuned.root_reply_risk_node_share_bp = SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_FAST;
                tuned.enable_move_class_coverage = false;
                tuned.enable_normal_root_safety_rerank = false;
                tuned.enable_normal_root_safety_deep_floor = false;
                tuned
            }
            SmartAutomovePreference::Normal => {
                let mut tuned = Self::with_normal_deeper_shape(config);
                tuned.max_visited_nodes = (tuned.max_visited_nodes * 3 / 2)
                    .clamp(tuned.max_visited_nodes, MAX_SMART_MAX_VISITED_NODES);
                tuned.root_branch_limit = tuned.root_branch_limit.saturating_sub(4).clamp(8, 36);
                tuned.node_branch_limit = (tuned.node_branch_limit + 2).clamp(8, 18);
                tuned.root_enum_limit =
                    (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 220);
                tuned.node_enum_limit =
                    (tuned.node_branch_limit * 5).clamp(tuned.node_branch_limit, 120);
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
                tuned.enable_root_mana_handoff_guard = false;
                tuned.enable_forced_drainer_attack = true;
                tuned.enable_forced_drainer_attack_fallback = true;
                tuned.enable_root_drainer_safety_prefilter = true;
                tuned.enable_root_spirit_development_pref = true;
                tuned.enable_root_reply_risk_guard = false;
                tuned.enable_move_class_coverage = true;
                tuned.enable_normal_root_safety_rerank = true;
                tuned.enable_normal_root_safety_deep_floor = true;
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
            enable_root_mana_handoff_guard: false,
            enable_forced_drainer_attack: true,
            enable_forced_drainer_attack_fallback: true,
            enable_root_drainer_safety_prefilter: true,
            enable_root_spirit_development_pref: true,
            enable_root_reply_risk_guard: false,
            root_reply_risk_score_margin: SMART_ROOT_REPLY_RISK_SCORE_MARGIN,
            root_reply_risk_shortlist_max: SMART_ROOT_REPLY_RISK_SHORTLIST_FAST,
            root_reply_risk_reply_limit: SMART_ROOT_REPLY_RISK_REPLY_LIMIT_FAST,
            root_reply_risk_node_share_bp: SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_FAST,
            enable_move_class_coverage: false,
            enable_normal_root_safety_rerank: false,
            enable_normal_root_safety_deep_floor: false,
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
    spirit_development: bool,
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
    spirit_development: bool,
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
enum MoveClass {
    ImmediateScore,
    DrainerAttack,
    DrainerSafetyRecover,
    SpiritDevelopment,
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
    spirit_development: bool,
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
            MoveClass::SpiritDevelopment => self.spirit_development,
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
    tactical_extension_trigger: bool,
    quiet_reduction_candidate: bool,
    classes: MoveClassFlags,
}

#[cfg(target_arch = "wasm32")]
struct AsyncSmartSearchState {
    game: MonsGame,
    perspective: Color,
    config: SmartSearchConfig,
    root_moves: Vec<ScoredRootMove>,
    next_index: usize,
    visited_nodes: usize,
    alpha: i32,
    scored_roots: Vec<RootEvaluation>,
    transposition_table: std::collections::HashMap<u64, TranspositionEntry>,
}

#[wasm_bindgen]
impl MonsGameModel {
    fn with_game(game: MonsGame) -> Self {
        Self {
            game,
            #[cfg(target_arch = "wasm32")]
            smart_search_in_progress: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
    }

    pub fn new() -> MonsGameModel {
        Self::with_game(MonsGame::new(true))
    }

    #[wasm_bindgen(js_name = newForSimulation)]
    pub fn new_for_simulation() -> MonsGameModel {
        Self::with_game(MonsGame::new(false))
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
        MonsGame::from_fen(fen, false).map(Self::with_game)
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
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;

        let Some(preference) = SmartAutomovePreference::from_api_value(preference) else {
            let message = format!(
                "invalid smart automove mode; expected '{}' or '{}'",
                SmartAutomovePreference::Fast.as_api_value(),
                SmartAutomovePreference::Normal.as_api_value()
            );
            return js_sys::Promise::reject(&JsValue::from_str(message.as_str()));
        };

        if self.smart_search_in_progress.get() {
            return js_sys::Promise::reject(&JsValue::from_str(
                "smart automove already in progress",
            ));
        }

        if let Some(opening_inputs) = Self::white_first_turn_opening_next_inputs(&self.game) {
            let mut game = self.game.clone_for_simulation();
            let input_fen = Input::fen_from_array(&opening_inputs);
            let output = game.process_input(opening_inputs, false, false);
            if matches!(output, Output::Events(_)) {
                return js_sys::Promise::resolve(&JsValue::from(OutputModel::new(
                    output,
                    input_fen.as_str(),
                )));
            }
        }

        self.smart_search_in_progress.set(true);
        let in_progress = self.smart_search_in_progress.clone();

        let config = Self::with_runtime_scoring_weights(
            &self.game,
            SmartSearchConfig::from_preference(preference),
        );
        let perspective = self.game.active_color;
        let game = self.game.clone_for_simulation();
        let root_moves = Self::ranked_root_moves(&game, perspective, config);
        let (root_moves, scout_visited_nodes) =
            Self::focused_root_candidates(&game, perspective, root_moves, config, true);

        let state = Rc::new(RefCell::new(AsyncSmartSearchState {
            game,
            perspective,
            config,
            root_moves,
            next_index: 0,
            visited_nodes: scout_visited_nodes,
            alpha: i32::MIN,
            scored_roots: Vec::new(),
            transposition_table: std::collections::HashMap::new(),
        }));

        js_sys::Promise::new(&mut move |resolve, reject| {
            let global = js_sys::global();
            let set_timeout = match js_sys::Reflect::get(&global, &JsValue::from_str("setTimeout"))
                .ok()
                .and_then(|value| value.dyn_into::<js_sys::Function>().ok())
            {
                Some(function) => function,
                None => {
                    in_progress.set(false);
                    let _ = reject.call1(
                        &JsValue::NULL,
                        &JsValue::from_str("setTimeout is not available"),
                    );
                    return;
                }
            };

            let tick: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
            let tick_for_closure = tick.clone();
            let state_inner = state.clone();
            let resolve_inner = resolve.clone();
            let reject_inner = reject.clone();
            let set_timeout_inner = set_timeout.clone();
            let in_progress_inner = in_progress.clone();

            *tick.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                let done = {
                    let mut borrowed = state_inner.borrow_mut();
                    Self::advance_async_search(&mut borrowed)
                };

                if done {
                    let output = {
                        let mut borrowed = state_inner.borrow_mut();
                        Self::finalize_async_search(&mut borrowed)
                    };
                    in_progress_inner.set(false);
                    let _ = resolve_inner.call1(&JsValue::NULL, &JsValue::from(output));
                    tick_for_closure.borrow_mut().take();
                    return;
                }

                let callback = {
                    let borrowed = tick_for_closure.borrow();
                    borrowed.as_ref().map(|cb| cb.as_ref().clone())
                };

                if let Some(cb) = callback {
                    if let Err(err) = set_timeout_inner.call2(
                        &JsValue::NULL,
                        cb.unchecked_ref(),
                        &JsValue::from_f64(0.0),
                    ) {
                        in_progress_inner.set(false);
                        let _ = reject_inner.call1(&JsValue::NULL, &err);
                        tick_for_closure.borrow_mut().take();
                    }
                }
            }) as Box<dyn FnMut()>));

            let initial_callback = {
                let borrowed = tick.borrow();
                borrowed.as_ref().map(|cb| cb.as_ref().clone())
            };
            if let Some(cb) = initial_callback {
                let schedule_result =
                    set_timeout.call2(&JsValue::NULL, cb.unchecked_ref(), &JsValue::from_f64(0.0));
                if let Err(err) = schedule_result {
                    in_progress.set(false);
                    let _ = reject.call1(&JsValue::NULL, &err);
                    tick.borrow_mut().take();
                }
            }
        })
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

        let mut inputs = Vec::new();
        let mut output = game.process_input(vec![], false, false);

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
                    output = game.process_input(inputs.clone(), false, false);
                }
                Output::NextInputOptions(options) => {
                    if options.is_empty() {
                        return OutputModel::new(Output::InvalidInput, "");
                    }
                    let random_index = random_index(options.len());
                    let next_input = options[random_index].input.clone();
                    inputs.push(next_input);
                    output = game.process_input(inputs.clone(), false, false);
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
        let mut locations = self
            .game
            .board
            .items
            .keys()
            .cloned()
            .collect::<Vec<Location>>();
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

        let opening_step = game.mons_moves_count.max(0) as usize;
        if opening_step >= WHITE_OPENING_BOOK[0].len() {
            return None;
        }

        let current_fen = game.fen();
        let mut viable_sequences = Vec::new();

        for (sequence_index, sequence) in WHITE_OPENING_BOOK.iter().enumerate() {
            let mut simulated = MonsGame::new(false);
            let mut prefix_is_valid = true;
            for step_fen in sequence.iter().take(opening_step) {
                let step_inputs = Input::array_from_fen(step_fen);
                if !matches!(
                    simulated.process_input(step_inputs, false, false),
                    Output::Events(_)
                ) {
                    prefix_is_valid = false;
                    break;
                }
            }

            if !prefix_is_valid || simulated.fen() != current_fen {
                continue;
            }

            let next_inputs = Input::array_from_fen(sequence[opening_step]);
            let mut probe = game.clone_for_simulation();
            if matches!(
                probe.process_input(next_inputs.clone(), true, false),
                Output::Events(_)
            ) {
                viable_sequences.push(sequence_index);
            }
        }

        if viable_sequences.is_empty() {
            return None;
        }

        let chosen = viable_sequences[random_index(viable_sequences.len())];
        Some(Input::array_from_fen(
            WHITE_OPENING_BOOK[chosen][opening_step],
        ))
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl MonsGameModel {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn with_runtime_scoring_weights(
        game: &MonsGame,
        mut config: SmartSearchConfig,
    ) -> SmartSearchConfig {
        config.scoring_weights = Self::runtime_phase_adaptive_scoring_weights(game, config.depth);
        config
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
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

    fn build_scored_root_move(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        own_drainer_vulnerable_before: bool,
        inputs: &[Input],
    ) -> Option<ScoredRootMove> {
        let (simulated_game, events) = Self::apply_inputs_for_search_with_events(game, inputs)?;
        let efficiency = if config.enable_root_efficiency {
            Self::move_efficiency_delta(
                game,
                &simulated_game,
                perspective,
                &events,
                true,
                config.enable_backtrack_penalty,
                config.enable_root_mana_handoff_guard,
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
        let own_drainer_vulnerable = if config.enable_root_drainer_safety_prefilter {
            Self::is_own_drainer_vulnerable_next_turn(&simulated_game, perspective)
        } else {
            false
        };
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

        Some(ScoredRootMove {
            inputs: inputs.to_vec(),
            game: simulated_game,
            heuristic: heuristic.saturating_add(ordering_bonus),
            efficiency,
            wins_immediately,
            attacks_opponent_drainer,
            own_drainer_vulnerable,
            spirit_development,
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
        board.items.iter().any(|(&location, item)| {
            location != base && Self::is_awake_spirit_item_for_color(item, color)
        })
    }

    fn spirit_action_target_count(board: &Board, location: Location) -> i32 {
        location
            .reachable_by_spirit_action()
            .into_iter()
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
            .count() as i32
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

    fn opponent_awake_drainer_location(board: &Board, perspective: Color) -> Option<Location> {
        board.items.iter().find_map(|(&location, item)| {
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
            .reachable_by_mystic_action()
            .into_iter()
            .map(|source| from.distance(&source))
            .min()
            .unwrap_or(i32::MAX)
    }

    fn min_steps_to_demon_attack_source(from: Location, target: Location) -> i32 {
        target
            .reachable_by_demon_action()
            .into_iter()
            .map(|source| from.distance(&source))
            .min()
            .unwrap_or(i32::MAX)
    }

    fn can_attempt_forced_drainer_attack_fallback(game: &MonsGame, perspective: Color) -> bool {
        if !game.player_can_move_mon() {
            return false;
        }
        let Some(opponent_drainer_location) =
            Self::opponent_awake_drainer_location(&game.board, perspective)
        else {
            return false;
        };

        let remaining_mon_moves = (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0);
        if remaining_mon_moves <= 0 {
            return false;
        }

        let opponent_drainer_guarded = Self::is_location_guarded_by_angel(
            &game.board,
            perspective.other(),
            opponent_drainer_location,
        );
        let bomb_pickup_locations = game
            .board
            .items
            .iter()
            .filter_map(|(&location, item)| match item {
                Item::Consumable {
                    consumable: Consumable::BombOrPotion,
                } => Some(location),
                _ => None,
            })
            .collect::<Vec<_>>();

        for (&location, item) in &game.board.items {
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
                return true;
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
                    return true;
                }
            }

            for bomb_location in &bomb_pickup_locations {
                let to_bomb = location.distance(bomb_location);
                if to_bomb > remaining_mon_moves {
                    continue;
                }
                let moves_after_pickup = remaining_mon_moves - to_bomb;
                if bomb_location.distance(&opponent_drainer_location) <= moves_after_pickup + 3 {
                    return true;
                }
            }
        }

        false
    }

    fn collect_forced_drainer_attack_inputs(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
        max_candidates: usize,
    ) -> Vec<Vec<Input>> {
        let mut memo_true = std::collections::HashSet::<u64>::new();
        let mut attack_inputs = Vec::new();
        let mut continuation_budget = Self::forced_drainer_attack_fallback_node_budget(config);
        let enum_limit = Self::forced_drainer_attack_fallback_enum_limit(config);
        let mut root_inputs = Self::enumerate_legal_inputs(game, usize::MAX);
        if root_inputs.len() > enum_limit {
            root_inputs.truncate(enum_limit);
        }
        for inputs in root_inputs {
            if attack_inputs.len() >= max_candidates.max(1) {
                break;
            }

            let Some((after, events)) = Self::apply_inputs_for_search_with_events(game, &inputs)
            else {
                continue;
            };
            if Self::events_include_opponent_drainer_fainted(&events, perspective) {
                attack_inputs.push(inputs);
                continue;
            }
            if after.active_color != perspective {
                continue;
            }
            if Self::can_attack_opponent_drainer_before_turn_ends(
                &after,
                perspective,
                enum_limit,
                &mut continuation_budget,
                &mut memo_true,
            ) {
                attack_inputs.push(inputs);
            }
        }

        attack_inputs
    }

    fn can_attack_opponent_drainer_before_turn_ends(
        game: &MonsGame,
        perspective: Color,
        enum_limit: usize,
        continuation_budget: &mut usize,
        memo_true: &mut std::collections::HashSet<u64>,
    ) -> bool {
        if game.active_color != perspective || *continuation_budget == 0 {
            return false;
        }
        let state_hash = Self::search_state_hash(game);
        if memo_true.contains(&state_hash) {
            return true;
        }

        let mut legal_inputs = Self::enumerate_legal_inputs(game, usize::MAX);
        if legal_inputs.len() > enum_limit {
            legal_inputs.truncate(enum_limit);
        }
        for inputs in legal_inputs {
            if *continuation_budget == 0 {
                break;
            }
            *continuation_budget = continuation_budget.saturating_sub(1);
            let Some((after, events)) = Self::apply_inputs_for_search_with_events(game, &inputs)
            else {
                continue;
            };
            if Self::events_include_opponent_drainer_fainted(&events, perspective) {
                memo_true.insert(state_hash);
                return true;
            }
            if after.active_color == perspective
                && Self::can_attack_opponent_drainer_before_turn_ends(
                    &after,
                    perspective,
                    enum_limit,
                    continuation_budget,
                    memo_true,
                )
            {
                memo_true.insert(state_hash);
                return true;
            }
        }

        false
    }

    fn ranked_root_moves(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
    ) -> Vec<ScoredRootMove> {
        let mut candidates = Vec::new();
        let own_drainer_vulnerable_before = if config.enable_move_class_coverage {
            Self::is_own_drainer_vulnerable_next_turn(game, perspective)
        } else {
            false
        };

        for inputs in Self::enumerate_legal_inputs(game, config.root_enum_limit) {
            if let Some(candidate) = Self::build_scored_root_move(
                game,
                perspective,
                config,
                own_drainer_vulnerable_before,
                inputs.as_slice(),
            ) {
                candidates.push(candidate);
            }
        }

        candidates.sort_by(|a, b| b.heuristic.cmp(&a.heuristic));
        let mut has_winning_candidate = candidates
            .iter()
            .any(|candidate| candidate.wins_immediately);
        let mut forced_turn_attack_input_fens: Option<std::collections::HashSet<String>> = None;
        if config.enable_forced_drainer_attack
            && config.enable_forced_drainer_attack_fallback
            && !has_winning_candidate
            && !candidates
                .iter()
                .any(|candidate| candidate.attacks_opponent_drainer)
            && Self::can_attempt_forced_drainer_attack_fallback(game, perspective)
        {
            let fallback_limit = Self::forced_drainer_attack_fallback_candidates_limit(config);
            let fallback_inputs = Self::collect_forced_drainer_attack_inputs(
                game,
                perspective,
                config,
                fallback_limit,
            );

            if !fallback_inputs.is_empty() {
                let forced_fens = fallback_inputs
                    .iter()
                    .map(|inputs| Input::fen_from_array(inputs.as_slice()))
                    .collect::<std::collections::HashSet<_>>();
                let mut seen_inputs = candidates
                    .iter()
                    .map(|candidate| Input::fen_from_array(candidate.inputs.as_slice()))
                    .collect::<std::collections::HashSet<_>>();

                for inputs in fallback_inputs {
                    let input_fen = Input::fen_from_array(inputs.as_slice());
                    if !seen_inputs.insert(input_fen) {
                        continue;
                    }
                    if let Some(candidate) = Self::build_scored_root_move(
                        game,
                        perspective,
                        config,
                        own_drainer_vulnerable_before,
                        inputs.as_slice(),
                    ) {
                        candidates.push(candidate);
                    }
                }

                candidates.sort_by(|a, b| b.heuristic.cmp(&a.heuristic));
                has_winning_candidate = candidates
                    .iter()
                    .any(|candidate| candidate.wins_immediately);
                forced_turn_attack_input_fens = Some(forced_fens);
            }
        }

        if config.enable_forced_drainer_attack
            && !has_winning_candidate
            && candidates
                .iter()
                .any(|candidate| candidate.attacks_opponent_drainer)
        {
            candidates.retain(|candidate| candidate.attacks_opponent_drainer);
        } else if config.enable_forced_drainer_attack {
            if let Some(forced_fens) = forced_turn_attack_input_fens {
                candidates.retain(|candidate| {
                    forced_fens.contains(&Input::fen_from_array(candidate.inputs.as_slice()))
                });
            }
        }
        if candidates.len() > config.root_branch_limit {
            if config.enable_move_class_coverage {
                candidates = Self::truncate_root_candidates_with_class_coverage(
                    candidates,
                    config.root_branch_limit,
                );
            } else {
                candidates.truncate(config.root_branch_limit);
            }
        }
        candidates
    }

    fn truncate_root_candidates_with_class_coverage(
        candidates: Vec<ScoredRootMove>,
        limit: usize,
    ) -> Vec<ScoredRootMove> {
        if candidates.len() <= limit {
            return candidates;
        }

        let best_heuristic = candidates[0].heuristic;
        let min_critical_heuristic =
            best_heuristic.saturating_sub(SMART_MOVE_CLASS_ROOT_SCORE_MARGIN.max(0));
        let mut selected_indices = std::collections::HashSet::<usize>::new();
        for class in [
            MoveClass::ImmediateScore,
            MoveClass::DrainerAttack,
            MoveClass::DrainerSafetyRecover,
            MoveClass::SpiritDevelopment,
        ] {
            if let Some((index, _)) = candidates.iter().enumerate().find(|(_, candidate)| {
                candidate.classes.has(class) && candidate.heuristic >= min_critical_heuristic
            }) {
                selected_indices.insert(index);
            }
        }

        let mut shortlisted = Vec::with_capacity(limit);
        for (index, candidate) in candidates.iter().enumerate() {
            if shortlisted.len() >= limit {
                break;
            }
            if selected_indices.contains(&index) {
                shortlisted.push(candidate.clone());
            }
        }
        for (index, candidate) in candidates.iter().enumerate() {
            if shortlisted.len() >= limit {
                break;
            }
            if selected_indices.contains(&index) {
                continue;
            }
            shortlisted.push(candidate.clone());
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
            spirit_development,
            carrier_progress,
            material,
            quiet,
        }
    }

    fn has_actor_carrier_progress(before: &MonsGame, after: &MonsGame, actor_color: Color) -> bool {
        let before_snapshot = Self::move_efficiency_snapshot(before, actor_color);
        let after_snapshot = Self::move_efficiency_snapshot(after, actor_color);
        if after_snapshot.my_carrier_count > before_snapshot.my_carrier_count {
            return true;
        }
        let unknown_steps = Config::BOARD_SIZE + 4;
        after_snapshot.my_best_carrier_steps < before_snapshot.my_best_carrier_steps
            && after_snapshot.my_best_carrier_steps < unknown_steps
    }

    fn enforce_tactical_child_top2(
        scored_states: &mut [(i32, RankedChildState)],
        maximizing: bool,
    ) {
        if scored_states.len() < 3 {
            return;
        }
        let tactical_margin = SMART_MOVE_CLASS_CHILD_SCORE_MARGIN.max(0);
        let top_has_tactical = scored_states
            .iter()
            .take(2)
            .any(|(_, state)| state.classes.is_tactical_priority());
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
                    if !state.classes.is_tactical_priority() {
                        return None;
                    }
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
                });
        let Some(replacement_index) = replacement_index else {
            return;
        };

        let swap_index = 1;
        scored_states.swap(swap_index, replacement_index);
    }

    #[cfg(test)]
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

    #[cfg(test)]
    fn smart_search_best_inputs_internal(
        game: &MonsGame,
        config: SmartSearchConfig,
        use_transposition_table: bool,
    ) -> Vec<Input> {
        let perspective = game.active_color;
        let root_moves = Self::ranked_root_moves(game, perspective, config);
        if root_moves.is_empty() {
            return Vec::new();
        }

        let (root_moves, scout_visited_nodes) = Self::focused_root_candidates(
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
        let mut transposition_table = std::collections::HashMap::new();

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
                use_transposition_table,
            );

            scored_roots.push(RootEvaluation {
                score: candidate_score,
                efficiency: candidate.efficiency,
                inputs: candidate.inputs,
                game: candidate.game,
                wins_immediately: candidate.wins_immediately,
                attacks_opponent_drainer: candidate.attacks_opponent_drainer,
                own_drainer_vulnerable: candidate.own_drainer_vulnerable,
                spirit_development: candidate.spirit_development,
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
        transposition_table: &mut std::collections::HashMap<u64, TranspositionEntry>,
        use_transposition_table: bool,
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
            use_transposition_table,
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
                use_transposition_table,
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

        let scout_depth = if config.depth <= 3 {
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
        let mut scout_transposition_table = std::collections::HashMap::new();
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
                    use_transposition_table,
                )
            } else {
                candidate.heuristic.saturating_add(candidate.efficiency / 2)
            };
            scout_scores[index] = score;
            best_scout_score = best_scout_score.max(score);
            scout_alpha = scout_alpha.max(score);
        }

        let focus_k = config.root_focus_k.max(1).min(root_moves.len());
        let mut ranked_indices = (0..root_moves.len())
            .map(|index| {
                let score = if scout_scores[index] == i32::MIN {
                    root_moves[index]
                        .heuristic
                        .saturating_add(root_moves[index].efficiency / 2)
                } else {
                    scout_scores[index]
                };
                (index, score)
            })
            .collect::<Vec<_>>();
        ranked_indices.sort_by(|a, b| b.1.cmp(&a.1));

        if ranked_indices.len() >= focus_k {
            let best_score = ranked_indices[0].1;
            let kth_score = ranked_indices[focus_k - 1].1;
            if best_score.saturating_sub(kth_score) <= SMART_TWO_PASS_ROOT_NARROW_SPREAD_FALLBACK {
                return (root_moves, 0);
            }
        }

        let mut selected_indices = std::collections::HashSet::new();
        for (index, _) in ranked_indices.iter().take(focus_k) {
            selected_indices.insert(*index);
        }
        for (index, score) in ranked_indices.iter().copied() {
            if score + SMART_TWO_PASS_ROOT_FOCUS_SCORE_MARGIN < best_scout_score {
                continue;
            }
            selected_indices.insert(index);
        }
        for (index, candidate) in root_moves.iter().enumerate() {
            if candidate.attacks_opponent_drainer {
                selected_indices.insert(index);
            }
        }

        if selected_indices.is_empty() {
            return (root_moves, 0);
        }

        let mut focused_with_scores = selected_indices
            .into_iter()
            .map(|index| {
                let score = if scout_scores[index] == i32::MIN {
                    root_moves[index]
                        .heuristic
                        .saturating_add(root_moves[index].efficiency / 2)
                } else {
                    scout_scores[index]
                };
                (index, score)
            })
            .collect::<Vec<_>>();
        focused_with_scores.sort_by(|a, b| b.1.cmp(&a.1));

        let focused_root_moves = focused_with_scores
            .into_iter()
            .map(|(index, _)| root_moves[index].clone())
            .collect::<Vec<_>>();

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
        transposition_table: &mut std::collections::HashMap<u64, TranspositionEntry>,
        extensions_remaining: usize,
        use_transposition_table: bool,
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
        let mut children =
            Self::ranked_child_states(game, perspective, maximizing, preferred_child_hash, config);
        if children.is_empty() {
            return evaluate_preferability_with_weights(game, perspective, config.scoring_weights);
        }

        let mut stopped_by_budget = false;
        let mut best_child_hash = 0u64;
        let value = if maximizing {
            let mut value = i32::MIN;
            for child in children.drain(..) {
                if *visited_nodes >= config.max_visited_nodes {
                    stopped_by_budget = true;
                    break;
                }

                let mut child_depth = depth.saturating_sub(1);
                let mut child_extensions_remaining = extensions_remaining;
                if config.enable_selective_extensions
                    && child.tactical_extension_trigger
                    && child_extensions_remaining > 0
                {
                    child_depth = depth;
                    child_extensions_remaining = child_extensions_remaining.saturating_sub(1);
                } else if config.enable_quiet_reductions
                    && child.quiet_reduction_candidate
                    && depth > 2
                {
                    child_depth = depth.saturating_sub(2);
                }

                *visited_nodes += 1;
                let score = Self::search_score(
                    &child.game,
                    perspective,
                    child_depth,
                    alpha,
                    beta,
                    visited_nodes,
                    config,
                    transposition_table,
                    child_extensions_remaining,
                    use_transposition_table,
                );
                if score > value {
                    value = score;
                    best_child_hash = child.hash;
                }
                alpha = alpha.max(value);
                if alpha >= beta {
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
            for child in children.drain(..) {
                if *visited_nodes >= config.max_visited_nodes {
                    stopped_by_budget = true;
                    break;
                }

                let mut child_depth = depth.saturating_sub(1);
                let mut child_extensions_remaining = extensions_remaining;
                if config.enable_selective_extensions
                    && child.tactical_extension_trigger
                    && child_extensions_remaining > 0
                {
                    child_depth = depth;
                    child_extensions_remaining = child_extensions_remaining.saturating_sub(1);
                } else if config.enable_quiet_reductions
                    && child.quiet_reduction_candidate
                    && depth > 2
                {
                    child_depth = depth.saturating_sub(2);
                }

                *visited_nodes += 1;
                let score = Self::search_score(
                    &child.game,
                    perspective,
                    child_depth,
                    alpha,
                    beta,
                    visited_nodes,
                    config,
                    transposition_table,
                    child_extensions_remaining,
                    use_transposition_table,
                );
                if score < value {
                    value = score;
                    best_child_hash = child.hash;
                }
                beta = beta.min(value);
                if beta <= alpha {
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

            if transposition_table.len() >= SMART_TRANSPOSITION_TABLE_MAX_ENTRIES
                && !transposition_table.contains_key(&state_key)
            {
                transposition_table.clear();
            }
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

        value
    }

    fn ranked_child_states(
        game: &MonsGame,
        perspective: Color,
        maximizing: bool,
        preferred_child_hash: Option<u64>,
        config: SmartSearchConfig,
    ) -> Vec<RankedChildState> {
        let mut scored_states: Vec<(i32, RankedChildState)> = Vec::new();
        let actor_color = game.active_color;
        let own_drainer_vulnerable_before = if config.enable_move_class_coverage {
            Self::is_own_drainer_vulnerable_next_turn(game, actor_color)
        } else {
            false
        };
        for inputs in Self::enumerate_legal_inputs(game, config.node_enum_limit) {
            let needs_events = config.enable_event_ordering_bonus
                || config.enable_move_class_coverage
                || config.enable_selective_extensions;
            let maybe_state = if needs_events {
                Self::apply_inputs_for_search_with_events(game, &inputs)
            } else {
                Self::apply_inputs_for_search(game, &inputs).map(|state| (state, Vec::new()))
            };

            if let Some((simulated_game, events)) = maybe_state {
                let child_hash = Self::search_state_hash(&simulated_game);
                let mut heuristic = Self::score_state(
                    &simulated_game,
                    perspective,
                    0,
                    config.depth,
                    config.scoring_weights,
                );

                let efficiency_delta = if config.enable_root_efficiency {
                    Self::move_efficiency_delta(
                        game,
                        &simulated_game,
                        perspective,
                        &events,
                        false,
                        false,
                        false,
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

                let own_drainer_vulnerable_after = if config.enable_move_class_coverage {
                    Self::is_own_drainer_vulnerable_next_turn(&simulated_game, actor_color)
                } else {
                    false
                };
                let tactical_extension_trigger =
                    Self::events_include_opponent_drainer_fainted(&events, actor_color)
                        || own_drainer_vulnerable_after
                        || events
                            .iter()
                            .any(|event| matches!(event, Event::ManaScored { .. }));
                let quiet_reduction_candidate = !Self::has_material_event(&events)
                    && efficiency_delta <= 0
                    && !tactical_extension_trigger;
                let classes = if config.enable_move_class_coverage {
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

                scored_states.push((
                    heuristic,
                    RankedChildState {
                        game: simulated_game,
                        hash: child_hash,
                        tactical_extension_trigger,
                        quiet_reduction_candidate,
                        classes,
                    },
                ));
            }
        }

        if maximizing {
            scored_states.sort_by(|a, b| b.0.cmp(&a.0));
        } else {
            scored_states.sort_by(|a, b| a.0.cmp(&b.0));
        }

        if config.enable_move_class_coverage && scored_states.len() >= 3 {
            Self::enforce_tactical_child_top2(&mut scored_states, maximizing);
        }

        if scored_states.len() > config.node_branch_limit {
            scored_states.truncate(config.node_branch_limit);
        }

        scored_states.into_iter().map(|(_, state)| state).collect()
    }

    fn enumerate_legal_inputs(game: &MonsGame, max_moves: usize) -> Vec<Vec<Input>> {
        let mut all_inputs = Vec::new();
        let mut partial_inputs = Vec::new();
        let mut simulated_game = game.clone_for_simulation();
        Self::collect_legal_inputs(
            &mut simulated_game,
            &mut partial_inputs,
            &mut all_inputs,
            max_moves,
        );
        all_inputs.sort_by(|a, b| Input::fen_from_array(a).cmp(&Input::fen_from_array(b)));
        all_inputs
    }

    fn collect_legal_inputs(
        game: &mut MonsGame,
        partial_inputs: &mut Vec<Input>,
        all_inputs: &mut Vec<Vec<Input>>,
        max_moves: usize,
    ) {
        if all_inputs.len() >= max_moves || partial_inputs.len() > SMART_MAX_INPUT_CHAIN {
            return;
        }

        match game.process_input(partial_inputs.clone(), true, false) {
            Output::InvalidInput => {}
            Output::Events(_) => all_inputs.push(partial_inputs.clone()),
            Output::LocationsToStartFrom(locations) => {
                for location in locations {
                    if all_inputs.len() >= max_moves {
                        break;
                    }
                    partial_inputs.push(Input::Location(location));
                    Self::collect_legal_inputs(game, partial_inputs, all_inputs, max_moves);
                    partial_inputs.pop();
                }
            }
            Output::NextInputOptions(options) => {
                for option in options {
                    if all_inputs.len() >= max_moves {
                        break;
                    }
                    partial_inputs.push(option.input);
                    Self::collect_legal_inputs(game, partial_inputs, all_inputs, max_moves);
                    partial_inputs.pop();
                }
            }
        }
    }

    fn apply_inputs_for_search(game: &MonsGame, inputs: &[Input]) -> Option<MonsGame> {
        Self::apply_inputs_for_search_with_events(game, inputs)
            .map(|(simulated_game, _)| simulated_game)
    }

    fn apply_inputs_for_search_with_events(
        game: &MonsGame,
        inputs: &[Input],
    ) -> Option<(MonsGame, Vec<Event>)> {
        let mut simulated_game = game.clone_for_simulation();
        match simulated_game.process_input(inputs.to_vec(), false, false) {
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
        let mut seen_moves: std::collections::HashSet<(Location, Location, Color, MonKind)> =
            std::collections::HashSet::new();
        for event in events {
            let Event::MonMove { item, from, to } = event else {
                continue;
            };
            let Some(mon) = item.mon() else {
                continue;
            };
            let reverse = (*to, *from, mon.color, mon.kind);
            if seen_moves.contains(&reverse) {
                return true;
            }
            seen_moves.insert((*from, *to, mon.color, mon.kind));
        }
        false
    }

    fn move_efficiency_delta(
        game: &MonsGame,
        simulated_game: &MonsGame,
        perspective: Color,
        events: &[Event],
        is_root: bool,
        apply_backtrack_penalty: bool,
        apply_root_mana_handoff_guard: bool,
    ) -> i32 {
        let before = Self::move_efficiency_snapshot(game, perspective);
        let after = Self::move_efficiency_snapshot(simulated_game, perspective);
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

        if is_root {
            let root_compensates_handoff = events
                .iter()
                .any(|event| matches!(event, Event::ManaScored { .. }))
                || Self::events_include_opponent_drainer_fainted(events, perspective);
            if apply_root_mana_handoff_guard && !root_compensates_handoff {
                delta -= Self::mana_handoff_penalty(events, perspective);
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
            if apply_backtrack_penalty && Self::has_roundtrip_mon_move(events) {
                delta -= SMART_ROOT_BACKTRACK_PENALTY;
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

    fn mana_handoff_penalty(events: &[Event], perspective: Color) -> i32 {
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
                    * SMART_ROOT_MANA_HANDOFF_PENALTY;
            }
        }

        penalty
    }

    fn move_efficiency_snapshot(game: &MonsGame, perspective: Color) -> MoveEfficiencySnapshot {
        let unknown_steps = Config::BOARD_SIZE + 4;
        let mana_locations = game
            .board
            .items
            .iter()
            .filter_map(|(location, item)| match item {
                Item::Mana { .. } => Some(*location),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut snapshot = MoveEfficiencySnapshot {
            my_best_carrier_steps: unknown_steps,
            opponent_best_carrier_steps: unknown_steps,
            my_best_drainer_to_mana_steps: unknown_steps,
            opponent_best_drainer_to_mana_steps: unknown_steps,
            my_carrier_count: 0,
            opponent_carrier_count: 0,
            my_spirit_on_base: false,
            opponent_spirit_on_base: false,
            my_spirit_action_targets: 0,
            opponent_spirit_action_targets: 0,
        };
        let my_spirit_base = Self::spirit_base_for_color(&game.board, perspective);
        let opponent_spirit_base = Self::spirit_base_for_color(&game.board, perspective.other());

        for (&location, item) in &game.board.items {
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
                        let spirit_targets =
                            Self::spirit_action_target_count(&game.board, location);
                        if mon.color == perspective {
                            snapshot.my_spirit_on_base = location == my_spirit_base;
                            snapshot.my_spirit_action_targets =
                                snapshot.my_spirit_action_targets.max(spirit_targets);
                        } else {
                            snapshot.opponent_spirit_on_base = location == opponent_spirit_base;
                            snapshot.opponent_spirit_action_targets =
                                snapshot.opponent_spirit_action_targets.max(spirit_targets);
                        }
                    }
                    if mon.kind != MonKind::Drainer {
                        continue;
                    }
                    let Some(nearest_mana_steps) =
                        Self::nearest_mana_steps_for_efficiency(location, &mana_locations)
                    else {
                        continue;
                    };

                    if mon.color == perspective {
                        snapshot.my_best_drainer_to_mana_steps = snapshot
                            .my_best_drainer_to_mana_steps
                            .min(nearest_mana_steps);
                    } else {
                        snapshot.opponent_best_drainer_to_mana_steps = snapshot
                            .opponent_best_drainer_to_mana_steps
                            .min(nearest_mana_steps);
                    }
                }
                Item::Mana { .. } | Item::Consumable { .. } => {}
            }
        }

        snapshot
    }

    fn nearest_mana_steps_for_efficiency(
        from: Location,
        mana_locations: &[Location],
    ) -> Option<i32> {
        mana_locations
            .iter()
            .map(|location| from.distance(location) as i32)
            .min()
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

    fn search_state_hash(game: &MonsGame) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut items_mix = 0u64;
        for (location, item) in &game.board.items {
            let mut item_hasher = std::collections::hash_map::DefaultHasher::new();
            location.hash(&mut item_hasher);
            item.hash(&mut item_hasher);
            let entry_hash = item_hasher.finish();
            let rotate =
                ((location.i as u32).wrapping_mul(13) ^ (location.j as u32).wrapping_mul(29)) & 63;
            items_mix ^= entry_hash
                .rotate_left(rotate)
                .wrapping_mul(0x9e3779b185ebca87);
        }

        let mut state_hasher = std::collections::hash_map::DefaultHasher::new();
        items_mix.hash(&mut state_hasher);
        game.white_score.hash(&mut state_hasher);
        game.black_score.hash(&mut state_hasher);
        game.active_color.hash(&mut state_hasher);
        game.actions_used_count.hash(&mut state_hasher);
        game.mana_moves_count.hash(&mut state_hasher);
        game.mons_moves_count.hash(&mut state_hasher);
        game.white_potions_count.hash(&mut state_hasher);
        game.black_potions_count.hash(&mut state_hasher);
        game.turn_number.hash(&mut state_hasher);
        state_hasher.finish()
    }

    #[cfg(target_arch = "wasm32")]
    fn advance_async_search(state: &mut AsyncSmartSearchState) -> bool {
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
            true,
        );

        state.scored_roots.push(RootEvaluation {
            score: candidate_score,
            efficiency: candidate.efficiency,
            inputs: candidate.inputs.clone(),
            game: candidate.game.clone(),
            wins_immediately: candidate.wins_immediately,
            attacks_opponent_drainer: candidate.attacks_opponent_drainer,
            own_drainer_vulnerable: candidate.own_drainer_vulnerable,
            spirit_development: candidate.spirit_development,
            classes: candidate.classes,
        });

        if candidate_score > state.alpha {
            state.alpha = candidate_score;
        }

        state.next_index += 1;
        state.next_index >= state.root_moves.len()
            || state.visited_nodes >= state.config.max_visited_nodes
    }

    #[cfg(target_arch = "wasm32")]
    fn finalize_async_search(state: &mut AsyncSmartSearchState) -> OutputModel {
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

    fn is_location_guarded_by_angel(board: &Board, color: Color, location: Location) -> bool {
        board
            .find_awake_angel(color)
            .map_or(false, |angel_location| {
                angel_location.distance(&location) == 1
            })
    }

    fn is_own_drainer_immediately_vulnerable(game: &MonsGame, perspective: Color) -> bool {
        let mut own_drainer_location = None;
        for (&location, item) in &game.board.items {
            let Some(mon) = item.mon() else {
                continue;
            };
            if mon.color != perspective || mon.kind != MonKind::Drainer {
                continue;
            }
            if mon.is_fainted() {
                return true;
            }
            own_drainer_location = Some(location);
            break;
        }

        let Some(drainer_location) = own_drainer_location else {
            return false;
        };

        let opponent = perspective.other();
        if game.active_color != opponent {
            return false;
        }

        let drainer_action_protected =
            Self::is_location_guarded_by_angel(&game.board, perspective, drainer_location);
        let opponent_can_use_action = game.player_can_use_action();

        for (&threat_location, item) in &game.board.items {
            match item {
                Item::Mon { mon } if mon.color == opponent && !mon.is_fainted() => {
                    if opponent_can_use_action
                        && !drainer_action_protected
                        && !matches!(game.board.square(threat_location), Square::MonBase { .. })
                    {
                        if mon.kind == MonKind::Mystic
                            && (threat_location.i - drainer_location.i).abs() == 2
                            && (threat_location.j - drainer_location.j).abs() == 2
                        {
                            return true;
                        }

                        if mon.kind == MonKind::Demon {
                            let di = (threat_location.i - drainer_location.i).abs();
                            let dj = (threat_location.j - drainer_location.j).abs();
                            if (di == 2 && dj == 0) || (di == 0 && dj == 2) {
                                let middle = threat_location.location_between(&drainer_location);
                                if game.board.item(middle).is_none()
                                    && !matches!(
                                        game.board.square(middle),
                                        Square::SupermanaBase | Square::MonBase { .. }
                                    )
                                {
                                    return true;
                                }
                            }
                        }
                    }
                }
                Item::MonWithConsumable { mon, consumable }
                    if mon.color == opponent
                        && !mon.is_fainted()
                        && *consumable == Consumable::Bomb
                        && threat_location.distance(&drainer_location) <= 3 =>
                {
                    return true;
                }
                Item::MonWithConsumable { .. }
                | Item::MonWithMana { .. }
                | Item::Mana { .. }
                | Item::Consumable { .. } => {}
                Item::Mon { .. } => {}
            }
        }

        false
    }

    fn is_own_drainer_vulnerable_next_turn(game: &MonsGame, perspective: Color) -> bool {
        let opponent = perspective.other();
        let mut probe = game.clone_for_simulation();
        probe.active_color = opponent;
        probe.actions_used_count = 0;
        probe.mana_moves_count = 0;
        probe.mons_moves_count = 0;
        Self::is_own_drainer_immediately_vulnerable(&probe, perspective)
    }

    fn should_prefer_spirit_development(game: &MonsGame, perspective: Color) -> bool {
        !game.is_first_turn()
            && game.player_can_move_mon()
            && Self::has_awake_spirit_on_base(&game.board, perspective)
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

        if config.enable_forced_drainer_attack
            && candidate_indices
                .iter()
                .any(|index| scored_roots[*index].attacks_opponent_drainer)
        {
            candidate_indices.retain(|index| scored_roots[*index].attacks_opponent_drainer);
            forced_attack_applied = true;
        }

        if config.enable_root_drainer_safety_prefilter && !forced_attack_applied {
            let best_score = candidate_indices
                .iter()
                .map(|index| scored_roots[*index].score)
                .max()
                .unwrap_or(i32::MIN);
            let margin = SMART_ROOT_DRAINER_SAFETY_SCORE_MARGIN.max(0);
            let safer_indices = candidate_indices
                .iter()
                .copied()
                .filter(|index| {
                    let root = &scored_roots[*index];
                    !root.own_drainer_vulnerable && root.score + margin >= best_score
                })
                .collect::<Vec<_>>();
            if !safer_indices.is_empty() {
                candidate_indices = safer_indices;
            }
        }

        if config.enable_root_spirit_development_pref
            && Self::should_prefer_spirit_development(game, perspective)
            && candidate_indices
                .iter()
                .any(|index| scored_roots[*index].spirit_development)
        {
            let best_score = candidate_indices
                .iter()
                .map(|index| scored_roots[*index].score)
                .max()
                .unwrap_or(i32::MIN);
            let margin = SMART_ROOT_SPIRIT_DEVELOPMENT_SCORE_MARGIN.max(0);
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

        candidate_indices
    }

    fn best_scored_root_index(
        scored_roots: &[RootEvaluation],
        candidate_indices: &[usize],
    ) -> usize {
        let mut best_index = candidate_indices.first().copied().unwrap_or(0);
        let mut best_score = i32::MIN;
        for index in candidate_indices.iter().copied() {
            let score = scored_roots[index].score;
            if score > best_score {
                best_score = score;
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
        let score_margin = SMART_ROOT_EFFICIENCY_SCORE_MARGIN.max(0);

        let mut best_index =
            Self::best_scored_root_index(scored_roots, candidate_indices.as_slice());
        let mut best_efficiency = i32::MIN;
        let mut best_shortlisted_score = i32::MIN;
        let prefer_spirit_development = config.enable_root_spirit_development_pref
            && Self::should_prefer_spirit_development(game, perspective);
        let mut best_spirit_development = scored_roots[best_index].spirit_development;

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
            if spirit_better
                || (equal_spirit_preference
                    && (evaluation.efficiency > best_efficiency
                        || (evaluation.efficiency == best_efficiency
                            && evaluation.score > best_shortlisted_score)))
            {
                best_index = index;
                best_efficiency = evaluation.efficiency;
                best_shortlisted_score = evaluation.score;
                best_spirit_development = evaluation.spirit_development;
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
        let score_margin = config.root_reply_risk_score_margin.max(0);
        let mut shortlist = candidate_indices
            .iter()
            .copied()
            .filter(|index| scored_roots[*index].score + score_margin >= best_score)
            .collect::<Vec<_>>();
        shortlist.sort_by(|a, b| scored_roots[*b].score.cmp(&scored_roots[*a].score));
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

        let replies = Self::enumerate_legal_inputs(state_after_move, reply_limit.max(1));
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
            let Some(after_reply) = Self::apply_inputs_for_search(state_after_move, &reply) else {
                continue;
            };
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
    ) -> bool {
        let candidate = &scored_roots[candidate_index];
        let incumbent = &scored_roots[incumbent_index];

        if candidate.wins_immediately != incumbent.wins_immediately {
            return candidate.wins_immediately;
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
        if candidate.attacks_opponent_drainer != incumbent.attacks_opponent_drainer {
            return candidate.attacks_opponent_drainer;
        }
        if candidate.spirit_development != incumbent.spirit_development {
            return candidate.spirit_development;
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
        for class in [
            MoveClass::CarrierProgress,
            MoveClass::Material,
            MoveClass::Quiet,
        ] {
            let candidate_has = candidate.classes.has(class);
            let incumbent_has = incumbent.classes.has(class);
            if candidate_has != incumbent_has {
                return candidate_has;
            }
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
        shortlist.sort_by(|a, b| scored_roots[*b].score.cmp(&scored_roots[*a].score));
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

        let mut best_index = best_scored_index;
        let mut best_snapshot = Self::normal_root_safety_snapshot(
            &scored_roots[best_scored_index].game,
            perspective,
            my_score_before,
            config.scoring_weights,
            reply_limit,
        );

        for index in shortlist_indices.iter().copied().skip(1) {
            let evaluation = &scored_roots[index];
            let snapshot = Self::normal_root_safety_snapshot(
                &evaluation.game,
                perspective,
                my_score_before,
                config.scoring_weights,
                reply_limit,
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
        let replies = Self::enumerate_legal_inputs(state_after_move, reply_limit.max(1));
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
            let Some(after_reply) = Self::apply_inputs_for_search(state_after_move, &reply) else {
                continue;
            };
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
                    && scored_roots[index].score > scored_roots[best_index].score)
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

        let replies = Self::enumerate_legal_inputs(state_after_move, reply_limit.max(1));
        if replies.is_empty() {
            return SMART_TERMINAL_SCORE / 4;
        }

        let mut worst = i32::MAX;
        for reply in replies {
            let Some(after_reply) = Self::apply_inputs_for_search(state_after_move, &reply) else {
                continue;
            };
            let score = match after_reply.winner_color() {
                Some(winner) if winner == perspective => SMART_TERMINAL_SCORE / 2,
                Some(_) => -SMART_TERMINAL_SCORE / 2,
                None => {
                    let mut visited_nodes = 0usize;
                    let mut transposition_table =
                        std::collections::HashMap::<u64, TranspositionEntry>::new();
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
                        true,
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
#[path = "mons_game_model_automove_experiments.rs"]
mod smart_automove_pool_tests;

#[cfg(test)]
mod opening_book_tests {
    use super::*;
    use std::collections::HashMap;

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
                &mut continuation_budget,
                &mut std::collections::HashSet::new(),
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
            Color::White
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
            Color::White
        ));
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
