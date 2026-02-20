#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
DEFAULT_RULES_DIR="rules-tests"
DEFAULT_CHUNKS_DIR="${REPO_DIR}/rules-tests-chunks"
LEGACY_ARCHIVE_PATH="${REPO_DIR}/rules-tests.tar.gz"

tar_mode_from_path() {
  case "$1" in
    *.tar.xz|*.txz) echo "J" ;;
    *.tar.gz|*.tgz) echo "z" ;;
    *.tar) echo "" ;;
    *) echo "z" ;;
  esac
}

print_help() {
  cat <<'EOF'
Run rules-tests fixtures against Mons game logic.

Usage:
  ./scripts/run-rules-tests.sh [script-options] [rules_tests options]

Script options:
  --chunks-dir <path>  Fixture chunk archive directory (default: rules-tests-chunks)
  --archive <path>     Single fixture archive path (legacy fallback)
  --dir <path>         Fixture directory (bypasses archive extraction)
  --help, -h           Show this help message

rules_tests options:
  --limit <n>
  --log <path>
  --verbose

Examples:
  ./scripts/run-rules-tests.sh --limit 200
  ./scripts/run-rules-tests.sh --chunks-dir ./rules-tests-chunks --limit 1000 --verbose
EOF
}

extract_archive_all() {
  local archive_path="$1"
  local destination="$2"
  local mode
  mode="$(tar_mode_from_path "${archive_path}")"
  if [[ -n "${mode}" ]]; then
    tar "-x${mode}f" "${archive_path}" -C "${destination}"
  else
    tar -xf "${archive_path}" -C "${destination}"
  fi
}

extract_archive_first_n() {
  local archive_path="$1"
  local n="$2"
  local destination="$3"
  local list_file="$4"
  local mode
  mode="$(tar_mode_from_path "${archive_path}")"

  if [[ -n "${mode}" ]]; then
    tar "-t${mode}f" "${archive_path}" \
      | awk -v limit="${n}" '
          BEGIN { count = 0 }
          {
            if ($0 ~ /\/$/) { next }
            print $0
            count++
            if (count >= limit) { exit }
          }
        ' >"${list_file}"
  else
    tar -tf "${archive_path}" \
      | awk -v limit="${n}" '
          BEGIN { count = 0 }
          {
            if ($0 ~ /\/$/) { next }
            print $0
            count++
            if (count >= limit) { exit }
          }
        ' >"${list_file}"
  fi

  if [[ ! -s "${list_file}" ]]; then
    return 1
  fi

  if [[ -n "${mode}" ]]; then
    tar "-x${mode}f" "${archive_path}" -C "${destination}" -T "${list_file}"
  else
    tar -xf "${archive_path}" -C "${destination}" -T "${list_file}"
  fi
  return 0
}

count_archive_files() {
  local archive_path="$1"
  local mode
  mode="$(tar_mode_from_path "${archive_path}")"
  if [[ -n "${mode}" ]]; then
    tar "-t${mode}f" "${archive_path}" | awk 'BEGIN { c=0 } $0 !~ /\/$/ { c++ } END { print c }'
  else
    tar -tf "${archive_path}" | awk 'BEGIN { c=0 } $0 !~ /\/$/ { c++ } END { print c }'
  fi
}

resolve_extracted_rules_dir() {
  local extracted_root="$1"
  if [[ -d "${extracted_root}/${DEFAULT_RULES_DIR}" ]]; then
    echo "${extracted_root}/${DEFAULT_RULES_DIR}"
    return
  fi

  top_dirs=()
  while IFS= read -r top_dir; do
    top_dirs+=("${top_dir}")
  done < <(find "${extracted_root}" -mindepth 1 -maxdepth 1 -type d | LC_ALL=C sort)
  if ((${#top_dirs[@]} == 1)); then
    echo "${top_dirs[0]}"
    return
  fi

  echo "${extracted_root}"
}

archive_path=""
chunks_dir="${DEFAULT_CHUNKS_DIR}"
rules_dir=""
runner_args=()

while (($# > 0)); do
  case "$1" in
    --archive)
      shift
      if (($# == 0)); then
        echo "error: --archive requires a value" >&2
        exit 2
      fi
      archive_path="$1"
      ;;
    --chunks-dir)
      shift
      if (($# == 0)); then
        echo "error: --chunks-dir requires a value" >&2
        exit 2
      fi
      chunks_dir="$1"
      ;;
    --dir)
      shift
      if (($# == 0)); then
        echo "error: --dir requires a value" >&2
        exit 2
      fi
      rules_dir="$1"
      ;;
    --help|-h)
      print_help
      exit 0
      ;;
    *)
      runner_args+=("$1")
      ;;
  esac
  shift
done

limit_value=""
for ((i=0; i<${#runner_args[@]}; i++)); do
  if [[ "${runner_args[$i]}" == "--limit" && $((i + 1)) -lt ${#runner_args[@]} ]]; then
    limit_value="${runner_args[$((i + 1))]}"
  fi
done

if [[ -n "${limit_value}" && ! "${limit_value}" =~ ^[0-9]+$ ]]; then
  echo "error: --limit must be a non-negative integer (got '${limit_value}')" >&2
  exit 2
fi

cd "${REPO_DIR}"

temp_dir=""
cleanup() {
  if [[ -n "${temp_dir}" && -d "${temp_dir}" ]]; then
    cleanup_started_at="$(date +%s)"
    echo "🧹 Cleaning up extracted fixtures at ${temp_dir}..."
    rm -rf "${temp_dir}"
    cleanup_elapsed="$(( $(date +%s) - cleanup_started_at ))"
    echo "🧹 Cleanup finished in ${cleanup_elapsed}s"
  fi
}
trap cleanup EXIT

if [[ -z "${rules_dir}" ]]; then
  if [[ -n "${archive_path}" ]]; then
    if [[ ! -f "${archive_path}" ]]; then
      echo "error: archive `${archive_path}` not found" >&2
      exit 1
    fi
    temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/mons-rules-tests.XXXXXX")"
    if [[ -n "${limit_value}" ]]; then
      list_file="${temp_dir}/extract.list"
      if ! extract_archive_first_n "${archive_path}" "${limit_value}" "${temp_dir}" "${list_file}"; then
        echo "error: archive `${archive_path}` has no fixture files" >&2
        exit 1
      fi
    else
      extract_archive_all "${archive_path}" "${temp_dir}"
    fi
    rules_dir="$(resolve_extracted_rules_dir "${temp_dir}")"
  elif [[ -d "${chunks_dir}" ]]; then
    chunk_archives=()
    while IFS= read -r chunk_archive; do
      chunk_archives+=("${chunk_archive}")
    done < <(
      find "${chunks_dir}" -maxdepth 1 -type f \
        \( -name '*.tar.gz' -o -name '*.tgz' -o -name '*.tar.xz' -o -name '*.txz' -o -name '*.tar' \) \
        | LC_ALL=C sort
    )
    if ((${#chunk_archives[@]} > 0)); then
      temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/mons-rules-tests.XXXXXX")"
      extracted=0
      for chunk_archive in "${chunk_archives[@]}"; do
        if [[ -n "${limit_value}" ]]; then
          remaining=$((limit_value - extracted))
          if ((remaining <= 0)); then
            break
          fi
          chunk_count="$(count_archive_files "${chunk_archive}")"
          if ((chunk_count <= remaining)); then
            extract_archive_all "${chunk_archive}" "${temp_dir}"
            extracted=$((extracted + chunk_count))
          else
            list_file="${temp_dir}/extract.list"
            extract_archive_first_n "${chunk_archive}" "${remaining}" "${temp_dir}" "${list_file}"
            extracted=$((extracted + remaining))
            break
          fi
        else
          extract_archive_all "${chunk_archive}" "${temp_dir}"
        fi
      done
      rules_dir="$(resolve_extracted_rules_dir "${temp_dir}")"
    fi
  fi

  if [[ -z "${rules_dir}" && -f "${LEGACY_ARCHIVE_PATH}" ]]; then
    temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/mons-rules-tests.XXXXXX")"
    if [[ -n "${limit_value}" ]]; then
      list_file="${temp_dir}/extract.list"
      if ! extract_archive_first_n "${LEGACY_ARCHIVE_PATH}" "${limit_value}" "${temp_dir}" "${list_file}"; then
        echo "error: archive `${LEGACY_ARCHIVE_PATH}` has no fixture files" >&2
        exit 1
      fi
    else
      extract_archive_all "${LEGACY_ARCHIVE_PATH}" "${temp_dir}"
    fi
    rules_dir="$(resolve_extracted_rules_dir "${temp_dir}")"
  fi

  if [[ -z "${rules_dir}" && -d "${DEFAULT_RULES_DIR}" ]]; then
    rules_dir="${DEFAULT_RULES_DIR}"
  fi

  if [[ -z "${rules_dir}" ]]; then
    echo "error: no fixtures found (expected --dir, --chunks-dir, or ${LEGACY_ARCHIVE_PATH})" >&2
    exit 1
  fi
fi

if ((${#runner_args[@]} > 0)); then
  cargo run --quiet --bin rules_tests -- --dir "${rules_dir}" "${runner_args[@]}"
else
  cargo run --quiet --bin rules_tests -- --dir "${rules_dir}"
fi
