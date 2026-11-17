# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Own One Plugin Process and Run a Native Story After Its Readiness Event."""

# === Imports === #

# Standard Library Imports
import os
import subprocess
import sys
from queue import Queue
from threading import Thread

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def read_output(process: subprocess.Popen[str], ready: Queue[str | None]) -> None:
    address = None
    try:
        if process.stdout is None:
            return
        for line in process.stdout:
            print(line, end="", flush=True)
            if address is None and line.startswith("harmos listening "):
                address = line.removeprefix("harmos listening ").strip()
                ready.put(address)
    finally:
        if address is None:
            ready.put(None)


def main() -> None:
    plugin, story, placement = sys.argv[1:]
    environment = dict(os.environ)
    environment.pop("HARMOS_ADDRESS", None)
    arguments = [plugin]
    if placement == "remote":
        arguments += ["--address", "127.0.0.1:0"]
    creationflags = spawn_flags()
    process = subprocess.Popen(
        arguments,
        env=environment,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=sys.stderr,
        text=True,
        encoding="utf-8",
        errors="replace",
        creationflags=creationflags,
    )
    ready: Queue[str | None] = Queue()
    output = Thread(target=read_output, args=(process, ready))
    output.start()
    try:
        address = ready.get()
        if address is None:
            raise RuntimeError("Start Plugin: Inspect the Plugin Output Above and Correct Its Startup Failure")
        subprocess.run(
            [story, address, placement],
            check=True,
            creationflags=creationflags,
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            timeout=240,
        )
        if placement == "managed":
            returncode = process.wait(timeout=240)
            if returncode:
                raise RuntimeError(
                    f"Stop Managed Plugin Failed (Exit {returncode}): Inspect the Plugin Output Above and Correct Its Shutdown"
                )
    finally:
        if process.poll() is None:
            process.terminate()
            process.wait()
        output.join()
        if process.stdin is not None:
            process.stdin.close()
        if process.stdout is not None:
            process.stdout.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
