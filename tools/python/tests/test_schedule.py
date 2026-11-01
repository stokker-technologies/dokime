# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Concurrent Lane Completion, Preparation And Failure Evidence."""

# === Imports === #

# Standard Library Imports
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.test import report, run, schedule

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ScheduleTests(unittest.TestCase):
    def test_archive_dispatch(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            command = schedule.command("unit", "rust", "dev", 2, directory)
            with patch.object(run.policy, "root", return_value=directory), patch.object(run, "test", return_value=0) as lane:
                status = run.main(command[3:])
            self.assertEqual(status, 0)
            lane.assert_called_once()
            self.assertEqual(lane.call_args.kwargs["archive_directory"], directory)
            self.assertNotIn("--archive-directory", lane.call_args.args[-1])

    def test_fixture_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            (directory / "tools").mkdir()
            (directory / "tools/project.toml").write_text("", encoding="utf-8")
            (directory / "tests/unit").mkdir(parents=True)

            def execute(command, cwd, environment):
                junit = Path(next(value.removeprefix("--junitxml=") for value in command if value.startswith("--junitxml=")))
                fixtures = Path(next(value.removeprefix("--basetemp=") for value in command if value.startswith("--basetemp=")))
                fixtures.mkdir(parents=True)
                (fixtures / "fixture-only.dat").write_text("test input", encoding="utf-8")
                junit.write_text('<testsuite name="fixture"><testcase name="passes" time="0.01"/></testsuite>', encoding="utf-8")
                return 0

            with patch.object(run, "_execute", side_effect=execute):
                status = run.test(directory, "unit", "python", ".", "dev", [], prepared=True)
            self.assertEqual(status, 0)
            evidence = report.location(directory, "dev", "unit-python")
            self.assertTrue((evidence / "junit.xml").is_file())
            self.assertEqual(list(evidence.rglob("fixture-only.dat")), [])

    def test_concurrent_lanes(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            script = directory / "lane.py"
            # Each Child Waits For The Other: A Serial Scheduler Cannot Complete.
            script.write_text(
                "import os, pathlib, sys, time\n"
                "root = pathlib.Path(sys.argv[1])\n"
                "name = sys.argv[2]\n"
                "assert (root / 'prepared').is_file()\n"
                "assert 'UV_PROJECT_ENVIRONMENT' not in os.environ\n"
                "(root / name).touch()\n"
                "deadline = time.monotonic() + 15\n"
                "while not all((root / peer).exists() for peer in ('left', 'right')):\n"
                "    if time.monotonic() > deadline: raise RuntimeError('lanes did not overlap')\n"
                "    time.sleep(0.01)\n"
                "print(name)\n"
                "sys.exit(int(sys.argv[3]))\n",
                encoding="utf-8",
            )
            (directory / "prepared").touch()
            destination = directory / "reports"
            destination.mkdir()
            commands = {
                "left": [sys.executable, str(script), str(directory), "left", "7"],
                "right": [sys.executable, str(script), str(directory), "right", "0"],
                "missing": [str(directory / "absent-executable")],
            }
            with patch.dict(os.environ, {"UV_PROJECT_ENVIRONMENT": "must-not-inherit"}):
                rows = schedule.execute(directory, commands, destination)
            results = {row["lane"]: row for row in rows}
            self.assertEqual(results["left"]["exit_code"], 7)
            self.assertEqual(results["right"]["exit_code"], 0)
            self.assertNotEqual(results["missing"]["exit_code"], 0)
            self.assertEqual((directory / results["left"]["log"]).read_text().strip(), "left")
            self.assertEqual((directory / results["right"]["log"]).read_text().strip(), "right")

    def test_instrumented_liveness_budget(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            (directory / "tools").mkdir()
            (directory / "tools/project.toml").write_text("", encoding="utf-8")
            rules = run.policy.load(directory)
            ordinary = rules.deadline("unit")
            instrumented = rules.deadline("unit", coverage=True)
            for environment, seconds, expected in (
                ({}, (ordinary + instrumented) / 2, "failed"),
                ({run.COVERAGE_VARIABLE: "1"}, (ordinary + instrumented) / 2, "passed"),
                ({run.COVERAGE_VARIABLE: "1"}, instrumented + 1, "failed"),
            ):
                with run._scratch(directory, "unit-python", "dev") as scratch:
                    junit = scratch / "junit.xml"
                    junit.write_text(f'<testsuite name="fixture"><testcase name="passes" time="{seconds}"/></testsuite>', encoding="utf-8")
                    with patch.object(run, "_execute", return_value=0):
                        result = run._lane(
                            directory, rules, "unit-python", "unit", "python", ".", "dev", [], directory, environment, junit, scratch
                        )
                self.assertEqual(result["outcome"], expected)

    def test_coverage_liveness_layers(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            (directory / "tools").mkdir()
            (directory / "tools/project.toml").write_text("[conventions.testing.deadlines]\nunit = 30\nacceptance = 60\n", encoding="utf-8")
            rules = run.policy.load(directory)
            seconds = rules.deadline("unit", coverage=True) + 1
            with run._scratch(directory, "coverage-python", "dev") as scratch:
                junit = scratch / "junit.xml"
                junit.write_text(
                    f'<testsuite><testcase classname="tests.unit.client" name="overrun" time="{seconds}"/>'
                    f'<testcase classname="tests.acceptance.backend" file="tests\\acceptance\\test_backend.py" name="within_budget" time="{seconds}"/></testsuite>',
                    encoding="utf-8",
                )
                with patch.object(run, "_execute", return_value=0):
                    result = run._lane(
                        directory,
                        rules,
                        "coverage-python",
                        "coverage",
                        "python",
                        ".",
                        "dev",
                        [],
                        directory,
                        {run.COVERAGE_VARIABLE: "1"},
                        junit,
                        scratch,
                    )
            self.assertEqual((result["passed"], result["failed"]), (1, 1))
            self.assertEqual(result["overruns"], ["tests.unit.client::overrun"])

    def test_aggregate_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            installed = directory / "wheel-ready"
            generated = directory / "contracts-ready"
            fixture = (
                "import pathlib, sys; "
                f"assert pathlib.Path({str(installed)!r}).is_file(); "
                f"assert pathlib.Path({str(generated)!r}).is_file(); "
                "print('fixture complete'); sys.exit(3)"
            )
            with (
                patch.object(schedule.policy, "root", return_value=directory),
                patch.object(schedule.policy, "lanes", return_value=[("integration", "python")]),
                patch.object(schedule.archive, "install_wheels", side_effect=lambda *_: installed.touch()),
                patch.object(schedule.generated, "restore", side_effect=lambda *_: generated.touch()),
                patch.object(schedule, "command", return_value=[sys.executable, "-c", fixture]),
            ):
                status = schedule.main([])
            self.assertNotEqual(status, 0)
            summary = next(directory.glob("target/test-reports/run-*/time-summary.json"))
            result = json.loads(summary.read_text(encoding="utf-8"))
            self.assertEqual(result["lanes"][0]["exit_code"], 3)
            self.assertIn("fixture complete", (directory / result["lanes"][0]["log"]).read_text())
            self.assertTrue(summary.with_suffix(".md").is_file())

    def test_shared_archive(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            fixture = (
                "import pathlib, sys; "
                "shared = pathlib.Path(sys.argv[sys.argv.index('--archive-directory') + 1]); "
                "assert (shared / 'ready').read_text() == 'extracted'; "
                f"(pathlib.Path({str(directory)!r}) / sys.argv[1]).write_text(str(shared))"
            )

            def extract(root, profile, destination):
                (destination / "ready").write_text("extracted", encoding="utf-8")

            with (
                patch.object(schedule.policy, "root", return_value=directory),
                patch.object(schedule.policy, "lanes", return_value=[("unit", "rust"), ("system", "rust")]),
                patch.object(schedule.archive, "install_wheels"),
                patch.object(schedule.archive, "extract", side_effect=extract) as extraction,
                patch.object(schedule.generated, "restore"),
                patch.object(
                    schedule,
                    "command",
                    side_effect=lambda category, language, profile, workers, extracted: [
                        sys.executable,
                        "-c",
                        fixture,
                        category,
                        "--archive-directory",
                        str(extracted),
                    ],
                ),
            ):
                status = schedule.main([])
            self.assertEqual(status, 0)
            extraction.assert_called_once()
            self.assertEqual((directory / "unit").read_text(), (directory / "system").read_text())


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
