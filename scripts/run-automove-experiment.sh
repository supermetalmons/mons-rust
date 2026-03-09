#!/usr/bin/env bash
set -euo pipefail

usage() {
  local triage_surfaces="opening_reply primary_pro reply_risk supermana opponent_mana spirit_setup drainer_safety cache_reuse"
  cat <<'EOF_HELP'
usage: ./scripts/run-automove-experiment.sh <stage> <candidate> [baseline]

stages:
  preflight       tactical guardrails + stage-1 cpu gate + exact-lite diagnostics
  triage          fixed-cost deterministic triage for fast/normal (requires SMART_TRIAGE_SURFACE)
  pre-screen      legacy reject-only generic screen with tighter fast-screen budgets
  fast-screen     quick active-pool screen against the baseline
  progressive     progressive duel against the baseline
  ladder          full promotion ladder against the baseline
  pro-triage      fixed-cost deterministic triage for pro (requires SMART_TRIAGE_SURFACE)
  pro-pre-screen  legacy reject-only pro screen vs normal and fast with tighter budgets
  pro-fast-screen pro fast screens vs normal and fast at the standard budgets
  pro-progressive pro progressive duels vs normal and fast
  pro-ladder      strict pro promotion ladder against the baseline

defaults:
  baseline = runtime_release_safe_pre_exact
EOF_HELP
  cat <<EOF_HELP
  triage surfaces = ${triage_surfaces}

examples:
  ./scripts/run-automove-experiment.sh preflight runtime_eff_non_exact_v2
  SMART_TRIAGE_SURFACE=opponent_mana ./scripts/run-automove-experiment.sh triage runtime_eff_non_exact_v2
  ./scripts/run-automove-experiment.sh fast-screen runtime_eff_non_exact_v2
  SMART_TRIAGE_SURFACE=opening_reply ./scripts/run-automove-experiment.sh pro-triage runtime_eff_non_exact_v2
  ./scripts/run-automove-experiment.sh pro-ladder runtime_eff_exact_lite_v1 runtime_release_safe_pre_exact
EOF_HELP
}

if [ "$#" -eq 1 ]; then
  case "$1" in
    -h|--help|help)
      usage
      exit 0
      ;;
  esac
fi

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
    "SMART_PRO_CANDIDATE_PROFILE=${candidate}" \
    "$@" \
    cargo test --release --lib "${test_name}" -- --ignored --nocapture
}

run_fast_screen() {
  local run_name="$1"
  shift
  run_cargo_logged \
    "${run_name}" \
    "smart_automove_pool_fast_screen" \
    "SMART_GATE_BASELINE_PROFILE=${baseline}" \
    "$@"
}

run_pro_fast_screens() {
  local run_prefix="$1"
  shift
  run_cargo_logged \
    "${run_prefix}_vs_normal_${candidate}" \
    "smart_automove_pool_pro_fast_screen_vs_normal" \
    "SMART_GATE_BASELINE_PROFILE=${baseline}" \
    "SMART_PRO_BASELINE_PROFILE=${baseline}" \
    "$@"
  run_cargo_logged \
    "${run_prefix}_vs_fast_${candidate}" \
    "smart_automove_pool_pro_fast_screen_vs_fast" \
    "SMART_GATE_BASELINE_PROFILE=${baseline}" \
    "SMART_PRO_BASELINE_PROFILE=${baseline}" \
    "$@"
}

run_pro_progressive() {
  run_cargo_logged \
    "pro_progressive_vs_normal_${candidate}" \
    "smart_automove_pool_pro_progressive_vs_normal" \
    "SMART_GATE_BASELINE_PROFILE=${baseline}" \
    "SMART_PRO_BASELINE_PROFILE=${baseline}"
  run_cargo_logged \
    "pro_progressive_vs_fast_${candidate}" \
    "smart_automove_pool_pro_progressive_vs_fast" \
    "SMART_GATE_BASELINE_PROFILE=${baseline}" \
    "SMART_PRO_BASELINE_PROFILE=${baseline}"
}

case "${stage}" in
  preflight)
    run_cargo_logged "tactical_${candidate}" "smart_automove_tactical_candidate_profile"
    run_cargo_logged "stage1_cpu_${candidate}" "smart_automove_pool_stage1_cpu_non_regression_gate"
    run_cargo_logged "exact_lite_diag_${candidate}" "smart_automove_pool_exact_lite_diagnostics_gate"
    ;;
  triage)
    triage_surface="${SMART_TRIAGE_SURFACE:-unset}"
    run_cargo_logged \
      "triage_${triage_surface}_${candidate}" \
      "smart_automove_pool_signal_triage" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}"
    ;;
  pre-screen)
    run_fast_screen \
      "pre_screen_${candidate}" \
      "SMART_PROGRESSIVE_SCREEN_INITIAL_GAMES=2" \
      "SMART_PROGRESSIVE_SCREEN_MAX_GAMES=4" \
      "SMART_PROGRESSIVE_SCREEN_REPEATS=1"
    ;;
  fast-screen)
    run_fast_screen "fast_screen_${candidate}"
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
  pro-triage)
    triage_surface="${SMART_TRIAGE_SURFACE:-unset}"
    run_cargo_logged \
      "pro_triage_${triage_surface}_${candidate}" \
      "smart_automove_pool_pro_signal_triage" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}" \
      "SMART_PRO_BASELINE_PROFILE=${baseline}"
    ;;
  pro-pre-screen)
    run_pro_fast_screens \
      "pro_pre_screen" \
      "SMART_PRO_FAST_SCREEN_GAMES=1" \
      "SMART_PRO_FAST_SCREEN_REPEATS=1"
    ;;
  pro-fast-screen)
    run_pro_fast_screens "pro_fast_screen"
    ;;
  pro-progressive)
    run_pro_progressive
    ;;
  pro-ladder)
    run_cargo_logged \
      "pro_ladder_${candidate}" \
      "smart_automove_pool_pro_promotion_ladder" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}" \
      "SMART_PRO_BASELINE_PROFILE=${baseline}"
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
