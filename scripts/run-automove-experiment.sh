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
  runtime-preflight       stage-1 cpu report (advisory for Pro) + exact-lite diagnostics; writes the duel stamp
  pro-triage              deterministic primary_pro triage
  pro-reliability         focused Pro-vs-Pro, Pro-vs-Normal, and Pro-vs-Fast reliability gate
  pro-reliability-confirm larger confirmation gate with the same three Pro matchups

defaults:
  shipping = shipping_pro_search for Pro stages
EOF_HELP
  cat <<'EOF_HELP'
examples:
  ./scripts/run-automove-experiment.sh guardrails frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh runtime-preflight frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh pro-triage frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh pro-reliability frontier_pro_v2_guarded
  ./scripts/run-automove-experiment.sh pro-reliability-confirm frontier_pro_v2_guarded
EOF_HELP
}

retained_profiles=(
  shipping_pro_search
  frontier_pro_v2_guarded
)

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
  SMART_EXPERIMENT_FRONTIER="${frontier_meta}" \
  SMART_EXPERIMENT_STAGE="${stage}" \
    SMART_EXPERIMENT_SHIPPING="${shipping_meta}" \
    ./scripts/run-experiment-logged.sh "${run_name}" -- "$@"
}

sanitize() {
  printf '%s' "$1" | tr '[:space:]/:' '_' | tr -cd '[:alnum:]_.-'
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
  run_cargo_logged \
    "stage1_cpu_${frontier}" \
    "smart_automove_pool_stage1_cpu_non_regression_gate" \
    "${stage1_extra_env[@]}"
  run_cargo_logged \
    "exact_lite_diag_${frontier}" \
    "smart_automove_pool_exact_lite_diagnostics_gate"
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

require_supported_profile "frontier" "${frontier}"
require_supported_profile "shipping" "${shipping}"

case "${stage}" in
  guardrails)
    run_cargo_logged "tactical_${frontier}" "smart_automove_tactical_selected_profile"
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
    run_pro_reliability_gate \
      "pro_reliability_${frontier}" \
      "SMART_PRO_RELIABILITY_REPEATS=3" \
      "SMART_PRO_RELIABILITY_GAMES=2" \
      "SMART_PRO_RELIABILITY_MAX_PLIES=96" \
      "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  pro-reliability-confirm)
    require_fresh_preflight_stamp "${frontier}"
    run_pro_reliability_gate \
      "pro_reliability_confirm_${frontier}" \
      "SMART_PRO_RELIABILITY_REPEATS=4" \
      "SMART_PRO_RELIABILITY_GAMES=4" \
      "SMART_PRO_RELIABILITY_MAX_PLIES=96" \
      "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  *)
    echo "unknown stage: ${stage}" >&2
    usage >&2
    exit 2
    ;;
esac
