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


def stable_id_value(value):
    if isinstance(value, bool):
        return str(value).lower()
    return str(value)


def state_id_from_pairs(pairs):
    return "|".join(f"{field}={stable_id_value(value)}" for field, value in pairs)


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


def floor_bucket(value):
    if value is None:
        return "omitted"
    if value <= -100000:
        return "terminal_bad"
    if value <= -1024:
        return "very_bad"
    if value <= -257:
        return "bad"
    if value <= -1:
        return "slightly_bad"
    if value <= 255:
        return "neutral"
    if value <= 1023:
        return "good"
    return "very_good"


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


def root_step_bucket(value):
    if value <= 0:
        return "none"
    if value == 1:
        return "one"
    if value == 2:
        return "two"
    if value <= 4:
        return "three_four"
    if value <= 12:
        return "five_twelve"
    return "unreachable"


def parse_utility(text):
    return {key: int(value) for key, value in UTILITY_RE.findall(text or "")}


def bool_axis(root, field):
    return str(bool(root.get(field, False))).lower()


def root_axes(root):
    family = root.get("family", "")
    band = rank_band(int(root.get("root_rank", -1)))
    utility = parse_utility(root.get("utility", ""))
    same_turn_window = int(root.get("same_turn_window", 0))
    safe_super_steps = int(root.get("safe_super_steps", 0))
    safe_opp_steps = int(root.get("safe_opp_steps", 0))
    score_path_steps = int(root.get("score_path_steps", 0))
    score_path_bucket = root_step_bucket(score_path_steps)
    safe_super_bucket = root_step_bucket(safe_super_steps)
    safe_opp_bucket = root_step_bucket(safe_opp_steps)
    window = window_bucket(same_turn_window)
    safety_detail = root.get("safety_detail", "")
    if not safety_detail:
        safety_detail = "vulnerable" if bool(root.get("vulnerable", False)) else "safe"
    progress = root.get("progress", "")
    if not progress:
        progress = "spirit_development" if bool(root.get("spirit_development", False)) else "quiet"
    reply_risk = root.get("reply_risk", "unknown")
    reply_bucket = root.get("reply_bucket", "")
    if not reply_bucket:
        reply_bucket = floor_bucket(root.get("reply_floor"))
    followup_bucket = root.get("followup_bucket", "")
    if not followup_bucket:
        followup_bucket = floor_bucket(root.get("followup_floor"))
    advisor_bucket = root.get("advisor_bucket", "unknown")
    path = root.get("path", "unknown")
    axes = [
        f"family={family}",
        f"rank_band={band}",
        f"family_rank={family}|{band}",
        f"same_turn_window={window}",
        f"score_path_steps={score_path_bucket}",
        f"safe_super_steps={safe_super_bucket}",
        f"safe_opp_steps={safe_opp_bucket}",
        f"safe_step_relation={step_relation(safe_super_steps, safe_opp_steps)}",
        f"root_race_shape={window}|score_path_{score_path_bucket}|super_{safe_super_bucket}|opp_{safe_opp_bucket}",
        f"family_rank_race_shape={family}|{band}|{score_path_bucket}|{safe_super_bucket}|{safe_opp_bucket}",
        "mana_score_now="
        f"super_score_{bool_axis(root, 'scores_supermana_this_turn')}|"
        f"opp_score_{bool_axis(root, 'scores_opponent_mana_this_turn')}|"
        f"super_pickup_{bool_axis(root, 'safe_supermana_pickup_now')}|"
        f"opp_pickup_{bool_axis(root, 'safe_opponent_mana_pickup_now')}",
        f"wins_immediately={bool_axis(root, 'wins_immediately')}",
        f"attacks={bool_axis(root, 'attacks')}",
        f"vulnerable={bool_axis(root, 'vulnerable')}",
        f"walk_vulnerable={bool_axis(root, 'walk_vulnerable')}",
        f"mana_handoff={bool_axis(root, 'mana_handoff')}",
        f"roundtrip={bool_axis(root, 'roundtrip')}",
        f"safety_detail={safety_detail}",
        f"progress={progress}",
        f"safety_progress={safety_detail}|{progress}",
        f"spirit_development={bool_axis(root, 'spirit_development')}",
        f"spirit_setup={bool_axis(root, 'spirit_setup')}",
        f"supermana_progress={bool_axis(root, 'supermana_progress')}",
        f"opponent_mana_progress={bool_axis(root, 'opponent_mana_progress')}",
        f"reply_risk={reply_risk}",
        f"reply_bucket={reply_bucket}",
        f"reply_progress={reply_risk}|{progress}",
        f"followup_bucket={followup_bucket}",
        f"followup_progress={followup_bucket}|{progress}",
        f"advisor_bucket={advisor_bucket}",
        f"path={path}",
        f"path_safety={path}|{safety_detail}",
        f"advisor_family={advisor_bucket}|{family}",
        f"family_rank_safety={family}|{band}|{safety_detail}",
        f"family_rank_reply={family}|{band}|{reply_risk}",
        f"family_rank_followup={family}|{band}|{followup_bucket}",
        f"safety_reply_progress={safety_detail}|{reply_risk}|{progress}",
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
                f"{family}|{band}|{window}|"
                f"{signed_bucket(utility.get('drainer_safety', 0))}",
                "family_rank_window_safety_detail="
                f"{family}|{band}|{window}|"
                f"{safety_detail}",
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


ROOT_POOL_ID_FIELDS = [
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


def summary_value(summary, field):
    if field == "source":
        return summary.get(field, "scored_roots")
    return summary.get(field, "")


def root_pool_id(summary):
    return state_id_from_pairs(
        (field, summary_value(summary, field)) for field in ROOT_POOL_ID_FIELDS
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
    data = dict(event["data"])
    data["_source_log"] = event["source_log"]
    data["_source_line"] = event["source_line"]
    if event["event_type"] == "FORCED_ROOT_ORACLE_ROOT":
        group["roots"].append(data)
    elif event["event_type"] == "FORCED_ROOT_ORACLE_LEGAL_ROOT":
        group["legal_roots"].append(data)
    group["source_logs"].add(event["source_log"])


def result_points(result):
    if result == "win":
        return 2
    if result == "draw":
        return 1
    return 0


def root_result_class(root):
    result = root.get("result", "unknown")
    if result in {"win", "draw", "loss"}:
        return result
    return "unknown"


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
            "safety_detail": first_winner.get("safety_detail", ""),
            "progress": first_winner.get("progress", ""),
            "score_path_steps": int(first_winner.get("score_path_steps", 0)),
            "reply_risk": first_winner.get("reply_risk", ""),
            "reply_bucket": first_winner.get("reply_bucket", ""),
            "followup_bucket": first_winner.get("followup_bucket", ""),
            "advisor_bucket": first_winner.get("advisor_bucket", ""),
            "path": first_winner.get("path", ""),
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


def root_field_value(root, field):
    if field == "rank_band":
        return rank_band(int(root.get("root_rank", -1)))
    if field == "score_bucket":
        return score_bucket(int(root.get("score", 0)))
    if field == "safety_detail":
        return root.get("safety_detail") or (
            "vulnerable" if bool(root.get("vulnerable", False)) else "safe"
        )
    if field == "progress":
        return root.get("progress") or (
            "spirit_development"
            if bool(root.get("spirit_development", False))
            else "quiet"
        )
    if field == "reply_bucket":
        return root.get("reply_bucket") or floor_bucket(root.get("reply_floor"))
    if field == "followup_bucket":
        return root.get("followup_bucket") or floor_bucket(root.get("followup_floor"))
    return root.get(field, "")


def root_provenance_items(root):
    family = root_field_value(root, "family")
    rank = root_field_value(root, "rank_band")
    path = root_field_value(root, "path")
    advisor = root_field_value(root, "advisor_bucket")
    safety = root_field_value(root, "safety_detail")
    progress = root_field_value(root, "progress")
    reply = root_field_value(root, "reply_bucket")
    followup = root_field_value(root, "followup_bucket")
    window = window_bucket(int(root.get("same_turn_window", 0)))
    score_path = root_step_bucket(int(root.get("score_path_steps", 0)))
    safe_super = root_step_bucket(int(root.get("safe_super_steps", 0)))
    safe_opp = root_step_bucket(int(root.get("safe_opp_steps", 0)))
    return [
        ("path", path),
        ("advisor_bucket", advisor),
        ("family", family),
        ("rank_band", rank),
        ("score_bucket", root_field_value(root, "score_bucket")),
        ("score_path_steps", score_path),
        ("root_race_shape", f"{window}|{score_path}|{safe_super}|{safe_opp}"),
        ("safety_detail", safety),
        ("progress", progress),
        ("reply_bucket", reply),
        ("followup_bucket", followup),
        ("path_family_rank", f"{path}|{family}|{rank}"),
        ("path_safety_progress", f"{path}|{safety}|{progress}"),
        ("advisor_family_rank", f"{advisor}|{family}|{rank}"),
        ("family_rank_safety", f"{family}|{rank}|{safety}"),
        (
            "family_rank_race_shape",
            f"{family}|{rank}|{score_path}|{safe_super}|{safe_opp}",
        ),
        ("family_rank_reply", f"{family}|{rank}|{reply}"),
        ("family_rank_followup", f"{family}|{rank}|{followup}"),
    ]


def sorted_provenance_rows(rows):
    return sorted(
        rows,
        key=lambda row: (
            -row["winner_label_count"],
            row["nonwinner_root_count"],
            -row["winner_root_count"],
            row["dimension"],
            row["key"],
        ),
    )


def root_pool_provenance_rows(groups):
    rollups = {}
    for group in groups:
        label = group["summary"].get("label", "")
        for root in group["roots"]:
            result = root_result_class(root)
            for dimension, key in root_provenance_items(root):
                row = rollups.setdefault(
                    (dimension, key),
                    {
                        "dimension": dimension,
                        "key": key,
                        "root_count": 0,
                        "winner_root_count": 0,
                        "draw_root_count": 0,
                        "nonwinner_root_count": 0,
                        "winner_labels": set(),
                        "nonwinner_labels": set(),
                    },
                )
                row["root_count"] += 1
                if result == "win":
                    row["winner_root_count"] += 1
                    row["winner_labels"].add(label)
                elif result == "draw":
                    row["draw_root_count"] += 1
                    row["nonwinner_labels"].add(label)
                else:
                    row["nonwinner_root_count"] += 1
                    row["nonwinner_labels"].add(label)

    rows = []
    for row in rollups.values():
        winner_labels = sorted(row["winner_labels"])
        nonwinner_labels = sorted(row["nonwinner_labels"])
        root_count = row["root_count"]
        rows.append(
            {
                "dimension": row["dimension"],
                "key": row["key"],
                "root_count": root_count,
                "winner_root_count": row["winner_root_count"],
                "draw_root_count": row["draw_root_count"],
                "nonwinner_root_count": row["nonwinner_root_count"],
                "winner_label_count": len(winner_labels),
                "winner_labels": winner_labels,
                "nonwinner_label_count": len(nonwinner_labels),
                "nonwinner_labels": nonwinner_labels,
                "winner_precision": round(row["winner_root_count"] / root_count, 4)
                if root_count > 0
                else 0.0,
            }
        )
    return sorted_provenance_rows(rows)


def root_pool_provenance_summary(groups):
    root_count = 0
    printed_all_tested = True
    result_counts = defaultdict(int)
    path_counts = defaultdict(int)
    family_counts = defaultdict(int)
    advisor_counts = defaultdict(int)
    winner_path_counts = defaultdict(int)
    winner_family_counts = defaultdict(int)
    winner_advisor_counts = defaultdict(int)

    for group in groups:
        summary = group["summary"]
        roots = group["roots"]
        root_count += len(roots)
        tested_roots = int(summary.get("tested_roots", 0))
        printed_all_tested = printed_all_tested and len(roots) >= tested_roots
        for root in roots:
            result = root_result_class(root)
            result_counts[result] += 1
            path = root_field_value(root, "path")
            family = root_field_value(root, "family")
            advisor = root_field_value(root, "advisor_bucket")
            path_counts[path] += 1
            family_counts[family] += 1
            advisor_counts[advisor] += 1
            if result == "win":
                winner_path_counts[path] += 1
                winner_family_counts[family] += 1
                winner_advisor_counts[advisor] += 1

    provenance_rows = root_pool_provenance_rows(groups)
    clean_repeated = [
        row
        for row in provenance_rows
        if row["winner_label_count"] >= 2 and row["nonwinner_root_count"] == 0
    ]
    return {
        "pool_count": len(groups),
        "printed_root_count": root_count,
        "printed_all_tested_roots": printed_all_tested,
        "result_counts": sorted_count_rows(result_counts),
        "path_counts": sorted_count_rows(path_counts),
        "family_counts": sorted_count_rows(family_counts),
        "advisor_bucket_counts": sorted_count_rows(advisor_counts),
        "winner_path_counts": sorted_count_rows(winner_path_counts),
        "winner_family_counts": sorted_count_rows(winner_family_counts),
        "winner_advisor_bucket_counts": sorted_count_rows(winner_advisor_counts),
        "clean_repeated_winner_provenance_count": len(clean_repeated),
        "top_clean_repeated_winner_provenance": clean_repeated[:8],
        "top_winner_provenance": provenance_rows[:16],
    }


def promising_repeated_axes(axis_rows, groups_with_wins):
    if groups_with_wins < 2:
        return []
    narrow_dimensions = {
        "advisor_family",
        "family_rank",
        "family_rank_followup",
        "family_rank_reply",
        "family_rank_safety",
        "family_rank_window_safety_detail",
        "family_rank_window_safety",
        "path_safety",
        "safety_reply_progress",
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


def root_pool_metadata(summary):
    return {
        "root_pool_id": root_pool_id(summary),
        "label": summary.get("label", ""),
        "continuation": summary.get("continuation", ""),
        "root_source": summary.get("root_source", ""),
        "opponent_mode": summary.get("opponent_mode", ""),
        "variant": summary.get("variant", ""),
        "active_color": summary.get("active_color", ""),
        "source": summary.get("source", "scored_roots"),
        "max_plies": int(summary.get("max_plies", 0)),
        "start_ply": int(summary.get("start_ply", 0)),
        "rollout_max_plies": int(summary.get("rollout_max_plies", 0)),
    }


def root_id(summary, root):
    return state_id_from_pairs(
        [
            ("root_pool_id", root_pool_id(summary)),
            ("root_rank", root.get("root_rank", "")),
            ("inputs", root.get("inputs", "")),
        ]
    )


def normalized_root_pool_summary_row(group):
    summary = group["summary"]
    tested_roots = int(summary.get("tested_roots", 0))
    return {
        "row_type": "forced_root_pool_summary",
        **root_pool_metadata(summary),
        "source_logs": sorted(group["source_logs"]),
        "tested_roots": tested_roots,
        "wins": int(summary.get("wins", 0)),
        "draws": int(summary.get("draws", 0)),
        "losses": int(summary.get("losses", 0)),
        "printed_roots": len(group["roots"]),
        "printed_legal_roots": len(group["legal_roots"]),
        "printed_all_tested_roots": len(group["roots"]) >= tested_roots,
    }


def normalized_forced_root_row(group, root):
    summary = group["summary"]
    utility = parse_utility(root.get("utility", ""))
    result = root_result_class(root)
    return {
        "row_type": "forced_root_pool_root",
        **root_pool_metadata(summary),
        "source_log": root.get("_source_log", ""),
        "source_line": int(root.get("_source_line", 0)),
        "root_id": root_id(summary, root),
        "result": result,
        "is_winning_root": result == "win",
        "root_rank": int(root.get("root_rank", -1)),
        "rank_band": root_field_value(root, "rank_band"),
        "score": int(root.get("score", 0)),
        "score_bucket": root_field_value(root, "score_bucket"),
        "inputs": root.get("inputs", ""),
        "family": root.get("family", ""),
        "wins_immediately": bool(root.get("wins_immediately", False)),
        "attacks": bool(root.get("attacks", False)),
        "vulnerable": bool(root.get("vulnerable", False)),
        "walk_vulnerable": bool(root.get("walk_vulnerable", False)),
        "mana_handoff": bool(root.get("mana_handoff", False)),
        "roundtrip": bool(root.get("roundtrip", False)),
        "safety_detail": root_field_value(root, "safety_detail"),
        "progress": root_field_value(root, "progress"),
        "spirit_development": bool(root.get("spirit_development", False)),
        "spirit_setup": bool(root.get("spirit_setup", False)),
        "supermana_progress": bool(root.get("supermana_progress", False)),
        "opponent_mana_progress": bool(root.get("opponent_mana_progress", False)),
        "safe_super_steps": int(root.get("safe_super_steps", 0)),
        "safe_opp_steps": int(root.get("safe_opp_steps", 0)),
        "score_path_steps": int(root.get("score_path_steps", 0)),
        "score_path_steps_bucket": root_step_bucket(
            int(root.get("score_path_steps", 0))
        ),
        "safe_super_steps_bucket": root_step_bucket(
            int(root.get("safe_super_steps", 0))
        ),
        "safe_opp_steps_bucket": root_step_bucket(
            int(root.get("safe_opp_steps", 0))
        ),
        "safe_step_relation": step_relation(
            int(root.get("safe_super_steps", 0)),
            int(root.get("safe_opp_steps", 0)),
        ),
        "same_turn_window": int(root.get("same_turn_window", 0)),
        "same_turn_window_bucket": window_bucket(
            int(root.get("same_turn_window", 0))
        ),
        "scores_supermana_this_turn": bool(
            root.get("scores_supermana_this_turn", False)
        ),
        "scores_opponent_mana_this_turn": bool(
            root.get("scores_opponent_mana_this_turn", False)
        ),
        "safe_supermana_pickup_now": bool(
            root.get("safe_supermana_pickup_now", False)
        ),
        "safe_opponent_mana_pickup_now": bool(
            root.get("safe_opponent_mana_pickup_now", False)
        ),
        "reply_floor": int(root.get("reply_floor", 0)),
        "reply_risk": root.get("reply_risk", ""),
        "reply_bucket": root_field_value(root, "reply_bucket"),
        "followup_floor": int(root.get("followup_floor", 0)),
        "followup_bucket": root_field_value(root, "followup_bucket"),
        "advisor": root.get("advisor", ""),
        "advisor_bucket": root.get("advisor_bucket", ""),
        "path": root.get("path", ""),
        "utility": root.get("utility", ""),
        "utility_avoid_immediate_loss": utility.get("avoid_immediate_loss", 0),
        "utility_deny_gain": utility.get("deny_gain", 0),
        "utility_drainer_attack": utility.get("drainer_attack", 0),
        "utility_drainer_safety": utility.get("drainer_safety", 0),
        "utility_score_delta": utility.get("score_delta", 0),
        "utility_eval_score": utility.get("eval_score", 0),
        "final": root.get("final", ""),
    }


def normalized_forced_root_axis_rows(group, root):
    summary = group["summary"]
    result = root_result_class(root)
    base = {
        "row_type": "forced_root_pool_axis",
        **root_pool_metadata(summary),
        "source_log": root.get("_source_log", ""),
        "source_line": int(root.get("_source_line", 0)),
        "root_id": root_id(summary, root),
        "result": result,
        "is_winning_root": result == "win",
        "root_rank": int(root.get("root_rank", -1)),
        "rank_band": root_field_value(root, "rank_band"),
        "score": int(root.get("score", 0)),
        "score_path_steps": int(root.get("score_path_steps", 0)),
        "inputs": root.get("inputs", ""),
        "family": root.get("family", ""),
        "advisor_bucket": root.get("advisor_bucket", ""),
        "path": root.get("path", ""),
    }
    return [
        {
            **base,
            "axis": axis,
            "dimension": axis_dimension(axis),
        }
        for axis in root_axes(root)
    ]


def normalized_legal_root_row(group, root):
    summary = group["summary"]
    result = root_result_class(root)
    return {
        "row_type": "forced_root_legal_root",
        **root_pool_metadata(summary),
        "source_log": root.get("_source_log", ""),
        "source_line": int(root.get("_source_line", 0)),
        "result": result,
        "is_winning_root": result == "win",
        "inputs": root.get("inputs", ""),
        "events": root.get("events", ""),
        "final": root.get("final", ""),
    }


def group_events(events):
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
    return groups, event_counts


def build_jsonl_rows(events):
    groups, _event_counts = group_events(events)
    rows = []
    for group in sorted(groups.values(), key=lambda item: item["key"]):
        rows.append(normalized_root_pool_summary_row(group))
        for root in sorted_roots(group["roots"]):
            rows.append(normalized_forced_root_row(group, root))
            rows.extend(normalized_forced_root_axis_rows(group, root))
        for root in sorted(
            group["legal_roots"], key=lambda item: item.get("inputs", "")
        ):
            rows.append(normalized_legal_root_row(group, root))
    return rows


def write_jsonl_rows(path, rows):
    row_type_counts = defaultdict(int)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            row_type_counts[row.get("row_type", "unknown")] += 1
            json.dump(row, handle, sort_keys=True, separators=(",", ":"))
            handle.write("\n")
    return {
        "path": str(path),
        "rows": sum(row_type_counts.values()),
        "row_type_counts": sorted_count_rows(row_type_counts),
    }


def summarize(events):
    groups, event_counts = group_events(events)
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
        "root_pool_provenance": root_pool_provenance_summary(group_values),
        "promising_repeated_axes": promising_repeated_axes(axis_rows, groups_with_wins)[
            :8
        ],
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
    parser.add_argument(
        "--jsonl-out",
        type=Path,
        help="write normalized forced-root pool provenance rows to this JSONL file",
    )
    parser.add_argument(
        "--jsonl-only",
        action="store_true",
        help="write --jsonl-out without printing the summary digest",
    )
    args = parser.parse_args()

    if args.jsonl_only and not args.jsonl_out:
        raise SystemExit("--jsonl-only requires --jsonl-out")

    missing = [str(path) for path in args.logs if not path.is_file()]
    if missing:
        raise SystemExit(f"missing log file(s): {', '.join(missing)}")

    events = parse_forced_root_oracle_lines(args.logs)
    digest = summarize(events)
    if args.jsonl_out:
        digest["jsonl_export"] = write_jsonl_rows(
            args.jsonl_out,
            build_jsonl_rows(events),
        )
    if args.jsonl_only:
        return
    if args.compact:
        json.dump(digest, sys.stdout, sort_keys=True, separators=(",", ":"))
    else:
        json.dump(digest, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
