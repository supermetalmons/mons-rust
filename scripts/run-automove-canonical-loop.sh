#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage:
  ./scripts/run-automove-canonical-loop.sh [--confirm] <frontier> [shipping]

default shipping profile:
  shipping_pro_search

canonical order:
  1. guardrails
  2. SMART_TRIAGE_SURFACE=primary_pro pro-triage
  3. runtime-preflight
  4. pro-reliability
  5. pro-reliability-confirm (only when --confirm is passed)
EOF
}

sanitize() {
  printf '%s' "$1" | tr '[:space:]/:' '_' | tr -cd '[:alnum:]_.-'
}

experiment_log_root() {
  echo "${SMART_EXPERIMENT_LOG_ROOT:-target/experiment-runs}"
}

experiment_stamp_dir() {
  echo "${SMART_EXPERIMENT_STAMP_DIR:-target/experiment-stamps}"
}

print_artifact_summary() {
  local status="$1"
  local safe_frontier
  safe_frontier="$(sanitize "${frontier}")"
  local frontier_log_dir
  frontier_log_dir="$(experiment_log_root)/${safe_frontier}"
  local stamp_path
  stamp_path="$(experiment_stamp_dir)/runtime_preflight_${safe_frontier}.stamp"

  if [ "${status}" -eq 0 ]; then
    echo "canonical loop complete for ${frontier}"
  else
    echo "canonical loop stopped at stage: ${last_stage}" >&2
  fi

  echo "artifacts:"
  echo "  frontier logs: ${frontier_log_dir}"
  echo "  runtime-preflight stamp: ${stamp_path}"
}

run_stage() {
  local stage_name="$1"
  last_stage="${stage_name}"
  case "${stage_name}" in
    pro-triage)
      SMART_TRIAGE_SURFACE=primary_pro \
        ./scripts/run-automove-experiment.sh "${stage_name}" "${frontier}" "${shipping}"
      ;;
    *)
      ./scripts/run-automove-experiment.sh "${stage_name}" "${frontier}" "${shipping}"
      ;;
  esac
}

confirm=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --confirm)
      confirm=true
      shift
      ;;
    -h|--help|help)
      usage
      exit 0
      ;;
    *)
      break
      ;;
  esac
done

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  usage >&2
  exit 2
fi

frontier="$1"
shipping="${2:-shipping_pro_search}"
last_stage="startup"

trap 'status=$?; print_artifact_summary "${status}"' EXIT

run_stage guardrails
run_stage pro-triage
run_stage runtime-preflight
run_stage pro-reliability

if [ "${confirm}" = true ]; then
  run_stage pro-reliability-confirm
fi
