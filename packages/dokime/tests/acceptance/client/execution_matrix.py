# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Pure Python Client For Shared Rust Backend Execution Scenarios"""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
import json
import sys
from contextlib import aclosing
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime.client import DokimeClient, _wire

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def drive(endpoint: str, contract: dict, scenario: dict) -> None:
    client = DokimeClient(endpoint)
    try:
        run_cycle = await client.new_cycle(suite=contract["suite"], template=contract["template"])
        (control,) = run_cycle.executions
        run = await control.start(request_key=scenario["name"])
        await run.status()
        while not (pending := [prompt async for prompt in run_cycle.pending_prompts()]):
            assert not (await run.status()).terminal
            await asyncio.sleep(0.01)
        assert len(pending) == 1
        execution_id = (await run.status()).last_execution_id

        async def progress_at(completed: int) -> None:
            progress_stream = await client._transport.stream(
                "Observation",
                "WatchExecutionProgress",
                _wire.request(
                    "Observation",
                    "WatchExecutionProgress",
                    test_execution_id=execution_id,
                ),
            )
            try:
                async for frame in progress_stream:
                    if frame.HasField("progress"):
                        assert frame.progress.phase == "Measuring"
                        assert frame.progress.total == 2
                        assert frame.progress.completed == completed
                        return
                raise AssertionError("progress ended before its snapshot")
            finally:
                await progress_stream.close()

        await progress_at(0)
        # Prompt admission does not certify that independent evidence readers
        # have reached the publisher. Follow each frontier to its first item.
        async with aclosing(run_cycle.records(record_types=[contract["record"]], until=run)) as events:
            await anext(events)
        async with aclosing(run_cycle.samples(streams=[contract["stream"]], page_size=1, until=run)) as samples:
            await anext(samples)

        live_records = [item async for item in run_cycle.records()]
        live_rows = [item async for item in run_cycle.samples(streams=[contract["stream"]], page_size=1)]
        assert [row.values["value"] for row in live_rows] == [1]
        assert not (await run.status()).terminal
        ready = next(item for item in live_records if item.type == contract["record"])
        assert ready.details == {"phase": "ready"}
        selected = [item async for item in run_cycle.records(record_ids=[ready.id])]
        assert [item.id for item in selected] == [ready.id]

        if scenario["action"] == "stop":
            progress = await run.status()
            assert progress.last_execution_id is not None
            control = await client.execution(progress.last_execution_id)
            await control.stop()
        else:
            await run_cycle.respond(pending[0].id, action=scenario["action"])
        if scenario["action"] == "yes":
            while not (next_prompts := [item async for item in run_cycle.pending_prompts() if item.id != pending[0].id]):
                assert not (await run.status()).terminal
                await asyncio.sleep(0.01)
            await progress_at(1)
            await run_cycle.respond(next_prompts[0].id, action="yes")
        status = await run.wait(poll_interval=0.02)
        records = [item async for item in run_cycle.records()]
        rows = [item async for item in run_cycle.samples(streams=[contract["stream"]], page_size=1)]
        summary = await run_cycle.result()
        cycle_document = await run_cycle.document()
        inventory = await client._transport.unary("Content", "ListArtifacts", _wire.request("Content", "ListArtifacts"))
        artifact = next(item for item in inventory.artifacts if item.filename.endswith("matrix.txt"))
        assert artifact.scope.execution_id == status.last_execution_id
        assert _wire.enum_name(artifact.state) == "complete"
        download = await client._transport.stream(
            "Content",
            "DownloadArtifact",
            _wire.request("Content", "DownloadArtifact", artifact_id=artifact.artifact_id),
        )
        data = bytearray()
        completed = False
        async for frame in download:
            kind = _wire.variant(frame.payload)
            payload = getattr(frame.payload, kind).field_0
            if kind == "Chunk":
                assert payload.offset == len(data)
                data.extend(payload.data)
            elif kind == "Complete":
                assert payload.size_bytes == len(data)
                assert payload.sha256 == hashlib.sha256(data).hexdigest()
                completed = True
        assert completed
        assert data == b"open\ncleanup\n"
        authored = [item for item in records if item.type == contract["record"]]
        execution = status.last_execution_id
        assert execution
        assert len({item.id for item in records}) == len(records)
        assert len({row.id for row in rows}) == len(rows)
        assert [item.sequence for item in records] == sorted(item.sequence for item in records)
        for item in [*authored, *rows]:
            assert item.scope["cycle_id"] == run.cycle_id
            assert item.scope["execution_id"] == execution
            assert item.timestamp is not None
        assert {row.id for row in live_rows} <= {row.id for row in rows}
        print(
            json.dumps(
                {
                    "status": summary.status,
                    "phase": status.phase.value,
                    "values": [row.values["value"] for row in rows],
                    "records": [item.details["phase"] for item in authored],
                    "cases": int(cycle_document.get("case_count", "0")),
                    "executions": int(cycle_document.get("execution_count", "0")),
                    "filtered": len(selected),
                    "live": len(live_rows),
                    "artifact": data.decode(),
                    "progress": {
                        "completed": 1 if scenario["action"] == "yes" else 0,
                        "total": 2,
                    },
                }
            ),
            flush=True,
        )
    finally:
        await client.close()


# === Private Functions === #

# === Entry Point === #
if __name__ == "__main__":
    contract = json.loads(Path(sys.argv[2]).read_text())
    scenario = next(item for item in contract["scenarios"] if item["name"] == sys.argv[3])
    asyncio.run(drive(sys.argv[1], contract, scenario))
