#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
DEFAULT_RULES_DIR="${REPO_DIR}/rules-tests"
DEFAULT_CHUNKS_DIR="${REPO_DIR}/rules-tests-chunks"
DEFAULT_CHUNK_SIZE=100000
DEFAULT_COMPRESSION="gz"

tar_mode_from_compression() {
  case "$1" in
    gz) echo "z" ;;
    xz) echo "J" ;;
    none) echo "" ;;
    *)
      echo "error: unsupported compression '$1' (expected gz, xz, or none)" >&2
      exit 2
      ;;
  esac
}

archive_extension_for_compression() {
  case "$1" in
    gz) echo "tar.gz" ;;
    xz) echo "tar.xz" ;;
    none) echo "tar" ;;
    *)
      echo "error: unsupported compression '$1' (expected gz, xz, or none)" >&2
      exit 2
      ;;
  esac
}

print_help() {
  cat <<'EOF'
Pack rules-tests fixtures into chunk archives.

Usage:
  ./scripts/pack-rules-tests.sh [options]

Options:
  --dir <path>           Source rules-tests directory (default: ./rules-tests)
  --chunks-dir <path>    Output directory for chunk archives (default: ./rules-tests-chunks)
  --chunk-size <n>       Fixtures per chunk (default: 100000)
  --compression <mode>   Archive compression: gz, xz, or none (default: gz)
  --remove-dir           Remove source directory after successful packing
  --help, -h             Show this help message
EOF
}

rules_dir="${DEFAULT_RULES_DIR}"
chunks_dir="${DEFAULT_CHUNKS_DIR}"
chunk_size="${DEFAULT_CHUNK_SIZE}"
compression="${DEFAULT_COMPRESSION}"
remove_dir="false"

while (($# > 0)); do
  case "$1" in
    --dir)
      shift
      if (($# == 0)); then
        echo "error: --dir requires a value" >&2
        exit 2
      fi
      rules_dir="$1"
      ;;
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
    --compression)
      shift
      if (($# == 0)); then
        echo "error: --compression requires a value" >&2
        exit 2
      fi
      compression="$1"
      ;;
    --remove-dir)
      remove_dir="true"
      ;;
    --help|-h)
      print_help
      exit 0
      ;;
    *)
      echo "error: unknown argument '$1'" >&2
      print_help
      exit 2
      ;;
  esac
  shift
done

if [[ ! "${chunk_size}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: --chunk-size must be a positive integer (got '${chunk_size}')" >&2
  exit 2
fi

if [[ ! -d "${rules_dir}" ]]; then
  echo "error: fixture directory '${rules_dir}' not found" >&2
  exit 1
fi

parent_dir="$(dirname "${rules_dir}")"
base_name="$(basename "${rules_dir}")"

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/mons-rules-pack.XXXXXX")"
names_file="${work_dir}/fixture-names.txt"
find "${rules_dir}" -maxdepth 1 -type f ! -name '.*' -print \
  | awk -F/ '{print $NF}' \
  | LC_ALL=C sort >"${names_file}"

fixture_count="$(wc -l <"${names_file}" | tr -d ' ')"
if ((fixture_count == 0)); then
  rm -rf "${work_dir}"
  echo "error: fixture directory '${rules_dir}' has no fixture files" >&2
  exit 1
fi

mkdir -p "${chunks_dir}"
find "${chunks_dir}" -maxdepth 1 -type f \
  \( -name '*.tar.gz' -o -name '*.tgz' -o -name '*.tar.xz' -o -name '*.txz' -o -name '*.tar' \) \
  -delete

list_file="${work_dir}/current.list"
cleanup() {
  if [[ -d "${work_dir}" ]]; then
    rm -rf "${work_dir}"
  fi
  if [[ -f "${list_file}" ]]; then
    rm -f "${list_file}"
  fi
}
trap cleanup EXIT

archive_ext="$(archive_extension_for_compression "${compression}")"
tar_mode="$(tar_mode_from_compression "${compression}")"
chunk_index=0

offset=0
while IFS= read -r fixture_name; do
  if ((offset % chunk_size == 0)); then
    chunk_index=$((chunk_index + 1))
    : >"${list_file}"
  fi

  printf '%s/%s\n' "${base_name}" "${fixture_name}" >>"${list_file}"
  offset=$((offset + 1))

  if ((offset % chunk_size != 0 && offset != fixture_count)); then
    continue
  fi

  chunk_path="$(printf '%s/chunk-%05d.%s' "${chunks_dir}" "${chunk_index}" "${archive_ext}")"
  if [[ -n "${tar_mode}" ]]; then
    tar "-c${tar_mode}f" "${chunk_path}" -C "${parent_dir}" -T "${list_file}"
  else
    tar -cf "${chunk_path}" -C "${parent_dir}" -T "${list_file}"
  fi
  start=$((offset - chunk_size))
  if ((start < 0)); then
    start=0
  fi
  echo "packed fixtures ${start}..$((offset - 1)) -> ${chunk_path}"
done <"${names_file}"

echo "packed ${fixture_count} fixtures from ${rules_dir} into ${chunk_index} chunk(s) at ${chunks_dir}"

if [[ "${remove_dir}" == "true" ]]; then
  rm -rf "${rules_dir}"
  echo "removed ${rules_dir}"
fi
