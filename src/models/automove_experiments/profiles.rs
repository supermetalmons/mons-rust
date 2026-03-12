use super::harness::env_usize;
use super::*;

const DEFAULT_PROMOTION_BASELINE_PROFILE: &str = "runtime_release_safe_pre_exact";
const PROFILE_RUNTIME_EFF_NON_EXACT_V1: &str = "runtime_eff_non_exact_v1";
const PROFILE_RUNTIME_EFF_NON_EXACT_V2: &str = "runtime_eff_non_exact_v2";
const PROFILE_RUNTIME_EFF_EXACT_LITE_V1: &str = "runtime_eff_exact_lite_v1";
const PROFILE_RUNTIME_ATTACKER_PROXIMITY_V1: &str = "runtime_attacker_proximity_v1";
const PROFILE_RUNTIME_PRO_DEEPER_EXTENSIONS_V1: &str = "runtime_pro_deeper_extensions_v1";
const PROFILE_RUNTIME_PRO_MORE_EXTENSIONS_V1: &str = "runtime_pro_more_extensions_v1";
const PROFILE_RUNTIME_NORMAL_NO_SAFETY_RERANK_V1: &str = "runtime_normal_no_safety_rerank_v1";
const PROFILE_RUNTIME_NORMAL_NO_TWO_PASS_V1: &str = "runtime_normal_no_two_pass_v1";
const PROFILE_RUNTIME_NORMAL_NO_EXTENSIONS_V1: &str = "runtime_normal_no_extensions_v1";
const PROFILE_RUNTIME_NORMAL_WIDER_REPLY_V1: &str = "runtime_normal_wider_reply_v1";
const PROFILE_RUNTIME_PRO_NO_QUIET_REDUCTIONS_V1: &str = "runtime_pro_no_quiet_reductions_v1";
const PROFILE_RUNTIME_PRO_NO_FUTILITY_V1: &str = "runtime_pro_no_futility_v1";
const PROFILE_RUNTIME_PRO_KILLER_ORDERING_V1: &str = "runtime_pro_killer_ordering_v1";
const PROFILE_RUNTIME_PRO_SEARCH_COMBO_V1: &str = "runtime_pro_search_combo_v1";
const PROFILE_RUNTIME_FAST_NO_QUIET_REDUCTIONS_V1: &str = "runtime_fast_no_quiet_reductions_v1";
const PROFILE_RUNTIME_FAST_TWO_PASS_V1: &str = "runtime_fast_spirit_deploy_v1";
const PROFILE_RUNTIME_NORMAL_ITERATIVE_DEEPENING_V1: &str = "runtime_normal_iterative_deepening_v1";
const PROFILE_RUNTIME_NORMAL_HISTORY_HEURISTIC_V1: &str = "runtime_normal_history_heuristic_v1";
const PROFILE_RUNTIME_PRO_HISTORY_HEURISTIC_V1: &str = "runtime_pro_history_heuristic_v1";
const PROFILE_RUNTIME_NORMAL_QUIESCENCE_V1: &str = "runtime_normal_quiescence_v1";
const PROFILE_RUNTIME_PRO_QUIESCENCE_V1: &str = "runtime_pro_quiescence_v1";
const PROFILE_RUNTIME_PRO_QUIESCENCE_V2: &str = "runtime_pro_quiescence_v2";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ExactLiteBudgets {
    pub root_call_budget: usize,
    pub static_call_budget: usize,
}

#[derive(Clone, Copy)]
struct AutomoveProfile {
    id: &'static str,
    selector: AutomoveSelector,
}

const RETAINED_PROFILES: [AutomoveProfile; 15] = [
    AutomoveProfile {
        id: "base",
        selector: model_base_profile,
    },
    AutomoveProfile {
        id: "runtime_current",
        selector: model_runtime_current_profile,
    },
    AutomoveProfile {
        id: "runtime_release_safe_pre_exact",
        selector: model_runtime_release_safe_pre_exact,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_EFF_NON_EXACT_V1,
        selector: model_runtime_eff_non_exact_v1,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_EFF_NON_EXACT_V2,
        selector: model_runtime_eff_non_exact_v2,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_EFF_EXACT_LITE_V1,
        selector: model_runtime_eff_exact_lite_v1,
    },
    AutomoveProfile {
        id: "swift_2024_eval_reference",
        selector: model_swift_2024_eval_reference,
    },
    AutomoveProfile {
        id: "swift_2024_style_reference",
        selector: model_swift_2024_style_reference,
    },
    AutomoveProfile {
        id: "runtime_pre_fast_root_quality_v1_normal_conversion_v3",
        selector: model_runtime_pre_fast_root_quality_v1_normal_conversion_v3,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_ATTACKER_PROXIMITY_V1,
        selector: model_runtime_attacker_proximity_v1,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_NORMAL_HISTORY_HEURISTIC_V1,
        selector: model_runtime_normal_history_heuristic_v1,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_PRO_HISTORY_HEURISTIC_V1,
        selector: model_runtime_pro_history_heuristic_v1,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_NORMAL_QUIESCENCE_V1,
        selector: model_runtime_normal_quiescence_v1,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_PRO_QUIESCENCE_V1,
        selector: model_runtime_pro_quiescence_v1,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_PRO_QUIESCENCE_V2,
        selector: model_runtime_pro_quiescence_v2,
    },
];

const CURATED_POOL_PROFILE_IDS: [&str; CURATED_POOL_SIZE] = [
    "runtime_current",
    "runtime_release_safe_pre_exact",
    "swift_2024_eval_reference",
    "swift_2024_style_reference",
    "runtime_pre_fast_root_quality_v1_normal_conversion_v3",
];

pub(super) const CANDIDATE_MODEL: AutomoveModel = AutomoveModel {
    id: "candidate",
    select_inputs: candidate_model,
};

fn runtime_selector_inputs(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let inputs = MonsGameModel::smart_search_best_inputs(game, config);
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

pub(super) fn model_current_best(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    runtime_selector_inputs(game, config)
}

pub(super) fn model_base_profile(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

pub(super) fn model_runtime_current_profile(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    model_current_best(game, config)
}

pub(super) fn model_runtime_release_safe_pre_exact(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    runtime_selector_inputs(game, configure_runtime_release_safe_pre_exact(config))
}

fn configure_runtime_release_safe_pre_exact(config: SmartSearchConfig) -> SmartSearchConfig {
    MonsGameModel::with_pre_exact_runtime_policy(config)
}

fn configure_runtime_eff_non_exact_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);

    if runtime.depth < 3 {
        runtime.root_reply_risk_score_margin = 130;
        runtime.root_reply_risk_shortlist_max = 4;
        runtime.root_reply_risk_reply_limit = 9;
        runtime.root_reply_risk_node_share_bp = 650;
        runtime.root_drainer_safety_score_margin = SMART_ROOT_DRAINER_SAFETY_SCORE_MARGIN;
        runtime.root_efficiency_score_margin = 1_700;
        runtime.enable_interview_soft_root_priors = true;
        runtime.enable_interview_deterministic_tiebreak = false;
        runtime.scoring_weights = &RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF;
    } else {
        runtime.enable_interview_deterministic_tiebreak = false;
    }

    MonsGameModel::with_pre_exact_runtime_policy(runtime)
}

fn configure_runtime_eff_non_exact_v2(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = configure_runtime_eff_non_exact_v1(game, config);

    if runtime.depth < 3 {
        runtime.root_reply_risk_score_margin = 130;
        runtime.root_reply_risk_shortlist_max = 5;
        runtime.root_reply_risk_reply_limit = 10;
        runtime.root_reply_risk_node_share_bp = 900;
        runtime.root_drainer_safety_score_margin =
            runtime.root_drainer_safety_score_margin.max(3_000);
        runtime.root_efficiency_score_margin = runtime.root_efficiency_score_margin.min(1_500);
        runtime.enable_interview_soft_root_priors = true;
        runtime.enable_interview_deterministic_tiebreak = false;
    } else {
        runtime.root_reply_risk_score_margin = runtime.root_reply_risk_score_margin.max(155);
        runtime.root_reply_risk_shortlist_max = runtime.root_reply_risk_shortlist_max.clamp(7, 8);
        runtime.root_reply_risk_reply_limit = runtime.root_reply_risk_reply_limit.clamp(18, 22);
        runtime.root_reply_risk_node_share_bp =
            runtime.root_reply_risk_node_share_bp.clamp(1_500, 1_900);
        runtime.root_drainer_safety_score_margin =
            runtime.root_drainer_safety_score_margin.max(4_500);
        runtime.root_efficiency_score_margin = runtime.root_efficiency_score_margin.min(1_300);
        runtime.enable_interview_soft_root_priors = true;
        runtime.enable_interview_deterministic_tiebreak = false;
        runtime.scoring_weights =
            MonsGameModel::runtime_phase_adaptive_attacker_proximity_scoring_weights(
                game,
                runtime.depth,
            );
    }
    MonsGameModel::with_pre_exact_runtime_policy(runtime)
}

fn configure_runtime_eff_exact_lite_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = configure_runtime_eff_non_exact_v1(game, config);
    runtime.enable_root_exact_tactics = false;
    runtime.enable_child_exact_tactics = false;
    runtime.enable_static_exact_evaluation = false;
    runtime.enable_exact_lite_progress_checks = true;
    runtime.enable_exact_lite_spirit_window_checks = true;
    if runtime.depth < 3 {
        runtime.exact_lite_root_call_budget = 0;
        runtime.exact_lite_static_call_budget = 0;
    } else if runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.exact_lite_root_call_budget = 1;
        runtime.exact_lite_static_call_budget = 1;
    } else {
        runtime.exact_lite_root_call_budget = 2;
        runtime.exact_lite_static_call_budget = 1;
    }
    runtime
}

pub(super) fn model_runtime_eff_non_exact_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, configure_runtime_eff_non_exact_v1(game, config))
}

pub(super) fn model_runtime_eff_non_exact_v2(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, configure_runtime_eff_non_exact_v2(game, config))
}

pub(super) fn model_runtime_eff_exact_lite_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, configure_runtime_eff_exact_lite_v1(game, config))
}

fn configure_runtime_attacker_proximity_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= 3 {
        runtime.scoring_weights =
            MonsGameModel::runtime_phase_adaptive_attacker_proximity_scoring_weights(
                game,
                runtime.depth,
            );
    }
    runtime
}

pub(super) fn model_runtime_attacker_proximity_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        game,
        configure_runtime_attacker_proximity_v1(game, config),
    )
}

fn configure_runtime_pro_wider_roots_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.root_focus_k = 4;
        runtime.root_focus_budget_share_bp = 6_500;
    }
    runtime
}

pub(super) fn model_runtime_pro_wider_roots_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_pro_wider_roots_v1(config),
    )
}

fn configure_runtime_normal_no_prepass_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= 3 && runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_forced_tactical_prepass = false;
    }
    runtime
}

pub(super) fn model_runtime_normal_no_prepass_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_normal_no_prepass_v1(config),
    )
}

fn configure_runtime_pro_deeper_extensions_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_selective_extensions = false;
    }
    runtime
}

pub(super) fn model_runtime_pro_deeper_extensions_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_pro_deeper_extensions_v1(config),
    )
}


fn configure_runtime_pro_more_extensions_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.max_extensions_per_path = 2;
        runtime.selective_extension_node_share_bp = 2_500;
    }
    runtime
}

pub(super) fn model_runtime_pro_more_extensions_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_pro_more_extensions_v1(config),
    )
}

fn configure_runtime_pro_no_quiet_reductions_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_selective_extensions = false;
        runtime.enable_quiet_reductions = false;
    }
    runtime
}

pub(super) fn model_runtime_pro_no_quiet_reductions_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_pro_no_quiet_reductions_v1(config),
    )
}

fn configure_runtime_pro_no_futility_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_futility_pruning = false;
    }
    runtime
}

pub(super) fn model_runtime_pro_no_futility_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_pro_no_futility_v1(config),
    )
}

fn configure_runtime_pro_killer_ordering_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_killer_move_ordering = true;
    }
    runtime
}

pub(super) fn model_runtime_pro_killer_ordering_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_pro_killer_ordering_v1(config),
    )
}

fn configure_runtime_pro_search_combo_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_killer_move_ordering = true;
        runtime.enable_futility_pruning = false;
        runtime.enable_pvs = true;
    }
    runtime
}

pub(super) fn model_runtime_pro_search_combo_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_pro_search_combo_v1(config),
    )
}

fn configure_runtime_fast_no_quiet_reductions_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth < 3 {
        runtime.enable_quiet_reductions = false;
    }
    runtime
}

pub(super) fn model_runtime_fast_no_quiet_reductions_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_fast_no_quiet_reductions_v1(config),
    )
}

fn configure_runtime_fast_spirit_deploy_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth < 3 {
        runtime.enable_interview_hard_spirit_deploy = true;
    }
    runtime
}

pub(super) fn model_runtime_fast_spirit_deploy_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_fast_spirit_deploy_v1(config),
    )
}

fn configure_runtime_normal_iterative_deepening_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= 3 && runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_root_aspiration = true;
    }
    runtime
}

pub(super) fn model_runtime_normal_iterative_deepening_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_normal_iterative_deepening_v1(config),
    )
}

fn configure_runtime_normal_wider_reply_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= 3 && runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.root_reply_risk_score_margin = 165;
        runtime.root_reply_risk_shortlist_max = 9;
        runtime.root_reply_risk_reply_limit = 22;
        runtime.root_reply_risk_node_share_bp = 1_800;
    }
    runtime
}

pub(super) fn model_runtime_normal_wider_reply_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_normal_wider_reply_v1(config),
    )
}

fn configure_runtime_normal_no_safety_rerank_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= 3 && runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_normal_root_safety_rerank = false;
        runtime.enable_normal_root_safety_deep_floor = false;
    }
    runtime
}

pub(super) fn model_runtime_normal_no_safety_rerank_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_normal_no_safety_rerank_v1(config),
    )
}

fn configure_runtime_normal_no_two_pass_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= 3 && runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_two_pass_root_allocation = false;
        runtime.enable_two_pass_volatility_focus = false;
    }
    runtime
}

pub(super) fn model_runtime_normal_no_two_pass_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_normal_no_two_pass_v1(config),
    )
}

fn configure_runtime_normal_no_extensions_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= 3 && runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_selective_extensions = false;
    }
    runtime
}

pub(super) fn model_runtime_normal_no_extensions_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_normal_no_extensions_v1(config),
    )
}

fn configure_runtime_pre_fast_root_quality_v1_normal_conversion_v3(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    if runtime.depth < 3 {
        runtime.root_reply_risk_score_margin = SMART_ROOT_REPLY_RISK_SCORE_MARGIN;
        runtime.root_reply_risk_shortlist_max = SMART_ROOT_REPLY_RISK_SHORTLIST_FAST;
        runtime.root_reply_risk_reply_limit = SMART_ROOT_REPLY_RISK_REPLY_LIMIT_FAST;
        runtime.root_reply_risk_node_share_bp = SMART_ROOT_REPLY_RISK_NODE_SHARE_BP_FAST;
        runtime.root_mana_handoff_penalty = 260;
        runtime.root_backtrack_penalty = 180;
        runtime.root_efficiency_score_margin = 1_900;
        runtime.root_anti_help_score_margin = 220;
    } else {
        runtime.root_reply_risk_score_margin = 170;
        runtime.root_reply_risk_shortlist_max = 6;
        runtime.root_reply_risk_reply_limit = 14;
        runtime.root_reply_risk_node_share_bp = 1_150;
        runtime.root_drainer_safety_score_margin = 4_000;
        runtime.selective_extension_node_share_bp = SMART_SELECTIVE_EXTENSION_NODE_SHARE_BP_NORMAL;
        runtime.root_efficiency_score_margin = 1_400;
    }
    runtime
}

pub(super) fn profile_runtime_config_for_name(
    profile_name: &str,
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Option<SmartSearchConfig> {
    let resolved = match profile_name {
        "base" | "runtime_current" => config,
        "runtime_release_safe_pre_exact" => configure_runtime_release_safe_pre_exact(config),
        PROFILE_RUNTIME_EFF_NON_EXACT_V1 => configure_runtime_eff_non_exact_v1(game, config),
        PROFILE_RUNTIME_EFF_NON_EXACT_V2 => configure_runtime_eff_non_exact_v2(game, config),
        PROFILE_RUNTIME_EFF_EXACT_LITE_V1 => configure_runtime_eff_exact_lite_v1(game, config),
        PROFILE_RUNTIME_ATTACKER_PROXIMITY_V1 => {
            configure_runtime_attacker_proximity_v1(game, config)
        }
        PROFILE_RUNTIME_PRO_DEEPER_EXTENSIONS_V1 => {
            configure_runtime_pro_deeper_extensions_v1(config)
        }
        PROFILE_RUNTIME_PRO_MORE_EXTENSIONS_V1 => {
            configure_runtime_pro_more_extensions_v1(config)
        }
        PROFILE_RUNTIME_NORMAL_NO_SAFETY_RERANK_V1 => {
            configure_runtime_normal_no_safety_rerank_v1(config)
        }
        PROFILE_RUNTIME_NORMAL_NO_TWO_PASS_V1 => {
            configure_runtime_normal_no_two_pass_v1(config)
        }
        PROFILE_RUNTIME_NORMAL_WIDER_REPLY_V1 => {
            configure_runtime_normal_wider_reply_v1(config)
        }
        PROFILE_RUNTIME_PRO_NO_QUIET_REDUCTIONS_V1 => {
            configure_runtime_pro_no_quiet_reductions_v1(config)
        }
        PROFILE_RUNTIME_PRO_NO_FUTILITY_V1 => {
            configure_runtime_pro_no_futility_v1(config)
        }
        PROFILE_RUNTIME_PRO_KILLER_ORDERING_V1 => {
            configure_runtime_pro_killer_ordering_v1(config)
        }
        PROFILE_RUNTIME_PRO_SEARCH_COMBO_V1 => {
            configure_runtime_pro_search_combo_v1(config)
        }
        PROFILE_RUNTIME_FAST_NO_QUIET_REDUCTIONS_V1 => {
            configure_runtime_fast_no_quiet_reductions_v1(config)
        }
        PROFILE_RUNTIME_FAST_TWO_PASS_V1 => {
            configure_runtime_fast_spirit_deploy_v1(config)
        }
        PROFILE_RUNTIME_NORMAL_ITERATIVE_DEEPENING_V1 => {
            configure_runtime_normal_iterative_deepening_v1(config)
        }
        "runtime_pre_fast_root_quality_v1_normal_conversion_v3" => {
            configure_runtime_pre_fast_root_quality_v1_normal_conversion_v3(game, config)
        }
        "swift_2024_eval_reference" => {
            let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
            runtime.scoring_weights = &SWIFT_2024_REFERENCE_SCORING_WEIGHTS;
            runtime
        }
        "swift_2024_style_reference" => {
            let mut swift_style =
                SmartSearchConfig::from_budget(3, LEGACY_NORMAL_MAX_VISITED_NODES).for_runtime();
            swift_style.scoring_weights = &SWIFT_2024_REFERENCE_SCORING_WEIGHTS;
            swift_style.enable_root_efficiency = false;
            swift_style.enable_event_ordering_bonus = false;
            swift_style.enable_backtrack_penalty = false;
            swift_style.enable_tt_best_child_ordering = false;
            swift_style.enable_two_pass_root_allocation = false;
            swift_style.enable_selective_extensions = false;
            swift_style.enable_quiet_reductions = false;
            swift_style.enable_root_mana_handoff_guard = false;
            swift_style.enable_forced_tactical_prepass = false;
            swift_style.enable_root_drainer_safety_prefilter = false;
            swift_style.enable_root_reply_risk_guard = false;
            swift_style.enable_move_class_coverage = false;
            swift_style.enable_child_move_class_coverage = false;
            swift_style.enable_strict_tactical_class_coverage = false;
            swift_style.enable_strict_anti_help_filter = false;
            swift_style.enable_interview_hard_spirit_deploy = false;
            swift_style.enable_interview_soft_root_priors = false;
            swift_style.enable_interview_deterministic_tiebreak = false;
            swift_style.enable_mana_start_mix_with_potion_actions = false;
            swift_style.enable_potion_progress_compensation = false;
            swift_style.enable_enhanced_drainer_vulnerability = false;
            swift_style.enable_supermana_prepass_exception = false;
            swift_style.enable_opponent_mana_prepass_exception = false;
            swift_style.enable_walk_threat_prefilter = false;
            swift_style.enable_killer_move_ordering = false;
            swift_style.enable_tt_depth_preferred_replacement = false;
            swift_style.enable_pvs = false;
            swift_style
        }
        _ => return None,
    };
    Some(resolved)
}

pub(super) fn profile_exact_lite_budgets(
    profile_name: &str,
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Option<ExactLiteBudgets> {
    let runtime = match profile_name {
        PROFILE_RUNTIME_EFF_EXACT_LITE_V1 => configure_runtime_eff_exact_lite_v1(game, config),
        _ => return None,
    };
    if !(runtime.enable_exact_lite_progress_checks
        || runtime.enable_exact_lite_spirit_window_checks)
    {
        return None;
    }
    Some(ExactLiteBudgets {
        root_call_budget: runtime.exact_lite_root_call_budget,
        static_call_budget: runtime.exact_lite_static_call_budget,
    })
}

fn configure_runtime_normal_history_heuristic_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= 3 && runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_history_heuristic = true;
    }
    runtime
}

pub(super) fn model_runtime_normal_history_heuristic_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_normal_history_heuristic_v1(config),
    )
}

fn configure_runtime_pro_history_heuristic_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_history_heuristic = true;
    }
    runtime
}

pub(super) fn model_runtime_pro_history_heuristic_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_pro_history_heuristic_v1(config),
    )
}

fn configure_runtime_normal_quiescence_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= 3 && runtime.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_quiescence_search = true;
        runtime.quiescence_node_budget = 30;
    }
    runtime
}

pub(super) fn model_runtime_normal_quiescence_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_normal_quiescence_v1(config),
    )
}

fn configure_runtime_pro_quiescence_v1(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_quiescence_search = true;
        runtime.quiescence_node_budget = 200;
    }
    runtime
}

pub(super) fn model_runtime_pro_quiescence_v1(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_pro_quiescence_v1(config),
    )
}

fn configure_runtime_pro_quiescence_v2(
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    let mut runtime = MonsGameModel::with_pre_exact_runtime_policy(config);
    if runtime.depth >= SMART_AUTOMOVE_PRO_DEPTH as usize {
        runtime.enable_quiescence_search = true;
        runtime.quiescence_node_budget = 30;
    }
    runtime
}

pub(super) fn model_runtime_pro_quiescence_v2(
    _game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        _game,
        configure_runtime_pro_quiescence_v2(config),
    )
}

pub(super) fn candidate_model(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let selector =
        profile_selector_from_name(candidate_profile().as_str()).unwrap_or(model_base_profile);
    selector(game, config)
}

pub(super) fn model_first_legal_automove(
    game: &MonsGame,
    _config: SmartSearchConfig,
) -> Vec<Input> {
    let mut simulated = game.clone_for_simulation();
    let automove_start_options = Some(SuggestedStartInputOptions::for_automove());
    let mut inputs = Vec::new();
    let mut output =
        simulated.process_input_with_start_options(vec![], false, false, automove_start_options);

    loop {
        match output {
            Output::InvalidInput => return Vec::new(),
            Output::LocationsToStartFrom(locations) => {
                let Some(location) = locations.first().copied() else {
                    return Vec::new();
                };
                inputs.push(Input::Location(location));
                output = simulated.process_input_with_start_options(
                    inputs.clone(),
                    false,
                    false,
                    automove_start_options,
                );
            }
            Output::NextInputOptions(options) => {
                let Some(next_input) = options.first() else {
                    return Vec::new();
                };
                inputs.push(next_input.input);
                output = simulated.process_input_with_start_options(
                    inputs.clone(),
                    false,
                    false,
                    automove_start_options,
                );
            }
            Output::Events(_) => return inputs,
        }
    }
}

pub(super) fn model_runtime_pre_fast_root_quality_v1_normal_conversion_v3(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(
        game,
        configure_runtime_pre_fast_root_quality_v1_normal_conversion_v3(game, config),
    )
}


pub(super) fn model_swift_2024_eval_reference(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights = &SWIFT_2024_REFERENCE_SCORING_WEIGHTS;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

pub(super) fn model_swift_2024_style_reference(
    game: &MonsGame,
    _config: SmartSearchConfig,
) -> Vec<Input> {
    let mut swift_style =
        SmartSearchConfig::from_budget(3, LEGACY_NORMAL_MAX_VISITED_NODES).for_runtime();
    swift_style.scoring_weights = &SWIFT_2024_REFERENCE_SCORING_WEIGHTS;
    swift_style.enable_root_efficiency = false;
    swift_style.enable_event_ordering_bonus = false;
    swift_style.enable_backtrack_penalty = false;
    swift_style.enable_tt_best_child_ordering = false;
    swift_style.enable_two_pass_root_allocation = false;
    swift_style.enable_selective_extensions = false;
    swift_style.enable_quiet_reductions = false;
    swift_style.enable_root_mana_handoff_guard = false;
    swift_style.enable_forced_tactical_prepass = false;
    swift_style.enable_root_drainer_safety_prefilter = false;
    swift_style.enable_root_reply_risk_guard = false;
    swift_style.enable_move_class_coverage = false;
    swift_style.enable_child_move_class_coverage = false;
    swift_style.enable_strict_tactical_class_coverage = false;
    swift_style.enable_strict_anti_help_filter = false;
    swift_style.enable_interview_hard_spirit_deploy = false;
    swift_style.enable_interview_soft_root_priors = false;
    swift_style.enable_interview_deterministic_tiebreak = false;
    swift_style.enable_mana_start_mix_with_potion_actions = false;
    swift_style.enable_potion_progress_compensation = false;
    swift_style.enable_enhanced_drainer_vulnerability = false;
    swift_style.enable_supermana_prepass_exception = false;
    swift_style.enable_opponent_mana_prepass_exception = false;
    swift_style.enable_walk_threat_prefilter = false;
    swift_style.enable_killer_move_ordering = false;
    swift_style.enable_tt_depth_preferred_replacement = false;
    swift_style.enable_pvs = false;
    MonsGameModel::smart_search_best_inputs_legacy_no_transposition(game, swift_style)
}

fn retained_profiles() -> &'static [AutomoveProfile] {
    &RETAINED_PROFILES
}

pub(super) fn retained_profile_ids() -> Vec<&'static str> {
    retained_profiles()
        .iter()
        .map(|profile| profile.id)
        .collect()
}

pub(super) fn curated_pool_profile_ids() -> &'static [&'static str] {
    &CURATED_POOL_PROFILE_IDS
}

pub(super) fn profile_selector_from_name(profile_name: &str) -> Option<AutomoveSelector> {
    retained_profiles()
        .iter()
        .find(|profile| profile.id == profile_name)
        .map(|profile| profile.selector)
}

pub(super) fn selected_pool_models() -> Vec<AutomoveModel> {
    let requested = env_usize("SMART_POOL_OPPONENTS").unwrap_or(CURATED_POOL_PROFILE_IDS.len());
    let limit = requested.clamp(1, CURATED_POOL_PROFILE_IDS.len());
    curated_pool_profile_ids()
        .iter()
        .take(limit)
        .map(|profile_id| AutomoveModel {
            id: profile_id,
            select_inputs: profile_selector_from_name(profile_id)
                .unwrap_or_else(|| panic!("curated pool profile '{}' must resolve", profile_id)),
        })
        .collect()
}

pub(super) fn candidate_profile() -> &'static String {
    static PROFILE: OnceLock<String> = OnceLock::new();
    PROFILE.get_or_init(|| {
        env::var("SMART_CANDIDATE_PROFILE")
            .ok()
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "base".to_string())
    })
}

pub(super) fn env_profile_name(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
}

pub(super) fn gate_baseline_profile_name() -> String {
    env_profile_name("SMART_GATE_BASELINE_PROFILE")
        .unwrap_or_else(|| DEFAULT_PROMOTION_BASELINE_PROFILE.to_string())
}

pub(super) fn pro_candidate_profile_name() -> String {
    env_profile_name("SMART_PRO_CANDIDATE_PROFILE")
        .or_else(|| env_profile_name("SMART_CANDIDATE_PROFILE"))
        .unwrap_or_else(|| "runtime_current".to_string())
}

pub(super) fn pro_baseline_profile_name() -> String {
    env_profile_name("SMART_PRO_BASELINE_PROFILE")
        .or_else(|| Some(gate_baseline_profile_name()))
        .unwrap_or_else(|| DEFAULT_PROMOTION_BASELINE_PROFILE.to_string())
}
