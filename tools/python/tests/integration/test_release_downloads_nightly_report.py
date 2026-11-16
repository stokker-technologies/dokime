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
from pathlib import Path
from unittest import mock

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools import policy
from dokime_tools.release import (
    reports,
)

from .environment import isolate_environment

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class NightlyReportTests(unittest.TestCase):
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
            outputs = reports.stage(root, shared, "0.1.3")
            names = set()
            for path in outputs:
                names.add(path.name)
            self.assertEqual(names, {"benchmarks.json", "dokime-reports-0.1.3.zip"})
            values = json.loads((shared / "benchmarks.json").read_text())["benchmarks"]
            self.assertEqual(len(values), 12)
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
            self.assertEqual(len(values), 8)
            identities = set()
            states = set()
            for row in values:
                identities.add((row["suite"], row["name"]))
                states.add(row["state"])
            self.assertEqual(len(identities), 8)
            self.assertEqual(states, {"cold", "warm"})
            notes = reports.table(published, {}, "https://example.test/run")
            for row in values:
                self.assertIn(f"{row['suite']} / {row['case']}", notes)
                self.assertIn(row["boundary"], notes)
            policy.write_json(output / "benchmarks-duplicate.json", summary)
            with self.assertRaisesRegex(policy.ToolError, "duplicate benchmark identity.*window"):
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
