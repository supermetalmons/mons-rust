#!/usr/bin/env python3
"""Summarize policy-matrix experiment logs into one JSON decision digest."""

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path


POLICY_MATRIX_PREFIX = "PRO_POLICY_MATRIX_"
NO_SOURCE_DECISIONS = {
    "baseline_save_risk",
    "coverage_gap",
    "no_candidate_route",
    "postprocess_only",
    "singleton_no_source",
}


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


def limited_count_rows(counter, limit=8):
    return sorted_count_rows(counter)[:limit]


def count_keys(counter):
    return {key for key, count in counter.items() if count > 0}


def rollup_decision_from_counts(decision_counts):
    decisions = count_keys(decision_counts)
    if not decisions:
        return "no_source"
    if decisions == {"inspect_for_source"}:
        return "inspect_for_source"
    if decisions.issubset(NO_SOURCE_DECISIONS):
        for decision in [
            "baseline_save_risk",
            "coverage_gap",
            "postprocess_only",
            "singleton_no_source",
            "no_candidate_route",
        ]:
            if decision in decisions:
                return decision
    return "mixed_review_required"


def rollup_permission_from_decision(decision):
    if decision == "inspect_for_source":
        return "inspect_for_source"
    if decision in NO_SOURCE_DECISIONS:
        return "no_source"
    return "mixed_review_required"


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
        "coverage_gap_entry_count": digest.get("coverage_gap_entry_count", 0),
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

    rollup_decision = rollup_decision_from_counts(decision_counts)
    digest["log_rollup"] = {
        "log_count": len(log_summaries),
        "rollup_decision": rollup_decision,
        "rollup_next_action": next_action_for_decision(rollup_decision),
        "rollup_permission": rollup_permission_from_decision(rollup_decision),
        "decision_counts": sorted_count_rows(decision_counts),
        "next_action_counts": sorted_count_rows(next_action_counts),
        "source_blocker_counts": sorted_count_rows(source_blocker_counts),
        "log_summaries": log_summaries,
    }
    return digest


def coverage_gap_group_key(record):
    return tuple(
        record.get(field, "")
        for field in [
            "panel",
            "duel",
            "seed_tag",
            "repeat",
            "opening_index",
            "variant",
            "candidate_is_white",
        ]
    )


def add_coverage_gap_record(groups, event):
    record = event["data"]
    if record.get("portfolio_class") != "no_policy_win":
        return

    key = coverage_gap_group_key(record)
    group = groups.setdefault(
        key,
        {
            "panel": record.get("panel", ""),
            "duel": record.get("duel", ""),
            "seed_tag": record.get("seed_tag", ""),
            "repeat": int(record.get("repeat", 0)),
            "opening_index": int(record.get("opening_index", 0)),
            "variant": record.get("variant", ""),
            "candidate_is_white": bool(record.get("candidate_is_white", False)),
            "opening": record.get("opening", ""),
            "policy_results": record.get("policy_results", ""),
            "winning_policies": record.get("winning_policies", ""),
            "source_logs": set(),
            "candidates": set(),
            "outcomes": defaultdict(int),
            "branches": defaultdict(int),
            "pairs": defaultdict(int),
            "mechanism_axes": defaultdict(int),
            "divergences": {},
            "record_count": 0,
        },
    )

    group["record_count"] += 1
    group["source_logs"].add(event["source_log"])
    group["candidates"].add(record.get("candidate", ""))
    group["outcomes"][record.get("outcome", "")] += 1

    branch = f"{record.get('baseline_branch', '')}->{record.get('candidate_branch', '')}"
    pair = f"{record.get('baseline_move', '')}->{record.get('candidate_move', '')}"
    group["branches"][branch] += 1
    group["pairs"][pair] += 1
    mechanism_axes = record.get("mechanism_axes", "")
    for axis in mechanism_axes.split("|"):
        if axis:
            group["mechanism_axes"][axis] += 1

    first_diff_ply = int(record.get("first_diff_ply", -1))
    if first_diff_ply < 0:
        return

    divergence_key = (
        record.get("candidate", ""),
        first_diff_ply,
        branch,
        pair,
    )
    group["divergences"].setdefault(
        divergence_key,
        {
            "candidate": record.get("candidate", ""),
            "outcome": record.get("outcome", ""),
            "first_diff_ply": first_diff_ply,
            "branch": branch,
            "pair": pair,
            "active_color": record.get("active_color", ""),
            "turn": int(record.get("turn", -1)),
            "mons_moves": int(record.get("mons_moves", -1)),
            "can_action": bool(record.get("can_action", False)),
            "can_mana": bool(record.get("can_mana", False)),
            "exact_context": record.get("exact_context", ""),
            "board": record.get("board", ""),
            "baseline_move": record.get("baseline_move", ""),
            "candidate_move": record.get("candidate_move", ""),
            "mechanism_axes": record.get("mechanism_axes", ""),
        },
    )


def sorted_coverage_gap_entries(groups):
    entries = []
    for group in groups.values():
        branches = dict(group["branches"])
        pairs = dict(group["pairs"])
        divergences = sorted(
            group["divergences"].values(),
            key=lambda item: (
                item["first_diff_ply"],
                item["candidate"],
                item["branch"],
                item["pair"],
            ),
        )
        entries.append(
            {
                "panel": group["panel"],
                "duel": group["duel"],
                "seed_tag": group["seed_tag"],
                "repeat": group["repeat"],
                "opening_index": group["opening_index"],
                "variant": group["variant"],
                "candidate_is_white": group["candidate_is_white"],
                "opening": group["opening"],
                "policy_results": group["policy_results"],
                "winning_policies": group["winning_policies"],
                "source_logs": sorted(group["source_logs"]),
                "record_count": group["record_count"],
                "candidate_count": len(group["candidates"]),
                "candidates": "|".join(sorted(group["candidates"])),
                "outcome_counts": sorted_count_rows(group["outcomes"]),
                "branch_count": len(branches),
                "branches": limited_count_rows(branches),
                "pair_count": len(pairs),
                "pairs": limited_count_rows(pairs),
                "top_mechanism_axes": limited_count_rows(group["mechanism_axes"], 5),
                "first_diff_count": len(divergences),
                "divergences": divergences[:5],
            }
        )

    return sorted(
        entries,
        key=lambda item: (
            item["panel"],
            item["duel"],
            item["seed_tag"],
            item["repeat"],
            item["opening_index"],
            item["variant"],
            str(item["candidate_is_white"]),
        ),
    )


def summarize(events):
    latest = {}
    route_buckets = defaultdict(list)
    filter_summaries = {}
    filter_details = defaultdict(list)
    coverage_gap_groups = {}
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
        elif event_type == "PRO_POLICY_MATRIX_CORPUS_RECORD":
            add_coverage_gap_record(coverage_gap_groups, event)

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
    coverage_gap_entries = sorted_coverage_gap_entries(coverage_gap_groups)

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
        "coverage_gap_entry_count": len(coverage_gap_entries),
        "coverage_gap_entries": coverage_gap_entries,
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
