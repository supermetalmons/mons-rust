#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: automove-direct-batch.sh <candidate_profile> [baseline_profile] [--hypothesis <text>] [--batch-id <id>] [--run-ladder]
EOF
  exit 2
}

if [ "$#" -lt 1 ]; then
  usage
fi

candidate_profile="$1"
shift

baseline_profile="runtime_current"
if [ "$#" -gt 0 ] && [ "${1#--}" = "$1" ]; then
  baseline_profile="$1"
  shift
fi

hypothesis="geometry-first secure carrier oracle"
batch_id="${AUTOMOVE_DIRECT_BATCH_ID:-$(date +%Y%m%d-%H%M%S)}"
run_ladder=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --hypothesis)
      [ "$#" -ge 2 ] || usage
      hypothesis="$2"
      shift 2
      ;;
    --batch-id)
      [ "$#" -ge 2 ] || usage
      batch_id="$2"
      shift 2
      ;;
    --run-ladder)
      run_ladder=1
      shift
      ;;
    *)
      usage
      ;;
  esac
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
batch_dir="${repo_root}/target/automove-direct-batches/${batch_id}"
journal_path="${batch_dir}/journal.jsonl"
summary_path="${batch_dir}/summary.txt"

mkdir -p "${batch_dir}"

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

run_step() {
  local step_name="$1"
  shift
  local log_path="${batch_dir}/${step_name}.log"
  local cmd_string
  cmd_string="$(printf '%q ' "$@")"
  local start_epoch end_epoch duration exit_code
  start_epoch="$(date +%s)"

  set +e
  "$@" >"${log_path}" 2>&1
  exit_code=$?
  set -e

  end_epoch="$(date +%s)"
  duration=$((end_epoch - start_epoch))

  printf '{"step":"%s","exit_code":%s,"duration_seconds":%s,"log":"%s","command":"%s"}\n' \
    "$(json_escape "${step_name}")" \
    "${exit_code}" \
    "${duration}" \
    "$(json_escape "${log_path}")" \
    "$(json_escape "${cmd_string}")" >>"${journal_path}"

  printf '%s exit=%s duration=%ss log=%s\n' \
    "${step_name}" "${exit_code}" "${duration}" "${log_path}" >>"${summary_path}"

  return "${exit_code}"
}

{
  printf 'batch_id=%s\n' "${batch_id}"
  printf 'candidate_profile=%s\n' "${candidate_profile}"
  printf 'baseline_profile=%s\n' "${baseline_profile}"
  printf 'hypothesis=%s\n' "${hypothesis}"
  printf '\n'
} >"${summary_path}"

printf '{"batch_id":"%s","candidate_profile":"%s","baseline_profile":"%s","hypothesis":"%s"}\n' \
  "$(json_escape "${batch_id}")" \
  "$(json_escape "${candidate_profile}")" \
  "$(json_escape "${baseline_profile}")" \
  "$(json_escape "${hypothesis}")" >"${journal_path}"

run_step focused_exact_tests \
  cargo test --release --lib exact_ -- --nocapture

run_step tactical_candidate \
  env SMART_CANDIDATE_PROFILE="${candidate_profile}" \
  cargo test --release --lib smart_automove_tactical_candidate_profile -- --ignored --nocapture

run_step exact_path_probes \
  env SMART_CANDIDATE_PROFILE="${candidate_profile}" \
  cargo test --release --lib smart_automove_exact_path_probe_suite -- --ignored --nocapture

run_step exact_query_speed \
  env SMART_CANDIDATE_PROFILE="${candidate_profile}" \
  cargo test --release --lib smart_automove_pool_exact_query_speed_probe -- --ignored --nocapture

run_step profile_speed \
  env SMART_CANDIDATE_PROFILE="${candidate_profile}" \
  cargo test --release --lib smart_automove_pool_profile_speed_probe -- --ignored --nocapture

if run_step fast_screen \
  env SMART_CANDIDATE_PROFILE="${candidate_profile}" \
      SMART_GATE_BASELINE_PROFILE="${baseline_profile}" \
  cargo test --release --lib smart_automove_pool_fast_screen -- --ignored --nocapture
then
  if run_step progressive_duel \
    env SMART_CANDIDATE_PROFILE="${candidate_profile}" \
        SMART_GATE_BASELINE_PROFILE="${baseline_profile}" \
    cargo test --release --lib smart_automove_pool_progressive_duel -- --ignored --nocapture
  then
    if [ "${run_ladder}" -eq 1 ]; then
      run_step promotion_ladder \
        env SMART_CANDIDATE_PROFILE="${candidate_profile}" \
            SMART_GATE_BASELINE_PROFILE="${baseline_profile}" \
        cargo test --release --lib smart_automove_pool_promotion_ladder -- --ignored --nocapture
    fi
  fi
fi

printf '\nartifacts=%s\n' "${batch_dir}" >>"${summary_path}"
printf '%s\n' "${batch_dir}"
