# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Bound Client Handles"""

# === Imports === #

# Standard Library Imports
import asyncio
from collections.abc import AsyncGenerator, AsyncIterator, Awaitable, Callable, Mapping, Sequence
from dataclasses import replace
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING, Protocol, cast
from uuid import uuid4

# Third Party Imports
import harmos
from grpc import StatusCode
from grpc.aio import AioRpcError
from harmos.plugin import Endpoint

# Local Imports
from ._authoring import _Template
from ._configuration import load, resolve
from ._errors import Failure, SdkError, call, invalid, project_failure, refused, transport_error
from ._generated import client_types
from ._generated.client import ClientRoot
from ._generated.shared import MapValue
from ._payloads import Payload, message_type, pack, unpack
from ._schemas import FrozenSchemaCache
from ._transfers import download, upload_bytes
from ._values import Artifact, Note, Outcome, Progress, PromptAnswer, PromptQuestion, Scalar, decode, encode
from ._wire.model.v1.evidence_pb2 import ArtifactScope
from ._wire.model.v1.metadata_pb2 import Metadata

# Type Checking Imports
if TYPE_CHECKING:
    from ._authoring import Record, Stream, TestTemplate
    from ._evidence import RecordsQuery, SamplesQuery
    from ._plugin import Plugin
    from .operations import Operation

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class _SealedProgress(Protocol):
    sealed: bool


class Client:
    def __init__(self, transport: ClientRoot) -> None:
        self._transport = transport
        self.plugins = Plugins(self)
        self.cases = Cases(self)
        self.cycles = Cycles(self)
        self.executions = Executions(self)
        self._schemas = FrozenSchemaCache()
        self._prompt_handler: Callable[[PromptQuestion], Awaitable[PromptAnswer]] | None = None
        self._prompt_task: asyncio.Task[None] | None = None
        self._closed = False
        self._close_task: asyncio.Task[None] | None = None

    async def cycle(self, *, label: str, request_id: str | None = None) -> "Cycle":
        identity = _identity(request_id)
        reply = await call(
            "cycles.create", self._transport.cycles.create([], Metadata(label=label), False, "", request_id=identity), identity
        )
        return Cycle(self, reply.cycle)

    def operation[Q: Payload, R: Payload](self, operation: "Operation[Q, R]") -> "BoundOperation[Q, R]":
        return BoundOperation(self, operation)

    def on_prompt(self, handler: Callable[[PromptQuestion], Awaitable[PromptAnswer]] | None) -> None:
        if self._close_task is not None:
            raise ValueError("Observe Prompts: Connect a New Client After Close")
        self._prompt_handler = handler
        if self._prompt_task is None and handler is not None:
            self._prompt_task = asyncio.create_task(self._answer_prompts())

    async def _answer_prompts(self) -> None:
        answered: set[str] = set()
        watch = self._transport.prompts.watch("", True)
        try:
            async for frame in watch:
                if frame.HasField("resync_required"):
                    raise SdkError(project_failure(frame.resync_required))
                for view in frame.prompts:
                    prompt = view.prompt
                    handler = self._prompt_handler
                    if handler is None or prompt.prompt_id in answered or view.HasField("response"):
                        continue
                    question = PromptQuestion(
                        prompt.prompt_id,
                        prompt.definition.message,
                        tuple(action.label for action in prompt.definition.actions),
                        prompt.scope.execution_id,
                    )
                    answer = await handler(question)
                    actions = [action for action in prompt.definition.actions if action.label == answer.choice]
                    if len(actions) != 1:
                        raise ValueError("Answer Prompt: Choose One Supplied Choice Label")
                    identity = _identity(None)
                    await call(
                        "prompts.answers.create",
                        self._transport.prompts.answers.create(
                            prompt.prompt_id, actions[0].identifier, MapValue(), True, request_id=identity
                        ),
                        identity,
                    )
                    answered.add(prompt.prompt_id)
        except AioRpcError as error:
            raise transport_error("prompts.watch", error) from error
        finally:
            await watch.aclose()

    async def close(self) -> None:
        if self._close_task is None:
            self._close_task = asyncio.create_task(self._close())
        await asyncio.shield(self._close_task)

    async def _close(self) -> None:
        failures: list[Failure] = []
        self._prompt_handler = None
        if self._prompt_task is not None:
            self._prompt_task.cancel()
            results = await asyncio.gather(self._prompt_task, return_exceptions=True)
            for result in results:
                if isinstance(result, BaseException) and not isinstance(result, asyncio.CancelledError):
                    failures.append(_cleanup_failure(result))
            self._prompt_task = None
        for entry in list(self.plugins._owned.values()):
            results = await asyncio.gather(entry.unregister(), return_exceptions=True)
            for result in results:
                if isinstance(result, BaseException):
                    failures.append(_cleanup_failure(result))
        results = await asyncio.gather(self._transport.close(), return_exceptions=True)
        for result in results:
            if isinstance(result, BaseException):
                failures.append(_cleanup_failure(result))
        self._closed = True
        if failures:
            cause: Failure | None = None
            for failure in reversed(failures):
                cause = _append_failure(failure, cause)
            raise SdkError(
                Failure("Refused", "cleanup_failed", "Client Cleanup Failed", "Resolve Each Cleanup Failure and Retry Close", cause=cause)
            )


class Plugins:
    def __init__(self, client: Client) -> None:
        self._client = client
        self._owned: dict[str, RegisteredPlugin] = {}
        self._bindings: dict[type, list[tuple[str, str, str]]] = {}
        self._templates: dict[str, _Template] = {}

    async def register(self, plugin: "Plugin", *, address: str | None = None, request_id: str | None = None) -> "RegisteredPlugin":
        if self._client._close_task is not None:
            raise ValueError("Register Plugin: Connect a New Client After Close")
        selected = resolve("DOKIME_PLUGIN_ADDRESS", address, load())
        identity = _identity(request_id)
        endpoint = await plugin._plugin.start(selected)
        entry: RegisteredPlugin | None = None
        try:
            reply = await call(
                "plugins.create",
                self._client._transport.plugins.create(
                    plugin._identifier,
                    client_types.PluginPlacement(remote=client_types.PluginRemote(url=f"http://{endpoint.address}")),
                    client_types.PluginValue(enabled=True),
                    request_id=identity,
                ),
                identity,
            )
            entry = RegisteredPlugin(self._client, reply.entry, endpoint)
            self._owned[entry.identifier] = entry
            entry._check_failure(identity)
            if entry.state != "Ready":
                watch = self._client._transport.plugins.watch(name=entry.identifier)
                try:
                    async for frame in watch:
                        if frame.resync_required:
                            raise invalid("plugins.watch", frame.resync_required)
                        if frame.HasField("snapshot"):
                            entries = frame.snapshot.entries
                        elif frame.HasField("change") and frame.change.HasField("entry"):
                            entries = [frame.change.entry]
                        else:
                            entries = []
                        for observed in entries:
                            if observed.name == entry.identifier:
                                entry._entry.CopyFrom(observed)
                                entry._check_failure(identity)
                                if entry.state == "Ready":
                                    self._bind(plugin, entry.identifier)
                                    return entry
                    raise invalid("plugins.watch", "Plugin Readiness Observation Ended Before Ready")
                except AioRpcError as error:
                    raise transport_error("plugins.watch", error, identity) from error
                finally:
                    await watch.aclose()
            entry._check_failure(identity)
            self._bind(plugin, entry.identifier)
            return entry
        except BaseException as error:
            try:
                if entry is not None:
                    await entry.unregister()
                else:
                    try:
                        observed = await self._client._transport.plugins.get(plugin._identifier)
                    except AioRpcError as lookup:
                        if lookup.code() != StatusCode.NOT_FOUND:
                            error.add_note(f"Read Failed Registration Entry: {lookup}")
                    else:
                        if observed.entry.placement.remote.url == f"http://{endpoint.address}":
                            await self._client._transport.plugins.delete(observed.entry.name, observed.entry.revision)
                    await endpoint.close()
            except (SdkError, AioRpcError, ValueError, OSError, RuntimeError) as cleanup:
                error.add_note(f"Registration Cleanup Failed: {cleanup}")
            raise

    def _bind(self, plugin: "Plugin", identifier: str) -> None:
        for suite in plugin._suites:
            for template in suite.templates:
                self._bindings.setdefault(template.instance, []).append((identifier, suite.identifier, template.identifier or ""))
                self._templates[template.identifier or ""] = template

    async def upload(self, bundle: str | Path | bytes, *, request_id: str | None = None) -> "RegisteredPlugin":
        identity = _identity(request_id)
        raise refused(
            "plugins.upload",
            "The Host Has No Product Managed-Bundle Installation Operation",
            "Publish the Product Plugins/Create Bundle Contract and Regenerate Bindings",
            request_id=identity,
        )

    async def get(self, identifier: str) -> "RegisteredPlugin":
        reply = await call("plugins.get", self._client._transport.plugins.get(identifier))
        return RegisteredPlugin(self._client, reply.entry)


class RegisteredPlugin:
    def __init__(self, client: Client, entry: client_types.PluginEntry, endpoint: Endpoint | None = None) -> None:
        self._client = client
        self._entry = client_types.PluginEntry.FromString(entry.SerializeToString())
        self._endpoint = endpoint
        self._unregister_id = _identity(None)
        self._detached = False

    @property
    def identifier(self) -> str:
        return self._entry.name

    @property
    def label(self) -> str:
        return self._entry.description.id or self.identifier

    @property
    def version(self) -> str:
        return self._entry.description.version

    @property
    def state(self) -> str:
        return client_types.PluginState.Name(self._entry.state).removeprefix("PLUGIN_STATE_").title()

    def _check_failure(self, request_id: str) -> None:
        if self._entry.HasField("failure"):
            raise SdkError(project_failure(self._entry.failure), request_id=request_id)
        if self.state in ("Unavailable", "Stopped", "Stopping"):
            raise refused(
                "plugins.register",
                "The Host Refused Plugin Readiness",
                "Resolve the Plugin Failure and Register Again",
                request_id=request_id,
            )

    async def refresh(self) -> "RegisteredPlugin":
        reply = await call("plugins.get", self._client._transport.plugins.get(self.identifier))
        self._entry.CopyFrom(reply.entry)
        return self

    async def unregister(self) -> None:
        if not self._detached:
            try:
                reply = await self._client._transport.plugins.get(self.identifier)
                await self._client._transport.plugins.delete(self.identifier, reply.entry.revision)
            except AioRpcError as error:
                if error.code() != StatusCode.NOT_FOUND:
                    raise transport_error("plugins.delete", error, self._unregister_id) from error
            self._detached = True
        if self._endpoint is not None:
            await self._endpoint.close()
            self._endpoint = None
        self._client.plugins._owned.pop(self.identifier, None)
        for template, bindings in list(self._client.plugins._bindings.items()):
            self._client.plugins._bindings[template] = [binding for binding in bindings if binding[0] != self.identifier]


class Cases:
    def __init__(self, client: Client) -> None:
        self._client = client

    async def materialize(
        self,
        template: "type[TestTemplate] | str",
        *,
        plugin: str | None = None,
        suite: str | None = None,
        parameters: Mapping[str, Sequence[Scalar]] | None = None,
        files: Mapping[str, str | None] | None = None,
        request_id: str | None = None,
    ) -> list["Case"]:
        if isinstance(template, str):
            if not plugin or not suite:
                raise ValueError("Materialize Cases: Supply Explicit Plugin and Suite Identities for a String Template")
            selected = template
        else:
            bindings = [
                item
                for item in self._client.plugins._bindings.get(template, [])
                if (plugin is None or item[0] == plugin) and (suite is None or item[1] == suite)
            ]
            if len(bindings) != 1:
                raise ValueError("Materialize Cases: Register One Unambiguous Plugin/ Suite Binding for This Template")
            plugin, suite, selected = bindings[0]
        selection = client_types.RegisteredSelection(
            suite_id=suite,
            template_id=selected,
            parameters=[
                {"key": key, "value": {"values": [encode(value) for value in values]}} for key, values in (parameters or {}).items()
            ],
            files=[
                {"key": key, "value": {"kind": {"clear": {}} if value is None else {"value": value}}}
                for key, value in (files or {}).items()
            ],
        )
        identity = _identity(request_id)
        reply = await call(
            "cases.create",
            self._client._transport.cases.create(client_types.SelectionPlan(selections=[selection]), False, request_id=identity),
            identity,
        )
        return [Case(self._client, item.case_id, item.metadata.label, item.suite_id, item.template_id) for item in reply.cases]

    async def get(self, identifier: str, *, cycle: str = "") -> "Case":
        reply = await call("cases.get", self._client._transport.cases.get(identifier, cycle))
        value = reply.value.summary
        return Case(self._client, value.case_id, value.metadata.label, value.suite_id, value.template_id, cycle, reply.resolved_at)


class Case:
    def __init__(
        self, client: Client, identifier: str, label: str, suite: str, template: str, cycle: str = "", at: harmos.ReadCut | None = None
    ) -> None:
        self._client = client
        self.identifier = identifier
        self.label = label
        self.suite = suite
        self.template = template
        self._cycle = cycle
        self._at = at

    async def parameters(self) -> dict[str, Scalar]:
        if self._at is None:
            await self.refresh()
        reply = await call("cases.parameters", self._client._transport.cases.parameters(self.identifier, self._cycle, at=self._at))
        return {item.name: decode(item.value) for item in reply.parameters}

    async def refresh(self) -> "Case":
        observed = await self._client.cases.get(self.identifier, cycle=self._cycle)
        self.label, self.suite, self.template, self._at = observed.label, observed.suite, observed.template, observed._at
        return self


class Cycles:
    def __init__(self, client: Client) -> None:
        self._client = client

    async def get(self, identifier: str) -> "Cycle":
        reply = await call("cycles.get", self._client._transport.cycles.get(identifier))
        return Cycle(self._client, reply.cycle, reply.revision)

    async def list(self, *, text: str = "", page_size: int = 0) -> AsyncIterator["Cycle"]:
        if page_size < 0:
            raise ValueError("List Resources: Supply a Nonnegative Page Size")
        at = None
        cursor = harmos.PageRequest(page_size=page_size)
        tokens: set[str] = set()
        while True:
            reply = await call(
                "cycles.list",
                self._client._transport.cycles.list(
                    client_types.CycleSelection(), client_types.CycleFilter(text=text), client_types.CycleOrder(), at=at, page=cursor
                ),
            )
            if at is None:
                at = reply.resolved_at
            elif at != reply.resolved_at:
                raise invalid("cycles.list", "The Retained Read Cut Changed")
            for cycle in reply.cycles:
                yield Cycle(self._client, cycle)
            if not reply.next_page_token:
                return
            if reply.next_page_token in tokens:
                raise invalid("cycles.list", "The Host Repeated a Page Token")
            tokens.add(reply.next_page_token)
            cursor.page_token = reply.next_page_token


class Executions:
    def __init__(self, client: Client) -> None:
        self._client = client

    async def get(self, identifier: str) -> "Execution":
        reply = await call("executions.get", self._client._transport.executions.get(identifier))
        return Execution(self._client, reply.execution)

    async def list(
        self, *, cycle: str | None = None, template: str | None = None, state: str | None = None, page_size: int = 0
    ) -> AsyncIterator["Execution"]:
        if page_size < 0:
            raise ValueError("List Executions: Supply a Nonnegative Page Size")
        statuses = {
            "open": client_types.EXECUTION_STATE_OPEN,
            "closed": client_types.EXECUTION_STATE_CLOSED,
            "sealed": client_types.EXECUTION_STATE_SEALED,
        }
        if state is not None and state.lower() not in statuses:
            raise ValueError("List Executions: Select Open, Closed or Sealed")
        selection = client_types.ExecutionSelection(
            cycle_ids=[cycle] if cycle else [],
            template_ids=[template] if template else [],
            statuses=[statuses[state.lower()]] if state else [],
        )
        if page_size < 0:
            raise ValueError("List Resources: Supply a Nonnegative Page Size")
        at = None
        cursor = harmos.PageRequest(page_size=page_size)
        tokens: set[str] = set()
        while True:
            reply = await call(
                "executions.list",
                self._client._transport.executions.list(
                    cycle or "", selection, client_types.ExecutionFilter(), client_types.ExecutionOrder(), at=at, page=cursor
                ),
            )
            if at is None:
                at = reply.resolved_at
            elif at != reply.resolved_at:
                raise invalid("executions.list", "The Retained Read Cut Changed")
            for execution in reply.executions:
                yield Execution(self._client, execution)
            if not reply.next_page_token:
                return
            if reply.next_page_token in tokens:
                raise invalid("executions.list", "The Host Repeated a Page Token")
            tokens.add(reply.next_page_token)
            cursor.page_token = reply.next_page_token


class Cycle:
    def __init__(self, client: Client, summary: client_types.CycleSummary, revision: bytes = b"") -> None:
        self._client = client
        self._summary = client_types.CycleSummary.FromString(summary.SerializeToString())
        self._revision = revision

    @property
    def identifier(self) -> str:
        return self._summary.cycle_id

    @property
    def label(self) -> str:
        return self._summary.metadata.label

    @property
    def phase(self) -> str:
        return client_types.CyclePhase.Name(self._summary.phase).removeprefix("CYCLE_PHASE_").replace("_", " ").title()

    async def execution(self, case: Case, *, request_id: str | None = None) -> "Execution":
        if case._client is not self._client:
            raise ValueError("Create Execution: Select a Case from This Client")
        identity = _identity(request_id)
        reply = await call(
            "executions.create",
            self._client._transport.executions.create(
                self.identifier, [client_types.ExecutionBinding(case_id=case.identifier)], request_id=identity
            ),
            identity,
        )
        if len(reply.executions) != 1:
            raise invalid("executions.create", "The Host Must Return Exactly One Bound Execution")
        return Execution(self._client, reply.executions[0], self, case)

    async def refresh(self) -> "Cycle":
        value = await self._client.cycles.get(self.identifier)
        self._summary, self._revision = value._summary, value._revision
        return self

    async def seal(self, *, request_id: str | None = None) -> None:
        identity = _identity(request_id)
        reply = await call("cycles.seal", self._client._transport.cycles.seal(self.identifier, request_id=identity), identity)
        if reply.seal:
            await self.refresh()
            return
        watch = self._client._transport.cycles.watch(
            client_types.CycleSelection(cycle_ids=[self.identifier]), client_types.CycleFilter(), client_types.CycleOrder()
        )
        try:
            async for frame in watch:
                if frame.HasField("resync_required"):
                    raise SdkError(project_failure(frame.resync_required), request_id=identity)
                for summary in frame.cycles:
                    if summary.cycle_id != self.identifier:
                        continue
                    self._summary.CopyFrom(summary)
                    if self.phase == "Sealed":
                        return
                    if self.phase == "Seal Failed":
                        raise refused("cycles.seal", summary.seal_failure, "Resolve the Seal Failure and Retry Seal", request_id=identity)
            raise invalid("cycles.watch", "Cycle Observation Ended Before Seal Settlement")
        except AioRpcError as error:
            raise transport_error("cycles.watch", error, identity) from error
        finally:
            await watch.aclose()

    async def evidence(self) -> bytes:
        return await _archive(self._client, client_types.EvidenceSubject(cycle=self.identifier))

    async def note(self, text: str, *, request_id: str | None = None) -> Note:
        return await _note(self._client, self.identifier, None, text, request_id)

    async def artifact(self, name: str, data: bytes, *, mime: str = "application/octet-stream", request_id: str | None = None) -> Artifact:
        return await _artifact(self._client, self.identifier, None, name, data, mime, request_id)


class Execution:
    def __init__(self, client: Client, detail: client_types.ExecutionDetail, cycle: Cycle | None = None, case: Case | None = None) -> None:
        self._client = client
        self._detail = client_types.ExecutionDetail.FromString(detail.SerializeToString())
        self.cycle = cycle or Cycle(client, client_types.CycleSummary(cycle_id=detail.cycle_id))
        self.case = case or Case(
            client, detail.summary.case_id, "", detail.admitted_suite.identifier, detail.admitted_template.identifier, detail.cycle_id
        )

    @property
    def identifier(self) -> str:
        return self._detail.summary.execution_id

    async def refresh(self) -> "Execution":
        reply = await call("executions.get", self._client._transport.executions.get(self.identifier))
        self._detail.CopyFrom(reply.execution)
        return self

    async def start(self, *, request_id: str | None = None) -> None:
        identity = _identity(request_id)
        reply = await call(
            "executions.start",
            self._client._transport.executions.start(self.identifier, client_types.EXECUTION_MODE_AUTOMATIC, request_id=identity),
            identity,
        )
        self._detail.CopyFrom(reply.execution)

    async def watch(self) -> AsyncGenerator[Progress]:
        # Require the Agreed Host Field; Never Translate the Superseded Terminal Contract.
        if "sealed" not in client_types.ProgressWatchResponse.DESCRIPTOR.fields_by_name:
            raise refused(
                "executions.progress.watch",
                "The Host Progress Contract Does Not Expose Sealed State",
                "Land seal-finality and Regenerate the Progress Bindings",
            )
        watch = self._client._transport.executions.progress.watch(self.identifier)
        try:
            async for frame in watch:
                sealed = cast(_SealedProgress, frame).sealed
                yield Progress(frame.phase, frame.value.completed, frame.value.total, frame.value.label, sealed)
                if sealed:
                    return
            raise invalid("executions.progress.watch", "Execution Observation Ended Without Sealed State")
        except AioRpcError as error:
            raise transport_error("executions.progress.watch", error) from error
        finally:
            await watch.aclose()

    async def run(self, *, request_id: str | None = None) -> AsyncGenerator[Progress]:
        await self.start(request_id=request_id)
        watch = self.watch()
        try:
            async for progress in watch:
                yield progress
        finally:
            await watch.aclose()

    async def stop(self, *, request_id: str | None = None) -> None:
        identity = _identity(request_id)
        reply = await call("executions.stop", self._client._transport.executions.stop(self.identifier, request_id=identity), identity)
        self._detail.CopyFrom(reply.execution)

    async def cancel(self, *, request_id: str | None = None) -> None:
        identity = _identity(request_id)
        reply = await call("executions.cancel", self._client._transport.executions.cancel(self.identifier, request_id=identity), identity)
        self._detail.CopyFrom(reply.execution)

    async def seal(self, *, request_id: str | None = None) -> Outcome:
        raise refused(
            "executions.seal",
            "The Host Has No Executions/Seal Operation",
            "Land seal-finality and Regenerate the Execution Bindings",
            request_id=_identity(request_id),
        )

    async def evidence(self) -> bytes:
        return await _archive(self._client, client_types.EvidenceSubject(execution=self.identifier))

    async def note(self, text: str, *, request_id: str | None = None) -> Note:
        return await _note(self._client, self.cycle.identifier, self.identifier, text, request_id)

    async def artifact(self, name: str, data: bytes, *, mime: str = "application/octet-stream", request_id: str | None = None) -> Artifact:
        return await _artifact(self._client, self.cycle.identifier, self.identifier, name, data, mime, request_id)

    def records[T: Payload](self, declaration: "Record[T] | str", *, step: str = "", row_type: type[T] | None = None) -> "RecordsQuery[T]":
        from ._evidence import RecordsQuery

        return RecordsQuery(self, declaration, step=step, row_type=row_type)

    def samples[T: Payload](
        self,
        stream: "Stream | str",
        *,
        fields: Sequence[str] = (),
        between: tuple[datetime | None, datetime | None] | None = None,
        row_type: type[T] | None = None,
    ) -> "SamplesQuery[T]":
        from ._evidence import SamplesQuery

        return SamplesQuery(self, stream, fields=fields, between=between, row_type=row_type)


class BoundOperation[Q: Payload, R: Payload]:
    def __init__(self, client: Client, operation: "Operation[Q, R]") -> None:
        self._client = client
        self._operation = operation

    async def __call__(self, value: Q) -> R:
        definition = self._operation._definition()
        request = pack(self._operation.request, value)
        observed = await call("operations.get", self._client._transport.operations.get(definition.id))
        if observed.contract.fingerprint != definition.fingerprint or observed.contract.visibility != harmos.VISIBILITY_PUBLIC:
            raise invalid("operations.get", "The Public Operation Contract Differs from the Declared Contract")
        payload = await call(
            "invocations.create",
            self._client._transport.invocations.create(definition.id, definition.fingerprint, request.SerializeToString()),
        )
        return unpack(self._operation.response, message_type(self._operation.response).FromString(payload))


# === Public Functions === #

# === Private Functions === #


def _append_failure(failure: Failure, following: Failure | None) -> Failure:
    cause = _append_failure(failure.cause, following) if failure.cause is not None else following
    return replace(failure, cause=cause)


def _cleanup_failure(error: BaseException) -> Failure:
    if isinstance(error, SdkError):
        return error.failure
    if isinstance(error, AioRpcError):
        return transport_error("client.close", error).failure
    return Failure("Permanent", type(error).__name__, str(error), "Resolve the Cleanup Failure and Close Again", "client.close")


def _identity(request_id: str | None) -> str:
    if request_id == "":
        raise ValueError("Mutate Resource: Supply a Nonempty Request Identity")
    return request_id if request_id is not None else str(uuid4())


async def _archive(client: Client, subject: client_types.EvidenceSubject) -> bytes:
    reply = await call("evidence.archive", client._transport.evidence.archive(subject))
    if not reply.HasField("file"):
        raise invalid("evidence.archive", "The Host Returned No Retained Archive")
    return await download(client._transport, reply.file)


async def _note(client: Client, cycle: str, execution: str | None, text: str, request_id: str | None) -> Note:
    owner = client_types.AnnotationOwner(
        owner=client_types.AnnotationOwnerOwner(execution=client_types.ExecutionNoteOwner(cycle_id=cycle, execution_id=execution))
        if execution
        else client_types.AnnotationOwnerOwner(cycle_id=cycle)
    )
    identity = _identity(request_id)
    reply = await call(
        "evidence.notes.create",
        client._transport.evidence.notes.create(owner, client_types.NoteWritable(message=text), request_id=identity),
        identity,
    )
    value = reply.note
    return Note(
        value.note_id,
        value.message,
        value.author if value.HasField("author") else None,
        value.created_at.ToDatetime(tzinfo=UTC),
        value.scope.cycle_id,
        value.scope.execution_id or None,
    )


async def _artifact(
    client: Client, cycle: str, execution: str | None, name: str, data: bytes, mime: str, request_id: str | None
) -> Artifact:
    identity = _identity(request_id)
    file = await upload_bytes(
        client._transport.files.uploads, data, metadata=harmos.FileMetadata(media_type=mime, expected_length=len(data)), request_id=identity
    )
    reply = await call(
        "content.artifacts.create",
        client._transport.content.artifacts.create(
            file, name, mime, scope=ArtifactScope(cycle_id=cycle, execution_id=execution), request_id=f"{identity}/attach"
        ),
        identity,
    )
    value = reply.artifact
    return Artifact(
        value.artifact_id,
        value.filename,
        value.content_type,
        value.size_bytes,
        bytes.fromhex(value.sha256),
        value.scope.cycle_id or None,
        value.scope.execution_id or None,
    )


# === Entry Point === #
