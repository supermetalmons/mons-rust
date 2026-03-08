#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: ./scripts/run-automove-experiment.sh <stage> <candidate> [baseline]

stages:
  preflight    tactical guardrails + stage-1 cpu gate + exact-lite diagnostics
  fast-screen  quick active-pool screen against the baseline
  progressive  progressive duel against the baseline
  ladder       full promotion ladder against the baseline

defaults:
  baseline = runtime_release_safe_pre_exact

examples:
  ./scripts/run-automove-experiment.sh preflight runtime_eff_non_exact_v2
  ./scripts/run-automove-experiment.sh fast-screen runtime_eff_non_exact_v2
  ./scripts/run-automove-experiment.sh progressive runtime_eff_exact_lite_v1 runtime_release_safe_pre_exact
EOF
}

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  usage >&2
  exit 2
fi

stage="$1"
candidate="$2"
baseline="${3:-runtime_release_safe_pre_exact}"

run_logged() {
  local run_name="$1"
  shift
  ./scripts/run-experiment-logged.sh "${run_name}" -- "$@"
}

run_cargo_logged() {
  local run_name="$1"
  local test_name="$2"
  shift 2
  run_logged "${run_name}" env \
    "SMART_CANDIDATE_PROFILE=${candidate}" \
    "$@" \
    cargo test --release --lib "${test_name}" -- --ignored --nocapture
}

case "${stage}" in
  preflight)
    run_cargo_logged "tactical_${candidate}" "smart_automove_tactical_candidate_profile"
    run_cargo_logged "stage1_cpu_${candidate}" "smart_automove_pool_stage1_cpu_non_regression_gate"
    run_cargo_logged "exact_lite_diag_${candidate}" "smart_automove_pool_exact_lite_diagnostics_gate"
    ;;
  fast-screen)
    run_cargo_logged \
      "fast_screen_${candidate}" \
      "smart_automove_pool_fast_screen" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}"
    ;;
  progressive)
    run_cargo_logged \
      "progressive_${candidate}" \
      "smart_automove_pool_progressive_duel" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}"
    ;;
  ladder)
    run_cargo_logged \
      "ladder_${candidate}" \
      "smart_automove_pool_promotion_ladder" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}"
    ;;
  -h|--help|help)
    usage
    ;;
  *)
    echo "unknown stage: ${stage}" >&2
    usage >&2
    exit 2
    ;;
esac
