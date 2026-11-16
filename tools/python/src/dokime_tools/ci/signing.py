# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Select, Sign and Verify Windows Product Bytes Before Packaging."""

# === Imports === #

# Standard Library Imports
import argparse
import os
import subprocess
from pathlib import Path

# isort: split

# Third Party Imports
# isort: split

# Local Imports
from ..process import spawn_flags
from . import transport

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("select", "package"))
    parser.add_argument("--profile", choices=("dev", "release"), required=True)
    options = parser.parse_args(arguments)
    if os.name != "nt":
        parser.error("Sign Windows Product: Run This Task on Windows")
    if options.operation == "select":
        transport.receive()
        provider = subprocess.check_output(
            ["pwsh", "-NoProfile", "-File", "tools/powershell/authenticode.ps1", "-Operation", "Select"],
            text=True,
            encoding="utf-8",
            creationflags=spawn_flags(),
        ).strip()
        if provider not in {"azure", "pfx", "none"}:
            raise ValueError("Select Signing Provider: Configure Azure or PFX Credentials")
        binary = Path(f"target/build/{options.profile}/windows-x86_64/bin").resolve()
        if not all((binary / name).is_file() for name in ("dokime.exe", "dokime-gui.exe")):
            raise FileNotFoundError("Select Signing Provider: Download the Matching Desktop and CLI Product Build")
        destination = Path(os.environ["GITHUB_OUTPUT"])
        previous = destination.read_text(encoding="utf-8") if destination.exists() else ""
        destination.write_text(previous + f"provider={provider}\nbin={binary}\n", encoding="utf-8", newline="\n")
        if provider == "none":
            print("::notice::Authenticode Skipped: No Signing Secrets Configured; Packaging Continues.")
        return 0
    provider = os.environ["DOKIME_AUTHENTICODE_PROVIDER"]
    if provider != "none":
        operation = "Sign" if provider == "pfx" else "Verify"
        for file in sorted(Path(os.environ["DOKIME_BIN"]).glob("*.exe")):
            subprocess.run(
                ["pwsh", "-NoProfile", "-File", "tools/powershell/authenticode.ps1", "-Operation", operation, "-Path", str(file)],
                check=True,
                creationflags=spawn_flags(),
            )
        subprocess.run(["mise", "run", "ci:distribution", "--", "--profile", options.profile], check=True, creationflags=spawn_flags())
    os.environ["CI_INPUT_ARCHIVES"] = "[]"
    result = subprocess.run(
        ["mise", "run", "package", "--", "--language", "rust", "--profile", options.profile], check=False, creationflags=spawn_flags()
    )
    return result.returncode


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
