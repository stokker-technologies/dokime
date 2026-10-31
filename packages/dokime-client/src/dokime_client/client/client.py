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
import asyncio
import json
from collections.abc import AsyncIterator, Mapping
from contextlib import asynccontextmanager
from importlib.resources import files
from pathlib import Path
from types import TracebackType
from typing import Any, Self

# Third Party Imports
from google.protobuf.message import Message

# Local Imports
from .._profiles import Profile
from ..operation import Operation, ResponseStream
from . import _wire
from ._transport import DiscoveredOperation, Transport, _cursor
from .errors import ProtocolError, RpcError
from .execution import Cycle, Execution
from .execution.request import _identity
from .models import Backend, Suite
from .models import Template as TemplateSummary
from .planning import Template
from .query import discovery
from .query.discovery import Alternatives, CycleSummary, ExecutionSummary
from .query.reader import QueryReader, QuerySettings
from .query.selection import At, Time, Where, Window

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PAGE_ITEMS = 100

# === Types === #

# === Classes === #


class DokimeClient:
    """One async connection. Read explicit local inputs before any connection.

    A client is used on one asyncio loop. Closing it never stops admitted work.
    HTTPS verifies TLS; explicit HTTP selects plaintext for an isolated lab.
    The listener decides whether a key is required, independently of TLS.
    RPC errors are not automatically retried, especially mutations.
    """

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
    ) -> None:
        preferences = preferences if preferences is not None else _client_preferences()
        self._profile: tuple[Profile, Path] | None = None
        self._query_settings = query_settings if query_settings is not None else QuerySettings()
        self._transport = Transport(
            endpoint,
            access_key,
            preferences.request_timeout_ms / 1000 if rpc_timeout is None else rpc_timeout,
            connect_timeout_ms=preferences.connect_timeout_ms,
            endpoint_stagger_ms=preferences.endpoint_stagger_ms,
            endpoints=endpoints,
            expected_backend_id=expected_backend_id,
            ca_certificate_pem=ca_certificate_pem,
        )

    @classmethod
    def from_pairing(
        cls,
        path: str | Path,
        *,
        preferences: Message | None = None,
        rpc_timeout: float | None = None,
        query_settings: QuerySettings | None = None,
    ) -> Self:
        """Read A Pairing Once; Verify Its Backend Identity On Connection"""
        from .pairing import read

        pairing = read(path)
        return cls(
            pairing["endpoints"][0],
            preferences=preferences,
            query_settings=query_settings,
            rpc_timeout=rpc_timeout,
            endpoints=tuple(pairing["endpoints"]),
            access_key=pairing["access_key"],
            expected_backend_id=pairing["backend_identifier"],
            ca_certificate_pem=pairing["ca_certificate_pem"],
        )

    @classmethod
    def from_profile(
        cls,
        identifier: str,
        *,
        path: str | Path | None = None,
        rpc_timeout: float | None = None,
        preferences: Message | None = None,
        query_settings: QuerySettings | None = None,
    ) -> Self:
        """Read An Unsecured Profile Without Native Keyring Access"""
        from .profiles import configuration_path, load

        preferences = preferences if preferences is not None else _client_preferences()
        profile = load(identifier, path=path)
        client = cls(
            profile.endpoint,
            endpoints=profile.endpoints,
            rpc_timeout=rpc_timeout,
            preferences=preferences,
            query_settings=query_settings,
            expected_backend_id=profile.expected_backend_id,
            ca_certificate_pem=profile.ca_certificate_pem,
        )
        if profile.secret == "keyring" and not client._transport._metadata:
            raise ValueError("This profile uses the native keyring; import its pairing with DokimeClient.from_pairing instead.")
        client._profile = (profile, configuration_path(path))
        return client

    def executions(
        self,
        *,
        template: Alternatives = None,
        cycle: Alternatives = None,
        status: Alternatives = None,
        since: Time | None = None,
        until: Time | None = None,
        archive: str = "unarchived",
        where: Where | None = None,
        between: Window | None = None,
        at: At = None,
        page_rows: int | None = None,
        page_bytes: int | None = None,
    ) -> QueryReader[ExecutionSummary]:
        """Discover Captured Execution Summaries At One Finite Local Snapshot."""
        return discovery.executions(
            self._transport,
            self._query_settings,
            template=template,
            cycle=cycle,
            status=status,
            since=since,
            until=until,
            archive=archive,
            where=where,
            between=between,
            at=at,
            page_rows=page_rows,
            page_bytes=page_bytes,
        )

    def cycles(
        self,
        *,
        cycle: Alternatives = None,
        phase: Alternatives = None,
        since: Time | None = None,
        until: Time | None = None,
        archive: str = "unarchived",
        where: Where | None = None,
        between: Window | None = None,
        at: At = None,
        page_rows: int | None = None,
        page_bytes: int | None = None,
    ) -> QueryReader[CycleSummary]:
        """Discover Captured Cycle Summaries Without Opening Owned Lifecycle Contexts."""
        return discovery.cycles(
            self._transport,
            self._query_settings,
            cycle=cycle,
            phase=phase,
            since=since,
            until=until,
            archive=archive,
            where=where,
            between=between,
            at=at,
            page_rows=page_rows,
            page_bytes=page_bytes,
        )

    async def connect(self) -> Backend:
        """Negotiate API identity and the operations available to the caller (also done lazily)."""
        backend = await self._transport.connect()
        if self._profile is not None:
            from .profiles import remember_endpoint

            try:
                async with self._transport._lock:
                    if self._profile is not None:
                        profile, path = self._profile
                        await asyncio.to_thread(remember_endpoint, profile, path, self._transport._endpoint)
                        self._profile = None
            except (OSError, ValueError):
                await self._transport.close()
                raise
        return backend

    async def operations(self) -> tuple[DiscoveredOperation, ...]:
        """Discover Authorized Providers With gRPC Declarations, Including Installed Plugins"""
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
        request = _wire.request("Catalog", "ListSuites", limit=PAGE_ITEMS)
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

    async def template(self, identifier: str) -> Template:
        """Resolve One Readable Template Identifier Across The Registered Catalog."""
        backend = await self.connect()
        matches = []
        async for suite in self._suite_identities():
            definition = await self._suite_by_id(suite.suite_id)
            for item in definition.templates:
                if identifier == item.metadata.identifier:
                    matches.append((suite.suite_id, item))
        if len(matches) > 1:
            raise ValueError("template identifier is ambiguous")
        if not matches:
            raise RpcError("NOT_FOUND", "template not found")
        suite_id, item = matches[0]
        _identity(item.template_id)
        return Template(item.metadata.identifier, item.metadata.label, suite_id, item.template_id, self._transport, backend)

    async def create_cycle(
        self,
        *,
        label: str,
        description: str | None = None,
        identifier: str | None = None,
        continue_automatically: bool = False,
    ) -> Cycle:
        """Create An Empty Active Cycle And Return Its Bound Handle.

        Cases join it only through ``Cycle.append``. This RPC has no
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
                continue_automatically=continue_automatically,
            ),
        )
        _cursor(backend, created)
        try:
            _identity(created.cycle_id)
        except ValueError as error:
            raise ProtocolError("created cycle identity is invalid") from error
        return Cycle(created.cycle_id, self._transport, backend, (), client=self)

    @asynccontextmanager
    async def new_cycle(
        self,
        label: str,
        *,
        description: str | None = None,
        identifier: str | None = None,
        continue_automatically: bool = False,
    ) -> AsyncIterator[Cycle]:
        """Create An Owned Cycle That Seals And Drains On Every Context Exit."""
        cycle = await self.create_cycle(
            label=label,
            description=description,
            identifier=identifier,
            continue_automatically=continue_automatically,
        )
        async with cycle:
            yield cycle

    async def cycle(self, cycle_id: str) -> Cycle:
        """Resolve a live cycle on the current backend."""
        return await Cycle._bind(self._transport, cycle_id, await self.connect(), client=self)

    async def execution(self, execution_id: str) -> Execution:
        """Bind an existing native identity without lookup or device work."""
        backend = await self.connect()
        return Execution._bind(execution_id, self._transport, backend)


# === Public Functions === #

# === Private Functions === #


def _client_preferences() -> Message:
    resource = files("dokime_client").joinpath("application-settings.json")
    defaults = json.loads(resource.read_text(encoding="utf-8"))
    request = _wire.request("Settings", "UpdateApplicationSettings")
    request.update.settings.client.connect_timeout_ms = defaults["client"]["connect_timeout_ms"]
    request.update.settings.client.endpoint_stagger_ms = defaults["client"]["endpoint_stagger_ms"]
    request.update.settings.client.request_timeout_ms = defaults["client"]["request_timeout_ms"]
    return request.update.settings.client


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
            TemplateSummary(
                t.metadata.identifier,
                t.metadata.label,
                tuple(p.parameter_id for p in t.parameters),
            )
            for t in item.templates
        ),
    )


# === Entry Point === #
