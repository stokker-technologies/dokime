# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Opt-in interoperability test against the actual Dokime CLI and Mini charger sidecar."""

# === Imports === #

# Standard Library Imports
import asyncio
import json
import os
import shutil
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime.client import Capture, DokimeClient, ExecutionEvidence, RunPhase

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# The completed capture's sealed report ships as the download's example when a
# destination is declared; a declared destination without a completed capture
# fails the lane rather than omitting the example.
EXAMPLE_VARIABLE = "DOKIME_EXAMPLE_REPORT"

# === Types === #

# === Classes === #

# === Public Functions === #


def test_native_stop(tmp_path: Path) -> None:
    async def scenario() -> None:
        root = Path(__file__).resolve().parents[5]
        executable = Path(os.environ["DOKIME_TEST_HOST"])
        bundle = Path(
            os.environ.get(
                "DOKIME_CLIENT_NATIVE_BUNDLE",
                root / "target/fixtures/mini-sidecar/mini-sidecar.dokime",
            )
        ).resolve()
        assert executable.is_file() and bundle.is_file(), (
            "run mise run build:release and prepare test fixtures first"
        )
        env_file = tmp_path / "empty.env"
        env_file.touch()
        stderr = (tmp_path / "backend.log").open("w")
        process = await asyncio.create_subprocess_exec(
            str(executable),
            "--port",
            "0",
            "--dev-control",
            "--shutdown-stdin",
            "--application-root",
            str(tmp_path / "backend"),
            "--env-file",
            str(env_file),
            "--plugin",
            str(bundle),
            "--no-seed-validation-artifact",
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=stderr,
        )
        client = None
        try:
            assert process.stdout is not None
            async with asyncio.timeout(45):
                while True:
                    line = await process.stdout.readline()
                    assert line, (tmp_path / "backend.log").read_text()
                    try:
                        ready = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    if ready.get("event") == "ready":
                        break
            # Dev-control starts with no plugins; explicitly load the selected
            # bundle through the supervisor protocol before exercising gRPC.
            assert process.stdin is not None
            process.stdin.write(
                (json.dumps({"command": "reload", "path": str(bundle)}) + "\n").encode()
            )
            await process.stdin.drain()
            async with asyncio.timeout(45):
                while True:
                    line = await process.stdout.readline()
                    assert line, (tmp_path / "backend.log").read_text()
                    try:
                        loaded = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    if loaded.get("event") in {"reloaded", "reload_failed"}:
                        assert loaded["event"] == "reloaded", loaded
                        break
            client = DokimeClient(ready["endpoint"])
            suite = await client.suite("mini.charger.steady")
            assert suite.templates
            parameters = {
                "ac_voltage_v": 230.0,
                "load_ohm": 100.0,
                "frequency_hz": 1.0,
                "duration_s": 0.3,
                "sample_interval_s": 0.1,
            }
            output = tmp_path / "capture"
            capture = await Capture.prepare(
                client,
                output=output,
                suite=suite.id,
                template=suite.templates[0].id,
                parameters=parameters,
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

            async with asyncio.timeout(60):
                async with asyncio.TaskGroup() as tasks:
                    tasks.create_task(events())
                    tasks.create_task(telemetry())
                    await live_sample.wait()
                    assert not (await run.status()).terminal
                    while True:
                        pending = [p async for p in cycle.pending_prompts()]
                        if pending:
                            break
                        await asyncio.sleep(0.05)
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
                filtered = [
                    e async for e in handle.records(record_ids=[records[-1].id])
                ]
                assert len(filtered) == 1 and filtered[0].id == records[-1].id
                replay = [s async for s in handle.samples(page_size=2)]
                assert {s.id for s in samples} == {s.id for s in replay}

                point = await handle.checkpoint()
                assert point is not None
                captured = ExecutionEvidence(handle._transport, point, handle._backend)
                document = await captured.document()
                assert document["execution_id"] == captured.checkpoint.execution_id
                assert document["snapshot"]["parameters"]
                assert await captured.traceability()
                historical_records = [item async for item in captured.records()]
                historical_samples = [item async for item in captured.samples()]
                own_records = {
                    item.id
                    for item in records
                    if item.scope.get("execution_id")
                    == captured.checkpoint.execution_id
                }
                historical_own = {
                    item.record.id
                    for item in historical_records
                    if item.record.scope.get("execution_id")
                    == captured.checkpoint.execution_id
                }
                assert historical_own == own_records
                assert {item.sample.id for item in historical_samples} == {
                    item.id for item in samples
                }
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

                stopped_output = tmp_path / "cancelled-capture"
                stopped_capture = await Capture.prepare(
                    client,
                    output=stopped_output,
                    suite=suite.id,
                    template=suite.templates[0].id,
                    parameters=parameters,
                )
                cancelled = await stopped_capture.start()
                # Allow admission to reach provider dispatch before requesting stop.
                while (await cancelled.status()).phase in {
                    RunPhase.QUEUED,
                    RunPhase.PREPARING,
                }:
                    await asyncio.sleep(0.05)
                status = await cancelled.status()
                assert status.last_execution_id is not None
                await (await client.execution(status.last_execution_id)).stop()
                await stopped_capture.collect(cancelled)
                assert (await cancelled.wait()).phase is RunPhase.STOPPED
                assert stopped_capture.result.request.phase is RunPhase.STOPPED
                assert (stopped_output / "report.html").is_file()
                await client.close()
                assert process.returncode is None, (
                    "client close must not stop the owned backend"
                )
                process.stdin.close()
                await asyncio.wait_for(process.wait(), 15)
                assert process.returncode == 0, (tmp_path / "backend.log").read_text()
        finally:
            if client is not None:
                await client.close()
            if process.stdin is not None:
                process.stdin.close()
            try:
                await asyncio.wait_for(process.wait(), 15)
            except TimeoutError:
                process.kill()
                await process.wait()
            stderr.close()

    asyncio.run(scenario())
    _export_example(tmp_path / "capture/report.html")


# === Private Functions === #


def _export_example(report: Path) -> None:
    declared = os.environ.get(EXAMPLE_VARIABLE)
    if not declared:
        return
    destination = Path(declared)
    if not destination.is_absolute():
        raise AssertionError(f"{EXAMPLE_VARIABLE} must be an absolute path")
    if not report.is_file() or not report.read_text(encoding="utf-8").strip():
        raise AssertionError(
            f"{EXAMPLE_VARIABLE} is declared but the capture produced no sealed report"
        )
    destination.parent.mkdir(parents=True, exist_ok=True)
    staged = destination.with_name(destination.name + ".pending")
    shutil.copyfile(report, staged)
    staged.replace(destination)


# === Entry Point === #
