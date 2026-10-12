# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Async access to Rust-owned execution artifacts."""

# === Imports === #

# Standard Library Imports
import asyncio
from collections.abc import Callable
from os import PathLike
from pathlib import Path
from typing import TYPE_CHECKING

# Third Party Imports

# Local Imports

# Type Checking Imports
if TYPE_CHECKING:
    from dokime import _native

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Artifacts:
    """The execution's shared artifact grant, including watched directories."""

    def __init__(self, native: "_native.Artifacts") -> None:
        self._native = native

    async def create(self, name: str) -> "ArtifactWriter":
        return ArtifactWriter(self._native.create(name))

    async def write(self, name: str, data: bytes) -> None:
        writer = await self.create(name)
        try:
            await writer.write_all(data)
            await writer.finish()
        except BaseException:
            await writer.abandon()
            raise

    async def directory(self, name: str) -> Path:
        return Path(await _joined(lambda: self._native.directory(name)))

    async def import_file(self, path: str | PathLike[str], name: str) -> None:
        await _joined(lambda: self._native.import_file(path, name))


class ArtifactWriter:
    """A Rust writer that can be retained across this execution's phases."""

    def __init__(self, native: "_native.ArtifactWriter") -> None:
        self._native = native
        self._lock = asyncio.Lock()

    async def write_all(self, data: bytes) -> None:
        async with self._lock:
            await _joined(lambda: self._native.write_all(data))

    async def flush(self) -> None:
        async with self._lock:
            self._native.flush()

    async def finish(self) -> None:
        async with self._lock:
            self._native.finish()

    async def abandon(self) -> None:
        async with self._lock:
            self._native.abandon()


# === Public Functions === #

# === Private Functions === #


async def _joined[T](operation: Callable[[], T]) -> T:
    # Python cancellation cannot stop a native file operation. Keep the owned
    # callback alive until it joins, before the runtime may release resources.
    task = asyncio.create_task(asyncio.to_thread(operation))
    try:
        return await asyncio.shield(task)
    except asyncio.CancelledError:
        while not task.done():
            try:
                await asyncio.shield(task)
            except asyncio.CancelledError:
                pass
        task.result()
        raise


# === Entry Point === #
