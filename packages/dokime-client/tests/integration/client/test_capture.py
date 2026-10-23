# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify capture completeness, immutable provenance and streamed transfer integrity."""

# === Imports === #

# Standard Library Imports
import asyncio
import base64
import json
from pathlib import Path
from typing import Any

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client import Capture, ExecutionEvidence, ProtocolError, RpcError

from .backend import EXECUTION_ID, SUITE_ID, TEMPLATE
from .capture_backend import serve

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_capture_execution(tmp_path: Path) -> None:
    parameters = tmp_path / "parameters.json"
    parameters.write_text('{"array":[1,2],"voltage":230}', encoding="utf-8")
    root = tmp_path / "capture"

    async def scenario() -> None:
        async with serve() as (backend, client):
            capture = await Capture.prepare(
                client,
                output=root,
                suite=SUITE_ID,
                template=TEMPLATE,
                parameters=json.loads(parameters.read_text(encoding="utf-8")),
            )
            request = await capture.start()
            backend.bad_report = "hash"
            with pytest.raises(ProtocolError):
                await capture.collect(request)
            assert not (root / "report.html").exists()
            assert not json.loads((root / "capture.json").read_text())["complete"]
            assert backend.created == 1
            assert len(backend.overrides["array"].values) == 1
            backend.bad_report = ""
            recovered = await Capture.open(client, output=root)
            result = await recovered.collect(await recovered.start())
            assert result.result.status == "passed"
            assert backend.created == backend.run_calls == 1
            state = json.loads((root / "capture.json").read_text())
            assert state["complete"]
            assert base64.b64decode(state["checkpoints"][0]["version"], validate=True) == backend.point.version
            assert (root / "report.html").read_bytes() == backend.html
            execution_root = root / "executions" / EXECUTION_ID
            captured = json.loads((execution_root / "execution.json").read_text())
            definition = captured["snapshot"]["case_definition"]
            assert definition["case_id"] == "captured-case"
            assert definition["template_id"] == "captured-template"
            assert definition["generation_index"] == "9007199254740993"
            assert definition["parameters"][0]["value"]["nodes"][0]["kind"]["Number"]["field_0"] == 230
            assert captured["snapshot"]["host_version"] == "captured-version"
            records = [json.loads(row) for row in (execution_root / "records.jsonl").read_text().splitlines()]
            samples = [json.loads(row) for row in (execution_root / "streams.jsonl").read_text().splitlines()]
            assert len(records) == 1
            assert records[0]["epoch"]
            assert records[0]["record"]["scope"]["execution_id"]
            assert len(samples) == 2
            live_records = [json.loads(row) for row in (root / "records.jsonl").read_text().splitlines()]
            assert len(live_records) == 2
            assert any(not item["scope"].get("execution_id") for item in live_records)
            assert len((root / "streams.jsonl").read_text().splitlines()) == 2
            assert samples[-1]["sample"]["values"]["voltage"] == 232
            trace = json.loads((execution_root / "traceability.json").read_text())
            assert trace[0]["entries"][0]["value"] == "frozen-serial"
            assert trace[0]["entries"][0]["source"] == "device"
            assert (execution_root / "artifacts/artifact-1").read_bytes() == backend.attachment
            assert not (root / "escape.txt").exists()
            assert not list(root.rglob("*.part"))

    asyncio.run(scenario())


def test_download_partial(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with serve() as (_, client):
            run_cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            await run_cycle.run(request_key="key")
            path = tmp_path / "report.html"
            partial = tmp_path / "report.html.part"
            path.write_text("previous report")
            partial.write_text("owned partial")
            with pytest.raises(OSError):
                await run_cycle.download_report(path)
            assert path.read_text() == "previous report"
            assert partial.read_text() == "owned partial"

    asyncio.run(scenario())


def test_cancelled_download(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            run_cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            await run_cycle.run(request_key="key")
            backend.bad_report = "stall"
            path = tmp_path / "report.html"
            path.write_text("previous successful report")
            download = asyncio.create_task(run_cycle.download_report(path))
            await backend.report_stalled.wait()
            download.cancel()
            with pytest.raises(asyncio.CancelledError):
                await download
            await client.close()
            await backend.report_cancelled.wait()
            assert path.read_text() == "previous successful report"
            assert not path.with_name(path.name + ".part").exists()

    asyncio.run(scenario())


@pytest.mark.parametrize(
    "failure",
    ["identity", "identifier", "offset", "hash", "truncated", "trailing", "status"],
)
def test_download_transfer(tmp_path: Path, failure: str) -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            run_cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            await run_cycle.run(request_key="key")
            backend.bad_report = failure
            path = tmp_path / "report.html"
            path.write_text("previous successful report")
            with pytest.raises((ProtocolError, RpcError)):
                await run_cycle.download_report(path)
            assert path.read_text() == "previous successful report"
            assert not path.with_name(path.name + ".part").exists()

    asyncio.run(scenario())


@pytest.mark.parametrize("payload", [[], {"bad": float("nan")}, {"bad": float("inf")}])
def test_params_mutation(tmp_path: Path, payload: Any) -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            output = tmp_path / "capture"
            with pytest.raises(ValueError):
                await Capture.prepare(
                    client,
                    output=output,
                    suite=SUITE_ID,
                    template=TEMPLATE,
                    parameters=payload,
                )
            assert backend.created == backend.run_calls == 0
            assert not output.exists()

    asyncio.run(scenario())


def test_history_refused() -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            run_cycle = await client.new_cycle(suite=SUITE_ID, template=TEMPLATE)
            run = await run_cycle.run(request_key="key")
            await run.wait()
            (handle,) = run_cycle.executions
            point = await handle.checkpoint()
            assert point is not None
            evidence = ExecutionEvidence(handle._transport, point, handle._backend)
            backend.unsealed = True
            with pytest.raises(ProtocolError, match="differs from the persisted seal"):
                await evidence.document()
            backend.history_issue = True
            with pytest.raises(ProtocolError, match="incomplete"):
                _ = [record async for record in evidence.records()]

    asyncio.run(scenario())


# === Private Functions === #

# === Entry Point === #
