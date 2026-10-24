# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Poll a remote test. Edit the constants below, then run this file."""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import math
import sys
from collections import Counter
from dataclasses import asdict
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_client.client import DokimeClient, TestExecutionHandle, Value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ENDPOINT = "http://127.0.0.1:8765"
SUITE = "mini.charger.steady"
TEMPLATE = "mini.charger.steady.measurement"
PARAMETERS: dict[str, Value] = {"duration_s": 1.0, "sample_interval_s": 0.1}

TOKEN_FILE: Path | None = None
CA_CERTIFICATE: Path | None = None

POLL_INTERVAL = 0.5

# === Types === #

# === Classes === #

# === Public Functions === #


async def poll(handle: TestExecutionHandle, interval: float) -> dict[str, object]:
    """Read forward cursors continuously, including final pages after completion."""
    samples: Counter[str] = Counter()
    events: Counter[str] = Counter()

    print("Reading Evidence...", file=sys.stderr, flush=True)

    async with asyncio.TaskGroup() as tasks:
        tasks.create_task(count_samples(handle, samples, interval))
        tasks.create_task(count_records(handle, events))
        outcome = await handle.wait(poll_interval=interval)

    # The Saved Outcome And Callback Drain Are Separate Guarantees.

    return {
        "run": {
            "execution_id": handle.id,
            "status": outcome.status,
            "result": asdict(outcome.result),
        },
        "samples_by_stream": dict(samples),
        "events_by_type": dict(events),
    }


async def main() -> None:
    if not math.isfinite(POLL_INTERVAL) or POLL_INTERVAL <= 0:
        raise ValueError("POLL_INTERVAL must be finite and greater than zero")

    token = TOKEN_FILE.read_text(encoding="utf-8").strip() if TOKEN_FILE else None
    client = DokimeClient(ENDPOINT, token=token, ca_certificate=CA_CERTIFICATE)

    try:
        cycle = await client.new_cycle(suite=SUITE, template=TEMPLATE, parameters=PARAMETERS)
        (handle,) = cycle.executions
        await handle.start(request_key="poll-example")
        summary = await poll(handle, POLL_INTERVAL)

        print("Summary:", json.dumps(summary, indent=2, default=str), flush=True)

    finally:
        # Disconnecting the client does not cancel the execution.
        await client.close()


# Keep one iterator alive to retain cursors.
async def count_samples(handle: TestExecutionHandle, counts: Counter[str], interval: float) -> None:
    async for sample in handle.samples(follow=True, poll_interval=interval):
        counts[sample.stream] += 1


# Summarize without printing individual records.
async def count_records(handle: TestExecutionHandle, counts: Counter[str]) -> None:
    async for record in handle.records(follow=True):
        counts[record.type] += 1


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    asyncio.run(main())
