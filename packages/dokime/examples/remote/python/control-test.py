# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Run And Export Two Simulated Executions"""

# === Imports === #

# Standard Library Imports
import argparse
import asyncio
import json
import sys
from collections import Counter
from collections.abc import Sequence
from contextlib import aclosing
from pathlib import Path
from uuid import uuid4

# Third Party Imports
from dokime.client import DokimeClient, DokimeError, TestExecutionHandle

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def observe(
    execution: TestExecutionHandle,
    *,
    record_types: Sequence[str],
    streams: Sequence[str],
    poll_interval: float = 0.2,
) -> dict[str, object]:
    """Follow Both Evidence Sources Until Their Final Pages Have Drained."""
    records: Counter[str] = Counter()
    samples: Counter[str] = Counter()

    async def read_records() -> None:
        async with aclosing(execution.records(record_types=record_types, follow=True, poll_interval=poll_interval)) as reader:
            async for record in reader:
                records[record.type] += 1
                print(f"Record: {record.type} #{record.sequence}", flush=True)

    async def read_samples() -> None:
        async with aclosing(execution.samples(streams=streams, follow=True, poll_interval=poll_interval)) as reader:
            async for sample in reader:
                samples[sample.stream] += 1
                print(f"Sample: {sample.stream} #{sample.sequence} {sample.values}", flush=True)

    async with asyncio.TaskGroup() as listeners:
        listeners.create_task(read_records())
        listeners.create_task(read_samples())
        outcome = await execution.wait(poll_interval=poll_interval)

    # A Durable Outcome Alone Does Not Prove That Listeners Drained.
    return {
        "execution_id": execution.id,
        "status": outcome.status,
        "records_by_type": dict(records),
        "samples_by_stream": dict(samples),
    }


async def run(arguments: argparse.Namespace) -> int:
    directory: Path = arguments.directory
    directory.mkdir(parents=True, exist_ok=True)
    for name in ("execution-1.zip", "execution-2.zip", "cycle.zip"):
        destination = directory / name
        if destination.exists() and not arguments.overwrite:
            raise FileExistsError(f"{destination} exists; choose another directory or pass --overwrite")

    async with DokimeClient(
        arguments.endpoint,
        key_file=arguments.key_file,
        env_file=arguments.env_file,
        ca_certificate=arguments.ca_certificate,
    ) as client:
        cycle_id: str | None = None
        execution: TestExecutionHandle | None = None
        exported: set[str] = set()
        destination: Path | None = None
        successful = True
        try:
            cycle_id = await client.create_cycle(label="Remote Pulse Demonstration")
            print(f"Cycle: {cycle_id}", flush=True)
            cycle = await client.cycle(cycle_id)

            for index in (1, 2):
                (execution,) = await client.append_cases(
                    cycle_id,
                    suite="obc.checks",
                    template="obc.checks.pulse",
                    parameters={"sample_count": arguments.sample_count},
                )
                request_key = str(uuid4())
                destination = directory / f"execution-{index}.zip"
                print(f"Execution: {execution.id} Request Key: {request_key} Mode: automatic", flush=True)
                await execution.start(request_key=request_key)
                snapshot = await execution.snapshot()
                status = snapshot.get("summary", {})
                print("Current Status:", json.dumps(status), flush=True)

                summary = await observe(execution, record_types=("obc.run.started", "obc.run.completed"), streams=("obc.pulse",))
                print("Drained Evidence:", json.dumps(summary), flush=True)
                successful = successful and summary["status"] == "passed"
                saved = await execution.export(destination, overwrite=arguments.overwrite)
                exported.add(execution.id)
                print(f"Execution Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)

            await cycle.seal()
            saved_cycle = await cycle.export(directory / "cycle.zip", overwrite=arguments.overwrite)
            print(f"Cycle Evidence: {saved_cycle.path} SHA-256: {saved_cycle.sha256}", flush=True)
            return 0 if successful else 1

        except asyncio.CancelledError:
            print("Cancellation Requested; Waiting For Saved Evidence", file=sys.stderr, flush=True)
            if cycle_id is None:
                print("Cycle Creation Was Not Confirmed; Inspect The Host Before Creating Another", file=sys.stderr, flush=True)
                raise
            if execution is not None and await execution.checkpoint() is None:
                await execution.stop()
            # Close Admission Before Waiting On An Interrupted Submission.
            cycle = await client.cycle(cycle_id)
            await cycle.seal()
            if execution is not None:
                outcome = await execution.wait()
                print(f"Saved Outcome: {outcome.status} Execution: {execution.id}", flush=True)
                if execution.id not in exported and destination is not None:
                    await execution.export(destination, overwrite=arguments.overwrite)

        # Only This Script's Newly Created Cycle Is Sealed.
        saved_cycle = await cycle.export(directory / "cycle.zip", overwrite=arguments.overwrite)
        print(f"Cycle Evidence: {saved_cycle.path} SHA-256: {saved_cycle.sha256}", flush=True)
        return 130


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)

    parser.add_argument("--endpoint", required=True)
    parser.add_argument("--ca-certificate", type=Path)
    parser.add_argument("--key-file", type=Path)
    parser.add_argument("--env-file", type=Path, help="Read this explicit dotenv file; process variables take precedence")

    parser.add_argument("--directory", type=Path, default=Path("test-evidence"))
    parser.add_argument("--overwrite", action="store_true")
    parser.add_argument("--sample-count", type=int, choices=range(1, 65), default=8, metavar="1..64")
    arguments = parser.parse_args()

    try:
        return asyncio.run(run(arguments))
    except KeyboardInterrupt:
        print("Client Closed; Cancellation May Be Unconfirmed. Retain The Cycle, Execution And Request Key Above", file=sys.stderr)
        return 130
    except (DokimeError, OSError, ValueError, ExceptionGroup) as error:
        print(f"Workflow Incomplete: {error}", file=sys.stderr)
        print("Remote Work May Continue. Recover With The Printed Identities; Do Not Create Another Cycle Blindly", file=sys.stderr)
        return 1


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
