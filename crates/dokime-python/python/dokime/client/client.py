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

# Third Party Imports

# Local Imports
from . import _wire
from ._generated.dokime.v1 import catalog_pb2 as catalog
from ._generated.dokime.v1 import value_pb2 as value
from dokime._native.client import ClientError
from ._transport import Transport, _error
from .models import Backend, Suite, Template, Value
from .execution import Cycle, RunRequest, TestExecutionHandle

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
    ) -> None:
        self._transport = Transport(endpoint, token, ca_certificate, rpc_timeout)

    async def connect(self) -> Backend:
        """Negotiate API identity and caller capabilities (also done lazily)."""
        return await self._transport.connect()

    async def close(self) -> None:
        """Close local subscriptions and connections; safe to repeat."""
        await self._transport.close()

    async def __aenter__(self) -> "DokimeClient":
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
        reader = await self._transport.suites()
        try:
            try:
                while (payload := await reader.message()) is not None:
                    yield _suite(catalog.SuiteDefinition.FromString(payload))
            finally:
                await reader.close()
        except ClientError as error:
            raise _error(error) from error

    async def suite(self, suite_id: str) -> Suite:
        """Resolve a registered UUID or a unique authored suite identifier."""
        return _suite(await self._transport.suite(suite_id))

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
        inputs = value.ValueObject(
            fields={key: _wire.encode(item) for key, item in (parameters or {}).items()}
        )
        created = await self._transport.plan_cycle(suite, template, inputs, label=label)
        return created.id

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
        native = await self._transport.execution(execution_id)
        return TestExecutionHandle(native, self._transport, backend)


# === Public Functions === #

# === Private Functions === #


def _suite(item: catalog.SuiteDefinition) -> Suite:
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
