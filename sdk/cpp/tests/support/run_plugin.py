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

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    plugin, story, placement = sys.argv[1:]
    environment = dict(os.environ)
    environment.pop("HARMOS_ADDRESS", None)
    arguments = [plugin]
    if placement == "remote":
        arguments += ["--address", "127.0.0.1:0"]
    creationflags = subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
    with subprocess.Popen(arguments, env=environment, stdout=subprocess.PIPE, text=True, creationflags=creationflags) as process:
        try:
            if process.stdout is None:
                raise RuntimeError("Plugin Output Is Required")
            for line in process.stdout:
                if line.startswith("harmos listening "):
                    address = line.removeprefix("harmos listening ").strip()
                    break
            else:
                raise RuntimeError("Plugin Exited Before Readiness")
            subprocess.run(
                [story, address, placement], check=True, creationflags=creationflags, stdin=sys.stdin, stdout=sys.stdout, stderr=sys.stderr
            )
            if placement == "managed" and process.wait() != 0:
                raise RuntimeError("Managed Plugin Failed on Exit")
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait()


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
