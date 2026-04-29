#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF_HELP'
usage:
  ./scripts/run-automove-experiment.sh <stage> <frontier> [shipping]

single-stage runner:
  use this script for one stage at a time or for diagnostics only

stages:
  guardrails              tactical guardrails only; the cheap first gate
  variant-smoke           cheap legal/public automove smoke over every game variant
  runtime-preflight       stage-1 cpu report (advisory for Pro) + exact-lite diagnostics; writes the duel stamp
  pro-triage              deterministic retained Classic primary_pro triage
  pro-reliability         sampled-variant Pro-vs-Pro, Pro-vs-Normal, and Pro-vs-Fast reliability gate
  pro-reliability-confirm all-variant confirmation gate with the same three Pro matchups
  pro-profile-sweep       diagnostic: sweep one or more test-only Pro profile candidates
  pro-profile-attribution diagnostic: attribute outcome deltas between two sweep candidates
  pro-promotion-dashboard diagnostic: summarize sampled + active-blocker promotion shape
  pro-sweep-decision-record diagnostic: aggregate nonwins/deltas for one sweep candidate
  pro-policy-matrix      diagnostic: compare multiple sweep policies on identical openings
  pro-policy-outcome-corpus diagnostic: policy-matrix outcome corpus plus stoplights
  pro-policy-cross-budget diagnostic: compare one policy choice across Pro/Normal/Fast opponents on identical openings
  pro-policy-winner      diagnostic: short-circuit first winning policy contexts
  pro-policy-corpus      diagnostic: policy-winner plus guarded mechanism aggregates

defaults:
  shipping = shipping_pro_search for Pro stages
EOF_HELP
  cat <<'EOF_HELP'
examples:
  ./scripts/run-automove-experiment.sh guardrails frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh variant-smoke frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh runtime-preflight frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh pro-triage frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh pro-reliability frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh pro-reliability-confirm frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh pro-profile-sweep frontier_pro_v2_raw
  SMART_PRO_SWEEP_ATTRIBUTION_RIGHT=frontier_pro_v2_raw ./scripts/run-automove-experiment.sh pro-profile-attribution frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh pro-promotion-dashboard frontier_pro_v2_raw
  ./scripts/run-automove-experiment.sh pro-sweep-decision-record frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh pro-policy-matrix frontier_pro_v2_guarded,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard
  ./scripts/run-automove-experiment.sh pro-policy-outcome-corpus frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,frontier_pro_v3_white_opening_utility_mana,shipping_pro_search_control,frontier_pro_v2_raw,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard,frontier_pro_v2_no_low_budget_guard
  ./scripts/run-automove-experiment.sh pro-policy-cross-budget frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,shipping_pro_search_control
  ./scripts/run-automove-experiment.sh pro-policy-winner frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,shipping_pro_search_control
  ./scripts/run-automove-experiment.sh pro-policy-corpus frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,frontier_pro_v3_white_opening_utility_mana,shipping_pro_search_control,frontier_pro_v2_raw,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard,frontier_pro_v2_no_low_budget_guard
EOF_HELP
}

retained_profiles=(
  shipping_pro_search
  frontier_pro_v2_guarded
)

sweep_candidates=(
  shipping_pro_search_control
  frontier_pro_v2_guarded
  frontier_pro_v2_raw
  frontier_pro_v2_no_selected_followup_projection
  frontier_pro_v3_full_scored_reply_guard
  frontier_pro_v2_no_low_budget_guard
  frontier_pro_v3_alternating_white_edge_mana
  frontier_pro_v3_white_opening_utility_mana
)

sweep_candidate_metadata() {
  local candidate_id="$1"
  case "${candidate_id}" in
    shipping_pro_search_control)
      metadata_mechanism="search-only shipping baseline as a policy candidate"
      metadata_expected_invariant="expose where retained search saves guarded without implying source routing"
      metadata_risk_rows="shipping-control repairs have been singleton or budget-conflicted"
      metadata_kill_condition="no repeated non-regressing mechanism below policy label"
      ;;
    frontier_pro_v2_guarded)
      metadata_mechanism="retained shipped Pro guarded wrapper path"
      metadata_expected_invariant="anchor promotion and corpus comparisons to the public Pro route"
      metadata_risk_rows="self-scouts can only report promotion shape or no-source evidence"
      metadata_kill_condition="dashboard not_promising or corpus source_permission no_source"
      ;;
    frontier_pro_v2_raw)
      metadata_mechanism="direct ProV2 turn-engine path without guarded wrapper fallback routing"
      metadata_expected_invariant="test whether guarded fallback routing is suppressing active-blocker strength"
      metadata_risk_rows="sampled Pro and guarded-save regressions"
      metadata_kill_condition="sampled dashboard miss or baseline-save contamination"
      ;;
    frontier_pro_v2_no_selected_followup_projection)
      metadata_mechanism="guarded runtime with selected followup projection disabled"
      metadata_expected_invariant="separate selected-root overcommit from durable root utility"
      metadata_risk_rows="active Fast only repairs and cross-budget conflicts"
      metadata_kill_condition="mixed cross-budget or fragmented route evidence"
      ;;
    frontier_pro_v3_full_scored_reply_guard)
      metadata_mechanism="reply-risk guard over the full scored root shortlist"
      metadata_expected_invariant="recover roots hidden by selected shortlist timing"
      metadata_risk_rows="guarded white saves and timing-specific baseline-better rows"
      metadata_kill_condition="coverage gap, baseline-save risk, or branch-pair fragmentation"
      ;;
    frontier_pro_v2_no_low_budget_guard)
      metadata_mechanism="guarded runtime with low-budget guard disabled"
      metadata_expected_invariant="test whether low-budget gating blocks active forward repairs"
      metadata_risk_rows="local active forward gains without sampled or budget stability"
      metadata_kill_condition="no-policy help or mixed cross-budget result"
      ;;
    frontier_pro_v3_alternating_white_edge_mana)
      metadata_mechanism="test-only alternating white opening edge-mana root preference"
      metadata_expected_invariant="cover the known alternating white sampled opening class"
      metadata_risk_rows="variant and opening scoped selector pressure"
      metadata_kill_condition="dashboard miss or singleton mechanism evidence"
      ;;
    frontier_pro_v3_white_opening_utility_mana)
      metadata_mechanism="test-only white opening quiet-mana utility selector"
      metadata_expected_invariant="cover the sampled Fast white corner-chain selected-root miss"
      metadata_risk_rows="narrow white opening utility gate and sampled-only overfit"
      metadata_kill_condition="dashboard miss, cost pressure, or fragmented corpus evidence"
      ;;
    *)
      metadata_mechanism="unknown"
      metadata_expected_invariant="unknown"
      metadata_risk_rows="unknown"
      metadata_kill_condition="unknown"
      ;;
  esac
}

print_one_sweep_candidate_metadata() {
  local role="$1"
  local candidate_id="$2"
  sweep_candidate_metadata "${candidate_id}"
  printf 'AUTOMOVE_SWEEP_CANDIDATE_METADATA {"role":"%s","id":"%s","mechanism":"%s","expected_invariant":"%s","risk_rows":"%s","kill_condition":"%s"}\n' \
    "${role}" \
    "${candidate_id}" \
    "${metadata_mechanism}" \
    "${metadata_expected_invariant}" \
    "${metadata_risk_rows}" \
    "${metadata_kill_condition}"
}

print_sweep_candidate_metadata() {
  local role="$1"
  local value="$2"
  local old_ifs="${IFS}"
  local token
  local supported
  IFS=','
  for token in ${value}; do
    IFS="${old_ifs}"
    token="$(printf '%s' "${token}" | xargs)"
    IFS=','
    if [ -z "${token}" ]; then
      continue
    fi
    if [ "${token}" = "all" ]; then
      for supported in "${sweep_candidates[@]}"; do
        print_one_sweep_candidate_metadata "${role}" "${supported}"
      done
      continue
    fi
    print_one_sweep_candidate_metadata "${role}" "${token}"
  done
  IFS="${old_ifs}"
}

profile_is_supported() {
  local profile="$1"
  local supported
  for supported in "${retained_profiles[@]}"; do
    if [ "${supported}" = "${profile}" ]; then
      return 0
    fi
  done
  return 1
}

require_supported_profile() {
  local role="$1"
  local profile="$2"
  if profile_is_supported "${profile}"; then
    return 0
  fi
  echo "unsupported ${role} profile: '${profile}'" >&2
  echo "supported profiles: ${retained_profiles[*]}" >&2
  exit 2
}

profile_is_sweep_candidate() {
  local profile="$1"
  local supported
  for supported in "${sweep_candidates[@]}"; do
    if [ "${supported}" = "${profile}" ]; then
      return 0
    fi
  done
  return 1
}

require_supported_sweep_candidate() {
  local role="$1"
  local profile="$2"
  if profile_is_sweep_candidate "${profile}"; then
    return 0
  fi
  echo "unsupported ${role} sweep candidate: '${profile}'" >&2
  echo "supported sweep candidates: all ${sweep_candidates[*]}" >&2
  exit 2
}

require_supported_sweep_filter() {
  local role="$1"
  local value="$2"
  local old_ifs="${IFS}"
  local token
  IFS=','
  for token in ${value}; do
    IFS="${old_ifs}"
    token="$(printf '%s' "${token}" | xargs)"
    IFS=','
    if [ -z "${token}" ] || [ "${token}" = "all" ]; then
      continue
    fi
    require_supported_sweep_candidate "${role}" "${token}"
  done
  IFS="${old_ifs}"
}

default_shipping_profile_for_stage() {
  echo "shipping_pro_search"
}

stage="${1:-}"
if [ -z "${stage}" ]; then
  usage >&2
  exit 2
fi

run_logged() {
  local run_name="$1"
  shift
  local frontier_meta="${frontier-}"
  local shipping_meta="${shipping-}"
  local variant_policy_meta="${SMART_EXPERIMENT_VARIANT_POLICY:-${SMART_AUTOMOVE_VARIANT_POLICY:-}}"
  local variant_list_meta="${SMART_EXPERIMENT_VARIANTS:-${SMART_AUTOMOVE_VARIANTS:-}}"
  SMART_EXPERIMENT_FRONTIER="${frontier_meta}" \
  SMART_EXPERIMENT_STAGE="${stage}" \
  SMART_EXPERIMENT_SHIPPING="${shipping_meta}" \
  SMART_EXPERIMENT_VARIANT_POLICY="${variant_policy_meta}" \
  SMART_EXPERIMENT_VARIANTS="${variant_list_meta}" \
    ./scripts/run-experiment-logged.sh "${run_name}" -- "$@"
}

sanitize() {
  printf '%s' "$1" | tr '[:space:]/:,' '_' | tr -cd '[:alnum:]_.-'
}

experiment_stamp_dir() {
  echo "${SMART_EXPERIMENT_STAMP_DIR:-target/experiment-stamps}"
}

preflight_stamp_path() {
  local safe_frontier
  safe_frontier="$(sanitize "$1")"
  echo "$(experiment_stamp_dir)/runtime_preflight_${safe_frontier}.stamp"
}

clear_preflight_stamp() {
  local stamp_path
  stamp_path="$(preflight_stamp_path "$1")"
  rm -f "${stamp_path}"
}

write_preflight_stamp() {
  local frontier_id="$1"
  local stamp_path
  stamp_path="$(preflight_stamp_path "${frontier_id}")"
  mkdir -p "$(dirname "${stamp_path}")"
  {
    echo "frontier=${frontier_id}"
    echo "shipping=${shipping}"
    echo "variant_policy=${SMART_AUTOMOVE_VARIANT_POLICY:-sampled}"
    echo "variants=${SMART_AUTOMOVE_VARIANTS:-<default>}"
    echo "written_epoch=$(date +%s)"
  } > "${stamp_path}"
  echo "runtime preflight stamp: ${stamp_path}"
}

require_fresh_preflight_stamp() {
  local frontier_id="$1"
  local stamp_path
  stamp_path="$(preflight_stamp_path "${frontier_id}")"
  local dependency_paths=(
    "src/models/mons_game.rs"
    "src/models/scoring.rs"
    "src/models/automove_exact.rs"
    "src/models/automove_turn_engine.rs"
    "src/models/automove_runtime_variants.rs"
    "src/models/mons_game_model.rs"
    "src/models/automove_experiments/profiles.rs"
  )
  if [ ! -f "${stamp_path}" ]; then
    echo "missing runtime-preflight stamp for '${frontier_id}': run './scripts/run-automove-experiment.sh runtime-preflight ${frontier_id}' first" >&2
    exit 2
  fi
  for dependency_path in "${dependency_paths[@]}"; do
    if [ "${dependency_path}" -nt "${stamp_path}" ]; then
      echo "stale runtime-preflight stamp for '${frontier_id}': rerun './scripts/run-automove-experiment.sh runtime-preflight ${frontier_id}' because the runtime or experiment profiles changed" >&2
      exit 2
    fi
  done
}

run_cargo_logged() {
  local run_name="$1"
  local test_name="$2"
  shift 2
  run_logged "${run_name}" env \
    "SMART_SELECTED_PROFILE=${frontier}" \
    "SMART_FRONTIER_PROFILE=${frontier}" \
    "$@" \
    cargo test --release --lib "${test_name}" -- --ignored --nocapture
}

run_runtime_preflight() {
  clear_preflight_stamp "${frontier}"
  local stage1_extra_env=()
  if [[ "${frontier}" == frontier_pro_* ]]; then
    stage1_extra_env+=("SMART_STAGE1_INCLUDE_PRO=true")
    stage1_extra_env+=("SMART_STAGE1_CPU_ADVISORY=true")
  fi
  SMART_EXPERIMENT_VARIANT_POLICY="sampled" run_cargo_logged \
    "stage1_cpu_${frontier}" \
    "smart_automove_pool_stage1_cpu_non_regression_gate" \
    "SMART_AUTOMOVE_VARIANT_POLICY=sampled" \
    "${stage1_extra_env[@]}"
  SMART_EXPERIMENT_VARIANT_POLICY="sampled" run_cargo_logged \
    "exact_lite_diag_${frontier}" \
    "smart_automove_pool_exact_lite_diagnostics_gate" \
    "SMART_AUTOMOVE_VARIANT_POLICY=sampled"
  write_preflight_stamp "${frontier}"
}

run_pro_reliability_gate() {
  local run_name="$1"
  shift
  local extra_env=("$@")
  run_cargo_logged \
    "${run_name}" \
    "smart_automove_pool_pro_reliability_gate" \
    "SMART_SHIPPING_PROFILE=${shipping}" \
    "SMART_PRO_RELIABILITY_FRONTIER_PROFILE=${frontier}" \
    "SMART_PRO_RELIABILITY_SHIPPING_PROFILE=${shipping}" \
    "${extra_env[@]}"
}

case "${stage}" in
  -h|--help|help)
    usage
    exit 0
    ;;
esac

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  usage >&2
  exit 2
fi

frontier="$2"
shipping="${3:-$(default_shipping_profile_for_stage "${stage}")}"

require_supported_profile "shipping" "${shipping}"

case "${stage}" in
  pro-profile-sweep)
    require_supported_sweep_filter "frontier" "${frontier}"
    print_sweep_candidate_metadata "frontier" "${frontier}"
    ;;
  pro-promotion-dashboard)
    require_supported_sweep_filter "frontier" "${frontier}"
    print_sweep_candidate_metadata "frontier" "${frontier}"
    ;;
  pro-sweep-decision-record)
    require_supported_sweep_candidate "frontier" "${frontier}"
    print_sweep_candidate_metadata "frontier" "${frontier}"
    ;;
  pro-policy-matrix)
    require_supported_sweep_filter "frontier" "${frontier}"
    print_sweep_candidate_metadata "frontier" "${frontier}"
    ;;
  pro-policy-outcome-corpus)
    require_supported_sweep_filter "frontier" "${frontier}"
    print_sweep_candidate_metadata "frontier" "${frontier}"
    ;;
  pro-policy-cross-budget)
    require_supported_sweep_filter "frontier" "${frontier}"
    print_sweep_candidate_metadata "frontier" "${frontier}"
    ;;
  pro-policy-winner)
    require_supported_sweep_filter "frontier" "${frontier}"
    print_sweep_candidate_metadata "frontier" "${frontier}"
    ;;
  pro-policy-corpus)
    require_supported_sweep_filter "frontier" "${frontier}"
    print_sweep_candidate_metadata "frontier" "${frontier}"
    ;;
  pro-profile-attribution)
    require_supported_sweep_candidate "frontier" "${frontier}"
    print_sweep_candidate_metadata "frontier" "${frontier}"
    if [ -n "${SMART_PRO_SWEEP_ATTRIBUTION_RIGHT:-}" ]; then
      require_supported_sweep_candidate "attribution right" "${SMART_PRO_SWEEP_ATTRIBUTION_RIGHT}"
      print_sweep_candidate_metadata "attribution_right" "${SMART_PRO_SWEEP_ATTRIBUTION_RIGHT}"
    fi
    ;;
  *)
    require_supported_profile "frontier" "${frontier}"
    ;;
esac

case "${stage}" in
  guardrails)
    run_cargo_logged "tactical_${frontier}" "smart_automove_tactical_selected_profile"
    ;;
  variant-smoke)
    SMART_EXPERIMENT_VARIANT_POLICY="all" run_cargo_logged \
      "variant_smoke_${frontier}" \
      "smart_automove_pool_variant_smoke_gate" \
      "SMART_AUTOMOVE_VARIANT_POLICY=all"
    ;;
  runtime-preflight)
    run_runtime_preflight
    ;;
  pro-triage)
    run_cargo_logged \
      "pro_triage_primary_pro_${frontier}" \
      "smart_automove_pool_pro_signal_triage" \
      "SMART_SHIPPING_PROFILE=${shipping}"
    ;;
  pro-reliability)
    require_fresh_preflight_stamp "${frontier}"
    SMART_EXPERIMENT_VARIANT_POLICY="sampled" run_pro_reliability_gate \
      "pro_reliability_${frontier}" \
      "SMART_AUTOMOVE_VARIANT_POLICY=sampled" \
      "SMART_PRO_RELIABILITY_REPEATS=3" \
      "SMART_PRO_RELIABILITY_GAMES=2" \
      "SMART_PRO_RELIABILITY_MAX_PLIES=96" \
      "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  pro-reliability-confirm)
    require_fresh_preflight_stamp "${frontier}"
    SMART_EXPERIMENT_VARIANT_POLICY="all" run_pro_reliability_gate \
      "pro_reliability_confirm_${frontier}" \
      "SMART_AUTOMOVE_VARIANT_POLICY=all" \
      "SMART_PRO_RELIABILITY_REQUIRE_VARIANT_FLOOR=true" \
      "SMART_PRO_RELIABILITY_REPEATS=2" \
      "SMART_PRO_RELIABILITY_GAMES=12" \
      "SMART_PRO_RELIABILITY_MAX_PLIES=96" \
      "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  pro-profile-sweep)
    run_cargo_logged \
      "pro_profile_sweep_$(sanitize "${frontier}")" \
      "smart_automove_pro_profile_sweep_probe" \
      "SMART_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_RELIABILITY_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_SWEEP_CANDIDATES=${frontier}"
    ;;
  pro-profile-attribution)
    attribution_right_env=()
    if [ -n "${SMART_PRO_SWEEP_ATTRIBUTION_RIGHT:-}" ]; then
      attribution_right_env=("SMART_PRO_SWEEP_ATTRIBUTION_RIGHT=${SMART_PRO_SWEEP_ATTRIBUTION_RIGHT}")
    fi
    run_cargo_logged \
      "pro_profile_attribution_$(sanitize "${frontier}")" \
      "smart_automove_pro_profile_attribution_probe" \
      "SMART_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_RELIABILITY_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_SWEEP_ATTRIBUTION_LEFT=${frontier}" \
      "${attribution_right_env[@]}"
    ;;
  pro-promotion-dashboard)
    run_cargo_logged \
      "pro_promotion_dashboard_$(sanitize "${frontier}")" \
      "smart_automove_pro_promotion_dashboard_probe" \
      "SMART_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_RELIABILITY_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_DASHBOARD_CANDIDATES=${frontier}"
    ;;
  pro-sweep-decision-record)
    run_cargo_logged \
      "pro_sweep_decision_record_$(sanitize "${frontier}")" \
      "smart_automove_pro_sweep_decision_record_probe" \
      "SMART_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_RELIABILITY_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_SWEEP_DECISION_RECORD_CANDIDATE=${frontier}"
    ;;
  pro-policy-matrix)
    run_cargo_logged \
      "pro_policy_matrix_$(sanitize "${frontier}")" \
      "smart_automove_pro_policy_matrix_probe" \
      "SMART_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_RELIABILITY_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_POLICY_MATRIX_CANDIDATES=${frontier}"
    ;;
  pro-policy-outcome-corpus)
    run_cargo_logged \
      "pro_policy_outcome_corpus_$(sanitize "${frontier}")" \
      "smart_automove_pro_policy_matrix_probe" \
      "SMART_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_RELIABILITY_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_POLICY_MATRIX_INCLUDE_CORPUS_RECORDS=${SMART_PRO_POLICY_MATRIX_INCLUDE_CORPUS_RECORDS:-true}" \
      "SMART_PRO_POLICY_MATRIX_CANDIDATES=${frontier}"
    ;;
  pro-policy-cross-budget)
    run_cargo_logged \
      "pro_policy_cross_budget_$(sanitize "${frontier}")" \
      "smart_automove_pro_policy_cross_budget_probe" \
      "SMART_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_RELIABILITY_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_POLICY_CROSS_BUDGET_CANDIDATES=${frontier}"
    ;;
  pro-policy-winner)
    run_cargo_logged \
      "pro_policy_winner_$(sanitize "${frontier}")" \
      "smart_automove_pro_policy_winner_probe" \
      "SMART_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_RELIABILITY_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_POLICY_WINNER_CANDIDATES=${frontier}"
    ;;
  pro-policy-corpus)
    run_cargo_logged \
      "pro_policy_corpus_$(sanitize "${frontier}")" \
      "smart_automove_pro_policy_winner_probe" \
      "SMART_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_RELIABILITY_SHIPPING_PROFILE=${shipping}" \
      "SMART_PRO_POLICY_WINNER_CANDIDATES=${frontier}" \
      "SMART_PRO_POLICY_WINNER_INCLUDE_MECHANISM=true"
    ;;
  *)
    echo "unknown stage: ${stage}" >&2
    usage >&2
    exit 2
    ;;
esac
