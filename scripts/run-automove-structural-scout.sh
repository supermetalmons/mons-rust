#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage:
  ./scripts/run-automove-structural-scout.sh [--corpus] [--confirm] <sweep-candidate[,candidate...]> [shipping]

Runs the promotion-dashboard panels that should precede any major Pro automove
runtime change. This is diagnostic-only: it does not promote profiles.

Default scout:
  1. pro-promotion-dashboard over canonical sampled variants
  2. pro-promotion-dashboard over active blocker variants
  3. candidate-vs-guarded delta inside each dashboard panel

When the candidate is exactly frontier_pro_v2_guarded, the guarded delta is
skipped by default because it is a redundant self-comparison. Set
SMART_PRO_DASHBOARD_INCLUDE_GUARDED=true to force it.

With --corpus:
  4. pro-policy-corpus over the default structural portfolio, unless
     SMART_PRO_POLICY_CORPUS_PORTFOLIO is set

With --confirm:
  5. all variants profile sweep, repeats=1, games=12

The candidate list must be supported by the pro-promotion-dashboard diagnostic in
./scripts/run-automove-experiment.sh.
EOF
}

confirm=false
corpus=false
default_policy_corpus_portfolio="frontier_pro_v2_guarded,frontier_pro_v3_alternating_white_edge_mana,frontier_pro_v3_white_opening_utility_mana,shipping_pro_search_control,frontier_pro_v2_raw,frontier_pro_v2_no_selected_followup_projection,frontier_pro_v3_full_scored_reply_guard,frontier_pro_v2_no_low_budget_guard"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --corpus)
      corpus=true
      shift
      ;;
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

default_dashboard_include_guarded() {
  if [ "${candidate}" = "frontier_pro_v2_guarded" ]; then
    echo "false"
  else
    echo "true"
  fi
}

run_dashboard() {
  local include_guarded
  include_guarded="${SMART_PRO_DASHBOARD_INCLUDE_GUARDED:-$(default_dashboard_include_guarded)}"
  echo "== automove structural scout: promotion-dashboard =="
  SMART_PRO_DASHBOARD_PANEL_FILTER="${SMART_PRO_DASHBOARD_PANEL_FILTER:-all}" \
  SMART_PRO_DASHBOARD_INCLUDE_GUARDED="${include_guarded}" \
    ./scripts/run-automove-experiment.sh pro-promotion-dashboard "${candidate}" "${shipping}"
}

run_policy_corpus() {
  local portfolio="${SMART_PRO_POLICY_CORPUS_PORTFOLIO:-${default_policy_corpus_portfolio}}"
  echo "== automove structural scout: policy-corpus =="
  SMART_PRO_POLICY_WINNER_PANEL_FILTER="${SMART_PRO_POLICY_WINNER_PANEL_FILTER:-all}" \
  SMART_PRO_POLICY_WINNER_DUEL_FILTER="${SMART_PRO_POLICY_WINNER_DUEL_FILTER:-all}" \
    ./scripts/run-automove-experiment.sh pro-policy-corpus "${portfolio}" "${shipping}"
}

run_profile_sweep_panel() {
  local panel="$1"
  shift
  echo "== automove structural scout: ${panel} =="
  env "$@" ./scripts/run-automove-experiment.sh pro-profile-sweep "${candidate}" "${shipping}"
}

run_dashboard

if [ "${corpus}" = true ]; then
  run_policy_corpus
fi

if [ "${confirm}" = true ]; then
  run_profile_sweep_panel \
    "all-variant-scout" \
    SMART_AUTOMOVE_VARIANT_POLICY=all \
    SMART_AUTOMOVE_VARIANTS= \
    SMART_PRO_SWEEP_REPEATS=1 \
    SMART_PRO_SWEEP_GAMES=12 \
    SMART_PRO_SWEEP_MAX_PLIES=96 \
    SMART_PRO_SWEEP_SEED_TAG=pro_profile_all_variant_scout_v1
fi
