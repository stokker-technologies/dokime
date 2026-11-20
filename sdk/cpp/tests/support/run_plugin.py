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
from concurrent.futures import ThreadPoolExecutor
from queue import Queue

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from grpc import ServicerContext, server
from harmos._wire.plugin.v1.launches_pb2 import LaunchesAnnounceRequest, LaunchesAnnounceResponse
from harmos._wire.plugin.v1.launches_pb2_grpc import LaunchesServicer, add_LaunchesServicer_to_server
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    plugin, story, placement = sys.argv[1:]
    environment = dict(os.environ)
    ready: Queue[str] = Queue()

    class Launches(LaunchesServicer):
        def Announce(self, request: LaunchesAnnounceRequest, context: ServicerContext) -> LaunchesAnnounceResponse:
            ready.put(request.address)
            return LaunchesAnnounceResponse()

    launches = server(ThreadPoolExecutor(max_workers=1))
    add_LaunchesServicer_to_server(Launches(), launches)
    port = launches.add_insecure_port("127.0.0.1:0")
    launches.start()
    environment.update(HARMOS_HOST_ENDPOINT=f"127.0.0.1:{port}", HARMOS_LAUNCH_TOKEN="fixture-launch")
    arguments = [plugin]
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
    try:
        address = ready.get(timeout=240)
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
        launches.stop(None).wait()
        if process.stdin is not None:
            process.stdin.close()
        if process.stdout is not None:
            process.stdout.close()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
