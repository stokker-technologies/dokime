# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator, AsyncIterator, Mapping
from contextlib import asynccontextmanager
from pathlib import Path
from types import TracebackType
from typing import Any, Self

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from ..operation import Operation, ResponseStream
from ._transport import DiscoveredOperation, Transport
from .execution import Cycle, Execution
from .models import Backend, Suite
from .planning import Template
from .query.discovery import Alternatives, CycleStatuses, CycleSummary, ExecutionStatuses, ExecutionSummary
from .query.reader import QueryReader, QuerySettings
from .query.schemas import RecordSchema
from .query.selection import At, Time, Where, Window

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PAGE_ITEMS: int

# === Types === #

# === Classes === #

class DokimeClient:
    _transport: Transport
    def __init__(
        self,
        endpoint: str = "http://127.0.0.1:8765",
        *,
        rpc_timeout: float | None = None,
        preferences: Message | None = None,
        expected_backend_id: str | None = None,
        ca_certificate_pem: str | None = None,
        access_key: str | None = None,
        endpoints: tuple[str, ...] = (),
        query_settings: QuerySettings | None = None,
    ) -> None: ...
    @classmethod
    def from_pairing(
        cls,
        path: str | Path,
        *,
        preferences: Message | None = None,
        rpc_timeout: float | None = None,
        query_settings: QuerySettings | None = None,
    ) -> Self: ...
    @classmethod
    def from_profile(
        cls,
        identifier: str,
        *,
        path: str | Path | None = None,
        rpc_timeout: float | None = None,
        preferences: Message | None = None,
        query_settings: QuerySettings | None = None,
    ) -> Self: ...
    def executions(
        self,
        *,
        template: Alternatives = None,
        cycle: Alternatives = None,
        status: ExecutionStatuses = None,
        since: Time | None = None,
        until: Time | None = None,
        archive: str = "unarchived",
        where: Where | None = None,
        between: Window | None = None,
        at: At = None,
        page_rows: int | None = None,
        page_bytes: int | None = None,
    ) -> QueryReader[ExecutionSummary]: ...
    def cycles(
        self,
        *,
        cycle: Alternatives = None,
        phase: Alternatives = None,
        status: CycleStatuses = None,
        since: Time | None = None,
        until: Time | None = None,
        archive: str = "unarchived",
        where: Where | None = None,
        between: Window | None = None,
        at: At = None,
        page_rows: int | None = None,
        page_bytes: int | None = None,
    ) -> QueryReader[CycleSummary]: ...
    def record_schemas(self, *, page_rows: int = ...) -> AsyncGenerator[RecordSchema]: ...
    async def connect(self) -> Backend: ...
    async def operations(self) -> tuple[DiscoveredOperation, ...]: ...
    async def show(self, identifier: str, *, owner: str | None = None) -> DiscoveredOperation: ...
    async def call(
        self,
        operation: Operation | str,
        request: Message | Mapping[str, Any],
        *,
        owner: str | None = None,
    ) -> Message: ...
    async def upload(self, operation: Operation, request: Message, items: AsyncIterator[Message]) -> Message: ...
    def download(self, operation: Operation, request: Message) -> ResponseStream: ...
    def exchange(self, operation: Operation, request: Message, items: AsyncIterator[Message]) -> ResponseStream: ...
    async def close(self) -> None: ...
    async def __aenter__(self) -> Self: ...
    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        traceback: TracebackType | None,
    ) -> None: ...
    def suites(self) -> AsyncIterator[Suite]: ...
    def _suite_identities(self): ...
    async def _suite_by_id(self, identifier: str) -> Message: ...
    async def _suite_definition(self, identifier: str) -> Message: ...
    async def suite(self, suite_id: str) -> Suite: ...
    async def template(self, identifier: str) -> Template: ...
    async def create_cycle(
        self,
        *,
        label: str,
        description: str | None = None,
        identifier: str | None = None,
        continue_automatically: bool = False,
    ) -> Cycle: ...
    @asynccontextmanager
    def new_cycle(
        self,
        label: str,
        *,
        description: str | None = None,
        identifier: str | None = None,
        continue_automatically: bool = False,
    ) -> AsyncIterator[Cycle]: ...
    async def cycle(self, cycle_id: str) -> Cycle: ...
    async def execution(self, execution_id: str) -> Execution: ...

def _client_preferences() -> Message: ...
def _readable(value: str) -> bool: ...
def _suite(item: Message) -> Suite: ...
