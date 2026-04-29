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
        events.extend(parse_policy_matrix_log(path))
    return events


def parse_policy_matrix_log(path):
    events = []
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


def next_action_for_decision(decision):
    if decision == "inspect_for_source":
        return "inspect_filtered_records"
    if decision == "coverage_gap":
        return "add_policy_or_root_feature"
    if decision == "baseline_save_risk":
        return "avoid_selector"
    if decision == "postprocess_only":
        return "build_outcome_corpus_v2"
    if decision == "singleton_no_source":
        return "widen_or_archive_singleton"
    if decision == "no_candidate_route":
        return "try_next_slice"
    return "keep_postprocess"


def source_blocker_for_decision(decision, summary, stoplight, recommendation):
    if decision == "inspect_for_source":
        return {"kind": "none"}
    if decision == "coverage_gap":
        return {
            "kind": "coverage_gap",
            "no_policy_wins": int(summary.get("no_policy_wins", 0)),
            "stoplight": stoplight.get("label", ""),
        }
    if decision == "baseline_save_risk":
        return {
            "kind": "baseline_save_risk",
            "route_key": recommendation.get("best_baseline_risk_key", ""),
            "candidate_only_states": int(
                recommendation.get("best_baseline_risk_candidate_only_states", 0)
            ),
            "baseline_better_states": int(
                recommendation.get("best_baseline_risk_baseline_better_states", 0)
            ),
        }
    if decision == "postprocess_only":
        return {
            "kind": "fragmented_routes",
            "clean_fragmented_routes": int(
                recommendation.get("clean_fragmented_routes", 0)
            ),
            "clean_low_fragmentation_routes": int(
                recommendation.get("clean_low_fragmentation_routes", 0)
            ),
        }
    if decision == "singleton_no_source":
        return {
            "kind": "singleton_candidate_routes",
            "candidate_signal_routes": int(
                recommendation.get("candidate_signal_routes", 0)
            ),
        }
    if decision == "no_candidate_route":
        return {
            "kind": "no_candidate_route",
            "candidate_signal_routes": int(
                recommendation.get("candidate_signal_routes", 0)
            ),
        }
    return {"kind": "unknown"}


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


def summarized_global_counts(summary):
    fields = [
        "total_games",
        "baseline_wins",
        "candidate_any_wins",
        "candidate_only_wins",
        "baseline_only_wins",
        "no_policy_wins",
    ]
    return {field: int(summary.get(field, 0)) for field in fields}


def summarized_recommendation_counts(recommendation):
    fields = [
        "candidate_signal_routes",
        "clean_low_fragmentation_routes",
        "clean_fragmented_routes",
        "baseline_risk_routes",
        "best_clean_candidate_only_states",
        "best_baseline_risk_candidate_only_states",
        "best_baseline_risk_baseline_better_states",
    ]
    return {field: int(recommendation.get(field, 0)) for field in fields}


def source_blocker_count_key(blocker):
    kind = blocker.get("kind", "unknown")
    if kind == "baseline_save_risk":
        route_key = blocker.get("route_key", "")
        return f"{kind}:{route_key}" if route_key else kind
    return kind


def sorted_count_rows(counter):
    return [
        {"key": key, "count": count}
        for key, count in sorted(counter.items(), key=lambda item: (-item[1], item[0]))
    ]


def log_summary(source_log, digest):
    recommendation = digest.get("route_recommendation", {})
    stoplight = digest.get("global_stoplight", {})
    return {
        "source_log": source_log,
        "event_counts": digest.get("event_counts", {}),
        "corpus_decision": digest.get("corpus_decision", ""),
        "next_action": digest.get("next_action", ""),
        "source_blocker": digest.get("source_blocker", {}),
        "route_permission": digest.get("route_permission", ""),
        "global_counts": summarized_global_counts(digest.get("global_summary", {})),
        "stoplight_label": stoplight.get("label", ""),
        "route_recommendation_label": recommendation.get("label", ""),
        "route_counts": summarized_recommendation_counts(recommendation),
    }


def add_log_rollup(digest, per_log_digests):
    if len(per_log_digests) <= 1:
        return digest

    decision_counts = defaultdict(int)
    next_action_counts = defaultdict(int)
    source_blocker_counts = defaultdict(int)
    log_summaries = []

    for source_log, per_log_digest in per_log_digests:
        summary = log_summary(source_log, per_log_digest)
        log_summaries.append(summary)
        decision_counts[summary["corpus_decision"]] += 1
        next_action_counts[summary["next_action"]] += 1
        source_blocker_counts[
            source_blocker_count_key(summary["source_blocker"])
        ] += 1

    digest["log_rollup"] = {
        "log_count": len(log_summaries),
        "decision_counts": sorted_count_rows(decision_counts),
        "next_action_counts": sorted_count_rows(next_action_counts),
        "source_blocker_counts": sorted_count_rows(source_blocker_counts),
        "log_summaries": log_summaries,
    }
    return digest


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

    decision = corpus_decision(global_summary, stoplight, recommendation)

    return {
        "event_counts": dict(sorted(event_counts.items())),
        "global_summary": global_summary,
        "global_stoplight": stoplight,
        "route_recommendation": recommendation,
        "route_permission": permission_from_recommendation(recommendation),
        "corpus_decision": decision,
        "next_action": next_action_for_decision(decision),
        "source_blocker": source_blocker_for_decision(
            decision, global_summary, stoplight, recommendation
        ),
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

    per_log_events = [(path, parse_policy_matrix_log(path)) for path in args.logs]
    digest = summarize(
        [
            event
            for _source_log, events in per_log_events
            for event in events
        ]
    )
    digest = add_log_rollup(
        digest,
        [(str(source_log), summarize(events)) for source_log, events in per_log_events],
    )
    if args.compact:
        json.dump(digest, sys.stdout, sort_keys=True, separators=(",", ":"))
    else:
        json.dump(digest, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
