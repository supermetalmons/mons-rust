#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage:
  ./scripts/postprocess-automove-outcome-corpus-log.sh <policy-matrix-log> [output-prefix]

Runs the standard Outcome Corpus V2 postprocessors for a logged policy-matrix or
pro-policy-outcome-corpus run. By default, artifacts are written next to the log:

  <log-base>.summary.json
  <log-base>.jsonl
  <log-base>.workbench.json
EOF
}

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  usage >&2
  exit 2
fi

log_path="$1"
output_prefix="${2:-}"

if [ ! -f "${log_path}" ]; then
  echo "missing policy-matrix log: ${log_path}" >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ -z "${output_prefix}" ]; then
  case "${log_path}" in
    *.log)
      output_prefix="${log_path%.log}"
      ;;
    *)
      output_prefix="${log_path}"
      ;;
  esac
fi

summary_path="${output_prefix}.summary.json"
jsonl_path="${output_prefix}.jsonl"
workbench_path="${output_prefix}.workbench.json"

"${repo_root}/scripts/summarize-automove-policy-matrix-log.py" \
  --compact \
  --jsonl-out "${jsonl_path}" \
  "${log_path}" >"${summary_path}"

"${repo_root}/scripts/summarize-automove-outcome-jsonl.py" \
  --compact \
  "${jsonl_path}" >"${workbench_path}"

python3 - "${summary_path}" "${workbench_path}" "${jsonl_path}" <<'PY'
import json
import sys

summary_path, workbench_path, jsonl_path = sys.argv[1:]

with open(summary_path, "r", encoding="utf-8") as handle:
    summary = json.load(handle)
with open(workbench_path, "r", encoding="utf-8") as handle:
    workbench = json.load(handle)

cross_budget = summary.get("cross_budget_axis_summary") or {}
root_pool = workbench.get("pro_v4_root_pool_discriminator") or {}
guarded_delta = workbench.get("pro_v4_root_pool_guarded_delta_discriminator") or {}

digest = {
    "corpus_decision": summary.get("corpus_decision"),
    "corpus_next_action": summary.get("next_action"),
    "source_blocker": summary.get("source_blocker"),
    "route_permission": summary.get("route_permission"),
    "source_candidate_rollups": len(cross_budget.get("source_candidate_rollups") or []),
    "blocked_candidate_rollups": len(cross_budget.get("blocked_candidate_rollups") or []),
    "coverage_gap_entry_count": summary.get("coverage_gap_entry_count"),
    "workbench_decision": workbench.get("workbench_decision"),
    "workbench_next_action": workbench.get("next_action"),
    "workbench_source_permission": workbench.get("source_permission"),
    "source_candidate_axis_count": workbench.get("source_candidate_axis_count"),
    "blocked_candidate_axis_count": workbench.get("blocked_candidate_axis_count"),
    "root_pool_decision": root_pool.get("discriminator_decision"),
    "root_pool_source_permission": root_pool.get("source_permission"),
    "guarded_delta_decision": guarded_delta.get("discriminator_decision"),
    "guarded_delta_source_permission": guarded_delta.get("source_permission"),
}

print("automove outcome-corpus postprocess:")
print(f"  summary: {summary_path}")
print(f"  jsonl: {jsonl_path}")
print(f"  workbench: {workbench_path}")
print("AUTOMOVE_OUTCOME_CORPUS_POSTPROCESS " + json.dumps(digest, sort_keys=True))
PY
