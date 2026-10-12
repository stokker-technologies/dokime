# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Local capture file ownership and immutable evidence export."""

# === Imports === #

# Standard Library Imports
import base64
from collections.abc import AsyncIterator, Iterator
from contextlib import contextmanager
from dataclasses import dataclass, field, asdict
from datetime import datetime
import json
import os
from pathlib import Path
import re
from typing import Any, TextIO

# Third Party Imports

# Local Imports
from ..history import ExecutionEvidence
from ..models import Record, Sample, SavedRecord, SavedSample

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass
class CaptureFiles:
    root: Path
    state: dict[str, Any]
    files: dict[str, TextIO] = field(default_factory=dict)

    @contextmanager
    def lease(self) -> Iterator[None]:
        lock = self.root / ".collect.lock"
        with lock.open("x", encoding="utf-8") as handle:
            handle.write(str(os.getpid()))
        try:
            yield
        finally:
            lock.unlink()

    def save(self) -> None:
        write_json(self.root / "capture.json", self.state)

    def execution_root(self, identifier: str) -> Path:
        path = self.root / "executions" / component(identifier)
        path.mkdir(parents=True, exist_ok=True)
        return path

    def open_live(self) -> None:
        for kind in ("records", "streams"):
            self.files[kind] = (self.root / f"{kind}.jsonl.part").open(
                "w", encoding="utf-8"
            )

    def append(self, kind: str, item: Record | Sample) -> None:
        output = self.files[kind]
        output.write(
            json.dumps(asdict(item), default=json_default, allow_nan=False) + "\n"
        )
        output.flush()

    def finish_live(self) -> None:
        self.close()
        for kind in ("records", "streams"):
            (self.root / f"{kind}.jsonl.part").replace(self.root / f"{kind}.jsonl")

    def close(self) -> None:
        for output in self.files.values():
            output.close()
        self.files.clear()


# === Public Functions === #


async def save_execution(execution: ExecutionEvidence, capture: CaptureFiles) -> None:
    root = capture.execution_root(execution.checkpoint.execution_id)
    write_json(root / "execution.json", await execution.document())
    write_json(
        root / "traceability.json",
        [asdict(item) for item in await execution.traceability()],
    )
    write_json(root / "schemas.json", await execution.schemas())
    await write_records(root / "records.jsonl", execution.records())
    await write_records(root / "streams.jsonl", execution.samples())
    artifacts = await execution.artifacts()
    artifact_root = root / "artifacts"
    artifact_root.mkdir(exist_ok=True)
    write_json(artifact_root / "manifest.json", [asdict(item) for item in artifacts])
    for artifact in artifacts:
        path = artifact_root / component(artifact.id)
        path.with_name(path.name + ".part").unlink(missing_ok=True)
        await execution.download_artifact(artifact, path)


async def write_records(
    path: Path,
    records: AsyncIterator[Record]
    | AsyncIterator[SavedRecord]
    | AsyncIterator[SavedSample]
    | AsyncIterator[Sample],
) -> None:
    partial = path.with_name(path.name + ".part")
    with partial.open("w", encoding="utf-8") as output:
        async for record in records:
            output.write(
                json.dumps(asdict(record), default=json_default, allow_nan=False) + "\n"
            )
    partial.replace(path)


# === Private Functions === #


def component(value: str) -> str:
    # Server identities are names, never paths. Original filenames live only in
    # the manifest, so an uploaded filename cannot choose a local destination.
    if not re.fullmatch(r"[A-Za-z0-9_-]+", value) or value.upper() in {
        "CON",
        "PRN",
        "AUX",
        "NUL",
        *(f"COM{i}" for i in range(10)),
        *(f"LPT{i}" for i in range(10)),
    }:
        raise ValueError("unsafe evidence identifier")
    return value


def json_default(value: object) -> str:
    if isinstance(value, bytes):
        return base64.b64encode(value).decode("ascii")
    if isinstance(value, datetime):
        return value.isoformat()
    if isinstance(value, Path):
        return str(value)
    raise TypeError(f"cannot encode {type(value).__name__}")


def write_json(path: Path, value: object) -> None:
    partial = path.with_name(path.name + ".part")
    with partial.open("w", encoding="utf-8") as output:
        json.dump(value, output, default=json_default, indent=2, allow_nan=False)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())
    partial.replace(path)


# === Entry Point === #
