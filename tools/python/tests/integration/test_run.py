# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Prove Story Results Ignore Measurements and Script Hangs Terminate."

# === Imports === #

# Standard Library Imports
import io
import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import Mock, patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build import paths
from dokime_tools.test import archive, benchmark, credentials, execution, job, lanes, policy, report

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[4]

# === Types === #

# === Classes === #


class RunTests(unittest.TestCase):
    def test_unit_lane_owns_credentials_through_success_and_failure(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        try:
            directory = Path(temporary_directory.name).resolve()
            scratch = directory / "scratch"
            scratch.mkdir()
            junit = scratch / "junit.xml"
            junit.write_text('<testsuite><testcase name="passes"/></testsuite>', encoding="utf-8")
            rules = policy.load(ROOT)
            store = Mock(environment={"DBUS_SESSION_BUS_ADDRESS": "unix:path=/private-credentials"})
            with (
                patch.object(credentials, "Store", return_value=store) as create,
                patch.object(execution, "execute", return_value=0) as execute,
            ):
                result = lanes.record(directory, rules, "unit-rust", "unit", "rust", ".", "dev", [], directory, {}, junit, scratch)
                self.assertEqual(result["outcome"], "passed")
                create.assert_called_once()
                self.assertIs(execute.call_args.args[2], store.environment)
                store.close.assert_called_once()

                execute.side_effect = RuntimeError("Controlled Runner Failure")
                self.assertRaises(
                    RuntimeError, lanes.record, directory, rules, "unit-rust", "unit", "rust", ".", "dev", [], directory, {}, junit, scratch
                )
                self.assertEqual(store.close.call_count, 2)
        finally:
            temporary_directory.cleanup()

    def test_benchmark_baseline_refresh_and_comparison(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            directory = Path(temporary).resolve()
            (directory / "tools").mkdir()
            (directory / "tools/project.toml").write_text("", encoding="utf-8")
            build = directory / "build"
            (build / "bench").mkdir(parents=True)
            artifact = {
                "reason": "compiler-artifact",
                "target": {"name": "system", "kind": ["bench"]},
                "manifest_path": "crates/example/Cargo.toml",
                "executable": "build/system",
            }
            (build / "bench/compiler.json").write_text(json.dumps(artifact), encoding="utf-8")
            (directory / "crates/example").mkdir(parents=True)
            median = 100.0
            native_measure = execution.measure

            def measure(command: list[str], cwd: Path, environment: dict[str, str], *, output: Path) -> tuple[int, int | None]:
                status, _peak = native_measure(
                    [
                        sys.executable,
                        "-c",
                        "import sys; print('Boundary | Scalar | producer handoff through durable acknowledgement', flush=True); print('Benchmarking sidecar_ingest/scalar/0', file=sys.stderr)",
                    ],
                    cwd,
                    environment,
                    output=output,
                )
                self.assertEqual(status, 0)
                home = Path(environment["CRITERION_HOME"])
                output = home / "scalar/new"
                output.mkdir(parents=True, exist_ok=True)
                (output / "benchmark.json").write_text(json.dumps({"full_id": "sidecar_ingest/scalar/0"}), encoding="utf-8")
                estimates = json.dumps({"median": {"point_estimate": median}})
                (output / "estimates.json").write_text(estimates, encoding="utf-8")
                return 0, None

            with (
                patch.dict(os.environ, {"GITHUB_SHA": "a" * 40}),
                patch.object(lanes, "preflight"),
                patch.object(archive, "_toolchain", return_value="fixture"),
                patch.object(archive, "fixtures"),
                patch.object(paths, "output", return_value=build),
                patch.object(execution, "measure", side_effect=measure),
                patch.object(time, "time", return_value=0),
            ):
                output = io.StringIO()
                with redirect_stdout(output):
                    status = benchmark.run(directory, "system", "rust", ".", False, [])
                self.assertEqual(status, 1)
                self.assertIn("sidecar_ingest/scalar/0", output.getvalue())
                self.assertIn("mise run bench -- --language rust --layer system --refresh-baseline", output.getvalue())
                self.assertEqual(list(directory.rglob("dev/estimates.json")), [])

                output = io.StringIO()
                with redirect_stdout(output):
                    status = benchmark.run(directory, "system", "rust", ".", True, [])
                self.assertEqual(status, 0)
                self.assertIn("::notice::", output.getvalue())
                self.assertIn("sidecar_ingest/scalar/0", output.getvalue())
                (stored,) = directory.rglob("dev/estimates.json")
                self.assertEqual(json.loads(stored.read_text())["median"]["point_estimate"], 100.0)

                median = 1000.0
                for refresh in (False, True):
                    output = io.StringIO()
                    with redirect_stdout(output):
                        status = benchmark.run(directory, "system", "rust", ".", refresh, [])
                    self.assertEqual(status, 0)
                    self.assertIn("benchmark regression: crates/example/system/sidecar_ingest/scalar/0: 900.0%", output.getvalue())
                    self.assertNotIn("::notice::", output.getvalue())
                    published = json.loads((directory / "target/benchmarks/benchmarks-rust-system.json").read_text())
                    self.assertEqual(published["benchmarks"][0]["value"], 1000.0)
                    self.assertEqual(published["benchmarks"][0]["baseline_value"], 100.0)
                    self.assertEqual(published["benchmarks"][0]["revision"], "a" * 40)
                    self.assertEqual(published["benchmarks"][0]["boundary"], "producer handoff through durable acknowledgement")
                    expected = 1000.0 if refresh else 100.0
                    self.assertEqual(json.loads(stored.read_text())["median"]["point_estimate"], expected)
        finally:
            temporary_directory.cleanup()

    def test_completed_story_measurements_preserve_results(self) -> None:
        for language in ("python", "typescript", "javascript"):
            for instrumented in (False, True):
                temporary_directory = tempfile.TemporaryDirectory()
                temporary = temporary_directory.name
                try:
                    with self.subTest(language=language, coverage=instrumented):
                        directory = Path(temporary).resolve()
                        (directory / "tools").mkdir()
                        (directory / "tools/project.toml").write_text("", encoding="utf-8")
                        rules = policy.load(directory)
                        seconds = rules.deadline("unit", coverage=True) + 1
                        key = f"unit-{language}"
                        scratch_directory = lanes.prepare_scratch(directory, key, "dev")
                        scratch = scratch_directory.path
                        try:
                            junit = scratch / "junit.xml"
                            document = (
                                f'<testsuite name="fixture"><testcase name="passes" time="{seconds}"/>'
                                '<testcase name="skips"><skipped/></testcase></testsuite>'
                            )
                            junit.write_text(document, encoding="utf-8")
                            environment = {lanes.COVERAGE_VARIABLE: "1"} if instrumented else {}
                            with patch.object(execution, "execute", return_value=0):
                                result = lanes.record(
                                    directory, rules, key, "unit", language, ".", "dev", [], directory, environment, junit, scratch
                                )
                        finally:
                            scratch_directory.close()
                        self.assertEqual((result["outcome"], result["passed"], result["failed"], result["skipped"]), ("passed", 1, 0, 1))
                        self.assertEqual(result["durations"], [("fixture", "passes", directory.name, seconds)])
                        destination = report.location(directory, "dev", key)
                        self.assertEqual((destination / "junit.xml").read_text(), document)
                        self.assertEqual(
                            json.loads((destination / "durations.json").read_text()), {"fixture": {"passes": {"seconds": seconds}}}
                        )
                finally:
                    temporary_directory.cleanup()

    def test_script_hang_kills_owned_process_tree(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            directory = Path(temporary).resolve()
            rules = policy.load(ROOT)
            process = Mock(pid=1234, stdout=None, stderr=None, stdin=None)
            process.wait.side_effect = [subprocess.TimeoutExpired("fixture", rules.deadline("system")), 0, 0]
            process.poll.return_value = None
            scratch_directory = lanes.prepare_scratch(directory, "system-typescript", "dev")
            scratch = scratch_directory.path
            try:
                junit = scratch / "junit.xml"
                junit.write_text('<testsuite><testcase name="finished"/></testsuite>', encoding="utf-8")
                with (
                    patch.object(job, "own_children"),
                    patch.object(execution.subprocess, "Popen", return_value=process) as spawn,
                    patch.object(execution.subprocess, "run") as terminate,
                    patch.object(execution.os, "killpg", create=True) as kill_group,
                ):
                    result = lanes.record(
                        directory,
                        rules,
                        "system-typescript",
                        "system",
                        "typescript",
                        ".",
                        "dev",
                        ["fixture"],
                        directory,
                        None,
                        junit,
                        scratch,
                    )
                self.assertIn("fixture", spawn.call_args.args[0])
                self.assertEqual(process.wait.call_args_list[0].kwargs, {"timeout": rules.deadline("system")})
                if os.name == "nt":
                    terminate.assert_called_once()
                    self.assertEqual(terminate.call_args.args[0], ["taskkill.exe", "/PID", str(process.pid), "/T", "/F"])
                    kill_group.assert_not_called()
                else:
                    terminate.assert_not_called()
                    kill_group.assert_called_once_with(process.pid, execution.signal.SIGKILL)
                process.kill.assert_called_once_with()
                self.assertEqual(process.wait.call_count, 3)
            finally:
                scratch_directory.close()
            self.assertEqual(result["outcome"], "failed")
        finally:
            temporary_directory.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
