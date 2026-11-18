# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Host-Free Bound Client Compositions"""

# === Imports === #

# Standard Library Imports
import asyncio
import unittest
from collections.abc import AsyncIterator
from datetime import UTC, datetime
from hashlib import sha256

# Third Party Imports
from google.protobuf.message import Message
from grpc import StatusCode
from grpc.aio import ServicerContext, server
from harmos import FailureValue
from harmos._wire.host.v1 import invocations_pb2, operations_pb2
from harmos._wire.host.v1.invocations_pb2_grpc import InvocationsServicer, add_InvocationsServicer_to_server
from harmos._wire.host.v1.operations_pb2_grpc import OperationsServicer, add_OperationsServicer_to_server

# Local Imports
from dokime_sdk._client import Case, Client
from dokime_sdk._connection import connect
from dokime_sdk._errors import SdkError
from dokime_sdk._values import Outcome, PromptAnswer, PromptQuestion, ResultStatus, Termination, decode
from dokime_sdk._wire.host.v1 import cases_pb2, cycles_pb2, evidence_pb2, executions_pb2, files_pb2, prompts_pb2
from dokime_sdk._wire.host.v1.cases_pb2_grpc import CasesServicer, add_CasesServicer_to_server
from dokime_sdk._wire.host.v1.cycles_pb2_grpc import CyclesServicer, add_CyclesServicer_to_server
from dokime_sdk._wire.host.v1.evidence import notes_pb2
from dokime_sdk._wire.host.v1.evidence.notes_pb2_grpc import NotesServicer, add_NotesServicer_to_server
from dokime_sdk._wire.host.v1.evidence_pb2_grpc import EvidenceServicer, add_EvidenceServicer_to_server
from dokime_sdk._wire.host.v1.executions import progress_pb2
from dokime_sdk._wire.host.v1.executions.progress_pb2_grpc import ProgressServicer, add_ProgressServicer_to_server
from dokime_sdk._wire.host.v1.executions_pb2_grpc import ExecutionsServicer, add_ExecutionsServicer_to_server
from dokime_sdk._wire.host.v1.files_pb2_grpc import FilesServicer, add_FilesServicer_to_server
from dokime_sdk._wire.host.v1.prompts import answers_pb2
from dokime_sdk._wire.host.v1.prompts.answers_pb2_grpc import AnswersServicer, add_AnswersServicer_to_server
from dokime_sdk._wire.host.v1.prompts_pb2_grpc import PromptsServicer, add_PromptsServicer_to_server
from dokime_sdk.operations import Operation, Visibility

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ClientHandlesStory(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.changed_binding = False
        self.settled_seal = False
        self.outcome = executions_pb2.ExecutionOutcome(evaluated={"status": "RESULT_STATUS_PASSED", "reason": "Evaluated"})
        self.end_unsealed = False
        owner = self
        self.trace: list[Message] = []
        trace = self.trace
        cycle = cycles_pb2.CycleSummary(cycle_id="cycle", metadata={"label": "Capture"}, phase="CYCLE_PHASE_ACTIVE")
        detail = executions_pb2.ExecutionDetail(
            summary={"execution_id": "execution", "case_id": "case"},
            cycle_id="cycle",
            admitted_suite={"identifier": "suite"},
            admitted_template={"identifier": "suite.template"},
        )
        self.data = b"retained archive"
        data = self.data
        digest = sha256(data).digest()

        mutations: dict[str, cycles_pb2.CyclesCreateRequest] = {}
        lost: set[str] = set()

        class Cycles(CyclesServicer):
            async def Create(self, request: cycles_pb2.CyclesCreateRequest, context: ServicerContext) -> cycles_pb2.CyclesCreateResponse:
                trace.append(request)
                previous = mutations.setdefault(request.request_id, request)
                if previous != request:
                    await context.abort(StatusCode.FAILED_PRECONDITION, "Request Payload Differs")
                if request.metadata.label == "Refuse":
                    failure = FailureValue(
                        classification="CLASSIFICATION_REFUSED",
                        code="sealed",
                        action="Choose an Active Cycle",
                        operation="cycles.create",
                        plugin="source",
                        details={"cycle": "cycle"},
                        diagnostics="retained diagnostics",
                        cause={"classification": "CLASSIFICATION_PERMANENT", "code": "cause", "details": {"device": "1"}},
                    )
                    context.set_trailing_metadata((("grpc-status-details-bin", failure.SerializeToString()),))
                    await context.abort(StatusCode.FAILED_PRECONDITION, "Refused")
                if request.metadata.label == "Lost Reply" and request.request_id not in lost:
                    lost.add(request.request_id)
                    await context.abort(StatusCode.UNAVAILABLE, "Reply Was Lost After Admission")
                return cycles_pb2.CyclesCreateResponse(cycle=cycle)

            async def Get(self, request: cycles_pb2.CyclesGetRequest, context: ServicerContext) -> cycles_pb2.CyclesGetResponse:
                trace.append(request)
                return cycles_pb2.CyclesGetResponse(cycle=cycle, revision=b"cycle-revision")

            async def List(self, request: cycles_pb2.CyclesListRequest, context: ServicerContext) -> cycles_pb2.CyclesListResponse:
                trace.append(request)
                return cycles_pb2.CyclesListResponse(
                    cycles=[cycle],
                    resolved_at={"positions": [{"sequence": 8}]},
                    next_page_token="next" if not request.page.page_token else "",
                )

            async def Seal(self, request: cycles_pb2.CyclesSealRequest, context: ServicerContext) -> cycles_pb2.CyclesSealResponse:
                trace.append(request)
                return cycles_pb2.CyclesSealResponse(seal=b"settled" if owner.settled_seal else b"")

            async def Watch(
                self, request: cycles_pb2.CyclesWatchRequest, context: ServicerContext
            ) -> AsyncIterator[cycles_pb2.CyclesWatchResponse]:
                trace.append(request)
                yield cycles_pb2.CyclesWatchResponse(cycles=[cycle])
                cycle.phase = cycles_pb2.CYCLE_PHASE_SEALED
                cycle.sealed = True
                yield cycles_pb2.CyclesWatchResponse(cycles=[cycle])

        class Cases(CasesServicer):
            async def Create(self, request: cases_pb2.CasesCreateRequest, context: ServicerContext) -> cases_pb2.CasesCreateResponse:
                trace.append(request)
                return cases_pb2.CasesCreateResponse(
                    cases=[{"case_id": "case", "suite_id": "suite", "template_id": "suite.template", "metadata": {"label": "Case"}}]
                )

            async def Get(self, request: cases_pb2.CasesGetRequest, context: ServicerContext) -> cases_pb2.CasesGetResponse:
                trace.append(request)
                return cases_pb2.CasesGetResponse(
                    value={
                        "summary": {"case_id": "case", "suite_id": "suite", "template_id": "suite.template", "metadata": {"label": "Case"}}
                    },
                    resolved_at={"positions": [{"sequence": 9}]},
                )

            async def Parameters(
                self, request: cases_pb2.CasesParametersRequest, context: ServicerContext
            ) -> cases_pb2.CasesParametersResponse:
                trace.append(request)
                return cases_pb2.CasesParametersResponse(parameters=[{"name": "count", "value": {"unsigned_integer": 2**64 - 1}}])

        class Executions(ExecutionsServicer):
            async def Create(
                self, request: executions_pb2.ExecutionsCreateRequest, context: ServicerContext
            ) -> executions_pb2.ExecutionsCreateResponse:
                trace.append(request)
                return executions_pb2.ExecutionsCreateResponse(executions=[detail])

            async def Get(
                self, request: executions_pb2.ExecutionsGetRequest, context: ServicerContext
            ) -> executions_pb2.ExecutionsGetResponse:
                trace.append(request)
                observed = executions_pb2.ExecutionDetail.FromString(detail.SerializeToString())
                if owner.changed_binding:
                    observed.summary.case_id = "foreign"
                return executions_pb2.ExecutionsGetResponse(execution=observed)

            async def Start(
                self, request: executions_pb2.ExecutionsStartRequest, context: ServicerContext
            ) -> executions_pb2.ExecutionsStartResponse:
                trace.append(request)
                return executions_pb2.ExecutionsStartResponse(execution=detail)

            async def Stop(
                self, request: executions_pb2.ExecutionsStopRequest, context: ServicerContext
            ) -> executions_pb2.ExecutionsStopResponse:
                trace.append(request)
                return executions_pb2.ExecutionsStopResponse(execution=detail)

            async def Cancel(
                self, request: executions_pb2.ExecutionsCancelRequest, context: ServicerContext
            ) -> executions_pb2.ExecutionsCancelResponse:
                trace.append(request)
                return executions_pb2.ExecutionsCancelResponse(execution=detail)

            async def Seal(
                self, request: executions_pb2.ExecutionsSealRequest, context: ServicerContext
            ) -> executions_pb2.ExecutionsSealResponse:
                trace.append(request)
                return executions_pb2.ExecutionsSealResponse(outcome=owner.outcome)

            async def List(
                self, request: executions_pb2.ExecutionsListRequest, context: ServicerContext
            ) -> executions_pb2.ExecutionsListResponse:
                trace.append(request)
                return executions_pb2.ExecutionsListResponse(
                    executions=[detail],
                    resolved_at={"positions": [{"sequence": 10}]},
                    next_page_token="next" if not request.page.page_token else "",
                )

        class Progress(ProgressServicer):
            async def Watch(
                self, request: progress_pb2.ProgressWatchRequest, context: ServicerContext
            ) -> AsyncIterator[progress_pb2.ProgressWatchResponse]:
                trace.append(request)
                yield progress_pb2.ProgressWatchResponse(phase="Cleaning Up", value={"completed": 1, "total": 2})
                if owner.end_unsealed:
                    return
                yield progress_pb2.ProgressWatchResponse(
                    phase="Sealed", value={"completed": 2, "total": 2}, sealed=True, outcome=owner.outcome
                )

        class Notes(NotesServicer):
            async def Create(self, request: notes_pb2.NotesCreateRequest, context: ServicerContext) -> notes_pb2.NotesCreateResponse:
                trace.append(request)
                execution = request.owner.owner.execution.execution_id
                cycle = request.owner.owner.execution.cycle_id if execution else request.owner.owner.cycle_id
                return notes_pb2.NotesCreateResponse(
                    note={
                        "note_id": "note",
                        "message": request.value.message,
                        "created_at": {"seconds": 1},
                        "scope": {"cycle_id": cycle, "execution_id": execution},
                    }
                )

        class Evidence(EvidenceServicer):
            async def Archive(
                self, request: evidence_pb2.EvidenceArchiveRequest, context: ServicerContext
            ) -> evidence_pb2.EvidenceArchiveResponse:
                trace.append(request)
                return evidence_pb2.EvidenceArchiveResponse(file={"length": len(data), "sha256": digest})

        class Files(FilesServicer):
            async def Get(self, request: files_pb2.FilesGetRequest, context: ServicerContext) -> files_pb2.FilesGetResponse:
                trace.append(request)
                return files_pb2.FilesGetResponse(chunk_bytes=4)

            async def Read(self, request: files_pb2.FilesReadRequest, context: ServicerContext) -> files_pb2.FilesReadResponse:
                trace.append(request)
                return files_pb2.FilesReadResponse(
                    data=data[request.offset : request.offset + request.max_bytes], total_length=len(data), sha256=digest
                )

        operation = Operation("echo", str, str, Visibility.PUBLIC)._definition()
        self.answered = asyncio.Event()
        self.prompts_released = asyncio.Event()
        answered = self.answered
        released = self.prompts_released

        class Operations(OperationsServicer):
            async def Get(
                self, request: operations_pb2.OperationsGetRequest, context: ServicerContext
            ) -> operations_pb2.OperationsGetResponse:
                trace.append(request)
                return operations_pb2.OperationsGetResponse(
                    operation={"contract": {"fingerprint": operation.fingerprint, "visibility": "VISIBILITY_PUBLIC"}}
                )

        class Invocations(InvocationsServicer):
            async def Create(
                self, request: invocations_pb2.InvocationsCreateRequest, context: ServicerContext
            ) -> invocations_pb2.InvocationsCreateResponse:
                trace.append(request)
                return invocations_pb2.InvocationsCreateResponse(payload=request.payload)

        class Prompts(PromptsServicer):
            async def Watch(
                self, request: prompts_pb2.PromptsWatchRequest, context: ServicerContext
            ) -> AsyncIterator[prompts_pb2.PromptsWatchResponse]:
                trace.append(request)
                prompt = prompts_pb2.PromptView(
                    prompt={
                        "prompt_id": "question",
                        "scope": {"execution_id": "execution"},
                        "definition": {"message": "Continue?", "actions": [{"identifier": "accept", "label": "Yes"}]},
                    }
                )
                try:
                    yield prompts_pb2.PromptsWatchResponse(prompts=[prompt])
                    yield prompts_pb2.PromptsWatchResponse(prompts=[prompt])
                    await asyncio.Event().wait()
                finally:
                    released.set()

        class Answers(AnswersServicer):
            async def Create(
                self, request: answers_pb2.AnswersCreateRequest, context: ServicerContext
            ) -> answers_pb2.AnswersCreateResponse:
                trace.append(request)
                answered.set()
                return answers_pb2.AnswersCreateResponse()

        self.host = server()
        for add, service in (
            (add_CyclesServicer_to_server, Cycles()),
            (add_CasesServicer_to_server, Cases()),
            (add_ExecutionsServicer_to_server, Executions()),
            (add_ProgressServicer_to_server, Progress()),
            (add_NotesServicer_to_server, Notes()),
            (add_EvidenceServicer_to_server, Evidence()),
            (add_FilesServicer_to_server, Files()),
            (add_OperationsServicer_to_server, Operations()),
            (add_InvocationsServicer_to_server, Invocations()),
            (add_PromptsServicer_to_server, Prompts()),
            (add_AnswersServicer_to_server, Answers()),
        ):
            add(service, self.host)
        port = self.host.add_insecure_port("127.0.0.1:0")
        await self.host.start()
        self.addAsyncCleanup(self.host.stop, None)
        self.client = await connect(f"http://127.0.0.1:{port}")
        self.addAsyncCleanup(self.client.close)

    async def test_create_lookup_controls_and_mutation_replay(self) -> None:
        cycle = await self.client.cycle(label="Capture", request_id="cycle-id")
        self.assertEqual(self.trace[-1], cycles_pb2.CyclesCreateRequest(metadata={"label": "Capture"}, request_id="cycle-id"))
        await self.client.cycle(label="Capture", request_id="cycle-id")
        self.assertEqual(self.trace[-2], self.trace[-1])
        cases = await self.client.cases.materialize(
            "suite.template",
            plugin="plugin",
            suite="suite",
            parameters={"count": [2**64 - 1]},
            files={"input": None},
            request_id="cases-id",
        )
        request = self.trace[-1]
        self.assertIsInstance(request, cases_pb2.CasesCreateRequest)
        assert isinstance(request, cases_pb2.CasesCreateRequest)
        self.assertFalse(request.validate_only)
        self.assertEqual(decode(request.plan.selections[0].parameters[0].value.values[0]), 2**64 - 1)
        self.assertEqual(request.plan.selections[0].files[0].value.kind.WhichOneof("kind"), "clear")
        execution = await cycle.execution(cases[0], request_id="execution-id")
        self.assertEqual(
            self.trace[-1],
            executions_pb2.ExecutionsCreateRequest(parent="cycle", executions=[{"case_id": "case"}], request_id="execution-id"),
        )
        await execution.start(request_id="start-id")
        await execution.stop(request_id="stop-id")
        await execution.cancel(request_id="cancel-id")
        self.assertEqual(
            self.trace[-3:],
            [
                executions_pb2.ExecutionsStartRequest(name="execution", mode="EXECUTION_MODE_AUTOMATIC", request_id="start-id"),
                executions_pb2.ExecutionsStopRequest(name="execution", request_id="stop-id"),
                executions_pb2.ExecutionsCancelRequest(name="execution", request_id="cancel-id"),
            ],
        )
        loaded = await self.client.executions.get("execution")
        self.assertEqual((loaded.cycle.identifier, loaded.case.identifier), ("cycle", "case"))
        self.assertEqual(await loaded.case.parameters(), {"count": 2**64 - 1})
        self.assertEqual(self.trace[-1], cases_pb2.CasesParametersRequest(name="case", parent="cycle", at={"positions": [{"sequence": 9}]}))
        await loaded.refresh()
        await cycle.refresh()
        self.assertEqual(cycle.label, "Capture")
        with self.assertRaises(ValueError):
            await cycle.execution(Case._bind(Client._bind(self.client._transport), "foreign", "", "", ""))

    async def test_refresh_refuses_a_changed_execution_binding(self) -> None:
        execution = await self.client.executions.get("execution")
        self.changed_binding = True
        with self.assertRaises(SdkError):
            await execution.refresh()
        self.assertEqual(execution.case.identifier, "case")
        self.assertEqual(execution._detail.summary.case_id, "case")

    async def test_settled_cycle_seal_uses_its_acknowledgement_without_extra_readback(self) -> None:
        cycle = await self.client.cycles.get("cycle")
        self.settled_seal = True
        self.trace.clear()
        await cycle.seal(request_id="settled-seal")
        self.assertEqual(self.trace, [cycles_pb2.CyclesSealRequest(name="cycle", request_id="settled-seal")])
        self.assertEqual(cycle.phase, "Sealed")

    async def test_lost_reply_keeps_the_identity_for_an_unchanged_replay(self) -> None:
        with self.assertRaises(SdkError) as failed:
            await self.client.cycle(label="Lost Reply")
        identity = failed.exception.request_id
        self.assertIsNotNone(identity)
        self.assertEqual(len(self.trace), 1)
        await self.client.cycle(label="Lost Reply", request_id=identity)
        self.assertEqual(self.trace[0], self.trace[1])
        with self.assertRaises(SdkError):
            await self.client.cycle(label="Different Payload", request_id=identity)

    async def test_remote_failures_preserve_classification_causes_details_and_request(self) -> None:
        with self.assertRaises(SdkError) as failed:
            await self.client.cycle(label="Refuse", request_id="refused-id")
        failure = failed.exception.failure
        self.assertEqual(
            (failure.classification, failure.code, failure.operation, failure.plugin), ("Refused", "sealed", "cycles.create", "source")
        )
        self.assertEqual(failure.details, {"cycle": "cycle", "request_id": "refused-id"})
        self.assertIsNotNone(failure.cause)
        assert failure.cause is not None
        self.assertEqual(failure.cause.details, {"device": "1"})

    async def test_list_retains_the_first_cut_and_filters(self) -> None:
        self.assertEqual(len([value async for value in self.client.cycles.list(text="Capture", page_size=1)]), 2)
        first, second = self.trace[-2:]
        self.assertIsInstance(first, cycles_pb2.CyclesListRequest)
        assert isinstance(first, cycles_pb2.CyclesListRequest) and isinstance(second, cycles_pb2.CyclesListRequest)
        self.assertFalse(first.HasField("at"))
        self.assertEqual(second.at.positions[0].sequence, 8)
        self.assertEqual((second.filter, second.selection, second.order), (first.filter, first.selection, first.order))
        self.assertEqual(
            len(
                [
                    value
                    async for value in self.client.executions.list(cycle="cycle", template="suite.template", state="sealed", page_size=1)
                ]
            ),
            2,
        )
        first, second = self.trace[-2:]
        assert isinstance(first, executions_pb2.ExecutionsListRequest) and isinstance(second, executions_pb2.ExecutionsListRequest)
        self.assertEqual(second.at.positions[0].sequence, 10)
        self.assertEqual(second.selection, first.selection)
        self.assertEqual(first.parent, "cycle")

    async def test_notes_seal_and_retained_archives(self) -> None:
        cycle = await self.client.cycles.get("cycle")
        execution = await self.client.executions.get("execution")
        note = await cycle.note("Cycle Note", request_id="cycle-note")
        self.assertEqual((note.cycle, note.execution, note.created_at), ("cycle", None, datetime(1970, 1, 1, 0, 0, 1, tzinfo=UTC)))
        note = await execution.note("Execution Note", request_id="execution-note")
        self.assertEqual((note.cycle, note.execution), ("cycle", "execution"))
        await cycle.seal(request_id="seal-id")
        self.assertEqual(cycle.phase, "Sealed")
        self.assertEqual(await cycle.evidence(), self.data)
        self.assertEqual(await execution.evidence(), self.data)
        archive = [item for item in self.trace if isinstance(item, evidence_pb2.EvidenceArchiveRequest)]
        self.assertEqual([item.subject.WhichOneof("kind") for item in archive], ["cycle", "execution"])

    async def test_public_operation_validates_then_invokes_and_decodes_plain_values(self) -> None:
        operation = Operation("echo", str, str, Visibility.PUBLIC)
        response = await self.client.operation(operation)("value")
        self.assertEqual(response, "value")
        self.assertIsInstance(self.trace[-2], operations_pb2.OperationsGetRequest)
        request = self.trace[-1]
        assert isinstance(request, invocations_pb2.InvocationsCreateRequest)
        self.assertEqual((request.operation, request.fingerprint), ("echo", operation._definition().fingerprint))

    async def test_prompt_handler_answers_once_and_close_releases_observation(self) -> None:
        questions: list[str] = []

        async def answer(question: "PromptQuestion") -> PromptAnswer:
            questions.append(question.identifier)
            return PromptAnswer("Yes")

        self.client.on_prompt(answer)
        await self.answered.wait()
        await self.client.close()
        await self.prompts_released.wait()
        self.assertEqual(questions, ["question"])
        replies = [request for request in self.trace if isinstance(request, answers_pb2.AnswersCreateRequest)]
        self.assertEqual([(reply.parent, reply.action, reply.accepted) for reply in replies], [("question", "accept", True)])
        self.assertTrue(replies[0].request_id)

    async def test_seal_projects_each_stable_outcome_without_readback(self) -> None:
        execution = await self.client.executions.get("execution")
        self.trace.clear()
        self.assertEqual(await execution.seal(request_id="seal-id"), Outcome(ResultStatus.PASSED, "evaluated", "Evaluated"))
        self.assertEqual(self.trace, [executions_pb2.ExecutionsSealRequest(name="execution", request_id="seal-id")])
        self.outcome = executions_pb2.ExecutionOutcome(not_evaluated={"termination": "TERMINATION_CANCELLED", "reason": "Interrupted"})
        self.assertEqual(await execution.seal(), Outcome(ResultStatus.PENDING, "not_evaluated", "Interrupted", Termination.CANCELLED))
        self.outcome = executions_pb2.ExecutionOutcome(
            declared={"status": "RESULT_STATUS_FAILED", "reason": "Imported", "importer": "reader"}
        )
        self.assertEqual(await execution.seal(), Outcome(ResultStatus.FAILED, "declared", "Imported", importer="reader"))
        self.outcome.Clear()
        with self.assertRaises(SdkError) as failed:
            await execution.seal(request_id="missing-outcome")
        self.assertEqual(failed.exception.request_id, "missing-outcome")

    async def test_run_starts_lazily_and_watch_requires_sealed_finality(self) -> None:
        execution = await self.client.executions.get("execution")
        self.trace.clear()
        run = execution.run(request_id="run")
        self.assertEqual(self.trace, [])
        values = [value async for value in run]
        self.assertEqual([(value.phase, value.sealed) for value in values], [("Cleaning Up", False), ("Sealed", True)])
        self.assertEqual(
            self.trace,
            [
                executions_pb2.ExecutionsStartRequest(name="execution", mode="EXECUTION_MODE_AUTOMATIC", request_id="run"),
                progress_pb2.ProgressWatchRequest(name="execution"),
            ],
        )
        self.trace.clear()
        watch = execution.watch()
        await anext(watch)
        await watch.aclose()
        self.assertEqual(self.trace, [progress_pb2.ProgressWatchRequest(name="execution")])
        self.end_unsealed = True
        with self.assertRaises(SdkError):
            [value async for value in execution.watch()]


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
