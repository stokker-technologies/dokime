# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import Callable, Generator, Mapping, Sequence
from dataclasses import dataclass
from typing import Any, ClassVar, Literal, NotRequired, Self, TypedDict, TypeVar, dataclass_transform

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
    label: NotRequired[str]
    color: NotRequired[str]
    value: ReferenceValue

class ReferenceBand(TypedDict, total=False):
    label: str
    color: str
    min: ReferenceValue
    max: ReferenceValue
    center: ReferenceValue
    tolerance: ReferenceValue

class RegistrationError(ValueError): ...

@dataclass(frozen=True)
class RetryPolicy:
    max_retries: int = 0
    delay_s: float = 0.0
    jitter: Literal["none", "deterministic"] = "none"

@dataclass(frozen=True)
class OperationPolicy:
    timeout_s: float
    attempt_timeout_s: float
    cancellation_timeout_s: float = 0.0
    retries: RetryPolicy = ...

@dataclass(frozen=True)
class PromptAnswer:
    action: str
    accepted: bool
    values: Mapping[str, Any]

class Invocation[T]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]
    def __init__(self, context: Context, request: Any, retry: RetryPolicy | None = None) -> None: ...
    def retry(self, policy: RetryPolicy) -> Self: ...
    async def start(self) -> OperationInstance[T]: ...
    def __await__(self) -> Generator[Any, None, T]: ...

class OperationInstance[T]:
    __type_params__: ClassVar[tuple[TypeVar, ...]]
    def __init__(self, inner: _native.OperationInstance, response: type[T]) -> None: ...
    @property
    def id(self) -> int: ...
    def request_cancel(self) -> None: ...
    async def join(self) -> T: ...
    async def cancel(self) -> T: ...

class StreamWriter:
    def __init__(self, context: Context, declaration: type, row: type) -> None: ...
    async def push(self, row: Any) -> None: ...
    async def push_columns(self, columns: Mapping[str, Any]) -> None: ...

class StreamSelection:
    def __init__(self, context: Context, row: type) -> None: ...
    def into(self, declaration: type) -> StreamWriter: ...
    def open(self) -> StreamWriter: ...

class Document:
    def __init__(self, context: Context) -> None: ...
    async def section(self, identifier: str, label: str, *, cards: Sequence[Mapping[str, Any]] = ()) -> None: ...

class Context:
    parameters: Any
    settings: Any
    resources: Any
    tick: Any
    reason: str | None
    def __init__(
        self,
        inner: _native.CallbackContext,
        *,
        parameters: Any = None,
        settings: Any = None,
        resources: Any = None,
        registered: Mapping[type, Any] | None = None,
        tick: Any = None,
    ) -> None: ...
    @property
    def execution(self) -> str | None: ...
    @property
    def document(self) -> Document: ...
    @property
    def cancelled(self) -> bool: ...
    def resource[T](self, resource: type[T]) -> T: ...
    def operation[T](self, request: Any) -> Invocation[T]: ...
    def stream(self, row: type) -> StreamSelection: ...
    def evidence(self) -> Mapping[str, Any]: ...
    async def log(self, message: str, *, level: Literal["trace", "debug", "info", "warn", "error"] = "info") -> None: ...
    async def publish(self, value: Any) -> None: ...
    async def ask(self, value: Any) -> PromptAnswer: ...

class Plugin:
    id: str
    version: str
    description: str
    catalog: tuple[Any, ...]
    def __init__(
        self,
        id: str,
        version: str,
        description: str = "",
        *,
        catalog: Sequence[Any],
        resources: Mapping[type, Any] | None = None,
        requires: Sequence[type] = (),
    ) -> None: ...
    def _bind(self) -> _native.EmbeddedProvider: ...

# === Public Functions === #

def parameter(
    *,
    label: str | None = None,
    description: str | None = None,
    default: Any = ...,
    required: bool = True,
    min: float | None = None,
    max: float | None = None,
    **metadata: Any,
) -> Any: ...
@dataclass_transform(field_specifiers=(parameter,))
def parameters[T](kind: type[T]) -> type[T]: ...
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
) -> Callable[[type[T]], type[T]]: ...
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
) -> Callable[[type[T]], type[T]]: ...
def suite[T](
    *, id: str, label: str, templates: Sequence[type], tags: Sequence[str] = (), description: str | None = None
) -> Callable[[type[T]], type[T]]: ...
def step(
    *, id: str | None = None, label: str | None = None, description: str | None = None, repeatable: bool = False
) -> Callable[[Callable[..., Any]], Callable[..., Any]]: ...
def criterion(
    *, id: str | None = None, label: str | None = None, description: str | None = None, expected: Any = None
) -> Callable[[Callable[..., Any]], Callable[..., Any]]: ...
def observer(
    *, id: str | None = None, label: str | None = None, interval_ms: int, streams: Sequence[type] = ()
) -> Callable[[Callable[..., Any]], Callable[..., Any]]: ...
@dataclass_transform(field_specifiers=(parameter,))
def stream[T](
    *, id: str, label: str, window: int, row: type | None = None, description: str | None = None
) -> Callable[[type[T]], type[T]]: ...
@dataclass_transform(field_specifiers=(parameter,))
def record[T](
    *, id: str, label: str, schema: int = 1, severity: Literal["debug", "info", "warning", "error"] = "info", description: str | None = None
) -> Callable[[type[T]], type[T]]: ...
@dataclass_transform(field_specifiers=(parameter,))
def prompt[T](
    *, id: str, label: str, message: str | None = None, archetype: str = "FormPrompt", actions: Sequence[Mapping[str, Any]] = ()
) -> Callable[[type[T]], type[T]]: ...
@dataclass_transform(field_specifiers=(parameter,))
def section[T](*, label: str, description: str | None = None, status: type | None = None) -> Callable[[type[T]], type[T]]: ...
@dataclass_transform(field_specifiers=(parameter,))
def settings[T](kind: type[T]) -> type[T]: ...

# === Private Functions === #

# === Entry Point === #
