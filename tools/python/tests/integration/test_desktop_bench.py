# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Desktop Benchmark Reporting And Refusal Preservation"""

# === Imports === #

# Standard Library Imports
import json
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import nullcontext
from pathlib import Path
from unittest.mock import patch

from dokime_tools.build import bench as bench_builder
from dokime_tools.ci import tasks

# Third Party Imports
# Local Imports
from dokime_tools.release import reports
from dokime_tools.test import desktop_bench, run

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DesktopBench(unittest.TestCase):
    def test_once_preserves_benchmark_selection_and_does_not_promote_baselines(self) -> None:
        with (
            patch.object(tasks.transport, "receive"),
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
            with self.assertRaises(SystemExit):
                tasks.main(["bench", "--binary", "system-desktop", "--once", "--refresh-baseline"])
            execute.assert_not_called()

    def test_desktop_build_embeds_its_surface_without_replacing_shipping_assets(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            live = directory / "crates/dokime-gui/dist/index.html"
            live.parent.mkdir(parents=True)
            live.write_bytes(b"shipping")
            output = bench_builder.paths.output(directory, "build", "bench")
            web = output / "web"

            def bundle(command, **options):
                if command[:3] == ["mise", "run", "build:typescript"]:
                    self.assertEqual(command, ["mise", "run", "build:typescript", "--profile", "dev"])
                    return subprocess.CompletedProcess(command, 0)
                self.assertEqual(command[:2], ["node", "benches/system/build.mjs"])
                self.assertEqual(Path(command[2]), web)
                web.mkdir(parents=True)
                (web / "index.html").write_bytes(b"benchmark")

            def compile(root, extra, selection, **options):
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
                patch.object(bench_builder.Path, "cwd", return_value=directory),
                patch.object(bench_builder.sys, "argv", ["bench", "--binary", "system-desktop"]),
                patch.dict(os.environ, {"TAURI_CONFIG": '{"bundle":{"active":false}}'}),
                patch.object(bench_builder.subprocess, "run", side_effect=bundle),
                patch.object(bench_builder, "build", side_effect=compile),
                patch.object(bench_builder.generated, "stage"),
                patch.object(bench_builder.archive, "stamp"),
            ):
                bench_builder.main()
                self.assertEqual(os.environ["TAURI_CONFIG"], '{"bundle":{"active":false}}')
            self.assertEqual(live.read_bytes(), b"shipping")

    def test_system_lanes_reclaim_child_scratch_and_keep_failure_reports(self) -> None:
        script = (
            "import json, os, pathlib, sys, tempfile; "
            "root = pathlib.Path(tempfile.gettempdir()); "
            "pathlib.Path(tempfile.mkdtemp(prefix='dokime-evidence-')).joinpath('rows').write_bytes(b'evidence'); "
            "tempfile.NamedTemporaryFile(prefix='dokime-output-', delete=False).close(); "
            "report = pathlib.Path(sys.argv[1]); "
            "report.with_name('scratch.json').write_text(json.dumps({'root': str(root)})); "
            "report.write_text('<testsuites><testsuite><testcase name=\"capture\">' + "
            "('<failure/>' if sys.argv[2] == '1' else '') + '</testcase></testsuite></testsuites>'); "
            "os._exit(int(sys.argv[2]))"
        )
        rules = run.policy.load(Path.cwd())
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.object(desktop_bench, "credentials", side_effect=lambda environment: nullcontext(environment)),
        ):
            directory = Path(temporary)
            for language in ("rust", "python", "typescript"):
                for failed in (False, True):
                    key = f"system-{language}"
                    scratch = directory / f"{language}-{int(failed)}"
                    scratch.mkdir()
                    junit = scratch / "junit.xml"
                    lane = run._lane(
                        directory,
                        rules,
                        key,
                        "system",
                        language,
                        ".",
                        "dev",
                        [sys.executable, "-c", script, str(junit), str(int(failed))],
                        directory,
                        {**os.environ, "DOKIME_KEEP_EVIDENCE": "0"},
                        junit,
                        scratch,
                    )
                    destination = run.report.location(directory, "dev", key)
                    native_root = Path(json.loads((destination / "scratch.json").read_text())["root"])
                    self.assertFalse(native_root.exists())
                    self.assertEqual(lane["outcome"], "failed" if failed else "passed")
                    self.assertTrue((destination / "junit.xml").is_file())

    def test_native_child_scratch_is_reclaimed_after_success_and_failure(self) -> None:
        original = dict(os.environ)
        script = (
            "import os, pathlib, tempfile; "
            "root = pathlib.Path(tempfile.mkdtemp(prefix='dokime-evidence-')); "
            "(root / 'rows').write_bytes(b'evidence'); "
            "tempfile.NamedTemporaryFile(prefix='dokime-output-', delete=False).close(); "
            "print(root.parent, flush=True); os._exit(0)"
        )
        with tempfile.TemporaryDirectory() as temporary:
            reports = Path(temporary) / "reports"
            for failed in (False, True):
                try:
                    with desktop_bench.scratch(original, reports) as environment:
                        child = subprocess.run([sys.executable, "-c", script], env=environment, capture_output=True, text=True, check=True)
                        root = Path(child.stdout.strip())
                        self.assertTrue(any(root.glob("dokime-evidence-*")))
                        self.assertTrue(any(root.glob("dokime-output-*")))
                        if failed:
                            raise ValueError("failed desktop run")
                except ValueError:
                    self.assertTrue(failed)
                self.assertFalse(root.exists())
                self.assertFalse(reports.exists())
            with desktop_bench.scratch({**original, "DOKIME_KEEP_EVIDENCE": "1"}, reports) as environment:
                child = subprocess.run([sys.executable, "-c", script], env=environment, capture_output=True, text=True, check=True)
                root = Path(child.stdout.strip())
            self.assertFalse(root.exists())
            self.assertTrue(any(reports.rglob("dokime-evidence-*")))
            self.assertTrue(any(reports.rglob("dokime-output-*")))
        self.assertEqual(dict(os.environ), original)

    def test_failed_preflight_does_not_publish_cached_measurements(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            previous = root / "target/benchmarks/benchmarks-rust-desktop.json"
            previous.parent.mkdir(parents=True)
            previous.write_text(json.dumps({"benchmarks": [{"name": "previous"}]}))
            with (
                patch.object(run.policy, "load"),
                patch.object(run, "_preflight", side_effect=run.policy.TestError("missing")),
                self.assertRaises(run.policy.TestError),
            ):
                run.bench(root, "system", "rust", ".", False, [], binary="system-desktop")
            self.assertFalse(previous.exists())

    def test_failed_native_initialization_clears_old_results_without_promoting_baseline(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            home = root / "native"
            home.mkdir()
            previous = json.dumps({"benchmarks": [{"name": "previous", "unit": "ms", "value": 10}]}).encode()
            (home / "dev.json").write_bytes(previous)
            (home / "new.json").write_bytes(previous)
            (home / "observations.json").write_text(json.dumps({"fixture": None, "results": [], "resources": []}))
            with patch.object(desktop_bench.policy, "execute", return_value="candidate"):
                desktop_bench.collect(root, home, {}, 20, refresh=True, success=False)
            self.assertEqual(json.loads((home / "new.json").read_text())["benchmarks"], [])
            self.assertEqual(json.loads((root / "target/benchmarks/benchmarks-rust-desktop.json").read_text())["benchmarks"], [])
            self.assertEqual((home / "dev.json").read_bytes(), previous)

    def test_stage_totals_quantiles_and_refusals(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
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


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
