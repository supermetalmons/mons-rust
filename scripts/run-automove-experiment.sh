#!/usr/bin/env bash
set -euo pipefail

usage() {
  local triage_surfaces="opening_reply primary_pro"
  local calibration_surfaces="reply_risk opponent_mana supermana"
  cat <<'EOF_HELP'
usage:
  ./scripts/run-automove-experiment.sh <stage> <candidate> [baseline]
  ./scripts/run-automove-experiment.sh triage-calibrate [surface|all]

stages:
  guardrails              tactical guardrails only; the cheap first gate
  runtime-preflight       stage-1 cpu report (advisory for Pro) + exact-lite diagnostics; writes the duel stamp
  triage-calibrate        retained-profile calibration for reply_risk/supermana/opponent_mana
  pro-triage              deterministic Pro triage for opening_reply or primary_pro
  pro-opening-speed-probe diagnostic opening-reply latency compare on fixed Pro fixtures
  pro-reliability         focused Pro-vs-Pro, Pro-vs-Normal, and Pro-vs-Fast reliability gate
  pro-reliability-confirm larger confirmation gate with the same three Pro matchups

defaults:
  baseline = runtime_current for Pro stages
  baseline = runtime_release_safe_pre_exact for non-Pro stages
EOF_HELP
  cat <<EOF_HELP
  pro triage surfaces = ${triage_surfaces}
  triage calibration surfaces = ${calibration_surfaces}

examples:
  ./scripts/run-automove-experiment.sh triage-calibrate
  ./scripts/run-automove-experiment.sh triage-calibrate opponent_mana
  ./scripts/run-automove-experiment.sh guardrails runtime_pro_turn_engine_v30
  ./scripts/run-automove-experiment.sh runtime-preflight runtime_pro_turn_engine_v30
  SMART_TRIAGE_SURFACE=primary_pro ./scripts/run-automove-experiment.sh pro-triage runtime_pro_turn_engine_v30
  ./scripts/run-automove-experiment.sh pro-opening-speed-probe runtime_pro_turn_engine_v30
  ./scripts/run-automove-experiment.sh pro-reliability runtime_pro_turn_engine_v30
  ./scripts/run-automove-experiment.sh pro-reliability-confirm runtime_pro_turn_engine_v30
EOF_HELP
}

retained_profiles=(
  base
  runtime_current
  runtime_release_safe_pre_exact
  runtime_eff_exact_lite_v1
  runtime_pre_fast_root_quality_v1_normal_conversion_v3
  swift_2024_eval_reference
  swift_2024_style_reference
  runtime_normal_from_fast_reference_v1
  runtime_pro_turn_engine_v30
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

default_baseline_for_stage() {
  case "$1" in
    pro-triage|pro-opening-speed-probe|pro-reliability|pro-reliability-confirm)
      echo "runtime_current"
      ;;
    *)
      echo "runtime_release_safe_pre_exact"
      ;;
  esac
}

stage="${1:-}"
if [ -z "${stage}" ]; then
  usage >&2
  exit 2
fi

run_logged() {
  local run_name="$1"
  shift
  local candidate_meta="${candidate-}"
  local baseline_meta="${baseline-}"
  SMART_EXPERIMENT_CANDIDATE="${candidate_meta}" \
    SMART_EXPERIMENT_STAGE="${stage}" \
    SMART_EXPERIMENT_BASELINE="${baseline_meta}" \
    ./scripts/run-experiment-logged.sh "${run_name}" -- "$@"
}

sanitize() {
  printf '%s' "$1" | tr '[:space:]/:' '_' | tr -cd '[:alnum:]_.-'
}

experiment_stamp_dir() {
  echo "${SMART_EXPERIMENT_STAMP_DIR:-target/experiment-stamps}"
}

preflight_stamp_path() {
  local safe_candidate
  safe_candidate="$(sanitize "$1")"
  echo "$(experiment_stamp_dir)/runtime_preflight_${safe_candidate}.stamp"
}

legacy_preflight_stamp_path() {
  local safe_candidate
  safe_candidate="$(sanitize "$1")"
  echo "target/experiment-runs/runtime_preflight_${safe_candidate}.stamp"
}

clear_preflight_stamp() {
  local stamp_path
  stamp_path="$(preflight_stamp_path "$1")"
  local legacy_stamp_path
  legacy_stamp_path="$(legacy_preflight_stamp_path "$1")"
  rm -f "${stamp_path}"
  rm -f "${legacy_stamp_path}"
}

write_preflight_stamp() {
  local candidate_name="$1"
  local stamp_path
  stamp_path="$(preflight_stamp_path "${candidate_name}")"
  mkdir -p "$(dirname "${stamp_path}")"
  {
    echo "candidate=${candidate_name}"
    echo "baseline=${baseline}"
    echo "written_epoch=$(date +%s)"
  } > "${stamp_path}"
  echo "runtime preflight stamp: ${stamp_path}"
}

require_fresh_preflight_stamp() {
  local candidate_name="$1"
  local stamp_path
  stamp_path="$(preflight_stamp_path "${candidate_name}")"
  local dependency_paths=(
    "src/models/mons_game.rs"
    "src/models/scoring.rs"
    "src/models/automove_exact.rs"
    "src/models/automove_turn_planner.rs"
    "src/models/automove_turn_engine.rs"
    "src/models/mons_game_model.rs"
    "src/models/automove_experiments/profiles.rs"
  )
  if [ ! -f "${stamp_path}" ]; then
    echo "missing runtime-preflight stamp for '${candidate_name}': run './scripts/run-automove-experiment.sh runtime-preflight ${candidate_name}' first" >&2
    exit 2
  fi
  for dependency_path in "${dependency_paths[@]}"; do
    if [ "${dependency_path}" -nt "${stamp_path}" ]; then
      echo "stale runtime-preflight stamp for '${candidate_name}': rerun './scripts/run-automove-experiment.sh runtime-preflight ${candidate_name}' because the runtime or experiment profiles changed" >&2
      exit 2
    fi
  done
}

run_cargo_logged() {
  local run_name="$1"
  local test_name="$2"
  shift 2
  run_logged "${run_name}" env \
    "SMART_CANDIDATE_PROFILE=${candidate}" \
    "SMART_PRO_CANDIDATE_PROFILE=${candidate}" \
    "$@" \
    cargo test --release --lib "${test_name}" -- --ignored --nocapture
}

run_runtime_preflight() {
  clear_preflight_stamp "${candidate}"
  local stage1_extra_env=()
  if [[ "${candidate}" == runtime_pro_* ]]; then
    stage1_extra_env+=("SMART_STAGE1_INCLUDE_PRO=true")
    stage1_extra_env+=("SMART_STAGE1_CPU_ADVISORY=true")
  fi
  run_cargo_logged \
    "stage1_cpu_${candidate}" \
    "smart_automove_pool_stage1_cpu_non_regression_gate" \
    "${stage1_extra_env[@]}"
  run_cargo_logged \
    "exact_lite_diag_${candidate}" \
    "smart_automove_pool_exact_lite_diagnostics_gate"
  write_preflight_stamp "${candidate}"
}

run_pro_reliability_gate() {
  local run_name="$1"
  shift
  local extra_env=("$@")
  run_cargo_logged \
    "${run_name}" \
    "smart_automove_pool_pro_reliability_gate" \
    "SMART_GATE_BASELINE_PROFILE=${baseline}" \
    "SMART_PRO_BASELINE_PROFILE=${baseline}" \
    "SMART_PRO_RELIABILITY_CANDIDATE_PROFILE=${candidate}" \
    "SMART_PRO_RELIABILITY_BASELINE_PROFILE=${baseline}" \
    "${extra_env[@]}"
}

run_pro_opening_speed_probe() {
  run_cargo_logged \
    "pro_opening_speed_probe_${candidate}" \
    "smart_automove_pool_opening_reply_speed_probe" \
    "SMART_OPENING_SPEED_COMPARE_PROFILE=${candidate}" \
    "SMART_OPENING_SPEED_BASELINE_PROFILE=${baseline}"
}

run_triage_calibration() {
  local surface="${1:-all}"
  local calibration_candidate
  case "${surface}" in
    all)
      run_triage_calibration "reply_risk"
      run_triage_calibration "opponent_mana"
      run_triage_calibration "supermana"
      return
      ;;
    reply_risk|opponent_mana)
      calibration_candidate="runtime_pre_fast_root_quality_v1_normal_conversion_v3"
      ;;
    supermana)
      calibration_candidate="runtime_eff_exact_lite_v1"
      ;;
    *)
      echo "unknown triage calibration surface: ${surface}" >&2
      echo "expected one of: reply_risk, opponent_mana, supermana, all" >&2
      exit 2
      ;;
  esac

  run_logged \
    "triage_calibrate_${surface}_${calibration_candidate}" \
    env \
    "SMART_CANDIDATE_PROFILE=${calibration_candidate}" \
    "SMART_PRO_CANDIDATE_PROFILE=${calibration_candidate}" \
    "SMART_GATE_BASELINE_PROFILE=runtime_release_safe_pre_exact" \
    "SMART_TRIAGE_SURFACE=${surface}" \
    cargo test --release --lib smart_automove_pool_surface_calibration -- --ignored --nocapture
}

case "${stage}" in
  -h|--help|help)
    usage
    exit 0
    ;;
  triage-calibrate)
    if [ "$#" -gt 2 ]; then
      usage >&2
      exit 2
    fi
    run_triage_calibration "${2:-all}"
    exit 0
    ;;
esac

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  usage >&2
  exit 2
fi

candidate="$2"
baseline="${3:-$(default_baseline_for_stage "${stage}")}"

require_supported_profile "candidate" "${candidate}"
require_supported_profile "baseline" "${baseline}"

case "${stage}" in
  guardrails)
    run_cargo_logged "tactical_${candidate}" "smart_automove_tactical_candidate_profile"
    ;;
  runtime-preflight)
    run_runtime_preflight
    ;;
  pro-triage)
    triage_surface="${SMART_TRIAGE_SURFACE:-unset}"
    run_cargo_logged \
      "pro_triage_${triage_surface}_${candidate}" \
      "smart_automove_pool_pro_signal_triage" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}" \
      "SMART_PRO_BASELINE_PROFILE=${baseline}"
    ;;
  pro-opening-speed-probe)
    run_pro_opening_speed_probe
    ;;
  pro-reliability)
    require_fresh_preflight_stamp "${candidate}"
    run_pro_reliability_gate \
      "pro_reliability_${candidate}" \
      "SMART_PRO_RELIABILITY_REPEATS=3" \
      "SMART_PRO_RELIABILITY_GAMES=2" \
      "SMART_PRO_RELIABILITY_MAX_PLIES=96" \
      "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  pro-reliability-confirm)
    require_fresh_preflight_stamp "${candidate}"
    run_pro_reliability_gate \
      "pro_reliability_confirm_${candidate}" \
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
