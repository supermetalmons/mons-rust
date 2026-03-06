#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: automove-iteration-supervisor.sh [--watchdog] [--state-dir <dir>]
EOF
  exit 2
}

script_path="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
once_script="${script_dir}/automove-iteration-once.sh"

mode="run"
state_dir="${repo_root}/target/automove-iteration"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --watchdog)
      mode="watchdog"
      shift
      ;;
    --state-dir)
      [ "$#" -ge 2 ] || usage
      state_dir="$2"
      shift 2
      ;;
    *)
      usage
      ;;
  esac
done

heartbeat_stale_seconds="${AUTOMOVE_ITERATION_STALE_SECONDS:-5400}"
failure_pause_seconds="${AUTOMOVE_ITERATION_FAILURE_PAUSE_SECONDS:-30}"
infra_failure_pause_seconds="${AUTOMOVE_ITERATION_INFRA_FAILURE_PAUSE_SECONDS:-300}"
infra_failure_limit="${AUTOMOVE_ITERATION_INFRA_FAILURE_LIMIT:-3}"

lock_file="${state_dir}/lock"
heartbeat_file="${state_dir}/heartbeat.json"
current_batch_file="${state_dir}/current-batch.json"
stop_file="${state_dir}/STOP"
supervisor_log="${state_dir}/supervisor.log"

mkdir -p "${state_dir}"

iso_now() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

epoch_now() {
  date +%s
}

read_lock_field() {
  local key="$1"
  [ -f "${lock_file}" ] || return 1
  sed -n "s/^${key}=//p" "${lock_file}" | head -n 1
}

heartbeat_epoch() {
  [ -f "${heartbeat_file}" ] || return 1
  sed -n 's/.*"epoch":[[:space:]]*\([0-9][0-9]*\).*/\1/p' "${heartbeat_file}" | head -n 1
}

heartbeat_age_seconds() {
  local last_epoch
  last_epoch="$(heartbeat_epoch 2>/dev/null || true)"
  if [ -z "${last_epoch}" ]; then
    echo 999999
    return 0
  fi
  echo $(( $(epoch_now) - last_epoch ))
}

last_result_failure_kind() {
  [ -f "${state_dir}/last-result.json" ] || return 1
  sed -n 's/.*"failure_kind":[[:space:]]*"\([^"]*\)".*/\1/p' \
    "${state_dir}/last-result.json" | head -n 1
}

pid_alive() {
  local pid="$1"
  [ -n "${pid}" ] && kill -0 "${pid}" 2>/dev/null
}

runner_healthy() {
  local pid
  pid="$(read_lock_field pid 2>/dev/null || true)"
  [ -n "${pid}" ] || return 1
  pid_alive "${pid}" || return 1
  [ "$(heartbeat_age_seconds)" -le "${heartbeat_stale_seconds}" ]
}

write_idle_status() {
  local status="$1"
  local phase="$2"
  cat >"${current_batch_file}" <<EOF
{
  "status": "${status}",
  "phase": "${phase}",
  "updated_at": "$(iso_now)"
}
EOF
}

write_supervisor_heartbeat() {
  local phase="$1"
  local batch_id="$2"
  cat >"${heartbeat_file}" <<EOF
{
  "batch_id": "${batch_id}",
  "phase": "${phase}",
  "epoch": $(epoch_now),
  "updated_at": "$(iso_now)",
  "supervisor_pid": $$,
  "worker_pid": 0,
  "log_path": "${supervisor_log}",
  "summary_path": ""
}
EOF
}

cleanup_lock() {
  local lock_pid
  lock_pid="$(read_lock_field pid 2>/dev/null || true)"
  if [ "${lock_pid}" = "$$" ]; then
    rm -f "${lock_file}"
  fi
}

acquire_lock() {
  local lock_contents
  lock_contents="$(cat <<EOF
pid=$$
started_at=$(iso_now)
started_epoch=$(epoch_now)
repo_root=${repo_root}
EOF
)"

  while true; do
    if ( set -o noclobber; printf '%s\n' "${lock_contents}" >"${lock_file}" ) 2>/dev/null; then
      trap cleanup_lock EXIT INT TERM
      return 0
    fi

    if runner_healthy; then
      printf 'healthy supervisor already running: pid=%s heartbeat_age=%ss\n' \
        "$(read_lock_field pid)" "$(heartbeat_age_seconds)"
      return 1
    fi

    take_over_stale_runner
    sleep 1
  done
}

take_over_stale_runner() {
  local stale_pid
  stale_pid="$(read_lock_field pid 2>/dev/null || true)"
  if [ -n "${stale_pid}" ] && pid_alive "${stale_pid}"; then
    kill "${stale_pid}" 2>/dev/null || true
    sleep 5
    if pid_alive "${stale_pid}"; then
      kill -9 "${stale_pid}" 2>/dev/null || true
    fi
  fi
  rm -f "${lock_file}"
}

run_watchdog() {
  if [ -f "${stop_file}" ]; then
    printf 'stop file present, watchdog will not restart the supervisor\n'
    exit 0
  fi

  if runner_healthy; then
    printf 'healthy supervisor pid=%s heartbeat_age=%ss\n' \
      "$(read_lock_field pid)" "$(heartbeat_age_seconds)"
    exit 0
  fi

  if [ -f "${lock_file}" ]; then
    take_over_stale_runner
  fi

  nohup "${script_path}" --state-dir "${state_dir}" >>"${supervisor_log}" 2>&1 &
  new_pid=$!
  sleep 1
  printf 'restarted supervisor pid=%s\n' "${new_pid}"
}

run_supervisor() {
  if [ -f "${stop_file}" ]; then
    write_idle_status "stopped" "stop_file_present"
    printf 'stop file present, supervisor not started\n'
    exit 0
  fi

  acquire_lock || exit 0
  write_idle_status "running" "idle"
  write_supervisor_heartbeat "idle" "idle"

  local batch_counter=0
  local infra_failures=0
  while true; do
    if [ -f "${stop_file}" ]; then
      write_idle_status "stopped" "stop_file_present"
      write_supervisor_heartbeat "stopped" "idle"
      printf 'stop file detected, exiting supervisor\n'
      break
    fi

    batch_counter=$((batch_counter + 1))
    batch_id="$(date +%Y%m%d-%H%M%S)-${batch_counter}"
    write_idle_status "running" "dispatching"
    write_supervisor_heartbeat "dispatching" "${batch_id}"

    if AUTOMOVE_ITERATION_SUPERVISOR_PID="$$" \
      AUTOMOVE_ITERATION_BATCH_ID="${batch_id}" \
      "${once_script}" --state-dir "${state_dir}" --batch-id "${batch_id}"; then
      infra_failures=0
      write_idle_status "running" "between_batches"
      write_supervisor_heartbeat "between_batches" "${batch_id}"
    else
      exit_code=$?
      failure_kind="$(last_result_failure_kind 2>/dev/null || true)"
      write_idle_status "running" "batch_failed"
      write_supervisor_heartbeat "batch_failed" "${batch_id}"
      printf 'batch %s failed with exit_code=%s failure_kind=%s\n' \
        "${batch_id}" "${exit_code}" "${failure_kind:-unknown}" >>"${supervisor_log}"

      if [ "${failure_kind:-}" = "infra" ]; then
        infra_failures=$((infra_failures + 1))
        if [ "${infra_failures}" -ge "${infra_failure_limit}" ]; then
          write_idle_status "stopped" "infra_failure_limit"
          write_supervisor_heartbeat "infra_failure_limit" "${batch_id}"
          printf 'stopping supervisor after %s consecutive infra failures\n' \
            "${infra_failures}" >>"${supervisor_log}"
          break
        fi
        sleep "${infra_failure_pause_seconds}"
      else
        infra_failures=0
        sleep "${failure_pause_seconds}"
      fi
    fi
  done
}

if [ "${mode}" = "watchdog" ]; then
  run_watchdog
else
  run_supervisor
fi
