#!/usr/bin/env bash
set -euo pipefail

dry_run=false

if [ "$#" -gt 1 ]; then
  echo "usage: ./scripts/clean-experiment-artifacts.sh [--dry-run]" >&2
  exit 2
fi

if [ "$#" -eq 1 ]; then
  case "$1" in
    --dry-run)
      dry_run=true
      ;;
    -h|--help|help)
      echo "usage: ./scripts/clean-experiment-artifacts.sh [--dry-run]"
      exit 0
      ;;
    *)
      echo "usage: ./scripts/clean-experiment-artifacts.sh [--dry-run]" >&2
      exit 2
      ;;
  esac
fi

paths=(
  ".DS_Store"
  "src/.DS_Store"
  "nohup.out"
  "target/experiment-runs"
)

found_any=false
for path in "${paths[@]}"; do
  if [ -e "${path}" ]; then
    found_any=true
    if [ "${dry_run}" = true ]; then
      echo "would remove ${path}"
    else
      rm -rf "${path}"
      echo "removed ${path}"
    fi
  fi
done

if [ "${found_any}" = false ]; then
  echo "nothing to clean"
fi
