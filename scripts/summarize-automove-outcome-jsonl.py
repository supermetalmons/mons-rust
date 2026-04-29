#!/usr/bin/env python3
"""Summarize normalized Outcome Corpus V2 JSONL workbench rows."""

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path


RECORD_CLASSES = [
    "candidate_better",
    "baseline_better",
    "no_policy",
    "same_outcome",
]
NO_SOURCE_STATUSES = {
    "baseline_save_risk",
    "coverage_gap",
    "fragmented_no_source",
    "singleton_candidate",
    "singleton_non_regressing",
}


def parse_jsonl_rows(paths):
    rows = []
    for path in paths:
        with path.open("r", encoding="utf-8") as handle:
            for line_number, line in enumerate(handle, start=1):
                line = line.strip()
                if not line:
                    continue
                try:
                    row = json.loads(line)
                except json.JSONDecodeError as error:
                    raise SystemExit(
                        f"{path}:{line_number}: invalid JSONL row: {error}"
                    ) from error
                row.setdefault("source_jsonl", str(path))
                row.setdefault("source_jsonl_line", line_number)
                rows.append(row)
    return rows


def int_field(row, field):
    return int(row.get(field, 0) or 0)


def sorted_count_rows(counter):
    return [
        {"key": key, "count": count}
        for key, count in sorted(counter.items(), key=lambda item: (-item[1], item[0]))
    ]


def split_pipe(value):
    return [item for item in str(value or "").split("|") if item]


def state_id(row):
    return row.get("state_id") or row.get("cross_budget_state_id") or ""


def axis_family(axis):
    first = str(axis or "none").split(" ", 1)[0]
    if first.startswith("axis="):
        return first[len("axis=") :]
    if "=" in first:
        return first.split("=", 1)[0]
    return first or "none"


def axis_group(axis):
    return {
        "key": axis,
        "axis_family": axis_family(axis),
        "record_count": 0,
        "states": set(),
        "cross_budget_states": set(),
        "class_records": defaultdict(int),
        "class_states": defaultdict(set),
        "axis_sources": defaultdict(int),
        "panels": set(),
        "duels": set(),
        "variants": set(),
        "candidates": set(),
        "branches": set(),
        "pairs": set(),
    }


def collect_policy_axis_groups(rows):
    groups = {}
    for row in rows:
        if row.get("row_type") != "policy_axis":
            continue
        axis = row.get("axis", "none")
        group = groups.setdefault(axis, axis_group(axis))
        record_class = row.get("record_class", "unknown")
        state = state_id(row)
        cross_state = row.get("cross_budget_state_id", "")
        group["record_count"] += 1
        if state:
            group["states"].add(state)
            group["class_states"][record_class].add(state)
        if cross_state:
            group["cross_budget_states"].add(cross_state)
        group["class_records"][record_class] += 1
        group["axis_sources"][row.get("axis_source", "")] += 1
        set_fields = {
            "panel": "panels",
            "duel": "duels",
            "variant": "variants",
            "candidate": "candidates",
            "branch": "branches",
            "pair": "pairs",
        }
        for field, set_field in set_fields.items():
            value = row.get(field, "")
            if value:
                group[set_field].add(value)
    return groups


def summarize_policy_axis_group(group):
    row = {
        "key": group["key"],
        "axis_family": group["axis_family"],
        "record_count": group["record_count"],
        "state_count": len(group["states"]),
        "cross_budget_state_count": len(group["cross_budget_states"]),
        "axis_sources": "|".join(
            item["key"] for item in sorted_count_rows(group["axis_sources"])
        ),
        "panel_count": len(group["panels"]),
        "duel_count": len(group["duels"]),
        "variant_count": len(group["variants"]),
        "candidate_count": len(group["candidates"]),
        "branch_count": len(group["branches"]),
        "pair_count": len(group["pairs"]),
    }
    for record_class in RECORD_CLASSES:
        row[f"{record_class}_records"] = int(
            group["class_records"].get(record_class, 0)
        )
        row[f"{record_class}_states"] = len(
            group["class_states"].get(record_class, set())
        )
    return row


def collect_policy_axis_summaries(rows):
    groups = collect_policy_axis_groups(rows)
    return {
        key: summarize_policy_axis_group(group)
        for key, group in groups.items()
    }


def row_type_counts(rows):
    counts = defaultdict(int)
    for row in rows:
        counts[row.get("row_type", "unknown")] += 1
    return counts


def policy_decision_counts(rows):
    record_class_counts = defaultdict(int)
    portfolio_class_counts = defaultdict(int)
    outcome_counts = defaultdict(int)
    state_ids = set()
    for row in rows:
        if row.get("row_type") != "policy_decision":
            continue
        record_class_counts[row.get("record_class", "unknown")] += 1
        portfolio_class_counts[row.get("portfolio_class", "unknown")] += 1
        outcome_counts[row.get("outcome", "unknown")] += 1
        state = state_id(row)
        if state:
            state_ids.add(state)
    return {
        "record_class_counts": sorted_count_rows(record_class_counts),
        "portfolio_class_counts": sorted_count_rows(portfolio_class_counts),
        "outcome_counts": sorted_count_rows(outcome_counts),
        "state_count": len(state_ids),
    }


def source_permission_for_status(status):
    if str(status).startswith("source_candidate_"):
        return "inspect_for_source"
    if status in NO_SOURCE_STATUSES:
        return "no_source"
    return "postprocess_only"


def blocker_for_rollup(row):
    status = row.get("source_status", "")
    if str(status).startswith("source_candidate_"):
        return "none"
    if int_field(row, "baseline_better_joined_states") > 0:
        return "baseline_save_risk"
    if int_field(row, "no_policy_joined_states") > 0:
        return "coverage_gap"
    if row.get("fragmented_dimensions", ""):
        return "fragmented_no_source"
    if status in {"singleton_candidate", "singleton_non_regressing"}:
        return status
    return status or "unknown"


def fragment_count(row):
    return len(split_pipe(row.get("fragmented_dimensions", "")))


def candidate_bearing_cross_budget_rows(rows):
    return [
        row
        for row in rows
        if row.get("row_type") == "cross_budget_axis_rollup"
        and int_field(row, "candidate_better_joined_states") > 0
    ]


def enrich_cross_budget_rollup(row, axis_summaries):
    axis = row.get("key", "")
    axis_summary = axis_summaries.get(axis, {})
    enriched = {
        "key": axis,
        "axis_family": axis_family(axis),
        "source_status": row.get("source_status", ""),
        "source_permission": source_permission_for_status(row.get("source_status", "")),
        "blocker": blocker_for_rollup(row),
        "candidate_better_joined_states": int_field(
            row, "candidate_better_joined_states"
        ),
        "baseline_better_joined_states": int_field(
            row, "baseline_better_joined_states"
        ),
        "no_policy_joined_states": int_field(row, "no_policy_joined_states"),
        "same_outcome_joined_states": int_field(row, "same_outcome_joined_states"),
        "all_budget_repair_joined_states": int_field(
            row, "all_budget_repair_joined_states"
        ),
        "non_regressing_repair_joined_states": int_field(
            row, "non_regressing_repair_joined_states"
        ),
        "joined_state_count": int_field(row, "joined_state_count"),
        "record_count": int_field(row, "record_count"),
        "duel_count": int_field(row, "duel_count"),
        "candidate_count": int_field(row, "candidate_count"),
        "branch_count": int_field(row, "branch_count"),
        "pair_count": int_field(row, "pair_count"),
        "fragmented_dimensions": row.get("fragmented_dimensions", ""),
        "fragment_count": fragment_count(row),
        "axis_candidate_better_states": int(axis_summary.get("candidate_better_states", 0)),
        "axis_baseline_better_states": int(axis_summary.get("baseline_better_states", 0)),
        "axis_no_policy_states": int(axis_summary.get("no_policy_states", 0)),
        "axis_same_outcome_states": int(axis_summary.get("same_outcome_states", 0)),
        "axis_sources": axis_summary.get("axis_sources", ""),
    }
    return enriched


def source_candidate_rows(rows, axis_summaries):
    return sorted(
        [
            enrich_cross_budget_rollup(row, axis_summaries)
            for row in candidate_bearing_cross_budget_rows(rows)
            if str(row.get("source_status", "")).startswith("source_candidate_")
        ],
        key=lambda row: (
            -row["all_budget_repair_joined_states"],
            -row["non_regressing_repair_joined_states"],
            -row["candidate_better_joined_states"],
            -row["joined_state_count"],
            row["key"],
        ),
    )


def blocked_candidate_rows(rows, axis_summaries):
    return sorted(
        [
            enrich_cross_budget_rollup(row, axis_summaries)
            for row in candidate_bearing_cross_budget_rows(rows)
            if not str(row.get("source_status", "")).startswith("source_candidate_")
        ],
        key=lambda row: (
            -row["candidate_better_joined_states"],
            row["baseline_better_joined_states"] + row["no_policy_joined_states"],
            row["fragment_count"],
            row["source_status"],
            -row["joined_state_count"],
            row["key"],
        ),
    )


def source_status_counts(rows):
    counts = defaultdict(int)
    permissions = defaultdict(int)
    blockers = defaultdict(int)
    for row in rows:
        if row.get("row_type") != "cross_budget_axis_rollup":
            continue
        status = row.get("source_status", "unknown")
        counts[status] += 1
        permissions[source_permission_for_status(status)] += 1
        if int_field(row, "candidate_better_joined_states") > 0:
            blockers[blocker_for_rollup(row)] += 1
    return {
        "source_status_counts": sorted_count_rows(counts),
        "source_permission_counts": sorted_count_rows(permissions),
        "candidate_bearing_blocker_counts": sorted_count_rows(blockers),
    }


def family_rollups(blocked_rows):
    groups = {}
    for row in blocked_rows:
        family = row["axis_family"]
        group = groups.setdefault(
            family,
            {
                "axis_family": family,
                "axis_count": 0,
                "candidate_better_joined_states": 0,
                "baseline_better_joined_states": 0,
                "no_policy_joined_states": 0,
                "fragmented_axis_count": 0,
                "source_statuses": defaultdict(int),
                "blockers": defaultdict(int),
            },
        )
        group["axis_count"] += 1
        group["candidate_better_joined_states"] += row[
            "candidate_better_joined_states"
        ]
        group["baseline_better_joined_states"] += row[
            "baseline_better_joined_states"
        ]
        group["no_policy_joined_states"] += row["no_policy_joined_states"]
        if row["fragmented_dimensions"]:
            group["fragmented_axis_count"] += 1
        group["source_statuses"][row["source_status"]] += 1
        group["blockers"][row["blocker"]] += 1

    rows = []
    for group in groups.values():
        rows.append(
            {
                "axis_family": group["axis_family"],
                "axis_count": group["axis_count"],
                "candidate_better_joined_states": group[
                    "candidate_better_joined_states"
                ],
                "baseline_better_joined_states": group[
                    "baseline_better_joined_states"
                ],
                "no_policy_joined_states": group["no_policy_joined_states"],
                "fragmented_axis_count": group["fragmented_axis_count"],
                "source_status_counts": sorted_count_rows(group["source_statuses"]),
                "blocker_counts": sorted_count_rows(group["blockers"]),
            }
        )
    return sorted(
        rows,
        key=lambda row: (
            -row["candidate_better_joined_states"],
            row["baseline_better_joined_states"] + row["no_policy_joined_states"],
            -row["axis_count"],
            row["axis_family"],
        ),
    )


def workbench_decision(source_rows, blocked_rows):
    if source_rows:
        return "inspect_for_source"
    if blocked_rows:
        if any(row["blocker"] == "fragmented_no_source" for row in blocked_rows):
            return "blocked_candidate_axes"
        return "contaminated_candidate_axes"
    return "no_candidate_axis"


def next_action(decision):
    if decision == "inspect_for_source":
        return "inspect_source_candidate_rows"
    if decision == "blocked_candidate_axes":
        return "design_below_policy_feature_from_blockers"
    if decision == "contaminated_candidate_axes":
        return "archive_or_add_new_feature_axis"
    return "try_next_slice"


def source_permission(decision):
    if decision == "inspect_for_source":
        return "inspect_for_source"
    return "no_source"


def summarize(rows, limit=12):
    axis_summaries = collect_policy_axis_summaries(rows)
    source_rows = source_candidate_rows(rows, axis_summaries)
    blocked_rows = blocked_candidate_rows(rows, axis_summaries)
    decision = workbench_decision(source_rows, blocked_rows)
    return {
        "row_counts": sorted_count_rows(row_type_counts(rows)),
        "policy_decisions": policy_decision_counts(rows),
        **source_status_counts(rows),
        "source_candidate_axis_count": len(source_rows),
        "blocked_candidate_axis_count": len(blocked_rows),
        "workbench_decision": decision,
        "next_action": next_action(decision),
        "source_permission": source_permission(decision),
        "top_source_candidate_axes": source_rows[:limit],
        "top_blocked_candidate_axes": blocked_rows[:limit],
        "blocked_axis_family_rollups": family_rollups(blocked_rows)[:limit],
    }


def main():
    parser = argparse.ArgumentParser(
        description=(
            "Read normalized Outcome Corpus V2 JSONL rows and rank "
            "candidate-bearing axes by source status, contamination, and "
            "fragmentation."
        )
    )
    parser.add_argument("jsonl", nargs="+", type=Path)
    parser.add_argument(
        "--limit",
        type=int,
        default=12,
        help="maximum rows per ranked section",
    )
    parser.add_argument(
        "--compact",
        action="store_true",
        help="emit compact JSON instead of pretty-printed JSON",
    )
    args = parser.parse_args()

    missing = [str(path) for path in args.jsonl if not path.is_file()]
    if missing:
        raise SystemExit(f"missing JSONL file(s): {', '.join(missing)}")

    digest = summarize(parse_jsonl_rows(args.jsonl), limit=max(1, args.limit))
    if args.compact:
        json.dump(digest, sys.stdout, sort_keys=True, separators=(",", ":"))
    else:
        json.dump(digest, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
