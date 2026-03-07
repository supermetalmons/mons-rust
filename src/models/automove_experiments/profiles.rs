use super::*;
use super::harness::env_usize;

const DEFAULT_PROMOTION_BASELINE_PROFILE: &str = "runtime_release_safe_pre_exact";

#[derive(Clone, Copy)]
struct AutomoveProfile {
    id: &'static str,
    selector: AutomoveSelector,
}

const RETAINED_PROFILES: [AutomoveProfile; 7] = [
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

pub(super) fn model_runtime_current_profile(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

fn release_safe_pre_exact_config(mut config: SmartSearchConfig) -> SmartSearchConfig {
    config.enable_root_exact_tactics = false;
    config.enable_child_exact_tactics = false;
    config.enable_static_exact_evaluation = false;
    config
}

pub(super) fn model_runtime_release_safe_pre_exact(
    game: &MonsGame,
    config: SmartSearchConfig,
) -> Vec<Input> {
    runtime_selector_inputs(game, release_safe_pre_exact_config(config))
}

pub(super) fn model_runtime_pre_pro_promotion_v1(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    model_current_best(game, config)
}

pub(super) fn candidate_model(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let selector = profile_selector_from_name(candidate_profile().as_str())
        .unwrap_or(model_base_profile);
    selector(game, config)
}

pub(super) fn model_first_legal_automove(game: &MonsGame, _config: SmartSearchConfig) -> Vec<Input> {
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

pub(super) fn model_swift_2024_eval_reference(game: &MonsGame, config: SmartSearchConfig) -> Vec<Input> {
    let mut runtime = MonsGameModel::with_runtime_scoring_weights(game, config);
    runtime.scoring_weights = &SWIFT_2024_REFERENCE_SCORING_WEIGHTS;
    MonsGameModel::smart_search_best_inputs(game, runtime)
}

pub(super) fn model_swift_2024_style_reference(game: &MonsGame, _config: SmartSearchConfig) -> Vec<Input> {
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
    retained_profiles().iter().map(|profile| profile.id).collect()
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

pub(super) fn ultra_candidate_profile_name() -> String {
    env_profile_name("SMART_ULTRA_CANDIDATE_PROFILE")
        .or_else(|| env_profile_name("SMART_CANDIDATE_PROFILE"))
        .unwrap_or_else(|| "runtime_current".to_string())
}

pub(super) fn ultra_baseline_profile_name() -> String {
    env_profile_name("SMART_ULTRA_BASELINE_PROFILE")
        .or_else(|| Some(gate_baseline_profile_name()))
        .unwrap_or_else(|| DEFAULT_PROMOTION_BASELINE_PROFILE.to_string())
}
