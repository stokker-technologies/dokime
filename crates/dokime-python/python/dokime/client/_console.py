# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Shared capture options and console presentation."""

# === Imports === #

# Standard Library Imports
import argparse
import asyncio
from dataclasses import dataclass
import math
from pathlib import Path
import signal
import sys

# Third Party Imports

# Local Imports
from dokime.client import Capture, DokimeClient, RunRequest, RunPhase

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(kw_only=True)
class CaptureArgs:
    output: Path
    endpoint: str = "http://127.0.0.1:8765"
    token_file: Path | None = None
    ca: Path | None = None
    report: str = "cycle.summary"
    cancel_after: float | None = None


# === Public Functions === #


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--endpoint", default="http://127.0.0.1:8765")
    parser.add_argument("--token-file", type=Path)
    parser.add_argument("--ca", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--report", default="cycle.summary", help="Registered Report Identifier"
    )
    parser.add_argument("--cancel-after", type=positive_seconds)


async def capture_run(
    client: DokimeClient, request: RunRequest, capture: Capture, args: CaptureArgs
) -> int:
    stop_requested = asyncio.Event()
    previous = signal.signal(signal.SIGINT, lambda *_: stop_requested.set())
    try:
        async with asyncio.TaskGroup() as tasks:
            tasks.create_task(show_progress(capture))
            stopper = tasks.create_task(
                stop_when_requested(client, request, stop_requested, args.cancel_after)
            )
            result = await capture.collect(request, report=args.report)
            stopper.cancel()
        print(f"Completed — {result.result.status.title()}")
        print(f"Saved To: {result.output}")
        if result.request.phase is RunPhase.INTERRUPTED:
            return 3
        return 0 if result.result.status == "passed" else 1
    finally:
        signal.signal(signal.SIGINT, previous)


# Library updates contain no individual samples.
async def show_progress(capture: Capture) -> None:
    async for update in capture.updates():
        print(update.message, file=sys.stderr, flush=True)


# Stop remote work, then let collection finish.
async def stop_when_requested(
    client: DokimeClient,
    request: RunRequest,
    requested: asyncio.Event,
    after: float | None,
) -> None:
    try:
        await asyncio.wait_for(requested.wait(), timeout=after)
    except TimeoutError:
        pass
    async for status in request.updates():
        if status.terminal:
            return
        if status.last_execution_id is not None:
            print("Stopping Execution...", file=sys.stderr, flush=True)
            control = await client.execution(status.last_execution_id)
            await control.stop()
            return


# === Private Functions === #


def positive_seconds(value: str) -> float:
    seconds = float(value)
    if not math.isfinite(seconds) or seconds <= 0:
        raise argparse.ArgumentTypeError("must be a finite number greater than zero")
    return seconds


# === Entry Point === #
