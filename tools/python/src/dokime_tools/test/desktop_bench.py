# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Retain Desktop Stage and Resource Observations in the Existing Nightly Inventory."

# === Imports === #

# Standard Library Imports
import json
import math
import shutil
from pathlib import Path
from statistics import median

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from .. import policy
from ..build import paths
from ..release import reports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def collect(directory: Path, home: Path, identity: dict, threshold: float, *, refresh: bool, success: bool) -> None:
    """Publish Explicit Missing Stages and Refusals Alongside Numeric Observations."""
    source = home / "observations.json"
    if not source.is_file():
        raise ValueError("Run Desktop Bench Tests: desktop benchmark wrote no observations")
    current = json.loads(source.read_text(encoding="utf-8"))
    stored = home / "dev.json"
    previous = json.loads(stored.read_text(encoding="utf-8")) if stored.is_file() else {}
    baselines = {(row["name"], row["unit"]): row["value"] for row in previous.get("benchmarks", [])}
    groups = {}
    for result in current["results"]:
        history_rows = result.get("rows", current["fixture"]["rows"])
        groups.setdefault((history_rows, result["count"], result["condition"]), []).append(result)
    revision = policy.execute(directory, ["git", "rev-parse", "HEAD"], capture=True).strip()
    rows = []
    for (history_rows, count, condition), samples in groups.items():
        prefix = f"desktop/{history_rows}/{count}/{condition}"
        boundary = (
            "Document request through bounded native delivery and visible draw; host and desktop reopen excluded; OS cache uncontrolled"
            if condition in {"cold", "warm"}
            else "Prepared view: producer handoff or named action through native delivery and visible draw; no publication durability barrier"
        )
        qualified = reports.identity("rust/desktop", paths.target(), prefix, boundary, state=condition)
        metrics = {}
        for sample in samples:
            for stage, values in sample["backendStages"].items():
                metrics.setdefault((stage, "ms"), []).append(sum(values))
            for stage, values in sample["stages"].items():
                metrics.setdefault((stage, "ms"), []).append(sum(values))
                metrics.setdefault((f"{stage}_call", "ms"), []).extend(values)
            if sample["queryMs"]:
                metrics.setdefault(("backend_query", "ms"), []).append(sum(sample["queryMs"]))
                metrics.setdefault(("backend_query_call", "ms"), []).extend(sample["queryMs"])
            for field, unit in (
                ("firstVisibleMs", "ms"),
                ("allVisibleMs", "ms"),
                ("durationMs", "ms"),
                ("resourceWindowMs", "ms"),
                ("cpuSeconds", "s"),
                ("peakMemoryBytes", "bytes"),
                ("bytes", "bytes"),
                ("queueDepth", "frames"),
                ("frameMisses", "frames"),
            ):
                value = sample.get(field)
                if value is not None:
                    metrics.setdefault((field, unit), []).append(value)
        refusal_count = sum(sample["outcome"] == "refused" for sample in samples)
        for (metric, unit), values in metrics.items():
            ordered = sorted(values)
            if not ordered or not all(math.isfinite(value) for value in ordered):
                raise ValueError(f"Run Desktop Bench Tests: invalid desktop samples: {prefix}/{metric}")
            for quantile, value in (("p50", median(ordered)), ("p95", ordered[max(0, math.ceil(len(ordered) * 0.95) - 1)])):
                name = f"{prefix}/{metric}/{quantile}"
                rows.append(
                    {
                        "name": name,
                        **qualified,
                        "unit": unit,
                        "value": value,
                        "baseline_value": baselines.get((name, unit)),
                        "revision": revision,
                        "threshold_percent": threshold,
                        "refused_samples": refusal_count,
                        "samples": len(ordered),
                    }
                )
        rows.append(
            {
                "name": f"{prefix}/refusals",
                **qualified,
                "unit": "cases",
                "value": refusal_count,
                "baseline_value": None,
                "revision": revision,
                "threshold_percent": threshold,
                "failures": sorted({error for sample in samples for error in sample["errors"]}),
                "partial_samples": sum(sample["outcome"] == "partial" for sample in samples),
                "missing_measurements": [
                    metric
                    for metric in (
                        "backend_query",
                        "plot.watch.frames",
                        "plot.watch.send",
                        "transport_bridge_wait",
                        "protobuf_decode",
                        "native_frame_adaptation",
                        "retained_update",
                        "canvas_draw",
                        "presentation_opportunity",
                        "firstVisibleMs",
                        "allVisibleMs",
                        "cpuSeconds",
                        "peakMemoryBytes",
                    )
                    if not any(name == metric for name, _unit in metrics)
                ],
                "unavailable": [
                    "physical_display_presentation",
                    "exclusive_encoding_native_bridge",
                    "os_cache_cold",
                    "native_server_queue_depth",
                ],
                "fixture": samples[0].get("fixture", current["fixture"]),
            }
        )
    completion_path = home / "completion.json"
    completion = json.loads(completion_path.read_text(encoding="utf-8")) if completion_path.is_file() else {}
    if not success:
        reason = (
            completion.get("failure")
            or "; ".join(completion.get("errors", []))
            or "Native desktop benchmark exited without completing measurements"
        )
        for history_rows in (2_073_600, 307_200, 1_536_000, 3_072_000):
            conditions = (
                ("cold", "warm", "live", "late", "zoom", "resize", "navigation", "stalled")
                if history_rows == 2_073_600
                else ("cold", "warm", "live")
            )
            for count in (30, 100):
                for condition in conditions:
                    if (history_rows, count, condition) in groups and completion.get("measuring") != f"{history_rows}/{count}/{condition}":
                        continue
                    case = f"desktop/{history_rows}/{count}/{condition}"
                    rows.append(
                        {
                            "name": case,
                            **reports.identity(
                                "rust/desktop",
                                paths.target(),
                                case,
                                "Document request or prepared-view action through visible draw",
                                state=condition,
                            ),
                            "status": "not_measured",
                            "reason": reason,
                            "value": None,
                            "baseline_value": None,
                            "revision": revision,
                        }
                    )
    summary = {
        "benchmarks": rows,
        "identity": identity,
        "fixture": current["fixture"],
        "unavailable": [
            "Physical display presentation (double requestAnimationFrame measures only a presentation opportunity)",
            "Exclusive encoding and native bridge time (transport_bridge_wait includes backend work and idle wait)",
            "OS disk-cache-cold reads (cold reopens a closed history with a fresh backend and desktop)",
            "Native bridge/server queue depth (queueDepth observes the actual WebView receive queue)",
        ],
    }
    policy.write_json(home / "new.json", summary)
    policy.write_json(directory / "target/benchmarks/benchmarks-rust-desktop.json", summary)
    if success and refresh:
        shutil.copyfile(home / "new.json", stored)


# === Private Functions === #

# === Entry Point === #
