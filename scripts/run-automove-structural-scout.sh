#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage:
  ./scripts/run-automove-structural-scout.sh [--confirm] <sweep-candidate[,candidate...]> [shipping]

Runs the profile-sweep panels that should precede any major Pro automove
runtime change. This is diagnostic-only: it does not promote profiles.

Default panels:
  1. canonical sampled variants, repeats=3, games=2
  2. active blocker variants, repeats=1, games=3

With --confirm:
  3. all variants, repeats=1, games=12

The candidate list must be supported by the pro-profile-sweep diagnostic in
./scripts/run-automove-experiment.sh.
EOF
}

confirm=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --confirm)
      confirm=true
      shift
      ;;
    -h|--help|help)
      usage
      exit 0
      ;;
    *)
      break
      ;;
  esac
done

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  usage >&2
  exit 2
fi

candidate="$1"
shipping="${2:-shipping_pro_search}"

run_panel() {
  local panel="$1"
  shift
  echo "== automove structural scout: ${panel} =="
  env "$@" ./scripts/run-automove-experiment.sh pro-profile-sweep "${candidate}" "${shipping}"
}

run_panel \
  "canonical-sampled" \
  SMART_AUTOMOVE_VARIANT_POLICY=sampled \
  SMART_AUTOMOVE_VARIANTS= \
  SMART_PRO_SWEEP_REPEATS=3 \
  SMART_PRO_SWEEP_GAMES=2 \
  SMART_PRO_SWEEP_MAX_PLIES=96 \
  SMART_PRO_SWEEP_SEED_TAG=pro_profile_sweep_v1

run_panel \
  "active-blockers" \
  SMART_AUTOMOVE_VARIANT_POLICY=sampled \
  SMART_AUTOMOVE_VARIANTS=outer_edge_mana_rows,alternating_mana_rows,forward_bridge_mana_rows \
  SMART_PRO_SWEEP_REPEATS=1 \
  SMART_PRO_SWEEP_GAMES=3 \
  SMART_PRO_SWEEP_MAX_PLIES=96 \
  SMART_PRO_SWEEP_SEED_TAG=pro_profile_active_blockers_v1

if [ "${confirm}" = true ]; then
  run_panel \
    "all-variant-scout" \
    SMART_AUTOMOVE_VARIANT_POLICY=all \
    SMART_AUTOMOVE_VARIANTS= \
    SMART_PRO_SWEEP_REPEATS=1 \
    SMART_PRO_SWEEP_GAMES=12 \
    SMART_PRO_SWEEP_MAX_PLIES=96 \
    SMART_PRO_SWEEP_SEED_TAG=pro_profile_all_variant_scout_v1
fi
