# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"Desktop Benchmark Reporting and Refusal Preservation"

# === Imports === #

# Standard Library Imports
import json
import os
import subprocess
import sys
import tempfile
import unittest
from itertools import product
from pathlib import Path
from typing import Any
from unittest.mock import patch

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.build.bench
import dokime_tools.process
import dokime_tools.test.credentials
import dokime_tools.test.scratch
from dokime_tools.ci import tasks
from dokime_tools.release import reports
from dokime_tools.test import benchmark, desktop_bench, lanes, policy, report

from .environment import Resource

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DesktopBench(unittest.TestCase):
    def test_storage_build_requires_the_report_before_compilation(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        try:
            directory = Path(temporary_directory.name)
            template = directory / "target/generated/report/report.html"

            def generate() -> None:
                template.parent.mkdir(parents=True)
                template.write_bytes(b"report template")

            def compile(root: Path, extra: list[str], selection: tuple[str, ...]) -> Path:
                self.assertEqual(template.read_bytes(), b"report template")
                self.assertEqual(root, directory)
                self.assertEqual(extra, ["--timings"])
                self.assertEqual(selection, ("--package", "dokime-runtime", "--bench", "storage"))
                return directory / "compiler.json"

            with (
                patch.object(dokime_tools.build.bench.Path, "cwd", return_value=directory),
                patch.object(dokime_tools.build.bench.sys, "argv", ["bench", "--binary", "storage", "--", "--timings"]),
                patch.object(dokime_tools.build.bench.generated, "prepare", side_effect=generate) as prepare,
                patch.object(dokime_tools.build.bench, "build", side_effect=compile) as build,
            ):
                dokime_tools.build.bench.main()
                template.unlink()
                prepare.side_effect = ValueError("generation failed")
                build.reset_mock()
                self.assertRaises(ValueError, dokime_tools.build.bench.main)
                build.assert_not_called()
        finally:
            temporary_directory.cleanup()

    def test_once_preserves_benchmark_selection_and_does_not_promote_baselines(self) -> None:
        with (
            patch.object(tasks.transport, "receive"),
            patch.dict(os.environ, {"COMPILER_CACHE_BACKEND": ""}),
            patch.object(tasks.subprocess, "run", return_value=subprocess.CompletedProcess([], 7)) as execute,
        ):
            for binary, consumer in (("storage", "bench:rust:storage"), ("system-desktop", "bench:rust:system")):
                self.assertEqual(tasks.main(["bench", "--binary", binary, "--layer", "system", "--once", "--select", "window"]), 7)
                command = execute.call_args.args[0]
                self.assertEqual(command[:4], ["mise", "run", consumer, "--"])
                self.assertIn("--once", command)
                self.assertEqual(command[command.index("--select") + 1], "window")
                self.assertNotIn("--refresh-baseline", command)
            execute.reset_mock()
            self.assertRaises(SystemExit, tasks.main, ["bench", "--binary", "system-desktop", "--once", "--refresh-baseline"])
            execute.assert_not_called()

    def test_desktop_build_embeds_its_surface_without_replacing_shipping_assets(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            directory = Path(temporary)
            live = directory / "crates/dokime-gui/dist/index.html"
            live.parent.mkdir(parents=True)
            live.write_bytes(b"shipping")
            output = dokime_tools.build.bench.paths.output(directory, "build", "bench")
            web = output / "web"

            def bundle(command: list[str], **options: Any) -> subprocess.CompletedProcess[str] | None:
                if command[:3] == ["mise", "run", "build:typescript"]:
                    self.assertEqual(command, ["mise", "run", "build:typescript", "--profile", "dev"])
                    return subprocess.CompletedProcess(command, 0)
                self.assertEqual(command[:2], ["node", "benches/system/build.mjs"])
                self.assertEqual(Path(command[2]), web)
                web.mkdir(parents=True)
                (web / "index.html").write_bytes(b"benchmark")

            def compile(root: Path, extra: list[str], selection: str, **options: Any) -> Path:
                self.assertTrue((web / "index.html").is_file())
                config = json.loads(options["environment"]["TAURI_CONFIG"])
                frontend = Path(config["build"]["frontendDist"])
                self.assertFalse(frontend.is_absolute())
                self.assertEqual((directory / "crates/dokime-gui/src-tauri" / frontend).resolve(), web)
                self.assertEqual(config["bundle"], {"active": False})
                self.assertIn("system-desktop", selection)
                (output / "fixtures").mkdir()
                (output / "fixtures/stamp.json").write_text('{"files":{},"metadata":{}}')
                return output / "bench/compiler.json"

            with (
                patch.object(dokime_tools.build.bench.Path, "cwd", return_value=directory),
                patch.object(dokime_tools.build.bench.sys, "argv", ["bench", "--binary", "system-desktop"]),
                patch.dict(os.environ, {"TAURI_CONFIG": '{"bundle":{"active":false}}'}),
                patch.object(dokime_tools.build.bench.subprocess, "run", side_effect=bundle),
                patch.object(dokime_tools.build.bench, "build", side_effect=compile),
                patch.object(dokime_tools.build.bench.generated, "stage"),
                patch.object(dokime_tools.build.bench.archive, "stamp"),
            ):
                dokime_tools.build.bench.main()
                self.assertEqual(os.environ["TAURI_CONFIG"], '{"bundle":{"active":false}}')
            self.assertEqual(live.read_bytes(), b"shipping")
        finally:
            temporary_directory.cleanup()

    def test_lanes_reclaim_child_scratch_and_keep_failure_reports(self) -> None:
        script = (
            "import json, os, pathlib, sys, tempfile; "
            "root = pathlib.Path(tempfile.gettempdir()); "
            "pathlib.Path(tempfile.mkdtemp(prefix='dokime-evidence-')).joinpath('rows').write_bytes(b'evidence'); "
            "tempfile.NamedTemporaryFile(prefix='dokime-output-', delete=False).close(); "
            "report = pathlib.Path(sys.argv[1]); "
            "report.with_name('scratch.json').write_text(json.dumps({'root': str(root)})); "
            'report.write_text(\'<testsuites><testsuite><testcase file="tests/system/capture.py" name="capture">\' + '
            "('<failure/>' if sys.argv[2] == '1' else '') + '</testcase></testsuite></testsuites>'); "
            "os._exit(int(sys.argv[2]))"
        )
        rules = policy.load(Path.cwd())
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            with patch.object(dokime_tools.test.credentials, "Store", side_effect=lambda environment: Resource(environment)):
                directory = Path(temporary)
                cases = [*product(("rust", "python", "typescript"), policy.CATEGORIES), ("rust", "coverage"), ("python", "coverage")]
                for language, category in cases:
                    for failed in (False, True):
                        key = f"{category}-{language}"
                        scratch = directory / f"{category}-{language}-{int(failed)}"
                        scratch.mkdir()
                        junit = scratch / "junit.xml"
                        lane = lanes.record(
                            directory,
                            rules,
                            key,
                            category,
                            language,
                            ".",
                            "dev",
                            [sys.executable, "-c", script, str(junit), str(int(failed))],
                            directory,
                            {**os.environ, "DOKIME_KEEP_EVIDENCE": "0", lanes.COVERAGE_VARIABLE: "1" if category == "coverage" else "0"},
                            junit,
                            scratch,
                        )
                        destination = report.location(directory, "dev", key)
                        native_root = Path(json.loads((destination / "scratch.json").read_text())["root"])
                        self.assertFalse(native_root.exists())
                        self.assertEqual(lane["outcome"], "failed" if failed else "passed")
                        self.assertTrue((destination / "junit.xml").is_file())
        finally:
            temporary_directory.cleanup()

    def test_native_child_scratch_is_reclaimed_after_success_and_failure(self) -> None:
        original = dict(os.environ)
        script = (
            "import os, pathlib, tempfile; "
            "root = pathlib.Path(tempfile.mkdtemp(prefix='dokime-evidence-')); "
            "(root / 'rows').write_bytes(b'evidence'); "
            "tempfile.NamedTemporaryFile(prefix='dokime-output-', delete=False).close(); "
            "print(root.parent, flush=True); os._exit(0)"
        )
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            reports = Path(temporary) / "reports"
            for failed in (False, True):
                try:
                    scratch_root = dokime_tools.test.scratch.Root(original, reports)
                    try:
                        environment = scratch_root.environment
                        child = subprocess.run([sys.executable, "-c", script], env=environment, capture_output=True, text=True, check=True)
                        root = Path(child.stdout.strip())
                        self.assertTrue(any(root.glob("dokime-evidence-*")))
                        self.assertTrue(any(root.glob("dokime-output-*")))
                        if failed:
                            raise ValueError("failed desktop run")
                    finally:
                        scratch_root.close()
                except ValueError:
                    self.assertTrue(failed)
                self.assertFalse(root.exists())
                self.assertFalse(reports.exists())
            scratch_root = dokime_tools.test.scratch.Root({**original, "DOKIME_KEEP_EVIDENCE": "1"}, reports)
            try:
                environment = scratch_root.environment
                child = subprocess.run([sys.executable, "-c", script], env=environment, capture_output=True, text=True, check=True)
                root = Path(child.stdout.strip())
            finally:
                scratch_root.close()
            self.assertFalse(root.exists())
            self.assertTrue(any(reports.rglob("dokime-evidence-*")))
            self.assertTrue(any(reports.rglob("dokime-output-*")))
        finally:
            temporary_directory.cleanup()
        self.assertEqual(dict(os.environ), original)

    def test_failed_preflight_does_not_publish_cached_measurements(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            previous = root / "target/benchmarks/benchmarks-rust-desktop.json"
            previous.parent.mkdir(parents=True)
            previous.write_text(json.dumps({"benchmarks": [{"name": "previous"}]}))
            with (
                patch.object(policy, "load"),
                patch.object(lanes, "preflight", side_effect=policy.TestError("missing")),
                self.assertRaises(policy.TestError),
            ):
                benchmark.run(root, "system", "rust", ".", False, [], binary="system-desktop")
            self.assertFalse(previous.exists())
        finally:
            temporary_directory.cleanup()

    def test_failed_native_initialization_clears_old_results_without_promoting_baseline(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            home = root / "native"
            home.mkdir()
            previous = json.dumps({"benchmarks": [{"name": "previous", "unit": "ms", "value": 10}]}).encode()
            (home / "dev.json").write_bytes(previous)
            (home / "new.json").write_bytes(previous)
            (home / "observations.json").write_text(json.dumps({"fixture": None, "results": [], "resources": []}))
            (home / "completion.json").write_text(json.dumps({"failure": "WebDriver close failed", "measuring": "2073600/30/cold"}))
            with patch.object(desktop_bench.policy, "execute", return_value="candidate"):
                desktop_bench.collect(root, home, {}, 20, refresh=True, success=False)
            rows = json.loads((home / "new.json").read_text())["benchmarks"]
            self.assertTrue(rows)
            self.assertTrue(all(row["status"] == "not_measured" and row["value"] is None and row["reason"] for row in rows))
            self.assertEqual({row["reason"] for row in rows}, {"WebDriver close failed"})
            self.assertEqual(json.loads((root / "target/benchmarks/benchmarks-rust-desktop.json").read_text())["benchmarks"], rows)
            self.assertEqual((home / "dev.json").read_bytes(), previous)
        finally:
            temporary_directory.cleanup()

    def test_stage_totals_quantiles_and_refusals(self) -> None:
        temporary_directory = tempfile.TemporaryDirectory()
        temporary = temporary_directory.name
        try:
            root = Path(temporary)
            home = root / "native"
            home.mkdir()
            samples = []
            for index in range(20):
                samples.append(
                    {
                        "count": 30,
                        "condition": "warm",
                        "outcome": "drawn",
                        "errors": [],
                        "stages": {"canvas_draw": [index, index + 1]},
                        "backendStages": {},
                        "queryMs": [index + 2],
                        "firstVisibleMs": index + 3,
                        "allVisibleMs": index + 4,
                        "durationMs": index + 5,
                        "cpuSeconds": None,
                        "peakMemoryBytes": None,
                        "queueDepth": 0,
                        "bytes": index * 10,
                        "frameMisses": 0,
                    }
                )
            samples.append(
                {
                    "count": 100,
                    "condition": "warm",
                    "outcome": "refused",
                    "errors": ["limit_exceeded"],
                    "stages": {},
                    "backendStages": {},
                    "queryMs": [],
                    "durationMs": 1,
                    "firstVisibleMs": None,
                    "allVisibleMs": None,
                    "queueDepth": 0,
                    "bytes": 0,
                    "frameMisses": 0,
                }
            )
            samples.append({**samples[0], "condition": "live", "outcome": "partial", "errors": ["plot_unavailable"]})
            samples.append({**samples[0], "rows": 307200, "durationMs": 99})
            (home / "observations.json").write_text(json.dumps({"results": samples, "fixture": {"rows": 2_073_600}}))
            with patch.object(desktop_bench.policy, "execute", return_value="candidate"):
                desktop_bench.collect(root, home, {}, 10, refresh=True, success=True)
            summary = json.loads((root / "target/benchmarks/benchmarks-rust-desktop.json").read_text())
            rows = {row["name"]: row for row in summary["benchmarks"]}
            self.assertEqual(rows["desktop/2073600/30/warm/canvas_draw/p50"]["value"], 20)
            self.assertEqual(rows["desktop/307200/30/warm/durationMs/p50"]["value"], 99)
            self.assertEqual(rows["desktop/2073600/30/warm/canvas_draw/p95"]["value"], 37)
            self.assertEqual(rows["desktop/2073600/30/warm/queueDepth/p50"]["value"], 0)
            self.assertNotIn("desktop/2073600/100/warm/firstVisibleMs/p50", rows)
            self.assertEqual(rows["desktop/2073600/100/warm/refusals"]["value"], 1)
            self.assertEqual(rows["desktop/2073600/100/warm/refusals"]["failures"], ["limit_exceeded"])
            partial = rows["desktop/2073600/30/live/refusals"]
            self.assertEqual((partial["value"], partial["partial_samples"], partial["failures"]), (0, 1, ["plot_unavailable"]))
            self.assertNotIn("desktop/2073600/30/warm/cpuSeconds/p50", rows)
            self.assertIn("cpuSeconds", rows["desktop/2073600/30/warm/refusals"]["missing_measurements"])
            nightly = root / "target/nightly-reports/desktop"
            nightly.mkdir(parents=True)
            (nightly / "benchmarks-rust-desktop.json").write_text(json.dumps(summary))
            published, _files = reports.collect(root)
            painted = next(row for row in published["benchmarks"] if row["name"] == "desktop/2073600/30/warm/canvas_draw/p50")
            self.assertEqual((painted["unit"], painted["value"]), ("ms", 20))
            refusal = next(row for row in published["benchmarks"] if row["name"] == "desktop/2073600/100/warm/refusals")
            self.assertIn("physical_display_presentation", refusal["unavailable"])
            self.assertEqual((home / "dev.json").read_bytes(), (home / "new.json").read_bytes())
            previous = (home / "dev.json").read_bytes()
            with patch.object(desktop_bench.policy, "execute", return_value="candidate"):
                desktop_bench.collect(root, home, {}, 10, refresh=True, success=False)
            self.assertEqual((home / "dev.json").read_bytes(), previous)
            newer = json.loads((home / "new.json").read_text())
            measured = next(row for row in newer["benchmarks"] if row["name"] == "desktop/2073600/30/warm/canvas_draw/p50")
            self.assertEqual(measured["baseline_value"], 20)
        finally:
            temporary_directory.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
