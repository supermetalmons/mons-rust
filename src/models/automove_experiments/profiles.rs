use super::harness::env_usize;
use super::*;

const DEFAULT_PROMOTION_BASELINE_PROFILE: &str = "runtime_release_safe_pre_exact";
const PROFILE_RUNTIME_EFF_NON_EXACT_V1: &str = "runtime_eff_non_exact_v1";
const PROFILE_RUNTIME_EFF_NON_EXACT_V2: &str = "runtime_eff_non_exact_v2";
pub(super) const PROFILE_RUNTIME_EFF_NON_EXACT_V3: &str = "runtime_eff_non_exact_v3";
const PROFILE_RUNTIME_EFF_EXACT_LITE_V1: &str = "runtime_eff_exact_lite_v1";
pub(super) const PROFILE_RUNTIME_HISTORICAL_0_1_109: &str = "runtime_historical_0_1_109";
pub(super) const PROFILE_RUNTIME_HISTORICAL_0_1_110: &str = "runtime_historical_0_1_110";
pub(super) const PROFILE_RUNTIME_HISTORICAL_POST_0_1_110_6C3D5CB: &str =
    "runtime_historical_post_0_1_110_6c3d5cb";
pub(super) const PROFILE_RUNTIME_HISTORICAL_POST_0_1_110_A70B842: &str =
    "runtime_historical_post_0_1_110_a70b842";
pub(super) const PROFILE_RUNTIME_HISTORICAL_PRE_EXACT_E9A05CE: &str =
    "runtime_historical_pre_exact_e9a05ce";
const LEGACY_HISTORICAL_PRO_MAX_VISITED_NODES_PRE_A70: i32 =
    SMART_AUTOMOVE_NORMAL_MAX_VISITED_NODES * 3;

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

const RETAINED_PROFILES: [AutomoveProfile; 17] = [
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
        id: PROFILE_RUNTIME_HISTORICAL_0_1_109,
        selector: model_runtime_historical_0_1_109,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_HISTORICAL_0_1_110,
        selector: model_runtime_historical_0_1_110,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_HISTORICAL_POST_0_1_110_6C3D5CB,
        selector: model_runtime_historical_post_0_1_110_6c3d5cb,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_HISTORICAL_POST_0_1_110_A70B842,
        selector: model_runtime_historical_post_0_1_110_a70b842,
    },
    AutomoveProfile {
        id: PROFILE_RUNTIME_HISTORICAL_PRE_EXACT_E9A05CE,
        selector: model_runtime_historical_pre_exact_e9a05ce,
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
        id: PROFILE_RUNTIME_EFF_NON_EXACT_V3,
        selector: model_runtime_eff_non_exact_v3,
    },
    AutomoveProfile {
        id: "runtime_efficient_v1",
        selector: model_runtime_eff_non_exact_v1,
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
        id: "runtime_pre_pro_promotion_v1",
        selector: model_runtime_pre_pro_promotion_v1,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HistoricalRuntimeProfile {
    V0109,
    V0110,
    Post0110UltraIntro,
    Post0110HighUtilPro,
    PreExactWave,
}

fn historical_profile_from_name(profile_name: &str) -> Option<HistoricalRuntimeProfile> {
    match profile_name {
        PROFILE_RUNTIME_HISTORICAL_0_1_109 => Some(HistoricalRuntimeProfile::V0109),
        PROFILE_RUNTIME_HISTORICAL_0_1_110 => Some(HistoricalRuntimeProfile::V0110),
        PROFILE_RUNTIME_HISTORICAL_POST_0_1_110_6C3D5CB => {
            Some(HistoricalRuntimeProfile::Post0110UltraIntro)
        }
        PROFILE_RUNTIME_HISTORICAL_POST_0_1_110_A70B842 => {
            Some(HistoricalRuntimeProfile::Post0110HighUtilPro)
        }
        PROFILE_RUNTIME_HISTORICAL_PRE_EXACT_E9A05CE => {
            Some(HistoricalRuntimeProfile::PreExactWave)
        }
        _ => None,
    }
}

fn historical_preference_from_config(config: SmartSearchConfig) -> SmartAutomovePreference {
    if config.depth < 3 {
        SmartAutomovePreference::Fast
    } else if config.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        SmartAutomovePreference::Normal
    } else {
        SmartAutomovePreference::Pro
    }
}

fn historical_budget_for_profile(
    profile: HistoricalRuntimeProfile,
    preference: SmartAutomovePreference,
) -> (i32, i32) {
    match preference {
        SmartAutomovePreference::Fast => (
            SMART_AUTOMOVE_FAST_DEPTH,
            SMART_AUTOMOVE_FAST_MAX_VISITED_NODES,
        ),
        SmartAutomovePreference::Normal => (
            SMART_AUTOMOVE_NORMAL_DEPTH,
            SMART_AUTOMOVE_NORMAL_MAX_VISITED_NODES,
        ),
        SmartAutomovePreference::Pro => match profile {
            HistoricalRuntimeProfile::V0109
            | HistoricalRuntimeProfile::V0110
            | HistoricalRuntimeProfile::Post0110UltraIntro => (
                SMART_AUTOMOVE_PRO_DEPTH,
                LEGACY_HISTORICAL_PRO_MAX_VISITED_NODES_PRE_A70,
            ),
            HistoricalRuntimeProfile::Post0110HighUtilPro
            | HistoricalRuntimeProfile::PreExactWave => (
                SMART_AUTOMOVE_PRO_DEPTH,
                SMART_AUTOMOVE_PRO_MAX_VISITED_NODES,
            ),
        },
    }
}

fn configure_historical_fast_runtime() -> SmartSearchConfig {
    let (depth, max_nodes) =
        historical_budget_for_profile(HistoricalRuntimeProfile::V0109, SmartAutomovePreference::Fast);
    let mut tuned = SmartSearchConfig::from_budget(depth, max_nodes)
        .for_runtime()
        .with_fast_wideroot_shape();
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
    tuned.potion_spend_penalty_normal = SMART_POTION_SPEND_NO_COMPENSATION_PENALTY_NORMAL;
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

fn configure_historical_normal_runtime() -> SmartSearchConfig {
    let (depth, max_nodes) = historical_budget_for_profile(
        HistoricalRuntimeProfile::V0109,
        SmartAutomovePreference::Normal,
    );
    let mut tuned = SmartSearchConfig::from_budget(depth, max_nodes)
        .for_runtime()
        .with_normal_deeper_shape();
    tuned.max_visited_nodes = (tuned.max_visited_nodes * 3 / 2)
        .clamp(tuned.max_visited_nodes, MAX_SMART_MAX_VISITED_NODES);
    tuned.max_visited_nodes = (tuned.max_visited_nodes * 112 / 100)
        .clamp(tuned.max_visited_nodes, MAX_SMART_MAX_VISITED_NODES);
    tuned.root_branch_limit = tuned.root_branch_limit.saturating_sub(1).clamp(12, 38);
    tuned.node_branch_limit = (tuned.node_branch_limit + 2).clamp(8, 18);
    tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 240);
    tuned.node_enum_limit = ((tuned.node_branch_limit + 2) * 6).clamp(tuned.node_branch_limit, 156);
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
    tuned.selective_extension_node_share_bp = SMART_SELECTIVE_EXTENSION_NODE_SHARE_BP_NORMAL;
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

fn configure_historical_pro_runtime_base(profile: HistoricalRuntimeProfile) -> SmartSearchConfig {
    let (depth, max_nodes) = historical_budget_for_profile(profile, SmartAutomovePreference::Pro);
    match profile {
        HistoricalRuntimeProfile::V0109 => {
            let mut tuned = SmartSearchConfig::from_budget(depth, max_nodes)
                .for_runtime()
                .with_normal_deeper_shape();
            tuned.max_visited_nodes = LEGACY_HISTORICAL_PRO_MAX_VISITED_NODES_PRE_A70 as usize;
            tuned.root_branch_limit = tuned.root_branch_limit.clamp(14, 42);
            tuned.node_branch_limit = (tuned.node_branch_limit + 2).clamp(10, 18);
            tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 252);
            tuned.node_enum_limit =
                ((tuned.node_branch_limit + 2) * 6).clamp(tuned.node_branch_limit, 168);
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
            tuned.enable_forced_tactical_prepass = true;
            tuned.enable_root_drainer_safety_prefilter = true;
            tuned.enable_root_spirit_development_pref = true;
            tuned.enable_root_reply_risk_guard = true;
            tuned.root_reply_risk_score_margin = 155;
            tuned.root_reply_risk_shortlist_max = 8;
            tuned.root_reply_risk_reply_limit = 20;
            tuned.root_reply_risk_node_share_bp = 1_600;
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
        HistoricalRuntimeProfile::V0110
        | HistoricalRuntimeProfile::Post0110UltraIntro
        | HistoricalRuntimeProfile::Post0110HighUtilPro
        | HistoricalRuntimeProfile::PreExactWave => {
            let mut tuned = SmartSearchConfig::from_budget(depth, max_nodes)
                .for_runtime()
                .with_normal_deeper_shape();
            tuned.max_visited_nodes = 9_800;
            tuned.root_branch_limit = tuned.root_branch_limit.clamp(14, 34);
            tuned.node_branch_limit = tuned.node_branch_limit.clamp(9, 15);
            tuned.root_enum_limit = (tuned.root_branch_limit * 6).clamp(tuned.root_branch_limit, 204);
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
    }
}

fn apply_historical_pro_context_profile(
    game: &MonsGame,
    mut config: SmartSearchConfig,
    profile: HistoricalRuntimeProfile,
) -> SmartSearchConfig {
    if config.depth < SMART_AUTOMOVE_PRO_DEPTH as usize {
        return config;
    }

    let context = if MonsGameModel::detect_opening_book_context(game) {
        ProRuntimeContext::OpeningBookDriven
    } else {
        ProRuntimeContext::Independent
    };

    match profile {
        HistoricalRuntimeProfile::V0109 => config,
        HistoricalRuntimeProfile::V0110 | HistoricalRuntimeProfile::Post0110UltraIntro => {
            config.max_visited_nodes = 10_200;
            config.enable_forced_tactical_prepass = false;
            config.root_branch_limit = config.root_branch_limit.clamp(14, 34);
            config.node_branch_limit = config.node_branch_limit.clamp(9, 15);
            config.root_enum_limit = (config.root_branch_limit * 6).clamp(config.root_branch_limit, 204);
            config.node_enum_limit =
                ((config.node_branch_limit + 2) * 6).clamp(config.node_branch_limit, 132);
            config.enable_futility_pruning = true;
            config.enable_quiet_reductions = true;
            config.quiet_reduction_depth_threshold = 2;
            config.enable_root_reply_risk_guard = true;
            config.enable_normal_root_safety_rerank = true;
            config.enable_selective_extensions = true;
            config.max_extensions_per_path = 1;
            match context {
                ProRuntimeContext::OpeningBookDriven => {
                    config.futility_margin = 2_500;
                    config.root_reply_risk_score_margin = 155;
                    config.root_reply_risk_shortlist_max = 7;
                    config.root_reply_risk_reply_limit = 18;
                    config.root_reply_risk_node_share_bp = 1_400;
                    config.enable_normal_root_safety_deep_floor = false;
                    config.root_drainer_safety_score_margin = 4_300;
                    config.selective_extension_node_share_bp = 1_200;
                }
                ProRuntimeContext::Unknown | ProRuntimeContext::Independent => {
                    config.futility_margin = 2_300;
                    config.root_reply_risk_score_margin = 165;
                    config.root_reply_risk_shortlist_max = 9;
                    config.root_reply_risk_reply_limit = 24;
                    config.root_reply_risk_node_share_bp = 2_000;
                    config.enable_normal_root_safety_deep_floor = true;
                    config.root_drainer_safety_score_margin = 4_800;
                    config.selective_extension_node_share_bp = 1_500;
                    config.scoring_weights =
                        MonsGameModel::runtime_phase_adaptive_attacker_proximity_scoring_weights(
                            game,
                            config.depth,
                        );
                    config.interview_soft_opponent_mana_progress_bonus = 280;
                    config.interview_soft_opponent_mana_score_bonus = 340;
                }
            }
            config
        }
        HistoricalRuntimeProfile::Post0110HighUtilPro | HistoricalRuntimeProfile::PreExactWave => {
            config.max_visited_nodes = SMART_AUTOMOVE_PRO_MAX_VISITED_NODES as usize;
            config.enable_forced_tactical_prepass = false;
            config.root_branch_limit = config.root_branch_limit.clamp(14, 34);
            config.node_branch_limit = config.node_branch_limit.clamp(9, 15);
            config.root_enum_limit = (config.root_branch_limit * 6).clamp(config.root_branch_limit, 204);
            config.node_enum_limit =
                ((config.node_branch_limit + 2) * 6).clamp(config.node_branch_limit, 132);
            config.enable_futility_pruning = true;
            config.enable_quiet_reductions = true;
            config.quiet_reduction_depth_threshold = 2;
            config.enable_root_reply_risk_guard = true;
            config.enable_normal_root_safety_rerank = true;
            config.enable_selective_extensions = true;
            config.max_extensions_per_path = 1;
            match context {
                ProRuntimeContext::OpeningBookDriven => {
                    config.futility_margin = 2_500;
                    config.root_reply_risk_score_margin = 155;
                    config.root_reply_risk_shortlist_max = 7;
                    config.root_reply_risk_reply_limit = 18;
                    config.root_reply_risk_node_share_bp = 1_400;
                    config.enable_normal_root_safety_deep_floor = false;
                    config.root_drainer_safety_score_margin = 4_300;
                    config.selective_extension_node_share_bp = 1_200;
                }
                ProRuntimeContext::Unknown | ProRuntimeContext::Independent => {
                    config.futility_margin = 2_300;
                    config.root_reply_risk_score_margin = 165;
                    config.root_reply_risk_shortlist_max = 9;
                    config.root_reply_risk_reply_limit = 24;
                    config.root_reply_risk_node_share_bp = 2_000;
                    config.enable_normal_root_safety_deep_floor = true;
                    config.root_drainer_safety_score_margin = 4_800;
                    config.selective_extension_node_share_bp = 1_500;
                    config.scoring_weights =
                        MonsGameModel::runtime_phase_adaptive_attacker_proximity_scoring_weights(
                            game,
                            config.depth,
                        );
                    if profile == HistoricalRuntimeProfile::PreExactWave {
                        config.interview_soft_opponent_mana_progress_bonus = 320;
                        config.interview_soft_opponent_mana_score_bonus = 400;
                    } else {
                        config.interview_soft_opponent_mana_progress_bonus = 280;
                        config.interview_soft_opponent_mana_score_bonus = 340;
                    }
                }
            }
            config
        }
    }
}

fn configure_historical_runtime_profile(
    game: &MonsGame,
    config: SmartSearchConfig,
    profile: HistoricalRuntimeProfile,
) -> SmartSearchConfig {
    let preference = historical_preference_from_config(config);
    let mut runtime = match preference {
        SmartAutomovePreference::Fast => configure_historical_fast_runtime(),
        SmartAutomovePreference::Normal => configure_historical_normal_runtime(),
        SmartAutomovePreference::Pro => configure_historical_pro_runtime_base(profile),
    };
    runtime = MonsGameModel::with_runtime_scoring_weights(game, runtime);
    if preference == SmartAutomovePreference::Pro {
        runtime = apply_historical_pro_context_profile(game, runtime, profile);
    }
    MonsGameModel::with_pre_exact_runtime_policy(runtime)
}

pub(super) fn historical_profile_runtime_config(
    profile_name: &str,
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Option<SmartSearchConfig> {
    historical_profile_from_name(profile_name)
        .map(|profile| configure_historical_runtime_profile(game, config, profile))
}

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
    runtime_selector_inputs(game, MonsGameModel::with_pre_exact_runtime_policy(config))
}

fn historical_runtime_selector_inputs(
    game: &MonsGame,
    config: SmartSearchConfig,
    profile: HistoricalRuntimeProfile,
) -> Vec<Input> {
    runtime_selector_inputs(game, configure_historical_runtime_profile(game, config, profile))
}

pub(super) fn model_runtime_historical_0_1_109(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    historical_runtime_selector_inputs(game, config, HistoricalRuntimeProfile::V0109)
}

pub(super) fn model_runtime_historical_0_1_110(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    historical_runtime_selector_inputs(game, config, HistoricalRuntimeProfile::V0110)
}

pub(super) fn model_runtime_historical_post_0_1_110_6c3d5cb(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    historical_runtime_selector_inputs(game, config, HistoricalRuntimeProfile::Post0110UltraIntro)
}

pub(super) fn model_runtime_historical_post_0_1_110_a70b842(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    historical_runtime_selector_inputs(game, config, HistoricalRuntimeProfile::Post0110HighUtilPro)
}

pub(super) fn model_runtime_historical_pre_exact_e9a05ce(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    historical_runtime_selector_inputs(game, config, HistoricalRuntimeProfile::PreExactWave)
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

fn configure_runtime_eff_non_exact_v3(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    match historical_preference_from_config(config) {
        SmartAutomovePreference::Fast => MonsGameModel::runtime_config_for_game_with_context(
            game,
            SmartAutomovePreference::Fast,
            ProRuntimeContext::Unknown,
        )
        .0,
        SmartAutomovePreference::Normal => MonsGameModel::runtime_config_for_game_with_context(
            game,
            SmartAutomovePreference::Normal,
            ProRuntimeContext::Unknown,
        )
        .0,
        SmartAutomovePreference::Pro => configure_historical_runtime_profile(
            game,
            config,
            HistoricalRuntimeProfile::PreExactWave,
        ),
    }
}

pub(super) fn runtime_eff_non_exact_v3_runtime_config(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> SmartSearchConfig {
    configure_runtime_eff_non_exact_v3(game, config)
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

pub(super) fn model_runtime_eff_non_exact_v3(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, configure_runtime_eff_non_exact_v3(game, config))
}

pub(super) fn model_runtime_eff_exact_lite_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    MonsGameModel::smart_search_best_inputs(game, configure_runtime_eff_exact_lite_v1(game, config))
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

pub(super) fn model_runtime_pre_pro_promotion_v1(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    model_current_best(game, config)
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
                inputs.push(next_input.input.clone());
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
    MonsGameModel::smart_search_best_inputs(game, runtime)
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
