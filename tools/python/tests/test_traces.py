# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify Span Summary Aggregation"""

# === Imports === #

# Standard Library Imports
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.process import spawn_flags
from dokime_tools.test import run, traces

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class TraceSummaryTests(unittest.TestCase):
    def test_span_summary(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            directory = root / "traces"
            directory.mkdir()

            def event(pid, identifier, parent, start, duration):
                return {
                    "ph": "X",
                    "pid": pid,
                    "tid": identifier,
                    "name": "operation",
                    "cat": "fixture",
                    "ts": start,
                    "dur": duration,
                    "args": {"span_id": identifier, "parent_id": parent},
                }

            events = [event(1, 1, None, 0, 10000), event(1, 2, 1, 1000, 5000), event(1, 3, 1, 4000, 5000), event(2, 1, None, 0, 2000)]
            (directory / "fixture.pftrace").write_text(json.dumps({"traceEvents": events}), encoding="utf-8")
            (directory / "killed.pftrace").write_text("{", encoding="utf-8")
            junit = root / "junit.xml"
            junit.write_text(
                '<testsuite name="fixture"><testcase name="fails" time="0.02"><failure/></testcase>'
                '<testcase name="passes" time="0.01"/></testsuite>',
                encoding="utf-8",
            )
            traces.summarize(root, junit)
            result = json.loads((root / "time-summary.json").read_text(encoding="utf-8"))
            self.assertEqual(result["by_total"][0]["total_ms"], 22)
            self.assertEqual(result["by_total"][0]["self_ms"], 14)
            self.assertEqual(result["incomplete_files"], ["killed.pftrace"])
            self.assertEqual([test["outcome"] for test in result["tests"]], ["failed", "passed"])
            self.assertEqual(result["slowest_tests"][0]["total_ms"], 20)
            junit.write_text("<", encoding="utf-8")
            traces.summarize(root, junit)
            result = json.loads((root / "time-summary.json").read_text(encoding="utf-8"))
            self.assertTrue(result["junit_error"])
            self.assertEqual(result["tests"], [])

    def test_failed_lane(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / "tools").mkdir()
            (root / "tools/project.toml").write_text("", encoding="utf-8")

            destinations = []

            def execute(command, cwd, environment):
                trace_directory = Path(environment["DOKIME_TEST_TRACE_DIR"])
                self.assertTrue(trace_directory.is_relative_to(root))
                destinations.append(trace_directory.parent)
                (trace_directory / "child-123.pftrace").write_text('{"traceEvents":[]}', encoding="utf-8")
                raise RuntimeError("fixture runner failure")

            with patch.object(run, "_execute", side_effect=execute), self.assertRaisesRegex(RuntimeError, "fixture runner failure"):
                run.test(root, "integration", "typescript", ".", "dev", ["node", "fixture", "{junit}"])
            summary = destinations[0] / "time-summary.json"
            self.assertEqual(json.loads(summary.read_text(encoding="utf-8"))["trace_files"], 1)

    def test_desktop_export(self):
        module = Path(__file__).resolve().parents[3] / "crates/dokime-gui/e2e/test-trace.mjs"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            script = root / "fixture.mjs"
            script.write_text(
                f"import {{ traceSpan }} from {json.dumps(module.as_uri())};\n"
                'await traceSpan("scene", "scene", async () => {\n'
                '  await Promise.all([traceSpan("left", "step", async () => {}), traceSpan("right", "step", async () => {})]);\n'
                "});\n"
                'try { await traceSpan("refusal", "step", async () => {throw new Error("fixture-private-payload");}); }\n'
                "catch { process.exitCode = 7; }\n",
                encoding="utf-8",
            )
            result = subprocess.run(
                ["node", str(script)],
                env={**os.environ, "DOKIME_TEST_TRACE_DIR": str(root)},
                creationflags=spawn_flags(),
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 7, result.stderr)
            files = list(root.glob("*.pftrace"))
            self.assertEqual(len(files), 1)
            text = files[0].read_text(encoding="utf-8")
            self.assertNotIn("fixture-private-payload", text)
            events = json.loads(text)["traceEvents"]
            scene = next(event for event in events if event["name"] == "scene")
            children = [event for event in events if event["name"] in ("left", "right")]
            self.assertEqual(len(children), 2)
            self.assertTrue(all(event["args"]["parent_id"] == scene["args"]["span_id"] for event in children))
            self.assertTrue(any(event["cat"] == "story" for event in events))

    def test_child_union(self):
        self.assertEqual(traces._union([(-10, 3), (2, 6), (8, 20), (30, 40)], 0, 10), 8)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
