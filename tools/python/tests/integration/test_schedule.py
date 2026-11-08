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
import io
import json
import os
import socket
import subprocess
import sys
import tarfile
import tempfile
import threading
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

# Third Party Imports
# Local Imports
from dokime_tools.test import archive, report, run, schedule

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ScheduleTests(unittest.TestCase):
    def test_layer_archive_keeps_runtime_inputs_without_other_layers(self):
        kinds = {
            "dokime": "lib",
            "dokime-cli": "bin",
            "integration": "test",
            "compile": "test",
            "acceptance": "test",
            "system": "test",
            "system-recovery": "test",
            "throughput": "bench",
        }
        layers = {
            "unit": {"dokime", "dokime-cli"},
            "integration": {"integration", "compile"},
            "acceptance": {"acceptance"},
            "system": {"system", "system-recovery"},
        }
        for category, expected in layers.items():
            with self.subTest(category=category), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary).resolve()
                target = directory / "producer/target"
                binaries = {
                    name: {"binary-name": name, "kind": kind, "binary-path": str(target / "debug/deps" / name)}
                    for name, kind in kinds.items()
                }
                metadata = {
                    "rust-build-meta": {"target-directory": str(target), "platforms": {"host": {"libdir": {"status": "unavailable"}}}},
                    "rust-binaries": binaries,
                }
                files = {
                    "target/nextest/binaries-metadata.json": json.dumps(metadata).encode(),
                    "target/nextest/cargo-metadata.json": b"{}",
                    **{f"target/debug/deps/{name}": name.encode() for name in binaries},
                    "target/debug/dokime": b"cli",
                    "target/debug/build/schema/output": b"generated schema",
                    "target/nextest/libdirs/host/runtime.dll": b"runtime library",
                }
                packed = archive.location(directory, "dev")
                packed.parent.mkdir(parents=True)
                with tarfile.open(packed, "w:zst") as output:
                    for name, data in files.items():
                        member = tarfile.TarInfo(name)
                        member.size = len(data)
                        output.addfile(member, io.BytesIO(data))

                extracted = directory / "extracted"
                archive.extract(directory, "dev", extracted, category=category)
                self.assertEqual(set(archive.binaries(directory, "dev", extracted)), expected)
                for name, data in files.items():
                    path = extracted / name
                    if name.startswith("target/debug/deps/") and Path(name).name not in expected:
                        self.assertFalse(path.exists())
                    elif name != "target/nextest/binaries-metadata.json":
                        self.assertEqual(path.read_bytes(), data)

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

    def test_native_task_graph_keeps_all_lanes_concurrent_and_reports_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            repository = Path(__file__).resolve().parents[4]
            configuration = tomllib.loads((repository / "mise.toml").read_text(encoding="utf-8"))
            # Project The Authored Graph Without Product Prerequisites Or Native Test Binaries.
            source = (repository / "mise.toml").read_text(encoding="utf-8")
            start = source.index("[tasks.test]")
            stop = source.index("# === Bench === #", start)
            tasks = source[start:stop]
            (directory / "tools/python/tests/integration").mkdir(parents=True)
            (directory / "tools/project.toml").write_text("", encoding="utf-8")
            (directory / "tools/python/tests/integration/test_prepared.py").write_text(
                "import pathlib, unittest\n"
                "class Prepared(unittest.TestCase):\n"
                "    def test_inputs(self):\n"
                "        self.assertTrue(pathlib.Path('wheel-ready').is_file())\n"
                "        self.assertTrue(pathlib.Path('contracts-ready').is_file())\n",
                encoding="utf-8",
            )
            native = directory / "native.py"
            native.write_text(
                "import os, pathlib, socket, sys\n"
                "root = pathlib.Path.cwd()\n"
                "assert (root / 'wheel-ready').is_file()\n"
                "assert (root / 'contracts-ready').is_file()\n"
                "assert 'UV_PROJECT_ENVIRONMENT' not in os.environ\n"
                "if '--archive-directory' in sys.argv:\n"
                "    shared = pathlib.Path(sys.argv[sys.argv.index('--archive-directory') + 1])\n"
                "    assert (shared / 'ready').read_text() == 'extracted'\n"
                "    (root / sys.argv[1]).write_text(str(shared))\n"
                "with socket.create_connection(('127.0.0.1', int(os.environ['DOKIME_GRAPH_PORT']))) as peer:\n"
                "    peer.sendall(b'ready')\n"
                "    assert peer.recv(1) == b'1'\n"
                "print(sys.argv[1])\n"
                "sys.exit(7 if sys.argv[1] == 'system-rust' else 0)\n",
                encoding="utf-8",
            )
            wrapper = directory / "wrapper.py"
            wrapper.write_text(
                "import pathlib, sys\n"
                f"sys.path.insert(0, {str(repository / 'tools/python/src')!r})\n"
                "from dokime_tools.test import schedule\n"
                "root = pathlib.Path.cwd()\n"
                "if sys.argv[1] == 'prerequisite':\n"
                "    assert not (root / 'wheel-ready').exists()\n"
                "    sys.exit(0)\n"
                "schedule.paths.target = lambda: 'fixture'\n"
                "schedule.archive.install_wheels = lambda *_: (root / 'wheel-ready').touch()\n"
                "schedule.generated.restore = lambda *_: (root / 'contracts-ready').touch()\n"
                "def extract(directory, profile, destination):\n"
                "    assert not (root / 'extractions').exists()\n"
                "    (root / 'extractions').write_text('1')\n"
                "    (destination / 'ready').write_text('extracted')\n"
                "schedule.archive.extract = extract\n"
                "def command(category, language, profile, workers, extracted):\n"
                "    result = [sys.executable, str(root / 'native.py'), f'{category}-{language}']\n"
                "    return result + (['--archive-directory', str(extracted)] if extracted else [])\n"
                "schedule.command = command\n"
                "sys.exit(schedule.main(sys.argv[2:]))\n",
                encoding="utf-8",
            )
            tool = f'"{sys.executable}" "{wrapper}"'
            header = f"[settings]\njobs={configuration['settings']['jobs']}\n[vars]\ntools={json.dumps(tool)}\npython={json.dumps(tool)}\nlane=''\n"
            prerequisite = (
                "\n[tasks.prepare]\nrun="
                + json.dumps(tool + " prerequisite")
                + "\n[tasks.'prepare:generated']\nrun=''\n[tasks.'prepare:desktop']\ndepends=['prepare']\nrun=''\n"
            )
            config = directory / "mise.toml"
            config.write_text(header + tasks + prerequisite, encoding="utf-8")
            environment = dict(os.environ, UV_PROJECT_ENVIRONMENT="must-not-inherit")
            # The Isolated Fixture Owns Different Resources From Its Calling Test Aggregate.
            environment.pop("MISE_TASK_LOCKS", None)
            subprocess.run(["mise", "trust", str(config)], cwd=directory, env=environment, capture_output=True, check=True)
            with socket.socket() as listener:
                listener.bind(("127.0.0.1", 0))
                listener.listen()
                environment["DOKIME_GRAPH_PORT"] = str(listener.getsockname()[1])
                peers = []

                def rendezvous():
                    # Release Only After Every Lane Arrives: Serial Execution Cannot Complete.
                    for _ in schedule.policy.lanes(directory):
                        peer, _ = listener.accept()
                        peers.append(peer)
                    for peer in peers:
                        with peer.makefile("rb") as incoming:
                            self.assertEqual(incoming.read(5), b"ready")
                        peer.sendall(b"1")
                        peer.close()

                barrier = threading.Thread(target=rendezvous, daemon=True)
                barrier.start()
                result = subprocess.run(
                    ["mise", "run", "test"], cwd=directory, env=environment, capture_output=True, text=True, timeout=60, check=False
                )
                barrier.join()
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
            summary = next(directory.glob("target/test-reports/run-*/time-summary.json"))
            document = json.loads(summary.read_text(encoding="utf-8"))
            results = {row["lane"]: row for row in document["lanes"]}
            self.assertEqual(results["system-rust"]["exit_code"], 7)
            self.assertTrue(all(row["exit_code"] == 0 for name, row in results.items() if name != "system-rust"))
            self.assertEqual(len(results), len(schedule.policy.lanes(directory)) + 1)
            for row in results.values():
                self.assertTrue((directory / row["log"]).is_file())
            self.assertTrue(summary.with_suffix(".md").is_file())
            self.assertEqual((directory / "unit-rust").read_text(), (directory / "system-rust").read_text())
            self.assertFalse(Path((directory / "unit-rust").read_text()).exists())

    def test_missing_result_fails_the_aggregate(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            with (
                patch.object(schedule.policy, "lanes", return_value=[("unit", "rust")]),
                patch.object(schedule.archive, "install_wheels"),
                patch.object(schedule.archive, "extract"),
                patch.object(schedule.generated, "restore"),
            ):
                schedule.prepare(directory, "dev")
                self.assertEqual(schedule.finish(directory, "dev"), 1)

    def test_lane_retains_process_start_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            with (
                patch.object(schedule.policy, "lanes", return_value=[("unit", "rust")]),
                patch.object(schedule.archive, "install_wheels"),
                patch.object(schedule.archive, "extract"),
                patch.object(schedule.generated, "restore"),
                patch.object(schedule, "command", return_value=[str(directory / "absent-executable")]),
            ):
                schedule.prepare(directory, "dev")
                schedule.lane(directory, "dev", "unit", "rust", "sdk/python")
                self.assertEqual(schedule.finish(directory, "dev"), 1)
            result = next(directory.glob("target/test-reports/run-*/unit-rust.json"))
            self.assertEqual(json.loads(result.read_text(encoding="utf-8"))["exit_code"], 1)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
