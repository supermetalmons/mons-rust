#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
DEFAULT_RULES_DIR="rules-tests"
DEFAULT_CHUNKS_DIR="${REPO_DIR}/rules-tests-chunks"
DEFAULT_CHUNK_SIZE=100000
LEGACY_ARCHIVE_PATH="${REPO_DIR}/rules-tests.tar.gz"
LEGACY_ARCHIVE_XZ_PATH="${REPO_DIR}/rules-tests.tar.xz"

tar_mode_from_path() {
  case "$1" in
    *.tar.xz|*.txz) echo "J" ;;
    *.tar.gz|*.tgz) echo "z" ;;
    *.tar) echo "" ;;
    *) echo "J" ;;
  esac
}

print_help() {
  cat <<'EOF'
Generate random unique rules-tests fixtures.

Usage:
  ./scripts/generate-rules-tests.sh [script-options] [generate_rules_tests options]

Script options:
  --chunks-dir <path>  Read/write fixture chunks (default: ./rules-tests-chunks)
  --chunk-size <n>     Fixtures per output chunk when repacking (default: 100000)
  --dir <path>         Write fixtures directly to directory (bypasses chunk mode)
  --help, -h          Show this help message

generate_rules_tests options:
  --target-new <n>

Examples:
  ./scripts/generate-rules-tests.sh --target-new 200
  ./scripts/generate-rules-tests.sh --chunks-dir ./rules-tests-chunks --target-new 50
EOF
}

chunks_dir="${DEFAULT_CHUNKS_DIR}"
chunk_size="${DEFAULT_CHUNK_SIZE}"
rules_dir=""
generator_args=()

while (($# > 0)); do
  case "$1" in
    --chunks-dir)
      shift
      if (($# == 0)); then
        echo "error: --chunks-dir requires a value" >&2
        exit 2
      fi
      chunks_dir="$1"
      ;;
    --chunk-size)
      shift
      if (($# == 0)); then
        echo "error: --chunk-size requires a value" >&2
        exit 2
      fi
      chunk_size="$1"
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
      generator_args+=("$1")
      ;;
  esac
  shift
done

cd "${REPO_DIR}"

if [[ ! "${chunk_size}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: --chunk-size must be a positive integer (got '${chunk_size}')" >&2
  exit 2
fi

if [[ -n "${rules_dir}" ]]; then
  cargo run --quiet --bin generate_rules_tests -- --dir "${rules_dir}" "${generator_args[@]}"
  exit 0
fi

has_target_new="false"
for arg in "${generator_args[@]}"; do
  if [[ "${arg}" == "--target-new" ]]; then
    has_target_new="true"
    break
  fi
done

if [[ "${has_target_new}" != "true" ]]; then
  cat >&2 <<'EOF'
error: chunk mode requires --target-new <n> so generated fixtures can be repacked safely.
hint: use --dir for continuous generation workflows.
EOF
  exit 2
fi

temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/mons-rules-tests-generate.XXXXXX")"
cleanup() {
  if [[ -d "${temp_dir}" ]]; then
    rm -rf "${temp_dir}"
  fi
}
trap cleanup EXIT

chunk_archives=()
if [[ -d "${chunks_dir}" ]]; then
  while IFS= read -r chunk_archive; do
    chunk_archives+=("${chunk_archive}")
  done < <(
    find "${chunks_dir}" -maxdepth 1 -type f \
      \( -name '*.tar.gz' -o -name '*.tgz' -o -name '*.tar.xz' -o -name '*.txz' -o -name '*.tar' \) \
      | LC_ALL=C sort
  )
fi

if ((${#chunk_archives[@]} > 0)); then
  for chunk_archive in "${chunk_archives[@]}"; do
    tar_mode="$(tar_mode_from_path "${chunk_archive}")"
    if [[ -n "${tar_mode}" ]]; then
      tar "-x${tar_mode}f" "${chunk_archive}" -C "${temp_dir}"
    else
      tar -xf "${chunk_archive}" -C "${temp_dir}"
    fi
  done
elif [[ "${chunks_dir}" == "${DEFAULT_CHUNKS_DIR}" && -f "${LEGACY_ARCHIVE_PATH}" ]]; then
  tar -xzf "${LEGACY_ARCHIVE_PATH}" -C "${temp_dir}"
elif [[ "${chunks_dir}" == "${DEFAULT_CHUNKS_DIR}" && -f "${LEGACY_ARCHIVE_XZ_PATH}" ]]; then
  tar -xJf "${LEGACY_ARCHIVE_XZ_PATH}" -C "${temp_dir}"
elif [[ "${chunks_dir}" == "${DEFAULT_CHUNKS_DIR}" && -d "${REPO_DIR}/${DEFAULT_RULES_DIR}" ]]; then
  cp -R "${REPO_DIR}/${DEFAULT_RULES_DIR}" "${temp_dir}/${DEFAULT_RULES_DIR}"
fi

rules_dir="${temp_dir}/${DEFAULT_RULES_DIR}"
mkdir -p "${rules_dir}"

cargo run --quiet --bin generate_rules_tests -- --dir "${rules_dir}" "${generator_args[@]}"
"${SCRIPT_DIR}/pack-rules-tests.sh" \
  --dir "${rules_dir}" \
  --chunks-dir "${chunks_dir}" \
  --chunk-size "${chunk_size}" \
  --compression gz
