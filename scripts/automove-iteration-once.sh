#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: automove-iteration-once.sh [--state-dir <dir>] [--batch-id <id>]
EOF
  exit 2
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"

state_dir="${repo_root}/target/automove-iteration"
batch_id="${AUTOMOVE_ITERATION_BATCH_ID:-$(date +%Y%m%d-%H%M%S)}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --state-dir)
      [ "$#" -ge 2 ] || usage
      state_dir="$2"
      shift 2
      ;;
    --batch-id)
      [ "$#" -ge 2 ] || usage
      batch_id="$2"
      shift 2
      ;;
    *)
      usage
      ;;
  esac
done

codex_bin="${CODEX_BIN:-/Applications/Codex.app/Contents/Resources/codex}"
heartbeat_interval="${AUTOMOVE_ITERATION_HEARTBEAT_SECONDS:-60}"
supervisor_pid="${AUTOMOVE_ITERATION_SUPERVISOR_PID:-$$}"

batches_dir="${state_dir}/batches"
heartbeat_file="${state_dir}/heartbeat.json"
current_batch_file="${state_dir}/current-batch.json"
last_result_file="${state_dir}/last-result.json"
last_error_log="${state_dir}/last-error.log"

mkdir -p "${batches_dir}"

log_path="${batches_dir}/${batch_id}.log"
summary_path="${batches_dir}/${batch_id}.summary.txt"
prompt_path="${batches_dir}/${batch_id}.prompt.txt"

iso_now() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

epoch_now() {
  date +%s
}

write_heartbeat() {
  local phase="$1"
  local now_iso
  local now_epoch
  now_iso="$(iso_now)"
  now_epoch="$(epoch_now)"
  cat >"${heartbeat_file}" <<EOF
{
  "batch_id": "${batch_id}",
  "phase": "${phase}",
  "epoch": ${now_epoch},
  "updated_at": "${now_iso}",
  "supervisor_pid": ${supervisor_pid},
  "worker_pid": $$,
  "log_path": "${log_path}",
  "summary_path": "${summary_path}"
}
EOF
}

write_current_batch() {
  local status="$1"
  local phase="$2"
  local now_iso
  now_iso="$(iso_now)"
  cat >"${current_batch_file}" <<EOF
{
  "batch_id": "${batch_id}",
  "status": "${status}",
  "phase": "${phase}",
  "updated_at": "${now_iso}",
  "log_path": "${log_path}",
  "summary_path": "${summary_path}",
  "prompt_path": "${prompt_path}"
}
EOF
}

write_last_result() {
  local status="$1"
  local exit_code="$2"
  local started_at="$3"
  local finished_at="$4"
  local duration_seconds="$5"
  cat >"${last_result_file}" <<EOF
{
  "batch_id": "${batch_id}",
  "status": "${status}",
  "exit_code": ${exit_code},
  "started_at": "${started_at}",
  "finished_at": "${finished_at}",
  "duration_seconds": ${duration_seconds},
  "log_path": "${log_path}",
  "summary_path": "${summary_path}",
  "prompt_path": "${prompt_path}"
}
EOF
}

build_prompt() {
  cat >"${prompt_path}" <<'EOF'
Continue the continuous automove iteration program in `/Users/ivan/Developer/mons/rust`.

Primary goal:
- make automove stronger by building and using fast, exact, cached path queries for tactical board questions
- especially drainer attackability, drainer pickup-and-score routes, safe supermana pickup/progress, spirit-assisted same-turn score/deny, and cache reuse across the same search turn

Hard constraints:
- start with `docs/automove-experiments.md`, especially Quick Reference
- do not read or scan `rules-tests/`
- use `apply_patch` for manual file edits
- prefer `rg` for search
- experiments run through the documented `cargo test --release --lib ... -- --ignored --nocapture` harness
- keep this batch bounded: pick one narrow hypothesis, implement it, run focused tests first, then fast screen, then progressive duel only if the fast screen passes, then full ladder only if the progressive result is clearly strong
- if the ladder fails on pool, run the pool regression diagnostic and record the weak buckets/opponents
- commit only durable infrastructure, validated diagnostics, or promoted candidates
- update `runtime_current` and docs only after the full promotion gates pass

Batch workflow:
1. Read the current docs and inspect recent evidence under `target/experiment-runs/`.
2. Choose one narrow hypothesis that best advances the exact-path program or addresses known weak buckets with exact-path-informed traces.
3. Implement or tune only that hypothesis.
4. Run focused correctness tests first.
5. Run the documented experiment pipeline as far as the gates justify.
6. Leave a concise summary in the final message with:
   - hypothesis
   - files changed
   - tests run
   - experiment commands and deltas
   - whether anything was promoted
   - commit hash if a commit was created

Always refer back to the user’s original automove request and prefer exact-path correctness over broad weight sweeping.
EOF
}

heartbeat_loop() {
  local child_pid="$1"
  while kill -0 "${child_pid}" 2>/dev/null; do
    write_heartbeat "codex_exec"
    sleep "${heartbeat_interval}"
  done
}

started_at="$(iso_now)"
start_epoch="$(epoch_now)"

build_prompt
write_current_batch "running" "preflight"
write_heartbeat "preflight"

if [ ! -x "${codex_bin}" ]; then
  printf 'codex binary not found or not executable: %s\n' "${codex_bin}" >"${last_error_log}"
  write_current_batch "failed" "missing_codex"
  write_last_result "failed" 127 "${started_at}" "$(iso_now)" 0
  exit 127
fi

: >"${log_path}"
: >"${summary_path}"
: >"${last_error_log}"

write_current_batch "running" "codex_exec"
write_heartbeat "codex_exec"

set +e
"${codex_bin}" exec --full-auto -C "${repo_root}" -o "${summary_path}" - <"${prompt_path}" \
  >"${log_path}" 2>&1 &
child_pid=$!
heartbeat_loop "${child_pid}" &
heartbeat_pid=$!

wait "${child_pid}"
exit_code=$?
kill "${heartbeat_pid}" 2>/dev/null || true
wait "${heartbeat_pid}" 2>/dev/null || true
set -e

finished_at="$(iso_now)"
end_epoch="$(epoch_now)"
duration_seconds=$((end_epoch - start_epoch))

if [ "${exit_code}" -eq 0 ]; then
  write_current_batch "completed" "done"
  write_heartbeat "done"
  : >"${last_error_log}"
  write_last_result "completed" "${exit_code}" "${started_at}" "${finished_at}" "${duration_seconds}"
else
  write_current_batch "failed" "done"
  write_heartbeat "failed"
  tail -n 200 "${log_path}" >"${last_error_log}" || true
  write_last_result "failed" "${exit_code}" "${started_at}" "${finished_at}" "${duration_seconds}"
fi

printf 'batch_id=%s exit_code=%s log=%s summary=%s\n' \
  "${batch_id}" "${exit_code}" "${log_path}" "${summary_path}"

exit "${exit_code}"
