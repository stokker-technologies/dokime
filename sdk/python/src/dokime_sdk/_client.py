# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Bound Client Handles"""

# === Imports === #

# Standard Library Imports
import asyncio
from collections.abc import AsyncGenerator, AsyncIterator, Awaitable, Callable, Mapping, Sequence
from datetime import UTC, datetime
from importlib.metadata import version
from typing import TYPE_CHECKING, Any
from uuid import NAMESPACE_URL, uuid4, uuid5

# Third Party Imports
import harmos
from google.protobuf import descriptor_pb2, json_format, message_factory
from google.protobuf.descriptor import Descriptor
from grpc import StatusCode
from grpc.aio import AioRpcError
from harmos import CLASSIFICATION_INVALID, CLASSIFICATION_PERMANENT, CLASSIFICATION_REFUSED, OperationError
from harmos._client_transport import unary
from harmos._wire.model.v1.failure_pb2 import Failure
from harmos.plugin import Endpoint

# Local Imports
import dokime_sdk._generated.client

from ._authoring import _Template
from ._evidence import RecordsQuery, SamplesQuery
from ._generated import client_types
from ._generated.shared import MapValue
from ._payloads import Payload
from ._schemas import FrozenSchemaCache, pool
from ._transfers import download, publish_artifact, verify_archive
from ._transport import call, chain, invalid, refused, reopen, transport_error
from ._values import (
    Artifact,
    CatalogTemplate,
    Note,
    Outcome,
    Progress,
    PromptAnswer,
    PromptQuestion,
    ResultStatus,
    Scalar,
    Termination,
    decode,
    encode,
)
from ._wire.model.v1.evidence_pb2 import ArtifactScope
from ._wire.model.v1.metadata_pb2 import Metadata

# Type Checking Imports
if TYPE_CHECKING:
    from ._authoring import Record, Stream, TestTemplate
    from ._plugin import Plugin

# === Public Exports === #

# === Constants === #

# Protobuf JSON Maps These Roots to Bare Values, So Payloads Carry Them as {"value": ...}, Like Plugin Calls.
_SCALAR_MESSAGES = frozenset(
    f"google.protobuf.{name}"
    for name in (
        "DoubleValue",
        "FloatValue",
        "Int64Value",
        "UInt64Value",
        "Int32Value",
        "UInt32Value",
        "BoolValue",
        "StringValue",
        "BytesValue",
        "Timestamp",
        "Duration",
        "FieldMask",
        "Value",
        "ListValue",
    )
)

# === Types === #

# === Classes === #


class Client:
    _transport: dokime_sdk._generated.client.Client
    _endpoint: str
    plugins: "Plugins"
    cases: "Cases"
    cycles: "Cycles"
    executions: "Executions"
    _schemas: FrozenSchemaCache
    _prompt_handler: Callable[[PromptQuestion], Awaitable[PromptAnswer]] | None
    _prompt_task: asyncio.Task[None] | None
    _close_task: asyncio.Task[None] | None
    _closed: bool

    def __init__(self) -> None:
        raise TypeError("Connect a Client with await connect()")

    @classmethod
    def _bind(cls, transport: dokime_sdk._generated.client.Client, endpoint: str = "") -> "Client":
        self = object.__new__(cls)
        self._transport = transport
        self._endpoint = endpoint
        self.plugins = Plugins(self)
        self.cases = Cases(self)
        self.cycles = Cycles(self)
        self.executions = Executions(self)
        self._schemas = FrozenSchemaCache()
        self._prompt_handler = None
        self._prompt_task = None
        self._closed = False
        self._close_task = None

        return self

    async def cycle(self, *, label: str, request_id: str | None = None) -> "Cycle":
        identity = _identity(request_id)
        reply = await call(
            "cycles.create", self._transport.cycles.create([], Metadata(label=label), False, "", request_id=identity), identity
        )
        return Cycle._bind(self, reply.cycle)

    async def operation(self, operation_id: str, payload: dict[str, Any] | None = None) -> dict[str, Any]:
        try:
            discovered = await call(operation_id, self._transport.operations.get(operation_id))
        except OperationError as error:
            if error.value.code != StatusCode.NOT_FOUND.name:
                raise
            failure = refused(
                operation_id, f"{operation_id}: The Host Serves No Such Operation", "Correct the Identifier or Register Its Plugin"
            )
            failure.value.code = "operation_not_found"
            raise failure from error
        contract = discovered.contract
        protobuf = contract.protobuf
        if not contract.HasField("protobuf") or protobuf.HasField("input") or protobuf.HasField("output") or contract.grpc.managed:
            failure = refused(
                operation_id, f"{operation_id}: Only Unary Protobuf Contracts Take a Dict", "Use a Typed Client for This Operation"
            )
            failure.value.code = "typed_client_required"
            raise failure
        types = pool(descriptor_pb2.FileDescriptorSet.FromString(protobuf.descriptors), operation_id)
        request = message_factory.GetMessageClass(types.FindMessageTypeByName(protobuf.request))()
        descriptor = types.FindMessageTypeByName(protobuf.response)
        response = message_factory.GetMessageClass(descriptor)
        value: Any = {} if payload is None else payload
        try:
            if protobuf.request in _SCALAR_MESSAGES:
                if value.keys() != {"value"}:
                    raise json_format.ParseError('Supply the Scalar Request as {"value": ...}')
                value = value["value"]
            json_format.ParseDict(value, request)
        except json_format.ParseError as error:
            raise OperationError(
                Failure(
                    classification=CLASSIFICATION_INVALID,
                    code="invalid_request",
                    message=f"{operation_id}: {error}",
                    action=f"Correct the Request to Match {protobuf.request}",
                    operation=operation_id,
                )
            ) from error
        if contract.HasField("grpc") and discovered.grpc_available:
            path = f"/{contract.grpc.package}.{contract.grpc.service}/{contract.grpc.method}"
            reply = await call(operation_id, unary(self._transport._channel, path, request, response, action=operation_id))
        else:
            encoded = self._transport.invocations.create(operation_id, contract.fingerprint, request.SerializeToString())
            reply = response.FromString(await call(operation_id, encoded))
        result = json_format.MessageToDict(reply, preserving_proto_field_name=True, always_print_fields_with_no_presence=True)
        _complete(descriptor, result)
        return result if isinstance(result, dict) and protobuf.response not in _SCALAR_MESSAGES else {"value": result}

    def on_prompt(self, handler: Callable[[PromptQuestion], Awaitable[PromptAnswer]] | None) -> None:
        if self._close_task is not None:
            raise ValueError("Observe Prompts: Connect a New Client After Close")
        self._prompt_handler = handler
        if self._prompt_task is None and handler is not None:
            self._prompt_task = asyncio.create_task(self._answer_prompts())

    async def _answer_prompts(self) -> None:
        answered: set[str] = set()
        watch = reopen(lambda: self._transport.prompts.watch("", True))
        try:
            async for frame in watch:
                if frame.HasField("resync_required"):
                    raise OperationError(frame.resync_required)
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
            prompt_results = await asyncio.gather(self._prompt_task, return_exceptions=True)
            for prompt_result in prompt_results:
                if isinstance(prompt_result, BaseException) and not isinstance(prompt_result, asyncio.CancelledError):
                    failures.append(_cleanup_failure(prompt_result))
            self._prompt_task = None
        registrations = list(self.plugins._pending)
        for registration in registrations:
            registration.cancel()
        results = await asyncio.gather(*registrations, return_exceptions=True)
        for registration_result in results:
            if isinstance(registration_result, BaseException) and not isinstance(registration_result, asyncio.CancelledError):
                failures.append(_cleanup_failure(registration_result))
        # Registered Entries Outlive Their Process: the Host Shows Them Offline Until the Plugin Registers Again.
        for entry in list(self.plugins._owned.values()):
            release_results = await asyncio.gather(entry._release(), return_exceptions=True)
            for release_result in release_results:
                if isinstance(release_result, BaseException):
                    failures.append(_cleanup_failure(release_result))
        close_results = await asyncio.gather(self._transport.close(), return_exceptions=True)
        for close_result in close_results:
            if isinstance(close_result, BaseException):
                failures.append(_cleanup_failure(close_result))
        self._closed = True
        if failures:
            raise OperationError(
                Failure(
                    classification=CLASSIFICATION_REFUSED,
                    code="cleanup_failed",
                    message="Client Cleanup Failed",
                    action="Resolve Each Cleanup Failure and Retry Close",
                    cause=chain(failures),
                )
            )


class Plugins:
    def __init__(self, client: Client) -> None:
        self._client = client
        self._owned: dict[str, RegisteredPlugin] = {}
        self._bindings: dict[type, list[tuple[str, str, str]]] = {}
        self._templates: dict[str, _Template] = {}
        self._pending: set[asyncio.Task[RegisteredPlugin]] = set()

    async def register(self, plugin: "Plugin", *, address: str | None = None, request_id: str | None = None) -> "RegisteredPlugin":
        pending = asyncio.create_task(self._register(plugin, address=address, request_id=request_id))
        self._pending.add(pending)
        try:
            return await pending
        finally:
            self._pending.discard(pending)

    async def _register(self, plugin: "Plugin", *, address: str | None = None, request_id: str | None = None) -> "RegisteredPlugin":
        if self._client._close_task is not None:
            raise ValueError("Register Plugin: Connect a New Client After Close")
        identity = _identity(request_id)
        endpoint = await plugin._activate(address)
        entry: RegisteredPlugin | None = None
        try:
            reply = await call(
                "plugins.create",
                self._client._transport.plugins.create(
                    plugin.identifier,
                    client_types.host.PluginPlacement(remote=client_types.host.PluginRemote(url=f"http://{endpoint.address}")),
                    client_types.host.PluginValue(enabled=True),
                    request_id=identity,
                    sdk_version=version("dokime-sdk"),
                ),
                identity,
            )
            entry = RegisteredPlugin._bind(self._client, reply.entry, endpoint, label=plugin._label, owner=plugin)
            self._owned[entry.identifier] = entry
            entry._check_failure(identity)
            if entry.state != "Ready":
                watch = self._client._transport.plugins.watch(name=entry.identifier)
                try:
                    async for frame in watch:
                        if frame.resync_required:
                            raise invalid("plugins.watch", frame.resync_required)
                        if frame.HasField("snapshot"):
                            entries = list(frame.snapshot.entries)
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
                    raise invalid("plugins.watch", "Plugin Readiness Observation Ended Before Ready", request_id=identity)
                except AioRpcError as error:
                    raise transport_error("plugins.watch", error, identity) from error
                finally:
                    await watch.aclose()
            entry._check_failure(identity)
            self._bind(plugin, entry.identifier)
            return entry
        except BaseException as error:
            # A Failed Registration Stops Only Its Listener; an Existing Entry Stays for the Next Start.
            try:
                if entry is not None:
                    await entry._release()
                else:
                    await endpoint.stop()
            except (OperationError, AioRpcError, ValueError, OSError, RuntimeError) as cleanup:
                error.add_note(f"Registration Cleanup Failed: {cleanup}")
            raise

    def _bind(self, plugin: "Plugin", identifier: str) -> None:
        for suite in plugin._suites:
            for template in suite.templates:
                self._bindings.setdefault(template.instance, []).append((identifier, suite.identifier, template.identifier or ""))
                self._templates[template.identifier or ""] = template

    async def get(self, identifier: str) -> "RegisteredPlugin":
        reply = await call("plugins.get", self._client._transport.plugins.get(identifier))
        owned = self._owned.get(identifier)
        if owned is not None:
            owned._entry.CopyFrom(reply.entry)
            return owned
        return RegisteredPlugin._bind(self._client, reply.entry)


class RegisteredPlugin:
    _client: Client
    _entry: client_types.host.PluginEntry
    _endpoint: Endpoint | None
    _unregister_id: str
    _detached: bool
    _label: str | None
    _owner: "Plugin | None"

    def __init__(self) -> None:
        raise TypeError("Obtain a Plugin Through client.plugins")

    @classmethod
    def _bind(
        cls,
        client: Client,
        entry: client_types.host.PluginEntry,
        endpoint: Endpoint | None = None,
        *,
        label: str | None = None,
        owner: "Plugin | None" = None,
    ) -> "RegisteredPlugin":
        self = object.__new__(cls)
        self._client = client
        if not entry.name:
            raise invalid("plugins.get", "The Host Returned No Plugin Entry Identity")
        self._entry = client_types.host.PluginEntry.FromString(entry.SerializeToString())
        self._endpoint = endpoint
        self._unregister_id = _identity(None)
        self._detached = False
        self._label = label
        self._owner = owner

        return self

    @property
    def identifier(self) -> str:
        return self._entry.name

    @property
    def label(self) -> str:
        return self._label or self._entry.description.id or self.identifier

    @property
    def version(self) -> str:
        return self._entry.description.version

    @property
    def state(self) -> str:
        return client_types.host.PluginState.Name(self._entry.state).removeprefix("PLUGIN_STATE_").title()

    def _check_failure(self, request_id: str) -> None:
        if self._entry.HasField("failure"):
            failure = OperationError(self._entry.failure)
            if request_id is not None:
                failure.value.details["request_id"] = request_id
            raise failure
        if self.state in ("Unavailable", "Offline", "Stopped", "Stopping"):
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
        await self._release()

    async def _release(self) -> None:
        try:
            if self._endpoint is not None:
                await self._endpoint.stop()
                self._endpoint = None
        finally:
            if self._owner is not None:
                await self._owner._close_resources()
                self._owner = None
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
            if not suite:
                raise ValueError("Materialize Cases: Supply the Suite Identity for a String Template")
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
        return [Case._bind(self._client, item.case_id, item.metadata.label, item.suite_id, item.template_id) for item in reply.cases]

    async def templates(self) -> list[CatalogTemplate]:
        reply = await call("suites.list", self._client._transport.suites.list(""))
        return [
            CatalogTemplate(template.metadata.identifier, template.metadata.label, suite.metadata.identifier, not suite.unavailable)
            for suite in reply.values
            for template in suite.templates
        ]

    async def get(self, identifier: str, *, cycle: str = "") -> "Case":
        reply = await call("cases.get", self._client._transport.cases.get(identifier, cycle))
        value = reply.value.summary
        return Case._bind(self._client, value.case_id, value.metadata.label, value.suite_id, value.template_id, cycle, reply.resolved_at)


class Case:
    _client: Client
    identifier: str
    label: str
    suite: str
    template: str
    _cycle: str
    _at: harmos.ReadCut | None

    def __init__(self) -> None:
        raise TypeError("Obtain a Case Through client.cases")

    @classmethod
    def _bind(
        cls, client: Client, identifier: str, label: str, suite: str, template: str, cycle: str = "", at: harmos.ReadCut | None = None
    ) -> "Case":
        self = object.__new__(cls)
        self._client = client
        self.identifier = identifier
        self.label = label
        self.suite = suite
        self.template = template
        self._cycle = cycle
        self._at = at

        return self

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
        return Cycle._bind(self._client, reply.cycle, reply.revision)

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
                yield Cycle._bind(self._client, cycle)
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
        return Execution._bind(self._client, reply.execution)

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
                yield Execution._bind(self._client, execution)
            if not reply.next_page_token:
                return
            if reply.next_page_token in tokens:
                raise invalid("executions.list", "The Host Repeated a Page Token")
            tokens.add(reply.next_page_token)
            cursor.page_token = reply.next_page_token


class Cycle:
    _client: Client
    _summary: client_types.CycleSummary
    _revision: bytes

    def __init__(self) -> None:
        raise TypeError("Obtain a Cycle Through client.cycle or client.cycles")

    @classmethod
    def _bind(cls, client: Client, summary: client_types.CycleSummary, revision: bytes = b"") -> "Cycle":
        self = object.__new__(cls)
        self._client = client
        if not summary.cycle_id:
            raise invalid("cycles.get", "The Host Returned No Cycle Identity")
        self._summary = client_types.CycleSummary.FromString(summary.SerializeToString())
        self._revision = revision

        return self

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
                self.identifier,
                [
                    client_types.ExecutionBinding(
                        case_id=case.identifier, execution_id=str(uuid5(NAMESPACE_URL, f"dokime/execution/{self.identifier}/{identity}"))
                    )
                ],
                request_id=identity,
            ),
            identity,
        )
        if len(reply.executions) != 1:
            raise invalid("executions.create", "The Host Must Return Exactly One Bound Execution", request_id=identity)
        return Execution._bind(self._client, reply.executions[0], self, case)

    async def refresh(self) -> "Cycle":
        value = await self._client.cycles.get(self.identifier)
        self._summary, self._revision = value._summary, value._revision
        return self

    async def seal(self, *, request_id: str | None = None) -> None:
        identity = _identity(request_id)
        reply = await call("cycles.seal", self._client._transport.cycles.seal(self.identifier, request_id=identity), identity)
        if not reply.HasField("cycle") or reply.cycle.cycle_id != self.identifier:
            raise invalid("cycles.seal", "The Host Returned an Invalid Cycle Admission", request_id=identity)
        self._summary.CopyFrom(reply.cycle)
        if self._summary.sealed:
            return
        watch = reopen(
            lambda: self._client._transport.cycles.watch(
                client_types.CycleSelection(cycle_ids=[self.identifier]),
                client_types.CycleFilter(archive=client_types.ARCHIVE_SCOPE_ALL),
                client_types.CycleOrder(),
            )
        )
        try:
            async for frame in watch:
                if frame.HasField("resync_required"):
                    raise invalid("cycles.seal", frame.resync_required.message, request_id=identity)
                cycle = next((cycle for cycle in frame.cycles if cycle.cycle_id == self.identifier), None)
                if cycle is None:
                    raise invalid("cycles.seal", "The Host Returned No Cycle Completion", request_id=identity)
                self._summary.CopyFrom(cycle)
                if cycle.phase == client_types.CYCLE_PHASE_SEAL_FAILED:
                    raise refused("cycles.seal", cycle.seal_failure, "Resolve the Seal Failure and Retry Seal", request_id=identity)
                if cycle.sealed and cycle.phase == client_types.CYCLE_PHASE_SEALED:
                    return
            raise invalid("cycles.seal", "Cycle Observation Closed Before Seal Settlement", request_id=identity)
        except AioRpcError as error:
            raise transport_error("cycles.seal", error, identity) from error
        finally:
            await watch.aclose()

    async def evidence(self) -> bytes:
        return await _archive(self._client, client_types.EvidenceSubject(cycle=self.identifier))

    async def report(self) -> bytes:
        reply = await call(
            "reports.create",
            self._client._transport.reports.create(client_types.ReportSubject(cycle=self.identifier), ""),
        )
        if not reply.HasField("file"):
            raise invalid("reports.create", "The Host Returned No Retained Cycle Report")
        return await download(self._client._transport, reply.file)

    async def note(self, text: str, *, request_id: str | None = None) -> Note:
        return await _note(self._client, self.identifier, None, text, request_id)

    async def artifact(self, name: str, data: bytes, *, mime: str | None = None, request_id: str | None = None) -> Artifact:
        return await publish_artifact(
            self._client._transport, ArtifactScope(cycle_id=self.identifier), name, data, mime=mime, request_id=request_id
        )


class Execution:
    _client: Client
    _detail: client_types.ExecutionDetail
    cycle: Cycle
    case: Case

    def __init__(self) -> None:
        raise TypeError("Obtain an Execution Through a Cycle or client.executions")

    @classmethod
    def _bind(
        cls, client: Client, detail: client_types.ExecutionDetail, cycle: Cycle | None = None, case: Case | None = None
    ) -> "Execution":
        self = object.__new__(cls)
        self._client = client
        if not detail.summary.execution_id or not detail.cycle_id or not detail.summary.case_id:
            raise invalid("executions.get", "The Host Returned an Incomplete Execution Binding")
        if (cycle is not None and detail.cycle_id != cycle.identifier) or (case is not None and detail.summary.case_id != case.identifier):
            raise invalid("executions.create", "The Host Returned an Execution Outside Its Cycle or Case")
        self._detail = client_types.ExecutionDetail.FromString(detail.SerializeToString())
        self.cycle = cycle or Cycle._bind(client, client_types.CycleSummary(cycle_id=detail.cycle_id))
        self.case = case or Case._bind(
            client, detail.summary.case_id, "", detail.admitted_suite.identifier, detail.admitted_template.identifier, detail.cycle_id
        )

        return self

    @property
    def identifier(self) -> str:
        return self._detail.summary.execution_id

    def _observe(self, detail: client_types.ExecutionDetail, request_id: str | None = None) -> None:
        if (
            detail.summary.execution_id != self.identifier
            or detail.cycle_id != self.cycle.identifier
            or detail.summary.case_id != self.case.identifier
        ):
            raise invalid("executions.get", "The Host Changed the Bound Execution Identity, Cycle or Case", request_id=request_id)
        self._detail.CopyFrom(detail)

    async def refresh(self) -> "Execution":
        reply = await call("executions.get", self._client._transport.executions.get(self.identifier))
        self._observe(reply.execution)
        return self

    async def start(self, *, request_id: str | None = None) -> None:
        identity = _identity(request_id)
        reply = await call(
            "executions.start",
            self._client._transport.executions.start(self.identifier, client_types.EXECUTION_MODE_AUTOMATIC, request_id=identity),
            identity,
        )
        self._observe(reply.execution, identity)

    async def watch(self) -> AsyncGenerator[Progress]:
        watch = reopen(lambda: self._client._transport.executions.progress.watch(self.identifier))
        last: Progress | None = None
        try:
            async for frame in watch:
                progress = Progress(frame.phase, frame.value.completed, frame.value.total, frame.value.label, frame.sealed)
                # A Reopened Watch Replays the Current Progress First.
                if progress != last:
                    last = progress
                    yield progress
                if progress.sealed:
                    return
            raise invalid("executions.progress.watch", "Execution Observation Closed Without Sealed State")
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
        self._observe(reply.execution, identity)

    async def cancel(self, *, request_id: str | None = None) -> None:
        identity = _identity(request_id)
        reply = await call("executions.cancel", self._client._transport.executions.cancel(self.identifier, request_id=identity), identity)
        self._observe(reply.execution, identity)

    async def seal(self, *, request_id: str | None = None) -> Outcome:
        identity = _identity(request_id)
        reply = await call("executions.seal", self._client._transport.executions.seal(self.identifier, request_id=identity), identity)
        value = reply.outcome
        attribution = value.WhichOneof("attribution")
        try:
            if attribution == "not_evaluated":
                return Outcome(ResultStatus.PENDING, attribution, value.not_evaluated.reason, Termination(value.not_evaluated.termination))
            if attribution == "evaluated":
                return Outcome(ResultStatus(value.evaluated.status), attribution, value.evaluated.reason)
            if attribution == "declared":
                return Outcome(ResultStatus(value.declared.status), attribution, value.declared.reason, importer=value.declared.importer)
        except ValueError as error:
            raise invalid("executions.seal", "The Host Returned an Unknown Result Status or Termination", request_id=identity) from error
        raise invalid("executions.seal", "The Host Returned No Stable Outcome", request_id=identity)

    async def evidence(self) -> bytes:
        return await _archive(self._client, client_types.EvidenceSubject(execution=self.identifier))

    async def note(self, text: str, *, request_id: str | None = None) -> Note:
        return await _note(self._client, self.cycle.identifier, self.identifier, text, request_id)

    async def artifact(self, name: str, data: bytes, *, mime: str | None = None, request_id: str | None = None) -> Artifact:
        return await publish_artifact(
            self._client._transport, ArtifactScope(execution_id=self.identifier), name, data, mime=mime, request_id=request_id
        )

    def records[T: Payload](self, declaration: "Record[T] | str", *, step: str = "", row_type: type[T] | None = None) -> "RecordsQuery[T]":
        return RecordsQuery._for_execution(self, declaration, step=step, row_type=row_type)

    def samples[T: Payload](
        self,
        stream: "Stream | str",
        *,
        fields: Sequence[str] = (),
        between: tuple[datetime | None, datetime | None] | None = None,
        row_type: type[T] | None = None,
    ) -> "SamplesQuery[T]":
        return SamplesQuery(self, stream, fields=fields, between=between, row_type=row_type)


# === Public Functions === #

# === Private Functions === #


def _complete(descriptor: Descriptor, value: object) -> None:
    # Unset Fields Read as null, Matching the Host's JSON for Plugin Calls.
    if descriptor.full_name.startswith("google.protobuf.") or not isinstance(value, dict):
        return
    for field in descriptor.fields:
        item = value.setdefault(field.name, None)
        nested = field.message_type
        if nested is None:
            continue
        if nested.GetOptions().map_entry:
            mapped = nested.fields_by_name["value"].message_type
            if mapped is not None:
                for entry in (item or {}).values():
                    _complete(mapped, entry)
        elif field.is_repeated:
            for entry in item or ():
                _complete(nested, entry)
        else:
            _complete(nested, item)


def _cleanup_failure(error: BaseException) -> Failure:
    if isinstance(error, OperationError):
        return error.value
    if isinstance(error, AioRpcError):
        return transport_error("client.close", error).value
    return Failure(
        classification=CLASSIFICATION_PERMANENT,
        code=type(error).__name__,
        message=str(error),
        action="Resolve the Cleanup Failure and Close Again",
        operation="client.close",
    )


def _identity(request_id: str | None) -> str:
    if request_id == "":
        raise ValueError("Mutate Resource: Supply a Nonempty Request Identity")
    return request_id if request_id is not None else str(uuid4())


async def _archive(client: Client, subject: client_types.EvidenceSubject) -> bytes:
    reply = await call("evidence.archive", client._transport.evidence.archive(subject))
    if not reply.HasField("file"):
        raise invalid("evidence.archive", "The Host Returned No Retained Archive")
    data = await download(client._transport, reply.file)
    return verify_archive(data)


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


# === Entry Point === #
