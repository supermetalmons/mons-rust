#!/usr/bin/env bash
set -u

if [ "$#" -lt 3 ]; then
  echo "usage: $0 <run_name> -- <command...>" >&2
  exit 2
fi

run_name="$1"
shift

if [ "$1" != "--" ]; then
  echo "usage: $0 <run_name> -- <command...>" >&2
  exit 2
fi
shift

if [ "$#" -eq 0 ]; then
  echo "usage: $0 <run_name> -- <command...>" >&2
  exit 2
fi

sanitize() {
  printf '%s' "$1" | tr '[:space:]/:,' '_' | tr -cd '[:alnum:]_.-'
}

shorten_component() {
  local value="$1"
  local max_length="${2:-96}"
  if [ -z "${value}" ]; then
    value="run"
  fi
  if [ "${#value}" -le "${max_length}" ]; then
    printf '%s' "${value}"
    return
  fi
  local hash
  hash="$(printf '%s' "${value}" | cksum | awk '{print $1}')"
  local suffix="_${hash}"
  local prefix_length=$((max_length - ${#suffix}))
  printf '%s%s' "${value:0:${prefix_length}}" "${suffix}"
}

frontier="${SMART_EXPERIMENT_FRONTIER:-}"
stage="${SMART_EXPERIMENT_STAGE:-}"
shipping="${SMART_EXPERIMENT_SHIPPING:-}"
target_mode="${SMART_EXPERIMENT_TARGET_MODE:-}"
variant_policy="${SMART_EXPERIMENT_VARIANT_POLICY:-${SMART_AUTOMOVE_VARIANT_POLICY:-}}"
variant_list="${SMART_EXPERIMENT_VARIANTS:-${SMART_AUTOMOVE_VARIANTS:-}}"

timestamp="$(date +%Y%m%d-%H%M%S)"
safe_name="$(shorten_component "$(sanitize "$run_name")" 96)"
if [ -n "${SMART_EXPERIMENT_LOG_DIR:-}" ]; then
  log_dir="${SMART_EXPERIMENT_LOG_DIR}"
  profile_scope="$(basename "${log_dir}")"
else
  log_root="${SMART_EXPERIMENT_LOG_ROOT:-target/experiment-runs}"
  if [ -n "${frontier}" ]; then
    profile_scope="$(shorten_component "$(sanitize "${frontier}")" 96)"
  else
    profile_scope="misc"
  fi
  log_dir="${log_root}/${profile_scope}"
fi
mkdir -p "$log_dir"

base_path="$log_dir/${timestamp}_${safe_name}"
log_path="${base_path}.log"
status_path="${base_path}.exit"
cmd_path="${base_path}.cmd"
meta_path="${base_path}.meta"

printf '%q ' "$@" >"$cmd_path"
echo >>"$cmd_path"

start_epoch="$(date +%s)"
echo "start_epoch=${start_epoch}" >"$meta_path"
echo "run_name=${run_name}" >>"$meta_path"
echo "frontier=${frontier}" >>"$meta_path"
echo "profile_scope=${profile_scope}" >>"$meta_path"
echo "stage=${stage}" >>"$meta_path"
echo "shipping=${shipping}" >>"$meta_path"
echo "target_mode=${target_mode}" >>"$meta_path"
echo "variant_policy=${variant_policy}" >>"$meta_path"
echo "variants=${variant_list}" >>"$meta_path"
echo "log_path=${log_path}" >>"$meta_path"
echo "cmd_path=${cmd_path}" >>"$meta_path"

env "SMART_EXPERIMENT_LOG_DIR=${log_dir}" "$@" >"$log_path" 2>&1
exit_code=$?

end_epoch="$(date +%s)"
{
  echo "${exit_code}"
} >"$status_path"
{
  echo "end_epoch=${end_epoch}"
  echo "duration_seconds=$((end_epoch - start_epoch))"
  echo "exit_code=${exit_code}"
  echo "status_path=${status_path}"
} >>"$meta_path"

echo "experiment run complete"
echo "  run_name: ${run_name}"
echo "  frontier: ${frontier:-<none>}"
echo "  scope: ${profile_scope}"
echo "  stage: ${stage:-<none>}"
echo "  shipping: ${shipping:-<none>}"
echo "  target_mode: ${target_mode:-<none>}"
echo "  variant_policy: ${variant_policy:-<default>}"
echo "  variants: ${variant_list:-<default>}"
echo "  command: $(cat "$cmd_path")"
echo "  log: ${log_path}"
echo "  exit: ${status_path} ($(cat "$status_path"))"
echo "  meta: ${meta_path}"

exit "${exit_code}"
