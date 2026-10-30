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
from pathlib import Path
from typing import Any

# Third Party Imports
import pytest

# Local Imports
from dokime_client.client import ExecutionEvidence, ProtocolError, RpcError

from .backend import SUITE_ID, TEMPLATE, collect, prepare
from .capture_backend import serve

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_saved_execution_evidence(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            cycle = await prepare(client, parameters={"array": [1, 2], "voltage": 230})
            execution = cycle.executions[0]
            await execution.start()
            records, samples = await asyncio.gather(collect(execution.records()), collect(execution.samples()))
            outcome = execution.outcome
            assert outcome.status == "passed"
            assert [record.type for record in records] == ["fixture.finished"]
            assert len(samples) == 2
            cycle_records = await collect(cycle.records())
            assert len(cycle_records) == 2
            assert any(not record.scope.get("execution_id") for record in cycle_records)
            assert len(backend.overrides["array"].values) == 1
            evidence = ExecutionEvidence(execution._transport, outcome.checkpoint, execution._backend)
            document = await evidence.document()
            definition = document["snapshot"]["case_definition"]
            assert definition["case_id"] == "captured-case"
            assert definition["template_id"] == "captured-template"
            assert definition["generation_index"] == "9007199254740993"
            assert definition["parameters"][0]["value"]["nodes"][0]["kind"]["Number"]["field_0"] == 230
            assert document["snapshot"]["host_version"] == "captured-version"
            saved_records = await collect(evidence.records())
            saved_samples = await collect(evidence.samples())
            assert len(saved_records) == 1
            assert saved_records[0].epoch
            assert saved_records[0].record.scope["execution_id"] == execution.id
            assert len(saved_samples) == 2
            assert saved_samples[-1].sample.values["voltage"] == 232
            trace = await evidence.traceability()
            assert trace[0].entries[0].value == "frozen-serial"
            assert trace[0].entries[0].source == "device"
            (artifact,) = await evidence.artifacts()
            attachment = await evidence.download_artifact(artifact, tmp_path / "attachment")
            assert attachment.path.read_bytes() == backend.attachment
            assert not (tmp_path / "escape.txt").exists()

            # Failed Transfer Preserves Output; Retrying Only Downloads Saved Bytes
            path = tmp_path / "report.html"
            path.write_bytes(b"previous report")
            backend.bad_report = "hash"
            with pytest.raises(ProtocolError):
                await cycle.download_report(path)
            assert path.read_bytes() == b"previous report"
            backend.bad_report = ""
            saved = await cycle.download_report(path)
            assert saved.path.read_bytes() == backend.html
            assert backend.created == backend.run_calls == 1
            assert not list(tmp_path.rglob("*.part"))

    asyncio.run(scenario())


def test_download_partial(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with serve() as (_, client):
            run_cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            await run_cycle.executions[0].start()
            await run_cycle.executions[0]
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
            run_cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            await run_cycle.executions[0].start()
            await run_cycle.executions[0]
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
            run_cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            await run_cycle.executions[0].start()
            await run_cycle.executions[0]
            backend.bad_report = failure
            path = tmp_path / "report.html"
            path.write_text("previous successful report")
            with pytest.raises((ProtocolError, RpcError)):
                await run_cycle.download_report(path)
            assert path.read_text() == "previous successful report"
            assert not path.with_name(path.name + ".part").exists()

    asyncio.run(scenario())


@pytest.mark.parametrize("payload", [[], {1: "bad"}, {"bad": float("nan")}, {"bad": float("inf")}, {"nested": [float("-inf")]}])
def test_params_mutation(tmp_path: Path, payload: Any) -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            template = await client.template(TEMPLATE)
            with pytest.raises(ValueError):
                await template.materialize(parameters=payload)
            assert backend.created == backend.run_calls == 0
            assert backend.materialize_calls == 0

    asyncio.run(scenario())


def test_history_refused() -> None:
    async def scenario() -> None:
        async with serve() as (backend, client):
            run_cycle = await prepare(client, suite=SUITE_ID, template=TEMPLATE)
            await run_cycle.executions[0].start()
            await run_cycle.executions[0]
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
