#!/usr/bin/env bash
set -euo pipefail

usage() {
  local triage_surfaces="opening_reply primary_pro reply_risk supermana opponent_mana normal_fast_gap normal_release_seed_gap spirit_setup drainer_safety cache_reuse"
  cat <<'EOF_HELP'
usage:
  ./scripts/run-automove-experiment.sh <stage> <candidate> [baseline]
  ./scripts/run-automove-experiment.sh triage-calibrate [surface|all]

stages:
  guardrails      tactical guardrails only; the cheap first gate before triage
  runtime-preflight stage-1 cpu gate + exact-lite diagnostics; writes the duel stamp
  preflight       tactical guardrails + stage-1 cpu gate + exact-lite diagnostics; also writes the duel stamp
  triage-calibrate fixed retained-profile calibration for reply_risk/supermana/opponent_mana
  triage          fixed-cost deterministic triage for fast/normal (requires SMART_TRIAGE_SURFACE)
  audit-screen    cheap duel-only audit for an occasional triage reject; honors SMART_PROMOTION_TARGET_MODE=fast|normal
  pre-screen      legacy reject-only generic screen with tighter fast-screen budgets
  fast-screen     quick duel-only active-pool screen against the baseline; requires the duel stamp and honors SMART_PROMOTION_TARGET_MODE=fast|normal
  progressive     duel-only progressive evaluation against the baseline; requires the duel stamp and honors SMART_PROMOTION_TARGET_MODE=fast|normal
  ladder          duel-only promotion ladder against the baseline; requires the duel stamp and honors SMART_PROMOTION_TARGET_MODE=fast|normal
  pro-triage      fixed-cost deterministic triage for pro (requires SMART_TRIAGE_SURFACE)
  pro-audit-screen cheap pro duel-only audit for an occasional pro-triage reject
  pro-pre-screen  legacy reject-only pro screen vs normal and fast with tighter budgets
  pro-fast-screen duel-only pro fast screens vs normal and fast; requires the duel stamp
  pro-progressive duel-only pro progressive duels vs normal and fast; requires the duel stamp
  pro-ladder      duel-only strict pro promotion ladder; requires the duel stamp

defaults:
  baseline = runtime_release_safe_pre_exact
  triage override: SMART_TRIAGE_SURFACE=normal_fast_gap defaults baseline to runtime_current unless explicitly provided
EOF_HELP
  cat <<EOF_HELP
  triage surfaces = ${triage_surfaces}

examples:
  ./scripts/run-automove-experiment.sh triage-calibrate
  ./scripts/run-automove-experiment.sh triage-calibrate opponent_mana
  ./scripts/run-automove-experiment.sh guardrails runtime_eff_non_exact_v2
  ./scripts/run-automove-experiment.sh preflight runtime_eff_non_exact_v2
  SMART_TRIAGE_SURFACE=opponent_mana ./scripts/run-automove-experiment.sh triage runtime_eff_non_exact_v2
  SMART_PROMOTION_TARGET_MODE=fast ./scripts/run-automove-experiment.sh audit-screen runtime_eff_non_exact_v2
  SMART_PROMOTION_TARGET_MODE=fast ./scripts/run-automove-experiment.sh fast-screen runtime_eff_non_exact_v2
  SMART_PROMOTION_TARGET_MODE=fast ./scripts/run-automove-experiment.sh progressive runtime_eff_non_exact_v2
  SMART_TRIAGE_SURFACE=opening_reply ./scripts/run-automove-experiment.sh pro-triage runtime_eff_non_exact_v2
  ./scripts/run-automove-experiment.sh pro-audit-screen runtime_eff_non_exact_v2
  ./scripts/run-automove-experiment.sh pro-ladder runtime_eff_exact_lite_v1 runtime_release_safe_pre_exact
EOF_HELP
}

stage="${1:-}"

if [ -z "${stage}" ]; then
  usage >&2
  exit 2
fi

run_logged() {
  local run_name="$1"
  shift
  ./scripts/run-experiment-logged.sh "${run_name}" -- "$@"
}

sanitize() {
  printf '%s' "$1" | tr '[:space:]/:' '_' | tr -cd '[:alnum:]_.-'
}

experiment_log_dir() {
  echo "${SMART_EXPERIMENT_LOG_DIR:-target/experiment-runs}"
}

preflight_stamp_path() {
  local safe_candidate
  safe_candidate="$(sanitize "$1")"
  echo "$(experiment_log_dir)/runtime_preflight_${safe_candidate}.stamp"
}

clear_preflight_stamp() {
  local stamp_path
  stamp_path="$(preflight_stamp_path "$1")"
  rm -f "${stamp_path}"
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
  if [ ! -f "${stamp_path}" ]; then
    echo "missing runtime-preflight stamp for '${candidate_name}': run './scripts/run-automove-experiment.sh runtime-preflight ${candidate_name}' first" >&2
    exit 2
  fi
  if [ src/models/mons_game_model.rs -nt "${stamp_path}" ] || [ src/models/automove_experiments/profiles.rs -nt "${stamp_path}" ]; then
    echo "stale runtime-preflight stamp for '${candidate_name}': rerun './scripts/run-automove-experiment.sh runtime-preflight ${candidate_name}' because the runtime or experiment profiles changed" >&2
    exit 2
  fi
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

run_audit_screen() {
  local run_name="$1"
  shift
  run_fast_screen \
    "${run_name}" \
    "SMART_PROGRESSIVE_SCREEN_INITIAL_GAMES=2" \
    "SMART_PROGRESSIVE_SCREEN_MAX_GAMES=4" \
    "SMART_PROGRESSIVE_SCREEN_REPEATS=1" \
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

run_pro_audit_screens() {
  local run_prefix="$1"
  shift
  run_pro_fast_screens \
    "${run_prefix}" \
    "SMART_PRO_FAST_SCREEN_GAMES=1" \
    "SMART_PRO_FAST_SCREEN_REPEATS=1" \
    "$@"
}

run_pro_progressive() {
  local extra_env=("$@")
  run_cargo_logged \
    "pro_progressive_vs_normal_${candidate}" \
    "smart_automove_pool_pro_progressive_vs_normal" \
    "SMART_GATE_BASELINE_PROFILE=${baseline}" \
    "SMART_PRO_BASELINE_PROFILE=${baseline}" \
    "${extra_env[@]}"
  run_cargo_logged \
    "pro_progressive_vs_fast_${candidate}" \
    "smart_automove_pool_pro_progressive_vs_fast" \
    "SMART_GATE_BASELINE_PROFILE=${baseline}" \
    "SMART_PRO_BASELINE_PROFILE=${baseline}" \
    "${extra_env[@]}"
}

run_triage_calibration() {
  local surface="${1:-all}"
  local candidate_profile
  case "${surface}" in
    all)
      run_triage_calibration "reply_risk"
      run_triage_calibration "opponent_mana"
      run_triage_calibration "supermana"
      return
      ;;
    reply_risk|opponent_mana)
      candidate_profile="runtime_pre_fast_root_quality_v1_normal_conversion_v3"
      ;;
    supermana)
      candidate_profile="runtime_eff_exact_lite_v1"
      ;;
    *)
      echo "unknown triage calibration surface: ${surface}" >&2
      echo "expected one of: reply_risk, opponent_mana, supermana, all" >&2
      exit 2
      ;;
  esac

  run_logged \
    "triage_calibrate_${surface}_${candidate_profile}" \
    env \
    "SMART_CANDIDATE_PROFILE=${candidate_profile}" \
    "SMART_PRO_CANDIDATE_PROFILE=${candidate_profile}" \
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
baseline="${3:-runtime_release_safe_pre_exact}"
baseline_was_explicit=false
if [ "$#" -eq 3 ]; then
  baseline_was_explicit=true
fi

case "${stage}" in
  guardrails)
    run_cargo_logged "tactical_${candidate}" "smart_automove_tactical_candidate_profile"
    ;;
  runtime-preflight)
    clear_preflight_stamp "${candidate}"
    run_cargo_logged "stage1_cpu_${candidate}" "smart_automove_pool_stage1_cpu_non_regression_gate"
    run_cargo_logged "exact_lite_diag_${candidate}" "smart_automove_pool_exact_lite_diagnostics_gate"
    write_preflight_stamp "${candidate}"
    ;;
  preflight)
    clear_preflight_stamp "${candidate}"
    run_cargo_logged "tactical_${candidate}" "smart_automove_tactical_candidate_profile"
    run_cargo_logged "stage1_cpu_${candidate}" "smart_automove_pool_stage1_cpu_non_regression_gate"
    run_cargo_logged "exact_lite_diag_${candidate}" "smart_automove_pool_exact_lite_diagnostics_gate"
    write_preflight_stamp "${candidate}"
    ;;
  triage)
    triage_surface="${SMART_TRIAGE_SURFACE:-unset}"
    if [ "${triage_surface}" = "normal_fast_gap" ] && [ "${baseline_was_explicit}" = false ]; then
      baseline="runtime_current"
      echo "triage surface normal_fast_gap: defaulting baseline to runtime_current"
    fi
    run_cargo_logged \
      "triage_${triage_surface}_${candidate}" \
      "smart_automove_pool_signal_triage" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}"
    ;;
  audit-screen)
    run_audit_screen "audit_screen_${candidate}" "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  pre-screen)
    run_audit_screen "pre_screen_${candidate}" "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  fast-screen)
    require_fresh_preflight_stamp "${candidate}"
    run_fast_screen "fast_screen_${candidate}" "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  progressive)
    require_fresh_preflight_stamp "${candidate}"
    run_cargo_logged \
      "progressive_${candidate}" \
      "smart_automove_pool_progressive_duel" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}" \
      "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  ladder)
    require_fresh_preflight_stamp "${candidate}"
    run_cargo_logged \
      "ladder_${candidate}" \
      "smart_automove_pool_promotion_ladder" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}" \
      "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  pro-triage)
    triage_surface="${SMART_TRIAGE_SURFACE:-unset}"
    run_cargo_logged \
      "pro_triage_${triage_surface}_${candidate}" \
      "smart_automove_pool_pro_signal_triage" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}" \
      "SMART_PRO_BASELINE_PROFILE=${baseline}"
    ;;
  pro-audit-screen)
    run_pro_audit_screens "pro_audit_screen" "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  pro-pre-screen)
    run_pro_audit_screens "pro_pre_screen" "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  pro-fast-screen)
    require_fresh_preflight_stamp "${candidate}"
    run_pro_fast_screens "pro_fast_screen" "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  pro-progressive)
    require_fresh_preflight_stamp "${candidate}"
    run_pro_progressive "SMART_SKIP_RUNTIME_PREFLIGHT=true"
    ;;
  pro-ladder)
    require_fresh_preflight_stamp "${candidate}"
    run_cargo_logged \
      "pro_ladder_${candidate}" \
      "smart_automove_pool_pro_promotion_ladder" \
      "SMART_GATE_BASELINE_PROFILE=${baseline}" \
      "SMART_PRO_BASELINE_PROFILE=${baseline}" \
      "SMART_SKIP_RUNTIME_PREFLIGHT=true"
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
