# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Execution context helpers for authored templates."""

from __future__ import annotations

# === Imports === #

# Standard Library Imports

from typing import Any, Literal, Self
from collections.abc import AsyncIterator, Sequence
from contextlib import AbstractAsyncContextManager, AsyncExitStack

# Third Party Imports

# Local Imports

from dokime.core.authoring.artifacts import Artifacts
from dokime.core.authoring.prompts import declared_prompt, _wire
from dokime.core.authoring.publications import decode
from dokime.core.authoring.parameters import _value_from_contract

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #


# === Classes === #


class TestContext:
    """One Rust-owned execution stage; all effects are asynchronous."""

    def __init__(
        self,
        native: Any,
        *,
        plugin: object = None,
        resources: object = None,
        reason: str | None = None,
        acquisitions: AsyncExitStack | None = None,
    ) -> None:
        self._native = native
        self.parameters = {
            name: _value_from_contract(value)
            for name, value in native.parameters.items()
        }
        self.plugin = plugin
        self.resources = resources
        self.reason = reason
        self._acquisitions = acquisitions

    async def acquire[T](self, resource: AbstractAsyncContextManager[T]) -> T:
        """Initialize one owned resource; failed initialization awaits partial rollback."""
        if self._acquisitions is None:
            raise RuntimeError(
                "resource acquisition is available only during initialization"
            )
        return await self._acquisitions.enter_async_context(resource)

    @property
    def artifacts(self) -> Artifacts:
        return Artifacts(self._native.artifacts)

    async def trace(self, message: str) -> None:
        await self._log("trace", message)

    async def debug(self, message: str) -> None:
        await self._log("debug", message)

    async def info(self, message: str) -> None:
        await self._log("info", message)

    async def warning(self, message: str) -> None:
        await self._log("warning", message)

    async def error(self, message: str) -> None:
        await self._log("error", message)

    async def _log(self, level: str, message: str) -> None:
        await self._native.request("log", {"level": level, "message": message})

    async def ask(self, prompt: object) -> object:
        if isinstance(prompt, type):
            raise TypeError("ask expects a prompt instance, for example Confirm()")
        declaration = declared_prompt(type(prompt))
        values = {
            field.name: _wire(getattr(prompt, field.name))
            for field in declaration.fields
        }
        response = await self._native.request(
            "ask",
            {
                "definition": declaration.definition(),
                "values": values,
            },
        )
        response["values"] = {
            name: _value_from_contract(value)
            for name, value in response.get("values", {}).items()
        }
        return declaration.decode(response)

    async def publish(self, value: object) -> None:
        publication = getattr(value, "__dokime_publication__", None)
        if publication is None:
            raise TypeError(
                "publish expects an instance of a declared record or stream row"
            )
        kind, payload = publication()
        await self._native.request(f"publish_{kind}", payload)

    async def progress[T](self, phase: str, items: Sequence[T]) -> AsyncIterator[T]:
        """Yield bounded work; Rust advances only after each body completes."""
        identifier = await self._native.request(
            "progress", {"phase": phase, "total": len(items)}
        )
        try:
            while (
                index := await self._native.request("progress_next", identifier)
            ) is not None:
                yield items[index]
        finally:
            await self._native.request("progress_close", identifier)

    async def lease[T](self, row_type: type[T]) -> Lease[T]:
        """Hold a producer's stream until close or the end of this step."""
        declaration = getattr(row_type, "__dokime_stream__")
        identifier = await self._native.request("lease", declaration.stream)
        return Lease(self._native, identifier, row_type)

    async def listen[T](self, item_type: type[T]) -> Listening[T]:
        """Subscribe to live rows or this execution's replayable record inbox."""
        return await _listen(self._native, item_type)

    async def samples(
        self,
        row_type: type,
        *,
        origin: Literal["default", "initialization", "cleanup"] | None = None,
    ) -> list:
        declaration = getattr(row_type, "__dokime_stream__")
        evidence = (
            self._native.evidence
            if origin is None
            else await self._native.request("criterion_evidence", origin)
        )
        rows = evidence["samples"].get(declaration.stream, [])
        return [decode(row_type, row["values"]) for row in rows]

    async def records(
        self,
        record_type: type,
        *,
        origin: Literal["default", "initialization", "cleanup"] | None = None,
    ) -> list:
        declaration = record_type.__dokime_record__
        evidence = (
            self._native.evidence
            if origin is None
            else await self._native.request("criterion_evidence", origin)
        )
        return [
            decode(record_type, record["details"])
            for record in evidence["subscribed_records"]
            if record["record_type"] == declaration.record_type
        ]

    async def steps_reached(self) -> list[str]:
        return list(self._native.evidence["steps"])


class Listening[T]:
    """A bounded Rust subscription; use async iteration and explicit close."""

    def __init__(self, native: Any, identifier: int, item_type: type[T]) -> None:
        self._native = native
        self._identifier = identifier
        self._item_type = item_type
        self._closed = False

    def __aiter__(self) -> Self:
        return self

    async def __anext__(self) -> T:
        if self._closed:
            raise StopAsyncIteration
        values = await self._native.request("listen_next", self._identifier)
        if values is None:
            self._closed = True
            raise StopAsyncIteration
        return decode(self._item_type, values)

    async def close(self) -> None:
        if not self._closed:
            await self._native.request("listen_close", self._identifier)
            self._closed = True


class Lease[T]:
    """A step-owned stream hold; closing retains all evidence already measured."""

    def __init__(self, native: Any, identifier: int, row_type: type[T]) -> None:
        self._native = native
        self._identifier = identifier
        self._row_type = row_type
        self._closed = False

    async def listen(self) -> Listening[T]:
        if self._closed:
            raise RuntimeError("lease is closed")
        return await _listen(self._native, self._row_type)

    async def close(self) -> None:
        if not self._closed:
            await self._native.request("lease_close", self._identifier)
            self._closed = True


async def _listen[T](native: Any, item_type: type[T]) -> Listening[T]:
    if declaration := getattr(item_type, "__dokime_stream__", None):
        selection = {"kind": "row", "id": declaration.stream}
    elif declaration := getattr(item_type, "__dokime_record__", None):
        selection = {"kind": "record", "id": declaration.record_type}
    else:
        raise TypeError("listen expects a declared row or record type")
    identifier = await native.request("listen", selection)
    return Listening(native, identifier, item_type)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
