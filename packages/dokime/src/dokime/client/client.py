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
from pathlib import Path
from types import TracebackType
from typing import Any, Self
from uuid import uuid4

# Third Party Imports
from google.protobuf.message import Message

from ..operation import Operation, ResponseStream

# Local Imports
from . import _wire
from ._transport import DiscoveredOperation, Transport, _cursor
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
    """One async connection. Construct without IO; close explicitly when done.

    A client is used on one asyncio loop. Closing it never stops admitted work.
    Local trusted listeners may use HTTP; remote listeners require HTTPS and a
    bearer token. RPC errors are not automatically retried, especially mutations.
    """

    def __init__(
        self,
        endpoint: str = "http://127.0.0.1:8765",
        *,
        token: str | None = None,
        ca_certificate: str | Path | None = None,
        rpc_timeout: float = 30,
        expected_backend_id: str | None = None,
        ca_certificate_pem: str | None = None,
    ) -> None:
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
        token: str | None = None,
        rpc_timeout: float = 30,
    ) -> Self:
        """Read a named profile once; callers provide secrets separately."""
        from .profiles import load

        profile = load(name, path=path)
        if profile.credential == "keyring" and token is None:
            raise ValueError(
                "this profile requires a bearer token supplied by the caller"
            )
        return cls(
            profile.endpoint,
            token=token,
            rpc_timeout=rpc_timeout,
            expected_backend_id=profile.expected_backend_id,
            ca_certificate_pem=profile.ca_certificate_pem,
        )

    async def connect(self) -> Backend:
        """Negotiate API identity and caller capabilities (also done lazily)."""
        return await self._transport.connect()

    async def operations(self) -> tuple[DiscoveredOperation, ...]:
        """Discover Authorized Operations, Including Separately Installed Plugins"""
        return await self._transport.operations()

    async def show(
        self, identifier: str, *, owner: str | None = None
    ) -> DiscoveredOperation:
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

    async def upload(
        self, operation: Operation, request: Message, items: AsyncIterator[Message]
    ) -> Message:
        """Send A Client Stream With A Canonical Request And Final Response"""
        return await self._transport.upload(operation, request, items)

    def download(self, operation: Operation, request: Message) -> ResponseStream:
        """Receive A Server Stream; Read Its Final Response With response()"""
        return self._transport.download(operation, request)

    def exchange(
        self, operation: Operation, request: Message, items: AsyncIterator[Message]
    ) -> ResponseStream:
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
        """List registered IDs. A concurrent catalog revision can abort paging."""
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
            if request.HasField("cursor") and not _wire.equal(
                request.cursor, page.cursor
            ):
                raise ProtocolError("suite page changed revision")
            if len(page.suites) > request.limit:
                raise ProtocolError("suite page exceeds its limit")
            previous = request.after_suite_id
            for identity in page.suites:
                try:
                    _identity(identity.suite_id)
                except ValueError as error:
                    raise ProtocolError("suite identity is invalid") from error
                if identity.suite_id <= previous:
                    raise ProtocolError("suite identities do not advance")
                previous = identity.suite_id
            if page.HasField("next_after_suite_id") and (
                not page.suites or page.next_after_suite_id != previous
            ):
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
        if (
            not item.HasField("identity")
            or item.identity.suite_id != identifier
            or not identifier.strip()
        ):
            raise ProtocolError("suite definition identity differs")
        return item

    async def _suite_definition(self, identifier: str) -> Message:
        try:
            return await self._suite_by_id(identifier)
        except RpcError as error:
            if error.code != "NOT_FOUND":
                raise
        matches = [
            item.suite_id
            async for item in self._suite_identities()
            if item.metadata.identifier == identifier
        ]
        if len(matches) > 1:
            raise ValueError("suite identifier is ambiguous; use its registered ID")
        if not matches:
            raise RpcError("NOT_FOUND", "suite not found")
        item = await self._suite_by_id(matches[0])
        if item.identity.metadata.identifier != identifier:
            raise ProtocolError("resolved suite identifier changed")
        return item

    async def suite(self, suite_id: str) -> Suite:
        """Resolve a registered UUID or a unique authored suite identifier."""
        return _suite(await self._suite_definition(suite_id))

    async def create_cycle(
        self,
        *,
        suite: str,
        template: str,
        parameters: Mapping[str, Value] | None = None,
        label: str | None = None,
    ) -> str:
        """Create one planned case using registered IDs or authored identifiers.

        This RPC has no idempotency key. An ambiguous failure must not be
        blindly retried. Omitted parameters use the registered defaults.
        """
        backend = await self.connect()
        definition = await self._suite_definition(suite)
        matches = [
            item
            for item in definition.templates
            if item.template_id == template or item.metadata.identifier == template
        ]
        if len(matches) != 1:
            raise ValueError("template must identify exactly one registered template")
        selected = matches[0]
        _identity(selected.template_id)
        key = str(uuid4())
        plan = {
            "max_cases": 1,
            "selections": [
                {
                    "suite_id": definition.identity.suite_id,
                    "template_id": selected.template_id,
                    "parameters": [
                        {"key": name, "value": {"values": [_wire.encode(item)]}}
                        for name, item in sorted((parameters or {}).items())
                    ],
                }
            ],
        }
        await self._transport.verify(backend, live=True)
        registered = await self._transport.unary(
            "Planning",
            "RegisterSelection",
            _wire.request(
                "Planning", "RegisterSelection", plan=plan, idempotency_key=key
            ),
        )
        _cursor(backend, registered)
        if len(registered.cases) != 1:
            raise ProtocolError("single-case registration returned another case count")
        case = registered.cases[0]
        if (
            case.suite_id != definition.identity.suite_id
            or case.template_id != selected.template_id
            or case.ordinal != 0
            or case.selection_index != 0
            or not case.case_id
        ):
            raise ProtocolError("registered case differs from selected template")
        await self._transport.verify(backend, live=True)
        created = await self._transport.unary(
            "Execution",
            "CreateCycle",
            _wire.request(
                "Execution",
                "CreateCycle",
                plan={"case_ids": [case.case_id]},
                metadata={
                    "identifier": f"client.{key}",
                    "label": label if label is not None else selected.metadata.label,
                },
            ),
        )
        _cursor(backend, created)
        try:
            _identity(created.cycle_id)
        except ValueError as error:
            raise ProtocolError("created cycle identity is invalid") from error
        return created.cycle_id

    async def new_cycle(
        self,
        *,
        suite: str,
        template: str,
        parameters: Mapping[str, Value] | None = None,
        label: str | None = None,
    ) -> Cycle:
        """Create a fresh cycle without submitting device work. Not idempotent."""
        identifier = await self.create_cycle(
            suite=suite, template=template, parameters=parameters, label=label
        )
        return await self.cycle(identifier)

    async def cycle(self, cycle_id: str) -> Cycle:
        """Resolve a live cycle on the current backend."""
        return await Cycle._bind(self._transport, cycle_id, await self.connect())

    async def get_run_request(self, request_id: str) -> RunRequest:
        """Recover original native published progress, without owning a cycle."""
        return await RunRequest.resolve(
            self._transport, await self.connect(), request_id
        )

    async def execution(self, execution_id: str) -> TestExecutionHandle:
        """Bind an existing native identity without lookup or device work."""
        backend = await self.connect()
        return TestExecutionHandle(execution_id, self._transport, backend)


# === Public Functions === #

# === Private Functions === #


def _suite(item: Message) -> Suite:
    return Suite(
        item.identity.suite_id,
        item.identity.metadata.identifier,
        item.identity.metadata.label,
        tuple(
            Template(
                t.template_id,
                t.metadata.identifier,
                t.metadata.label,
                tuple(p.parameter_id for p in t.parameters),
            )
            for t in item.templates
        ),
    )


# === Entry Point === #
