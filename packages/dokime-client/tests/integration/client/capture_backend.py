# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Extend The Scripted Backend With Sealed History, Reports And Prompts"""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
import json
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import Any

# Third Party Imports
import grpc

# Local Imports
from dokime_client.client import DokimeClient, _wire

from . import backend
from .backend import CASE_ID, COMMAND_ID, CYCLE_ID, EXECUTION_ID, BackendFixture
from .canonical import _message, _value

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


class CaptureBackend(BackendFixture):
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
        self.methods["Prompts"] = ["ListPending", "GetPrompt", "Respond"]
        self.pending: list[dict[str, Any]] = []
        self.prompt_responses: dict[str, dict[str, Any]] = {}
        self.response_calls: list[str] = []
        self.bad_report = ""
        self.prompt_stream: str | None = None
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
        execution = request.execution_id if request.HasField("execution_id") else None
        selection = json.dumps([CYCLE_ID, execution], separators=(",", ":")).encode()
        stream = self.prompt_stream or "prompts.pending:" + hashlib.sha256(selection).hexdigest()
        return _message(
            "Prompts",
            "ListPending",
            direction="response",
            path="",
            prompts=[
                {
                    "prompt_id": prompt["prompt_id"],
                    "definition": {key: prompt["definition"][key] for key in ("prompt_type", "schema_version", "title")},
                    "scope": prompt["scope"],
                    "created_at": prompt["created_at"],
                }
                for prompt in self.pending
                if prompt["prompt_id"] not in self.prompt_responses
                and (execution is None or prompt["scope"].get("execution_id") == execution)
            ],
            cursor=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
                stream=stream,
                position=1,
            ),
        )

    async def GetPrompt(self, request: Any, context: Any) -> Any:
        (prompt,) = [item for item in self.pending if item["prompt_id"] == request.prompt_id]
        return _message(
            "Prompts",
            "GetPrompt",
            direction="response",
            path="",
            prompt=prompt,
            response=self.prompt_responses.get(request.prompt_id),
            observed_cursor={
                "epoch": self.epoch,
                "stream": self.prompt_stream or "prompts",
                "position": 1,
            },
        )

    async def Respond(self, request: Any, context: Any) -> Any:
        self.response_calls.append(request.prompt_id)
        if request.prompt_id in self.prompt_responses:
            await context.abort(grpc.StatusCode.FAILED_PRECONDITION, "prompt already answered")
        self.prompt_responses[request.prompt_id] = {
            "prompt_id": request.prompt_id,
            "action": request.action,
            "values": request.values,
            "accepted": request.accepted,
        }
        return _message(
            "Prompts",
            "Respond",
            direction="response",
            path="",
            prompt_id=request.prompt_id,
            command_id=COMMAND_ID,
            committed_revision=2,
            observed_cursor={"epoch": self.epoch, "stream": "prompts", "position": 2},
        )

    async def ListExecutions(self, request: Any, context: Any) -> Any:
        return _message(
            "Execution",
            "ListExecutions",
            direction="response",
            path="",
            executions=[
                _message(
                    "Execution",
                    "GetExecution",
                    direction="response",
                    path="summary",
                    execution_id=EXECUTION_ID,
                    case_id=CASE_ID,
                    ordinal=0,
                )
            ],
            cursor=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
            ),
        )

    async def BrowseExecutions(self, request: Any, context: Any) -> Any:
        return _message(
            "History",
            "BrowseExecutions",
            direction="response",
            path="",
            entries=[
                _message(
                    "History",
                    "BrowseExecutions",
                    direction="response",
                    path="entries",
                    metadata=_message(
                        "History",
                        "BrowseExecutions",
                        direction="response",
                        path="entries.metadata",
                        execution_id=EXECUTION_ID,
                        cycle_id=CYCLE_ID,
                    ),
                    checkpoint=self.point,
                )
            ],
            discovery_complete=True,
        )

    async def GetCheckpointDocument(self, request: Any, context: Any) -> Any:
        assert _wire.equal(request.checkpoint, self.point)
        return _message(
            "History",
            "GetCheckpointDocument",
            direction="response",
            path="",
            checkpoint=self.point,
            acquisition_finished=not self.unsealed,
            document=_message(
                "History",
                "GetCheckpointDocument",
                direction="response",
                path="document",
                execution=_message(
                    "History",
                    "GetCheckpointDocument",
                    direction="response",
                    path="document.execution",
                    execution_id=EXECUTION_ID,
                    status=self.seal_status,
                    result=_message(
                        "Execution",
                        "GetCycle",
                        direction="response",
                        path="result",
                        status={"Passed": {}},
                        passed=1,
                    ),
                    snapshot=_message(
                        "History",
                        "GetCheckpointDocument",
                        direction="response",
                        path="document.execution.snapshot",
                        template=_message(
                            "Execution",
                            "GetCase",
                            direction="response",
                            path="suite",
                            identifier="captured.template",
                        ),
                        case_definition=_message(
                            "History",
                            "GetCheckpointDocument",
                            direction="response",
                            path="document.execution.snapshot.case_definition",
                            case_id="captured-case",
                            template_id="captured-template",
                            metadata=_message(
                                "Execution",
                                "GetCase",
                                direction="response",
                                path="suite",
                                label="Charger At 230 V",
                            ),
                            suite=_message(
                                "Execution",
                                "GetCase",
                                direction="response",
                                path="suite",
                                identifier="charger.bench",
                            ),
                            parameters=[
                                _message(
                                    "Execution",
                                    "ListCaseParameters",
                                    direction="response",
                                    path="parameters",
                                    name="voltage",
                                    value=_value(number_value=230),
                                )
                            ],
                            generation_index=9_007_199_254_740_993,
                        ),
                        host_version="captured-version",
                    ),
                ),
            ),
        )

    async def GetCheckpointContext(self, request: Any, context: Any) -> Any:
        assert _wire.equal(request.checkpoint, self.point)
        return _message(
            "History",
            "GetCheckpointContext",
            direction="response",
            path="",
            checkpoint=self.point,
            traceability=[
                _message(
                    "Observation",
                    "GetTraceability",
                    direction="response",
                    path="snapshots",
                    traceability_id="instrument",
                    entries=[
                        _message(
                            "Observation",
                            "GetTraceability",
                            direction="response",
                            path="snapshots.entries",
                            metadata=_message(
                                "Execution",
                                "GetCase",
                                direction="response",
                                path="suite",
                                identifier="serial",
                            ),
                            value=_value(text_value="frozen-serial"),
                            source="device",
                        ),
                    ],
                ),
            ],
        )

    async def GetCheckpointEvidence(self, request: Any, context: Any) -> Any:
        return _message(
            "History",
            "GetCheckpointEvidence",
            direction="response",
            path="",
            checkpoint=self.point,
            telemetry_tables=[
                _message(
                    "History",
                    "GetCheckpointEvidence",
                    direction="response",
                    path="telemetry_tables",
                    table="voltage",
                )
            ],
            artifacts=[
                _message(
                    "Content",
                    "UploadArtifact",
                    direction="response",
                    path="",
                    artifact_id="artifact-1",
                    filename="../../escape.txt",
                    size_bytes=len(self.attachment),
                    sha256=hashlib.sha256(self.attachment).hexdigest(),
                    state={"Complete": {}},
                )
            ],
        )

    async def ListCheckpointRecords(self, request: Any, context: Any) -> Any:
        start = RECORD_CURSORS.index(request.after) + 1 if request.after else 0
        scope = _wire.nested("Evidence", "ListRecords", "selection.scope", execution_id=EXECUTION_ID)
        records = [item for item in self.records if self.record_selected(item, scope)]
        return _message(
            "History",
            "ListCheckpointRecords",
            direction="response",
            path="",
            checkpoint=self.point,
            records=[
                _message(
                    "History",
                    "ListCheckpointRecords",
                    direction="response",
                    path="records",
                    epoch=self.epoch,
                    record=item,
                )
                for item in records[start : start + 1]
            ],
            next_after=RECORD_CURSORS[start] if start + 1 < len(records) else None,
            issues=[
                _message(
                    "History",
                    "BrowseExecutions",
                    direction="response",
                    path="issues",
                    message="missing data",
                )
            ]
            if self.history_issue
            else [],
        )

    async def ListCheckpointTelemetry(self, request: Any, context: Any) -> Any:
        start = SAMPLE_CURSORS.index(request.after) + 1 if request.after else 0
        rows = [
            _message(
                "History",
                "ListCheckpointTelemetry",
                direction="response",
                path="rows",
                epoch=self.epoch,
                ordinal=item.sequence,
                sample=item,
            )
            for item in self.samples[start : start + 1]
        ]
        return _message(
            "History",
            "ListCheckpointTelemetry",
            direction="response",
            path="",
            checkpoint=self.point,
            table=request.table,
            rows=rows,
            next_after=SAMPLE_CURSORS[start] if start + 1 < len(self.samples) else None,
        )

    async def DownloadCheckpointArtifact(self, request: Any, context: Any) -> Any:
        digest = hashlib.sha256(self.attachment).hexdigest()
        yield _message(
            "Content",
            "DownloadArtifact",
            direction="output",
            payload={
                "Metadata": {
                    "field_0": _message(
                        "Content",
                        "UploadArtifact",
                        direction="response",
                        path="",
                        artifact_id="artifact-1",
                        size_bytes=len(self.attachment),
                        sha256=digest,
                    )
                }
            },
        )
        yield _message(
            "Content",
            "DownloadArtifact",
            direction="output",
            payload={
                "Chunk": {
                    "field_0": _message(
                        "Content",
                        "DownloadAppearanceAsset",
                        direction="output",
                        path="payload.Chunk.field_0",
                        data=self.attachment,
                    )
                }
            },
        )
        yield _message(
            "Content",
            "DownloadArtifact",
            direction="output",
            payload={
                "Complete": {
                    "field_0": _message(
                        "Content",
                        "DownloadAppearanceAsset",
                        direction="output",
                        path="payload.Complete.field_0",
                        size_bytes=len(self.attachment),
                        sha256=digest,
                    )
                }
            },
        )

    async def DownloadCycleReport(self, request: Any, context: Any) -> Any:
        yield _message(
            "Reports",
            "DownloadCycleReport",
            direction="output",
            payload={
                "Report": {
                    "field_0": _message(
                        "Reports",
                        "DownloadCycleReport",
                        direction="output",
                        path="payload.Report.field_0",
                        test_cycle_id="wrong" if self.bad_report == "identity" else CYCLE_ID,
                        identifier="wrong" if self.bad_report == "identifier" else request.identifier,
                    )
                }
            },
        )
        yield _message(
            "Reports",
            "DownloadCycleReport",
            direction="output",
            payload={
                "Chunk": {
                    "field_0": _message(
                        "Reports",
                        "DownloadCycleReport",
                        direction="output",
                        path="payload.Chunk.field_0",
                        offset=1 if self.bad_report == "offset" else 0,
                        data=self.html,
                    )
                }
            },
        )
        if self.bad_report == "stall":
            self.report_stalled.set()
            try:
                await asyncio.Event().wait()
            finally:
                self.report_cancelled.set()
        if self.bad_report == "truncated":
            return
        yield _message(
            "Reports",
            "DownloadCycleReport",
            direction="output",
            payload={
                "Complete": {
                    "field_0": _message(
                        "Reports",
                        "DownloadCycleReport",
                        direction="output",
                        path="payload.Complete.field_0",
                        size_bytes=len(self.html),
                        sha256="bad" if self.bad_report == "hash" else hashlib.sha256(self.html).hexdigest(),
                    )
                }
            },
        )
        if self.bad_report == "trailing":
            yield _message(
                "Reports",
                "DownloadCycleReport",
                direction="output",
                payload={
                    "Chunk": {
                        "field_0": _message(
                            "Reports",
                            "DownloadCycleReport",
                            direction="output",
                            path="payload.Chunk.field_0",
                            data=b"extra",
                        )
                    }
                },
            )
        if self.bad_report == "status":
            await context.abort(grpc.StatusCode.DATA_LOSS, "failed after completion frame")


# === Public Functions === #


@asynccontextmanager
async def serve() -> AsyncIterator[tuple[CaptureBackend, DokimeClient]]:
    async with backend.serve(CaptureBackend()) as (implementation, client):
        yield implementation, client


# === Private Functions === #

# === Entry Point === #
