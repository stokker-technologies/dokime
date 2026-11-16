# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Select Publish Tasks from the Public Arguments."

# === Imports === #

# Standard Library Imports
import argparse
import os

# isort: split

# Third Party Imports

# isort: split

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def select(args: argparse.Namespace, parser: argparse.ArgumentParser) -> list[tuple[str, list[str]]] | None:
    tasks: list[tuple[str, list[str]]] = []
    if args.registry:
        if args.channel != "stable" or args.stage:
            parser.error("--registry pypi requires stable publication without --stage")
        tasks = [("release:publish:python:dry" if args.dry else "release:publish:python", [])]
        return tasks
    if args.channel == "nightly" and not args.dry:
        if args.stage:
            parser.error("nightly publishes the complete package inventory")
        from .. import policy
        from ..release import nightly

        nightly.publish(policy.root())
        return None
    if args.dry:
        # The Chained Crate Dry Run Reads The Channel; Only Nightly Reports Git Pins As Pending.
        os.environ["RELEASE_CHANNEL"] = args.channel
        tasks = [("release:publish:dry", [])]
    else:
        tasks = []
        for tasks_stage in [args.stage] if args.stage else ["assets", "finalize"]:
            tasks.append((f"release:publish:{tasks_stage}", []))
    return tasks


# === Private Functions === #

# === Entry Point === #
