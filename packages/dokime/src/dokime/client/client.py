# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Remote discovery and execution submission."""

# === Imports === #

# Standard Library Imports
from collections.abc import AsyncIterator, Mapping
from datetime import datetime
from pathlib import Path
from types import TracebackType
from typing import Any, Self

# Third Party Imports
from google.protobuf.message import Message

from ..operation import Operation, ResponseStream

# Local Imports
from . import _wire
from ._transport import DiscoveredOperation, Transport, _cursor
from .credentials import load_key
from .errors import ProtocolError, RpcError
from .execution import Cycle, RunRequest, TestExecutionHandle
from .execution.request import _identity
from .models import Backend, Suite, Template, Value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DokimeClient:
    """One async connection. Read explicit local inputs before any connection.

    A client is used on one asyncio loop. Closing it never stops admitted work.
    Local trusted listeners may use HTTP; remote listeners require HTTPS and a
    bearer token. RPC errors are not automatically retried, especially mutations.
    """

    def __init__(
        self,
        endpoint: str = "http://127.0.0.1:8765",
        *,
        key_file: str | Path | None = None,
        env_file: str | Path | None = None,
        ca_certificate: str | Path | None = None,
        rpc_timeout: float = 30,
        expected_backend_id: str | None = None,
        ca_certificate_pem: str | None = None,
    ) -> None:
        token = load_key(key_file=key_file, env_file=env_file)
        self._transport = Transport(
            endpoint,
            token,
            ca_certificate,
            rpc_timeout,
            expected_backend_id=expected_backend_id,
            ca_certificate_pem=ca_certificate_pem,
        )

    @classmethod
    def from_profile(
        cls,
        name: str,
        *,
        path: str | Path | None = None,
        key_file: str | Path | None = None,
        env_file: str | Path | None = None,
        rpc_timeout: float = 30,
    ) -> Self:
        """Read a named profile once; callers provide secrets separately."""
        from .profiles import load

        profile = load(name, path=path)
        client = cls(
            profile.endpoint,
            key_file=key_file,
            env_file=env_file,
            rpc_timeout=rpc_timeout,
            expected_backend_id=profile.expected_backend_id,
            ca_certificate_pem=profile.ca_certificate_pem,
        )
        if profile.credential == "keyring" and not client._transport._metadata:
            raise ValueError("This profile requires --key-file or DOKIME_KEY supplied by the caller.")
        return client

    async def connect(self) -> Backend:
        """Negotiate API identity and the operations available to the caller (also done lazily)."""
        return await self._transport.connect()

    async def operations(self) -> tuple[DiscoveredOperation, ...]:
        """Discover Authorized Operations, Including Separately Installed Plugins"""
        return await self._transport.operations()

    async def show(self, identifier: str, *, owner: str | None = None) -> DiscoveredOperation:
        """Resolve One Live Provider And Its Canonical Contract"""
        return await self._transport.show(identifier, owner=owner)

    async def call(
        self,
        operation: Operation | str,
        request: Message | Mapping[str, Any],
        *,
        owner: str | None = None,
    ) -> Message:
        """Invoke A Unary Operation After Checking Its Live Contract"""
        return await self._transport.call(operation, request, owner=owner)

    async def upload(self, operation: Operation, request: Message, items: AsyncIterator[Message]) -> Message:
        """Send A Client Stream With A Canonical Request And Final Response"""
        return await self._transport.upload(operation, request, items)

    def download(self, operation: Operation, request: Message) -> ResponseStream:
        """Receive A Server Stream; Read Its Final Response With response()"""
        return self._transport.download(operation, request)

    def exchange(self, operation: Operation, request: Message, items: AsyncIterator[Message]) -> ResponseStream:
        """Exchange Canonical Items And Retain The Final Response"""
        return self._transport.exchange(operation, request, items)

    async def close(self) -> None:
        """Close local subscriptions and connections; safe to repeat."""
        await self._transport.close()

    async def __aenter__(self) -> Self:
        await self.connect()
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        await self.close()

    async def suites(self) -> AsyncIterator[Suite]:
        """List suites by ID. A concurrent catalog revision can abort paging."""
        async for identity in self._suite_identities():
            item = await self._suite_by_id(identity.suite_id)
            if not _wire.equal(item.identity, identity):
                raise ProtocolError("suite definition differs from listed identity")
            yield _suite(item)

    async def _suite_identities(self):
        backend = await self.connect()
        request = _wire.request("Catalog", "ListSuites", limit=100)
        while True:
            await self._transport.verify(backend, live=True)
            page = await self._transport.unary("Catalog", "ListSuites", request)
            _cursor(backend, page, "cursor")
            if request.HasField("cursor") and not _wire.equal(request.cursor, page.cursor):
                raise ProtocolError("suite page changed revision")
            if len(page.suites) > request.limit:
                raise ProtocolError("suite page exceeds its limit")
            previous = request.after_suite_id
            for identity in page.suites:
                if not _readable(identity.suite_id):
                    raise ProtocolError("suite identity is invalid")
                if identity.suite_id <= previous:
                    raise ProtocolError("suite identities do not advance")
                previous = identity.suite_id
            if page.HasField("next_after_suite_id") and (not page.suites or page.next_after_suite_id != previous):
                raise ProtocolError("invalid suite continuation")
            for identity in page.suites:
                yield identity
            if not page.HasField("next_after_suite_id"):
                return
            request.after_suite_id = previous
            _wire.copy(request.cursor, page.cursor)

    async def _suite_by_id(self, identifier: str) -> Message:
        backend = await self.connect()
        await self._transport.verify(backend, live=True)
        item = await self._transport.unary(
            "Catalog",
            "GetSuite",
            _wire.request("Catalog", "GetSuite", suite_id=identifier),
        )
        if not item.HasField("identity") or item.identity.suite_id != identifier or not identifier.strip():
            raise ProtocolError("suite definition identity differs")
        return item

    async def _suite_definition(self, identifier: str) -> Message:
        try:
            return await self._suite_by_id(identifier)
        except RpcError as error:
            if error.code != "NOT_FOUND":
                raise
        matches = [item.suite_id async for item in self._suite_identities() if item.metadata.identifier == identifier]
        if len(matches) > 1:
            raise ValueError("suite identifier is ambiguous")
        if not matches:
            raise RpcError("NOT_FOUND", "suite not found")
        item = await self._suite_by_id(matches[0])
        if item.identity.metadata.identifier != identifier:
            raise ProtocolError("resolved suite identifier changed")
        return item

    async def suite(self, suite_id: str) -> Suite:
        """Resolve a suite by its ID, as the catalog lists it."""
        return _suite(await self._suite_definition(suite_id))

    async def create_cycle(
        self,
        *,
        label: str,
        description: str | None = None,
        identifier: str | None = None,
        continue_on_failure: bool = False,
    ) -> str:
        """Create an empty Ready cycle and return its ID.

        Cases join it only through ``append_cases``. This RPC has no
        idempotency key: an ambiguous failure must not be blindly retried. An
        omitted identifier is derived by the backend.
        """
        if not label.strip():
            raise ValueError("cycle label is required")
        backend = await self.connect()
        metadata: dict[str, str] = {"label": label}
        if description is not None:
            metadata["description"] = description
        if identifier is not None:
            metadata["identifier"] = identifier
        await self._transport.verify(backend, live=True)
        created = await self._transport.unary(
            "Execution",
            "CreateCycle",
            _wire.request(
                "Execution",
                "CreateCycle",
                metadata=metadata,
                continue_on_failure=continue_on_failure,
            ),
        )
        _cursor(backend, created)
        try:
            _identity(created.cycle_id)
        except ValueError as error:
            raise ProtocolError("created cycle identity is invalid") from error
        return created.cycle_id

    async def append_cases(
        self,
        cycle_id: str,
        *,
        suite: str,
        template: str,
        parameters: Mapping[str, Value] | None = None,
    ) -> None:
        """Append the cases a suite template generates to an existing cycle.

        Suite and template are named by their IDs; omitted parameters use the
        registered defaults. Case labels continue the cycle's numbering.
        """
        _identity(cycle_id)
        plan = await self._selection(suite, template, parameters)
        await self._append(cycle_id, plan)

    async def new_cycle(
        self,
        *,
        suite: str,
        template: str,
        parameters: Mapping[str, Value] | None = None,
        label: str | None = None,
    ) -> Cycle:
        """Create a cycle and append one template's cases without running them.

        Composes ``create_cycle`` and ``append_cases`` after validating the
        selection; not idempotent. An omitted label names the cycle after the
        moment it was created.
        """
        plan = await self._selection(suite, template, parameters)
        identifier = await self.create_cycle(label=label if label is not None else _default_cycle_label())
        await self._append(identifier, plan)
        return await self.cycle(identifier)

    async def _selection(
        self,
        suite: str,
        template: str,
        parameters: Mapping[str, Value] | None,
    ) -> dict[str, Any]:
        definition = await self._suite_definition(suite)
        # A template is named by its readable ID; the registered UUID stays on
        # the wire between this client and the backend.
        matches = [item for item in definition.templates if item.metadata.identifier == template]
        if len(matches) != 1:
            raise ValueError("template must identify exactly one registered template")
        selected = matches[0]
        _identity(selected.template_id)
        return {
            "selections": [
                {
                    "suite_id": definition.identity.suite_id,
                    "template_id": selected.template_id,
                    "parameters": [
                        {"key": name, "value": {"values": [_wire.encode(item)]}} for name, item in sorted((parameters or {}).items())
                    ],
                }
            ],
        }

    async def _append(self, cycle_id: str, plan: dict[str, Any]) -> None:
        backend = await self.connect()
        await self._transport.verify(backend, live=True)
        appended = await self._transport.unary(
            "Execution",
            "AppendPlannedCases",
            _wire.request(
                "Execution",
                "AppendPlannedCases",
                cycle_id=cycle_id,
                plan={**plan, "cycle_id": cycle_id},
            ),
        )
        _cursor(backend, appended)

    async def cycle(self, cycle_id: str) -> Cycle:
        """Resolve a live cycle on the current backend."""
        return await Cycle._bind(self._transport, cycle_id, await self.connect())

    async def get_run_request(self, request_id: str) -> RunRequest:
        """Recover original native published progress, without owning a cycle."""
        return await RunRequest.resolve(self._transport, await self.connect(), request_id)

    async def execution(self, execution_id: str) -> TestExecutionHandle:
        """Bind an existing native identity without lookup or device work."""
        backend = await self.connect()
        return TestExecutionHandle(execution_id, self._transport, backend)


# === Public Functions === #


# === Private Functions === #


def _default_cycle_label() -> str:
    # Named after the local moment it is created, as the desktop names it.
    return f"Test Cycle - {datetime.now().astimezone():%Y-%m-%d %H:%M}"


def _readable(value: str) -> bool:
    # A suite ID is a readable identifier: ASCII letters, digits, `.`, `_` and
    # `-`, starting with a letter, digit or underscore.
    return (
        value.isascii()
        and bool(value)
        and (value[0].isalnum() or value[0] == "_")
        and all(character.isalnum() or character in "._-" for character in value)
    )


def _suite(item: Message) -> Suite:
    return Suite(
        item.identity.suite_id,
        item.identity.metadata.identifier,
        item.identity.metadata.label,
        tuple(
            Template(
                t.metadata.identifier,
                t.metadata.label,
                tuple(p.parameter_id for p in t.parameters),
            )
            for t in item.templates
        ),
    )


# === Entry Point === #
