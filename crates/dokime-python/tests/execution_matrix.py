# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Python fixture and client for shared execution scenarios"""

# === Imports === #

# Standard Library Imports
import asyncio
from contextlib import aclosing, asynccontextmanager
import json
import hashlib
import os
from pathlib import Path
import sys

# Third Party Imports

# Local Imports
from dokime.client import DokimeClient
from dokime.client._generated.dokime.v1 import (
    content_pb2 as content,
    observation_pb2 as observation,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


async def serve(root: Path) -> None:
    import dokime

    def trace(value: str) -> None:
        with (root / "lifecycle.txt").open("a") as output:
            output.write(value + "\n")

    @dokime.record(id="matrix.phase")
    class Phase:
        phase: str

    @dokime.stream(id="matrix.reading")
    class Reading:
        value: int

    @dokime.prompt(id="matrix.continue", label="Continue?")
    class Continue(dokime.Confirmation):
        pass

    class Probe(dokime.TestCaseTemplate):
        template_id = "matrix.probe"
        records = (Phase,)
        streams = (Reading,)
        prompts = (Continue,)

        @dokime.initialize
        async def initialize_handles(self, ctx):
            return await ctx.acquire(self._managed_resources(ctx))

        @asynccontextmanager
        async def _managed_resources(self, ctx):
            trace("open")
            writer = await ctx.artifacts.create("matrix.txt")
            await writer.write_all(b"open\n")
            try:
                yield writer
            finally:
                await writer.finish()
                trace("close")

        @dokime.step
        async def measure(self, ctx):
            async for value in ctx.progress("Measuring", range(1, 3)):
                await ctx.publish(Reading(value))
                if value == 1:
                    await ctx.publish(Phase("ready"))
                if not await ctx.ask(Continue()):
                    raise ValueError("operator refused")
            await ctx.publish(Phase("done"))

        @dokime.cleanup
        async def finish(self, ctx):
            await ctx.resources.write_all(b"cleanup\n")
            trace("cleanup")

        @staticmethod
        @dokime.criterion
        async def accepted(ctx):
            return len(await ctx.samples(Reading)) == 2

    class Lab(dokime.TestSuite):
        suite_id = "matrix.lab"
        members = (Probe,)

    runtime = dokime.Runtime(application_root=root / "backend")
    try:
        await runtime.register(Lab)
        client = await runtime.start()
        # Test supervisor discovery; production callers retain the returned client.
        print(json.dumps({"endpoint": client._transport._target.endpoint}), flush=True)
        await asyncio.to_thread(sys.stdin.readline)
    finally:
        await runtime.close()


async def drive(endpoint: str, contract: dict, scenario: dict) -> None:
    client = DokimeClient(endpoint)
    try:
        run_cycle = await client.new_cycle(
            suite=contract["suite"], template=contract["template"]
        )
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
                observation.WatchExecutionProgressRequest(
                    test_execution_id=execution_id
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
        # have reached the publisher. Follow each frontier with a fixed budget.
        async with asyncio.timeout(25 if os.environ.get("COVERAGE") == "1" else 5):
            async with aclosing(
                run_cycle.records(record_types=[contract["record"]], until=run)
            ) as events:
                await anext(events)
            async with aclosing(
                run_cycle.samples(streams=[contract["stream"]], page_size=1, until=run)
            ) as samples:
                await anext(samples)

        live_records = [item async for item in run_cycle.records()]
        live_rows = [
            item
            async for item in run_cycle.samples(
                streams=[contract["stream"]], page_size=1
            )
        ]
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
            while not (
                next_prompts := [
                    item
                    async for item in run_cycle.pending_prompts()
                    if item.id != pending[0].id
                ]
            ):
                assert not (await run.status()).terminal
                await asyncio.sleep(0.01)
            await progress_at(1)
            await run_cycle.respond(next_prompts[0].id, action="yes")
        status = await run.wait(timeout=20, poll_interval=0.02)
        records = [item async for item in run_cycle.records()]
        rows = [
            item
            async for item in run_cycle.samples(
                streams=[contract["stream"]], page_size=1
            )
        ]
        summary = await run_cycle.result()
        cycle_document = await run_cycle.document()
        inventory = await client._transport.unary(
            "Content", "ListArtifacts", content.ListArtifactsRequest()
        )
        artifact = next(
            item for item in inventory.artifacts if item.filename.endswith("matrix.txt")
        )
        assert artifact.scope.execution_id == status.last_execution_id
        assert artifact.state == content.ARTIFACT_STATE_COMPLETE
        download = await client._transport.stream(
            "Content",
            "DownloadArtifact",
            content.DownloadArtifactRequest(artifact_id=artifact.artifact_id),
        )
        data = bytearray()
        completed = False
        async for frame in download:
            if frame.WhichOneof("payload") == "chunk":
                assert frame.chunk.offset == len(data)
                data.extend(frame.chunk.data)
            elif frame.WhichOneof("payload") == "complete":
                assert frame.complete.size_bytes == len(data)
                assert frame.complete.sha256 == hashlib.sha256(data).hexdigest()
                completed = True
        assert completed
        assert data == b"open\ncleanup\n"
        authored = [item for item in records if item.type == contract["record"]]
        execution = status.last_execution_id
        assert execution
        assert len({item.id for item in records}) == len(records)
        assert len({row.id for row in rows}) == len(rows)
        assert [item.sequence for item in records] == sorted(
            item.sequence for item in records
        )
        for item in [*authored, *rows]:
            assert item.scope["cycle_id"] == run.cycle_id
            assert item.scope["execution_id"] == execution
            assert item.timestamp is not None
        assert set(row.id for row in live_rows) <= set(row.id for row in rows)
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
    if sys.argv[1] == "serve":
        asyncio.run(serve(Path(sys.argv[2])))
    else:
        contract = json.loads(Path(sys.argv[3]).read_text())
        scenario = next(
            item for item in contract["scenarios"] if item["name"] == sys.argv[4]
        )
        asyncio.run(
            asyncio.wait_for(drive(sys.argv[2], contract, scenario), timeout=35)
        )
