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
  echo "$1" | tr '[:space:]/:' '_' | tr -cd '[:alnum:]_.-'
}

timestamp="$(date +%Y%m%d-%H%M%S)"
safe_name="$(sanitize "$run_name")"
log_dir="${SMART_EXPERIMENT_LOG_DIR:-target/experiment-runs}"
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
echo "log_path=${log_path}" >>"$meta_path"
echo "cmd_path=${cmd_path}" >>"$meta_path"

"$@" >"$log_path" 2>&1
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
echo "  command: $(cat "$cmd_path")"
echo "  log: ${log_path}"
echo "  exit: ${status_path} ($(cat "$status_path"))"
echo "  meta: ${meta_path}"

exit "${exit_code}"
