#!/usr/bin/env python3
"""Summarize normalized Outcome Corpus V2 JSONL workbench rows."""

import argparse
import json
import re
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
DEFAULT_OVERLAP_FAMILIES = [
    "reply_floor_progress",
    "role",
    "winner_reply_floor",
    "root_safety_detail",
    "safety_progress",
]


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


def parse_family_filter(value):
    if not value:
        return []
    return [item.strip() for item in value.split(",") if item.strip()]


def parse_state_filter(value):
    if value is None:
        return None
    return [item.strip() for item in value.split(",") if item.strip()]


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


def new_family_overlap_group(family):
    return {
        "axis_family": family,
        "candidate_states": set(),
        "baseline_states": set(),
        "no_policy_states": set(),
        "same_outcome_states": set(),
        "candidate_axes": defaultdict(set),
        "baseline_axes": defaultdict(set),
        "no_policy_axes": defaultdict(set),
        "candidate_policies": set(),
        "branches": set(),
        "pairs": set(),
    }


def family_overlap_groups(rows, families):
    wanted = set(families)
    groups = {family: new_family_overlap_group(family) for family in families}
    for row in rows:
        if row.get("row_type") != "policy_axis":
            continue
        family = axis_family(row.get("axis", "none"))
        if family not in wanted:
            continue
        group = groups.setdefault(family, new_family_overlap_group(family))
        cross_state = row.get("cross_budget_state_id") or state_id(row)
        if not cross_state:
            continue
        record_class = row.get("record_class", "")
        axis = row.get("axis", "none")
        if record_class == "candidate_better":
            group["candidate_states"].add(cross_state)
            group["candidate_axes"][axis].add(cross_state)
            if row.get("candidate", ""):
                group["candidate_policies"].add(row.get("candidate", ""))
            if row.get("branch", ""):
                group["branches"].add(row.get("branch", ""))
            if row.get("pair", ""):
                group["pairs"].add(row.get("pair", ""))
        elif record_class == "baseline_better":
            group["baseline_states"].add(cross_state)
            group["baseline_axes"][axis].add(cross_state)
        elif record_class == "no_policy":
            group["no_policy_states"].add(cross_state)
            group["no_policy_axes"][axis].add(cross_state)
        elif record_class == "same_outcome":
            group["same_outcome_states"].add(cross_state)
    return groups


def sorted_axis_state_rows(axis_states, limit=8):
    rows = [
        {"axis": axis, "state_count": len(states), "states": sorted(states)}
        for axis, states in axis_states.items()
    ]
    return sorted(rows, key=lambda row: (-row["state_count"], row["axis"]))[:limit]


def summarize_family_overlap_group(group):
    candidate_states = group["candidate_states"]
    baseline_states = group["baseline_states"]
    no_policy_states = group["no_policy_states"]
    contaminated_states = baseline_states | no_policy_states
    candidate_axis_state_counts = [
        len(states) for states in group["candidate_axes"].values()
    ]
    repeated_candidate_axis_count = sum(
        1 for count in candidate_axis_state_counts if count > 1
    )
    return {
        "axis_family": group["axis_family"],
        "candidate_state_count": len(candidate_states),
        "baseline_state_count": len(baseline_states),
        "no_policy_state_count": len(no_policy_states),
        "same_outcome_state_count": len(group["same_outcome_states"]),
        "candidate_state_ids": sorted(candidate_states),
        "baseline_state_ids": sorted(baseline_states),
        "no_policy_state_ids": sorted(no_policy_states),
        "contaminated_candidate_state_count": len(
            candidate_states & contaminated_states
        ),
        "clean_candidate_state_count": len(candidate_states - contaminated_states),
        "candidate_axis_count": len(group["candidate_axes"]),
        "repeated_candidate_axis_count": repeated_candidate_axis_count,
        "singleton_candidate_axis_count": len(group["candidate_axes"])
        - repeated_candidate_axis_count,
        "candidate_policy_count": len(group["candidate_policies"]),
        "branch_count": len(group["branches"]),
        "pair_count": len(group["pairs"]),
        "top_candidate_axes": sorted_axis_state_rows(group["candidate_axes"]),
        "top_baseline_axes": sorted_axis_state_rows(group["baseline_axes"]),
        "top_no_policy_axes": sorted_axis_state_rows(group["no_policy_axes"]),
    }


def family_overlap_matrix(family_rows):
    rows = []
    for left in family_rows:
        left_states = set(left["candidate_state_ids"])
        for right in family_rows:
            right_states = set(right["candidate_state_ids"])
            if left["axis_family"] >= right["axis_family"]:
                continue
            intersection = left_states & right_states
            union = left_states | right_states
            rows.append(
                {
                    "left_family": left["axis_family"],
                    "right_family": right["axis_family"],
                    "left_candidate_state_count": len(left_states),
                    "right_candidate_state_count": len(right_states),
                    "overlap_state_count": len(intersection),
                    "union_state_count": len(union),
                    "jaccard": round(len(intersection) / len(union), 4)
                    if union
                    else 0.0,
                    "overlap_state_ids": sorted(intersection),
                }
            )
    return sorted(
        rows,
        key=lambda row: (
            -row["overlap_state_count"],
            -row["jaccard"],
            row["left_family"],
            row["right_family"],
        ),
    )


def family_overlap_decision(family_rows, matrix):
    candidate_rows = [row for row in family_rows if row["candidate_state_count"] > 0]
    if not candidate_rows:
        return "no_candidate_family_overlap"
    clean_candidate_states = sum(
        row["clean_candidate_state_count"] for row in candidate_rows
    )
    if (
        clean_candidate_states > 0
        and any(row["repeated_candidate_axis_count"] > 0 for row in candidate_rows)
    ):
        return "inspect_repeated_family_axis"
    max_overlap = max((row["overlap_state_count"] for row in matrix), default=0)
    if max_overlap > 0 and clean_candidate_states > 0:
        return "shared_clean_singleton_state_family"
    if max_overlap > 0:
        return "shared_contaminated_family_states"
    if clean_candidate_states > 0:
        return "independent_clean_singleton_families"
    return "independent_contaminated_singleton_families"


def family_overlap_summary(rows, families):
    groups = family_overlap_groups(rows, families)
    family_rows = [
        summarize_family_overlap_group(groups[family])
        for family in families
    ]
    matrix = family_overlap_matrix(family_rows)
    decision = family_overlap_decision(family_rows, matrix)
    return {
        "families": families,
        "family_rows": family_rows,
        "overlap_matrix": matrix,
        "family_overlap_decision": decision,
        "next_action": {
            "inspect_repeated_family_axis": "inspect_repeated_exact_axis",
            "shared_clean_singleton_state_family": "design_shared_state_feature",
            "shared_contaminated_family_states": "add_discriminator_or_archive_family",
            "independent_clean_singleton_families": "widen_or_archive_singletons",
            "independent_contaminated_singleton_families": "archive_contaminated_singletons",
            "no_candidate_family_overlap": "try_next_family_set",
        }.get(decision, "review"),
    }


def new_state_axis_detail(axis):
    return {
        "axis": axis,
        "axis_family": axis_family(axis),
        "record_count": 0,
        "axis_sources": defaultdict(int),
        "policies": set(),
        "duels": set(),
        "branches": set(),
        "pairs": set(),
        "portfolio_classes": set(),
        "outcomes": set(),
        "first_diff_plies": set(),
    }


def new_state_discriminator_group(state):
    return {
        "cross_budget_state_id": state,
        "panel": "",
        "seed_family": "",
        "repeat": "",
        "opening_index": "",
        "variant": "",
        "candidate_is_white": "",
        "record_classes": defaultdict(int),
        "families_by_class": defaultdict(set),
        "policies_by_class": defaultdict(set),
        "duels_by_class": defaultdict(set),
        "branches_by_class": defaultdict(set),
        "pairs_by_class": defaultdict(set),
        "axes_by_class": defaultdict(dict),
    }


def add_state_axis_detail(group, row):
    record_class = row.get("record_class", "unknown")
    axis = row.get("axis", "none")
    axes = group["axes_by_class"][record_class]
    detail = axes.setdefault(axis, new_state_axis_detail(axis))
    detail["record_count"] += 1
    detail["axis_sources"][row.get("axis_source", "")] += 1
    set_fields = {
        "candidate": "policies",
        "duel": "duels",
        "branch": "branches",
        "pair": "pairs",
        "portfolio_class": "portfolio_classes",
        "outcome": "outcomes",
    }
    for field, set_field in set_fields.items():
        value = row.get(field, "")
        if value:
            detail[set_field].add(value)
    first_diff_ply = row.get("first_diff_ply", "")
    if first_diff_ply not in {"", None}:
        detail["first_diff_plies"].add(first_diff_ply)


def add_state_discriminator_row(groups, row):
    cross_state = row.get("cross_budget_state_id") or state_id(row)
    if not cross_state:
        return
    group = groups.setdefault(cross_state, new_state_discriminator_group(cross_state))
    for field in [
        "panel",
        "seed_family",
        "repeat",
        "opening_index",
        "variant",
        "candidate_is_white",
    ]:
        if group[field] == "":
            group[field] = row.get(field, "")

    record_class = row.get("record_class", "unknown")
    family = axis_family(row.get("axis", "none"))
    group["record_classes"][record_class] += 1
    group["families_by_class"][record_class].add(family)
    for field, set_name in [
        ("candidate", "policies_by_class"),
        ("duel", "duels_by_class"),
        ("branch", "branches_by_class"),
        ("pair", "pairs_by_class"),
    ]:
        value = row.get(field, "")
        if value:
            group[set_name][record_class].add(value)
    add_state_axis_detail(group, row)


def state_discriminator_groups(rows):
    groups = {}
    for row in rows:
        if row.get("row_type") != "policy_axis":
            continue
        add_state_discriminator_row(groups, row)
    return groups


def pipe_join(items):
    return "|".join(str(item) for item in sorted(items, key=str))


def pipe_split(value):
    if not value:
        return set()
    return {item for item in str(value).split("|") if item}


def summarize_state_axis_detail(detail, overlap_classes=None):
    if overlap_classes is None:
        overlap_classes = []
    plies = sorted(detail["first_diff_plies"], key=str)
    return {
        "axis": detail["axis"],
        "axis_family": detail["axis_family"],
        "record_count": detail["record_count"],
        "axis_sources": "|".join(
            item["key"] for item in sorted_count_rows(detail["axis_sources"])
        ),
        "overlap_classes": "|".join(sorted(overlap_classes)),
        "policy_count": len(detail["policies"]),
        "policies": pipe_join(detail["policies"]),
        "duel_count": len(detail["duels"]),
        "duels": pipe_join(detail["duels"]),
        "branch_count": len(detail["branches"]),
        "branches": pipe_join(detail["branches"]),
        "pair_count": len(detail["pairs"]),
        "pairs": pipe_join(detail["pairs"]),
        "portfolio_classes": pipe_join(detail["portfolio_classes"]),
        "outcomes": pipe_join(detail["outcomes"]),
        "first_diff_ply_count": len(plies),
        "first_diff_plies": pipe_join(plies),
    }


def sorted_state_axis_rows(group, record_class, axes, limit):
    class_axes = group["axes_by_class"].get(record_class, {})
    baseline_axes = set(group["axes_by_class"].get("baseline_better", {}))
    no_policy_axes = set(group["axes_by_class"].get("no_policy", {}))
    same_outcome_axes = set(group["axes_by_class"].get("same_outcome", {}))
    rows = []
    for axis in axes:
        detail = class_axes.get(axis)
        if not detail:
            continue
        overlap_classes = []
        if axis in baseline_axes and record_class != "baseline_better":
            overlap_classes.append("baseline_better")
        if axis in no_policy_axes and record_class != "no_policy":
            overlap_classes.append("no_policy")
        if axis in same_outcome_axes and record_class != "same_outcome":
            overlap_classes.append("same_outcome")
        rows.append(summarize_state_axis_detail(detail, overlap_classes))
    return sorted(
        rows,
        key=lambda row: (
            -row["record_count"],
            row["axis_family"],
            row["axis"],
        ),
    )[:limit]


def clean_state_axis(detail):
    return (
        len(detail["policies"]) <= 1
        and len(detail["branches"]) <= 1
        and len(detail["pairs"]) <= 1
    )


def default_state_targets_from_family_overlap(family_summary):
    state_family_counts = defaultdict(int)
    contaminated_family_counts = defaultdict(int)
    for row in family_summary.get("family_rows", []):
        baseline_or_no_policy = set(row["baseline_state_ids"]) | set(
            row["no_policy_state_ids"]
        )
        for state in row["candidate_state_ids"]:
            state_family_counts[state] += 1
            if state in baseline_or_no_policy:
                contaminated_family_counts[state] += 1
    shared_contaminated = [
        state
        for state, count in state_family_counts.items()
        if count > 1 and contaminated_family_counts.get(state, 0) > 0
    ]
    if shared_contaminated:
        return sorted(
            shared_contaminated,
            key=lambda state: (
                -state_family_counts[state],
                -contaminated_family_counts[state],
                state,
            ),
        )
    shared = [state for state, count in state_family_counts.items() if count > 1]
    if shared:
        return sorted(shared, key=lambda state: (-state_family_counts[state], state))
    return sorted(state_family_counts)


def state_discriminator_row_decision(row):
    if not row["present"]:
        return "missing_state"
    if row["candidate_axis_count"] == 0:
        return "no_candidate_state_axes"
    if row["candidate_unique_axis_count"] == 0:
        return "no_state_discriminator"
    if row["candidate_unique_family_count"] == 0:
        return "no_unique_state_family"
    if row["clean_unique_candidate_axis_count"] > 0:
        return "inspect_state_candidate_axes"
    return "fragmented_state_discriminator"


def summarize_state_discriminator_group(group, state_axis_limit):
    class_axes = group["axes_by_class"]
    candidate_axes = set(class_axes.get("candidate_better", {}))
    baseline_axes = set(class_axes.get("baseline_better", {}))
    no_policy_axes = set(class_axes.get("no_policy", {}))
    same_outcome_axes = set(class_axes.get("same_outcome", {}))
    contaminating_axes = baseline_axes | no_policy_axes
    candidate_unique_axes = candidate_axes - contaminating_axes
    candidate_contaminated_axes = candidate_axes & contaminating_axes
    candidate_same_outcome_axes = candidate_axes & same_outcome_axes
    candidate_details = class_axes.get("candidate_better", {})
    clean_unique_axes = [
        axis
        for axis in candidate_unique_axes
        if clean_state_axis(candidate_details[axis])
    ]

    candidate_families = group["families_by_class"].get("candidate_better", set())
    baseline_families = group["families_by_class"].get("baseline_better", set())
    no_policy_families = group["families_by_class"].get("no_policy", set())
    contaminating_families = baseline_families | no_policy_families
    row = {
        "cross_budget_state_id": group["cross_budget_state_id"],
        "present": True,
        "panel": group["panel"],
        "seed_family": group["seed_family"],
        "repeat": group["repeat"],
        "opening_index": group["opening_index"],
        "variant": group["variant"],
        "candidate_is_white": group["candidate_is_white"],
        "record_class_counts": sorted_count_rows(group["record_classes"]),
        "candidate_axis_count": len(candidate_axes),
        "baseline_axis_count": len(baseline_axes),
        "no_policy_axis_count": len(no_policy_axes),
        "same_outcome_axis_count": len(same_outcome_axes),
        "candidate_unique_axis_count": len(candidate_unique_axes),
        "clean_unique_candidate_axis_count": len(clean_unique_axes),
        "candidate_contaminated_axis_count": len(candidate_contaminated_axes),
        "candidate_baseline_overlap_axis_count": len(candidate_axes & baseline_axes),
        "candidate_no_policy_overlap_axis_count": len(candidate_axes & no_policy_axes),
        "candidate_same_outcome_overlap_axis_count": len(
            candidate_same_outcome_axes
        ),
        "candidate_family_count": len(candidate_families),
        "candidate_unique_family_count": len(
            candidate_families - contaminating_families
        ),
        "candidate_contaminated_family_count": len(
            candidate_families & contaminating_families
        ),
        "candidate_families": pipe_join(candidate_families),
        "baseline_families": pipe_join(baseline_families),
        "no_policy_families": pipe_join(no_policy_families),
        "candidate_unique_families": pipe_join(
            candidate_families - contaminating_families
        ),
        "candidate_contaminated_families": pipe_join(
            candidate_families & contaminating_families
        ),
        "candidate_policy_count": len(
            group["policies_by_class"].get("candidate_better", set())
        ),
        "candidate_policies": pipe_join(
            group["policies_by_class"].get("candidate_better", set())
        ),
        "baseline_policy_count": len(
            group["policies_by_class"].get("baseline_better", set())
        ),
        "baseline_policies": pipe_join(
            group["policies_by_class"].get("baseline_better", set())
        ),
        "no_policy_candidate_count": len(
            group["policies_by_class"].get("no_policy", set())
        ),
        "no_policy_candidates": pipe_join(
            group["policies_by_class"].get("no_policy", set())
        ),
        "candidate_branch_count": len(
            group["branches_by_class"].get("candidate_better", set())
        ),
        "candidate_pair_count": len(
            group["pairs_by_class"].get("candidate_better", set())
        ),
        "top_candidate_unique_axes": sorted_state_axis_rows(
            group, "candidate_better", candidate_unique_axes, state_axis_limit
        ),
        "top_candidate_contaminated_axes": sorted_state_axis_rows(
            group, "candidate_better", candidate_contaminated_axes, state_axis_limit
        ),
        "top_baseline_axes": sorted_state_axis_rows(
            group, "baseline_better", baseline_axes, state_axis_limit
        ),
        "top_no_policy_axes": sorted_state_axis_rows(
            group, "no_policy", no_policy_axes, state_axis_limit
        ),
    }
    row["state_discriminator_decision"] = state_discriminator_row_decision(row)
    return row


def missing_state_discriminator_row(state):
    return {
        "cross_budget_state_id": state,
        "present": False,
        "candidate_axis_count": 0,
        "baseline_axis_count": 0,
        "no_policy_axis_count": 0,
        "same_outcome_axis_count": 0,
        "candidate_unique_axis_count": 0,
        "clean_unique_candidate_axis_count": 0,
        "candidate_contaminated_axis_count": 0,
        "state_discriminator_decision": "missing_state",
    }


def state_discriminator_decision(state_rows):
    if not state_rows:
        return "no_target_states"
    decisions = [row["state_discriminator_decision"] for row in state_rows]
    if all(decision == "missing_state" for decision in decisions):
        return "target_states_missing"
    if "inspect_state_candidate_axes" in decisions:
        return "inspect_state_candidate_axes"
    if "fragmented_state_discriminator" in decisions:
        return "fragmented_state_discriminator"
    if "no_unique_state_family" in decisions:
        return "no_state_family_discriminator"
    if "no_state_discriminator" in decisions:
        return "no_state_discriminator"
    if "no_candidate_state_axes" in decisions:
        return "no_candidate_state_axes"
    return "review_state_discriminator"


def state_discriminator_next_action(decision):
    return {
        "inspect_state_candidate_axes": "inspect_unique_state_axes_before_feature",
        "fragmented_state_discriminator": "archive_or_add_below_policy_feature",
        "no_state_family_discriminator": "archive_current_families_or_add_new_feature_axis",
        "no_state_discriminator": "archive_contaminated_families",
        "no_candidate_state_axes": "try_next_state_set",
        "target_states_missing": "rerun_jsonl_with_target_states",
        "no_target_states": "try_next_family_set",
    }.get(decision, "review")


def state_discriminator_summary(
    rows,
    family_summary,
    target_states=None,
    state_axis_limit=8,
):
    if target_states is None:
        target_states = default_state_targets_from_family_overlap(family_summary)
        target_source = (
            f"family_overlap_{family_summary.get('family_overlap_decision', 'unknown')}"
        )
    else:
        target_source = "cli_states"
    groups = state_discriminator_groups(rows)
    state_rows = [
        summarize_state_discriminator_group(groups[state], state_axis_limit)
        if state in groups
        else missing_state_discriminator_row(state)
        for state in target_states
    ]
    decision = state_discriminator_decision(state_rows)
    return {
        "target_source": target_source,
        "target_state_count": len(target_states),
        "target_state_ids": target_states,
        "state_rows": state_rows,
        "state_discriminator_decision": decision,
        "next_action": state_discriminator_next_action(decision),
    }


def default_token_target_families(state_summary, overlap_families):
    families = set()
    for row in state_summary.get("state_rows", []):
        families.update(pipe_split(row.get("candidate_contaminated_families", "")))
    if families:
        return sorted(families)
    return sorted(overlap_families)


def parse_axis_fields(axis):
    fields = []
    for part in str(axis or "").split():
        if "=" not in part:
            continue
        key, value = part.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key or not value:
            continue
        fields.append((key, value))
    return fields


def axis_value_atoms(value):
    atoms = [
        item
        for item in re.split(r"[:+/,|]+", str(value))
        if item and item != value
    ]
    return sorted(set(atoms))


def axis_tokens(axis):
    tokens = []
    for key, value in parse_axis_fields(axis):
        if key == "axis":
            continue
        tokens.append(
            {
                "token": f"{key}={value}",
                "token_kind": "field_value",
                "field": key,
                "value": value,
            }
        )
        for atom in axis_value_atoms(value):
            tokens.append(
                {
                    "token": f"{key}:atom={atom}",
                    "token_kind": "field_atom",
                    "field": key,
                    "value": atom,
                }
            )
    return tokens


def new_axis_token_group(token, token_kind, field, value):
    return {
        "token": token,
        "token_kind": token_kind,
        "field": field,
        "value": value,
        "record_count": 0,
        "axis_families": set(),
        "axes": set(),
        "class_records": defaultdict(int),
        "class_states": defaultdict(set),
        "class_policies": defaultdict(set),
        "class_duels": defaultdict(set),
        "class_branches": defaultdict(set),
        "class_pairs": defaultdict(set),
    }


def add_axis_token_row(groups, row):
    record_class = row.get("record_class", "unknown")
    state = row.get("cross_budget_state_id") or state_id(row)
    if not state:
        return
    family = axis_family(row.get("axis", "none"))
    for token in axis_tokens(row.get("axis", "none")):
        group = groups.setdefault(
            token["token"],
            new_axis_token_group(
                token["token"],
                token["token_kind"],
                token["field"],
                token["value"],
            ),
        )
        group["record_count"] += 1
        group["axis_families"].add(family)
        group["axes"].add(row.get("axis", "none"))
        group["class_records"][record_class] += 1
        group["class_states"][record_class].add(state)
        for field, set_name in [
            ("candidate", "class_policies"),
            ("duel", "class_duels"),
            ("branch", "class_branches"),
            ("pair", "class_pairs"),
        ]:
            value = row.get(field, "")
            if value:
                group[set_name][record_class].add(value)


def axis_token_groups(rows, target_states, target_families):
    wanted_states = set(target_states)
    wanted_families = set(target_families)
    groups = {}
    for row in rows:
        if row.get("row_type") != "policy_axis":
            continue
        state = row.get("cross_budget_state_id") or state_id(row)
        if state not in wanted_states:
            continue
        family = axis_family(row.get("axis", "none"))
        if family not in wanted_families:
            continue
        add_axis_token_row(groups, row)
    return groups


def summarize_axis_token_group(group):
    row = {
        "token": group["token"],
        "token_kind": group["token_kind"],
        "field": group["field"],
        "value": group["value"],
        "record_count": group["record_count"],
        "axis_family_count": len(group["axis_families"]),
        "axis_families": pipe_join(group["axis_families"]),
        "axis_count": len(group["axes"]),
        "sample_axes": sorted(group["axes"])[:5],
    }
    for record_class in RECORD_CLASSES:
        states = group["class_states"].get(record_class, set())
        row[f"{record_class}_records"] = int(
            group["class_records"].get(record_class, 0)
        )
        row[f"{record_class}_state_count"] = len(states)
        row[f"{record_class}_state_ids"] = sorted(states)
        row[f"{record_class}_policy_count"] = len(
            group["class_policies"].get(record_class, set())
        )
        row[f"{record_class}_policies"] = pipe_join(
            group["class_policies"].get(record_class, set())
        )
        row[f"{record_class}_duel_count"] = len(
            group["class_duels"].get(record_class, set())
        )
        row[f"{record_class}_branch_count"] = len(
            group["class_branches"].get(record_class, set())
        )
        row[f"{record_class}_pair_count"] = len(
            group["class_pairs"].get(record_class, set())
        )
    row["baseline_or_no_policy_state_count"] = len(
        group["class_states"].get("baseline_better", set())
        | group["class_states"].get("no_policy", set())
    )
    row["candidate_clean_from_baseline_no_policy"] = (
        row["candidate_better_state_count"] > 0
        and row["baseline_or_no_policy_state_count"] == 0
    )
    row["candidate_repeated_across_target_states"] = (
        row["candidate_better_state_count"] > 1
    )
    return row


def sort_axis_token_rows(rows):
    return sorted(
        rows,
        key=lambda row: (
            -row["candidate_better_state_count"],
            -row["candidate_better_records"],
            row["same_outcome_state_count"],
            row["candidate_better_policy_count"],
            row["candidate_better_branch_count"],
            row["candidate_better_pair_count"],
            row["token_kind"],
            row["token"],
        ),
    )


def axis_token_decision(clean_repeated, clean_singleton, contaminated):
    if clean_repeated:
        return "inspect_repeated_candidate_tokens"
    if clean_singleton:
        return "singleton_candidate_tokens"
    if contaminated:
        return "no_clean_candidate_tokens"
    return "no_candidate_tokens"


def axis_token_next_action(decision):
    return {
        "inspect_repeated_candidate_tokens": "inspect_token_feature_before_runtime",
        "singleton_candidate_tokens": "archive_or_widen_token_singletons",
        "no_clean_candidate_tokens": "archive_current_families_or_add_new_feature_axis",
        "no_candidate_tokens": "try_next_family_set",
    }.get(decision, "review")


def axis_token_discriminator_summary(
    rows,
    state_summary,
    overlap_families,
    token_limit,
    token_families=None,
):
    target_states = state_summary.get("target_state_ids", [])
    target_families = (
        sorted(token_families)
        if token_families is not None
        else default_token_target_families(state_summary, overlap_families)
    )
    target_family_source = (
        "cli_token_families"
        if token_families is not None
        else "state_contaminated_families"
    )
    groups = axis_token_groups(rows, target_states, target_families)
    token_rows = [
        row
        for row in (summarize_axis_token_group(group) for group in groups.values())
        if row["candidate_better_state_count"] > 0
    ]
    clean_tokens = [
        row
        for row in token_rows
        if row["candidate_clean_from_baseline_no_policy"]
    ]
    clean_repeated = [
        row for row in clean_tokens if row["candidate_repeated_across_target_states"]
    ]
    clean_singleton = [
        row for row in clean_tokens if not row["candidate_repeated_across_target_states"]
    ]
    contaminated = [
        row
        for row in token_rows
        if not row["candidate_clean_from_baseline_no_policy"]
    ]
    decision = axis_token_decision(clean_repeated, clean_singleton, contaminated)
    return {
        "target_source": state_summary.get("target_source", ""),
        "target_state_count": len(target_states),
        "target_state_ids": target_states,
        "target_family_source": target_family_source,
        "target_family_count": len(target_families),
        "target_families": target_families,
        "candidate_token_count": len(token_rows),
        "clean_repeated_candidate_token_count": len(clean_repeated),
        "clean_singleton_candidate_token_count": len(clean_singleton),
        "contaminated_candidate_token_count": len(contaminated),
        "axis_token_decision": decision,
        "next_action": axis_token_next_action(decision),
        "top_clean_repeated_candidate_tokens": sort_axis_token_rows(clean_repeated)[
            :token_limit
        ],
        "top_clean_singleton_candidate_tokens": sort_axis_token_rows(clean_singleton)[
            :token_limit
        ],
        "top_contaminated_candidate_tokens": sort_axis_token_rows(contaminated)[
            :token_limit
        ],
    }


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


def summarize(
    rows,
    limit=12,
    overlap_families=None,
    target_states=None,
    state_axis_limit=None,
    token_families=None,
):
    if overlap_families is None:
        overlap_families = DEFAULT_OVERLAP_FAMILIES
    if state_axis_limit is None:
        state_axis_limit = limit
    axis_summaries = collect_policy_axis_summaries(rows)
    source_rows = source_candidate_rows(rows, axis_summaries)
    blocked_rows = blocked_candidate_rows(rows, axis_summaries)
    decision = workbench_decision(source_rows, blocked_rows)
    family_summary = family_overlap_summary(rows, overlap_families)
    state_summary = state_discriminator_summary(
        rows,
        family_summary,
        target_states=target_states,
        state_axis_limit=state_axis_limit,
    )
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
        "family_overlap": family_summary,
        "state_discriminator": state_summary,
        "axis_token_discriminator": axis_token_discriminator_summary(
            rows,
            state_summary,
            overlap_families,
            token_limit=limit,
            token_families=token_families,
        ),
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
    parser.add_argument(
        "--families",
        default=",".join(DEFAULT_OVERLAP_FAMILIES),
        help=(
            "comma-separated axis families for the family-overlap drilldown "
            f"(default: {','.join(DEFAULT_OVERLAP_FAMILIES)})"
        ),
    )
    parser.add_argument(
        "--states",
        default=None,
        help=(
            "comma-separated cross-budget state ids for the state discriminator "
            "drilldown (default: shared contaminated family-overlap states)"
        ),
    )
    parser.add_argument(
        "--state-axis-limit",
        type=int,
        default=None,
        help="maximum axes per state discriminator section (default: --limit)",
    )
    parser.add_argument(
        "--token-families",
        default=None,
        help=(
            "comma-separated axis families for the token discriminator "
            "(default: candidate-contaminated families from target states)"
        ),
    )
    args = parser.parse_args()

    missing = [str(path) for path in args.jsonl if not path.is_file()]
    if missing:
        raise SystemExit(f"missing JSONL file(s): {', '.join(missing)}")

    digest = summarize(
        parse_jsonl_rows(args.jsonl),
        limit=max(1, args.limit),
        overlap_families=parse_family_filter(args.families),
        target_states=parse_state_filter(args.states),
        state_axis_limit=max(1, args.state_axis_limit)
        if args.state_axis_limit is not None
        else None,
        token_families=parse_family_filter(args.token_families)
        if args.token_families is not None
        else None,
    )
    if args.compact:
        json.dump(digest, sys.stdout, sort_keys=True, separators=(",", ":"))
    else:
        json.dump(digest, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
