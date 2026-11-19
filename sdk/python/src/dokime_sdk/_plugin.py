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
from collections.abc import Awaitable, Callable, Sequence
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
from harmos.plugin import Publisher, Window, observer, on_stop, operation, publisher, window_observer

# Local Imports
import dokime_sdk.operations
from dokime_sdk._wire.model.v1.prompts_pb2 import PromptDefinitions
from dokime_sdk._wire.model.v1.record_contracts_pb2 import RecordContract, RecordContracts
from dokime_sdk._wire.model.v1.suites_pb2 import Suites

from ._authoring import Materialization, Sampling, Stream, Suite, _Template, bind_suite, build_identifier, build_label
from ._generated.host import HostOperation, host_requirement
from ._generated.shared import SecretsDeclarations
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
    identifier: ClassVar[str | None] = None
    label: ClassVar[str | None] = None
    version: ClassVar[str | None] = None
    suites: ClassVar[Sequence[type[Suite]]] = ()
    operations: ClassVar[Sequence[Callable[..., object]]] = ()
    settings: ClassVar[Sequence[dokime_sdk.operations.SettingDeclaration]] = ()
    secrets: ClassVar[Sequence[dokime_sdk.operations.SecretDeclaration]] = ()

    def __init__(self) -> None:
        self._identifier = build_identifier(type(self).__name__, self.identifier, class_name=True)
        self._label = self.label if self.label is not None else build_label(type(self).__name__)
        self._version = self.version if self.version is not None else _distribution_version(type(self))
        self._suites = tuple(bind_suite(item) for item in tuple(self.suites))
        self._records = tuple((template, record) for suite in self._suites for template in suite.templates for record in template.records)
        self._operations = tuple(self.operations)
        self._secrets = tuple(self.secrets)
        self._settings = tuple(self.settings)
        self._requires = tuple(host_requirement(item) for item in HostOperation)
        self._sessions: dict[tuple[str, str], _Session] = {}
        self._ready: dict[tuple[str, str], asyncio.Event] = {}
        templates = [template for suite in self._suites for template in suite.templates]
        for name, identifiers in (
            ("suite", [item.identifier for item in self._suites]),
            ("template", [item.identifier for item in templates]),
            ("record", [template.resolve(item) for template, item in self._records]),
            ("secret", [item.identifier for item in self._secrets]),
        ):
            if any(not item for item in identifiers) or len(identifiers) != len(set(identifiers)):
                raise ValueError(f"Build Plugin: Supply Unique, Nonempty {name.title()} Identifiers")
        methods: dict[str, Callable[..., Awaitable[Message | None]]] = {}
        operation_identities = []
        for index, function in enumerate(self._operations):
            if not isinstance(function, FunctionType) or not hasattr(function, "__dokime_operation__"):
                raise TypeError("Build Plugin: List Decorated Plain Functions in operations")
            definition = cast(dokime_sdk.operations.Operation[Any, Any], vars(function)["__dokime_operation__"])
            operation_identities.append(definition.identifier)
            methods[f"external_{index}"] = dokime_sdk.operations._bind_operation(function, self)
        if len(operation_identities) != len(set(operation_identities)):
            raise ValueError("Build Plugin: Give Every Operation a Unique Identifier")
        for index, template in enumerate(templates):
            methods[f"materialize_{index}"] = self._materialize(template)
            if template.definition().HasField("initialize"):
                methods[f"initialize_{index}"] = self._execution(template, "setup")
            methods[f"cleanup_{index}"] = self._execution(template, "cleanup")
            lifecycle: Sequence[tuple[str, Sequence[Any]]] = (("step", template.steps), ("criterion", template.criteria))
            for kind, definitions in lifecycle:
                for position, definition in enumerate(definitions):
                    methods[f"{kind}_{index}_{position}"] = self._execution(
                        template, kind, definition.handler, getattr(definition, "retries", 0), definition.policy
                    )
            for position, (name, method) in enumerate(template._methods.items()):
                if hasattr(method, "__dokime_stream__"):
                    methods[f"stream_{index}_{position}"] = self._stream(template, name, method)
        self._streams: dict[int, str] = {}
        plugin_methods: dict[str, object] = {}
        for cls in reversed(type(self).__mro__):
            plugin_methods.update(vars(cls))
        for index, (name, method) in enumerate(plugin_methods.items()):
            declaration = getattr(method, "__dokime_stream__", None)
            if not isinstance(declaration, Stream):
                continue
            if not isinstance(method, FunctionType):
                raise TypeError("Declare Plugin Stream: Decorate a Plain Method")
            identity = build_identifier(name, declaration.identifier, prefix=self._identifier)
            if identity in self._streams.values():
                raise ValueError("Build Plugin: Give Every Plugin Stream a Unique Identifier")
            self._streams[id(method)] = identity
            self._streams[id(declaration)] = identity
            methods[f"plugin_stream_{index}"] = self._stream(None, name, method)
        methods["release"] = self._stop()
        adapter = type("DokimeOperations", (), methods)()
        contributions: list[Message] = [Suites(suites=[item.definition() for item in self._suites])] if self._suites else []
        if self._records:
            contributions.append(
                RecordContracts(
                    records=[
                        RecordContract(
                            metadata={"identifier": template.resolve(item), "label": item.label or template.resolve(item)},
                            message=message_type(item.row).DESCRIPTOR.full_name,
                        )
                        for template, item in self._records
                    ],
                )
            )
        prompts = [value for template in templates for value in template._prompts.values()]
        if prompts:
            contributions.append(PromptDefinitions(prompts=prompts))
        if self._secrets:
            contributions.append(SecretsDeclarations(secrets=[item._definition() for item in self._secrets]))
        self._plugin = harmos.plugin.Plugin(
            adapter,
            id=self._identifier,
            version=self._version,
            requires=self._requires,
            sections=contributions,
            settings=[item._definition() for item in self._settings],
            messages=[*(message_type(record.row) for _, record in self._records)],
        )

        self._plugin.declarations()

    def declarations(self) -> Declarations:
        return self._plugin.declarations()

    async def initialize(self, ctx: dokime_sdk.operations.Context) -> None:
        pass

    async def finalize(self, ctx: dokime_sdk.operations.Context) -> None:
        pass

    async def serve(self, address: str | None = None) -> None:
        endpoint = await self._activate(address)
        termination = asyncio.create_task(endpoint.server.wait_for_termination())
        try:
            print(f"harmos listening {endpoint.address}", flush=True)
            await asyncio.shield(termination)
        except asyncio.CancelledError:
            pass
        finally:
            await endpoint.close()
            await termination

    async def _activate(self, address: str | None = None) -> harmos.plugin.Endpoint:
        from ._configuration import load, resolve

        if type(self).initialize is not Plugin.initialize or type(self).finalize is not Plugin.finalize:
            raise ValueError(
                "Activate Plugin: Upgrade harmos-sdk to Provide initialize(ctx) and finalize(ctx) Activation Hooks "
                "With an Admitted harmos.plugin.Context; Plugin.start in harmos/_authoring.py Only Binds a Listener, "
                "and on_stop in harmos/_contracts.py Accepts No Context. Keep Plugin Hooks Unoverridden Until Then"
            )
        return await self._plugin.start(resolve("DOKIME_PLUGIN_ADDRESS", address, load()))

    def _materialize(self, template: _Template) -> Callable[..., Awaitable[Message]]:
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
            "setup": (ExecutionsInitializeRequest, ExecutionsInitializeResponse),
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
                self,
                {name: decode(value) for name, value in request.test_case.parameters.items()},
                template,
                termination=request.termination if isinstance(request, ExecutionsCleanupRequest) else None,
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
            except dokime_sdk.operations.FailureError as error:
                raise harmos.Failure(dokime_sdk.operations._failure(error.failure)) from error
            finally:
                if kind == "cleanup":
                    self._sessions.pop(key, None)
                    self._ready.pop(key, None)

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
            replay_safe=bool(retries),
        )(execute)

    def _stream(
        self, template: _Template | None, name: str, method: FunctionType
    ) -> Callable[..., Awaitable[Message]] | Callable[..., Awaitable[None]]:
        definition = cast(Stream, vars(method)["__dokime_stream__"])
        identity = template.resolve(method) if template is not None else self._streams[id(method)]
        sampling = cast(Sampling | None, getattr(method, "__dokime_observer__", None))

        async def observe(adapter: object, transport: harmos.plugin.Context) -> Message:
            instance, context = await self._stream_context(template, transport)
            result = getattr(instance, name)(context)
            if inspect.isawaitable(result):
                result = await result
            return pack(definition.row, result)

        async def observe_window(adapter: object, transport: harmos.plugin.Context, window: Window) -> Message:
            instance, context = await self._stream_context(template, transport)
            result = getattr(instance, name)(context, window)
            if inspect.isawaitable(result):
                result = await result
            columns = cast(type[Message], sampling.columns if sampling else None)
            return pack(columns, result)

        async def publish(adapter: object, transport: harmos.plugin.Context, publisher: Publisher) -> None:
            instance, context = await self._stream_context(template, transport)
            async for result in getattr(instance, name)(context):
                row = pack(definition.row, result)
                await publisher.publish(row)

        if sampling:
            nanoseconds = round(sampling.interval_ms * 1_000_000)
            duration = Duration(seconds=nanoseconds // 1_000_000_000, nanos=nanoseconds % 1_000_000_000)
            if sampling.columns is None:
                return observer(
                    identity,
                    row=message_type(definition.row),
                    interval=duration,
                    interval_setting=sampling.interval_setting,
                    window_setting=sampling.window_setting,
                    description=definition.label,
                )(observe)
            return window_observer(
                identity,
                row=message_type(definition.row),
                columns=sampling.columns,
                interval=duration,
                interval_setting=sampling.interval_setting,
                window_setting=sampling.window_setting,
                description=definition.label,
            )(observe_window)
        return publisher(identity, row=message_type(definition.row), description=definition.label)(publish)

    async def _stream_context(
        self,
        template: _Template | None,
        transport: harmos.plugin.Context,
    ) -> tuple[object, dokime_sdk.operations.Context]:
        if template is None:
            return self, dokime_sdk.operations.Context(transport, self)
        key = (cast(str, template.identifier), transport.scope())
        await self._ready.setdefault(key, asyncio.Event()).wait()
        session = self._sessions[key]
        context = Context(transport, self, {key: decode(value) for key, value in session.case.message.parameters.items()}, template)
        return session.instance, context

    def _stop(self) -> Callable[..., Awaitable[None]]:
        async def release(adapter: object) -> None:
            self._sessions.clear()
            self._ready.clear()

        return on_stop(release)


# === Public Functions === #


# === Private Functions === #


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
        if url.scheme == "file" and source.is_relative_to(Path(url2pathname(url.path)).resolve()):
            versions.append(distribution.version)
    if len(versions) == 1:
        return versions[0]
    raise ValueError("Build Plugin: Install Its Owning Distribution or Declare version on the Plugin Class")


# === Entry Point === #
