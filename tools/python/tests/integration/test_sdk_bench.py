# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Verify SDK Benchmark Result Shaping"""

# === Imports === #

# Standard Library Imports
import argparse
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# isort: split

# Third Party Imports
# isort: split

# Local Imports
from dokime_tools.release import reports
from dokime_tools.task import bench
from dokime_tools.test import protocol_bench, sdk_bench

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class SdkBenchTests(unittest.TestCase):
    def test_protocol_sizes_keep_byte_units_and_reuse_the_report_baseline(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        try:
            root = Path(temporary.name)
            home = root / "native"
            home.mkdir()
            (home / "encoded-sizes.json").write_text(json.dumps({"protocol/evidence/size": 77}), encoding="utf-8")
            with patch.object(protocol_bench.policy, "execute", return_value="revision"):
                protocol_bench.collect(root, home, {}, 20, refresh=True, success=True)
                protocol_bench.collect(root, home, {}, 20, refresh=False, success=True)
            row = json.loads((home / "protocol-new.json").read_text(encoding="utf-8"))["benchmarks"][0]
            self.assertEqual(row["unit"], "bytes")
            self.assertEqual(row["value"], row["baseline_value"])
        finally:
            temporary.cleanup()

    def test_sdk_selection_preserves_language_filter_and_once(self) -> None:
        args = argparse.Namespace(
            layer=None, language="rust", binary="system-sdk", select="^sdk/python/", once=True, refresh_baseline=False
        )
        self.assertEqual(
            bench.select(args, argparse.ArgumentParser()),
            [("bench:rust:system", ["--once", "--binary", "system-sdk", "--select", "^sdk/python/"])],
        )

    def test_collector_preserves_units_baselines_and_unavailable_stages(self) -> None:
        log = "SDK\tsdk/rust/round_trip/echo/p50\tns\t123\tRound Trip p50 [ns]\t\nSDK\tsdk/rust/memory/idle\tbytes\tnot_measured\tMemory [bytes]\tUnavailable\n"
        baseline = {"benchmarks": [{"name": "sdk/rust/round_trip/echo/p50", "unit": "ns", "value": 100}]}
        result = sdk_bench.collect(log, {"language": "rust"}, baseline, "revision", 20)
        measured, missing = result["benchmarks"]
        self.assertEqual((measured["value"], measured["baseline_value"], measured["unit"]), (123, 100, "ns"))
        self.assertEqual((missing["status"], missing["reason"], missing["value"]), ("not_measured", "Unavailable", None))
        self.assertIsNone(
            sdk_bench.collect(log, {}, {"benchmarks": [{"name": measured["name"], "unit": "ms", "value": 100}]}, "revision", 20)[
                "benchmarks"
            ][0]["baseline_value"]
        )

    def test_throughput_regressions_report_decreases_without_gating(self) -> None:
        log = "SDK\tsdk/python/throughput/ceiling\tsamples/s\t50\tThroughput [samples/s]\t"
        baseline = {"benchmarks": [{"name": "sdk/python/throughput/ceiling", "unit": "samples/s", "value": 100}]}
        summary = sdk_bench.collect(log, {}, baseline, "revision", 20)
        slower = reports.table(summary | {"coverage": []}, {}, "https://example.com/workflow")
        faster = reports.table(
            sdk_bench.collect(log.replace("\t50\t", "\t150\t"), {}, baseline, "revision", 20) | {"coverage": []},
            {},
            "https://example.com/workflow",
        )
        self.assertIn("+50.0%", slower)
        self.assertNotIn("+50.0%", faster)

    def test_collector_refuses_nonfinite_duplicate_and_unexplained_values(self) -> None:
        row = "SDK\tsdk/python/start_to_ready\tms\t1\tStart to Ready [ms]\t"
        for log in (row + "\n" + row, row.replace("\t1\t", "\tnan\t"), row.replace("\t1\t", "\tnot_measured\t")):
            with self.assertRaises(ValueError):
                sdk_bench.collect(log, {}, {}, "revision", 20)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
