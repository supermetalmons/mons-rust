#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage:
  ./scripts/cleanup-automove-iteration-artifacts.sh [--dry-run]

Removes scratch artifacts created by automove iteration loops.

Default targets are intentionally narrow:
  /tmp/automove-*
  /private/tmp/automove-*
  /tmp/mons_rust-*.sample.txt
  /private/tmp/mons_rust-*.sample.txt

When /tmp and /private/tmp point at the same directory, the script scans that
scratch root only once.

Use --dry-run to print the matching paths without removing them.
EOF
}

dry_run=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run=true
      shift
      ;;
    -h|--help|help)
      usage
      exit 0
      ;;
    *)
      usage >&2
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

shopt -s nullglob

tmp_roots=(/private/tmp)
if [ ! /tmp -ef /private/tmp ]; then
  tmp_roots+=(/tmp)
fi

paths=()
for root in "${tmp_roots[@]}"; do
  for path in "${root}"/automove-* "${root}"/mons_rust-*.sample.txt; do
    case "${path}" in
      /tmp/automove-*|/private/tmp/automove-*|/tmp/mons_rust-*.sample.txt|/private/tmp/mons_rust-*.sample.txt)
        paths+=("${path}")
        ;;
      *)
        echo "refusing unexpected cleanup target: ${path}" >&2
        exit 3
        ;;
    esac
  done
done

if [ "${#paths[@]}" -eq 0 ]; then
  echo "automove cleanup: no matching scratch artifacts"
  exit 0
fi

if [ "${dry_run}" = true ]; then
  echo "automove cleanup dry-run:"
  printf '  %s\n' "${paths[@]}"
  exit 0
fi

rm -rf -- "${paths[@]}"
echo "automove cleanup: removed ${#paths[@]} scratch artifact(s)"
