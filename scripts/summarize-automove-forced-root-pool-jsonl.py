#!/usr/bin/env python3
"""Summarize normalized forced-root pool JSONL discriminator signals."""

import argparse
import itertools
import json
import re
import sys
from collections import defaultdict
from pathlib import Path


SIGNAL_SECTIONS = {
    "exact_axis": "exact_axis",
    "token": "token",
    "token_pair": "token_pair",
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


def sorted_count_rows(counter):
    return [
        {"key": key, "count": count}
        for key, count in sorted(counter.items(), key=lambda item: (-item[1], item[0]))
    ]


def row_type_counts(rows):
    counts = defaultdict(int)
    for row in rows:
        counts[row.get("row_type", "unknown")] += 1
    return counts


def pipe_join(items):
    return "|".join(str(item) for item in sorted(items, key=str))


def parse_axis_fields(axis):
    fields = []
    for part in str(axis or "").split():
        if "=" not in part:
            continue
        key, value = part.split("=", 1)
        key = key.strip()
        value = value.strip()
        if key and value:
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
        tokens.append(
            {
                "signal": f"{key}={value}",
                "signal_kind": "field_value",
                "field": key,
                "value": value,
            }
        )
        for atom in axis_value_atoms(value):
            tokens.append(
                {
                    "signal": f"{key}:atom={atom}",
                    "signal_kind": "field_atom",
                    "field": key,
                    "value": atom,
                }
            )
    return tokens


def token_pairs(tokens):
    tokens_by_signal = {token["signal"]: token for token in tokens}
    unique_tokens = [tokens_by_signal[key] for key in sorted(tokens_by_signal)]
    pairs = []
    for left, right in itertools.combinations(unique_tokens, 2):
        if left["field"] == right["field"]:
            continue
        pairs.append(
            {
                "signal": f"{left['signal']} && {right['signal']}",
                "signal_kind": "token_pair",
                "field": f"{left['field']}&&{right['field']}",
                "value": f"{left['value']}&&{right['value']}",
                "left_signal": left["signal"],
                "right_signal": right["signal"],
                "left_field": left["field"],
                "right_field": right["field"],
                "left_value": left["value"],
                "right_value": right["value"],
            }
        )
    return pairs


def root_rows(rows):
    roots = {}
    for row in rows:
        if row.get("row_type") != "forced_root_pool_root":
            continue
        root_id = row.get("root_id", "")
        if root_id:
            roots[root_id] = row
    return roots


def axis_rows_by_root(rows, roots):
    axes_by_root = defaultdict(list)
    for row in rows:
        if row.get("row_type") != "forced_root_pool_axis":
            continue
        root_id = row.get("root_id", "")
        if root_id in roots:
            axes_by_root[root_id].append(row)
    return axes_by_root


def new_signal_group(signal, signal_type, signal_kind, field, value):
    return {
        "signal": signal,
        "signal_type": signal_type,
        "signal_kind": signal_kind,
        "field": field,
        "value": value,
        "roots": set(),
        "winner_roots": set(),
        "nonwinner_roots": set(),
        "labels": set(),
        "winner_labels": set(),
        "nonwinner_labels": set(),
        "axes": set(),
        "families": set(),
        "winner_families": set(),
        "paths": set(),
        "winner_paths": set(),
        "rank_bands": set(),
        "winner_rank_bands": set(),
        "sample_winner_roots": [],
        "sample_nonwinner_roots": [],
    }


def root_sample(root):
    return {
        "label": root.get("label", ""),
        "root_rank": int(root.get("root_rank", -1)),
        "rank_band": root.get("rank_band", ""),
        "family": root.get("family", ""),
        "path": root.get("path", ""),
        "advisor_bucket": root.get("advisor_bucket", ""),
        "result": root.get("result", ""),
        "inputs": root.get("inputs", ""),
    }


def add_signal_occurrence(groups, signal_item, signal_type, root, axis=None):
    signal = signal_item["signal"]
    group = groups.setdefault(
        signal,
        new_signal_group(
            signal,
            signal_type,
            signal_item.get("signal_kind", signal_type),
            signal_item.get("field", ""),
            signal_item.get("value", ""),
        ),
    )
    for field in [
        "left_signal",
        "right_signal",
        "left_field",
        "right_field",
        "left_value",
        "right_value",
    ]:
        if field in signal_item:
            group.setdefault(field, signal_item[field])
    root_id = root.get("root_id", "")
    label = root.get("label", "")
    is_win = bool(root.get("is_winning_root", False))
    group["roots"].add(root_id)
    if label:
        group["labels"].add(label)
    if axis:
        group["axes"].add(axis)
    for field, set_name in [
        ("family", "families"),
        ("path", "paths"),
        ("rank_band", "rank_bands"),
    ]:
        value = root.get(field, "")
        if value:
            group[set_name].add(value)
    if is_win:
        group["winner_roots"].add(root_id)
        if label:
            group["winner_labels"].add(label)
        for field, set_name in [
            ("family", "winner_families"),
            ("path", "winner_paths"),
            ("rank_band", "winner_rank_bands"),
        ]:
            value = root.get(field, "")
            if value:
                group[set_name].add(value)
        if len(group["sample_winner_roots"]) < 5:
            group["sample_winner_roots"].append(root_sample(root))
    else:
        group["nonwinner_roots"].add(root_id)
        if label:
            group["nonwinner_labels"].add(label)
        if len(group["sample_nonwinner_roots"]) < 5:
            group["sample_nonwinner_roots"].append(root_sample(root))


def exact_axis_signal(axis_row):
    axis = axis_row.get("axis", "")
    return {
        "signal": axis,
        "signal_kind": "exact_axis",
        "field": axis_row.get("dimension", ""),
        "value": axis.split("=", 1)[1] if "=" in axis else axis,
    }


def collect_signal_groups(roots, axes_by_root, signal_type):
    groups = {}
    for root_id, root in roots.items():
        axis_rows = axes_by_root.get(root_id, [])
        if signal_type == "exact_axis":
            seen_axes = set()
            for axis_row in axis_rows:
                axis = axis_row.get("axis", "")
                if not axis or axis in seen_axes:
                    continue
                seen_axes.add(axis)
                add_signal_occurrence(
                    groups,
                    exact_axis_signal(axis_row),
                    signal_type,
                    root,
                    axis=axis,
                )
            continue

        tokens = []
        for axis_row in axis_rows:
            axis = axis_row.get("axis", "")
            for token in axis_tokens(axis):
                token["axis"] = axis
                tokens.append(token)

        if signal_type == "token":
            seen_tokens = {}
            for token in tokens:
                seen_tokens.setdefault(token["signal"], token)
            for token in seen_tokens.values():
                add_signal_occurrence(
                    groups,
                    token,
                    signal_type,
                    root,
                    axis=token.get("axis", ""),
                )
            continue

        if signal_type == "token_pair":
            seen_pairs = {}
            for pair in token_pairs(tokens):
                seen_pairs.setdefault(pair["signal"], pair)
            for pair in seen_pairs.values():
                add_signal_occurrence(groups, pair, signal_type, root)
            continue

        raise AssertionError(f"unknown signal type: {signal_type}")
    return groups


def summarize_signal_group(group):
    root_count = len(group["roots"])
    winner_root_count = len(group["winner_roots"])
    nonwinner_root_count = len(group["nonwinner_roots"])
    row = {
        "signal": group["signal"],
        "signal_type": group["signal_type"],
        "signal_kind": group["signal_kind"],
        "field": group["field"],
        "value": group["value"],
        "root_count": root_count,
        "winner_root_count": winner_root_count,
        "nonwinner_root_count": nonwinner_root_count,
        "label_count": len(group["labels"]),
        "labels": sorted(group["labels"]),
        "winner_label_count": len(group["winner_labels"]),
        "winner_labels": sorted(group["winner_labels"]),
        "nonwinner_label_count": len(group["nonwinner_labels"]),
        "nonwinner_labels": sorted(group["nonwinner_labels"]),
        "winner_precision": round(winner_root_count / root_count, 4)
        if root_count > 0
        else 0.0,
        "axis_count": len(group["axes"]),
        "sample_axes": sorted(group["axes"])[:5],
        "family_count": len(group["families"]),
        "families": pipe_join(group["families"]),
        "winner_family_count": len(group["winner_families"]),
        "winner_families": pipe_join(group["winner_families"]),
        "path_count": len(group["paths"]),
        "paths": pipe_join(group["paths"]),
        "winner_paths": pipe_join(group["winner_paths"]),
        "rank_bands": pipe_join(group["rank_bands"]),
        "winner_rank_bands": pipe_join(group["winner_rank_bands"]),
        "sample_winner_roots": group["sample_winner_roots"],
        "sample_nonwinner_roots": group["sample_nonwinner_roots"],
    }
    for field in [
        "left_signal",
        "right_signal",
        "left_field",
        "right_field",
        "left_value",
        "right_value",
    ]:
        if field in group:
            row[field] = group[field]
    if winner_root_count == 0:
        fragmentation = "no_winner_roots"
    elif nonwinner_root_count > 0:
        fragmentation = "nonwinner_contaminated"
    elif row["winner_label_count"] <= 1:
        fragmentation = "singleton_label"
    elif row["winner_family_count"] > 1:
        fragmentation = "family_fragmented"
    else:
        fragmentation = "clean_repeated"
    row["fragmentation"] = fragmentation
    row["source_permission"] = (
        "inspect_for_source" if fragmentation == "clean_repeated" else "no_source"
    )
    return row


def sort_signal_rows(rows):
    return sorted(
        rows,
        key=lambda row: (
            -row["winner_label_count"],
            row["nonwinner_root_count"],
            -row["winner_root_count"],
            -row["winner_precision"],
            row["signal_kind"],
            row["signal"],
        ),
    )


def signal_decision(clean_repeated, family_fragmented, clean_singleton, contaminated):
    if clean_repeated:
        return "inspect_clean_repeated_winner_signal"
    if family_fragmented:
        return "fragmented_repeated_winner_signal"
    if clean_singleton:
        return "singleton_winner_signal"
    if contaminated:
        return "nonwinner_contaminated_winner_signal"
    return "no_winner_signal"


def next_action_for_decision(decision):
    return {
        "inspect_clean_repeated_winner_signal": "validate_signal_against_outcome_corpus",
        "fragmented_repeated_winner_signal": "add_below_family_discriminator",
        "singleton_winner_signal": "widen_root_pool_or_archive_singletons",
        "nonwinner_contaminated_winner_signal": "archive_or_design_new_root_feature",
        "no_winner_signal": "collect_oracle_rows_with_winning_roots",
    }.get(decision, "review")


def summarize_signal_section(roots, axes_by_root, signal_type, limit):
    groups = collect_signal_groups(roots, axes_by_root, signal_type)
    rows = [
        row
        for row in (summarize_signal_group(group) for group in groups.values())
        if row["winner_root_count"] > 0
    ]
    clean_repeated = [
        row for row in rows if row["fragmentation"] == "clean_repeated"
    ]
    family_fragmented = [
        row for row in rows if row["fragmentation"] == "family_fragmented"
    ]
    clean_singleton = [
        row for row in rows if row["fragmentation"] == "singleton_label"
    ]
    contaminated = [
        row for row in rows if row["fragmentation"] == "nonwinner_contaminated"
    ]
    decision = signal_decision(
        clean_repeated, family_fragmented, clean_singleton, contaminated
    )
    return {
        "signal_type": signal_type,
        "winner_signal_count": len(rows),
        "clean_repeated_winner_signal_count": len(clean_repeated),
        "family_fragmented_repeated_winner_signal_count": len(family_fragmented),
        "clean_singleton_winner_signal_count": len(clean_singleton),
        "nonwinner_contaminated_winner_signal_count": len(contaminated),
        "signal_decision": decision,
        "source_permission": (
            "inspect_for_source"
            if decision == "inspect_clean_repeated_winner_signal"
            else "no_source"
        ),
        "next_action": next_action_for_decision(decision),
        "top_clean_repeated_winner_signals": sort_signal_rows(clean_repeated)[:limit],
        "top_family_fragmented_repeated_winner_signals": sort_signal_rows(
            family_fragmented
        )[:limit],
        "top_clean_singleton_winner_signals": sort_signal_rows(clean_singleton)[
            :limit
        ],
        "top_nonwinner_contaminated_winner_signals": sort_signal_rows(contaminated)[
            :limit
        ],
    }


def root_pool_summary(rows, roots):
    labels = set()
    labels_with_wins = set()
    root_pools = set()
    result_counts = defaultdict(int)
    family_counts = defaultdict(int)
    winner_family_counts = defaultdict(int)
    path_counts = defaultdict(int)
    winner_path_counts = defaultdict(int)
    for root in roots.values():
        label = root.get("label", "")
        result = root.get("result", "unknown")
        family = root.get("family", "")
        path = root.get("path", "")
        if label:
            labels.add(label)
        if root.get("root_pool_id", ""):
            root_pools.add(root.get("root_pool_id", ""))
        result_counts[result] += 1
        if family:
            family_counts[family] += 1
        if path:
            path_counts[path] += 1
        if root.get("is_winning_root", False):
            if label:
                labels_with_wins.add(label)
            if family:
                winner_family_counts[family] += 1
            if path:
                winner_path_counts[path] += 1
    return {
        "root_pool_count": len(root_pools),
        "label_count": len(labels),
        "labels": sorted(labels),
        "labels_with_wins": sorted(labels_with_wins),
        "root_count": len(roots),
        "result_counts": sorted_count_rows(result_counts),
        "family_counts": sorted_count_rows(family_counts),
        "winner_family_counts": sorted_count_rows(winner_family_counts),
        "path_counts": sorted_count_rows(path_counts),
        "winner_path_counts": sorted_count_rows(winner_path_counts),
        "row_counts": sorted_count_rows(row_type_counts(rows)),
    }


def workbench_decision(sections):
    if any(
        section["source_permission"] == "inspect_for_source"
        for section in sections.values()
    ):
        return "inspect_for_source"
    if any(
        section["clean_singleton_winner_signal_count"] > 0
        or section["family_fragmented_repeated_winner_signal_count"] > 0
        for section in sections.values()
    ):
        return "fragmented_or_singleton_winner_signals"
    if any(
        section["nonwinner_contaminated_winner_signal_count"] > 0
        for section in sections.values()
    ):
        return "nonwinner_contaminated_winner_signals"
    return "no_winner_signals"


def next_action(decision):
    return {
        "inspect_for_source": "validate_signal_against_outcome_corpus",
        "fragmented_or_singleton_winner_signals": "widen_root_pool_or_add_discriminator",
        "nonwinner_contaminated_winner_signals": "design_new_root_feature",
        "no_winner_signals": "collect_oracle_rows_with_winning_roots",
    }.get(decision, "review")


def summarize(rows, limit=12):
    roots = root_rows(rows)
    axes_by_root = axis_rows_by_root(rows, roots)
    sections = {
        section_name: summarize_signal_section(
            roots,
            axes_by_root,
            signal_type,
            limit,
        )
        for section_name, signal_type in SIGNAL_SECTIONS.items()
    }
    decision = workbench_decision(sections)
    return {
        "root_pool": root_pool_summary(rows, roots),
        "workbench_decision": decision,
        "source_permission": "inspect_for_source"
        if decision == "inspect_for_source"
        else "no_source",
        "next_action": next_action(decision),
        **sections,
    }


def main():
    parser = argparse.ArgumentParser(
        description=(
            "Read normalized forced-root pool JSONL rows and rank winner "
            "signals by exact axis, token, and root-level token pair."
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
