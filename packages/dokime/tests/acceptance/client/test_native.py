# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Capture And Stop Charger Runs Against The Built Server And Plugin Bundle"""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import shutil
import subprocess
from collections.abc import Callable, Iterator
from dataclasses import dataclass
from pathlib import Path
from typing import IO, Any

# Third Party Imports
import pytest

# Local Imports
from dokime.client import Capture, DokimeClient, ExecutionEvidence, RunPhase

# Type Checking Imports

# === Public Exports === #

# === Constants === #

SUITE = "obc.charger.steady"
PARAMETERS = {
    "ac_voltage_v": 230.0,
    "load_ohm": 100.0,
    "frequency_hz": 1.0,
    "duration_s": 0.3,
    "sample_interval_s": 0.1,
}
# The Packaging Lane Ships The Completed Capture's Sealed Report As Its Example.
EXAMPLE_REPORT = "target/validation/examples/charger-report.html"

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class Host:
    endpoint: str
    process: subprocess.Popen[bytes]
    log: Path


# === Public Functions === #


@pytest.fixture(scope="module")
def native(
    host: Path,
    artifact: Callable[[str], Path],
    tmp_path_factory: pytest.TempPathFactory,
) -> Iterator[Host]:
    bundle = artifact("plugins/obc-plugin.dokime")
    root = tmp_path_factory.mktemp("native")
    env_file = root / "empty.env"
    env_file.touch()
    log = root / "backend.log"

    # Start The Server And Await Its Ready Line
    with log.open("w") as stderr:
        process = subprocess.Popen(
            [
                str(host),
                "--port",
                "0",
                "--dev-control",
                "--shutdown-stdin",
                "--application-root",
                str(root / "backend"),
                "--env-file",
                str(env_file),
                "--plugin",
                str(bundle),
                "--no-seed-validation-artifact",
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=stderr,
        )
        try:
            assert process.stdin is not None and process.stdout is not None
            ready = _event(process.stdout, {"ready"}, log)

            # Dev-control starts with no plugins; explicitly load the selected
            # bundle through the supervisor protocol before exercising gRPC.
            reload = {"command": "reload", "path": str(bundle)}
            process.stdin.write((json.dumps(reload) + "\n").encode())
            process.stdin.flush()
            loaded = _event(process.stdout, {"reloaded", "reload_failed"}, log)
            assert loaded["event"] == "reloaded", loaded

            yield Host(ready["endpoint"], process, log)

            # Closing Stdin Is The Owned Shutdown Request
            assert process.poll() is None, "client close must not stop the backend"
            process.stdin.close()
            assert process.wait(15) == 0, log.read_text()
        finally:
            if process.poll() is None:
                if process.stdin is not None and not process.stdin.closed:
                    process.stdin.close()
                try:
                    process.wait(15)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


def test_native_capture(native: Host, workspace: Path, tmp_path: Path) -> None:
    output = tmp_path / "capture"

    async def scenario() -> None:
        async with DokimeClient(native.endpoint) as client:
            suite = await client.suite(SUITE)
            assert suite.templates
            capture = await Capture.prepare(
                client,
                output=output,
                suite=suite.id,
                template=suite.templates[0].id,
                parameters=PARAMETERS,
            )
            run = await capture.start()
            cycle = await client.cycle(run.cycle_id)
            (handle,) = cycle.executions
            records = []
            samples = []
            live_sample = asyncio.Event()

            async def events() -> None:
                async for event in handle.records(follow=True):
                    records.append(event)

            async def telemetry() -> None:
                async for sample in handle.samples(
                    follow=True, poll_interval=0.05, page_size=2
                ):
                    samples.append(sample)
                    live_sample.set()

            # Follow Live Evidence And Answer The Operator Prompt
            async with asyncio.TaskGroup() as tasks:
                tasks.create_task(events())
                tasks.create_task(telemetry())
                await live_sample.wait()
                assert not (await run.status()).terminal
                pending = await _pending(cycle)
                assert "submit" in pending[0].actions
                await cycle.respond(
                    pending[0].id,
                    action="submit",
                    values={
                        "operator_note": "Python client interoperability",
                        "accepted": True,
                    },
                )
                status = await run.wait()
            assert status.phase is RunPhase.COMPLETED
            assert records and samples
            assert len({e.id for e in records}) == len(records)
            assert len({s.id for s in samples}) == len(samples)
            assert len(cycle.executions) == 1

            # Replay Matches What Was Followed Live
            filtered = [e async for e in handle.records(record_ids=[records[-1].id])]
            assert len(filtered) == 1 and filtered[0].id == records[-1].id
            replay = [s async for s in handle.samples(page_size=2)]
            assert {s.id for s in samples} == {s.id for s in replay}

            # Sealed History Holds The Execution's Own Evidence
            point = await handle.checkpoint()
            assert point is not None
            captured = ExecutionEvidence(handle._transport, point, handle._backend)
            document = await captured.document()
            assert document["execution_id"] == captured.checkpoint.execution_id
            assert document["snapshot"]["parameters"]
            assert await captured.traceability()
            historical_records = [item async for item in captured.records()]
            historical_samples = [item async for item in captured.samples()]
            own = captured.checkpoint.execution_id
            assert {
                item.record.id
                for item in historical_records
                if item.record.scope.get("execution_id") == own
            } == {item.id for item in records if item.scope.get("execution_id") == own}
            assert {item.sample.id for item in historical_samples} == {
                item.id for item in samples
            }

            # The Capture Saves Live Records And The Sealed Report
            await capture.collect(run)
            assert capture.result.output == output
            saved_records = [
                json.loads(line)
                for line in (output / "records.jsonl").read_text().splitlines()
            ]
            assert {
                item["id"]
                for item in saved_records
                if item["scope"].get("execution_id") == handle.id
            } == {item.id for item in records}
            assert (output / "report.html").read_text(encoding="utf-8").strip()

    asyncio.run(scenario())
    _export_example(output / "report.html", workspace / EXAMPLE_REPORT)


def test_native_stop(native: Host, tmp_path: Path) -> None:
    async def scenario() -> None:
        async with DokimeClient(native.endpoint) as client:
            suite = await client.suite(SUITE)
            output = tmp_path / "cancelled-capture"
            capture = await Capture.prepare(
                client,
                output=output,
                suite=suite.id,
                template=suite.templates[0].id,
                parameters=PARAMETERS,
            )
            cancelled = await capture.start()

            # Stop Once Admission Has Reached Provider Dispatch
            async for status in cancelled.updates(poll_interval=0.05):
                if status.phase not in {RunPhase.QUEUED, RunPhase.PREPARING}:
                    break
            status = await cancelled.status()
            assert status.last_execution_id is not None
            await (await client.execution(status.last_execution_id)).stop()

            await capture.collect(cancelled)
            assert (await cancelled.wait()).phase is RunPhase.STOPPED
            assert capture.result.request.phase is RunPhase.STOPPED
            assert (output / "report.html").is_file()

    asyncio.run(scenario())


# === Private Functions === #


def _event(stdout: IO[bytes], names: set[str], log: Path) -> dict[str, Any]:
    """Read Supervisor Lines Until One Of The Named Readiness Events"""
    while True:
        line = stdout.readline()
        assert line, log.read_text()
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("event") in names:
            return event


async def _pending(cycle: Any) -> list[Any]:
    # The Server Publishes No Prompt Notification; Poll Its Pending List.
    while True:
        pending = [prompt async for prompt in cycle.pending_prompts()]
        if pending:
            return pending
        await asyncio.sleep(0.05)


def _export_example(report: Path, destination: Path) -> None:
    if not report.is_file() or not report.read_text(encoding="utf-8").strip():
        raise AssertionError("the completed capture produced no sealed report")
    destination.parent.mkdir(parents=True, exist_ok=True)
    staged = destination.with_name(destination.name + ".pending")
    shutil.copyfile(report, staged)
    staged.replace(destination)


# === Entry Point === #
