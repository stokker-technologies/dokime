# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Test Execution Operations."""

# === Imports === #

# Standard Library Imports
import os
import signal
import subprocess
import sys
from collections.abc import Sequence
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from ..process import spawn_flags
from . import policy

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def console() -> dict:
    """Keep Native Console Signaling Available Without a Visible Window."""
    if os.name != "nt":
        return {}
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = subprocess.SW_HIDE
    return {
        "startupinfo": startup,
        "creationflags": spawn_flags(subprocess.CREATE_NEW_CONSOLE),
        "stdin": sys.stdin,
        "stdout": sys.stdout,
        "stderr": sys.stderr,
    }


def execute(command: Sequence[str], cwd: Path, environment: dict[str, str] | None, *, timeout_s: float | None = None) -> int:
    from .job import own_children

    own_children()
    if os.name == "nt":
        command = [sys.executable, "-m", "dokime_tools.test.job", "--", *command]
    try:
        process = subprocess.Popen(
            list(command), cwd=cwd, env=environment, start_new_session=os.name != "nt" and timeout_s is not None, **console()
        )
        try:
            try:
                return process.wait(timeout=timeout_s)
            except subprocess.TimeoutExpired:
                print(f"{command[0]}: script lane exceeded its {timeout_s}-second hang guard", file=sys.stderr)
                if os.name == "nt":
                    subprocess.run(
                        ["taskkill.exe", "/PID", str(process.pid), "/T", "/F"],
                        check=False,
                        capture_output=True,
                        creationflags=spawn_flags(),
                    )
                else:
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                if process.poll() is None:
                    process.kill()
                process.wait()
                return 1
        finally:
            if process.stdout is not None:
                process.stdout.close()
            if process.stderr is not None:
                process.stderr.close()
            if process.stdin is not None:
                process.stdin.close()
            process.wait()
    except OSError as error:
        raise policy.TestError(f"Run Execution Tests: {command[0]} is unavailable: {error}") from error


def measure(command: Sequence[str], cwd: Path, environment: dict[str, str], *, output: Path | None = None) -> tuple[int, int | None]:
    """Exit status and peak resident bytes of one process and the children it waited for."""
    from .job import own_children

    own_children()
    if os.name == "nt":
        command = [sys.executable, "-m", "dokime_tools.test.job", "--", *command]
    try:
        spawn_options = console()
        if output is not None:
            spawn_options.update(stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace")
        process = subprocess.Popen(list(command), cwd=cwd, env=environment, **spawn_options)
    except OSError as error:
        raise policy.TestError(f"Run Execution Tests: {command[0]} is unavailable: {error}") from error
    if output is not None:
        stream = process.stdout
        assert stream is not None, "Measure Native Benchmark: Capture Requires An Output Pipe"
        written = output.open("w", encoding="utf-8")
        try:
            try:
                for line in stream:
                    print(line, end="", flush=True)
                    written.write(line)
            finally:
                stream.close()
        finally:
            written.close()
    if not hasattr(os, "wait4"):
        return process.wait(), None
    _, status, usage = os.wait4(process.pid, 0)
    process.returncode = os.waitstatus_to_exitcode(status)
    scale = 1 if sys.platform == "darwin" else 1024
    return process.returncode, usage.ru_maxrss * scale


# === Private Functions === #

# === Entry Point === #
