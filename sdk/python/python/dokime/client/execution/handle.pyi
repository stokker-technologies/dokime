# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncGenerator, Generator, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from .. import prompts
from .._transport import Transport
from ..files import FileIdentity, FileUpdate
from ..history import Checkpoint, Download
from ..models import (
    Backend,
    CommandAction,
    ExecutionCommandReceipt,
    Prompt,
    Record,
    ResultSummary,
    Sample,
    Value,
)
from ..planning import TestCase
from ..query.evidence import execution_records, telemetry

# Type Checking Imports

# === Public Exports === #

# === Constants === #

CHECKPOINT_BYTES: int
REQUEST_POLL_MS: int
SAMPLE_PAGE_ITEMS: int
VALUE_DEPTH: int

# === Types === #

# === Classes === #

@dataclass(frozen=True)
class ExecutionOutcome:
    checkpoint: Checkpoint
    status: str
    result: ResultSummary

    @property
    def exit_code(self) -> int: ...

class Execution:
    _transport: Transport
    __slots__ = ("_backend", "_case", "_cycle_id", "_id", "_outcome", "_transport")

    query_records = execution_records
    telemetry = telemetry

    def __init__(self, test_case: TestCase) -> None: ...
    @classmethod
    def _bind(cls, execution_id: str, transport: Transport, backend: Backend) -> Execution: ...
    def _require_bound(self) -> None: ...
    @property
    def id(self) -> str: ...
    @property
    def outcome(self) -> ExecutionOutcome: ...
    def __await__(self) -> Generator[Any, None, ExecutionOutcome]: ...
    async def export(self, destination: str | Path, *, overwrite: bool = False) -> Download: ...
    async def snapshot(self) -> dict[str, Any]: ...
    async def _prompt_scope(self) -> prompts.Scope: ...
    def pending_prompts(self) -> AsyncGenerator[Prompt]: ...
    async def prompt(self, prompt_id: str) -> dict[str, Any]: ...
    async def respond(self, prompt_id: str, *, action: str, values: Mapping[str, Value] | None = None, accepted: bool = True) -> None: ...
    async def _snapshot(self) -> Message: ...
    async def checkpoint(self) -> Checkpoint | None: ...
    async def _document_at(self, point: Any) -> Message: ...
    async def wait(self, *, timeout: float | None = None, poll_interval: float | None = None) -> ExecutionOutcome: ...
    async def start(
        self,
        *,
        mode: Literal["automatic", "interactive"] = "automatic",
    ) -> None: ...
    def follow_file(
        self,
        relative_path: str,
        destination: str | Path,
        *,
        identity: FileIdentity | None = None,
        max_bytes: int | None = None,
    ) -> AsyncGenerator[FileUpdate]: ...
    def records(
        self,
        *types: str,
        record_ids: Sequence[str] = (),
        poll_interval: float | None = None,
    ) -> AsyncGenerator[Record]: ...
    def samples(
        self,
        *streams: str,
        page_size: int | None = None,
    ) -> AsyncGenerator[Sample]: ...
    async def next(self, *, request_id: str, command_id: str) -> ExecutionCommandReceipt: ...
    async def run_step(self, step_id: str, *, request_id: str, command_id: str) -> ExecutionCommandReceipt: ...
    async def finish(self, *, request_id: str, command_id: str) -> ExecutionCommandReceipt: ...
    async def _command(
        self,
        request_id: str,
        command_id: str,
        action: CommandAction,
        step_id: str | None = None,
    ) -> ExecutionCommandReceipt: ...
    async def command_receipt(self, request_id: str, command_id: str) -> ExecutionCommandReceipt: ...
    async def _control(self, method: str) -> None: ...
    async def pause(self) -> None: ...
    async def resume(self) -> None: ...
    async def stop(self) -> None: ...

def _admission(backend: Backend, reply: Message) -> Message: ...
def _command_receipt(
    execution_id: str,
    request_id: str,
    command_id: str,
    receipt: Message,
) -> None: ...
