# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Preserve Owned Development Process Cleanup"""

# === Imports === #

# Standard Library Imports

import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


# Third Party Imports

import pytest


# Local Imports

from dokime_tools import desktop


# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_development_config() -> None:
    command, environment, description = desktop.dev_plan(
        {
            "DOKIME_DEV_HOST": "127.0.0.1",
            "DOKIME_DEV_PORT": "6199",
            "TAURI_CONFIG": "",
        },
        [],
    )
    assert description["devUrl"] == "http://127.0.0.1:6199"
    assert description["tauriConfig"]["build"]["devUrl"] == description["devUrl"]
    assert description["tauriConfig"]["bundle"]["externalBin"] == []
    assert command[:4] == [
        "node",
        "./node_modules/@tauri-apps/cli/tauri.js",
        "dev",
        "--config",
    ]
    assert json.loads(command[4])["bundle"]["externalBin"] == []
    assert "TAURI_CONFIG" not in environment
    assert desktop.merge_config(
        {"a": [1], "b": {"x": 1}}, {"a": [2], "b": {"y": 2}}
    ) == {"a": [2], "b": {"x": 1, "y": 2}}


def test_descendant_cleanup(tmp_path: Path) -> None:
    if os.name == "nt":
        pytest.skip("POSIX process groups are Unix-only")
    for mode in ["signal", "leader", "stubborn"]:
        directory = tmp_path / mode
        directory.mkdir()
        ready, stopped, finish = [
            directory / name for name in ["ready", "stopped", "finish"]
        ]
        handler = (
            "signal.SIG_IGN"
            if mode == "stubborn"
            else f"lambda *_: (Path({str(stopped)!r}).write_text('done'), exit(0))"
        )
        child_source = f"""
import os, signal, time
from pathlib import Path
signal.signal(signal.SIGTERM, {handler})
Path({str(ready)!r}).write_text(str(os.getpid()))
while True: time.sleep(0.02)
"""
        leader_source = f"""
import subprocess, sys, time
from pathlib import Path
subprocess.Popen([sys.executable, '-c', {child_source!r}])
while not Path({str(finish)!r}).exists(): time.sleep(0.02)
"""
        command = [sys.executable, "-c", leader_source]
        supervisor_source = f"""
from pathlib import Path
from dokime_tools.desktop import supervise
import os
raise SystemExit(supervise([{command!r}], cwd=Path({str(directory)!r}), environment=dict(os.environ)))
"""
        unrelated = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(30)"]
        )
        supervisor = subprocess.Popen([sys.executable, "-c", supervisor_source])
        owned_pid = None
        try:
            _wait(ready)
            owned_pid = int(ready.read_text())
            if mode == "signal":
                supervisor.send_signal(signal.SIGTERM)
            else:
                finish.write_text("done")
            if mode != "stubborn":
                _wait(stopped)
            assert supervisor.wait(timeout=10) == (
                128 + signal.SIGTERM if mode == "signal" else 0
            )
            deadline = time.monotonic() + 2
            while _running(owned_pid) and time.monotonic() < deadline:
                time.sleep(0.02)
            assert not _running(owned_pid), (
                "owned descendants must not survive their leader"
            )
            assert unrelated.poll() is None
            os.kill(unrelated.pid, 0)
        finally:
            if owned_pid is not None and _running(owned_pid):
                os.kill(owned_pid, signal.SIGKILL)
            if supervisor.poll() is None:
                supervisor.terminate()
                supervisor.wait(timeout=10)
            unrelated.terminate()
            unrelated.wait(timeout=10)


# === Private Functions === #


def _running(pid: int) -> bool:
    status = Path(f"/proc/{pid}/stat")
    if sys.platform == "linux":
        try:
            return status.read_text().split(") ", 1)[1][0] != "Z"
        except FileNotFoundError:
            return False
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    return True


def _wait(path: Path) -> None:
    deadline = time.monotonic() + 10
    while not path.exists():
        if time.monotonic() >= deadline:
            raise AssertionError(f"timed out waiting for {path}")
        time.sleep(0.02)


# === Entry Point === #
