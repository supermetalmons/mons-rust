#!/usr/bin/env python3
"""Summarize policy-matrix experiment logs into one JSON decision digest."""

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path


POLICY_MATRIX_PREFIX = "PRO_POLICY_MATRIX_"


def parse_policy_matrix_lines(paths):
    events = []
    for path in paths:
        with path.open("r", encoding="utf-8") as handle:
            for line_number, line in enumerate(handle, start=1):
                line = line.rstrip("\n")
                if not line.startswith(POLICY_MATRIX_PREFIX):
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


def permission_from_recommendation(recommendation):
    if not recommendation:
        return "missing_recommendation"
    label = recommendation.get("label", "")
    if label == "narrow_low_fragmentation_route":
        return "inspect_for_source"
    if label == "build_outcome_corpus_v2":
        return "postprocess_only"
    return "no_source"


def corpus_decision(summary, stoplight, recommendation):
    stoplight_label = stoplight.get("label", "")
    recommendation_label = recommendation.get("label", "")
    no_policy_wins = int(summary.get("no_policy_wins", 0))
    baseline_only_wins = int(summary.get("baseline_only_wins", 0))

    if recommendation_label == "narrow_low_fragmentation_route":
        return "inspect_for_source"
    if no_policy_wins > 0 or stoplight_label == "coverage_gap":
        return "coverage_gap"
    if (
        baseline_only_wins > 0
        or stoplight_label == "baseline_save_risk"
        or recommendation_label == "baseline_save_risk_only"
    ):
        return "baseline_save_risk"
    if recommendation_label == "build_outcome_corpus_v2":
        return "postprocess_only"
    if recommendation_label == "singleton_candidate_routes":
        return "singleton_no_source"
    if recommendation_label == "no_candidate_route":
        return "no_candidate_route"
    return "no_source"


def permission_from_filter_summary(summary):
    if not summary:
        return "missing_summary"
    records = int(summary.get("breakdown_records", 0))
    if records == 0:
        return "no_matching_records"
    fragmented_dimensions = []
    for field, dimension in [
        ("candidate_count", "candidate_policy"),
        ("branch_count", "branch"),
        ("pair_count", "first_move_pair"),
    ]:
        if int(summary.get(field, 0)) > 1:
            fragmented_dimensions.append(dimension)
    if fragmented_dimensions:
        return "fragmented_no_source"
    return "focused_candidate"


def sorted_details(details):
    return sorted(
        details,
        key=lambda item: (
            item.get("dimension", ""),
            int(item.get("rank", 0)),
            item.get("key", ""),
        ),
    )


def summarize(events):
    latest = {}
    route_buckets = defaultdict(list)
    filter_summaries = {}
    filter_details = defaultdict(list)
    event_counts = defaultdict(int)

    for event in events:
        event_type = event["event_type"]
        data = event["data"]
        event_counts[event_type] += 1
        if event_type in {
            "PRO_POLICY_MATRIX_GLOBAL_SUMMARY",
            "PRO_POLICY_MATRIX_GLOBAL_STOPLIGHT",
            "PRO_POLICY_MATRIX_GLOBAL_ROUTE_RECOMMENDATION",
        }:
            latest[event_type] = data
        elif event_type == "PRO_POLICY_MATRIX_GLOBAL_ROUTE_BUCKET":
            route_buckets[data.get("bucket", "unknown")].append(data)
        elif event_type == "PRO_POLICY_MATRIX_RECORD_FILTER_SUMMARY":
            filter_summaries[data.get("record_axis_filter", "")] = data
        elif event_type == "PRO_POLICY_MATRIX_RECORD_FILTER_DETAIL":
            filter_details[data.get("record_axis_filter", "")].append(data)

    recommendation = latest.get("PRO_POLICY_MATRIX_GLOBAL_ROUTE_RECOMMENDATION", {})
    global_summary = latest.get("PRO_POLICY_MATRIX_GLOBAL_SUMMARY", {})
    stoplight = latest.get("PRO_POLICY_MATRIX_GLOBAL_STOPLIGHT", {})
    filters = []
    for record_axis_filter, filter_summary in sorted(filter_summaries.items()):
        details = sorted_details(filter_details.get(record_axis_filter, []))
        filters.append(
            {
                "record_axis_filter": record_axis_filter,
                "permission": permission_from_filter_summary(filter_summary),
                "summary": filter_summary,
                "details": details,
            }
        )

    return {
        "event_counts": dict(sorted(event_counts.items())),
        "global_summary": global_summary,
        "global_stoplight": stoplight,
        "route_recommendation": recommendation,
        "route_permission": permission_from_recommendation(recommendation),
        "corpus_decision": corpus_decision(global_summary, stoplight, recommendation),
        "route_buckets": {
            bucket: sorted(rows, key=lambda row: int(row.get("rank", 0)))
            for bucket, rows in sorted(route_buckets.items())
        },
        "record_filters": filters,
    }


def main():
    parser = argparse.ArgumentParser(
        description=(
            "Read one or more experiment logs and summarize PRO_POLICY_MATRIX_* "
            "JSON lines into a compact decision digest."
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

    digest = summarize(parse_policy_matrix_lines(args.logs))
    if args.compact:
        json.dump(digest, sys.stdout, sort_keys=True, separators=(",", ":"))
    else:
        json.dump(digest, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
