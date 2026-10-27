# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Run Independent Attempts With Listeners And Saved Evidence"""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import sys
from collections import Counter
from pathlib import Path

# Third Party Imports
from dokime_client.client import Cycle, DokimeClient, DokimeError, Execution, Record, Sample

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# Connection Parameters
PAIRING_FILE = Path("../lab.dokime-pair")

# Case And Execution Parameters
TEMPLATE = "obc.checks.pulse"
CYCLE_LABEL = "Remote Pulse Demonstration"
SAMPLE_COUNT = 8
EXECUTION_COUNT = 2
WAIT_SECONDS = 60.0

# Observation Filters
RECORD_TYPES = ("obc.run.*",)
STREAMS = ("obc.pulse",)

# Evidence Files
OUTPUT_DIRECTORY = Path("test-evidence")
OVERWRITE = False

# === Types === #

# === Classes === #

# === Public Functions === #


async def run() -> int:
    # The Story Spells Out Two Attempts; Keep One To Study A Single Run.
    if EXECUTION_COUNT not in (1, 2) or WAIT_SECONDS <= 0:
        raise ValueError("EXECUTION_COUNT must be 1 or 2 and WAIT_SECONDS must be positive")
    OUTPUT_DIRECTORY.mkdir(parents=True, exist_ok=True)
    if (OUTPUT_DIRECTORY / "execution-1.zip").exists() and not OVERWRITE:
        raise FileExistsError("execution-1.zip exists; choose another output directory or enable OVERWRITE")
    if EXECUTION_COUNT == 2 and (OUTPUT_DIRECTORY / "execution-2.zip").exists() and not OVERWRITE:
        raise FileExistsError("execution-2.zip exists; choose another output directory or enable OVERWRITE")
    if (OUTPUT_DIRECTORY / "cycle.zip").exists() and not OVERWRITE:
        raise FileExistsError("cycle.zip exists; choose another output directory or enable OVERWRITE")

    async with DokimeClient.from_pairing(PAIRING_FILE) as client:
        cycle: Cycle | None = None
        execution: Execution | None = None
        exported: set[str] = set()
        destination: Path | None = None
        try:
            # Materialize One Immutable Case; Each Execution Gets Its Own UUID.
            template = await client.template(TEMPLATE)
            (test_case,) = await template.materialize(parameters={"sample_count": SAMPLE_COUNT})
            first = Execution(test_case)
            second = Execution(test_case)

            # Append Both Attempts Without Starting Hardware.
            cycle = await client.create_cycle(label=CYCLE_LABEL, continue_automatically=False)
            print(f"Cycle: {cycle.id}", flush=True)
            Path("cycle.json").write_text(json.dumps({"cycle_id": cycle.id}, indent=2) + "\n", encoding="utf-8")
            print(f"Execution: {first.id}", flush=True)
            if EXECUTION_COUNT == 2:
                print(f"Execution: {second.id}", flush=True)
                await cycle.append([first, second])
            else:
                await cycle.append([first])

            # Run Attempt 1; The Client Waits For Its Saved Outcome And Drains Observations.
            execution = first
            destination = OUTPUT_DIRECTORY / "execution-1.zip"
            records: asyncio.Queue[Record] = asyncio.Queue()
            samples: asyncio.Queue[Sample] = asyncio.Queue()
            outcome = await first.run(
                record_types=RECORD_TYPES,
                streams=STREAMS,
                on_record=records.put,
                on_sample=samples.put,
                timeout=WAIT_SECONDS,
            )

            # Print The Returned Observations In Order Within Each Reader.
            records_by_type: Counter[str] = Counter()
            samples_by_stream: Counter[str] = Counter()
            for _ in range(records.qsize()):
                record = records.get_nowait()
                records_by_type[record.type] += 1
                print(f"Record: {record.type} #{record.sequence}", flush=True)
            for _ in range(samples.qsize()):
                sample = samples.get_nowait()
                samples_by_stream[sample.stream] += 1
                print(f"Sample: {sample.stream} #{sample.sequence} {sample.values}", flush=True)
            print(f"Saved Outcome: {outcome.status} Execution: {first.id}", flush=True)
            drained = {
                "execution_id": first.id,
                "status": outcome.status,
                "records_by_type": dict(records_by_type),
                "samples_by_stream": dict(samples_by_stream),
            }
            print("Drained Evidence:", json.dumps(drained, sort_keys=True), flush=True)
            Path("execution-1.json").write_text(json.dumps(drained, indent=2, sort_keys=True) + "\n", encoding="utf-8")
            saved = await first.export(OUTPUT_DIRECTORY / "execution-1.zip", overwrite=OVERWRITE)
            exported.add(first.id)
            print(f"Execution Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)

            if EXECUTION_COUNT == 2:
                # Run Attempt 2; The Client Waits For Its Saved Outcome And Drains Observations.
                execution = second
                destination = OUTPUT_DIRECTORY / "execution-2.zip"
                records = asyncio.Queue()
                samples = asyncio.Queue()
                outcome = await second.run(
                    record_types=RECORD_TYPES,
                    streams=STREAMS,
                    on_record=records.put,
                    on_sample=samples.put,
                    timeout=WAIT_SECONDS,
                )

                # Print The Returned Observations In Order Within Each Reader.
                records_by_type = Counter()
                samples_by_stream = Counter()
                for _ in range(records.qsize()):
                    record = records.get_nowait()
                    records_by_type[record.type] += 1
                    print(f"Record: {record.type} #{record.sequence}", flush=True)
                for _ in range(samples.qsize()):
                    sample = samples.get_nowait()
                    samples_by_stream[sample.stream] += 1
                    print(f"Sample: {sample.stream} #{sample.sequence} {sample.values}", flush=True)
                print(f"Saved Outcome: {outcome.status} Execution: {second.id}", flush=True)
                drained = {
                    "execution_id": second.id,
                    "status": outcome.status,
                    "records_by_type": dict(records_by_type),
                    "samples_by_stream": dict(samples_by_stream),
                }
                print("Drained Evidence:", json.dumps(drained, sort_keys=True), flush=True)
                Path("execution-2.json").write_text(json.dumps(drained, indent=2, sort_keys=True) + "\n", encoding="utf-8")
                saved = await second.export(OUTPUT_DIRECTORY / "execution-2.zip", overwrite=OVERWRITE)
                exported.add(second.id)
                print(f"Execution Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)

            # Seal The Container And Save Its Complete Bundle
            await cycle.seal()
            saved = await cycle.export(OUTPUT_DIRECTORY / "cycle.zip", overwrite=OVERWRITE)
            print(f"Cycle Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)
            return 0

        except asyncio.CancelledError:
            # Recover Only This Script's Execution And Cycle
            print("Cancellation Requested; Waiting For Saved Evidence", file=sys.stderr, flush=True)
            if cycle is None:
                print("Cycle Creation Was Not Confirmed; Inspect The Server Before Creating Another", file=sys.stderr, flush=True)
                raise
            if execution is not None and await execution.checkpoint() is None:
                await execution.stop()
            await cycle.seal()
            if execution is not None:
                outcome = await execution.wait()
                print(f"Saved Outcome: {outcome.status} Execution: {execution.id}", flush=True)
                if execution.id not in exported and destination is not None:
                    await execution.export(destination, overwrite=OVERWRITE)
            saved = await cycle.export(OUTPUT_DIRECTORY / "cycle.zip", overwrite=OVERWRITE)
            print(f"Cycle Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)
            return 130


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    try:
        raise SystemExit(asyncio.run(run()))
    except KeyboardInterrupt:
        print("Recovery Was Not Confirmed; Retain The Printed Cycle And Execution Identities", file=sys.stderr)
        raise SystemExit(130)
    except (DokimeError, OSError, ValueError, TimeoutError, ExceptionGroup) as error:
        print(f"Workflow Incomplete: {error}. Inspect The Printed Identities Before Resubmitting", file=sys.stderr)
        raise SystemExit(1)
