# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Select Test Tasks from the Public Arguments."

# === Imports === #

# Standard Library Imports
import argparse
from pathlib import Path

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


def select(args: argparse.Namespace, languages: list[str], profile: list[str]) -> list[tuple[str, list[str]]]:
    tasks: list[tuple[str, list[str]]] = []
    if args.wheel:
        tasks = [("test:python:wheel", profile + (["--stubs"] if args.bindings else []))]
    elif args.coverage:
        tasks = []
        for tasks_language in languages:
            if tasks_language not in ("rust", "python"):
                continue
            tasks.append((f"test:{tasks_language}:coverage", profile))

    else:
        layers = [args.layer] if args.layer else ["unit", "integration", "acceptance", "system"]
        if "python" in languages and "integration" in layers:
            tasks.append(
                (
                    "test:python:integration",
                    profile + ["--project", "tools/python"] + (["--select", args.select] if args.select else []),
                )
            )
        if args.project != "tools/python":
            for language in languages:
                for layer in layers:
                    if language == "typescript" and layer == "acceptance":
                        continue
                    if language == "python" and not (Path("sdk/python/tests") / layer).is_dir():
                        continue
                    options = profile.copy()
                    if args.select:
                        options += ["--select", args.select]
                    tasks.append((f"test:{language}:{layer}", options))
    return tasks


# === Private Functions === #

# === Entry Point === #
