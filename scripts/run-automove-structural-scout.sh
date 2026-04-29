#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage:
  ./scripts/run-automove-structural-scout.sh [--corpus] [--outcome-corpus] [--confirm] <sweep-candidate[,candidate...]> [shipping]

Runs the promotion-dashboard panels that should precede any major Pro automove
runtime change. This is diagnostic-only: it does not promote profiles.

When there is no live runtime hypothesis, read docs/automove-major-reset-plan.md
first. In that mode, prefer --outcome-corpus for corpus work or a test-only
ProV4 sweep candidate for architecture work; do not use this scout to justify
another static selector over existing policy labels.

Default scout:
  1. pro-promotion-dashboard over canonical sampled variants
  2. pro-promotion-dashboard over active blocker variants
  3. candidate-vs-guarded delta inside each dashboard panel

When the candidate is exactly frontier_pro_v2_guarded, the guarded delta is
skipped by default because it is a redundant self-comparison. Set
SMART_PRO_DASHBOARD_INCLUDE_GUARDED=true to force it.

With --corpus:
  4. pro-policy-corpus over the default structural portfolio, unless
     SMART_PRO_POLICY_CORPUS_PORTFOLIO is set
     Defaults to SMART_PRO_POLICY_WINNER_STATE_LIMIT=2 and
     SMART_PRO_POLICY_WINNER_CANDIDATE_TRACE_LIMIT=64 unless overridden.

With --outcome-corpus:
  4. pro-policy-outcome-corpus over the default structural portfolio, unless
     SMART_PRO_POLICY_OUTCOME_CORPUS_PORTFOLIO is set
     Defaults to SMART_PRO_POLICY_MATRIX_STATE_LIMIT=2 and
     SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT=6 unless overridden.
     SMART_PRO_POLICY_MATRIX_INCLUDE_PORTFOLIO_MECHANISM_CLASS=true unless
     overridden.
     After a successful run, the log is postprocessed into .summary.json,
     .jsonl, and .workbench.json artifacts unless
     SMART_AUTOMOVE_SCOUT_POSTPROCESS_OUTCOME=false.

With --confirm:
  5. all variants profile sweep, repeats=1, games=12

After all successful stages, the scout prints AUTOMOVE_STRUCTURAL_SCOUT_DECISION
with the dashboard stoplight plus any outcome-corpus postprocess decision.

The candidate list must be supported by the pro-promotion-dashboard diagnostic in
./scripts/run-automove-experiment.sh.
EOF
}

confirm=false
corpus=false
outcome_corpus=false
default_policy_corpus_portfolio="frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,frontier_pro_v3_white_opening_utility_mana,shipping_pro_search_control,frontier_pro_v2_raw,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard,frontier_pro_v2_no_low_budget_guard"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --corpus)
      corpus=true
      shift
      ;;
    --outcome-corpus)
      outcome_corpus=true
      shift
      ;;
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

candidate="$1"
shipping="${2:-shipping_pro_search}"
dashboard_log_path=""
outcome_log_path=""
outcome_summary_path=""
outcome_jsonl_path=""
outcome_workbench_path=""

log_path_from_capture() {
  awk '/^  log: / { sub(/^  log: /, ""); print }' "$1" | tail -n 1
}

output_prefix_for_log() {
  case "$1" in
    *.log)
      printf '%s\n' "${1%.log}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

default_dashboard_include_guarded() {
  if [ "${candidate}" = "frontier_pro_v2_guarded" ]; then
    echo "false"
  else
    echo "true"
  fi
}

run_dashboard() {
  local include_guarded
  local promotion_fast_fail
  local capture_path
  local status
  include_guarded="${SMART_PRO_DASHBOARD_INCLUDE_GUARDED:-$(default_dashboard_include_guarded)}"
  promotion_fast_fail="${SMART_PRO_DASHBOARD_PROMOTION_FAST_FAIL:-true}"
  echo "== automove structural scout: promotion-dashboard =="
  capture_path="$(mktemp /tmp/automove-structural-scout-dashboard.XXXXXX)"
  set +e
  SMART_PRO_DASHBOARD_PANEL_FILTER="${SMART_PRO_DASHBOARD_PANEL_FILTER:-all}" \
  SMART_PRO_DASHBOARD_PROMOTION_FAST_FAIL="${promotion_fast_fail}" \
  SMART_PRO_DASHBOARD_INCLUDE_GUARDED="${include_guarded}" \
    ./scripts/run-automove-experiment.sh pro-promotion-dashboard "${candidate}" "${shipping}" \
      | tee "${capture_path}"
  status=$?
  set -e

  dashboard_log_path="$(log_path_from_capture "${capture_path}")"
  rm -f "${capture_path}"
  if [ "${status}" -ne 0 ]; then
    return "${status}"
  fi
  if [ -z "${dashboard_log_path}" ]; then
    echo "automove structural scout: no promotion-dashboard log path found" >&2
    return 3
  fi
}

run_policy_corpus() {
  local portfolio="${SMART_PRO_POLICY_CORPUS_PORTFOLIO:-${default_policy_corpus_portfolio}}"
  echo "== automove structural scout: policy-corpus =="
  SMART_PRO_POLICY_WINNER_PANEL_FILTER="${SMART_PRO_POLICY_WINNER_PANEL_FILTER:-all}" \
  SMART_PRO_POLICY_WINNER_DUEL_FILTER="${SMART_PRO_POLICY_WINNER_DUEL_FILTER:-all}" \
  SMART_PRO_POLICY_WINNER_STATE_LIMIT="${SMART_PRO_POLICY_WINNER_STATE_LIMIT:-2}" \
  SMART_PRO_POLICY_WINNER_CANDIDATE_TRACE_LIMIT="${SMART_PRO_POLICY_WINNER_CANDIDATE_TRACE_LIMIT:-64}" \
    ./scripts/run-automove-experiment.sh pro-policy-corpus "${portfolio}" "${shipping}"
}

run_policy_outcome_corpus() {
  local portfolio="${SMART_PRO_POLICY_OUTCOME_CORPUS_PORTFOLIO:-${default_policy_corpus_portfolio}}"
  local capture_path
  local log_path
  local status
  echo "== automove structural scout: policy-outcome-corpus =="
  capture_path="$(mktemp /tmp/automove-structural-scout-outcome.XXXXXX)"
  set +e
  SMART_PRO_POLICY_MATRIX_PANEL_FILTER="${SMART_PRO_POLICY_MATRIX_PANEL_FILTER:-all}" \
  SMART_PRO_POLICY_MATRIX_DUEL_FILTER="${SMART_PRO_POLICY_MATRIX_DUEL_FILTER:-all}" \
  SMART_PRO_POLICY_MATRIX_STATE_LIMIT="${SMART_PRO_POLICY_MATRIX_STATE_LIMIT:-2}" \
  SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT="${SMART_PRO_POLICY_MATRIX_TOTAL_STATE_LIMIT:-6}" \
  SMART_PRO_POLICY_MATRIX_INCLUDE_PORTFOLIO_MECHANISM_CLASS="${SMART_PRO_POLICY_MATRIX_INCLUDE_PORTFOLIO_MECHANISM_CLASS:-true}" \
    ./scripts/run-automove-experiment.sh pro-policy-outcome-corpus "${portfolio}" "${shipping}" \
      | tee "${capture_path}"
  status=$?
  set -e

  log_path="$(awk '/^  log: / { sub(/^  log: /, ""); print }' "${capture_path}" | tail -n 1)"
  rm -f "${capture_path}"
  if [ "${status}" -ne 0 ]; then
    return "${status}"
  fi
  outcome_log_path="${log_path}"
  if [ "${SMART_AUTOMOVE_SCOUT_POSTPROCESS_OUTCOME:-true}" = "true" ]; then
    if [ -z "${log_path}" ]; then
      echo "automove structural scout: no policy-outcome-corpus log path found" >&2
      return 3
    fi
    local output_prefix
    output_prefix="$(output_prefix_for_log "${log_path}")"
    outcome_summary_path="${output_prefix}.summary.json"
    outcome_jsonl_path="${output_prefix}.jsonl"
    outcome_workbench_path="${output_prefix}.workbench.json"
    ./scripts/postprocess-automove-outcome-corpus-log.sh "${log_path}"
  fi
}

run_profile_sweep_panel() {
  local panel="$1"
  shift
  echo "== automove structural scout: ${panel} =="
  env "$@" ./scripts/run-automove-experiment.sh pro-profile-sweep "${candidate}" "${shipping}"
}

emit_structural_scout_decision() {
  python3 - \
    "${candidate}" \
    "${shipping}" \
    "${dashboard_log_path}" \
    "${outcome_log_path}" \
    "${outcome_summary_path}" \
    "${outcome_workbench_path}" <<'PY'
import json
import os
import sys

candidate, shipping, dashboard_log, outcome_log, summary_path, workbench_path = sys.argv[1:]


def last_json_line(path, prefix):
    if not path or not os.path.isfile(path):
        return None
    found = None
    with open(path, "r", encoding="utf-8") as handle:
        for line in handle:
            if line.startswith(prefix + " "):
                try:
                    found = json.loads(line.split(" ", 1)[1])
                except json.JSONDecodeError:
                    continue
    return found


def load_json(path):
    if not path or not os.path.isfile(path):
        return None
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


dashboard = last_json_line(dashboard_log, "PRO_PROMOTION_DASHBOARD_STOPLIGHT") or {}
summary = load_json(summary_path) or {}
workbench = load_json(workbench_path) or {}
cross_budget = summary.get("cross_budget_axis_summary") or {}

dashboard_label = dashboard.get("label")
if dashboard_label == "promotable_shape":
    promotion_decision = "confirm_before_promotion"
elif dashboard_label in {"not_promising", "cost_blocked"}:
    promotion_decision = "do_not_promote"
elif dashboard_label:
    promotion_decision = "diagnostic_only"
else:
    promotion_decision = "unknown"

route_permission = summary.get("route_permission")
workbench_permission = workbench.get("source_permission")
if not summary and not workbench:
    source_decision = "not_evaluated"
elif route_permission == "no_source" or workbench_permission == "no_source":
    source_decision = "no_runtime_source"
elif route_permission in {"postprocess_only", "fragmented_no_source"}:
    source_decision = "postprocess_only"
elif route_permission:
    source_decision = "inspect_source_candidate"
else:
    source_decision = "unknown"

if source_decision in {"no_runtime_source", "postprocess_only"}:
    scout_decision = "record_no_source"
elif source_decision == "inspect_source_candidate":
    scout_decision = "inspect_source_candidate_before_runtime"
elif promotion_decision == "confirm_before_promotion":
    scout_decision = "run_confirm_before_promotion"
elif promotion_decision == "do_not_promote":
    scout_decision = "record_no_promotion"
else:
    scout_decision = "read_stage_logs"

digest = {
    "candidate": candidate,
    "shipping": shipping,
    "dashboard": {
        "label": dashboard_label,
        "classification": dashboard.get("classification"),
        "panels": dashboard.get("panels"),
        "max_candidate_avg_ms": dashboard.get("max_candidate_avg_ms"),
    },
    "dashboard_log": dashboard_log or None,
    "outcome_log": outcome_log or None,
    "outcome_summary": summary_path or None,
    "outcome_workbench": workbench_path or None,
    "outcome": {
        "corpus_decision": summary.get("corpus_decision"),
        "next_action": summary.get("next_action"),
        "route_permission": route_permission,
        "source_blocker": summary.get("source_blocker"),
        "coverage_gap_entry_count": summary.get("coverage_gap_entry_count"),
        "source_candidate_rollups": len(cross_budget.get("source_candidate_rollups") or []),
        "blocked_candidate_rollups": len(cross_budget.get("blocked_candidate_rollups") or []),
    },
    "workbench": {
        "decision": workbench.get("workbench_decision"),
        "next_action": workbench.get("next_action"),
        "source_permission": workbench_permission,
        "source_candidate_axis_count": workbench.get("source_candidate_axis_count"),
        "blocked_candidate_axis_count": workbench.get("blocked_candidate_axis_count"),
    },
    "promotion_decision": promotion_decision,
    "source_decision": source_decision,
    "scout_decision": scout_decision,
}

print("automove structural scout decision:")
if dashboard_log:
    print(f"  dashboard_log: {dashboard_log}")
if outcome_log:
    print(f"  outcome_log: {outcome_log}")
if summary_path:
    print(f"  outcome_summary: {summary_path}")
if workbench_path:
    print(f"  outcome_workbench: {workbench_path}")
print("AUTOMOVE_STRUCTURAL_SCOUT_DECISION " + json.dumps(digest, sort_keys=True))
PY
}

run_dashboard

if [ "${corpus}" = true ]; then
  run_policy_corpus
fi

if [ "${outcome_corpus}" = true ]; then
  run_policy_outcome_corpus
fi

if [ "${confirm}" = true ]; then
  run_profile_sweep_panel \
    "all-variant-scout" \
    SMART_AUTOMOVE_VARIANT_POLICY=all \
    SMART_AUTOMOVE_VARIANTS= \
    SMART_PRO_SWEEP_REPEATS=1 \
    SMART_PRO_SWEEP_GAMES=12 \
    SMART_PRO_SWEEP_MAX_PLIES=96 \
    SMART_PRO_SWEEP_SEED_TAG=pro_profile_all_variant_scout_v1
fi

emit_structural_scout_decision
