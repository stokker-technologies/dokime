# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Dokime Execution Operations on the Pinned Harmos Remote Plugin SDK."""

# === Imports === #

# Standard Library Imports
import asyncio
import inspect
import json
import math
import os
from collections.abc import Awaitable, Callable, Mapping, Sequence
from dataclasses import dataclass
from importlib.metadata import distributions, packages_distributions, version
from pathlib import Path
from types import FunctionType
from typing import Any, ClassVar, cast
from urllib.parse import urlsplit
from urllib.request import url2pathname

# Third Party Imports
import harmos
import harmos.plugin
from google.protobuf.duration_pb2 import Duration
from google.protobuf.message import Message
from harmos import VISIBILITY_PROTECTED, Declarations, OperationPolicy
from harmos.plugin import Catalog, Lane, Publisher, Window, observer, operation, publisher, window_observer

# Local Imports
import dokime_sdk.operations
from dokime_sdk._wire.model.v1.prompts_pb2 import PromptDefinitions
from dokime_sdk._wire.model.v1.record_contracts_pb2 import RecordContract, RecordContracts
from dokime_sdk._wire.model.v1.suites_pb2 import Suites

from . import _logs
from ._authoring import Materialization, Observer, Record, Stream, Suite, _Template, bind_suite, build_identifier, build_label
from ._configuration import load
from ._connection import connect
from ._errors import SdkError, wire_failure
from ._generated.host import HostOperation, host_requirement
from ._generated.shared import SecretsDeclarations, Value
from ._payloads import message_type, pack
from ._values import decode
from ._wire.plugin.v1.criteria_pb2 import CriteriaEvaluateRequest, CriteriaEvaluateResponse
from ._wire.plugin.v1.executions_pb2 import (
    ExecutionsCleanupRequest,
    ExecutionsCleanupResponse,
    ExecutionsInitializeRequest,
    ExecutionsInitializeResponse,
)
from ._wire.plugin.v1.steps_pb2 import StepsExecuteRequest, StepsExecuteResponse
from ._wire.plugin.v1.test_cases_pb2 import TestCasesMaterializeRequest, TestCasesMaterializeResponse
from .templates._context import Context

# Type Checking Imports

# === Public Exports === #


# === Constants === #

# === Types === #

# === Classes === #


@dataclass
class _Session:
    instance: object
    case: Materialization
    origin: bytes


class Plugin:
    identifier: str
    label: ClassVar[str | None] = None
    version: ClassVar[str | None] = None
    suites: ClassVar[Sequence[type[Suite]]] = ()
    lanes: ClassVar[Sequence[Lane]] = ()
    observers: ClassVar[Sequence[type[Observer[Any]]]] = ()
    records: ClassVar[Sequence[Record[Any]]] = ()
    operations: ClassVar[Sequence[Callable[..., object]]] = ()
    settings: ClassVar[Sequence[type[dokime_sdk.operations.Settings]]] = ()
    secrets: ClassVar[Sequence[dokime_sdk.operations.Secret]] = ()
    failure: ClassVar[Callable[[Exception], harmos.OperationError | None] | None] = None
    resources: ClassVar[Sequence[type[dokime_sdk.operations.Resource]]] = ()

    def __init__(self) -> None:
        _logs.install()
        self.identifier = build_identifier(type(self).__name__, getattr(type(self), "identifier", None), class_name=True)
        self._label = self.label if self.label is not None else build_label(type(self).__name__)
        self._version = self.version if self.version is not None else _distribution_version(type(self))
        self._suites = tuple(bind_suite(item, self.identifier) for item in tuple(self.suites))
        self._plugin_records = tuple(self.records)
        self._record_identities: dict[int, str] = {}
        for record in self._plugin_records:
            if not isinstance(record, Record) or not record.identifier:
                raise ValueError("Declare Plugin Record: Supply a Record With an Identifier")
            identity = build_identifier(record.identifier, record.identifier, prefix=self.identifier)
            if identity in self._record_identities.values():
                raise ValueError("Declare Plugin Records: Give Every Record a Unique Identifier")
            self._record_identities[id(record)] = identity
        for suite in self._suites:
            for template in suite.templates:
                template._identifiers.update(self._record_identities)
        self._operations = tuple(self.operations)
        plugin_methods: dict[str, object] = {}
        for cls in reversed(type(self).__mro__):
            plugin_methods.update(vars(cls))
        self._secrets = tuple(self.secrets)
        self._resources = tuple(self.resources)
        for resource in self._resources:
            if not isinstance(resource, type) or not issubclass(resource, dokime_sdk.operations.Resource):
                raise TypeError("Build Plugin: Declare Resource Classes in resources")
        if len(self._resources) != len(set(self._resources)):
            raise ValueError("Build Plugin: Declare Each Resource Class Once")
        self._resource_locks = {item: asyncio.Lock() for item in self._resources}
        self._resource_values: dict[type[dokime_sdk.operations.Resource], dokime_sdk.operations.Resource] = {}
        self._resources_closed = False
        self._settings = tuple(self.settings)
        for settings in self._settings:
            if not isinstance(settings, type) or not issubclass(settings, dokime_sdk.operations.Settings):
                raise TypeError("Build Plugin: Declare Settings Classes in settings")
        if len(self._settings) != len(set(self._settings)):
            raise ValueError("Build Plugin: Declare Each Settings Class Once")
        self._settings_values: dict[type[dokime_sdk.operations.Settings], dokime_sdk.operations.Settings] = {}
        self._sessions: dict[tuple[str, str], _Session] = {}
        templates = [template for suite in self._suites for template in suite.templates]
        for name, identifiers in (
            ("suite", [item.identifier for item in self._suites]),
            ("template", [item.identifier for item in templates]),
            ("record", list(self._record_identities.values())),
            ("secret", [item.identifier for item in self._secrets]),
        ):
            if any(not item for item in identifiers) or len(identifiers) != len(set(identifiers)):
                raise ValueError(f"Build Plugin: Supply Unique, Nonempty {name.title()} Identifiers")
        methods: dict[str, Callable[..., Awaitable[Message | None]]] = {}
        operation_identities = []
        for index, function in enumerate(self._operations):
            if not isinstance(function, FunctionType) or not hasattr(function, "__dokime_operation__"):
                raise TypeError("Build Plugin: List Decorated Plain Functions in operations")
            definition = cast(harmos.plugin.Operation, vars(function)["__dokime_operation__"])
            identity = build_identifier(function.__name__, definition.id, prefix=self.identifier)
            operation_identities.append(identity)
            methods[f"external_{index}"] = dokime_sdk.operations._bind_operation(function, self, identity)
        if len(operation_identities) != len(set(operation_identities)):
            raise ValueError("Build Plugin: Give Every Operation a Unique Identifier")
        for index, template in enumerate(templates):
            methods[f"materialize_{index}"] = self._materialize(template)
            if template.definition().HasField("initialize"):
                methods[f"initialize_{index}"] = self._execution(template, "initialize")
            methods[f"cleanup_{index}"] = self._execution(template, "cleanup")
            lifecycle: Sequence[tuple[str, Sequence[Any]]] = (("step", template.steps), ("criterion", template.criteria))
            for kind, definitions in lifecycle:
                for position, definition in enumerate(definitions):
                    methods[f"{kind}_{index}_{position}"] = self._execution(
                        template, kind, definition.handler, getattr(definition, "retries", 0), definition.policy
                    )
        self._streams: dict[int, str] = {}
        requires = tuple(host_requirement(item) for item in HostOperation)
        for index, (name, method) in enumerate(plugin_methods.items()):
            declaration = getattr(method, "__dokime_stream__", None)
            if not isinstance(declaration, Stream):
                continue
            if not isinstance(method, FunctionType):
                raise TypeError("Declare Plugin Stream: Decorate a Plain Method")
            identity = build_identifier(name, declaration.identifier, prefix=self.identifier)
            if identity in self._streams.values():
                raise ValueError("Build Plugin: Give Every Plugin Stream a Unique Identifier")
            self._streams[id(method)] = identity
            self._streams[id(declaration)] = identity
            methods[f"plugin_stream_{index}"] = self._stream(name, method)
        self._observers = tuple(self.observers)
        for index, declaration in enumerate(self._observers):
            if not isinstance(declaration, type) or not issubclass(declaration, Observer):
                raise TypeError("Declare Observers: Supply Observer Classes")
            identity = build_identifier(declaration.__name__, declaration.identifier, prefix=self.identifier, class_name=True)
            if identity in self._streams.values():
                raise ValueError("Declare Observers: Give Each Stream a Unique Identifier")
            self._streams[id(declaration)] = identity
            methods[f"observer_{index}"] = self._observer(declaration, identity)
        for template in templates:
            for declaration in template.uses_streams:
                if isinstance(declaration, type):
                    identity = self._streams.get(id(declaration))
                    if identity is None:
                        raise ValueError("Use Observer: Declare Its Class on the Plugin")
                    template._identifiers[id(declaration)] = identity
        methods["initialize"] = self._lifecycle("initialize")
        methods["finalize"] = self._lifecycle("finalize")
        adapter = type("DokimeOperations", (), methods)()
        contributions: list[Message] = [Suites(suites=[item.definition() for item in self._suites])] if self._suites else []
        if self._plugin_records:
            contributions.append(
                RecordContracts(
                    records=[
                        RecordContract(
                            metadata={"identifier": self._record_identities[id(item)], "label": item.label or item.identifier},
                            message=message_type(item.row).DESCRIPTOR.full_name,
                        )
                        for item in self._plugin_records
                    ],
                )
            )
        prompts = [value for template in templates for value in template._prompts.values()]
        if prompts:
            contributions.append(PromptDefinitions(prompts=prompts))
        if self._secrets:
            contributions.append(SecretsDeclarations(secrets=[item._definition() for item in self._secrets]))
        catalog = Catalog(
            adapter,
            requires=requires,
            sections=contributions,
            settings=[field._definition(name) for item in self._settings for name, field in item._fields().items()],
            messages=[message_type(record.row) for record in self._plugin_records],
        )
        self._plugin = harmos.plugin.Plugin(
            id=self.identifier,
            version=self._version,
            register=catalog.register,
            lanes=tuple(self.lanes),
            failure_mapper=type(self).failure,
        )

        self._plugin.declarations()

    def declarations(self) -> Declarations:
        return self._plugin.declarations()

    async def initialize(self, ctx: dokime_sdk.operations.Context) -> None:
        pass

    async def finalize(self, ctx: dokime_sdk.operations.Context) -> None:
        pass

    async def serve(self) -> None:
        """Run From Launch Credentials, Pairing Configuration or Standalone Defaults."""
        launched = "HARMOS_HOST_ENDPOINT" in os.environ or "HARMOS_LAUNCH_TOKEN" in os.environ
        if launched:
            try:
                await self._plugin.run()
            except asyncio.CancelledError:
                pass
            finally:
                await self._close_resources()
            return
        values = load()
        remote = os.environ.get("DOKIME_REMOTE", values.get("DOKIME_REMOTE", ""))
        client = await connect(remote) if remote else None
        endpoint: harmos.plugin.Endpoint | None = None
        termination: asyncio.Task[bool] | None = None
        try:
            if client is None:
                endpoint = await self._activate()
            else:
                registered = await client.plugins.register(self)
                endpoint = cast(harmos.plugin.Endpoint, registered._endpoint)
            termination = asyncio.create_task(endpoint.server.wait_for_termination())
            await asyncio.shield(termination)
        except asyncio.CancelledError:
            pass
        finally:
            try:
                if client is not None:
                    await client.close()
                elif endpoint is not None:
                    await endpoint.stop()
            finally:
                await self._close_resources()
            if termination is not None:
                await termination

    async def _activate(self, address: str | None = None) -> harmos.plugin.Endpoint:
        if address == "":
            raise ValueError("Start Plugin: Supply a Nonempty Listener Address")
        return await self._plugin.start("127.0.0.1:0" if address is None else address)

    def _materialize(self, template: _Template) -> Callable[..., Awaitable[Message]]:
        async def execute(adapter: object, transport: harmos.plugin.Context, request: TestCasesMaterializeRequest) -> Message:
            try:
                transport.scope()
            except harmos.OperationError as error:
                if error.value.code != "scope_required":
                    raise
            else:
                raise ValueError("Materialize Test Case: Execution Scope Must Be Empty")
            if request.template != template.identifier:
                raise ValueError("Materialize Test Case: Supply the Bound Template Without an Execution Scope")
            return TestCasesMaterializeResponse(test_case=_build(template, request.parameters).message)

        return operation(
            template._lifecycle["materialize"],
            request=TestCasesMaterializeRequest,
            response=TestCasesMaterializeResponse,
            visibility=VISIBILITY_PROTECTED,
        )(execute)

    def _execution(
        self,
        template: _Template,
        kind: str,
        handler: Callable[..., object] | None = None,
        retries: int = 0,
        policy: dokime_sdk.operations.OperationPolicy | None = None,
    ) -> Callable[..., Awaitable[Message]]:
        contracts: dict[str, tuple[type[Message], type[Message]]] = {
            "step": (StepsExecuteRequest, StepsExecuteResponse),
            "criterion": (CriteriaEvaluateRequest, CriteriaEvaluateResponse),
            "initialize": (ExecutionsInitializeRequest, ExecutionsInitializeResponse),
            "cleanup": (ExecutionsCleanupRequest, ExecutionsCleanupResponse),
        }
        request_type, response_type = contracts[kind]

        async def execute(
            adapter: object,
            transport: harmos.plugin.Context,
            request: StepsExecuteRequest | CriteriaEvaluateRequest | ExecutionsInitializeRequest | ExecutionsCleanupRequest,
        ) -> Message:
            if not request.execution or transport.scope() != request.execution:
                raise ValueError(f"{kind.title()} Execution: Execution and Harmos Scope Must Agree")
            origin = request.origin
            if (
                origin.WhichOneof("kind") != "plugin_run"
                or not origin.plugin_run.plugin
                or origin.plugin_run.template != template.identifier
                or origin.plugin_run.test_case != request.test_case
            ):
                raise ValueError(f"{kind.title()} Execution: Origin and Frozen Test Case Must Agree")
            key = (template.identifier, request.execution)
            session = self._sessions.get(key)
            if session is None:
                case = _build(template, request.test_case.parameters)
                if case.message != request.test_case:
                    raise ValueError(f"{kind.title()} Execution: Frozen Test Case Does Not Match Materialization")
                session = _Session(template.instance(), case, origin.SerializeToString(deterministic=True))
                self._sessions[key] = session
            if session.case.message != request.test_case or session.origin != origin.SerializeToString(deterministic=True):
                raise ValueError(f"{kind.title()} Execution: Frozen Definition Changed")
            context = Context(
                transport,
                self,
                {name: decode(value) for name, value in request.test_case.parameters.items()},
                template,
                termination=request.termination if isinstance(request, ExecutionsCleanupRequest) else None,
            )
            function = handler or template._methods.get(kind)
            arguments: dict[str, object] = {}
            if kind in ("step", "criterion"):
                item_identity = getattr(request, kind)
                function, arguments = session.case.bindings[kind, item_identity]
                if function is not handler:
                    raise ValueError(f"{kind.title()} Execution: Item Belongs to Another Operation")
            try:
                result = await _logs.run(context, function, session.instance, context, **arguments) if function is not None else None
                if kind == "criterion":
                    if not isinstance(result, bool):
                        raise TypeError("Evaluate Criterion: Handler Must Return bool")
                    return CriteriaEvaluateResponse(passed=result)
                return response_type()
            except SdkError as error:
                raise harmos.OperationError(wire_failure(error.failure)) from error
            finally:
                if kind == "cleanup":
                    self._sessions.pop(key, None)

        identity = template.resolve(handler) if handler is not None else template._lifecycle[kind]
        wire_policy = policy._definition() if policy is not None else None
        if retries:
            wire_policy = wire_policy or OperationPolicy()
            wire_policy.retries.total_attempts = retries + 1
        return operation(
            identity,
            request=request_type,
            response=response_type,
            visibility=VISIBILITY_PROTECTED,
            policy=wire_policy,
            replay_safe=bool(retries or policy is not None and policy.retries is not None),
        )(execute)

    def _observer(self, declaration: type[Observer[Any]], identity: str) -> Callable[..., Awaitable[Message]]:
        interval_ms = declaration.interval_ms
        if not math.isfinite(interval_ms) or interval_ms <= 0:
            raise ValueError("Declare Observer: Supply a Positive Interval")
        instance: Observer[Any] | None = None

        async def start(transport: harmos.plugin.Context) -> None:
            nonlocal instance
            instance = declaration()

        async def stop(transport: harmos.plugin.Context) -> None:
            nonlocal instance
            closing, instance = instance, None
            if closing is not None:
                await closing.close()

        async def sample(adapter: object, transport: harmos.plugin.Context, *arguments: Window) -> Message:
            if instance is None:
                raise ValueError("Sample Observer: Start Sampling Before Requesting a Window")
            context = dokime_sdk.operations.Context(transport, self)
            result = await _logs.run(context, getattr(instance, "sample"), context, *arguments)
            return pack(declaration.columns or declaration.row, result)

        nanoseconds = round(interval_ms * 1_000_000)
        duration = Duration(seconds=nanoseconds // 1_000_000_000, nanos=nanoseconds % 1_000_000_000)
        options = dict(
            row=message_type(declaration.row), interval=duration,
            interval_setting=declaration.interval_setting, window_setting=declaration.window_setting,
            description=declaration.label or build_label(declaration.__name__), on_start=start, on_stop=stop,
        )
        if declaration.columns is not None:
            return window_observer(identity, columns=declaration.columns, **options)(sample)
        return observer(identity, **options)(sample)

    def _stream(self, name: str, method: FunctionType) -> Callable[..., Awaitable[None]]:
        definition = cast(Stream, vars(method)["__dokime_stream__"])
        identity = self._streams[id(method)]

        async def publish(adapter: object, transport: harmos.plugin.Context, publisher: Publisher) -> None:
            context = dokime_sdk.operations.Context(transport, self)

            async def produce() -> None:
                async for result in getattr(self, name)(context):
                    row = pack(definition.row, result)
                    await _logs.flush()
                    await publisher.publish(row)

            await _logs.run(context, produce)

        return publisher(identity, row=message_type(definition.row), description=definition.label)(publish)

    def _lifecycle(self, name: str) -> Callable[..., Awaitable[None]]:
        async def invoke(adapter: object, transport: harmos.plugin.Context) -> None:
            context = dokime_sdk.operations.Context(transport, self)
            try:
                if name == "initialize":
                    self._resources_closed = False
                    self._settings_values.clear()

                    async def initialize() -> None:
                        try:
                            if self._settings:
                                snapshot = await transport.settings()
                                self._settings_values = {item: item(snapshot.value) for item in self._settings}
                            await self.initialize(context)
                        except BaseException:
                            await self._close_resources()
                            raise

                    await _logs.run(context, initialize)
                else:

                    async def finalize() -> None:
                        try:
                            await self.finalize(context)
                        finally:
                            await self._close_resources()

                    await _logs.run(context, finalize)
            except SdkError as error:
                raise harmos.OperationError(wire_failure(error.failure)) from error
            finally:
                if name == "finalize":
                    self._sessions.clear()

        return invoke

    async def _close_resources(self) -> None:
        self._resources_closed = True
        failures = []
        for lock in self._resource_locks.values():
            async with lock:
                pass
        while self._resource_values:
            _, value = self._resource_values.popitem()
            try:
                await value.close()
            except Exception as error:
                failures.append(error)
        self._settings_values.clear()
        if failures:
            raise ExceptionGroup("Close Plugin Resources: Resolve the Release Failures, Then Restart", failures)


# === Public Functions === #


# === Private Functions === #


def _build(template: _Template, parameters: Mapping[str, Value]) -> Materialization:
    """Materialize From the Host-Resolved Values; the Case Echoes Their Wire Form, Never a Re-Encoding."""
    case = template.build({key: decode(value) for key, value in parameters.items()})
    case.message.ClearField("parameters")
    for key, value in parameters.items():
        case.message.parameters[key].CopyFrom(value)
    return case


def _distribution_version(plugin: type[Plugin]) -> str:
    owners = packages_distributions().get(plugin.__module__.split(".")[0], ())
    if len(owners) == 1:
        return version(owners[0])
    # Editable Wheels May List Only Their Import Hook, Not the Source Package.
    source = Path(inspect.getfile(plugin)).resolve()
    versions = []
    for distribution in distributions():
        direct_url = json.loads(distribution.read_text("direct_url.json") or "{}")
        if direct_url.get("dir_info", {}).get("editable") is not True:
            continue
        url = urlsplit(direct_url["url"])
        if url.scheme != "file":
            continue
        directory = Path(url2pathname(url.path)).resolve()
        if source.is_relative_to(directory):
            versions.append((len(directory.parts), distribution.version))
    if versions:
        depth = max(depth for depth, _ in versions)
        nearest = [value for candidate, value in versions if candidate == depth]
        if len(nearest) == 1:
            return nearest[0]
    raise ValueError("Build Plugin: Install Its Owning Distribution or Declare version on the Plugin Class")


# === Entry Point === #
