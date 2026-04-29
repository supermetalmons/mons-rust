#!/usr/bin/env python3
"""Summarize forced-root oracle logs into one compact JSON digest."""

import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path


FORCED_ROOT_PREFIX = "FORCED_ROOT_ORACLE_"
UTILITY_RE = re.compile(r"(\w+):\s*(-?\d+)")


def parse_forced_root_oracle_lines(paths):
    events = []
    for path in paths:
        events.extend(parse_forced_root_oracle_log(path))
    return events


def parse_forced_root_oracle_log(path):
    events = []
    with path.open("r", encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, start=1):
            line = line.rstrip("\n")
            if not line.startswith(FORCED_ROOT_PREFIX):
                continue
            try:
                event_type, payload = line.split(" ", 1)
            except ValueError:
                continue
            try:
                data = json.loads(payload)
            except json.JSONDecodeError as error:
                raise SystemExit(
                    f"{path}:{line_number}: invalid JSON after {event_type}: {error}"
                ) from error
            events.append(
                {
                    "event_type": event_type,
                    "source_log": str(path),
                    "source_line": line_number,
                    "data": data,
                }
            )
    return events


def sorted_count_rows(counter):
    return [
        {"key": key, "count": count}
        for key, count in sorted(counter.items(), key=lambda item: (-item[1], item[0]))
    ]


def limited_count_rows(counter, limit=8):
    return sorted_count_rows(counter)[:limit]


def rank_band(rank):
    if rank < 0:
        return "unknown"
    if rank == 0:
        return "rank0"
    if rank == 1:
        return "rank1"
    if rank <= 3:
        return "rank2_3"
    if rank <= 7:
        return "rank4_7"
    return "rank8_plus"


def signed_bucket(value):
    if value > 0:
        return "positive"
    if value < 0:
        return "negative"
    return "zero"


def score_delta_bucket(value):
    if value >= 500:
        return "positive_500_plus"
    if value >= 100:
        return "positive_100_499"
    if value > 0:
        return "positive_1_99"
    if value == 0:
        return "zero"
    if value <= -500:
        return "negative_500_plus"
    if value <= -100:
        return "negative_100_499"
    return "negative_1_99"


def score_bucket(value):
    if value >= 1000:
        return "score_1000_plus"
    if value >= 500:
        return "score_500_999"
    if value >= 0:
        return "score_0_499"
    return "score_negative"


def window_bucket(value):
    if value <= 0:
        return "window0"
    if value == 1:
        return "window1"
    return "window2_plus"


def step_relation(super_steps, opponent_steps):
    delta = super_steps - opponent_steps
    if delta > 0:
        return "super_ahead"
    if delta < 0:
        return "opponent_ahead"
    return "tied"


def parse_utility(text):
    return {key: int(value) for key, value in UTILITY_RE.findall(text or "")}


def root_axes(root):
    family = root.get("family", "")
    band = rank_band(int(root.get("root_rank", -1)))
    utility = parse_utility(root.get("utility", ""))
    same_turn_window = int(root.get("same_turn_window", 0))
    safe_super_steps = int(root.get("safe_super_steps", 0))
    safe_opp_steps = int(root.get("safe_opp_steps", 0))
    axes = [
        f"family={family}",
        f"rank_band={band}",
        f"family_rank={family}|{band}",
        f"same_turn_window={window_bucket(same_turn_window)}",
        f"safe_step_relation={step_relation(safe_super_steps, safe_opp_steps)}",
        f"wins_immediately={str(bool(root.get('wins_immediately', False))).lower()}",
        f"attacks={str(bool(root.get('attacks', False))).lower()}",
        f"vulnerable={str(bool(root.get('vulnerable', False))).lower()}",
        f"spirit_development={str(bool(root.get('spirit_development', False))).lower()}",
        f"spirit_setup={str(bool(root.get('spirit_setup', False))).lower()}",
        f"supermana_progress={str(bool(root.get('supermana_progress', False))).lower()}",
        f"opponent_mana_progress={str(bool(root.get('opponent_mana_progress', False))).lower()}",
    ]
    if utility:
        axes.extend(
            [
                f"utility.avoid_immediate_loss={utility.get('avoid_immediate_loss', 0)}",
                f"utility.deny_gain={utility.get('deny_gain', 0)}",
                f"utility.drainer_attack={utility.get('drainer_attack', 0)}",
                "utility.drainer_safety="
                f"{signed_bucket(utility.get('drainer_safety', 0))}",
                "utility.score_delta="
                f"{score_delta_bucket(utility.get('score_delta', 0))}",
                f"utility.eval_score={score_bucket(utility.get('eval_score', 0))}",
                "family_rank_window_safety="
                f"{family}|{band}|{window_bucket(same_turn_window)}|"
                f"{signed_bucket(utility.get('drainer_safety', 0))}",
            ]
        )
    return axes


def group_key(summary):
    return tuple(
        str(summary.get(field, ""))
        for field in [
            "label",
            "continuation",
            "root_source",
            "opponent_mode",
            "variant",
            "active_color",
            "max_plies",
            "start_ply",
            "rollout_max_plies",
            "source",
        ]
    )


def new_group(summary, event):
    return {
        "key": group_key(summary),
        "source_logs": {event["source_log"]},
        "summary": dict(summary),
        "roots": [],
        "legal_roots": [],
    }


def add_row_to_group(group, event):
    if event["event_type"] == "FORCED_ROOT_ORACLE_ROOT":
        group["roots"].append(event["data"])
    elif event["event_type"] == "FORCED_ROOT_ORACLE_LEGAL_ROOT":
        group["legal_roots"].append(event["data"])
    group["source_logs"].add(event["source_log"])


def result_points(result):
    if result == "win":
        return 2
    if result == "draw":
        return 1
    return 0


def sorted_roots(roots):
    return sorted(
        roots,
        key=lambda root: (
            -result_points(root.get("result", "")),
            int(root.get("root_rank", 10**9)),
            -int(root.get("score", 0)),
            root.get("inputs", ""),
        ),
    )


def summarize_group(group):
    summary = group["summary"]
    roots = sorted_roots(group["roots"])
    legal_roots = group["legal_roots"]
    winning_roots = [root for root in roots if root.get("result") == "win"]
    winner_families = defaultdict(int)
    winner_rank_bands = defaultdict(int)
    winner_axes = defaultdict(int)
    for root in winning_roots:
        winner_families[root.get("family", "")] += 1
        winner_rank_bands[rank_band(int(root.get("root_rank", -1)))] += 1
        for axis in root_axes(root):
            winner_axes[axis] += 1

    first_winner = winning_roots[0] if winning_roots else {}
    return {
        "label": summary.get("label", ""),
        "continuation": summary.get("continuation", ""),
        "root_source": summary.get("root_source", ""),
        "opponent_mode": summary.get("opponent_mode", ""),
        "variant": summary.get("variant", ""),
        "active_color": summary.get("active_color", ""),
        "source_logs": sorted(group["source_logs"]),
        "max_plies": int(summary.get("max_plies", 0)),
        "start_ply": int(summary.get("start_ply", 0)),
        "rollout_max_plies": int(summary.get("rollout_max_plies", 0)),
        "tested_roots": int(summary.get("tested_roots", 0)),
        "wins": int(summary.get("wins", 0)),
        "draws": int(summary.get("draws", 0)),
        "losses": int(summary.get("losses", 0)),
        "printed_roots": len(roots),
        "printed_legal_roots": len(legal_roots),
        "first_winning_root": {
            "root_rank": int(first_winner.get("root_rank", -1)),
            "rank_band": rank_band(int(first_winner.get("root_rank", -1))),
            "score": int(first_winner.get("score", 0)),
            "inputs": first_winner.get("inputs", ""),
            "family": first_winner.get("family", ""),
            "same_turn_window": int(first_winner.get("same_turn_window", 0)),
            "utility": first_winner.get("utility", ""),
        }
        if first_winner
        else {},
        "winner_family_count": len(winner_families),
        "winner_families": sorted_count_rows(winner_families),
        "winner_rank_band_count": len(winner_rank_bands),
        "winner_rank_bands": sorted_count_rows(winner_rank_bands),
        "top_winner_axes": limited_count_rows(winner_axes, 12),
    }


def axis_dimension(axis):
    return axis.split("=", 1)[0]


def root_is_win(root):
    return root.get("result") == "win"


def axis_separation(groups):
    winner_counts = defaultdict(int)
    nonwinner_counts = defaultdict(int)
    winner_labels = defaultdict(set)
    nonwinner_labels = defaultdict(set)
    axis_dimensions = {}
    for group in groups:
        label = group["summary"].get("label", "")
        for root in group["roots"]:
            for axis in root_axes(root):
                if root_is_win(root):
                    winner_counts[axis] += 1
                    winner_labels[axis].add(label)
                else:
                    nonwinner_counts[axis] += 1
                    nonwinner_labels[axis].add(label)
                axis_dimensions[axis] = axis_dimension(axis)

    rows = []
    for axis in set(winner_counts) | set(nonwinner_counts):
        labels = sorted(winner_labels[axis])
        nonwinner_label_rows = sorted(nonwinner_labels[axis])
        winner_count = winner_counts[axis]
        nonwinner_count = nonwinner_counts[axis]
        root_count = winner_count + nonwinner_count
        rows.append(
            {
                "axis": axis,
                "dimension": axis_dimensions.get(axis, ""),
                "root_count": root_count,
                "winner_count": winner_count,
                "nonwinner_count": nonwinner_count,
                "label_count": len(labels),
                "labels": labels,
                "nonwinner_label_count": len(nonwinner_label_rows),
                "nonwinner_labels": nonwinner_label_rows,
                "winner_precision": round(winner_count / root_count, 4)
                if root_count > 0
                else 0.0,
            }
        )
    return sorted(
        rows,
        key=lambda row: (
            -row["label_count"],
            row["nonwinner_count"],
            -row["winner_count"],
            row["axis"],
        ),
    )


def promising_repeated_axes(axis_rows, groups_with_wins):
    if groups_with_wins < 2:
        return []
    narrow_dimensions = {
        "family_rank",
        "family_rank_window_safety",
    }
    return [
        row
        for row in axis_rows
        if row["dimension"] in narrow_dimensions
        and row["label_count"] >= 2
        and row["nonwinner_count"] == 0
    ]


def oracle_decision(group_summaries, axis_rows):
    if not group_summaries:
        return "no_oracle_data"
    groups_with_wins = sum(1 for group in group_summaries if group["wins"] > 0)
    if groups_with_wins == 0:
        return "missing_winning_roots"
    if groups_with_wins < len(group_summaries):
        return "partial_root_coverage"
    if any(
        row["label_count"] == groups_with_wins
        for row in promising_repeated_axes(axis_rows, groups_with_wins)
    ):
        return "inspect_repeated_root_feature"
    return "fragmented_root_features"


def next_action_for_decision(decision):
    if decision == "inspect_repeated_root_feature":
        return "inspect_for_prov4_feature"
    if decision == "missing_winning_roots":
        return "add_root_generation_feature"
    if decision == "partial_root_coverage":
        return "separate_missing_roots_from_ranking"
    if decision == "fragmented_root_features":
        return "return_to_outcome_corpus_feature_extraction"
    return "collect_forced_root_oracle_logs"


def summarize(events):
    groups = {}
    current_group_by_label = {}
    event_counts = defaultdict(int)

    for event in events:
        event_type = event["event_type"]
        data = event["data"]
        event_counts[event_type] += 1
        if event_type == "FORCED_ROOT_ORACLE_SUMMARY":
            key = group_key(data)
            group = groups.setdefault(key, new_group(data, event))
            group["summary"] = dict(data)
            group["source_logs"].add(event["source_log"])
            current_group_by_label[data.get("label", "")] = key
        elif event_type in {
            "FORCED_ROOT_ORACLE_ROOT",
            "FORCED_ROOT_ORACLE_LEGAL_ROOT",
        }:
            label = data.get("label", "")
            key = current_group_by_label.get(label)
            if key is not None:
                add_row_to_group(groups[key], event)

    group_values = list(groups.values())
    group_summaries = sorted(
        [summarize_group(group) for group in group_values],
        key=lambda group: (
            group["label"],
            group["continuation"],
            group["root_source"],
            group["start_ply"],
        ),
    )
    axis_rows = axis_separation(group_values)
    decision = oracle_decision(group_summaries, axis_rows)
    groups_with_wins = sum(1 for group in group_summaries if group["wins"] > 0)

    return {
        "event_counts": dict(sorted(event_counts.items())),
        "summary_count": len(group_summaries),
        "root_coverage": {
            "groups": len(group_summaries),
            "groups_with_wins": groups_with_wins,
            "groups_without_wins": len(group_summaries) - groups_with_wins,
            "tested_roots": sum(group["tested_roots"] for group in group_summaries),
            "wins": sum(group["wins"] for group in group_summaries),
            "draws": sum(group["draws"] for group in group_summaries),
            "losses": sum(group["losses"] for group in group_summaries),
        },
        "oracle_decision": decision,
        "next_action": next_action_for_decision(decision),
        "promising_repeated_axes": promising_repeated_axes(axis_rows, groups_with_wins)[:8],
        "repeated_winner_axes": axis_rows[:24],
        "group_summaries": group_summaries,
    }


def main():
    parser = argparse.ArgumentParser(
        description=(
            "Read one or more forced-root oracle logs and summarize "
            "FORCED_ROOT_ORACLE_* JSON lines into a compact digest."
        )
    )
    parser.add_argument("logs", nargs="+", type=Path)
    parser.add_argument(
        "--compact",
        action="store_true",
        help="emit compact JSON instead of pretty-printed JSON",
    )
    args = parser.parse_args()

    missing = [str(path) for path in args.logs if not path.is_file()]
    if missing:
        raise SystemExit(f"missing log file(s): {', '.join(missing)}")

    digest = summarize(parse_forced_root_oracle_lines(args.logs))
    if args.compact:
        json.dump(digest, sys.stdout, sort_keys=True, separators=(",", ":"))
    else:
        json.dump(digest, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
