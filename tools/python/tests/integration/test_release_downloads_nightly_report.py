# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Nightly Report Tests."""

# === Imports === #

# Standard Library Imports
import json
import tempfile
import unittest
import zipfile
from itertools import product
from pathlib import Path
from unittest import mock

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.ci.matrices import SDK_LANGUAGES
from dokime_tools.release import reports
from dokime_tools.test import desktop_bench

from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class NightlyReportTests(unittest.TestCase):
    def test_sdk_comparison_composes_partial_jobs_and_refuses_ambiguous_values(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary.name)
            for platform, multiplier in (("linux-x86_64", 1), ("windows-x86_64", 10)):
                for language, value in zip(SDK_LANGUAGES, (12, 24, 18, 9)):
                    rows = []
                    for measure, unit in (("round_trip/echo/p50", "ns"), ("throughput/ceiling", "samples/s")):
                        name = f"sdk/{language}/{measure}"
                        rows.append(
                            {
                                "name": name,
                                **reports.identity(f"sdk/{language}", platform, name, "echo"),
                                "unit": unit,
                                "value": value * multiplier,
                                "baseline_value": None,
                                "threshold_percent": 20,
                            }
                        )
                    output = root / "target/nightly-reports" / platform / language / f"benchmarks-sdk-{language}.json"
                    policy.write_json(output, {"benchmarks": rows})
            summary, _ = reports.collect(root)
            comparison = reports.sdk_comparison(summary["benchmarks"])
            self.assertTrue(reports.table(summary, {}, "https://example.test/run").endswith(comparison))

            # Include Partial Jobs Without Inventing Measurements.
            rows = summary["benchmarks"][:2]
            rows[1] = {**rows[1], "status": "not_measured", "reason": "memory|unavailable\nplatform", "value": None}
            reports.sdk_comparison(rows)

            # Refuse Ambiguous Values Instead of Choosing One Job's Row.
            with self.assertRaises(policy.ToolError):
                reports.sdk_comparison([rows[0], rows[0]])
            for value in (float("nan"), float("inf"), True, None):
                with self.assertRaises(policy.ToolError):
                    reports.sdk_comparison([{**rows[0], "value": value}])
            self.assertEqual(reports.sdk_comparison([]), "")
        finally:
            temporary.cleanup()

    def test_failed_desktop_split_jobs_merge_only_their_selected_cases(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary_directory.name)
            expected = {
                "round-trip": {
                    f"desktop/2073600/{count}/{condition}"
                    for count, condition in product((30, 100), ("cold", "warm", "live", "late", "zoom", "resize", "navigation", "stalled"))
                },
                "history": {
                    f"desktop/{rows}/{count}/{condition}"
                    for rows, count, condition in product((307200, 1536000, 3072000), (30, 100), ("cold", "warm", "live"))
                },
            }
            missing = {}

            # Collect Both Failed Jobs Through the Desktop Reporter.
            for job, selection in (("round-trip", "^desktop_round_trip/"), ("history", "^desktop_history_round_trip/")):
                home = root / job
                home.mkdir()
                policy.write_json(home / "observations.json", {"fixture": None, "results": []})
                policy.write_json(home / "completion.json", {"failure": "Native Initialization Failed", "measuring": "2073600/30/cold"})
                with (
                    mock.patch.object(desktop_bench.policy, "execute", return_value="candidate"),
                    mock.patch.object(desktop_bench.paths, "target", return_value="windows-x86_64"),
                ):
                    desktop_bench.collect(root, home, {"arguments": [selection]}, 20, refresh=True, success=False)
                summary = json.loads((home / "new.json").read_text())
                self.assertTrue(all(row["status"] == "not_measured" and row["value"] is None for row in summary["benchmarks"]))
                missing[job] = {row["case"] for row in summary["benchmarks"]}
                self.assertEqual(missing[job], expected[job])
                output = root / "target/nightly-reports" / job / "benchmarks-rust-desktop.json"
                policy.write_json(output, summary)

            # Merge Disjoint Failure Reports Without Losing Any Case.
            self.assertTrue(missing["round-trip"].isdisjoint(missing["history"]))
            published, _ = reports.collect(root)
            self.assertEqual({row["case"] for row in published["benchmarks"]}, expected["round-trip"] | expected["history"])

            # Apply an Anchored Case Filter Alongside Diagnostic Mode.
            selection = "^desktop_history_round_trip/1536000/(30|100)/warm$"
            with (
                mock.patch.object(desktop_bench.policy, "execute", return_value="candidate"),
                mock.patch.object(desktop_bench.paths, "target", return_value="windows-x86_64"),
            ):
                desktop_bench.collect(root, home, {"arguments": ["--test", selection]}, 20, refresh=False, success=False)
            summary = json.loads((home / "new.json").read_text())
            self.assertEqual({row["case"] for row in summary["benchmarks"]}, {"desktop/1536000/30/warm", "desktop/1536000/100/warm"})

            # Keep Completed Measurements Inside the Same Selection.
            sample = {"count": 30, "condition": "warm", "outcome": "drawn", "errors": [], "backendStages": {}, "stages": {}, "queryMs": []}
            policy.write_json(
                home / "observations.json",
                {"fixture": {"rows": 2073600}, "results": [sample, {**sample, "rows": 1536000, "durationMs": 7}]},
            )
            with (
                mock.patch.object(desktop_bench.policy, "execute", return_value="candidate"),
                mock.patch.object(desktop_bench.paths, "target", return_value="windows-x86_64"),
            ):
                desktop_bench.collect(root, home, {"arguments": [selection]}, 20, refresh=False, success=False)
            summary = json.loads((home / "new.json").read_text())
            rows = {row["name"]: row for row in summary["benchmarks"]}
            self.assertEqual(rows["desktop/1536000/30/warm/durationMs/p50"]["value"], 7)
        finally:
            temporary_directory.cleanup()

    def test_collects_all_suites_baselines_and_coverage(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            root = Path(directory)
            shared = root / "shared"
            shared.mkdir()
            for suite in ("unit", "integration", "integration-plots", "system"):
                output = root / "target/nightly-reports" / suite
                output.mkdir(parents=True)
                rows = [(suite + "/fast", 10.0, 20.0), (suite + "/regressed", 40.0, 20.0), (suite + "/seeded", 1.0, None)]
                with mock.patch.object(reports.paths, "target", return_value="windows-x86_64"):
                    measurements = reports.measurements(rows, "rust/" + suite, "a" * 40, 15)
                policy.write_json(output / f"benchmarks-rust-{suite}.json", measurements)
            coverage = root / "target/nightly-reports/coverage/rust/coverage.xml"
            coverage.parent.mkdir(parents=True)
            coverage.write_text('<coverage line-rate="0.75"/>')
            reports.stage(root, shared, "0.1.3")
            values = json.loads((shared / "benchmarks.json").read_text())["benchmarks"]
            lanes = set()
            for row in values:
                lanes.add(row["lane"])
            expected = set()
            for suite in ("unit", "integration", "integration-plots", "system"):
                expected.add("rust/" + suite)
            self.assertEqual(lanes, expected)
            self.assertTrue(all(row["unit"] == "ns" and row["revision"] == "a" * 40 for row in values))
            summary = json.loads((shared / "reports.json").read_text())
            self.assertEqual(summary["coverage"][0]["line_percent"], 75.0)
            notes = reports.table(summary, {"bench": {"result": "failure"}}, "https://example.test/run")
            self.assertIn("100.0%", notes)
            self.assertIn("/fast", notes)
            archive = zipfile.ZipFile(shared / "dokime-reports-0.1.3.zip")
            try:
                self.assertIn("summary.json", archive.namelist())
                self.assertIn("coverage/rust/coverage.xml", archive.namelist())
            finally:
                archive.close()
        finally:
            directory_directory.cleanup()

    def test_platform_lane_and_state_keep_every_case_and_refuse_duplicates(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            shared = root / "shared"
            shared.mkdir()
            for platform in ("linux-x86_64", "windows-x86_64"):
                for lane in ("rust/integration", "rust/system"):
                    output = root / "target/nightly-reports" / platform / lane
                    output.mkdir(parents=True)
                    rows = []
                    for state, value in (("warm", 10), ("cold", 20)):
                        rows.append((f"crates/example/window_{state}/10", value, None))
                    boundaries = {}
                    for name, _, _ in rows:
                        boundaries[name] = "store read through reduced points"
                    with mock.patch.object(reports.paths, "target", return_value=platform):
                        summary = reports.measurements(rows, lane, "a" * 40, 15, boundaries=boundaries)
                    policy.write_json(output / "benchmarks-window.json", summary)
            reports.stage(root, shared, "0.1.3")
            published = json.loads((shared / "reports.json").read_text())
            values = published["benchmarks"]
            states = set()
            for row in values:
                states.add(row["state"])
            self.assertEqual(states, {"cold", "warm"})
            notes = reports.table(published, {}, "https://example.test/run")
            for row in values:
                self.assertIn(f"{row['suite']} / {row['case']}", notes)
                self.assertIn(row["boundary"], notes)
            policy.write_json(output / "benchmarks-duplicate.json", summary)
            with self.assertRaises(policy.ToolError):
                reports.collect(root)
        finally:
            temporary_directory.cleanup()

    def test_failed_jobs_without_reports_still_produce_assets(self) -> None:
        directory_directory = tempfile.TemporaryDirectory()
        directory = directory_directory.name
        try:
            shared = Path(directory)
            partial = shared / "target/nightly-reports/coverage/python/coverage.xml"
            partial.parent.mkdir(parents=True)
            partial.write_text("<coverage")
            reports.stage(shared, shared, "0.1.3")
            summary = json.loads((shared / "reports.json").read_text())
            self.assertEqual(len(summary["errors"]), 1)
            self.assertEqual(json.loads((shared / "benchmarks.json").read_text()), {"benchmarks": []})
            self.assertTrue((shared / "dokime-reports-0.1.3.zip").is_file())
        finally:
            directory_directory.cleanup()


# === Public Functions === #


def setUpModule() -> None:
    isolate_environment()


# === Private Functions === #

# === Entry Point === #
