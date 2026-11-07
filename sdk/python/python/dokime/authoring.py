# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Python Plugin Declarations"""

# === Imports === #

# Standard Library Imports
import dataclasses
import enum
import inspect
import json
import types
from collections.abc import Callable, Generator, Mapping, Sequence
from dataclasses import MISSING, dataclass, field
from typing import Annotated, Any, Literal, NotRequired, Self, TypedDict, dataclass_transform, get_args, get_origin, get_type_hints

# Third Party Imports
# Local Imports
from . import _native

# Type Checking Imports


# === Public Exports === #

__all__ = [
    "Context",
    "Document",
    "Invocation",
    "OperationInstance",
    "OperationPolicy",
    "Plugin",
    "PromptAnswer",
    "ReferenceBand",
    "ReferenceLine",
    "ReferenceValue",
    "RegistrationError",
    "RetryPolicy",
    "StreamSelection",
    "StreamWriter",
    "criterion",
    "observer",
    "operation",
    "parameter",
    "parameters",
    "prompt",
    "record",
    "section",
    "settings",
    "step",
    "stream",
    "suite",
    "template",
]

# === Constants === #

# === Types === #


type ReferenceValue = float | Mapping[str, Any]


# === Classes === #


class ReferenceLine(TypedDict):
    """A Y-axis reference in stored units, scaled with the plot data."""

    label: NotRequired[str]
    color: NotRequired[str]
    value: ReferenceValue


class ReferenceBand(TypedDict, total=False):
    """Y-axis bounds or center ± tolerance in stored units, scaled with the data."""

    label: str
    color: str
    min: ReferenceValue
    max: ReferenceValue
    center: ReferenceValue
    tolerance: ReferenceValue


class RegistrationError(ValueError):
    """A Plugin Declaration Does Not Match Its Native Contract."""


@dataclass(frozen=True)
class RetryPolicy:
    max_retries: int = 0
    delay_s: float = 0.0
    jitter: Literal["none", "deterministic"] = "none"

    def _wire(self) -> Any:
        if self.max_retries == 0:
            return "never"
        return {
            "on_transient": {
                "max_retries": self.max_retries,
                "backoff": {"fixed": {"delay": _duration(self.delay_s), "jitter": self.jitter}},
            }
        }


@dataclass(frozen=True)
class OperationPolicy:
    timeout_s: float
    attempt_timeout_s: float
    cancellation_timeout_s: float = 0.0
    retries: RetryPolicy = RetryPolicy()

    def _wire(self) -> dict[str, Any]:
        return {
            "timeout": _duration(self.timeout_s),
            "attempt_timeout": _duration(self.attempt_timeout_s),
            "cancellation_timeout": _duration(self.cancellation_timeout_s),
            "retries": self.retries._wire(),
        }


@dataclass(frozen=True)
class PromptAnswer:
    action: str
    accepted: bool
    values: Mapping[str, Any]


class Invocation[T]:
    def __init__(self, context: "Context", request: Any, retry: RetryPolicy | None = None) -> None:
        self._context = context
        self._request = request
        self._retry = retry

    def retry(self, policy: RetryPolicy) -> Self:
        return type(self)(self._context, self._request, policy)

    async def start(self) -> "OperationInstance[T]":
        declaration = _meta(type(self._request), "operation")
        response = declaration["response"]
        native = await self._context._inner.start(
            declaration["id"],
            _json(_plain(self._request)),
            _json(_shape(type(self._request))),
            _json(_shape(response)),
            _json(self._retry._wire()) if self._retry is not None else None,
        )
        return OperationInstance(native, response)

    async def _resolve(self) -> T:
        return await (await self.start()).join()

    def __await__(self) -> Generator[Any, None, T]:
        return self._resolve().__await__()


class OperationInstance[T]:
    def __init__(self, inner: _native.OperationInstance, response: type[T]) -> None:
        self._inner = inner
        self._response = response

    @property
    def id(self) -> int:
        return self._inner.id

    def request_cancel(self) -> None:
        self._inner.request_cancel()

    async def join(self) -> T:
        return _decode(self._response, json.loads(await self._inner.join()))

    async def cancel(self) -> T:
        self.request_cancel()
        return await self.join()


class StreamWriter:
    def __init__(self, context: "Context", declaration: type, row: type) -> None:
        self._context = context
        self._declaration = declaration
        self._row = row

    async def push(self, row: Any) -> None:
        if type(row) is not self._row:
            raise TypeError("row does not match the declared stream")
        await self._context._inner.push(_meta(self._declaration, "stream")["id"], _json(_plain(row)))

    async def push_columns(self, columns: Mapping[str, Any]) -> None:
        await self._context._inner.push_columns(_meta(self._declaration, "stream")["id"], dict(columns))


class StreamSelection:
    def __init__(self, context: "Context", row: type) -> None:
        self._context = context
        self._row = row

    def into(self, declaration: type) -> StreamWriter:
        metadata = _meta(declaration, "stream")
        if metadata.get("row", declaration) is not self._row:
            raise RegistrationError("stream does not publish the selected row type")
        return StreamWriter(self._context, declaration, self._row)

    def open(self) -> StreamWriter:
        return self.into(self._row)


class Document:
    """Document cards use native mappings: XY plots select ``plot_type="xy"``.

    Their ``style`` defaults to ``"line"``; ``"points"`` uses density reduction,
    while ``"line_points"`` and ``"step"`` retain the ordered envelope.
    """

    def __init__(self, context: "Context") -> None:
        self._context = context

    async def section(self, identifier: str, label: str, *, cards: Sequence[Mapping[str, Any]] = ()) -> None:
        await self._context._inner.command("section", _json({"metadata": _metadata(identifier, label), "cards": list(cards)}))


class Context:
    def __init__(
        self,
        inner: _native.CallbackContext,
        *,
        parameters: Any = None,
        settings: Any = None,
        resources: Any = None,
        registered: Mapping[type, Any] | None = None,
        tick: Any = None,
    ) -> None:
        self._inner = inner
        self.parameters = parameters
        self.settings = settings
        self.resources = resources
        self.tick = tick
        self.reason: str | None = None
        self._registered = registered or {}

    @property
    def execution(self) -> str | None:
        return self._inner.execution

    @property
    def document(self) -> Document:
        return Document(self)

    @property
    def cancelled(self) -> bool:
        return self._inner.cancelled

    def resource[T](self, resource: type[T]) -> T:
        return self._registered[resource]

    def operation[T](self, request: Any) -> Invocation[T]:
        _meta(type(request), "operation")
        return Invocation(self, request)

    def stream(self, row: type) -> StreamSelection:
        return StreamSelection(self, row)

    def evidence(self) -> Mapping[str, Any]:
        return json.loads(self._inner.evidence())

    async def log(self, message: str, *, level: Literal["trace", "debug", "info", "warn", "error"] = "info") -> None:
        await self._inner.command("log", _json({"level": level, "message": message}))

    async def publish(self, value: Any) -> None:
        metadata = _meta(type(value), "record")
        await self._inner.command(
            "record", _json({"record_type": metadata["id"], "severity": metadata.get("severity", "info"), "details": _plain(value)})
        )

    async def ask(self, value: Any) -> PromptAnswer:
        metadata = _meta(type(value), "prompt")
        result = json.loads(await self._inner.command("ask", _json({"prompt": metadata["id"], "values": _plain(value)})))
        return PromptAnswer(**result)


class Plugin:
    def __init__(
        self,
        id: str,
        version: str,
        description: str = "",
        *,
        catalog: Sequence[Any],
        resources: Mapping[type, Any] | None = None,
        requires: Sequence[type] = (),
    ) -> None:
        self.id = id
        self.version = version
        self.description = description
        self.catalog = tuple(catalog)
        self._resources = dict(resources or {})
        self._requires = tuple(requires)
        self._settings_types: list[type] = []
        self._templates: dict[str, type] = {}
        self._operations: dict[str, type] = {}
        self._observers: dict[str, Callable[..., Any]] = {}
        try:
            self._spec = self._compile()
            self._catalog = json.loads(_native.validate_plugin(_json(self._spec)))
        except (TypeError, ValueError) as error:
            raise RegistrationError(str(error)) from error

    def _compile(self) -> dict[str, Any]:
        result: dict[str, Any] = {
            "id": self.id,
            "version": self.version,
            "description": self.description,
            "operations": [],
            "streams": [],
            "templates": [],
            "suites": [],
            "observers": [],
            "settings": {"contributions": []},
            "settings_status": None,
            "records": [],
            "prompts": [],
            "requirements": [],
        }
        entries = list(self.catalog)
        for entry in entries:
            metadata = _meta(entry)
            kind = metadata["kind"]
            if kind == "suite":
                members = metadata["templates"]
                result["suites"].append(
                    {
                        "id": metadata["id"],
                        "label": metadata["label"],
                        "description": metadata.get("description"),
                        "tags": list(metadata.get("tags", ())),
                        "templates": [_meta(member, "template")["id"] for member in members],
                    }
                )
                for member in members:
                    declaration = _meta(member, "template")
                    self._templates[declaration["id"]] = member
                    result["templates"].append(_template_spec(member))
            elif kind == "operation":
                identifier = metadata["id"]
                if identifier in self._operations:
                    raise RegistrationError(f"duplicate operation {identifier}")
                self._operations[identifier] = entry
                if not callable(getattr(entry, "run", None)) or len(inspect.signature(entry.run).parameters) != 2:
                    raise RegistrationError(f"operation {identifier} needs run(self, ctx)")
                for resource in metadata.get("resources", ()):
                    if resource not in self._resources:
                        raise RegistrationError(f"operation {identifier} requires resource {resource.__name__}")
                result["operations"].append(
                    {
                        "id": identifier,
                        "description": metadata.get("description", ""),
                        "policy": metadata["policy"]._wire(),
                        "visibility": metadata["visibility"],
                        "replay": metadata["replay"],
                        "request": _shape(entry),
                        "response": _shape(metadata["response"]),
                        "requires": [_meta(required, "operation")["id"] for required in metadata["operations"]],
                    }
                )
            elif kind == "stream":
                row = metadata.get("row", entry)
                shape = _shape(row)
                result["streams"].append(
                    {
                        "id": metadata["id"],
                        "label": metadata["label"],
                        "description": metadata.get("description"),
                        "window": metadata["window"],
                        "row": shape,
                        "columns": _columns(row),
                    }
                )
            elif kind == "record":
                result["records"].append(_record_spec(entry))
            elif kind == "prompt":
                result["prompts"].append(_prompt_spec(entry))
            elif kind == "observer":
                self._observers[metadata["id"]] = entry
                result["observers"].append(_observer_spec(metadata))
            elif kind in ("settings", "section"):
                self._settings_types.append(entry)
                result["settings"]["contributions"].extend(_settings_spec(entry))
                groups = get_type_hints(entry).values() if kind == "settings" else [entry]
                for group in groups:
                    status = _meta(group, "section").get("status")
                    if status is not None:
                        identifier = _meta(status, "operation")["id"]
                        if result["settings_status"] not in (None, identifier):
                            raise RegistrationError("settings sections must share the plugin's one native status operation")
                        result["settings_status"] = identifier
            else:
                raise RegistrationError(f"{kind} cannot be contributed directly by a plugin")
        required_entries = list(self._requires)
        required_entries.extend(required for template_type in self._templates.values() for required in _meta(template_type)["requires"])
        required_entries.extend(
            required for operation_type in self._operations.values() for required in _meta(operation_type)["operations"]
        )
        for required in required_entries:
            identifier = _meta(required, "operation")["id"]
            metadata = _meta(required, "operation")
            result["requirements"].append(
                {
                    "id": identifier,
                    "description": metadata["description"],
                    "policy": metadata["policy"]._wire(),
                    "visibility": metadata["visibility"],
                    "replay": metadata["replay"],
                    "request": _shape(required),
                    "response": _shape(metadata["response"]),
                    "requires": [],
                }
            )
        return result

    def _context(
        self, native: _native.CallbackContext, *, parameters_type: type | None = None, shared: Any = None, tick: Any = None
    ) -> Context:
        parameters = json.loads(native.parameters)
        values = json.loads(native.settings)
        configured = [_settings_snapshot(kind, values) for kind in self._settings_types]
        snapshot = configured[0] if len(configured) == 1 else tuple(configured)
        return Context(
            native,
            parameters=_decode(parameters_type, parameters) if parameters_type else types.SimpleNamespace(**parameters),
            settings=snapshot,
            resources=shared,
            registered=self._resources,
            tick=tick,
        )

    def _bind(self) -> _native.EmbeddedProvider:
        callbacks: dict[str, Callable[..., Any]] = {}
        for identifier, operation_type in self._operations.items():
            function = operation_type.run
            callbacks[f"operation:{identifier}"] = _wrap(
                function, lambda native, payload, kind=operation_type: (kind.run, (_decode(kind, payload), self._context(native)))
            )
        for identifier, observer_function in self._observers.items():
            callbacks[f"observer:{identifier}"] = _wrap(
                observer_function, lambda native, payload, function=observer_function: (function, (self._context(native, tick=payload),))
            )
        for identifier, template_type in self._templates.items():
            callbacks.update(self._template_callbacks(identifier, template_type))
        return _native.embedded_provider(_json(self._spec), callbacks)

    def _template_callbacks(self, identifier: str, kind: type) -> dict[str, Callable[..., Any]]:
        states: dict[str | None, list[Any]] = {}
        parameters_type = _meta(kind)["parameters"]
        prefix = f"template:{identifier}"
        initialize = getattr(kind, "initialize", None)

        def prepare_initialize(native: _native.CallbackContext, payload: Any) -> tuple[Callable[..., Any], tuple[Any, ...]]:
            instance = kind()
            states[native.execution] = [instance, None]
            context = self._context(native, parameters_type=parameters_type)
            function = getattr(instance, "initialize", lambda context: None)
            return function, (context,)

        def initialized(native: _native.CallbackContext, result: Any) -> None:
            states[native.execution][1] = result

        def cleanup(native: _native.CallbackContext) -> None:
            states.pop(native.execution, None)

        callbacks = {f"{prefix}:initialize": _wrap(initialize, prepare_initialize, finish=initialized, failed=cleanup)}
        for name, descriptor in vars(kind).items():
            function = descriptor.__func__ if isinstance(descriptor, staticmethod) else descriptor
            declaration = getattr(function, "__dokime__", None)
            if declaration is None or declaration["kind"] not in ("step", "criterion", "observer"):
                continue
            stage = declaration["kind"]

            def prepare(
                native: _native.CallbackContext, payload: Any, method=name, stage=stage, function=function
            ) -> tuple[Callable[..., Any], tuple[Any, ...]]:
                instance, shared = states.get(native.execution, [None, None])
                context = self._context(native, parameters_type=parameters_type, shared=shared, tick=payload)
                target = function if stage in ("observer", "criterion") else getattr(instance, method)
                return target, (context,)

            callbacks[f"{prefix}:{stage}:{declaration['id']}"] = _wrap(function, prepare)

        def prepare_cleanup(native: _native.CallbackContext, payload: Any) -> tuple[Callable[..., Any], tuple[Any, ...]]:
            instance, shared = states.get(native.execution, [None, None])
            context = self._context(native, parameters_type=parameters_type, shared=shared)
            context.reason = payload.get("reason")
            return getattr(instance, "cleanup", lambda context: None), (context,)

        callbacks[f"{prefix}:cleanup"] = _wrap(getattr(kind, "cleanup", None), prepare_cleanup, final=cleanup)
        return callbacks


# === Public Functions === #


def parameter(
    *,
    label: str | None = None,
    description: str | None = None,
    default: Any = MISSING,
    required: bool = True,
    min: float | None = None,
    max: float | None = None,
    **metadata: Any,
) -> Any:
    accepted = {"unit", "expansion", "runtime_writable", "choices", "list", "group_id", "role", "display_unit", "format", "step"}
    unknown = set(metadata) - accepted
    if unknown:
        raise RegistrationError(f"unknown parameter metadata: {', '.join(sorted(unknown))}")
    return field(
        default=default, metadata={"label": label, "description": description, "required": required, "min": min, "max": max, **metadata}
    )


@dataclass_transform(field_specifiers=(parameter,))
def parameters[T](kind: type[T]) -> type[T]:
    return dataclasses.dataclass(kind)


@dataclass_transform(field_specifiers=(parameter,))
def operation[T](
    *,
    id: str,
    response: type | None = None,
    policy: OperationPolicy,
    visibility: Literal["private", "public", "host"] = "private",
    replay: Literal["unknown", "idempotent", "deduplicated"] = "unknown",
    description: str = "",
    resources: Sequence[type] = (),
    operations: Sequence[type] = (),
) -> Callable[[type[T]], type[T]]:
    return _declare(
        "operation",
        id=id,
        response=response,
        policy=policy,
        visibility=visibility,
        replay=replay,
        description=description,
        resources=resources,
        operations=operations,
    )


def template[T](
    *,
    id: str,
    label: str,
    parameters: type | None = None,
    requires: Sequence[type] = (),
    subscribes: Sequence[type] = (),
    streams: Sequence[type] = (),
    prompts: Sequence[type] = (),
    tags: Sequence[str] = (),
    description: str | None = None,
    document: Mapping[str, Any] | None = None,
) -> Callable[[type[T]], type[T]]:
    return _declare(
        "template",
        id=id,
        label=label,
        parameters=parameters,
        requires=requires,
        subscribes=subscribes,
        streams=streams,
        prompts=prompts,
        tags=tags,
        description=description,
        document=document,
    )


def suite[T](
    *, id: str, label: str, templates: Sequence[type], tags: Sequence[str] = (), description: str | None = None
) -> Callable[[type[T]], type[T]]:
    return _declare("suite", id=id, label=label, templates=templates, tags=tags, description=description)


def step(
    *, id: str | None = None, label: str | None = None, description: str | None = None, repeatable: bool = False
) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
    return _declare("step", id=id, label=label, description=description, repeatable=repeatable)


def criterion(
    *, id: str | None = None, label: str | None = None, description: str | None = None, expected: Any = None
) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
    return _declare("criterion", id=id, label=label, description=description, expected=expected)


def observer(
    *, id: str | None = None, label: str | None = None, interval_ms: int, streams: Sequence[type] = ()
) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
    return _declare("observer", id=id, label=label, interval_ms=interval_ms, streams=streams)


@dataclass_transform(field_specifiers=(parameter,))
def stream[T](*, id: str, label: str, window: int, row: type | None = None, description: str | None = None) -> Callable[[type[T]], type[T]]:
    return _declare("stream", id=id, label=label, window=window, row=row, description=description)


@dataclass_transform(field_specifiers=(parameter,))
def record[T](
    *, id: str, label: str, schema: int = 1, severity: Literal["debug", "info", "warning", "error"] = "info", description: str | None = None
) -> Callable[[type[T]], type[T]]:
    return _declare("record", id=id, label=label, schema=schema, severity=severity, description=description)


@dataclass_transform(field_specifiers=(parameter,))
def prompt[T](
    *, id: str, label: str, message: str | None = None, archetype: str = "FormPrompt", actions: Sequence[Mapping[str, Any]] = ()
) -> Callable[[type[T]], type[T]]:
    return _declare("prompt", id=id, label=label, message=message, archetype=archetype, actions=actions)


@dataclass_transform(field_specifiers=(parameter,))
def section[T](*, label: str, description: str | None = None, status: type | None = None) -> Callable[[type[T]], type[T]]:
    return _declare("section", label=label, description=description, status=status)


@dataclass_transform(field_specifiers=(parameter,))
def settings[T](kind: type[T]) -> type[T]:
    return _declare("settings")(kind)


# === Private Functions === #


def _declare(kind: str, **metadata: Any) -> Callable[..., Any]:
    def decorate(value: Any) -> Any:
        if hasattr(value, "__dokime__"):
            raise RegistrationError(f"{value.__name__} already has a declaration")
        declaration = {"kind": kind, **metadata}
        if "id" in declaration and declaration["id"] is None:
            declaration["id"] = value.__name__
        if "label" in declaration and declaration["label"] is None:
            declaration["label"] = value.__name__.replace("_", " ").title()
        if kind == "stream" and declaration["row"] is None:
            declaration["row"] = value
        if (
            isinstance(value, type)
            and kind in ("operation", "record", "prompt", "section", "settings", "stream")
            and not dataclasses.is_dataclass(value)
        ):
            value = dataclasses.dataclass(value)
        value.__dokime__ = declaration
        return value

    return decorate


def _meta(value: Any, expected: str | None = None) -> dict[str, Any]:
    metadata = getattr(value, "__dokime__", None)
    if metadata is None or expected is not None and metadata["kind"] != expected:
        raise RegistrationError(f"{getattr(value, '__name__', value)} is not a declared {expected or 'catalog entry'}")
    return metadata


def _json(value: Any) -> str:
    return json.dumps(value, allow_nan=False)


def _duration(seconds: float) -> dict[str, int]:
    if seconds < 0:
        raise RegistrationError("durations must be nonnegative")
    nanos = round(seconds * 1_000_000_000)
    return {"secs": nanos // 1_000_000_000, "nanos": nanos % 1_000_000_000}


def _plain(value: Any) -> Any:
    if dataclasses.is_dataclass(value) and not isinstance(value, type):
        return {item.name: _plain(getattr(value, item.name)) for item in dataclasses.fields(value)}
    if isinstance(value, enum.Enum):
        return {value.name: None}
    if isinstance(value, bytes):
        return list(value)
    if isinstance(value, Mapping):
        return {key: _plain(item) for key, item in value.items()}
    if isinstance(value, (tuple, list)):
        return [_plain(item) for item in value]
    return value


def _shape(kind: Any) -> dict[str, Any]:
    definitions: list[list[dict[str, Any]]] = []
    seen: dict[Any, int] = {}

    def message(item: Any) -> int:
        if item in seen:
            return seen[item]
        index = len(definitions)
        seen[item] = index
        definitions.append([])
        if item in (None, type(None)):
            return index
        if isinstance(item, type) and issubclass(item, enum.Enum):
            for number, variant in enumerate(item, 1):
                definitions[index].append(
                    {
                        "name": variant.name,
                        "tag": number,
                        "kind": {"type": "message", "definition": message(type(None))},
                        "cardinality": "variant",
                    }
                )
        elif dataclasses.is_dataclass(item):
            hints = get_type_hints(item, include_extras=True)
            for number, member in enumerate(dataclasses.fields(item), 1):
                value_kind, cardinality = field_kind(hints[member.name])
                definitions[index].append({"name": member.name, "tag": number, "kind": value_kind, "cardinality": cardinality})
        else:
            value_kind, cardinality = field_kind(item)
            definitions[index].append({"name": "value", "tag": 1, "kind": value_kind, "cardinality": cardinality})
        return index

    def field_kind(item: Any) -> tuple[dict[str, Any], str]:
        origin = get_origin(item)
        if origin is Annotated:
            base, *annotations = get_args(item)
            if annotations and annotations[0] in ("u32", "u64", "i32", "i64", "f32", "f64"):
                return {"type": annotations[0]}, "required"
            return field_kind(base)
        if origin in (types.UnionType, __import__("typing").Union):
            members = get_args(item)
            if len(members) != 2 or type(None) not in members:
                raise RegistrationError("portable unions require one optional value")
            value_kind, cardinality = field_kind(next(member for member in members if member is not type(None)))
            if cardinality != "required":
                raise RegistrationError("nested optional collections require a dataclass wrapper")
            return value_kind, "optional"
        if origin is list:
            value_kind, cardinality = field_kind(get_args(item)[0])
            if cardinality != "required":
                raise RegistrationError("nested collections require a dataclass wrapper")
            return value_kind, "repeated"
        primitives = {bool: "bool", int: "i64", float: "f64", str: "string", bytes: "bytes"}
        if item in primitives:
            return {"type": primitives[item]}, "required"
        if dataclasses.is_dataclass(item) or isinstance(item, type) and issubclass(item, enum.Enum):
            return {"type": "message", "definition": message(item)}, "required"
        raise RegistrationError(f"unsupported portable field type {item}")

    root = message(kind)
    structured = dataclasses.is_dataclass(kind) or isinstance(kind, type) and issubclass(kind, enum.Enum)
    return {"schema": {"root": root, "definitions": definitions}, "scalar": kind not in (None, type(None)) and not structured}


def _decode(kind: Any, value: Any) -> Any:
    if kind in (None, type(None)):
        return None
    origin = get_origin(kind)
    if origin is Annotated:
        return _decode(get_args(kind)[0], value)
    if origin is list:
        return [_decode(get_args(kind)[0], item) for item in value]
    if origin in (types.UnionType, __import__("typing").Union):
        return None if value is None else _decode(next(item for item in get_args(kind) if item is not type(None)), value)
    if isinstance(kind, type) and issubclass(kind, enum.Enum):
        return kind[next(iter(value))]
    if dataclasses.is_dataclass(kind):
        if value is None:
            return kind()
        hints = get_type_hints(kind, include_extras=True)
        return kind(**{item.name: _decode(hints[item.name], value[item.name]) for item in dataclasses.fields(kind) if item.name in value})
    if kind is int and isinstance(value, float):
        if not value.is_integer():
            raise ValueError("an integer field received a fractional value")
        return int(value)
    return bytes(value) if kind is bytes else value


def _metadata(identifier: str, label: str, description: str | None = None) -> dict[str, Any]:
    return {"identifier": identifier, "label": label, "description": description}


def _value_type(kind: Any) -> str:
    origin = get_origin(kind)
    if origin is Annotated:
        return _value_type(get_args(kind)[0])
    if origin in (types.UnionType, __import__("typing").Union):
        return _value_type(next(item for item in get_args(kind) if item is not type(None)))
    return {bool: "boolean", int: "integer", float: "number", str: "text", bytes: "array"}.get(
        kind, "array" if origin is list else "object"
    )


def _columns(kind: type) -> list[dict[str, Any]]:
    if not dataclasses.is_dataclass(kind):
        raise RegistrationError("row declarations must be dataclasses")
    hints = get_type_hints(kind, include_extras=True)
    result = []
    for item in dataclasses.fields(kind):
        nullable = type(None) in get_args(hints[item.name])
        result.append(
            {
                "metadata": _metadata(
                    item.name, item.metadata.get("label") or item.name.replace("_", " ").title(), item.metadata.get("description")
                ),
                "value_type": _value_type(hints[item.name]),
                "role": item.metadata.get("role", "value"),
                "unit": item.metadata.get("unit"),
                "display_unit": item.metadata.get("display_unit"),
                "required": not nullable,
                "nullable": nullable,
                "default": None,
                "format": item.metadata.get("format"),
            }
        )
    return result


def _observer_spec(metadata: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "id": metadata["id"],
        "label": metadata["label"],
        "interval_ms": metadata["interval_ms"],
        "streams": [_meta(item, "stream")["id"] for item in metadata["streams"]],
    }


def _template_spec(kind: type) -> dict[str, Any]:
    metadata = _meta(kind, "template")
    parameters_type = metadata["parameters"]
    declared_parameters = []
    if parameters_type:
        hints = get_type_hints(parameters_type, include_extras=True)
        for item in dataclasses.fields(parameters_type):
            default = None if item.default is MISSING else _plain(item.default)
            declared_parameters.append(
                {
                    "id": item.name,
                    "label": item.metadata.get("label") or item.name.replace("_", " ").title(),
                    "description": item.metadata.get("description"),
                    "value_type": _value_type(hints[item.name]),
                    "default": default,
                    "required": item.metadata.get("required", True),
                    "min": item.metadata.get("min"),
                    "max": item.metadata.get("max"),
                    "unit": item.metadata.get("unit"),
                    "expansion": item.metadata.get("expansion", {"kind": "single"}),
                    "runtime_writable": item.metadata.get("runtime_writable", False),
                    "choices": _plain(item.metadata.get("choices")),
                    "list": item.metadata.get("list"),
                    "group_id": item.metadata.get("group_id"),
                }
            )
    result = {
        "id": metadata["id"],
        "label": metadata["label"],
        "description": metadata["description"],
        "tags": list(metadata["tags"]),
        "parameters": declared_parameters,
        "steps": [],
        "criteria": [],
        "observers": [],
        "streams": [_meta(item, "stream")["id"] for item in metadata["streams"]],
        "requires": [_meta(item, "operation")["id"] for item in metadata["requires"]],
        "prompts": [_meta(item, "prompt")["id"] for item in metadata["prompts"]],
        "document": metadata["document"],
        "subscriptions": [
            {"Stream" if _meta(item)["kind"] == "stream" else "Record": _meta(item)["id"]} for item in metadata["subscribes"]
        ],
    }
    for descriptor in vars(kind).values():
        function = descriptor.__func__ if isinstance(descriptor, staticmethod) else descriptor
        declaration = getattr(function, "__dokime__", None)
        if declaration is None:
            continue
        stage = declaration["kind"]
        if stage == "observer":
            if len(inspect.signature(function).parameters) != 1:
                raise RegistrationError("template observers accept ctx only and use shared resources")
            result["observers"].append(_observer_spec(declaration))
        elif stage in ("step", "criterion"):
            count = 2 if stage == "step" else 1
            if len(inspect.signature(function).parameters) != count:
                raise RegistrationError(f"{stage} accepts {'self and ctx' if count == 2 else 'ctx only'}")
            result["steps" if stage == "step" else "criteria"].append(
                {
                    key: declaration[key]
                    for key in (
                        ("id", "label", "description", "repeatable") if stage == "step" else ("id", "label", "description", "expected")
                    )
                }
            )
    return result


def _settings_spec(kind: type) -> list[dict[str, Any]]:
    if not dataclasses.is_dataclass(kind):
        raise RegistrationError("settings declarations must be dataclasses")
    metadata = _meta(kind)
    if metadata["kind"] == "settings":
        return [entry for section_type in get_type_hints(kind).values() for entry in _settings_spec(section_type)]
    result = []
    hints = get_type_hints(kind, include_extras=True)
    for item in dataclasses.fields(kind):
        if item.default is MISSING:
            raise RegistrationError(f"setting {item.name} needs an explicit default")
        value_type = _value_type(hints[item.name])
        entry = {
            "id": item.name,
            "title": item.metadata.get("label") or item.name.replace("_", " ").title(),
            "description": item.metadata.get("description"),
            "default": _plain(item.default),
        }
        if value_type in ("integer", "number"):
            entry.update(kind="number", min=item.metadata.get("min"), max=item.metadata.get("max"), step=item.metadata.get("step"))
        elif value_type == "boolean":
            entry["kind"] = "boolean"
        elif value_type == "text":
            entry.update(kind="text_area", required=item.metadata.get("required", True))
        else:
            raise RegistrationError(f"unsupported setting type {hints[item.name]}")
        result.append(entry)
    return result


def _settings_snapshot(kind: type, values: Mapping[str, Any]) -> Any:
    if not dataclasses.is_dataclass(kind):
        raise RegistrationError("settings declarations must be dataclasses")
    if _meta(kind)["kind"] == "settings":
        return kind(**{name: _settings_snapshot(section_type, values) for name, section_type in get_type_hints(kind).items()})
    hints = get_type_hints(kind)
    return kind(**{item.name: _decode(hints[item.name], values[item.name]) for item in dataclasses.fields(kind)})


def _record_spec(kind: type) -> dict[str, Any]:
    metadata = _meta(kind, "record")
    fields = [
        {key: value for key, value in column.items() if key not in ("role", "display_unit")} | {"choices": []} for column in _columns(kind)
    ]
    return {
        "record_type": metadata["id"],
        "schema_version": metadata["schema"],
        "metadata": _metadata(metadata["id"], metadata["label"], metadata["description"]),
        "template_id": None,
        "owner": None,
        "default_severity": metadata["severity"],
        "fields": fields,
        "validation_mode": None,
        "allow_extra_fields": False,
    }


def _prompt_spec(kind: type) -> dict[str, Any]:
    metadata = _meta(kind, "prompt")
    fields = []
    for column in _columns(kind):
        fields.append(
            {
                "metadata": column["metadata"],
                "value_type": column["value_type"],
                "kind": {"number": "float", "integer": "int", "boolean": "boolean"}.get(column["value_type"], "text"),
                "multiple": False,
                "required": column["required"],
                "default": None,
            }
        )
    actions = list(metadata["actions"]) or [
        {"id": "submit", "label": "Submit", "emphasis": "primary"},
        {"id": "cancel", "label": "Cancel", "emphasis": "secondary"},
    ]
    return {
        "prompt_type": metadata["id"],
        "schema_version": 1,
        "archetype": metadata["archetype"],
        "title": metadata["label"],
        "message": metadata["message"],
        "fields": fields,
        "actions": actions,
        "rules": [],
        "condition": None,
        "owner": None,
        "template_identifier": None,
    }


def _wrap(
    function: Callable[..., Any] | None,
    prepare: Callable[..., Any],
    *,
    finish: Callable[..., Any] | None = None,
    failed: Callable[..., Any] | None = None,
    final: Callable[..., Any] | None = None,
) -> Callable[..., Any]:
    if inspect.iscoroutinefunction(function):

        async def asynchronous(native: _native.CallbackContext, payload: Any) -> Any:
            try:
                target, arguments = prepare(native, payload)
                value = await target(*arguments)
                if native.cancelled:
                    raise RuntimeError("callback context is closed")
                if finish:
                    finish(native, value)
                    return None
                return _plain(value)
            except BaseException:
                if failed:
                    failed(native)
                raise
            finally:
                if final:
                    final(native)

        return asynchronous

    def synchronous(native: _native.CallbackContext, payload: Any) -> Any:
        try:
            target, arguments = prepare(native, payload)
            value = target(*arguments)
            if native.cancelled:
                raise RuntimeError("callback context is closed")
            if finish:
                finish(native, value)
                return None
            return _plain(value)
        except BaseException:
            if failed:
                failed(native)
            raise
        finally:
            if final:
                final(native)

    return synchronous


# === Entry Point === #
