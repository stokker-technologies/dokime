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
import harmos.plugin
from google.protobuf.duration_pb2 import Duration
from google.protobuf.message import Message
from harmos import VISIBILITY_PROTECTED, Declarations, OperationPolicy, RetryPolicy, SettingDeclaration
from harmos.plugin import Operation, Publisher, Window, observer, on_stop, operation, publisher, window_observer

import dokime.operations
from dokime._wire.model.v1.prompts_pb2 import PromptDefinitions
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
from .authoring import _SUITES, Case, Sampling, Stream, Suite, Template, build_identifier
from .payloads import message_type
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
        requires: Sequence[Operation[Any, Any] | dokime.operations.Operation[Any, Any] | HostOperation] | None = None,
        messages: Sequence[type[Message]] = (),
        operations: object | None = None,
        secrets: Sequence[SecretDeclaration] = (),
        settings: Sequence[SettingDeclaration] = (),
    ) -> None:
        self.identifier = identifier
        self.version = version
        self.suites = tuple(_suite(item) for item in suites)
        self.records = tuple((template, record) for suite in self.suites for template in suite.templates for record in template.records)
        self.requires = (
            tuple(host_requirement(item) for item in HostOperation)
            if requires is None
            else tuple(
                host_requirement(item)
                if isinstance(item, HostOperation)
                else item.wire()
                if isinstance(item, dokime.operations.Operation)
                else item
                for item in requires
            )
        )
        self.secrets = tuple(secrets)
        self._sessions: dict[tuple[str, str], _Session] = {}
        self._ready: dict[tuple[str, str], asyncio.Event] = {}
        templates = [template for suite in self.suites for template in suite.templates]
        for name, identifiers in (
            ("suite", [item.identifier for item in self.suites]),
            ("template", [item.identifier for item in templates]),
            ("record", [template.resolve(item) for template, item in self.records]),
            ("secret", [item.metadata.identifier for item in self.secrets]),
        ):
            if any(not item for item in identifiers) or len(identifiers) != len(set(identifiers)):
                raise ValueError(f"Build Plugin: Duplicate or Empty {name} Identifier")
        methods = {}
        if operations is not None:
            for cls in reversed(type(operations).__mro__):
                for name, method in vars(cls).items():
                    if hasattr(method, "__dokime_operation__"):
                        methods[f"external_{name}"] = dokime.operations.bind_operation(getattr(operations, name))
                        continue
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
        if self.records:
            contributions.append(
                RecordContracts(
                    records=[
                        RecordContract(
                            metadata={"identifier": template.resolve(item), "label": item.label or template.resolve(item)},
                            message=message_type(item.row).DESCRIPTOR.full_name,
                        )
                        for template, item in self.records
                    ],
                )
            )
        prompts = [value for template in templates for value in template._prompts.values()]
        if prompts:
            contributions.append(PromptDefinitions(prompts=prompts))
        if self.secrets:
            contributions.append(SecretsDeclarations(secrets=self.secrets))
        self._plugin = harmos.plugin.Plugin(
            adapter,
            id=identifier,
            version=version,
            requires=self.requires,
            sections=contributions,
            settings=settings,
            messages=[*messages, *(message_type(record.row) for _, record in self.records)],
        )

        self._plugin.declarations()

    def declarations(self) -> Declarations:
        return self._plugin.declarations()

    async def start(self, address: str = "127.0.0.1:7001") -> "Endpoint":
        # Harmos Owns Transport, Fingerprints, Cancellation And Stream Backpressure.
        endpoint = await self._plugin.start(address)
        return Endpoint(endpoint, self)

    def _materialize(self, template: Template) -> Callable[..., Awaitable[Message]]:
        async def execute(adapter: object, transport: harmos.plugin.Context, request: TestCasesMaterializeRequest) -> Message:
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
            template._lifecycle["materialize"],
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
                template,
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

        identity = template.resolve(handler) if handler is not None else template._lifecycle[kind]
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
        identity = template.resolve(method)
        sampling = cast(Sampling | None, getattr(method, "__dokime_observer__", None))

        async def observe(adapter: object, transport: harmos.plugin.Context) -> Message:
            key = (cast(str, template.identifier), transport.scope())
            await self._ready.setdefault(key, asyncio.Event()).wait()
            session = self._sessions[key]
            context = Context(transport, {key: decode(value) for key, value in session.case.message.parameters.items()}, template)
            result = getattr(session.instance, name)(context)
            if inspect.isawaitable(result):
                result = await result
            return result if isinstance(result, definition.row) else definition.row(value=result)

        async def observe_window(adapter: object, transport: harmos.plugin.Context, window: Window) -> Message:
            key = (cast(str, template.identifier), transport.scope())
            await self._ready.setdefault(key, asyncio.Event()).wait()
            session = self._sessions[key]
            context = Context(transport, {key: decode(value) for key, value in session.case.message.parameters.items()}, template)
            result = getattr(session.instance, name)(context, window)
            if inspect.isawaitable(result):
                result = await result
            columns = cast(type[Message], sampling.columns if sampling else None)
            return result if isinstance(result, columns) else columns(value=result)

        async def publish(adapter: object, transport: harmos.plugin.Context, publisher: Publisher) -> None:
            key = (cast(str, template.identifier), transport.scope())
            await self._ready.setdefault(key, asyncio.Event()).wait()
            session = self._sessions[key]
            context = Context(transport, {key: decode(value) for key, value in session.case.message.parameters.items()}, template)
            async for result in getattr(session.instance, name)(context):
                row = result if isinstance(result, definition.row) else definition.row(value=result)
                await publisher.publish(row)

        if sampling:
            nanoseconds = round(sampling.interval_ms * 1_000_000)
            duration = Duration(seconds=nanoseconds // 1_000_000_000, nanos=nanoseconds % 1_000_000_000)
            if sampling.columns is None:
                return observer(
                    identity,
                    row=definition.row,
                    interval=duration,
                    interval_setting=sampling.interval_setting,
                    window_setting=sampling.window_setting,
                    description=definition.label,
                )(observe)
            return window_observer(
                identity,
                row=definition.row,
                columns=sampling.columns,
                interval=duration,
                interval_setting=sampling.interval_setting,
                window_setting=sampling.window_setting,
                description=definition.label,
            )(observe_window)
        return publisher(identity, row=definition.row, description=definition.label)(publish)

    def _stop(self) -> Callable[..., Awaitable[None]]:
        async def release(adapter: object) -> None:
            self._sessions.clear()
            self._ready.clear()

        return on_stop(release)


class Endpoint:
    def __init__(self, transport: harmos.plugin.Endpoint, plugin: Plugin) -> None:
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
    registered = value if isinstance(value, Suite) else _SUITES.get(value) if isinstance(value, type) else None
    if not isinstance(registered, Suite):
        raise TypeError("Build Plugin: Supply a Suite or a Class Decorated with suite")
    name = registered.templates[0].instance.__name__ if registered._single else ""
    identity = build_identifier(name, registered.identifier, class_name=True)
    templates = tuple(template.bind(identity, single=registered._single) for template in registered.templates)
    return Suite(identity, templates, registered.label, registered._single)


# === Entry Point === #
