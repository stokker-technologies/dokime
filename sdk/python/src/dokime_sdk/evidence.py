# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Finite Evidence Queries, Optional Dataframes and Evidence Archive Exports."""

# === Imports === #

# Standard Library Imports
import os
from collections.abc import AsyncIterator, Sequence
from dataclasses import dataclass, field
from datetime import UTC, datetime, timedelta
from hashlib import sha256
from io import FileIO
from pathlib import Path
from tempfile import mkstemp
from typing import TYPE_CHECKING

# Third Party Imports
from harmos import FileReference, PageRequest, ReadCut

# Local Imports
from dokime_sdk import EvidenceScope, model

from .client import (
    EXECUTION_STATE_CLOSED,
    EXECUTION_STATE_OPEN,
    EXECUTION_STATE_SEALED,
    BucketRequest,
    ClientRoot,
    EvidenceSubject,
    ExecutionDetail,
    ExecutionFilter,
    ExecutionOrder,
    ExecutionSelection,
)
from .values import Scalar, decode

# Type Checking Imports
if TYPE_CHECKING:
    import pandas
    import polars

# === Public Exports === #

__all__ = ["Cycle", "Execution", "Telemetry", "TelemetryRow", "cycle", "execution", "executions"]

# === Constants === #

DEFAULT_MAX_ROWS = 100_000
STATUSES = {"open": EXECUTION_STATE_OPEN, "closed": EXECUTION_STATE_CLOSED, "sealed": EXECUTION_STATE_SEALED}
EPOCH = datetime(1970, 1, 1, tzinfo=UTC)

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class TelemetryRow:
    """One Saved Sample with Its Exact Source Time and Decoded Field Values."""

    sequence: int
    timestamp_ns: int
    values: dict[str, Scalar] = field(default_factory=dict)

    @property
    def timestamp(self) -> datetime:
        # Python Datetimes Hold Microseconds; Use timestamp_ns for the Exact Source Time.
        return EPOCH + timedelta(microseconds=self.timestamp_ns // 1_000)


class Telemetry:
    """A Finite Read of One Stream at One Saved Cut; It Never Follows Later Samples."""

    def __init__(
        self,
        client: ClientRoot,
        execution_id: str,
        stream: str,
        fields: Sequence[str],
        between: model.Window | None,
        at: ReadCut | None,
        page_size: int,
    ) -> None:
        self._client = client
        self._execution_id = execution_id
        self._stream = stream
        self._fields = list(fields)
        self._between = between or model.Window()
        self.snapshot = at
        self._page_size = page_size
        # Wire Kinds Seen per Field Choose Exact Dataframe Types; Python Magnitudes Cannot.
        self._kinds: dict[str, set[str]] = {}

    async def pages(self) -> AsyncIterator[list[TelemetryRow]]:
        token = ""
        while True:
            page = await self._client.evidence.samples.list(
                EvidenceScope(execution_id=self._execution_id),
                [self._stream],
                [],
                [],
                [],
                self._fields,
                self._between,
                BucketRequest(),
                False,
                at=self.snapshot,
                page=PageRequest(page_size=self._page_size, page_token=token),
            )
            if self.snapshot is None:
                self.snapshot = page.resolved_at
            rows = []
            for sample in page.samples:
                if sample.WhichOneof("kind") != "raw" or sample.raw.stream != self._stream:
                    raise ValueError("Read Telemetry: The Host Returned a Sample Outside the Selected Stream")
                raw = sample.raw
                values = {}
                for name, value in raw.values.values.items():
                    self._kinds.setdefault(name, set()).add(value.WhichOneof("kind") or "null")
                    values[name] = decode(value)
                rows.append(TelemetryRow(raw.sequence, raw.timestamp.seconds * 1_000_000_000 + raw.timestamp.nanos, values))
            yield rows
            token = page.next_page_token
            if not token:
                return

    async def collect(self, *, max_rows: int = DEFAULT_MAX_ROWS) -> list[TelemetryRow]:
        if max_rows <= 0:
            raise ValueError("Read Telemetry: Supply a Positive Row Limit")
        collected: list[TelemetryRow] = []
        async for rows in self.pages():
            collected.extend(rows)
            if len(collected) > max_rows:
                raise ValueError(f"Read Telemetry: More Than {max_rows} Rows; Narrow the Window or Raise max_rows")
        return collected

    async def to_polars(self, *, max_rows: int = DEFAULT_MAX_ROWS) -> "polars.DataFrame":
        try:
            import polars
        except ImportError as error:
            raise ImportError("Read Telemetry: Install the dokime-sdk[polars] Extra for to_polars") from error
        rows = await self.collect(max_rows=max_rows)
        columns = {
            "sequence": polars.Series("sequence", [row.sequence for row in rows], dtype=polars.UInt64),
            "timestamp": polars.Series("timestamp", [row.timestamp_ns for row in rows], dtype=polars.Int64).cast(
                polars.Datetime("ns", "UTC")
            ),
        }
        for name in sorted(self._kinds):
            values = [row.values.get(name) for row in rows]
            kind = self._kind(name)
            if kind == "unsigned_integer":
                columns[name] = polars.Series(name, values, dtype=polars.UInt64)
            elif kind == "integer":
                columns[name] = polars.Series(name, values, dtype=polars.Int64)
            elif kind == "number":
                columns[name] = polars.Series(name, values, dtype=polars.Float64)
            elif kind == "boolean":
                columns[name] = polars.Series(name, values, dtype=polars.Boolean)
            elif kind == "text":
                columns[name] = polars.Series(name, values, dtype=polars.String)
            elif kind == "data":
                columns[name] = polars.Series(name, values, dtype=polars.Binary)
            elif kind == "datetime":
                columns[name] = polars.Series(name, values, dtype=polars.Datetime("us", "UTC"))
            else:
                # Exact Fractions, Wide Integers and Mixed Kinds Keep Their Python Values.
                columns[name] = polars.Series(name, values, dtype=polars.Object)
        return polars.DataFrame(list(columns.values()))

    async def to_pandas(self, *, max_rows: int = DEFAULT_MAX_ROWS) -> "pandas.DataFrame":
        try:
            import pandas
        except ImportError as error:
            raise ImportError("Read Telemetry: Install the dokime-sdk[pandas] Extra for to_pandas") from error
        rows = await self.collect(max_rows=max_rows)
        columns = {
            "sequence": pandas.Series([row.sequence for row in rows], dtype="UInt64"),
            "timestamp": pandas.Series(pandas.to_datetime([row.timestamp_ns for row in rows], unit="ns", utc=True)),
        }
        for name in sorted(self._kinds):
            values = [row.values.get(name) for row in rows]
            dtype = {
                "unsigned_integer": "UInt64",
                "integer": "Int64",
                "number": "Float64",
                "boolean": "boolean",
                "text": "string",
                "datetime": "datetime64[us, UTC]",
            }.get(self._kind(name), "object")
            columns[name] = pandas.Series(values, dtype=dtype)
        return pandas.DataFrame(columns)

    def _kind(self, name: str) -> str:
        kinds = self._kinds[name] - {"null"}
        return kinds.pop() if len(kinds) == 1 else "object"


class Cycle:
    """A Cycle Selected by Identifier for Archive Export."""

    def __init__(self, client: ClientRoot, identifier: str) -> None:
        self.client = client
        self.id = identifier

    async def export(self, path: str | Path, *, overwrite: bool = False) -> Path:
        return await _export(self.client, EvidenceSubject(cycle=self.id), Path(path), overwrite)


class Execution:
    """A Retained Execution with Finite Telemetry Reads and Archive Export."""

    def __init__(self, client: ClientRoot, detail: ExecutionDetail) -> None:
        self.client = client
        self.detail = detail

    @property
    def id(self) -> str:
        return self.detail.summary.execution_id

    @property
    def cycle(self) -> Cycle:
        return Cycle(self.client, self.detail.cycle_id)

    def telemetry(
        self,
        stream: str,
        *,
        fields: Sequence[str] = (),
        between: model.Window | None = None,
        at: ReadCut | None = None,
        page_size: int = 0,
    ) -> Telemetry:
        return Telemetry(self.client, self.id, stream, fields, between, at, page_size)

    async def export(self, path: str | Path, *, overwrite: bool = False) -> Path:
        return await _export(self.client, EvidenceSubject(execution=self.id), Path(path), overwrite)


# === Public Functions === #


async def executions(
    client: ClientRoot, *, template: str | None = None, status: str | None = None, cycle: str | None = None
) -> AsyncIterator[Execution]:
    """Yield Retained Executions at One Fixed Cut, Without Starting or Following Any."""
    if status is not None and status not in STATUSES:
        raise ValueError(f"List Executions: Choose a Status from {', '.join(STATUSES)}")
    selection = ExecutionSelection(
        template_ids=[template] if template else [],
        cycle_ids=[cycle] if cycle else [],
        statuses=[STATUSES[status]] if status else [],
    )
    at: ReadCut | None = None
    token = ""
    while True:
        page = await client.executions.list("", selection, ExecutionFilter(), ExecutionOrder(), at=at, page=PageRequest(page_token=token))
        at = page.resolved_at
        for detail in page.executions:
            yield Execution(client, detail)
        token = page.next_page_token
        if not token:
            return


async def execution(client: ClientRoot, identifier: str) -> Execution:
    return Execution(client, (await client.executions.get(identifier)).execution)


def cycle(client: ClientRoot, identifier: str) -> Cycle:
    return Cycle(client, identifier)


# === Private Functions === #


async def _export(client: ClientRoot, subject: EvidenceSubject, path: Path, overwrite: bool) -> Path:
    # Refuse an Existing Destination Before Asking the Host to Build the Archive.
    if path.exists() and not overwrite:
        raise FileExistsError(f"Export Evidence: {path} Exists; Choose a New Path or Pass overwrite=True")
    archive = await client.evidence.archive(subject)
    if not archive.HasField("file"):
        raise ValueError("Export Evidence: The Host Returned No Archive File")
    descriptor, partial = mkstemp(prefix=f".{path.name}.", suffix=".partial", dir=path.parent)
    target = FileIO(descriptor, "w")
    try:
        try:
            length = await _download(client, archive.file, target)
            target.flush()
            os.fsync(target.fileno())
        finally:
            target.close()
        if length != archive.file.length:
            raise ValueError("Export Evidence: Downloaded Length Differs from the Archive")
        if overwrite:
            os.replace(partial, path)
        else:
            # Linking Publishes Atomically and Refuses a Destination Created Meanwhile.
            os.link(partial, path)
            os.unlink(partial)
        return path
    except BaseException:
        Path(partial).unlink(missing_ok=True)
        raise


async def _download(client: ClientRoot, file: FileReference, target: FileIO) -> int:
    limits = await client.files.get(file)
    if limits.chunk_bytes <= 0:
        raise ValueError("Export Evidence: The Host Returned No Transfer Chunk Bound")
    digest = sha256()
    offset = 0
    while offset < file.length:
        page = await client.files.read(file, offset, limits.chunk_bytes)
        if page.total_length != file.length or page.sha256 != file.sha256:
            raise ValueError("Export Evidence: The Archive Changed During Download")
        if not page.data or offset + len(page.data) > file.length:
            raise ValueError("Export Evidence: Invalid Archive Page")
        target.write(page.data)
        digest.update(page.data)
        offset += len(page.data)
    if digest.digest() != file.sha256:
        raise ValueError("Export Evidence: Archive Digest Mismatch")
    return offset


# === Entry Point === #
