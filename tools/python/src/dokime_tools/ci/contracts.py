# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Transfer One Explicit Canonical Contract Export To Native Consumers."""

# === Imports === #

# Standard Library Imports
import argparse
import os
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools import contracts
from dokime_tools.check.release import digest, source_revision

# Type Checking Imports

# === Public Exports === #

# === Constants === #
ARCHIVE = Path("target/contracts/dokime-contract.tar")

# === Types === #

# === Classes === #

# === Public Functions === #


def publish(root: Path) -> None:
    revision = source_revision(root)
    if revision != os.environ["GITHUB_SHA"]:
        raise ValueError("portable contracts require the exact native source commit")
    archive = root / ARCHIVE
    checksum = archive.with_suffix(".tar.sha256")
    expected = f"{digest(archive)}  {archive.name}\n"
    if checksum.read_text(encoding="ascii") != expected:
        raise ValueError("portable contract archive checksum differs")
    output = os.environ.get("GITHUB_OUTPUT")
    if output:
        with Path(output).open("a", encoding="utf-8") as stream:
            stream.write(f"sha256={digest(archive)}\n")
            stream.write(
                f"artifact_name=contracts-{os.environ['DOKIME_CI_PLATFORM']}-{os.environ['GITHUB_RUN_ID']}-{os.environ['GITHUB_RUN_ATTEMPT']}\n"
            )
            stream.write(
                f"tool_artifact_name=harmos-tools-{os.environ['DOKIME_CI_PLATFORM']}-{os.environ['GITHUB_RUN_ID']}-{os.environ['GITHUB_RUN_ATTEMPT']}\n"
            )


def restore(root: Path) -> None:
    export = Path(os.environ["DOKIME_CONTRACT_EXPORT"])
    if not export.is_absolute():
        export = root / export
    contracts.restore(
        root / ARCHIVE,
        os.environ["DOKIME_CONTRACT_SHA256"],
        os.environ["GITHUB_SHA"],
        export,
    )


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("publish", "restore"))
    options = parser.parse_args()
    {"publish": publish, "restore": restore}[options.command](Path.cwd())
