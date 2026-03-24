#!/usr/bin/env bash

set -euo pipefail

usage() {
    cat <<'EOF' >&2
Usage: ./scripts/capture-process-sample.sh <pid> [duration_seconds] [label]

Examples:
  ./scripts/capture-process-sample.sh 12345
  ./scripts/capture-process-sample.sh 12345 5 pro_reliability
EOF
}

if [[ $# -lt 1 || $# -gt 3 ]]; then
    usage
    exit 1
fi

pid="$1"
duration="${2:-5}"
label="${3:-}"

if ! [[ "$pid" =~ ^[0-9]+$ ]]; then
    echo "capture-process-sample: pid must be numeric" >&2
    exit 1
fi

if ! [[ "$duration" =~ ^[0-9]+$ ]]; then
    echo "capture-process-sample: duration must be an integer number of seconds" >&2
    exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sample_dir="$repo_root/target/experiment-runs/misc/samples"
mkdir -p "$sample_dir"

timestamp="$(date +%Y%m%d-%H%M%S)"
label_suffix=""
if [[ -n "$label" ]]; then
    safe_label="$(printf '%s' "$label" | tr -cs 'A-Za-z0-9._-' '_')"
    label_suffix="_${safe_label}"
fi

output_path="$sample_dir/${timestamp}_pid${pid}${label_suffix}.sample.txt"

sample "$pid" "$duration" -file "$output_path"

echo "sample written to $output_path"
