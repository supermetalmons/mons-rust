#!/usr/bin/env python3
"""Summarize normalized Outcome Corpus V2 JSONL workbench rows."""

import argparse
import itertools
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
CONTRAST_BLOCKER_CLASSES = [
    "baseline_better",
    "no_policy",
    "same_outcome",
]
ROOT_POOL_SIGNAL_FIELDS = [
    "family",
    "rank_bucket",
    "advisor_bucket",
    "path",
    "safety_detail",
    "progress",
    "efficiency",
    "setup_gain",
    "soft_priority",
    "keeps_awake",
    "reply_floor",
    "reply_risk",
    "followup_floor",
]
ROOT_POOL_COMPOUND_SIGNAL_FIELDS = [
    ("family_rank", ("family", "rank_bucket")),
    ("family_path", ("family", "path")),
    ("family_reply", ("family", "reply_risk")),
    ("family_progress", ("family", "progress")),
    ("advisor_family", ("advisor_bucket", "family")),
    ("path_safety", ("path", "safety_detail")),
    ("safety_progress", ("safety_detail", "progress")),
    ("reply_progress", ("reply_risk", "progress")),
    ("setup_progress", ("setup_gain", "progress")),
]
ROOT_POOL_GUARDED_ORIGIN_KINDS = {
    "guarded_selected",
    "guarded_pre_accept",
    "guarded_head",
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


def parse_family_filter(value):
    if not value:
        return []
    return [item.strip() for item in value.split(",") if item.strip()]


def parse_state_filter(value):
    if value is None:
        return None
    return [item.strip() for item in value.split(",") if item.strip()]


def parse_excluded_families(value):
    return set(parse_family_filter(value))


def parse_optional_family_set(value):
    if value is None:
        return None
    return set(parse_family_filter(value))


def state_id(row):
    return row.get("state_id") or row.get("cross_budget_state_id") or ""


def axis_family(axis):
    first = str(axis or "none").split(" ", 1)[0]
    if first.startswith("axis="):
        return first[len("axis=") :]
    if "=" in first:
        return first.split("=", 1)[0]
    return first or "none"


def excluded_axis_family(family, excluded_families):
    if family in excluded_families:
        return True
    if "root_trajectory" in excluded_families and "trajectory" in family:
        return True
    if "race_delta" in excluded_families and "race_delta" in family:
        return True
    if "race_shape" in excluded_families and "race_shape" in family:
        return True
    if "root_pool" in excluded_families and family.startswith("root_pool"):
        return True
    return False


def contrast_axis_allowed(axis, excluded_families):
    return not excluded_axis_family(axis_family(axis), excluded_families)


def included_signal_family(family, included_families):
    return included_families is None or family in included_families


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


def axis_token_pairs(axis):
    tokens_by_key = {}
    for token in axis_tokens(axis):
        tokens_by_key.setdefault(token["token"], token)
    tokens = [tokens_by_key[key] for key in sorted(tokens_by_key)]
    pairs = []
    for left, right in itertools.combinations(tokens, 2):
        if left["field"] == right["field"]:
            continue
        pairs.append(
            {
                "token": f"{left['token']} && {right['token']}",
                "token_kind": "token_pair",
                "field": f"{left['field']}&&{right['field']}",
                "value": f"{left['value']}&&{right['value']}",
                "left_token": left["token"],
                "right_token": right["token"],
                "left_field": left["field"],
                "right_field": right["field"],
                "left_value": left["value"],
                "right_value": right["value"],
            }
        )
    return pairs


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


def new_axis_token_pair_group(pair):
    group = new_axis_token_group(
        pair["token"],
        pair["token_kind"],
        pair["field"],
        pair["value"],
    )
    group["left_token"] = pair["left_token"]
    group["right_token"] = pair["right_token"]
    group["left_field"] = pair["left_field"]
    group["right_field"] = pair["right_field"]
    group["left_value"] = pair["left_value"]
    group["right_value"] = pair["right_value"]
    return group


def add_axis_token_item_row(groups, row, item, new_group):
    record_class = row.get("record_class", "unknown")
    state = row.get("cross_budget_state_id") or state_id(row)
    if not state:
        return
    family = axis_family(row.get("axis", "none"))
    group = groups.setdefault(item["token"], new_group(item))
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


def add_axis_token_row(groups, row):
    for token in axis_tokens(row.get("axis", "none")):
        add_axis_token_item_row(
            groups,
            row,
            token,
            lambda item: new_axis_token_group(
                item["token"],
                item["token_kind"],
                item["field"],
                item["value"],
            ),
        )


def add_axis_token_pair_row(groups, row):
    for pair in axis_token_pairs(row.get("axis", "none")):
        add_axis_token_item_row(groups, row, pair, new_axis_token_pair_group)


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


def axis_token_pair_groups(rows, target_states, target_families):
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
        add_axis_token_pair_row(groups, row)
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
    for field in [
        "left_token",
        "right_token",
        "left_field",
        "right_field",
        "left_value",
        "right_value",
    ]:
        if field in group:
            row[field] = group[field]
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
    fragmented_dimensions = []
    if row["candidate_better_policy_count"] > 1:
        fragmented_dimensions.append("policy")
    if row["candidate_better_branch_count"] > 1:
        fragmented_dimensions.append("branch")
    if row["candidate_better_pair_count"] > 1:
        fragmented_dimensions.append("pair")
    if row["same_outcome_state_count"] > 0:
        fragmented_dimensions.append("same_outcome")
    row["candidate_fragmented_dimensions"] = "|".join(fragmented_dimensions)
    row["candidate_fragment_count"] = len(fragmented_dimensions)
    row["candidate_low_fragmentation"] = not fragmented_dimensions
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


def axis_token_decision(
    low_fragmentation_repeated,
    fragmented_repeated,
    clean_singleton,
    contaminated,
):
    if low_fragmentation_repeated:
        return "inspect_repeated_candidate_tokens"
    if fragmented_repeated:
        return "fragmented_repeated_candidate_tokens"
    if clean_singleton:
        return "singleton_candidate_tokens"
    if contaminated:
        return "no_clean_candidate_tokens"
    return "no_candidate_tokens"


def axis_token_next_action(decision):
    return {
        "inspect_repeated_candidate_tokens": "inspect_token_feature_before_runtime",
        "fragmented_repeated_candidate_tokens": "archive_or_widen_fragmented_tokens",
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
    low_fragmentation_repeated = [
        row for row in clean_repeated if row["candidate_low_fragmentation"]
    ]
    fragmented_repeated = [
        row for row in clean_repeated if not row["candidate_low_fragmentation"]
    ]
    clean_singleton = [
        row for row in clean_tokens if not row["candidate_repeated_across_target_states"]
    ]
    contaminated = [
        row
        for row in token_rows
        if not row["candidate_clean_from_baseline_no_policy"]
    ]
    decision = axis_token_decision(
        low_fragmentation_repeated,
        fragmented_repeated,
        clean_singleton,
        contaminated,
    )
    return {
        "target_source": state_summary.get("target_source", ""),
        "target_state_count": len(target_states),
        "target_state_ids": target_states,
        "target_family_source": target_family_source,
        "target_family_count": len(target_families),
        "target_families": target_families,
        "candidate_token_count": len(token_rows),
        "clean_repeated_candidate_token_count": len(clean_repeated),
        "low_fragmentation_clean_repeated_candidate_token_count": len(
            low_fragmentation_repeated
        ),
        "fragmented_clean_repeated_candidate_token_count": len(fragmented_repeated),
        "clean_singleton_candidate_token_count": len(clean_singleton),
        "contaminated_candidate_token_count": len(contaminated),
        "axis_token_decision": decision,
        "next_action": axis_token_next_action(decision),
        "top_low_fragmentation_clean_repeated_candidate_tokens": sort_axis_token_rows(
            low_fragmentation_repeated
        )[:token_limit],
        "top_fragmented_clean_repeated_candidate_tokens": sort_axis_token_rows(
            fragmented_repeated
        )[:token_limit],
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


def axis_token_pair_discriminator_summary(
    rows,
    state_summary,
    overlap_families,
    pair_limit,
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
    groups = axis_token_pair_groups(rows, target_states, target_families)
    pair_rows = [
        row
        for row in (summarize_axis_token_group(group) for group in groups.values())
        if row["candidate_better_state_count"] > 0
    ]
    clean_pairs = [
        row for row in pair_rows if row["candidate_clean_from_baseline_no_policy"]
    ]
    clean_repeated = [
        row for row in clean_pairs if row["candidate_repeated_across_target_states"]
    ]
    low_fragmentation_repeated = [
        row for row in clean_repeated if row["candidate_low_fragmentation"]
    ]
    fragmented_repeated = [
        row for row in clean_repeated if not row["candidate_low_fragmentation"]
    ]
    clean_singleton = [
        row for row in clean_pairs if not row["candidate_repeated_across_target_states"]
    ]
    contaminated = [
        row for row in pair_rows if not row["candidate_clean_from_baseline_no_policy"]
    ]
    decision = axis_token_decision(
        low_fragmentation_repeated,
        fragmented_repeated,
        clean_singleton,
        contaminated,
    )
    return {
        "target_source": state_summary.get("target_source", ""),
        "target_state_count": len(target_states),
        "target_state_ids": target_states,
        "target_family_source": target_family_source,
        "target_family_count": len(target_families),
        "target_families": target_families,
        "candidate_token_pair_count": len(pair_rows),
        "clean_repeated_candidate_token_pair_count": len(clean_repeated),
        "low_fragmentation_clean_repeated_candidate_token_pair_count": len(
            low_fragmentation_repeated
        ),
        "fragmented_clean_repeated_candidate_token_pair_count": len(
            fragmented_repeated
        ),
        "clean_singleton_candidate_token_pair_count": len(clean_singleton),
        "contaminated_candidate_token_pair_count": len(contaminated),
        "axis_token_pair_decision": decision,
        "next_action": axis_token_next_action(decision),
        "top_low_fragmentation_clean_repeated_candidate_token_pairs": sort_axis_token_rows(
            low_fragmentation_repeated
        )[:pair_limit],
        "top_fragmented_clean_repeated_candidate_token_pairs": sort_axis_token_rows(
            fragmented_repeated
        )[:pair_limit],
        "top_clean_repeated_candidate_token_pairs": sort_axis_token_rows(
            clean_repeated
        )[:pair_limit],
        "top_clean_singleton_candidate_token_pairs": sort_axis_token_rows(
            clean_singleton
        )[:pair_limit],
        "top_contaminated_candidate_token_pairs": sort_axis_token_rows(contaminated)[
            :pair_limit
        ],
    }


def policy_record_key(row):
    return (row.get("source_jsonl", ""), row.get("source_log", ""), row.get("source_line", ""))


def new_policy_contrast_record(row):
    return {
        "record_key": "|".join(str(item) for item in policy_record_key(row)),
        "source_jsonl": row.get("source_jsonl", ""),
        "source_log": row.get("source_log", ""),
        "source_line": row.get("source_line", ""),
        "record_class": row.get("record_class", "unknown"),
        "cross_budget_state_id": row.get("cross_budget_state_id") or state_id(row),
        "state_id": state_id(row),
        "panel": row.get("panel", ""),
        "duel": row.get("duel", ""),
        "seed_family": row.get("seed_family", ""),
        "repeat": row.get("repeat", ""),
        "opening_index": row.get("opening_index", ""),
        "variant": row.get("variant", ""),
        "candidate_is_white": row.get("candidate_is_white", ""),
        "candidate": row.get("candidate", ""),
        "baseline": row.get("baseline", ""),
        "portfolio_class": row.get("portfolio_class", ""),
        "outcome": row.get("outcome", ""),
        "branch": row.get("branch", ""),
        "pair": row.get("pair", ""),
        "first_diff_ply": row.get("first_diff_ply", ""),
        "axes": set(),
        "axis_families": set(),
        "signals": set(),
    }


def add_policy_contrast_axis(record, row, excluded_families, contrast_families):
    axis = row.get("axis", "none")
    if not contrast_axis_allowed(axis, excluded_families):
        return
    family = axis_family(axis)
    signals = set()
    signal_families = set()
    if included_signal_family(family, contrast_families):
        signal_families.add(family)
        signals.add(f"axis={axis}")
        signals.add(f"axis_family={family}")
    for token in axis_tokens(axis):
        family = signal_family(token["token"])
        if included_signal_family(family, contrast_families):
            signal_families.add(family)
            signals.add(token["token"])
    if not signals:
        return
    record["axes"].add(axis)
    record["axis_families"].update(signal_families)
    record["signals"].update(signals)


def policy_contrast_records(rows, excluded_families, contrast_families):
    records = {}
    for row in rows:
        if row.get("row_type") not in {"policy_decision", "policy_axis"}:
            continue
        key = policy_record_key(row)
        records.setdefault(key, new_policy_contrast_record(row))
    for row in rows:
        if row.get("row_type") != "policy_axis":
            continue
        key = policy_record_key(row)
        record = records.setdefault(key, new_policy_contrast_record(row))
        add_policy_contrast_axis(record, row, excluded_families, contrast_families)
    return [
        record
        for record in records.values()
        if record["record_class"] in RECORD_CLASSES
    ]


def contrast_record_sample(record):
    return {
        "record_class": record.get("record_class", ""),
        "cross_budget_state_id": record.get("cross_budget_state_id", ""),
        "panel": record.get("panel", ""),
        "duel": record.get("duel", ""),
        "variant": record.get("variant", ""),
        "candidate_is_white": record.get("candidate_is_white", ""),
        "candidate": record.get("candidate", ""),
        "branch": record.get("branch", ""),
        "pair": record.get("pair", ""),
        "first_diff_ply": record.get("first_diff_ply", ""),
        "axis_families": pipe_join(record.get("axis_families", set())),
        "axis_count": len(record.get("axes", set())),
    }


def contrast_first_diff_distance(left, right):
    return abs(
        int_field(left, "first_diff_ply") - int_field(right, "first_diff_ply")
    )


def nearest_policy_contrast_sibling(candidate, siblings):
    candidate_signals = candidate.get("signals", set())
    candidates = []
    for sibling in siblings:
        sibling_signals = sibling.get("signals", set())
        shared = candidate_signals & sibling_signals
        union_size = len(candidate_signals | sibling_signals)
        same_duel_rank = 0 if sibling.get("duel", "") == candidate.get("duel", "") else 1
        same_panel_rank = 0 if sibling.get("panel", "") == candidate.get("panel", "") else 1
        candidates.append(
            (
                same_duel_rank,
                same_panel_rank,
                -len(shared),
                -round(len(shared) / union_size, 4) if union_size else 0,
                contrast_first_diff_distance(candidate, sibling),
                sibling.get("record_key", ""),
                sibling,
                shared,
            )
        )
    if not candidates:
        return None, set()
    candidates.sort(key=lambda item: item[:6])
    return candidates[0][6], candidates[0][7]


def new_contrast_signal_group(signal):
    return {
        "signal": signal,
        "count": 0,
        "states": set(),
        "contrast_classes": set(),
        "axis_families": set(),
        "candidate_policies": set(),
        "branches": set(),
        "pairs": set(),
        "duels": set(),
        "sample_contrasts": [],
    }


def signal_family(signal):
    if signal.startswith("axis="):
        return axis_family(signal[len("axis=") :])
    if signal.startswith("axis_family="):
        return signal.split("=", 1)[1]
    field = signal.split("=", 1)[0].split(":", 1)[0]
    return field or "token"


def add_contrast_signal_occurrence(groups, signal, candidate, blocker, contrast_class):
    group = groups.setdefault(signal, new_contrast_signal_group(signal))
    group["count"] += 1
    state = candidate.get("cross_budget_state_id", "")
    if state:
        group["states"].add(state)
    group["contrast_classes"].add(contrast_class)
    group["axis_families"].add(signal_family(signal))
    for field, set_name in [
        ("candidate", "candidate_policies"),
        ("branch", "branches"),
        ("pair", "pairs"),
        ("duel", "duels"),
    ]:
        value = candidate.get(field, "")
        if value:
            group[set_name].add(value)
    if len(group["sample_contrasts"]) < 5:
        group["sample_contrasts"].append(
            {
                "candidate": contrast_record_sample(candidate),
                "blocker": contrast_record_sample(blocker),
                "contrast_class": contrast_class,
            }
        )


def summarize_contrast_signal_group(group):
    fragmented_dimensions = []
    if len(group["candidate_policies"]) > 1:
        fragmented_dimensions.append("policy")
    if len(group["branches"]) > 1:
        fragmented_dimensions.append("branch")
    if len(group["pairs"]) > 1:
        fragmented_dimensions.append("pair")
    if len(group["duels"]) > 1:
        fragmented_dimensions.append("duel")
    return {
        "signal": group["signal"],
        "signal_family": signal_family(group["signal"]),
        "count": group["count"],
        "state_count": len(group["states"]),
        "states": sorted(group["states"]),
        "contrast_class_count": len(group["contrast_classes"]),
        "contrast_classes": pipe_join(group["contrast_classes"]),
        "axis_family_count": len(group["axis_families"]),
        "axis_families": pipe_join(group["axis_families"]),
        "candidate_policy_count": len(group["candidate_policies"]),
        "candidate_policies": pipe_join(group["candidate_policies"]),
        "branch_count": len(group["branches"]),
        "branches": pipe_join(group["branches"]),
        "pair_count": len(group["pairs"]),
        "pairs": pipe_join(group["pairs"]),
        "duel_count": len(group["duels"]),
        "duels": pipe_join(group["duels"]),
        "fragmented_dimensions": pipe_join(fragmented_dimensions),
        "fragment_count": len(fragmented_dimensions),
        "sample_contrasts": group["sample_contrasts"],
    }


def sort_contrast_signal_rows(rows):
    return sorted(
        rows,
        key=lambda row: (
            -row["state_count"],
            -row["contrast_class_count"],
            row["fragment_count"],
            -row["count"],
            row["signal_family"],
            row["signal"],
        ),
    )


def new_contrast_family_group(family):
    return {
        "axis_family": family,
        "count": 0,
        "states": set(),
        "contrast_classes": set(),
        "candidate_policies": set(),
        "branches": set(),
        "pairs": set(),
        "duels": set(),
    }


def add_contrast_family_occurrence(groups, family, candidate, contrast_class):
    group = groups.setdefault(family, new_contrast_family_group(family))
    group["count"] += 1
    state = candidate.get("cross_budget_state_id", "")
    if state:
        group["states"].add(state)
    group["contrast_classes"].add(contrast_class)
    for field, set_name in [
        ("candidate", "candidate_policies"),
        ("branch", "branches"),
        ("pair", "pairs"),
        ("duel", "duels"),
    ]:
        value = candidate.get(field, "")
        if value:
            group[set_name].add(value)


def summarize_contrast_family_group(group):
    fragmented_dimensions = []
    if len(group["candidate_policies"]) > 1:
        fragmented_dimensions.append("policy")
    if len(group["branches"]) > 1:
        fragmented_dimensions.append("branch")
    if len(group["pairs"]) > 1:
        fragmented_dimensions.append("pair")
    if len(group["duels"]) > 1:
        fragmented_dimensions.append("duel")
    return {
        "axis_family": group["axis_family"],
        "count": group["count"],
        "state_count": len(group["states"]),
        "states": sorted(group["states"]),
        "contrast_class_count": len(group["contrast_classes"]),
        "contrast_classes": pipe_join(group["contrast_classes"]),
        "candidate_policy_count": len(group["candidate_policies"]),
        "branch_count": len(group["branches"]),
        "pair_count": len(group["pairs"]),
        "duel_count": len(group["duels"]),
        "fragmented_dimensions": pipe_join(fragmented_dimensions),
        "fragment_count": len(fragmented_dimensions),
    }


def sort_contrast_family_rows(rows):
    return sorted(
        rows,
        key=lambda row: (
            -row["state_count"],
            -row["contrast_class_count"],
            row["fragment_count"],
            -row["count"],
            row["axis_family"],
        ),
    )


def policy_contrast_report(rows, excluded_families, contrast_families, limit):
    records = policy_contrast_records(rows, excluded_families, contrast_families)
    records_by_state = defaultdict(list)
    class_counts = defaultdict(int)
    for record in records:
        class_counts[record["record_class"]] += 1
        state = record.get("cross_budget_state_id", "")
        if state:
            records_by_state[state].append(record)

    signal_groups = {}
    family_groups = {}
    contrast_rows = []
    candidate_records = [
        record for record in records if record["record_class"] == "candidate_better"
    ]
    for candidate in candidate_records:
        state_records = records_by_state.get(candidate.get("cross_budget_state_id", ""), [])
        for contrast_class in CONTRAST_BLOCKER_CLASSES:
            siblings = [
                record
                for record in state_records
                if record["record_class"] == contrast_class
            ]
            blocker, shared = nearest_policy_contrast_sibling(candidate, siblings)
            if blocker is None:
                continue
            candidate_only = candidate["signals"] - blocker["signals"]
            candidate_only_families = {
                family
                for signal in candidate_only
                for family in [signal_family(signal)]
                if not excluded_axis_family(family, excluded_families)
            }
            for signal in candidate_only:
                if excluded_axis_family(signal_family(signal), excluded_families):
                    continue
                add_contrast_signal_occurrence(
                    signal_groups,
                    signal,
                    candidate,
                    blocker,
                    contrast_class,
                )
            for family in candidate_only_families:
                add_contrast_family_occurrence(
                    family_groups,
                    family,
                    candidate,
                    contrast_class,
                )
            contrast_rows.append(
                {
                    "contrast_class": contrast_class,
                    "candidate": contrast_record_sample(candidate),
                    "nearest_blocker": contrast_record_sample(blocker),
                    "shared_signal_count": len(shared),
                    "candidate_signal_count": len(candidate["signals"]),
                    "blocker_signal_count": len(blocker["signals"]),
                    "candidate_only_signal_count": len(candidate_only),
                    "candidate_only_families": pipe_join(candidate_only_families),
                    "sample_candidate_only_signals": sorted(candidate_only)[:10],
                }
            )

    signal_rows = [
        summarize_contrast_signal_group(group)
        for group in signal_groups.values()
    ]
    family_rows = [
        summarize_contrast_family_group(group)
        for group in family_groups.values()
    ]
    repeated_signals = [row for row in signal_rows if row["state_count"] > 1]
    low_fragmentation_repeated_signals = [
        row for row in repeated_signals if row["fragment_count"] == 0
    ]
    fragmented_repeated_signals = [
        row for row in repeated_signals if row["fragment_count"] > 0
    ]
    repeated_families = [row for row in family_rows if row["state_count"] > 1]
    low_fragmentation_repeated_families = [
        row for row in repeated_families if row["fragment_count"] == 0
    ]
    fragmented_repeated_families = [
        row for row in repeated_families if row["fragment_count"] > 0
    ]
    if low_fragmentation_repeated_signals or low_fragmentation_repeated_families:
        decision = "inspect_repeated_outcome_contrast"
        next_action_value = "validate_contrast_family_in_focused_outcome_slice"
    elif fragmented_repeated_signals or fragmented_repeated_families:
        decision = "fragmented_repeated_outcome_contrast"
        next_action_value = "add_below_fragmented_family_feature_or_archive"
    elif contrast_rows:
        decision = "singleton_outcome_contrast_only"
        next_action_value = "return_to_feature_backlog"
    else:
        decision = "no_outcome_contrast_available"
        next_action_value = "collect_candidate_and_blocker_rows"

    return {
        "contrast_decision": decision,
        "source_permission": "no_source",
        "next_action": next_action_value,
        "excluded_families": sorted(excluded_families),
        "contrast_family_source": "cli_contrast_families"
        if contrast_families is not None
        else "all_non_excluded_families",
        "contrast_families": sorted(contrast_families)
        if contrast_families is not None
        else [],
        "record_count": len(records),
        "record_class_counts": sorted_count_rows(class_counts),
        "candidate_record_count": len(candidate_records),
        "contrast_count": len(contrast_rows),
        "candidate_only_signal_count": len(signal_rows),
        "repeated_candidate_only_signal_count": len(repeated_signals),
        "low_fragmentation_repeated_signal_count": len(
            low_fragmentation_repeated_signals
        ),
        "fragmented_repeated_signal_count": len(fragmented_repeated_signals),
        "candidate_only_family_count": len(family_rows),
        "repeated_candidate_only_family_count": len(repeated_families),
        "low_fragmentation_repeated_family_count": len(
            low_fragmentation_repeated_families
        ),
        "fragmented_repeated_family_count": len(fragmented_repeated_families),
        "top_low_fragmentation_repeated_signals": sort_contrast_signal_rows(
            low_fragmentation_repeated_signals
        )[:limit],
        "top_fragmented_repeated_signals": sort_contrast_signal_rows(
            fragmented_repeated_signals
        )[:limit],
        "top_candidate_only_signals": sort_contrast_signal_rows(signal_rows)[:limit],
        "top_low_fragmentation_repeated_families": sort_contrast_family_rows(
            low_fragmentation_repeated_families
        )[:limit],
        "top_fragmented_repeated_families": sort_contrast_family_rows(
            fragmented_repeated_families
        )[:limit],
        "top_candidate_only_families": sort_contrast_family_rows(family_rows)[:limit],
        "sample_contrasts": contrast_rows[:limit],
    }


def root_pool_rows(rows):
    return [row for row in rows if row.get("row_type") == "pro_v4_root_pool_root"]


def root_pool_origin_kinds(row):
    return set(split_pipe(row.get("origin_kinds", "")))


def root_pool_cross_state(row):
    return row.get("cross_budget_state_id") or state_id(row)


def root_pool_pair(row):
    baseline = row.get("baseline_move", "")
    candidate = row.get("candidate_move", "")
    if baseline or candidate:
        return f"{baseline}->{candidate}"
    return ""


def root_pool_snapshot_id(row):
    return "|".join(
        [
            state_id(row),
            f"candidate={row.get('candidate', '')}",
            f"board={row.get('board', '')}",
            f"baseline_move={row.get('baseline_move', '')}",
            f"candidate_move={row.get('candidate_move', '')}",
        ]
    )


def root_pool_is_candidate_winning_policy(row):
    return (
        row.get("portfolio_class") == "candidate_only_win"
        and "winning_policy_output" in root_pool_origin_kinds(row)
    )


def root_pool_blocker_reasons(row, candidate_cross_states):
    kinds = root_pool_origin_kinds(row)
    reasons = set()
    guarded_kinds = kinds & ROOT_POOL_GUARDED_ORIGIN_KINDS
    for kind in guarded_kinds:
        reasons.add(kind)

    is_candidate = root_pool_is_candidate_winning_policy(row)
    if root_pool_cross_state(row) in candidate_cross_states and not is_candidate:
        portfolio_class = row.get("portfolio_class", "")
        if portfolio_class and portfolio_class != "candidate_only_win":
            reasons.add(f"same_state_{portfolio_class}")
        if "policy_output" in kinds and "winning_policy_output" not in kinds:
            reasons.add("same_state_nonwinning_policy_output")
    return reasons


def root_pool_field_value(row, field):
    value = str(row.get(field, "")).strip()
    if not value or value == "omitted":
        return ""
    return value


def root_pool_signal_items(row):
    items = []
    for field in ROOT_POOL_SIGNAL_FIELDS:
        value = root_pool_field_value(row, field)
        if not value:
            continue
        items.append(
            {
                "signal": f"root_pool:{field}={value}",
                "signal_kind": "field_value",
                "field": field,
                "value": value,
            }
        )
    for compound, fields in ROOT_POOL_COMPOUND_SIGNAL_FIELDS:
        values = [root_pool_field_value(row, field) for field in fields]
        if not all(values):
            continue
        value = "|".join(values)
        items.append(
            {
                "signal": f"root_pool:{compound}={value}",
                "signal_kind": "field_compound",
                "field": compound,
                "value": value,
            }
        )
    return items


def root_pool_sample_root(row):
    return {
        "cross_budget_state_id": root_pool_cross_state(row),
        "state_id": state_id(row),
        "portfolio_class": row.get("portfolio_class", ""),
        "outcome": row.get("outcome", ""),
        "candidate": row.get("candidate", ""),
        "duel": row.get("duel", ""),
        "variant": row.get("variant", ""),
        "pair": root_pool_pair(row),
        "origin_kinds": row.get("origin_kinds", ""),
        "policies": row.get("policies", ""),
        "family": row.get("family", ""),
        "rank_bucket": row.get("rank_bucket", ""),
        "advisor_bucket": row.get("advisor_bucket", ""),
        "path": row.get("path", ""),
        "safety_detail": row.get("safety_detail", ""),
        "progress": row.get("progress", ""),
        "reply_risk": row.get("reply_risk", ""),
    }


def new_root_pool_signal_group(item):
    return {
        "signal": item["signal"],
        "signal_kind": item["signal_kind"],
        "field": item["field"],
        "value": item["value"],
        "candidate_roots": 0,
        "candidate_states": set(),
        "candidate_snapshots": set(),
        "candidate_profiles": set(),
        "candidate_policies": set(),
        "candidate_duels": set(),
        "candidate_pairs": set(),
        "candidate_variants": set(),
        "blocker_roots": 0,
        "blocker_states": set(),
        "blocker_snapshots": set(),
        "blocker_reasons": defaultdict(int),
        "guarded_blocker_roots": 0,
        "same_state_blocker_roots": 0,
        "sample_candidate_roots": [],
        "sample_blocker_roots": [],
    }


def add_root_pool_candidate_occurrence(group, row):
    group["candidate_roots"] += 1
    state = root_pool_cross_state(row)
    snapshot = root_pool_snapshot_id(row)
    if state:
        group["candidate_states"].add(state)
    if snapshot:
        group["candidate_snapshots"].add(snapshot)
    for value in split_pipe(row.get("policies", "")):
        group["candidate_policies"].add(value)
    for field, set_name in [
        ("candidate", "candidate_profiles"),
        ("duel", "candidate_duels"),
        ("variant", "candidate_variants"),
    ]:
        value = row.get(field, "")
        if value:
            group[set_name].add(value)
    pair = root_pool_pair(row)
    if pair:
        group["candidate_pairs"].add(pair)
    if len(group["sample_candidate_roots"]) < 5:
        group["sample_candidate_roots"].append(root_pool_sample_root(row))


def add_root_pool_blocker_occurrence(group, row, reasons):
    group["blocker_roots"] += 1
    state = root_pool_cross_state(row)
    snapshot = root_pool_snapshot_id(row)
    if state:
        group["blocker_states"].add(state)
    if snapshot:
        group["blocker_snapshots"].add(snapshot)
    if reasons & ROOT_POOL_GUARDED_ORIGIN_KINDS:
        group["guarded_blocker_roots"] += 1
    if any(reason.startswith("same_state_") for reason in reasons):
        group["same_state_blocker_roots"] += 1
    for reason in reasons:
        group["blocker_reasons"][reason] += 1
    if len(group["sample_blocker_roots"]) < 5:
        sample = root_pool_sample_root(row)
        sample["blocker_reasons"] = pipe_join(reasons)
        group["sample_blocker_roots"].append(sample)


def summarize_root_pool_signal_group(group):
    fragmented_dimensions = []
    if len(group["candidate_profiles"]) > 1:
        fragmented_dimensions.append("candidate")
    if len(group["candidate_policies"]) > 1:
        fragmented_dimensions.append("policy")
    if len(group["candidate_duels"]) > 1:
        fragmented_dimensions.append("duel")
    if len(group["candidate_pairs"]) > 1:
        fragmented_dimensions.append("pair")
    if len(group["candidate_variants"]) > 1:
        fragmented_dimensions.append("variant")
    return {
        "signal": group["signal"],
        "signal_kind": group["signal_kind"],
        "field": group["field"],
        "value": group["value"],
        "candidate_root_count": group["candidate_roots"],
        "candidate_state_count": len(group["candidate_states"]),
        "candidate_state_ids": sorted(group["candidate_states"]),
        "candidate_snapshot_count": len(group["candidate_snapshots"]),
        "candidate_profile_count": len(group["candidate_profiles"]),
        "candidate_profiles": pipe_join(group["candidate_profiles"]),
        "candidate_policy_count": len(group["candidate_policies"]),
        "candidate_policies": pipe_join(group["candidate_policies"]),
        "candidate_duel_count": len(group["candidate_duels"]),
        "candidate_duels": pipe_join(group["candidate_duels"]),
        "candidate_pair_count": len(group["candidate_pairs"]),
        "candidate_pairs": pipe_join(group["candidate_pairs"]),
        "candidate_variant_count": len(group["candidate_variants"]),
        "candidate_variants": pipe_join(group["candidate_variants"]),
        "blocker_root_count": group["blocker_roots"],
        "blocker_state_count": len(group["blocker_states"]),
        "blocker_snapshot_count": len(group["blocker_snapshots"]),
        "guarded_blocker_root_count": group["guarded_blocker_roots"],
        "same_state_blocker_root_count": group["same_state_blocker_roots"],
        "blocker_reason_counts": sorted_count_rows(group["blocker_reasons"]),
        "candidate_clean_from_blockers": group["blocker_roots"] == 0,
        "candidate_repeated_across_states": len(group["candidate_states"]) > 1,
        "candidate_fragmented_dimensions": pipe_join(fragmented_dimensions),
        "candidate_fragment_count": len(fragmented_dimensions),
        "candidate_low_fragmentation": not fragmented_dimensions,
        "sample_candidate_roots": group["sample_candidate_roots"],
        "sample_blocker_roots": group["sample_blocker_roots"],
    }


def sort_root_pool_signal_rows(rows):
    return sorted(
        rows,
        key=lambda row: (
            -row["candidate_state_count"],
            row["candidate_fragment_count"],
            row["blocker_root_count"],
            -row["candidate_root_count"],
            row["field"],
            row["signal"],
        ),
    )


def root_pool_discriminator_decision(
    low_fragmentation_repeated,
    fragmented_repeated,
    clean_singleton,
    contaminated,
    candidate_roots,
    root_rows_value,
):
    if not root_rows_value:
        return "no_pro_v4_root_pool_rows"
    if not candidate_roots:
        return "no_candidate_only_winning_policy_roots"
    if low_fragmentation_repeated:
        return "inspect_repeated_root_pool_discriminator"
    if fragmented_repeated:
        return "fragmented_repeated_root_pool_signal"
    if clean_singleton:
        return "singleton_root_pool_signal"
    if contaminated:
        return "contaminated_root_pool_signals"
    return "no_candidate_root_pool_signals"


def root_pool_discriminator_next_action(decision):
    return {
        "inspect_repeated_root_pool_discriminator": (
            "validate_root_pool_signal_in_focused_outcome_slice"
        ),
        "fragmented_repeated_root_pool_signal": (
            "add_below_fragmented_root_feature_or_archive"
        ),
        "singleton_root_pool_signal": "archive_or_widen_root_pool_singletons",
        "contaminated_root_pool_signals": "add_new_root_feature_or_archive_current_fields",
        "no_candidate_only_winning_policy_roots": "rerun_with_candidate_only_root_pool_rows",
        "no_pro_v4_root_pool_rows": "enable_pro_v4_root_pool_export",
        "no_candidate_root_pool_signals": "add_new_root_feature_or_archive_current_fields",
    }.get(decision, "review")


def root_pool_discriminator_summary(rows, limit):
    roots = root_pool_rows(rows)
    candidate_roots = [
        row for row in roots if root_pool_is_candidate_winning_policy(row)
    ]
    candidate_cross_states = {
        root_pool_cross_state(row) for row in candidate_roots if root_pool_cross_state(row)
    }
    candidate_snapshots = {
        root_pool_snapshot_id(row) for row in candidate_roots if root_pool_snapshot_id(row)
    }

    blocker_roots = []
    blocker_reason_by_id = {}
    for row in roots:
        reasons = root_pool_blocker_reasons(row, candidate_cross_states)
        if not reasons:
            continue
        blocker_roots.append(row)
        blocker_reason_by_id[id(row)] = reasons

    groups = {}
    candidate_signal_fields = defaultdict(int)
    blocker_signal_fields = defaultdict(int)
    for row in candidate_roots:
        for item in root_pool_signal_items(row):
            group = groups.setdefault(
                item["signal"], new_root_pool_signal_group(item)
            )
            candidate_signal_fields[item["field"]] += 1
            add_root_pool_candidate_occurrence(group, row)
    for row in blocker_roots:
        reasons = blocker_reason_by_id[id(row)]
        for item in root_pool_signal_items(row):
            group = groups.setdefault(
                item["signal"], new_root_pool_signal_group(item)
            )
            blocker_signal_fields[item["field"]] += 1
            add_root_pool_blocker_occurrence(group, row, reasons)

    signal_rows = [
        summarize_root_pool_signal_group(group)
        for group in groups.values()
        if group["candidate_roots"] > 0
    ]
    clean = [row for row in signal_rows if row["candidate_clean_from_blockers"]]
    clean_repeated = [
        row for row in clean if row["candidate_repeated_across_states"]
    ]
    low_fragmentation_repeated = [
        row for row in clean_repeated if row["candidate_low_fragmentation"]
    ]
    fragmented_repeated = [
        row for row in clean_repeated if not row["candidate_low_fragmentation"]
    ]
    clean_singleton = [
        row for row in clean if not row["candidate_repeated_across_states"]
    ]
    contaminated = [
        row for row in signal_rows if not row["candidate_clean_from_blockers"]
    ]
    decision = root_pool_discriminator_decision(
        low_fragmentation_repeated,
        fragmented_repeated,
        clean_singleton,
        contaminated,
        candidate_roots,
        roots,
    )
    source_permission_value = (
        "inspect_for_source"
        if decision == "inspect_repeated_root_pool_discriminator"
        else "no_source"
    )
    return {
        "discriminator_decision": decision,
        "source_permission": source_permission_value,
        "next_action": root_pool_discriminator_next_action(decision),
        "root_count": len(roots),
        "candidate_only_winning_policy_root_count": len(candidate_roots),
        "candidate_cross_budget_state_count": len(candidate_cross_states),
        "candidate_cross_budget_state_ids": sorted(candidate_cross_states),
        "candidate_snapshot_count": len(candidate_snapshots),
        "blocker_root_count": len(blocker_roots),
        "guarded_blocker_root_count": sum(
            1
            for row in blocker_roots
            if blocker_reason_by_id[id(row)] & ROOT_POOL_GUARDED_ORIGIN_KINDS
        ),
        "same_state_blocker_root_count": sum(
            1
            for row in blocker_roots
            if any(
                reason.startswith("same_state_")
                for reason in blocker_reason_by_id[id(row)]
            )
        ),
        "candidate_signal_count": len(signal_rows),
        "clean_repeated_candidate_signal_count": len(clean_repeated),
        "low_fragmentation_repeated_candidate_signal_count": len(
            low_fragmentation_repeated
        ),
        "fragmented_repeated_candidate_signal_count": len(fragmented_repeated),
        "clean_singleton_candidate_signal_count": len(clean_singleton),
        "contaminated_candidate_signal_count": len(contaminated),
        "candidate_signal_field_counts": sorted_count_rows(candidate_signal_fields),
        "blocker_signal_field_counts": sorted_count_rows(blocker_signal_fields),
        "top_low_fragmentation_repeated_candidate_signals": sort_root_pool_signal_rows(
            low_fragmentation_repeated
        )[:limit],
        "top_fragmented_repeated_candidate_signals": sort_root_pool_signal_rows(
            fragmented_repeated
        )[:limit],
        "top_clean_repeated_candidate_signals": sort_root_pool_signal_rows(
            clean_repeated
        )[:limit],
        "top_clean_singleton_candidate_signals": sort_root_pool_signal_rows(
            clean_singleton
        )[:limit],
        "top_contaminated_candidate_signals": sort_root_pool_signal_rows(
            contaminated
        )[:limit],
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
    include_contrast=False,
    exclude_families=None,
    contrast_families=None,
):
    if overlap_families is None:
        overlap_families = DEFAULT_OVERLAP_FAMILIES
    if exclude_families is None:
        exclude_families = set()
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
    digest = {
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
        "axis_token_pair_discriminator": axis_token_pair_discriminator_summary(
            rows,
            state_summary,
            overlap_families,
            pair_limit=limit,
            token_families=token_families,
        ),
        "pro_v4_root_pool_discriminator": root_pool_discriminator_summary(
            rows,
            limit=limit,
        ),
    }
    if include_contrast:
        digest["contrast_report"] = policy_contrast_report(
            rows,
            exclude_families,
            contrast_families,
            limit,
        )
    return digest


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
    parser.add_argument(
        "--contrast-report",
        action="store_true",
        help=(
            "also emit a postprocess-only candidate-vs-blocker contrast report "
            "over normalized outcome rows"
        ),
    )
    parser.add_argument(
        "--exclude-families",
        default="",
        help=(
            "comma-separated axis families to exclude from the contrast report "
            "(supports aliases such as root_trajectory and race_shape)"
        ),
    )
    parser.add_argument(
        "--contrast-families",
        default=None,
        help=(
            "comma-separated signal families to keep in the contrast report "
            "(default: all non-excluded families)"
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
        include_contrast=args.contrast_report,
        exclude_families=parse_excluded_families(args.exclude_families),
        contrast_families=parse_optional_family_set(args.contrast_families),
    )
    if args.compact:
        json.dump(digest, sys.stdout, sort_keys=True, separators=(",", ":"))
    else:
        json.dump(digest, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
