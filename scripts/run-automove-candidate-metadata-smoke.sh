#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage:
  ./scripts/run-automove-candidate-metadata-smoke.sh [--log-root <path>] [--keep-capture] [candidate] [shipping]

Runs a capped structural-scout outcome-corpus smoke and validates that
AUTOMOVE_SWEEP_CANDIDATE_METADATA is printed for the dashboard candidate and
the reset portfolio before the logged diagnostics spend.

Defaults:
  candidate = frontier_pro_v2_guarded
  shipping  = shipping_pro_search
  log root  = /tmp/automove-candidate-metadata-smoke

The script intentionally uses one stable command path so it can be approved once
for future metadata-smoke runs.
EOF
}

default_portfolio=(
  frontier_pro_v2_guarded
  frontier_pro_v3_alternating_white_edge_mana
  frontier_pro_v3_white_opening_utility_mana
  shipping_pro_search_control
  frontier_pro_v2_raw
  frontier_pro_v2_no_selected_followup_projection
  frontier_pro_v3_full_scored_reply_guard
  frontier_pro_v2_no_low_budget_guard
)

log_root="${SMART_AUTOMOVE_METADATA_SMOKE_LOG_ROOT:-/tmp/automove-candidate-metadata-smoke}"
keep_capture=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --log-root)
      if [ "$#" -lt 2 ]; then
        usage >&2
        echo "missing value for --log-root" >&2
        exit 2
      fi
      log_root="$2"
      shift 2
      ;;
    --keep-capture)
      keep_capture=true
      shift
      ;;
    -h|--help|help)
      usage
      exit 0
      ;;
    --)
      shift
      break
      ;;
    -*)
      usage >&2
      echo "unknown argument: $1" >&2
      exit 2
      ;;
    *)
      break
      ;;
  esac
done

if [ "$#" -gt 2 ]; then
  usage >&2
  exit 2
fi

candidate="${1:-frontier_pro_v2_guarded}"
shipping="${2:-shipping_pro_search}"
capture_path="$(mktemp /tmp/automove-candidate-metadata-smoke-output.XXXXXX)"

cleanup_capture() {
  if [ "${keep_capture}" = true ]; then
    echo "automove candidate metadata smoke: capture retained at ${capture_path}"
    return
  fi
  rm -f "${capture_path}"
}
trap cleanup_capture EXIT

require_metadata() {
  local role="$1"
  local candidate_id="$2"
  if ! awk \
      -v role="\"role\":\"${role}\"" \
      -v candidate_id="\"id\":\"${candidate_id}\"" \
      '/^AUTOMOVE_SWEEP_CANDIDATE_METADATA / && index($0, role) && index($0, candidate_id) { found = 1; exit } END { exit found ? 0 : 1 }' \
      "${capture_path}"; then
    echo "automove candidate metadata smoke: missing metadata role=${role} id=${candidate_id}" >&2
    return 4
  fi
}

echo "== automove candidate metadata smoke =="
echo "candidate: ${candidate}"
echo "shipping: ${shipping}"
echo "log_root: ${log_root}"

set +e
SMART_EXPERIMENT_LOG_ROOT="${log_root}" \
SMART_PRO_DASHBOARD_PANEL_FILTER="${SMART_PRO_DASHBOARD_PANEL_FILTER:-sampled}" \
SMART_PRO_DASHBOARD_PROMOTION_FAST_FAIL="${SMART_PRO_DASHBOARD_PROMOTION_FAST_FAIL:-true}" \
SMART_PRO_POLICY_MATRIX_PANEL_FILTER="${SMART_PRO_POLICY_MATRIX_PANEL_FILTER:-sampled}" \
SMART_PRO_POLICY_MATRIX_DUEL_FILTER="${SMART_PRO_POLICY_MATRIX_DUEL_FILTER:-vs_shipping_fast}" \
SMART_PRO_POLICY_MATRIX_STATE_LIMIT="${SMART_PRO_POLICY_MATRIX_STATE_LIMIT:-1}" \
SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT="${SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT:-2}" \
  ./scripts/run-automove-structural-scout.sh --outcome-corpus "${candidate}" "${shipping}" \
    | tee "${capture_path}"
scout_status="${PIPESTATUS[0]}"
set -e

if [ "${scout_status}" -ne 0 ]; then
  echo "automove candidate metadata smoke: structural scout failed with status ${scout_status}" >&2
  exit "${scout_status}"
fi

require_metadata "frontier" "${candidate}"
for portfolio_candidate in "${default_portfolio[@]}"; do
  require_metadata "frontier" "${portfolio_candidate}"
done

if ! grep -q '^AUTOMOVE_OUTCOME_CORPUS_POSTPROCESS ' "${capture_path}"; then
  echo "automove candidate metadata smoke: missing outcome-corpus postprocess line" >&2
  exit 4
fi

if ! grep -q '^AUTOMOVE_STRUCTURAL_SCOUT_DECISION ' "${capture_path}"; then
  echo "automove candidate metadata smoke: missing structural scout decision line" >&2
  exit 4
fi

metadata_count="$(awk '/^AUTOMOVE_SWEEP_CANDIDATE_METADATA / { count++ } END { print count + 0 }' "${capture_path}")"
printf 'AUTOMOVE_CANDIDATE_METADATA_SMOKE_RESULT status=passed candidate=%s metadata_lines=%s log_root=%s\n' \
  "${candidate}" \
  "${metadata_count}" \
  "${log_root}"
