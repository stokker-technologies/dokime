# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prove Story Results Ignore Measurements And Script Hangs Terminate."""

# === Imports === #

# Standard Library Imports
import io
import json
import runpy
import socket
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from types import ModuleType
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.test import run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[4]

# === Types === #

# === Classes === #


class RunTests(unittest.TestCase):
    runners: list[ModuleType]

    @classmethod
    def setUpClass(cls):
        cls.runners = [run]
        for path in sorted((ROOT / "crates/cargo-dokime/templates").rglob("test/run.py.tmpl")):
            namespace = runpy.run_path(str(path), run_name="harmos_tools.test.template_runner")
            module = ModuleType(namespace["__name__"])
            module.__dict__.update(namespace)
            cls.runners.append(module)

    def test_benchmark_baseline_refresh_and_comparison(self):
        with tempfile.TemporaryDirectory() as temporary:
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
            median = 100.0

            def measure(command, cwd, environment):
                home = Path(environment["CRITERION_HOME"])
                output = home / "scalar/new"
                output.mkdir(parents=True, exist_ok=True)
                (output / "benchmark.json").write_text(json.dumps({"full_id": "sidecar_ingest/scalar/0"}), encoding="utf-8")
                estimates = json.dumps({"median": {"point_estimate": median}})
                (output / "estimates.json").write_text(estimates, encoding="utf-8")
                return 0, None

            with (
                patch.object(run, "_preflight"),
                patch.object(run.archive, "_toolchain", return_value="fixture"),
                patch.object(run.archive, "fixtures"),
                patch.object(run.paths, "output", return_value=build),
                patch.object(run, "_measured", side_effect=measure),
                patch.object(run.time, "time", return_value=0),
            ):
                output = io.StringIO()
                with redirect_stdout(output):
                    status = run.bench(directory, "system", "rust", ".", False, [])
                self.assertEqual(status, 1)
                self.assertIn("sidecar_ingest/scalar/0", output.getvalue())
                self.assertIn("mise run bench:rust:system --refresh-baseline", output.getvalue())
                self.assertEqual(list(directory.rglob("dev/estimates.json")), [])

                output = io.StringIO()
                with redirect_stdout(output):
                    status = run.bench(directory, "system", "rust", ".", True, [])
                self.assertEqual(status, 0)
                self.assertIn("::notice::", output.getvalue())
                self.assertIn("sidecar_ingest/scalar/0", output.getvalue())
                (stored,) = directory.rglob("dev/estimates.json")
                self.assertEqual(json.loads(stored.read_text())["median"]["point_estimate"], 100.0)

                median = 1000.0
                for refresh in (False, True):
                    output = io.StringIO()
                    with redirect_stdout(output):
                        status = run.bench(directory, "system", "rust", ".", refresh, [])
                    self.assertEqual(status, 0)
                    self.assertIn("benchmark regression: sidecar_ingest/scalar/0: 900.0%", output.getvalue())
                    self.assertNotIn("::notice::", output.getvalue())
                    expected = 1000.0 if refresh else 100.0
                    self.assertEqual(json.loads(stored.read_text())["median"]["point_estimate"], expected)

    def test_completed_story_measurements_preserve_results(self):
        for runner in self.runners:
            for language in ("python", "typescript", "javascript"):
                for instrumented in (False, True):
                    with (
                        self.subTest(runner=runner.__file__, language=language, coverage=instrumented),
                        tempfile.TemporaryDirectory() as temporary,
                    ):
                        directory = Path(temporary).resolve()
                        (directory / "tools").mkdir()
                        (directory / "tools/project.toml").write_text("", encoding="utf-8")
                        rules = runner.policy.load(directory)
                        seconds = rules.deadline("unit", coverage=True) + 1
                        key = f"unit-{language}"
                        with runner._scratch(directory, key, "dev") as scratch:
                            junit = scratch / "junit.xml"
                            document = (
                                f'<testsuite name="fixture"><testcase name="passes" time="{seconds}"/>'
                                '<testcase name="skips"><skipped/></testcase></testsuite>'
                            )
                            junit.write_text(document, encoding="utf-8")
                            environment = {runner.COVERAGE_VARIABLE: "1"} if instrumented else {}
                            with patch.dict(runner._lane.__globals__, _execute=lambda *args, **kwargs: 0):
                                result = runner._lane(
                                    directory, rules, key, "unit", language, ".", "dev", [], directory, environment, junit, scratch
                                )
                        self.assertEqual((result["outcome"], result["passed"], result["failed"], result["skipped"]), ("passed", 1, 0, 1))
                        self.assertEqual(result["durations"], [("fixture", "passes", directory.name, seconds)])
                        destination = runner.report.location(directory, "dev", key)
                        self.assertEqual((destination / "junit.xml").read_text(), document)
                        self.assertEqual(
                            json.loads((destination / "durations.json").read_text()), {"fixture": {"passes": {"seconds": seconds}}}
                        )

    def test_script_hang_kills_owned_process_tree(self):
        for runner in self.runners:
            with self.subTest(runner=runner.__file__), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary).resolve()
                # The Child Holds A Socket Until Killed; The Parent Reports Before Hanging.
                child = (
                    "import pathlib, socket, sys, threading; "
                    "server = socket.socket(); server.bind(('127.0.0.1', 0)); server.listen(); "
                    "pathlib.Path(sys.argv[1]).write_text(str(server.getsockname()[1])); "
                    "threading.Event().wait()"
                )
                parent = (
                    "import pathlib, subprocess, sys, threading, time\n"
                    "port = pathlib.Path(sys.argv[2])\n"
                    f"subprocess.Popen([sys.executable, '-c', {child!r}, str(port)])\n"
                    "while not port.exists(): time.sleep(0.01)\n"
                    "pathlib.Path(sys.argv[1]).write_text('<testsuite><testcase name=\"finished\"/></testsuite>')\n"
                    "threading.Event().wait()\n"
                )
                port = directory / "port"
                rules = runner.policy.Policy(limits={}, deadlines={"system": 3}, overrides=())
                with runner._scratch(directory, "system-typescript", "dev") as scratch:
                    junit = scratch / "junit.xml"
                    result = runner._lane(
                        directory,
                        rules,
                        "system-typescript",
                        "system",
                        "typescript",
                        ".",
                        "dev",
                        [sys.executable, "-c", parent, str(junit), str(port)],
                        directory,
                        None,
                        junit,
                        scratch,
                    )
                self.assertEqual(result["outcome"], "failed")
                with socket.socket() as client:
                    self.assertNotEqual(client.connect_ex(("127.0.0.1", int(port.read_text()))), 0)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
