#!/usr/bin/env bash

set -euo pipefail

usage() {
    cat <<'EOF' >&2
Usage: ./scripts/clean-process-samples.sh [--dry-run]
EOF
}

dry_run=0
case "${1:-}" in
    "")
        ;;
    --dry-run)
        dry_run=1
        ;;
    *)
        usage
        exit 1
        ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sample_dir="$repo_root/target/experiment-runs/misc/samples"

if [[ ! -d "$sample_dir" ]]; then
    echo "no process sample directory at $sample_dir"
    exit 0
fi

sample_files=()
while IFS= read -r sample_file; do
    sample_files+=("$sample_file")
done < <(find "$sample_dir" -type f -name '*.sample.txt' | sort)

if [[ "${#sample_files[@]}" -eq 0 ]]; then
    echo "no process sample files under $sample_dir"
    exit 0
fi

printf '%s\n' "${sample_files[@]}"

if [[ "$dry_run" -eq 1 ]]; then
    exit 0
fi

rm -f "${sample_files[@]}"
echo "removed ${#sample_files[@]} process sample file(s) from $sample_dir"
