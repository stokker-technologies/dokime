# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Serve A Scripted Backend Over Real In-Process gRPC Sockets"""

# === Imports === #

# Standard Library Imports
import asyncio
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import Any

# Third Party Imports
import grpc
from google.protobuf.message import Message

# Local Imports
from dokime.client import DokimeClient, _wire

from .canonical import _message, _value, _value_object, register

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Suites and templates are named by readable IDs; only the template's
# registered UUID travels on the planning wire.
SUITE_ID = "bench.checks"
TEMPLATE = "bench.checks.probe"
TEMPLATE_ID = "25eb4f2c-542a-48a0-b4a1-bb0827c7c212"
REQUEST_ID = "98184567-8924-4fb0-b4e2-667347e097a1"
CASE_ID = "a13ae695-1f02-4464-9f7b-0b14ecffcb41"
CYCLE_ID = "ad18e512-38cf-4e49-9e07-6da29595521e"
EXECUTION_ID = "47acbd5a-7496-4947-8078-0b282cf58643"
COMMAND_ID = "70aefc57-b498-4335-924b-5a45614fb313"

# === Types === #

# === Classes === #


class BackendFixture:
    def __init__(self) -> None:
        self.endpoint = ""
        self.command_reads = 0
        self.command_execution_id = EXECUTION_ID
        self.epoch = "epoch-1"
        self.backend_id = "backend-1"
        self.version = "0.6.0"
        self.health_epoch: str | None = None
        self.availability_epoch: str | None = None
        self.readiness = {"Ready": {}}
        self.max_request_bytes = 16 * 1024 * 1024
        self.max_response_bytes = 16 * 1024 * 1024
        self.package = "dokime.v1"
        self.descriptor = bytes(32)
        self.key: str | None = None
        self.created = 0
        self.append_calls = 0
        self.cycle_request: Any = None
        self.restart_during: str | None = None
        self.execution_control_calls: list[str] = []
        self.stop_calls = 0
        self.rerun_calls: list[str] = []
        self.case_runs: list[str] = []
        self.run_calls = 0
        self.receipt_reads = 0
        self.fail_receipt_read = False
        self.phase = {"Queued": {}}
        self.fail_admission = False
        self.execution_mode: Message | None = None
        self.deny_run = False
        self.records: list[Any] = []
        self.samples: list[Any] = []
        self.sample_page_size = 5000
        self.changed = asyncio.Event()
        # Gates Start Open; A Story Clears One To Hold That Phase Until Released.
        self.finish_gate = asyncio.Event()
        self.finish_gate.set()
        self.terminal = asyncio.Event()
        self.sealed = asyncio.Event()
        self.seal_automatically = True
        self.seal_status = {"Passed": {}}
        self.point = _message(
            "History",
            "GetExecutionCheckpoint",
            direction="response",
            path="",
            execution_identifier=EXECUTION_ID,
            position=_message(
                "History",
                "GetExecutionCheckpoint",
                direction="response",
                path="position",
                history=_message(
                    "History",
                    "GetExecutionCheckpoint",
                    direction="response",
                    path="position.history",
                    store="fixture.store",
                    epoch="fixture.epoch",
                ),
                sequence=7,
            ),
            version=b"\x0dfixture.store\x0dfixture.epoch\x07",
            evidence_through=_message(
                "History",
                "GetExecutionCheckpoint",
                direction="response",
                path="position",
                history=_message(
                    "History",
                    "GetExecutionCheckpoint",
                    direction="response",
                    path="position.history",
                    store="fixture.store",
                    epoch="fixture.epoch",
                ),
                sequence=3,
            ),
        )
        self.job: asyncio.Task[None] | None = None
        self.overrides: Any = None
        self.identity_gate = asyncio.Event()
        self.identity_gate.set()
        self.catalog_gate = asyncio.Event()
        self.catalog_gate.set()
        self.catalog_deadline: float | None = None
        self.checkpoint_missed = asyncio.Event()
        self.records_watched = asyncio.Event()
        self.records_parked = asyncio.Event()
        self.catalog_started = asyncio.Event()
        self.suite_page: Any = None
        self.catalog_finished = asyncio.Event()
        self.sent_cursors: list[dict[str, int]] = []
        self.methods = {
            "Backend": ["GetIdentity", "GetHealth", "GetAvailability"],
            "Catalog": ["ListSuites", "GetSuite"],
            "Execution": [
                "CreateCycle",
                "AppendPlannedCases",
                "RunCycle",
                "RunExecution",
                "GetRunRequest",
                "GetExecutionCommand",
                "GetCycle",
                "GetExecution",
                "ListExecutions",
                "PauseExecution",
                "ResumeExecution",
                "StopExecution",
                "ScheduleCaseRerun",
                "RunCase",
            ],
            "Evidence": ["WatchRecords", "ListRecords"],
            "Telemetry": ["ListSamples"],
            "History": ["GetExecutionCheckpoint", "GetCheckpointDocument"],
        }

    async def GetIdentity(self, request: Any, context: Any) -> Any:
        await self.identity_gate.wait()
        return _message(
            "Backend",
            "GetIdentity",
            direction="response",
            path="",
            backend_id=self.backend_id,
            epoch=self.epoch,
            api=_message(
                "Backend",
                "GetIdentity",
                direction="response",
                path="api",
                package=self.package,
                version=self.version,
                descriptor_sha256=self.descriptor,
            ),
        )

    async def GetHealth(self, request: Any, context: Any) -> Any:
        return _message(
            "Backend",
            "GetHealth",
            direction="response",
            path="",
            backend_id=self.backend_id,
            epoch=self.health_epoch or self.epoch,
            readiness=self.readiness,
        )

    async def GetAvailability(self, request: Any, context: Any) -> Any:
        return _message(
            "Backend",
            "GetAvailability",
            direction="response",
            path="",
            limits=_message(
                "Backend",
                "GetAvailability",
                direction="response",
                path="limits",
                max_request_bytes=self.max_request_bytes,
                max_response_bytes=self.max_response_bytes,
            ),
            backend_id=self.backend_id,
            epoch=self.availability_epoch or self.epoch,
            methods=[
                f"/dokime.v1.{service}/{method}"
                for service, methods in self.methods.items()
                for method in methods
            ],
        )

    async def ListSuites(self, request: Any, context: Any) -> Any:
        self.catalog_deadline = context.time_remaining()
        self.catalog_started.set()
        try:
            await self.catalog_gate.wait()
        finally:
            self.catalog_finished.set()
        if self.suite_page is not None:
            return self.suite_page
        return _message(
            "Catalog",
            "ListSuites",
            direction="response",
            path="",
            suites=[
                _message(
                    "Catalog",
                    "ListSuites",
                    direction="response",
                    path="suites",
                    suite_id=SUITE_ID,
                    metadata=_message(
                        "Execution",
                        "GetCase",
                        direction="response",
                        path="suite",
                        identifier=SUITE_ID,
                        label="Suite",
                    ),
                )
            ],
            cursor=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
                stream="runtime.state",
                position=1,
            ),
        )

    async def GetSuite(self, request: Any, context: Any) -> Any:
        if request.suite_id != SUITE_ID:
            await context.abort(grpc.StatusCode.NOT_FOUND, "suite not found")
        return _message(
            "Catalog",
            "GetSuite",
            direction="response",
            path="",
            identity=_message(
                "Catalog",
                "ListSuites",
                direction="response",
                path="suites",
                suite_id=SUITE_ID,
                metadata=_message(
                    "Execution",
                    "GetCase",
                    direction="response",
                    path="suite",
                    identifier=SUITE_ID,
                    label="Suite",
                ),
            ),
            templates=[
                _message(
                    "Catalog",
                    "GetSuite",
                    direction="response",
                    path="templates",
                    template_id=TEMPLATE_ID,
                    metadata=_message(
                        "Execution",
                        "GetCase",
                        direction="response",
                        path="suite",
                        identifier=TEMPLATE,
                        label="Template",
                    ),
                )
            ],
        )

    async def CreateCycle(self, request: Any, context: Any) -> Any:
        self.created += 1
        if self.restart_during == "CreateCycle":
            self.epoch = "epoch-2"
        assert request.metadata.label.strip()
        assert not request.metadata.identifier
        self.cycle_request = request
        return _message(
            "Execution",
            "CreateCycle",
            direction="response",
            path="",
            cycle_id=CYCLE_ID,
            observed_cursor=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
            ),
        )

    async def AppendPlannedCases(self, request: Any, context: Any) -> Any:
        self.append_calls += 1
        if self.restart_during == "AppendPlannedCases":
            self.epoch = "epoch-2"
        assert request.cycle_id == CYCLE_ID == request.plan.cycle_id
        assert len(request.plan.selections) == 1
        assert not request.plan.case_ids
        self.overrides = {
            entry.key: entry.value for entry in request.plan.selections[0].parameters
        }
        return _message(
            "Execution",
            "AppendPlannedCases",
            direction="response",
            path="",
            observed_cursor=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
            ),
        )

    async def RunCycle(self, request: Any, context: Any) -> Any:
        self.run_calls += 1
        if self.deny_run:
            return _message(
                "Execution",
                "RunCycle",
                direction="response",
                outcome={
                    "Rejected": {
                        "field_0": _message(
                            "Execution",
                            "RunCase",
                            direction="response",
                            path="outcome.Rejected.field_0",
                            code="provider_unavailable",
                            message="No provider",
                        )
                    }
                },
            )
        if self.key is None:
            self.key = request.idempotency_key
            self.phase = {"Running": {}}
            self.job = asyncio.create_task(self.produce())
        elif self.key != request.idempotency_key:
            return _message(
                "Execution",
                "RunCycle",
                direction="response",
                outcome={
                    "Rejected": {
                        "field_0": _message(
                            "Execution",
                            "RunCase",
                            direction="response",
                            path="outcome.Rejected.field_0",
                            code="request_identity_conflict",
                        )
                    }
                },
            )
        if self.fail_admission:
            self.fail_admission = False
            await context.abort(
                grpc.StatusCode.UNAVAILABLE, "response lost after admission"
            )
        return _message(
            "Execution",
            "RunCycle",
            direction="response",
            outcome={
                "Accepted": {
                    "field_0": _message(
                        "Execution",
                        "RunCase",
                        direction="response",
                        path="outcome.Accepted.field_0",
                        command_id=REQUEST_ID,
                        cursor=_message(
                            "Evidence",
                            "ListRecords",
                            direction="request",
                            path="after",
                            epoch=self.epoch,
                        ),
                    )
                }
            },
        )

    async def RunExecution(self, request: Any, context: Any) -> Any:
        assert request.execution_id == EXECUTION_ID
        if self.execution_mode is None:
            self.execution_mode = request.mode
        elif self.execution_mode != request.mode:
            return _message(
                "Execution",
                "RunCycle",
                direction="response",
                outcome={
                    "Rejected": {
                        "field_0": _message(
                            "Execution",
                            "RunCase",
                            direction="response",
                            path="outcome.Rejected.field_0",
                            code="request_identity_conflict",
                        )
                    }
                },
            )
        return await self.RunCycle(
            _message(
                "Execution",
                "RunCycle",
                direction="request",
                path="",
                cycle_id=CYCLE_ID,
                idempotency_key=request.idempotency_key,
            ),
            context,
        )

    async def GetRunRequest(self, request: Any, context: Any) -> Any:
        self.receipt_reads += 1
        if self.fail_receipt_read:
            self.fail_receipt_read = False
            await context.abort(
                grpc.StatusCode.UNAVAILABLE, "fixture receipt read failed"
            )
        assert request.request_id == REQUEST_ID
        return _message(
            "Execution",
            "GetRunRequest",
            direction="response",
            path="",
            request_id=REQUEST_ID,
            cycle_id=CYCLE_ID,
            phase=self.phase,
            planned_count=1,
            finished_count=int(self.terminal.is_set()),
        )

    async def GetExecutionCommand(self, request: Any, context: Any) -> Any:
        assert request.request_id == REQUEST_ID
        assert request.command_id == COMMAND_ID
        self.command_reads += 1
        return _message(
            "Execution",
            "GetExecutionCommand",
            direction="response",
            path="",
            request_id=REQUEST_ID,
            command_id=COMMAND_ID,
            execution_id=self.command_execution_id,
            action={"Next": {}},
            phase={"Interrupted": {}},
        )

    async def GetCycle(self, request: Any, context: Any) -> Any:
        if self.restart_during == "GetCycle":
            self.epoch = "epoch-2"
        return _message(
            "Execution",
            "GetCycle",
            direction="response",
            path="",
            cycle_id=request.cycle_id,
            observed_cursor=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
            ),
            case_count=1,
            execution_count=1,
            result=_message(
                "Execution",
                "GetCycle",
                direction="response",
                path="result",
                status={"Passed": {}},
                passed=1,
            ),
        )

    async def GetExecution(self, request: Any, context: Any) -> Any:
        assert request.execution_id == EXECUTION_ID
        return _message(
            "Execution",
            "GetExecution",
            direction="response",
            path="",
            summary=_message(
                "Execution",
                "GetExecution",
                direction="response",
                path="summary",
                execution_id=EXECUTION_ID,
                case_id=CASE_ID,
                status={"Passed": {}} if self.terminal.is_set() else {"Running": {}},
            ),
            observed_cursor=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
            ),
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
            ]
            if request.cycle_id == CYCLE_ID
            else [],
            cursor=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
                stream="runtime.state",
                position=1,
            ),
        )

    async def GetExecutionCheckpoint(self, request: Any, context: Any) -> Any:
        assert request.execution_identifier == EXECUTION_ID
        if not self.sealed.is_set():
            self.checkpoint_missed.set()
            await context.abort(
                grpc.StatusCode.NOT_FOUND, "execution has no saved seal"
            )
        return self.point

    async def GetCheckpointDocument(self, request: Any, context: Any) -> Any:
        assert _wire.equal(request.checkpoint, self.point)
        assert self.sealed.is_set()
        passed = self.seal_status == {"Passed": {}}
        return _message(
            "History",
            "GetCheckpointDocument",
            direction="response",
            path="",
            checkpoint=self.point,
            acquisition_finished=True,
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
                    case_id=CASE_ID,
                    status=self.seal_status,
                    result=_message(
                        "Execution",
                        "GetCycle",
                        direction="response",
                        path="result",
                        status={"Passed": {}} if passed else {"Cancelled": {}},
                        passed=int(passed),
                    ),
                ),
            ),
        )

    async def PauseExecution(self, request: Any, context: Any) -> Any:
        return await self.execution_control("PauseExecution", request, context)

    async def ResumeExecution(self, request: Any, context: Any) -> Any:
        return await self.execution_control("ResumeExecution", request, context)

    async def StopExecution(self, request: Any, context: Any) -> Any:
        return await self.execution_control("StopExecution", request, context)

    async def execution_control(self, method: str, request: Any, context: Any) -> Any:
        assert request.execution_id == EXECUTION_ID
        self.execution_control_calls.append(method)
        if self.restart_during == method:
            self.epoch = "epoch-2"
        if method == "StopExecution":
            if not self.terminal.is_set():
                self.stop_calls += 1
                if self.job is not None:
                    self.job.cancel()
                    await asyncio.gather(self.job, return_exceptions=True)
                self.publish("stopped")
                self.phase = {"Stopped": {}}
                self.terminal.set()
                self.seal_status = {"Cancelled": {}}
                if self.seal_automatically:
                    self.sealed.set()
        elif self.terminal.is_set():
            await context.abort(
                grpc.StatusCode.FAILED_PRECONDITION, "execution is finished"
            )
        return _message(
            "Execution",
            "AppendPlannedCases",
            direction="response",
            path="",
            observed_cursor=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
            ),
        )

    async def ScheduleCaseRerun(self, request: Any, context: Any) -> Any:
        assert request.cycle_id == CYCLE_ID
        self.rerun_calls.append(request.case_id)
        return _message(
            "Execution",
            "AppendPlannedCases",
            direction="response",
            path="",
            observed_cursor=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
            ),
        )

    async def RunCase(self, request: Any, context: Any) -> Any:
        assert request.cycle_id == CYCLE_ID
        self.case_runs.append(f"{request.case_id}:{request.idempotency_key}")
        return await self.RunCycle(
            _message(
                "Execution",
                "RunCycle",
                direction="request",
                path="",
                cycle_id=CYCLE_ID,
                idempotency_key=request.idempotency_key,
            ),
            context,
        )

    def publish(self, kind: str) -> None:
        n = len(self.records) + 1
        self.records.append(
            _message(
                "Evidence",
                "ListRecords",
                direction="response",
                path="records",
                record_id=f"event-{n}",
                record_type=f"fixture.{kind}",
                sequence=n,
                scope=_message(
                    "Execution",
                    "GetExecution",
                    direction="response",
                    path="scope",
                    cycle_id=CYCLE_ID,
                    execution_id=EXECUTION_ID,
                ),
                severity={"Info": {}},
                details=_value_object(fields={"number": _value(integer_value=n)}),
            )
        )
        self.samples.append(
            _message(
                "Telemetry",
                "ListSamples",
                direction="response",
                path="samples",
                sample_id=f"sample-{n}",
                stream="voltage",
                sequence=n,
                scope=_message(
                    "Execution",
                    "GetExecution",
                    direction="response",
                    path="scope",
                    cycle_id=CYCLE_ID,
                    execution_id=EXECUTION_ID,
                ),
                values=_value_object(
                    fields={"voltage": _value(number_value=230.0 + n)}
                ),
            )
        )
        self.changed.set()

    async def produce(self) -> None:
        self.publish("started")
        await self.finish_gate.wait()
        self.publish("finished")
        self.phase = {"Completed": {}}
        self.terminal.set()
        if self.seal_automatically:
            self.sealed.set()

    def page(self, selection: Any, after: int, through: int) -> Any:
        scanned = min(after + 1, through)
        records = [
            r
            for r in self.records[after:scanned]
            if (not selection.record_types or r.record_type in selection.record_types)
            and (
                not selection.scope.HasField("execution_id")
                or r.scope.execution_id == selection.scope.execution_id
            )
            and (
                not selection.scope.HasField("cycle_id")
                or r.scope.cycle_id == selection.scope.cycle_id
            )
        ]
        return _message(
            "Evidence",
            "ListRecords",
            direction="response",
            path="",
            records=records,
            scanned=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
                stream="records",
                position=scanned,
            ),
            through=_message(
                "Evidence",
                "ListRecords",
                direction="request",
                path="after",
                epoch=self.epoch,
                stream="records",
                position=through,
            ),
            replay_complete=scanned == through,
        )

    async def ListRecords(self, request: Any, context: Any) -> Any:
        assert request.HasField("after") == request.HasField("through")
        return self.page(
            request.selection,
            request.after.position,
            request.through.position
            if request.HasField("through")
            else len(self.records),
        )

    async def WatchRecords(self, request: Any, context: Any) -> Any:
        position = 0
        self.records_watched.set()
        yield self.page(request.selection, 0, 0)
        while True:
            self.changed.clear()
            if position < len(self.records):
                page = self.page(request.selection, position, len(self.records))
                position = page.scanned.position
                yield page
            else:
                self.records_parked.set()
                await self.changed.wait()

    async def ListSamples(self, request: Any, context: Any) -> Any:
        assert not request.latest
        self.sent_cursors.append(
            {entry.key: entry.value for entry in request.after_stream_cursors}
        )
        position = next(
            (
                entry.value
                for entry in request.after_stream_cursors
                if entry.key == "voltage"
            ),
            0,
        )
        limit = min(request.limit, self.sample_page_size)
        samples = self.samples[position : position + limit]
        through = position + len(samples)
        return _message(
            "Telemetry",
            "ListSamples",
            direction="response",
            path="",
            samples=samples,
            through_stream_cursors={"voltage": through},
            complete=len(samples) < limit,
        )


# === Public Functions === #


@asynccontextmanager
async def serve(
    implementation: BackendFixture | None = None,
) -> AsyncIterator[tuple[Any, DokimeClient]]:
    backend = BackendFixture() if implementation is None else implementation
    server = grpc.aio.server()
    register(server, backend)
    port = server.add_insecure_port("127.0.0.1:0")
    await server.start()
    backend.endpoint = f"http://127.0.0.1:{port}"
    client = DokimeClient(backend.endpoint)
    try:
        yield backend, client
    finally:
        await client.close()
        if backend.job is not None:
            backend.job.cancel()
            await asyncio.gather(backend.job, return_exceptions=True)
        await server.stop(0)


async def collect[T](iterator: AsyncIterator[T]) -> list[T]:
    return [item async for item in iterator]


# === Private Functions === #

# === Entry Point === #
