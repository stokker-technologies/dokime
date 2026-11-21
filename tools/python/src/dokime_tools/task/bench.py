# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Select Bench Tasks from the Public Arguments."

# === Imports === #

# Standard Library Imports
import argparse

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


def select(args: argparse.Namespace, parser: argparse.ArgumentParser) -> list[tuple[str, list[str]]]:
    tasks: list[tuple[str, list[str]]] = []
    layers = [args.layer] if args.layer else ["unit", "integration", "system"]
    if args.binary in {"system-sdk", "protocol"}:
        if args.language not in (None, "rust") or args.layer not in (None, "system"):
            parser.error("SDK and protocol benchmarks use the Rust system producer")
        layers = ["system"]
    if args.binary == "storage":
        if args.language not in (None, "rust") or args.refresh_baseline:
            parser.error("storage benchmarks use Rust and native Criterion output; baseline promotion is separate")
        layers = []
        options = ["--select", args.select] if args.select else []
        if args.once:
            options += ["--once"]
        tasks = [("bench:rust:storage", options)]
    if "acceptance" in layers:
        parser.error("bench layers are unit, integration and system")
    for language in [args.language] if args.language else ["rust"]:
        for layer in layers:
            if language != "rust" and (language != "python" or layer != "system"):
                continue
            options = ["--refresh-baseline"] if args.refresh_baseline else []
            if args.once:
                if language != "rust":
                    parser.error("--once selects native Rust benchmarks")
                options += ["--once"]
            if args.binary:
                options += ["--binary", args.binary]
            if args.select:
                options += ["--select", args.select]
            tasks.append((f"bench:{language}:{layer}", options))
    return tasks


# === Private Functions === #

# === Entry Point === #
