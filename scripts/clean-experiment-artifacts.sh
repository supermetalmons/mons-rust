#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: ./scripts/clean-experiment-artifacts.sh [--dry-run] [--logs-only|--stamps-only] [--candidate <id>]

options:
  --dry-run            print removals without deleting anything
  --logs-only          remove experiment logs only
  --stamps-only        remove runtime-preflight stamps only
  --candidate <id>     remove artifacts for a single candidate
  -h, --help, help     show this help
EOF
}

sanitize() {
  printf '%s' "$1" | tr '[:space:]/:' '_' | tr -cd '[:alnum:]_.-'
}

remove_path() {
  local path="$1"
  if [ ! -e "${path}" ]; then
    return
  fi
  found_any=true
  if [ "${dry_run}" = true ]; then
    echo "would remove ${path}"
  else
    rm -rf "${path}"
    echo "removed ${path}"
  fi
}

dry_run=false
clean_logs=true
clean_stamps=true
candidate=""
found_any=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run=true
      ;;
    --logs-only)
      if [ "${clean_stamps}" = false ]; then
        echo "cannot combine --logs-only with --stamps-only" >&2
        exit 2
      fi
      clean_stamps=false
      ;;
    --stamps-only)
      if [ "${clean_logs}" = false ]; then
        echo "cannot combine --logs-only with --stamps-only" >&2
        exit 2
      fi
      clean_logs=false
      ;;
    --candidate)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --candidate" >&2
        exit 2
      fi
      candidate="$2"
      shift
      ;;
    -h|--help|help)
      usage
      exit 0
      ;;
    *)
      usage >&2
      exit 2
      ;;
  esac
  shift
done

if [ -n "${candidate}" ]; then
  safe_candidate="$(sanitize "${candidate}")"
  if [ "${clean_logs}" = true ]; then
    remove_path "target/experiment-runs/${safe_candidate}"
  fi
  if [ "${clean_stamps}" = true ]; then
    remove_path "target/experiment-stamps/runtime_preflight_${safe_candidate}.stamp"
  fi
else
  if [ "${clean_logs}" = true ] && [ "${clean_stamps}" = true ]; then
    remove_path ".DS_Store"
    remove_path "src/.DS_Store"
    remove_path "nohup.out"
  fi
  if [ "${clean_logs}" = true ]; then
    remove_path "target/experiment-runs"
  fi
  if [ "${clean_stamps}" = true ]; then
    remove_path "target/experiment-stamps"
  fi
fi

if [ "${found_any}" = false ]; then
  echo "nothing to clean"
fi
