# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Summarize Payload-Free Test Span Lifetimes"""

# === Imports === #

# Standard Library Imports
import json
from collections import defaultdict
from pathlib import Path
from typing import Any

# Third Party Imports
# Local Imports
from . import policy, report

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def summarize(destination: Path, junit: Path) -> None:
    """Summarize Lifetimes; Overlapping Children Count Once Against Their Parent."""
    totals: dict[tuple[str, str], dict[str, Any]] = {}
    incomplete: list[str] = []
    files = sorted((destination / "traces").glob("*.pftrace"))
    for path in files:
        try:
            document = json.loads(path.read_text(encoding="utf-8"))
            events = document["traceEvents"]
        except (OSError, ValueError, KeyError, TypeError):
            incomplete.append(path.name)
            continue
        spans = [event for event in events if event.get("ph") == "X"]
        children: dict[tuple[int, int], list[tuple[float, float]]] = defaultdict(list)
        for event in spans:
            parent = event.get("args", {}).get("parent_id")
            if parent is not None:
                children[(event["pid"], parent)].append((event["ts"], event["ts"] + event["dur"]))
        for event in spans:
            start, duration = event["ts"], event["dur"]
            identity = (event["pid"], event.get("args", {}).get("span_id"))
            covered = _union(children.get(identity, []), start, start + duration)
            key = (event.get("cat", ""), event["name"])
            row = totals.setdefault(key, {"target": key[0], "name": key[1], "count": 0, "total_ms": 0.0, "self_ms": 0.0})
            row["count"] += 1
            row["total_ms"] += duration / 1000
            row["self_ms"] += max(0, duration - covered) / 1000
    by_total = sorted(totals.values(), key=lambda row: (-row["total_ms"], row["target"], row["name"]))
    by_self = sorted(totals.values(), key=lambda row: (-row["self_ms"], row["target"], row["name"]))
    tests: list[dict[str, Any]] = []
    junit_error = None
    if junit.is_file():
        try:
            cases = report.cases(junit.read_text(encoding="utf-8"), "", "rust", "")
        except (OSError, ValueError, policy.TestError):
            cases = []
            junit_error = "JUnit report could not be read"
        for case in cases:
            tests.append({"suite": case.suite, "name": case.name, "total_ms": case.seconds * 1000, "outcome": case.outcome})
    else:
        junit_error = "JUnit report was not produced"
    tests.sort(key=lambda row: (-row["total_ms"], row["suite"], row["name"]))
    document = {
        "trace_files": len(files),
        "junit_error": junit_error,
        "incomplete_files": incomplete,
        "by_total": by_total[:30],
        "by_self": by_self[:30],
        "tests": tests,
        "slowest_tests": tests[:30],
    }
    report.dump(destination / "time-summary.json", document)
    lines = [
        "# Lane Time Summary",
        "",
        f"Trace Files: {len(files)}; Unreadable Files: {len(incomplete)}",
        "",
        "Durations are elapsed lifetimes, including async waits. Self time subtracts the union of direct child lifetimes.",
        "Overlapping spans and processes are not additive; these values are not CPU time.",
        "",
    ]
    for heading, rows in [("Top 30 Spans By Total Duration", by_total[:30]), ("Top 30 Spans By Self Time", by_self[:30])]:
        lines += [f"## {heading}", "", "| Span | Count | Total (ms) | Self (ms) |", "| --- | ---: | ---: | ---: |"]
        lines += [
            f"| {_cell(row['target'] + '::' + row['name'])} | {row['count']} | {row['total_ms']:.3f} | {row['self_ms']:.3f} |"
            for row in rows
        ]
        lines += [""]
    for heading, rows in [("Slowest Tests", tests[:30]), ("Per-Test Totals", tests)]:
        lines += [f"## {heading}", "", "| Test | Total (ms) | Outcome |", "| --- | ---: | --- |"]
        lines += [f"| {_cell(row['suite'] + '::' + row['name'])} | {row['total_ms']:.3f} | {row['outcome']} |" for row in rows]
        lines += [""]
    if junit_error:
        lines += [junit_error, ""]
    if incomplete:
        lines += ["## Unreadable Traces", "", *[f"- {_cell(name)}" for name in incomplete], ""]
    report.write(destination / "time-summary.md", "\n".join(lines))


# === Private Functions === #


def _union(intervals: list[tuple[float, float]], start: float, end: float) -> float:
    covered, through = 0.0, start
    for left, right in sorted(intervals):
        left, right = max(start, left), min(end, right)
        if right > max(through, left):
            covered += right - max(through, left)
            through = right
    return covered


def _cell(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ").replace("\r", " ")


# === Entry Point === #
