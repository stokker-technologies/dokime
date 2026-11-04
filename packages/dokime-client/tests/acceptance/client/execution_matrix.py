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
from dokime_client.client import DokimeClient, Execution, _wire

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def drive(endpoint: str, contract: dict, scenario: dict) -> None:
    client = DokimeClient(endpoint)
    changes = None
    try:
        changes = await client._transport.stream("Observation", "WatchRuntime", _wire.request("Observation", "WatchRuntime"))
        template = await client.template(contract["template"])
        (case,) = await template.materialize()
        control = Execution(case)
        run_cycle = await client.create_cycle(label="Execution Matrix", continue_automatically=False)
        await run_cycle.append([control])
        await control.start()
        execution_id = control.id
        while not (pending := [prompt async for prompt in run_cycle.pending_prompts()]):
            assert await control.checkpoint() is None
            await anext(changes)
        assert len(pending) == 1

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
        async with aclosing(control.records(contract["record"])) as events:
            await anext(events)
        async with aclosing(control.samples(contract["stream"], page_size=1)) as samples:
            await anext(samples)

        live_records = [item async for item in run_cycle.records()]
        live_rows = [item async for item in run_cycle.samples(streams=[contract["stream"]], page_size=1)]
        assert [row.values["value"] for row in live_rows] == [1]
        assert await control.checkpoint() is None
        ready = next(item for item in live_records if item.type == contract["record"])
        assert ready.details == {"phase": "ready"}
        selected = [item async for item in run_cycle.records(record_ids=[ready.id])]
        assert [item.id for item in selected] == [ready.id]

        if scenario["action"] == "stop":
            await control.stop()
        else:
            await run_cycle.respond(pending[0].id, action=scenario["action"])
        if scenario["action"] == "yes":
            while not (next_prompts := [item async for item in run_cycle.pending_prompts() if item.id != pending[0].id]):
                assert await control.checkpoint() is None
                await anext(changes)
            await progress_at(1)
            await run_cycle.respond(next_prompts[0].id, action="yes")
        while await control.checkpoint() is None:
            await anext(changes)
        status = await control.wait()
        records = [item async for item in run_cycle.records()]
        rows = [item async for item in run_cycle.samples(streams=[contract["stream"]], page_size=1)]
        await run_cycle.seal()
        summary = await run_cycle.result()
        cycle_document = await run_cycle.document()
        inventory = await client._transport.unary("Content", "ListArtifacts", _wire.request("Content", "ListArtifacts"))
        artifact = next(
            item for item in inventory.artifacts if item.filename.endswith("matrix.txt") and item.scope.execution_id == control.id
        )
        assert artifact.scope.execution_id == control.id
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
        execution = control.id
        assert execution
        assert len({item.id for item in records}) == len(records)
        assert len({row.id for row in rows}) == len(rows)
        assert [item.sequence for item in records] == sorted(item.sequence for item in records)
        for item in [*authored, *rows]:
            assert item.scope["cycle_id"] == run_cycle.id
            assert item.scope["execution_id"] == execution
            assert item.timestamp is not None
        assert {row.id for row in live_rows} <= {row.id for row in rows}
        print(
            json.dumps(
                {
                    "status": summary.status,
                    "phase": {"passed": "completed", "failed": "failed", "cancelled": "stopped"}[status.status],
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
        if changes is not None:
            await changes.close()
        await client.close()


# === Private Functions === #

# === Entry Point === #
if __name__ == "__main__":
    contract = json.loads(Path(sys.argv[2]).read_text())
    scenario = next(item for item in contract["scenarios"] if item["name"] == sys.argv[3])
    asyncio.run(drive(sys.argv[1], contract, scenario))
