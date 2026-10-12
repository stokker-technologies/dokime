# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Exercise the public client over actual gRPC sockets through the shared Rust transport."""

# === Imports === #

# Standard Library Imports
from collections.abc import Callable
import asyncio
import math
from datetime import UTC, datetime, timedelta
from contextlib import asynccontextmanager
import json
import runpy
from pathlib import Path
from typing import Any

# Third Party Imports
import grpc
import pytest

# Local Imports
from dokime.client import BackendChanged, ClientClosed, DokimeClient, ProtocolError
from dokime.client import (
    CommandPhase,
    ExecutionSubmissionUncertain,
    RpcError,
    RunPhase,
    RunRejected,
    SubmissionUncertain,
)
from dokime.client._generated.dokime.v1 import backend_pb2 as backend
from dokime.client._generated.dokime.v1 import backend_pb2_grpc
from dokime.client._generated.dokime.v1 import catalog_pb2 as catalog
from dokime.client._generated.dokime.v1 import catalog_pb2_grpc
from dokime.client._generated.dokime.v1 import common_pb2 as common
from dokime.client._generated.dokime.v1 import checkpoint_pb2 as checkpoint
from dokime.client._generated.dokime.v1 import evidence_pb2 as evidence
from dokime.client._generated.dokime.v1 import evidence_pb2_grpc
from dokime.client._generated.dokime.v1 import execution_pb2 as execution
from dokime.client._generated.dokime.v1 import execution_pb2_grpc
from dokime.client._generated.dokime.v1 import history_pb2 as history
from dokime.client._generated.dokime.v1 import history_pb2_grpc
from dokime.client._generated.dokime.v1 import planning_pb2 as planning
from dokime.client._generated.dokime.v1 import planning_pb2_grpc
from dokime.client._generated.dokime.v1 import telemetry_pb2 as telemetry
from dokime.client._generated.dokime.v1 import telemetry_pb2_grpc
from dokime.client._generated.dokime.v1 import value_pb2 as value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

SUITE_ID = "25eb4f2c-542a-48a0-b4a1-bb0827c7c211"
TEMPLATE_ID = "25eb4f2c-542a-48a0-b4a1-bb0827c7c212"
REQUEST_ID = "98184567-8924-4fb0-b4e2-667347e097a1"
CASE_ID = "a13ae695-1f02-4464-9f7b-0b14ecffcb41"
CYCLE_ID = "ad18e512-38cf-4e49-9e07-6da29595521e"
EXECUTION_ID = "47acbd5a-7496-4947-8078-0b282cf58643"
COMMAND_ID = "70aefc57-b498-4335-924b-5a45614fb313"

# === Types === #

# === Classes === #


class BackendFixture(
    backend_pb2_grpc.BackendServicer,
    catalog_pb2_grpc.CatalogServicer,
    planning_pb2_grpc.PlanningServicer,
    execution_pb2_grpc.ExecutionServicer,
    evidence_pb2_grpc.EvidenceServicer,
    telemetry_pb2_grpc.TelemetryServicer,
    history_pb2_grpc.HistoryServicer,
):
    def __init__(self) -> None:
        self.endpoint = ""
        self.command_reads = 0
        self.command_execution_id = EXECUTION_ID
        self.epoch = "epoch-1"
        self.backend_id = "backend-1"
        self.version = "0.6.0"
        self.health_epoch: str | None = None
        self.capabilities_epoch: str | None = None
        self.readiness = backend.READINESS_READY
        self.max_request_bytes = 1024
        self.max_response_bytes = 1024
        self.package = "dokime.v1"
        self.descriptor = b""
        self.key: str | None = None
        self.created = 0
        self.registration_calls = 0
        self.restart_during: str | None = None
        self.execution_control_calls: list[str] = []
        self.stop_calls = 0
        self.run_calls = 0
        self.receipt_reads = 0
        self.fail_receipt_read = False
        self.phase = execution.RUN_REQUEST_PHASE_QUEUED
        self.fail_admission = False
        self.execution_mode: int | None = None
        self.deny_run = False
        self.records: list[Any] = []
        self.samples: list[Any] = []
        self.changed = asyncio.Event()
        self.terminal = asyncio.Event()
        self.sealed = asyncio.Event()
        self.seal_automatically = True
        self.seal_status = execution.EXECUTION_STATUS_PASSED
        self.point = checkpoint.ExecutionCheckpoint(
            execution_identifier=EXECUTION_ID,
            position=checkpoint.StorageCursor(
                history=checkpoint.StoreHistory(
                    store="fixture.store", epoch="fixture.epoch"
                ),
                sequence=7,
            ),
            # Canonical Native ObjectVersionToken: Store, History Epoch, Revision.
            version=b"\x0dfixture.store\x0dfixture.epoch\x07",
            evidence_through=checkpoint.StorageCursor(
                history=checkpoint.StoreHistory(
                    store="fixture.store", epoch="fixture.epoch"
                ),
                sequence=3,
            ),
        )
        self.job: asyncio.Task[None] | None = None
        self.overrides: Any = None
        self.identity_delay = 0.0
        self.catalog_delay = 0.0
        self.catalog_deadline: float | None = None
        self.catalog_started = asyncio.Event()
        self.catalog_finished = asyncio.Event()
        self.sent_cursors: list[dict[str, int]] = []
        self.methods = {
            "Backend": ["GetIdentity", "GetHealth", "GetCapabilities"],
            "Catalog": ["ListSuites", "GetSuite"],
            "Planning": ["RegisterSelection"],
            "Execution": [
                "CreateCycle",
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
            ],
            "Evidence": ["WatchRecords", "ListRecords"],
            "Telemetry": ["ListSamples"],
            "History": ["GetExecutionCheckpoint", "GetCheckpointDocument"],
        }

    async def GetIdentity(self, request: Any, context: Any) -> Any:
        await asyncio.sleep(self.identity_delay)
        return backend.BackendIdentity(
            backend_id=self.backend_id,
            epoch=self.epoch,
            api=backend.ApiIdentity(
                package=self.package,
                version=self.version,
                descriptor_sha256=self.descriptor,
            ),
        )

    async def GetHealth(self, request: Any, context: Any) -> Any:
        return backend.BackendHealth(
            backend_id=self.backend_id,
            epoch=self.health_epoch or self.epoch,
            readiness=self.readiness,
        )

    async def GetCapabilities(self, request: Any, context: Any) -> Any:
        return backend.BackendCapabilities(
            limits=backend.BackendLimits(
                max_request_bytes=self.max_request_bytes,
                max_response_bytes=self.max_response_bytes,
            ),
            backend_id=self.backend_id,
            epoch=self.capabilities_epoch or self.epoch,
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
            await asyncio.sleep(self.catalog_delay)
        finally:
            self.catalog_finished.set()
        return catalog.ListSuitesReply(
            suites=[
                catalog.SuiteIdentity(
                    suite_id=SUITE_ID,
                    metadata=common.Metadata(identifier="suite", label="Suite"),
                )
            ],
            cursor=common.Cursor(epoch=self.epoch, stream="runtime.state", position=1),
        )

    async def GetSuite(self, request: Any, context: Any) -> Any:
        if request.suite_id != SUITE_ID:
            await context.abort(grpc.StatusCode.NOT_FOUND, "suite not found")
        return catalog.SuiteDefinition(
            identity=catalog.SuiteIdentity(
                suite_id=SUITE_ID,
                metadata=common.Metadata(identifier="suite", label="Suite"),
            ),
            templates=[
                catalog.TemplateDefinition(
                    template_id=TEMPLATE_ID,
                    metadata=common.Metadata(identifier="template", label="Template"),
                )
            ],
        )

    async def RegisterSelection(self, request: Any, context: Any) -> Any:
        self.registration_calls += 1
        if self.restart_during == "RegisterSelection":
            self.epoch = "epoch-2"
        assert request.idempotency_key
        assert request.plan.max_cases == 1
        assert len(request.plan.selections) == 1
        assert not request.plan.case_ids
        self.overrides = request.plan.selections[0].parameters
        return planning.PreviewSelectionReply(
            cases=[
                planning.PlannedCase(
                    case_id=CASE_ID, suite_id=SUITE_ID, template_id=TEMPLATE_ID
                )
            ],
            observed_cursor=common.Cursor(epoch=self.epoch),
        )

    async def CreateCycle(self, request: Any, context: Any) -> Any:
        self.created += 1
        if self.restart_during == "CreateCycle":
            self.epoch = "epoch-2"
        assert list(request.plan.case_ids) == [CASE_ID]
        assert not request.plan.selections
        assert request.metadata.identifier and request.metadata.label
        return execution.CreateCycleReply(
            cycle_id=CYCLE_ID, observed_cursor=common.Cursor(epoch=self.epoch)
        )

    async def RunCycle(self, request: Any, context: Any) -> Any:
        self.run_calls += 1
        if self.deny_run:
            return execution.RunReply(
                rejected=execution.Rejection(
                    code="provider_unavailable", message="No provider"
                )
            )
        if self.key is None:
            self.key = request.idempotency_key
            self.phase = execution.RUN_REQUEST_PHASE_RUNNING
            self.job = asyncio.create_task(self.produce())
        elif self.key != request.idempotency_key:
            return execution.RunReply(
                rejected=execution.Rejection(code="request_identity_conflict")
            )
        if self.fail_admission:
            self.fail_admission = False
            await context.abort(
                grpc.StatusCode.UNAVAILABLE, "response lost after admission"
            )
        return execution.RunReply(
            accepted=execution.Admission(
                command_id=REQUEST_ID, cursor=common.Cursor(epoch=self.epoch)
            )
        )

    async def RunExecution(self, request: Any, context: Any) -> Any:
        assert request.execution_id == EXECUTION_ID
        if self.execution_mode is None:
            self.execution_mode = request.mode
        elif self.execution_mode != request.mode:
            return execution.RunReply(
                rejected=execution.Rejection(code="request_identity_conflict")
            )
        return await self.RunCycle(
            execution.RunCycleRequest(
                cycle_id=CYCLE_ID, idempotency_key=request.idempotency_key
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
        return execution.RunRequestReceipt(
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
        return execution.ExecutionCommandReceipt(
            request_id=REQUEST_ID,
            command_id=COMMAND_ID,
            execution_id=self.command_execution_id,
            action=execution.EXECUTION_COMMAND_ACTION_NEXT,
            phase=execution.EXECUTION_COMMAND_PHASE_INTERRUPTED,
        )

    async def GetCycle(self, request: Any, context: Any) -> Any:
        if self.restart_during == "GetCycle":
            self.epoch = "epoch-2"
        return execution.CycleSummary(
            cycle_id=request.cycle_id,
            observed_cursor=common.Cursor(epoch=self.epoch),
            case_count=1,
            execution_count=1,
            result=execution.ResultSummary(
                status=execution.RESULT_STATUS_PASSED, passed=1
            ),
        )

    async def GetExecution(self, request: Any, context: Any) -> Any:
        assert request.execution_id == EXECUTION_ID
        return execution.ExecutionDetail(
            summary=execution.ExecutionSummary(
                execution_id=EXECUTION_ID,
                case_id=CASE_ID,
                status=execution.EXECUTION_STATUS_PASSED
                if self.terminal.is_set()
                else execution.EXECUTION_STATUS_RUNNING,
            ),
            observed_cursor=common.Cursor(epoch=self.epoch),
        )

    async def ListExecutions(self, request: Any, context: Any) -> Any:
        return execution.ListExecutionsReply(
            executions=[
                execution.ExecutionSummary(
                    execution_id=EXECUTION_ID, case_id=CASE_ID, ordinal=0
                )
            ]
            if request.cycle_id == CYCLE_ID
            else [],
            cursor=common.Cursor(epoch=self.epoch, stream="runtime.state", position=1),
        )

    async def GetExecutionCheckpoint(self, request: Any, context: Any) -> Any:
        assert request.execution_identifier == EXECUTION_ID
        if not self.sealed.is_set():
            await context.abort(
                grpc.StatusCode.NOT_FOUND, "execution has no saved seal"
            )
        return self.point

    async def GetCheckpointDocument(self, request: Any, context: Any) -> Any:
        assert request.checkpoint == self.point
        assert self.sealed.is_set()
        passed = self.seal_status == execution.EXECUTION_STATUS_PASSED
        return history.CheckpointDocument(
            checkpoint=self.point,
            acquisition_finished=True,
            document=history.HistoricalExecutionDocument(
                execution=history.HistoricalTestExecution(
                    execution_id=EXECUTION_ID,
                    case_id=CASE_ID,
                    status=self.seal_status,
                    result=execution.ResultSummary(
                        status=execution.RESULT_STATUS_PASSED
                        if passed
                        else execution.RESULT_STATUS_CANCELLED,
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
                self.phase = execution.RUN_REQUEST_PHASE_STOPPED
                self.terminal.set()
                self.seal_status = execution.EXECUTION_STATUS_CANCELLED
                if self.seal_automatically:
                    self.sealed.set()
        elif self.terminal.is_set():
            await context.abort(
                grpc.StatusCode.FAILED_PRECONDITION, "execution is finished"
            )
        return execution.ControlReply(observed_cursor=common.Cursor(epoch=self.epoch))

    def publish(self, kind: str) -> None:
        n = len(self.records) + 1
        self.records.append(
            evidence.CapturedRecord(
                record_id=f"event-{n}",
                record_type=f"fixture.{kind}",
                sequence=n,
                scope=common.EvidenceScope(
                    cycle_id=CYCLE_ID, execution_id=EXECUTION_ID
                ),
                severity=common.SEVERITY_INFO,
                details=value.ValueObject(
                    fields={"number": value.Value(integer_value=n)}
                ),
            )
        )
        self.samples.append(
            telemetry.TelemetrySample(
                sample_id=f"sample-{n}",
                stream="voltage",
                sequence=n,
                scope=common.EvidenceScope(
                    cycle_id=CYCLE_ID, execution_id=EXECUTION_ID
                ),
                values=value.ValueObject(
                    fields={"voltage": value.Value(number_value=230.0 + n)}
                ),
            )
        )
        self.changed.set()

    async def produce(self) -> None:
        self.publish("started")
        await asyncio.sleep(0.12)
        self.publish("finished")
        self.phase = execution.RUN_REQUEST_PHASE_COMPLETED
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
        return evidence.RecordPage(
            records=records,
            scanned=common.Cursor(epoch=self.epoch, stream="records", position=scanned),
            through=common.Cursor(epoch=self.epoch, stream="records", position=through),
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
        yield self.page(request.selection, 0, 0)
        while True:
            self.changed.clear()
            if position < len(self.records):
                page = self.page(request.selection, position, len(self.records))
                position = page.scanned.position
                yield page
            else:
                await self.changed.wait()

    async def ListSamples(self, request: Any, context: Any) -> Any:
        assert not request.latest
        self.sent_cursors.append(dict(request.after_stream_cursors))
        position = request.after_stream_cursors.get("voltage", 0)
        samples = self.samples[position : position + request.limit]
        through = position + len(samples)
        return telemetry.TelemetrySamplesReply(
            samples=samples,
            through_stream_cursors={"voltage": through},
            complete=len(samples) < request.limit,
        )


# === Public Functions === #


def case_native_attempt() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            client._transport.timeout = 0.05
            server.identity_delay = 10
            with pytest.raises(RpcError) as error:
                await client.connect()
            assert error.value.code == "DEADLINE_EXCEEDED"
            assert client._transport.backend is None
            server.identity_delay = 0
            assert (await client.connect()).id == server.backend_id

    asyncio.run(scenario())


def case_native_call() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            await client.connect()
            client._transport.timeout = 0.05
            server.catalog_delay = 10
            with pytest.raises(RpcError) as error:
                await collect(client.suites())
            assert error.value.code == "DEADLINE_EXCEEDED"
            assert server.catalog_deadline is not None
            assert 0 < server.catalog_deadline <= 0.1
            await asyncio.wait_for(server.catalog_finished.wait(), 1)
            server.catalog_delay = 0
            assert (await collect(client.suites()))[0].id == SUITE_ID

    asyncio.run(scenario())


def case_native_run() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            await client.connect()
            server.catalog_delay = 10
            pending = asyncio.create_task(collect(client.suites()))
            await server.catalog_started.wait()
            await client.close()
            with pytest.raises(ClientClosed):
                await pending
            await asyncio.wait_for(server.catalog_finished.wait(), 1)
            assert server.stop_calls == 0

    asyncio.run(scenario())


def case_polling_summarizing() -> None:
    poll = runpy.run_path(
        str(Path(__file__).parents[3] / "examples" / "client" / "poll_run.py")
    )["poll"]

    async def scenario() -> None:
        async with fixture() as (server, client):
            run_cycle = await client.new_cycle(suite="suite", template="template")
            (handle,) = run_cycle.executions
            await handle.start(request_key="polling-example")
            summary = await poll(handle, 0.01)
            assert summary["samples_by_stream"] == {"voltage": 2}
            assert summary["events_by_type"] == {
                "fixture.started": 1,
                "fixture.finished": 1,
            }
            assert summary["run"]["result"]["passed"] == 1
            assert any(position.get("voltage") == 1 for position in server.sent_cursors)
            assert server.stop_calls == 0

    asyncio.run(scenario())


def case_execution_terminal() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            server.seal_automatically = False
            cycle = await client.new_cycle(suite="suite", template="template")
            (handle,) = cycle.executions
            await handle.start(request_key="delayed-seal")
            await asyncio.wait_for(server.terminal.wait(), 1)
            with pytest.raises(TimeoutError):
                await handle.wait(timeout=0.03, poll_interval=0.005)
            assert server.stop_calls == 0
            assert await handle.checkpoint() is None
            server.sealed.set()
            outcome = await handle.wait(timeout=1, poll_interval=0.005)
            assert outcome.status == "passed"
            assert outcome.result.passed == 1
            assert outcome.checkpoint._wire() == server.point

    asyncio.run(scenario())


def case_execution_wait() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            cycle = await client.new_cycle(suite="suite", template="template")
            (handle,) = cycle.executions
            entered = asyncio.Event()
            release = asyncio.Event()
            records = []
            samples = []

            async def on_record(item: Any) -> None:
                entered.set()
                await release.wait()
                records.append(item.id)

            async def on_sample(item: Any) -> None:
                samples.append(item.id)

            running = asyncio.create_task(
                handle.run(
                    request_key="callback-drain",
                    on_record=on_record,
                    on_sample=on_sample,
                    poll_interval=0.005,
                )
            )
            try:
                await asyncio.wait_for(entered.wait(), 1)
                outcome = await handle.wait(timeout=1, poll_interval=0.005)
                assert outcome.status == "passed"
                assert not running.done()
                release.set()
                assert (await asyncio.wait_for(running, 1)) == outcome
                assert records == ["event-1", "event-2"]
                assert samples == ["sample-1", "sample-2"]
                assert server.stop_calls == 0
            finally:
                release.set()
                running.cancel()
                await asyncio.gather(running, return_exceptions=True)

    asyncio.run(scenario())


def case_callback_work() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            cycle = await client.new_cycle(suite="suite", template="template")
            (handle,) = cycle.executions

            async def on_record(item: Any) -> None:
                raise RuntimeError("callback failed")

            with pytest.raises(ExceptionGroup) as failed:
                await handle.run(
                    request_key="failed-callback",
                    on_record=on_record,
                    poll_interval=0.005,
                )
            assert any(
                isinstance(error, RuntimeError) for error in failed.value.exceptions
            )
            assert server.stop_calls == 0
            outcome = await handle.wait(timeout=1, poll_interval=0.005)
            assert outcome.status == "passed"
            assert server.stop_calls == 0

    asyncio.run(scenario())


def case_live_values() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            suites = [s async for s in client.suites()]
            assert suites[0].templates[0].id == TEMPLATE_ID
            run_cycle = await client.new_cycle(
                suite="suite",
                template="template",
                parameters={"integer": 2**60, "flag": True, "array": [1, None]},
            )
            run = await run_cycle.run(request_key="lab-order")
            assert server.overrides is not None
            assert server.overrides["integer"].values[0].integer_value == 2**60
            assert server.overrides["flag"].values[0].bool_value is True
            assert (
                server.overrides["array"]
                .values[0]
                .array_value.values[1]
                .WhichOneof("kind")
                == "null_value"
            )
            events, samples, status = await asyncio.gather(
                collect(run_cycle.records(until=run)),
                collect(run_cycle.samples(page_size=1, poll_interval=0.01, until=run)),
                run.wait(poll_interval=0.01),
            )
            assert [r.id for r in events] == ["event-1", "event-2"]
            assert [s.values["voltage"] for s in samples] == [231.0, 232.0]
            assert status.phase is RunPhase.COMPLETED
            summary = await run_cycle.result()
            assert summary.passed == 1
            assert server.stop_calls == 0
            assert any(p.get("voltage") == 2 for p in server.sent_cursors)

    asyncio.run(scenario())


def case_replay_pages() -> None:
    async def scenario() -> None:
        async with fixture() as (_, client):
            run_cycle = await client.new_cycle(suite="suite", template="template")
            run = await run_cycle.run(request_key="lab-order")
            await run.wait(poll_interval=0.01)
            records = await collect(
                run_cycle.records(
                    record_types=["fixture.finished"], record_ids=["event-2"]
                )
            )
            assert [r.id for r in records] == ["event-2"]
            assert (
                await collect(run_cycle.records(record_ids=["missing"], until=run))
                == []
            )
            assert len(await collect(run_cycle.samples(page_size=1))) == 2

    asyncio.run(scenario())


def case_stop_evidence() -> None:
    async def scenario() -> None:
        async with fixture() as (_, client):
            run_cycle = await client.new_cycle(suite="suite", template="template")
            run = await run_cycle.run(request_key="lab-order")
            records_task = asyncio.create_task(collect(run_cycle.records(until=run)))
            control = await client.execution(EXECUTION_ID)
            await control.stop()
            status = await run.wait(poll_interval=0.01)
            assert status.phase is RunPhase.STOPPED
            assert (await records_task)[-1].type == "fixture.stopped"

    asyncio.run(scenario())


def case_wait_run() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            run_cycle = await client.new_cycle(suite="suite", template="template")
            run = await run_cycle.run(request_key="lab-order")
            with pytest.raises(TimeoutError):
                await run.wait(timeout=0.01, poll_interval=0.001)
            await client.close()
            await client.close()
            await asyncio.wait_for(server.terminal.wait(), 1)
            assert server.stop_calls == 0
            with pytest.raises(ClientClosed):
                await run.status()

    asyncio.run(scenario())


def case_ambiguous_cycle() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            server.fail_admission = True
            with pytest.raises(SubmissionUncertain) as caught:
                cycle = await client.new_cycle(suite="suite", template="template")
                await cycle.run(request_key="durable-key")
            assert caught.value.cycle_id == CYCLE_ID
            run_cycle = await client.cycle(caught.value.cycle_id)
            run = await run_cycle.run(request_key=caught.value.request_key)
            recovered = await client.get_run_request(run.id)
            assert recovered.id == run.id
            assert server.created == 1 and server.run_calls == 2
            assert (await run.wait()).phase is RunPhase.COMPLETED

    asyncio.run(scenario())


def case_rejections_explicit() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            server.deny_run = True
            with pytest.raises(RunRejected, match="provider_unavailable"):
                cycle = await client.new_cycle(suite="suite", template="template")
                await cycle.run(request_key="key")
        async with fixture() as (server, client):
            server.methods["Execution"].remove("RunCycle")
            with pytest.raises(RpcError, match="not available"):
                cycle = await client.cycle(CYCLE_ID)
                await cycle.run(request_key="key")
            assert server.run_calls == 0

    asyncio.run(scenario())


def case_restart_receipts() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            run_cycle = await client.new_cycle(suite="suite", template="template")
            run = await run_cycle.run(request_key="key")
            server.epoch = "epoch-2"
            assert (await run.status()).id == run.id
            with pytest.raises(BackendChanged):
                await collect(run_cycle.samples(until=run))
            with pytest.raises(BackendChanged):
                await collect(run_cycle.records(until=run))
            control = await client.execution(EXECUTION_ID)
            with pytest.raises(BackendChanged):
                await control.stop()
            server.backend_id = "different-installation"
            with pytest.raises(BackendChanged):
                await run.status()

    asyncio.run(scenario())


@pytest.mark.parametrize("method", ["RegisterSelection", "CreateCycle", "GetCycle"])
def test_cycle_replay(method: str) -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            if method == "GetCycle":
                cycle_id = await client.create_cycle(suite="suite", template="template")
                server.restart_during = method
                with pytest.raises(BackendChanged):
                    await client.cycle(cycle_id)
            else:
                server.restart_during = method
                with pytest.raises(BackendChanged):
                    await client.create_cycle(suite="suite", template="template")
            assert server.registration_calls == 1
            assert server.created == (0 if method == "RegisterSelection" else 1)
            assert server.run_calls == 0

    asyncio.run(scenario())


@pytest.mark.parametrize("method", ["pause", "resume", "stop"])
def test_execution_checked(method: str) -> None:
    async def scenario() -> None:
        rpc = f"{method.title()}Execution"
        async with fixture() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            await getattr(handle, method)()
            assert server.execution_control_calls == [rpc]
        async with fixture() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            server.restart_during = rpc
            with pytest.raises(BackendChanged):
                await getattr(handle, method)()
            assert server.execution_control_calls == [rpc]
            assert server.run_calls == 0
            assert server.created == 0

    asyncio.run(scenario())


def case_command_replay() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            assert handle.id == EXECUTION_ID
            assert server.command_reads == 0
            server.epoch = "epoch-2"
            receipt = await handle.command_receipt(REQUEST_ID, COMMAND_ID)
            assert receipt.phase is CommandPhase.INTERRUPTED
            assert receipt.terminal
            assert receipt.execution_id == EXECUTION_ID
            assert server.command_reads == 1
            server.command_execution_id = CYCLE_ID
            with pytest.raises(RpcError) as foreign:
                await handle.command_receipt(REQUEST_ID, COMMAND_ID)
            assert foreign.value.code == "DATA_LOSS"
            assert server.command_reads == 2
            server.backend_id = "different-installation"
            with pytest.raises(BackendChanged):
                await handle.command_receipt(REQUEST_ID, COMMAND_ID)
            assert server.command_reads == 2
            assert server.run_calls == 0
            assert server.created == 0

    asyncio.run(scenario())


def case_protocol_validation() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            server.version = "0.5.0"
            with pytest.raises(ProtocolError):
                await client.connect()
        async with fixture() as (server, client):
            cycle = await client.new_cycle(suite="suite", template="template")
            for key in ["", " ", "bad\nkey", "x" * 257]:
                with pytest.raises(ValueError):
                    await cycle.run(request_key=key)
            with pytest.raises(ValueError):
                await client.new_cycle(
                    suite="suite", template="template", parameters={"voltage": 2**64}
                )
            assert server.created == 1
            assert server.registration_calls == 1
            assert server.run_calls == 0

    asyncio.run(scenario())


@pytest.mark.parametrize(
    "endpoint",
    [
        "http://remote.example:8765",
        "ftp://localhost:8765",
        "https://user:pass@host:8765",
        "https://host:8765/path",
        "localhost:8765",
    ],
)
def test_endpoint_refusals(endpoint: str) -> None:
    with pytest.raises(ValueError):
        DokimeClient(endpoint)


def case_completed_backend() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            run_cycle = await client.new_cycle(suite="suite", template="template")
            run = await run_cycle.run(request_key="done")
            await run.wait(poll_interval=0.01)
            control = await client.execution(EXECUTION_ID)
            await control.stop()
            assert server.stop_calls == 0
            with pytest.raises(RpcError) as pause:
                await control.pause()
            assert pause.value.code == "FAILED_PRECONDITION"
            with pytest.raises(RpcError) as resume:
                await control.resume()
            assert resume.value.code == "FAILED_PRECONDITION"

    asyncio.run(scenario())


@asynccontextmanager
async def fixture():
    implementation = BackendFixture()
    server = grpc.aio.server()
    backend_pb2_grpc.add_BackendServicer_to_server(implementation, server)
    catalog_pb2_grpc.add_CatalogServicer_to_server(implementation, server)
    planning_pb2_grpc.add_PlanningServicer_to_server(implementation, server)
    execution_pb2_grpc.add_ExecutionServicer_to_server(implementation, server)
    evidence_pb2_grpc.add_EvidenceServicer_to_server(implementation, server)
    telemetry_pb2_grpc.add_TelemetryServicer_to_server(implementation, server)
    history_pb2_grpc.add_HistoryServicer_to_server(implementation, server)
    port = server.add_insecure_port("127.0.0.1:0")
    await server.start()
    implementation.endpoint = f"http://127.0.0.1:{port}"
    client = DokimeClient(implementation.endpoint, rpc_timeout=1)
    try:
        yield implementation, client
    finally:
        await client.close()
        if implementation.job is not None:
            implementation.job.cancel()
            await asyncio.gather(implementation.job, return_exceptions=True)
        await server.stop(0)


async def collect(iterator):
    return [item async for item in iterator]


def case_shared_scenarios() -> None:
    scenarios = json.loads(
        (
            Path(__file__).resolve().parents[5]
            / "crates/dokime-protocol/proto/fixtures/negotiation.json"
        ).read_text()
    )

    async def scenario() -> None:
        for case in scenarios:
            async with fixture() as (server, client):
                for field in (
                    "health_epoch",
                    "capabilities_epoch",
                    "readiness",
                    "max_request_bytes",
                    "max_response_bytes",
                ):
                    if field in case:
                        setattr(server, field, case[field])
                server.version = case["version"]
                server.package = case.get("package", "dokime.v1")
                server.descriptor = bytes([case.get("descriptor", 0)]) * 32
                if missing := case.get("missing"):
                    server.methods["Backend"].remove(missing.rsplit("/", 1)[1])

                if case["accepted"]:
                    connected = await client.connect()
                    assert connected.api_version == server.version, case["name"]
                elif case.get("error") == "unavailable":
                    with pytest.raises(RpcError) as error:
                        await client.connect()
                    assert error.value.code == "UNAVAILABLE"
                else:
                    with pytest.raises(ProtocolError):
                        await client.connect()
                assert server.created == 0, case["name"]

    asyncio.run(scenario())


def case_values_dates() -> None:
    from dokime.client._wire import encode, decode

    for integer in [-(2**63), 2**63 - 1, 2**63, 2**64 - 1]:
        wire = encode(integer)
        assert decode(value.Value.FromString(wire.SerializeToString())) == integer
    assert encode(2**64 - 1).WhichOneof("kind") == "unsigned_value"
    assert (
        encode(decode(value.Value(unsigned_value=0))).WhichOneof("kind")
        == "unsigned_value"
    )
    for number in [float("nan"), float("inf"), float("-inf")]:
        wire = encode(number)
        result = decode(value.Value.FromString(wire.SerializeToString()))
        assert isinstance(result, float)
        assert math.isnan(result) if math.isnan(number) else result == number
    date = datetime(2026, 9, 28, 12, 13, 14, 123456, tzinfo=UTC)
    assert decode(encode(date)) == date
    precise = value.Value(datetime_value="2026-09-28T12:13:14.123456789Z")
    assert encode(decode(precise)) == precise
    decoded = decode(precise)
    assert isinstance(decoded, datetime)
    assert decoded != date
    assert decoded > date
    assert len({decoded, date}) == 2
    assert encode(decoded.replace(second=15)).datetime_value.startswith(
        "2026-09-28T12:13:15."
    )
    assert encode(decoded + timedelta(seconds=1)).datetime_value.startswith(
        "2026-09-28T12:13:15."
    )


# === Private Functions === #

# === Entry Point === #


def case_execution_mode() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            with pytest.raises(ValueError):
                await handle.start(request_key="")
            assert server.run_calls == 0
            server.fail_admission = True
            with pytest.raises(ExecutionSubmissionUncertain) as caught:
                await handle.start(request_key="execution-key", mode="interactive")
            failure = caught.value
            assert failure.execution_id == EXECUTION_ID
            assert failure.request_key == "execution-key"
            assert failure.mode == "interactive"
            assert failure.__cause__ is not None
            recovered = await handle.start(
                request_key=failure.request_key, mode=failure.mode
            )
            assert recovered.id == REQUEST_ID
            assert server.created == 0
            assert server.run_calls == 2
            assert server.execution_mode == execution.EXECUTION_MODE_INTERACTIVE

    asyncio.run(scenario())


def case_execution_observation() -> None:
    async def scenario() -> None:
        async with fixture() as (server, client):
            handle = await client.execution(EXECUTION_ID)
            server.fail_receipt_read = True
            request = await handle.start(request_key="acknowledged-execution")
            assert request.id == REQUEST_ID
            assert server.receipt_reads == 0
            with pytest.raises(ProtocolError):
                _ = request.cycle_id
            with pytest.raises(RpcError):
                await request.status()
            assert request.id == REQUEST_ID
            await request.status()
            assert request.cycle_id == CYCLE_ID
            assert server.receipt_reads == 2
            assert server.run_calls == 1
            assert server.created == 0

    asyncio.run(scenario())


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_native_attempt, id="native_attempt"),
        pytest.param(case_native_call, id="native_call"),
        pytest.param(case_native_run, id="native_run"),
        pytest.param(case_polling_summarizing, id="polling_summarizing"),
        pytest.param(case_execution_terminal, id="execution_terminal"),
        pytest.param(case_execution_wait, id="execution_wait"),
        pytest.param(case_callback_work, id="callback_work"),
        pytest.param(case_live_values, id="live_values"),
        pytest.param(case_replay_pages, id="replay_pages"),
        pytest.param(case_stop_evidence, id="stop_evidence"),
        pytest.param(case_wait_run, id="wait_run"),
        pytest.param(case_ambiguous_cycle, id="ambiguous_cycle"),
        pytest.param(case_rejections_explicit, id="rejections_explicit"),
        pytest.param(case_restart_receipts, id="restart_receipts"),
        pytest.param(case_command_replay, id="command_replay"),
        pytest.param(case_protocol_validation, id="protocol_validation"),
        pytest.param(case_completed_backend, id="completed_backend"),
        pytest.param(case_shared_scenarios, id="shared_scenarios"),
        pytest.param(case_values_dates, id="values_dates"),
        pytest.param(case_execution_mode, id="execution_mode"),
        pytest.param(case_execution_observation, id="execution_observation"),
    ],
)
def test_contract(scenario: Callable[..., None]) -> None:
    scenario()
