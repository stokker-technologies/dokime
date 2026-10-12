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
from collections.abc import Callable
import base64
import asyncio
from contextlib import asynccontextmanager
import hashlib
import json
from pathlib import Path
from typing import Any

# Third Party Imports
import grpc
import pytest

# Local Imports
from dokime.client import cli, commands
from dokime.client import DokimeClient, ExecutionEvidence, ProtocolError, RpcError
from dokime.client._generated.dokime.v1 import backend_pb2_grpc, catalog_pb2_grpc
from dokime.client._generated.dokime.v1 import common_pb2 as common
from dokime.client._generated.dokime.v1 import content_pb2 as content
from dokime.client._generated.dokime.v1 import execution_pb2 as execution
from dokime.client._generated.dokime.v1 import execution_pb2_grpc, evidence_pb2_grpc
from dokime.client._generated.dokime.v1 import history_pb2 as history
from dokime.client._generated.dokime.v1 import history_pb2_grpc
from dokime.client._generated.dokime.v1 import planning_pb2_grpc
from dokime.client._generated.dokime.v1 import prompts_pb2 as prompts
from dokime.client._generated.dokime.v1 import prompts_pb2_grpc
from dokime.client._generated.dokime.v1 import reports_pb2 as reports
from dokime.client._generated.dokime.v1 import reports_pb2_grpc, telemetry_pb2_grpc
from dokime.client._generated.dokime.v1 import value_pb2 as value
from .test_client import BackendFixture, CASE_ID, CYCLE_ID, EXECUTION_ID

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Native V1 Golden Cursors Verified By The Rust Saved-Evidence Unit Tests.
RECORD_CURSORS = (
    "AS5it6Cxuin6UNn09dqykr_kwsHzfyQLo2_cd4OgzaFEAAAAAAAAAAE",
    "AS5it6Cxuin6UNn09dqykr_kwsHzfyQLo2_cd4OgzaFEAAAAAAAAAAI",
)
SAMPLE_CURSORS = (
    "AS-NHy5qsY3VX7wdUoUc1hTUnmoBSgaqNQrN5IyZu8n1AAAAAAAAAAE",
    "AS-NHy5qsY3VX7wdUoUc1hTUnmoBSgaqNQrN5IyZu8n1AAAAAAAAAAI",
)


# === Types === #

# === Classes === #


class CaptureBackend(
    BackendFixture,
    history_pb2_grpc.HistoryServicer,
    reports_pb2_grpc.ReportsServicer,
    prompts_pb2_grpc.PromptsServicer,
):
    def __init__(self) -> None:
        super().__init__()
        self.methods["History"] = [
            "BrowseExecutions",
            "GetExecutionCheckpoint",
            "GetCheckpointDocument",
            "GetCheckpointContext",
            "GetCheckpointEvidence",
            "ListCheckpointRecords",
            "ListCheckpointTelemetry",
            "DownloadCheckpointArtifact",
        ]
        self.methods["Reports"] = ["DownloadCycleReport"]
        self.methods["Prompts"] = ["ListPending"]
        self.bad_report = ""
        self.report_stalled = asyncio.Event()
        self.report_cancelled = asyncio.Event()
        self.history_issue = False
        self.unsealed = False
        self.html = b"<!doctype html><html>Captured Report</html>"
        self.attachment = b"instrument evidence"

    def publish(self, kind: str) -> None:
        super().publish(kind)
        for item in (self.records[-1], self.samples[-1]):
            item.scope.cycle_id = CYCLE_ID
            item.scope.execution_id = EXECUTION_ID
        # A cycle-only lifecycle record is valid live evidence, but does not
        # belong to this execution's historical checkpoint.
        if len(self.records) == 1:
            self.records[-1].scope.ClearField("execution_id")

    async def ListPending(self, request: Any, context: Any) -> Any:
        assert request.cycle_id == CYCLE_ID
        # Native Selection Golden Verified By The Portable Prompt Tests.
        return prompts.ListPendingPromptsReply(
            cursor=common.Cursor(
                epoch=self.epoch,
                stream="prompts.pending:c4e98c8f616061bebf0d5119b0b549f764f8207d9a4660c207dcff2523da467b",
                position=1,
            )
        )

    async def ListExecutions(self, request: Any, context: Any) -> Any:
        return execution.ListExecutionsReply(
            executions=[
                execution.ExecutionSummary(
                    execution_id=EXECUTION_ID, case_id=CASE_ID, ordinal=0
                )
            ],
            cursor=common.Cursor(epoch=self.epoch),
        )

    async def BrowseExecutions(self, request: Any, context: Any) -> Any:
        return history.ExecutionBrowsePage(
            entries=[
                history.ExecutionBrowseEntry(
                    metadata=history.HistoricalExecutionMetadata(
                        execution_id=EXECUTION_ID, cycle_id=CYCLE_ID
                    ),
                    checkpoint=self.point,
                )
            ],
            discovery_complete=True,
        )

    async def GetCheckpointDocument(self, request: Any, context: Any) -> Any:
        assert request.checkpoint == self.point
        return history.CheckpointDocument(
            checkpoint=self.point,
            acquisition_finished=not self.unsealed,
            document=history.HistoricalExecutionDocument(
                execution=history.HistoricalTestExecution(
                    execution_id=EXECUTION_ID,
                    status=self.seal_status,
                    result=execution.ResultSummary(
                        status=execution.RESULT_STATUS_PASSED, passed=1
                    ),
                    snapshot=history.HistoricalExecutionSnapshot(
                        template=common.Metadata(identifier="captured.template"),
                        case_definition=history.HistoricalCaseDefinition(
                            case_id="captured-case",
                            template_id="captured-template",
                            metadata=common.Metadata(label="Charger At 230 V"),
                            suite=common.Metadata(identifier="charger.bench"),
                            parameters=[
                                execution.CaseParameter(
                                    name="voltage",
                                    value=value.Value(number_value=230),
                                )
                            ],
                            generation_index=9_007_199_254_740_993,
                        ),
                        host_version="captured-version",
                    ),
                )
            ),
        )

    async def GetCheckpointContext(self, request: Any, context: Any) -> Any:
        assert request.checkpoint == self.point
        return history.CheckpointContext(
            checkpoint=self.point,
            traceability=[
                common.TraceabilitySnapshot(
                    traceability_id="instrument",
                    entries=[
                        common.TraceabilityEntry(
                            metadata=common.Metadata(identifier="serial"),
                            value=value.Value(text_value="frozen-serial"),
                            source="device",
                        ),
                    ],
                ),
            ],
        )

    async def GetCheckpointEvidence(self, request: Any, context: Any) -> Any:
        return history.CheckpointEvidence(
            checkpoint=self.point,
            telemetry_tables=[history.CheckpointTelemetryTable(table="voltage")],
            artifacts=[
                content.ArtifactMetadata(
                    artifact_id="artifact-1",
                    filename="../../escape.txt",
                    size_bytes=len(self.attachment),
                    sha256=hashlib.sha256(self.attachment).hexdigest(),
                    state=content.ARTIFACT_STATE_COMPLETE,
                )
            ],
        )

    async def ListCheckpointRecords(self, request: Any, context: Any) -> Any:
        start = RECORD_CURSORS.index(request.after) + 1 if request.after else 0
        records = [item for item in self.records if item.scope.execution_id]
        return history.CheckpointRecordPage(
            checkpoint=self.point,
            records=[
                history.SavedRecord(epoch=self.epoch, record=item)
                for item in records[start : start + 1]
            ],
            next_after=RECORD_CURSORS[start] if start + 1 < len(records) else None,
            issues=[history.HistoryIssue(message="missing data")]
            if self.history_issue
            else [],
        )

    async def ListCheckpointTelemetry(self, request: Any, context: Any) -> Any:
        start = SAMPLE_CURSORS.index(request.after) + 1 if request.after else 0
        rows = [
            history.SavedTelemetrySample(
                epoch=self.epoch,
                ordinal=item.sequence,
                sample=item,
            )
            for item in self.samples[start : start + 1]
        ]
        return history.CheckpointTelemetryPage(
            checkpoint=self.point,
            table=request.table,
            rows=rows,
            next_after=SAMPLE_CURSORS[start] if start + 1 < len(self.samples) else None,
        )

    async def DownloadCheckpointArtifact(self, request: Any, context: Any) -> Any:
        digest = hashlib.sha256(self.attachment).hexdigest()
        yield content.ArtifactFrame(
            metadata=content.ArtifactMetadata(
                artifact_id="artifact-1",
                size_bytes=len(self.attachment),
                sha256=digest,
            )
        )
        yield content.ArtifactFrame(chunk=content.ArtifactChunk(data=self.attachment))
        yield content.ArtifactFrame(
            complete=content.ArtifactCompletion(
                size_bytes=len(self.attachment), sha256=digest
            )
        )

    async def DownloadCycleReport(self, request: Any, context: Any) -> Any:
        yield reports.ReportHtmlFrame(
            report=reports.ReportMetadata(
                test_cycle_id="wrong" if self.bad_report == "identity" else CYCLE_ID,
            )
        )
        yield reports.ReportHtmlFrame(
            chunk=reports.ReportHtmlChunk(
                offset=1 if self.bad_report == "offset" else 0,
                data=self.html,
            )
        )
        if self.bad_report == "stall":
            self.report_stalled.set()
            try:
                await asyncio.Event().wait()
            finally:
                self.report_cancelled.set()
        if self.bad_report == "truncated":
            return
        yield reports.ReportHtmlFrame(
            complete=reports.ReportHtmlCompletion(
                size_bytes=len(self.html),
                sha256="bad"
                if self.bad_report == "hash"
                else hashlib.sha256(self.html).hexdigest(),
            )
        )
        if self.bad_report == "trailing":
            yield reports.ReportHtmlFrame(chunk=reports.ReportHtmlChunk(data=b"extra"))
        if self.bad_report == "status":
            await context.abort(
                grpc.StatusCode.DATA_LOSS, "failed after completion frame"
            )


# === Public Functions === #


def test_capture_execution(tmp_path: Path, capsys: Any) -> None:
    parameters = tmp_path / "parameters.json"
    parameters.write_text('{"array":[1,2],"voltage":230}', encoding="utf-8")
    root = tmp_path / "capture"

    async def scenario() -> None:
        async with fixture() as (backend, client):
            args = cli.arguments(
                [
                    "run",
                    "--suite",
                    "suite",
                    "--template",
                    "template",
                    "--parameters-file",
                    str(parameters),
                    "--output",
                    str(root),
                ]
            )
            backend.bad_report = "hash"
            with pytest.RaisesGroup(ProtocolError):
                await commands.dispatch(client, args)
            assert not (root / "report.html").exists()
            assert not json.loads((root / "capture.json").read_text())["complete"]
            assert backend.created == 1
            assert len(backend.overrides["array"].values) == 1
            backend.bad_report = ""
            collect = cli.arguments(["collect", "--output", str(root)])
            assert await commands.dispatch(client, collect) == 0
            assert backend.created == backend.run_calls == 1
            state = json.loads((root / "capture.json").read_text())
            assert state["complete"]
            assert (
                base64.b64decode(state["checkpoints"][0]["version"], validate=True)
                == backend.point.version
            )
            assert (root / "report.html").read_bytes() == backend.html
            execution_root = root / "executions" / EXECUTION_ID
            captured = json.loads((execution_root / "execution.json").read_text())
            definition = captured["snapshot"]["case_definition"]
            assert definition["case_id"] == "captured-case"
            assert definition["template_id"] == "captured-template"
            assert definition["generation_index"] == "9007199254740993"
            assert definition["parameters"][0]["value"]["number_value"] == 230
            assert captured["snapshot"]["host_version"] == "captured-version"
            records = [
                json.loads(row)
                for row in (execution_root / "records.jsonl").read_text().splitlines()
            ]
            samples = [
                json.loads(row)
                for row in (execution_root / "streams.jsonl").read_text().splitlines()
            ]
            assert len(records) == 1
            assert records[0]["epoch"]
            assert records[0]["record"]["scope"]["execution_id"]
            assert len(samples) == 2
            live_records = [
                json.loads(row)
                for row in (root / "records.jsonl").read_text().splitlines()
            ]
            assert len(live_records) == 2
            assert any(not item["scope"].get("execution_id") for item in live_records)
            assert len((root / "streams.jsonl").read_text().splitlines()) == 2
            assert samples[-1]["sample"]["values"]["voltage"] == 232
            trace = json.loads((execution_root / "traceability.json").read_text())
            assert trace[0]["entries"][0]["value"] == "frozen-serial"
            assert trace[0]["entries"][0]["source"] == "device"
            assert (
                execution_root / "artifacts/artifact-1"
            ).read_bytes() == backend.attachment
            assert not (root / "escape.txt").exists()
            assert not list(root.rglob("*.part"))

    asyncio.run(scenario())
    output = capsys.readouterr()
    assert "Capturing Data..." in output.err
    assert "Sample:" not in output.out + output.err
    assert "Event:" not in output.out + output.err


def case_download_partial(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with fixture() as (_, client):
            run_cycle = await client.new_cycle(suite="suite", template="template")
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


def case_cancelled_reader(
    tmp_path: Path,
) -> None:
    async def scenario() -> None:
        async with fixture() as (backend, client):
            run_cycle = await client.new_cycle(suite="suite", template="template")
            await run_cycle.run(request_key="key")
            backend.bad_report = "stall"
            path = tmp_path / "report.html"
            path.write_text("previous successful report")
            download = asyncio.create_task(run_cycle.download_report(path))
            await asyncio.wait_for(backend.report_stalled.wait(), 1)
            download.cancel()
            with pytest.raises(asyncio.CancelledError):
                await download
            await client.close()
            await asyncio.wait_for(backend.report_cancelled.wait(), 1)
            assert path.read_text() == "previous successful report"

    asyncio.run(scenario())


@pytest.mark.parametrize(
    "failure", ["identity", "offset", "hash", "truncated", "trailing", "status"]
)
def test_download_transfer(tmp_path: Path, failure: str) -> None:
    async def scenario() -> None:
        async with fixture() as (backend, client):
            run_cycle = await client.new_cycle(suite="suite", template="template")
            await run_cycle.run(request_key="key")
            backend.bad_report = failure
            path = tmp_path / "report.html"
            path.write_text("previous successful report")
            with pytest.raises((ProtocolError, RpcError)):
                await run_cycle.download_report(path)
            assert path.read_text() == "previous successful report"

    asyncio.run(scenario())


@pytest.mark.parametrize("payload", ["[]", "null", '{"bad":NaN}', '{"bad":1e999}'])
def test_params_mutation(tmp_path: Path, payload: str) -> None:
    async def scenario() -> None:
        async with fixture() as (backend, client):
            args = cli.arguments(
                [
                    "run",
                    "--suite",
                    "suite",
                    "--template",
                    "template",
                    "--parameters",
                    payload,
                    "--output",
                    str(tmp_path / "capture"),
                ]
            )
            with pytest.raises(ValueError):
                await commands.dispatch(client, args)
            assert backend.created == backend.run_calls == 0
            assert not args.output.exists()

    asyncio.run(scenario())


def case_history_fail(tmp_path: Path) -> None:
    async def scenario() -> None:
        async with fixture() as (backend, client):
            run_cycle = await client.new_cycle(suite="suite", template="template")
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


@asynccontextmanager
async def fixture():
    implementation = CaptureBackend()
    server = grpc.aio.server()
    for module, name in (
        (backend_pb2_grpc, "Backend"),
        (catalog_pb2_grpc, "Catalog"),
        (planning_pb2_grpc, "Planning"),
        (execution_pb2_grpc, "Execution"),
        (evidence_pb2_grpc, "Evidence"),
        (telemetry_pb2_grpc, "Telemetry"),
        (history_pb2_grpc, "History"),
        (reports_pb2_grpc, "Reports"),
        (prompts_pb2_grpc, "Prompts"),
    ):
        getattr(module, f"add_{name}Servicer_to_server")(implementation, server)
    port = server.add_insecure_port("127.0.0.1:0")
    await server.start()
    client = DokimeClient(f"http://127.0.0.1:{port}", rpc_timeout=1)
    try:
        yield implementation, client
    finally:
        await client.close()
        if implementation.job is not None:
            implementation.job.cancel()
            await asyncio.gather(implementation.job, return_exceptions=True)
        await server.stop(0)


# === Entry Point === #


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_download_partial, id="download_partial"),
        pytest.param(case_cancelled_reader, id="cancelled_reader"),
        pytest.param(case_history_fail, id="history_fail"),
    ],
)
def test_contract(scenario: Callable[..., None], tmp_path: Path) -> None:
    scenario(tmp_path)
