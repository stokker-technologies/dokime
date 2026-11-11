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
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.test import desktop_bench
from dokime_tools.release import reports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DesktopBench(unittest.TestCase):
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
            (home / "observations.json").write_text(json.dumps({"results": samples, "fixture": {"rows": 2_073_600}}))
            with patch.object(desktop_bench.policy, "execute", return_value="candidate"):
                desktop_bench.collect(root, home, {}, 10, refresh=True, success=True)
            summary = json.loads((root / "target/benchmarks/benchmarks-rust-desktop.json").read_text())
            rows = {row["name"]: row for row in summary["benchmarks"]}
            self.assertEqual(rows["desktop/30/warm/canvas_draw/p50"]["value"], 20)
            self.assertEqual(rows["desktop/30/warm/canvas_draw/p95"]["value"], 37)
            self.assertEqual(rows["desktop/30/warm/queueDepth/p50"]["value"], 0)
            self.assertNotIn("desktop/100/warm/firstVisibleMs/p50", rows)
            self.assertEqual(rows["desktop/100/warm/refusals"]["value"], 1)
            self.assertEqual(rows["desktop/100/warm/refusals"]["failures"], ["limit_exceeded"])
            self.assertNotIn("desktop/30/warm/cpuSeconds/p50", rows)
            self.assertIn("cpuSeconds", rows["desktop/30/warm/refusals"]["missing_measurements"])
            nightly = root / "target/nightly-reports/desktop"
            nightly.mkdir(parents=True)
            (nightly / "benchmarks-rust-desktop.json").write_text(json.dumps(summary))
            published, _files = reports.collect(root)
            painted = next(row for row in published["benchmarks"] if row["name"] == "desktop/30/warm/canvas_draw/p50")
            self.assertEqual((painted["unit"], painted["value"]), ("ms", 20))
            refusal = next(row for row in published["benchmarks"] if row["name"] == "desktop/100/warm/refusals")
            self.assertIn("physical_display_presentation", refusal["unavailable"])
            self.assertEqual((home / "dev.json").read_bytes(), (home / "new.json").read_bytes())
            previous = (home / "dev.json").read_bytes()
            with patch.object(desktop_bench.policy, "execute", return_value="candidate"):
                desktop_bench.collect(root, home, {}, 10, refresh=True, success=False)
            self.assertEqual((home / "dev.json").read_bytes(), previous)
            newer = json.loads((home / "new.json").read_text())
            measured = next(row for row in newer["benchmarks"] if row["name"] == "desktop/30/warm/canvas_draw/p50")
            self.assertEqual(measured["baseline_value"], 20)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
