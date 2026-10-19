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
ENDPOINT = "https://127.0.0.1:8766"
CA_CERTIFICATE: Path | None = Path("server.pem")
KEY_FILE: Path | None = Path("remote.key")
ENV_FILE: Path | None = None

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


async def print_record(record: Record) -> None:
    print(f"Record: {record.type} #{record.sequence}", flush=True)


async def print_sample(sample: Sample) -> None:
    print(f"Sample: {sample.stream} #{sample.sequence} {sample.values}", flush=True)


async def run() -> int:
    if EXECUTION_COUNT < 1 or WAIT_SECONDS <= 0:
        raise ValueError("execution count and wait deadline must be positive")
    OUTPUT_DIRECTORY.mkdir(parents=True, exist_ok=True)

    async with DokimeClient(ENDPOINT, ca_certificate=CA_CERTIFICATE, key_file=KEY_FILE, env_file=ENV_FILE) as client:
        cycle: Cycle | None = None
        execution: Execution | None = None
        exported: set[str] = set()
        destination: Path | None = None
        try:
            # Materialize Cases And Construct Independent Attempts
            template = await client.template(TEMPLATE)
            cases = await template.materialize(parameters={"sample_count": SAMPLE_COUNT})
            executions = []
            for test_case in cases * EXECUTION_COUNT:
                executions.append(Execution(test_case))

            # Reserve Evidence Destinations Before Starting Any Device Work
            filenames = [f"execution-{index}.zip" for index in range(1, len(executions) + 1)] + ["cycle.zip"]
            for name in filenames:
                path = OUTPUT_DIRECTORY / name
                if path.exists() and not OVERWRITE:
                    raise FileExistsError(f"{path} exists; choose another output directory or enable OVERWRITE")

            # Create The Cycle And Append Without Starting
            cycle = await client.create_cycle(label=CYCLE_LABEL, continue_automatically=False)
            print(f"Cycle: {cycle.id}", flush=True)
            for selected in executions:
                print(f"Execution: {selected.id}", flush=True)
            await cycle.append(executions)

            # Run Sequentially, Drain Callbacks And Save Each Bundle
            successful = True
            for index, execution in enumerate(executions, start=1):
                destination = OUTPUT_DIRECTORY / f"execution-{index}.zip"
                records_by_type: Counter[str] = Counter()
                samples_by_stream: Counter[str] = Counter()

                async def record_received(record: Record, counts: Counter[str] = records_by_type) -> None:
                    counts[record.type] += 1
                    await print_record(record)

                async def sample_received(sample: Sample, counts: Counter[str] = samples_by_stream) -> None:
                    counts[sample.stream] += 1
                    await print_sample(sample)

                outcome = await execution.run(
                    record_types=RECORD_TYPES,
                    streams=STREAMS,
                    on_record=record_received,
                    on_sample=sample_received,
                    timeout=WAIT_SECONDS,
                )
                print(f"Saved Outcome: {outcome.status} Execution: {execution.id}", flush=True)
                drained = {
                    "execution_id": execution.id,
                    "status": outcome.status,
                    "records_by_type": dict(records_by_type),
                    "samples_by_stream": dict(samples_by_stream),
                }
                print("Drained Evidence:", json.dumps(drained, sort_keys=True), flush=True)
                successful = successful and outcome.status == "passed"
                saved = await execution.export(destination, overwrite=OVERWRITE)
                exported.add(execution.id)
                print(f"Execution Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)

            # Seal The Container And Save Its Complete Bundle
            await cycle.seal()
            saved = await cycle.export(OUTPUT_DIRECTORY / "cycle.zip", overwrite=OVERWRITE)
            print(f"Cycle Evidence: {saved.path} SHA-256: {saved.sha256}", flush=True)
            return 0 if successful else 1

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


def main() -> int:
    try:
        return asyncio.run(run())
    except KeyboardInterrupt:
        print("Recovery Was Not Confirmed; Retain The Printed Cycle And Execution Identities", file=sys.stderr)
        return 130
    except (DokimeError, OSError, ValueError, TimeoutError, ExceptionGroup) as error:
        print(f"Workflow Incomplete: {error}. Inspect The Printed Identities Before Resubmitting", file=sys.stderr)
        return 1


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
