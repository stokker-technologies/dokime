# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from collections.abc import Callable
import json
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime import _native

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def case_json_writers(tmp_path):
    json_path = tmp_path / "nested" / "payload.json"
    jsonl_path = tmp_path / "records.jsonl"

    _native.write_json_file(
        str(json_path),
        {"message": "Gruezi", "items": [{"value": 1}, {"value": None}]},
    )
    _native.write_json_lines(str(jsonl_path), [{"sequence": 1}, {"sequence": 2}])

    assert json.loads(json_path.read_text(encoding="utf-8")) == {
        "message": "Gruezi",
        "items": [{"value": 1}, {"value": None}],
    }
    assert [
        json.loads(line) for line in jsonl_path.read_text(encoding="utf-8").splitlines()
    ] == [
        {"sequence": 1},
        {"sequence": 2},
    ]


def case_cancelled_returning(tmp_path: Path) -> None:
    """Threaded file work must not outlive its cancelled authoring callback."""
    import asyncio
    import threading

    import pytest

    from dokime.core.authoring.artifacts import _joined

    async def run() -> None:
        loop = asyncio.get_running_loop()
        started = asyncio.Event()
        release = threading.Event()
        output = tmp_path / "joined.txt"

        def write() -> None:
            loop.call_soon_threadsafe(started.set)
            assert release.wait(timeout=5)
            output.write_text("finished")

        task = asyncio.create_task(_joined(write))
        try:
            await asyncio.wait_for(started.wait(), timeout=5)
            task.cancel()
            await asyncio.sleep(0)
            task.cancel()
            await asyncio.sleep(0)
            assert not task.done()
        finally:
            release.set()
        with pytest.raises(asyncio.CancelledError):
            await task
        assert output.read_text() == "finished"

    asyncio.run(run())


def case_authored_grpc(tmp_path: Path) -> None:
    import asyncio

    import dokime
    from dokime.client._generated.dokime.v1 import content_pb2 as content

    source = tmp_path / "source.txt"
    source.write_bytes(b"imported\n" * 10_000)

    async def run() -> None:
        class Capture(dokime.TestCaseTemplate):
            template_id = "io.capture"

            @dokime.step
            async def capture(self, ctx):
                await ctx.artifacts.write("inline.txt", b"inline\n")
                await ctx.artifacts.import_file(source, "imported.txt")
                directory = await ctx.artifacts.directory("watched")
                await asyncio.to_thread(
                    (directory / "note.txt").write_bytes, b"watched\n"
                )
                writer = await ctx.artifacts.create("abandoned.txt")
                await writer.write_all(b"incomplete")
                await writer.flush()
                await writer.abandon()

        class Lab(dokime.TestSuite):
            suite_id = "io.lab"
            members = (Capture,)

        runtime = dokime.Runtime(application_root=tmp_path / "backend")
        try:
            await runtime.register(Lab)
            client = await runtime.start()
            execution_cycle = await client.new_cycle(
                suite="io.lab", template="io.capture"
            )
            execution = await execution_cycle.run(request_key="capture")
            await execution.wait(timeout=15)
            assert (await execution_cycle.result()).status == "passed"
            inventory = await client._transport.unary(
                "Content", "ListArtifacts", content.ListArtifactsRequest()
            )
            found = {}
            for artifact in inventory.artifacts:
                assert artifact.state == content.ARTIFACT_STATE_COMPLETE
                download = await client._transport.stream(
                    "Content",
                    "DownloadArtifact",
                    content.DownloadArtifactRequest(artifact_id=artifact.artifact_id),
                )
                data = bytearray()
                async for frame in download:
                    if frame.WhichOneof("payload") == "chunk":
                        assert frame.chunk.offset == len(data)
                        data.extend(frame.chunk.data)
                found[artifact.filename] = bytes(data)
            assert found == {
                "inline.txt": b"inline\n",
                "imported.txt": source.read_bytes(),
                "note.txt": b"watched\n",
            }
        finally:
            await runtime.close()

    asyncio.run(run())


# === Private Functions === #

# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_json_writers, id="json_writers"),
        pytest.param(case_cancelled_returning, id="cancelled_returning"),
        pytest.param(case_authored_grpc, id="authored_grpc"),
    ],
)
def test_contract(scenario: Callable[..., None], tmp_path) -> None:
    scenario(tmp_path)
