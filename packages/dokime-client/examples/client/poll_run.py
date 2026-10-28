# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Poll a remote test. Edit the constants below, then run this file."""

import asyncio
import json
import math
import sys
from collections import Counter
from contextlib import aclosing
from dataclasses import asdict
from pathlib import Path

from dokime_client.client import DokimeClient, Execution, Value

PAIRING_FILE = Path("lab.dokime-pair")
CYCLE_LABEL = "Polling Demonstration"
TEMPLATE = "obc.checks.pulse"
PARAMETERS: dict[str, Value] = {"sample_count": 8}


POLL_INTERVAL = 0.5


async def poll(handle: Execution, interval: float) -> dict[str, object]:
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

    client = DokimeClient.from_pairing(PAIRING_FILE)

    try:
        template = await client.template(TEMPLATE)
        (test_case,) = await template.materialize(parameters=PARAMETERS)
        handle = Execution(test_case)
        cycle = await client.create_cycle(label=CYCLE_LABEL, continue_automatically=False)
        print(f"Cycle: {cycle.id} Execution: {handle.id}", flush=True)
        Path("cycle.json").write_text(json.dumps({"cycle_id": cycle.id}, indent=2) + "\n", encoding="utf-8")
        await cycle.append([handle])
        await handle.start()
        summary = await poll(handle, POLL_INTERVAL)

        print("Summary:", json.dumps(summary, indent=2, default=str), flush=True)
        Path("summary.json").write_text(json.dumps(summary, indent=2, default=str) + "\n", encoding="utf-8")
        await cycle.seal()

    finally:
        # Disconnecting the client does not cancel the execution.
        await client.close()


# Keep one iterator alive to retain cursors.
async def count_samples(handle: Execution, counts: Counter[str], interval: float) -> None:
    async with aclosing(handle.samples(follow=True, poll_interval=interval)) as reader:
        async for sample in reader:
            counts[sample.stream] += 1


# Summarize without printing individual records.
async def count_records(handle: Execution, counts: Counter[str]) -> None:
    async with aclosing(handle.records(follow=True)) as reader:
        async for record in reader:
            counts[record.type] += 1


if __name__ == "__main__":
    asyncio.run(main())
