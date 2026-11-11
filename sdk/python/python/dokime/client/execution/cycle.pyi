# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator, Mapping, Sequence
from pathlib import Path
from types import TracebackType
from typing import Any, Protocol, Self

# Third Party Imports
from google.protobuf.message import Message

from .._transport import Transport
from ..history import Download
from ..models import (
    Backend,
    Prompt,
    Record,
    ResultSummary,
    Sample,
    SaveManifest,
    Value,
)

# Local Imports
from ..planning import Template
from ..query.evidence import cycle_records
from .handle import Execution

# Type Checking Imports

# === Public Exports === #

# === Constants === #

MAXIMUM_SAMPLE_PAGE_ITEMS: int
PAGE_ITEMS: int
SAMPLE_PAGE_ITEMS: int
VALUE_DEPTH: int

# === Types === #

# === Classes === #

class _Client(Protocol):
    async def template(self, identifier: str) -> Template: ...

class Cycle:
    _transport: Transport
    query_records = cycle_records

    def __init__(
        self,
        cycle_id: str,
        transport: Transport,
        backend: Backend,
        executions: tuple[Execution, ...],
        *,
        client: _Client | None = None,
    ) -> None: ...
    async def __aenter__(self) -> Self: ...
    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        traceback: TracebackType | None,
    ) -> None: ...
    async def _close(self) -> None: ...
    async def start(self, template: str, **parameters: Value) -> Execution | tuple[Execution, ...]: ...
    async def save(self, directory: str | Path, report: str | None = None) -> SaveManifest: ...
    @property
    def id(self) -> str: ...
    async def append(self, executions: Sequence[Execution]) -> None: ...
    async def export(self, destination: str | Path, *, overwrite: bool = False) -> Download: ...
    @property
    def executions(self) -> tuple[Execution, ...]: ...
    @classmethod
    async def _bind(cls, transport: Transport, cycle_id: str, backend: Backend, *, client: _Client | None = None) -> Cycle: ...
    async def result(self) -> ResultSummary: ...
    async def seal(self) -> None: ...
    async def document(self) -> dict[str, Any]: ...
    async def _snapshot(self) -> Message: ...
    async def download_report(
        self,
        destination: str | Path,
        *,
        identifier: str = "cycle.summary",
    ) -> Download: ...
    def pending_prompts(self) -> AsyncGenerator[Prompt]: ...
    async def respond(
        self,
        prompt_id: str,
        *,
        action: str,
        values: Mapping[str, Value] | None = None,
        accepted: bool = True,
    ) -> None: ...
    def records(
        self,
        *,
        record_types: Sequence[str] = (),
        record_ids: Sequence[str] = (),
        execution_id: str | None = None,
    ) -> AsyncGenerator[Record]: ...
    def samples(
        self,
        *,
        streams: Sequence[str] = (),
        execution_id: str | None = None,
        page_size: int | None = None,
    ) -> AsyncGenerator[Sample]: ...
