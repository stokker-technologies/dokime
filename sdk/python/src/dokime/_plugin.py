# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Dokime Execution Operations On The Pinned Harmos Remote Plugin SDK."""

# === Imports === #

# Standard Library Imports
import asyncio
import inspect
from collections.abc import Awaitable, Callable, Sequence
from dataclasses import dataclass
from types import FunctionType
from typing import Any, cast

# Third Party Imports
import harmos
from google.protobuf.duration_pb2 import Duration
from google.protobuf.message import Message
from harmos import VISIBILITY_PROTECTED, Declarations, OperationPolicy, RetryPolicy, SettingDeclaration
from harmos.plugin import Context as InvocationContext
from harmos.plugin import Endpoint as LibraryEndpoint
from harmos.plugin import Operation, Publisher, observer, on_stop, operation, publisher
from harmos.plugin import Plugin as LibraryPlugin

from dokime._wire.model.v1.record_contracts_pb2 import RecordContract, RecordContracts
from dokime._wire.model.v1.suites_pb2 import Suites

from ._context import Context
from ._generated.host import HostOperation, host_requirement
from ._generated.shared import SecretDeclaration, SecretsDeclarations
from ._wire.plugin.v1.criteria_pb2 import CriteriaEvaluateRequest, CriteriaEvaluateResponse
from ._wire.plugin.v1.executions_pb2 import (
    ExecutionsCleanupRequest,
    ExecutionsCleanupResponse,
    ExecutionsInitializeRequest,
    ExecutionsInitializeResponse,
)
from ._wire.plugin.v1.steps_pb2 import StepsExecuteRequest, StepsExecuteResponse
from ._wire.plugin.v1.test_cases_pb2 import TestCasesMaterializeRequest, TestCasesMaterializeResponse

# Local Imports
from .authoring import _SUITES, Case, Record, Stream, Suite, Template
from .values import decode

# Type Checking Imports

# === Public Exports === #

__all__ = ["Endpoint", "Plugin"]

# === Constants === #

# === Types === #

# === Classes === #


@dataclass
class _Session:
    instance: object
    case: Case
    origin: bytes


class Plugin:
    def __init__(
        self,
        identifier: str,
        version: str,
        *,
        suites: Sequence[Suite | type] = (),
        records: Sequence[Record] = (),
        requires: Sequence[Operation[Any, Any] | HostOperation] | None = None,
        sections: Sequence[Message] = (),
        messages: Sequence[type[Message]] = (),
        operations: object | None = None,
        secrets: Sequence[SecretDeclaration] = (),
        settings: Sequence[SettingDeclaration] = (),
    ) -> None:
        self.identifier = identifier
        self.version = version
        self.suites = tuple(_suite(item) for item in suites)
        self.records = tuple(records)
        self.requires = (
            tuple(host_requirement(item) for item in HostOperation)
            if requires is None
            else tuple(host_requirement(item) if isinstance(item, HostOperation) else item for item in requires)
        )
        self.secrets = tuple(secrets)
        self._sessions: dict[tuple[str, str], _Session] = {}
        self._ready: dict[tuple[str, str], asyncio.Event] = {}
        templates = [template for suite in self.suites for template in suite.templates]
        for name, identifiers in (
            ("suite", [item.identifier for item in self.suites]),
            ("template", [item.identifier for item in templates]),
            ("record", [item.identifier for item in self.records]),
            ("secret", [item.metadata.identifier for item in self.secrets]),
        ):
            if any(not item for item in identifiers) or len(identifiers) != len(set(identifiers)):
                raise ValueError(f"Build Plugin: Duplicate or Empty {name} Identifier")
        methods = {}
        if operations is not None:
            for cls in reversed(type(operations).__mro__):
                for name, method in vars(cls).items():
                    if any(
                        hasattr(method, attribute)
                        for attribute in ("__harmos_operation__", "__harmos_observer__", "__harmos_publisher__", "__harmos_stop__")
                    ):
                        methods[f"external_{name}"] = getattr(operations, name)
        for index, template in enumerate(templates):
            methods[f"materialize_{index}"] = self._materialize(template)
            if "initialize" in template._methods:
                methods[f"initialize_{index}"] = self._execution(template, "initialize")
            methods[f"cleanup_{index}"] = self._execution(template, "cleanup")
            lifecycle: Sequence[tuple[str, Sequence[Any]]] = (("step", template.steps), ("criterion", template.criteria))
            for kind, definitions in lifecycle:
                for position, definition in enumerate(definitions):
                    methods[f"{kind}_{index}_{position}"] = self._execution(
                        template, kind, definition.handler, getattr(definition, "retries", 0)
                    )
            for position, (name, method) in enumerate(template._methods.items()):
                if hasattr(method, "__dokime_stream__"):
                    methods[f"stream_{index}_{position}"] = self._stream(template, name, method)
        methods["release"] = self._stop()
        adapter = type("DokimeOperations", (), methods)()
        contributions: list[Message] = [Suites(suites=[item.definition() for item in self.suites])] if self.suites else []
        if records:
            contributions.append(
                RecordContracts(
                    records=[
                        RecordContract(
                            metadata={"identifier": item.identifier, "label": item.label or item.identifier},
                            message=item.row.DESCRIPTOR.full_name,
                        )
                        for item in records
                    ],
                )
            )
        if self.secrets:
            contributions.append(SecretsDeclarations(secrets=self.secrets))
        self._plugin = LibraryPlugin(
            adapter,
            id=identifier,
            version=version,
            requires=self.requires,
            sections=[*sections, *contributions],
            settings=settings,
            messages=[*messages, *(record.row for record in self.records)],
        )

    def declarations(self) -> Declarations:
        return self._plugin.declarations()

    async def start(self, address: str = "127.0.0.1:7001") -> "Endpoint":
        # Harmos Owns Transport, Fingerprints, Cancellation And Stream Backpressure.
        endpoint = await self._plugin.start(address)
        return Endpoint(endpoint, self)

    def _materialize(self, template: Template) -> Callable[..., Awaitable[Message]]:
        async def execute(adapter: object, transport: InvocationContext, request: TestCasesMaterializeRequest) -> Message:
            try:
                transport.scope()
            except harmos.Failure as error:
                if error.value.code != "scope_required":
                    raise
            else:
                raise ValueError("Materialize Test Case: Execution Scope Must Be Empty")
            if request.template != template.identifier:
                raise ValueError("Materialize Test Case: Supply the Bound Template Without an Execution Scope")
            return TestCasesMaterializeResponse(
                test_case=template.build({key: decode(value) for key, value in request.parameters.items()}).message,
            )

        return operation(
            f"{template.identifier}.materialize",
            request=TestCasesMaterializeRequest,
            response=TestCasesMaterializeResponse,
            visibility=VISIBILITY_PROTECTED,
        )(execute)

    def _execution(
        self, template: Template, kind: str, handler: Callable[..., object] | None = None, retries: int = 0
    ) -> Callable[..., Awaitable[Message]]:
        request_type, response_type = {
            "step": (StepsExecuteRequest, StepsExecuteResponse),
            "criterion": (CriteriaEvaluateRequest, CriteriaEvaluateResponse),
            "initialize": (ExecutionsInitializeRequest, ExecutionsInitializeResponse),
            "cleanup": (ExecutionsCleanupRequest, ExecutionsCleanupResponse),
        }[kind]

        async def execute(
            adapter: object,
            transport: InvocationContext,
            request: StepsExecuteRequest | CriteriaEvaluateRequest | ExecutionsInitializeRequest | ExecutionsCleanupRequest,
        ) -> Message:
            if not request.execution or transport.scope() != request.execution:
                raise ValueError(f"{kind.title()} Execution: Execution and Harmos Scope Must Agree")
            origin = request.origin
            if (
                origin.WhichOneof("kind") != "plugin_run"
                or origin.plugin_run.plugin != self.identifier
                or origin.plugin_run.template != template.identifier
                or origin.plugin_run.test_case != request.test_case
            ):
                raise ValueError(f"{kind.title()} Execution: Origin and Frozen Test Case Must Agree")
            key = (template.identifier, request.execution)
            session = self._sessions.get(key)
            if session is None:
                parameters = {name: decode(value) for name, value in request.test_case.parameters.items()}
                case = template.build(parameters)
                if case.message != request.test_case:
                    raise ValueError(f"{kind.title()} Execution: Frozen Test Case Does Not Match Materialization")
                session = _Session(template.instance(), case, origin.SerializeToString(deterministic=True))
                self._sessions[key] = session
                self._ready.setdefault(key, asyncio.Event()).set()
            if session.case.message != request.test_case or session.origin != origin.SerializeToString(deterministic=True):
                raise ValueError(f"{kind.title()} Execution: Frozen Definition Changed")
            context = Context(
                transport,
                {name: decode(value) for name, value in request.test_case.parameters.items()},
                self.requires,
                self.records,
                termination=request.termination if isinstance(request, ExecutionsCleanupRequest) else None,
                secrets=[item.metadata.identifier for item in self.secrets],
            )
            function = handler or template._methods.get(kind)
            arguments: dict[str, object] = {}
            if kind in ("step", "criterion"):
                identity = getattr(request, kind)
                function, arguments = session.case.bindings[kind, identity]
                if function is not handler:
                    raise ValueError(f"{kind.title()} Execution: Item Belongs to Another Operation")
            try:
                result = function(session.instance, context, **arguments) if function is not None else None
                if inspect.isawaitable(result):
                    result = await result
                if kind == "criterion":
                    if not isinstance(result, bool):
                        raise TypeError("Evaluate Criterion: Handler Must Return bool")
                    return CriteriaEvaluateResponse(passed=result)
                return response_type()
            finally:
                if kind == "cleanup":
                    self._sessions.pop(key, None)
                    self._ready.pop(key, None)

        identity = template.operation(handler, kind) if handler is not None else f"{template.identifier}.{kind}"
        policy = OperationPolicy(retries=RetryPolicy(total_attempts=retries + 1, initial={"nanos": 100_000_000})) if retries else None
        return operation(
            identity,
            request=request_type,
            response=response_type,
            visibility=VISIBILITY_PROTECTED,
            policy=policy,
            replay_safe=bool(retries),
        )(execute)

    def _stream(
        self, template: Template, name: str, method: FunctionType
    ) -> Callable[..., Awaitable[Message]] | Callable[..., Awaitable[None]]:
        definition = cast(Stream, vars(method)["__dokime_stream__"])
        interval = getattr(method, "__dokime_observer__", None)

        async def observe(adapter: object, transport: InvocationContext) -> Message:
            key = (template.identifier, transport.scope())
            await self._ready.setdefault(key, asyncio.Event()).wait()
            session = self._sessions[key]
            context = Context(
                transport, {key: decode(value) for key, value in session.case.message.parameters.items()}, self.requires, self.records
            )
            result = getattr(session.instance, name)(context)
            if inspect.isawaitable(result):
                result = await result
            return result if isinstance(result, definition.row) else definition.row(value=result)

        async def publish(adapter: object, transport: InvocationContext, publisher: Publisher) -> None:
            key = (template.identifier, transport.scope())
            await self._ready.setdefault(key, asyncio.Event()).wait()
            session = self._sessions[key]
            context = Context(
                transport, {key: decode(value) for key, value in session.case.message.parameters.items()}, self.requires, self.records
            )
            async for result in getattr(session.instance, name)(context):
                row = result if isinstance(result, definition.row) else definition.row(value=result)
                await publisher.publish(row)

        if interval:
            milliseconds, setting = interval
            duration = Duration(seconds=milliseconds // 1000, nanos=(milliseconds % 1000) * 1_000_000)
            return observer(
                definition.identifier, row=definition.row, interval=duration, interval_setting=setting, description=definition.label
            )(observe)
        return publisher(definition.identifier, row=definition.row, description=definition.label)(publish)

    def _stop(self) -> Callable[..., Awaitable[None]]:
        async def release(adapter: object) -> None:
            self._sessions.clear()
            self._ready.clear()

        return on_stop(release)


class Endpoint:
    def __init__(self, transport: LibraryEndpoint, plugin: Plugin) -> None:
        self._transport = transport
        self.plugin = plugin
        self.url = f"http://{transport.address}"
        self.address = transport.address

    async def close(self) -> None:
        await self._transport.close()
        self.plugin._sessions.clear()
        self.plugin._ready.clear()


# === Public Functions === #


# === Private Functions === #


def _suite(value: Suite | type) -> Suite:
    if isinstance(value, Suite):
        return value
    registered = _SUITES.get(value) if isinstance(value, type) else None
    if not isinstance(registered, Suite):
        raise TypeError("Build Plugin: Supply a Suite or a Class Decorated with suite")
    return registered


# === Entry Point === #
