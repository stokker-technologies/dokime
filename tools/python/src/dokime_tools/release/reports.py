# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***
"""Collect Nightly Measurements And Raw Reports"""

# === Imports === #

# Standard Library Imports
import json
import math
import re
import zipfile
from pathlib import Path
from xml.etree import ElementTree

# Third Party Imports
# Local Imports
from .. import policy
from ..build import paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def measurements(
    rows: list[tuple[str, float, float | None]], suite: str, revision: str, threshold: float, *, boundaries: dict[str, str] | None = None
) -> dict:
    """Preserve Every Native Measurement And Its Pre-Run Baseline."""
    benchmarks = []
    for name, value, baseline in rows:
        if not math.isfinite(value) or (baseline is not None and not math.isfinite(baseline)):
            raise policy.ToolError(f"non-finite benchmark measurement: {name}")
        benchmarks.append(
            {
                "name": name,
                **identity(
                    suite,
                    paths.target(),
                    name,
                    (boundaries or {}).get(name, "Native timed iteration; fixture preparation follows the owning harness"),
                ),
                "unit": "ns",
                "value": value,
                "baseline_value": baseline,
                "revision": revision,
                "threshold_percent": threshold,
            }
        )
    return {"benchmarks": benchmarks}


def identity(lane: str, platform: str, case: str, boundary: str, *, state: str | None = None) -> dict:
    """Qualify Measurements At Their Producer, Independently Of Artifact Names."""
    if state is None:
        parts = re.split(r"[/_]", case)
        state = next(
            (part for part in parts if part in {"cold", "warm", "live", "late", "zoom", "resize", "navigation", "stalled"}), "default"
        )
    return {"suite": f"{lane}/{platform}/{state}", "lane": lane, "platform": platform, "case": case, "state": state, "boundary": boundary}


def collect(directory: Path) -> tuple[dict, list[Path]]:
    """Collect This Run's Downloaded Reports Without Requiring Successful Jobs."""
    source = directory / "target/nightly-reports"
    benchmarks = []
    coverage = []
    errors = []
    files = sorted(path for path in source.rglob("*") if path.is_file())
    for path in files:
        if path.is_symlink():
            raise policy.ToolError("nightly reports must not contain linked files")
        try:
            if path.name.startswith("benchmarks-") and path.suffix == ".json":
                benchmarks.extend(json.loads(path.read_text(encoding="utf-8"))["benchmarks"])
            if path.name == "coverage.xml":
                root = ElementTree.fromstring(path.read_bytes())
                parts = path.relative_to(source).parts
                language = next((part for part in parts if part in {"rust", "python", "typescript"}), "unknown")
                coverage.append(
                    {
                        "language": language,
                        "line_percent": float(root.attrib["line-rate"]) * 100,
                        "report": path.relative_to(source).as_posix(),
                    }
                )
        except (ValueError, KeyError, ElementTree.ParseError) as error:
            errors.append({"report": path.relative_to(source).as_posix(), "error": str(error)})
    identities = set()
    for row in benchmarks:
        key = (row["suite"], row["name"])
        if key in identities:
            raise policy.ToolError(f"duplicate benchmark identity in nightly reports: {key!r}")
        identities.add(key)
    return {"coverage": coverage, "benchmarks": benchmarks, "errors": errors}, files


def stage(directory: Path, shared: Path, version: str) -> list[Path]:
    """Package Available Measurements And Raw Evidence In The Release Inventory."""
    summary, files = collect(directory)
    source = directory / "target/nightly-reports"
    index = shared / "benchmarks.json"
    benchmarks = sorted(summary["benchmarks"], key=lambda row: (row["suite"], row["name"]))
    policy.write_json(index, {"benchmarks": benchmarks})
    policy.write_json(shared / "reports.json", summary)
    destination = shared / f"dokime-reports-{version}.zip"
    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.write(index, "benchmarks.json")
        archive.write(shared / "reports.json", "summary.json")
        for path in files:
            archive.write(path, path.relative_to(source).as_posix())
    return [destination, index]


def table(summary: dict, results: dict, url: str) -> str:
    """A Compact Report Of Available Measurements And Every Suite Outcome."""
    lines = ["| Result | Measurement |", "|---|---|"]
    for row in summary["coverage"]:
        lines.append(f"| Coverage / {row['language']} | {row['line_percent']:.2f}% |")
    if not summary["coverage"]:
        lines.append("| Coverage | Unavailable; see suite results |")
    cases = {}
    for row in summary["benchmarks"]:
        case = row.get("case", row["name"])
        key = (row["suite"], case)
        cases.setdefault(key, []).append(row)
    for (suite, case), rows in sorted(cases.items()):
        name = f"Benchmark / {suite} / {case}".replace("|", "\\|").replace("\n", " ")
        boundary = rows[0].get("boundary", "Boundary unavailable in producer report")
        measured = sum(row.get("status") != "not_measured" for row in rows)
        missing = [row["reason"] for row in rows if row.get("status") == "not_measured"]
        result = f"{measured} measurements; {boundary}" if measured else f"Not measured: {'; '.join(missing)}; {boundary}"
        if measured and missing:
            result += f"; Not measured: {'; '.join(missing)}"
        result = result.replace("|", "\\|").replace("\n", " ")
        lines.append(f"| {name} | {result} |")
    for row in summary["benchmarks"]:
        if row.get("status") == "not_measured":
            continue
        baseline = row["baseline_value"]
        if baseline is None or baseline <= 0:
            continue
        change = (row["value"] - baseline) / baseline * 100
        if change > row["threshold_percent"]:
            name = row["name"].replace("|", "\\|").replace("\n", " ")
            lines.append(f"| Benchmark / {row['suite']} / {name} | +{change:.1f}% (limit {row['threshold_percent']:g}%) |")
    for row in summary.get("errors", []):
        name = row["report"].replace("|", "\\|")
        lines.append(f"| Report / {name} | Incomplete; raw output retained |")
    for name, job in sorted(results.items()):
        lines.append(f"| Suite / {name} | {job['result']} |")
    lines.append(f"| Run | [Workflow Results]({url}) |")
    return "\n".join(lines)


# === Private Functions === #

# === Entry Point === #
