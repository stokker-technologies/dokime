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
from collections.abc import Sequence
from dataclasses import dataclass
from typing import Any

# Third Party Imports
import harmos
from google.protobuf.duration_pb2 import Duration
from google.protobuf.message import Message
from harmos.v1 import types_pb2 as wire

from ._context import HOST_OPERATIONS, Context

# Local Imports
from .authoring import Case, Record, Suite, Template
from .v1 import message, message_type
from .values import decode

# Type Checking Imports

# === Public Exports === #

__all__ = ["Endpoint", "Plugin", "serve"]

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
        requires: Sequence[harmos.Operation] | None = None,
        sections: Sequence[Message] = (),
        messages: Sequence[type[Message]] = (),
        operations: object | None = None,
    ) -> None:
        self.identifier = identifier
        self.version = version
        self.suites = tuple(_suite(item) for item in suites)
        self.records = tuple(records)
        self.requires = HOST_OPERATIONS if requires is None else tuple(requires)
        self._sessions: dict[tuple[str, str], _Session] = {}
        self._ready: dict[tuple[str, str], asyncio.Event] = {}
        templates = [template for suite in self.suites for template in suite.templates]
        for name, identifiers in (
            ("suite", [item.identifier for item in self.suites]),
            ("template", [item.identifier for item in templates]),
            ("record", [item.identifier for item in self.records]),
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
        contributions = [message("Suites", suites=[item.definition() for item in self.suites])] if self.suites else []
        if records:
            contributions.append(
                message(
                    "RecordContracts",
                    records=[
                        message(
                            "RecordContract",
                            metadata={"identifier": item.identifier, "label": item.label or item.identifier},
                            message=item.row.DESCRIPTOR.full_name,
                        )
                        for item in records
                    ],
                )
            )
        self._plugin = harmos.Plugin(
            adapter,
            id=identifier,
            version=version,
            requires=self.requires,
            sections=[*sections, *contributions],
            messages=[*messages, *(record.row for record in self.records)],
        )

    def catalog(self) -> wire.Catalog:
        return self._plugin.catalog()

    async def start(self, address: str = "127.0.0.1:7001") -> "Endpoint":
        # Harmos Owns Transport, Fingerprints, Cancellation And Stream Backpressure.
        transport = _ServingPlugin(self)
        endpoint = await transport.start(address)
        return Endpoint(endpoint, self)

    def _materialize(self, template: Template):
        async def execute(adapter, transport, request):
            try:
                transport.scope()
            except harmos.Failure as error:
                if error.value.code != "scope_required":
                    raise
            else:
                raise ValueError("Materialize Test Case: Execution Scope Must Be Empty")
            if request.template != template.identifier:
                raise ValueError("Materialize Test Case: Supply the Bound Template Without an Execution Scope")
            return message(
                "dokime.plugin.v1.TestCasesMaterializeResponse",
                test_case=template.build({key: decode(value) for key, value in request.parameters.items()}).message,
            )

        return harmos.operation(
            f"{template.identifier}.materialize",
            request=message_type("dokime.plugin.v1.TestCasesMaterializeRequest"),
            response=message_type("dokime.plugin.v1.TestCasesMaterializeResponse"),
            visibility=wire.VISIBILITY_PROTECTED,
        )(execute)

    def _execution(self, template: Template, kind: str, handler=None, retries: int = 0):
        family = {
            "step": "StepsExecute",
            "criterion": "CriteriaEvaluate",
            "initialize": "ExecutionsInitialize",
            "cleanup": "ExecutionsCleanup",
        }[kind]

        async def execute(adapter, transport, request):
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
                termination=request.termination if kind == "cleanup" else None,
            )
            function = handler or template._methods.get(kind)
            arguments = {}
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
                    return message("dokime.plugin.v1.CriteriaEvaluateResponse", passed=result)
                return message(f"dokime.plugin.v1.{family}Response")
            finally:
                if kind == "cleanup":
                    self._sessions.pop(key, None)
                    self._ready.pop(key, None)

        operation = template.operation(handler, kind) if handler is not None else f"{template.identifier}.{kind}"
        policy = (
            wire.OperationPolicy(retries=wire.RetryPolicy(total_attempts=retries + 1, initial={"nanos": 100_000_000})) if retries else None
        )
        return harmos.operation(
            operation,
            request=message_type(f"dokime.plugin.v1.{family}Request"),
            response=message_type(f"dokime.plugin.v1.{family}Response"),
            visibility=wire.VISIBILITY_PROTECTED,
            policy=policy,
            replay_safe=bool(retries),
        )(execute)

    def _stream(self, template: Template, name: str, method):
        definition = method.__dokime_stream__
        interval = getattr(method, "__dokime_observer__", None)

        async def observe(adapter, transport):
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

        async def publish(adapter, transport, publisher):
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
            return harmos.observer(
                definition.identifier, row=definition.row, interval=duration, interval_setting=setting, description=definition.label
            )(observe)
        return harmos.publisher(definition.identifier, row=definition.row, description=definition.label)(publish)

    def _stop(self):
        async def release(adapter):
            self._sessions.clear()
            self._ready.clear()

        return harmos.on_stop(release)


class _ServingPlugin(harmos.Plugin):
    def __init__(self, owner: Plugin) -> None:
        self._owner = owner
        super().__init__(
            owner._plugin.instance, id=owner.identifier, version=owner.version, requires=owner.requires, sections=owner._plugin.sections
        )

    def catalog(self) -> wire.Catalog:
        return self._owner.catalog()


class Endpoint:
    def __init__(self, transport: Any, plugin: Plugin) -> None:
        self._transport = transport
        self.plugin = plugin
        self.url = f"http://{transport.address}"
        self.address = transport.address

    async def close(self) -> None:
        await self._transport.close()
        self.plugin._sessions.clear()
        self.plugin._ready.clear()


# === Public Functions === #


async def serve(
    *suites: Suite | type | Plugin,
    address: str = "127.0.0.1:7001",
    identifier: str | None = None,
    version: str = "0.1.0",
    records: Sequence[Record] = (),
    requires: Sequence[harmos.Operation] | None = None,
) -> Endpoint:
    if len(suites) == 1 and isinstance(suites[0], Plugin):
        return await suites[0].start(address)
    values = tuple(_suite(item) for item in suites)
    if not values:
        raise ValueError("Serve requires a plugin or at least one suite")
    plugin = Plugin(identifier or values[0].identifier, version, suites=values, records=records, requires=requires)
    return await plugin.start(address)


# === Private Functions === #


def _suite(value: Suite | type | Plugin) -> Suite:
    if isinstance(value, Suite):
        return value
    registered = getattr(value, "__dokime_suite__", None)
    if not isinstance(registered, Suite):
        raise TypeError("Expected a Suite or a class decorated with @dokime.suite")
    return registered


# === Entry Point === #
